//! Acceptance tests for `rigel-format`, subsystem S2.
//!
//! Written from `docs/requirements/puppet-acceptance.md` by `acceptance-planner`,
//! **without sight of an implementation** — there is none; `crates/rigel-format`
//! is a declared surface whose every body is `unimplemented!()`. These tests
//! therefore fail, and that is what confirms them: a criterion is confirmed by a
//! test that is written and fails for the stated reason
//! (`.claude/rules/testing.md`).
//!
//! **An acceptance test that seems wrong is a finding, not an obstacle**
//! (`docs/testing.md`). None of these may be edited to make an implementation
//! pass.
//!
//! # Reading this suite
//!
//! One module per group of the criteria document, and every test names the
//! criterion it implements in its name and in a comment above it. Where a
//! criterion has two halves and only one is this subsystem's — the stored shape
//! here, the evaluated behaviour in S4, the resident bytes in S5 — the test
//! carries the half it can and says in the comment where the other half lives.
//!
//! # This file exists so cargo runs the directory
//!
//! Cargo auto-discovers `tests/*.rs` and `tests/<dir>/main.rs` and nothing else.
//! A `tests/acceptance/` holding only test modules is **silently skipped**, and
//! a test directory nothing runs is worse than no test directory
//! (`docs/testing.md`).

mod support;

/// The counting allocator, shared with `tests/fuzz.rs`.
///
/// Added by task 8, which is where `LIM-3`'s *peak resident bytes during the
/// attempt stay below the declared limit* stops being a gap the module comment
/// in `limits.rs` records and becomes an assertion. A `#[global_allocator]` is a
/// property of a binary rather than of a library, so it is registered here and
/// `rigel-format` does not know it exists.
#[path = "../shared/counting_allocator.rs"]
mod counting;

#[global_allocator]
static ALLOCATOR: counting::Counting = counting::Counting;

mod canonical;
mod execution;
mod extensions;
mod identity;
mod limits;
mod stored_shape;
mod validation;
mod version;
