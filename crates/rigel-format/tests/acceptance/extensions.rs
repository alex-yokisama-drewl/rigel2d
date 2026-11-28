//! Section F — unrecognised data is marked as such by whoever wrote it.
//!
//! Asserts [I1](../../../../docs/invariants.md) — nothing a puppet supplies is
//! executed — and [I6](../../../../docs/invariants.md) — a plugin never arrives
//! inside a puppet.

use rigel_format::{Rule, load, save};

use crate::support::{
    all_strings, assert_rejected_naming, bytes, dir_entries, fixture, puppet_json, scratch,
};

/// EXT-1 — marked extension data declares an identifier, the version it expects
/// and a human-readable name; the validator accepts it and the loader preserves
/// it.
#[test]
fn ext_1_marked_extension_data_is_accepted_and_preserved() {
    let loaded = load(&fixture("rigged")).expect("the base fixture loads");
    let entries: Vec<_> = loaded
        .puppet
        .nodes
        .iter()
        .flat_map(|n| n.extensions.entries.iter())
        .collect();
    assert!(
        !entries.is_empty(),
        "the base fixture carries extension data in a namespace no build understands"
    );
    for entry in &entries {
        assert!(!entry.namespace.is_empty(), "an extension identifier");
        assert!(!entry.version.is_empty(), "the version it expects");
        assert!(!entry.name.is_empty(), "a human-readable name");
        assert!(
            !entry.data.is_empty(),
            "the data itself, kept as it arrived"
        );
    }

    let dir = scratch("ext-1");
    let written = dir.join("preserved.rigel2d");
    save(&loaded.puppet, &written).expect("a puppet carrying extension data saves");
    let reloaded = load(&written).expect("it loads again").puppet;
    assert_eq!(
        loaded.puppet, reloaded,
        "the loader preserves what it does not understand"
    );
}

/// EXT-2 — the same values with the marker removed, at the current version, are
/// **rejected as malformed** rather than preserved. Without this a typo and an
/// extension are the same thing.
#[test]
fn ext_2_unmarked_extension_values_are_rejected() {
    assert_rejected_naming("rigged:ext-unmarked", Rule::UnmarkedData);
}

/// EXT-3 — extension data with no declared version, and with no human-readable
/// name, are each rejected.
#[test]
fn ext_3_extension_data_must_declare_a_version_and_a_name() {
    assert_rejected_naming("rigged:ext-no-version", Rule::ExtensionDeclaresVersion);
    assert_rejected_naming("rigged:ext-no-label", Rule::ExtensionDeclaresName);
}

/// EXT-4 — contents verbatim, position canonical. The two halves that look like
/// a contradiction and are not.
///
/// The same extension data at a different position in the input saves to the
/// bytes `rigged` saves to, because position is set by the canonical ordering,
/// keyed on the marker — which a build can read without understanding the
/// contents.
#[test]
fn ext_4_contents_are_verbatim_and_position_is_canonical() {
    let dir = scratch("ext-4");
    let from_moved = dir.join("from-moved.rigel2d");
    let from_base = dir.join("from-base.rigel2d");

    let moved = load(&fixture("rigged:ext-moved")).expect("it loads").puppet;
    let base = load(&fixture("rigged")).expect("it loads").puppet;
    save(&moved, &from_moved).expect("it saves");
    save(&base, &from_base).expect("it saves");

    assert_ne!(
        bytes(&fixture("rigged:ext-moved")),
        bytes(&fixture("rigged")),
        "the two fixtures differ in the input, or this asserts nothing"
    );
    assert_eq!(
        bytes(&from_moved),
        bytes(&from_base),
        "position is set by the canonical ordering, not by where it arrived"
    );

    let arrived = puppet_json(&fixture("rigged:ext-moved"));
    let written = puppet_json(&from_moved);
    for id in [11u64, 12] {
        let arrived_x = crate::support::json_node(&arrived, id)["x"].clone();
        let written_x = crate::support::json_node(&written, id)["x"].clone();
        assert_eq!(
            arrived_x, written_x,
            "the contents on node {id} are byte for byte what arrived"
        );
    }
}

/// EXT-5 — hostile-shaped strings are preserved verbatim, and the load resolves,
/// interpolates and executes none of them.
///
/// The half asserted here is preservation and the filesystem: the load is given
/// a directory containing only the puppet, and that directory is unchanged and
/// gains nothing. **The process-spawn, dynamic-library-load and outbound-
/// connection half needs an interception harness that does not exist** — see
/// `EXEC-1` and the gap recorded in `docs/wip/s2-format.md`.
#[test]
fn ext_5_hostile_shaped_strings_are_preserved_and_never_resolved() {
    let dir = scratch("ext-5");
    let path = crate::support::copy_fixture_into("rigged:ext-hostile", &dir);
    let before = dir_entries(&dir);

    let arrived = puppet_json(&path);
    let loaded = load(&path).expect("hostile-shaped strings are data, and the file is valid");

    assert_eq!(
        before,
        dir_entries(&dir),
        "a load creates and resolves nothing in the puppet's own container"
    );

    let out = scratch("ext-5-out").join("saved.rigel2d");
    save(&loaded.puppet, &out).expect("it saves");
    assert_eq!(
        all_strings(&arrived),
        all_strings(&puppet_json(&out)),
        "every string value is preserved verbatim through load and save"
    );
}

/// EXT-6 — a puppet declaring it carries data for a named extension at a named
/// version installs, registers and loads **nothing**, and the declaration is
/// reported as missing. [I6](../../../../docs/invariants.md).
#[test]
fn ext_6_a_declared_extension_is_reported_and_never_installed() {
    let dir = scratch("ext-6");
    let path = crate::support::copy_fixture_into("rigged:ext-declares-plugin", &dir);
    let before = dir_entries(&dir);

    let loaded = load(&path).expect("a declaration is inert data and the file is valid");

    assert_eq!(
        before,
        dir_entries(&dir),
        "declaring is not installing — nothing arrived on disk"
    );
    let declared = loaded
        .report
        .unrecognised_extensions
        .iter()
        .find(|n| n.namespace == "com.example.physics-plus")
        .expect("the declaration is reported to the user as missing");
    assert!(
        !declared.name.is_empty() && !declared.version.is_empty(),
        "named, at a named version, so the user can be told what is missing"
    );
}

/// EXT-7 — a later-version file's unmarked material is accepted and preserved
/// **opaquely**, exactly as marked extension data is, and none of it is
/// interpreted.
///
/// *Not interpreted* is asserted differentially: removing it from the fixture —
/// which is what `rigged` is — changes no reported entity. Byte-stability across
/// a round trip is `VER-6`.
#[test]
fn ext_7_a_later_versions_unmarked_material_is_preserved_and_never_interpreted() {
    let base = load(&fixture("rigged")).expect("it loads").puppet;
    let later = load(&fixture("rigged:future-minor"))
        .expect("it loads")
        .puppet;

    assert!(
        puppet_json(&fixture("rigged:future-minor"))["nodes"]
            .as_array()
            .expect("a nodes array")
            .iter()
            .any(|n| n["kind"].as_str() == Some("warp_field")),
        "the fixture carries a node kind no build knows, or this asserts nothing"
    );

    assert_eq!(
        base.nodes.len(),
        later.nodes.len(),
        "an unknown node kind is opaque material, never a reported entity"
    );
    assert_eq!(
        base.parameters, later.parameters,
        "nothing unfamiliar reached a parameter"
    );
    assert_eq!(base.bindings, later.bindings);
    assert_eq!(base.textures, later.textures);
    assert_eq!(
        base.animations, later.animations,
        "nothing unfamiliar reached draw order or evaluation"
    );
}

/// EXT-8 — the gate is the **declared version**, not recognisability. One
/// differential assertion over byte-for-byte the same unmarked material.
#[test]
fn ext_8_the_version_declaration_is_what_licenses_unmarked_material() {
    let unmarked = puppet_json(&fixture("rigged:ext-unmarked"));
    let later = puppet_json(&fixture("rigged:future-minor"));
    let from_unmarked = crate::support::json_node_carrying(&unmarked, "wiggle")["wiggle"].clone();
    let from_later = crate::support::json_node_carrying(&later, "wiggle")["wiggle"].clone();
    assert!(
        !from_unmarked.is_null(),
        "the fixtures carry the material this criterion is about"
    );
    assert_eq!(
        from_unmarked, from_later,
        "the same material, so the only difference is the declared version"
    );

    assert!(
        load(&fixture("rigged:ext-unmarked")).is_err(),
        "at the current version, unmarked material is a defect"
    );
    assert!(
        load(&fixture("rigged:future-minor")).is_ok(),
        "at a later version, the same material is licensed"
    );
}

/// EXT-9 — the finite-number rule is **not** gated by the declared version, in
/// two halves.
///
/// The second half is the one an implementer gets wrong, by reading *later
/// version* as *accept the file*: it is the difference between rejecting a NaN
/// and letting one reach the frame loop.
#[test]
fn ext_9_finiteness_is_not_gated_by_the_declared_version() {
    let opaque = load(&fixture("rigged:future-minor-nonfinite"))
        .expect("bytes inside opaquely preserved material are never examined");

    let dir = scratch("ext-9");
    let first = dir.join("first.rigel2d");
    let second = dir.join("second.rigel2d");
    save(&opaque.puppet, &first).expect("it saves");
    save(&load(&first).expect("it loads").puppet, &second).expect("it saves again");
    assert_eq!(
        bytes(&first),
        bytes(&second),
        "opaquely preserved material survives byte-stably"
    );

    assert_rejected_naming("rigged:future-minor-nan-interpreted", Rule::NumbersFinite);
}
