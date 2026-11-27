//! Parsing, validation and canonical writing of a puppet file.
//!
//! No GPU, no window, no toolkit — see `docs/stack.md`.
//!
//! # This was a contract, and is being filled in
//!
//! The surface was declared by `acceptance-planner` from the criteria in
//! `docs/requirements/puppet-acceptance.md`, before any implementation existed and
//! without sight of one, so that the acceptance tests in `tests/acceptance/` have
//! something to compile against. Tasks 4 to 7 of `docs/wip/s2-format.md` fill it
//! in. **The writer and the validator's algorithm are still `unimplemented!()`**
//! — [`save`] is task 7 and [`validate`] and [`validate_file`] are task 6 — so a
//! load that gets as far as validating panics there rather than returning.
//!
//! Each item says what it must do and names the criterion that drives it. Where
//! this surface and a criterion disagree, **the criterion wins** — this crate is
//! derived from it, not the other way round.
//!
//! # The shape of a load
//!
//! [`load`] takes a path and **returns a value**. It never mutates anything the
//! caller already holds, which is how
//! [I4](../../../docs/invariants.md) — *a load that fails never replaces what is
//! already displayed* — is met structurally rather than by care: a caller that
//! holds a puppet and asks for another cannot have lost the first, because the
//! failing call had nothing to lose it through.
//!
//! # The two error families
//!
//! [`Rejection`] is why a file was refused: a malformed container, a major
//! version this build does not implement, a load limit, unparsable JSON, or a
//! list of [`Violation`]s. [`SaveError`] is why a save was refused, and the
//! validation half of it carries the same [`Violation`]s — the validator is one
//! validator, gating load and save alike (`VAL-13`).

#![forbid(unsafe_code)]

mod archive;
pub mod containment;
mod decode;
mod json;
pub mod limits;
pub mod load;
pub mod puppet;
pub mod save;
pub mod validate;
pub mod version;

pub use containment::{NodeKind, may_be_root, may_contain};
pub use limits::{Basis, Figure, LimitKind, declared_limit};
pub use load::{
    ArchiveFault, ExtensionNotice, LoadReport, Loaded, Rejection, VersionGap, load, read_version,
};
pub use puppet::{
    Animation, Axis, AxisPolarity, Binding, BindingKind, BindingValues, ChainDeformer,
    ChannelMapping, CompositionMode, ExtensionEntry, Extensions, Id, Joint, Keyframe, Keypoint,
    Mesh, MeshGroup, Node, NodeBody, OpaqueNode, OutputSource, Parameter, Part, PhysicsNode,
    Preserved, PreservedMembers, Provenance, Puppet, ResponseCurve, TextureFlip, TextureRecord,
    Track, Transform,
};
pub use save::{SaveError, SaveOutcome, SaveStep, save, save_as, save_with_injected_failure};
pub use validate::{
    PhysicsConstant, Rule, ValidRange, Violation, declared_range, validate, validate_file,
};
pub use version::FormatVersion;
