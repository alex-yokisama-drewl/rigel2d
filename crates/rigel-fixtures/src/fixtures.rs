//! The catalogue: every fixture, and the one thing that makes it what it is.
//!
//! Three rules bind this file.
//!
//! - **One defect per defect fixture.** A rejection can then never be
//!   attributed to the wrong rule.
//! - **Every fixture declares the current format version unless its name says
//!   otherwise.** The validator's rules on unmarked data and on unfamiliar
//!   enumerated values are gated on the declared version, so a defect fixture
//!   declaring a later version would be accepted and the criterion would stop
//!   testing anything.
//! - **Nothing is a path, a URL or a command**, except in `rigged:ext-hostile`,
//!   where hostile-*shaped* strings are inert data a loader must carry and
//!   never act on.

use serde_json::{Value, json};

use crate::archive::{self, Entry};
use crate::image;
use crate::json::Doc;
use crate::puppet::{self, *};
use crate::rawzip::{self, RawEntry};

/// What a conforming loader does with a fixture.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// The load succeeds. Some of these carry a warning with them.
    Accepted,
    /// The load is refused.
    Rejected,
}

impl Expect {
    pub fn as_str(self) -> &'static str {
        match self {
            Expect::Accepted => "accepted",
            Expect::Rejected => "rejected",
        }
    }
}

pub struct Fixture {
    /// The catalogue name, as `docs/requirements/puppet-acceptance.md` writes it.
    pub name: String,
    /// The file it is written to. The catalogue's `:` is not a legal character
    /// in a Windows path and the push gates check out on Windows, so it becomes
    /// a `.` here.
    pub file: String,
    pub expect: Expect,
    /// The one thing that makes this fixture what it is.
    pub note: String,
    pub bytes: Vec<u8>,
}

fn file_name_for(name: &str) -> String {
    format!("{}.rigel2d", name.replace(':', "."))
}

/// Assemble the three kinds of archive entry `docs/format.md` defines.
fn archive_of(doc: &Doc, major: u64, minor: u64, payloads: &[(u64, Vec<u8>)]) -> Vec<u8> {
    let mut entries = vec![
        Entry::new("puppet.json", doc.to_json().into_bytes()),
        Entry::new("rigel2d.json", manifest(major, minor).into_bytes()),
    ];
    for (id, bytes) in payloads {
        entries.push(Entry::new(format!("textures/{id}.png"), bytes.clone()));
    }
    archive::write(entries)
}

struct Catalogue(Vec<Fixture>);

impl Catalogue {
    fn push(&mut self, name: &str, expect: Expect, note: &str, bytes: Vec<u8>) {
        assert!(
            !self.0.iter().any(|f| f.name == name),
            "duplicate fixture name {name}"
        );
        self.0.push(Fixture {
            name: name.to_string(),
            file: file_name_for(name),
            expect,
            note: note.to_string(),
            bytes,
        });
    }

    /// `rigged` with a change applied, at the current format version.
    fn variant(&mut self, name: &str, expect: Expect, note: &str, change: impl FnOnce(&mut Doc)) {
        self.variant_at(name, expect, note, VERSION_MAJOR, VERSION_MINOR, change);
    }

    /// `rigged` with a change applied, declaring a stated format version.
    fn variant_at(
        &mut self,
        name: &str,
        expect: Expect,
        note: &str,
        major: u64,
        minor: u64,
        change: impl FnOnce(&mut Doc),
    ) {
        let mut doc = puppet::rigged();
        change(&mut doc);
        let bytes = archive_of(&doc, major, minor, &puppet::texture_payloads());
        self.push(name, expect, note, bytes);
    }
}

/// Set one field of a node's local transform to literal JSON text.
fn raw_rotation(doc: &mut Doc, node: u64, literal: &str) {
    let placeholder = doc.raws.add(literal);
    doc.node(node)
        .get_mut("transform")
        .and_then(Value::as_object_mut)
        .expect("transform is an object")
        .insert("rotation".into(), placeholder);
}

/// Move the wiggle extension on the face part out of its namespace, losing the
/// marker, the declared version and the human-readable name with it. Both
/// `rigged:ext-unmarked` and `rigged:future-minor` carry exactly this, so the
/// unmarked material in the two is byte-for-byte the same and the only
/// difference between them is the declared version.
fn hoist_wiggle_unmarked(doc: &mut Doc) {
    let face = doc.node(N_FACE);
    face.remove("x");
    face.insert("wiggle".into(), puppet::wiggle_unmarked());
}

/// `rigged:future-minor`'s three licensed unknowns: an unmarked core field, an
/// unknown node kind, and an unfamiliar value in a known enumerated field.
fn future_minor_unknowns(doc: &mut Doc) {
    hoist_wiggle_unmarked(doc);
    let mut warp = puppet::leaf_node(SPARE, "warp_field", "warp", N_ROOT);
    warp.as_object_mut()
        .unwrap()
        .insert("strength".into(), json!(0.75));
    doc.push_node(warp);
    doc.node(N_PHYSICS)
        .insert("output_source".into(), json!(["torque"]));
}

/// The containment pairs the format's table forbids, each as one added leaf.
const FORBIDDEN_CONTAINMENT: &[(&str, u64, &str, &str)] = &[
    (
        "rigged:part-with-child",
        N_FACE,
        "part",
        "a part contains a part",
    ),
    (
        "rigged:part-contains-mesh-group",
        N_FACE,
        "mesh_group",
        "a part contains a mesh group",
    ),
    (
        "rigged:part-contains-composite",
        N_FACE,
        "composite",
        "a part contains a composite",
    ),
    (
        "rigged:part-contains-physics",
        N_FACE,
        "physics",
        "a part contains a physics node",
    ),
    (
        "rigged:physics-contains-part",
        N_PHYSICS_TIP,
        "part",
        "a physics node contains a part",
    ),
    (
        "rigged:physics-contains-mesh-group",
        N_PHYSICS_TIP,
        "mesh_group",
        "a physics node contains a mesh group",
    ),
    (
        "rigged:physics-contains-composite",
        N_PHYSICS_TIP,
        "composite",
        "a physics node contains a composite",
    ),
];

/// The physics constants that admit a degenerate value, and the value.
const DEGENERATE_PHYSICS: &[(&str, &str, f64, &str)] = &[
    (
        "rigged:physics-length-zero",
        "length",
        0.0,
        "`length` at zero",
    ),
    (
        "rigged:physics-length-negative",
        "length",
        -120.0,
        "`length` negative",
    ),
    (
        "rigged:physics-frequency-zero",
        "frequency",
        0.0,
        "`frequency` at zero",
    ),
    (
        "rigged:physics-frequency-negative",
        "frequency",
        -1.4,
        "`frequency` negative",
    ),
    (
        "rigged:physics-gravity-negative",
        "gravity",
        -0.8,
        "`gravity` negative",
    ),
    (
        "rigged:physics-angle-damping-negative",
        "angle_damping",
        -0.35,
        "`angle_damping` negative",
    ),
    (
        "rigged:physics-length-damping-negative",
        "length_damping",
        -0.6,
        "`length_damping` negative",
    ),
];

pub fn all() -> Vec<Fixture> {
    let mut c = Catalogue(Vec::new());
    base(&mut c);
    versions(&mut c);
    extensions(&mut c);
    tree(&mut c);
    parameters_and_tracks(&mut c);
    numbers(&mut c);
    identifiers(&mut c);
    physics(&mut c);
    textures(&mut c);
    containers(&mut c);
    c.0
}

// ------------------------------------------------------------------ the base

fn base(c: &mut Catalogue) {
    let rigged = puppet::rigged();
    c.push(
        "rigged",
        Expect::Accepted,
        "the base valid puppet every other fixture is a variant of",
        archive_of(
            &rigged,
            VERSION_MAJOR,
            VERSION_MINOR,
            &puppet::texture_payloads(),
        ),
    );

    let one_part = puppet::one_part();
    c.push(
        "one-part",
        Expect::Accepted,
        "the smallest valid puppet: a root that contains, one part, one texture",
        archive_of(
            &one_part,
            VERSION_MAJOR,
            VERSION_MINOR,
            &[puppet::texture_payloads()[0].clone()],
        ),
    );

    // `one-part` with the wrapping composite gone, so the root is a part — the
    // one node kind the format refuses to root a puppet at.
    let mut root_is_part = puppet::one_part();
    root_is_part
        .obj()
        .insert("name".into(), json!("root-is-part"));
    root_is_part.obj().insert("root".into(), json!(N_FACE));
    root_is_part
        .obj()
        .get_mut("nodes")
        .and_then(Value::as_array_mut)
        .unwrap()
        .retain(|n| n.get("id").and_then(Value::as_u64) != Some(N_ROOT));
    root_is_part.node(N_FACE).insert("parent".into(), json!(0));
    c.push(
        "root-is-part",
        Expect::Rejected,
        "the root is a part, which is not a kind that contains",
        archive_of(
            &root_is_part,
            VERSION_MAJOR,
            VERSION_MINOR,
            &[puppet::texture_payloads()[0].clone()],
        ),
    );

    // A mesh group nested inside the first, carrying a part of its own which
    // carries a binding.
    c.variant(
        "rigged:deep",
        Expect::Accepted,
        "a second mesh group inside the first, with a part and a binding of its own",
        |doc| {
            doc.push_node(puppet::leaf_node(SPARE, "mesh_group", "inner_body", N_BODY));
            doc.push_node(puppet::leaf_node(SPARE + 1, "part", "inner_part", SPARE));
            let binding = json!({
                "id": SPARE + 2,
                "kind": "transform",
                "parameter": P_HEAD_TURN,
                "target": SPARE + 1,
                "values": [
                    {"rotation": -0.05, "scale": [1.0, 1.0], "translation": [-2.0, 0.0]},
                    {"rotation": 0.0, "scale": [1.0, 1.0], "translation": [0.0, 0.0]},
                    {"rotation": 0.05, "scale": [1.0, 1.0], "translation": [2.0, 0.0]},
                ],
            });
            doc.obj()
                .get_mut("bindings")
                .and_then(Value::as_array_mut)
                .unwrap()
                .push(binding);
        },
    );

    // The physics subtree extended to a chain of twelve links under one anchor.
    // The length is in each link's name so the fixture states it itself.
    c.variant(
        "rigged:physics-chain",
        Expect::Accepted,
        "the physics subtree extended to a chain of twelve links under one anchor",
        |doc| {
            doc.node(N_PHYSICS_TIP)
                .insert("name".into(), json!("link_02_of_12"));
            doc.node(N_PHYSICS)
                .insert("name".into(), json!("link_01_of_12"));
            let mut parent = N_PHYSICS_TIP;
            for index in 3..=12u64 {
                let id = 1000 + index;
                let mut link = puppet::leaf_node(id, "physics", "", parent);
                {
                    let object = link.as_object_mut().unwrap();
                    object.insert("name".into(), json!(format!("link_{index:02}_of_12")));
                    object.insert("length".into(), json!(40.0));
                    object.insert("gravity".into(), json!(0.4));
                    object.insert("frequency".into(), json!(2.0));
                    object.insert("angle_damping".into(), json!(0.3));
                    object.insert("length_damping".into(), json!(0.55));
                    object.insert(
                        "transform".into(),
                        json!({
                            "rotation": 0.0,
                            "scale": [1.0, 1.0],
                            "translation": [0.0, 24.0],
                        }),
                    );
                }
                doc.push_node(link);
                parent = id;
            }
        },
    );

    // The physics node drives a parameter that a binding on one of its own
    // ancestors reads: a dataflow cycle broken in time, and valid.
    c.variant(
        "rigged:physics-drives-ancestor",
        Expect::Accepted,
        "the binding on the physics-driven parameter moved onto that node's ancestor",
        |doc| {
            doc.binding(B_BACK_TRANSFORM)
                .insert("target".into(), json!(N_BODY));
        },
    );

    // A puppet carrying no gravity uses the default rather than being invalid,
    // so a generator that has never heard of physics writes a valid file.
    c.variant(
        "rigged:no-gravity",
        Expect::Accepted,
        "the puppet's gravity absent, so the default stands",
        |doc| {
            doc.obj().remove("gravity");
        },
    );

    // Just under the provisional node-count load limit.
    c.variant(
        "rigged:near-limit",
        Expect::Accepted,
        "padded to one node under the provisional node-count load limit",
        |doc| {
            let existing = doc
                .obj()
                .get("nodes")
                .and_then(Value::as_array)
                .unwrap()
                .len();
            for index in 0..(PROVISIONAL_NODE_LIMIT - 1 - existing) {
                let id = 1000 + index as u64;
                doc.push_node(puppet::leaf_node(
                    id,
                    "mesh_group",
                    &format!("filler_{index:03}"),
                    N_ROOT,
                ));
            }
        },
    );
}

// --------------------------------------------------------------- the version

fn versions(c: &mut Catalogue) {
    c.variant_at(
        "rigged:future-minor",
        Expect::Accepted,
        "a later minor version, licensing an unmarked core field, an unknown node kind \
         and an unfamiliar enumerated value",
        VERSION_MAJOR,
        VERSION_MINOR + 1,
        future_minor_unknowns,
    );

    c.variant_at(
        "rigged:future-minor-nonfinite",
        Expect::Accepted,
        "bytes that read as a non-finite number inside the opaquely preserved material",
        VERSION_MAJOR,
        VERSION_MINOR + 1,
        |doc| {
            future_minor_unknowns(doc);
            let placeholder = doc.raws.add("1e400");
            doc.node(SPARE).insert("strength".into(), placeholder);
        },
    );

    c.variant_at(
        "rigged:future-minor-nan-interpreted",
        Expect::Rejected,
        "a non-finite value in a known numeric field of a later-version file",
        VERSION_MAJOR,
        VERSION_MINOR + 1,
        |doc| raw_rotation(doc, N_HAIR, "1e400"),
    );

    c.variant_at(
        "rigged:future-minor-polarity-unfamiliar",
        Expect::Accepted,
        "an axis polarity value the format does not define, in a later-version file",
        VERSION_MAJOR,
        VERSION_MINOR + 1,
        unfamiliar_polarity,
    );

    c.variant_at(
        "rigged:old-version",
        Expect::Accepted,
        "an earlier minor version, which this build upgrades on save",
        VERSION_MAJOR,
        VERSION_MINOR - 1,
        |_| {},
    );

    c.variant_at(
        "rigged:breaking-version",
        Expect::Rejected,
        "a later major version, which is a declared break in compatibility",
        VERSION_MAJOR + 1,
        0,
        |_| {},
    );
}

// ------------------------------------------------------------ extension data

fn extensions(c: &mut Catalogue) {
    c.variant(
        "rigged:ext-unmarked",
        Expect::Rejected,
        "the same extension values with the marker removed, at the current version",
        hoist_wiggle_unmarked,
    );

    c.variant(
        "rigged:ext-no-version",
        Expect::Rejected,
        "extension data with no declared version",
        |doc| {
            let face = doc.node(N_FACE);
            let block = crate::json::extension_block(face);
            block
                .get_mut("com.example.wiggle")
                .and_then(Value::as_object_mut)
                .unwrap()
                .remove("version");
        },
    );

    c.variant(
        "rigged:ext-no-label",
        Expect::Rejected,
        "extension data with no human-readable name",
        |doc| {
            let face = doc.node(N_FACE);
            let block = crate::json::extension_block(face);
            block
                .get_mut("com.example.wiggle")
                .and_then(Value::as_object_mut)
                .unwrap()
                .remove("name");
        },
    );

    c.variant(
        "rigged:unmarked-core-data",
        Expect::Rejected,
        "a key the core format does not define, outside `x`",
        |doc| {
            doc.node(N_HAIR).insert("wiggle_amount".into(), json!(0.5));
        },
    );

    // The same two namespaces presented in the other order. The saved bytes are
    // expected to be `rigged`'s, because position is set by the canonical
    // ordering keyed on the marker, not by where the material arrived.
    c.variant(
        "rigged:ext-moved",
        Expect::Accepted,
        "`rigged`'s two extension namespaces written in the other order",
        |doc| {
            let reversed = format!(
                "{{\"com.example.wiggle\":{},\"com.alt.notes\":{}}}",
                serde_json::to_string(&puppet::wiggle_extension()).unwrap(),
                serde_json::to_string(&puppet::notes_extension()).unwrap(),
            );
            let placeholder = doc.raws.add(reversed);
            doc.node(N_BODY).insert("x".into(), placeholder);
        },
    );

    c.variant(
        "rigged:ext-unknown",
        Expect::Accepted,
        "a further extension namespace no build understands, with its own name and version",
        |doc| {
            let root = doc.node(N_ROOT);
            let block = crate::json::extension_block(root);
            block.insert(
                "com.example.ribbon".into(),
                puppet::extension(
                    "3.0",
                    "Ribbon Simulator",
                    json!({"ribbons": 3, "stiffness": 0.4}),
                ),
            );
        },
    );

    c.variant(
        "rigged:ext-declares-plugin",
        Expect::Accepted,
        "a declaration that the file carries data for a named extension at a named version",
        |doc| {
            let root = doc.node(N_ROOT);
            let block = crate::json::extension_block(root);
            block.insert(
                "com.example.physics-plus".into(),
                puppet::extension(
                    "2.0",
                    "Physics Plus",
                    json!({"solver": "verlet", "substeps": 4}),
                ),
            );
        },
    );

    c.variant(
        "rigged:ext-references-texture",
        Expect::Accepted,
        "extension data in an unreadable namespace that also names a referenced texture",
        |doc| {
            let hair = doc.node(N_HAIR);
            let block = crate::json::extension_block(hair);
            block.insert(
                "com.example.ribbon".into(),
                puppet::extension(
                    "3.0",
                    "Ribbon Simulator",
                    json!({"draws_with": T_HAIR, "ribbons": 2}),
                ),
            );
        },
    );

    // Hostile *shapes*, as inert data. Nothing here is ever resolved, spawned,
    // opened or interpolated — carrying it untouched is the whole assertion.
    c.variant(
        "rigged:ext-hostile",
        Expect::Accepted,
        "path-, URL-, command- and format-string-shaped values in every string-bearing field",
        |doc| {
            const HOSTILE: &[&str] = &[
                "/etc/passwd",
                "../../../../etc/shadow",
                "https://example.invalid/payload",
                "$(echo rigel) && echo rigel",
                "%s%n%d {0} {name}",
            ];
            doc.obj().insert("name".into(), json!(HOSTILE[0]));
            for (index, id) in [N_ROOT, N_BODY, N_FACE, N_HAIR, N_PHYSICS]
                .iter()
                .enumerate()
            {
                doc.node(*id)
                    .insert("name".into(), json!(HOSTILE[index % HOSTILE.len()]));
            }
            for (index, id) in [T_FACE, T_HAIR, T_MASK].iter().enumerate() {
                doc.texture(*id)
                    .insert("name".into(), json!(HOSTILE[index % HOSTILE.len()]));
            }
            for (index, id) in [P_HEAD_TURN, P_MOUTH_OPEN, P_BODY_LEAN, P_HAIR_SWAY]
                .iter()
                .enumerate()
            {
                doc.parameter(*id)
                    .insert("name".into(), json!(HOSTILE[index % HOSTILE.len()]));
            }
            doc.entity("animations", A_IDLE)
                .insert("name".into(), json!(HOSTILE[3]));
            let mapping = doc.entity("channel_mappings", CM_MOUTH);
            mapping.insert("channel".into(), json!(HOSTILE[2]));
            mapping.insert("source".into(), json!(HOSTILE[1]));
            for (index, id) in [N_FACE, N_HAIR, N_HAIR_BACK, N_FACE_MASK]
                .iter()
                .enumerate()
            {
                let node = doc.node(*id);
                let provenance = node
                    .get_mut("provenance")
                    .and_then(Value::as_object_mut)
                    .unwrap();
                provenance.insert("source_id".into(), json!(HOSTILE[index % HOSTILE.len()]));
            }
            let face = doc.node(N_FACE);
            let block = crate::json::extension_block(face);
            block.insert(
                "com.example.hostile".into(),
                puppet::extension(
                    "1.0",
                    "../../etc/hosts",
                    json!({
                        "command": "$(echo rigel)",
                        "format": "%s%n",
                        "path": "/etc/passwd",
                        "traversal": "../../../../etc/shadow",
                        "url": "https://example.invalid/payload",
                    }),
                ),
            );
        },
    );
}

// ------------------------------------------------------------------ the tree

fn tree(c: &mut Catalogue) {
    c.variant(
        "rigged:cycle",
        Expect::Rejected,
        "the head of the physics chain reparented under its own child",
        |doc| {
            doc.node(N_PHYSICS)
                .insert("parent".into(), json!(N_PHYSICS_TIP));
        },
    );

    c.variant(
        "rigged:no-root",
        Expect::Rejected,
        "the puppet names no root",
        |doc| {
            doc.obj().insert("root".into(), json!(0));
        },
    );

    c.variant(
        "rigged:two-roots",
        Expect::Rejected,
        "a second node with no parent beside the root",
        |doc| {
            doc.node(N_HAIR_BACK).insert("parent".into(), json!(0));
        },
    );

    c.variant(
        "rigged:dangling-node-ref",
        Expect::Rejected,
        "a mask naming a node that is not there",
        |doc| {
            doc.node(N_HAIR).insert("mask".into(), json!(9999));
        },
    );

    c.variant(
        "rigged:dangling-texture-ref",
        Expect::Rejected,
        "a part naming a texture that is not there",
        |doc| {
            doc.node(N_HAIR_BACK).insert("texture".into(), json!(9999));
        },
    );

    c.variant(
        "rigged:dangling-param-ref",
        Expect::Rejected,
        "a binding naming a parameter that is not there",
        |doc| {
            doc.binding(B_BODY_TRANSFORM)
                .insert("parameter".into(), json!(9999));
        },
    );

    for &(name, parent, kind, note) in FORBIDDEN_CONTAINMENT {
        c.variant(name, Expect::Rejected, note, move |doc| {
            doc.push_node(puppet::leaf_node(SPARE, kind, "intruder", parent));
        });
    }

    // Every pair the containment table permits, in one file. The permit half of
    // the same table-driven criterion the fixtures above drive the refuse half
    // of. `rigged` already carries composite→mesh group, composite→part, mesh
    // group→part, mesh group→physics and physics→physics.
    c.variant(
        "rigged:containment-permitted",
        Expect::Accepted,
        "every container-and-child pair the containment table permits, in one file",
        |doc| {
            doc.push_node(puppet::leaf_node(
                SPARE,
                "mesh_group",
                "nested_group",
                N_BODY,
            ));
            doc.push_node(puppet::leaf_node(
                SPARE + 1,
                "composite",
                "nested_composite",
                N_BODY,
            ));
            doc.push_node(puppet::leaf_node(
                SPARE + 2,
                "composite",
                "inner_composite",
                SPARE + 1,
            ));
            doc.push_node(puppet::leaf_node(
                SPARE + 3,
                "physics",
                "composite_physics",
                SPARE + 1,
            ));
        },
    );
}

// ----------------------------------------- parameters, keypoints and tracks

fn unfamiliar_polarity(doc: &mut Doc) {
    doc.parameter(P_HEAD_TURN)
        .get_mut("axes")
        .and_then(Value::as_array_mut)
        .unwrap()[0]
        .as_object_mut()
        .unwrap()
        .insert("polarity".into(), json!("radial"));
}

fn parameters_and_tracks(c: &mut Catalogue) {
    c.variant(
        "rigged:keypoints-unordered",
        Expect::Rejected,
        "a keypoint out of order along its axis",
        |doc| {
            doc.keypoint(P_HEAD_TURN, K_HEAD_LEFT)
                .insert("position".into(), json!([0.5]));
        },
    );

    c.variant(
        "rigged:keypoint-outside-declared-range",
        Expect::Rejected,
        "a keypoint past the range its parameter declares",
        |doc| {
            doc.keypoint(P_HEAD_TURN, K_HEAD_RIGHT)
                .insert("position".into(), json!([1.5]));
        },
    );

    c.variant(
        "rigged:no-neutral-keypoint",
        Expect::Rejected,
        "a parameter with no neutral keypoint",
        |doc| {
            doc.keypoint(P_HEAD_TURN, K_HEAD_NEUTRAL).remove("neutral");
        },
    );

    c.variant(
        "rigged:two-neutral-keypoints",
        Expect::Rejected,
        "a parameter with two neutral keypoints",
        |doc| {
            doc.keypoint(P_HEAD_TURN, K_HEAD_LEFT)
                .insert("neutral".into(), json!(true));
        },
    );

    c.variant(
        "rigged:missing-polarity",
        Expect::Rejected,
        "a parameter axis declaring no polarity at all",
        |doc| {
            doc.parameter(P_HEAD_TURN)
                .get_mut("axes")
                .and_then(Value::as_array_mut)
                .unwrap()[0]
                .as_object_mut()
                .unwrap()
                .remove("polarity");
        },
    );

    c.variant(
        "rigged:polarity-value-unfamiliar",
        Expect::Rejected,
        "an axis polarity value the format does not define, at the current version",
        unfamiliar_polarity,
    );

    c.variant(
        "rigged:unknown-enum-unmarked",
        Expect::Rejected,
        "an unfamiliar value in a known enumerated field, at the current version",
        |doc| {
            doc.node(N_PHYSICS)
                .insert("output_source".into(), json!(["torque"]));
        },
    );

    c.variant(
        "rigged:binding-count-mismatch-unmarked",
        Expect::Rejected,
        "a deformation binding whose value count no longer matches its mesh, unmarked",
        drop_one_deformation_offset,
    );

    c.variant(
        "rigged:binding-count-mismatch-marked-stale",
        Expect::Accepted,
        "the same mismatch, marked stale — the valid case",
        |doc| {
            drop_one_deformation_offset(doc);
            doc.binding(B_FACE_DEFORMATION)
                .insert("stale".into(), json!(true));
        },
    );

    c.variant(
        "rigged:binding-count-match-marked-stale",
        Expect::Rejected,
        "a binding marked stale whose value count does match its mesh",
        |doc| {
            doc.binding(B_FACE_DEFORMATION)
                .insert("stale".into(), json!(true));
        },
    );

    c.variant(
        "rigged:keyframes-unordered",
        Expect::Rejected,
        "a track's keyframes out of order in time",
        |doc| {
            doc.track(A_IDLE, TR_HEAD)
                .get_mut("keyframes")
                .and_then(Value::as_array_mut)
                .unwrap()[1]
                .as_object_mut()
                .unwrap()
                .insert("time".into(), json!(1.5));
        },
    );

    c.variant(
        "rigged:keyframe-outside-declared-range",
        Expect::Rejected,
        "a keyframe value past the range its parameter declares",
        |doc| {
            doc.track(A_IDLE, TR_HEAD)
                .get_mut("keyframes")
                .and_then(Value::as_array_mut)
                .unwrap()[1]
                .as_object_mut()
                .unwrap()
                .insert("value".into(), json!([1.5]));
        },
    );

    c.variant(
        "rigged:blend-duration-negative",
        Expect::Rejected,
        "a track's blend duration below zero",
        |doc| {
            doc.track(A_IDLE, TR_HEAD)
                .insert("blend_duration".into(), json!(-0.25));
        },
    );

    c.variant(
        "rigged:two-tracks-one-parameter",
        Expect::Rejected,
        "one animation carrying two tracks for one parameter",
        |doc| {
            let extra = json!({
                "blend_duration": 0.25,
                "id": SPARE,
                "keyframes": [
                    {"time": 0.0, "value": [0.0]},
                    {"time": 1.0, "value": [-0.5]},
                ],
                "parameter": P_HEAD_TURN,
            });
            doc.entity("animations", A_IDLE)
                .get_mut("tracks")
                .and_then(Value::as_array_mut)
                .unwrap()
                .push(extra);
        },
    );

    c.variant(
        "rigged:two-base-drivers",
        Expect::Rejected,
        "one parameter fed by a channel mapping and a physics output at once",
        |doc| {
            doc.entity("channel_mappings", CM_MOUTH)
                .insert("parameter".into(), json!(P_HAIR_SWAY));
        },
    );
}

/// Shorten every keypoint's offset list by one vertex, so the deformation no
/// longer matches the mesh it applies to.
fn drop_one_deformation_offset(doc: &mut Doc) {
    let values = doc
        .binding(B_FACE_DEFORMATION)
        .get_mut("values")
        .and_then(Value::as_array_mut)
        .unwrap();
    for per_keypoint in values.iter_mut() {
        per_keypoint.as_array_mut().unwrap().pop();
    }
}

// ----------------------------------------------------------------- numbers

fn numbers(c: &mut Catalogue) {
    // JSON has no literal for a quiet NaN. A writer that is not ours emits the
    // bare token — Python's json module does, by default — so that is what the
    // fixture carries. README.md records that this one is not strict JSON and
    // the other two are.
    c.variant(
        "rigged:nan",
        Expect::Rejected,
        "a number this build interprets that is not a number",
        |doc| raw_rotation(doc, N_HAIR, "NaN"),
    );

    c.variant(
        "rigged:positive-infinity",
        Expect::Rejected,
        "a number this build interprets that overflows to positive infinity",
        |doc| raw_rotation(doc, N_HAIR_BACK, "1e400"),
    );

    c.variant(
        "rigged:negative-infinity",
        Expect::Rejected,
        "a number this build interprets that overflows to negative infinity",
        |doc| raw_rotation(doc, N_FACE_MASK, "-1e400"),
    );

    // Finite as a 64-bit literal, infinite once narrowed to the precision the
    // runtime evaluates in.
    c.variant(
        "rigged:nonfinite-after-narrowing",
        Expect::Rejected,
        "a value finite at 64 bits and infinite at the precision evaluation uses",
        |doc| {
            doc.node(N_FACE)
                .get_mut("transform")
                .and_then(Value::as_object_mut)
                .unwrap()
                .insert("translation".into(), json!([3.5e38, -8.0]));
        },
    );
}

// ------------------------------------------------------------- identifiers

fn identifiers(c: &mut Catalogue) {
    c.variant(
        "rigged:dup-id-same-kind",
        Expect::Rejected,
        "two parameters sharing an identifier",
        |doc| {
            doc.parameter(P_MOUTH_OPEN)
                .insert("id".into(), json!(P_HEAD_TURN));
        },
    );

    c.variant(
        "rigged:dup-id-cross-kind",
        Expect::Rejected,
        "a node and a texture sharing an identifier",
        |doc| {
            doc.node(N_PHYSICS_TIP).insert("id".into(), json!(T_FACE));
        },
    );

    c.variant(
        "rigged:id-above-high-water-mark",
        Expect::Rejected,
        "an identifier above the high-water mark the puppet records",
        |doc| {
            let joints = doc
                .node(N_HAIR)
                .get_mut("chain_deformer")
                .and_then(Value::as_object_mut)
                .unwrap()
                .get_mut("joints")
                .and_then(Value::as_array_mut)
                .unwrap();
            let last = joints.len() - 1;
            joints[last]
                .as_object_mut()
                .unwrap()
                .insert("id".into(), json!(HIGH_WATER_MARK + 1));
        },
    );

    c.variant(
        "rigged:id-zero",
        Expect::Rejected,
        "an entity carrying the reserved identifier zero",
        |doc| {
            let joints = doc
                .node(N_HAIR)
                .get_mut("chain_deformer")
                .and_then(Value::as_object_mut)
                .unwrap()
                .get_mut("joints")
                .and_then(Value::as_array_mut)
                .unwrap();
            let last = joints.len() - 1;
            joints[last]
                .as_object_mut()
                .unwrap()
                .insert("id".into(), json!(0));
        },
    );
}

// ---------------------------------------------------------------- physics

fn physics(c: &mut Catalogue) {
    for &(name, field, value, note) in DEGENERATE_PHYSICS {
        c.variant(name, Expect::Rejected, note, move |doc| {
            doc.node(N_PHYSICS).insert(field.into(), json!(value));
        });
    }
}

// --------------------------------------------------------------- textures

fn textures(c: &mut Catalogue) {
    // A texture no part references, as a third-party writer would produce.
    let mut doc = puppet::rigged();
    doc.obj()
        .get_mut("textures")
        .and_then(Value::as_array_mut)
        .unwrap()
        .push(json!({
            "authored_size": [16, 16],
            "crop": [0, 0, 16, 16],
            "id": 23,
            "name": "unused_draft",
        }));
    let mut payloads = puppet::texture_payloads();
    payloads.push((23, image::payload(16, 16, 4)));
    c.push(
        "rigged:orphan-texture",
        Expect::Accepted,
        "a fourth texture no part references",
        archive_of(&doc, VERSION_MAJOR, VERSION_MINOR, &payloads),
    );

    // A payload present but undecodable. Not a rejection: the texture is
    // reported unreadable, the part keeps its reference, and a save writes the
    // bytes back rather than re-encoding something it could not read.
    let doc = puppet::rigged();
    let mut payloads = puppet::texture_payloads();
    payloads[0] = (T_FACE, image::corrupt_payload());
    c.push(
        "rigged:texture-payload-corrupt",
        Expect::Accepted,
        "one texture payload present and undecodable",
        archive_of(&doc, VERSION_MAJOR, VERSION_MINOR, &payloads),
    );

    // Texture flips: one part with none, one with left–right, a third sharing
    // the same payload with both axes set.
    c.variant(
        "rigged:flipped-pair",
        Expect::Accepted,
        "three parts sharing one texture with none, one and both flip axes set",
        |doc| {
            doc.node(N_HAIR_BACK).insert(
                "texture_flip".into(),
                json!({"left_right": true, "top_bottom": false}),
            );
            let mut third = puppet::leaf_node(SPARE, "part", "hair_side", N_ROOT);
            third.as_object_mut().unwrap().insert(
                "texture_flip".into(),
                json!({"left_right": true, "top_bottom": true}),
            );
            doc.push_node(third);
        },
    );
}

// ---------------------------------------------- the container, not the JSON

fn containers(c: &mut Catalogue) {
    let doc = puppet::rigged();
    let manifest_bytes = manifest(VERSION_MAJOR, VERSION_MINOR).into_bytes();
    let puppet_bytes = doc.to_json().into_bytes();
    let payloads = puppet::texture_payloads();

    let honest_entries = |bomb: Option<RawEntry>| -> Vec<RawEntry> {
        let mut entries = vec![
            RawEntry::deflated("puppet.json", &puppet_bytes),
            RawEntry::deflated("rigel2d.json", &manifest_bytes),
        ];
        for (id, bytes) in &payloads {
            match &bomb {
                Some(_) if *id == T_FACE => {}
                _ => entries.push(RawEntry::stored(
                    &format!("textures/{id}.png"),
                    bytes.clone(),
                )),
            }
        }
        if let Some(bomb) = bomb {
            entries.insert(2, bomb);
        }
        entries
    };

    // Declares a size within any plausible limit and expands to a quarter of a
    // gigabyte. The limit has to be enforced against bytes actually
    // decompressed; the declaration is good only for refusing early.
    let expansion = vec![0u8; 256 * 1024 * 1024];
    let mut bomb = RawEntry::deflated(&format!("textures/{T_FACE}.png"), &expansion);
    bomb.declared_uncompressed = 1024;
    c.push(
        "rigged:decompression-bomb",
        Expect::Rejected,
        "an entry declaring 1 KiB and expanding to 256 MiB",
        rawzip::write(&honest_entries(Some(bomb))),
    );

    // An entry name at the largest a zip name field can express.
    let mut entries = honest_entries(None);
    let long = "a".repeat(u16::MAX as usize);
    entries.push(RawEntry::stored_named(
        long.into_bytes(),
        b"ignored".to_vec(),
    ));
    c.push(
        "rigged:entry-name-overlong",
        Expect::Rejected,
        "an entry name 65535 bytes long",
        rawzip::write(&entries),
    );

    // Two entries under one name, with different contents.
    let mut entries = honest_entries(None);
    entries.insert(
        1,
        RawEntry::deflated("puppet.json", br#"{"id":2,"name":"second"}"#),
    );
    c.push(
        "rigged:duplicate-entry-names",
        Expect::Rejected,
        "two entries named `puppet.json`, with different contents",
        rawzip::write(&entries),
    );

    // The second entry's central-directory offset points into the first
    // entry's data, so the two regions overlap.
    let mut entries = honest_entries(None);
    entries[1].offset_override = Some(40);
    c.push(
        "rigged:overlapping-local-header-data",
        Expect::Rejected,
        "a central-directory offset landing inside another entry's data",
        rawzip::write(&entries),
    );

    // An entry name the format does not define. Well formed as an archive,
    // which is why this one is written by the canonical writer.
    let mut archive_entries = vec![
        Entry::new("notes.txt", b"an entry the format does not define".to_vec()),
        Entry::new("puppet.json", puppet_bytes.clone()),
        Entry::new("rigel2d.json", manifest_bytes.clone()),
    ];
    for (id, bytes) in &payloads {
        archive_entries.push(Entry::new(format!("textures/{id}.png"), bytes.clone()));
    }
    c.push(
        "rigged:undefined-entry-name",
        Expect::Rejected,
        "an entry the format's table does not define",
        archive::write(archive_entries),
    );

    // `rigged` repacked the way ordinary tooling repacks it. `zip -r` emits a
    // directory marker for `textures/` by default, and a reader that refuses
    // one refuses every puppet a third party has opened and rezipped, which is
    // the whole of `TOOL-3`. Caught at task 9's gate.
    let mut marked = vec![
        Entry::new("puppet.json", puppet_bytes.clone()),
        Entry::new("rigel2d.json", manifest_bytes.clone()),
    ];
    for (id, bytes) in &payloads {
        marked.push(Entry::new(format!("textures/{id}.png"), bytes.clone()));
    }
    c.push(
        "rigged:directory-marker",
        Expect::Accepted,
        "a `textures/` directory marker, as ordinary repacking tooling emits",
        archive::write_with_directory_markers(marked.clone(), &["textures/"]),
    );

    // The same marker deflated rather than stored, which is `size == 0` with a
    // compressed size of two. The first fix for the gate's catch keyed on the
    // compressed size and so accepted the stored form and refused this one —
    // the same defect in a narrower shape, and the reason the boundary below
    // is now pinned from both sides.
    c.push(
        "rigged:directory-marker-deflated",
        Expect::Accepted,
        "a `textures/` directory marker written deflated, as Python's `zipfile` emits",
        archive::write_with_deflated_directory_markers(marked.clone(), &["textures/"]),
    );

    // A directory the format's entry table does not name. Tolerating markers
    // must not become tolerating any empty entry.
    c.push(
        "rigged:directory-marker-undefined",
        Expect::Rejected,
        "an `assets/` directory marker, which the entry table does not name",
        archive::write_with_directory_markers(marked.clone(), &["assets/"]),
    );

    // `textures/` carrying bytes is a name the format does not define, not a
    // marker, however the archive spells it.
    let mut fat = marked.clone();
    fat.push(Entry::new(
        "textures/",
        b"an entry wearing a marker's name".to_vec(),
    ));
    c.push(
        "rigged:directory-marker-not-empty",
        Expect::Rejected,
        "a `textures/` entry carrying bytes, which is not a marker",
        archive::write_naming_a_directory(fat),
    );
}
