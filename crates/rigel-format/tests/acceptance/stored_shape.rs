//! The stored shape round-trips — sections B, N, O, P and Q, in the half S2
//! owns.
//!
//! Several of these criteria have two halves: what the file carries, which is
//! this subsystem's, and what evaluation does with it, which is S4's or S5's.
//! Each test says which half it carries. `CHAIN-4` and `PHYS-5` are `judgment` —
//! a negative over the design, read at review — and are not here.

use std::collections::{BTreeMap, BTreeSet};

use rigel_format::{
    AxisPolarity, CompositionMode, Id, NodeBody, TextureFlip, load, save, validate, validate_file,
};

use crate::support::{
    bytes, dir_entries, entry_names, fixture, json_node, looks_like_png, png_dimensions,
    puppet_json, scratch,
};

fn puppet(name: &str) -> rigel_format::Puppet {
    load(&fixture(name))
        .unwrap_or_else(|e| panic!("{name} is valid and must load: {e}"))
        .puppet
}

fn round_trip(name: &str, label: &str) -> rigel_format::Puppet {
    let subject = puppet(name);
    let dir = scratch(label);
    let written = dir.join("round-trip.rigel2d");
    save(&subject, &written).unwrap_or_else(|e| panic!("{name} saves: {e:?}"));
    load(&written)
        .unwrap_or_else(|e| panic!("what we wrote loads: {e}"))
        .puppet
}

/// FILE-1 — one file. A puppet copied alone into an empty directory loads, and
/// nothing is resolved from outside it.
#[test]
fn file_1_a_puppet_alone_in_a_directory_loads() {
    let dir = scratch("file-1");
    let alone = crate::support::copy_fixture_into("rigged", &dir);
    assert_eq!(
        dir_entries(&dir).len(),
        1,
        "the directory holds the puppet and nothing else"
    );

    assert_eq!(
        puppet("rigged"),
        load(&alone).expect("it loads from anywhere").puppet,
        "nothing is resolved from outside the file"
    );
}

/// FILE-2 — a save leaves the puppet and nothing else: no temporary left behind,
/// no derived artifact that could be mistaken for a working copy.
///
/// The one exception is the version-upgrade backup, which `VER-4` asserts and
/// which this save is not.
#[test]
fn file_2_a_save_leaves_the_puppet_and_nothing_else() {
    let dir = scratch("file-2");
    let written = dir.join("saved.rigel2d");
    let outcome = save(&puppet("rigged"), &written).expect("it saves");

    assert!(
        outcome.backup.is_none(),
        "this save is not an upgrade, so nothing else is due"
    );
    assert_eq!(
        dir_entries(&dir),
        vec!["saved.rigel2d".to_owned()],
        "no temporary and no derived artifact survives the save"
    );
}

/// FILE-5 — import provenance is stored per part and round-trips, and contains
/// no path, username or machine name.
///
/// *Nothing in it is ever resolved* is `EXEC-1`'s and `EXEC-2`'s.
#[test]
fn file_5_import_provenance_is_stored_per_part_and_carries_no_machine() {
    let before = puppet("rigged");
    let after = round_trip("rigged", "file-5");

    let provenances: Vec<_> = before
        .nodes
        .iter()
        .filter_map(|n| match &n.body {
            NodeBody::Part(part) => part.provenance.as_ref(),
            _ => None,
        })
        .collect();
    assert!(
        !provenances.is_empty(),
        "the base fixture carries import provenance on every part"
    );
    for provenance in &provenances {
        assert!(
            !provenance.source_id.is_empty(),
            "a source identifier taken from the source document"
        );
        assert!(
            !provenance.content_hash.is_empty(),
            "and a content hash of the imported pixels"
        );
        for value in [&provenance.source_id, &provenance.content_hash] {
            assert!(
                !value.contains('/')
                    && !value.contains('\\')
                    && !value.starts_with('~')
                    && !value.contains(':'),
                "provenance carries no path: {value:?}"
            );
        }
    }

    assert_eq!(before.nodes, after.nodes, "and it round-trips");
}

/// TEX-4 — a texture's content occupies a known sub-rectangle of a padded image,
/// and what is stored is the content bounding box.
///
/// The other half — *the part's evaluated geometry places the artwork where it
/// was before cropping* — needs evaluation and is S4's. What the file owns is
/// the record that makes it reconstructible: the crop rectangle **and** the
/// authored dimensions, so the authored space a mesh's uvs live in can be
/// rebuilt.
#[test]
fn tex_4_a_padded_texture_is_stored_as_its_content_box() {
    let subject = puppet("rigged");
    let padded = subject
        .textures
        .iter()
        .find(|t| [t.crop[2], t.crop[3]] != t.authored_size)
        .expect("the base fixture carries content surrounded by transparent padding");

    assert!(
        padded.crop[2] < padded.authored_size[0] || padded.crop[3] < padded.authored_size[1],
        "the content box is smaller than the authored image"
    );
    assert_eq!(
        png_dimensions(&padded.payload),
        (padded.crop[2], padded.crop[3]),
        "the stored payload is the content bounding box, not the padded image"
    );
    assert!(
        padded.authored_size[0] > 0 && padded.authored_size[1] > 0,
        "the authored dimensions are kept, or the authored space cannot be rebuilt"
    );

    assert_eq!(
        subject.textures,
        round_trip("rigged", "tex-4").textures,
        "the record round-trips"
    );
}

/// TEX-5 — the saved file holds exactly one payload per distinct texture entity
/// and nothing else that decodes as an image.
///
/// Together with `FILE-4` this is how *the layered source is not embedded* is
/// asserted; `FILE-4` needs the import path and is Studio's.
#[test]
fn tex_5_one_payload_per_texture_entity_and_nothing_else_that_is_an_image() {
    let subject = puppet("rigged");
    let dir = scratch("tex-5");
    let written = dir.join("payloads.rigel2d");
    save(&subject, &written).expect("it saves");

    let entries = crate::support::entries(&written);
    let payloads: BTreeSet<&String> = entries
        .keys()
        .filter(|name| name.starts_with("textures/"))
        .collect();
    assert_eq!(
        payloads.len(),
        subject.textures.len(),
        "exactly one payload per distinct texture entity"
    );
    for texture in &subject.textures {
        let expected = format!("textures/{}.png", texture.id.0);
        assert!(
            entries.contains_key(&expected),
            "a payload addressable by the texture's identifier: {expected}"
        );
    }
    for (name, payload) in &entries {
        if name.starts_with("textures/") {
            continue;
        }
        assert!(
            !looks_like_png(payload),
            "{name} decodes as an image and is not a texture payload"
        );
    }
}

/// FLIP-1 — a texture flip is stored on the **part's reference**, per axis, and
/// one drawing stays one drawing.
///
/// The sampling half — *the two parts sample the drawing reversed relative to
/// each other* — is evaluation's and is S4's. The wrong build this test catches
/// is the one that stores the flip on the texture, so flipping one eye flips
/// both, or materialises a reversed second payload, so a pair of eyes costs two
/// textures.
#[test]
fn flip_1_a_texture_flip_is_stored_on_the_parts_reference() {
    let subject = puppet("rigged:flipped-pair");

    let mut by_texture: BTreeMap<Id, Vec<TextureFlip>> = BTreeMap::new();
    for node in &subject.nodes {
        if let NodeBody::Part(part) = &node.body {
            by_texture
                .entry(part.texture)
                .or_default()
                .push(part.texture_flip);
        }
    }
    let (shared, flips) = by_texture
        .iter()
        .find(|(_, parts)| parts.len() > 1)
        .expect("the fixture has parts sharing one texture");
    assert!(
        flips.iter().collect::<BTreeSet<_>>().len() > 1,
        "and they carry different flips, or this asserts nothing"
    );
    assert_eq!(
        subject.textures.iter().filter(|t| t.id == *shared).count(),
        1,
        "one texture entity, however many parts flip it"
    );

    let dir = scratch("flip-1");
    let written = dir.join("flipped.rigel2d");
    save(&subject, &written).expect("it saves");
    let payloads = entry_names(&written)
        .into_iter()
        .filter(|n| n.starts_with("textures/"))
        .count();
    assert_eq!(
        payloads,
        subject.textures.len(),
        "a flip materialises no second payload — it is not a mirror"
    );
    assert_eq!(
        subject,
        load(&written).expect("it loads").puppet,
        "the flip round-trips on the part"
    );
}

/// FLIP-2 — the two axes are **independent**.
///
/// Left–right and both-set are in the fixture; top–bottom alone is not, so it is
/// made here by setting it and round-tripping. A fixture carrying a top–bottom
/// flip on its own would be better, and its absence is reported in
/// `docs/wip/s2-format.md`. The wrong build is one flag standing for two axes,
/// which cannot express the ordinary case of a part reversed top to bottom.
#[test]
fn flip_2_the_two_flip_axes_are_independent() {
    let subject = puppet("rigged:flipped-pair");
    let carried: BTreeSet<TextureFlip> = subject
        .nodes
        .iter()
        .filter_map(|n| match &n.body {
            NodeBody::Part(part) => Some(part.texture_flip),
            _ => None,
        })
        .collect();
    assert!(
        carried.contains(&TextureFlip {
            left_right: true,
            top_bottom: false
        }),
        "the fixture carries a left–right flip"
    );
    assert!(
        carried.contains(&TextureFlip {
            left_right: true,
            top_bottom: true
        }),
        "and one with both axes set"
    );

    let mut with_top_bottom = subject.clone();
    let target = with_top_bottom
        .nodes
        .iter_mut()
        .find_map(|n| match &mut n.body {
            NodeBody::Part(part) => Some(part),
            _ => None,
        })
        .expect("a part");
    target.texture_flip = TextureFlip {
        left_right: false,
        top_bottom: true,
    };

    let dir = scratch("flip-2");
    let written = dir.join("top-bottom.rigel2d");
    save(&with_top_bottom, &written).expect("it saves");
    assert_eq!(
        with_top_bottom,
        load(&written).expect("it loads").puppet,
        "a top–bottom flip on its own round-trips and is distinguishable"
    );
}

/// POL-1 — axis polarity is declared **per dimension**, and takes one of three
/// values.
///
/// The fixture carrying the declared value *unknown* is `rigged:paired`, which
/// serves `authoring-acceptance.md` and is deferred out of this subsystem, so
/// that value is asserted by being written and round-tripped here. The wrong
/// build is one polarity per parameter — which makes a two-dimensional head
/// parameter either un-mirrorable or wrongly reversed on the axis that has no
/// mirror sense — or a two-valued field, which leaves *unknown* nowhere to be
/// written.
#[test]
fn pol_1_axis_polarity_is_declared_per_dimension() {
    let subject = puppet("rigged");

    let two_dimensional = subject
        .parameters
        .iter()
        .find(|p| p.axes.len() == 2)
        .expect("the base fixture carries a two-dimensional parameter");
    assert_eq!(
        two_dimensional.axes[0].polarity,
        AxisPolarity::Bilateral,
        "one bilateral axis"
    );
    assert_eq!(
        two_dimensional.axes[1].polarity,
        AxisPolarity::Unipolar,
        "and one unipolar, on the same parameter"
    );

    assert!(
        subject.parameters.iter().any(|p| p.axes.len() == 1),
        "a one-dimensional parameter declares one"
    );
    for parameter in &subject.parameters {
        assert!(
            !parameter.axes.is_empty(),
            "every parameter declares a polarity per dimension"
        );
    }

    let mut with_unknown = subject.clone();
    with_unknown.parameters[0].axes[0].polarity = AxisPolarity::Unknown;
    let dir = scratch("pol-1");
    let written = dir.join("unknown.rigel2d");
    save(&with_unknown, &written).expect("the third value is writable");
    assert_eq!(
        load(&written).expect("it loads").puppet.parameters[0].axes[0].polarity,
        AxisPolarity::Unknown,
        "and readable per dimension"
    );
}

/// DET-6 — the stored form carries a composition mode from the first version,
/// and it is written even though it holds its default.
///
/// A mode added later is then a new value and cannot alter a puppet authored
/// before it, which is the whole reason the field exists today. `DET-4`'s
/// hand-derived expectation is evaluation's and is S4's.
#[test]
fn det_6_the_composition_mode_is_carried_and_always_written() {
    let subject = puppet("rigged");
    assert_eq!(
        subject.composition_mode,
        CompositionMode::Sum,
        "the value meaning the fixed order specified now"
    );

    assert!(
        puppet_json(&fixture("rigged"))
            .get("composition_mode")
            .is_some(),
        "the fixture carries it"
    );

    let dir = scratch("det-6");
    let written = dir.join("mode.rigel2d");
    save(&subject, &written).expect("it saves");
    assert!(
        puppet_json(&written).get("composition_mode").is_some(),
        "and it is written back although it holds the only value the format defines"
    );
    assert_eq!(
        subject.composition_mode,
        load(&written).expect("it loads").puppet.composition_mode,
        "and it round-trips"
    );
}

/// CHAIN-1 — a chain deformer survives save and load with its joints and their
/// identifiers intact.
///
/// *Evaluation after reload is bit-identical* is S4's.
#[test]
fn chain_1_a_chain_deformer_survives_with_its_joints() {
    let before = puppet("rigged");
    let chain = before
        .nodes
        .iter()
        .find_map(|n| match &n.body {
            NodeBody::Part(part) => part.chain_deformer.as_ref(),
            _ => None,
        })
        .expect("the base fixture carries a chain deformer on one part");
    assert!(chain.joints.len() > 1, "with joints");
    assert_eq!(
        chain
            .joints
            .iter()
            .map(|j| j.id)
            .collect::<BTreeSet<_>>()
            .len(),
        chain.joints.len(),
        "each joint an entity with its own identifier"
    );

    assert_eq!(
        before.nodes,
        round_trip("rigged", "chain-1").nodes,
        "and all of it survives a round trip"
    );
}

/// CHAIN-2 — a chain deformer is a permanent structure, not a bulk edit.
///
/// Moving one joint and moving it back returns the file to its bytes. A chain
/// deformer that had baked itself into per-vertex offsets at creation fails
/// this, which is the bulk-edit-tool behaviour the requirement rejects.
#[test]
fn chain_2_moving_a_joint_and_back_returns_the_original_bytes() {
    let dir = scratch("chain-2");
    let original = dir.join("original.rigel2d");
    let restored = dir.join("restored.rigel2d");

    let subject = puppet("rigged");
    save(&subject, &original).expect("it saves");

    let mut moved = subject.clone();
    let joint = moved
        .nodes
        .iter_mut()
        .find_map(|n| match &mut n.body {
            NodeBody::Part(part) => part.chain_deformer.as_mut(),
            _ => None,
        })
        .expect("a chain deformer")
        .joints
        .first_mut()
        .expect("a joint");
    let was = joint.position;
    joint.position = [was[0] + 13.5, was[1] - 4.25];

    let mut back = moved.clone();
    back.nodes
        .iter_mut()
        .find_map(|n| match &mut n.body {
            NodeBody::Part(part) => part.chain_deformer.as_mut(),
            _ => None,
        })
        .expect("a chain deformer")
        .joints
        .first_mut()
        .expect("a joint")
        .position = was;

    save(&back, &restored).expect("it saves");
    assert_eq!(
        bytes(&original),
        bytes(&restored),
        "a chain deformer adjusts the rig rather than baking into it"
    );
}

/// The keys a physics node may carry in the stored form.
///
/// `docs/format.md`'s *A physics node* table, plus what every node carries and
/// the extension marker. Nothing simulated, and **no reference to anything** for
/// the anchor to be found by.
const PHYSICS_NODE_KEYS: &[&str] = &[
    "id",
    "kind",
    "name",
    "parent",
    "transform",
    "length",
    "gravity",
    "frequency",
    "angle_damping",
    "length_damping",
    "output",
    "output_source",
    "output_scale",
    "x",
];

/// PHYS-1 — a physics node is a node, and its anchor is its own origin.
///
/// Each is in the tree with an identifier, a parent and a position among its
/// siblings, and all three survive save and load. The anchor half — *change an
/// ancestor's transform and the point the simulation hangs from moves with it* —
/// is evaluation's and is S4's; what the **file** carries is the absence this
/// test asserts: no stored reference from the physics node to anything for it to
/// find that point by. The wrong build keeps physics in a list beside the tree
/// with a stored reference to the node it hangs from.
#[test]
fn phys_1_a_physics_node_is_a_node_with_no_anchor_reference() {
    let before = puppet("rigged");
    let physics: Vec<_> = before
        .nodes
        .iter()
        .filter(|n| matches!(n.body, NodeBody::Physics(_)))
        .collect();
    assert!(
        physics.len() > 1,
        "the base fixture carries a physics subtree"
    );
    let live: BTreeSet<Id> = before.nodes.iter().map(|n| n.id).collect();
    for node in &physics {
        assert_ne!(node.id, Id::NONE, "with an identifier");
        assert!(live.contains(&node.parent), "and a parent in the tree");
    }
    let order_before: Vec<Id> = before.nodes.iter().map(|n| n.id).collect();

    let after = round_trip("rigged", "phys-1");
    assert_eq!(
        order_before,
        after.nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
        "its place among its siblings survives save and load"
    );

    let doc = puppet_json(&fixture("rigged"));
    for node in &physics {
        let stored = json_node(&doc, node.id.0);
        let keys: Vec<&String> = stored.as_object().expect("an object").keys().collect();
        for key in keys {
            assert!(
                PHYSICS_NODE_KEYS.contains(&key.as_str()),
                "a physics node carries {key:?}, which the format does not define for it"
            );
        }
    }
}

/// PHYS-4 — driving a parameter is optional.
///
/// The base fixture carries a physics node that drives a parameter and one that
/// drives none. The one driving nothing is neither rejected, nor reported as a
/// defect, nor repaired by being given a parameter — and in a longer chain it
/// still carries the links below it. The wrong build requires the reference, so
/// a middle link of a pendulum cannot be expressed and the rigger invents a
/// parameter nothing reads.
#[test]
fn phys_4_driving_a_parameter_is_optional() {
    let loaded = load(&fixture("rigged")).expect("it loads");
    let driving: Vec<&rigel_format::Node> = loaded
        .puppet
        .nodes
        .iter()
        .filter(|n| matches!(&n.body, NodeBody::Physics(p) if p.output != Id::NONE))
        .collect();
    let driverless: Vec<&rigel_format::Node> = loaded
        .puppet
        .nodes
        .iter()
        .filter(|n| matches!(&n.body, NodeBody::Physics(p) if p.output == Id::NONE))
        .collect();
    assert!(!driving.is_empty(), "one physics node drives a parameter");
    assert!(!driverless.is_empty(), "and one drives none");
    assert!(
        loaded.report.warnings.is_empty(),
        "driving nothing is not reported as a defect"
    );
    assert!(
        validate(&loaded.puppet).is_ok(),
        "and is not rejected or repaired"
    );

    assert_eq!(
        loaded.puppet.nodes,
        round_trip("rigged", "phys-4").nodes,
        "both round-trip unchanged"
    );

    // A link driving nothing still carries the chain below it.
    let chain = puppet("rigged:physics-chain");
    let middle = chain
        .nodes
        .iter()
        .find(|n| {
            matches!(&n.body, NodeBody::Physics(p) if p.output == Id::NONE)
                && chain.nodes.iter().any(|c| c.parent == n.id)
        })
        .expect("a middle link, driving nothing and carrying more chain");
    assert!(
        chain
            .nodes
            .iter()
            .any(|c| c.parent == middle.id && matches!(c.body, NodeBody::Physics(_))),
        "and what it carries is physics"
    );
}

/// PHYS-6 — one gravity for the puppet, scaled per node.
///
/// The file half: exactly one gravity on the puppet, a multiplier on each
/// physics node, and both round-trip. The simulation half — *change the puppet's
/// gravity and every physics node's displacement changes in proportion to its
/// own multiplier* — is S4's. The wrong build stores a gravity per physics node,
/// so the rigger calming every pendulum at once edits each one and misses the one
/// they forgot.
#[test]
fn phys_6_one_gravity_for_the_puppet_scaled_per_node() {
    let before = puppet("rigged");
    let doc = puppet_json(&fixture("rigged"));
    assert!(
        doc.get("gravity").is_some(),
        "the puppet carries its own gravity"
    );

    let multipliers: Vec<f64> = before
        .nodes
        .iter()
        .filter_map(|n| match &n.body {
            NodeBody::Physics(physics) => Some(physics.gravity),
            _ => None,
        })
        .collect();
    assert!(
        multipliers.len() > 1,
        "and every physics node carries its own multiplier"
    );
    assert!(
        multipliers.windows(2).any(|w| w[0] != w[1]),
        "distinct ones, or a per-node multiplier could be the puppet's gravity in disguise"
    );

    let after = round_trip("rigged", "phys-6");
    assert_eq!(before.gravity, after.gravity, "the puppet's gravity");
    assert_eq!(before.nodes, after.nodes, "and each node's multiplier");
}

/// PHYS-8 — simulation state is never stored.
///
/// The file half: a physics node's stored object carries only the constants the
/// format defines, and two loads of one file produce one puppet. The *save
/// mid-simulation equals a save at rest* and *identical first frame* halves need
/// evaluation and are S4's. The wrong build serialises the running state, so no
/// two saves of an untouched puppet agree — and `CANON-1` then fails for a reason
/// nobody looks for in physics.
#[test]
fn phys_8_no_simulation_state_reaches_the_file() {
    let doc = puppet_json(&fixture("rigged"));
    let mut seen = 0;
    for node in doc["nodes"].as_array().expect("a nodes array") {
        if node["kind"].as_str() != Some("physics") {
            continue;
        }
        seen += 1;
        for key in node.as_object().expect("an object").keys() {
            assert!(
                PHYSICS_NODE_KEYS.contains(&key.as_str()),
                "a physics node carries {key:?}, which is not a constant the format defines"
            );
        }
    }
    assert!(seen > 0, "the fixture carries physics nodes");

    assert_eq!(
        puppet("rigged"),
        puppet("rigged"),
        "two loads of one file produce one puppet, with nothing accumulated"
    );
}

/// PHYS-9 — a physics node driving a parameter that a binding on its **own
/// ancestor** reads is valid.
///
/// The dataflow cycle is deliberate and is broken in time by the previous-frame
/// read; the acyclicity rule binds the **tree**, which `rigged:cycle` breaks and
/// this fixture does not. *Evaluates deterministically across repeated runs* is
/// S4's. The wrong build walks drivers as though they were tree edges and
/// refuses a file the format permits — a pendulum driving the node it hangs from
/// is the ordinary hair rig.
#[test]
fn phys_9_a_physics_node_driving_its_own_ancestors_binding_is_valid() {
    let path = fixture("rigged:physics-drives-ancestor");
    assert!(
        validate_file(&path).is_ok(),
        "the standalone validator accepts it"
    );
    let subject = load(&path).expect("and so does the loader").puppet;

    // The fixture is what it says it is: a physics node drives a parameter that a
    // binding on one of that node's own ancestors reads.
    let ancestors = |mut id: Id| {
        let mut out = BTreeSet::new();
        while let Some(node) = subject.nodes.iter().find(|n| n.id == id) {
            out.insert(node.parent);
            if node.parent == Id::NONE {
                break;
            }
            id = node.parent;
        }
        out
    };
    let found = subject.nodes.iter().any(|n| match &n.body {
        NodeBody::Physics(physics) if physics.output != Id::NONE => {
            let above = ancestors(n.id);
            subject
                .bindings
                .iter()
                .any(|b| b.parameter == physics.output && above.contains(&b.target))
        }
        _ => false,
    });
    assert!(found, "the fixture carries the dataflow cycle it names");

    let dir = scratch("phys-9");
    save(&subject, &dir.join("ancestor.rigel2d")).expect("and it saves");
}

/// PHYS-10 — a puppet carrying **no gravity** uses the default rather than being
/// invalid.
///
/// So that a generator that has never heard of physics writes a valid file. The
/// default is the per-node multiplier's, `1.0`, and because a defaulted field is
/// absent when it holds its default, a save of such a puppet writes none either —
/// which is what stops the file churning the first time our build touches it.
#[test]
fn phys_10_a_puppet_with_no_gravity_uses_the_default() {
    let source = fixture("rigged:no-gravity");
    assert!(
        puppet_json(&source).get("gravity").is_none(),
        "the fixture carries no gravity, or this asserts nothing"
    );
    assert!(
        validate_file(&source).is_ok(),
        "an absent gravity is valid, not malformed"
    );

    let subject = load(&source).expect("it loads").puppet;
    assert_eq!(
        subject.gravity, 1.0,
        "the default stands, matching the per-node multiplier's default"
    );

    let dir = scratch("phys-10");
    let written = dir.join("no-gravity.rigel2d");
    save(&subject, &written).expect("it saves");
    assert!(
        puppet_json(&written).get("gravity").is_none(),
        "a defaulted field is absent when it holds its default"
    );
    assert_eq!(
        bytes(&source),
        bytes(&written),
        "so a puppet with no gravity does not churn when this build saves it"
    );

    let with_gravity = puppet("rigged");
    assert_ne!(
        with_gravity.gravity, subject.gravity,
        "the base fixture states a gravity that is not the default, so the two differ"
    );
    assert_eq!(
        with_gravity.nodes, subject.nodes,
        "and nothing else about the puppet changes with it"
    );
}
