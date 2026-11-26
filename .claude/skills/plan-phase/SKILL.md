---
name: plan-phase
description: Decomposes agreed design into an ordered phase plan in docs/wip/plan.md, and commissions the acceptance criteria. Reached from the workflow skill when design exists and no plan is in flight.
---

# Plan a phase

Produces `docs/wip/plan.md`: which subsystems get built, in what order where order matters.

A conversation before it is a file.

## 1. Decompose

Into subsystems that can each be built, reviewed and landed as a unit. The test for a good unit:
**an implementer could build it from an assignment plus a named slice of the docs**, without the
whole picture. That is not an aesthetic preference here — it is literally what `module-dev` will
be given.

Two failure modes, in opposite directions:

- **Too small.** Agent-drafted decompositions tend to over-slice and over-sequence, producing a
  chain of tasks each too small to be worth a dispatch and a brief. If two units would always be
  built together by one person in one sitting, they are one unit.
- **Too large.** A unit nobody can review in one pass. If you cannot state what it is when it is
  done, in a sentence, it is too large.

## 2. Order

Say what genuinely blocks what, and say what does not. False sequencing is as costly as missing
sequencing — it hides the work that could have been done in any order and makes a plan look more
determined than it is.

Prefer an order where something is verifiable end to end early. A phase that integrates only at
the end discovers its design problems only at the end.

## 3. Keep it thin

**Reference the design docs rather than restating them.** Anything technical worth writing is
almost certainly already in `docs/`, and a second copy is one that will drift. A line like
*"see `items.md` §6.4 for the access rule"* is right; a paragraph re-explaining the access rule
is not.

## 4. Commission the acceptance criteria

Once the plan is agreed, dispatch `acceptance-planner` with the requirements for the phase. It
writes criteria and the tests, before any implementation exists, and it must not be given sight
of implementation.

Its most useful output is usually the **gaps in the requirements** — specification defects found
before anyone builds against them. Route those back into `docs/requirements/` before starting.

## 5. Agree and commit

Agree the plan with the user, then commit `docs/wip/plan.md` to the default branch. It reaches
the default branch; feature documents never do.

## Done when

- Subsystems are listed, ordered, with dependencies stated.
- Acceptance criteria and tests exist and have been confirmed failing.
- Requirements gaps found by the planner are resolved.
- The user agrees. Committed.

Then route to `feature` for the first subsystem.
