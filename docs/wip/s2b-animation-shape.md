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

**Current task:** 1, 2 and 3 done. The code half of task 4's review findings is done — see
*Fix pass* at the end of this file. What remains of task 4 is routing the handoff, the
documentation pass, and marking S2b done in [plan.md](plan.md)
**Blocked on:** nothing. `cargo test --workspace`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets`, `scripts/fixtures-check.sh` and `scripts/docs-check.sh`
are all clean, and `scripts/test-record.sh` has recorded the run. The aggregate limit's peak
assertion no longer blocks: it now asserts against `TotalLoadMemory` and a doubling of
`ParseTreeBytes`, which is the suite's own precedent for a limit the tree reaches before the
decoder does, and the reasoning stays recorded under *Contradicts docs*, **the peak assertion**
**Was blocked on:** nothing. The aggregate keyframe limit is **262,144**, settled by the owner on
2026-11-28, and `rigged:keyframes-over-aggregate` is built against it and verified to be
refused for that rule and no other. Both earlier figures were unreachable and the record of why
stays in *Contradicts docs*, because the failure mode repeats: a figure checked against one
ceiling and not the next one down.

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
  **262,144** over the whole puppet, below what `ParseTreeBytes` admits. Its basis is
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
- **The aggregate keyframe limit is 262,144** — owner's decision, 2026-11-28, at the second
  attempt. Two earlier figures could never fire, each set against one ceiling without checking
  the next one down: 16,777,216 against residency, then 1,048,576 against the payload limit but
  not against `ParseTreeBytes`, which admits about 586,000 and is the real constraint. A
  keyframe costs some 458 bytes parsed against 24 resident, so the tree fills roughly nineteen
  times faster than memory does. The figure sits below the crossover with more than double the
  margin, deliberately: the per-keyframe charge is an estimate that may be retuned, and a limit
  set close under it would quietly stop firing when that happened.
  [format.md](../format.md) *Load limits* carries the three ceilings, the arithmetic and what
  the figure refuses. Rejected alongside it: 524,288, whose tenth of margin is thin against a
  figure already wrong twice; raising the payload limit, which would stop the catalogued
  decompression bomb being refused; and dropping the aggregate entirely, which would leave the
  bound legible only by dividing 256 MiB by a charge documented in `json.rs`.
- **The acceptance tests are written after the fixtures and before the implementation, by
  `acceptance-planner`, which also binds the public surface** — the shape S2 used, for the
  reason [testing.md](../testing.md) gives. The planner may read `format.md`,
  `puppet-acceptance.md` and `puppet.rs`'s public types, because a test that loads a file has to
  know the encoding and the surface it calls. It may **not** read `decode.rs`, `validate.rs` or
  `save.rs`, which are what it is testing.

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- **Committing a just-over fixture for the aggregate keyframe limit at 16,777,216.** 519 MB of
  text, 44 MB deflated, and refused by the payload limit rather than the aggregate one.
- **Committing one at the corrected 1,048,576.** Small enough to commit — 33.4 MB of text,
  2.66 MB deflated — and still the wrong fixture: `rigel-validate` refuses it for
  `parse-tree-bytes`. Both figures and the measured crossover are in the handoff's
  *Contradicts docs*; nobody needs to rebuild either.
- **Spelling keyframes more compactly to buy parse-tree margin.** The charge is dominated by
  structure, not digits: about 418 of the 458 bytes a keyframe costs are the object, its two
  member names and the enums, none of which a terser number spelling touches. Shortening every
  number to one character moves the crossover by under a tenth.
- **A Bézier whose control times stay strictly inside its segment, for the degenerate case.**
  There is no such curve: the zero derivative `INTERP-5` exists to sweep requires a control
  point on a keyframe.
- **Carrying a Bézier segment's control points as a payload on `Interpolation::Bezier`.** It
  reads better and it makes `rigged:bezier-without-control-points` unrepresentable in memory —
  so only the decoder could refuse that fixture, and `validate(&Puppet)` could never reach it.
  The defect has a committed fixture precisely so a validator refuses it. See *Decisions made*.
- **Folding a non-finite Bézier control value into `Rule::NumbersFinite`**, by adding control
  points to the validator's `numbers()` walk. It works and the fixture is refused — for two
  rules at once, which is what a one-defect corpus exists to prevent. Task 3.
- **Sorting an animation's triggers in the writer rather than in the decoder.** A preserved
  member's path carries the trigger's index, so sorting at save time moves preserved material
  onto a different trigger. Task 3.

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

### 2. The acceptance tests, and the surface they bind — `done`

Dispatch `acceptance-planner`. It writes the section R format-half criteria as tests in
`crates/rigel-format/tests/acceptance/`, and adds the public fields to `puppet.rs` that those
tests call — the surface, not the behaviour.

**The suite goes red here and that is correct.** S2 did the same; the tests describe what is
asked for, and task 3 makes them pass.

**Done when:** the workspace compiles, the new tests fail for the stated reason rather than by
not building, and every criterion in *Acceptance criteria* above has a test or a recorded reason
why it has none. No user gate.

### 3. Decode, validate, write — `done`

Dispatch `module-dev`. Make the tests from task 2 pass: the decode path, the validation rules,
the canonical writer, the JSON Schema, and the two new entries in
[limits.rs](../../crates/rigel-format/src/limits.rs) with the aggregate counted before
allocating.

**Never edit an acceptance test to make it pass.** One that seems wrong is a finding. One was:
the peak-memory half of the aggregate limit test. It is left failing and reported.

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
- `rigged:keyframes-over-aggregate`, the just-over fixture for the aggregate keyframe limit, at
  the settled figure. It is the largest fixture in the corpus by an order of magnitude; what it
  costs to build and to test is under *Traps*.
- The encoding helpers the fixtures compose, in
  [puppet.rs](../../crates/rigel-fixtures/src/puppet.rs): the end behaviours, the interpolation
  modes beyond linear, the two named scene events, a trigger, a keyframe, a keyframe opening a
  segment in a stated mode, and a keyframe opening a Bézier segment.
- Unit tests over all of it in [tests.rs](../../crates/rigel-fixtures/src/tests.rs), including a
  corpus-wide one asserting that no **accepted** fixture carries a malformed Bézier segment and
  one asserting that no trigger outside `rigged:trigger-extra-field` carries anything but an
  event name.
- The acceptance tests for section R's format half, in
  [animation_shape.rs](../../crates/rigel-format/tests/acceptance/animation_shape.rs), registered
  in that suite's `main.rs` — cargo discovers `tests/<dir>/main.rs` only, so the `mod` line is
  what makes the file run. Each test names its criterion and says which half of it is S4's,
  Studio's or Live's.
- The public surface those tests call, in
  [puppet.rs](../../crates/rigel-format/src/puppet.rs): `Keyframe::interpolation` and
  `Keyframe::control_points`, `Animation::end_behaviour` and `Animation::triggers`,
  `Parameter::order` and `Parameter::mappable`, with the types `Interpolation`, `ControlPoint`,
  `EndBehaviour` and `Trigger` beside them and re-exported from `lib.rs`. **The surface only** —
  no decode, no validation, no writing.

**Task 3 — decode, validate, write.**

- The five fields read, validated and written: `interpolation` and `control_points` on a
  keyframe, `end_behaviour` and `triggers` on an animation, `order` and `mappable` on a
  parameter. Every `// Task 2 (acceptance surface)` placeholder is gone, the invented
  `EndBehaviour::Loop` in [decode.rs](../../crates/rigel-format/src/decode.rs) among them.
- Six `Rule` variants in [validate.rs](../../crates/rigel-format/src/validate.rs), one per
  defect the corpus carries and `format.md`'s *Validation* list does not state:
  `end-behaviour-declared`, `interpolation-opens-a-segment`, `trigger-events-unique`,
  `bezier-segment-carries-control-points`, `bezier-control-time-inside-segment`,
  `bezier-control-value-finite`. Each defect fixture is now refused for **its own rule and no
  other**, verified one at a time through `rigel-validate`.
- Two rows in [limits.rs](../../crates/rigel-format/src/limits.rs): `TotalKeyframeCount` at
  262,144, basis `Memory`, and `TriggersPerAnimation` at 64, basis `Provisional`.
- The committed JSON Schema carries every new field, plus a `trigger` and a `control_point`
  definition. The unit test holding the schema's member lists against the reader's key sets
  now covers both, and [schema/README.md](../../crates/rigel-format/schema/README.md) carries a
  boundary row per new rule.


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
- **`Parameter::order` is `i64`.** `format.md` says only *an integer*. A stranger's writer may
  emit a negative one and nothing forbids it, so the signed type admits everything the
  specification permits. Task 3's decoder and the JSON Schema must agree.
- **A Bézier segment's control points are a field of the keyframe, not a payload on
  `Interpolation::Bezier`.** This is what keeps `rigged:bezier-without-control-points`
  **representable in memory**, so the *validator* refuses it rather than the decoder — which is
  what *one validator gates load and save* (`VAL-13`) means for this defect. The variant-payload
  shape makes it unreachable from `validate(&Puppet)` entirely.
- **Exactly two control points is a type invariant**, `Option<[ControlPoint; 2]>`. `format.md`
  states no count rule and no fixture carries a wrong count, so a three-point file is refused at
  decode. Task 3 must change this to a `Vec` if that is to be a validation rule instead.
- **The six defect fixtures with no rule named in the test.** `rigged:end-behaviour-absent`,
  `rigged:interpolation-on-last-keyframe`, `rigged:duplicate-trigger-event` and the three Bézier
  defects are asserted as *refused by the loader and by the standalone validator, with a bounded
  message*, and no further. `Rule` lives in `validate.rs`, which the planner may not read or
  edit, and no existing variant covers them — naming one would be designing the validator from
  the test. **Task 3 owes a `Rule` variant for each**, and a later pass can then tighten these to
  `assert_rejected_naming`. Where the criterion or `format.md` does name the rule — unmarked data
  for a trigger's extra field, the enumerated-value rule for an unfamiliar token — it is pinned.
- **The two new limits are asserted by their figure and their note, not by a `LimitKind` name.**
  `figure_for(262_144, "keyframe")` and `figure_for(64, "trigger")` find the single row in
  `FIGURES` carrying that figure. Task 3 is therefore free to name the kinds, and is not free to
  move either figure without the tests noticing.
- **Defect fixtures beyond the four the assignment named.** The assignment called its list a
  floor. Added from `format.md`'s *Validation* and from the enumerated-field rule: an end
  behaviour that is absent, an interpolation mode on a track's last keyframe, a Bézier segment
  declaring the mode and carrying no control points, and the current-version half of each
  unfamiliar-value pair. Added from the data concern on the canonical writer binding no reader:
  an explicit `linear`, an explicit `mappable: true`, and an out-of-order `triggers` list, each
  **accepted** and canonicalised on save. Added from the security concern on triggers: an event
  name shaped like a condition, which is **accepted**, because the set of names is open and an
  unrecognised one is never a validation failure.

**Task 3.**

- **The aggregate keyframe count runs in the `Decoder`, charged per track before the keyframe
  collection is allocated.** The counter is a field on the decoder rather than a check over a
  finished collection, which is what the assignment asked for. It is the reason the peak
  assertion below cannot hold: see *Contradicts docs*.
- **A non-finite Bézier control value gets its own rule rather than `NumbersFinite`**, and
  control points are therefore **not** added to the validator's `numbers()` walk. One defect
  naming one rule is what makes a one-defect corpus worth having, and a control point checked
  in two places would name two.
- **Exactly two control points stays a type invariant**, so a three-point file is
  `Rejection::Malformed` at decode rather than a validation rule. No fixture carries a wrong
  count and `format.md` states no count rule, so a `Vec` plus a rule would be a validator
  designed from nothing. Task 2 asked to be told either way: **this is the answer**, and it is
  one line to reverse if a fixture ever needs it.
- **A negative `order` is accepted.** `format.md` says only *an integer* and the reader admits
  everything the specification permits; the schema says `"type": "integer"` to match. Refusing
  one would be a rule no document states. Reported under *Contradicts docs* as a gap, not
  decided silently.
- **Triggers are sorted by event name in the decoder, not in the writer.** `save.rs` documents
  itself as writing *a puppet already in canonical order*, and sorting at read keeps a
  preserved member's path — `triggers/<n>` — pointing at the same trigger through a round trip.
  Sorting in the writer would renumber those paths under the preserved material.
- **An absent `end_behaviour` yields `EndBehaviour::Unfamiliar("")` after the violation is
  raised**, deliberately not one of the three the format defines. The file is already refused,
  so the value is never read — and a test cannot accidentally pass against it, which is what
  the invented `Loop` made possible.
- **The trigger limit is not written into the schema as a `maxItems`.**
  `schema/README.md`'s *The load limits are not here either* says load limits are deliberately
  absent from the schema; the security entry's *the limit set and the schema agree* is read as
  the member lists agreeing, which the unit test enforces.


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
  aggregate one, which is the wrong rule. **Settled 2026-11-28** — the owner lowered the figure
  to 1,048,576, below what the payload limit admits at any number spelling, and
  [format.md](../format.md) *Load limits* now carries the corrected figure, the arithmetic, and
  what it refuses. **That correction was itself wrong** — it cleared the payload limit and not
  the parse tree. Next entry.
- **The aggregate keyframe limit at 1,048,576 is refused by `ParseTreeBytes` before it is
  reached.** The correction moved the figure below what `DecompressedPayloadBytes` admits, but
  the payload limit bounds the *text* and `ParseTreeBytes` bounds what the text parses into,
  which for keyframes is the larger quantity by an order of magnitude. A keyframe parses to
  about **458 bytes** charged against that budget — the object's enum and its growth slot, two
  member names held twice over, an enum per number, an array, and each number kept as its own
  heap string — so 256 MiB admits roughly **586,000** keyframes. 1,048,577 is about 1.8 times
  that. **Measured, not estimated:** the candidate archives were built and run through
  `rigel-validate`, and the crossover bisected.

  | Keyframes over the whole puppet | `puppet.json` text | Archive | What `rigel-validate` says |
  |---|---|---|---|
  | 1,048,577 | 33.4 MB | 2.66 MB | `limit.parse-tree-bytes` — the wrong rule |
  | 587,500 | 18.7 MB | 1.49 MB | `limit.parse-tree-bytes` — the wrong rule |
  | 586,250 | 18.7 MB | 1.49 MB | parses; reaches the aggregate rule |
  | 524,289 | 16.7 MB | 1.33 MB | parses; reaches the aggregate rule |
  | 262,145 | 8.4 MB | 0.67 MB | parses; reaches the aggregate rule |

  Every row keeps each track at or under `KeyframesPerTrack`, the animation count far inside
  `AnimationCount`, and the text far inside `DecompressedPayloadBytes`; the only limit that
  changes between the rows is the parse tree. **Settled 2026-11-28** — the owner chose
  **262,144**, which fires with more than double the margin and commits a 0.67 MB fixture, over
  524,288, whose tenth of margin is thin against a figure already wrong twice. The table above
  was reproduced independently by the dispatching session before the figure was changed.
  `rigged:keyframes-over-aggregate` is now built at 262,145 and carries 8.3 MB of text in a
  0.68 MB archive. It was verified the same way the table was: `rigel-validate` parses it
  through to `unmarked-data [end_behaviour]` — the expected S2b red — rather than refusing it
  for any limit, and no `limit.*` rejection appears anywhere in the workspace suite.
- **[README.md](../../crates/rigel-fixtures/README.md) stated *The load limits are not here*
  and that the just-over fixture per declared limit was deferred because the limits did not
  exist.** They landed in S2: [limits.rs](../../crates/rigel-format/src/limits.rs) carries a
  figure for every kind. **Corrected in place**, since it is that crate's own README and the
  edit is small. The live consequence it now records is **not** fixed and is still owed:
  `PROVISIONAL_NODE_LIMIT` in that crate is 256 while `NodeCount` is 2,160, so
  `rigged:near-limit` is nowhere near the node limit and the fixture no longer tests what its
  name says. Regenerating it would add two thousand nodes to the committed corpus, which is a
  decision rather than an edit.
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

- **[format.md](../format.md) *Validation* lists no rule for four defects the corpus now
  carries**: an absent `end_behaviour`, an interpolation mode on a track's last keyframe, a
  Bézier segment declaring the mode and carrying no control points, and one animation naming one
  scene event twice. The last is stated under *A trigger* but not in the *Validation* list, and
  the first three are only task 1's stated assumptions. The rule list is where a completeness
  check will look, so each needs a line there — and task 3 needs a `Rule` variant per line.
- **[format.md](../format.md) does not say whether a parameter's `order` may be negative.** The
  surface admits it; nothing in the document or the corpus decides it. See *Decisions made*.
- **[puppet-acceptance.md](../requirements/puppet-acceptance.md)'s *Criteria not written* still
  records *the figure of the total-keyframes limit* as missing** — *the requirement ... states no
  figure, so none can be asserted*. The figure is settled at 262,144 and
  [format.md](../format.md) *Load limits* carries it with its arithmetic, so that row is stale
  and `LIM-9` can assert the figure after all. The acceptance test does assert it.

**Task 3.**

- **The peak assertion in
  `lim_1_the_aggregate_keyframe_limit_is_declared_once_and_bites_before_allocating` cannot hold
  for this reader, and is left failing rather than edited.** It asserts that loading
  `rigged:keyframes-over-aggregate` peaks below `DecompressedPayloadBytes`, 67,108,864 bytes.
  Measured under the counting allocator, the failed load peaks at **145,840,208 bytes** — about
  556 bytes per keyframe, against the 458 `format.md` *Load limits* charges one in the parse
  tree.

  The cause is architectural and already written down: `limits.rs` says of `ParseTreeBytes`
  that *nothing downstream can refuse it, because every collection limit in `decode` runs after
  the tree exists*. A fixture **one** keyframe over an aggregate has to be read almost
  entirely before the count can exceed it, so the tree for 262,144 keyframes is built whatever
  the counting is. Counting inside the JSON parser rather than in the decoder moves the refusal
  earlier by one keyframe and the peak not at all. No aggregate figure fixes it either: at the
  measured cost, peaking under 64 MiB needs a limit near 120,000, which collides with
  `KeyframesPerTrack`.

  The only implementation that satisfies it is a **streaming pre-scan of the payload text that
  counts keyframes before the tree is built**. That is a second parser with a second definition
  of *a keyframe*, run on every load, and it would single this limit out from every other
  collection limit in the format, which the project settled run after the tree with
  `ParseTreeBytes` as the compensating bound. It is not what the assignment asked for — *a
  running count across the decode path* — and it was not built.

  **The suite's own precedent is the fix.**
  `a_payload_whose_parsed_tree_exceeds_the_budget_is_refused_before_it_is_built` asserts
  `peak < TotalLoadMemory` and `peak < 2 * ParseTreeBytes`. Both hold here — 145.8 MB against
  1 GiB and against 536 MB — and both say what the criterion means for a limit the tree reaches
  before the decoder does. Everything else in the test passes: the figure is declared once, its
  basis is `Memory`, and the rejection is `limit.total-keyframe-count` at 262,144.

- **`format.md` *Load limits* charges a keyframe 458 bytes in the parse tree; the measurement
  above puts live allocation nearer 556.** The 458 is `json.rs`'s accounting figure, and the
  gap means `ParseTreeBytes` admits rather more real memory than its arithmetic suggests —
  about 326 MB at its own 586,000-keyframe ceiling, against a 256 MiB budget. The measured peak
  includes the transient doubling a growing `Vec` really holds, so this is an upper bound
  rather than a settled figure, but the direction is wrong and the figure is quoted in
  `format.md` as if it bounded memory.

- **`format.md` *Validation* still lists no rule for the four defects task 1 reported**, and a
  fifth: an absent `end_behaviour`, an interpolation mode on a track's last keyframe, a Bézier
  segment declaring the mode with no control points, one animation naming one event twice, and
  — new here — a Bézier **control value** that is not finite, which the list folds into the
  control-times bullet rather than naming. Each now has a `Rule`; each still needs a line in
  that list, which is where a completeness check will look.

- **`format.md` does not say whether a parameter's `order` may be negative**, as task 1
  reported. The decoder and the schema now both admit one. If it should be refused, that is a
  document change and a seventh rule, not an edit here.

- **`schema/README.md` stated *The load limits are not here either*** and that is still true,
  including for `TriggersPerAnimation`. Recorded because the security concern entry's phrase
  *the limit set and the schema agree* reads, at a glance, as asking for the figures.

- **The limits-completeness and allocation-check scripts the assignment names do not exist.**
  `scripts/` carries no such script and nothing in `tests/` implements either walk; both
  concern entries in [security.md](../concerns/security.md) are still `script` with no script.
  The new decode paths were audited by hand instead: every `with_capacity` sized from a
  file-derived length — `triggers`, `keyframes`, `control_points` — is preceded by its limit
  check. The one exception follows an existing pattern rather than introducing one: a control
  point's `value` array is sized from the file with no limit, exactly as a keyframe's `value`
  and a keypoint's `position` already are, because the keyframe/keypoint arity rule is `[open]`
  and `ParseTreeBytes` is what bounds them.


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
- **A just-over fixture for a limit has to clear every *neighbouring* limit, not just sit over
  its own.** This one took three attempts and both failures were the same mistake at different
  depths: a figure checked against one ceiling while the next one down was lower. Check a
  candidate by building it and running `rigel-validate` over it — the rejection line names the
  rule, so a fixture testing the wrong one is one command away from being obvious. The
  measurements are in *Contradicts docs* so nobody redoes them.
- **Its tracks are deliberately at half `KeyframesPerTrack`, not at it.** Filling them to
  exactly the per-track figure is legal and needs fewer tracks, but it leaves the fixture
  resting on whether that limit is read as admitting its own figure or refusing it. After two
  rounds lost to limit-boundary readings, the daylight is worth four extra tracks.
- **It is the largest fixture in the corpus by an order of magnitude, and it is not free.**
  Measured warm, so the figures are the fixture rather than a compile:

  | | Without it | With it |
  |---|---|---|
  | `cargo run -p rigel-fixtures` | 0.37 s, 10 MB peak | 0.68 s, 307 MB peak |
  | `cargo test -p rigel-fixtures` | 7.43 s, 203 MB peak | 10.24 s, 907 MB peak |

  `scripts/fixtures-check.sh` goes from about 4.0 s to 5.3 s. The test suite costs roughly twice
  the generator because `the_builder_is_deterministic` builds the whole corpus a second time.
  Nothing here is near a limit on any machine this project targets, but it is the first fixture
  whose cost is visible at all, and S3's generated workloads are larger again — so if the suite
  needs a diet, this is where it starts, and *Deferred* names the lever.
- **`EndBehaviour::Loop` in `decode.rs` is an invented placeholder, and the only value in the
  tree that nobody chose.** The field is not `Option`, there is no end behaviour decoded today,
  and the crate has to compile — so every `Animation` the decoder builds currently claims to
  loop. A test that passes against that is testing nothing. Every placeholder task 2 left is
  tagged `// Task 2 (acceptance surface)`, in `decode.rs` and in the `#[cfg(test)]` blocks of
  `validate.rs`, `save.rs` and `puppet.rs`.
- **Four of the new tests pass today, and one of those passes for the wrong reason.**
  `end_4_…`, `interp_4_…` and `a_track_carries_no_field_saying_how_it_composes` read the
  committed fixture bytes and assert an absence, which holds because task 1 built the fixtures
  correctly — they guard the corpus rather than the reader. The fourth,
  `a_mode_on_the_last_keyframe_opens_no_segment_and_is_refused`, passes because the fixture is
  currently refused for `unmarked-data [interpolation]` rather than for the defect it carries.
  It only becomes meaningful when task 3 lands the rule.

**Task 3.**

- **The aggregate keyframe fixture costs 145 MB and five seconds every time the acceptance
  suite runs**, whatever the assertion over it says. The figure above is what the counting
  allocator reports; nothing else in the suite is within an order of magnitude.
- **A preserved member's path carries an index, so re-sorting a collection at save time
  silently moves preserved material onto a different entity.** Triggers are the first
  collection whose canonical order is not the order it was read in, which is why the sort
  happens at read. Anything that later sorts in the writer has to renumber the preserved paths
  with it.
- **`EVERY_RULE` in `validate.rs`'s tests is three lists, not one** — the enum, the `as_str`
  match and the exhaustive match in the test — and only the last fails to compile when a
  variant is missed. A variant added to two of the three builds and passes.


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
- **An animation's and a track's key sets are closed to what `format.md` defines.**
  `end_4_…` and `a_track_carries_no_field_saying_how_it_composes` enumerate the keys an
  animation, a track and a keyframe may carry. A field added to any of the three in a later
  version fails those tests, which is deliberate — but it means the enumeration has to be
  maintained with the format.

**Task 3.**

- **A control point on a non-Bézier segment is not refused.** No fixture carries one and
  `format.md` states no rule, so the reader keeps it and writes it back. The acceptance suite
  asserts the *accepted* corpus pairs them correctly, which is a claim about the corpus rather
  than about the reader.
- **A non-finite Bézier control *time* is caught by `bezier-control-time-inside-segment`**,
  because a NaN compares false against both endpoints. That is a true rejection for the right
  reason by accident rather than by design; no fixture carries one.


**Deferred:**

- **Regenerating `rigged:near-limit` against the real `NodeCount` limit.** Reported above;
  outside this task.
- **Shrinking what `rigged:keyframes-over-aggregate` costs to build.** The builder materialises
  its keyframes as `serde_json` values, which is what the memory under *Traps* is. The crate's
  `Raws` escape hatch would let the keyframe array be composed as literal text instead and would
  remove most of it. Not done: it means hand-spelling floats and matching `serde_json`'s
  shortest-round-trip format exactly, which is a correctness risk taken for a cost nothing is
  yet paying. The lever is here if S3's generated workloads make it worth pulling.

**Task 3.**

- **A streaming pre-scan that would let the aggregate keyframe limit bite before the parse tree
  exists.** Reported above under *Contradicts docs*; it is a format-wide architectural change
  and is out of this task's scope either way.
- **Writing the two `script` concern entries the assignment expected to run.** They do not
  exist, and writing them is a task of its own rather than a side effect of this one.


---

## Fix pass

The code half of the three reviewers' findings on this branch, against `92e66dc`. Nothing in the
slots above is changed; this is what the fix pass added.

**Built:**

- **A non-finite control value is refused whatever the segment interpolates.**
  `validate.rs`'s `segments()` now checks every keyframe carrying `control_points`, before either
  of the two `continue`s that used to skip it — the last keyframe, and any mode that is not
  Bézier. The control-*time* check stays Bézier-only and non-last, because it names a segment and
  has none otherwise. The two gates are now different on purpose and the doc comment says why.
- **`rigged:control-value-nonfinite-without-bezier`**, the defect fixture for it: a keyframe with
  no `interpolation` member at all carrying `control_points` with `1e400`. Verified through
  `rigel-validate` to be refused for `bezier-control-value-finite` and nothing else;
  `rigged:bezier-control-value-nonfinite` still names that rule and no other.
- **The writer establishes an animation's trigger order.** `canonicalise()` sorts `triggers` by
  event name and repaths the preserved material sitting on them, exactly as a keypoint reorder
  permutes a binding's values. `repath` is now generic over the first path step. The decoder no
  longer sorts — see *Decisions made*.
- **`LimitKind::CoordinateCount`**, at **1024**, basis `Provisional`, with
  `PROVISIONAL_COORDINATE_COUNT` beside it in `rigel-fixtures` and a unit test pinning
  `rigged:coordinates-over-limit` to it. One `Decoder::coordinate_list` helper now bounds all
  three sites — a keypoint's `position`, a keyframe's `value`, a control point's `value` —
  before the `Vec::with_capacity` sized from the file.
- **A fourth allocation site fixed, found while fixing the three.** `Decoder::coordinates`, the
  mesh's point lists, checked `VertexCount` on the *returned vector* rather than on the array
  length, so the capacity was reserved first. Moved in front of the allocation; the rejection and
  its message are unchanged.
- Unit tests for each: the control value under every mode and on the last keyframe, the time rule
  staying Bézier-only, the writer's trigger sort carrying a preserved member to its new seat, and
  both new fixtures held to the shape that makes them the defect their name says.
- The two spliced doc comments repaired — `save.rs`'s `an_empty_collection_is_not_written` and
  `tests/acceptance/limits.rs`'s `lim_3_a_decompression_bomb_fails_as_a_limit`, the second of
  which is also on `master`.

**Decisions made:**

- **Fix 2 is the sort-and-repath option, not the key-by-event-name one.** Both remove the
  position dependence the recorded objection rested on. Sorting follows a mechanism that already
  exists, is already tested and is already understood — `canonicalise` permutes a parameter's
  keypoints and repaths the binding values underneath them — and generalising `repath` to take
  its path head is the whole change. Keying a preserved path by event name would introduce a
  *second* way of addressing preserved material, name-keyed beside position-keyed, for one
  collection; it also keys on a value whose uniqueness is a validation rule, and `decode` runs
  before `validate`, so a file with a duplicate event would collide two preserved rows during a
  decode that is about to be refused anyway. One mechanism beats two.
- **The decoder no longer sorts triggers, and the writer owns the order outright.** The decoder
  sorts no other domain collection — textures, parameters, bindings, animations, tracks and
  keypoints are all ordered by `canonicalise` — and triggers were the sole exception. Keeping
  both sorts would also leave the writer's repath dead for every file that ever came through the
  reader. `rigged:triggers-unsorted` now exercises the writer end to end.
- **`CoordinateCount` is 1024.** Three orders of magnitude above every answer the arity `[open]`
  could have — a parameter is one- or two-dimensional, so the candidates are at most `AxisCount`
  — which is what keeps it a bound on an allocation rather than an assertion about the rule. The
  style is `KeypointCount`'s and `ResponseCurvePoints`': a provisional figure with its reason
  written down. Nothing downstream reads it; the validator still zips coordinates against axes.
- **The mesh allocation fix was taken rather than reported.** It is the same rule, the same
  shape, in the function next to the three the brief named, and a one-line move. Writing a doc
  comment about the allocation-check rule while leaving a known violation of it two screens away
  was the alternative.

**Contradicts docs:**

- **`docs/concerns/security.md`'s allocation entry is a `script` with no script, and the gap is
  not theoretical.** The sites in `decode.rs` that sized an allocation from a file-derived length
  with no check in front of it were a keypoint's `position`, a keyframe's `value` and a control
  point's `value` — the ones the review found — plus `Decoder::coordinates`, the mesh's point
  lists, whose `VertexCount` check ran *after* the capacity was reserved. A hand audit found that
  last one; the entry's own grep would have found every one of them without being asked. Task 3
  already reported the script's absence, and this is what that absence cost.
- **`docs/format.md` *Validation* folds the Bézier control-point rules into one bullet** — *a
  Bézier segment's control times inside the segment, and every control value finite* — and the
  two are not one rule. The time rule is meaningless without a segment and so is properly gated
  on the mode; the value rule is not, and gating it on the mode is exactly the defect this pass
  fixed. The list should carry them as two lines with the second stated unconditionally: **every
  control point's value is finite, whatever mode its keyframe declares**. Task 3 already reported
  that the list names no rule for the value at all.
- **`docs/format.md` *Validation* is silent on a control point on a non-Bézier segment**, which
  the reader accepts and writes back (task 3's assumption, unchanged). The new fixture makes that
  silence load-bearing: it is a file the reader keeps, refused only for the non-finite number in
  it. If such a keyframe should instead be refused, that is a document change and a rule, not an
  edit here.
- **`docs/format.md` *Load limits* lists the limits on animation but not the one on a coordinate
  list**, which is new here and is the only limit in `limits.rs` whose figure is deliberately
  chosen so as *not* to answer a question the same document marks `[open]`. That relationship —
  a limit sized to stay clear of an open rule — is worth a line, because the next reader
  tightening figures would otherwise read 1024 as slack.
- **`PreservedMembers::path`'s own doc comment said only `axes` and `keyframes` are addressed by
  position.** `values`, `triggers` and `control_points` are too, and `values` already was when
  the comment was written — `save.rs`'s `repath` has always existed to move it. Corrected in
  place, since it is the field's own comment.
- **`crates/rigel-fixtures/README.md` said two figures the crate invents and listed them.**
  Corrected in place to point at the constants instead, and to record that the trigger and
  coordinate figures are now in step with `limits.rs` rather than ahead of it. The
  `PROVISIONAL_NODE_LIMIT` staleness it records is unchanged and still owed.

**Traps:**

- **`Writer` does not canonicalise.** `save`'s one entry point calls `canonicalise` and then
  `Writer`, in that order, and a unit test that builds a `Writer` directly sees the puppet
  exactly as it was assembled. A test written to check canonical order through `Writer` alone
  passes or fails for reasons that have nothing to do with the writer.
- **A fixture carrying `1e400` cannot be asserted on through `serde_json`.** `parsed()` in
  `rigel-fixtures`'s tests panics on it — the number is out of range — so a unit test over such a
  fixture reads the entry's text. `rigged:positive-infinity` has the same property for the same
  reason.
- **`decode.rs` already had a method called `coordinates`**, the mesh's pair lists. The new
  single-value helper is `coordinate_list`; the collision is silent until the argument counts
  disagree.

**Assumptions:**

- **A keyframe may carry `control_points` without declaring Bézier, and that stays accepted.**
  Unchanged from task 3, and now the premise the new fixture rests on: it is refused for its
  non-finite number and for nothing else. Stated in *Contradicts docs* as a gap in the document.

**Deferred:**

- **Writing the `script` concern entries for the limits-completeness and allocation walks.** Still
  absent, still a task of its own, and now with a measured cost attached — see *Contradicts docs*.

**Tried and rejected:**

- **Keying a trigger's preserved path by event name instead of by position.** Reasons under
  *Decisions made*. Recorded so it is not re-proposed: it is a reasonable design and it loses on
  having two addressing schemes where one does.
- **Sorting triggers in both the decoder and the writer.** Correct and harmless, and it leaves
  the writer's repath unexercised by every file the corpus holds, since the decoder would have
  sorted them already.

> **Superseded above:** *Tried and rejected*'s **Sorting an animation's triggers in the writer
> rather than in the decoder** and **Folding a non-finite Bézier control value into
> `Rule::NumbersFinite`**. The first is now what the writer does — the objection was that a
> preserved path carries an index, and `canonicalise` already moves indexed preserved material.
> The second stands as written: control points are still outside the `numbers()` walk and still
> have their own rule. What changed is only *which keyframes* that rule is applied to.

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
