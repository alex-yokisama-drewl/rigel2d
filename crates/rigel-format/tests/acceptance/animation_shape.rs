//! Section R — the **format half**: per-segment interpolation, an animation's
//! end behaviour, its triggers, and a parameter's stored order and `mappable`
//! flag.
//!
//! Written from `docs/requirements/puppet-acceptance.md` section R and
//! `docs/format.md` by `acceptance-planner`, for subsystem S2b, **without sight
//! of `decode.rs`, `validate.rs`, `save.rs`, `load.rs`, `json.rs` or
//! `archive.rs`**. These assert what the requirement asks for, not what the
//! reader happens to do.
//!
//! # What is here and what is not
//!
//! S2b **stores and restores; it does not compute**. Every criterion in section
//! R that needs a curve sampled, a sum composed, a clamp applied or a trigger
//! dispatched is **S4's**, and the ones split between the two say so where they
//! are implemented here. In particular `ADD-*`, `CLAMP-*`, `INTERP-1`,
//! `INTERP-5`, `END-1`, `END-2`, `END-3`, `LOST-1`, `LONG-*` and `MAP-2` are
//! not here at all.
//!
//! `ORD-1`'s *Live's mapping list presents the parameters in it* and `MAP-1`'s
//! *the targets Live offers* are **Live's**, and Live does not exist. What a
//! file can answer is asserted; what an application must do with it is named in
//! the test that carries the other half.
//!
//! # Naming a rule, and where that is deliberately not done
//!
//! [`crate::support::assert_rejected_naming`] pins a rejection to the rule it
//! must name, which is what makes a one-defect corpus worth having. It is used
//! wherever the criterion or `docs/format.md` **names the rule** — unmarked
//! data for a trigger's extra field, the enumerated-value rule for an
//! unfamiliar token. Where neither does, [`assert_refused`] asserts the
//! rejection and no more: choosing a rule name here would be designing the
//! validator from the test, which is the one thing this suite must not do.
//! **Task 3 owes a `Rule` variant for each of those six fixtures**, and a later
//! pass may tighten these assertions once the variants exist.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use rigel_format::{
    Animation, Basis, EndBehaviour, Id, Interpolation, Keyframe, LimitKind, NodeBody, Parameter,
    Puppet, Rejection, Rule, Track, declared_limit, limits::FIGURES, load, save, validate,
    validate_file,
};

use crate::counting::peak_bytes;
use crate::support::{assert_rejected_naming, bytes, fixture, puppet_json, scratch};

// ---------------------------------------------------------------- scaffolding

fn puppet(name: &str) -> Puppet {
    load(&fixture(name))
        .unwrap_or_else(|e| panic!("{name} is valid and must load: {e}"))
        .puppet
}

/// Saves a puppet into a scratch directory of its own and returns the path.
fn save_to(subject: &Puppet, label: &str) -> std::path::PathBuf {
    let dir = scratch(label);
    let written = dir.join("written.rigel2d");
    save(subject, &written).unwrap_or_else(|e| panic!("{label} saves: {e:?}"));
    written
}

fn round_trip(name: &str, label: &str) -> Puppet {
    let written = save_to(&puppet(name), label);
    load(&written)
        .unwrap_or_else(|e| panic!("what we wrote loads: {e}"))
        .puppet
}

/// A fixture the catalogue marks `rejected` is refused — by the loader and by
/// the standalone validator alike — with a bounded, non-empty message.
///
/// Weaker than [`assert_rejected_naming`] on purpose: see the module comment.
///
/// **It cannot tell the right refusal from the wrong one, and today it gets
/// the wrong one.** Before task 3, every fixture passed to this is refused for
/// `unmarked-data` on a field this build has never heard of, rather than for
/// the defect its catalogue row names — and `rigged:end-behaviour-absent` is
/// not refused at all, because removing a field nothing reads changes nothing.
/// That is the correct red for S2b, and it is also why a `Rule` variant per
/// defect is **owed by task 3**: until one exists, these assertions hold
/// against a build that refuses the file for any reason whatsoever.
fn assert_refused(name: &str) -> Rejection {
    let path = fixture(name);
    assert!(
        validate_file(&path).is_err(),
        "{name} carries a defect and the standalone validator must refuse it (VAL-13)"
    );
    let rejection = load(&path)
        .err()
        .unwrap_or_else(|| panic!("{name} carries a defect and must be refused"));
    let message = rejection.to_string();
    assert!(
        !message.is_empty() && message.len() <= Rejection::MAX_MESSAGE_BYTES,
        "{name}'s rejection message is readable and bounded (VAL-12)"
    );
    rejection
}

/// The one animation of a puppet carrying triggers.
fn triggered_animation(subject: &Puppet) -> &Animation {
    subject
        .animations
        .iter()
        .find(|a| !a.triggers.is_empty())
        .expect("the fixture carries an animation with triggers")
}

/// The track in this puppet whose segments do not all interpolate the same way.
///
/// Found by shape rather than by identifier, so the corpus can be regenerated
/// without this test quietly ceasing to find the material it is about.
fn mixed_track(subject: &Puppet) -> &Track {
    subject
        .animations
        .iter()
        .flat_map(|a| &a.tracks)
        .find(|t| {
            t.keyframes
                .iter()
                .map(|k| &k.interpolation)
                .collect::<HashSet<_>>()
                .len()
                > 1
        })
        .expect("the fixture carries a track mixing interpolation modes")
}

/// Every keyframe object anywhere in a parsed `puppet.json`.
fn json_keyframes(doc: &serde_json::Value) -> Vec<serde_json::Value> {
    doc["animations"]
        .as_array()
        .expect("an animations array")
        .iter()
        .flat_map(|a| a["tracks"].as_array().expect("a tracks array"))
        .flat_map(|t| t["keyframes"].as_array().expect("a keyframes array"))
        .cloned()
        .collect()
}

/// Every trigger object anywhere in a parsed `puppet.json`.
fn json_triggers(doc: &serde_json::Value) -> Vec<serde_json::Value> {
    doc["animations"]
        .as_array()
        .expect("an animations array")
        .iter()
        .filter_map(|a| a.get("triggers"))
        .flat_map(|t| t.as_array().expect("triggers is an array"))
        .cloned()
        .collect()
}

/// Every reference a puppet stores, as a set of triples, so that *nothing
/// moved* can be asserted over all of them at once rather than field by field.
fn references(subject: &Puppet) -> BTreeSet<(&'static str, u64, u64)> {
    let mut out = BTreeSet::new();
    out.insert(("root", subject.id.0, subject.root.0));
    for node in &subject.nodes {
        out.insert(("node.parent", node.id.0, node.parent.0));
        match &node.body {
            NodeBody::Part(part) => {
                out.insert(("part.texture", node.id.0, part.texture.0));
                out.insert(("part.mask", node.id.0, part.mask.0));
            }
            NodeBody::Physics(physics) => {
                out.insert(("physics.output", node.id.0, physics.output.0));
            }
            _ => {}
        }
    }
    for binding in &subject.bindings {
        out.insert(("binding.parameter", binding.id.0, binding.parameter.0));
        out.insert(("binding.target", binding.id.0, binding.target.0));
    }
    for mapping in &subject.channel_mappings {
        out.insert(("mapping.parameter", mapping.id.0, mapping.parameter.0));
    }
    for animation in &subject.animations {
        for track in &animation.tracks {
            out.insert(("track.parameter", track.id.0, track.parameter.0));
        }
    }
    out
}

/// Every identifier a puppet carries.
fn identifiers(subject: &Puppet) -> BTreeSet<u64> {
    let mut out: BTreeSet<u64> = BTreeSet::new();
    out.insert(subject.id.0);
    out.extend(subject.nodes.iter().map(|n| n.id.0));
    out.extend(subject.textures.iter().map(|t| t.id.0));
    out.extend(subject.bindings.iter().map(|b| b.id.0));
    out.extend(subject.channel_mappings.iter().map(|m| m.id.0));
    for parameter in &subject.parameters {
        out.insert(parameter.id.0);
        out.extend(parameter.keypoints.iter().map(|k| k.id.0));
    }
    for animation in &subject.animations {
        out.insert(animation.id.0);
        out.extend(animation.tracks.iter().map(|t| t.id.0));
    }
    out
}

/// The parameter a channel mapping feeds.
fn mapped_parameter(subject: &Puppet) -> &Parameter {
    let fed: BTreeSet<Id> = subject
        .channel_mappings
        .iter()
        .map(|m| m.parameter)
        .collect();
    subject
        .parameters
        .iter()
        .find(|p| fed.contains(&p.id))
        .expect("the fixture carries a channel mapping")
}

// ------------------------------------------------------------- interpolation

/// INTERP-2 — **a mode belongs to a segment, not a track.**
///
/// The half this subsystem owns: the modes **round-trip per segment**, and a
/// track whose segments are all linear **stores no mode anywhere**. *Each
/// segment evaluates under its own mode* needs a curve sampled and is S4's.
///
/// The storage rule is `docs/format.md`'s: the mode sits on the **earlier**
/// keyframe of its segment, the last keyframe carries none, and linear is the
/// absent form. The wrong build this catches is one mode per track, which
/// cannot express or import a track that mixes them.
#[test]
fn interp_2_modes_round_trip_per_segment_and_a_linear_track_stores_none() {
    let subject = puppet("rigged:mixed-interpolation");
    let track = mixed_track(&subject);

    let modes: Vec<&Interpolation> = track.keyframes.iter().map(|k| &k.interpolation).collect();
    assert_eq!(
        modes,
        vec![
            &Interpolation::Stepped,
            &Interpolation::InverseStepped,
            &Interpolation::Linear,
            &Interpolation::Bezier,
            &Interpolation::Bezier,
            &Interpolation::Linear,
        ],
        "one mode per segment, on the earlier keyframe of each"
    );
    let last = track.keyframes.last().expect("a last keyframe");
    assert_eq!(
        last.interpolation,
        Interpolation::Linear,
        "the last keyframe opens no segment and carries no mode — the absent form"
    );
    assert!(
        last.control_points.is_none(),
        "and no control points either"
    );
    for keyframe in &track.keyframes {
        assert_eq!(
            keyframe.control_points.is_some(),
            keyframe.interpolation == Interpolation::Bezier,
            "control points ride on the Bézier segments and on nothing else"
        );
    }

    // The same file carries an all-linear track, which must store no mode.
    let linear_track = subject
        .animations
        .iter()
        .flat_map(|a| &a.tracks)
        .find(|t| t.id != track.id)
        .expect("the fixture carries a second track");
    assert!(
        linear_track
            .keyframes
            .iter()
            .all(|k| k.interpolation == Interpolation::Linear),
        "the second track is all linear, or this asserts nothing"
    );

    let after = round_trip("rigged:mixed-interpolation", "interp-2");
    assert_eq!(
        subject.animations, after.animations,
        "every mode round-trips with its own segment"
    );
    assert_eq!(
        bytes(&fixture("rigged:mixed-interpolation")),
        bytes(&save_to(&subject, "interp-2-bytes")),
        "and a save of the unmodified puppet reproduces the file"
    );

    // `rigged` is the control: every segment in it is linear, so no keyframe
    // anywhere in the file stores a mode.
    for keyframe in json_keyframes(&puppet_json(&fixture("rigged"))) {
        assert!(
            keyframe.get("interpolation").is_none(),
            "an all-linear track stores no mode anywhere: {keyframe}"
        );
    }
    let written = save_to(&puppet("rigged"), "interp-2-linear");
    for keyframe in json_keyframes(&puppet_json(&written)) {
        assert!(
            keyframe.get("interpolation").is_none(),
            "and the writer stores none either — linear is the absent form"
        );
    }
}

/// INTERP-3, the **stored** half — a mode follows **its own segment**, keyed by
/// the keyframe that opens it rather than by position in the list.
///
/// The half over *every operation that inserts or deletes a keyframe* needs
/// editing operations that do not exist and is **Studio's**. What can be
/// asserted from the file alone is that the pairing of segment start time to
/// mode survives a round trip — a writer that emitted modes positionally from a
/// collection it had re-sorted would break this and pass `INTERP-2`'s sequence
/// check only by luck.
#[test]
fn interp_3_the_stored_half_pairs_each_mode_with_its_own_segment() {
    let before = puppet("rigged:mixed-interpolation");
    let after = round_trip("rigged:mixed-interpolation", "interp-3");

    let by_time = |subject: &Puppet| -> BTreeMap<String, String> {
        mixed_track(subject)
            .keyframes
            .iter()
            .map(|k| (format!("{:?}", k.time), format!("{:?}", k.interpolation)))
            .collect()
    };
    assert_eq!(
        by_time(&before),
        by_time(&after),
        "each mode is still on the keyframe that opens its own segment"
    );

    let controls = |subject: &Puppet| -> Vec<(String, String)> {
        mixed_track(subject)
            .keyframes
            .iter()
            .filter_map(|k| {
                k.control_points
                    .as_ref()
                    .map(|c| (format!("{:?}", k.time), format!("{c:?}")))
            })
            .collect()
    };
    assert_eq!(
        controls(&before),
        controls(&after),
        "and so are the control points of each Bézier segment"
    );
}

/// INTERP-4, the format half — **a parameter carries no interpolation field.**
///
/// Interpolation between a parameter's keypoints is linear only, so there is
/// nothing to store: the field exists on a keyframe and nowhere else. *A
/// binding evaluates linearly between keypoints, unaffected by any track's
/// modes* is evaluation's and is S4's. The wrong build lets a track's mode
/// setting leak to keypoint interpolation, which needs somewhere to be written
/// down first — this test is what removes that somewhere.
#[test]
fn interp_4_a_parameter_carries_no_interpolation_field() {
    for name in ["rigged", "rigged:mixed-interpolation"] {
        let doc = puppet_json(&fixture(name));
        for parameter in doc["parameters"].as_array().expect("a parameters array") {
            assert!(
                parameter.get("interpolation").is_none(),
                "{name}: a parameter carries no interpolation field"
            );
            for axis in parameter["axes"].as_array().expect("an axes array") {
                assert!(
                    axis.get("interpolation").is_none(),
                    "{name}: nor does one of its axes"
                );
            }
            for keypoint in parameter["keypoints"].as_array().expect("keypoints") {
                assert!(
                    keypoint.get("interpolation").is_none(),
                    "{name}: nor does one of its keypoints"
                );
            }
        }
    }
}

/// A Bézier segment's control points are validated: times inside the segment,
/// values finite, and a mode that needs them does not arrive without them.
///
/// Carries the `security` concern entry *a track's keyframe times are finite
/// and strictly ordered, its values finite and within its parameter's range,
/// and its blend durations finite and non-negative* — already enforced for the
/// keyframes themselves, and new for the control points.
///
/// **The interval is closed.** `docs/format.md` says control times *lie inside
/// the segment* and does not settle the endpoints; they must be admitted, or
/// the degenerate curve the corpus carries and `INTERP-5` sweeps is
/// inexpressible — there is no other way to produce the zero derivative. The
/// accepted half of this test is what pins that reading.
#[test]
fn a_bezier_segments_control_points_are_validated() {
    for name in [
        "rigged:bezier-without-control-points",
        "rigged:bezier-control-time-outside-segment",
        "rigged:bezier-control-value-nonfinite",
    ] {
        assert_refused(name);
    }

    let subject = puppet("rigged:mixed-interpolation");
    let track = mixed_track(&subject);
    let degenerate = track
        .keyframes
        .iter()
        .find(|k| {
            k.control_points
                .as_ref()
                .is_some_and(|c| c[0].time == k.time && c[1].time == k.time)
        })
        .expect("the fixture carries a degenerate Bézier, both controls on the keyframe");
    let segment_end = track
        .keyframes
        .iter()
        .find(|k| k.time > degenerate.time)
        .expect("the degenerate segment has a later keyframe");
    for control in degenerate.control_points.as_ref().expect("control points") {
        assert!(
            control.time >= degenerate.time && control.time <= segment_end.time,
            "a control time lies in the closed interval of its segment"
        );
        assert!(
            control.value.iter().all(|v| v.is_finite()),
            "and every control value is finite"
        );
    }
    assert!(
        validate(&subject).is_ok(),
        "a control point sitting on a keyframe is admitted, or the degenerate curve \
         cannot be written at all"
    );
}

// -------------------------------------------------------------- end behaviour

/// END-4 — **end behaviour is not a type.**
///
/// `judgment` in the criteria, and the half of it a file can answer is
/// asserted: the puppet carries **no animation kind alongside the end
/// behaviour**, and a looping, a holding and a releasing animation are the same
/// entity with a different answer to one question — the same key set, differing
/// in that one member's value.
///
/// The wrong build gives an animation a `kind`, after which looping becomes a
/// species and a rigger has to choose it before authoring anything.
#[test]
fn end_4_end_behaviour_is_one_field_and_not_a_kind_of_animation() {
    // Everything `docs/format.md` defines on an animation, plus the extension
    // marker. Nothing here names a species of animation.
    const ANIMATION_KEYS: &[&str] = &["id", "name", "tracks", "end_behaviour", "triggers", "x"];

    let doc = puppet_json(&fixture("rigged:endings"));
    let animations = doc["animations"].as_array().expect("an animations array");
    let mut behaviours = BTreeSet::new();
    let mut shapes = BTreeSet::new();
    for animation in animations {
        let keys: BTreeSet<&str> = animation
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        for key in &keys {
            assert!(
                ANIMATION_KEYS.contains(key),
                "an animation carries {key:?}, which the format does not define for it"
            );
        }
        behaviours.insert(
            animation["end_behaviour"]
                .as_str()
                .expect("an end behaviour")
                .to_owned(),
        );
        shapes.insert(keys);
    }
    assert_eq!(
        behaviours.len(),
        3,
        "the fixture carries one animation per end behaviour"
    );
    assert_eq!(
        shapes.len(),
        1,
        "and they are one entity with one field differing, not three kinds: {shapes:?}"
    );
}

/// An animation's end behaviour is **always written**, round-trips, and its
/// absence is a defect.
///
/// `docs/format.md` states no default for the field, so the absent-when-default
/// rule does not reach it and a reader meeting it absent has nothing to fall
/// back on. The three values are `END-1`, `END-2` and `END-3`'s subject; what
/// is asserted here is only that the file carries them and gives them back.
#[test]
fn an_end_behaviour_is_always_written_and_round_trips() {
    let doc = puppet_json(&fixture("rigged"));
    for animation in doc["animations"].as_array().expect("an animations array") {
        assert!(
            animation.get("end_behaviour").is_some(),
            "a field with no stated default is always written, even in the base fixture"
        );
    }

    let subject = puppet("rigged:endings");
    let carried: HashSet<EndBehaviour> = subject
        .animations
        .iter()
        .map(|a| a.end_behaviour.clone())
        .collect();
    assert!(
        carried.contains(&EndBehaviour::Loop)
            && carried.contains(&EndBehaviour::Hold)
            && carried.contains(&EndBehaviour::Release),
        "all three behaviours the format defines are expressible: {carried:?}"
    );

    assert_eq!(
        subject.animations,
        round_trip("rigged:endings", "endings").animations,
        "and each round-trips with its own animation"
    );
    assert_eq!(
        bytes(&fixture("rigged:endings")),
        bytes(&save_to(&subject, "endings-bytes")),
        "byte-stably, through our writer"
    );

    assert_refused("rigged:end-behaviour-absent");
}

// ------------------------------------------------------------------ triggers

/// LOST-3 — **an event name nothing raises never fires and is not invalid.**
///
/// The format half: `rigged:triggered` **validates, loads and saves
/// byte-stably**, and nothing about the unraised name is reported as a defect.
/// *The animation behind it never starts* needs dispatch and is **S4's**.
///
/// The wrong build rejects the puppet for an event it has not heard of, so a
/// puppet made for a scene or a plugin the recipient lacks will not open.
#[test]
fn lost_3_an_event_name_nothing_raises_is_valid_and_saves_byte_stably() {
    let path = fixture("rigged:triggered");
    assert!(
        validate_file(&path).is_ok(),
        "the standalone validator accepts an event name it has never heard of"
    );
    let loaded = load(&path).expect("and so does the loader");
    let subject = &loaded.puppet;
    let animation = triggered_animation(subject);

    let events: Vec<&str> = animation
        .triggers
        .iter()
        .map(|t| t.event.as_str())
        .collect();
    assert_eq!(
        events.len(),
        3,
        "the fixture waits on three events: one raised on load, one for a lost \
         source, and one nothing raises"
    );
    assert_eq!(
        events.iter().collect::<BTreeSet<_>>().len(),
        events.len(),
        "each named once — one animation naming one event twice is invalid"
    );
    let unraised = events
        .iter()
        .find(|e| !["on_load", "source_lost"].contains(e))
        .expect("one name nothing raises");
    assert!(
        !loaded
            .report
            .warnings
            .iter()
            .any(|w| w.contains(unraised) || w.contains(&animation.name)),
        "nothing about the unraised name is reported as a defect"
    );
    assert!(
        validate(subject).is_ok(),
        "and it is not a validation failure at any version"
    );

    assert_eq!(
        subject.animations,
        round_trip("rigged:triggered", "lost-3").animations,
        "every trigger round-trips, the unraised one included"
    );
    assert_eq!(
        bytes(&path),
        bytes(&save_to(subject, "lost-3-bytes")),
        "and a save of the unmodified puppet reproduces the file"
    );
}

/// LOST-4, the **rejection** half — a trigger carrying any field beyond an
/// event name is **unmarked data** at the current version.
///
/// *A trigger starts exactly the animation it lives on* needs dispatch and is
/// **S4's**. The criterion names the rule — `EXT-2`'s shape — so it is pinned
/// here rather than left open.
///
/// Carries the `security` concern entry *a trigger is data selecting from a
/// set, and no field of it is evaluated*, and asserts **I1**: a condition, a
/// delay or a target is where a puppet would stop being data, and the shape is
/// what keeps it out.
#[test]
fn lost_4_a_trigger_carrying_more_than_an_event_name_is_unmarked_data() {
    assert_rejected_naming("rigged:trigger-extra-field", Rule::UnmarkedData);
    assert_refused("rigged:duplicate-trigger-event");

    // The accepted half: an event name shaped like a condition is a **name**.
    // The set of names is open at every version, so this is an ordinary puppet
    // — and the string is carried verbatim, never parsed into the comparison it
    // looks like.
    let path = fixture("rigged:trigger-expression-shaped-event");
    assert!(
        validate_file(&path).is_ok(),
        "an unrecognised event name is never a validation failure, whatever its shape"
    );
    let subject = load(&path).expect("it loads").puppet;
    let event = &triggered_animation(&subject)
        .triggers
        .first()
        .expect("a trigger")
        .event;
    assert!(
        event.contains('>') && event.contains("and"),
        "the fixture carries an expression-shaped name, or this asserts nothing"
    );
    assert_eq!(
        bytes(&path),
        bytes(&save_to(&subject, "lost-4-bytes")),
        "and it is written back unchanged, as the string it is"
    );

    // No trigger in the corpus's accepted fixtures carries anything but an
    // event name. The surface has nowhere to put one; this is the same claim
    // read off the stored form, where a later field would first appear.
    for name in [
        "rigged:triggered",
        "rigged:trigger-expression-shaped-event",
        "rigged:triggers-unsorted",
    ] {
        for trigger in json_triggers(&puppet_json(&fixture(name))) {
            let keys: Vec<&String> = trigger.as_object().expect("an object").keys().collect();
            assert_eq!(
                keys,
                vec!["event"],
                "{name}: a trigger carries one thing — the name of the event it waits for"
            );
        }
    }
}

// ------------------------------------------------- an unfamiliar value, again

/// UNF-1, the format half — an unfamiliar value in a **later-version** file
/// **loads and is written back verbatim**.
///
/// The *equals the control puppet with that track deleted / with that animation
/// absent* halves need evaluation and are **S4's**; so does *even when its
/// trigger fires*. What a file can answer is that the token survives: a build
/// with no code for a value must carry it, and must not fall back to linear or
/// to loop, which is the guess the rule forbids.
///
/// Carries the `data` concern entry *every enumerated field declares what a
/// build that does not know its value does, and carries a fixture of one*: both
/// new enumerated fields have a pair of fixtures, one each side of the version
/// gate.
#[test]
fn unf_1_an_unfamiliar_mode_and_end_behaviour_load_and_are_written_back_verbatim() {
    let subject = puppet("rigged:future-minor-interpolation-unfamiliar");
    let token = subject
        .animations
        .iter()
        .flat_map(|a| &a.tracks)
        .flat_map(|t| &t.keyframes)
        .find_map(|k| match &k.interpolation {
            Interpolation::Unfamiliar(token) => Some(token.clone()),
            _ => None,
        })
        .expect("the unfamiliar mode loads, carrying its token");
    assert!(!token.is_empty(), "and the token is the file's, verbatim");
    let track = subject
        .animations
        .iter()
        .flat_map(|a| &a.tracks)
        .find(|t| {
            t.keyframes
                .iter()
                .any(|k| matches!(k.interpolation, Interpolation::Unfamiliar(_)))
        })
        .expect("the track carrying it");
    let at = track
        .keyframes
        .iter()
        .position(|k| matches!(k.interpolation, Interpolation::Unfamiliar(_)))
        .expect("its position");
    assert!(
        at > 0 && at + 1 < track.keyframes.len(),
        "the mode sits on a segment in the **middle** of the track, so a build \
         dropping only the segment can be told from one dropping the track"
    );

    let ending = puppet("rigged:future-minor-end-behaviour-unfamiliar");
    assert!(
        ending
            .animations
            .iter()
            .any(|a| matches!(a.end_behaviour, EndBehaviour::Unfamiliar(_))),
        "the unfamiliar end behaviour loads, carrying its token"
    );

    for (name, label) in [
        ("rigged:future-minor-interpolation-unfamiliar", "unf-1-mode"),
        ("rigged:future-minor-end-behaviour-unfamiliar", "unf-1-end"),
    ] {
        let loaded = puppet(name);
        assert_eq!(
            bytes(&fixture(name)),
            bytes(&save_to(&loaded, label)),
            "{name} is written back verbatim"
        );
    }
}

/// The same two tokens at the **current** version are invalid, which is the
/// gate the whole unfamiliar-value rule hangs on.
///
/// A writer at this version had the whole list and used something else, so a
/// typo and an addition are indistinguishable — the same argument that makes
/// unmarked data outside `x` invalid. The rule is the one `VAL-7` and `VAL-17`
/// already name for the polarity pair.
#[test]
fn an_unfamiliar_mode_or_ending_is_invalid_at_the_current_version() {
    assert_rejected_naming(
        "rigged:interpolation-value-unfamiliar",
        Rule::EnumeratedValueKnown,
    );
    assert_rejected_naming(
        "rigged:end-behaviour-value-unfamiliar",
        Rule::EnumeratedValueKnown,
    );
}

/// A mode on a track's **last** keyframe names a segment that does not exist.
///
/// The storage rule and the defect it makes expressible: the mode belongs to
/// the earlier keyframe of a segment, and the last keyframe opens none.
#[test]
fn a_mode_on_the_last_keyframe_opens_no_segment_and_is_refused() {
    assert_refused("rigged:interpolation-on-last-keyframe");
}

// ------------------------------------------------ order and the mappable flag

/// ORD-1, the format half — **a parameter's presentation order is stored, and
/// resolves nothing.**
///
/// The rigger's order round-trips; the collection itself stays in **identifier
/// order**, because a canonical sort by the arrangement would erase it and a
/// position would start deciding something (**I3**); and reordering changes
/// **no identifier and no reference**. *Live's mapping list presents the
/// parameters in it* is **Live's**, and Live does not exist.
///
/// The wrong build presents in identifier order, which is the order nobody
/// chose — and the one this test's *differs from identifier order* assertion
/// exists to keep distinguishable.
#[test]
fn ord_1_the_parameter_order_is_stored_round_trips_and_resolves_nothing() {
    let subject = puppet("rigged:ordered-parameters");

    let stored: Vec<(u64, Option<i64>)> = subject
        .parameters
        .iter()
        .map(|p| (p.id.0, p.order))
        .collect();
    assert!(
        stored.iter().any(|(_, order)| order.is_some()),
        "the fixture carries a rigger's order"
    );
    assert!(
        stored.iter().any(|(_, order)| order.is_none()),
        "and one parameter with none, which is legal and sorts last"
    );
    assert!(
        subject.parameters.windows(2).all(|w| w[0].id < w[1].id),
        "the collection itself stays in identifier order, whatever the arrangement"
    );

    // The presentation order: by the stated order, absent last, ties by
    // identifier. It differs from identifier order, or the fixture could not
    // tell a build presenting in identifier order from a correct one.
    let mut presented: Vec<(bool, Option<i64>, u64)> = subject
        .parameters
        .iter()
        .map(|p| (p.order.is_none(), p.order, p.id.0))
        .collect();
    presented.sort();
    let presented: Vec<u64> = presented.into_iter().map(|(_, _, id)| id).collect();
    let by_identifier: Vec<u64> = subject.parameters.iter().map(|p| p.id.0).collect();
    assert_ne!(
        presented, by_identifier,
        "the rigger's arrangement is not identifier order"
    );

    let after = round_trip("rigged:ordered-parameters", "ord-1");
    assert_eq!(
        after
            .parameters
            .iter()
            .map(|p| (p.id.0, p.order))
            .collect::<Vec<_>>(),
        stored,
        "the order round-trips, on the parameter it was set on"
    );
    assert_eq!(
        bytes(&fixture("rigged:ordered-parameters")),
        bytes(&save_to(&subject, "ord-1-bytes")),
        "byte-stably"
    );

    // Reordering changes no identifier and no reference. ID-3 is the broader
    // half; this is the half that belongs to the arrangement.
    let mut reordered = subject.clone();
    let highest = reordered
        .parameters
        .iter()
        .filter_map(|p| p.order)
        .max()
        .expect("an order");
    for parameter in &mut reordered.parameters {
        parameter.order = parameter.order.map(|order| highest - order);
    }
    let written = save_to(&reordered, "ord-1-reordered");
    let reloaded = load(&written).expect("a reordered puppet loads").puppet;
    assert_eq!(
        identifiers(&subject),
        identifiers(&reloaded),
        "reordering changes no identifier"
    );
    assert_eq!(
        references(&subject),
        references(&reloaded),
        "and no reference — nothing resolves through the order"
    );
}

/// MAP-1, the format half — **`mappable: false` hides, and does not forbid.**
///
/// A parameter declared not mappable **and carrying no channel mapping**
/// validates: the flag is presentation, never a validation rule. *It is absent
/// from the targets Live offers* is **Live's**.
///
/// The wrong build rejects the file, or treats the flag as a rule — after which
/// a rigger hiding a parameter from a streamer's list has made the puppet
/// invalid.
#[test]
fn map_1_a_parameter_declared_not_mappable_validates() {
    let path = fixture("rigged:ordered-parameters");
    assert!(
        validate_file(&path).is_ok(),
        "the standalone validator accepts a parameter declared not mappable"
    );
    let subject = load(&path).expect("and so does the loader").puppet;

    let mapped: BTreeSet<Id> = subject
        .channel_mappings
        .iter()
        .map(|m| m.parameter)
        .collect();
    let hidden_and_unmapped = subject
        .parameters
        .iter()
        .find(|p| !p.mappable && !mapped.contains(&p.id))
        .expect("the fixture carries a parameter not mappable and carrying no mapping");
    assert!(
        !hidden_and_unmapped.mappable,
        "the flag is false as the file declares it"
    );
    assert!(validate(&subject).is_ok(), "and the puppet is valid");

    // Absent means offered, which is the direction that fails safely: an older
    // or third-party puppet hides nothing.
    let base = puppet("rigged");
    assert!(
        base.parameters.iter().all(|p| p.mappable),
        "a puppet declaring nothing offers every parameter"
    );
    let base_doc = puppet_json(&fixture("rigged"));
    for parameter in base_doc["parameters"].as_array().expect("parameters") {
        assert!(
            parameter.get("mappable").is_none(),
            "and stores nothing — the flag is absent when true"
        );
    }

    // Setting or clearing the flag on any parameter is never what makes a
    // puppet valid or invalid.
    let mut flipped = subject.clone();
    for parameter in &mut flipped.parameters {
        parameter.mappable = !parameter.mappable;
    }
    assert!(
        validate(&flipped).is_ok(),
        "the flag is not a validation rule in either direction"
    );
}

/// MAP-3 — **the derived value is never written back.**
///
/// A parameter declaring `mappable: false` that *does* carry a channel mapping
/// is offered by Live anyway (`MAP-2`, **Live's**), and that derived `true`
/// must never reach the file: the moment the mapping is removed, the rigger's
/// intent is what has to be left. So the flag stays `false` through a save, and
/// stays `false` through a save with the mapping removed.
#[test]
fn map_3_the_derived_mappable_value_is_never_written_back() {
    let subject = puppet("rigged:ordered-parameters");
    let mapped = mapped_parameter(&subject);
    assert!(
        !mapped.mappable,
        "the fixture's mapped parameter declares `mappable: false`, or this asserts nothing"
    );
    let mapped_id = mapped.id;

    let stored_flag = |path: &std::path::Path, id: Id| -> Option<bool> {
        puppet_json(path)["parameters"]
            .as_array()
            .expect("a parameters array")
            .iter()
            .find(|p| p["id"].as_u64() == Some(id.0))
            .expect("the parameter is in the file")
            .get("mappable")
            .map(|v| v.as_bool().expect("a boolean"))
    };

    let written = save_to(&subject, "map-3");
    assert_eq!(
        stored_flag(&written, mapped_id),
        Some(false),
        "a save leaves the stored flag false, not the derived true"
    );

    let mut without_mapping = subject.clone();
    without_mapping
        .channel_mappings
        .retain(|m| m.parameter != mapped_id);
    let written = save_to(&without_mapping, "map-3-unmapped");
    assert_eq!(
        stored_flag(&written, mapped_id),
        Some(false),
        "and removing the mapping leaves the rigger's intent where it was"
    );
    assert!(
        !load(&written)
            .expect("it loads")
            .puppet
            .parameters
            .iter()
            .find(|p| p.id == mapped_id)
            .expect("the parameter")
            .mappable,
        "so the parameter is hidden again, rather than silently offered for ever"
    );
}

/// MAP-4 — **the flag is not inferred from animation.**
///
/// A parameter both mapped and animated keeps the flag it was given, and
/// setting the flag changes nothing about whether it may be animated. Under
/// additive composition a parameter may legitimately be both, so deriving the
/// flag from *is it animated* answers the wrong question.
#[test]
fn map_4_a_parameter_both_mapped_and_animated_keeps_the_flag_it_was_given() {
    let subject = puppet("rigged:ordered-parameters");
    let mapped = mapped_parameter(&subject);
    let id = mapped.id;
    assert!(
        subject
            .animations
            .iter()
            .flat_map(|a| &a.tracks)
            .any(|t| t.parameter == id),
        "the fixture's mapped parameter is also animated, or this asserts nothing"
    );
    assert!(!mapped.mappable, "and declares `mappable: false`");

    let after = round_trip("rigged:ordered-parameters", "map-4");
    let kept = after
        .parameters
        .iter()
        .find(|p| p.id == id)
        .expect("the parameter");
    assert!(
        !kept.mappable,
        "being mapped and animated at once does not change the flag it was given"
    );

    // And the flag decides nothing about whether a parameter may be animated.
    let mut offered = subject.clone();
    offered
        .parameters
        .iter_mut()
        .find(|p| p.id == id)
        .expect("the parameter")
        .mappable = true;
    assert!(
        validate(&offered).is_ok(),
        "setting the flag leaves the track exactly where it was"
    );
    let reloaded = load(&save_to(&offered, "map-4-offered"))
        .expect("it loads")
        .puppet;
    assert_eq!(
        reloaded
            .animations
            .iter()
            .flat_map(|a| &a.tracks)
            .filter(|t| t.parameter == id)
            .count(),
        1,
        "the parameter is animated whatever the flag says"
    );
}

/// A parameter's `order` and `mappable` flag are **presentation**, and change
/// nothing else about what is stored.
///
/// The `ux` concern entry *a parameter's order and `mappable` flag never change
/// what the puppet evaluates to*, in the half a format crate can answer:
/// shuffle every order, flip every flag, and assert that what the file carries
/// beyond those two members is identical. The evaluated half is **S4's**, and
/// needs an evaluator.
///
/// Also the `data` entry *presentation data is stored as a field, never as a
/// collection's order* — asserted at `ORD-1` by the collection staying in
/// identifier order.
#[test]
fn presentation_state_does_not_leak_into_what_is_stored() {
    let subject = puppet("rigged:ordered-parameters");

    let mut shuffled = subject.clone();
    let count = shuffled.parameters.len() as i64;
    for (at, parameter) in shuffled.parameters.iter_mut().enumerate() {
        parameter.order = Some(count - at as i64);
        parameter.mappable = !parameter.mappable;
    }

    let before = puppet_json(&save_to(&subject, "presentation-before"));
    let after = puppet_json(&save_to(&shuffled, "presentation-after"));

    let strip = |doc: &serde_json::Value| -> serde_json::Value {
        let mut doc = doc.clone();
        for parameter in doc["parameters"].as_array_mut().expect("parameters") {
            let object = parameter.as_object_mut().expect("an object");
            object.remove("order");
            object.remove("mappable");
        }
        doc
    };
    assert_eq!(
        strip(&before),
        strip(&after),
        "an arrangement and a flag change themselves and nothing else"
    );
}

// ----------------------------------------- what the writer canonicalises away

/// The reader accepts more than the writer emits, for each of the new fields.
///
/// Carries the `data` concern entry on the canonical writer binding no reader:
/// an explicit `linear`, an explicit `mappable: true` and an unsorted
/// `triggers` list are each **accepted**, and each is written back in the one
/// canonical form. The first two name puppets indistinguishable from `rigged`,
/// so the bytes a save produces are `rigged`'s own — which is the sharpest form
/// the claim takes.
#[test]
fn the_writer_canonicalises_what_the_reader_accepts() {
    for name in [
        "rigged:interpolation-linear-explicit",
        "rigged:mappable-true-explicit",
    ] {
        let path = fixture(name);
        assert!(validate_file(&path).is_ok(), "{name} is accepted");
        assert_eq!(
            bytes(&fixture("rigged")),
            bytes(&save_to(&puppet(name), name)),
            "{name} is the base puppet spelled out, so it saves to the base puppet's bytes"
        );
    }

    let path = fixture("rigged:triggers-unsorted");
    assert!(
        validate_file(&path).is_ok(),
        "an unsorted triggers list is accepted — ordering binds the writer"
    );
    let written = save_to(&puppet("rigged:triggers-unsorted"), "triggers-unsorted");
    let events: Vec<String> = json_triggers(&puppet_json(&written))
        .iter()
        .map(|t| t["event"].as_str().expect("an event name").to_owned())
        .collect();
    let mut sorted = events.clone();
    sorted.sort();
    assert_eq!(
        events, sorted,
        "and an animation's triggers are written ordered by event name"
    );
}

// --------------------------------------------------------- the two new limits

/// The one row in `limits.rs` whose figure is this one, or a failure naming
/// what was looked for.
///
/// The **kind's name** is deliberately not asserted: naming it here would be
/// choosing it for the implementation. What this feature settled is the
/// **figure** and what it bounds, and those are what this pins — so task 3 is
/// free to name the kinds and is not free to move the figures unnoticed.
fn figure_for(limit: u64, what: &str) -> LimitKind {
    let rows: Vec<&rigel_format::Figure> =
        FIGURES.iter().filter(|f| f.limit == Some(limit)).collect();
    assert_eq!(
        rows.len(),
        1,
        "exactly one row in the one place limits are declared carries the {what} \
         limit of {limit} (LIM-1)"
    );
    let row = rows[0];
    assert!(
        row.note.contains(what),
        "{:?}'s note must say it bounds {what}, or the figure is a coincidence: {}",
        row.kind,
        row.note
    );
    row.kind
}

/// LIM-1 and LIM-8 for the **aggregate keyframe limit** — declared in the one
/// place, and exercised by a fixture just over it.
///
/// `LIM-9` is the criterion that requires the aggregate to exist at all; the
/// figure is `docs/format.md`'s, **262,144 over the whole puppet**, set from
/// memory and not from the workload. The fixture keeps every per-animation,
/// per-track and per-keyframe count inside its own limit, so a refusal naming
/// any other rule means the aggregate is not what refused it.
///
/// *Checked before allocating* is asserted here as a **bound on what the
/// refusal holds**, and deliberately not as a bound below the payload text.
///
/// A fixture one keyframe over an **aggregate** has to be read almost to its
/// end before any running count can exceed, so a reader that counts as it goes
/// and one that counts a finished collection differ at this fixture by a single
/// keyframe: on the order of the 458 bytes `docs/format.md` *Load limits*
/// charges a parsed keyframe, against the ~120 MB the 262,144 permitted ones
/// cost. **No peak threshold separates those two readers here** — that is a gap
/// on record, not something a tighter figure would reach. An assertion below
/// `DecompressedPayloadBytes` does not separate them either: what it separates
/// is a reader that builds a parse tree at all from one that pre-scans the
/// payload text, which `LimitKind::ParseTreeBytes` states in terms is not how
/// collection limits in the decode path work, and it would require a refused
/// load to hold a quarter of what the lawful file one keyframe smaller must
/// hold.
///
/// What is asserted is the property the limit exists for
/// (`docs/requirements/puppet.md`, *A limit stops a hostile file*): the refusal
/// stays inside the whole load's budget, and inside the budget that bounds the
/// structure this payload builds — the ceiling `docs/format.md` names as the
/// real constraint, and the one the aggregate was set below with more than
/// double the margin.
#[test]
fn lim_1_the_aggregate_keyframe_limit_is_declared_once_and_bites_before_allocating() {
    const AGGREGATE: u64 = 262_144;
    let kind = figure_for(AGGREGATE, "keyframe");
    let row = FIGURES.iter().find(|f| f.kind == kind).expect("the row");
    assert_eq!(
        row.basis,
        Basis::Memory,
        "the aggregate is set from memory, not as a multiple of a workload — \
         re-deriving it from the workload table is the error KeyframesPerTrack \
         already carries a note about"
    );

    let total = declared_limit(LimitKind::TotalLoadMemory).expect("a figure");
    let tree = declared_limit(LimitKind::ParseTreeBytes).expect("a figure");
    let mut outcome = None;
    let peak = peak_bytes(|| outcome = Some(load(&fixture("rigged:keyframes-over-aggregate"))));
    assert!(
        peak < total,
        "a file refused by the aggregate held {peak} bytes, which is not below the \
         whole load's budget of {total} — the aggregate exists so that a file \
         traded between strangers cannot exhaust memory"
    );
    assert!(
        peak < tree,
        "the aggregate sits below the parse-tree budget of {tree} with more than \
         double the margin, so a load refused by it must still be inside that \
         budget; it held {peak}"
    );

    match outcome
        .expect("the load ran")
        .expect_err("one over is refused")
    {
        Rejection::LimitExceeded {
            limit, declared, ..
        } => {
            assert_eq!(
                limit, kind,
                "a puppet one keyframe over the aggregate is refused by the aggregate, \
                 not by a neighbouring limit"
            );
            assert_eq!(
                declared, AGGREGATE,
                "the figure reported is the one declared"
            );
        }
        other => panic!("refused as {other:?} rather than as a limit"),
    }
}

/// LIM-8 for an animation's **triggers** — a new stored collection, so it needs
/// a declared limit, and `LIM-1` then needs a fixture just over it.
///
/// The figure is 64, provisional in `limits.rs`'s sense: no `stress` dimension
/// counts triggers and none will, because an animation waits on a handful of
/// scene events. It must match the figure `rigged:triggers-over-limit` was
/// built against, or that fixture is just over nothing.
#[test]
fn lim_8_an_animations_triggers_are_a_collection_with_a_declared_limit() {
    const TRIGGERS: u64 = 64;
    let kind = figure_for(TRIGGERS, "trigger");

    let mut outcome = None;
    let _ = peak_bytes(|| outcome = Some(load(&fixture("rigged:triggers-over-limit"))));
    match outcome
        .expect("the load ran")
        .expect_err("one over is refused")
    {
        Rejection::LimitExceeded {
            limit, declared, ..
        } => {
            assert_eq!(
                limit, kind,
                "refused by the limit on an animation's triggers"
            );
            assert_eq!(
                declared, TRIGGERS,
                "the figure reported is the one declared"
            );
        }
        other => panic!("refused as {other:?} rather than as a limit"),
    }

    // And the accepted fixtures stay well inside it, or the limit is refusing
    // the corpus rather than a hostile file.
    let subject = puppet("rigged:triggered");
    assert!(
        triggered_animation(&subject).triggers.len() < TRIGGERS as usize,
        "an authored animation waits on a handful of events"
    );
}

/// The `security` concern entry *an aggregate limit exists wherever per-item
/// limits multiply*.
///
/// Keyframes are the generating case: the animation, track and keyframe figures
/// multiply into hundreds of millions, bounded only by the parse-tree cap. This
/// asserts the condition that makes an aggregate necessary **and** that one
/// exists below the product — so a later build raising a per-item figure
/// without revisiting the aggregate fails here rather than quietly permitting
/// the product again.
#[test]
fn an_aggregate_limit_exists_where_the_per_item_limits_multiply() {
    let animations = declared_limit(LimitKind::AnimationCount).expect("a figure");
    let tracks = declared_limit(LimitKind::TracksPerAnimation).expect("a figure");
    let keyframes = declared_limit(LimitKind::KeyframesPerTrack).expect("a figure");
    let product = animations
        .checked_mul(tracks)
        .and_then(|n| n.checked_mul(keyframes))
        .expect("the product is representable");

    let aggregate = declared_limit(figure_for(262_144, "keyframe")).expect("a figure");
    assert!(
        aggregate < product,
        "the per-item limits multiply to {product} keyframes, so an aggregate below \
         that is what actually bounds a load; this one is {aggregate}"
    );
    assert!(
        aggregate >= keyframes,
        "and it admits a single track at its own full figure, or the per-track limit \
         could never be reached and one of the two is wrong"
    );
}

/// A keyframe has nowhere to carry a composition mode, and a track has no field
/// saying it composes additively.
///
/// `docs/format.md` is explicit: a track composes additively and carries **no
/// field that says so**. A replacing mode, if one is ever needed, arrives later
/// with absent meaning additive — so a field written today would be the thing
/// that stops that working.
#[test]
fn a_track_carries_no_field_saying_how_it_composes() {
    const TRACK_KEYS: &[&str] = &["id", "parameter", "blend_duration", "keyframes", "x"];
    const KEYFRAME_KEYS: &[&str] = &["time", "value", "interpolation", "control_points", "x"];

    for name in ["rigged", "rigged:mixed-interpolation", "rigged:endings"] {
        let doc = puppet_json(&fixture(name));
        for animation in doc["animations"].as_array().expect("animations") {
            for track in animation["tracks"].as_array().expect("tracks") {
                for key in track.as_object().expect("an object").keys() {
                    assert!(
                        TRACK_KEYS.contains(&key.as_str()),
                        "{name}: a track carries {key:?}, which the format does not define for it"
                    );
                }
            }
        }
        for keyframe in json_keyframes(&doc) {
            for key in keyframe.as_object().expect("an object").keys() {
                assert!(
                    KEYFRAME_KEYS.contains(&key.as_str()),
                    "{name}: a keyframe carries {key:?}, which the format does not define \
                     for it"
                );
            }
        }
    }

    // The same claim on the surface a test would reach for: a keyframe is a
    // time, a value, the mode of the segment it opens, and that segment's
    // control points.
    let _ = |k: Keyframe| (k.time, k.value, k.interpolation, k.control_points);
}
