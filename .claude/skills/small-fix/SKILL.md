---
name: small-fix
description: The explicit bypass lane for work too small for the full feature process - a one-line fix, a copy change, a dependency bump. Reached from the workflow skill. Defines the threshold so that skipping the process is a decision rather than a habit.
---

# Small fix

This lane exists so that bypassing the process is **a decision with a stated threshold**, not a
habit. The full process on a one-line fix is absurd, so it would be skipped anyway — and once
skipping is normal it happens to things that needed the process.

## The threshold

All of these must hold. If any one fails, it is a `feature`.

- **One concern.** A single defect or a single small change, not a cluster.
- **No new behaviour.** Fixing what was specified, not adding to it.
- **No decision.** Nothing touching an invariant, a `[proposed]` or `[open]` section, a
  contract, a data shape, or a term in `docs/vocabulary.md`.
- **No new dependency** and no configuration change that affects how the project runs.
- **Small enough to review in one pass**, and reviewable from the diff alone.
- **A test can express it**, or no test is needed because nothing observable changed.

**If you are arguing with the threshold, you are over it.** The argument costs more than the
lane it would have saved.

## The lane

1. **Branch.** `fix/<slug>`. Not the default branch — the pre-commit hook will refuse.
2. **Write the failing test first** where anything observable changed. A fix with no test is a
   fix that comes back.
3. **Fix it.**
4. **Run the checks.** Formatter, linter, type checker, suite, via `scripts/test-record.sh`.
5. **Check the documentation.** Only this: did this change make something a document says
   false? If yes, fix the document in the same commit. If the answer needs the whole doc set to
   determine, the threshold has failed — this is a `feature`.
6. **Commit, push, PR.** One commit, unless the test and the fix genuinely want separating.

No feature document. No assignment. No handoff. That is the point of the lane.

## What still applies

- **Review** if the fix touches anything with a failure mode worth the dispatch —
  authentication, money, data migration, concurrency. Judgment, but err toward dispatching
  `code-reviewer`: a small diff is a cheap review.
- **A gate** if a person should look at the result.
- **Harvesting.** If this fix is for something a human caught, it still becomes a checklist
  entry in `docs/concerns/`. A bug found twice is a process failure, and the lane does not
  exempt it. Add it now — there is no `finish-cycle` on this lane to harvest it later.

## When it turns out to be bigger

Stop. Say so. Branch properly and route to `feature`.

Discovering mid-fix that something is larger than it looked is the normal case, not a failure of
judgment. Continuing anyway is the failure — the work lands without an assignment, without
acceptance criteria, and without anything to review it against.
