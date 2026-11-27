//! Reading a puppet file, treating it as hostile input throughout.
//!
//! # The order a load happens in
//!
//! It is the order `docs/format.md` asks for, and each step is reached only
//! because the one before it passed.
//!
//! 1. **The central directory.** Every container fault — an entry name longer
//!    than the format defines, two entries under one name, an entry the table
//!    does not define, a region overlapping another's, a required entry absent
//!    — is refused from the directory alone, decompressing nothing. It comes
//!    first because the manifest cannot be found without walking the directory
//!    that names it.
//! 2. **`rigel2d.json`, on its own.** Deciding whether to refuse a file costs
//!    one small bounded read, before `puppet.json` or any texture is touched.
//!    A differing major is refused whole here and never opened partially.
//! 3. **The parse**, under the load limits, which are enforced against bytes
//!    actually produced rather than against anything the file declares.
//! 4. **Validation**, over the puppet the parse produced.
//!
//! # Why `I4` needs no care here
//!
//! [`load`] takes a path and **returns a value**
//! ([I4](../../../docs/invariants.md) — *a load that fails never replaces what
//! is already displayed*). There is no partial-replacement path to get wrong,
//! because the loader has nothing of the caller's to replace. Nothing in this
//! module may be changed to write into a puppet the caller already holds.

use std::fmt;
use std::path::Path;

use crate::archive::{self, Role};
use crate::json;
use crate::limits::LimitKind;
use crate::puppet::{Id, Puppet};
use crate::validate::Violation;
use crate::version::FormatVersion;

/// What a successful load produced.
///
/// A **value**, not a mutation. [I4](../../../docs/invariants.md) — *a load that
/// fails never replaces what is already displayed* — holds structurally here:
/// there is no partial-replacement path to get wrong, because the loader has
/// nothing of the caller's to replace.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub puppet: Puppet,
    pub report: LoadReport,
}

/// What the runtime reports about a load that succeeded.
///
/// Asserted on directly, never through an interface: neither application's
/// interface is designed, and [I2](../../../docs/invariants.md) binds where any
/// of it may surface in Live (`MSG-1`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadReport {
    /// Present when the file declares a later minor than this build knows. The
    /// file opened, is editable, and everything unrecognised is preserved
    /// (`TOOL-4`, `MSG-1`).
    pub version_gap: Option<VersionGap>,
    /// Every extension namespace in the file this build does not implement, by
    /// its human-readable name and declared version. A puppet declaring it
    /// carries data for a named extension is reported here as missing, and
    /// **nothing is installed, registered or loaded**
    /// ([I6](../../../docs/invariants.md), `EXT-6`, `MSG-1`).
    pub unrecognised_extensions: Vec<ExtensionNotice>,
    /// Textures whose payload is present and would not decode. The puppet is not
    /// rejected, the part keeps its reference, and a save writes the undecodable
    /// payload back byte for byte (`VAL-10`).
    pub unreadable_textures: Vec<Id>,
    /// Bindings kept although their value count no longer matches the mesh they
    /// apply to. A state, reported to the rigger, not a failure (`VAL-9`,
    /// `ART-5`).
    pub stale_bindings: Vec<Id>,
    /// For a person to read. Where a version gap or an unrecognised extension is
    /// reported, carries the warning that the features behind it may not work in
    /// this build (`MSG-1`).
    ///
    /// Never a promise that the file will be preserved intact, and never an
    /// implied install route for a missing extension — `MSG-2` is the gate on
    /// that, and it is a person's to settle.
    pub warnings: Vec<String>,
}

/// A file declaring a later minor version than this build knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionGap {
    pub found: FormatVersion,
    pub known: FormatVersion,
}

/// One extension namespace this build does not implement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExtensionNotice {
    /// The extension's own identifier. **Not** what a person is shown.
    pub namespace: String,
    /// Human-readable, and what a person is shown (`MSG-1`, `MSG-2`).
    pub name: String,
    /// The version the data expects.
    pub version: String,
}

/// Why a file was refused.
///
/// Every variant names the rule broken, and [`Rejection::offenders`] carries at
/// least one offending identifier wherever the file got far enough for there to
/// be one (`VAL-12`). A container or JSON fault is refused before any entity
/// exists to name, which is a limit of the format rather than of this type — see
/// the `rigged:nan` note in `docs/wip/s2-format.md`.
#[derive(Clone, Debug, PartialEq)]
pub enum Rejection {
    /// The archive itself is malformed or hostile. Refused before anything is
    /// decompressed.
    Archive { fault: ArchiveFault, detail: String },
    /// A declared major version this build does not implement. Refused whole,
    /// never opened partially; the message names the version (`VER-2`).
    IncompatibleVersion {
        found: FormatVersion,
        known: FormatVersion,
    },
    /// A declared load limit was exceeded, checked **before allocating**
    /// (`LIM-2`, `LIM-3`, `LIM-7`).
    LimitExceeded {
        limit: LimitKind,
        /// The figure enforced, so a reader can tell which limit bit.
        declared: u64,
        detail: String,
    },
    /// `rigel2d.json` or `puppet.json` is not well-formed JSON. Strict: a bare
    /// `NaN` token is not JSON and is refused here rather than by
    /// [`crate::Rule::NumbersFinite`].
    Malformed { detail: String },
    /// One or more validation rules broken.
    Invalid(Vec<Violation>),
}

impl Rejection {
    /// The ceiling on a rendered rejection. `VAL-12` asks for bounded output: a
    /// message that can grow with the file is a second denial-of-service route
    /// through the error path.
    pub const MAX_MESSAGE_BYTES: usize = 4096;

    /// The rule broken, as a name a person and a test can both read. Never
    /// empty.
    pub fn rule_name(&self) -> String {
        match self {
            Rejection::Archive { fault, .. } => format!("archive.{}", fault.as_str()),
            Rejection::IncompatibleVersion { .. } => String::from("version.major-differs"),
            Rejection::LimitExceeded { limit, .. } => format!("limit.{}", limit.as_str()),
            Rejection::Malformed { .. } => String::from("malformed-json"),
            Rejection::Invalid(violations) if violations.is_empty() => String::from("invalid"),
            Rejection::Invalid(violations) => {
                let mut names: Vec<&'static str> = violations
                    .iter()
                    .map(|broken| broken.rule.as_str())
                    .collect();
                names.dedup();
                names.join(", ")
            }
        }
    }

    /// Every identifier this rejection names. Empty only for a container,
    /// version or JSON fault, which is refused before any entity exists.
    pub fn offenders(&self) -> Vec<Id> {
        let mut out: Vec<Id> = self
            .violations()
            .iter()
            .flat_map(|broken| broken.offenders.iter().copied())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Every rule violation this rejection carries.
    ///
    /// Empty for a container, version or JSON fault, which is refused before any
    /// rule is reached — which is exactly why `rigged:nan`, carrying a bare `NaN`
    /// token that is not strict JSON, may be refused without naming an
    /// identifier.
    pub fn violations(&self) -> &[Violation] {
        match self {
            Rejection::Invalid(violations) => violations,
            _ => &[],
        }
    }
}

impl fmt::Display for Rejection {
    /// At most [`Rejection::MAX_MESSAGE_BYTES`], always naming the rule.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rendered = format!("rejected: {}", self.rule_name());
        match self {
            Rejection::Archive { detail, .. } | Rejection::Malformed { detail } => {
                rendered.push_str(" — ");
                rendered.push_str(detail);
            }
            Rejection::IncompatibleVersion { found, known } => {
                rendered.push_str(&format!(
                    " — the file declares version {}.{}, and this build implements {}.{}",
                    found.major, found.minor, known.major, known.minor
                ));
            }
            Rejection::LimitExceeded {
                declared, detail, ..
            } => {
                rendered.push_str(&format!(" — {detail} (the declared limit is {declared})"));
            }
            Rejection::Invalid(violations) => {
                for broken in violations {
                    rendered.push_str("\n  ");
                    rendered.push_str(broken.rule.as_str());
                    if let Some(field) = &broken.field {
                        rendered.push_str(&format!(" [{field}]"));
                    }
                    if !broken.offenders.is_empty() {
                        let named: Vec<String> = broken
                            .offenders
                            .iter()
                            .map(|offender| offender.0.to_string())
                            .collect();
                        rendered.push_str(&format!(" at {}", named.join(", ")));
                    }
                    rendered.push_str(": ");
                    rendered.push_str(&broken.detail);
                    if rendered.len() > Self::MAX_MESSAGE_BYTES {
                        break;
                    }
                }
            }
        }
        f.write_str(&bounded(rendered))
    }
}

/// Cuts a rendered rejection to [`Rejection::MAX_MESSAGE_BYTES`], on a
/// character boundary, with an ellipsis so a reader knows it was cut.
fn bounded(mut rendered: String) -> String {
    const ELLIPSIS: &str = "…";
    if rendered.len() <= Rejection::MAX_MESSAGE_BYTES {
        return rendered;
    }
    let mut cut = Rejection::MAX_MESSAGE_BYTES - ELLIPSIS.len();
    while cut > 0 && !rendered.is_char_boundary(cut) {
        cut -= 1;
    }
    rendered.truncate(cut);
    rendered.push_str(ELLIPSIS);
    rendered
}

impl std::error::Error for Rejection {}

/// What is wrong with the container, before any of it is trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArchiveFault {
    /// Not a zip archive at all.
    NotAnArchive,
    /// An entry the format's table does not define.
    UndefinedEntryName,
    /// Two entries with one name.
    DuplicateEntryName,
    /// An entry name longer than [`LimitKind::EntryNameBytes`] permits.
    EntryNameTooLong,
    /// A central-directory offset landing inside another entry's data.
    OverlappingEntryData,
    /// An entry the format requires is absent.
    MissingRequiredEntry,
}

impl ArchiveFault {
    /// The fault's name, for a rejection a person and a test can both read.
    pub fn as_str(self) -> &'static str {
        match self {
            ArchiveFault::NotAnArchive => "not-an-archive",
            ArchiveFault::UndefinedEntryName => "undefined-entry-name",
            ArchiveFault::DuplicateEntryName => "duplicate-entry-name",
            ArchiveFault::EntryNameTooLong => "entry-name-too-long",
            ArchiveFault::OverlappingEntryData => "overlapping-entry-data",
            ArchiveFault::MissingRequiredEntry => "missing-required-entry",
        }
    }
}

/// Reads the declared version, and nothing else.
///
/// `rigel2d.json` exists so that deciding whether to refuse a file costs one
/// small bounded read, **before any load limit is at risk and before the manifest
/// is parsed** (`docs/format.md`). A caller deciding whether to open a file at
/// all uses this; [`load`] performs the same check for itself.
///
/// The version is **returned, not judged**: refusing a differing major is
/// [`load`]'s, so that a caller can tell a person which version a file wants.
pub fn read_version(path: &Path) -> Result<FormatVersion, Rejection> {
    let mut container = archive::open(path)?;
    let entry = container
        .find(Role::Manifest)
        .ok_or_else(|| Rejection::Archive {
            fault: ArchiveFault::MissingRequiredEntry,
            detail: String::from("the archive carries no rigel2d.json"),
        })?
        .clone();
    let bytes = container.read(&entry)?;
    manifest_version(&bytes)
}

/// The format name and the two numbers `rigel2d.json` carries, and nothing
/// a reader must interpret.
fn manifest_version(bytes: &[u8]) -> Result<FormatVersion, Rejection> {
    let document = json::parse(bytes).map_err(|e| Rejection::Malformed {
        detail: format!("rigel2d.json: {}", e.detail),
    })?;
    let declared = document
        .get("format")
        .and_then(json::Json::as_str)
        .ok_or_else(|| Rejection::Malformed {
            detail: String::from("rigel2d.json declares no format name"),
        })?;
    if declared != crate::version::FORMAT_NAME {
        return Err(Rejection::Malformed {
            detail: format!("rigel2d.json declares the format {declared:?}"),
        });
    }
    let version = document
        .get("version")
        .ok_or_else(|| Rejection::Malformed {
            detail: String::from("rigel2d.json declares no version"),
        })?;
    let number = |key: &str| -> Result<u32, Rejection> {
        version
            .get(key)
            .and_then(json::Json::as_u64)
            .and_then(|found| u32::try_from(found).ok())
            .ok_or_else(|| Rejection::Malformed {
                detail: format!("rigel2d.json's {key} version is not a number"),
            })
    };
    Ok(FormatVersion {
        major: number("major")?,
        minor: number("minor")?,
    })
}

/// Loads a puppet file.
///
/// Treats the file as hostile throughout: limits are enforced against bytes
/// actually produced rather than against anything the file declares, and nothing
/// in the file is executed, resolved or interpolated
/// ([I1](../../../docs/invariants.md), `EXEC-1`, `EXT-5`).
///
/// **Opening never writes** (`VER-3`), on the rejecting path as much as the
/// succeeding one (`VAL-11`).
///
/// Material this build does not understand — marked extension data, and in a
/// file declaring a later minor version, unmarked fields, unknown node kinds and
/// unfamiliar enumerated values alike — is preserved opaquely and never
/// interpreted, so that it survives a save byte for byte (`VER-1`, `VER-6`,
/// `EXT-7`, `EXT-9`).
pub fn load(path: &Path) -> Result<Loaded, Rejection> {
    let opened = read_file(path)?;
    // The rules the source text alone can show are reached first, before the
    // validator, because a `Puppet` cannot hold the facts they are about — an
    // absent member, a key outside `x`, a token gated on the declared version.
    // Both halves live in `validate::verdict`, which `validate_file` calls too,
    // so the loader's verdict and the standalone one are the same verdict and
    // not two that drift (`VAL-13`).
    crate::validate::verdict(&opened)?;
    Ok(Loaded {
        puppet: opened.puppet,
        report: opened.report,
    })
}

/// A file read into a puppet, with the rules only the source text could show.
///
/// Shared with [`crate::validate::validate_file`], so that the standalone
/// verdict and the loader's are one verdict rather than two implementations
/// that drift (`VAL-13`).
#[derive(Debug)]
pub(crate) struct Opened {
    pub puppet: Puppet,
    pub report: LoadReport,
    pub violations: Vec<Violation>,
}

pub(crate) fn read_file(path: &Path) -> Result<Opened, Rejection> {
    let mut container = archive::open(path)?;

    // Step 2: the manifest, on its own, before anything else is decompressed.
    let manifest = container
        .find(Role::Manifest)
        .expect("archive::open refuses a file with no manifest")
        .clone();
    let found = manifest_version(&container.read(&manifest)?)?;
    let known = FormatVersion::CURRENT;
    if found.major != known.major {
        return Err(Rejection::IncompatibleVersion { found, known });
    }
    let licensed = found.minor > known.minor;

    // Step 3: the structure, and the parse.
    let structure = container
        .find(Role::Structure)
        .expect("archive::open refuses a file with no structure entry")
        .clone();
    let bytes = container.read(&structure)?;
    let document = json::parse(&bytes).map_err(|e| Rejection::Malformed {
        detail: format!("puppet.json: {}", e.detail),
    })?;
    let mut decoded = crate::decode::decode(&document, licensed)?;

    // The texture payloads, each under the same per-entry limit.
    let mut unreadable = Vec::new();
    for record in &mut decoded.puppet.textures {
        let entry = container
            .find(Role::Texture(record.id.0))
            .ok_or_else(|| Rejection::Archive {
                fault: ArchiveFault::MissingRequiredEntry,
                detail: format!("texture {} has no payload entry", record.id.0),
            })?
            .clone();
        record.payload = container.read(&entry)?;
        match png_dimensions(&record.payload) {
            Some([width, height]) => {
                archive::within(LimitKind::TextureDimension, u64::from(width), "a payload")?;
                archive::within(LimitKind::TextureDimension, u64::from(height), "a payload")?;
            }
            // Not a rejection: the texture is reported, the part keeps its
            // reference, and a save writes the bytes back rather than
            // re-encoding something it could not read (`VAL-10`).
            None => unreadable.push(record.id),
        }
    }
    // An entry naming a texture the puppet does not carry would be material
    // riding along inside the file with nothing accounting for it.
    for entry in container.entries() {
        if let Role::Texture(id) = entry.role
            && !decoded
                .puppet
                .textures
                .iter()
                .any(|record| record.id.0 == id)
        {
            return Err(Rejection::Malformed {
                detail: format!(
                    "the archive carries a payload for texture {id}, which the puppet does not"
                ),
            });
        }
    }

    let version_gap = licensed.then_some(VersionGap { found, known });
    let mut warnings = Vec::new();
    if version_gap.is_some() {
        warnings.push(format!(
            "This file was written for format version {}.{}, and this build implements {}.{}. \
             It opens and stays editable; features added after {}.{} will not work here.",
            found.major, found.minor, known.major, known.minor, known.major, known.minor
        ));
    }
    if !decoded.unrecognised_extensions.is_empty() {
        let named: Vec<&str> = decoded
            .unrecognised_extensions
            .iter()
            .map(|notice| notice.name.as_str())
            .collect();
        warnings.push(format!(
            "This build has no code for {}. Whatever those add to the puppet will not work here.",
            named.join(", ")
        ));
    }

    Ok(Opened {
        puppet: decoded.puppet,
        report: LoadReport {
            version_gap,
            unrecognised_extensions: decoded.unrecognised_extensions,
            unreadable_textures: unreadable,
            stale_bindings: decoded.stale_bindings,
            warnings,
        },
        violations: decoded.violations,
    })
}

/// A PNG's declared dimensions, or `None` for a payload that is not one.
///
/// A **header** read, not a decode. The format crate creates no GPU resource
/// and decodes no image — that is `rigel-runtime`'s — and what `VAL-10` asks
/// for is whether the payload is an image at all. A signature with rubbish
/// after it, which is what `rigged:texture-payload-corrupt` carries, fails at
/// the first chunk.
fn png_dimensions(payload: &[u8]) -> Option<[u32; 2]> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    /// A PNG's first chunk is always `IHDR`, and its data is always 13 bytes
    /// (PNG specification, 11.2.2).
    const IHDR_LENGTH: [u8; 4] = [0, 0, 0, 13];

    if !payload.starts_with(SIGNATURE) || payload.len() < SIGNATURE.len() + 8 + 13 {
        return None;
    }
    let chunk = &payload[SIGNATURE.len()..];
    if chunk[0..4] != IHDR_LENGTH || &chunk[4..8] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(chunk[8..12].try_into().ok()?);
    let height = u32::from_be_bytes(chunk[12..16].try_into().ok()?);
    if width == 0 || height == 0 {
        return None;
    }
    // Every PNG ends with an empty IEND chunk, which is what a payload cut
    // short is missing.
    if !payload.ends_with(b"IEND\xae\x42\x60\x82") {
        return None;
    }
    Some([width, height])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puppet::{AxisPolarity, NodeBody, OutputSource, TextureFlip};
    use crate::validate::Rule;
    use std::path::PathBuf;

    /// The hand-authored corpus, which is built by `rigel-fixtures` from
    /// `docs/format.md` using **no project code**. Reading it here is what makes
    /// these tests about the specification rather than about this crate
    /// agreeing with itself.
    fn fixture(file: &str) -> PathBuf {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(file);
        assert!(path.is_file(), "{path:?} is missing");
        path
    }

    /// The parse, with the validator left out of it: what this task owns ends
    /// where [`crate::validate`] begins, and every body in that module is still
    /// `unimplemented!()`.
    fn opened(file: &str) -> Opened {
        read_file(&fixture(file)).unwrap_or_else(|e| panic!("{file} is read: {e}"))
    }

    /// The refusal the reader reaches, which is what [`load`] turns into an
    /// error before the validator runs. A file refused for a container, version
    /// or limit fault never produces an [`Opened`] at all; one breaking a rule
    /// the source text alone can show produces one carrying the violations.
    fn refused(file: &str) -> Rejection {
        match read_file(&fixture(file)) {
            Err(rejection) => rejection,
            Ok(opened) if !opened.violations.is_empty() => Rejection::Invalid(opened.violations),
            Ok(_) => {
                panic!("{file} carries a defect the reader must refuse before validation")
            }
        }
    }

    fn node(puppet: &Puppet, id: u64) -> &crate::puppet::Node {
        puppet
            .nodes
            .iter()
            .find(|node| node.id == Id(id))
            .unwrap_or_else(|| panic!("no node {id}"))
    }

    // ------------------------------------------------- the whole corpus

    /// **No fixture the catalogue calls valid is refused by the reader**, and
    /// every refusal the reader does reach names a rule in bounded output.
    ///
    /// The sweep that catches a reader too strict in a way no single test
    /// names — a limit set below a real puppet, a member read as required that
    /// the format defaults, a container check that refuses an honest archive.
    /// The other direction is the validator's: a fixture the catalogue calls
    /// invalid may well parse here, because most of what makes a puppet invalid
    /// is a rule over the puppet rather than a fact about its text.
    #[test]
    fn no_valid_fixture_is_refused_by_the_reader() {
        let directory = fixture("rigged.rigel2d")
            .parent()
            .expect("the corpus directory")
            .to_owned();
        let raw = std::fs::read(directory.join("catalogue.json")).expect("the catalogue reads");
        let catalogue = crate::json::parse(&raw).expect("the catalogue is JSON");
        let rows = catalogue
            .get("fixtures")
            .and_then(crate::json::Json::as_array)
            .expect("a fixtures array");
        assert!(
            rows.len() > 50,
            "the corpus is there, or this asserts nothing"
        );

        for row in rows {
            let name = row
                .get("name")
                .and_then(crate::json::Json::as_str)
                .expect("a name");
            let file = row
                .get("file")
                .and_then(crate::json::Json::as_str)
                .expect("a file");
            let valid = row.get("expect").and_then(crate::json::Json::as_str) == Some("accepted");
            match read_file(&directory.join(file)) {
                Ok(opened) => assert!(
                    !valid || opened.violations.is_empty(),
                    "{name} is a valid puppet and the reader found {:?}",
                    opened
                        .violations
                        .iter()
                        .map(|broken| broken.rule.as_str())
                        .collect::<Vec<_>>()
                ),
                Err(rejection) => {
                    assert!(
                        !valid,
                        "{name} is a valid puppet and was refused: {rejection}"
                    );
                    assert!(
                        !rejection.rule_name().is_empty(),
                        "{name} is refused without naming a rule"
                    );
                    assert!(rejection.to_string().len() <= Rejection::MAX_MESSAGE_BYTES);
                }
            }
        }
    }

    // ------------------------------------------------------ the parse

    /// The base fixture decodes to the puppet the catalogue describes, piece by
    /// piece. Everything else in this module is a variant of this one.
    #[test]
    fn the_base_fixture_decodes_to_what_it_carries() {
        let puppet = opened("rigged.rigel2d").puppet;

        assert_eq!(puppet.id, Id(1));
        assert_eq!(puppet.name, "rigged");
        assert_eq!(puppet.high_water_mark, 10_000);
        assert_eq!(puppet.root, Id(10));
        assert_eq!(puppet.gravity, 1.6, "stated, and not the default");
        assert_eq!(puppet.composition_mode, crate::puppet::CompositionMode::Sum);
        assert!(
            puppet.preserved.is_empty(),
            "a file at this version preserves nothing: there is nothing it may carry"
        );

        // The tree, in the order the file carries it, which is depth-first
        // preorder with siblings in draw order.
        assert_eq!(
            puppet.nodes.iter().map(|n| n.id.0).collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14, 15, 16, 17]
        );
        assert_eq!(node(&puppet, 10).parent, Id::NONE, "the root has no parent");
        assert!(matches!(node(&puppet, 10).body, NodeBody::Composite));
        assert!(matches!(node(&puppet, 11).body, NodeBody::MeshGroup(_)));

        let NodeBody::Part(hair) = &node(&puppet, 13).body else {
            panic!("node 13 is a part")
        };
        assert_eq!(hair.texture, Id(21), "a part references a texture");
        assert_eq!(hair.mask, Id(17));
        assert_eq!(hair.texture_flip, TextureFlip::default(), "absent is unset");
        let chain = hair.chain_deformer.as_ref().expect("a chain deformer");
        assert_eq!(chain.id, Id(80));
        assert_eq!(
            chain.joints.iter().map(|j| j.id.0).collect::<Vec<_>>(),
            vec![81, 82, 83]
        );
        let provenance = hair.provenance.as_ref().expect("import provenance");
        assert_eq!(provenance.source_id, "layer-hair");

        let NodeBody::Part(face) = &node(&puppet, 12).body else {
            panic!("node 12 is a part")
        };
        assert_eq!(face.mask, Id::NONE, "an absent mask is the reserved zero");
        assert_eq!(face.mesh.vertices.len(), 4);
        assert_eq!(face.mesh.triangles.len(), 2);

        // The physics constants `docs/format.md` defines, and the defaults it
        // gives for the three a node driving nothing leaves out.
        let NodeBody::Physics(driver) = &node(&puppet, 14).body else {
            panic!("node 14 is physics")
        };
        assert_eq!(driver.length, 120.0);
        assert_eq!(driver.gravity, 0.8);
        assert_eq!(driver.frequency, 1.4);
        assert_eq!(driver.angle_damping, 0.35);
        assert_eq!(driver.length_damping, 0.6);
        assert_eq!(driver.output, Id(33));
        assert_eq!(driver.output_source, vec![OutputSource::Length]);
        assert_eq!(driver.output_scale, vec![0.75]);

        let NodeBody::Physics(tip) = &node(&puppet, 15).body else {
            panic!("node 15 is physics")
        };
        assert_eq!(tip.output, Id::NONE, "driving nothing is ordinary");
        assert!(tip.output_source.is_empty());
        assert!(tip.output_scale.is_empty());

        // A parameter declares a polarity per axis, and the two-dimensional one
        // declares a different value on each.
        let lean = puppet
            .parameters
            .iter()
            .find(|p| p.id == Id(32))
            .expect("the two-dimensional parameter");
        assert_eq!(lean.axes.len(), 2);
        assert_eq!(lean.axes[0].polarity, AxisPolarity::Bilateral);
        assert_eq!(lean.axes[1].polarity, AxisPolarity::Unipolar);
        assert_eq!(lean.keypoints.len(), 6);
        assert_eq!(
            lean.keypoints.iter().filter(|k| k.neutral).count(),
            1,
            "an absent neutral mark is false"
        );

        assert_eq!(
            puppet.bindings.iter().map(|b| b.id.0).collect::<Vec<_>>(),
            vec![50, 51, 52, 53]
        );
        assert!(puppet.bindings.iter().all(|b| !b.stale));
        assert_eq!(puppet.animations.len(), 1);
        assert_eq!(puppet.animations[0].tracks.len(), 2);
        assert_eq!(puppet.channel_mappings.len(), 1);
        assert_eq!(puppet.channel_mappings[0].channel, "mouth_open");
        assert_eq!(
            puppet.channel_mappings[0].source, "",
            "a mapping names no source in almost every puppet"
        );
        assert_eq!(
            puppet.textures.iter().map(|t| t.id.0).collect::<Vec<_>>(),
            vec![20, 21, 22]
        );
        assert!(
            puppet.textures.iter().all(|t| !t.payload.is_empty()),
            "every record's payload came out of the archive"
        );
    }

    /// The smallest valid puppet leaves four collections out entirely, and
    /// absent is empty rather than malformed.
    #[test]
    fn a_puppet_carrying_no_parameters_is_read_as_carrying_none() {
        let puppet = opened("one-part.rigel2d").puppet;
        assert!(puppet.parameters.is_empty());
        assert!(puppet.bindings.is_empty());
        assert!(puppet.animations.is_empty());
        assert!(puppet.channel_mappings.is_empty());
        assert!(!puppet.nodes.is_empty());
    }

    /// `PHYS-10`'s reading half: an absent gravity is the default, and reading
    /// it changes nothing else about the puppet.
    #[test]
    fn an_absent_gravity_is_the_default_and_touches_nothing_else() {
        let without = opened("rigged.no-gravity.rigel2d").puppet;
        let with = opened("rigged.rigel2d").puppet;
        assert_eq!(without.gravity, 1.0);
        assert_ne!(with.gravity, without.gravity);
        assert_eq!(with.nodes, without.nodes);
    }

    // ------------------------------------------- the container faults

    #[test]
    fn each_hostile_container_is_refused_as_the_fault_it_is() {
        let cases = [
            (
                "rigged.entry-name-overlong.rigel2d",
                ArchiveFault::EntryNameTooLong,
            ),
            (
                "rigged.duplicate-entry-names.rigel2d",
                ArchiveFault::DuplicateEntryName,
            ),
            (
                "rigged.overlapping-local-header-data.rigel2d",
                ArchiveFault::OverlappingEntryData,
            ),
            (
                "rigged.undefined-entry-name.rigel2d",
                ArchiveFault::UndefinedEntryName,
            ),
        ];
        for (file, expected) in cases {
            match refused(file) {
                Rejection::Archive { fault, .. } => {
                    assert_eq!(fault, expected, "{file} is refused as the wrong fault")
                }
                other => panic!("{file} was refused as {other:?}"),
            }
        }
    }

    /// A file that is not an archive at all, and one that is an archive and not
    /// a puppet, are both refused rather than read part-way.
    #[test]
    fn something_that_is_not_an_archive_is_refused() {
        let directory = fixture("rigged.rigel2d")
            .parent()
            .expect("a directory")
            .to_owned();
        match read_file(&directory.join("catalogue.json")) {
            Err(Rejection::Archive {
                fault: ArchiveFault::NotAnArchive,
                ..
            }) => {}
            other => panic!("a JSON file is not an archive: {other:?}"),
        }
        match read_file(&directory.join("no-such-file.rigel2d")) {
            Err(Rejection::Archive {
                fault: ArchiveFault::NotAnArchive,
                ..
            }) => {}
            other => panic!("a missing file is refused: {other:?}"),
        }
    }

    /// `LIM-3`: the declaration refuses early and decides nothing else. This
    /// entry declares a kilobyte and produces a quarter of a gigabyte, and the
    /// limit is enforced against what it produced.
    #[test]
    fn a_decompression_bomb_is_refused_against_bytes_actually_produced() {
        match refused("rigged.decompression-bomb.rigel2d") {
            Rejection::LimitExceeded {
                limit,
                declared,
                detail,
            } => {
                assert_eq!(limit, LimitKind::DecompressedPayloadBytes);
                assert_eq!(
                    Some(declared),
                    crate::limits::declared_limit(LimitKind::DecompressedPayloadBytes)
                );
                assert!(
                    detail.contains("produced"),
                    "refused for what it produced, not for what it declared: {detail}"
                );
            }
            other => panic!("refused as {other:?}"),
        }
    }

    // --------------------------------------------------- the version gate

    #[test]
    fn the_version_is_readable_on_its_own() {
        assert_eq!(
            read_version(&fixture("rigged.rigel2d")).expect("it reads"),
            FormatVersion::CURRENT
        );
        let breaking =
            read_version(&fixture("rigged.breaking-version.rigel2d")).expect("it still reads");
        assert_ne!(
            breaking.major,
            FormatVersion::CURRENT.major,
            "read_version reports the version rather than judging it, so a caller \
             can say which version a file wants"
        );
    }

    #[test]
    fn a_differing_major_is_refused_whole() {
        match refused("rigged.breaking-version.rigel2d") {
            Rejection::IncompatibleVersion { found, known } => {
                assert_ne!(found.major, known.major);
                assert_eq!(known, FormatVersion::CURRENT);
            }
            other => panic!("refused as {other:?}"),
        }
    }

    #[test]
    fn an_earlier_minor_opens_with_no_gap_reported() {
        let opened = opened("rigged.old-version.rigel2d");
        assert!(opened.report.version_gap.is_none());
        assert!(opened.violations.is_empty());
    }

    // ------------------------------------------------------ preservation

    /// `VER-1`, `EXT-7`, `TOOL-4`: a later minor licenses three kinds of
    /// material this build has no code for, and each survives somewhere the
    /// writer can find it again.
    #[test]
    fn a_later_minor_licenses_and_preserves_what_this_build_cannot_read() {
        let later = opened("rigged.future-minor.rigel2d");
        assert!(later.violations.is_empty(), "a later minor is not a defect");
        let gap = later.report.version_gap.expect("a version gap is reported");
        assert!(gap.found > gap.known);
        assert!(!later.report.warnings.is_empty());

        let puppet = &later.puppet;
        let base = opened("rigged.rigel2d").puppet;

        // An unknown node kind is opaque material, never a reported entity.
        assert_eq!(
            puppet.nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
            base.nodes.iter().map(|n| n.id).collect::<Vec<_>>()
        );
        let opaque = &puppet.preserved.opaque_nodes;
        assert_eq!(opaque.len(), 1, "the warp_field node");
        assert_eq!(opaque[0].parent, puppet.root);
        assert!(
            String::from_utf8_lossy(&opaque[0].source).contains("warp_field"),
            "kept as the text it arrived as"
        );

        // An unmarked member of a node this build does know is kept against it.
        let kept = puppet.preserved.at(Id(12), &[]);
        assert_eq!(kept.len(), 1, "the unmarked wiggle member");
        assert_eq!(kept[0].0, "wiggle");
        assert_eq!(
            String::from_utf8_lossy(&kept[0].1),
            r#"{"amplitude":0.25,"axis":"horizontal","segments":4}"#,
            "contents verbatim"
        );

        // An unfamiliar value in a field this build interprets carries its
        // token, and is a value distinct from every one the format defines.
        let NodeBody::Physics(physics) = &node(puppet, 14).body else {
            panic!("node 14 is physics")
        };
        assert_eq!(
            physics.output_source,
            vec![OutputSource::Unfamiliar(String::from("torque"))]
        );
    }

    /// `EXT-8`: the gate is the **declared version**, not recognisability. The
    /// same bytes, licensed above and a defect here.
    #[test]
    fn the_same_unmarked_material_at_this_version_is_a_defect() {
        let rejection = refused("rigged.ext-unmarked.rigel2d");
        let named: Vec<Rule> = rejection.violations().iter().map(|v| v.rule).collect();
        assert!(named.contains(&Rule::UnmarkedData), "{named:?}");
        assert!(
            rejection.offenders().contains(&Id(12)),
            "naming the node that carried it"
        );
        assert_eq!(
            rejection.violations()[0].field.as_deref(),
            Some("wiggle"),
            "and the key"
        );
    }

    /// `VAL-7`, `VAL-20`: the same gate over an enumerated field's value.
    #[test]
    fn an_unfamiliar_enumerated_value_is_gated_on_the_declared_version() {
        for file in [
            "rigged.unknown-enum-unmarked.rigel2d",
            "rigged.polarity-value-unfamiliar.rigel2d",
        ] {
            let rejection = refused(file);
            let named: Vec<Rule> = rejection.violations().iter().map(|v| v.rule).collect();
            assert!(
                named.contains(&Rule::EnumeratedValueKnown),
                "{file}: {named:?}"
            );
            assert!(!rejection.offenders().is_empty());
        }

        let licensed = opened("rigged.future-minor-polarity-unfamiliar.rigel2d");
        assert!(licensed.violations.is_empty());
        assert!(
            licensed
                .puppet
                .parameters
                .iter()
                .flat_map(|p| p.axes.iter())
                .any(|axis| matches!(axis.polarity, AxisPolarity::Unfamiliar(_))),
            "the same value, licensed by the declared version"
        );
    }

    /// `EXT-9`: bytes inside opaquely preserved material are never examined, so
    /// a literal that would overflow to infinity is carried rather than read.
    #[test]
    fn a_nonfinite_literal_inside_opaque_material_is_never_read_as_a_number() {
        let opened = opened("rigged.future-minor-nonfinite.rigel2d");
        assert!(opened.violations.is_empty());
        let carried: Vec<String> = opened
            .puppet
            .preserved
            .opaque_nodes
            .iter()
            .map(|node| String::from_utf8_lossy(&node.source).into_owned())
            .collect();
        assert!(
            carried.iter().any(|source| source.contains("1e400")),
            "the literal is kept as its own text: {carried:?}"
        );
    }

    /// `VAL-21`, `VAL-17`: an absence is not the value *unknown*, and is
    /// refused at every declared version — a `Puppet` has no inhabitant for an
    /// axis that declared nothing, so the rule is unreachable after the parse.
    #[test]
    fn an_absent_polarity_is_refused_rather_than_read_as_unknown() {
        let rejection = refused("rigged.missing-polarity.rigel2d");
        let named: Vec<Rule> = rejection.violations().iter().map(|v| v.rule).collect();
        assert!(named.contains(&Rule::AxisPolarityDeclared), "{named:?}");
    }

    // -------------------------------------------------- extension data

    /// `EXT-1`, `EXT-3`, `MSG-1`: the marker is read into typed entries,
    /// every namespace is reported as unrecognised because this build
    /// implements none, and a marker missing its version or its name is a
    /// defect.
    #[test]
    fn marked_extension_data_is_read_reported_and_never_acted_on() {
        let opened = opened("rigged.rigel2d");
        let entries: Vec<_> = opened
            .puppet
            .nodes
            .iter()
            .flat_map(|n| n.extensions.entries.iter())
            .collect();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert!(!entry.namespace.is_empty());
            assert!(!entry.name.is_empty());
            assert!(!entry.version.is_empty());
            assert!(!entry.data.is_empty());
        }
        let notices = &opened.report.unrecognised_extensions;
        assert!(
            notices
                .windows(2)
                .all(|pair| pair[0].namespace < pair[1].namespace),
            "reported once each, in namespace order"
        );
        for notice in notices {
            assert_ne!(
                notice.name, notice.namespace,
                "a person is shown the human-readable name"
            );
        }

        for (file, rule) in [
            (
                "rigged.ext-no-version.rigel2d",
                Rule::ExtensionDeclaresVersion,
            ),
            ("rigged.ext-no-label.rigel2d", Rule::ExtensionDeclaresName),
        ] {
            let rejection = refused(file);
            let named: Vec<Rule> = rejection.violations().iter().map(|v| v.rule).collect();
            assert!(named.contains(&rule), "{file}: {named:?}");
        }
    }

    /// `EXT-4`: position is canonical. The same two namespaces written in the
    /// other order read back in one order, so the writer has nothing left to
    /// decide.
    #[test]
    fn extension_entries_come_back_in_namespace_order_whatever_order_they_arrived() {
        let base = opened("rigged.rigel2d").puppet;
        let moved = opened("rigged.ext-moved.rigel2d").puppet;
        for (left, right) in base.nodes.iter().zip(&moved.nodes) {
            assert_eq!(
                left.extensions, right.extensions,
                "node {} reads the same either way",
                left.id.0
            );
        }
    }

    /// `EXT-6`, [I6](../../../docs/invariants.md): a declaration is reported
    /// and is otherwise inert. Nothing here installs, registers or loads.
    #[test]
    fn a_declared_extension_is_reported_as_missing() {
        let opened = opened("rigged.ext-declares-plugin.rigel2d");
        let declared = opened
            .report
            .unrecognised_extensions
            .iter()
            .find(|notice| notice.namespace == "com.example.physics-plus")
            .expect("the declaration is reported");
        assert!(!declared.name.is_empty() && !declared.version.is_empty());
    }

    /// `EXT-5`, `EXEC-2`: a hostile-shaped string is data. It is read into the
    /// puppet as it stands and nothing resolves it.
    #[test]
    fn hostile_shaped_strings_are_read_as_the_strings_they_are() {
        let opened = opened("rigged.ext-hostile.rigel2d");
        assert!(opened.violations.is_empty());
        let mut strings: Vec<&str> = opened
            .puppet
            .nodes
            .iter()
            .map(|node| node.name.as_str())
            .collect();
        strings.push(opened.puppet.name.as_str());
        assert!(
            strings
                .iter()
                .any(|value| value.contains("..") || value.starts_with('/')),
            "the fixture carries path-shaped values, or this asserts nothing: {strings:?}"
        );
    }

    // ------------------------------------------------------ the report

    /// `VAL-9`: a stale binding is a state, reported to the rigger.
    #[test]
    fn the_stale_mark_is_read_and_reported() {
        let opened = opened("rigged.binding-count-mismatch-marked-stale.rigel2d");
        let marked: Vec<Id> = opened
            .puppet
            .bindings
            .iter()
            .filter(|binding| binding.stale)
            .map(|binding| binding.id)
            .collect();
        assert!(!marked.is_empty());
        assert_eq!(marked, opened.report.stale_bindings);
    }

    /// `VAL-10`: an undecodable payload is reported, not rejected, and the
    /// bytes are kept exactly so a save can write them back rather than
    /// re-encode something it could not read.
    #[test]
    fn an_undecodable_payload_is_reported_and_kept_verbatim() {
        let opened = opened("rigged.texture-payload-corrupt.rigel2d");
        assert_eq!(opened.report.unreadable_textures.len(), 1);
        let id = opened.report.unreadable_textures[0];
        let record = opened
            .puppet
            .textures
            .iter()
            .find(|texture| texture.id == id)
            .expect("the record is still there");
        assert!(
            record.payload.starts_with(b"\x89PNG\r\n\x1a\n"),
            "the bytes are the bytes that arrived"
        );
        assert!(
            opened
                .puppet
                .nodes
                .iter()
                .any(|node| matches!(&node.body, NodeBody::Part(part) if part.texture == id)),
            "and the part keeps its reference"
        );
        // Every other payload read as an image.
        assert_eq!(
            opened
                .puppet
                .textures
                .iter()
                .filter(|texture| texture.id != id)
                .count(),
            2
        );
    }

    fn violation(rule: Rule, offenders: Vec<u64>) -> Violation {
        Violation {
            rule,
            offenders: offenders.into_iter().map(Id).collect(),
            field: Some(String::from("length")),
            detail: String::from("a detail"),
        }
    }

    /// `VAL-12`: every rejection names a rule, whatever family it is in.
    #[test]
    fn every_rejection_names_a_rule() {
        let rejections = [
            Rejection::Archive {
                fault: ArchiveFault::DuplicateEntryName,
                detail: String::from("two entries share one name"),
            },
            Rejection::IncompatibleVersion {
                found: FormatVersion { major: 2, minor: 0 },
                known: FormatVersion::CURRENT,
            },
            Rejection::LimitExceeded {
                limit: LimitKind::NodeCount,
                declared: 2160,
                detail: String::from("nodes: 2161 exceeds 2160"),
            },
            Rejection::Malformed {
                detail: String::from("not JSON"),
            },
            Rejection::Invalid(vec![violation(Rule::TreeAcyclic, vec![14])]),
        ];
        for rejection in rejections {
            assert!(!rejection.rule_name().is_empty(), "{rejection:?}");
            let rendered = rejection.to_string();
            assert!(!rendered.is_empty());
            assert!(rendered.len() <= Rejection::MAX_MESSAGE_BYTES);
            assert!(
                rendered.contains(&rejection.rule_name()),
                "the message names the rule: {rendered}"
            );
        }
    }

    /// `VER-2`: the message names the version, so a person can be told what
    /// wrote the file.
    #[test]
    fn an_incompatible_version_is_named_in_the_message() {
        let rejection = Rejection::IncompatibleVersion {
            found: FormatVersion { major: 2, minor: 0 },
            known: FormatVersion::CURRENT,
        };
        assert!(rejection.to_string().contains('2'));
        assert!(rejection.offenders().is_empty(), "no entity exists yet");
        assert!(rejection.violations().is_empty());
    }

    /// `VAL-12`: output stays bounded however many rules a file breaks.
    #[test]
    fn a_rejection_naming_many_rules_is_still_bounded() {
        let many: Vec<Violation> = (0..4096)
            .map(|n| violation(Rule::UnmarkedData, vec![n]))
            .collect();
        let rejection = Rejection::Invalid(many);
        let rendered = rejection.to_string();
        assert!(
            rendered.len() <= Rejection::MAX_MESSAGE_BYTES,
            "{} bytes is not bounded output",
            rendered.len()
        );
        assert!(rendered.ends_with('…'), "and says it was cut");
    }

    #[test]
    fn offenders_are_every_identifier_named_once() {
        let rejection = Rejection::Invalid(vec![
            violation(Rule::ReferenceResolves, vec![13, 17]),
            violation(Rule::ReferenceResolves, vec![17]),
        ]);
        assert_eq!(rejection.offenders(), vec![Id(13), Id(17)]);
    }

    #[test]
    fn a_manifest_is_the_format_name_and_two_numbers() {
        let good = br#"{"format":"rigel2d-puppet","version":{"major":1,"minor":1}}"#;
        assert_eq!(
            manifest_version(good).expect("it reads"),
            FormatVersion { major: 1, minor: 1 }
        );

        for bad in [
            &br#"{"version":{"major":1,"minor":1}}"#[..],
            &br#"{"format":"something-else","version":{"major":1,"minor":1}}"#[..],
            &br#"{"format":"rigel2d-puppet"}"#[..],
            &br#"{"format":"rigel2d-puppet","version":{"major":1}}"#[..],
            &br#"{"format":"rigel2d-puppet","version":{"major":-1,"minor":1}}"#[..],
            &b"not json"[..],
        ] {
            assert!(
                manifest_version(bad).is_err(),
                "{:?} must be refused",
                String::from_utf8_lossy(bad)
            );
        }
    }

    /// A header read, not a decode: a signature with rubbish after it is what
    /// `rigged:texture-payload-corrupt` carries, and it must not read as an
    /// image (`VAL-10`).
    #[test]
    fn a_payload_is_an_image_only_if_its_header_says_so() {
        let mut corrupt = b"\x89PNG\r\n\x1a\n".to_vec();
        corrupt.extend_from_slice(b"this is not a PNG beyond the signature");
        assert_eq!(png_dimensions(&corrupt), None);

        assert_eq!(png_dimensions(b""), None);
        assert_eq!(png_dimensions(b"\x89PNG\r\n\x1a\n"), None);
        assert_eq!(png_dimensions(b"GIF89a................"), None);

        let mut real = b"\x89PNG\r\n\x1a\n".to_vec();
        real.extend_from_slice(&[0, 0, 0, 13]);
        real.extend_from_slice(b"IHDR");
        real.extend_from_slice(&24u32.to_be_bytes());
        real.extend_from_slice(&20u32.to_be_bytes());
        real.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
        real.extend_from_slice(b"IEND\xae\x42\x60\x82");
        assert_eq!(png_dimensions(&real), Some([24, 20]));

        // A payload cut short before its end chunk is not an image either.
        let truncated = &real[..real.len() - 4];
        assert_eq!(png_dimensions(truncated), None);
    }
}
