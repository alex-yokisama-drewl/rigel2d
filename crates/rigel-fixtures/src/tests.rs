//! Unit tests over the corpus the builder produces.
//!
//! The expensive half of this task is checking a long, dense catalogue row item
//! by item, and checking a rule — one defect per fixture, the current version
//! unless the name says otherwise — across seventy-odd files. Both are done
//! here rather than by reading, because both are exactly the kind of thing a
//! reader skips.

use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::sync::OnceLock;

use serde_json::Value;

use crate::fixtures::{self, Expect, Fixture};
use crate::puppet::{self, *};

/// Fixtures whose defect is in the archive container, so a zip reader is
/// entitled to refuse them before any entry is reached.
const MALFORMED_CONTAINER: &[&str] = &[
    "rigged:decompression-bomb",
    "rigged:entry-name-overlong",
    "rigged:duplicate-entry-names",
    "rigged:overlapping-local-header-data",
];

/// Fixtures that are valid puppets in a container the canonical writer would
/// not have produced. They carry every entry the format defines, so the
/// structural assertions still apply; the canonical-archive ones do not,
/// because the point of the fixture is that a reader accepts a container
/// ordinary tooling wrote.
const NONCANONICAL_CONTAINER: &[&str] = &[
    "rigged:directory-marker",
    "rigged:directory-marker-deflated",
    "rigged:directory-marker-undefined",
    "rigged:directory-marker-not-empty",
];

/// Fixtures whose name says they declare something other than the current
/// version.
fn declares_another_version(name: &str) -> bool {
    name.contains("future-minor")
        || name.contains("old-version")
        || name.contains("breaking-version")
}

/// The corpus, built once for the whole test binary. Building it is not cheap —
/// the decompression bomb alone deflates a quarter of a gigabyte — and every
/// test below wants the same copy.
fn corpus() -> &'static [Fixture] {
    static CORPUS: OnceLock<Vec<Fixture>> = OnceLock::new();
    CORPUS.get_or_init(fixtures::all)
}

fn entry(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut out = Vec::new();
    file.read_to_end(&mut out).ok()?;
    Some(out)
}

fn puppet_json(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(&entry(bytes, "puppet.json")?).ok()
}

fn fixture(name: &str) -> &'static Fixture {
    corpus()
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no fixture {name}"))
}

/// The parsed `puppet.json` of a named fixture.
fn parsed(name: &str) -> Value {
    puppet_json(&fixture(name).bytes).expect("fixture carries a parseable puppet.json")
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("no `{key}` collection"))
        .as_slice()
}

fn node_by_id(puppet: &Value, id: u64) -> &Value {
    array(puppet, "nodes")
        .iter()
        .find(|n| n.get("id").and_then(Value::as_u64) == Some(id))
        .unwrap_or_else(|| panic!("no node {id}"))
}

#[test]
fn names_and_files_are_unique() {
    let corpus = corpus();
    let names: BTreeSet<_> = corpus.iter().map(|f| f.name.clone()).collect();
    let files: BTreeSet<_> = corpus.iter().map(|f| f.file.clone()).collect();
    assert_eq!(names.len(), corpus.len(), "fixture names collide");
    assert_eq!(files.len(), corpus.len(), "fixture file names collide");
}

#[test]
fn file_names_are_portable() {
    // The push gates check the tree out on Windows, where `:` cannot appear in
    // a path. Catching it here is cheaper than a red matrix job.
    for fixture in corpus() {
        assert!(
            !fixture.file.contains(':'),
            "{} is not a legal Windows path",
            fixture.file
        );
    }
}

#[test]
fn the_builder_is_deterministic() {
    let first = corpus();
    let second = fixtures::all();
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.bytes, b.bytes, "{} is not reproducible", a.name);
    }
}

#[test]
fn every_fixture_declares_the_current_version_unless_its_name_says_otherwise() {
    for fixture in corpus() {
        let Some(bytes) = entry(&fixture.bytes, "rigel2d.json") else {
            assert!(
                MALFORMED_CONTAINER.contains(&fixture.name.as_str()),
                "{} has no readable rigel2d.json and is not a container fixture",
                fixture.name
            );
            continue;
        };
        let manifest: Value = serde_json::from_slice(&bytes).expect("manifest parses");
        let version = &manifest["version"];
        if declares_another_version(&fixture.name) {
            assert_ne!(
                (version["major"].as_u64(), version["minor"].as_u64()),
                (Some(VERSION_MAJOR), Some(VERSION_MINOR)),
                "{} is named for another version and declares the current one",
                fixture.name
            );
        } else {
            assert_eq!(
                (version["major"].as_u64(), version["minor"].as_u64()),
                (Some(VERSION_MAJOR), Some(VERSION_MINOR)),
                "{} must declare the current version",
                fixture.name
            );
        }
    }
}

#[test]
fn the_manifest_carries_the_version_and_nothing_else() {
    let bytes = entry(&fixture("rigged").bytes, "rigel2d.json").expect("rigged has a manifest");
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    let keys: Vec<_> = manifest.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, vec!["format".to_string(), "version".to_string()]);
}

#[test]
fn every_well_formed_fixture_carries_the_entries_the_format_defines() {
    for fixture in corpus() {
        if MALFORMED_CONTAINER.contains(&fixture.name.as_str()) {
            continue;
        }
        let mut archive = zip::ZipArchive::new(Cursor::new(fixture.bytes.clone()))
            .unwrap_or_else(|error| panic!("{} is not a readable archive: {error}", fixture.name));
        let names: Vec<String> = (0..archive.len())
            .map(|index| archive.by_index(index).unwrap().name().unwrap().to_string())
            .collect();
        assert!(
            names.contains(&"puppet.json".to_string()),
            "{} has no puppet.json",
            fixture.name
        );
        assert!(
            names.contains(&"rigel2d.json".to_string()),
            "{} has no rigel2d.json",
            fixture.name
        );
        assert!(
            names.iter().any(|n| n.starts_with("textures/")),
            "{} has no texture payload",
            fixture.name
        );
        // A non-canonical fixture is exempt from exactly its stated
        // deviation and from nothing else: the entries that are not directory
        // names must still be sorted, and it may carry at most the one
        // directory entry that is the point of the fixture. Exempting the
        // whole assertion would let a second stray entry in unnoticed.
        let directories: Vec<&String> = names.iter().filter(|n| n.ends_with('/')).collect();
        if NONCANONICAL_CONTAINER.contains(&fixture.name.as_str()) {
            assert!(
                directories.len() <= 1,
                "{} carries more than one directory entry: {directories:?}",
                fixture.name
            );
            let mut rest: Vec<&String> = names.iter().filter(|n| !n.ends_with('/')).collect();
            let ordered = rest.clone();
            rest.sort();
            assert_eq!(
                rest, ordered,
                "{}'s non-directory entries are not sorted",
                fixture.name
            );
            continue;
        }
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(sorted, names, "{}'s entries are not sorted", fixture.name);
        assert!(
            directories.is_empty(),
            "{} carries a directory entry",
            fixture.name
        );
    }
}

#[test]
fn rigged_is_canonical_json() {
    // Reparsing into a BTreeMap and reserialising reproduces canonical form:
    // keys sorted, no insignificant whitespace, one number format. If the
    // committed bytes already are that, the two texts are equal.
    for name in ["rigged", "one-part"] {
        let bytes = entry(&fixture(name).bytes, "puppet.json").unwrap();
        let text = String::from_utf8(bytes).expect("puppet.json is UTF-8");
        let value: Value = serde_json::from_str(&text).expect("puppet.json parses");
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            text,
            "{name} is not canonically serialised"
        );
    }
}

#[test]
fn nothing_but_the_hostile_fixture_carries_a_path_a_url_or_a_command() {
    // docs/invariants.md I1. A fixture that quietly acquired a path-shaped
    // string would make every criterion asserting the hostile case weaker, and
    // nothing else would notice.
    fn strings(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(text) => out.push(text.clone()),
            Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
            Value::Object(map) => {
                for (key, item) in map {
                    out.push(key.clone());
                    strings(item, out);
                }
            }
            _ => {}
        }
    }
    for fixture in corpus() {
        if fixture.name == "rigged:ext-hostile" {
            continue;
        }
        let Some(puppet) = puppet_json(&fixture.bytes) else {
            continue;
        };
        let mut found = Vec::new();
        strings(&puppet, &mut found);
        for text in found {
            for shape in ["/", "\\", "://", "$(", "%n"] {
                assert!(
                    !text.contains(shape),
                    "{} carries {text:?}, which is shaped like a path or a command",
                    fixture.name
                );
            }
        }
    }
}

#[test]
fn every_defect_fixture_differs_from_rigged() {
    let rigged = fixture("rigged");
    for fixture in corpus() {
        if fixture.name == "rigged" || fixture.expect != Expect::Rejected {
            continue;
        }
        assert_ne!(
            fixture.bytes, rigged.bytes,
            "{} is byte-identical to rigged and so carries no defect",
            fixture.name
        );
    }
}

/// `rigged` carries everything the catalogue row says it carries.
///
/// The row is one long sentence and it is easy to lose a clause in it, so each
/// clause is a block here with the clause quoted above it.
#[test]
fn rigged_carries_everything_the_catalogue_row_names() {
    let puppet = &parsed("rigged");

    // "A root composite"
    assert_eq!(puppet["root"].as_u64(), Some(N_ROOT));
    assert_eq!(node_by_id(puppet, N_ROOT)["kind"], "composite");
    assert_eq!(node_by_id(puppet, N_ROOT)["parent"].as_u64(), Some(0));

    // "a mesh group with two part children"
    assert_eq!(node_by_id(puppet, N_BODY)["kind"], "mesh_group");
    let part_children: Vec<u64> = array(puppet, "nodes")
        .iter()
        .filter(|n| n["parent"].as_u64() == Some(N_BODY) && n["kind"] == "part")
        .map(|n| n["id"].as_u64().unwrap())
        .collect();
    assert_eq!(part_children, vec![N_FACE, N_HAIR]);

    // "a third part at root referencing the same texture as one of those two"
    assert_eq!(
        node_by_id(puppet, N_HAIR_BACK)["parent"].as_u64(),
        Some(N_ROOT)
    );
    assert_eq!(
        node_by_id(puppet, N_HAIR_BACK)["texture"],
        node_by_id(puppet, N_HAIR)["texture"]
    );

    // "one part acting as a mask for another"
    assert_eq!(
        node_by_id(puppet, N_HAIR)["mask"].as_u64(),
        Some(N_FACE_MASK)
    );
    assert_eq!(node_by_id(puppet, N_FACE_MASK)["kind"], "part");

    // "three textures of distinct dimensions, one of them content surrounded by
    // transparent padding of known extent"
    let textures = array(puppet, "textures");
    assert_eq!(textures.len(), 3);
    let sizes: BTreeSet<String> = textures
        .iter()
        .map(|t| t["authored_size"].to_string())
        .collect();
    assert_eq!(sizes.len(), 3, "texture dimensions are not distinct");
    let padded = textures
        .iter()
        .find(|t| t["id"].as_u64() == Some(T_FACE))
        .unwrap();
    assert_eq!(padded["authored_size"], serde_json::json!([64, 64]));
    assert_eq!(padded["crop"], serde_json::json!([16, 12, 24, 20]));

    // "four parameters — two one-dimensional, one two-dimensional, one with a
    // single keypoint — each declaring its axis range and carrying exactly one
    // neutral keypoint"
    let parameters = array(puppet, "parameters");
    assert_eq!(parameters.len(), 4);
    let dimensions: Vec<usize> = parameters
        .iter()
        .map(|p| p["axes"].as_array().unwrap().len())
        .collect();
    assert_eq!(dimensions.iter().filter(|d| **d == 1).count(), 3);
    assert_eq!(dimensions.iter().filter(|d| **d == 2).count(), 1);
    assert!(
        parameters
            .iter()
            .any(|p| p["keypoints"].as_array().unwrap().len() == 1),
        "no parameter carries a single keypoint"
    );
    for parameter in parameters {
        for axis in parameter["axes"].as_array().unwrap() {
            assert!(axis.get("range").is_some(), "an axis declares no range");
            assert!(
                axis.get("polarity").is_some(),
                "an axis declares no polarity"
            );
        }
        let neutral = parameter["keypoints"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|k| k.get("neutral") == Some(&Value::Bool(true)))
            .count();
        assert_eq!(neutral, 1, "a parameter has {neutral} neutral keypoints");
    }
    // "one bilateral axis and one unipolar" on the two-dimensional one (POL-1)
    let two_dimensional = parameters
        .iter()
        .find(|p| p["axes"].as_array().unwrap().len() == 2)
        .unwrap();
    let polarities: Vec<&str> = two_dimensional["axes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["polarity"].as_str().unwrap())
        .collect();
    assert_eq!(polarities, vec!["bilateral", "unipolar"]);

    // "against one node both a transform binding and a deformation binding"
    let on_face: BTreeSet<&str> = array(puppet, "bindings")
        .iter()
        .filter(|b| b["target"].as_u64() == Some(N_FACE))
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        on_face,
        ["deformation", "transform"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    // and that deformation's value count matches its mesh, so it is not stale
    let deformation = array(puppet, "bindings")
        .iter()
        .find(|b| b["id"].as_u64() == Some(B_FACE_DEFORMATION))
        .unwrap();
    let vertices = node_by_id(puppet, N_FACE)["mesh"]["vertices"]
        .as_array()
        .unwrap()
        .len();
    for per_keypoint in deformation["values"].as_array().unwrap() {
        assert_eq!(per_keypoint.as_array().unwrap().len(), vertices);
    }
    assert!(deformation.get("stale").is_none());

    // "a chain deformer on one part"
    let chain = &node_by_id(puppet, N_HAIR)["chain_deformer"];
    assert_eq!(chain["id"].as_u64(), Some(CHAIN));
    assert_eq!(chain["joints"].as_array().unwrap().len(), 3);

    // "a physics subtree — a physics node anchored under the mesh group, offset
    // from its anchor and driving one parameter that nothing else drives,
    // containing a second physics node that drives nothing — each carrying its
    // own gravity multiplier"
    let head = node_by_id(puppet, N_PHYSICS);
    assert_eq!(head["kind"], "physics");
    assert_eq!(head["parent"].as_u64(), Some(N_BODY));
    assert_ne!(
        head["transform"]["translation"],
        serde_json::json!([0.0, 0.0])
    );
    assert_eq!(head["output"].as_u64(), Some(P_HAIR_SWAY));
    let tip = node_by_id(puppet, N_PHYSICS_TIP);
    assert_eq!(tip["kind"], "physics");
    assert_eq!(tip["parent"].as_u64(), Some(N_PHYSICS));
    assert!(tip.get("output").is_none(), "the tip drives something");
    assert!(head["gravity"].is_number() && tip["gravity"].is_number());
    // nothing else drives P_HAIR_SWAY
    assert!(
        !array(puppet, "channel_mappings")
            .iter()
            .any(|m| m["parameter"].as_u64() == Some(P_HAIR_SWAY))
    );

    // "one gravity for the puppet"
    assert!(puppet["gravity"].is_number());
    assert!(
        array(puppet, "nodes")
            .iter()
            .filter(|n| n["kind"] == "physics")
            .all(|n| n["gravity"].is_number()),
        "a physics node carries no multiplier"
    );

    // "an animation carrying one track for each of two parameters"
    let animations = array(puppet, "animations");
    assert_eq!(animations.len(), 1);
    let tracks = animations[0]["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    let driven: BTreeSet<u64> = tracks
        .iter()
        .map(|t| t["parameter"].as_u64().unwrap())
        .collect();
    assert_eq!(driven.len(), 2, "both tracks drive one parameter");

    // "a channel mapping from one named channel to a parameter no physics node
    // drives, with a response curve"
    let mappings = array(puppet, "channel_mappings");
    assert_eq!(mappings.len(), 1);
    assert!(!mappings[0]["channel"].as_str().unwrap().is_empty());
    assert!(mappings[0].get("response_curve").is_some());
    let physics_outputs: BTreeSet<u64> = array(puppet, "nodes")
        .iter()
        .filter_map(|n| n.get("output").and_then(Value::as_u64))
        .collect();
    assert!(!physics_outputs.contains(&mappings[0]["parameter"].as_u64().unwrap()));

    // "import provenance on every part"
    for node in array(puppet, "nodes")
        .iter()
        .filter(|n| n["kind"] == "part")
    {
        let provenance = node
            .get("provenance")
            .unwrap_or_else(|| panic!("{} carries no provenance", node["name"]));
        assert!(provenance.get("source_id").is_some());
        assert!(provenance.get("content_hash").is_some());
    }

    // "extension data in a namespace no build understands, at two places in the
    // tree"
    let with_extensions: Vec<u64> = array(puppet, "nodes")
        .iter()
        .filter(|n| n.get("x").is_some())
        .map(|n| n["id"].as_u64().unwrap())
        .collect();
    assert_eq!(with_extensions, vec![N_BODY, N_FACE]);
    for id in with_extensions {
        for (_, extension) in node_by_id(puppet, id)["x"].as_object().unwrap() {
            assert!(extension.get("version").is_some());
            assert!(extension.get("name").is_some());
            assert!(extension.get("data").is_some());
        }
    }

    // The composition mode is stored from the first version (DET-6), and the
    // mirror axes are derived rather than stored (MAXIS-1).
    assert_eq!(puppet["composition_mode"], "sum");
    assert!(puppet.get("mirror_axes").is_none());
}

#[test]
fn one_part_is_the_smallest_valid_puppet() {
    let puppet = &parsed("one-part");
    assert_eq!(array(puppet, "nodes").len(), 2);
    assert_eq!(array(puppet, "textures").len(), 1);
    assert_eq!(
        node_by_id(puppet, puppet["root"].as_u64().unwrap())["kind"],
        "composite"
    );
    assert!(puppet.get("parameters").is_none());
    assert!(puppet.get("bindings").is_none());
}

#[test]
fn the_unmarked_material_is_byte_identical_across_the_version_gate() {
    // EXT-8 and VAL-20 are differential assertions: the same material, one file
    // declaring the current version and one a later one. If the two differ in
    // anything but the declaration, neither criterion tests what it says.
    for (current, later, needle) in [
        ("rigged:ext-unmarked", "rigged:future-minor", "\"wiggle\":"),
        (
            "rigged:polarity-value-unfamiliar",
            "rigged:future-minor-polarity-unfamiliar",
            "\"polarity\":\"radial\"",
        ),
        (
            "rigged:interpolation-value-unfamiliar",
            "rigged:future-minor-interpolation-unfamiliar",
            "\"interpolation\":\"ease_elastic\"",
        ),
        (
            "rigged:end-behaviour-value-unfamiliar",
            "rigged:future-minor-end-behaviour-unfamiliar",
            "\"end_behaviour\":\"ping_pong\"",
        ),
    ] {
        let pick = |name: &str| {
            let bytes = entry(&fixture(name).bytes, "puppet.json").unwrap();
            let text = String::from_utf8(bytes).unwrap();
            let at = text.find(needle).unwrap_or_else(|| {
                panic!("{name} does not carry {needle}");
            });
            text[at..at + needle.len() + 40].to_string()
        };
        assert_eq!(pick(current), pick(later), "{current} and {later} differ");
    }
}

#[test]
fn the_degenerate_physics_fixtures_cover_every_constant_with_a_range() {
    // VAL-25 is table-driven over the constants the format defines, and a
    // constant with no stated valid range fails the check. The format states a
    // range for five of them; this is the other half, that each has a fixture.
    for constant in [
        "length",
        "gravity",
        "frequency",
        "angle_damping",
        "length_damping",
    ] {
        let hyphenated = constant.replace('_', "-");
        assert!(
            corpus()
                .iter()
                .any(|f| f.name.starts_with(&format!("rigged:physics-{hyphenated}"))),
            "no degenerate fixture for `{constant}`"
        );
    }
}

#[test]
fn the_decompression_bomb_declares_a_small_size_and_expands_far_beyond_it() {
    let fixture = fixture("rigged:decompression-bomb");
    // Read the declared uncompressed size out of the central directory the same
    // way a loader would, without decompressing anything.
    let mut archive = zip::ZipArchive::new(Cursor::new(fixture.bytes.clone()))
        .expect("the bomb is a readable archive");
    let declared = archive
        .by_name(&format!("textures/{T_FACE}.png"))
        .expect("the bombed entry is there")
        .size();
    assert_eq!(declared, 1024, "the declaration is not small");
    assert!(
        fixture.bytes.len() < 1024 * 1024,
        "the fixture itself is too large to commit"
    );
}

#[test]
fn the_payloads_are_cropped_to_the_crop_rectangle() {
    // docs/format.md: one entry per distinct texture, cropped to its own
    // content. The record carries the authored dimensions; the payload does not.
    for (id, bytes) in puppet::texture_payloads() {
        assert_eq!(
            &bytes[..8],
            b"\x89PNG\r\n\x1a\n",
            "texture {id} is not a PNG"
        );
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        let expected = match id {
            T_FACE => (24, 20),
            T_HAIR => (48, 16),
            _ => (32, 24),
        };
        assert_eq!((width, height), expected, "texture {id} is not cropped");
    }
}

// ---------------- the animation shape: modes, endings, triggers and order

/// Every track of every animation in a fixture, with its parent animation.
fn tracks_of(puppet: &Value) -> Vec<(&Value, &Value)> {
    puppet
        .get("animations")
        .and_then(Value::as_array)
        .map(|animations| {
            animations
                .iter()
                .flat_map(|animation| {
                    animation["tracks"]
                        .as_array()
                        .expect("an animation has tracks")
                        .iter()
                        .map(move |track| (animation, track))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn rigged_carries_the_end_behaviour_and_no_other_field_of_the_animation_round() {
    // `rigged` is the control every variant below is read against. Only
    // `end_behaviour` is always written; the rest are absent in their default
    // form, and a `rigged` that quietly acquired one would make
    // INTERP-2, ORD-1 and MAP-1 test nothing.
    let puppet = &parsed("rigged");
    let animations = array(puppet, "animations");
    for animation in animations {
        assert_eq!(
            animation["end_behaviour"].as_str(),
            Some(END_LOOP),
            "an animation carries no end behaviour"
        );
        assert!(
            animation.get("triggers").is_none(),
            "rigged carries a trigger"
        );
    }
    for (_, track) in tracks_of(puppet) {
        for keyframe in track["keyframes"].as_array().unwrap() {
            assert!(
                keyframe.get("interpolation").is_none(),
                "rigged stores an interpolation mode, but every segment in it is linear"
            );
            assert!(keyframe.get("control_points").is_none());
        }
    }
    for parameter in array(puppet, "parameters") {
        assert!(parameter.get("order").is_none(), "rigged stores an order");
        assert!(
            parameter.get("mappable").is_none(),
            "rigged stores `mappable`, which is absent when true"
        );
    }
}

#[test]
fn the_mixed_interpolation_fixture_carries_one_segment_per_mode() {
    let puppet = &parsed("rigged:mixed-interpolation");
    let mut modes: Vec<Option<&str>> = Vec::new();
    let mut all_linear_track_seen = false;
    for (_, track) in tracks_of(puppet) {
        let keyframes = track["keyframes"].as_array().unwrap();
        // The last keyframe opens no segment, so it carries no mode.
        assert!(
            keyframes.last().unwrap().get("interpolation").is_none(),
            "a track's last keyframe carries a mode"
        );
        if track["id"].as_u64() == Some(TR_HEAD) {
            modes = keyframes
                .iter()
                .map(|k| k.get("interpolation").and_then(Value::as_str))
                .collect();
        } else if keyframes.iter().all(|k| k.get("interpolation").is_none()) {
            all_linear_track_seen = true;
        }
    }
    assert_eq!(
        modes,
        vec![
            Some(INTERP_STEPPED),
            Some(INTERP_INVERSE_STEPPED),
            None, // linear is the absent form
            Some(INTERP_BEZIER),
            Some(INTERP_BEZIER),
            None, // the last keyframe
        ]
    );
    assert!(
        all_linear_track_seen,
        "the fixture carries no all-linear track, so INTERP-2's other half is untested"
    );

    // The expectations ride as extension data, because outside `x` they would
    // be unmarked core data and invalid.
    let animation = array(puppet, "animations")
        .iter()
        .find(|a| a["id"].as_u64() == Some(A_IDLE))
        .unwrap();
    let expectations = &animation["x"]["com.rigel2d.fixture-expectations"];
    assert_eq!(expectations["version"], "1.0");
    assert_eq!(expectations["data"]["track"].as_u64(), Some(TR_HEAD));
    let samples = expectations["data"]["samples"].as_array().unwrap();
    assert_eq!(samples.len(), 6);
    // Every sample time lies inside the track's extent, or it is an expectation
    // about nothing.
    let keyframes: Vec<f64> = array(puppet, "animations")
        .iter()
        .flat_map(|a| a["tracks"].as_array().unwrap())
        .find(|t| t["id"].as_u64() == Some(TR_HEAD))
        .unwrap()["keyframes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["time"].as_f64().unwrap())
        .collect();
    let (first, last) = (keyframes[0], *keyframes.last().unwrap());
    for sample in samples {
        let time = sample["time"].as_f64().unwrap();
        assert!(
            (first..=last).contains(&time),
            "the expectation at {time} is outside the track"
        );
        assert!(sample["value"].as_f64().unwrap().is_finite());
    }
}

#[test]
fn every_bezier_segment_in_an_accepted_fixture_is_well_formed() {
    // docs/format.md: a Bézier segment's control times lie inside the segment
    // and every control value is finite. The defect fixtures break exactly one
    // of those; an *accepted* fixture that broke one would be rejected for a
    // reason its catalogue note does not name, and nothing else would notice.
    for fixture in corpus() {
        if fixture.expect != Expect::Accepted {
            continue;
        }
        let Some(puppet) = puppet_json(&fixture.bytes) else {
            continue;
        };
        for (_, track) in tracks_of(&puppet) {
            let keyframes = track["keyframes"].as_array().unwrap();
            for (index, keyframe) in keyframes.iter().enumerate() {
                if keyframe.get("interpolation").and_then(Value::as_str) != Some(INTERP_BEZIER) {
                    continue;
                }
                let start = keyframe["time"].as_f64().unwrap();
                let end = keyframes[index + 1]["time"].as_f64().unwrap();
                let controls = keyframe["control_points"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{} has a Bézier with no controls", fixture.name));
                assert_eq!(controls.len(), 2, "{} is not a cubic", fixture.name);
                for control in controls {
                    let time = control["time"].as_f64().unwrap();
                    assert!(
                        (start..=end).contains(&time),
                        "{}: a control time {time} is outside [{start}, {end}]",
                        fixture.name
                    );
                    for value in control["value"].as_array().unwrap() {
                        assert!(
                            value.as_f64().is_some_and(f64::is_finite),
                            "{}: a control value is not finite",
                            fixture.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_endings_fixture_carries_one_animation_per_end_behaviour() {
    let puppet = &parsed("rigged:endings");
    let behaviours: BTreeSet<&str> = array(puppet, "animations")
        .iter()
        .map(|a| a["end_behaviour"].as_str().unwrap())
        .collect();
    assert_eq!(
        behaviours,
        [END_HOLD, END_LOOP, END_RELEASE]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );
    // "each with a known last keyframe, duration and blend"
    for (_, track) in tracks_of(puppet) {
        assert!(track["blend_duration"].as_f64().is_some_and(f64::is_finite));
        assert!(track["keyframes"].as_array().unwrap().len() >= 2);
    }
    // The releasing animation blends out at two different rates, which is the
    // half of END-3 that only a fixture can carry.
    let releasing = array(puppet, "animations")
        .iter()
        .find(|a| a["end_behaviour"] == END_RELEASE)
        .unwrap();
    let rates: BTreeSet<String> = releasing["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["blend_duration"].to_string())
        .collect();
    assert_eq!(
        rates.len(),
        2,
        "the releasing tracks share a blend duration"
    );
}

#[test]
fn a_trigger_carries_an_event_name_and_nothing_else() {
    // docs/format.md: no identifier, no target, no command. The rejected
    // fixture is the one place a trigger carries more, and it is named for it.
    //
    // `x` is the one further key a trigger may carry, because `x` is permitted
    // on any object and the material under it is inert by construction. It is
    // also the one key that must not be reached for to widen this rule: a
    // trigger growing a condition or a target is what the assertion exists to
    // catch, so `x` is admitted only on the fixture named for carrying it.
    for fixture in corpus() {
        let Some(puppet) = puppet_json(&fixture.bytes) else {
            continue;
        };
        let Some(animations) = puppet.get("animations").and_then(Value::as_array) else {
            continue;
        };
        for animation in animations {
            let Some(triggers) = animation.get("triggers").and_then(Value::as_array) else {
                continue;
            };
            for trigger in triggers {
                let keys: Vec<&str> = trigger
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect();
                if fixture.name == "rigged:trigger-extra-field" {
                    assert!(keys.len() > 1, "the extra-field fixture carries no extra");
                    continue;
                }
                let marked = fixture.name == "rigged:ext-on-trigger" && keys == ["event", "x"];
                assert!(
                    keys == ["event"] || marked,
                    "{} carries a trigger with more than an event name: {keys:?}",
                    fixture.name
                );
                assert!(trigger["event"].is_string());
            }
        }
    }
}

#[test]
fn the_triggered_fixture_carries_a_raised_a_lost_and_an_unraised_event() {
    let puppet = &parsed("rigged:triggered");
    let events: Vec<&str> = array(puppet, "animations")
        .iter()
        .filter_map(|a| a.get("triggers"))
        .flat_map(|t| t.as_array().unwrap())
        .map(|t| t["event"].as_str().unwrap())
        .collect();
    assert_eq!(
        events,
        vec![EVENT_ON_LOAD, "plugin_ribbon_snapped", EVENT_SOURCE_LOST]
    );
    // Canonically, triggers are ordered by event name.
    let mut sorted = events.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, events, "the triggers are not in canonical order");
    // And `rigged:triggers-unsorted` is the fixture that is deliberately not.
    let unsorted = &parsed("rigged:triggers-unsorted");
    let events: Vec<&str> = array(unsorted, "animations")
        .iter()
        .filter_map(|a| a.get("triggers"))
        .flat_map(|t| t.as_array().unwrap())
        .map(|t| t["event"].as_str().unwrap())
        .collect();
    let mut sorted = events.clone();
    sorted.sort_unstable();
    assert_ne!(sorted, events, "the unsorted fixture is in canonical order");
}

#[test]
fn the_ordered_parameters_fixture_presents_unlike_identifier_order() {
    let puppet = &parsed("rigged:ordered-parameters");
    let parameters = array(puppet, "parameters");
    // The collection itself stays in identifier order — the arrangement is a
    // field, never the order of the collection (I3).
    let ids: Vec<u64> = parameters
        .iter()
        .map(|p| p["id"].as_u64().unwrap())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "`parameters` is not in identifier order");

    // The stored order differs from it, or the fixture tests nothing. Absent
    // sorts last, ties break by identifier.
    let mut presented: Vec<(Option<i64>, u64)> = parameters
        .iter()
        .map(|p| {
            (
                p.get("order").and_then(Value::as_i64),
                p["id"].as_u64().unwrap(),
            )
        })
        .collect();
    presented.sort_by_key(|(order, id)| (order.is_none(), *order, *id));
    let presented_ids: Vec<u64> = presented.into_iter().map(|(_, id)| id).collect();
    assert_ne!(
        presented_ids, ids,
        "the rigger's order is identifier order, so ORD-1 asserts nothing"
    );
    assert!(
        parameters.iter().any(|p| p.get("order").is_none()),
        "no parameter carries no order, so the absent case is untested"
    );

    // One parameter not mappable and fed by a channel mapping (MAP-2, MAP-3),
    // and one not mappable and fed by nothing (MAP-1).
    let mapped: BTreeSet<u64> = array(puppet, "channel_mappings")
        .iter()
        .map(|m| m["parameter"].as_u64().unwrap())
        .collect();
    let hidden: Vec<u64> = parameters
        .iter()
        .filter(|p| p.get("mappable") == Some(&Value::Bool(false)))
        .map(|p| p["id"].as_u64().unwrap())
        .collect();
    assert!(hidden.iter().any(|id| mapped.contains(id)));
    assert!(hidden.iter().any(|id| !mapped.contains(id)));

    // And the mapped one is animated too, which is MAP-4's "both mapped and
    // animated keeps the flag it was given".
    let animated: BTreeSet<u64> = tracks_of(puppet)
        .iter()
        .map(|(_, t)| t["parameter"].as_u64().unwrap())
        .collect();
    assert!(
        hidden
            .iter()
            .any(|id| mapped.contains(id) && animated.contains(id))
    );
}

#[test]
fn the_trigger_limit_fixture_is_one_over_the_declared_figure() {
    let puppet = &parsed("rigged:triggers-over-limit");
    let longest = array(puppet, "animations")
        .iter()
        .filter_map(|a| a.get("triggers").and_then(Value::as_array))
        .map(Vec::len)
        .max()
        .expect("the fixture carries triggers");
    assert_eq!(
        longest,
        PROVISIONAL_TRIGGERS_PER_ANIMATION + 1,
        "the just-over fixture is not just over"
    );
    // Distinct names, or it would be refused as a duplicate event instead.
    let events: BTreeSet<&str> = array(puppet, "animations")
        .iter()
        .filter_map(|a| a.get("triggers").and_then(Value::as_array))
        .flatten()
        .map(|t| t["event"].as_str().unwrap())
        .collect();
    assert_eq!(
        events.len(),
        longest,
        "the fixture also duplicates an event"
    );
}

/// The coordinate-list fixture is one over its figure, and clear of every
/// neighbouring limit — the per-track and aggregate keyframe counts above all,
/// since it is a keyframe it lengthens.
#[test]
fn the_coordinate_limit_fixture_is_one_over_the_declared_figure() {
    let puppet = &parsed("rigged:coordinates-over-limit");
    let lists: Vec<usize> = array(puppet, "animations")
        .iter()
        .filter_map(|a| a.get("tracks").and_then(Value::as_array))
        .flatten()
        .filter_map(|t| t.get("keyframes").and_then(Value::as_array))
        .flatten()
        .filter_map(|k| k.get("value").and_then(Value::as_array))
        .map(Vec::len)
        .collect();
    assert_eq!(
        lists.iter().copied().max(),
        Some(PROVISIONAL_COORDINATE_COUNT + 1),
        "the just-over fixture is not just over"
    );
    assert_eq!(
        lists.iter().filter(|held| **held > 2).count(),
        1,
        "exactly one coordinate list is over, so the fixture names one defect"
    );
    // A keyframe is what it lengthens, so both keyframe counts have to stay
    // far inside their own limits or the fixture is refused for the wrong
    // rule — the mistake that cost `rigged:keyframes-over-aggregate` two
    // rounds.
    let per_track: Vec<usize> = array(puppet, "animations")
        .iter()
        .filter_map(|a| a.get("tracks").and_then(Value::as_array))
        .flatten()
        .filter_map(|t| t.get("keyframes").and_then(Value::as_array))
        .map(Vec::len)
        .collect();
    let longest = per_track.iter().copied().max().unwrap_or_default();
    let total: usize = per_track.iter().sum();
    assert!(
        longest < KEYFRAMES_PER_TRACK,
        "a track carries {longest} keyframes and the fixture would be refused per track"
    );
    assert!(
        total < AGGREGATE_KEYFRAME_LIMIT,
        "the fixture carries {total} keyframes and would be refused for the aggregate"
    );
}

/// A keyframe carrying control points on a segment that is **not** Bézier is
/// still refused for a non-finite control value.
///
/// The fixture exists because the check was gated on the mode while the reader
/// reads `control_points` whatever the mode says, so the number reached a
/// writer with no way to spell it. This holds the fixture to the shape that
/// makes it that defect and no other.
///
/// Asserted over the **text**, because the fixture's whole point is a number
/// `serde_json` refuses to parse — the same reason
/// `rigged:positive-infinity` carries one.
#[test]
fn the_non_bezier_control_value_fixture_declares_no_mode() {
    let bytes = entry(
        &fixture("rigged:control-value-nonfinite-without-bezier").bytes,
        "puppet.json",
    )
    .expect("the fixture carries a puppet.json");
    let text = String::from_utf8(bytes).expect("UTF-8");
    assert_eq!(
        text.matches("control_points").count(),
        1,
        "one keyframe carries control points, so the fixture names one defect"
    );
    assert!(
        text.contains("1e400"),
        "the overflowing control value is gone, so the fixture tests nothing"
    );
    assert!(
        !text.contains("interpolation"),
        "a mode is declared somewhere, so the defect is not the one the name says"
    );
}

#[test]
fn the_aggregate_keyframe_fixture_is_one_over_and_clear_of_every_neighbouring_limit() {
    // The fixture that took three attempts. Two earlier figures for this limit
    // could never fire, because a file just over either was refused by the
    // payload limit or by the parse-tree limit first — and a fixture refused
    // for a rule its catalogue note does not name tests the wrong thing while
    // looking green. So this asserts both halves: that it is exactly one over
    // the aggregate, and that nothing else is anywhere near firing.
    let fixture = fixture("rigged:keyframes-over-aggregate");
    let text = entry(&fixture.bytes, "puppet.json").expect("the fixture carries a puppet.json");
    let puppet: Value = serde_json::from_slice(&text).expect("it parses");

    let per_track: Vec<usize> = array(&puppet, "animations")
        .iter()
        .flat_map(|a| a["tracks"].as_array().unwrap())
        .map(|t| t["keyframes"].as_array().unwrap().len())
        .collect();

    assert_eq!(
        per_track.iter().sum::<usize>(),
        AGGREGATE_KEYFRAME_LIMIT + 1,
        "the just-over fixture is not just over"
    );

    // Every neighbouring count, with room to spare. `docs/format.md` *Load
    // limits* carries the ceilings these are read against.
    assert!(
        *per_track.iter().max().unwrap() <= KEYFRAMES_PER_TRACK / 2,
        "a track is close enough to the per-track limit that an inclusive \
         reading of it would refuse this fixture first"
    );
    assert!(
        array(&puppet, "animations").len() * 8 <= 128,
        "too many animations"
    );
    assert!(
        array(&puppet, "parameters").len() <= 600,
        "too many parameters"
    );
    for animation in array(&puppet, "animations") {
        assert!(animation["tracks"].as_array().unwrap().len() <= 600);
        // An animation carries at most one track per parameter.
        let driven: BTreeSet<u64> = animation["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["parameter"].as_u64().unwrap())
            .collect();
        assert_eq!(
            driven.len(),
            animation["tracks"].as_array().unwrap().len(),
            "an animation carries two tracks for one parameter"
        );
    }

    // The payload text, against `DecompressedPayloadBytes` at 64 MiB. The
    // parse tree is what actually binds and cannot be measured from here
    // without the reader — `docs/format.md` carries that arithmetic — but the
    // text is the half this crate can hold itself to.
    assert!(
        text.len() * 4 < 64 * 1024 * 1024,
        "the payload is within a factor of four of its limit, which is less \
         margin than this fixture was built with"
    );

    // And it stays committable.
    assert!(
        fixture.bytes.len() < 2 * 1024 * 1024,
        "the committed fixture has grown past two megabytes"
    );
}

/// Each fixture hanging marked extension data on one entity of the animation
/// shape is its base fixture plus exactly that, and nothing else.
///
/// The assertion is differential: reach the entity the fixture is named for,
/// take the `x` block off it, require that block to hold the one namespace and
/// to be marked — a namespace key, a declared version and a human-readable
/// name — and then require what is left to equal the base fixture's document.
/// Without the differential half, a variant that quietly changed a second
/// thing would exercise a shape nobody described, and the acceptance test
/// driven off the catalogue would still pass.
#[test]
fn the_marked_material_fixtures_are_their_base_plus_one_namespace() {
    fn by_id(items: &mut Value, id: u64) -> &mut Value {
        items
            .as_array_mut()
            .expect("a collection")
            .iter_mut()
            .find(|item| item["id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("no entity {id}"))
    }

    type Reach = fn(&mut Value) -> &mut Value;
    let on_trigger: Reach = |doc| &mut by_id(&mut doc["animations"], A_IDLE)["triggers"][1];
    let on_keyframe: Reach = |doc| {
        &mut by_id(
            &mut by_id(&mut doc["animations"], A_IDLE)["tracks"],
            TR_HEAD,
        )["keyframes"][0]
    };
    let on_control_point: Reach = |doc| {
        &mut by_id(
            &mut by_id(&mut doc["animations"], A_IDLE)["tracks"],
            TR_HEAD,
        )["keyframes"][3]["control_points"][0]
    };
    let on_parameter: Reach = |doc| by_id(&mut doc["parameters"], P_HEAD_TURN);

    for (name, base, reach, what) in [
        (
            "rigged:ext-on-trigger",
            "rigged:triggered",
            on_trigger,
            "a trigger",
        ),
        (
            "rigged:ext-on-keyframe",
            "rigged:mixed-interpolation",
            on_keyframe,
            "a keyframe",
        ),
        (
            "rigged:ext-on-control-point",
            "rigged:mixed-interpolation",
            on_control_point,
            "a control point",
        ),
        (
            "rigged:ext-on-parameter",
            "rigged:ordered-parameters",
            on_parameter,
            "a parameter",
        ),
    ] {
        let mut doc = parsed(name);
        let entity = reach(&mut doc);
        let block = entity
            .as_object_mut()
            .expect("an object")
            .remove("x")
            .unwrap_or_else(|| panic!("{name} carries no marked material on {what}"));
        let block = block.as_object().expect("`x` is an object");
        assert_eq!(
            block.keys().map(String::as_str).collect::<Vec<&str>>(),
            vec!["com.alt.notes"],
            "{name} carries more than the one namespace on {what}"
        );
        let entry = &block["com.alt.notes"];
        for part in ["version", "name", "data"] {
            assert!(
                entry.get(part).is_some(),
                "{name}: the material on {what} declares no {part}, so it is not marked"
            );
        }
        assert_eq!(
            doc,
            parsed(base),
            "{name} differs from {base} in more than the marked material on {what}"
        );
    }
}

/// The entity each of those fixtures is named for is the one a reader reaches
/// by position, or by a field this build also reads — which is what makes them
/// worth having beyond the animation `rigged:mixed-interpolation` already
/// carries material on.
#[test]
fn the_marked_material_sits_beside_what_the_build_reads() {
    let keyframe = &parsed("rigged:ext-on-keyframe")["animations"][0]["tracks"];
    let keyframe = &keyframe
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"].as_u64() == Some(TR_HEAD))
        .expect("the head track")["keyframes"][0];
    assert!(
        keyframe.get("interpolation").is_some(),
        "the annotated keyframe opens no segment, so the material sits beside no core field"
    );

    let parameter = parsed("rigged:ext-on-parameter")["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_u64() == Some(P_HEAD_TURN))
        .expect("the head-turn parameter")
        .clone();
    assert!(
        parameter.get("order").is_some(),
        "the annotated parameter carries no order, so a reordering would disturb nothing"
    );
}
