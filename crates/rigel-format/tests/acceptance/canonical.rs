//! Section L — the same puppet saves to the same bytes.
//!
//! `CANON-4`, `CANON-5` and `CANON-6` are out of scope here: the first two are
//! Studio's and Live's, the third is a property test over an operation set and
//! an undo stack that do not exist. `docs/wip/s2-format.md` assigns them.

use std::path::Path;

use rigel_format::{BindingValues, load, save};

use crate::support::{bytes, entries, fixture, looks_like_png, png_dimensions, scratch};

/// CANON-1 — load, save, load, save: the two serialisations are identical.
#[test]
fn canon_1_a_round_trip_is_byte_stable() {
    let dir = scratch("canon-1");
    let a = dir.join("a.rigel2d");
    let b = dir.join("b.rigel2d");

    let first = load(&fixture("rigged")).expect("it loads").puppet;
    save(&first, &a).expect("it saves");
    let second = load(&a).expect("what we wrote loads").puppet;
    save(&second, &b).expect("it saves again");

    assert_eq!(bytes(&a), bytes(&b), "A and B must be byte-identical");
}

/// Saving an unmodified puppet is byte-identical to the file it came from.
///
/// The `data` concern entry of that name, and the sharpest form of the canonical
/// rule available here: `rigged` was authored by a builder that uses **no project
/// code**, from `docs/format.md`, so a difference is our writer disagreeing with
/// the specification rather than with itself. `CANON-1` would pass against a
/// writer that is merely self-consistent.
#[test]
fn saving_an_unmodified_puppet_reproduces_the_file_it_came_from() {
    for name in ["rigged", "one-part", "rigged:deep", "rigged:orphan-texture"] {
        let source = fixture(name);
        let dir = scratch(&format!("canon-identity-{name}"));
        let written = dir.join("saved.rigel2d");
        let subject = load(&source).expect("it loads").puppet;
        save(&subject, &written).expect("it saves");
        assert_eq!(
            bytes(&source),
            bytes(&written),
            "{name}: a save of an unmodified puppet must reproduce the file"
        );
    }
}

/// CANON-2 — nothing about the machine that wrote the file reaches it.
///
/// Username, hostname, working directory, destination path, locale, time zone,
/// and the clock advanced between saves. The last two are the ones a zip writer
/// leaks by default, which is why `docs/format.md` pins modification times to
/// 1980-01-01.
/// The name of this test, which it needs in order to re-run itself.
const CANON_2: &str = "canonical::canon_2_the_saved_bytes_carry_nothing_about_this_machine";

/// Set on the child, naming where it must write. Its presence is what tells
/// this test it is the child rather than the parent.
const CANON_2_CHILD: &str = "RIGEL_ACCEPTANCE_CANON_2_TARGET";

#[test]
fn canon_2_the_saved_bytes_carry_nothing_about_this_machine() {
    // The child: one save, into the path it was given, and nothing else.
    if let Ok(target) = std::env::var(CANON_2_CHILD) {
        let subject = load(&fixture("rigged")).expect("it loads").puppet;
        save(&subject, Path::new(&target)).expect("it saves");
        return;
    }

    let home = scratch("canon-2");
    let first_dir = home.join("one");
    let second_dir = home.join("a-rather-longer-directory-name");
    std::fs::create_dir_all(&first_dir).expect("a directory");
    std::fs::create_dir_all(&second_dir).expect("a directory");
    let first = first_dir.join("first.rigel2d");
    let second = second_dir.join("second-with-a-longer-name.rigel2d");

    let marker_user = "acceptance-user-f1f1";
    let marker_host = "acceptance-host-9c9c";

    save_as_child(
        &first,
        &first_dir,
        &[
            ("USER", marker_user),
            ("USERNAME", marker_user),
            ("LOGNAME", marker_user),
            ("HOSTNAME", marker_host),
            ("TZ", "Pacific/Kiritimati"),
            ("LANG", "tr_TR.UTF-8"),
            ("LC_ALL", "tr_TR.UTF-8"),
        ],
    );
    // The clock advances between the two saves, so a modification time taken
    // from it would differ.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    save_as_child(
        &second,
        &second_dir,
        &[
            ("USER", "someone-else"),
            ("USERNAME", "someone-else"),
            ("LOGNAME", "someone-else"),
            ("HOSTNAME", "another-host"),
            ("TZ", "Europe/Lisbon"),
            ("LANG", "C"),
            ("LC_ALL", "C"),
        ],
    );

    let first_bytes = bytes(&first);
    assert_eq!(
        first_bytes,
        bytes(&second),
        "the same puppet saves to the same bytes whatever the machine says"
    );
    for leak in [
        marker_user,
        marker_host,
        "someone-else",
        "another-host",
        "first.rigel2d",
        "second-with-a-longer-name.rigel2d",
    ] {
        assert!(
            !contains(&first_bytes, leak.as_bytes()),
            "the saved bytes carry {leak:?}"
        );
    }
    assert!(
        !contains(&first_bytes, first_dir.to_string_lossy().as_bytes()),
        "the saved bytes carry a path they were saved to"
    );
}

/// Runs this same test binary again, as a child, to perform one save under a
/// different username, hostname, locale, time zone and working directory.
///
/// A child rather than `std::env::set_var` and `set_current_dir`, which are
/// process-global: mutating the environment of a running test binary is a data
/// race against every other test in it, and changing the working directory
/// changes it for all of them. The environment *is* the subject of this
/// criterion, so it has to vary — in a process of its own.
fn save_as_child(target: &Path, working_directory: &Path, environment: &[(&str, &str)]) {
    let mut command = std::process::Command::new(
        std::env::current_exe().expect("the test binary knows where it is"),
    );
    command
        .args(["--exact", CANON_2, "--test-threads=1", "--nocapture"])
        .current_dir(working_directory)
        .env(CANON_2_CHILD, target);
    for (key, value) in environment {
        command.env(key, value);
    }
    let run = command.output().expect("the child runs");
    assert!(
        run.status.success(),
        "the save in the child failed:\n{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    assert!(target.is_file(), "the child wrote {target:?}");
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// CANON-3 — ordering is the build's rule, not the input's.
///
/// The same entities with the same identifiers, assembled in two different
/// orders, save to the same bytes.
#[test]
fn canon_3_input_order_does_not_reach_the_file() {
    let dir = scratch("canon-3");
    let forwards = dir.join("forwards.rigel2d");
    let backwards = dir.join("backwards.rigel2d");

    let subject = load(&fixture("rigged")).expect("it loads").puppet;
    save(&subject, &forwards).expect("it saves");

    let mut permuted = subject.clone();
    // `nodes` is regrouped, not reversed. `docs/format.md` makes order among
    // siblings *draw order* — "the one thing order is allowed to decide" — and
    // `ID-3` asserts that reversing sibling order **does** reach the file, as
    // that criterion's whole point. A reversal here would assert the opposite,
    // and the two tests could never both pass whatever the writer did.
    //
    // `CANON-3` is about the order the same entities were *added* in, not about
    // sibling order: "ordering is the build's rule, not the input's". So this
    // walks the same tree grouped by parent instead of in depth-first preorder —
    // a different array, every parent's children in the same relative order, the
    // same puppet. Canonicalisation has to put it back.
    permuted.nodes = {
        let original = permuted.nodes.clone();
        let mut parents = Vec::new();
        for node in &original {
            if !parents.contains(&node.parent) {
                parents.push(node.parent);
            }
        }
        parents.reverse();
        let mut regrouped = Vec::with_capacity(original.len());
        for parent in parents {
            regrouped.extend(original.iter().filter(|n| n.parent == parent).cloned());
        }
        assert_eq!(regrouped.len(), original.len(), "every node is still here");
        assert_ne!(
            regrouped.iter().map(|n| n.id).collect::<Vec<_>>(),
            original.iter().map(|n| n.id).collect::<Vec<_>>(),
            "and the array order really did change"
        );
        regrouped
    };
    permuted.parameters.reverse();
    permuted.bindings.reverse();
    permuted.textures.reverse();
    permuted.channel_mappings.reverse();
    for animation in &mut permuted.animations {
        animation.tracks.reverse();
    }
    for parameter in &mut permuted.parameters {
        parameter.keypoints.reverse();
        for binding in &mut permuted.bindings {
            if binding.parameter != parameter.id {
                continue;
            }
            match &mut binding.values {
                BindingValues::Transform(values) => values.reverse(),
                BindingValues::Deformation(values) => values.reverse(),
            }
        }
    }
    save(&permuted, &backwards).expect("it saves");

    assert_eq!(
        bytes(&forwards),
        bytes(&backwards),
        "ordering is the build's rule, not the input's"
    );
}

/// CANON-7, the half that is this subsystem's — texture payloads are lossless
/// in storage.
///
/// The criterion's own form compares a decoded payload against **the imported
/// artwork**, which needs the import path and is Studio's. What the format owns
/// is that a payload is carried through a round trip without being re-encoded,
/// and that what is stored is the content rectangle the record declares.
#[test]
fn canon_7_texture_payloads_are_carried_losslessly() {
    let source = fixture("rigged");
    let subject = load(&source).expect("it loads").puppet;
    let dir = scratch("canon-7");
    let written = dir.join("payloads.rigel2d");
    save(&subject, &written).expect("it saves");

    assert_payloads_match(&source, &written);

    for texture in &subject.textures {
        assert!(
            looks_like_png(&texture.payload),
            "every payload is a PNG, which is lossless and carries alpha"
        );
        let (width, height) = png_dimensions(&texture.payload);
        assert_eq!(
            (width, height),
            (texture.crop[2], texture.crop[3]),
            "what is stored is the content rectangle the record declares"
        );
    }
}

fn assert_payloads_match(before: &Path, after: &Path) {
    let arrived = entries(before);
    let written = entries(after);
    for (name, payload) in arrived.iter().filter(|(n, _)| n.starts_with("textures/")) {
        assert_eq!(
            Some(payload),
            written.get(name),
            "{name} was re-encoded rather than carried"
        );
    }
}
