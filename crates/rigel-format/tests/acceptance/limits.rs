//! Section J — load limits.
//!
//! A **load limit** is not a ceiling and not an authoring limit: it is a defence
//! against a corrupt or hostile file, enforced **before allocating**.
//!
//! # What is not here, and why
//!
//! `LIM-2`, `LIM-6` and `LIM-7` all want a fixture **just over** a declared
//! limit, and `LIM-1`'s second half wants one per limit. `LIM-4` fixes every
//! figure at four times the corresponding `stress` dimension, and the `stress`
//! workload is S3's — so no limit figure exists to build a fixture against, and
//! one built against a guessed figure would be rebuilt anyway. `LIM-4` itself is
//! out of scope for the same reason. Recorded under *Deferred* in
//! `docs/wip/s2-format.md`.
//!
//! `LIM-5` is `judgment` — *no load limit appears as a choice presented to the
//! person rigging* is read at review and at Studio's interface gate. `LIM-8` is
//! `script`: it walks the format definition and fails on any collection or
//! string with no limit, which needs the format definition to exist as something
//! walkable.
//!
//! # The memory half, which task 8 added
//!
//! `LIM-2`, `LIM-3` and `LIM-7` each ask that **peak resident bytes during a
//! failed load stay below the declared payload limit**. That is what *checked
//! before allocating* means in practice, and no assertion over a returned
//! [`Rejection`] reaches it: a loader that inflates the expansion and then
//! reports a limit returns exactly the same value as one that never inflated it.
//! The counting allocator in `tests/shared/counting_allocator.rs` is what tells
//! the two apart, and the assertions below use it.
//!
//! **What still has no fixture** is `LIM-2`'s and `LIM-7`'s *just over* corpus —
//! a texture past the dimension limit nothing references, and material past the
//! payload limit inside a later-version file's opaquely preserved region. Those
//! need fixtures the corpus does not carry and tests `acceptance-planner` writes;
//! the figures they were blocked on now exist. Reported in
//! `docs/wip/s2-format.md`.

use rigel_format::{ArchiveFault, LimitKind, Rejection, declared_limit, load};

use crate::counting::peak_bytes;
use crate::support::fixture;

/// LIM-1, the half that can be written — every load limit is declared in **one
/// place**, and every kind the loader trusts resolves to a figure there.
///
/// The other half — *and every declared limit has a fixture just over it* — is
/// blocked on S3, as the module comment says. `LIM-8`'s *every stored collection
/// and string has a declared limit* is the same slice read the other way: a
/// collection added to the format is added to [`LimitKind::ALL`], and this fails
/// until it has a figure.
#[test]
fn lim_1_every_load_limit_is_declared_in_one_place() {
    assert!(
        !LimitKind::ALL.is_empty(),
        "the loader trusts sizes and counts, so there are limits to declare"
    );
    for kind in LimitKind::ALL {
        let figure = declared_limit(*kind)
            .unwrap_or_else(|| panic!("{kind:?} is trusted by the loader and has no limit"));
        assert!(figure > 0, "{kind:?}'s limit must be a usable figure");
    }
}

/// A payload inside every byte limit whose **parsed tree** is not.
///
/// **Written at task 10, from a review finding, not from the criteria** — the
/// separation that keeps acceptance tests independent of the implementation
/// does not apply to it, and it is marked so that nobody mistakes it for a
/// criterion.
///
/// `LIM-3`'s own fixture is a decompression bomb: text that expands. This is
/// the shape underneath it. The text stays inside
/// `DecompressedPayloadBytes`, and the *tree* built from it does not, because
/// every value is a 32-byte enum and every number a separately allocated
/// string. Nothing downstream can refuse it: every collection limit in
/// `decode` runs once the tree already exists.
///
/// Measured before the limit was added: a 63 KB archive of this shape reached
/// 1.9 GiB resident, against a `TotalLoadMemory` of 1 GiB. The assertion that
/// matters is the peak, not the verdict — the verdict was always correct, and
/// arrived after the allocation.
#[test]
fn a_payload_whose_parsed_tree_exceeds_the_budget_is_refused_before_it_is_built() {
    let tree_limit = declared_limit(LimitKind::ParseTreeBytes).expect("a declared figure");
    let total = declared_limit(LimitKind::TotalLoadMemory).expect("a declared figure");
    let mut outcome = None;
    let peak = peak_bytes(|| outcome = Some(load(&fixture("rigged:parse-tree-bomb"))));
    assert!(
        peak < total,
        "the failed load held {peak} bytes, which is not below the whole load's          budget of {total}"
    );
    assert!(
        peak < 2 * tree_limit,
        "the tree limit is {tree_limit}; a load refused by it held {peak} bytes,          which is not within a doubling of the figure it was refused by"
    );

    let rejection = outcome
        .expect("the load ran")
        .expect_err("a payload whose tree exceeds the budget is refused");
    match rejection {
        Rejection::LimitExceeded {
            limit, declared, ..
        } => {
            assert_eq!(
                limit,
                LimitKind::ParseTreeBytes,
                "refused as the wrong limit"
            );
            assert_eq!(
                declared, tree_limit,
                "the figure reported is the one declared"
            );
        }
        other => panic!("refused as {other:?} rather than as a limit"),
    }
}

/// LIM-3 — a declared size within the limit expanding far beyond it fails the
/// load, and fails it as a limit rather than as anything else.
///
/// *Checked before allocating* means the expansion was never produced: the limit
/// is enforced against bytes **actually decompressed**, and the entry's
/// declaration — which is attacker-controlled — is used only to refuse early. A
/// loader that trusts the header has a decompression bomb.
///
/// The criterion also asks that **peak resident bytes during the attempt stay
/// below the declared limit**, which is the half the counting allocator added in
/// task 8 carries. It is the assertion that tells a loader which refused the
/// expansion apart from one which produced it and then complained.
#[test]
fn lim_3_a_decompression_bomb_fails_as_a_limit() {
    let payload_limit =
        declared_limit(LimitKind::DecompressedPayloadBytes).expect("a declared figure");
    let mut outcome = None;
    let peak = peak_bytes(|| outcome = Some(load(&fixture("rigged:decompression-bomb"))));
    assert!(
        peak < payload_limit,
        "the bomb expands to 256 MiB; the failed load held {peak} bytes, which is \
         not below the declared limit of {payload_limit}"
    );

    let rejection = outcome
        .expect("the load ran")
        .expect_err("an entry expanding past the limit is refused");
    match rejection {
        Rejection::LimitExceeded {
            limit, declared, ..
        } => {
            assert_eq!(
                limit,
                LimitKind::DecompressedPayloadBytes,
                "refused by the limit on bytes actually decompressed"
            );
            assert_eq!(
                Some(declared),
                declared_limit(LimitKind::DecompressedPayloadBytes),
                "the figure reported is the one declared in the one place"
            );
        }
        other => panic!("refused for the wrong reason: {other:?}"),
    }
}

/// The zip-level hostile fixtures are each refused, and refused as the container
/// fault they are.
///
/// Carries the `security` concern entry *hostile fixtures rejected cleanly*. Not
/// one of the numbered criteria: these fixtures exist because an archive brings
/// container faults a JSON file could not have, and each is refused **before
/// anything is decompressed**.
///
/// *Before anything is decompressed* is the clause task 8 made assertable:
/// every one of these refusals is read out of the central directory, so the
/// peak each costs is a fraction of the payload limit rather than near it. The
/// bound asserted is the same one `LIM-2`, `LIM-3` and `LIM-7` name.
#[test]
fn hostile_containers_are_refused_as_container_faults() {
    let payload_limit =
        declared_limit(LimitKind::DecompressedPayloadBytes).expect("a declared figure");
    let cases = [
        ("rigged:entry-name-overlong", ArchiveFault::EntryNameTooLong),
        (
            "rigged:duplicate-entry-names",
            ArchiveFault::DuplicateEntryName,
        ),
        (
            "rigged:overlapping-local-header-data",
            ArchiveFault::OverlappingEntryData,
        ),
        (
            "rigged:undefined-entry-name",
            ArchiveFault::UndefinedEntryName,
        ),
    ];
    for (name, expected) in cases {
        let mut outcome = None;
        let peak = peak_bytes(|| outcome = Some(load(&fixture(name))));
        assert!(
            peak < payload_limit,
            "{name} is refused from the central directory, so the failed load must \
             hold far less than {payload_limit} bytes; it held {peak}"
        );

        let rejection = outcome
            .expect("the load ran")
            .err()
            .unwrap_or_else(|| panic!("{name} must be refused"));
        match rejection {
            Rejection::Archive { fault, .. } => assert_eq!(
                fault, expected,
                "{name} must be refused as {expected:?}, not {fault:?}"
            ),
            Rejection::LimitExceeded { limit, .. }
                if expected == ArchiveFault::EntryNameTooLong =>
            {
                assert_eq!(
                    limit,
                    LimitKind::EntryNameBytes,
                    "an over-long entry name is either a container fault or its own limit"
                );
            }
            other => panic!("{name} was refused for the wrong reason: {other:?}"),
        }
    }
}
