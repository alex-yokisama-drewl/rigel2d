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
//!
//! # What is checked here, and what is checked by the reader
//!
//! [`validate`] sees a [`Puppet`] and nothing else, so the rules it can reach
//! are the ones a puppet holds: identity, the tree, references, ranges, counts,
//! finiteness. Three families are the reader's instead, because they are facts
//! about the **source text** that a puppet cannot carry — unmarked data, an
//! unfamiliar enumerated value, and a member that is simply absent. They are
//! raised in `crate::decode` and are listed in the table there.
//!
//! The consequence that is easiest to get wrong: **[`validate`] admits an
//! unfamiliar enumerated value unconditionally.** Whether one is legal depends
//! on the file's **declared version** (`docs/format.md`, *An unfamiliar value in
//! a field this build knows*), and a [`Puppet`] carries no version —
//! deliberately, because a save decides whether it is upgrading by reading the
//! file it replaces. Gating it here would load a later-minor file under `VER-1`
//! and then refuse to save it, which is the compatibility rule inverted.
//!
//! # The schema beside this module
//!
//! `docs/format.md` requires the validator to be runnable against the
//! specification without our code: `schema/puppet.schema.json` is what a JSON
//! Schema can express of these rules, and `schema/README.md` says where the
//! boundary falls and which rules stay here. A unit test below holds the
//! schema's member lists against `crate::decode`'s, so the two cannot drift.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::containment::{may_be_root, may_contain};
use crate::load::Rejection;
use crate::puppet::{BindingValues, Id, Node, NodeBody, Parameter, Puppet, Transform};

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

impl Rule {
    /// The rule's name, as a person and a test can both read it.
    ///
    /// Added by task 5 because every rejection the reader produces has to name
    /// its rule ([`Rejection::rule_name`], `VAL-12`), and the reader reaches
    /// three rules of its own before the validator runs at all.
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::IdentifiersUnique => "identifiers-unique",
            Rule::IdentifierWithinHighWaterMark => "identifier-within-high-water-mark",
            Rule::IdentifierZeroReserved => "identifier-zero-reserved",
            Rule::TreeAcyclic => "tree-acyclic",
            Rule::ExactlyOneRoot => "exactly-one-root",
            Rule::RootIsAKindThatContains => "root-is-a-kind-that-contains",
            Rule::Containment => "containment",
            Rule::ReferenceResolves => "reference-resolves",
            Rule::KeypointsOrdered => "keypoints-ordered",
            Rule::KeypointWithinDeclaredRange => "keypoint-within-declared-range",
            Rule::ExactlyOneNeutralKeypoint => "exactly-one-neutral-keypoint",
            Rule::AxisPolarityDeclared => "axis-polarity-declared",
            Rule::EnumeratedValueKnown => "enumerated-value-known",
            Rule::UnmarkedData => "unmarked-data",
            Rule::ExtensionDeclaresVersion => "extension-declares-version",
            Rule::ExtensionDeclaresName => "extension-declares-name",
            Rule::AtMostOneBaseDriver => "at-most-one-base-driver",
            Rule::BindingValueCountAgreesWithStaleMark => {
                "binding-value-count-agrees-with-stale-mark"
            }
            Rule::NumbersFinite => "numbers-finite",
            Rule::KeyframesOrdered => "keyframes-ordered",
            Rule::KeyframeWithinDeclaredRange => "keyframe-within-declared-range",
            Rule::BlendDurationNonNegative => "blend-duration-non-negative",
            Rule::AtMostOneTrackPerParameter => "at-most-one-track-per-parameter",
            Rule::PhysicsConstantWithinRange => "physics-constant-within-range",
        }
    }
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

    /// The constant's name, which is the key `docs/format.md`'s physics table
    /// gives it and the field a rejection names (`VAL-25`).
    ///
    /// Added by task 6: the criterion asks the rejection to name the
    /// **constant**, and an identifier list alone cannot say which of the five
    /// a node was out of range on.
    pub fn as_str(self) -> &'static str {
        match self {
            PhysicsConstant::Length => "length",
            PhysicsConstant::Gravity => "gravity",
            PhysicsConstant::Frequency => "frequency",
            PhysicsConstant::AngleDamping => "angle_damping",
            PhysicsConstant::LengthDamping => "length_damping",
        }
    }
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

/// One constant's stated range.
#[derive(Clone, Copy, Debug)]
pub struct RangeDeclaration {
    /// The constant this row declares.
    pub constant: PhysicsConstant,
    /// What the validator enforces for it.
    pub range: ValidRange,
}

/// The one place a physics constant's range is declared (`VAL-25`).
///
/// `docs/format.md`, *What the simulation drives*: `length` and `frequency`
/// above zero, `gravity` and both dampings at or above zero, every value finite.
/// Upper bounds are `[open]` there pending the integrator-stability sweep due
/// with the evaluation pass, so every [`ValidRange::max`] is `None` — which is
/// an upper bound nobody has stated, not an upper bound of infinity. Finiteness
/// is [`Rule::NumbersFinite`]'s and is enforced for every number in the file
/// rather than only for these five.
pub const RANGES: &[RangeDeclaration] = &[
    RangeDeclaration {
        constant: PhysicsConstant::Length,
        range: ValidRange {
            min: 0.0,
            min_inclusive: false,
            max: None,
        },
    },
    RangeDeclaration {
        constant: PhysicsConstant::Gravity,
        range: ValidRange {
            min: 0.0,
            min_inclusive: true,
            max: None,
        },
    },
    RangeDeclaration {
        constant: PhysicsConstant::Frequency,
        range: ValidRange {
            min: 0.0,
            min_inclusive: false,
            max: None,
        },
    },
    RangeDeclaration {
        constant: PhysicsConstant::AngleDamping,
        range: ValidRange {
            min: 0.0,
            min_inclusive: true,
            max: None,
        },
    },
    RangeDeclaration {
        constant: PhysicsConstant::LengthDamping,
        range: ValidRange {
            min: 0.0,
            min_inclusive: true,
            max: None,
        },
    },
];

/// The declared range for one constant.
///
/// `None` means no range has been stated, which fails the check: validating only
/// finiteness is what `VAL-6` already does, and a finite file does not imply a
/// finite simulation (`VAL-25`).
pub fn declared_range(constant: PhysicsConstant) -> Option<ValidRange> {
    RANGES
        .iter()
        .find(|row| row.constant == constant)
        .map(|row| row.range)
}

/// The most violations one verdict carries.
///
/// `VAL-12` asks for bounded output, and a list that grows with the puppet is a
/// denial-of-service route through the error path. The same figure
/// `crate::decode` caps its own list at, for the same reason.
const MAX_VIOLATIONS: usize = 64;

/// The verdict on a puppet already in memory.
///
/// The gate on **save** as much as on load: a save refuses to write a puppet this
/// rejects (`VAL-13`).
pub fn validate(puppet: &Puppet) -> Result<(), Vec<Violation>> {
    let mut checks = Checks::new(puppet);
    // Order matters only for which violations survive the cap: the structural
    // rules a one-defect fixture is built around go in before the sweep over
    // every number in the file, which is the one check that can produce many.
    checks.identifiers();
    checks.tree();
    checks.references();
    checks.parameters();
    checks.bindings();
    checks.animations();
    checks.base_drivers();
    checks.physics();
    checks.numbers();

    if checks.found.is_empty() {
        Ok(())
    } else {
        Err(checks.found)
    }
}

/// The verdict on a file, reached without loading it into a [`Puppet`] first.
///
/// What the standalone `rigel-validate` command wraps, and what `VAL-13` asserts
/// agrees with [`crate::load`] on every fixture in the catalogue. It therefore
/// reaches container, version and limit refusals as well as rule violations,
/// which is why it returns a [`Rejection`] rather than a list of [`Violation`]s.
///
/// It shares `read_file` and `verdict` with [`crate::load`], so
/// the standalone verdict and the loader's are **one** verdict rather than two
/// implementations that drift — which is the whole of `VAL-13`.
pub fn validate_file(path: &Path) -> Result<(), Rejection> {
    verdict(&crate::load::read_file(path)?)
}

/// The verdict on a file already read, as a [`Rejection`].
///
/// The one place the reader's own violations and the validator's are combined,
/// shared by [`crate::load`] and [`validate_file`]. The reader's come first
/// because they are about the source text: a file carrying a key the format does
/// not define is refused for that, not for whatever the half-read puppet looks
/// like afterwards.
pub(crate) fn verdict(opened: &crate::load::Opened) -> Result<(), Rejection> {
    if !opened.violations.is_empty() {
        return Err(Rejection::Invalid(opened.violations.clone()));
    }
    validate(&opened.puppet).map_err(Rejection::Invalid)
}

/// Whether a number this build interprets is finite **at the precision the
/// runtime evaluates in**.
///
/// `VAL-18`: judged at the precision evaluation uses rather than the precision
/// the file was parsed at, so that two tools reading one file cannot disagree
/// about whether it is valid. A value finite as a double and infinite as a
/// single is the case the rule exists for.
fn finite(value: f64) -> bool {
    value.is_finite() && (value as f32).is_finite()
}

/// The rules of `docs/format.md`'s *Validation* list, over one puppet.
///
/// Holds the puppet and the violations found so far, and nothing else: every
/// check reads the puppet and appends, so a rule added later is a method and a
/// call rather than a change of shape.
struct Checks<'a> {
    puppet: &'a Puppet,
    nodes: BTreeMap<Id, &'a Node>,
    parameters: BTreeMap<Id, &'a Parameter>,
    textures: BTreeSet<Id>,
    found: Vec<Violation>,
}

impl<'a> Checks<'a> {
    fn new(puppet: &'a Puppet) -> Self {
        Checks {
            puppet,
            nodes: puppet.nodes.iter().map(|node| (node.id, node)).collect(),
            parameters: puppet
                .parameters
                .iter()
                .map(|parameter| (parameter.id, parameter))
                .collect(),
            textures: puppet.textures.iter().map(|texture| texture.id).collect(),
            found: Vec::new(),
        }
    }

    /// Records one broken rule.
    ///
    /// Never records a violation naming nothing: `VAL-12` asks every rejection
    /// to name at least one offending identifier, and a rule broken by the
    /// puppet as a whole names the puppet.
    fn violate(&mut self, rule: Rule, offenders: Vec<Id>, field: Option<&str>, detail: String) {
        if self.found.len() >= MAX_VIOLATIONS {
            return;
        }
        let offenders = if offenders.is_empty() {
            vec![self.puppet.id]
        } else {
            offenders
        };
        self.found.push(Violation {
            rule,
            offenders,
            field: field.map(str::to_owned),
            detail,
        });
    }

    // ------------------------------------------------------- identity

    /// Identifiers unique, none above the high-water mark, none the reserved
    /// zero (`ID-5`, `docs/format.md` — *Identifiers are integers, not UUIDs*).
    ///
    /// Driven over [`Puppet::identifiers`], which enumerates every piece *What a
    /// puppet is* lists, so a kind added to that list is covered here without an
    /// edit.
    fn identifiers(&mut self) {
        let mut seen = BTreeSet::new();
        let mut repeated = BTreeSet::new();
        let mut zero = false;
        let mut above = BTreeSet::new();
        for id in self.puppet.identifiers() {
            if !seen.insert(id) {
                repeated.insert(id);
            }
            if id == Id::NONE {
                zero = true;
            }
            if id.0 > self.puppet.high_water_mark {
                above.insert(id);
            }
        }

        if !repeated.is_empty() {
            let offenders: Vec<Id> = repeated.iter().copied().collect();
            self.violate(
                Rule::IdentifiersUnique,
                offenders,
                None,
                String::from("one identifier names more than one entity"),
            );
        }
        if zero {
            self.violate(
                Rule::IdentifierZeroReserved,
                vec![Id::NONE],
                None,
                String::from(
                    "an entity carries the reserved identifier zero, which means no reference",
                ),
            );
        }
        if !above.is_empty() {
            let mark = self.puppet.high_water_mark;
            let offenders: Vec<Id> = above.iter().copied().collect();
            self.violate(
                Rule::IdentifierWithinHighWaterMark,
                offenders,
                None,
                format!("above the high-water mark the puppet records, {mark}"),
            );
        }
    }

    // ----------------------------------------------------------- tree

    /// One root, that root a kind that contains, the tree acyclic, and every
    /// parent–child pair one the containment table permits.
    fn tree(&mut self) {
        self.one_root();
        self.root_contains();
        self.acyclic();
        self.containment();
    }

    fn one_root(&mut self) {
        let parentless: Vec<Id> = self
            .puppet
            .nodes
            .iter()
            .filter(|node| node.parent == Id::NONE)
            .map(|node| node.id)
            .collect();
        let named = self.puppet.root;

        if parentless.len() != 1 {
            let count = parentless.len();
            self.violate(
                Rule::ExactlyOneRoot,
                parentless,
                None,
                format!("{count} nodes have no parent, and exactly one may"),
            );
            return;
        }
        if parentless[0] != named {
            let root = parentless[0];
            let mut offenders = vec![root];
            if named != Id::NONE {
                offenders.push(named);
            }
            self.violate(
                Rule::ExactlyOneRoot,
                offenders,
                Some("root"),
                format!(
                    "the puppet names {} as its root and {} is the node with no parent",
                    named.0, root.0
                ),
            );
        }
    }

    fn root_contains(&mut self) {
        let Some(root) = self.nodes.get(&self.puppet.root).copied() else {
            // Either there is no root at all or it names nothing, which
            // `one_root` has already reported.
            return;
        };
        let kind = root.body.kind();
        if may_be_root(kind) != Some(true) {
            self.violate(
                Rule::RootIsAKindThatContains,
                vec![root.id],
                Some("root"),
                format!("{kind:?} is not a kind the containment declaration lets be the root"),
            );
        }
    }

    /// The tree is acyclic.
    ///
    /// Binds the **tree** and nothing else: a physics node driving a parameter
    /// that a binding on its own ancestor reads is a dataflow cycle broken in
    /// time by the previous-frame read, and is valid (`PHYS-9`).
    ///
    /// Every node has at most one parent, so the walk is up a chain rather than
    /// a search: each node is coloured once, which keeps a hostile file from
    /// costing more than one pass.
    fn acyclic(&mut self) {
        let order: BTreeMap<Id, usize> = self
            .puppet
            .nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (node.id, at))
            .collect();
        // 0: not walked, 1: on the path being walked now, 2: settled.
        let mut colour = vec![0u8; self.puppet.nodes.len()];
        let mut cycle: Option<Vec<Id>> = None;

        for start in 0..self.puppet.nodes.len() {
            if colour[start] != 0 {
                continue;
            }
            let mut path: Vec<usize> = Vec::new();
            let mut at = start;
            loop {
                if colour[at] == 1 {
                    let from = path.iter().position(|step| *step == at).unwrap_or(0);
                    cycle.get_or_insert_with(|| {
                        path[from..]
                            .iter()
                            .map(|step| self.puppet.nodes[*step].id)
                            .collect()
                    });
                    break;
                }
                if colour[at] == 2 {
                    break;
                }
                colour[at] = 1;
                path.push(at);
                let parent = self.puppet.nodes[at].parent;
                match order.get(&parent) {
                    Some(next) => at = *next,
                    // No parent, or one that does not resolve — `references`
                    // reports the second.
                    None => break,
                }
            }
            for step in path {
                colour[step] = 2;
            }
        }

        if let Some(offenders) = cycle {
            self.violate(
                Rule::TreeAcyclic,
                offenders,
                Some("parent"),
                String::from("these nodes contain one another"),
            );
        }
    }

    /// Every node's kind is one its parent's kind may contain, **read from the
    /// table** rather than from rules naming kinds (`VAL-16`, `VAL-22`,
    /// `VAL-23`).
    ///
    /// A pair the declaration has no answer for fails the check rather than
    /// inheriting one: a kind added later must arrive with a stated answer
    /// (`VAL-24`).
    fn containment(&mut self) {
        for node in &self.puppet.nodes {
            let Some(parent) = self.nodes.get(&node.parent).copied() else {
                continue;
            };
            let (above, below) = (parent.body.kind(), node.body.kind());
            if may_contain(above, below) == Some(true) {
                continue;
            }
            self.violate(
                Rule::Containment,
                vec![node.id, parent.id],
                Some("parent"),
                match may_contain(above, below) {
                    Some(false) => format!("{above:?} may not contain {below:?}"),
                    _ => {
                        format!("the containment declaration has no answer for {above:?}/{below:?}")
                    }
                },
            );
        }
    }

    // ---------------------------------------------------- references

    /// Every stored reference resolves, or is the reserved zero (`VAL-3`).
    ///
    /// The reserved zero means *no reference* wherever a reference is optional —
    /// a part with no mask, a physics node driving nothing (`PHYS-4`).
    fn references(&mut self) {
        for node in &self.puppet.nodes {
            if node.parent != Id::NONE && !self.nodes.contains_key(&node.parent) {
                let names = node.parent.0;
                self.violate(
                    Rule::ReferenceResolves,
                    vec![node.id],
                    Some("parent"),
                    format!("names node {names}, which the puppet does not carry"),
                );
            }
            match &node.body {
                NodeBody::Part(part) => {
                    if part.texture != Id::NONE && !self.textures.contains(&part.texture) {
                        let names = part.texture.0;
                        self.violate(
                            Rule::ReferenceResolves,
                            vec![node.id],
                            Some("texture"),
                            format!("names texture {names}, which the puppet does not carry"),
                        );
                    }
                    if part.mask != Id::NONE && !self.nodes.contains_key(&part.mask) {
                        let names = part.mask.0;
                        self.violate(
                            Rule::ReferenceResolves,
                            vec![node.id],
                            Some("mask"),
                            format!("names node {names}, which the puppet does not carry"),
                        );
                    }
                }
                NodeBody::Physics(physics) => {
                    if physics.output != Id::NONE && !self.parameters.contains_key(&physics.output)
                    {
                        let names = physics.output.0;
                        self.violate(
                            Rule::ReferenceResolves,
                            vec![node.id],
                            Some("output"),
                            format!("names parameter {names}, which the puppet does not carry"),
                        );
                    }
                }
                NodeBody::MeshGroup(_) | NodeBody::Composite => {}
            }
        }

        for binding in &self.puppet.bindings {
            if !self.parameters.contains_key(&binding.parameter) {
                let names = binding.parameter.0;
                self.violate(
                    Rule::ReferenceResolves,
                    vec![binding.id],
                    Some("parameter"),
                    format!("names parameter {names}, which the puppet does not carry"),
                );
            }
            if !self.nodes.contains_key(&binding.target) {
                let names = binding.target.0;
                self.violate(
                    Rule::ReferenceResolves,
                    vec![binding.id],
                    Some("target"),
                    format!("names node {names}, which the puppet does not carry"),
                );
            }
        }

        for animation in &self.puppet.animations {
            for track in &animation.tracks {
                if !self.parameters.contains_key(&track.parameter) {
                    let names = track.parameter.0;
                    self.violate(
                        Rule::ReferenceResolves,
                        vec![animation.id, track.id],
                        Some("parameter"),
                        format!("names parameter {names}, which the puppet does not carry"),
                    );
                }
            }
        }

        for mapping in &self.puppet.channel_mappings {
            if !self.parameters.contains_key(&mapping.parameter) {
                let names = mapping.parameter.0;
                self.violate(
                    Rule::ReferenceResolves,
                    vec![mapping.id],
                    Some("parameter"),
                    format!("names parameter {names}, which the puppet does not carry"),
                );
            }
        }
    }

    // --------------------------------------------------- parameters

    /// Keypoints ordered and within the range their parameter declares, and
    /// exactly one of them neutral (`VAL-4`, `VAL-15`).
    fn parameters(&mut self) {
        for parameter in &self.puppet.parameters {
            let mut previous: Option<&[f64]> = None;
            for keypoint in &parameter.keypoints {
                if let Some(before) = previous
                    && !ascends(before, &keypoint.position)
                {
                    self.violate(
                        Rule::KeypointsOrdered,
                        vec![parameter.id, keypoint.id],
                        Some("position"),
                        String::from("a keypoint does not come after the one before it"),
                    );
                }
                previous = Some(&keypoint.position);

                for (axis, coordinate) in parameter.axes.iter().zip(&keypoint.position) {
                    let (low, high) = span(axis.range);
                    if *coordinate < low || *coordinate > high {
                        self.violate(
                            Rule::KeypointWithinDeclaredRange,
                            vec![parameter.id, keypoint.id],
                            Some("position"),
                            format!("{coordinate} is outside the declared range {low}..{high}"),
                        );
                    }
                }
            }

            let neutral: Vec<Id> = parameter
                .keypoints
                .iter()
                .filter(|keypoint| keypoint.neutral)
                .map(|keypoint| keypoint.id)
                .collect();
            if neutral.len() != 1 {
                let count = neutral.len();
                let mut offenders = vec![parameter.id];
                offenders.extend(neutral);
                self.violate(
                    Rule::ExactlyOneNeutralKeypoint,
                    offenders,
                    Some("neutral"),
                    format!("{count} keypoints are neutral, and exactly one must be"),
                );
            }
        }
    }

    // ----------------------------------------------------- bindings

    /// A binding's value counts agree with the mark it carries, in **both**
    /// directions (`VAL-5`, `VAL-9`, `ART-5`).
    ///
    /// Stale is a state, not a failure: a binding whose values no longer match
    /// the mesh they apply to, **and which carries the mark**, is valid, loads,
    /// renders as absent and saves unchanged. The same mismatch without the mark
    /// is malformed, and so is the mark with a matching count — without the
    /// second half a file can claim a staleness that is not there, and retained
    /// work and a corrupt file look identical.
    fn bindings(&mut self) {
        for binding in &self.puppet.bindings {
            let (Some(parameter), Some(target)) = (
                self.parameters.get(&binding.parameter).copied(),
                self.nodes.get(&binding.target).copied(),
            ) else {
                // A reference that does not resolve is `references`'s to report,
                // and there is nothing to compare the counts against.
                continue;
            };

            let keypoints = parameter.keypoints.len();
            let mut agrees = binding.values.len() == keypoints;
            let mut detail = format!(
                "{} values against {keypoints} keypoints",
                binding.values.len()
            );
            if let BindingValues::Deformation(values) = &binding.values
                && let Some(vertices) = vertices_of(target)
            {
                let matched = values.iter().all(|offsets| offsets.len() == vertices);
                if !matched {
                    detail = format!("per-vertex offsets against a mesh of {vertices} vertices");
                }
                agrees = agrees && matched;
            }

            match (agrees, binding.stale) {
                (false, false) => self.violate(
                    Rule::BindingValueCountAgreesWithStaleMark,
                    vec![binding.id, target.id],
                    Some("values"),
                    format!("{detail}, and the binding is not marked stale"),
                ),
                (true, true) => self.violate(
                    Rule::BindingValueCountAgreesWithStaleMark,
                    vec![binding.id, target.id],
                    Some("stale"),
                    String::from("marked stale, and its values match what they apply to"),
                ),
                _ => {}
            }
        }
    }

    // --------------------------------------------------- animations

    /// A track's keyframes carry the rules a parameter's keypoints do, and an
    /// animation carries at most one track per parameter (`VAL-26`, `VAL-27`).
    ///
    /// Every violation here names **the animation and the track**, because a
    /// track is only reachable through the animation that holds it.
    fn animations(&mut self) {
        for animation in &self.puppet.animations {
            let mut driven: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
            for track in &animation.tracks {
                driven.entry(track.parameter).or_default().push(track.id);

                if track.blend_duration < 0.0 {
                    let found = track.blend_duration;
                    self.violate(
                        Rule::BlendDurationNonNegative,
                        vec![animation.id, track.id],
                        Some("blend_duration"),
                        format!("{found} is below zero"),
                    );
                }

                let mut previous: Option<f64> = None;
                for keyframe in &track.keyframes {
                    if let Some(before) = previous
                        && !follows(before, keyframe.time)
                    {
                        self.violate(
                            Rule::KeyframesOrdered,
                            vec![animation.id, track.id],
                            Some("time"),
                            String::from("a keyframe does not come after the one before it"),
                        );
                    }
                    previous = Some(keyframe.time);

                    let Some(parameter) = self.parameters.get(&track.parameter).copied() else {
                        continue;
                    };
                    for (axis, coordinate) in parameter.axes.iter().zip(&keyframe.value) {
                        let (low, high) = span(axis.range);
                        if *coordinate < low || *coordinate > high {
                            self.violate(
                                Rule::KeyframeWithinDeclaredRange,
                                vec![animation.id, track.id],
                                Some("value"),
                                format!("{coordinate} is outside the declared range {low}..{high}"),
                            );
                        }
                    }
                }
            }

            for (parameter, tracks) in driven {
                if tracks.len() < 2 {
                    continue;
                }
                let count = tracks.len();
                let mut offenders = vec![animation.id, parameter];
                offenders.extend(tracks);
                self.violate(
                    Rule::AtMostOneTrackPerParameter,
                    offenders,
                    Some("parameter"),
                    format!("{count} tracks drive one parameter, and at most one may"),
                );
            }
        }
    }

    // ------------------------------------------------- base drivers

    /// Every parameter carries at most one base driver — a channel mapping, a
    /// physics output, or none (`docs/format.md`).
    ///
    /// Two is the validation failure the evaluation pass relies on: with two,
    /// what the parameter holds depends on which driver ran last.
    fn base_drivers(&mut self) {
        let mut drivers: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        for mapping in &self.puppet.channel_mappings {
            drivers
                .entry(mapping.parameter)
                .or_default()
                .push(mapping.id);
        }
        for node in &self.puppet.nodes {
            if let NodeBody::Physics(physics) = &node.body
                && physics.output != Id::NONE
            {
                drivers.entry(physics.output).or_default().push(node.id);
            }
        }

        for (parameter, by) in drivers {
            if by.len() < 2 {
                continue;
            }
            let count = by.len();
            let mut offenders = vec![parameter];
            offenders.extend(by);
            self.violate(
                Rule::AtMostOneBaseDriver,
                offenders,
                None,
                format!("{count} base drivers feed one parameter, and at most one may"),
            );
        }
    }

    // ----------------------------------------------------- physics

    /// A physics node's simulation constants are within their declared ranges,
    /// and a constant with **no stated range** fails the check (`VAL-25`).
    ///
    /// Driven over [`PhysicsConstant::ALL`], so a constant added to the format
    /// arrives needing a stated range rather than being silently unchecked.
    fn physics(&mut self) {
        for node in &self.puppet.nodes {
            let NodeBody::Physics(physics) = &node.body else {
                continue;
            };
            for constant in PhysicsConstant::ALL {
                let held = match constant {
                    PhysicsConstant::Length => physics.length,
                    PhysicsConstant::Gravity => physics.gravity,
                    PhysicsConstant::Frequency => physics.frequency,
                    PhysicsConstant::AngleDamping => physics.angle_damping,
                    PhysicsConstant::LengthDamping => physics.length_damping,
                };
                let field = constant.as_str();
                let Some(range) = declared_range(*constant) else {
                    self.violate(
                        Rule::PhysicsConstantWithinRange,
                        vec![node.id],
                        Some(field),
                        String::from("no valid range has been stated for this constant"),
                    );
                    continue;
                };
                let below = if range.min_inclusive {
                    held < range.min
                } else {
                    held <= range.min
                };
                if below {
                    let edge = if range.min_inclusive {
                        "at or above"
                    } else {
                        "above"
                    };
                    let min = range.min;
                    self.violate(
                        Rule::PhysicsConstantWithinRange,
                        vec![node.id],
                        Some(field),
                        format!("{held} is not {edge} {min}"),
                    );
                    continue;
                }
                if let Some(max) = range.max
                    && held > max
                {
                    self.violate(
                        Rule::PhysicsConstantWithinRange,
                        vec![node.id],
                        Some(field),
                        format!("{held} is above {max}"),
                    );
                }
            }
        }
    }

    // ----------------------------------------------------- numbers

    /// Every number this build interprets is finite, at the precision
    /// evaluation uses (`VAL-6`, `VAL-18`).
    ///
    /// A number inside material this build does not interpret is **not** here:
    /// it is bytes under [`crate::Preserved`] or an extension's `data`, never
    /// read as a number at all, which is what lets a later-version file carry
    /// `1e400` through a round trip untouched (`EXT-9`).
    fn numbers(&mut self) {
        let puppet = self.puppet;
        self.number(puppet.id, "gravity", puppet.gravity);

        for node in &puppet.nodes {
            self.transform(node.id, "transform", &node.transform);
            match &node.body {
                NodeBody::Part(part) => {
                    self.mesh(node.id, &part.mesh);
                    if let Some(chain) = &part.chain_deformer {
                        for joint in &chain.joints {
                            self.pair(joint.id, "position", joint.position);
                        }
                    }
                }
                NodeBody::MeshGroup(group) => self.mesh(node.id, &group.mesh),
                NodeBody::Composite => {}
                NodeBody::Physics(physics) => {
                    for constant in PhysicsConstant::ALL {
                        let held = match constant {
                            PhysicsConstant::Length => physics.length,
                            PhysicsConstant::Gravity => physics.gravity,
                            PhysicsConstant::Frequency => physics.frequency,
                            PhysicsConstant::AngleDamping => physics.angle_damping,
                            PhysicsConstant::LengthDamping => physics.length_damping,
                        };
                        self.number(node.id, constant.as_str(), held);
                    }
                    for gain in &physics.output_scale {
                        self.number(node.id, "output_scale", *gain);
                    }
                }
            }
        }

        for parameter in &puppet.parameters {
            for axis in &parameter.axes {
                self.pair(parameter.id, "range", axis.range);
            }
            for keypoint in &parameter.keypoints {
                for coordinate in &keypoint.position {
                    self.number(keypoint.id, "position", *coordinate);
                }
            }
        }

        for binding in &puppet.bindings {
            match &binding.values {
                BindingValues::Transform(values) => {
                    for held in values {
                        self.transform(binding.id, "values", held);
                    }
                }
                BindingValues::Deformation(values) => {
                    for offsets in values {
                        for offset in offsets {
                            self.pair(binding.id, "values", *offset);
                        }
                    }
                }
            }
        }

        for animation in &puppet.animations {
            for track in &animation.tracks {
                self.number(track.id, "blend_duration", track.blend_duration);
                for keyframe in &track.keyframes {
                    self.number(track.id, "time", keyframe.time);
                    for coordinate in &keyframe.value {
                        self.number(track.id, "value", *coordinate);
                    }
                }
            }
        }

        for mapping in &puppet.channel_mappings {
            for point in &mapping.response_curve.points {
                self.pair(mapping.id, "response_curve", *point);
            }
        }
    }

    fn number(&mut self, owner: Id, field: &str, held: f64) {
        if finite(held) {
            return;
        }
        self.violate(
            Rule::NumbersFinite,
            vec![owner],
            Some(field),
            format!("{held} is not a finite number at the precision evaluation uses"),
        );
    }

    fn pair(&mut self, owner: Id, field: &str, held: [f64; 2]) {
        for coordinate in held {
            self.number(owner, field, coordinate);
        }
    }

    fn transform(&mut self, owner: Id, field: &str, held: &Transform) {
        self.pair(owner, field, held.translation);
        self.number(owner, field, held.rotation);
        self.pair(owner, field, held.scale);
    }

    fn mesh(&mut self, owner: Id, held: &crate::puppet::Mesh) {
        for vertex in &held.vertices {
            self.pair(owner, "vertices", *vertex);
        }
        for uv in &held.uvs {
            self.pair(owner, "uvs", *uv);
        }
    }
}

/// Whether one keypoint position comes strictly after another.
///
/// Compared coordinate by coordinate, which is the order
/// `docs/format.md` stores keypoints in — *keypoints by position along their
/// axis* — and which gives a two-dimensional parameter a total order without a
/// second rule. Strict, so two keypoints at one position are out of order as
/// well: interpolating between them has no defined result.
fn ascends(before: &[f64], after: &[f64]) -> bool {
    for (held, next) in before.iter().zip(after) {
        if next > held {
            return true;
        }
        if next < held {
            return false;
        }
    }
    // Equal as far as both go: longer is after, equal is not.
    after.len() > before.len()
}

/// Whether one keyframe time comes strictly after another.
///
/// The rule [`ascends`] applies to a keypoint position, over the one coordinate
/// a time is. Ordering is load-bearing rather than cosmetic: a lookup that seeks
/// rather than scans returns wrong values silently on unordered times instead of
/// failing (`VAL-26`).
fn follows(before: f64, after: f64) -> bool {
    ascends(&[before], &[after])
}

/// A declared range as `(low, high)`, whichever way round it was written.
fn span(range: [f64; 2]) -> (f64, f64) {
    (range[0].min(range[1]), range[0].max(range[1]))
}

/// The vertex count of the mesh a node carries, or `None` for a node with none.
fn vertices_of(node: &Node) -> Option<usize> {
    match &node.body {
        NodeBody::Part(part) => Some(part.mesh.vertices.len()),
        NodeBody::MeshGroup(group) => Some(group.mesh.vertices.len()),
        NodeBody::Composite | NodeBody::Physics(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every constant the format defines has a stated range, which is the half
    /// of `VAL-25` that holds before a file is read.
    #[test]
    fn every_constant_has_a_range() {
        for constant in PhysicsConstant::ALL {
            assert!(
                declared_range(*constant).is_some(),
                "{constant:?} has no stated range"
            );
            let rows = RANGES.iter().filter(|r| r.constant == *constant).count();
            assert_eq!(rows, 1, "{constant:?} has {rows} rows in RANGES");
        }
        assert_eq!(
            RANGES.len(),
            PhysicsConstant::ALL.len(),
            "RANGES carries a row for every constant and no others"
        );
    }

    /// The sign rules `docs/format.md` states: two constants must be **above**
    /// zero and three may be zero. Conflating them rejects a rig that turns
    /// gravity off for one strand, which is the documented way to make a spike.
    #[test]
    fn the_lower_bounds_are_what_the_format_states() {
        let above_zero = [PhysicsConstant::Length, PhysicsConstant::Frequency];
        for constant in PhysicsConstant::ALL {
            let range = declared_range(*constant).expect("a range");
            assert_eq!(range.min, 0.0, "{constant:?}'s lower bound is zero");
            assert_eq!(
                range.min_inclusive,
                !above_zero.contains(constant),
                "{constant:?}: whether zero itself is admitted"
            );
        }
    }

    /// Upper bounds are `[open]` in `docs/format.md`. An invented one would be a
    /// measurement nobody took, presented as a rule.
    #[test]
    fn no_upper_bound_is_stated_yet() {
        for constant in PhysicsConstant::ALL {
            assert_eq!(
                declared_range(*constant).expect("a range").max,
                None,
                "{constant:?} has an upper bound the format does not state"
            );
        }
    }

    // ------------------------------------------------------ the subject

    use crate::containment::NodeKind;
    use crate::puppet::{
        Animation, Axis, AxisPolarity, Binding, BindingKind, ChannelMapping, CompositionMode,
        Extensions, Keyframe, Keypoint, Mesh, MeshGroup, Node, Parameter, Part, PhysicsNode,
        Preserved, ResponseCurve, TextureFlip, TextureRecord, Track,
    };

    fn transform() -> Transform {
        Transform {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }

    fn mesh(vertices: usize) -> Mesh {
        Mesh {
            vertices: (0..vertices).map(|n| [n as f64, 0.0]).collect(),
            uvs: (0..vertices).map(|n| [n as f64, 0.0]).collect(),
            triangles: vec![[0, 1, 2]],
        }
    }

    fn node(id: u64, parent: u64, body: NodeBody) -> Node {
        Node {
            id: Id(id),
            parent: Id(parent),
            name: format!("node {id}"),
            transform: transform(),
            body,
            extensions: Extensions::default(),
        }
    }

    fn physics() -> PhysicsNode {
        PhysicsNode {
            length: 100.0,
            gravity: 1.0,
            frequency: 1.0,
            angle_damping: 0.5,
            length_damping: 0.5,
            output: Id::NONE,
            output_source: Vec::new(),
            output_scale: Vec::new(),
        }
    }

    fn part() -> Part {
        Part {
            mesh: mesh(3),
            texture: Id(10),
            texture_flip: TextureFlip::default(),
            mask: Id::NONE,
            provenance: None,
            chain_deformer: None,
        }
    }

    /// A puppet carrying one of everything a rule here is about, and valid.
    ///
    /// Every test below starts from this and breaks exactly one thing, which is
    /// the same discipline the fixture corpus is built on: a rejection that
    /// cannot be attributed to the change is not evidence about the rule.
    fn subject() -> Puppet {
        Puppet {
            id: Id(1),
            name: String::from("subject"),
            high_water_mark: 100,
            root: Id(2),
            gravity: 1.0,
            composition_mode: CompositionMode::Sum,
            declared_version: None,
            nodes: vec![
                node(2, 0, NodeBody::Composite),
                node(3, 2, NodeBody::Part(part())),
                node(4, 2, NodeBody::Physics(physics())),
            ],
            parameters: vec![Parameter {
                // Task 2 (acceptance surface): a placeholder keeping the crate compiling.
                order: None,
                mappable: true,
                id: Id(20),
                name: String::from("head_turn"),
                axes: vec![Axis {
                    range: [-1.0, 1.0],
                    polarity: AxisPolarity::Bilateral,
                }],
                keypoints: vec![
                    Keypoint {
                        id: Id(21),
                        position: vec![-1.0],
                        neutral: false,
                    },
                    Keypoint {
                        id: Id(22),
                        position: vec![0.0],
                        neutral: true,
                    },
                    Keypoint {
                        id: Id(23),
                        position: vec![1.0],
                        neutral: false,
                    },
                ],
            }],
            bindings: vec![Binding {
                id: Id(30),
                kind: BindingKind::Transform,
                parameter: Id(20),
                target: Id(3),
                values: BindingValues::Transform(vec![transform(), transform(), transform()]),
                stale: false,
            }],
            animations: vec![Animation {
                // Task 2 (acceptance surface): a placeholder keeping the crate compiling.
                end_behaviour: crate::puppet::EndBehaviour::Loop,
                triggers: Vec::new(),
                id: Id(40),
                name: String::from("idle"),
                tracks: vec![Track {
                    id: Id(41),
                    parameter: Id(20),
                    blend_duration: 0.25,
                    keyframes: vec![
                        Keyframe {
                            // Task 2 (acceptance surface): a placeholder keeping the crate compiling.
                            interpolation: crate::puppet::Interpolation::Linear,
                            control_points: None,
                            time: 0.0,
                            value: vec![0.0],
                        },
                        Keyframe {
                            // Task 2 (acceptance surface): a placeholder keeping the crate compiling.
                            interpolation: crate::puppet::Interpolation::Linear,
                            control_points: None,
                            time: 1.0,
                            value: vec![1.0],
                        },
                    ],
                }],
            }],
            channel_mappings: vec![ChannelMapping {
                id: Id(50),
                channel: String::from("head_turn"),
                parameter: Id(20),
                source: String::new(),
                response_curve: ResponseCurve {
                    points: vec![[0.0, 0.0], [1.0, 1.0]],
                },
            }],
            textures: vec![TextureRecord {
                id: Id(10),
                name: String::from("face"),
                authored_size: [16, 16],
                crop: [0, 0, 16, 16],
                payload: Vec::new(),
            }],
            extensions: Extensions::default(),
            preserved: Preserved::default(),
        }
    }

    /// The rules a puppet is rejected for, or an empty list for one that is not.
    fn rules(puppet: &Puppet) -> Vec<Rule> {
        match validate(puppet) {
            Ok(()) => Vec::new(),
            Err(found) => {
                for broken in &found {
                    assert!(
                        !broken.offenders.is_empty(),
                        "{:?} names no offending identifier, which VAL-12 requires",
                        broken.rule
                    );
                }
                found.into_iter().map(|broken| broken.rule).collect()
            }
        }
    }

    /// Asserts a puppet is rejected naming **this** rule, not merely rejected.
    fn assert_names(puppet: &Puppet, rule: Rule) {
        let named = rules(puppet);
        assert!(
            named.contains(&rule),
            "expected {rule:?}, and the verdict was {named:?}"
        );
    }

    /// The premise every test below rests on. If this fails, every assertion
    /// that something *is* rejected is passing for the wrong reason.
    #[test]
    fn the_subject_is_valid() {
        assert_eq!(rules(&subject()), Vec::new());
    }

    // --------------------------------------------------------- identity

    #[test]
    fn a_repeated_identifier_is_rejected() {
        let mut broken = subject();
        broken.textures[0].id = Id(20);
        assert_names(&broken, Rule::IdentifiersUnique);

        let mut across_kinds = subject();
        across_kinds.nodes[1].id = Id(10);
        assert_names(&across_kinds, Rule::IdentifiersUnique);
    }

    /// The reserved zero means *no reference*, so an entity carrying it is
    /// invalid while an optional reference holding it is simply empty.
    #[test]
    fn the_reserved_zero_is_rejected_as_an_entity_and_admitted_as_a_reference() {
        let mut broken = subject();
        broken.parameters[0].keypoints[0].id = Id::NONE;
        assert_names(&broken, Rule::IdentifierZeroReserved);

        let mut empty_reference = subject();
        if let NodeBody::Part(part) = &mut empty_reference.nodes[1].body {
            part.mask = Id::NONE;
        }
        assert_eq!(rules(&empty_reference), Vec::new());
    }

    #[test]
    fn an_identifier_above_the_high_water_mark_is_rejected() {
        let mut broken = subject();
        broken.high_water_mark = 30;
        assert_names(&broken, Rule::IdentifierWithinHighWaterMark);
    }

    // ------------------------------------------------------------- tree

    /// A cycle must be **found**, and finding it must terminate — a walk up the
    /// parent chain with no colouring loops for ever on this puppet.
    #[test]
    fn a_cycle_is_rejected_and_the_walk_terminates() {
        let mut broken = subject();
        broken.nodes.push(node(5, 6, NodeBody::Composite));
        broken.nodes.push(node(6, 5, NodeBody::Composite));
        assert_names(&broken, Rule::TreeAcyclic);
    }

    #[test]
    fn no_root_and_two_roots_are_both_rejected() {
        let mut none = subject();
        none.root = Id::NONE;
        assert_names(&none, Rule::ExactlyOneRoot);

        let mut two = subject();
        two.nodes[1].parent = Id::NONE;
        assert_names(&two, Rule::ExactlyOneRoot);
    }

    #[test]
    fn a_root_that_is_not_a_kind_that_contains_is_rejected() {
        let mut broken = subject();
        broken.nodes = vec![node(2, 0, NodeBody::Part(part()))];
        broken.bindings.clear();
        assert_names(&broken, Rule::RootIsAKindThatContains);
    }

    /// Read from the containment declaration in both directions, over the square
    /// of the kinds, so a kind added to the table is covered without a new test.
    #[test]
    fn containment_is_read_from_the_declaration() {
        let body = |kind: NodeKind| match kind {
            NodeKind::Part => NodeBody::Part(part()),
            NodeKind::MeshGroup => NodeBody::MeshGroup(MeshGroup { mesh: mesh(3) }),
            NodeKind::Composite => NodeBody::Composite,
            NodeKind::Physics => NodeBody::Physics(physics()),
        };

        for parent in NodeKind::ALL {
            for child in NodeKind::ALL {
                let mut puppet = subject();
                puppet.bindings.clear();
                puppet.nodes = vec![
                    node(2, 0, NodeBody::Composite),
                    node(3, 2, body(*parent)),
                    node(4, 3, body(*child)),
                ];
                let permitted = may_contain(*parent, *child).expect("a stated answer");
                let named = rules(&puppet);
                assert_eq!(
                    named.contains(&Rule::Containment),
                    !permitted || may_contain(NodeKind::Composite, *parent) != Some(true),
                    "{parent:?} containing {child:?}: the declaration says {permitted}, \
                     and the verdict was {named:?}"
                );
            }
        }
    }

    // ------------------------------------------------------ references

    #[test]
    fn every_kind_of_dangling_reference_is_rejected() {
        let missing = Id(999);

        let mut parent = subject();
        parent.nodes[1].parent = missing;
        assert_names(&parent, Rule::ReferenceResolves);

        let mut texture = subject();
        if let NodeBody::Part(part) = &mut texture.nodes[1].body {
            part.texture = missing;
        }
        assert_names(&texture, Rule::ReferenceResolves);

        let mut mask = subject();
        if let NodeBody::Part(part) = &mut mask.nodes[1].body {
            part.mask = missing;
        }
        assert_names(&mask, Rule::ReferenceResolves);

        let mut output = subject();
        if let NodeBody::Physics(node) = &mut output.nodes[2].body {
            node.output = missing;
        }
        assert_names(&output, Rule::ReferenceResolves);

        let mut parameter = subject();
        parameter.bindings[0].parameter = missing;
        assert_names(&parameter, Rule::ReferenceResolves);

        let mut target = subject();
        target.bindings[0].target = missing;
        assert_names(&target, Rule::ReferenceResolves);

        let mut track = subject();
        track.animations[0].tracks[0].parameter = missing;
        assert_names(&track, Rule::ReferenceResolves);

        let mut mapping = subject();
        mapping.channel_mappings[0].parameter = missing;
        assert_names(&mapping, Rule::ReferenceResolves);
    }

    /// A physics node driving a parameter that a binding on its own **ancestor**
    /// reads is valid: the acyclicity rule binds the tree, and that dataflow
    /// cycle is broken in time by the previous-frame read (`PHYS-9`).
    #[test]
    fn a_physics_node_driving_its_own_ancestors_binding_is_valid() {
        let mut puppet = subject();
        puppet.nodes = vec![
            node(2, 0, NodeBody::Composite),
            node(3, 2, NodeBody::MeshGroup(MeshGroup { mesh: mesh(3) })),
            node(4, 3, NodeBody::Physics(physics())),
        ];
        if let NodeBody::Physics(driver) = &mut puppet.nodes[2].body {
            driver.output = Id(20);
            driver.output_source = vec![crate::puppet::OutputSource::Angle];
            driver.output_scale = vec![1.0];
        }
        // The binding on node 3 reads the parameter the physics node below it
        // drives, and the channel mapping is gone so the parameter has one base
        // driver.
        puppet.channel_mappings.clear();
        assert_eq!(rules(&puppet), Vec::new());
    }

    // ------------------------------------------------------ parameters

    #[test]
    fn keypoints_are_ordered_and_within_the_declared_range() {
        let mut unordered = subject();
        unordered.parameters[0].keypoints[0].position = vec![0.5];
        assert_names(&unordered, Rule::KeypointsOrdered);

        let mut repeated = subject();
        repeated.parameters[0].keypoints[1].position = vec![-1.0];
        assert_names(&repeated, Rule::KeypointsOrdered);

        let mut outside = subject();
        outside.parameters[0].keypoints[2].position = vec![1.5];
        assert_names(&outside, Rule::KeypointWithinDeclaredRange);
    }

    /// A two-dimensional parameter is ordered coordinate by coordinate, which is
    /// what gives it a total order without a second rule.
    #[test]
    fn a_two_dimensional_parameter_is_ordered_coordinate_by_coordinate() {
        assert!(ascends(&[-1.0, 1.0], &[0.0, 0.0]));
        assert!(ascends(&[0.0, 0.0], &[0.0, 1.0]));
        assert!(!ascends(&[0.0, 1.0], &[0.0, 0.0]));
        assert!(!ascends(&[0.0, 0.0], &[0.0, 0.0]), "equal is not after");
    }

    #[test]
    fn a_parameter_carries_exactly_one_neutral_keypoint() {
        let mut none = subject();
        none.parameters[0].keypoints[1].neutral = false;
        assert_names(&none, Rule::ExactlyOneNeutralKeypoint);

        let mut two = subject();
        two.parameters[0].keypoints[0].neutral = true;
        assert_names(&two, Rule::ExactlyOneNeutralKeypoint);
    }

    // -------------------------------------------------------- bindings

    /// The three-way distinction the mark exists for: a mismatch with the mark
    /// is a **state**, a mismatch without it is malformed, and the mark with no
    /// mismatch is malformed too. Without the third, a file from a stranger can
    /// claim a staleness that is not there.
    #[test]
    fn the_stale_mark_must_agree_with_the_counts_in_both_directions() {
        let deforming = |offsets: usize, stale: bool| {
            let mut puppet = subject();
            puppet.bindings[0].kind = BindingKind::Deformation;
            puppet.bindings[0].values = BindingValues::Deformation(vec![
                vec![[0.0, 0.0]; offsets],
                vec![[0.0, 0.0]; offsets],
                vec![[0.0, 0.0]; offsets],
            ]);
            puppet.bindings[0].stale = stale;
            puppet
        };

        // The target's mesh carries three vertices.
        assert_eq!(rules(&deforming(3, false)), Vec::new());
        assert_eq!(
            rules(&deforming(2, true)),
            Vec::new(),
            "a mismatch carrying the mark is a state, not a failure"
        );
        assert_names(
            &deforming(2, false),
            Rule::BindingValueCountAgreesWithStaleMark,
        );
        assert_names(
            &deforming(3, true),
            Rule::BindingValueCountAgreesWithStaleMark,
        );
    }

    /// The other count the mark governs: one value per keypoint of the
    /// parameter, whatever the binding drives.
    #[test]
    fn a_binding_stores_one_value_per_keypoint_or_is_marked_stale() {
        let mut short = subject();
        short.bindings[0].values = BindingValues::Transform(vec![transform()]);
        assert_names(&short, Rule::BindingValueCountAgreesWithStaleMark);

        short.bindings[0].stale = true;
        assert_eq!(rules(&short), Vec::new());
    }

    // ------------------------------------------------------ animations

    #[test]
    fn a_tracks_keyframes_carry_the_rules_a_parameters_keypoints_do() {
        let mut unordered = subject();
        unordered.animations[0].tracks[0].keyframes[0].time = 1.5;
        assert_names(&unordered, Rule::KeyframesOrdered);

        let mut outside = subject();
        outside.animations[0].tracks[0].keyframes[1].value = vec![1.5];
        assert_names(&outside, Rule::KeyframeWithinDeclaredRange);

        let mut blend = subject();
        blend.animations[0].tracks[0].blend_duration = -0.25;
        assert_names(&blend, Rule::BlendDurationNonNegative);
    }

    /// `VAL-26` asks each of these to name the animation **and** the track, and
    /// a track is reachable only through the animation that holds it.
    #[test]
    fn a_track_rejection_names_the_animation_and_the_track() {
        let mut broken = subject();
        broken.animations[0].tracks[0].blend_duration = -1.0;
        let found = validate(&broken).expect_err("a negative blend duration is rejected");
        let named = found
            .iter()
            .find(|broken| broken.rule == Rule::BlendDurationNonNegative)
            .expect("the rule");
        assert!(named.offenders.contains(&Id(40)), "the animation");
        assert!(named.offenders.contains(&Id(41)), "the track");
        assert_eq!(named.field.as_deref(), Some("blend_duration"));
    }

    #[test]
    fn two_tracks_for_one_parameter_are_rejected() {
        let mut broken = subject();
        let mut second = broken.animations[0].tracks[0].clone();
        second.id = Id(42);
        broken.animations[0].tracks.push(second);
        assert_names(&broken, Rule::AtMostOneTrackPerParameter);
    }

    // ---------------------------------------------------- base drivers

    #[test]
    fn a_parameter_carries_at_most_one_base_driver() {
        let mut broken = subject();
        if let NodeBody::Physics(driver) = &mut broken.nodes[2].body {
            driver.output = Id(20);
        }
        assert_names(&broken, Rule::AtMostOneBaseDriver);

        // One or the other alone is ordinary.
        let mut physics_only = broken.clone();
        physics_only.channel_mappings.clear();
        assert_eq!(rules(&physics_only), Vec::new());
    }

    // --------------------------------------------------------- physics

    /// Driven over `PhysicsConstant::ALL`, so a constant added to the format is
    /// covered here without a new case.
    #[test]
    fn every_physics_constant_is_range_validated_and_names_itself() {
        for constant in PhysicsConstant::ALL {
            let range = declared_range(*constant).expect("a stated range");
            let degenerate = if range.min_inclusive {
                range.min - 1.0
            } else {
                range.min
            };

            let mut broken = subject();
            if let NodeBody::Physics(node) = &mut broken.nodes[2].body {
                match constant {
                    PhysicsConstant::Length => node.length = degenerate,
                    PhysicsConstant::Gravity => node.gravity = degenerate,
                    PhysicsConstant::Frequency => node.frequency = degenerate,
                    PhysicsConstant::AngleDamping => node.angle_damping = degenerate,
                    PhysicsConstant::LengthDamping => node.length_damping = degenerate,
                }
            }
            let found = validate(&broken).expect_err("a degenerate constant is rejected");
            let named = found
                .iter()
                .find(|broken| broken.rule == Rule::PhysicsConstantWithinRange)
                .unwrap_or_else(|| panic!("{constant:?} was not range-validated"));
            assert_eq!(named.field.as_deref(), Some(constant.as_str()));
            assert!(named.offenders.contains(&Id(4)), "the node");
        }
    }

    /// Turning gravity off for one strand is the documented way to make a spike,
    /// so zero must be admitted where the format admits it.
    #[test]
    fn zero_is_admitted_where_the_format_admits_it() {
        let mut spike = subject();
        if let NodeBody::Physics(node) = &mut spike.nodes[2].body {
            node.gravity = 0.0;
            node.angle_damping = 0.0;
            node.length_damping = 0.0;
        }
        assert_eq!(rules(&spike), Vec::new());
    }

    // --------------------------------------------------------- numbers

    #[test]
    fn a_non_finite_number_is_rejected_wherever_it_sits() {
        let mut rotation = subject();
        rotation.nodes[1].transform.rotation = f64::INFINITY;
        assert_names(&rotation, Rule::NumbersFinite);

        let mut vertex = subject();
        if let NodeBody::Part(part) = &mut vertex.nodes[1].body {
            part.mesh.vertices[0] = [f64::NAN, 0.0];
        }
        assert_names(&vertex, Rule::NumbersFinite);

        let mut keyframe = subject();
        keyframe.animations[0].tracks[0].keyframes[0].time = f64::NEG_INFINITY;
        assert_names(&keyframe, Rule::NumbersFinite);

        let mut curve = subject();
        curve.channel_mappings[0].response_curve.points[0] = [f64::INFINITY, 0.0];
        assert_names(&curve, Rule::NumbersFinite);
    }

    /// `VAL-18`: judged at the precision evaluation uses. A value finite as a
    /// double and infinite as a single is what two tools reading one file would
    /// otherwise disagree about.
    #[test]
    fn finiteness_is_judged_at_the_precision_evaluation_uses() {
        assert!(finite(3.4e38));
        assert!(!finite(3.5e38), "finite at 64 bits, infinite at 32");
        assert!(!finite(f64::INFINITY));
        assert!(!finite(f64::NAN));

        let mut broken = subject();
        broken.nodes[1].transform.translation = [3.5e38, 0.0];
        assert_names(&broken, Rule::NumbersFinite);
    }

    // ---------------------------------------- the version gate is not here

    /// **`validate` admits an unfamiliar enumerated value unconditionally**, and
    /// this test is the record of why rather than an incidental assertion.
    ///
    /// Whether one is legal depends on the file's declared version, and a
    /// `Puppet` carries none — deliberately, because a save decides whether it is
    /// upgrading by reading the file it replaces. Gating it here would let
    /// `VER-1` load a later-minor file that could then never be saved, which is
    /// the compatibility rule inverted. The gate lives where the declaration is
    /// readable: `load` and `validate_file`, through `crate::decode`.
    #[test]
    fn an_unfamiliar_enumerated_value_is_admitted_in_memory() {
        let mut later = subject();
        later.parameters[0].axes[0].polarity = AxisPolarity::Unfamiliar(String::from("radial"));
        later.composition_mode = CompositionMode::Unfamiliar(String::from("multiply"));
        if let NodeBody::Physics(node) = &mut later.nodes[2].body {
            node.output = Id(20);
            node.output_source = vec![crate::puppet::OutputSource::Unfamiliar(String::from(
                "torque",
            ))];
            node.output_scale = vec![1.0];
        }
        later.channel_mappings.clear();
        assert_eq!(rules(&later), Vec::new());
    }

    /// The declared value *unknown* is a value, not an absence, and is valid.
    #[test]
    fn the_declared_value_unknown_is_valid() {
        let mut puppet = subject();
        puppet.parameters[0].axes[0].polarity = AxisPolarity::Unknown;
        assert_eq!(rules(&puppet), Vec::new());
    }

    // ------------------------------------------------- bounded output

    #[test]
    fn the_verdict_is_bounded_however_broken_the_puppet_is() {
        let mut broken = subject();
        for n in 0..4096u64 {
            broken
                .nodes
                .push(node(1000 + n, 999, NodeBody::Part(part())));
        }
        let found = validate(&broken).expect_err("every one of those is dangling");
        assert!(
            found.len() <= MAX_VIOLATIONS,
            "{} violations is not bounded output",
            found.len()
        );
    }

    // -------------------------------------------------- the whole corpus

    fn corpus() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    /// **Every fixture reaches the outcome `catalogue.json` records for it**,
    /// through the standalone verdict.
    ///
    /// The sweep task 5 added for the reader, carried to the validator, which is
    /// where the other direction becomes checkable: a fixture the catalogue calls
    /// invalid is refused *here* if it was not refused earlier, so a rule quietly
    /// left out shows up as a fixture that loads.
    #[test]
    fn every_fixture_reaches_its_catalogued_outcome() {
        let directory = corpus();
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
            let accepted =
                row.get("expect").and_then(crate::json::Json::as_str) == Some("accepted");

            let verdict = validate_file(&directory.join(file));
            match (&verdict, accepted) {
                (Ok(()), true) | (Err(_), false) => {}
                (Ok(()), false) => panic!("{name} is catalogued rejected and was accepted"),
                (Err(rejection), true) => panic!("{name} is catalogued accepted: {rejection}"),
            }

            if let Err(rejection) = verdict {
                assert!(
                    !rejection.rule_name().is_empty(),
                    "{name} is refused without naming a rule"
                );
                let rendered = rejection.to_string();
                assert!(
                    !rendered.is_empty()
                        && rendered.len() <= crate::load::Rejection::MAX_MESSAGE_BYTES,
                    "{name}'s message is {} bytes, which is not bounded",
                    rendered.len()
                );
                if !rejection.violations().is_empty() {
                    assert!(
                        !rejection.offenders().is_empty(),
                        "{name} breaks a rule and names no offending identifier"
                    );
                }
            }
        }
    }

    /// The standalone verdict and the loader's are the same verdict on every
    /// fixture, which is the whole of `VAL-13` and the reason both go through
    /// [`verdict`].
    #[test]
    fn the_standalone_verdict_and_the_loaders_agree() {
        let directory = corpus();
        let raw = std::fs::read(directory.join("catalogue.json")).expect("the catalogue reads");
        let catalogue = crate::json::parse(&raw).expect("the catalogue is JSON");
        for row in catalogue
            .get("fixtures")
            .and_then(crate::json::Json::as_array)
            .expect("a fixtures array")
        {
            let name = row
                .get("name")
                .and_then(crate::json::Json::as_str)
                .expect("a name");
            let path = directory.join(
                row.get("file")
                    .and_then(crate::json::Json::as_str)
                    .expect("a file"),
            );
            assert_eq!(
                crate::load::load(&path).is_ok(),
                validate_file(&path).is_ok(),
                "{name}: the standalone verdict and the loader's disagree"
            );
        }
    }

    // ------------------------------------------------------- the schema

    fn schema() -> crate::json::Json {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schema")
            .join("puppet.schema.json");
        let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("{path:?} reads: {e}"));
        crate::json::parse(&raw).expect("the schema is JSON")
    }

    /// The members one `$defs` object lists, with `x` dropped: the marker is
    /// legal on any object and is not a member the core format defines.
    fn members(schema: &crate::json::Json, definition: &str) -> Vec<String> {
        let mut found: Vec<String> = schema
            .get("$defs")
            .and_then(|defs| defs.get(definition))
            .and_then(|object| object.get("properties"))
            .and_then(crate::json::Json::as_object)
            .unwrap_or_else(|| panic!("the schema defines no {definition} with properties"))
            .iter()
            .map(|(name, _)| name.clone())
            .filter(|name| name != "x")
            .collect();
        found.sort();
        found
    }

    fn expected(keys: &[&str]) -> Vec<String> {
        let mut out: Vec<String> = keys.iter().map(|key| (*key).to_owned()).collect();
        out.sort();
        out
    }

    /// **The committed schema and the reader name the same members.**
    ///
    /// The schema is the half of validation a third party can run without our
    /// code, which makes it exactly the kind of artifact that drifts: nothing
    /// compiles it and nothing loads it. Holding it against `crate::decode`'s key
    /// sets — which are what the reader accepts and what `Rule::UnmarkedData`
    /// is judged against — turns a member added to one and not the other into a
    /// failure here.
    #[test]
    fn the_schema_lists_the_members_the_reader_knows() {
        use crate::decode::{
            ANIMATION_KEYS, AXIS_KEYS, BINDING_KEYS, CHAIN_KEYS, CURVE_KEYS, EXTENSION_KEYS,
            FLIP_KEYS, JOINT_KEYS, KEYFRAME_KEYS, KEYPOINT_KEYS, MAPPING_KEYS, MESH_KEYS,
            NODE_KEYS, PARAMETER_KEYS, PROVENANCE_KEYS, PUPPET_KEYS, TEXTURE_KEYS, TRACK_KEYS,
            TRANSFORM_KEYS,
        };

        let schema = schema();
        for (definition, keys) in [
            ("puppet", PUPPET_KEYS),
            ("node", NODE_KEYS),
            ("transform", TRANSFORM_KEYS),
            ("mesh", MESH_KEYS),
            ("texture_flip", FLIP_KEYS),
            ("provenance", PROVENANCE_KEYS),
            ("chain_deformer", CHAIN_KEYS),
            ("joint", JOINT_KEYS),
            ("parameter", PARAMETER_KEYS),
            ("axis", AXIS_KEYS),
            ("keypoint", KEYPOINT_KEYS),
            ("binding", BINDING_KEYS),
            ("animation", ANIMATION_KEYS),
            ("track", TRACK_KEYS),
            ("keyframe", KEYFRAME_KEYS),
            ("channel_mapping", MAPPING_KEYS),
            ("response_curve", CURVE_KEYS),
            ("texture", TEXTURE_KEYS),
        ] {
            assert_eq!(
                members(&schema, definition),
                expected(keys),
                "the schema's {definition} and the reader's key set disagree"
            );
        }

        // An extension entry is the one object keyed by its own namespace, so
        // its members sit under `additionalProperties`.
        let mut entry: Vec<String> = schema
            .get("$defs")
            .and_then(|defs| defs.get("extensions"))
            .and_then(|object| object.get("additionalProperties"))
            .and_then(|object| object.get("properties"))
            .and_then(crate::json::Json::as_object)
            .expect("the schema defines an extension entry")
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        entry.sort();
        assert_eq!(entry, expected(EXTENSION_KEYS));
    }

    /// A node's members depend on its kind, so the schema states them as
    /// conditionals — and each branch must carry that kind's key set.
    #[test]
    fn the_schemas_node_branches_match_the_kinds_the_reader_knows() {
        use crate::decode::{KNOWN_NODE_KINDS, MESH_GROUP_KEYS, PART_KEYS, PHYSICS_KEYS};

        let schema = schema();
        let node = schema
            .get("$defs")
            .and_then(|defs| defs.get("node"))
            .expect("the schema defines a node");

        let mut kinds: Vec<String> = node
            .get("properties")
            .and_then(|properties| properties.get("kind"))
            .and_then(|kind| kind.get("enum"))
            .and_then(crate::json::Json::as_array)
            .expect("the kind enum")
            .iter()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect();
        kinds.sort();
        assert_eq!(
            kinds,
            expected(KNOWN_NODE_KINDS),
            "the schema's node kinds and the reader's disagree"
        );

        let branches = node
            .get("allOf")
            .and_then(crate::json::Json::as_array)
            .expect("one branch per kind");
        for (kind, keys) in [
            ("part", PART_KEYS),
            ("mesh_group", MESH_GROUP_KEYS),
            ("composite", &[] as &[&str]),
            ("physics", PHYSICS_KEYS),
        ] {
            let branch = branches
                .iter()
                .find(|branch| {
                    branch
                        .get("if")
                        .and_then(|guard| guard.get("properties"))
                        .and_then(|properties| properties.get("kind"))
                        .and_then(|held| held.get("const"))
                        .and_then(crate::json::Json::as_str)
                        == Some(kind)
                })
                .unwrap_or_else(|| panic!("the schema has no branch for {kind}"));
            let mut found: Vec<String> = branch
                .get("then")
                .and_then(|then| then.get("properties"))
                .and_then(crate::json::Json::as_object)
                .unwrap_or_else(|| panic!("{kind}'s branch states no properties"))
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            found.sort();
            assert_eq!(found, expected(keys), "{kind}'s members in the schema");
        }
    }

    /// Every rule has a row in the schema's boundary document, saying whether a
    /// schema can express it.
    ///
    /// That document is the whole point of committing a schema — a third party
    /// who runs it has to know what they have *not* checked — and a rule added
    /// to [`Rule`] with no row leaves them a silent gap.
    #[test]
    fn the_schema_boundary_document_accounts_for_every_rule() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schema")
            .join("README.md");
        let boundary = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        for rule in EVERY_RULE {
            assert!(
                boundary.contains(rule.as_str()),
                "{}'s side of the schema boundary is not stated in schema/README.md",
                rule.as_str()
            );
        }
    }

    /// Every variant of [`Rule`], for the test above.
    ///
    /// Written out rather than derived, so that adding a variant fails to
    /// compile the `match` below until it is added here too.
    const EVERY_RULE: &[Rule] = &[
        Rule::IdentifiersUnique,
        Rule::IdentifierWithinHighWaterMark,
        Rule::IdentifierZeroReserved,
        Rule::TreeAcyclic,
        Rule::ExactlyOneRoot,
        Rule::RootIsAKindThatContains,
        Rule::Containment,
        Rule::ReferenceResolves,
        Rule::KeypointsOrdered,
        Rule::KeypointWithinDeclaredRange,
        Rule::ExactlyOneNeutralKeypoint,
        Rule::AxisPolarityDeclared,
        Rule::EnumeratedValueKnown,
        Rule::UnmarkedData,
        Rule::ExtensionDeclaresVersion,
        Rule::ExtensionDeclaresName,
        Rule::AtMostOneBaseDriver,
        Rule::BindingValueCountAgreesWithStaleMark,
        Rule::NumbersFinite,
        Rule::KeyframesOrdered,
        Rule::KeyframeWithinDeclaredRange,
        Rule::BlendDurationNonNegative,
        Rule::AtMostOneTrackPerParameter,
        Rule::PhysicsConstantWithinRange,
    ];

    /// [`EVERY_RULE`] really is every rule: the `match` is exhaustive, so a
    /// variant added to [`Rule`] stops this compiling until it is listed.
    #[test]
    fn every_rule_is_listed() {
        for rule in EVERY_RULE {
            // The arms are the point; the value is not.
            let _: &'static str = match rule {
                Rule::IdentifiersUnique => "",
                Rule::IdentifierWithinHighWaterMark => "",
                Rule::IdentifierZeroReserved => "",
                Rule::TreeAcyclic => "",
                Rule::ExactlyOneRoot => "",
                Rule::RootIsAKindThatContains => "",
                Rule::Containment => "",
                Rule::ReferenceResolves => "",
                Rule::KeypointsOrdered => "",
                Rule::KeypointWithinDeclaredRange => "",
                Rule::ExactlyOneNeutralKeypoint => "",
                Rule::AxisPolarityDeclared => "",
                Rule::EnumeratedValueKnown => "",
                Rule::UnmarkedData => "",
                Rule::ExtensionDeclaresVersion => "",
                Rule::ExtensionDeclaresName => "",
                Rule::AtMostOneBaseDriver => "",
                Rule::BindingValueCountAgreesWithStaleMark => "",
                Rule::NumbersFinite => "",
                Rule::KeyframesOrdered => "",
                Rule::KeyframeWithinDeclaredRange => "",
                Rule::BlendDurationNonNegative => "",
                Rule::AtMostOneTrackPerParameter => "",
                Rule::PhysicsConstantWithinRange => "",
            };
        }
        let mut names: Vec<&str> = EVERY_RULE.iter().map(|rule| rule.as_str()).collect();
        names.sort_unstable();
        let listed = names.len();
        names.dedup();
        assert_eq!(listed, names.len(), "two rules share a name");
    }
}
