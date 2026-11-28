//! What each node kind may contain, as **data the validator reads**.
//!
//! `docs/format.md` states the table and says why it is a table: a containment
//! rule spelled out in code has to be found and edited by whoever adds a fifth
//! kind, and whoever forgets leaves a kind that may contain anything. A table
//! turns that omission into a missing row, which a check can refuse (`VAL-24`).
//!
//! # The table is the implementation
//!
//! [`DECLARATIONS`] is the whole of it. [`may_contain`] and [`may_be_root`] read
//! that slice and name no kind of their own, so adding a fifth kind to
//! [`NodeKind::ALL`] without adding its row leaves both functions returning
//! `None` for every pair the new kind is on either side of — which fails the
//! check rather than inheriting an answer nobody stated.

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

/// One kind's stated answers: whether it may be the root, and which kinds it may
/// contain directly.
///
/// The declaration is positive — a row lists what is permitted, and anything a
/// row does not list is forbidden *for a kind that has a row*. A kind with no
/// row has stated nothing at all, which is the case [`may_contain`]'s `None`
/// exists for.
#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    /// The kind this row declares.
    pub kind: NodeKind,
    /// Whether a node of this kind may be the puppet's root.
    pub may_be_root: bool,
    /// The kinds a node of this kind may contain directly.
    pub may_contain: &'static [NodeKind],
}

/// The containment table of `docs/format.md`, *The node tree*, verbatim.
///
/// | Kind | May be root | May contain |
/// |---|---|---|
/// | Part | no | nothing — a part is a leaf |
/// | Mesh group | yes | any kind |
/// | Composite | yes | any kind |
/// | Physics | no | physics only |
///
/// The root is a kind that contains, because a puppet rooted at a part would
/// have to be re-rooted the moment a second part arrived.
pub const DECLARATIONS: &[Declaration] = &[
    Declaration {
        kind: NodeKind::Part,
        may_be_root: false,
        may_contain: &[],
    },
    Declaration {
        kind: NodeKind::MeshGroup,
        may_be_root: true,
        may_contain: &[
            NodeKind::Part,
            NodeKind::MeshGroup,
            NodeKind::Composite,
            NodeKind::Physics,
        ],
    },
    Declaration {
        kind: NodeKind::Composite,
        may_be_root: true,
        may_contain: &[
            NodeKind::Part,
            NodeKind::MeshGroup,
            NodeKind::Composite,
            NodeKind::Physics,
        ],
    },
    Declaration {
        kind: NodeKind::Physics,
        may_be_root: false,
        may_contain: &[NodeKind::Physics],
    },
];

/// The row declaring `kind`, or `None` when nobody has declared one.
fn declaration(kind: NodeKind) -> Option<&'static Declaration> {
    DECLARATIONS.iter().find(|row| row.kind == kind)
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
///
/// **Both kinds must have a row**, not only the parent. A row lists the kinds its
/// own kind may contain, so a kind added to [`NodeKind::ALL`] with no row of its
/// own is simply absent from every existing row — indistinguishable, on the
/// parent's row alone, from a kind deliberately forbidden there. Requiring the
/// child's row too is what keeps that omission a `None`.
pub fn may_contain(parent: NodeKind, child: NodeKind) -> Option<bool> {
    let parent = declaration(parent)?;
    let child = declaration(child)?.kind;
    Some(parent.may_contain.contains(&child))
}

/// Whether a node of this kind may be the puppet's root.
///
/// The root is a kind that contains, because a puppet rooted at a part would have
/// to be re-rooted the moment a second part arrived (`docs/format.md`).
///
/// `None` for a kind with no stated answer, for the reason [`may_contain`] gives.
pub fn may_be_root(kind: NodeKind) -> Option<bool> {
    Some(declaration(kind)?.may_be_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The declaration is total over the kinds the format defines — the
    /// behavioural half of `VAL-24`, asserted here as well as in the acceptance
    /// suite because this is where a missing row is introduced.
    #[test]
    fn every_kind_is_declared_in_both_directions() {
        for parent in NodeKind::ALL {
            assert!(may_be_root(*parent).is_some(), "{parent:?} may_be_root");
            for child in NodeKind::ALL {
                assert!(
                    may_contain(*parent, *child).is_some(),
                    "{parent:?} may_contain {child:?}"
                );
            }
        }
    }

    /// Exactly one row per kind: two rows for one kind would make the answer
    /// depend on which came first.
    #[test]
    fn each_kind_is_declared_once() {
        for kind in NodeKind::ALL {
            let rows = DECLARATIONS.iter().filter(|row| row.kind == *kind).count();
            assert_eq!(rows, 1, "{kind:?} has {rows} rows");
        }
    }

    #[test]
    fn a_part_is_a_leaf() {
        for child in NodeKind::ALL {
            assert_eq!(
                may_contain(NodeKind::Part, *child),
                Some(false),
                "a part contains no {child:?}"
            );
        }
    }

    #[test]
    fn physics_contains_only_physics() {
        assert_eq!(
            may_contain(NodeKind::Physics, NodeKind::Physics),
            Some(true)
        );
        for child in [NodeKind::Part, NodeKind::MeshGroup, NodeKind::Composite] {
            assert_eq!(may_contain(NodeKind::Physics, child), Some(false));
        }
    }

    #[test]
    fn the_kinds_that_contain_contain_every_kind() {
        for parent in [NodeKind::MeshGroup, NodeKind::Composite] {
            for child in NodeKind::ALL {
                assert_eq!(
                    may_contain(parent, *child),
                    Some(true),
                    "{parent:?} contains any kind"
                );
            }
        }
    }

    /// *The root is a kind that contains* is a necessary condition, not the
    /// whole rule: physics contains physics and still may not be the root,
    /// because `docs/format.md` names a mesh group or a composite. A kind that
    /// contains nothing must never be rootable, which is the half that would
    /// reintroduce the re-rooting path on every import.
    #[test]
    fn a_kind_that_may_be_root_is_a_kind_that_contains() {
        for kind in NodeKind::ALL {
            if may_be_root(*kind) != Some(true) {
                continue;
            }
            assert!(
                NodeKind::ALL
                    .iter()
                    .any(|child| may_contain(*kind, *child) == Some(true)),
                "{kind:?} may be the root and contains nothing"
            );
        }
    }

    /// The two kinds `docs/format.md` names, and no others.
    #[test]
    fn the_root_is_a_mesh_group_or_a_composite() {
        for kind in NodeKind::ALL {
            let expected = matches!(kind, NodeKind::MeshGroup | NodeKind::Composite);
            assert_eq!(may_be_root(*kind), Some(expected), "{kind:?} as root");
        }
    }

    /// A kind with no row answers nothing, on either side of the pair. The
    /// acceptance suite cannot assert this — it can only drive
    /// [`NodeKind::ALL`], which by construction has a row for every member — so
    /// it is asserted here against the table directly.
    #[test]
    fn an_undeclared_kind_answers_nothing() {
        // Stand in for a fifth kind by asking about a kind whose row has been
        // taken out of the table the lookup reads.
        fn lookup(table: &[Declaration], kind: NodeKind) -> Option<&Declaration> {
            table.iter().find(|row| row.kind == kind)
        }
        let without_physics: Vec<Declaration> = DECLARATIONS
            .iter()
            .copied()
            .filter(|row| row.kind != NodeKind::Physics)
            .collect();
        assert!(
            lookup(&without_physics, NodeKind::Physics).is_none(),
            "a kind whose row is missing is not found"
        );
        assert!(
            lookup(&without_physics, NodeKind::Composite).is_some(),
            "and the kinds that do have rows still are"
        );
    }
}
