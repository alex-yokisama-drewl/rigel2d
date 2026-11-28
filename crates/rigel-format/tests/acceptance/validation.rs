//! Section H — a puppet that fails its integrity rules is rejected, not
//! repaired. With `ART-5` and `TRACK-1`, which belong with it.
//!
//! Each defect fixture is `rigged` with exactly one change, declaring the
//! current version, so a rejection can be attributed to the rule the fixture
//! breaks rather than to the first rule the validator happens to reach.
//!
//! `VAL-14` is a **human gate** and is not here: a person writes a validator
//! against the specification without our code, and *wrong looks like* a rule
//! whose outcome the reader cannot determine from the file alone.

use std::collections::BTreeSet;
use std::process::Command;

use rigel_format::{
    AxisPolarity, BindingValues, Id, NodeBody, NodeKind, PhysicsConstant, Rejection, Rule,
    declared_range, load, may_be_root, may_contain, save, validate, validate_file,
};

use crate::support::{
    Expect, assert_rejected_naming, bytes, catalogue, fixture, fixture_copy, scratch,
};

/// VAL-1 — the tree is acyclic.
#[test]
fn val_1_a_cycle_in_the_tree_is_rejected() {
    assert_rejected_naming("rigged:cycle", Rule::TreeAcyclic);
}

/// VAL-2 — one root.
#[test]
fn val_2_no_root_and_two_roots_are_rejected() {
    assert_rejected_naming("rigged:no-root", Rule::ExactlyOneRoot);
    assert_rejected_naming("rigged:two-roots", Rule::ExactlyOneRoot);
}

/// VAL-3 — every reference resolves.
#[test]
fn val_3_a_dangling_reference_is_rejected() {
    for name in [
        "rigged:dangling-node-ref",
        "rigged:dangling-texture-ref",
        "rigged:dangling-param-ref",
    ] {
        assert_rejected_naming(name, Rule::ReferenceResolves);
    }
}

/// VAL-4 — keypoints are ordered and within the range **the parameter
/// declares**, which is why a parameter declares one.
#[test]
fn val_4_keypoints_are_ordered_and_within_the_declared_range() {
    assert_rejected_naming("rigged:keypoints-unordered", Rule::KeypointsOrdered);
    assert_rejected_naming(
        "rigged:keypoint-outside-declared-range",
        Rule::KeypointWithinDeclaredRange,
    );
}

/// VAL-5 — the stale mark agrees with the value counts, in **both** directions.
///
/// A criterion that checks only the unmarked mismatch accepts a file claiming a
/// staleness that is not there, and the derived state is authoritative: a file
/// from a stranger can lie, a derivation cannot.
#[test]
fn val_5_the_stale_mark_must_agree_with_the_value_counts() {
    assert_rejected_naming(
        "rigged:binding-count-mismatch-unmarked",
        Rule::BindingValueCountAgreesWithStaleMark,
    );
    assert_rejected_naming(
        "rigged:binding-count-match-marked-stale",
        Rule::BindingValueCountAgreesWithStaleMark,
    );
}

/// VAL-6 — every number is finite, rejected and never clamped, in a file of any
/// declared version.
///
/// `rigged:nan` carries a bare `NaN` token, which a writer that is not ours
/// emits and which **is not strict JSON**. A strict parser refuses the file
/// before the finiteness rule is reached, so that one rejection may name no
/// identifier; the infinity pair uses `1e400`, which is strict JSON and
/// overflows on parse, and those two can. Recorded in `docs/wip/s2-format.md`.
#[test]
fn val_6_non_finite_numbers_are_rejected() {
    assert_rejected_naming("rigged:positive-infinity", Rule::NumbersFinite);
    assert_rejected_naming("rigged:negative-infinity", Rule::NumbersFinite);

    let rejection = load(&fixture("rigged:nan")).expect_err("a NaN is rejected, never clamped");
    let finiteness = rejection
        .violations()
        .iter()
        .any(|v| v.rule == Rule::NumbersFinite);
    let not_json = matches!(rejection, Rejection::Malformed { .. });
    assert!(
        finiteness || not_json,
        "a bare NaN token is refused either as finiteness or as malformed JSON, \
         not for some third reason: {rejection:?}"
    );
}

/// VAL-7 — enumerated values are known, or marked as extensions, in a file whose
/// declared version this build knows.
#[test]
fn val_7_an_unfamiliar_enumerated_value_is_rejected_at_the_current_version() {
    assert_rejected_naming("rigged:unknown-enum-unmarked", Rule::EnumeratedValueKnown);
}

/// VAL-8 — unmarked data the core format does not define is invalid, in a file
/// whose declared version this build knows.
#[test]
fn val_8_unmarked_core_data_is_rejected_at_the_current_version() {
    assert_rejected_naming("rigged:unmarked-core-data", Rule::UnmarkedData);
}

/// VAL-9 — the **valid** stale case: counts mismatch and the mark is present, so
/// mark and derivation agree. Conflating this with `VAL-5` rejects a file the
/// format defines as valid.
#[test]
fn val_9_a_marked_stale_binding_is_valid_and_survives_a_round_trip() {
    let loaded = load(&fixture("rigged:binding-count-mismatch-marked-stale"))
        .expect("a stale binding is a state, not a failure");
    let stale: Vec<Id> = loaded
        .puppet
        .bindings
        .iter()
        .filter(|b| b.stale)
        .map(|b| b.id)
        .collect();
    assert!(!stale.is_empty(), "the fixture carries the stale mark");
    assert_eq!(
        stale, loaded.report.stale_bindings,
        "a stale binding is reported to the rigger"
    );

    let dir = scratch("val-9");
    let written = dir.join("stale.rigel2d");
    save(&loaded.puppet, &written).expect("a stale binding saves");
    assert_eq!(
        loaded.puppet,
        load(&written).expect("it loads again").puppet,
        "the mark survives the round trip"
    );
}

/// VAL-10 — an undecodable texture payload does not reject the puppet. The
/// texture is reported unreadable, the part keeps its reference, and a save
/// writes the payload back byte for byte rather than re-encoding something it
/// could not read.
#[test]
fn val_10_an_undecodable_payload_is_reported_not_rejected() {
    let loaded =
        load(&fixture("rigged:texture-payload-corrupt")).expect("the puppet is not rejected");
    assert!(
        !loaded.report.unreadable_textures.is_empty(),
        "the texture is reported unreadable"
    );
    let unreadable = loaded.report.unreadable_textures[0];
    assert!(
        loaded.puppet.nodes.iter().any(|n| matches!(
            &n.body, NodeBody::Part(part) if part.texture == unreadable
        )),
        "the part keeps its reference"
    );
    let arrived = loaded
        .puppet
        .textures
        .iter()
        .find(|t| t.id == unreadable)
        .expect("the texture record is there")
        .payload
        .clone();

    let dir = scratch("val-10");
    let written = dir.join("corrupt.rigel2d");
    save(&loaded.puppet, &written).expect("it saves");
    let reloaded = load(&written).expect("it loads").puppet;
    assert_eq!(
        arrived,
        reloaded
            .textures
            .iter()
            .find(|t| t.id == unreadable)
            .expect("the texture record is still there")
            .payload,
        "the undecodable payload is written back byte for byte"
    );
}

/// VAL-11 — a load never silently repairs and writes back. Every rejected
/// fixture in the catalogue, byte-identical after the rejected load.
#[test]
fn val_11_a_rejected_load_leaves_the_file_byte_identical() {
    for entry in catalogue()
        .fixtures
        .iter()
        .filter(|f| f.expect == Expect::Rejected)
    {
        let path = fixture_copy(&entry.name);
        let before = bytes(&path);
        assert!(
            load(&path).is_err(),
            "{} is catalogued as rejected and must be",
            entry.name
        );
        assert_eq!(before, bytes(&path), "{} was written to", entry.name);
    }
}

/// VAL-12 — every rejection names the rule broken and at least one offending
/// identifier, in bounded output.
///
/// The identifier half is asserted where the file got far enough for an
/// identifier to exist: a container, version or JSON fault is refused before any
/// entity has been read, which is a property of the format rather than a gap in
/// the error type.
#[test]
fn val_12_every_rejection_names_a_rule_and_is_bounded() {
    for entry in catalogue()
        .fixtures
        .iter()
        .filter(|f| f.expect == Expect::Rejected)
    {
        let rejection = load(&fixture(&entry.name))
            .err()
            .unwrap_or_else(|| panic!("{} must be rejected", entry.name));
        assert!(
            !rejection.rule_name().is_empty(),
            "{} is rejected without naming a rule",
            entry.name
        );
        let rendered = rejection.to_string();
        assert!(
            !rendered.is_empty() && rendered.len() <= Rejection::MAX_MESSAGE_BYTES,
            "{}'s message is {} bytes, which is not bounded output",
            entry.name,
            rendered.len()
        );
        if !rejection.violations().is_empty() {
            assert!(
                !rejection.offenders().is_empty(),
                "{} breaks a rule and must name at least one offending identifier",
                entry.name
            );
        }
    }
}

/// VAL-13 — one validator gates load and save, and runs as a standalone command.
///
/// Three assertions, and the third is the one a second validator fails: the
/// standalone verdict equals the loader's **on every fixture in the catalogue**,
/// not on a sample.
#[test]
fn val_13_one_validator_gates_load_and_save_and_runs_standalone() {
    for entry in &catalogue().fixtures {
        let path = fixture(&entry.name);
        let through_loader = load(&path).is_ok();
        let standalone = validate_file(&path).is_ok();
        assert_eq!(
            through_loader, standalone,
            "{}: the standalone verdict and the loader's disagree",
            entry.name
        );
        assert_eq!(
            through_loader,
            entry.expect == Expect::Accepted,
            "{}: the verdict disagrees with the catalogue",
            entry.name
        );

        let run = Command::new(env!("CARGO_BIN_EXE_rigel-validate"))
            .arg(&path)
            .output()
            .expect("the standalone command runs");
        let said = String::from_utf8_lossy(&run.stdout);
        let first = said.lines().next().unwrap_or_default();
        if through_loader {
            assert_eq!(first, "accepted", "{}: the command's verdict", entry.name);
            assert!(run.status.success());
        } else {
            assert!(
                first.starts_with("rejected"),
                "{}: the command said {first:?}",
                entry.name
            );
            assert!(!run.status.success());
        }
    }

    // A save refuses to write a puppet the validator rejects.
    let mut broken = load(&fixture("rigged")).expect("it loads").puppet;
    let dangling = Id(u64::MAX);
    broken.bindings.first_mut().expect("a binding").parameter = dangling;
    assert!(
        validate(&broken).is_err(),
        "a dangling reference is invalid in memory as much as on disk"
    );
    let dir = scratch("val-13");
    let refused = dir.join("refused.rigel2d");
    assert!(
        save(&broken, &refused).is_err(),
        "a save refuses to write a puppet the validator rejects"
    );
    assert!(!refused.exists(), "and writes nothing while refusing");
}

/// VAL-15 — every parameter has exactly one neutral keypoint. Both none and more
/// than one are invalid, and *easing to neutral* has no meaning without it.
#[test]
fn val_15_a_parameter_has_exactly_one_neutral_keypoint() {
    assert_rejected_naming(
        "rigged:no-neutral-keypoint",
        Rule::ExactlyOneNeutralKeypoint,
    );
    assert_rejected_naming(
        "rigged:two-neutral-keypoints",
        Rule::ExactlyOneNeutralKeypoint,
    );
}

/// VAL-16 — no part has children. A part draws and is a leaf.
///
/// Asserted here as the criterion names it, and from the declaration at
/// `VAL-23`.
#[test]
fn val_16_a_part_has_no_children() {
    assert_rejected_naming("rigged:part-with-child", Rule::Containment);
}

/// VAL-17 — each of a parameter's axes declares an axis polarity, which is one
/// of the three the format defines. Neither fixture is defaulted nor read as
/// *unknown*.
#[test]
fn val_17_an_axis_declares_a_polarity_the_format_defines() {
    assert_rejected_naming("rigged:missing-polarity", Rule::AxisPolarityDeclared);
    assert_rejected_naming(
        "rigged:polarity-value-unfamiliar",
        Rule::EnumeratedValueKnown,
    );
}

/// VAL-18 — finiteness is judged at the precision the runtime evaluates in, not
/// at the precision the file was parsed in. Two tools reading one file must not
/// disagree about whether it is valid.
#[test]
fn val_18_finiteness_is_judged_at_the_precision_evaluation_uses() {
    assert_rejected_naming("rigged:nonfinite-after-narrowing", Rule::NumbersFinite);
}

/// VAL-19 — an unreferenced texture is **valid**, and the validator must not
/// invent a rule against one.
///
/// The cost half (`COLL-14`) needs resident-byte accounting and is S5's.
#[test]
fn val_19_an_unreferenced_texture_is_valid() {
    let path = fixture("rigged:orphan-texture");
    assert!(
        validate_file(&path).is_ok(),
        "the standalone validator accepts an unreferenced texture"
    );
    let loaded = load(&path).expect("and so does the loader");
    let referenced: BTreeSet<Id> = loaded
        .puppet
        .nodes
        .iter()
        .filter_map(|n| match &n.body {
            NodeBody::Part(part) => Some(part.texture),
            _ => None,
        })
        .collect();
    assert!(
        loaded
            .puppet
            .textures
            .iter()
            .any(|t| !referenced.contains(&t.id)),
        "the fixture carries a texture no part references"
    );

    let dir = scratch("val-19");
    save(&loaded.puppet, &dir.join("orphan.rigel2d")).expect("and a save of it succeeds");
}

/// VAL-20 — the unfamiliar polarity value carries the **version gate**, in one
/// differential assertion over byte-for-byte the same value.
#[test]
fn val_20_the_unfamiliar_polarity_value_is_gated_on_the_declared_version() {
    let at_current = crate::support::puppet_json(&fixture("rigged:polarity-value-unfamiliar"));
    let at_later = crate::support::puppet_json(&fixture("rigged:future-minor-polarity-unfamiliar"));
    let polarities = |doc: &serde_json::Value| -> Vec<String> {
        doc["parameters"]
            .as_array()
            .expect("parameters")
            .iter()
            .flat_map(|p| p["axes"].as_array().expect("axes").iter())
            .filter_map(|a| a["polarity"].as_str().map(str::to_owned))
            .collect()
    };
    assert_eq!(
        polarities(&at_current),
        polarities(&at_later),
        "the same value in both files, so the only difference is the declared version"
    );

    assert!(
        load(&fixture("rigged:polarity-value-unfamiliar")).is_err(),
        "at the current version it is unknown and therefore a defect"
    );
    assert!(
        load(&fixture("rigged:future-minor-polarity-unfamiliar")).is_ok(),
        "at a later version it is licensed and preserved opaquely"
    );
}

/// VAL-21 — an absent polarity and the declared value *unknown* are different
/// things, asserted differentially on one field.
///
/// The acceptance half is built here rather than taken from `rigged:paired`,
/// which serves `authoring-acceptance.md` and is deferred out of this subsystem:
/// an axis is set to *unknown* in memory and round-tripped, which is the same
/// claim — that the value is writable and survives.
#[test]
fn val_21_an_absent_polarity_is_not_the_value_unknown() {
    assert_rejected_naming("rigged:missing-polarity", Rule::AxisPolarityDeclared);

    let mut subject = load(&fixture("rigged")).expect("it loads").puppet;
    subject
        .parameters
        .first_mut()
        .expect("a parameter")
        .axes
        .first_mut()
        .expect("an axis")
        .polarity = AxisPolarity::Unknown;
    assert!(
        validate(&subject).is_ok(),
        "the declared value unknown is valid"
    );

    let dir = scratch("val-21");
    let written = dir.join("unknown-polarity.rigel2d");
    save(&subject, &written).expect("it saves");
    let reloaded = load(&written).expect("it loads").puppet;
    assert_eq!(
        reloaded.parameters[0].axes[0].polarity,
        AxisPolarity::Unknown,
        "the value is intact, not inferred from an absence and not lost"
    );
}

/// VAL-22 — a physics node contains only physics. The declaration a cascade
/// would otherwise ride through to artwork.
#[test]
fn val_22_physics_contains_only_physics() {
    assert_rejected_naming("rigged:physics-contains-part", Rule::Containment);
    assert_rejected_naming("rigged:physics-contains-mesh-group", Rule::Containment);
}

/// VAL-23 — the per-kind containment declaration is enforced in **both**
/// directions, table-driven from the declaration itself.
///
/// `VAL-16` and `VAL-22` are two instances of this. Driving it from
/// [`may_contain`] rather than from a list here is what covers a kind added
/// later without a new criterion. The wrong build satisfies the rejection half
/// by refusing every child of a kind it has no rule for — which stops a
/// composite inside a mesh group loading, and the puppet that fails is somebody
/// else's.
#[test]
fn val_23_containment_is_enforced_in_both_directions() {
    let permitted = load(&fixture("rigged:containment-permitted"))
        .expect("every permitted pair in one file validates and loads")
        .puppet;
    let dir = scratch("val-23");
    save(&permitted, &dir.join("permitted.rigel2d")).expect("and saves");

    let kind_of = |body: &NodeBody| match body {
        NodeBody::Part(_) => NodeKind::Part,
        NodeBody::MeshGroup(_) => NodeKind::MeshGroup,
        NodeBody::Composite => NodeKind::Composite,
        NodeBody::Physics(_) => NodeKind::Physics,
    };
    let mut carried: BTreeSet<(NodeKind, NodeKind)> = BTreeSet::new();
    for node in &permitted.nodes {
        if let Some(parent) = permitted.nodes.iter().find(|n| n.id == node.parent) {
            carried.insert((kind_of(&parent.body), kind_of(&node.body)));
        }
    }

    for parent in NodeKind::ALL {
        for child in NodeKind::ALL {
            let declared = may_contain(*parent, *child).unwrap_or_else(|| {
                panic!("the containment declaration has no answer for {parent:?}/{child:?}")
            });
            if declared {
                assert!(
                    carried.contains(&(*parent, *child)),
                    "{parent:?} may contain {child:?}, and rigged:containment-permitted \
                     must carry that pair"
                );
            } else {
                assert_rejected_naming(&forbidden_fixture(*parent, *child), Rule::Containment);
            }
        }
    }
}

/// The fixture carrying one forbidden containment pair.
///
/// The corpus's own naming, kept here rather than derived from the crate: a test
/// that asked the implementation what a kind is called would stop being a test
/// of the corpus.
fn forbidden_fixture(parent: NodeKind, child: NodeKind) -> String {
    if parent == NodeKind::Part && child == NodeKind::Part {
        return "rigged:part-with-child".to_owned();
    }
    let name = |kind: NodeKind| match kind {
        NodeKind::Part => "part",
        NodeKind::MeshGroup => "mesh-group",
        NodeKind::Composite => "composite",
        NodeKind::Physics => "physics",
    };
    format!("rigged:{}-contains-{}", name(parent), name(child))
}

/// VAL-24 — every node kind the format defines declares what it may contain, and
/// a kind with no declaration **fails the check** rather than inheriting a
/// default.
///
/// Tagged `script` in the criteria and owned by task 4; this is the behavioural
/// half available from here — the declaration is total over the kinds the format
/// defines, including the root question.
#[test]
fn val_24_every_node_kind_declares_what_it_may_contain() {
    for parent in NodeKind::ALL {
        assert!(
            may_be_root(*parent).is_some(),
            "{parent:?} must declare whether it may be the root"
        );
        for child in NodeKind::ALL {
            assert!(
                may_contain(*parent, *child).is_some(),
                "{parent:?} must declare whether it may contain {child:?}"
            );
        }
    }
}

/// VAL-25 — a physics node's simulation constants are range-validated at load,
/// before anything is allocated for them, and a constant with **no stated valid
/// range** fails the check.
///
/// Checking only that the numbers are finite is what `VAL-6` already does: a
/// finite file does not imply a finite simulation, and nothing downstream
/// re-checks.
#[test]
fn val_25_physics_constants_are_range_validated() {
    for constant in PhysicsConstant::ALL {
        assert!(
            declared_range(*constant).is_some(),
            "{constant:?} has no stated valid range"
        );
    }

    // Each degenerate value the format admits a fixture for, with the field the
    // rejection must name.
    let degenerate = [
        ("rigged:physics-length-zero", "length"),
        ("rigged:physics-length-negative", "length"),
        ("rigged:physics-frequency-zero", "frequency"),
        ("rigged:physics-frequency-negative", "frequency"),
        ("rigged:physics-gravity-negative", "gravity"),
        ("rigged:physics-angle-damping-negative", "angle_damping"),
        ("rigged:physics-length-damping-negative", "length_damping"),
    ];
    for (name, field) in degenerate {
        let rejection = load(&fixture(name))
            .err()
            .unwrap_or_else(|| panic!("{name} must be rejected"));
        let named = rejection
            .violations()
            .iter()
            .find(|v| v.rule == Rule::PhysicsConstantWithinRange)
            .unwrap_or_else(|| panic!("{name} must be rejected naming the range rule"));
        assert_eq!(
            named.field.as_deref(),
            Some(field),
            "{name}'s rejection names the constant"
        );
        assert!(
            !named.offenders.is_empty(),
            "{name}'s rejection names the node"
        );
    }
}

/// VAL-26 — a track's keyframes carry the rules a parameter's keypoints do, and
/// each rejection names the rule, the animation and the track.
///
/// Ordering is load-bearing rather than cosmetic: a lookup that seeks rather
/// than scans returns wrong values silently on unordered times instead of
/// failing.
#[test]
fn val_26_a_tracks_keyframes_are_validated() {
    let cases = [
        ("rigged:keyframes-unordered", Rule::KeyframesOrdered),
        (
            "rigged:keyframe-outside-declared-range",
            Rule::KeyframeWithinDeclaredRange,
        ),
        (
            "rigged:blend-duration-negative",
            Rule::BlendDurationNonNegative,
        ),
    ];
    let base = load(&fixture("rigged")).expect("it loads").puppet;
    let animations: BTreeSet<Id> = base.animations.iter().map(|a| a.id).collect();
    let tracks: BTreeSet<Id> = base
        .animations
        .iter()
        .flat_map(|a| a.tracks.iter().map(|t| t.id))
        .collect();

    for (name, rule) in cases {
        let rejection = load(&fixture(name))
            .err()
            .unwrap_or_else(|| panic!("{name} must be rejected"));
        let named = rejection
            .violations()
            .iter()
            .find(|v| v.rule == rule)
            .unwrap_or_else(|| panic!("{name} must be rejected naming {rule:?}"));
        let offenders: BTreeSet<Id> = named.offenders.iter().copied().collect();
        assert!(
            offenders.intersection(&animations).next().is_some(),
            "{name}'s rejection names the animation"
        );
        assert!(
            offenders.intersection(&tracks).next().is_some(),
            "{name}'s rejection names the track"
        );
    }
}

/// VAL-27 — an animation carries at most one track for any one parameter.
///
/// *Two tracks naming one parameter is a validation failure rather than a shape
/// the format cannot express*: the collection is a list, so the duplicate is
/// expressible and must be rejected, because keying it by parameter would make
/// the identifier serve as both key and reference and lose the legibility this
/// format exists for.
#[test]
fn val_27_two_tracks_for_one_parameter_are_rejected() {
    let rejection = load(&fixture("rigged:two-tracks-one-parameter"))
        .expect_err("two tracks for one parameter is malformed");
    let named = rejection
        .violations()
        .iter()
        .find(|v| v.rule == Rule::AtMostOneTrackPerParameter)
        .expect("rejected naming the one-track-per-parameter rule");

    let base = load(&fixture("rigged")).expect("it loads").puppet;
    let animations: BTreeSet<Id> = base.animations.iter().map(|a| a.id).collect();
    let offenders: BTreeSet<Id> = named.offenders.iter().copied().collect();
    assert!(
        offenders.intersection(&animations).next().is_some(),
        "the rejection names the animation the duplicate is in"
    );
    assert!(
        offenders.len() >= 2,
        "and the parameter or the tracks that collide, so it can be acted on"
    );
}

/// ART-5 — a stale deformation is not a malformed file: the validator accepts
/// the puppet and a save succeeds.
///
/// The route to staleness the criterion names — regenerating a mesh with a
/// different vertex count (`ART-4`) — is Studio's and out of scope here, so the
/// subject is the fixture that carries the same end state.
#[test]
fn art_5_a_stale_binding_validates_and_saves() {
    let path = fixture("rigged:binding-count-mismatch-marked-stale");
    assert!(
        validate_file(&path).is_ok(),
        "the standalone validator accepts a stale binding"
    );
    let subject = load(&path).expect("and so does the loader").puppet;
    assert!(
        validate(&subject).is_ok(),
        "the same validator accepts it in memory"
    );
    let dir = scratch("art-5");
    save(&subject, &dir.join("stale.rigel2d")).expect("and a save succeeds");
}

/// TRACK-1 — a track drives exactly one parameter, and the tracks of an
/// animation are distinguishable entities rather than one collection keyed by
/// parameter.
///
/// The wrong build keys an animation by parameter with no track entity, after
/// which nothing can be named per animation when a parameter is deleted, counted
/// against a limit, or restored by undo.
#[test]
fn track_1_a_track_drives_exactly_one_parameter() {
    let loaded = load(&fixture("rigged")).expect("it loads").puppet;
    let animation = loaded.animations.first().expect("an animation");
    assert_eq!(
        animation.tracks.len(),
        2,
        "the base fixture's animation carries one track for each of two parameters"
    );

    let parameters: BTreeSet<Id> = loaded.parameters.iter().map(|p| p.id).collect();
    let driven: BTreeSet<Id> = animation.tracks.iter().map(|t| t.parameter).collect();
    assert_eq!(
        driven.len(),
        animation.tracks.len(),
        "each track names exactly one parameter, and no two name the same"
    );
    assert!(
        driven.is_subset(&parameters),
        "each names a parameter of this puppet"
    );

    let identities: BTreeSet<Id> = animation.tracks.iter().map(|t| t.id).collect();
    assert_eq!(
        identities.len(),
        animation.tracks.len(),
        "the tracks are entities in their own right, each with an identifier"
    );
    assert!(
        animation.tracks.iter().all(|t| !t.keyframes.is_empty()),
        "each carries its own keyframes"
    );

    let dir = scratch("track-1");
    let written = dir.join("tracks.rigel2d");
    save(&loaded, &written).expect("it saves");
    assert_eq!(
        loaded.animations,
        load(&written).expect("it loads").puppet.animations,
        "each track round-trips with its identifier and its keyframes"
    );
}

/// The validation rules `docs/format.md` states that no criterion gives a name
/// to, asserted through the catalogue so they are not silently unexercised.
///
/// Not a criterion: the corpus carries one fixture per rule and
/// `docs/wip/s2-format.md` records why each was built. The rule names are the
/// subject — these fixtures are what stop the one-defect corpus shrinking to
/// whatever the criteria happened to enumerate.
#[test]
fn the_format_rules_without_a_criterion_are_still_enforced() {
    assert_rejected_naming("root-is-part", Rule::RootIsAKindThatContains);
    assert_rejected_naming("rigged:two-base-drivers", Rule::AtMostOneBaseDriver);
    assert_rejected_naming(
        "rigged:id-above-high-water-mark",
        Rule::IdentifierWithinHighWaterMark,
    );
    assert_rejected_naming("rigged:id-zero", Rule::IdentifierZeroReserved);
}

/// Binding values are stored one entry per keypoint, which is the count
/// `VAL-5` compares against. Asserted on the base fixture so that the pairing at
/// `VAL-5`/`VAL-9` has a known-good baseline.
#[test]
fn a_bindings_values_are_one_per_keypoint_of_its_parameter() {
    let loaded = load(&fixture("rigged")).expect("it loads").puppet;
    for binding in &loaded.bindings {
        let parameter = loaded
            .parameters
            .iter()
            .find(|p| p.id == binding.parameter)
            .expect("a binding names a parameter of this puppet");
        assert!(!binding.stale, "the base fixture carries no stale binding");
        let stored = match &binding.values {
            BindingValues::Transform(values) => values.len(),
            BindingValues::Deformation(values) => values.len(),
        };
        assert_eq!(
            stored,
            parameter.keypoints.len(),
            "binding {:?} stores one value per keypoint of {:?}",
            binding.id,
            parameter.id
        );
    }
}
