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

**Current task:** 1 done except the aggregate keyframe just-over fixture; task 2 is next
**Blocked on:** nothing for task 2. One question is owed to design and does not block it —
[format.md](../format.md)'s aggregate keyframe limit of 16,777,216 cannot be reached by any file
the payload limit admits, so `LIM-1`'s *every declared limit has a fixture just over it* cannot
be satisfied for it. See *Contradicts docs* below.

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

- **Committing a just-over fixture for the aggregate keyframe limit at any size.** Measured at
  519 MB of text and 44 MB deflated, and refused by the payload limit rather than the aggregate
  one. The arithmetic is in the handoff's *Contradicts docs*.
- **A Bézier whose control times stay strictly inside its segment, for the degenerate case.**
  There is no such curve: the zero derivative `INTERP-5` exists to sweep requires a control
  point on a keyframe.

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

-

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The fixtures — `done`

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

- The animation-shape fixtures, in `crates/rigel-fixtures`. `catalogue.json` beside the corpus
  names each one, what a conforming loader does with it, and the one thing that makes it what it
  is; the builder section is `animation_shape` in
  [fixtures.rs](../../crates/rigel-fixtures/src/fixtures.rs).
- `rigged` itself gains `end_behaviour` on its animation and nothing else — see *Decisions*.
- The encoding helpers the fixtures compose, in
  [puppet.rs](../../crates/rigel-fixtures/src/puppet.rs): the end behaviours, the interpolation
  modes beyond linear, the two named scene events, a trigger, a keyframe, a keyframe opening a
  segment in a stated mode, and a keyframe opening a Bézier segment.
- Unit tests over all of it in [tests.rs](../../crates/rigel-fixtures/src/tests.rs), including a
  corpus-wide one asserting that no **accepted** fixture carries a malformed Bézier segment and
  one asserting that no trigger outside `rigged:trigger-extra-field` carries anything but an
  event name.

**Decisions made:**

- **`rigged` gains `end_behaviour` and no other new field.** It is the only one of the five that
  is always written; the rest are absent in their default form (linear is the absent
  interpolation, `mappable` is absent when true, an absent `order` is legal and sorts last, and
  an empty `triggers` follows the corpus's existing treatment of empty collections — `one-part`
  already omits `parameters` and `bindings`). Giving `rigged` an order, a flag, a trigger or a
  mode would destroy the control that `INTERP-2`, `ORD-1` and `MAP-1` are each read against.
- **A trigger is `{"event": "<name>"}`.** `docs/format.md` names the collection `triggers` and
  says an entry carries the event name and nothing else, but not the key. `event` is the
  vocabulary's word for the thing.
- **A Bézier segment's control points are shaped like keyframes** — `control_points` on the
  segment's earlier keyframe, a two-element list of `{"time", "value"}`, the value an array per
  axis. `format.md` requires control times and finite control values and specifies no encoding.
  Mirroring a keyframe makes the arity follow the parameter's axes automatically, the way a
  keyframe's does, and leaves a reader one shape rather than two.
- **The two scene event names are `on_load` and `source_lost`**, in the `snake_case` the crate
  uses for every key. The set is open, so these are the two with a use rather than an enumeration.
- **The committed sample expectations for `rigged:mixed-interpolation` ride as extension data**,
  under `com.rigel2d.fixture-expectations`. The fixture row asks for them and there is nowhere
  else in a puppet they can legally sit: outside `x` they would be unmarked core data and the
  file would be invalid. They are marked, inert and carried untouched, which is exactly what a
  committed expectation needs to be. Every one is exact and hand-derived — the stepped and
  inverse-stepped values are the segment's own, the linear one is a midpoint, the symmetric
  Bézier passes through the segment midpoint by the symmetry of its control points, and the
  degenerate Bézier is sampled only at its endpoints, which every cubic reproduces exactly.
- **`rigged:ordered-parameters` declares `mappable: false` on two parameters, not one.** The
  fixture row names only the mapped one; `MAP-1` needs a parameter that is not mappable **and
  carries no channel mapping**, and no other fixture carries one.
- **The provisional trigger limit is 64**, declared as
  `PROVISIONAL_TRIGGERS_PER_ANIMATION` in `puppet.rs` beside the existing
  `PROVISIONAL_NODE_LIMIT`. No `stress` dimension counts triggers and none will — an animation
  waits on a handful of scene events — so it is `Provisional` in `limits.rs`'s sense.
  **Task 3's figure in `limits.rs` must be this one**, or `rigged:triggers-over-limit` stops
  being just over anything; the crate's unit test asserts the fixture is exactly one over the
  constant, so the two can only drift if both are edited.
- **Defect fixtures beyond the four the assignment named.** The assignment called its list a
  floor. Added from `format.md`'s *Validation* and from the enumerated-field rule: an end
  behaviour that is absent, an interpolation mode on a track's last keyframe, a Bézier segment
  declaring the mode and carrying no control points, and the current-version half of each
  unfamiliar-value pair. Added from the data concern on the canonical writer binding no reader:
  an explicit `linear`, an explicit `mappable: true`, and an out-of-order `triggers` list, each
  **accepted** and canonicalised on save. Added from the security concern on triggers: an event
  name shaped like a condition, which is **accepted**, because the set of names is open and an
  unrecognised one is never a validation failure.

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
- **[format.md](../format.md) *Load limits* sets the aggregate keyframe limit at 16,777,216, and
  no file that limit could ever refuse is loadable.** A keyframe cannot be spelled in JSON in
  fewer than about twenty bytes — `{"time":T,"value":[V]}` — and `DecompressedPayloadBytes` in
  [limits.rs](../../crates/rigel-format/src/limits.rs) is 64 MiB, so the payload limit already
  bounds a puppet at roughly three million keyframes. The aggregate limit sits above that by a
  factor of five and can never fire. A just-over fixture is correspondingly impossible: the
  smallest honest one — 128 animations at two or three tracks each, every track at the 65,536
  per-track limit, 16,777,217 keyframes in all — measures **519 MB of `puppet.json` text and
  44 MB deflated**, and would be refused for exceeding the payload limit rather than the
  aggregate one, which is the wrong rule. The figure needs re-deriving against what the payload
  limit admits, and that is a design decision rather than this feature's.
- **[README.md](../README.md) in `crates/rigel-fixtures` states *The load limits are not here*
  and that the just-over fixture per declared limit is deferred because the limits do not
  exist.** They landed in S2: [limits.rs](../../crates/rigel-format/src/limits.rs) carries a
  figure for every kind. The consequence is live rather than cosmetic —
  `PROVISIONAL_NODE_LIMIT` in that crate is 256 while `NodeCount` is 2,160, so
  `rigged:near-limit` is nowhere near the node limit and the fixture no longer tests what its
  name says. Not touched here: regenerating it would add two thousand nodes to the committed
  corpus and is outside this task.
- **[format.md](../format.md) *How a segment interpolates* says a Bézier segment's control times
  *lie inside the segment* and does not say whether the endpoints count.** They must: the only
  way to make a degenerate Bézier — the one
  [puppet-acceptance.md](../requirements/puppet-acceptance.md)'s fixture row asks for and
  `INTERP-5` sweeps — is a control point coincident with a keyframe, which gives the zero
  derivative that the Newton step divides by. A strictly-interior reading makes the degenerate
  case inexpressible. The fixtures take the closed interval; see *Assumptions*.
- **[format.md](../format.md) names `interpolation`, `end_behaviour`, `triggers`, `order` and
  `mappable` but gives no key for a trigger's event name and no encoding for a Bézier segment's
  control points.** Both were decided here; see *Decisions made*.

**Traps:**

- **`rigged` is the control, and a field added to it silently disarms a criterion.** Adding
  `order`, `mappable`, a trigger or an interpolation mode to the base fixture would leave
  `INTERP-2`'s *a track whose segments are all linear stores no mode anywhere*, `ORD-1` and
  `MAP-1` passing against a fixture that no longer isolates what they name. A unit test now
  asserts that `rigged` carries `end_behaviour` and none of the others.
- **A defect fixture that declares a later version is accepted and its criterion stops testing
  anything.** The crate's existing test keys on the fixture's *name*, so a new
  `rigged:future-minor-…` variant must genuinely be the accepted half of a pair. The two new
  unfamiliar-value pairs are held to that by the differential test, which asserts the material
  is byte-identical across the version gate.
- **The aggregate keyframe fixture is not merely large, it is impossible.** Two hours could be
  spent making it smaller before noticing that the payload limit refuses it first. The
  arithmetic is in *Contradicts docs* above so nobody redoes it.

**Assumptions:**

- **A Bézier control time may equal a segment endpoint.** Stated above; the degenerate fixture
  depends on it, and task 3's validator must admit the closed interval or
  `rigged:mixed-interpolation` is refused.
- **An animation with no `end_behaviour` is invalid.** `format.md` says the field is always
  written because no default is stated for it, and a reader meeting it absent has nothing to
  fall back on. `rigged:end-behaviour-absent` is catalogued `rejected` on that reading.
- **An interpolation mode on a track's last keyframe is invalid**, rather than ignored. The
  storage rule says the last keyframe carries none, and a mode there names a segment that does
  not exist. `rigged:interpolation-on-last-keyframe` is catalogued `rejected` on that reading.
- **An empty `triggers` is absent rather than written empty.** Taken from the corpus's existing
  treatment of empty collections rather than from a stated rule.

**Deferred:**

- **`rigged:keyframes-over-aggregate`, the just-over fixture for the aggregate keyframe limit.**
  Impossible at the figure `format.md` states — the arithmetic is in *Contradicts docs*. It
  returns when the figure does.
- **Regenerating `rigged:near-limit` against the real `NodeCount` limit.** Reported above;
  outside this task.

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
