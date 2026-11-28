//! `rigged` — the base valid puppet every other fixture is a variant of.
//!
//! Built item by item against the catalogue row in
//! `docs/requirements/puppet-acceptance.md`, and encoded against
//! `docs/format.md`. The row is long and dense; the comments below name which
//! clause each block satisfies so the two can be checked against each other
//! without reading the whole file.

use serde_json::{Map, Value, json};

use crate::json::Doc;

/// The format version this build knows.
///
/// The minor is 1 rather than 0 because `rigged:old-version` has to declare "an
/// earlier version this build upgrades" (VER-4), and an earlier *minor* only
/// exists if the current one is not the first. README.md carries the note.
pub const VERSION_MAJOR: u64 = 1;
pub const VERSION_MINOR: u64 = 1;

pub const PUPPET: u64 = 1;

// The node tree. Depth-first preorder is the canonical order of `nodes`, and
// order among siblings is draw order — the one thing order is allowed to decide.
pub const N_ROOT: u64 = 10; // composite, the root
pub const N_BODY: u64 = 11; // mesh group
pub const N_FACE: u64 = 12; // part, under the mesh group
pub const N_HAIR: u64 = 13; // part, under the mesh group
pub const N_PHYSICS: u64 = 14; // physics, under the mesh group
pub const N_PHYSICS_TIP: u64 = 15; // physics, under the physics node
pub const N_HAIR_BACK: u64 = 16; // part at root, sharing the hair texture
pub const N_FACE_MASK: u64 = 17; // part at root, masking the hair

pub const T_FACE: u64 = 20; // content inside transparent padding
pub const T_HAIR: u64 = 21; // shared by two parts
pub const T_MASK: u64 = 22;

pub const P_HEAD_TURN: u64 = 30; // one-dimensional, bilateral
pub const P_MOUTH_OPEN: u64 = 31; // one-dimensional, unipolar, fed by the mapping
pub const P_BODY_LEAN: u64 = 32; // two-dimensional, one axis of each polarity
pub const P_HAIR_SWAY: u64 = 33; // a single keypoint, driven by physics

pub const K_HEAD_LEFT: u64 = 300;
pub const K_HEAD_NEUTRAL: u64 = 301;
pub const K_HEAD_RIGHT: u64 = 302;
pub const K_MOUTH_SHUT: u64 = 310;
pub const K_MOUTH_OPEN: u64 = 311;
pub const K_LEAN_NEUTRAL: u64 = 322;

pub const B_FACE_TRANSFORM: u64 = 50;
pub const B_FACE_DEFORMATION: u64 = 51;
pub const B_BODY_TRANSFORM: u64 = 52;
pub const B_BACK_TRANSFORM: u64 = 53;

pub const A_IDLE: u64 = 60;
pub const TR_HEAD: u64 = 61;
pub const TR_LEAN: u64 = 62;

pub const CM_MOUTH: u64 = 70;

pub const CHAIN: u64 = 80;
pub const CHAIN_JOINT_TIP: u64 = 83;

pub const HIGH_WATER_MARK: u64 = 10000;

/// Identifiers from here up are free for a variant to allocate without
/// colliding with anything `rigged` carries. Still under the high-water mark.
pub const SPARE: u64 = 900;

/// The second extension namespace, carried beside the wiggle one on the mesh
/// group so that `rigged:ext-moved` has a pair to present in the other order.
pub fn notes_extension() -> Value {
    extension(
        "0.4",
        "Rig Notes",
        json!({"note": "the body group carries the lean"}),
    )
}

/// The provisional node-count load limit `rigged:near-limit` sits just under.
///
/// Provisional: `LIM-4` ties every limit to four times the `stress` workload's
/// corresponding dimension, and that workload is S3's. README.md says what has
/// to happen when the real figure arrives.
pub const PROVISIONAL_NODE_LIMIT: usize = 256;

/// The provisional limit on one animation's `triggers`.
///
/// `triggers` is a new stored collection, so `LIM-8` requires a figure and
/// `LIM-1` requires a fixture just over it. No `stress` dimension counts
/// triggers and none ever will — an animation waits on a handful of scene
/// events — so this is a `Provisional` figure in `limits.rs`'s sense: far past
/// any authored rig and far below anything that costs memory. **The figure in
/// `crates/rigel-format/src/limits.rs` must match this one**, or
/// `rigged:triggers-over-limit` stops being just over anything.
pub const PROVISIONAL_TRIGGERS_PER_ANIMATION: usize = 64;

/// The end behaviours `docs/format.md` *How an animation ends* defines. Always
/// written, because no default is stated for the field.
pub const END_LOOP: &str = "loop";
pub const END_HOLD: &str = "hold";
pub const END_RELEASE: &str = "release";

/// The interpolation modes `docs/format.md` *How a segment interpolates*
/// defines beyond linear. **Linear is the absent form** and so has no constant:
/// a track whose segments are all linear stores no mode anywhere.
pub const INTERP_STEPPED: &str = "stepped";
pub const INTERP_INVERSE_STEPPED: &str = "inverse_stepped";
pub const INTERP_BEZIER: &str = "bezier";

/// The two scene events `docs/format.md` *A trigger* names a use for. The set
/// is open: an unrecognised name never fires and is never a validation failure,
/// which is what `rigged:triggered` carries a third name for.
pub const EVENT_ON_LOAD: &str = "on_load";
pub const EVENT_SOURCE_LOST: &str = "source_lost";

/// One trigger: the name of the scene event its animation waits for, and
/// nothing else. No identifier, no target, no command — see `docs/format.md`.
pub fn trigger(event: &str) -> Value {
    json!({"event": event})
}

/// One keyframe of a one-axis track.
pub fn keyframe(time: f64, value: f64) -> Value {
    json!({"time": time, "value": [value]})
}

/// A keyframe that opens a segment interpolating in a stated mode. The mode
/// belongs to the **earlier** keyframe of its segment.
pub fn keyframe_with(time: f64, value: f64, mode: &str) -> Value {
    let mut frame = keyframe(time, value);
    frame
        .as_object_mut()
        .expect("keyframe is an object")
        .insert("interpolation".into(), json!(mode));
    frame
}

/// A keyframe opening a Bézier segment, carrying the two control points the
/// cubic passes through.
///
/// A control point is shaped like a keyframe — a time and a value per axis — so
/// that its arity follows the parameter's axes the way a keyframe's does, and
/// so that a reader meets one shape rather than two.
pub fn keyframe_bezier(time: f64, value: f64, controls: [(f64, f64); 2]) -> Value {
    let mut frame = keyframe_with(time, value, INTERP_BEZIER);
    frame.as_object_mut().unwrap().insert(
        "control_points".into(),
        json!([
            keyframe(controls[0].0, controls[0].1),
            keyframe(controls[1].0, controls[1].1),
        ]),
    );
    frame
}

/// `rigel2d.json`: the format name and version, and nothing else.
pub fn manifest(major: u64, minor: u64) -> String {
    serde_json::to_string(&json!({
        "format": "rigel2d-puppet",
        "version": {"major": major, "minor": minor},
    }))
    .expect("manifest is serialisable")
}

fn transform(tx: f64, ty: f64, rotation: f64, sx: f64, sy: f64) -> Value {
    json!({"rotation": rotation, "scale": [sx, sy], "translation": [tx, ty]})
}

fn identity() -> Value {
    transform(0.0, 0.0, 0.0, 1.0, 1.0)
}

/// A four-vertex quad in the authored pixel space of its texture.
///
/// `uvs` are in the texture's **authored** space, which is what lets the loader
/// reconstruct that space from the crop rectangle and the authored dimensions.
fn quad(width: f64, height: f64) -> Value {
    json!({
        "triangles": [[0, 1, 2], [0, 2, 3]],
        "uvs": [[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]],
        "vertices": [[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]],
    })
}

/// Import provenance: a source identifier and a content hash, and nothing that
/// is a path, a username or a machine name (FILE-5, CANON-2).
fn provenance(source_id: &str, hash: &str) -> Value {
    json!({"content_hash": hash, "source_id": source_id})
}

/// One extension-data entry: the marker is the namespace key it sits under.
pub fn extension(version: &str, name: &str, data: Value) -> Value {
    json!({"data": data, "name": name, "version": version})
}

/// The wiggle extension `rigged` carries at two places in the tree. No build
/// understands this namespace, which is the point of it.
pub fn wiggle_extension() -> Value {
    extension(
        "1.2",
        "Wiggle Deformer",
        json!({"amplitude": 0.25, "axis": "horizontal", "segments": 4}),
    )
}

/// The same values with the marker gone: no namespace, no version, no name.
/// `rigged:ext-unmarked` and `rigged:future-minor` both carry exactly this, at
/// exactly this position, which is what makes EXT-8's differential assertion a
/// difference of declared version alone.
pub fn wiggle_unmarked() -> Value {
    json!({"amplitude": 0.25, "axis": "horizontal", "segments": 4})
}

fn node(id: u64, kind: &str, name: &str, parent: u64, local: Value) -> Value {
    json!({"id": id, "kind": kind, "name": name, "parent": parent, "transform": local})
}

/// A leaf node of a given kind, for the containment fixtures. A part needs a
/// texture and a mesh to be a part at all; the other kinds are complete empty.
pub fn leaf_node(id: u64, kind: &str, name: &str, parent: u64) -> Value {
    let mut value = node(id, kind, name, parent, identity());
    if kind == "part" {
        let object = value.as_object_mut().expect("node is an object");
        object.insert("texture".into(), json!(T_HAIR));
        object.insert("mesh".into(), quad(48.0, 16.0));
        object.insert(
            "provenance".into(),
            provenance(
                "layer-added",
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        );
    }
    value
}

fn nodes() -> Vec<Value> {
    // The root is a composite, which is a kind that contains.
    let root = node(N_ROOT, "composite", "root", 0, identity());

    // A mesh group with two part children, and the physics subtree under it.
    let mut body = node(N_BODY, "mesh_group", "body", N_ROOT, identity());
    body.as_object_mut().unwrap().insert(
        "mesh".into(),
        json!({
            "triangles": [[0, 1, 2], [0, 2, 3]],
            "uvs": [[0.0, 0.0], [128.0, 0.0], [128.0, 128.0], [0.0, 128.0]],
            "vertices": [[-64.0, -64.0], [64.0, -64.0], [64.0, 64.0], [-64.0, 64.0]],
        }),
    );
    // Extension data, site one of two: two namespaces, so `rigged:ext-moved` has
    // a pair to present in the other order.
    body.as_object_mut().unwrap().insert(
        "x".into(),
        json!({
            "com.alt.notes": notes_extension(),
            "com.example.wiggle": wiggle_extension(),
        }),
    );

    let mut face = node(
        N_FACE,
        "part",
        "face",
        N_BODY,
        transform(0.0, -8.0, 0.0, 1.0, 1.0),
    );
    {
        let object = face.as_object_mut().unwrap();
        object.insert("texture".into(), json!(T_FACE));
        object.insert("mesh".into(), quad(64.0, 64.0));
        object.insert(
            "provenance".into(),
            provenance(
                "layer-face",
                "1f0a3c5d7e9b1d3f5a7c9e0b2d4f6a8c0e1b3d5f7a9c0e2b4d6f8a0c2e4b6d8f",
            ),
        );
        // Extension data, site two of two.
        object.insert(
            "x".into(),
            json!({"com.example.wiggle": wiggle_extension()}),
        );
    }

    let mut hair = node(
        N_HAIR,
        "part",
        "hair",
        N_BODY,
        transform(0.0, -40.0, 0.0, 1.0, 1.0),
    );
    {
        let object = hair.as_object_mut().unwrap();
        object.insert("texture".into(), json!(T_HAIR));
        object.insert("mesh".into(), quad(48.0, 16.0));
        // One part acting as a mask for another: the reference is held by the
        // part that is masked, so deleting the mask leaves this one unmasked.
        object.insert("mask".into(), json!(N_FACE_MASK));
        // A chain deformer, a permanent part of the rig rather than a bulk edit.
        object.insert(
            "chain_deformer".into(),
            json!({
                "id": CHAIN,
                "joints": [
                    {"id": 81, "position": [24.0, 0.0]},
                    {"id": 82, "position": [24.0, 6.0]},
                    {"id": CHAIN_JOINT_TIP, "position": [24.0, 12.0]},
                ],
            }),
        );
        object.insert(
            "provenance".into(),
            provenance(
                "layer-hair",
                "2b4d6f8a0c2e4b6d8f1f0a3c5d7e9b1d3f5a7c9e0b2d4f6a8c0e1b3d5f7a9c0e",
            ),
        );
    }

    // A physics node anchored under the mesh group, offset from its anchor,
    // driving one parameter nothing else drives, containing a second physics
    // node that drives nothing. Each carries its own gravity multiplier. Every
    // field at its default is absent, which is the stored-form rule.
    let mut physics = node(
        N_PHYSICS,
        "physics",
        "hair_physics",
        N_BODY,
        transform(18.0, -36.0, 0.0, 1.0, 1.0),
    );
    {
        let object = physics.as_object_mut().unwrap();
        object.insert("length".into(), json!(120.0));
        object.insert("gravity".into(), json!(0.8));
        object.insert("frequency".into(), json!(1.4));
        object.insert("angle_damping".into(), json!(0.35));
        object.insert("length_damping".into(), json!(0.6));
        object.insert("output".into(), json!(P_HAIR_SWAY));
        object.insert("output_source".into(), json!(["length"]));
        object.insert("output_scale".into(), json!([0.75]));
    }

    let mut physics_tip = node(
        N_PHYSICS_TIP,
        "physics",
        "hair_physics_tip",
        N_PHYSICS,
        transform(0.0, 40.0, 0.0, 1.0, 1.0),
    );
    {
        let object = physics_tip.as_object_mut().unwrap();
        object.insert("length".into(), json!(60.0));
        object.insert("gravity".into(), json!(0.4));
        object.insert("frequency".into(), json!(2.0));
        object.insert("angle_damping".into(), json!(0.3));
        object.insert("length_damping".into(), json!(0.55));
        // `output` absent: this one drives nothing, which is legal.
    }

    // A third part at root referencing the same texture as one of the two above.
    let mut hair_back = node(
        N_HAIR_BACK,
        "part",
        "hair_back",
        N_ROOT,
        transform(0.0, -44.0, 0.0, 1.0, 1.0),
    );
    {
        let object = hair_back.as_object_mut().unwrap();
        object.insert("texture".into(), json!(T_HAIR));
        object.insert("mesh".into(), quad(48.0, 16.0));
        object.insert(
            "provenance".into(),
            provenance(
                "layer-hair-back",
                "3c5d7e9b1d3f5a7c9e0b2d4f6a8c0e1b3d5f7a9c0e2b4d6f8a0c2e4b6d8f1f0a",
            ),
        );
    }

    let mut face_mask = node(
        N_FACE_MASK,
        "part",
        "face_mask",
        N_ROOT,
        transform(0.0, -8.0, 0.0, 1.0, 1.0),
    );
    {
        let object = face_mask.as_object_mut().unwrap();
        object.insert("texture".into(), json!(T_MASK));
        object.insert("mesh".into(), quad(32.0, 24.0));
        object.insert(
            "provenance".into(),
            provenance(
                "layer-face-mask",
                "4d6f8a0c2e4b6d8f1f0a3c5d7e9b1d3f5a7c9e0b2d4f6a8c0e1b3d5f7a9c0e2b",
            ),
        );
    }

    vec![
        root,
        body,
        face,
        hair,
        physics,
        physics_tip,
        hair_back,
        face_mask,
    ]
}

/// Three textures of distinct dimensions, one of them content surrounded by
/// transparent padding of known extent.
fn textures() -> Vec<Value> {
    vec![
        json!({
            "authored_size": [64, 64],
            "crop": [16, 12, 24, 20],
            "id": T_FACE,
            "name": "face",
        }),
        json!({
            "authored_size": [48, 16],
            "crop": [0, 0, 48, 16],
            "id": T_HAIR,
            "name": "hair",
        }),
        json!({
            "authored_size": [32, 24],
            "crop": [0, 0, 32, 24],
            "id": T_MASK,
            "name": "face_mask",
        }),
    ]
}

/// The payload bytes for each texture record, cropped to the crop rectangle.
pub fn texture_payloads() -> Vec<(u64, Vec<u8>)> {
    vec![
        (T_FACE, crate::image::payload(24, 20, 1)),
        (T_HAIR, crate::image::payload(48, 16, 2)),
        (T_MASK, crate::image::payload(32, 24, 3)),
    ]
}

/// Four parameters: two one-dimensional, one two-dimensional, one with a single
/// keypoint. Each declares its axis range and carries exactly one neutral
/// keypoint, and each axis declares its own polarity.
fn parameters() -> Vec<Value> {
    vec![
        json!({
            "axes": [{"polarity": "bilateral", "range": [-1.0, 1.0]}],
            "id": P_HEAD_TURN,
            "keypoints": [
                {"id": K_HEAD_LEFT, "position": [-1.0]},
                {"id": K_HEAD_NEUTRAL, "neutral": true, "position": [0.0]},
                {"id": K_HEAD_RIGHT, "position": [1.0]},
            ],
            "name": "head_turn",
        }),
        json!({
            "axes": [{"polarity": "unipolar", "range": [0.0, 1.0]}],
            "id": P_MOUTH_OPEN,
            "keypoints": [
                {"id": K_MOUTH_SHUT, "neutral": true, "position": [0.0]},
                {"id": K_MOUTH_OPEN, "position": [1.0]},
            ],
            "name": "mouth_open",
        }),
        json!({
            "axes": [
                {"polarity": "bilateral", "range": [-1.0, 1.0]},
                {"polarity": "unipolar", "range": [0.0, 1.0]},
            ],
            "id": P_BODY_LEAN,
            "keypoints": [
                {"id": 320, "position": [-1.0, 0.0]},
                {"id": 321, "position": [-1.0, 1.0]},
                {"id": K_LEAN_NEUTRAL, "neutral": true, "position": [0.0, 0.0]},
                {"id": 323, "position": [0.0, 1.0]},
                {"id": 324, "position": [1.0, 0.0]},
                {"id": 325, "position": [1.0, 1.0]},
            ],
            "name": "body_lean",
        }),
        json!({
            "axes": [{"polarity": "bilateral", "range": [-1.0, 1.0]}],
            "id": P_HAIR_SWAY,
            "keypoints": [{"id": 330, "neutral": true, "position": [0.0]}],
            "name": "hair_sway",
        }),
    ]
}

fn delta(tx: f64, ty: f64, rotation: f64) -> Value {
    transform(tx, ty, rotation, 1.0, 1.0)
}

/// Against one node — the face part — both a transform binding and a
/// deformation binding. The deformation's value count matches its mesh, so it
/// is not stale and carries no mark.
fn bindings() -> Vec<Value> {
    vec![
        json!({
            "id": B_FACE_TRANSFORM,
            "kind": "transform",
            "parameter": P_HEAD_TURN,
            "target": N_FACE,
            "values": [delta(-6.0, 0.0, -0.08), delta(0.0, 0.0, 0.0), delta(6.0, 0.0, 0.08)],
        }),
        json!({
            "id": B_FACE_DEFORMATION,
            "kind": "deformation",
            "parameter": P_HEAD_TURN,
            "target": N_FACE,
            "values": [
                [[-2.0, 0.0], [-1.0, 0.0], [-1.0, 1.0], [-2.0, 1.0]],
                [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                [[2.0, 0.0], [1.0, 0.0], [1.0, -1.0], [2.0, -1.0]],
            ],
        }),
        json!({
            "id": B_BODY_TRANSFORM,
            "kind": "transform",
            "parameter": P_BODY_LEAN,
            "target": N_BODY,
            "values": [
                delta(-10.0, 0.0, -0.12),
                delta(-10.0, -4.0, -0.12),
                delta(0.0, 0.0, 0.0),
                delta(0.0, -4.0, 0.0),
                delta(10.0, 0.0, 0.12),
                delta(10.0, -4.0, 0.12),
            ],
        }),
        // A binding on the parameter the physics node drives, on a node that is
        // **not** an ancestor of that physics node. `rigged:physics-drives-ancestor`
        // is this binding repointed at one that is.
        json!({
            "id": B_BACK_TRANSFORM,
            "kind": "transform",
            "parameter": P_HAIR_SWAY,
            "target": N_HAIR_BACK,
            "values": [delta(0.0, 0.0, 0.0)],
        }),
    ]
}

/// An animation carrying one track for each of two parameters.
///
/// **`end_behaviour` is written**, because `docs/format.md` states no default
/// for it and a field with no stated default is always written. It is the one
/// field of the 2026-11-28 animation round that `rigged` itself must carry:
/// every other one is absent in its default form, and `rigged` is the control
/// the variants below are read against.
///
/// So no track carries an `interpolation` — every segment here is linear, which
/// is the absent form, and `INTERP-2` asserts that an all-linear track stores
/// no mode anywhere. No animation carries `triggers`, no parameter carries an
/// `order`, and no parameter carries `mappable`, which is absent when true.
fn animations() -> Vec<Value> {
    vec![json!({
        "end_behaviour": END_LOOP,
        "id": A_IDLE,
        "name": "idle",
        "tracks": [
            {
                "blend_duration": 0.25,
                "id": TR_HEAD,
                "keyframes": [
                    {"time": 0.0, "value": [0.0]},
                    {"time": 0.5, "value": [0.5]},
                    {"time": 1.0, "value": [0.0]},
                ],
                "parameter": P_HEAD_TURN,
            },
            {
                "blend_duration": 0.1,
                "id": TR_LEAN,
                "keyframes": [
                    {"time": 0.0, "value": [0.0, 0.0]},
                    {"time": 1.0, "value": [0.5, 0.25]},
                ],
                "parameter": P_BODY_LEAN,
            },
        ],
    })]
}

/// A channel mapping from one named channel to a parameter no physics node
/// drives, with a response curve over the normalised channel.
fn channel_mappings() -> Vec<Value> {
    vec![json!({
        "channel": "mouth_open",
        "id": CM_MOUTH,
        "parameter": P_MOUTH_OPEN,
        "response_curve": {"points": [[0.0, 0.0], [0.5, 0.7], [1.0, 1.0]]},
        "source": "",
    })]
}

/// `rigged` itself.
///
/// No `mirror_axes` key: the axes default to derived, and MAXIS-1 asserts the
/// base fixture stores none.
pub fn rigged() -> Doc {
    let mut root = Map::new();
    root.insert("animations".into(), Value::Array(animations()));
    root.insert("bindings".into(), Value::Array(bindings()));
    root.insert("channel_mappings".into(), Value::Array(channel_mappings()));
    root.insert("composition_mode".into(), json!("sum"));
    root.insert("gravity".into(), json!(1.6));
    root.insert("high_water_mark".into(), json!(HIGH_WATER_MARK));
    root.insert("id".into(), json!(PUPPET));
    root.insert("name".into(), json!("rigged"));
    root.insert("nodes".into(), Value::Array(nodes()));
    root.insert("parameters".into(), Value::Array(parameters()));
    root.insert("root".into(), json!(N_ROOT));
    root.insert("textures".into(), Value::Array(textures()));
    Doc::new(Value::Object(root))
}

/// `one-part`: the smallest puppet `docs/format.md` calls valid — a root that
/// contains, one part, one texture, nothing else.
///
/// README.md records why this is not literally "one root part": the format
/// requires the root to be a kind that contains.
pub fn one_part() -> Doc {
    let mut part = node(N_FACE, "part", "face", N_ROOT, identity());
    {
        let object = part.as_object_mut().unwrap();
        object.insert("texture".into(), json!(T_FACE));
        object.insert("mesh".into(), quad(64.0, 64.0));
        object.insert(
            "provenance".into(),
            provenance(
                "layer-face",
                "1f0a3c5d7e9b1d3f5a7c9e0b2d4f6a8c0e1b3d5f7a9c0e2b4d6f8a0c2e4b6d8f",
            ),
        );
    }
    let mut root = Map::new();
    root.insert("composition_mode".into(), json!("sum"));
    root.insert("high_water_mark".into(), json!(HIGH_WATER_MARK));
    root.insert("id".into(), json!(2));
    root.insert("name".into(), json!("one-part"));
    root.insert(
        "nodes".into(),
        json!([node(N_ROOT, "composite", "root", 0, identity()), part]),
    );
    root.insert("root".into(), json!(N_ROOT));
    root.insert(
        "textures".into(),
        json!([{
            "authored_size": [64, 64],
            "crop": [16, 12, 24, 20],
            "id": T_FACE,
            "name": "face",
        }]),
    );
    Doc::new(Value::Object(root))
}
