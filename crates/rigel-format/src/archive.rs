//! The container, treated as hostile.
//!
//! A puppet is a zip archive (`docs/format.md` — *The container is an archive
//! with a manifest*), and an archive brings faults a plain JSON file could not
//! have: two entries under one name, a central-directory offset landing inside
//! another entry's data, a name longer than anything the format defines, and an
//! entry that declares a kilobyte and expands to a quarter of a gigabyte.
//!
//! # What is checked, and in what order
//!
//! Everything in [`open`] is read from the **central directory alone** and
//! decompresses nothing. That is what makes it cheap enough to run before the
//! manifest is read, and it is also forced: the manifest cannot be found without
//! walking the directory that names it.
//!
//! [`Container::read`] is where a byte is first produced, and it is where the
//! payload limits bite. **A zip entry's declared decompressed size is
//! attacker-controlled**, so the declaration is used only to refuse early and
//! the limit is enforced against bytes actually produced (`LIM-3`,
//! `docs/format.md` — *Load limits*). A loader that trusts the header has a
//! decompression bomb.
//!
//! Every entry's CRC is checked against the one the directory records, so a
//! single flipped byte is refused here rather than reaching the parser
//! (`EXEC-5`).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::limits::LimitKind;
use crate::load::{ArchiveFault, Rejection};

/// The entry carrying the format version, and nothing a reader must interpret.
pub(crate) const MANIFEST: &str = "rigel2d.json";
/// The entry carrying the whole structure.
pub(crate) const STRUCTURE: &str = "puppet.json";
/// Where a texture payload's name begins.
const TEXTURE_PREFIX: &str = "textures/";
/// And what it ends with.
const TEXTURE_SUFFIX: &str = ".png";

/// What an entry is, by the name the format gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Manifest,
    Structure,
    /// A texture payload, addressable by the identifier of the texture it
    /// belongs to ([I3](../../../docs/invariants.md)).
    Texture(u64),
}

/// One entry of the central directory, before any of it is trusted.
#[derive(Clone, Debug)]
pub(crate) struct Entry {
    pub index: usize,
    pub role: Role,
    /// What the entry *says* it decompresses to. Attacker-controlled, and used
    /// only to refuse early.
    pub declared_size: u64,
}

/// An archive whose container has already been refused or accepted whole.
pub(crate) struct Container {
    archive: zip::ZipArchive<File>,
    entries: Vec<Entry>,
    /// Bytes decompressed so far, against [`LimitKind::TotalLoadMemory`].
    spent: u64,
}

/// The figure for a limit, or a refusal.
///
/// An absent figure is a **defect**, not *no limit*: a size the loader trusts
/// with nothing bounding it is the evasion load limits exist to close (`LIM-1`,
/// `LIM-8`).
pub(crate) fn required_limit(kind: LimitKind) -> Result<u64, Rejection> {
    crate::limits::declared_limit(kind).ok_or(Rejection::LimitExceeded {
        limit: kind,
        declared: 0,
        detail: String::from(
            "no figure is declared for this limit, and an undeclared limit is a \
             defect rather than permission",
        ),
    })
}

/// Refuses a count or size against its declared limit.
pub(crate) fn within(kind: LimitKind, found: u64, what: &str) -> Result<(), Rejection> {
    let limit = required_limit(kind)?;
    if found > limit {
        return Err(Rejection::LimitExceeded {
            limit: kind,
            declared: limit,
            detail: format!("{what}: {found} exceeds {limit}"),
        });
    }
    Ok(())
}

fn fault(fault: ArchiveFault, detail: impl Into<String>) -> Rejection {
    Rejection::Archive {
        fault,
        detail: detail.into(),
    }
}

/// Opens an archive and refuses every container fault, decompressing nothing.
pub(crate) fn open(path: &Path) -> Result<Container, Rejection> {
    let file = File::open(path)
        .map_err(|e| fault(ArchiveFault::NotAnArchive, format!("not opened: {e}")))?;
    let archive = zip::ZipArchive::new(file).map_err(|e| {
        fault(
            ArchiveFault::NotAnArchive,
            format!("not a zip archive: {e}"),
        )
    })?;

    // The archive's own size, before any name is looked at.
    within(
        LimitKind::ArchiveEntries,
        archive.len() as u64,
        "entries in the archive",
    )?;

    // Name lengths first: reading a name is the cheapest thing here, and an
    // over-long one is refused before it is asked what it means.
    let name_limit = required_limit(LimitKind::EntryNameBytes)?;
    for raw in archive.file_names_raw() {
        if raw.len() as u64 > name_limit {
            return Err(fault(
                ArchiveFault::EntryNameTooLong,
                format!("an entry name of {} bytes exceeds {name_limit}", raw.len()),
            ));
        }
    }

    // Two entries under one name make what the name holds undecidable.
    if archive.has_duplicate_names() {
        return Err(fault(
            ArchiveFault::DuplicateEntryName,
            "two entries share one name",
        ));
    }

    // Every name is one the format's table defines.
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive
            .by_index_data(index)
            .map_err(|e| fault(ArchiveFault::NotAnArchive, format!("entry {index}: {e}")))?;
        let raw = entry.name_raw().to_vec();
        let name = std::str::from_utf8(&raw).map_err(|_| {
            fault(
                ArchiveFault::UndefinedEntryName,
                "an entry name that is not UTF-8",
            )
        })?;
        let Some(role) = role_of(name) else {
            return Err(fault(
                ArchiveFault::UndefinedEntryName,
                format!("{name:?} is not an entry the format defines"),
            ));
        };
        match entry.compression() {
            zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated => {}
            other => {
                return Err(fault(
                    ArchiveFault::NotAnArchive,
                    format!("{name:?} uses {other}, which the format does not define"),
                ));
            }
        }
        entries.push(Entry {
            index,
            role,
            declared_size: entry.size(),
        });
    }

    refuse_overlap(&archive)?;

    for (role, name) in [(Role::Manifest, MANIFEST), (Role::Structure, STRUCTURE)] {
        if !entries.iter().any(|entry| entry.role == role) {
            return Err(fault(
                ArchiveFault::MissingRequiredEntry,
                format!("the archive carries no {name}"),
            ));
        }
    }

    Ok(Container {
        archive,
        entries,
        spent: 0,
    })
}

/// The entry a name denotes, or `None` for a name the format does not define.
fn role_of(name: &str) -> Option<Role> {
    if name == MANIFEST {
        return Some(Role::Manifest);
    }
    if name == STRUCTURE {
        return Some(Role::Structure);
    }
    let digits = name
        .strip_prefix(TEXTURE_PREFIX)
        .and_then(|rest| rest.strip_suffix(TEXTURE_SUFFIX))?;
    // An identifier is a decimal integer and is spelled one way, so that one
    // texture cannot be addressed by two names.
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    Some(Role::Texture(digits.parse::<u64>().ok()?))
}

/// Refuses an archive whose entries' regions overlap.
///
/// Computed from the central directory alone, and **conservatively**: the local
/// header's extra field is not recorded there, so a region's end is taken as the
/// smallest it could be. An overlap found this way is therefore always real.
fn refuse_overlap(archive: &zip::ZipArchive<File>) -> Result<(), Rejection> {
    /// A local file header is a fixed 30 bytes, then the name, then the extra
    /// field (PKWARE APPNOTE, 4.3.7).
    const LOCAL_HEADER_FIXED: u64 = 30;

    let mut regions: Vec<(u64, u64, String)> = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive
            .by_index_data(index)
            .map_err(|e| fault(ArchiveFault::NotAnArchive, format!("entry {index}: {e}")))?;
        let start = entry.header_start();
        let end = start
            .saturating_add(LOCAL_HEADER_FIXED)
            .saturating_add(entry.name_raw().len() as u64)
            .saturating_add(entry.compressed_size());
        regions.push((start, end, entry.name_raw_lossy()));
    }
    regions.sort_by_key(|(start, _, _)| *start);
    for pair in regions.windows(2) {
        let (_, first_end, first_name) = &pair[0];
        let (second_start, _, second_name) = &pair[1];
        if second_start < first_end {
            return Err(fault(
                ArchiveFault::OverlappingEntryData,
                format!("{second_name} begins inside {first_name}"),
            ));
        }
    }
    Ok(())
}

/// A name for a message, with no promise that it was ever valid UTF-8.
trait NameLossy {
    fn name_raw_lossy(&self) -> String;
}

impl NameLossy for zip::read::ZipFileEntry<'_> {
    fn name_raw_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_raw()).into_owned()
    }
}

impl Container {
    /// Every entry, in the order the central directory carries them.
    pub(crate) fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The entry with this role, if the archive carries one.
    pub(crate) fn find(&self, role: Role) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.role == role)
    }

    /// One entry's bytes, under the payload limits.
    ///
    /// The declared size refuses early and decides nothing else: what is
    /// enforced is the count of bytes this call actually produces, so an entry
    /// that declares a kilobyte and expands to a quarter of a gigabyte is
    /// refused at the limit rather than at its own word (`LIM-3`).
    pub(crate) fn read(&mut self, entry: &Entry) -> Result<Vec<u8>, Rejection> {
        let limit = required_limit(LimitKind::DecompressedPayloadBytes)?;
        let total = required_limit(LimitKind::TotalLoadMemory)?;

        // The declaration is good for one thing: refusing without decompressing.
        if entry.declared_size > limit {
            return Err(Rejection::LimitExceeded {
                limit: LimitKind::DecompressedPayloadBytes,
                declared: limit,
                detail: format!(
                    "an entry declaring {} bytes exceeds {limit}",
                    entry.declared_size
                ),
            });
        }

        let (method, expected_crc) = {
            let data = self
                .archive
                .by_index_data(entry.index)
                .map_err(|e| fault(ArchiveFault::NotAnArchive, format!("{e}")))?;
            (data.compression(), data.crc32())
        };

        // The raw stream, so that what is produced is counted here rather than
        // cut short at whatever the entry declared.
        let raw = self
            .archive
            .by_index_raw(entry.index)
            .map_err(|e| fault(ArchiveFault::NotAnArchive, format!("{e}")))?;

        let mut produced = Vec::new();
        let read = match method {
            zip::CompressionMethod::Stored => raw.take(limit + 1).read_to_end(&mut produced),
            zip::CompressionMethod::Deflated => flate2::read::DeflateDecoder::new(raw)
                .take(limit + 1)
                .read_to_end(&mut produced),
            other => {
                return Err(fault(
                    ArchiveFault::NotAnArchive,
                    format!("{other} is not a compression method the format defines"),
                ));
            }
        };

        if produced.len() as u64 > limit {
            return Err(Rejection::LimitExceeded {
                limit: LimitKind::DecompressedPayloadBytes,
                declared: limit,
                detail: format!(
                    "an entry declaring {} bytes produced more than {limit}",
                    entry.declared_size
                ),
            });
        }
        read.map_err(|e| fault(ArchiveFault::NotAnArchive, format!("entry unreadable: {e}")))?;

        let mut crc = flate2::Crc::new();
        crc.update(&produced);
        if crc.sum() != expected_crc {
            return Err(fault(
                ArchiveFault::NotAnArchive,
                "an entry's contents do not match the checksum the archive records",
            ));
        }

        self.spent = self.spent.saturating_add(produced.len() as u64);
        if self.spent > total {
            return Err(Rejection::LimitExceeded {
                limit: LimitKind::TotalLoadMemory,
                declared: total,
                detail: format!("the load has produced {} bytes", self.spent),
            });
        }

        Ok(produced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_the_format_defines_and_no_others() {
        assert_eq!(role_of("rigel2d.json"), Some(Role::Manifest));
        assert_eq!(role_of("puppet.json"), Some(Role::Structure));
        assert_eq!(role_of("textures/20.png"), Some(Role::Texture(20)));
        assert_eq!(role_of("textures/0.png"), Some(Role::Texture(0)));

        for undefined in [
            "notes.txt",
            "",
            "textures/",
            "textures/.png",
            "textures/20.PNG",
            "textures/20.png/",
            "textures/020.png",
            "textures/-1.png",
            "textures/1e3.png",
            "textures/20.png.bak",
            "Textures/20.png",
            "textures\\20.png",
            "textures/18446744073709551616.png",
            "./puppet.json",
            "/puppet.json",
            "../puppet.json",
        ] {
            assert_eq!(
                role_of(undefined),
                None,
                "{undefined:?} must not be defined"
            );
        }
    }

    /// A limit with no figure refuses rather than permits. Vacuous while every
    /// figure is stated, and it is the behaviour that matters if one is ever
    /// removed.
    #[test]
    fn a_count_is_refused_against_its_declared_figure() {
        let limit =
            crate::limits::declared_limit(LimitKind::ArchiveEntries).expect("a stated figure");
        assert!(within(LimitKind::ArchiveEntries, limit, "entries").is_ok());
        let over = within(LimitKind::ArchiveEntries, limit + 1, "entries")
            .expect_err("one past the figure is refused");
        match over {
            Rejection::LimitExceeded {
                limit: kind,
                declared,
                ..
            } => {
                assert_eq!(kind, LimitKind::ArchiveEntries);
                assert_eq!(declared, limit, "the figure reported is the one declared");
            }
            other => panic!("refused as {other:?}"),
        }
    }
}
