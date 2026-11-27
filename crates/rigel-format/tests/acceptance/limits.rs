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

use rigel_format::{ArchiveFault, LimitKind, Rejection, declared_limit, load};

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

/// LIM-3 — a declared size within the limit expanding far beyond it fails the
/// load, and fails it as a limit rather than as anything else.
///
/// *Checked before allocating* means the expansion was never produced: the limit
/// is enforced against bytes **actually decompressed**, and the entry's
/// declaration — which is attacker-controlled — is used only to refuse early. A
/// loader that trusts the header has a decompression bomb.
///
/// The criterion also asks that **peak resident bytes during the attempt stay
/// below the declared limit**. That needs a counting allocator the acceptance
/// suite does not have; it is reported as the half this test does not carry.
#[test]
fn lim_3_a_decompression_bomb_fails_as_a_limit() {
    let rejection = load(&fixture("rigged:decompression-bomb"))
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
#[test]
fn hostile_containers_are_refused_as_container_faults() {
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
        let rejection = load(&fixture(name))
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
