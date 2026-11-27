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

/// What one entry produced, and how much of it was kept.
struct Produce {
    /// The bytes retained — at most the `retain` ceiling asked for.
    kept: Vec<u8>,
    /// Bytes the stream produced, which is what a limit is enforced against and
    /// may be larger than `kept`.
    count: u64,
    /// Why the stream stopped early, if it did. Held rather than returned so
    /// that a limit is reported before a decode error — an entry that is both
    /// over the limit and malformed is over the limit first.
    unreadable: Option<std::io::Error>,
}

/// Decompresses one entry, retaining `retain` bytes and counting all of them.
///
/// The ceiling is what keeps a failed load's peak below the limit: the bomb's
/// expansion streams through a fixed chunk and is dropped, so the refusal costs
/// the chunk and the entry's declared size rather than the expansion.
///
/// It stops one byte past `limit`, because one byte past settles the question
/// and the rest is work a hostile file asked for (`LIM-6` — the bound on a
/// rejected load is linear work, and this is where the loop is bounded).
fn produce<R: Read>(mut source: R, retain: usize, limit: u64) -> Produce {
    /// Large enough that a real payload costs few reads, small enough that it
    /// is noise beside any limit.
    const CHUNK: usize = 64 * 1024;

    // One chunk to start with, and **one** jump to the full ceiling when a
    // stream actually overflows it. Neither half is incidental:
    //
    //   * doubling the way `read_to_end` does holds the old block and the new
    //     one at every step, and that transient is what put the old shape of
    //     this function at three times the payload limit;
    //   * reserving the declared size up front instead would let a one-kilobyte
    //     file that merely *claims* to hold sixty-four megabytes cost sixty-four
    //     megabytes, which is the same amplification wearing different clothes.
    //
    // So the declaration bounds the buffer and the stream decides when to take
    // it.
    let mut kept = Vec::with_capacity(retain.min(CHUNK));
    // On the heap rather than the stack, so that the counting allocator in the
    // test binaries sees this buffer too — a measurement that silently excludes
    // the scratch space is not the measurement `LIM-3` asked for.
    let mut scratch = vec![0u8; CHUNK];
    let mut count: u64 = 0;
    let mut unreadable = None;
    loop {
        let produced = match source.read(&mut scratch) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                unreadable = Some(e);
                break;
            }
        };
        count = count.saturating_add(produced as u64);
        if kept.len() < retain {
            let room = retain - kept.len();
            let wanted = produced.min(room);
            if kept.len() + wanted > kept.capacity() {
                kept.reserve_exact(retain - kept.len());
            }
            kept.extend_from_slice(&scratch[..wanted]);
        }
        if count > limit {
            break;
        }
    }
    Produce {
        kept,
        count,
        unreadable,
    }
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
    /// The declared size refuses early and bounds what this call **retains**. It
    /// decides nothing about acceptance: what is enforced is the count of bytes
    /// this call actually produces, so an entry that declares a kilobyte and
    /// expands to a quarter of a gigabyte is refused at the limit rather than at
    /// its own word (`LIM-3`).
    ///
    /// # Why the declaration bounds the buffer, which looks like trusting it
    ///
    /// It is the opposite direction from the one `LIM-3` warns about. Trusting a
    /// *small* declaration to mean a small expansion is the decompression bomb;
    /// using it as a **ceiling on what is held** cannot be, because it has
    /// already been refused against the limit above. Producing more bytes than
    /// the central directory records is a refusal either way — a zip entry's
    /// uncompressed size is not advisory — so nothing past that ceiling is ever
    /// wanted, and holding it is the whole of what the bomb buys.
    ///
    /// The shape this replaced was `take(limit + 1).read_to_end(&mut produced)`,
    /// which returned the right [`Rejection`] and held **192 MiB** doing it: 64
    /// MiB of expansion and 128 MiB of `Vec` growth transients. `LIM-3` asks for
    /// peak bytes below the declared limit and that is three times over it. It
    /// was invisible until `tests/shared/counting_allocator.rs` measured it, and
    /// `limits::lim_3_a_decompression_bomb_fails_as_a_limit` now does.
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

        // Retain at most what the directory declared; count everything.
        let retain = usize::try_from(entry.declared_size).unwrap_or(usize::MAX);
        let Produce {
            kept: produced,
            count,
            unreadable,
        } = match method {
            zip::CompressionMethod::Stored => produce(raw, retain, limit),
            zip::CompressionMethod::Deflated => {
                produce(flate2::read::DeflateDecoder::new(raw), retain, limit)
            }
            other => {
                return Err(fault(
                    ArchiveFault::NotAnArchive,
                    format!("{other} is not a compression method the format defines"),
                ));
            }
        };

        if count > limit {
            return Err(Rejection::LimitExceeded {
                limit: LimitKind::DecompressedPayloadBytes,
                declared: limit,
                detail: format!(
                    "an entry declaring {} bytes produced more than {limit}",
                    entry.declared_size
                ),
            });
        }
        if let Some(e) = unreadable {
            return Err(fault(
                ArchiveFault::NotAnArchive,
                format!("entry unreadable: {e}"),
            ));
        }
        // Under the limit and still not the size the directory records: the
        // entry and its own central-directory row disagree about what it holds,
        // which is a broken container rather than a limit.
        if count != entry.declared_size {
            return Err(fault(
                ArchiveFault::NotAnArchive,
                format!(
                    "an entry produced {count} bytes where the archive records {}",
                    entry.declared_size
                ),
            ));
        }

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

    /// What `produce` retains is the ceiling it was given; what it counts is
    /// everything the stream produced. The two differing is the whole of how a
    /// decompression bomb is refused without being held (`LIM-3`).
    #[test]
    fn a_stream_past_the_ceiling_is_counted_and_not_kept() {
        let stream = vec![7u8; 10_000];
        let out = produce(stream.as_slice(), 100, 1_000_000);
        assert_eq!(out.count, 10_000, "every byte produced is counted");
        assert_eq!(out.kept.len(), 100, "and only the ceiling is held");
        assert!(out.unreadable.is_none());
        assert!(out.kept.iter().all(|b| *b == 7), "the prefix is the prefix");
    }

    /// One byte past the limit settles it, so the loop stops there rather than
    /// running a hostile entry to its end.
    #[test]
    fn a_stream_past_the_limit_stops_rather_than_draining() {
        let stream = vec![0u8; 8 * 1024 * 1024];
        let out = produce(stream.as_slice(), 16, 1024);
        assert!(out.count > 1024, "it produced enough to decide");
        assert!(
            out.count < 8 * 1024 * 1024,
            "and stopped instead of draining the whole stream; it read {}",
            out.count
        );
        assert_eq!(out.kept.len(), 16);
    }

    /// A stream shorter than its ceiling keeps all of it, which is every
    /// ordinary entry.
    #[test]
    fn a_stream_within_the_ceiling_is_kept_whole() {
        let stream = vec![3u8; 64];
        let out = produce(stream.as_slice(), 64, 1_000_000);
        assert_eq!(out.count, 64);
        assert_eq!(out.kept, stream);
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
