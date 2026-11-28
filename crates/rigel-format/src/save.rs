//! Writing a puppet file: canonical bytes, through one atomic path.
//!
//! # The order the three jobs happen in
//!
//! **Canonicalise, then validate, then write**, and the first two are the way
//! round that is easy to get wrong.
//!
//! `docs/format.md` states ordering twice — its *Validation* list requires a
//! parameter's keypoints ordered, and its *Canonical serialisation* section
//! makes ordering part of the canonical form. The first reads as a rule about a
//! value and the second is a rule about **writing**, and `ID-3` is what settles
//! which: it reverses a puppet's nodes, parameters, textures, bindings and every
//! parameter's keypoints *in memory*, then expects the save to succeed and the
//! reload to come back canonically ordered. That holds only if the writer puts
//! the puppet in canonical order **before** handing it to the validator. See
//! `docs/wip/s2-format.md`, task 6's *Traps*.
//!
//! # Where the atomicity comes from
//!
//! [I7](../../../docs/invariants.md) — *a failed save never damages the previous
//! file* — is structural here rather than careful: the whole archive is built in
//! memory, written to a temporary **in the destination directory**, flushed, and
//! only then renamed over the destination. Nothing truncates or appends to the
//! file being replaced, so there is no window in which it is half a puppet.
//! [`SaveStep`] names each step and [`save_with_injected_failure`] fails at one,
//! which is how `VER-5` asserts it rather than inspecting it.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::json::Json;
use crate::puppet::{
    Animation, Binding, BindingValues, ChannelMapping, CompositionMode, Extensions, Id, Mesh, Node,
    NodeBody, OutputSource, Parameter, Part, PhysicsNode, Preserved, Puppet, TextureFlip,
    TextureRecord, Track, Transform,
};
use crate::validate::Violation;
use crate::version::{FORMAT_NAME, FormatVersion};

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

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Invalid(violations) => {
                write!(f, "refused: the puppet is invalid")?;
                for broken in violations {
                    write!(f, "\n  {}: {}", broken.rule.as_str(), broken.detail)?;
                }
                Ok(())
            }
            SaveError::Io(detail) => write!(f, "refused: {detail}"),
        }
    }
}

impl std::error::Error for SaveError {}

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
    ///
    /// This is also the order they happen in: the previous bytes are secured
    /// before anything else begins, because an upgrade is the one save that
    /// cannot be undone by saving again.
    pub const ALL: &'static [SaveStep] = &[
        SaveStep::WriteBackup,
        SaveStep::WriteTemporary,
        SaveStep::FlushTemporary,
        SaveStep::Rename,
    ];

    fn as_str(self) -> &'static str {
        match self {
            SaveStep::WriteBackup => "writing the version-upgrade backup",
            SaveStep::WriteTemporary => "writing the temporary",
            SaveStep::FlushTemporary => "flushing the temporary",
            SaveStep::Rename => "renaming the temporary over the destination",
        }
    }
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
pub fn save(puppet: &Puppet, path: &Path) -> Result<SaveOutcome, SaveError> {
    write_file(puppet, path, None)
}

/// Writes `puppet` to `path` as a **different puppet**.
///
/// Issues a fresh identifier for the puppet itself before writing, because the
/// identifier identifies the puppet rather than the file: copying a file is not
/// a new puppet, and deliberately making a different one is (`ID-9`).
pub fn save_as(puppet: &mut Puppet, path: &Path) -> Result<SaveOutcome, SaveError> {
    puppet.id = puppet.issue_identifier();
    write_file(puppet, path, None)
}

/// [`save`], failing deliberately at one step.
///
/// Exists for `VER-5` and [I7](../../../docs/invariants.md): *a failed save never
/// damages the previous file* is a claim about every step of the write, and the
/// only way to assert it is to make each step fail. Production code never calls
/// this; it is part of the public surface because the acceptance suite is a
/// separate crate target and I7 is not a thing to assert by inspection.
pub fn save_with_injected_failure(
    puppet: &Puppet,
    path: &Path,
    fail_at: SaveStep,
) -> Result<SaveOutcome, SaveError> {
    write_file(puppet, path, Some(fail_at))
}

// --------------------------------------------------------------- the write

fn io(detail: impl Into<String>) -> SaveError {
    SaveError::Io(detail.into())
}

/// The one write path. [`save`], [`save_as`] and [`save_with_injected_failure`]
/// differ only in what they do to the puppet first and where they fail.
fn write_file(
    puppet: &Puppet,
    path: &Path,
    fail_at: Option<SaveStep>,
) -> Result<SaveOutcome, SaveError> {
    // Canonical order first, so that the validator sees the puppet as it will
    // be written rather than as the caller happened to assemble it. See the
    // module header.
    let canonical = canonicalise(puppet);
    crate::validate::validate(&canonical).map_err(SaveError::Invalid)?;

    let version = canonical.declared_version.unwrap_or(FormatVersion::CURRENT);
    let bytes = archive(&canonical, version)?;

    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let upgrading = upgrade_from(path, version);

    // Step 1: the previous bytes, before anything else begins.
    let mut backup: Option<PathBuf> = None;
    let mut backup_is_ours = false;
    if fail_at == Some(SaveStep::WriteBackup) {
        return Err(injected(SaveStep::WriteBackup));
    }
    if let Some(held) = upgrading {
        let sibling = backup_path(path, held);
        // **An existing backup is never replaced.** The backup exists because
        // an upgrade is the one save that saving again cannot undo, so
        // overwriting an older one takes away the thing it was kept for. The
        // save continues: the puppet is still written, and the backup already
        // on disk still holds a readable file at the version its name claims.
        if sibling.exists() {
            backup = Some(sibling);
        } else {
            backup_is_ours = true;
            let previous =
                fs::read(path).map_err(|e| io(format!("{path:?} is not readable: {e}")))?;
            // Through a temporary of its own, so that a half-written backup
            // never stands in for the file it exists to preserve.
            put(directory, &sibling, &previous, None)?;
            backup = Some(sibling);
        }
    }

    // Steps 2 to 4: the file itself. Nothing touches `path` until the new bytes
    // are complete on disk, which is where [I7] comes from.
    let undo = |error: SaveError| -> SaveError {
        if backup_is_ours && let Some(sibling) = &backup {
            let _ = fs::remove_file(sibling);
        }
        error
    };
    match put(directory, path, &bytes, fail_at) {
        Ok(()) => Ok(SaveOutcome {
            path: path.to_path_buf(),
            backup,
        }),
        Err(error) => Err(undo(error)),
    }
}

fn injected(step: SaveStep) -> SaveError {
    io(format!("a failure was injected while {}", step.as_str()))
}

/// Writes `bytes` to `target` without the destination ever holding a partial
/// file: a temporary beside it, flushed, then renamed over.
///
/// `fail_at` is the acceptance suite's injection point and is `None` everywhere
/// else; a step it names is not performed, and the temporary is removed.
fn put(
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    fail_at: Option<SaveStep>,
) -> Result<(), SaveError> {
    let temporary = temporary_beside(directory, target);
    let clean = |error: SaveError| -> SaveError {
        let _ = fs::remove_file(&temporary);
        error
    };

    if fail_at == Some(SaveStep::WriteTemporary) {
        return Err(clean(injected(SaveStep::WriteTemporary)));
    }
    // **Exclusive create, not `File::create`.** `create` truncates whatever is
    // at the path and follows a symlink to wherever it points, so a name
    // already taken — by a leftover from a killed save, or by something
    // planted in a directory the user chose — would be written through.
    // `create_new` refuses instead, which is also what keeps a save touching
    // nothing outside the file's own directory.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| clean(io(format!("{temporary:?} is not writable: {e}"))))?;
    file.write_all(bytes)
        .map_err(|e| clean(io(format!("{temporary:?} is not written: {e}"))))?;

    if fail_at == Some(SaveStep::FlushTemporary) {
        drop(file);
        return Err(clean(injected(SaveStep::FlushTemporary)));
    }
    file.sync_all()
        .map_err(|e| clean(io(format!("{temporary:?} is not flushed: {e}"))))?;
    drop(file);

    if fail_at == Some(SaveStep::Rename) {
        return Err(clean(injected(SaveStep::Rename)));
    }
    fs::rename(&temporary, target).map_err(|e| {
        clean(io(format!(
            "{temporary:?} is not renamed over {target:?}: {e}"
        )))
    })?;
    Ok(())
}

/// A name no puppet file has, in the directory the destination sits in — the
/// same filesystem, because a rename across one is not atomic.
///
/// Removed by the rename on the way out, which is what `FILE-2` asserts: a save
/// leaves the puppet and nothing else.
fn temporary_beside(directory: &Path, target: &Path) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let stem = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("puppet"));
    directory.join(format!(".{stem}.saving-{}-{n}", std::process::id()))
}

/// The version the file being replaced declares, when replacing it is an
/// **upgrade** — the same major, a lower minor (`docs/format.md`, *The upgrading
/// save keeps the previous bytes*).
///
/// A destination that is not there, is not a puppet, or declares the version
/// being written is not an upgrade and gets no backup.
fn upgrade_from(path: &Path, writing: FormatVersion) -> Option<FormatVersion> {
    if !path.exists() {
        return None;
    }
    let found = crate::load::read_version(path).ok()?;
    (found.major == writing.major && found.minor < writing.minor).then_some(found)
}

/// The backup's name: the original with `.bak-<major>.<minor>` appended, the
/// version the backup **holds** rather than the one being written (`VER-4`).
fn backup_path(path: &Path, held: FormatVersion) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("puppet"));
    name.push_str(&format!(".bak-{}.{}", held.major, held.minor));
    path.with_file_name(name)
}

// -------------------------------------------------------- canonical order

/// The puppet as it will be written: every collection in the order
/// `docs/format.md`'s *Canonical serialisation* states.
///
/// `nodes` depth-first preorder with siblings in draw order; every collection
/// carrying identifiers ordered by identifier; keypoints by position, **with
/// each binding's values permuted alongside them**, because the stored form
/// indexes a binding's values positionally by keypoint and reordering one
/// without the other is a change of meaning rather than a reorder (`ID-3`).
fn canonicalise(puppet: &Puppet) -> Puppet {
    let mut out = puppet.clone();

    out.textures.sort_by_key(|texture| texture.id);
    out.parameters.sort_by_key(|parameter| parameter.id);
    out.bindings.sort_by_key(|binding| binding.id);
    out.animations.sort_by_key(|animation| animation.id);
    for animation in &mut out.animations {
        animation.tracks.sort_by_key(|track| track.id);
    }
    out.channel_mappings.sort_by_key(|mapping| mapping.id);

    // Keypoints by position, and the values stored against them with them.
    let mut permutations: Vec<(Id, Vec<usize>)> = Vec::new();
    for parameter in &mut out.parameters {
        let order = sorting_permutation(parameter);
        if order.iter().enumerate().all(|(at, from)| at == *from) {
            continue;
        }
        parameter.keypoints = order
            .iter()
            .map(|from| parameter.keypoints[*from].clone())
            .collect();
        permutations.push((parameter.id, order));
    }
    for (parameter, order) in &permutations {
        for binding in &mut out.bindings {
            if binding.parameter != *parameter || binding.values.len() != order.len() {
                continue;
            }
            binding.values = permute(&binding.values, order);
            repath(&mut out.preserved, binding.id, order);
        }
    }

    // The tree, and with it the opaque entries that sit in it.
    let order = node_order(&out);
    let nodes = std::mem::take(&mut out.nodes);
    let opaque = std::mem::take(&mut out.preserved.opaque_nodes);
    for slot in &order {
        match slot {
            Slot::Typed(at) => out.nodes.push(nodes[*at].clone()),
            Slot::Opaque(at) => out.preserved.opaque_nodes.push(opaque[*at].clone()),
        }
    }

    out.extensions
        .entries
        .sort_by(|a, b| a.namespace.cmp(&b.namespace));
    for node in &mut out.nodes {
        node.extensions
            .entries
            .sort_by(|a, b| a.namespace.cmp(&b.namespace));
    }
    out.preserved
        .members
        .sort_by(|a, b| a.owner.cmp(&b.owner).then_with(|| a.path.cmp(&b.path)));
    out
}

/// The indices of a parameter's keypoints in position order.
///
/// Lexicographic over the position vector, which is the order the corpus
/// carries and the one [`crate::validate`] enforces — a parameter may declare
/// two axes, so *along their axis* names no order for a two-dimensional one.
/// Stable, so two keypoints at one position keep the order they were in; that
/// pair is invalid and the validator is what says so.
fn sorting_permutation(parameter: &Parameter) -> Vec<usize> {
    let mut order: Vec<usize> = (0..parameter.keypoints.len()).collect();
    order.sort_by(|a, b| {
        let (left, right) = (
            &parameter.keypoints[*a].position,
            &parameter.keypoints[*b].position,
        );
        for (held, next) in left.iter().zip(right) {
            match held.partial_cmp(next) {
                Some(std::cmp::Ordering::Equal) | None => {}
                Some(other) => return other,
            }
        }
        left.len().cmp(&right.len())
    });
    order
}

fn permute(values: &BindingValues, order: &[usize]) -> BindingValues {
    match values {
        BindingValues::Transform(held) => {
            BindingValues::Transform(order.iter().map(|from| held[*from]).collect())
        }
        BindingValues::Deformation(held) => {
            BindingValues::Deformation(order.iter().map(|from| held[*from].clone()).collect())
        }
    }
}

/// Moves a binding's preserved members with the values they sit on.
///
/// A later-version file's unknown member on one of a transform binding's values
/// is located at `["values", "<position>"]`, which is the one preserved path a
/// keypoint reorder invalidates — everything else is keyed on an identifier and
/// survives a reorder untouched.
fn repath(preserved: &mut Preserved, binding: Id, order: &[usize]) {
    // `order[at]` is where the value at `at` came *from*, so the inverse is
    // what rewrites an old position into a new one.
    let mut moved_to: BTreeMap<String, usize> = BTreeMap::new();
    for (at, from) in order.iter().enumerate() {
        moved_to.insert(from.to_string(), at);
    }
    for row in &mut preserved.members {
        if row.owner != binding || row.path.len() != 2 || row.path[0] != "values" {
            continue;
        }
        if let Some(at) = moved_to.get(&row.path[1]) {
            row.path[1] = at.to_string();
        }
    }
}

/// One entry of the stored `nodes` array: a node this build reads, or one of a
/// kind it does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Typed(usize),
    Opaque(usize),
}

/// The stored order of `nodes`: depth-first preorder, siblings in draw order.
///
/// Opaque entries sit in it too — the writer has to put a node of a kind it
/// cannot read back in the tree, which is why [`crate::OpaqueNode`] carries the
/// `id` and `parent` it declares and nothing else. A parent's typed children
/// come before its opaque ones, because the two arrive in separate collections
/// and their interleaving is not recorded.
///
/// Tolerant of a tree that is not one: this runs **before** the validator, so a
/// cycle or a second root must come out as some order rather than as a hang.
fn node_order(puppet: &Puppet) -> Vec<Slot> {
    let opaque = &puppet.preserved.opaque_nodes;
    let mut children: BTreeMap<Id, Vec<Slot>> = BTreeMap::new();
    for (at, node) in puppet.nodes.iter().enumerate() {
        children
            .entry(node.parent)
            .or_default()
            .push(Slot::Typed(at));
    }
    for (at, node) in opaque.iter().enumerate() {
        children
            .entry(node.parent)
            .or_default()
            .push(Slot::Opaque(at));
    }

    let mut seen_typed = vec![false; puppet.nodes.len()];
    let mut seen_opaque = vec![false; opaque.len()];
    let mut out: Vec<Slot> = Vec::with_capacity(puppet.nodes.len() + opaque.len());
    let mut stack: Vec<Slot> = children.get(&Id::NONE).cloned().unwrap_or_default();
    stack.reverse();
    while let Some(slot) = stack.pop() {
        let id = match slot {
            Slot::Typed(at) => {
                if std::mem::replace(&mut seen_typed[at], true) {
                    continue;
                }
                puppet.nodes[at].id
            }
            Slot::Opaque(at) => {
                if std::mem::replace(&mut seen_opaque[at], true) {
                    continue;
                }
                opaque[at].id
            }
        };
        out.push(slot);
        if let Some(below) = children.get(&id) {
            for child in below.iter().rev() {
                stack.push(*child);
            }
        }
    }
    // Anything the walk did not reach — a second root, a cycle, a parent that
    // resolves to nothing. All three are rules the validator names, and all
    // three still have to be written somewhere for it to name them.
    for (at, seen) in seen_typed.iter().enumerate() {
        if !seen {
            out.push(Slot::Typed(at));
        }
    }
    for (at, seen) in seen_opaque.iter().enumerate() {
        if !seen {
            out.push(Slot::Opaque(at));
        }
    }
    out
}

// --------------------------------------------------------- the archive

/// The default a field is absent at, per `docs/format.md`.
const DEFAULT_GRAVITY: f64 = 1.0;
const DEFAULT_LENGTH: f64 = 100.0;
const DEFAULT_FREQUENCY: f64 = 1.0;
const DEFAULT_DAMPING: f64 = 0.5;

/// Where a texture payload's entry name begins and ends.
const TEXTURE_PREFIX: &str = "textures/";
const TEXTURE_SUFFIX: &str = ".png";

/// The whole archive, in memory.
///
/// Built before anything is written, so the one thing the filesystem sees is a
/// complete file ([I7](../../../docs/invariants.md)).
fn archive(puppet: &Puppet, version: FormatVersion) -> Result<Vec<u8>, SaveError> {
    let mut entries: Vec<(String, Vec<u8>)> = vec![
        (
            String::from("puppet.json"),
            Writer { puppet }.document().to_canonical_bytes(),
        ),
        (String::from("rigel2d.json"), manifest(version)),
    ];
    for texture in &puppet.textures {
        entries.push((
            format!("{TEXTURE_PREFIX}{}{TEXTURE_SUFFIX}", texture.id.0),
            texture.payload.clone(),
        ));
    }
    // Entry order fixed, sorted by path.
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // The pinned modification time: the earliest a zip date field expresses, so
    // there is no question of it having come from a clock.
    let fixed = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
        .map_err(|e| io(format!("1980-01-01 is not a zip date: {e}")))?;

    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        for (name, data) in &entries {
            // Compression fixed per entry kind: a PNG is already deflated inside
            // its own container, and storing it keeps the payload extractable in
            // one step, which is what `TOOL-3` rests on. The deflate level is
            // whatever the locked dependency set defaults to, which is what
            // fixes it.
            let method = if name.ends_with(TEXTURE_SUFFIX) {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            };
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(method)
                .last_modified_time(fixed)
                .large_file(false)
                // Pinned, so no umask of the writing machine reaches the file.
                .unix_permissions(0o644);
            writer
                .start_file(name.clone(), options)
                .map_err(|e| io(format!("{name} is not started: {e}")))?;
            writer
                .write_all(data)
                .map_err(|e| io(format!("{name} is not written: {e}")))?;
        }
        writer
            .finish()
            .map_err(|e| io(format!("the archive is not finished: {e}")))?;
    }
    Ok(buffer.into_inner())
}

/// `rigel2d.json`: the format name and the version, and nothing else.
fn manifest(version: FormatVersion) -> Vec<u8> {
    let mut numbers = Object::new();
    numbers.integer("major", u64::from(version.major));
    numbers.integer("minor", u64::from(version.minor));
    let mut out = Object::new();
    out.text("format", FORMAT_NAME);
    out.put("version", numbers.into_json());
    out.into_json().to_canonical_bytes()
}

// ------------------------------------------------------------ the document

/// A JSON object under construction.
///
/// Members go in in whatever order the code finds them and come out sorted:
/// [`Json::write_canonical`] is the one place the canonical member order lives.
struct Object(Vec<(String, Json)>);

impl Object {
    fn new() -> Self {
        Object(Vec::new())
    }

    fn put(&mut self, key: &str, value: Json) {
        self.0.push((key.to_owned(), value));
    }

    fn number(&mut self, key: &str, value: f64) {
        self.put(key, number(value));
    }

    fn integer(&mut self, key: &str, value: u64) {
        self.put(key, Json::Number(value.to_string()));
    }

    fn text(&mut self, key: &str, value: &str) {
        self.put(key, Json::String(value.to_owned()));
    }

    /// A reference, written only when it names something. The reserved zero
    /// means *no reference*, and an absent member reads back as it.
    fn reference(&mut self, key: &str, id: Id) {
        if id != Id::NONE {
            self.integer(key, id.0);
        }
    }

    fn into_json(self) -> Json {
        Json::Object(self.0)
    }
}

/// One number, in the one format the whole file uses.
///
/// The shortest decimal that reads back as the same `f64`, which is what the
/// corpus was authored with — a unit test below holds the two against each
/// other, because a writer that disagrees with the specification's own corpus
/// about how to spell a number produces a file that churns for no reason.
///
/// Only ever reached for a finite value: [`crate::validate`] runs first and
/// [`crate::Rule::NumbersFinite`] refuses every number this build interprets
/// that is not one.
fn number(value: f64) -> Json {
    Json::Number(if value.is_finite() {
        decimal(value)
    } else {
        // Unreachable: the save was validated. JSON cannot spell a non-finite
        // number, so there is nothing better to write than a finite one.
        debug_assert!(false, "a non-finite number reached the writer");
        String::from("0.0")
    })
}

/// The shortest decimal that reads back as the same `f64`, spelled the way the
/// corpus spells it.
///
/// Rust's own shortest-round-trip formatter and the one `rigel-fixtures` writes
/// through differ in exactly one place: a positive exponent carries a `+` there
/// and no sign here. Everything else — where the exponential form begins, how
/// many digits it carries, the trailing `.0` on an integral value — is the same
/// shortest representation, and the unit test below holds the two against each
/// other over the corpus's values, the edges of the type and a seeded sample.
fn decimal(value: f64) -> String {
    let rendered = format!("{value:?}");
    match rendered.split_once('e') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}e+{exponent}")
        }
        _ => rendered,
    }
}

/// Material kept as the bytes it arrived as.
fn raw(bytes: &[u8]) -> Json {
    Json::Raw(String::from_utf8_lossy(bytes).into_owned())
}

fn pair(value: [f64; 2]) -> Json {
    Json::Array(value.iter().map(|held| number(*held)).collect())
}

fn pairs(values: &[[f64; 2]]) -> Json {
    Json::Array(values.iter().map(|held| pair(*held)).collect())
}

fn composition_mode(mode: &CompositionMode) -> &str {
    match mode {
        CompositionMode::Sum => "sum",
        CompositionMode::Unfamiliar(token) => token,
    }
}

fn output_source(source: &OutputSource) -> &str {
    match source {
        OutputSource::Angle => "angle",
        OutputSource::Length => "length",
        OutputSource::X => "x",
        OutputSource::Y => "y",
        OutputSource::Unfamiliar(token) => token,
    }
}

fn axis_polarity(polarity: &crate::puppet::AxisPolarity) -> &str {
    use crate::puppet::AxisPolarity;
    match polarity {
        AxisPolarity::Bilateral => "bilateral",
        AxisPolarity::Unipolar => "unipolar",
        // A value the format defines, written rather than left to be inferred
        // from an absence (`VAL-21`).
        AxisPolarity::Unknown => "unknown",
        AxisPolarity::Unfamiliar(token) => token,
    }
}

/// `puppet.json`, from a puppet already in canonical order.
struct Writer<'a> {
    puppet: &'a Puppet,
}

impl Writer<'_> {
    fn document(&self) -> Json {
        let puppet = self.puppet;
        let mut out = Object::new();
        out.integer("id", puppet.id.0);
        out.text("name", &puppet.name);
        out.integer("high_water_mark", puppet.high_water_mark);
        out.integer("root", puppet.root.0);
        // Absent when it holds its default, which is the whole of the
        // stored-form rule (`PHYS-10`).
        if puppet.gravity != DEFAULT_GRAVITY {
            out.number("gravity", puppet.gravity);
        }
        // Written even holding the only value the format defines, which is the
        // single stated exception to that rule (`DET-6`).
        out.text(
            "composition_mode",
            composition_mode(&puppet.composition_mode),
        );

        let order = node_order(puppet);
        if !order.is_empty() {
            out.put(
                "nodes",
                Json::Array(order.iter().map(|slot| self.slot(*slot)).collect()),
            );
        }
        if !puppet.parameters.is_empty() {
            out.put(
                "parameters",
                Json::Array(
                    puppet
                        .parameters
                        .iter()
                        .map(|parameter| self.parameter(parameter))
                        .collect(),
                ),
            );
        }
        if !puppet.bindings.is_empty() {
            out.put(
                "bindings",
                Json::Array(
                    puppet
                        .bindings
                        .iter()
                        .map(|binding| self.binding(binding))
                        .collect(),
                ),
            );
        }
        if !puppet.animations.is_empty() {
            out.put(
                "animations",
                Json::Array(
                    puppet
                        .animations
                        .iter()
                        .map(|animation| self.animation(animation))
                        .collect(),
                ),
            );
        }
        if !puppet.channel_mappings.is_empty() {
            out.put(
                "channel_mappings",
                Json::Array(
                    puppet
                        .channel_mappings
                        .iter()
                        .map(|mapping| self.mapping(mapping))
                        .collect(),
                ),
            );
        }
        if !puppet.textures.is_empty() {
            out.put(
                "textures",
                Json::Array(
                    puppet
                        .textures
                        .iter()
                        .map(|texture| self.texture(texture))
                        .collect(),
                ),
            );
        }
        self.extensions(&mut out, puppet.id, &[], &puppet.extensions);
        self.preserve(&mut out, puppet.id, &[]);
        out.into_json()
    }

    // ---------------------------------------------------- preservation

    /// Every member this build has no typed slot for, back where its marker
    /// puts it.
    ///
    /// Contents verbatim, position canonical — the pair `EXT-4` asserts. Each
    /// value is the text it arrived as, and where it is written is set by the
    /// canonical member order rather than by where it was found.
    ///
    /// Reads **every** row matching the owner and path rather than the first:
    /// an object can carry both an `x` marker the reader had nowhere typed to
    /// put and unmarked members a later version licensed, and those arrive as
    /// two rows.
    fn preserve(&self, object: &mut Object, owner: Id, path: &[&str]) {
        for row in &self.puppet.preserved.members {
            if row.owner != owner || row.path.len() != path.len() {
                continue;
            }
            if !row.path.iter().zip(path).all(|(held, want)| held == want) {
                continue;
            }
            for (name, text) in &row.members {
                object.put(name, raw(text));
            }
        }
    }

    /// Whether anything was preserved at one place, which is what keeps an
    /// object that would otherwise be omitted from taking its contents with it.
    fn preserved_at(&self, owner: Id, path: &[&str]) -> bool {
        self.puppet.preserved.members.iter().any(|row| {
            row.owner == owner
                && row.path.len() == path.len()
                && row.path.iter().zip(path).all(|(held, want)| held == want)
        })
    }

    /// The `x` marker: namespaced values the core format does not define,
    /// ordered by namespace, which is the canonical position — keyed on the
    /// marker, which a build can read without understanding the contents.
    fn extensions(&self, object: &mut Object, owner: Id, path: &[&str], held: &Extensions) {
        if held.entries.is_empty() {
            return;
        }
        let mut marker = Object::new();
        for entry in &held.entries {
            let mut body = Object::new();
            body.text("name", &entry.name);
            body.text("version", &entry.version);
            body.put("data", raw(&entry.data));
            let mut inside: Vec<&str> = path.to_vec();
            inside.push("x");
            inside.push(&entry.namespace);
            self.preserve(&mut body, owner, &inside);
            marker.put(&entry.namespace, body.into_json());
        }
        object.put("x", marker.into_json());
    }

    // ----------------------------------------------------------- nodes

    fn slot(&self, slot: Slot) -> Json {
        match slot {
            Slot::Typed(at) => self.node(&self.puppet.nodes[at]),
            // A node of a kind this build does not know, back exactly as it
            // arrived and placed by the `id` and `parent` it declares (`EXT-7`).
            Slot::Opaque(at) => raw(&self.puppet.preserved.opaque_nodes[at].source),
        }
    }

    fn node(&self, node: &Node) -> Json {
        let mut out = Object::new();
        out.integer("id", node.id.0);
        // Written even for the root, whose parent is the reserved zero: the
        // tree is a flat array and `parent` is how it is read back.
        out.integer("parent", node.parent.0);
        out.text("name", &node.name);
        out.put(
            "transform",
            self.transform(node.id, &["transform"], &node.transform),
        );
        match &node.body {
            NodeBody::Composite => out.text("kind", "composite"),
            NodeBody::MeshGroup(group) => {
                out.text("kind", "mesh_group");
                self.mesh(&mut out, node.id, &group.mesh);
            }
            NodeBody::Part(part) => {
                out.text("kind", "part");
                self.part(&mut out, node.id, part);
            }
            NodeBody::Physics(physics) => {
                out.text("kind", "physics");
                self.physics(&mut out, physics);
            }
        }
        self.extensions(&mut out, node.id, &[], &node.extensions);
        self.preserve(&mut out, node.id, &[]);
        out.into_json()
    }

    fn part(&self, out: &mut Object, owner: Id, part: &Part) {
        self.mesh(out, owner, &part.mesh);
        out.integer("texture", part.texture.0);
        out.reference("mask", part.mask);
        if part.texture_flip != TextureFlip::default()
            || self.preserved_at(owner, &["texture_flip"])
        {
            let mut flip = Object::new();
            flip.put("left_right", Json::Bool(part.texture_flip.left_right));
            flip.put("top_bottom", Json::Bool(part.texture_flip.top_bottom));
            self.preserve(&mut flip, owner, &["texture_flip"]);
            out.put("texture_flip", flip.into_json());
        }
        if let Some(provenance) = &part.provenance {
            let mut record = Object::new();
            record.text("source_id", &provenance.source_id);
            record.text("content_hash", &provenance.content_hash);
            self.preserve(&mut record, owner, &["provenance"]);
            out.put("provenance", record.into_json());
        }
        if let Some(chain) = &part.chain_deformer {
            let mut deformer = Object::new();
            deformer.integer("id", chain.id.0);
            deformer.put(
                "joints",
                Json::Array(
                    chain
                        .joints
                        .iter()
                        .map(|joint| {
                            let mut held = Object::new();
                            held.integer("id", joint.id.0);
                            held.put("position", pair(joint.position));
                            self.preserve(&mut held, joint.id, &[]);
                            held.into_json()
                        })
                        .collect(),
                ),
            );
            self.preserve(&mut deformer, chain.id, &[]);
            out.put("chain_deformer", deformer.into_json());
        }
    }

    /// Every constant absent when it holds the default `docs/format.md`'s *A
    /// physics node* table gives it, and nothing simulated ever (`PHYS-8`).
    fn physics(&self, out: &mut Object, physics: &PhysicsNode) {
        if physics.length != DEFAULT_LENGTH {
            out.number("length", physics.length);
        }
        if physics.gravity != DEFAULT_GRAVITY {
            out.number("gravity", physics.gravity);
        }
        if physics.frequency != DEFAULT_FREQUENCY {
            out.number("frequency", physics.frequency);
        }
        if physics.angle_damping != DEFAULT_DAMPING {
            out.number("angle_damping", physics.angle_damping);
        }
        if physics.length_damping != DEFAULT_DAMPING {
            out.number("length_damping", physics.length_damping);
        }
        out.reference("output", physics.output);
        if !physics.output_source.is_empty() {
            out.put(
                "output_source",
                Json::Array(
                    physics
                        .output_source
                        .iter()
                        .map(|source| Json::String(output_source(source).to_owned()))
                        .collect(),
                ),
            );
        }
        if !physics.output_scale.is_empty() {
            out.put(
                "output_scale",
                Json::Array(
                    physics
                        .output_scale
                        .iter()
                        .map(|gain| number(*gain))
                        .collect(),
                ),
            );
        }
    }

    /// A node's transform, **always written whole**: the document gives its
    /// rotation and scale no default, so an identity one is written like any
    /// other (`docs/format.md`, *Canonical serialisation*).
    fn transform(&self, owner: Id, path: &[&str], transform: &Transform) -> Json {
        let mut out = Object::new();
        out.put("translation", pair(transform.translation));
        out.number("rotation", transform.rotation);
        out.put("scale", pair(transform.scale));
        self.preserve(&mut out, owner, path);
        out.into_json()
    }

    /// The geometry a part or a mesh group carries. **Absent is an empty mesh**:
    /// a mesh group that deforms nothing yet carries none, which is what
    /// `rigged:containment-permitted` is built from.
    fn mesh(&self, out: &mut Object, owner: Id, mesh: &Mesh) {
        if mesh.vertices.is_empty()
            && mesh.uvs.is_empty()
            && mesh.triangles.is_empty()
            && !self.preserved_at(owner, &["mesh"])
        {
            return;
        }
        let mut held = Object::new();
        held.put("vertices", pairs(&mesh.vertices));
        held.put("uvs", pairs(&mesh.uvs));
        held.put(
            "triangles",
            Json::Array(
                mesh.triangles
                    .iter()
                    .map(|corners| {
                        Json::Array(
                            corners
                                .iter()
                                .map(|corner| Json::Number(corner.to_string()))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        self.preserve(&mut held, owner, &["mesh"]);
        out.put("mesh", held.into_json());
    }

    // ------------------------------------------------------ parameters

    fn parameter(&self, parameter: &Parameter) -> Json {
        let mut out = Object::new();
        out.integer("id", parameter.id.0);
        out.text("name", &parameter.name);
        out.put(
            "axes",
            Json::Array(
                parameter
                    .axes
                    .iter()
                    .enumerate()
                    .map(|(at, axis)| {
                        let mut held = Object::new();
                        held.put("range", pair(axis.range));
                        held.text("polarity", axis_polarity(&axis.polarity));
                        let step = at.to_string();
                        self.preserve(&mut held, parameter.id, &["axes", step.as_str()]);
                        held.into_json()
                    })
                    .collect(),
            ),
        );
        out.put(
            "keypoints",
            Json::Array(
                parameter
                    .keypoints
                    .iter()
                    .map(|keypoint| {
                        let mut held = Object::new();
                        held.integer("id", keypoint.id.0);
                        held.put(
                            "position",
                            Json::Array(
                                keypoint
                                    .position
                                    .iter()
                                    .map(|coordinate| number(*coordinate))
                                    .collect(),
                            ),
                        );
                        if keypoint.neutral {
                            held.put("neutral", Json::Bool(true));
                        }
                        self.preserve(&mut held, keypoint.id, &[]);
                        held.into_json()
                    })
                    .collect(),
            ),
        );
        self.preserve(&mut out, parameter.id, &[]);
        out.into_json()
    }

    // -------------------------------------------------------- bindings

    fn binding(&self, binding: &Binding) -> Json {
        let mut out = Object::new();
        out.integer("id", binding.id.0);
        out.text(
            "kind",
            match binding.kind {
                crate::puppet::BindingKind::Transform => "transform",
                crate::puppet::BindingKind::Deformation => "deformation",
            },
        );
        out.integer("parameter", binding.parameter.0);
        out.integer("target", binding.target.0);
        out.put(
            "values",
            match &binding.values {
                BindingValues::Transform(values) => Json::Array(
                    values
                        .iter()
                        .enumerate()
                        .map(|(at, held)| {
                            let step = at.to_string();
                            self.transform(binding.id, &["values", step.as_str()], held)
                        })
                        .collect(),
                ),
                BindingValues::Deformation(values) => {
                    Json::Array(values.iter().map(|offsets| pairs(offsets)).collect())
                }
            },
        );
        // Written only when set (`docs/format.md`, *Stale bindings are a state*).
        if binding.stale {
            out.put("stale", Json::Bool(true));
        }
        self.preserve(&mut out, binding.id, &[]);
        out.into_json()
    }

    // ------------------------------------------------------ animations

    fn animation(&self, animation: &Animation) -> Json {
        let mut out = Object::new();
        out.integer("id", animation.id.0);
        out.text("name", &animation.name);
        out.put(
            "tracks",
            Json::Array(
                animation
                    .tracks
                    .iter()
                    .map(|track| self.track(track))
                    .collect(),
            ),
        );
        self.preserve(&mut out, animation.id, &[]);
        out.into_json()
    }

    fn track(&self, track: &Track) -> Json {
        let mut out = Object::new();
        out.integer("id", track.id.0);
        out.integer("parameter", track.parameter.0);
        out.number("blend_duration", track.blend_duration);
        out.put(
            "keyframes",
            Json::Array(
                track
                    .keyframes
                    .iter()
                    .enumerate()
                    .map(|(at, keyframe)| {
                        let mut held = Object::new();
                        held.number("time", keyframe.time);
                        held.put(
                            "value",
                            Json::Array(
                                keyframe
                                    .value
                                    .iter()
                                    .map(|coordinate| number(*coordinate))
                                    .collect(),
                            ),
                        );
                        let step = at.to_string();
                        self.preserve(&mut held, track.id, &["keyframes", step.as_str()]);
                        held.into_json()
                    })
                    .collect(),
            ),
        );
        self.preserve(&mut out, track.id, &[]);
        out.into_json()
    }

    // ------------------------------------------------ channel mappings

    fn mapping(&self, mapping: &ChannelMapping) -> Json {
        let mut out = Object::new();
        out.integer("id", mapping.id.0);
        // A channel is named, not identified: it is supplied by a source the
        // puppet has never met ([I3](../../../docs/invariants.md)).
        out.text("channel", &mapping.channel);
        out.integer("parameter", mapping.parameter.0);
        // The document gives `source` no default, so it is written whether or
        // not it is empty — which it is in almost every puppet.
        out.text("source", &mapping.source);
        let mut curve = Object::new();
        curve.put("points", pairs(&mapping.response_curve.points));
        self.preserve(&mut curve, mapping.id, &["response_curve"]);
        out.put("response_curve", curve.into_json());
        self.preserve(&mut out, mapping.id, &[]);
        out.into_json()
    }

    // --------------------------------------------------------- textures

    fn texture(&self, texture: &TextureRecord) -> Json {
        let mut out = Object::new();
        out.integer("id", texture.id.0);
        out.text("name", &texture.name);
        out.put(
            "authored_size",
            Json::Array(
                texture
                    .authored_size
                    .iter()
                    .map(|pixels| Json::Number(pixels.to_string()))
                    .collect(),
            ),
        );
        out.put(
            "crop",
            Json::Array(
                texture
                    .crop
                    .iter()
                    .map(|pixels| Json::Number(pixels.to_string()))
                    .collect(),
            ),
        );
        self.preserve(&mut out, texture.id, &[]);
        out.into_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puppet::{Axis, AxisPolarity, BindingKind, Keypoint, MeshGroup};

    fn transform() -> Transform {
        Transform {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }

    fn empty_mesh() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            uvs: Vec::new(),
            triangles: Vec::new(),
        }
    }

    fn node(id: u64, parent: u64) -> Node {
        Node {
            id: Id(id),
            parent: Id(parent),
            name: format!("n{id}"),
            transform: transform(),
            body: NodeBody::MeshGroup(MeshGroup { mesh: empty_mesh() }),
            extensions: Extensions::default(),
        }
    }

    fn subject() -> Puppet {
        Puppet {
            id: Id(1),
            name: String::from("subject"),
            high_water_mark: 100,
            root: Id(10),
            gravity: 1.0,
            composition_mode: CompositionMode::Sum,
            declared_version: None,
            nodes: vec![node(10, 0), node(11, 10), node(12, 11), node(13, 10)],
            parameters: Vec::new(),
            bindings: Vec::new(),
            animations: Vec::new(),
            channel_mappings: Vec::new(),
            textures: Vec::new(),
            extensions: Extensions::default(),
            preserved: Preserved::default(),
        }
    }

    fn ids(puppet: &Puppet) -> Vec<u64> {
        puppet.nodes.iter().map(|node| node.id.0).collect()
    }

    /// The number format is the corpus's.
    ///
    /// `rigel-fixtures` writes every fixture through `serde_json`, and
    /// `CANON-1`'s sharper form asserts that a save of an unmodified puppet
    /// reproduces the file it came from — which it cannot if the two spell a
    /// number differently. The two are different implementations of *the
    /// shortest decimal that reads back as the same double*, so this holds them
    /// against each other rather than trusting that they agree.
    ///
    /// **Where they do not agree**, and the reason the sweep below is bounded:
    /// where two shortest decimals are equidistant from the value, the two pick
    /// different last digits — `-1149636667324797.3` here against
    /// `-1149636667324797.2` there, both of which read back as the same double.
    /// Over two million doubles drawn uniformly from `-10_000..10_000`, which is
    /// every magnitude a puppet's coordinates, gains and times occupy, they
    /// never disagree; the ties are at magnitudes past `2^53` and at the edges
    /// of the exponent. `docs/wip/s2-format.md` reports it: `docs/format.md`
    /// asks for *a fixed number format* and names none.
    #[test]
    fn a_number_is_spelled_the_way_the_corpus_spells_it() {
        for held in [
            0.0f64,
            -0.0,
            1.0,
            -1.0,
            1.6,
            0.25,
            -0.08,
            0.5,
            0.7,
            0.1,
            120.0,
            0.8,
            1.4,
            0.35,
            0.6,
            0.75,
            60.0,
            0.4,
            2.0,
            0.3,
            0.55,
            64.0,
            -64.0,
            128.0,
            -8.0,
            -40.0,
            18.0,
            -36.0,
            24.0,
            12.0,
            -6.0,
            -0.12,
            13.5,
            -4.25,
            3.5e38,
            1e-7,
            1e16,
            1.0 / 3.0,
            0.1 + 0.2,
            1e100,
            -1e-100,
        ] {
            agree(held);
        }

        // A sample across the magnitudes a puppet actually carries, because the
        // corpus's own values exercise none of the places the two formatters
        // could differ — the exponential form, a long mantissa, a negative
        // exponent.
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _ in 0..50_000 {
            agree((next() as f64 / u64::MAX as f64) * 20_000.0 - 10_000.0);
        }
    }

    /// And whatever the value, what is written reads back as itself. Holds for
    /// the magnitudes the test above excludes as well.
    #[test]
    fn a_number_always_reads_back_as_itself() {
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        for _ in 0..50_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let held = f64::from_bits(seed);
            if !held.is_finite() {
                continue;
            }
            let Json::Number(text) = number(held) else {
                panic!("a number is not a number");
            };
            assert_eq!(text.parse::<f64>().expect("it reads back"), held);
            assert!(
                crate::json::parse(text.as_bytes()).is_ok(),
                "{text} is not JSON"
            );
        }
    }

    fn agree(held: f64) {
        let ours = match number(held) {
            Json::Number(text) => text,
            other => panic!("a number is not {other:?}"),
        };
        assert_eq!(
            ours,
            serde_json::to_string(&held).expect("a finite number serialises"),
            "{held:?} is spelled differently from the way the corpus spells it"
        );
        assert_eq!(
            ours.parse::<f64>().expect("it reads back"),
            held,
            "and it reads back as itself"
        );
    }

    /// Preorder, siblings in the order the array carried them. `ID-3` rests on
    /// this: reversing the array reverses sibling order and nothing else.
    #[test]
    fn nodes_come_out_in_depth_first_preorder() {
        let ordered = canonicalise(&subject());
        assert_eq!(ids(&ordered), vec![10, 11, 12, 13]);

        let mut reversed = subject();
        reversed.nodes.reverse();
        let ordered = canonicalise(&reversed);
        assert_eq!(
            ids(&ordered),
            vec![10, 13, 11, 12],
            "the children of each parent come back in the order they were put in"
        );
    }

    /// A tree that is not one still comes out as some order: canonicalisation
    /// runs **before** the validator, so a cycle has to be written for the
    /// validator to be able to name it.
    #[test]
    fn a_cycle_is_ordered_rather_than_walked_for_ever() {
        let mut cyclic = subject();
        cyclic.nodes[0].parent = Id(12);
        let ordered = canonicalise(&cyclic);
        let mut found = ids(&ordered);
        found.sort_unstable();
        assert_eq!(found, vec![10, 11, 12, 13], "every node is still written");

        let mut itself = subject();
        itself.nodes[1].parent = Id(11);
        let ordered = canonicalise(&itself);
        assert_eq!(ordered.nodes.len(), 4);
    }

    /// Keypoints come out in position order and each binding's values come with
    /// them, which is the half of `ID-3` the writer owns.
    #[test]
    fn keypoints_sort_and_take_their_binding_values_with_them() {
        let mut puppet = subject();
        puppet.parameters.push(Parameter {
            // Task 2 (acceptance surface): a placeholder keeping the crate compiling.
            order: None,
            mappable: true,
            id: Id(20),
            name: String::from("p"),
            axes: vec![Axis {
                range: [-1.0, 1.0],
                polarity: AxisPolarity::Bilateral,
            }],
            keypoints: vec![
                Keypoint {
                    id: Id(23),
                    position: vec![1.0],
                    neutral: false,
                },
                Keypoint {
                    id: Id(22),
                    position: vec![0.0],
                    neutral: true,
                },
                Keypoint {
                    id: Id(21),
                    position: vec![-1.0],
                    neutral: false,
                },
            ],
        });
        let mark = |x: f64| Transform {
            translation: [x, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        };
        puppet.bindings.push(Binding {
            id: Id(30),
            kind: BindingKind::Transform,
            parameter: Id(20),
            target: Id(11),
            values: BindingValues::Transform(vec![mark(1.0), mark(0.0), mark(-1.0)]),
            stale: false,
        });

        let ordered = canonicalise(&puppet);
        assert_eq!(
            ordered.parameters[0]
                .keypoints
                .iter()
                .map(|keypoint| keypoint.id.0)
                .collect::<Vec<_>>(),
            vec![21, 22, 23]
        );
        match &ordered.bindings[0].values {
            BindingValues::Transform(values) => assert_eq!(
                values
                    .iter()
                    .map(|held| held.translation[0])
                    .collect::<Vec<_>>(),
                vec![-1.0, 0.0, 1.0],
                "the value against each keypoint moved with it"
            ),
            other => panic!("the wrong shape: {other:?}"),
        }
    }

    /// The backup is named for the version it **holds**, not the one written.
    #[test]
    fn the_backup_is_named_for_the_version_it_holds() {
        assert_eq!(
            backup_path(
                Path::new("/rigs/hero.rigel2d"),
                FormatVersion { major: 1, minor: 0 }
            ),
            PathBuf::from("/rigs/hero.rigel2d.bak-1.0")
        );
    }

    /// The manifest is the format name and two numbers, and nothing a reader
    /// must interpret.
    #[test]
    fn the_manifest_carries_the_name_and_the_version() {
        assert_eq!(
            manifest(FormatVersion::CURRENT),
            br#"{"format":"rigel2d-puppet","version":{"major":1,"minor":1}}"#
        );
    }

    /// Empty is absent, because the corpus's smallest valid puppet writes no
    /// `parameters`, `bindings`, `animations` or `channel_mappings` key at all
    /// and a save of it has to reproduce the file it came from.
    #[test]
    fn an_empty_collection_is_not_written() {
        let document = Writer { puppet: &subject() }
            .document()
            .to_canonical_bytes();
        let text = String::from_utf8(document).expect("UTF-8");
        for absent in [
            "parameters",
            "bindings",
            "animations",
            "channel_mappings",
            "textures",
            "gravity",
        ] {
            assert!(!text.contains(absent), "{absent} was written: {text}");
        }
        assert!(text.contains("composition_mode"), "{text}");
    }
}
