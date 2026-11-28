# Feature: the animation stored shape

**Branch:** `feature/s2b-animation-shape`
**Phase subsystem:** S2b in [plan.md](plan.md)

`rigel-format` reads, validates and writes the animation shape as it stood before the
requirements round of 2026-11-28. That round settled per-segment interpolation, animation end
behaviour, triggers, and a parameter's stored order and `mappable` flag, and
[format.md](../format.md) specifies all of it — but no code implements any of it. This feature
lands the **stored shape**: the fields, their canonical form, their validation, their limits and
their round-trip. It does not evaluate any of it.

Done when a puppet carrying every one of those fields round-trips byte-identically, each new
field's unfamiliar-value and defect cases are rejected or preserved as
[format.md](../format.md) specifies, the aggregate keyframe limit bites before allocating, and
the format half of [puppet-acceptance.md](../requirements/puppet-acceptance.md) section R passes.

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** not started
**Blocked on:** nothing

---

## Assignment

### Goal

Make `rigel-format` read, validate and write the animation and parameter fields
[format.md](../format.md) specifies, so that S3 can generate a workload and S4 can evaluate one
against a format that matches the design.

### Out of scope

**Everything that evaluates.** This feature stores and restores; it does not compute. In
particular:

- **No curve evaluation.** Nothing samples a track at a time, solves a Bézier, applies a
  stepped or inverse-stepped segment, or composes a track with a base driver. Section R's
  `ADD-*`, `CLAMP-*`, `INTERP-1`, `INTERP-5`, `END-1`, `END-2`, `END-3`, `LOST-1`, `LONG-1`,
  `LONG-2` and `MAP-2` are **S4's**, and the tests for them are not written here.
- **No trigger dispatch.** A trigger is stored, validated and written back. Nothing raises a
  scene event, matches one, or starts an animation. `LOST-4`'s *starts exactly the animation it
  lives on* half is S4's; its *a trigger carrying any field beyond an event name is rejected*
  half is here.
- **No presentation.** `ORD-1`'s *Live's mapping list presents the parameters in it* and
  `MAP-1`/`MAP-2`'s *the targets Live offers* are Live's, and Live does not exist.
- **No workload generator, no `reference`, no `stress`, no `long-animation`.** Those are S3.
  Fixtures built here are hand-authored, small, and exist only to exercise the stored shape.
- **No change to the keypoint/keyframe arity rule.** [format.md](../format.md) *Validation*
  marks it `[open]` and says **nothing may assume an answer**. The validator keeps zipping
  coordinates against axes. Do not close it, and do not work around it.
- **No texture pixel-area limit.** Deferred by the owner on 2026-11-28.

### Doc set

Read these and no others.

- `CLAUDE.md` — current state and invariants
- [vocabulary.md](../vocabulary.md) — the terms. **Load-bearing**. *Track*, *keyframe*,
  *parameter*, *animation* and *channel mapping* all have exact senses here
- [invariants.md](../invariants.md) — what must never break. **I1** and **I3** bind directly:
  a trigger names an event outward and must never become something evaluated, and nothing new
  here may be referenced by name, index or position
- [format.md](../format.md) — **the specification for this feature.** *An unfamiliar value in a
  field this build knows*, *Canonical serialisation, and where zip fights it*, *An animation and
  its tracks* with its three subsections, *Validation* and *Load limits*
- [puppet-acceptance.md](../requirements/puppet-acceptance.md) — *The fixtures*, and section R
- [requirements/puppet.md](../requirements/puppet.md) — *An animation adds to its parameter* and
  *Parameters are ordered, and declared mappable or not*, for the requirements the criteria trace
  to
- [concerns/security.md](../concerns/security.md) — the entries listed under *Concern criteria*
- [concerns/data.md](../concerns/data.md) — the extension-data and canonical-writer entries
- [testing.md](../testing.md) — the harness, and who writes which kind of test

### What is being added

Each row is specified in [format.md](../format.md); the rightmost column says where.

| Field | Carried on | Specified in |
|---|---|---|
| `interpolation` | A keyframe — the **earlier** keyframe of its segment; the last carries none. Linear is the absent form | *How a segment interpolates* |
| `end_behaviour` | An animation. **Always written**, because no default is stated for it | *How an animation ends* |
| `triggers` | An animation. A list; each entry carries an event name and nothing else | *A trigger* |
| `order` | A parameter. An integer; equal and absent are both legal, ties break by identifier, absent sorts last | *Canonical serialisation* |
| `mappable` | A parameter. A boolean, **absent when true** | *Canonical serialisation* |

`interpolation` and `end_behaviour` are enumerated fields, so each gains the extra inhabitant
*An unfamiliar value in a field this build knows* gives every enumerated field: invalid at the
current version, carried verbatim at a later declared one. `AxisPolarity` in
[puppet.rs](../../crates/rigel-format/src/puppet.rs) is the shape already in the tree for that.

A trigger's event name is **not** an enumerated field — the set of names is open at every
version, and an unrecognised name never fires and is never a validation failure.

### New load limits

[limits.rs](../../crates/rigel-format/src/limits.rs) is the one place a limit is declared
(`LIM-1`), and every stored collection has one (`LIM-8`). Two kinds are missing:

- **the aggregate keyframe limit**, which [format.md](../format.md) *Load limits* sets at
  **16,777,216** over the whole puppet, on a stated 24-bytes-per-keyframe argument. Its basis is
  `Memory` and **not** `Stress` — `KeyframesPerTrack` carries the note on why re-deriving it as
  a workload multiple is the error. It must be checked **before allocating**, which needs a
  running count across the decode path rather than a check on a finished collection;
- **a limit on an animation's `triggers`**, which is a new stored collection.

`format.md` says the aggregate is **over the whole puppet**;
[2026-11-28-animation-dimensions.md](../research/2026-11-28-animation-dimensions.md) says *per
animation* in two places. **`format.md` governs** — it is the design document for the format,
and the figure's own arithmetic is stated against a puppet. Report the research file's wording
under *Contradicts docs* rather than reconciling it here.

### Acceptance criteria

From [puppet-acceptance.md](../requirements/puppet-acceptance.md) section R — **the format half
only**. Each of these is answerable by storing, validating and writing back, with nothing
evaluated:

| ID | Why it is in scope |
|---|---|
| `INTERP-2` | Modes round-trip per segment; a track whose segments are all linear stores no mode anywhere |
| `INTERP-4` | A parameter carries no interpolation field |
| `END-4` | `judgment` — the puppet carries no animation kind alongside the end behaviour |
| `LOST-3` | `rigged:triggered` validates, loads and saves byte-stably; an event name nothing raises is not invalid |
| `LOST-4` | The rejection half: a trigger carrying any field beyond an event name is unmarked data at the current version |
| `UNF-1` | The *loads and is written back verbatim* half, for the unfamiliar interpolation mode and the unfamiliar end behaviour. The *equals the control puppet* halves are S4's |
| `ORD-1` | The order round-trips; reordering changes no identifier and no reference |
| `MAP-1` | A parameter declared not mappable validates — the flag is not a validation rule |
| `MAP-3` | Save and the stored flag is still `false` |
| `MAP-4` | A parameter both mapped and animated keeps the flag it was given |
| `LIM-1`, `LIM-8` | From section J: every declared limit is in one place and has a fixture just over it |

Section D's `TOOL-4` and section E's `VER-1` and `VER-6` already exist and must keep passing
with the new fields present.

**`INTERP-3` is split.** Its *stores modes by position* trap is what the earlier-keyframe
storage rule exists to prevent, and the stored half — a mode belongs to its segment and survives
a round trip — is asserted here. The half over *every operation that inserts or deletes a
keyframe* needs editing operations that do not exist, and is Studio's.

### Concern criteria

From [concerns/](../concerns/README.md). These are specifications, not later checks.

| Entry | Dimension | What it requires here |
|---|---|---|
| **An aggregate limit exists wherever per-item limits multiply** — `script` | security | Extend the limits completeness walk to require an aggregate wherever per-item limits multiply. This entry was seeded by the sweep that found animation × track × keyframe permitting hundreds of millions of keyframes |
| **Every stored collection and string has a declared limit, and the limit set and the schema agree** — `script` | security | `triggers` is a new collection and needs a figure. The **committed JSON Schema** in `crates/rigel-format/schema/` is the other half of *agree* and must gain every new field |
| **Load limits are defined in one place and each is exercised by a fixture just over it** — `test` | security | The aggregate keyframe limit and the trigger limit each need a just-over fixture |
| **Every allocation sized from a file-derived length is preceded by a limit check on that length** — `script` | security | The existing grep must still pass over the new decode paths |
| **A trigger is data selecting from a set, and no field of it is evaluated** — `test` | security | Fixtures carrying an expression-like string as an event name, and an unmarked extra field where a condition or a target would go. This is the entry that keeps the line where the sweep found it, and it is **I1** |
| **A track's keyframe times are finite and strictly ordered, its values finite and within its parameter's range, and its blend durations finite and non-negative** — `test` | security | Already enforced; the Bézier control points are new. `format.md` requires control **times inside the segment** and every control **value finite** |
| **Unmarked data the core format does not define is rejected; marked extension data round-trips untouched** — `judgment` | data | Holds for every new field and for a trigger |
| **Presentation state does not leak into behaviour** — `test` | ux | Shuffle `order`, flip `mappable`, assert identical output. Both are presentation and neither may change what is stored beyond itself |

### Already ruled out

- **Deferring this to S4.** Considered and rejected by the owner on 2026-11-28: S3 and S4 both
  need the shape, and folding it into either makes the other block on it. A subsystem of its own
  keeps S3 and S4 independent, which [plan.md](plan.md) says they are.
- **Making the generator emit the retired shape and regenerating later.** Same decision. The
  three animation dimension rows were measured on 2026-11-28 precisely so the generator could
  use them; emitting them into a shape the design has retired wastes that.
- **Closing the keypoint/keyframe arity `[open]`.** It sits in the middle of this work and looks
  cheap. [format.md](../format.md) says nothing may assume an answer, and
  [open-questions.md](open-questions.md) files it under *waiting on design work*. Leave it.
- **Deriving the aggregate keyframe limit from the workload table.** `KeyframesPerTrack` already
  carries the note: a workload multiple is the wrong quantity for this kind of figure, and is how
  the old 4096 came to be exceeded by a one-minute animation.
- **A `composition_mode` or any field saying a track composes additively.**
  [format.md](../format.md) is explicit that a track carries no such field, and that a replacing
  mode, if one is ever needed, arrives later with **absent meaning additive**.
- **Giving a trigger an identifier.** Deliberate: nothing refers to a trigger, so an identifier
  would be a value nothing resolves and **I3** would bind for no purpose.

---

## Decisions

Made for this feature. Anything meeting [decisions.md](../decisions.md)'s admission test moves
there immediately — a narrow set. This list is everything else, and it survives every task
cleanup.

- **The animation format change is its own subsystem, S2b, rather than part of S3 or S4** —
  owner's decision, 2026-11-28. Both S3 and S4 need the shape, and attaching it to either would
  make the other block on it. [plan.md](plan.md) gains a row rather than growing one.
- **`format.md` governs over the research file on where the aggregate keyframe limit binds** —
  whole puppet, not per animation. The design document is the format's specification, and the
  figure's stated arithmetic is a whole-puppet one.
- **The acceptance tests are written after the fixtures and before the implementation, by
  `acceptance-planner`, which also binds the public surface** — the shape S2 used, for the
  reason [testing.md](../testing.md) gives. The planner may read `format.md`,
  `puppet-acceptance.md` and `puppet.rs`'s public types, because a test that loads a file has to
  know the encoding and the surface it calls. It may **not** read `decode.rs`, `validate.rs` or
  `save.rs`, which are what it is testing.

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

-

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

-

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The fixtures — `todo`

Hand-author the fixtures section R's format half needs, in `rigel-fixtures`, from
[format.md](../format.md) with third-party libraries only. That crate depends on no other crate
in this workspace and **must stay that way** — reaching for a `rigel-format` type is the one
failure it exists to prevent.

From [puppet-acceptance.md](../requirements/puppet-acceptance.md)'s *The fixtures*:
`rigged:mixed-interpolation`, `rigged:endings`, `rigged:triggered`, `rigged:ordered-parameters`,
and the `rigged:future-minor-…-unfamiliar` variants for an interpolation mode in the middle of a
track and for an end behaviour. Plus the defect fixtures the criteria and concern entries above
name — a trigger with an extra field, a duplicate event name on one animation, a Bézier control
time outside its segment, a non-finite control value — and a just-over fixture for each new load
limit. `rigged` itself gains the new fields, since every fixture derives from it.

**Done when:** the fixtures build, `scripts/fixtures-check.sh` passes, `catalogue.json` names
each one and what a conforming loader does with it, and the committed bytes are in the tree. No
user gate — the third-party gate on the corpus was run in S2 and this adds to it rather than
changing how it is written.

### 2. The acceptance tests, and the surface they bind — `todo`

Dispatch `acceptance-planner`. It writes the section R format-half criteria as tests in
`crates/rigel-format/tests/acceptance/`, and adds the public fields to `puppet.rs` that those
tests call — the surface, not the behaviour.

**The suite goes red here and that is correct.** S2 did the same; the tests describe what is
asked for, and task 3 makes them pass.

**Done when:** the workspace compiles, the new tests fail for the stated reason rather than by
not building, and every criterion in *Acceptance criteria* above has a test or a recorded reason
why it has none. No user gate.

### 3. Decode, validate, write — `todo`

Dispatch `module-dev`. Make the tests from task 2 pass: the decode path, the validation rules,
the canonical writer, the JSON Schema, and the two new entries in
[limits.rs](../../crates/rigel-format/src/limits.rs) with the aggregate counted before
allocating.

**Never edit an acceptance test to make it pass.** One that seems wrong is a finding.

**Done when:** `scripts/test-record.sh` records a pass for the commit, `scripts/docs-check.sh`
passes, and the limits completeness and allocation-check scripts pass over the new paths. No
user gate — this is a format-internal change with no surface a person can look at that an
assertion cannot.

### 4. Review, route the handoff, move the plan on — `todo`

`code-reviewer` on the diff against this assignment, and `concern-analyst` in review mode for
**security** and **data**. Route the handoff slots, dispatch `doc-writer`, mark S2b done in
[plan.md](plan.md), delete this file in the same commit.

**Done when:** the review findings are fixed or consciously declined with a reason, `plan.md`
carries what S3 and S4 inherit from this feature, and the branch is ready to merge.

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**

**Decisions made:**

**Contradicts docs:**

- [README.md](../README.md) states *the workspace builds and the push gates run; no production
  code is implemented yet* and *The next work is requirements, not code — S2 is gated on gaps
  that `plan.md` names under* What blocks S2. Both were true before S2 landed and neither is now:
  `rigel-format` carries production code, and S2 is done. Found while orienting, before this
  branch existed.
- [2026-11-28-animation-dimensions.md](../research/2026-11-28-animation-dimensions.md) says the
  aggregate keyframe limit should bind **per animation**, in its *Verdict* table and again under
  *The distribution is the finding*. [format.md](../format.md) binds it per **puppet**. See
  *New load limits* above for which governs and why.

**Traps:**

**Assumptions:**

**Deferred:**

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
