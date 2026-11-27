//! What each node kind may contain, as **data the validator reads**.
//!
//! `docs/format.md` states the table and says why it is a table: a containment
//! rule spelled out in code has to be found and edited by whoever adds a fifth
//! kind, and whoever forgets leaves a kind that may contain anything. A table
//! turns that omission into a missing row, which a check can refuse (`VAL-24`).

/// The kinds of node the format defines.
///
/// A kind added later is added to [`NodeKind::ALL`], which is what makes
/// [`may_contain`] and [`may_be_root`] fail to be total until somebody states its
/// answer (`VAL-24`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    Part,
    MeshGroup,
    Composite,
    Physics,
}

impl NodeKind {
    /// Every kind the format defines. The acceptance suite drives `VAL-23` over
    /// the square of this, so a kind added here is covered without a new test.
    pub const ALL: &'static [NodeKind] = &[
        NodeKind::Part,
        NodeKind::MeshGroup,
        NodeKind::Composite,
        NodeKind::Physics,
    ];
}

/// Whether a node of `child` may sit directly inside a node of `parent`.
///
/// Read from the declaration, in both directions: every pair it forbids is
/// rejected naming the rule and the offending node, and every pair it permits
/// validates, loads and saves (`VAL-23`). *A part is a leaf* (`VAL-16`) and
/// *physics contains only physics* (`VAL-22`) are two rows of it, not two rules.
///
/// `None` means the declaration has no answer for that pair, which **fails the
/// check** rather than meaning *no*: a kind added later must arrive with a stated
/// answer instead of inheriting a default nobody chose (`VAL-24`). A build that
/// refuses every pair it has no rule for satisfies the rejection half of
/// `VAL-23` and breaks a composite inside a mesh group, which is permitted.
pub fn may_contain(_parent: NodeKind, _child: NodeKind) -> Option<bool> {
    unimplemented!()
}

/// Whether a node of this kind may be the puppet's root.
///
/// The root is a kind that contains, because a puppet rooted at a part would have
/// to be re-rooted the moment a second part arrived (`docs/format.md`).
///
/// `None` for a kind with no stated answer, for the reason [`may_contain`] gives.
pub fn may_be_root(_kind: NodeKind) -> Option<bool> {
    unimplemented!()
}
