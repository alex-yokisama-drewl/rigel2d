//! Reading a puppet file, treating it as hostile input throughout.

use std::fmt;
use std::path::Path;

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
        unimplemented!()
    }

    /// Every identifier this rejection names. Empty only for a container,
    /// version or JSON fault, which is refused before any entity exists.
    pub fn offenders(&self) -> Vec<Id> {
        unimplemented!()
    }

    /// Every rule violation this rejection carries.
    ///
    /// Empty for a container, version or JSON fault, which is refused before any
    /// rule is reached — which is exactly why `rigged:nan`, carrying a bare `NaN`
    /// token that is not strict JSON, may be refused without naming an
    /// identifier.
    pub fn violations(&self) -> &[Violation] {
        unimplemented!()
    }
}

impl fmt::Display for Rejection {
    /// At most [`Rejection::MAX_MESSAGE_BYTES`], always naming the rule.
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
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

/// Reads the declared version, and nothing else.
///
/// `rigel2d.json` exists so that deciding whether to refuse a file costs one
/// small bounded read, **before any load limit is at risk and before the manifest
/// is parsed** (`docs/format.md`). A caller deciding whether to open a file at
/// all uses this; [`load`] performs the same check for itself.
pub fn read_version(_path: &Path) -> Result<FormatVersion, Rejection> {
    unimplemented!()
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
pub fn load(_path: &Path) -> Result<Loaded, Rejection> {
    unimplemented!()
}
