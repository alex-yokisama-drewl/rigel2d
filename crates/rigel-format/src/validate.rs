//! The validator: one validator, gating load and save, runnable standalone.
//!
//! `VAL-13` is the whole of this module's shape — the verdict the loader reaches
//! and the verdict the standalone command reaches must be the same verdict on
//! every file, because two validators that drift are two definitions of a valid
//! puppet.
//!
//! Every rejection names **the rule broken** and **at least one offending
//! identifier**, in bounded output (`VAL-12`). That is what makes a corpus of
//! one-defect fixtures worth having: a rejection that names no rule cannot be
//! attributed to the defect the fixture carries.

use std::path::Path;

use crate::load::Rejection;
use crate::puppet::{Id, Puppet};

/// A rule a puppet must satisfy, named so a rejection can be attributed.
///
/// Every rule `docs/format.md`'s *Validation* list states, plus the ones the
/// criteria add. A rule is a stable name: a fixture carries exactly one defect,
/// and the acceptance suite asserts the rejection names *that* rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// Identifiers are unique within a puppet, within and across kinds (`ID-5`).
    IdentifiersUnique,
    /// No identifier is above the high-water mark the puppet records.
    IdentifierWithinHighWaterMark,
    /// No entity carries the reserved identifier zero.
    IdentifierZeroReserved,
    /// The tree is acyclic. Binds the **tree**; a physics node driving a
    /// parameter a binding on its own ancestor reads is not a cycle (`PHYS-9`).
    TreeAcyclic,
    /// Exactly one node has no parent, and the puppet names it (`VAL-2`).
    ExactlyOneRoot,
    /// The root is a kind that contains ([`crate::may_be_root`]).
    RootIsAKindThatContains,
    /// Every node's children are of a kind its parent's kind may contain, read
    /// from [`crate::may_contain`] (`VAL-16`, `VAL-22`, `VAL-23`).
    Containment,
    /// Every stored reference resolves, or is the reserved zero (`VAL-3`).
    ReferenceResolves,
    /// A parameter's keypoints are ordered along its axes (`VAL-4`).
    KeypointsOrdered,
    /// Every keypoint is within the range its parameter declares (`VAL-4`).
    KeypointWithinDeclaredRange,
    /// Every parameter has exactly one neutral keypoint — none and more than one
    /// are both invalid (`VAL-15`).
    ExactlyOneNeutralKeypoint,
    /// Each of a parameter's axes **declares** a polarity. An absence is not the
    /// value *unknown* (`VAL-17`, `VAL-21`).
    AxisPolarityDeclared,
    /// An enumerated value is one the format defines, or is under `x` — in a
    /// file whose declared version this build knows (`VAL-7`, `VAL-20`).
    EnumeratedValueKnown,
    /// A key the core format does not define, outside `x`, is invalid — in a
    /// file whose declared version this build knows (`VAL-8`, `EXT-2`, `EXT-8`).
    UnmarkedData,
    /// Extension data declares the version it expects (`EXT-3`).
    ExtensionDeclaresVersion,
    /// Extension data declares a human-readable name (`EXT-3`).
    ExtensionDeclaresName,
    /// A parameter carries at most one base driver — a channel mapping, a
    /// physics output, or none (`docs/format.md`).
    AtMostOneBaseDriver,
    /// A binding's value count matches its target's mesh, or the binding is
    /// marked stale. **Both** disagreements are invalid: a mark without a
    /// mismatch is as malformed as a mismatch without a mark (`VAL-5`,
    /// `VAL-9`).
    BindingValueCountAgreesWithStaleMark,
    /// Every number this build interprets is finite, judged at the precision the
    /// runtime evaluates in, in a file of **any** declared version (`VAL-6`,
    /// `VAL-18`, `EXT-9`).
    NumbersFinite,
    /// A track's keyframes are ordered in time (`VAL-26`).
    KeyframesOrdered,
    /// A keyframe's value is within the declared range of the parameter its
    /// track drives (`VAL-26`).
    KeyframeWithinDeclaredRange,
    /// A track's blend duration is at or above zero (`VAL-26`).
    BlendDurationNonNegative,
    /// An animation carries at most one track for any one parameter (`VAL-27`).
    AtMostOneTrackPerParameter,
    /// A physics node's simulation constants are within their declared ranges
    /// (`VAL-25`, [`declared_range`]).
    PhysicsConstantWithinRange,
}

/// One broken rule, with enough to act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    /// Which rule. Never absent — a rejection that names no rule cannot be
    /// acted on and cannot be attributed (`VAL-12`).
    pub rule: Rule,
    /// At least one identifier of an entity the rule was broken by, and every
    /// identifier the rule names: `VAL-26` names the animation **and** the
    /// track, `VAL-25` the node.
    pub offenders: Vec<Id>,
    /// The field or constant the rule names, where it names one — `"length"`
    /// for `VAL-25`, the key for [`Rule::UnmarkedData`].
    pub field: Option<String>,
    /// Bounded, and for a person to read. Never the whole file.
    pub detail: String,
}

/// A simulation constant on a physics node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysicsConstant {
    Length,
    Gravity,
    Frequency,
    AngleDamping,
    LengthDamping,
}

impl PhysicsConstant {
    /// Every constant the format defines for a physics node. `VAL-25` is driven
    /// over this, so a constant added later arrives needing a stated range.
    pub const ALL: &'static [PhysicsConstant] = &[
        PhysicsConstant::Length,
        PhysicsConstant::Gravity,
        PhysicsConstant::Frequency,
        PhysicsConstant::AngleDamping,
        PhysicsConstant::LengthDamping,
    ];
}

/// The range a physics constant is validated against, before anything is
/// allocated for it.
///
/// `docs/format.md` leaves upper bounds `[open]` beyond sign and finiteness, so
/// [`ValidRange::max`] is `None` today for every constant. The lower bound is
/// not open and is what `VAL-25`'s fixtures carry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidRange {
    pub min: f64,
    /// `false` for `length` and `frequency`, which must be **above** zero;
    /// `true` for `gravity` and the dampings, which may be zero.
    pub min_inclusive: bool,
    /// `None` while upper bounds are `[open]` in `docs/format.md`, pending the
    /// integrator-stability sweep due with the evaluation pass.
    pub max: Option<f64>,
}

/// The declared range for one constant.
///
/// `None` means no range has been stated, which fails the check: validating only
/// finiteness is what `VAL-6` already does, and a finite file does not imply a
/// finite simulation (`VAL-25`).
pub fn declared_range(_constant: PhysicsConstant) -> Option<ValidRange> {
    unimplemented!()
}

/// The verdict on a puppet already in memory.
///
/// The gate on **save** as much as on load: a save refuses to write a puppet this
/// rejects (`VAL-13`).
pub fn validate(_puppet: &Puppet) -> Result<(), Vec<Violation>> {
    unimplemented!()
}

/// The verdict on a file, reached without loading it into a [`Puppet`] first.
///
/// What the standalone `rigel-validate` command wraps, and what `VAL-13` asserts
/// agrees with [`crate::load`] on every fixture in the catalogue. It therefore
/// reaches container, version and limit refusals as well as rule violations,
/// which is why it returns a [`Rejection`] rather than a list of [`Violation`]s.
pub fn validate_file(_path: &Path) -> Result<(), Rejection> {
    unimplemented!()
}
