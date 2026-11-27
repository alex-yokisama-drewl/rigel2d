//! Sections D and E — version, compatibility, and what the user is told.
//!
//! `TOOL-2` and `TOOL-3` are this subsystem's **human gate** and are not here:
//! a person inspects `rigged` with `unzip`, a text editor and an image viewer,
//! changes one parameter's name and one keypoint value, repacks it, and extracts
//! one texture payload on its own. `MSG-2` is the other gate — a person reads the
//! message for `rigged:future-minor` and `rigged:ext-unknown`. Both are set out
//! in `docs/wip/s2-format.md`, task 9.

use rigel_format::{FormatVersion, Rejection, SaveStep, load, save, save_with_injected_failure};

use crate::support::{
    bytes, catalogue, fixture, fixture_copy, modified, puppet_json, scratch, version_json,
};

/// The corpus and the build must agree about what *the current version* is.
///
/// Not a criterion of its own: it is the premise every version criterion rests
/// on. A corpus authored at one version against a build writing another makes
/// `VER-1` … `VER-6`, `EXT-7`, `EXT-8` and `VAL-20` silently vacuous, because the
/// gate those criteria test is the declared version.
#[test]
fn the_corpus_and_the_build_agree_on_the_current_version() {
    let (major, minor) = catalogue().version;
    assert_eq!(
        FormatVersion { major, minor },
        FormatVersion::CURRENT,
        "catalogue.json and FormatVersion::CURRENT must name one version"
    );
    assert_eq!(
        version_json(&fixture("rigged"))["version"]["major"].as_u64(),
        Some(u64::from(FormatVersion::CURRENT.major))
    );
    assert_eq!(
        version_json(&fixture("rigged"))["version"]["minor"].as_u64(),
        Some(u64::from(FormatVersion::CURRENT.minor))
    );
}

/// VER-1 — a later-version file is edited and saved with the unfamiliar material
/// coming back as it arrived, unmarked as much as marked, and stays editable.
///
/// *Where* that material sits is `EXT-4`'s question and a different one. Whole-
/// file byte stability across a save is `VER-6`.
#[test]
fn ver_1_unfamiliar_material_survives_an_edit_to_a_value_we_understand() {
    let source = fixture("rigged:future-minor");
    let arrived = puppet_json(&source);
    let mut subject = load(&source).expect("a later minor opens").puppet;

    // A value this build understands, changed to something it is not.
    let edited_node = subject
        .nodes
        .iter()
        .position(|n| n.transform.translation != [7.0, 11.0])
        .expect("a node to edit");
    let edited_id = subject.nodes[edited_node].id;
    subject.nodes[edited_node].transform.translation = [7.0, 11.0];

    let dir = scratch("ver-1");
    let written = dir.join("edited.rigel2d");
    save(&subject, &written).expect("a later-version file stays editable");
    let saved = puppet_json(&written);

    // The edit is present.
    let reloaded = load(&written).expect("the saved file still loads").puppet;
    assert_eq!(
        reloaded
            .nodes
            .iter()
            .find(|n| n.id == edited_id)
            .expect("the edited node")
            .transform
            .translation,
        [7.0, 11.0],
        "the edit this build understands is present"
    );

    // The unfamiliar material — the unmarked core field, the unknown node kind
    // and the unfamiliar enumerated value — comes back as it arrived.
    let unfamiliar = |doc: &serde_json::Value| -> Vec<serde_json::Value> {
        let nodes = doc["nodes"].as_array().expect("a nodes array");
        let mut out: Vec<serde_json::Value> = nodes
            .iter()
            .filter(|n| {
                n.get("wiggle").is_some()
                    || n["kind"].as_str() == Some("warp_field")
                    || n.get("output_source").is_some()
            })
            .cloned()
            .collect();
        out.sort_by_key(|n| n["id"].as_u64().unwrap_or_default());
        out
    };
    let arrived_unfamiliar = unfamiliar(&arrived);
    assert!(
        !arrived_unfamiliar.is_empty(),
        "the fixture carries unfamiliar material, or this asserts nothing"
    );
    assert_eq!(
        arrived_unfamiliar,
        unfamiliar(&saved),
        "the contents of the unfamiliar material are what arrived"
    );
}

/// VER-2 — a declared break in compatibility is refused, naming the version, and
/// the file is untouched.
///
/// The other half — *the puppet already loaded still renders unchanged*,
/// [I4](../../../../docs/invariants.md) — needs a render surface and is S5's. It
/// holds structurally here because a load produces a value rather than mutating
/// one: there is no partial-replacement path to assert against.
#[test]
fn ver_2_a_breaking_version_is_refused_naming_the_version() {
    let path = fixture_copy("rigged:breaking-version");
    let before = bytes(&path);
    let before_modified = modified(&path);

    let rejection = load(&path).expect_err("a differing major is refused");
    match rejection {
        Rejection::IncompatibleVersion { found, known } => {
            assert_ne!(found.major, known.major, "the break is in the major number");
            assert_eq!(known, FormatVersion::CURRENT);
            assert!(
                rejection_text(&load(&path).expect_err("still refused"))
                    .contains(&found.major.to_string()),
                "the message names the version"
            );
        }
        other => panic!("refused for the wrong reason: {other:?}"),
    }

    assert_eq!(before, bytes(&path), "a refused load never writes");
    assert_eq!(before_modified, modified(&path));
}

fn rejection_text(rejection: &Rejection) -> String {
    rejection.to_string()
}

/// VER-3 — opening never writes. Every fixture in the catalogue, accepted and
/// rejected alike.
#[test]
fn ver_3_opening_never_writes() {
    for entry in &catalogue().fixtures {
        let path = fixture_copy(&entry.name);
        let before = bytes(&path);
        let before_modified = modified(&path);
        let _ = load(&path);
        assert_eq!(before, bytes(&path), "{} changed on open", entry.name);
        assert_eq!(
            before_modified,
            modified(&path),
            "{}'s modification time changed on open",
            entry.name
        );
    }
}

/// VER-4 — an upgrading save keeps the previous bytes as a sibling, and the
/// saved file declares the current version.
///
/// Byte-identity with the pre-upgrade file necessarily breaks here, and that is
/// not a canonical-serialisation failure: `CANON-1` is scoped to one build.
#[test]
fn ver_4_an_upgrading_save_keeps_the_previous_bytes() {
    let path = fixture_copy("rigged:old-version");
    let arrived = bytes(&path);
    let arrived_version = version_json(&path);
    assert_eq!(
        arrived_version["version"]["major"].as_u64(),
        Some(u64::from(FormatVersion::CURRENT.major)),
        "the fixture is an earlier minor, not a different major"
    );

    let subject = load(&path).expect("an earlier minor opens").puppet;
    let outcome = save(&subject, &path).expect("the upgrading save succeeds");

    let backup = outcome
        .backup
        .as_ref()
        .expect("an upgrading save names the backup it wrote");
    let expected_name = format!(
        "{}.bak-{}.{}",
        path.file_name().expect("a file name").to_string_lossy(),
        arrived_version["version"]["major"].as_u64().unwrap_or(0),
        arrived_version["version"]["minor"].as_u64().unwrap_or(0),
    );
    assert_eq!(
        backup.file_name().expect("a file name").to_string_lossy(),
        expected_name,
        "the suffix is the version the backup holds, not the version written"
    );
    assert_eq!(
        backup.parent(),
        path.parent(),
        "the backup is a sibling, because the person has to be able to find it"
    );
    assert_eq!(
        arrived,
        bytes(backup),
        "the backup holds the pre-upgrade bytes exactly"
    );
    assert_eq!(
        version_json(&path)["version"]["minor"].as_u64(),
        Some(u64::from(FormatVersion::CURRENT.minor)),
        "the saved file declares the current version"
    );
}

/// VER-5 — a failure injected at each step of the upgrading save leaves the
/// original intact. [I7](../../../../docs/invariants.md).
#[test]
fn ver_5_a_failed_upgrading_save_damages_nothing() {
    for step in SaveStep::ALL {
        let path = fixture_copy("rigged:old-version");
        let arrived = bytes(&path);
        let subject = load(&path).expect("an earlier minor opens").puppet;

        let outcome = save_with_injected_failure(&subject, &path, *step);
        assert!(
            outcome.is_err(),
            "a failure injected at {step:?} must fail the save"
        );
        assert_eq!(
            arrived,
            bytes(&path),
            "the original is intact after a failure at {step:?}"
        );
        assert!(
            load(&path).is_ok(),
            "the original still loads after a failure at {step:?}"
        );
    }
}

/// VER-6 — carrying someone else's data does not make the file churn.
#[test]
fn ver_6_a_later_version_file_round_trips_byte_stably() {
    let dir = scratch("ver-6");
    let first = dir.join("first.rigel2d");
    let second = dir.join("second.rigel2d");

    let subject = load(&fixture("rigged:future-minor"))
        .expect("a later minor opens")
        .puppet;
    save(&subject, &first).expect("the first save succeeds");
    let reloaded = load(&first).expect("the saved file loads").puppet;
    save(&reloaded, &second).expect("the second save succeeds");

    assert_eq!(
        bytes(&first),
        bytes(&second),
        "two serialisations of a file carrying unfamiliar material are identical"
    );
}

/// TOOL-4 — a later-version file loads, its unmarked core field, unknown node
/// kind and unfamiliar enumerated value notwithstanding, and every entity this
/// build **does** know is present and correct.
///
/// Asserted differentially against `rigged`, which is the same puppet without
/// the later-version material. The physics node carrying the unfamiliar
/// enumerated value is excluded: the specification does not say what an
/// unfamiliar value in a known enumerated field becomes in memory — reported as a
/// gap in `docs/wip/s2-format.md`.
#[test]
fn tool_4_a_later_version_file_loads_with_every_known_entity_intact() {
    let known = load(&fixture("rigged"))
        .expect("the base fixture loads")
        .puppet;
    let later = load(&fixture("rigged:future-minor"))
        .expect("a later minor opens")
        .puppet;

    assert_eq!(
        known.parameters, later.parameters,
        "parameters this build knows are present and correct"
    );
    assert_eq!(known.bindings, later.bindings, "bindings");
    assert_eq!(known.animations, later.animations, "animations");
    assert_eq!(
        known.channel_mappings, later.channel_mappings,
        "channel mappings"
    );
    assert_eq!(known.textures, later.textures, "textures");
    assert_eq!(known.root, later.root, "the root");
    assert_eq!(known.gravity, later.gravity, "the puppet's gravity");

    let known_ids: Vec<_> = known.nodes.iter().map(|n| n.id).collect();
    let later_ids: Vec<_> = later.nodes.iter().map(|n| n.id).collect();
    assert_eq!(
        known_ids, later_ids,
        "a node of a kind this build does not know is preserved opaquely, not \
         reported as an entity (EXT-7)"
    );
}

/// MSG-1 — the reported load outcome names the version gap, names each
/// unrecognised extension by its **human-readable name** and declared version,
/// and warns that the features behind them may not work in this build.
///
/// Asserted on the load outcome the runtime reports, not on any interface:
/// neither application's interface is designed, and
/// [I2](../../../../docs/invariants.md) binds where it may surface in Live.
#[test]
fn msg_1_the_load_outcome_names_the_version_gap_and_each_extension() {
    let later = load(&fixture("rigged:future-minor")).expect("a later minor opens");
    let gap = later
        .report
        .version_gap
        .expect("a later minor is reported as a version gap");
    assert_eq!(gap.known, FormatVersion::CURRENT);
    assert!(
        gap.found > gap.known,
        "the gap is that the file is ahead of this build"
    );
    assert!(
        !later.report.warnings.is_empty(),
        "a version gap carries a warning for a person to read"
    );

    let unknown = load(&fixture("rigged:ext-unknown")).expect("an unknown extension opens");
    assert!(
        unknown.report.version_gap.is_none(),
        "the fixture declares the current version"
    );
    let notices = &unknown.report.unrecognised_extensions;
    assert!(
        !notices.is_empty(),
        "an extension no build understands is reported"
    );
    for notice in notices {
        assert!(
            !notice.name.is_empty(),
            "an extension is named by its human-readable name, not its identifier"
        );
        assert_ne!(
            notice.name, notice.namespace,
            "the internal identifier is not what a person is shown"
        );
        assert!(!notice.version.is_empty(), "with the version it expects");
    }
    assert!(
        notices.iter().any(|n| n.namespace == "com.example.ribbon"),
        "the namespace this fixture adds is among them"
    );
    assert!(
        !unknown.report.warnings.is_empty(),
        "with the warning that the features behind them may not work here"
    );
}
