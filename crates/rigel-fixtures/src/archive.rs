//! The canonical archive container.
//!
//! `docs/format.md` lists every zip default that has to be pinned, because a
//! writer that forgets one produces a file that churns in version control for
//! no reason:
//!
//! - entry order fixed, sorted by path;
//! - modification times fixed to a constant, never the clock;
//! - no directory entries;
//! - compression method and level fixed per entry kind;
//! - no extra fields, and no external attributes carrying the writing machine's
//!   umask;
//! - file names UTF-8, with no platform path separators.
//!
//! Two of those the specification states without naming a value, and the values
//! chosen here are in README.md beside this file.

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;

/// One entry of the archive.
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

impl Entry {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Entry {
            name: name.into(),
            data,
        }
    }
}

/// The pinned modification time: the earliest a zip date field can hold, so
/// there is no question of it having come from a clock.
fn fixed_time() -> zip::DateTime {
    zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("1980-01-01 is a valid zip date")
}

/// Compression is fixed per entry kind: a PNG payload is already deflated
/// inside its own container, so storing it keeps the payload extractable with
/// no second pass and keeps the archive smaller than recompressing would.
fn method_for(name: &str) -> zip::CompressionMethod {
    if name.ends_with(".png") {
        zip::CompressionMethod::Stored
    } else {
        zip::CompressionMethod::Deflated
    }
}

/// Write entries into an archive carrying directory markers, as the ordinary
/// command-line `zip -r` emits by default.
///
/// Not canonical — [`write`] refuses a directory entry, because the canonical
/// writer must not emit one (`docs/format.md` — *Canonical serialisation*).
/// This exists because a reader must nonetheless **accept** one: a third party
/// repacking a puppet with ordinary tooling produces exactly this, and
/// `TOOL-3` is the requirement that they can.
pub fn write_with_directory_markers(entries: Vec<Entry>, directories: &[&str]) -> Vec<u8> {
    write_with_markers_compressed(entries, directories, false)
}

/// As [`write_with_directory_markers`], with the markers deflated rather than
/// stored.
///
/// An empty deflate stream is two bytes, so a marker written this way has
/// `size == 0` and `compressed_size == 2`. Python's `zipfile` produces exactly
/// this, and a reader keying its tolerance on the compressed size accepts the
/// stored form and refuses this one.
pub fn write_with_deflated_directory_markers(entries: Vec<Entry>, directories: &[&str]) -> Vec<u8> {
    write_with_markers_compressed(entries, directories, true)
}

fn write_with_markers_compressed(
    entries: Vec<Entry>,
    directories: &[&str],
    deflate_markers: bool,
) -> Vec<u8> {
    let mut entries = entries;
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let marker_method = if deflate_markers {
            zip::CompressionMethod::Deflated
        } else {
            zip::CompressionMethod::Stored
        };
        let options = SimpleFileOptions::default()
            .compression_method(marker_method)
            .last_modified_time(fixed_time())
            .large_file(false)
            .unix_permissions(0o755);
        for directory in directories {
            writer
                .add_directory(*directory, options)
                .expect("directory marker starts");
        }
        for entry in &entries {
            let options = SimpleFileOptions::default()
                .compression_method(method_for(&entry.name))
                .last_modified_time(fixed_time())
                .large_file(false)
                .unix_permissions(0o644);
            writer
                .start_file(entry.name.clone(), options)
                .expect("archive entry starts");
            writer.write_all(&entry.data).expect("archive entry writes");
        }
        writer.finish().expect("archive finishes");
    }
    buffer.into_inner()
}

/// Write entries into a canonical archive.
///
/// Sorting happens here rather than at the call sites, so a fixture cannot
/// accidentally depend on the order it listed its entries in.
pub fn write(entries: Vec<Entry>) -> Vec<u8> {
    write_checking_names(entries, true)
}

/// As [`write`], without the no-directory-entry assertion.
///
/// For the one fixture whose purpose is an entry wearing a directory name
/// while carrying bytes. The canonical writer must never produce one, which is
/// why [`write`] refuses it.
pub fn write_naming_a_directory(entries: Vec<Entry>) -> Vec<u8> {
    write_checking_names(entries, false)
}

fn write_checking_names(entries: Vec<Entry>, refuse_directories: bool) -> Vec<u8> {
    let mut entries = entries;
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        for entry in &entries {
            assert!(
                !entry.name.contains('\\'),
                "entry names carry no platform path separator: {}",
                entry.name
            );
            assert!(
                !refuse_directories || !entry.name.ends_with('/'),
                "no directory entries: {}",
                entry.name
            );
            let options = SimpleFileOptions::default()
                .compression_method(method_for(&entry.name))
                .last_modified_time(fixed_time())
                .large_file(false)
                .unix_permissions(0o644);
            writer
                .start_file(entry.name.clone(), options)
                .expect("archive entry starts");
            writer.write_all(&entry.data).expect("archive entry writes");
        }
        writer.finish().expect("archive finishes");
    }
    buffer.into_inner()
}
