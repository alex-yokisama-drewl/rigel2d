//! Section A — every entity has a stable identity.
//!
//! Asserts [I3](../../../../docs/invariants.md): every reference in a stored
//! puppet is by stable identifier, never by name, index or position.

use std::collections::BTreeSet;
use std::path::Path;

use rigel_format::{
    Axis, AxisPolarity, BindingValues, Id, Keypoint, Mesh, Node, NodeBody, Parameter, Part, Puppet,
    Transform, load, save, save_as,
};

use crate::support::{bytes, fixture, scratch};

/// Loads a fixture by catalogue name, failing loudly rather than skipping.
fn puppet(name: &str) -> Puppet {
    load(&fixture(name))
        .unwrap_or_else(|e| panic!("{name} is a valid puppet and must load: {e}"))
        .puppet
}

fn save_to(puppet: &Puppet, dir: &Path, file: &str) -> std::path::PathBuf {
    let path = dir.join(file);
    save(puppet, &path).unwrap_or_else(|e| panic!("the save succeeds: {e:?}"));
    path
}

fn reload(path: &Path) -> Puppet {
    load(path)
        .unwrap_or_else(|e| panic!("what we just wrote must load: {e}"))
        .puppet
}

fn id_set(puppet: &Puppet) -> BTreeSet<Id> {
    puppet.identifiers().into_iter().collect()
}

/// Every stored reference, as *(what kind of reference, from, to)*.
///
/// Enumerated here rather than asserted one at a time, so that `ID-2` and `ID-3`
/// assert *every* reference resolves to the same entity rather than the handful
/// a test author thought of.
fn references(puppet: &Puppet) -> Vec<(&'static str, Id, Id)> {
    let mut out = Vec::new();
    out.push(("puppet.root", puppet.id, puppet.root));
    for node in &puppet.nodes {
        out.push(("node.parent", node.id, node.parent));
        match &node.body {
            NodeBody::Part(part) => {
                out.push(("part.texture", node.id, part.texture));
                out.push(("part.mask", node.id, part.mask));
            }
            NodeBody::Physics(physics) => out.push(("physics.output", node.id, physics.output)),
            NodeBody::MeshGroup(_) | NodeBody::Composite => {}
        }
    }
    for binding in &puppet.bindings {
        out.push(("binding.parameter", binding.id, binding.parameter));
        out.push(("binding.target", binding.id, binding.target));
    }
    for animation in &puppet.animations {
        for track in &animation.tracks {
            out.push(("track.parameter", track.id, track.parameter));
        }
    }
    for mapping in &puppet.channel_mappings {
        out.push(("mapping.parameter", mapping.id, mapping.parameter));
    }
    out.sort();
    out
}

/// ID-1 — every identifier recorded before a round trip is recovered after it,
/// member for member, enumerated from the puppet rather than from a list kept
/// here.
#[test]
fn id_1_every_identifier_survives_a_round_trip() {
    let before = puppet("rigged");
    let dir = scratch("id-1");
    let written = save_to(&before, &dir, "round-trip.rigel2d");
    let after = reload(&written);

    assert!(
        !before.identifiers().is_empty(),
        "the base fixture carries entities, so it carries identifiers"
    );
    assert_eq!(
        id_set(&before),
        id_set(&after),
        "the identifier set must be recovered exactly"
    );
}

/// ID-2 — a rename moves no identifier and breaks no reference. Names are for
/// people and change; identifiers do not.
#[test]
fn id_2_renaming_every_entity_moves_no_identifier() {
    let mut subject = puppet("rigged");
    let before_ids = id_set(&subject);
    let before_refs = references(&subject);

    subject.name = "renamed-puppet".to_owned();
    for (n, node) in subject.nodes.iter_mut().enumerate() {
        node.name = format!("renamed-node-{n}");
    }
    for (n, parameter) in subject.parameters.iter_mut().enumerate() {
        parameter.name = format!("renamed-parameter-{n}");
    }
    for (n, animation) in subject.animations.iter_mut().enumerate() {
        animation.name = format!("renamed-animation-{n}");
    }
    for (n, texture) in subject.textures.iter_mut().enumerate() {
        texture.name = format!("renamed-texture-{n}");
    }

    let dir = scratch("id-2");
    let after = reload(&save_to(&subject, &dir, "renamed.rigel2d"));

    assert_eq!(before_ids, id_set(&after), "a rename issues no identifier");
    assert_eq!(
        before_refs,
        references(&after),
        "every reference still names the entity it named before the rename"
    );
}

/// ID-3 — a reorder moves no identifier and breaks no reference, and draw order
/// is the one thing order decides.
///
/// Keypoints are reversed together with the binding values stored against them,
/// because the stored form indexes a binding's values positionally by keypoint —
/// reversing one without the other would be a change of meaning rather than a
/// reorder. See the finding on `ID-3` in `docs/wip/s2-format.md`.
#[test]
fn id_3_reordering_moves_no_identifier_and_only_draw_order_follows() {
    let original = puppet("rigged");
    let mut subject = original.clone();
    let before_ids = id_set(&subject);
    let before_refs = references(&subject);
    let sibling_order: Vec<Id> = subject.nodes.iter().map(|n| n.id).collect();

    subject.nodes.reverse();
    subject.parameters.reverse();
    subject.textures.reverse();
    subject.bindings.reverse();
    for parameter in &mut subject.parameters {
        parameter.keypoints.reverse();
        for binding in &mut subject.bindings {
            if binding.parameter != parameter.id {
                continue;
            }
            match &mut binding.values {
                BindingValues::Transform(values) => values.reverse(),
                BindingValues::Deformation(values) => values.reverse(),
            }
        }
    }

    let dir = scratch("id-3");
    let after = reload(&save_to(&subject, &dir, "reordered.rigel2d"));

    assert_eq!(before_ids, id_set(&after), "a reorder issues no identifier");
    assert_eq!(
        before_refs,
        references(&after),
        "every reference still names the entity it named before the reorder"
    );

    let after_order: Vec<Id> = after.nodes.iter().map(|n| n.id).collect();
    assert_ne!(
        sibling_order, after_order,
        "draw order follows the tree's new order — it is the one thing order decides"
    );
    for parent in sibling_order.iter() {
        let before_children: Vec<Id> = original
            .nodes
            .iter()
            .filter(|n| n.parent == *parent)
            .map(|n| n.id)
            .collect();
        let mut expected = before_children.clone();
        expected.reverse();
        let after_children: Vec<Id> = after
            .nodes
            .iter()
            .filter(|n| n.parent == *parent)
            .map(|n| n.id)
            .collect();
        assert_eq!(
            expected, after_children,
            "the children of {parent:?} come back in the order they were put in"
        );
    }

    let mut canonical = after.parameters.clone();
    canonical.sort_by_key(|p| p.id);
    assert_eq!(
        original.parameters, canonical,
        "keypoints and the values stored against them come back aligned"
    );
}

/// ID-4 — an identifier is never reused after a deletion, within one run and
/// across a reload.
///
/// The collection route into this (`COLL-6`) is Studio's and out of scope here.
#[test]
fn id_4_a_deleted_identifier_is_never_reissued() {
    let dir = scratch("id-4");
    let mut subject = puppet("rigged");
    let mut retired: BTreeSet<Id> = BTreeSet::new();

    for round in 0..2 {
        // Delete a part and a parameter, with the references that would dangle.
        let doomed_part = subject
            .nodes
            .iter()
            .rev()
            .find(|n| matches!(n.body, NodeBody::Part(_)))
            .expect("a part to delete")
            .id;
        let doomed_parameter = subject.parameters.last().expect("a parameter to delete").id;

        retired.insert(doomed_part);
        retired.insert(doomed_parameter);
        retired.extend(
            subject
                .parameters
                .iter()
                .filter(|p| p.id == doomed_parameter)
                .flat_map(|p| p.keypoints.iter().map(|k| k.id)),
        );
        retired.extend(
            subject
                .bindings
                .iter()
                .filter(|b| b.parameter == doomed_parameter || b.target == doomed_part)
                .map(|b| b.id),
        );

        subject.nodes.retain(|n| n.id != doomed_part);
        subject.parameters.retain(|p| p.id != doomed_parameter);
        subject
            .bindings
            .retain(|b| b.parameter != doomed_parameter && b.target != doomed_part);
        subject
            .channel_mappings
            .retain(|m| m.parameter != doomed_parameter);
        for animation in &mut subject.animations {
            animation.tracks.retain(|t| t.parameter != doomed_parameter);
        }
        for node in &mut subject.nodes {
            match &mut node.body {
                NodeBody::Part(part) if part.mask == doomed_part => part.mask = Id::NONE,
                NodeBody::Physics(physics) if physics.output == doomed_parameter => {
                    physics.output = Id::NONE;
                    physics.output_source.clear();
                    physics.output_scale.clear();
                }
                _ => {}
            }
        }

        let after_delete = reload(&save_to(
            &subject,
            &dir,
            &format!("after-delete-{round}.rigel2d"),
        ));
        subject = after_delete;

        // Create a part and a parameter, and assert nothing retired came back.
        let texture = subject.textures.first().expect("a texture").id;
        let fresh_node = subject.issue_identifier();
        let fresh_parameter = subject.issue_identifier();
        let fresh_keypoint = subject.issue_identifier();
        for issued in [fresh_node, fresh_parameter, fresh_keypoint] {
            assert!(
                !retired.contains(&issued),
                "{issued:?} was retired and must never be issued again"
            );
            assert_ne!(issued, Id::NONE, "the reserved zero is never issued");
        }

        subject.nodes.push(Node {
            id: fresh_node,
            parent: subject.root,
            name: format!("fresh-part-{round}"),
            transform: Transform {
                translation: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            },
            body: NodeBody::Part(Part {
                mesh: Mesh {
                    vertices: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                    uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                    triangles: vec![[0, 1, 2]],
                },
                texture,
                texture_flip: Default::default(),
                mask: Id::NONE,
                provenance: None,
                chain_deformer: None,
            }),
            extensions: Default::default(),
        });
        subject.parameters.push(Parameter {
            id: fresh_parameter,
            name: format!("fresh-parameter-{round}"),
            axes: vec![Axis {
                range: [-1.0, 1.0],
                polarity: AxisPolarity::Unknown,
            }],
            keypoints: vec![Keypoint {
                id: fresh_keypoint,
                position: vec![0.0],
                neutral: true,
            }],
        });

        subject = reload(&save_to(
            &subject,
            &dir,
            &format!("after-create-{round}.rigel2d"),
        ));

        let live = id_set(&subject);
        for id in &retired {
            assert!(
                !live.contains(id),
                "{id:?} was deleted and has been reissued after a reload"
            );
        }
    }
}

/// ID-5 — a duplicate identifier is rejected, within one kind and across kinds.
#[test]
fn id_5_duplicate_identifiers_are_rejected() {
    for name in ["rigged:dup-id-same-kind", "rigged:dup-id-cross-kind"] {
        crate::support::assert_rejected_naming(name, rigel_format::Rule::IdentifiersUnique);
    }
}

/// ID-6 — the puppet's own identifier survives an edit and a rename. It is what
/// a scene confirms the file against.
#[test]
fn id_6_the_puppets_own_identifier_survives_being_edited() {
    let mut subject = puppet("rigged");
    let before = subject.id;

    let fresh = subject.issue_identifier();
    let template = subject
        .nodes
        .iter()
        .find(|n| matches!(n.body, NodeBody::Part(_)))
        .expect("a part to copy")
        .clone();
    subject.nodes.push(Node {
        id: fresh,
        name: "added-part".to_owned(),
        ..template
    });
    subject.name = "renamed-after-editing".to_owned();

    let dir = scratch("id-6");
    let after = reload(&save_to(&subject, &dir, "edited.rigel2d"));

    assert_eq!(
        before, after.id,
        "the puppet's identifier is unchanged by editing and renaming it"
    );
}

/// ID-9 — the identifier identifies the **puppet**, not the file. A copy carries
/// the same one; a *save as* is the deliberate act that issues a new one.
///
/// The half about a scene reference resolving to either is the scene's, and is
/// out of scope in S2.
#[test]
fn id_9_a_copy_keeps_the_identifier_and_a_save_as_issues_a_new_one() {
    let original = fixture("rigged");
    let dir = scratch("id-9");
    let copy = dir.join("copy.rigel2d");
    std::fs::copy(&original, &copy).expect("the file copies");

    let from_original = reload(&original);
    let from_copy = reload(&copy);
    assert_eq!(
        from_original.id, from_copy.id,
        "copying a file does not make a different puppet"
    );
    assert_eq!(
        bytes(&original),
        bytes(&copy),
        "the copy is the same file, so nothing else can explain a difference"
    );

    let mut saved_as = from_original.clone();
    let elsewhere = dir.join("saved-as.rigel2d");
    save_as(&mut saved_as, &elsewhere).unwrap_or_else(|e| panic!("the save as succeeds: {e:?}"));
    assert_ne!(
        from_original.id, saved_as.id,
        "a save as is the deliberate act of making a different puppet"
    );
    assert_eq!(
        saved_as.id,
        reload(&elsewhere).id,
        "the new identifier is what was written"
    );
}

/// ID-8 — the catch-all that fails when a kind is forgotten entirely.
///
/// Every piece *What a puppet is* lists is present after a round trip and equal
/// to what arrived: masks, chain deformers and their joints, channel mappings,
/// the tracks each animation is composed of, and the puppet's gravity as much as
/// the node tree, textures, parameters, bindings and animations. **Physics is a
/// kind of node**, so it is asserted as part of the tree and with its place in
/// it. A mask is a reference rather than a free-standing entity, so it is
/// asserted as a reference that still resolves.
///
/// `ID-7` is the other half of this section and is `judgment`: no fixture can
/// carry a reference the format does not define, so there is nothing to assert
/// against — it is read at review against the specification.
#[test]
fn id_8_every_piece_of_a_puppet_survives_a_round_trip() {
    let before = puppet("rigged");
    let dir = scratch("id-8");
    let after = reload(&save_to(&before, &dir, "pieces.rigel2d"));

    // Each piece is carried by the base fixture, so an empty one here is a
    // kind that was dropped rather than a fixture that never had it.
    assert!(!before.nodes.is_empty(), "the node tree");
    assert!(!before.textures.is_empty(), "textures");
    assert!(!before.parameters.is_empty(), "parameters");
    assert!(!before.bindings.is_empty(), "bindings");
    assert!(!before.animations.is_empty(), "animations");
    assert!(!before.channel_mappings.is_empty(), "channel mappings");
    assert!(
        before.animations.iter().any(|a| !a.tracks.is_empty()),
        "the tracks an animation is composed of"
    );
    assert!(
        before.nodes.iter().any(|n| matches!(
            &n.body,
            NodeBody::Part(part) if part.chain_deformer.as_ref().is_some_and(|c| !c.joints.is_empty())
        )),
        "a chain deformer and its joints"
    );
    assert!(
        before.nodes.iter().any(|n| matches!(
            &n.body, NodeBody::Part(part) if part.mask != Id::NONE
        )),
        "a mask, which is a reference rather than an entity"
    );
    assert!(
        before
            .nodes
            .iter()
            .any(|n| matches!(n.body, NodeBody::Physics(_))),
        "physics, which is a kind of node and sits in the tree"
    );

    assert_eq!(
        before, after,
        "every piece is present and equal after load, save, load"
    );

    // A mask is asserted as a reference that still resolves.
    let live: BTreeSet<Id> = after.nodes.iter().map(|n| n.id).collect();
    for node in &after.nodes {
        if let NodeBody::Part(part) = &node.body
            && part.mask != Id::NONE
        {
            assert!(
                live.contains(&part.mask),
                "the mask on {:?} still resolves after the round trip",
                node.id
            );
        }
    }
}
