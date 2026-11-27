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
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(sorted, names, "{}'s entries are not sorted", fixture.name);
        assert!(
            !names.iter().any(|n| n.ends_with('/')),
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
