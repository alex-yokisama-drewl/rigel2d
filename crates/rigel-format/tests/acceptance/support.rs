//! Scaffolding the acceptance suite shares.
//!
//! Nothing here asserts anything. It reaches fixtures **by catalogue name**,
//! never by path, so that a fixture renamed on disk — `:` is not a filename on
//! Windows, which is why `rigged:deep` is `rigged.deep.rigel2d` — moves in one
//! place.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;

/// What the catalogue says a load of this fixture must do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    Accepted,
    Rejected,
}

/// One row of `tests/fixtures/catalogue.json`.
#[derive(Clone, Debug)]
pub struct Fixture {
    /// The catalogue name, which is what every test uses — `rigged:deep`.
    pub name: String,
    /// The file on disk, which is not the name.
    pub file: String,
    /// The one thing that makes this fixture what it is.
    pub note: String,
    pub expect: Expect,
}

/// The committed corpus, read once.
#[derive(Debug)]
pub struct Catalogue {
    pub fixtures: Vec<Fixture>,
    /// The format version the corpus was authored at.
    pub version: (u32, u32),
}

impl Catalogue {
    /// The row for a catalogue name, or a failure naming what was asked for —
    /// a test that silently skips a missing fixture asserts nothing.
    pub fn fixture(&self, name: &str) -> &Fixture {
        self.fixtures
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("no fixture named {name:?} in catalogue.json"))
    }

    /// The path to a fixture, by catalogue name.
    pub fn path(&self, name: &str) -> PathBuf {
        let path = fixture_dir().join(&self.fixture(name).file);
        assert!(path.is_file(), "catalogue names a missing file: {path:?}");
        path
    }
}

/// Where the committed corpus lives.
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// The catalogue, parsed once per test binary.
pub fn catalogue() -> &'static Catalogue {
    static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let raw = fs::read(fixture_dir().join("catalogue.json")).expect("catalogue.json is read");
        let doc: serde_json::Value = serde_json::from_slice(&raw).expect("catalogue.json is JSON");
        let fixtures = doc["fixtures"]
            .as_array()
            .expect("catalogue.json carries a fixtures array")
            .iter()
            .map(|f| Fixture {
                name: f["name"].as_str().expect("a fixture name").to_owned(),
                file: f["file"].as_str().expect("a fixture file").to_owned(),
                note: f["note"].as_str().unwrap_or_default().to_owned(),
                expect: match f["expect"].as_str() {
                    Some("accepted") => Expect::Accepted,
                    Some("rejected") => Expect::Rejected,
                    other => panic!("unknown expect {other:?} in catalogue.json"),
                },
            })
            .collect();
        let version = (
            doc["version"]["major"].as_u64().expect("a major version") as u32,
            doc["version"]["minor"].as_u64().expect("a minor version") as u32,
        );
        Catalogue { fixtures, version }
    })
}

/// The path to a fixture, by catalogue name. The spelling every test uses.
pub fn fixture(name: &str) -> PathBuf {
    catalogue().path(name)
}

/// A fresh, empty directory outside the repository, for a test that writes.
///
/// Fixtures are read-only: a test that saves does so here, and one asserting
/// *opening never writes* asserts against a copy here rather than against the
/// committed corpus.
pub fn scratch(label: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "rigel2d-acceptance-{}-{}-{}",
        process::id(),
        n,
        label.replace(['/', ':'], "-")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// Copies a fixture into a directory of its own and returns the copy's path.
pub fn copy_fixture_into(name: &str, dir: &Path) -> PathBuf {
    let source = fixture(name);
    let target = dir.join(source.file_name().expect("a file name"));
    fs::copy(&source, &target).expect("the fixture copies");
    target
}

/// A fixture copied into a fresh scratch directory, which is where a test that
/// saves over it must work.
pub fn fixture_copy(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_fixture_into(name, &dir)
}

/// Every byte of a file.
pub fn bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("{path:?} is read: {e}"))
}

/// A file's modification time, for *opening never writes* (`VER-3`).
pub fn modified(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or_else(|e| panic!("{path:?} has a modification time: {e}"))
}

/// The names of every entry in an archive, in the order the archive carries
/// them.
pub fn entry_names(archive: &Path) -> Vec<String> {
    let file = fs::File::open(archive).expect("the archive opens");
    let mut zip = zip::ZipArchive::new(file).expect("the archive is a zip");
    (0..zip.len())
        .map(|i| {
            zip.by_index(i)
                .expect("an entry")
                .name()
                .expect("an entry name")
                .into_owned()
        })
        .collect()
}

/// One entry's bytes out of an archive.
pub fn entry_bytes(archive: &Path, entry: &str) -> Vec<u8> {
    use std::io::Read as _;
    let file = fs::File::open(archive).expect("the archive opens");
    let mut zip = zip::ZipArchive::new(file).expect("the archive is a zip");
    let mut found = Vec::new();
    zip.by_name(entry)
        .unwrap_or_else(|e| panic!("{archive:?} carries {entry}: {e}"))
        .read_to_end(&mut found)
        .expect("the entry reads");
    found
}

/// Every entry of an archive, by name.
pub fn entries(archive: &Path) -> BTreeMap<String, Vec<u8>> {
    entry_names(archive)
        .into_iter()
        .map(|name| {
            let payload = entry_bytes(archive, &name);
            (name, payload)
        })
        .collect()
}

/// The parsed structure entry of an archive.
///
/// Used where a criterion is about the **file** rather than about what the
/// loader made of it — canonical position, which keys a physics node carries,
/// whether a defaulted field was written.
pub fn puppet_json(archive: &Path) -> serde_json::Value {
    serde_json::from_slice(&entry_bytes(archive, "puppet.json")).expect("puppet.json is JSON")
}

/// The parsed version entry of an archive.
pub fn version_json(archive: &Path) -> serde_json::Value {
    serde_json::from_slice(&entry_bytes(archive, "rigel2d.json")).expect("rigel2d.json is JSON")
}

/// The node object with this identifier, out of a parsed `puppet.json`.
pub fn json_node(doc: &serde_json::Value, id: u64) -> serde_json::Value {
    doc["nodes"]
        .as_array()
        .expect("a nodes array")
        .iter()
        .find(|n| n["id"].as_u64() == Some(id))
        .unwrap_or_else(|| panic!("puppet.json carries no node {id}"))
        .clone()
}

/// Every string value anywhere in a JSON document, as a sorted multiset.
///
/// The shape `EXT-5` needs: *preserved verbatim* over values this build has no
/// business reading, asserted without naming any of them.
pub fn all_strings(value: &serde_json::Value) -> Vec<String> {
    fn walk(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Array(items) => items.iter().for_each(|i| walk(i, out)),
            serde_json::Value::Object(fields) => {
                for (key, item) in fields {
                    out.push(key.clone());
                    walk(item, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(value, &mut out);
    out.sort();
    out
}

/// A PNG's declared dimensions, read straight out of its IHDR.
///
/// No decoder: the question is what the stored payload *says* it is, which is a
/// header read, and pulling an image library into the acceptance suite to answer
/// it would put a second decoder's opinion in the way.
pub fn png_dimensions(payload: &[u8]) -> (u32, u32) {
    const MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
    assert!(
        payload.len() > 24 && payload.starts_with(MAGIC),
        "the payload is a PNG"
    );
    let width = u32::from_be_bytes(payload[16..20].try_into().expect("four bytes"));
    let height = u32::from_be_bytes(payload[20..24].try_into().expect("four bytes"));
    (width, height)
}

/// Whether a byte stream begins with a PNG signature.
pub fn looks_like_png(payload: &[u8]) -> bool {
    payload.starts_with(b"\x89PNG\r\n\x1a\n")
}

/// Asserts that a fixture is rejected, and rejected **naming the rule it
/// breaks** rather than some other one.
///
/// A corpus of one-defect fixtures is only worth having if a rejection can be
/// attributed: a test that asserts nothing but *some error* passes against a
/// build that refuses every file. `VAL-12` is the criterion that requires the
/// rule and an identifier to be there at all.
pub fn assert_rejected_naming(name: &str, rule: rigel_format::Rule) {
    let rejection = rigel_format::load(&fixture(name))
        .err()
        .unwrap_or_else(|| panic!("{name} carries a defect and must be rejected"));
    let named: Vec<rigel_format::Rule> = rejection.violations().iter().map(|v| v.rule).collect();
    assert!(
        named.contains(&rule),
        "{name} must be rejected naming {rule:?}, not {named:?}"
    );
    assert!(
        !rejection.offenders().is_empty(),
        "{name}'s rejection names at least one offending identifier (VAL-12)"
    );
    assert!(
        rejection.to_string().len() <= rigel_format::Rejection::MAX_MESSAGE_BYTES,
        "{name}'s rejection message is bounded (VAL-12)"
    );
}

/// Every file directly inside a directory, sorted.
pub fn dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("the directory reads")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}
