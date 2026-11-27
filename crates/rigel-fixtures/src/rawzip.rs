//! Hand-assembled archives, for the fixtures whose whole purpose is a malformed
//! container.
//!
//! A zip library will not emit two entries under one name, a central directory
//! whose offsets overlap, or a header that lies about how far an entry expands
//! — refusing to is the point of a zip library. These fixtures exist so the
//! loader is made to meet exactly those things, so the bytes are laid out here
//! field by field against the PKWARE APPNOTE structures.
//!
//! Only the container is hand-assembled. The `puppet.json` and texture payloads
//! inside are the same ones every other fixture carries.

use std::io::Write;

const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const EOCD_SIGNATURE: u32 = 0x0605_4b50;

/// 1980-01-01 00:00:00 in MS-DOS form — the same constant `archive` pins.
const DOS_TIME: u16 = 0;
const DOS_DATE: u16 = 0x0021;

pub const STORED: u16 = 0;
pub const DEFLATED: u16 = 8;

/// Deflate with no zlib or gzip wrapper, which is what a zip entry carries.
pub fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).expect("deflate accepts bytes");
    encoder.finish().expect("deflate finishes")
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

/// One entry, with every header field separately settable so a fixture can lie
/// about any one of them.
pub struct RawEntry {
    pub name: Vec<u8>,
    pub method: u16,
    /// The bytes written after the local header.
    pub data: Vec<u8>,
    pub crc: u32,
    pub declared_compressed: u32,
    pub declared_uncompressed: u32,
    /// A local-header offset written into the central directory instead of the
    /// real one. This is how an overlapping layout is expressed.
    pub offset_override: Option<u32>,
}

impl RawEntry {
    /// An honest entry: the header fields describe the bytes.
    pub fn stored(name: &str, data: Vec<u8>) -> Self {
        let crc = crc32(&data);
        let size = data.len() as u32;
        RawEntry {
            name: name.as_bytes().to_vec(),
            method: STORED,
            data,
            crc,
            declared_compressed: size,
            declared_uncompressed: size,
            offset_override: None,
        }
    }

    /// An honest stored entry whose name is arbitrary bytes rather than a
    /// `&str` — what the over-long-name fixture needs.
    pub fn stored_named(name: Vec<u8>, data: Vec<u8>) -> Self {
        let crc = crc32(&data);
        let size = data.len() as u32;
        RawEntry {
            name,
            method: STORED,
            data,
            crc,
            declared_compressed: size,
            declared_uncompressed: size,
            offset_override: None,
        }
    }

    /// An honest deflated entry.
    pub fn deflated(name: &str, plain: &[u8]) -> Self {
        let data = deflate(plain);
        RawEntry {
            name: name.as_bytes().to_vec(),
            method: DEFLATED,
            crc: crc32(plain),
            declared_compressed: data.len() as u32,
            declared_uncompressed: plain.len() as u32,
            data,
            offset_override: None,
        }
    }
}

fn put16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Assemble an archive from entries in exactly the order given — no sorting, no
/// validation, no deduplication.
pub fn write(entries: &[RawEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut offsets = Vec::with_capacity(entries.len());

    for entry in entries {
        offsets.push(out.len() as u32);
        put32(&mut out, LOCAL_SIGNATURE);
        put16(&mut out, 20); // version needed
        put16(&mut out, 0); // general purpose flags
        put16(&mut out, entry.method);
        put16(&mut out, DOS_TIME);
        put16(&mut out, DOS_DATE);
        put32(&mut out, entry.crc);
        put32(&mut out, entry.declared_compressed);
        put32(&mut out, entry.declared_uncompressed);
        put16(&mut out, entry.name.len() as u16);
        put16(&mut out, 0); // extra field length
        out.extend_from_slice(&entry.name);
        out.extend_from_slice(&entry.data);
    }

    let central_start = out.len() as u32;
    for (entry, offset) in entries.iter().zip(offsets.iter()) {
        put32(&mut out, CENTRAL_SIGNATURE);
        put16(&mut out, 20); // version made by
        put16(&mut out, 20); // version needed
        put16(&mut out, 0); // general purpose flags
        put16(&mut out, entry.method);
        put16(&mut out, DOS_TIME);
        put16(&mut out, DOS_DATE);
        put32(&mut out, entry.crc);
        put32(&mut out, entry.declared_compressed);
        put32(&mut out, entry.declared_uncompressed);
        put16(&mut out, entry.name.len() as u16);
        put16(&mut out, 0); // extra field length
        put16(&mut out, 0); // comment length
        put16(&mut out, 0); // disk number start
        put16(&mut out, 0); // internal attributes
        put32(&mut out, 0); // external attributes
        put32(&mut out, entry.offset_override.unwrap_or(*offset));
        out.extend_from_slice(&entry.name);
    }
    let central_size = out.len() as u32 - central_start;

    put32(&mut out, EOCD_SIGNATURE);
    put16(&mut out, 0); // this disk
    put16(&mut out, 0); // disk with the central directory
    put16(&mut out, entries.len() as u16);
    put16(&mut out, entries.len() as u16);
    put32(&mut out, central_size);
    put32(&mut out, central_start);
    put16(&mut out, 0); // archive comment length
    out
}
