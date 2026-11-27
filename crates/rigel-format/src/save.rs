//! Writing a puppet file: canonical bytes, through one atomic path.

use std::path::{Path, PathBuf};

use crate::puppet::Puppet;
use crate::validate::Violation;

/// What a successful save produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveOutcome {
    /// The file written.
    pub path: PathBuf,
    /// The sibling holding the pre-upgrade bytes, when the file being replaced
    /// declared a lower minor version. Named by appending
    /// `.bak-<major>.<minor>` — the version the backup **holds** (`VER-4`,
    /// `docs/format.md`).
    ///
    /// `None` for every save that is not an upgrade, which is what lets `FILE-2`
    /// assert the directory afterwards holds the puppet and nothing else.
    pub backup: Option<PathBuf>,
}

/// Why a save was refused.
#[derive(Clone, Debug, PartialEq)]
pub enum SaveError {
    /// The validator rejected the puppet. The **same** validator that gates a
    /// load: a save never writes a puppet a load would refuse (`VAL-13`,
    /// `EDIT-1`).
    Invalid(Vec<Violation>),
    /// The write failed. The previous file is intact
    /// ([I7](../../../docs/invariants.md)).
    Io(String),
}

/// One step of the write, for the failure injection `VER-5` and
/// [I7](../../../docs/invariants.md) require.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveStep {
    /// Writing the version-upgrade backup sibling.
    WriteBackup,
    /// Writing the new bytes to a temporary in the destination directory.
    WriteTemporary,
    /// Flushing that temporary to the filesystem.
    FlushTemporary,
    /// Renaming it over the destination.
    Rename,
}

impl SaveStep {
    /// Every step a save goes through. `VER-5` injects a failure at each, and a
    /// step added later is covered by being added here.
    pub const ALL: &'static [SaveStep] = &[
        SaveStep::WriteBackup,
        SaveStep::WriteTemporary,
        SaveStep::FlushTemporary,
        SaveStep::Rename,
    ];
}

/// Writes `puppet` to `path`, canonically and atomically.
///
/// Canonical means the same puppet produces the same bytes: JSON keys sorted,
/// collections in their stated order, numbers in one format, and every zip
/// default pinned — entry order, modification times, compression per entry kind,
/// external attributes (`CANON-1`, `CANON-3`, `docs/format.md`). Nothing about
/// the machine that wrote it reaches the file: no username, no hostname, no path
/// (`CANON-2`).
///
/// Material this build does not understand is written back **verbatim**,
/// positioned by its marker. Re-emitting a stranger's values through our own
/// number formatter would damage values we by definition do not understand
/// (`EXT-4`, `VER-1`).
///
/// Validates first, and refuses rather than writing a puppet a load would reject
/// (`VAL-13`). Atomic: a failure at any step leaves the previous file intact
/// ([I7](../../../docs/invariants.md), `VER-5`).
///
/// When the file being replaced declares a lower minor version, the bytes read
/// are kept as a sibling before anything is replaced (`VER-4`).
pub fn save(_puppet: &Puppet, _path: &Path) -> Result<SaveOutcome, SaveError> {
    unimplemented!()
}

/// Writes `puppet` to `path` as a **different puppet**.
///
/// Issues a fresh identifier for the puppet itself before writing, because the
/// identifier identifies the puppet rather than the file: copying a file is not
/// a new puppet, and deliberately making a different one is (`ID-9`).
pub fn save_as(_puppet: &mut Puppet, _path: &Path) -> Result<SaveOutcome, SaveError> {
    unimplemented!()
}

/// [`save`], failing deliberately at one step.
///
/// Exists for `VER-5` and [I7](../../../docs/invariants.md): *a failed save never
/// damages the previous file* is a claim about every step of the write, and the
/// only way to assert it is to make each step fail. Production code never calls
/// this; it is part of the public surface because the acceptance suite is a
/// separate crate target and I7 is not a thing to assert by inspection.
pub fn save_with_injected_failure(
    _puppet: &Puppet,
    _path: &Path,
    _fail_at: SaveStep,
) -> Result<SaveOutcome, SaveError> {
    unimplemented!()
}
