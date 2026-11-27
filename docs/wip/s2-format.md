# Feature: `rigel-format` — read, validate, write

**Branch:** `feature/s2-format`
**Phase subsystem:** S2 in [plan.md](plan.md)

Fills `crates/rigel-format`, today a doc comment, with the three things a puppet file needs: a
reader that treats the file as hostile input, a validator that is the single gate on both load
and save and can also be run standalone, and a writer that produces canonical bytes through one
atomic path. No evaluation, no GPU, no window, no application. Done when a puppet archive
round-trips byte-identically, the hand-authored malformed corpus is rejected with stale bindings
accepted as a state, load limits bite before anything is allocated, and extension data survives
untouched.

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** tasks 1 to 6 are `done`, as is the encoding promotion that stood between
tasks 1 and 3 — see the section above the task list. **Next: task 7, the writer.**
**Blocked on:** nothing.

**Gate 2 is red from here**, as designed: task 3 wrote the acceptance tests against a declared
surface whose every body is `unimplemented!()`, and they fail. Tasks 4 to 7 turn them green.
After task 6 **every remaining failure is at an `unimplemented!()` in `save.rs`**, and nothing
ends inside `tests/` — the two that did after task 5, `EXEC-5` and `CANON-2`, are the validator
wearing an assertion and `EXEC-5` now passes. Task 6's handoff carries the table.

**Deferred out of task 1 and now closed:** `scripts/fixtures-check.sh` is wired into gate 2.

**The branch is expected to be red from task 3 until task 7.** Acceptance tests are written
before the implementation and are confirmed failing by being written
([plan.md](plan.md#the-tests-and-why-they-are-not-written-yet-decided)). Push gates block a
*merge*, not a commit, so a red gate 2 mid-branch is the designed state rather than a defect to
chase. It must be green before the PR merges.

---

## Assignment

### Goal

Implement `rigel-format` so that a puppet file can be read, validated and written per
[format.md](../format.md), against acceptance tests written from the requirements without sight
of the implementation.

### Out of scope

Named explicitly, because every one of these has a criterion in
[puppet-acceptance.md](../requirements/puppet-acceptance.md) that reads as though it belongs here.

| Out | Why, and where it lands |
|---|---|
| **Evaluation of any kind** | S4. Nothing here poses a tree, composes a deformation, or steps a simulation. `DET-1`…`DET-5`, `PHYS-2`, `PHYS-3`, `PHYS-7`, `TRACK-2`, `CHAIN-3`, `ART-11`, `FILE-8` are S4's |
| **Anything GPU or window** | S5. `TEX-1`, `TEX-2`, `TEX-3`, `COLL-14` need a device or resident-byte accounting |
| **The generated workloads** | S3. `TOOL-1` and `LIM-4`'s *4× the `stress` dimension* figure cannot be asserted until the generator exists. `LIM-1` and `LIM-8` — that a limit exists, is declared in one place and has a just-over fixture — **are** in scope |
| **Every editor operation** | Studio, next cycle. The whole `ART-*`, `COLL-*`, `EDIT-*`, `MAXIS-*`, `MIR-*` surface, plus `CANON-4`, `CANON-6`, `TEX-7`, `POL-2`, `FILE-3`, `FILE-4`. In-memory mutation sufficient to serve `ID-2`, `ID-3` and `ID-4` is in scope; an undo stack is not |
| **Channel mapping behaviour** | Live. The stored shape round-trips here; `CANON-5`'s *change a mapping in the player* half does not |
| **Upper bounds on physics constants** | `[open]` in [format.md](../format.md), and due with S4 as a measurement. `VAL-25` validates sign and finiteness, which is what the document commits to today |

### Doc set

The implementer reads these and no others. The rest of `docs/` is deliberately withheld —
routing what this work turns up is the dispatching session's job, not the implementer's.

- `CLAUDE.md` — current state and invariants
- [vocabulary.md](../vocabulary.md) — **load-bearing**, read first
- [invariants.md](../invariants.md) — what must never break. **I1, I3, I4 and I7 all bite here**
- [format.md](../format.md) — the specification this implements. The primary document
- [requirements/puppet.md](../requirements/puppet.md) — what the format is *for*, where a
  specification question needs the reason behind the rule
- [requirements/puppet-acceptance.md](../requirements/puppet-acceptance.md) — the criteria and
  the fixture catalogue
- [testing.md](../testing.md) — the harness, where each kind of test lives, and the
  bare-nested-directory trap
- [stack.md](../stack.md) — the crate split, the push gates, why `rigel-format` builds with no
  GPU and no toolkit

### Acceptance criteria

From [puppet-acceptance.md](../requirements/puppet-acceptance.md). Referenced, never restated —
if a criterion here and that document disagree, that document wins and the disagreement is a
finding.

| Group | Criteria |
|---|---|
| Identity and references | `ID-1` … `ID-9` |
| Version and compatibility | `VER-1` … `VER-6`, `TOOL-4`, `MSG-1`, `MSG-2` |
| Extension and unmarked data | `EXT-1` … `EXT-9` |
| Validation | `VAL-1` … `VAL-26`, `ART-5`, `TRACK-1` |
| Load limits | `LIM-1`, `LIM-2`, `LIM-3`, `LIM-5` … `LIM-8` |
| Nothing is executed | `EXEC-1` … `EXEC-5` |
| Canonical bytes | `CANON-1`, `CANON-2`, `CANON-3`, `CANON-7` |
| Stored shape round-trips | `FILE-1`, `FILE-2`, `FILE-5`, `TEX-4`, `TEX-5`, `FLIP-1`, `FLIP-2`, `POL-1`, `DET-6`, `CHAIN-1`, `CHAIN-2`, `CHAIN-4`, `PHYS-1`, `PHYS-4`, `PHYS-5`, `PHYS-6`, `PHYS-8`, `PHYS-9`, `VAL-19` |
| Third-party tooling | `TOOL-2`, `TOOL-3`, `VAL-13`, `VAL-14` |

**Two criteria are listed in that document as writable and not yet written**, in its
[*Criteria not written*](../requirements/puppet-acceptance.md#criteria-not-written-open) table:
an animation carrying two tracks for one parameter, and a puppet carrying no gravity. Both are
this subsystem's and `acceptance-planner` writes them in task 3, into
`puppet-acceptance.md` alongside the criteria they belong with.

### Concern criteria

From [concerns/](../concerns/), via the *Concern entries carried* table in
[puppet-acceptance.md](../requirements/puppet-acceptance.md#concern-entries-carried-decided).
These are specifications for this work, not checks that happen to it later.

- **data** — one validator gates load and save and runs standalone; unmarked data rejected and
  marked extension data round-trips; unknown fields and entities survive a round trip; atomic
  write with failure injected at each step; saving an unmodified puppet is byte-identical;
  physics state never serialised; every node kind carries a containment declaration and a
  declaration per reference kind.
- **security** — hostile fixtures rejected cleanly; load limits in one place with a fixture just
  over each; every stored collection and string has a declared limit; physics constants
  range-validated before anything is allocated; a track's keyframe times, values and blend
  durations validated; nothing supplied by a file is executed; every parser of untrusted input
  survives hostile input; a failed load leaves the displayed puppet unchanged
  ([I4](../invariants.md) — here, the loader must not be capable of partial replacement).
- **ops** — gate 2 fails when the workspace runs zero tests. S2 is the first subsystem with
  tests to lose, which is why that check is this branch's and not a later one's.
- **security**, inherited and due with the first registry dependency, which this branch adds —
  every `uses:` in `.github/workflows/` pinned to a 40-hex commit SHA, and the locked dependency
  graph containing no network-client crate and no git or alternative-registry source.

### Already ruled out

- **Deriving the acceptance tests from [format.md](../format.md).** The criteria were written
  from [puppet.md](../requirements/puppet.md) alone and deliberately without sight of the design;
  a test derived from the design can only establish that the design was implemented. The
  test-writing agent *implements* the existing criteria and may read `format.md` for the
  encoding, because a test that loads a file has to know one.
- **Building the fixtures with project code.** A fixture produced by our writer and read by our
  reader asserts only that we agree with ourselves. `rigged` and its defect variants are
  hand-authored from the specification using third-party zip and JSON only. This is the same
  rule [puppet-acceptance.md](../requirements/puppet-acceptance.md) already puts on the generated
  workloads.
- **The implementer writing the acceptance tests.** Tests written by the author of the code
  encode the code's assumptions ([testing.md](../testing.md)).
- **Canonicalising extension data on save.** It looks like the obvious reading of "canonical
  serialisation", and it is wrong:
  [puppet.md:403-408](../requirements/puppet.md#the-same-puppet-saves-to-the-same-bytes-proposed)
  settles it. Preservation governs *contents* — preserved material is written back verbatim.
  Canonical ordering governs *position*, keyed on the marker a build can read without
  understanding the contents. Re-emitting a stranger's `x` subtree through our own number
  formatter would silently damage values we by definition do not understand.
- **A separate authoring payload in the archive.** Withdrawn 2026-11-27;
  [decisions.md](../decisions.md) carries why, and `FILE-6`/`FILE-7` are retired.
- **Trusting a zip entry's declared decompressed size.** It is attacker-controlled. The limit is
  enforced against bytes actually decompressed; the declaration is used only to refuse early.

---

## Decisions

Made for this feature. Anything meeting [decisions.md](../decisions.md)'s admission test moves
there immediately — a narrow set. This list is everything else, and it survives every task
cleanup.

- **The licence was settled before this branch opened** — `MIT OR Apache-2.0`, owner's choice,
  landed on `master` in its own change. It is here only because this branch adds the first
  registry dependencies, whose licences must stay compatible with it.
- **A red gate 2 between tasks 3 and 7 is expected**, for the reason under *Status*. If it is
  still red at task 8, that is a defect.
- **`PHYS-4`'s test was narrowed to its own criterion.** It asserted
  `loaded.report.warnings.is_empty()` for `rigged`, while `MSG-1` asserts the warnings for
  `rigged:ext-unknown` are **not** empty — and `rigged:ext-unknown` is `rigged` plus one more
  namespace. `rigged` already carries extension data no build understands, at two places in the
  tree, because [the fixture catalogue](../requirements/puppet-acceptance.md#the-fixtures-proposed)
  requires it. **The two tests could not both pass**, whatever the implementation did.
  `MSG-1` is faithful to its criterion; `PHYS-4`'s criterion says the node driving nothing is
  *neither rejected, nor reported as a defect, nor repaired* — a claim about that node, not about
  the whole load. The assertion now says exactly that: the driverless node appears in no defect
  list and no warning names it. The second of the two test edits on this branch, and the same
  class as the first — an assertion that was false by construction of the fixture, corrected
  rather than relaxed. `PHYS-4` still fails, at the validator, with its peers.

- **`EXT-8`'s test was corrected, not relaxed.** It read node `11` for material the fixtures
  carry on node `12`, so its own opening assertion — *the fixtures carry the material this
  criterion is about* — failed and the criterion tested nothing. It now finds the node by the key
  it carries, which is what `rigel-fixtures`'s own differential test does and is robust to the
  corpus being regenerated. **This is the one edit made to an acceptance test on this branch**,
  and it is a correction of a factual error about fixture contents rather than a judgement about
  behaviour: the test asserts strictly more after it than before. The rule it sits against —
  never edit a test to make it pass — is intact, because this edit makes a test that could never
  pass able to fail for the right reason, which it now does, at the loader.

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- **Encoding the node tree as nested `children`.** It is the more legible shape and it is wrong
  here: a nested tree cannot express a cycle, a second root or no root at all, so `rigged:cycle`,
  `rigged:two-roots` and `rigged:no-root` would have no fixture and *the tree is acyclic with one
  root* nothing to reject. The tree is a flat `nodes` array, each node naming its parent by
  identifier, with the puppet naming its root.
- **A fault-injection point behind a cargo feature.** `VER-5` needs the save to fail at each
  step, and a `#[cfg(feature = "…")]` hook reads as the tidier shape. It is not: the acceptance
  suite is a separate crate target, so the feature would have to be on for an ordinary
  `cargo test --workspace` — at which point it is a public function with extra ceremony.
  `save_with_injected_failure` is public and says in its doc comment why.
- **Asserting `VAL-13`'s standalone command on its exit status alone.** A panicking stub exits
  non-zero, which is the correct verdict for every rejected fixture in the corpus — so half the
  assertions would pass against nothing at all. The command's verdict goes to stdout and the test
  reads that.
- **Inventing provisional load-limit figures so `LIM-1` goes green.** It would turn one test and
  unblock several more, and it would make a criterion about a measurement read green over a
  measurement nobody has taken — `LIM-4` says what each figure is and S3 is what produces it.
  The rows exist with `None` figures instead, so the set is complete and the absence is visible.
  The counter-argument is real and is recorded in *Contradicts docs*: without figures the branch
  cannot be fully green at task 8. If that is answered by stating provisional figures, they go
  in `limits::FIGURES` and nowhere else.
- **Giving `Puppet` the version it was loaded at, so `validate(&Puppet)` can gate an unfamiliar
  enumerated value on it.** It is the obvious fix and it contradicts a decision already taken —
  *a save decides whether it is upgrading by reading the file it is replacing* — which exists
  because a puppet assembled in memory has no version to carry. The gate stays where the
  declaration is readable: `load` and `validate_file`.
- **Keying `may_contain` on the parent's row alone.** Simpler, and it answers `Some(false)` for
  a child kind nobody has declared — which is `VAL-24` reading green over the omission it exists
  to catch.
- **Hand-rolling the whole archive writer to keep the `zip` crate out of the locked graph.** It
  would shrink the dependency set, and it would remove the only independent check that these
  files are real zips — which is exactly what task 9's gate rests on. The `zip` crate writes
  every canonical fixture; hand-assembled bytes are used only where the fixture's purpose is a
  malformed container, and no library will emit one of those.

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

- *(none yet)*

---

## The encoding went into `format.md` — **done**

Task 1 turned up the one thing that changed this plan, and it is recorded here rather than
inside a task because it sat between them. It is **closed**; this entry stays so nobody reopens
it.

[format.md](../format.md) named JSON keys for the physics fields and for extension data **and for
nothing else**, so encoding `rigged` meant settling the key names, the tree encoding, the
collection orders, the current version number, the compression per entry kind and the gravity
default — and they got settled in `crates/rigel-fixtures/README.md`, which is not a document
anybody is told to read. That is the failure [vocabulary.md](../vocabulary.md) describes: terms
invented by an implementer do not propagate, they collide.

They now live in [format.md](../format.md), under the `[proposed]` sections they belong to, and
the builder's README points at it. These were gaps rather than promotions, and were decided here
rather than carried across:

| Settled | Where |
|---|---|
| A puppet file's extension is `.rigel2d` | *The container is an archive with a manifest* |
| An upgrading save keeps the previous bytes as a `.bak-<major>.<minor>` sibling | *The upgrading save keeps the previous bytes*, a new subsection. `VER-4` and `VER-5` required one and the document described none |
| The canonical-zip list asked for "no external attributes", which no Unix-written entry can satisfy | *Canonical serialisation* — now pinned to `0644`, which is what the rule was protecting |

The animation and track rules also moved out of *What the simulation drives*, a physics
subsection they did not belong in, into a section of their own.

**Task 3 is unblocked.** It could not be commissioned before this landed, because a test that
loads a file has to know the encoding. Task 2 is independent and may go either side.

**The other `Contradicts docs` findings are routed at task 10, not now**, with one exception
already closed: every member crate now inherits the workspace `license`.

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The hand-authored fixture corpus — `done`

`rigged` and every defect variant the in-scope criteria name, built by a committed builder that
uses **no project code** — third-party zip and JSON only, driven from
[format.md](../format.md). Lives where [testing.md](../testing.md) puts fixtures; the builder is
committed beside them so a fixture can be regenerated rather than hand-patched.

This task adds the workspace's first registry dependencies, which is what makes task 2 due.

**Done when:** every fixture named by an in-scope criterion exists and is reproducible from the
builder; `rigged` carries everything the catalogue says it carries; each defect fixture differs
from `rigged` in exactly one way, so a rejection cannot be attributed to the wrong rule; every
fixture declares the current format version unless its name says otherwise. No user gate — this
is checkable by reading the catalogue against the builder.

### 2. Dependency and workflow gates — `done`

Due with the first registry dependency, not later ([plan.md](plan.md)). A check that every
`uses:` in `.github/workflows/` is a 40-hex commit SHA, and a check that the locked dependency
graph contains no network-client crate and no git or alternative-registry source. Both as
scripts, wired into gate 5's job.

**Done:** `scripts/actions-pinned.sh` and `scripts/locked-graph.sh`, both wired into gate 5, and
`scripts/tests-ran.sh` into gate 2 alongside `scripts/fixtures-check.sh` — the latter deferred
out of task 1 because wiring it needed this same workflow edit. Each carries a `--self-test` that
plants every forbidden and every allowed form and runs before its real scan, matching
`no-outbound.sh`; each fails rather than passes when it finds nothing to look at.
[stack.md](../stack.md#on-push-proposed)'s gate table and the `security` and `ops` entries are
updated.

Gate 2 now runs the suite through `scripts/test-record.sh` so the step after it reads the record
rather than re-running the suite — `.claude/rules/testing.md` makes results state on disk. That
also starts producing the record the release entry in [ops.md](../concerns/ops.md) wants and
notes nothing yet produces.

**Still open in those entries, deliberately:** a vulnerability scan and a place to record a triage
decision. Both are `finish-cycle`'s, not this task's.

### 3. Acceptance tests — `done`

Dispatched to `acceptance-planner`, **which must not be shown an implementation** — there is
none, which is why this task is here rather than after task 7. It implements the in-scope
criteria into `tests/acceptance/`, adds the `main.rs` that stops cargo silently skipping the
directory, and writes the two criteria that
[puppet-acceptance.md](../requirements/puppet-acceptance.md#criteria-not-written-open) records as
writable and unwritten.

Also the `ops` check that gate 2 fails on zero tests — it has a subject from this commit onward.

**Done when:** every in-scope criterion has a test or a stated reason it has none; the suite
compiles; the tests **fail**, which is what confirms them. No user gate.

**Done:** `crates/rigel-format/src/` is the declared surface, `tests/acceptance/` the suite. The
tests that have no criterion behind them, and the criteria that have no test, are both in the
handoff below. `scripts/tests-ran.sh` already had its subject from task 2; it now has tests to
lose as well as a record to read.

### 4. Core types, identifiers and the containment table — `done`

**This task's shape changed at task 3.** It no longer designs the in-memory puppet — task 3
declared it, because tests cannot compile against nothing. What is in `crates/rigel-format/src/`
is a **contract**: types, errors and signatures, doc-commented with the criterion that drives
each, and every body `unimplemented!()`. Task 4 implements the contract it was handed.

Implement: the entities, the unsigned 64-bit identifier with its high-water mark and reserved
zero, the per-kind containment declaration as **data the validator reads** rather than rules
naming kinds, and mutation sufficient for `ID-2`, `ID-3` and `ID-4` and no more.

**A signature may be wrong, and changing one is allowed** — it was derived from the criteria by
an agent that had implemented nothing. What is *not* allowed is changing a **test** to fit the
code. If a test is wrong, that is a finding and it goes in the handoff.

**Done when:** `VAL-24` passes — a node kind with no containment declaration fails a check rather
than inheriting a default — and the identity tests in `tests/acceptance/identity.rs` pass. No
user gate.

**The second half of that was unsatisfiable and is carried to task 7.** Every test in
`identity.rs` loads a fixture and saves it, so none of them can pass before the reader and the
writer exist. `VAL-24` passes. See *Contradicts docs* below.

**Why `may_contain`, `may_be_root`, `declared_range` and `declared_limit` return `Option`**, which
looks over-engineered until you need it: `VAL-24`, `VAL-25` and `LIM-1` each require an
*undeclared* answer to fail a check rather than default to false. A `bool` cannot distinguish
"declared, and forbidden" from "nobody declared this", which is the whole point of each of the
criteria named above.

### 5. The reader — `done`

Archive and manifest, the version gate (`rigel2d.json` read on its own, before any load limit is
at risk), load limits enforced **before allocating**, and the parsed source kept alongside the
typed structures so unknown material survives. Treats the file as hostile throughout.

**Done when:** the version, extension, limit and preservation criteria pass; `I4` holds
structurally — a failed load cannot have replaced anything, because the loader produces a value
rather than mutating one. No user gate.

**Done**, in four modules — `json.rs`, `archive.rs`, `decode.rs` and the body of `load.rs` — and
the figures in `limits.rs`. `I4` holds structurally and nothing in the loader writes into a
puppet a caller already holds.

**Only the criteria whose refusal is reached *before* validation can go green here**, and they
do: the version gate, the archive faults, the load-limit refusals, and the three rule families a
`Puppet` cannot hold — unmarked data, an unfamiliar enumerated value, and the absent-member
rules. Everything that needs a **successful** load still ends at `validate`, which is task 6's
and is still `unimplemented!()`.

**Against `cargo test -p rigel-format --test acceptance`, run once per test with `--exact`:**

| Where it ends | Tests |
|---|---|
| passes | 11 |
| `src/validate.rs:261` — `validate`, task 6 | 67 |
| `src/validate.rs:271` — `validate_file`, task 6 | 4 |
| `tests/acceptance/execution.rs:154` — `EXEC-5` | 1 |
| `tests/acceptance/canonical.rs:159` — `CANON-2` | 1 |

The two that end inside `tests/` are both the unimplemented validator wearing an assertion.
`CANON-2` runs the test binary again as a child to vary the environment, and the child dies at
`validate`; the parent reports only that the child failed. `EXEC-5` loads 256 single-byte
mutations of `rigged` under `catch_unwind`, and twenty of them land in zip header fields no
loader reads — modification times, external attributes, version-needed, and local-header fields
the central directory supersedes — so those files load successfully and then panic at `validate`,
which the test records as a panicked round. Both go green with task 6 and neither was touched.

The 69 unit tests in `src/` all pass, including a catalogue-driven sweep asserting that **no
fixture the catalogue calls valid is refused by the reader**.

**Task 5 states the load-limit figures.** `limits::FIGURES` has a row per `LimitKind` and no
figure: task 4 declined to invent a measurement, which was right. Task 5 is the task that
*enforces* a limit, so it is the task that must say what it enforces. Without figures `VAL-13`
stays red too, because `rigged:decompression-bomb` and `rigged:entry-name-overlong` are
catalogued `rejected` and nothing can refuse them against a threshold that does not exist.

**Most figures are derivable today, and are not a judgement call.**
[LIM-4](../requirements/puppet-acceptance.md#j-load-limits-proposed) fixes every limit at **4× the
corresponding `stress` dimension**, and those dimensions are already committed in
[platform.md](../requirements/platform.md#the-reference-puppet-and-the-stress-puppet-proposed) —
S3 generates the *puppet*, not the table. Derive, do not invent. `LIM-4` asserts the figure
exactly rather than "at or above", so a limit that is merely generous fails it.

**Two traps in that table**, both stated in it and both easy to miss. The *Nodes* row excludes
physics, so a node limit must use the **sum** of the Nodes and Physics nodes rows — `LIM-4` says
so explicitly. And the *Bindings* row is marked **understated**.

**Provisional figures are needed only where that table has no row**: decompressed payload size,
tree depth, string length, total load memory, tracks per animation, keyframes per track. Mark
those provisional **in `limits::FIGURES` and nowhere else**, so whoever lands the real figure
replaces a table rather than hunting constants — `PROVISIONAL_NODE_LIMIT` in `rigel-fixtures` is
the precedent. Tracks and keyframes have no dimension because the corpus counted `.motion3.json`
files rather than the curves inside them; [open-questions.md](open-questions.md) carries it and
it is due before S4.

**The finding against [plan.md](plan.md), routed at task 10**, is narrower than it first looked:
not that S2 must ship provisional figures for everything, but that the plan does not record which
limits have no workload dimension to be derived from, nor that S2 is where that gap first bites.

### 6. The validator — `done`

The single gate on load and save, plus the committed JSON Schema for what a schema can express
and the standalone command `VAL-13` needs. Every rejection names the rule broken and at least one
offending identifier, in bounded output.

**Done when:** the whole malformed corpus is rejected and stale bindings are accepted as a state;
the standalone verdict equals the embedded one on every fixture. No user gate.

**`validate(&Puppet)` cannot apply the version gate, and must not pretend to.** An unfamiliar
enumerated value is legal or not depending on the file's **declared version**
([format.md](../format.md)), and a `Puppet` carries no version — deliberately, because a save
decides whether it is upgrading by reading the file it replaces. So `Rule::EnumeratedValueKnown`
is reachable only from `load` and `validate_file`, and `validate(&Puppet)` admits `Unfamiliar`
unconditionally. Otherwise `VER-1` loads a later-minor file and then cannot save it, which is
exactly the compatibility rule inverted. `VAL-13` compares the standalone verdict with the
embedded one and does **not** catch this, because both go through `validate_file`.

**Done**, in `validate.rs`, `src/bin/rigel-validate.rs` and `schema/`. Every fixture reaches the
outcome [the catalogue](../../crates/rigel-format/tests/fixtures/catalogue.json) records for it,
through `validate_file` and through `load` alike, asserted as a sweep beside the reader's.
`load` and `validate_file` now share one `verdict`, so `VAL-13`'s *the same validator* is
structural rather than a resemblance. The schema and the document stating its boundary are in
`crates/rigel-format/schema/`, held against the reader's member lists by a unit test.

**Every remaining acceptance failure is at an `unimplemented!()` in `save.rs`** and is task 7's.
Nothing ends inside `tests/`.

### 7. The writer — `todo`

**Read task 6's *Traps* first.** A reordered puppet is valid in memory and the writer is what
canonicalises it, which decides the shape of a save — `ID-3` is the test that settles it, and it
has never run.

Canonical JSON and canonical zip — every default a zip writer brings pinned — the one atomic
write path, and the version-upgrade backup. Preserved material written back verbatim, positioned
by its marker.

**Done when:** round-trip byte-identity holds; a failure injected at each step of the write
leaves the previous file intact ([I7](../invariants.md)); the saved bytes carry nothing about the
machine that wrote them. No user gate.

### 8. Hostile input — `todo`

A time-boxed fuzz run over the parser, and the zip-level hostile fixtures: decompression bomb,
over-long entry name, duplicate entry names, overlapping local-header data, an entry name the
format does not define.

**Done when:** no crash, no hang, bounded memory; each hostile entry is refused before anything
is decompressed. No user gate.

### 9. The third-party gate — `todo` *(human gate)*

The one thing in S2 a person must settle, and the whole case for the format being an archive.

**What to look at:** `rigged`, given only [format.md](../format.md) and ordinary command-line
tooling — `unzip`, a text editor, an image viewer. Inspect it, change one value, repack it,
reload it in the validator. Separately, extract one texture payload and open it on its own.
Then read the load message for `rigged:future-minor` and for a file carrying extension data this
build does not know.

**What would count as wrong:** needing anything beyond the specification and those tools to
understand the layout; a repacked file being refused for a reason the specification does not
state; a payload that will not open standalone; a message that promises the file will be
preserved when it will not, or that implies an install route for a missing extension when none
exists.

**Not gated on the owner re-running the suite.** Everything deterministic is already asserted in
tasks 1–8; this gate exists for the judgment those cannot express.

### 10. Finish — `todo`

Review (`code-reviewer`, and `concern-analyst` in review mode for data, security and ops), route
the handoff, `scripts/docs-check.sh`, move [plan.md](plan.md) on, delete this file. Steps 4 and 5
share one commit, the last before merge.

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**

Task 1. `crates/rigel-fixtures`, a workspace member that depends on no other crate here and
writes the corpus to a directory given on the command line, and the corpus itself at
`crates/rigel-format/tests/fixtures`. `scripts/fixtures-check.sh` regenerates into a temporary
directory and asserts the committed bytes are reproduced exactly; each of its failure paths — a
changed artifact, a missing one, a stray one — was run against a broken tree.
`crates/rigel-fixtures/README.md` records the
encoding decisions the specification left open and is the document tasks 3 to 7 need. The
builder's own tests check the `rigged` catalogue row clause by clause, the current-version rule
across the whole corpus, canonical JSON, the differential pairs EXT-8 and VAL-20 rest on, and
that nothing but `rigged:ext-hostile` carries a path-, URL- or command-shaped string. The corpus
also carries `catalogue.json`, mapping each catalogue name to its file, the expected load outcome
and its one distinguishing feature, so a test reaches a fixture by name rather than by path.

Task 3. The **public surface of `rigel-format` as a contract** — `puppet`, `version`, `load`,
`validate`, `save`, `limits` and `containment`, plus the `rigel-validate` binary `VAL-13` asks
for. Every body is `unimplemented!()`; every item carries what it must do and the criterion that
drives it. Derived from the criteria, not from the fixtures and not from convenience: where a
criterion asks a rejection to name the rule broken and at least one offending identifier, the
error type carries both.

The **acceptance suite** at `crates/rigel-format/tests/acceptance/`, with the `main.rs` that stops
cargo silently skipping the directory. One module per group of the criteria document, and every
test names the criterion it implements in its name and in a comment above it. Fixtures are reached through `catalogue.json` **by name**, never
by path, so the `:`-to-`.` renaming on disk lives in one place. `zip`, `flate2` and `serde_json`
are dev-dependencies of `rigel-format` at the versions and features `rigel-fixtures` already
pins, so the locked graph gained nothing.

**Under `cargo test --workspace --locked`, 83 of the suite's 84 tests fail, every one at an
`unimplemented!()` in the crate** — not at an assertion
about what a stub does, which is the failure mode that would make them worthless. The one that
passes is `version::the_corpus_and_the_build_agree_on_the_current_version`, and it is not a
criterion: it asserts that `catalogue.json`, `rigged`'s `rigel2d.json` and
`FormatVersion::CURRENT` name one version. That is the premise `VER-1` … `VER-6`, `EXT-7`,
`EXT-8` and `VAL-20` all rest on — the gate those criteria test *is* the declared version — so a
drift there would make them silently vacuous rather than fail.

The two criteria [puppet-acceptance.md](../requirements/puppet-acceptance.md) recorded as
writable and unwritten are written: **`VAL-27`**, an animation carrying two tracks for one
parameter, beside the other validation criteria; **`PHYS-10`**, a puppet carrying no gravity,
beside `PHYS-6`'s one-gravity rule. Their rows are gone from *Criteria not written*. No existing
criterion was touched.

Task 4. The half of the contract that needs no file I/O, implemented against the contract task 3
declared. `puppet.rs` — `Puppet::identifiers`, `Puppet::issue_identifier`, `NodeBody::kind`,
`BindingValues::len`/`is_empty`. `containment.rs` — `containment::DECLARATIONS`, the table
`may_contain` and `may_be_root` read. `limits.rs` — `limits::FIGURES`, one row per `LimitKind`,
every figure `None`. `validate.rs` — `validate::RANGES`, which `declared_range` reads.
`version.rs` needed nothing; `FormatVersion::CURRENT` was already a constant.

The three enumerated fields the core format defines in typed form — `AxisPolarity`,
`OutputSource`, `CompositionMode` — are **open**, each with an `Unfamiliar(String)` variant
carrying the token verbatim, per *An unfamiliar value in a field this build knows*. That is the
gap task 3 reported as most likely to block task 5, closed in the types.

Unit tests beside each module, covering the tables' totality, the sign rules, the identifier
mark across a deletion, and that the unfamiliar inhabitant is distinct from every value the
format defines — `AxisPolarity::Unknown` most of all.

`load.rs`, `save.rs`, `validate::validate`, `validate::validate_file` and `rigel-validate` are
untouched and still `unimplemented!()`.

Task 5. The reader, and the four modules it needed.

`json.rs` — a strict JSON reader and a canonical writer, crate-internal like `archive.rs` and
`decode.rs`, with **every number kept as the text it arrived as**. Written rather than taken from a crate because the two requirements no general
JSON library meets together are exactly this format's: a stranger's number must survive a save
byte for byte, and `1e400` inside opaquely preserved material must come back as `1e400` rather
than as `inf` or as a parse failure (`EXT-9`), while the same literal in a field this build
interprets must overflow to infinity so `VAL-6` has something to reject. Strict about everything
JSON does not define — the bare `NaN` and `Infinity` tokens, trailing commas, control characters
in strings, invalid escapes and invalid UTF-8, a duplicate member name, and nesting past a depth
cap — because a file is traded between strangers.

`archive.rs` — the container, treated as hostile. Everything in `open` is read from the **central
directory alone and decompresses nothing**: the entry count, entry-name lengths, duplicate names,
names the format's table does not define, overlapping regions, and a required entry absent.
`read` is where a byte is first produced and where the payload limits bite; it reads the **raw**
entry and inflates it under a cap of its own, because the `zip` crate stops at the size the entry
declares and the declaration is the one thing a bomb controls. Every entry's CRC is checked
against the directory's.

`decode.rs` — `puppet.json` into a `Puppet`, keeping two failures apart: a **shape** this build
cannot read at all is `Rejection::Malformed`, and a **rule** the file breaks is a `Violation`.
Three families of rule are raised here because a `Puppet` cannot hold the fact they are about —
`UnmarkedData` and `EnumeratedValueKnown`, both gated on the declared version, and the three
absent-member rules `AxisPolarityDeclared`, `ExtensionDeclaresVersion` and
`ExtensionDeclaresName`.

`load.rs` — `read_version`, `load`, the `Rejection` accessors and `Display` task 4 left
unimplemented, and a PNG **header** read for `VAL-10`'s undecodable payload. `read_file` is the
shared parse `validate_file` reuses in task 6, so the standalone verdict and the loader's stay
one verdict rather than two implementations that drift.

**The order `load` performs its steps in**, each reached only because the one before it passed:

1. the central directory, and every container fault, decompressing nothing;
2. `rigel2d.json` on its own, and the version gate — a differing major is refused whole here;
3. `puppet.json` under the load limits, and the parse;
4. the texture payloads, under the same per-entry limit;
5. validation.

Step 1 precedes step 2 although `format.md` leads with the manifest, because the manifest cannot
be found without walking the directory that names it — and those checks read nothing beyond that
directory, so nothing is decompressed before the version is known either way.

`limits.rs` — `FIGURES` now carries a figure, a `Basis` and a one-line note per row. `puppet.rs` —
`Preserved`, `PreservedMembers` and `OpaqueNode`, and `Puppet::preserved`. `validate.rs` — only
`Rule::as_str`, which every rejection the reader produces needs to name itself.

`zip` and `flate2` are promoted from dev-dependencies to dependencies at the versions and features
`rigel-fixtures` already pins. **`Cargo.lock` is unchanged**: the locked graph gained nothing.

**Nothing already in the contract changed shape.** Every edit to the public surface is an
addition: `Puppet::preserved` and the three types behind it, `Basis` and two fields on
`limits::Figure`, `version::FORMAT_NAME`, and an `as_str` on `LimitKind`, `ArchiveFault` and
`Rule` so that every rejection can name itself. No existing function's parameters or return type
moved, which is why the acceptance suite compiles untouched.

Unit tests beside each module: the JSON reader against what it must keep and what it must refuse,
the limit table against the committed `stress` dimensions and against the two fixtures a figure
has to bite on, the entry-name classifier, the rejection accessors, the PNG header read — and, in
`load.rs`, the reader against the real corpus, including a catalogue-driven sweep asserting that
**no fixture the catalogue calls valid is refused by the reader**.

Task 6. The validator, the standalone command and the committed schema.

`validate.rs` — `validate(&Puppet)`, which implements every rule in
[format.md](../format.md)'s *Validation* list and every rule `Rule` enumerates, except the three
families that are the reader's. `validate_file`, and `verdict`, which is the one place the
reader's violations and the validator's are combined and which **`load` now calls too**: the
loader's verdict and the standalone one are the same code rather than two that resemble each
other, which is what `VAL-13` is about. `PhysicsConstant::as_str`, because `VAL-25` asks the
rejection to name the constant.

`src/bin/rigel-validate.rs` — the command, against the contract its own doc comment already
declared: one path argument, `accepted` or the bounded rejection on stdout, exit `0`, `1` or `2`.

`crates/rigel-format/schema/` — `puppet.schema.json`, `rigel2d.schema.json`, and `README.md`,
which states **where the boundary falls**: a row per rule saying whether a schema can express it,
what the schema does not cover, and the three places it is stricter than this build's reader. The
schema expresses more than it first looks: the reserved zero as a minimum, the required polarity,
every enumerated list, `additionalProperties: false` for unmarked data, the physics sign rules,
a non-negative blend duration, and finiteness — as the single-precision bounds, which is
`VAL-18`'s rule stated rather than approximated.

Unit tests beside the module: one per rule, built by breaking a valid in-memory puppet in exactly
one way; the containment check driven over the square of `NodeKind::ALL` and the range check over
`PhysicsConstant::ALL`; and **two catalogue-driven sweeps** — every fixture reaches the outcome
the catalogue records for it through `validate_file`, and the loader's verdict equals the
standalone one on every fixture. Two more hold the committed schema against `decode.rs`'s member
lists and the boundary document against `Rule`, because a schema and a table in a document are
exactly the artifacts that drift unchecked.

**Decisions made:**

- **The current format version is `1.1`.** `rigged:old-version` must declare "an earlier version
  this build upgrades", and an earlier *minor* exists only if the current one is not the first.
- **The node tree is a flat `nodes` array with `parent` references and a top-level `root`.** See
  *Tried and rejected*. Canonical order is depth-first preorder, and order among siblings is draw
  order; every other collection is ordered by identifier, keypoints by position.
- **Every JSON key not named by `format.md` is the term `vocabulary.md` sanctions, in
  `snake_case`.** `format.md` names keys for the physics fields and for extension data and for
  nothing else, so encoding `rigged` meant choosing the rest. **Now promoted into
  [format.md](../format.md)**, which is where it governs; the builder's README points at it. This
  touched `format.md`'s `[proposed]` sections throughout, and none was promoted to `[decided]`.
- **Compression is Deflate for JSON and Stored for PNG**, at the zip crate's default level pinned
  by `Cargo.lock`; modification times are 1980-01-01; external attributes are a pinned `0644`.
  `format.md` requires each of these fixed and names a value for none.
- **Fixtures beyond those the criteria name.** `format.md`'s validation list states four rules
  that no criterion gives a fixture: the root being a kind that contains, at most one base driver
  per parameter, identifiers below the high-water mark, and the reserved zero. Each has one —
  `root-is-part`, `rigged:two-base-drivers`, `rigged:id-above-high-water-mark`, `rigged:id-zero`.
  The two criteria [puppet-acceptance.md](../requirements/puppet-acceptance.md#criteria-not-written-open)
  records as writable and unwritten have theirs — `rigged:two-tracks-one-parameter` and
  `rigged:no-gravity` — so task 3 has a subject when it writes them. `VAL-23` is table-driven over
  the containment declaration in both directions, so every pair the table forbids has a fixture
  and `rigged:containment-permitted` carries every pair it permits in one file.
- **Dependencies**, all `MIT` or `MIT OR Apache-2.0`: `zip` (the container, default features off
  to keep bzip2, lzma, zstd, aes-crypto and `time` out of the graph), `flate2` with the pure-Rust
  backend (zip's and png's deflate, and the bomb's), `png` (texture payloads), `serde_json` (its
  map is a `BTreeMap`, which is `format.md`'s sorted keys for free), `crc32fast` (the
  hand-assembled archives). The locked graph gained no network-client crate, no git source and no
  alternative registry — which is task 2's check, now with a subject.

- **`Rule` is a closed enum, one variant per rule the specification or a criterion states**, and
  a `Rejection` carries the `Violation`s that name them. **Because** a corpus of one-defect
  fixtures is only worth having if a rejection can be *attributed*: a test asserting nothing but
  *some error* passes against a build that refuses every file, and `VAL-12` exists to make the
  rule nameable. The granularity is a guess an implementer may want to change — if so that is a
  finding against these tests, not an obstacle to route around.
- **Rejections split into the faults reached before any entity exists and the rules broken after**
  — `Archive`, `IncompatibleVersion`, `LimitExceeded`, `Malformed`, and `Invalid(Vec<Violation>)`.
  **Because** `VAL-12` asks every rejection to name an offending identifier, and a container or
  JSON fault is refused before there is one. The split makes that a property of the variant rather
  than an exception in prose, and it is what lets `rigged:nan` be asserted honestly.
- **`may_contain`, `may_be_root`, `declared_range` and `declared_limit` all return `Option`.**
  **Because** `VAL-24`, `VAL-25` and `LIM-1` each say in terms that *no stated answer* must fail a
  check rather than inherit a default — and a `bool` cannot tell *declared false* from *nobody
  decided*.
- **`Violation` carries `field: Option<String>`** beside its offenders, because `VAL-25` asks the
  rejection to name the **constant** and `VAL-26` the rule, the animation and the track: an
  identifier list alone cannot say which field of a node was out of range.
- **`Rejection::MAX_MESSAGE_BYTES` is 4096**, an invented figure. `VAL-12` requires *bounded*
  output and names no bound; a message that grows with the file is a second denial-of-service
  route through the error path. Change it in one place if it is wrong.
- **`save_with_injected_failure` is public.** `VER-5` and [I7](../invariants.md) are claims about
  **every step** of a write, and the only way to assert them is to make each step fail. The
  acceptance suite is a separate crate target, so a `#[cfg(test)]` hook cannot reach it.
- **`rigel-validate` is the standalone command, and its contract is declared in its own doc
  comment**: one path argument, `accepted` or `rejected: …` on stdout, exit `0` or `1`, `2` on
  misuse. Nothing in `docs/` named the command `VAL-13` requires or said what it prints, and a
  test that asserted only the exit status would pass by accident against a panicking stub —
  a panic is a non-zero exit, which is the right answer for half the corpus.
- **`save_as(&mut Puppet, …)` exists and issues a fresh puppet identifier.** `ID-9` requires it
  and no document says where it lives; the format crate is the only place in S2 that can.
- **A save decides whether it is upgrading by reading the file it is replacing**, rather than the
  puppet carrying the version it was loaded at. **Because** [format.md](../format.md) describes
  the backup as *the bytes it read*, and a puppet assembled in memory has no version to carry.
- **`NodeBody` is closed over the four kinds**, and a node of a kind this build does not know is
  **not** in `Puppet::nodes`. `EXT-7` says removing it from the fixture changes *no reported
  entity*, so it is opaque material rather than a tree entry. `TOOL-4` and `EXT-7` both assert it,
  and both are the tests to re-read if an implementer disagrees.
- **`ExtensionEntry::data` is `Vec<u8>`, verbatim.** Holding it as parsed JSON would re-emit a
  stranger's numbers through our own formatter, which is what *Already ruled out* says must not
  happen — and it keeps a JSON crate out of the library's dependencies entirely.
- **`CANON-2` varies the environment in child processes**, re-running the test binary with
  `--exact`, rather than calling `std::env::set_var` and `set_current_dir`. Both are
  process-global: mutating the environment of a running test binary races every other test in it
  (the panic handler reads `RUST_BACKTRACE`), and changing the working directory changes it for
  all of them. The environment *is* this criterion's subject, so it has to vary somewhere it
  cannot hurt anything.
- **`EXEC-5` is a seeded mutation loop, not a fuzzer.** A time-boxed fuzz run is task 8's and does
  not belong in the default suite; the regression form of the same claim — a fixed seed, 256
  single-byte mutations of `rigged`, every one refused rather than panicked on — does, and it
  fails today for the right reason.
- **`LimitKind` covers every stored collection and string in the model**, not only the sizes the
  requirement enumerates. `LIM-8` says the governing sentence is *every size and count the loader
  trusts* and that driving the check from the format is what bounds a kind added later. The
  figures are S3's; the **set** is not.

Task 4.

- **`AxisPolarity`, `OutputSource` and `CompositionMode` each gained `Unfamiliar(String)`,
  carrying the token verbatim.** **Because** `docs/format.md` now states that every enumerated
  field the core format defines has one more inhabitant than it lists, and a closed enum cannot
  hold it. The variant name is *unfamiliar*, which is the document's word — deliberately not
  *unknown*, which `AxisPolarity` already spends on a value the format defines and the rigger
  declined to decide.
- **Those three enums, and `Axis`, are no longer `Copy`.** A token is a `String`. `Clone`,
  `Debug`, `PartialEq`, `Eq` and `Hash` all survive; nothing in the acceptance suite needed
  `Copy`, and nothing outside this crate consumes these types yet, so the cost falls entirely
  inside S2.
- **`may_contain` requires a row for the child kind as well as the parent.** **Because** a row
  lists the kinds its own kind may contain, so a kind added to `NodeKind::ALL` with no row of
  its own is merely absent from every existing row — indistinguishable, on the parent's row
  alone, from a kind deliberately forbidden there. Keyed on the parent only, `VAL-24` would read
  green for a kind nobody had declared. The child's row is what keeps that omission a `None`.
- **`limits::FIGURES` carries a row per `LimitKind` with every figure `None`.** **Because**
  `LIM-4` fixes each figure at four times a `stress` dimension and `stress` is S3's. The rows
  exist so the **set** is already complete and S3 edits numbers rather than structure; the
  figures stay absent so `LIM-1` does not read green over a measurement nobody has taken. This
  is the decision to revisit first if the branch has to be green before S3 — see *Contradicts
  docs*.
- **`issue_identifier` advances the mark before reading it**, through `checked_add`. The first
  identifier a puppet issues is therefore one, which is why `Id::NONE` is never issued without a
  special case. `checked_add` rather than `+` so exhaustion is a stated panic rather than a wrap
  into reuse, which is the one failure this function exists to prevent.
- **`Puppet::identifiers` includes the puppet's own identifier**, which its doc comment asks for
  — *the identifier of the puppet and of every entity it contains*. `ID-1` compares the set
  before and after a round trip, so including it strengthens the assertion rather than weakening
  it.
- **No mutation method was added.** `ID-2`, `ID-3` and `ID-4` are served by the public fields
  the tests already mutate directly; anything more would be the editor's, next cycle.

Task 5.

- **The crate parses JSON itself rather than taking `serde_json`.** **Because** preservation is
  about bytes: a number that has been through a float formatter is not the number that arrived,
  and `rigged:future-minor-nonfinite` requires `1e400` inside opaque material to be carried
  without ever being read as a number. `serde_json` with `arbitrary_precision` would do it and
  would turn that feature on for `rigel-fixtures` too through feature unification — the same
  hazard task 1 recorded for `preserve_order`, pointed at the crate that authors the corpus. It
  also keeps a JSON crate out of the library's dependencies, which task 3 had already chosen for
  `ExtensionEntry::data`.
- **`zip` reads the container and `flate2` inflates the payload, but the container *checks* are
  ours.** **Because** `zip` refuses none of the four hostile containers on its own:
  `ZipArchive::new` opens every one of them. It does expose `has_duplicate_names`, and
  `has_overlapping_entries` returns an *error* rather than `false` on the overlapping fixture,
  which cannot be told from a genuinely unreadable header. The overlap check is therefore computed
  from the central directory here, conservatively — a local header's extra field is not recorded
  there, so a region's end is taken as the smallest it could be and an overlap found this way is
  always real.
- **The decompression limit is enforced per archive entry, against the raw stream.** **Because**
  the `zip` crate truncates an entry at its declared uncompressed size and reports *"larger than
  its declared uncompressed size"* — which is the entry's own word, and `LIM-3` requires the
  refusal to be a **limit**. Reading `by_index_raw` and inflating under our own cap is what makes
  the figure the thing enforced.
- **Preservation hangs off `Puppet`, keyed by the identifier of the entity that carried the
  material.** **Because** the acceptance suite constructs `Node`, `Part`, `Parameter`, `Mesh`,
  `Axis`, `Keypoint`, `Transform` and `TextureFlip` by struct literal, so a field added to any of
  them would force an edit to a test — which is the one thing an implementer may not do. Keying on
  an identifier costs nothing: every entity carries one, uniquely, and it survives the entity being
  reordered in a way an array index would not.
- **A `PreservedMembers` row carries a `path` as well as an owner.** **Because** the objects
  inside an entity — its transform, its mesh, one axis, one keyframe — have no identifier of their
  own, and a later minor version adding a field to a transform must not be silently dropped. Only
  `axes` and `keyframes` are addressed by position, and both are positional in the format already.
- **A node of a kind this build does not know is an `OpaqueNode`, carrying the `id` and `parent`
  it declares.** **Because** the writer has to put it back in the canonical order, which is keyed
  on exactly those two members. They are read to **place** the entry and for nothing else; nothing
  in the puppet may reference a node this build cannot interpret.
- **A mesh is optional and absent is an empty mesh.** **Because** `rigged:containment-permitted`
  and `rigged:near-limit` are both built from mesh-group leaves that carry none, and both are
  catalogued `accepted`. Whether a *part* may have an empty mesh is a rule about the puppet and is
  the validator's. This was found by `val_23` failing, and it is the only defect the gates caught
  in this task.
- **`load` returns the reader's violations before calling the validator.** **Because** the three
  families above are facts about the source text; a rejection that named them *after* the
  structural rules would have to carry a version the `Puppet` does not have. It also means
  `EXT-2`, `EXT-3`, `EXT-8`, `VAL-7`, `VAL-8`, `VAL-17` and `VAL-20` are green now rather than at
  task 6.
- **`read_version` reports the version rather than judging it.** Its own doc comment says *reads
  the declared version, and nothing else*, and a caller deciding whether to open a file at all
  needs to be able to say **which** version the file wants. `load` applies the gate.
- **`rigel2d.json`'s `format` member is checked against `rigel2d-puppet`.** **Because**
  `format.md` says that entry carries the format name and the version, and a zip that happens to
  hold a `rigel2d.json` should be refused for what it is rather than part-way through the parse.
- **An undecodable payload is decided by a PNG *header* read, not a decode.** Signature, an
  `IHDR` chunk of the length the PNG specification fixes, non-zero dimensions, and the `IEND`
  chunk at the end. **Because** this crate builds with no GPU and decodes no image — that is
  `rigel-runtime`'s — and what `VAL-10` asks is whether the payload is an image at all. It also
  keeps the `png` crate out of the library. A payload with a valid header and a corrupt image
  reads as readable here, which is stated in *Assumptions*.
- **Every rejection the reader produces names its rule, and `Display` is truncated on a character
  boundary with an ellipsis.** `MAX_MESSAGE_BYTES` was task 3's figure and is unchanged; a
  violation list is additionally capped at 64 entries inside the decoder, because a list that
  grows with the file is the same denial-of-service route by another road.
- **An absent figure in `limits::FIGURES` refuses the load.** Its doc comment already said an
  absent figure is a defect rather than *no limit*; this is that sentence implemented, and it is
  unreachable today because every row carries a figure.

Task 6.

- **`validate(&Puppet)` admits an unfamiliar enumerated value unconditionally**, which task 4
  raised and task 6 was told to settle. Settled as stated: the gate is the **declared version**
  and a `Puppet` carries none, so `Rule::EnumeratedValueKnown` stays reachable only from the
  reader. A unit test carries the reasoning so that it is a decision in the suite rather than an
  omission somebody later "fixes".
- **`load` and `validate_file` share `validate::verdict`.** **Because** `VAL-13` asserts the two
  verdicts agree on every fixture, and two call sites performing the same two steps in the same
  order is a resemblance a later edit breaks silently. One function is the claim; the test is
  then a regression check rather than the only thing holding them together. This is the one edit
  task 6 made to `load.rs`.
- **Keypoints are ordered lexicographically over the position vector, strictly.**
  [format.md](../format.md) says *keypoints by position along their axis* and a parameter may
  declare two axes, so *along their axis* names no order for a two-dimensional one. Comparing
  coordinate by coordinate is what the corpus already carries — `body_lean`'s keypoints are in
  exactly that order — and it gives any number of axes a total order without a second rule.
  Strict, so two keypoints at one position are out of order as well: interpolating between them
  has no defined result. See *Contradicts docs*.
- **`Rule::BindingValueCountAgreesWithStaleMark` covers both counts a binding can disagree on** —
  one value per keypoint of its parameter, and, for a deformation, one offset per vertex of the
  target's mesh. **Because** [format.md](../format.md) names only the mesh and `BindingValues::len`
  names only the keypoints, while the mark means one thing: *this binding no longer matches what
  it applies to*. Two rules would make a fixture's rejection depend on which count it broke, and
  `rigged:binding-count-mismatch-unmarked` breaks the one the document does not name.
- **Every violation names an offender, defaulting to the puppet's own identifier.** **Because**
  `VAL-12` asks every rejection naming a rule to name at least one offending identifier, and a
  rule broken by the puppet as a whole — no root at all, say — otherwise names nothing. The
  default is in `violate`, so a rule added later cannot forget it.
- **The verdict collects every rule broken rather than stopping at the first**, capped at the
  same figure the decoder caps its own list at. **Because** a rejection naming one rule makes the
  corpus's one-defect discipline the only thing standing between a test and a false
  attribution; and
  because a person repairing a file wants the list. The cap is what keeps `VAL-12`'s *bounded*
  true for a hostile file, and the structural checks run before the sweep over every number so
  that it is the sweep that gets cut.
- **The committed schema lives in `crates/rigel-format/schema/`, not in `docs/`.** **Because** it
  is an artifact the crate's tests hold against the decoder, not prose; `docs/README.md` indexes
  documents and this is a pair of JSON files with a document explaining them. The boundary
  document is beside them for the same reason — a third party who runs the schema needs it in the
  same directory, not in a tree they have not been given.
- **The schema is not run in CI, and says so.** **Because** validating a document against it needs
  a JSON Schema implementation in the locked graph, bought to check files the acceptance suite
  already loads through the validator. What the suite does hold is the schema's **member lists**
  against `decode.rs`'s, which is the half that drifts. Running it is `VAL-14`'s human gate.
- **`decode.rs`'s key sets are `pub(crate)`**, for that test and nothing else, with a comment
  above them saying so.

**Contradicts docs:**

- **`one-part` cannot be what both documents say it is.**
  [puppet-acceptance.md](../requirements/puppet-acceptance.md) calls it "the smallest valid
  puppet: one root part, one texture, nothing else"; [format.md](../format.md) says the root is a
  kind that contains and argues the case from what an import produces. A root part is therefore
  invalid. The fixture keeps *valid* and wraps the part in a composite, and `root-is-part` is the
  same file without the wrapper. **`ART-19`'s premise dies with it** — it deletes `one-part`'s
  root part expecting a tree with no root, and the root is now a composite that survives. `ART-19`
  is out of scope here, so it is reported rather than fixed.
- **`rigged`'s catalogue row leaves one texture unreferenced on its plain reading.** Three parts,
  two of which share a texture, reference two of the three textures. That would put an
  unreferenced texture in the base fixture, which makes `rigged:orphan-texture` nearly redundant
  and `COLL-8`'s premise already true in `rigged`. Read instead as four parts — the masking part
  being its own rather than one of the three — every texture is referenced and
  `rigged:orphan-texture` is a real variant. The fixture takes that reading; the row should say
  which it meant.
- **`format.md` requires every number finite, and JSON cannot express a NaN.** `rigged:nan`
  therefore carries the bare `NaN` token, which is what a writer that is not ours emits — Python's
  `json` module does, by default — and is not strict JSON. A strict parser rejects the file
  before the finiteness rule is reached, so this one rejection may not be able to name an
  offending identifier the way `VAL-12` asks. The infinity pair uses `1e400`, which is strict JSON
  and overflows on parse, and those two can.
- **`format.md` describes no version-upgrade backup**, while `VER-4` and `VER-5` require one and
  task 7 is scoped to build it. Its version table says a lower minor opens normally and says
  nothing about upgrading or about writing anything.
- **`format.md` defines no shape for a track or a keyframe**, while `VAL-26`, carried from
  `concerns/security.md`, requires "a track's keyframe times, values and blend durations" to be
  validated. The fixtures carry `blend_duration` per track because the criterion needs a subject.
- **`format.md` says the puppet's gravity "is absent when it is the default" and never says what
  the default is.** The fixtures assume `1.0`, matching the per-node multiplier's default.
- **`format.md` names no file extension for a puppet file**, and nothing else in `docs/` does
  either. The corpus uses `.rigel2d` as a fixture convention, which decides nothing.
- **`format.md`'s canonical-zip list asks for "no external attributes".** A zip entry written
  through a Unix version-made-by always carries some; what the rule protects against is the
  writing machine's umask, which a pinned `0644` does not carry. Worth a word in the document, so
  the writer in task 7 is not asked for something no archive can hold.
- **`testing.md`'s harness table puts acceptance tests in `tests/acceptance/`** without saying
  whose. The workspace is virtual — there is no root package — so a workspace-level `tests/`
  directory is not compiled by anything. For S2 they have to live in `crates/rigel-format/tests/`,
  and the fixtures are committed there with them.
- **`LIM-1`, `LIM-2` and `LIM-7` cannot be satisfied in this subsystem.** They want a fixture just
  over each declared load limit; `LIM-4` fixes every limit at four times a `stress` workload
  dimension; [s2-format.md](s2-format.md) puts that workload out of scope. No limit figure exists
  to build a fixture against. See *Deferred*.
- **`rigged:near-limit` serves no in-scope criterion.** Its only consumer is `EDIT-2`, inside the
  `EDIT-*` surface this subsystem's *Out of scope* table assigns to Studio. It was named as in
  scope in the assignment given to this task, so it was built, against an invented node-count
  limit.
- **`rigged:ext-unknown` and `rigged:ext-declares-plugin` are named by `MSG-1` and `EXT-6` but
  have no row in the fixture catalogue**, so their contents were inferred. For `EXT-6` the
  inference matters: `I6` says a puppet may declare it carries data for a named plugin at a named
  version, and `format.md` defines no mechanism for that beyond the ordinary `x` marker, which
  already carries an identifier, a version and a human-readable name. The fixture treats the
  marker as the declaration.
- **The workspace declared `license` under `[workspace.package]` and no member inherited it**, so
  the key reached cargo for nothing. **Fixed in task 1's commit** — every member crate now sets
  `license.workspace = true`. A defect in the licence change that landed on `master` just before
  this branch opened, found here because `rigel-fixtures` matched the existing crates rather than
  introducing an inconsistency. Nothing to route at task 10.

Task 3. Found while writing tests from the criteria against the fixtures, and **reported rather
than fixed** — an acceptance criterion written without sight of the implementation is not an
implementer's to edit.

- **`TRACK-1` still points at *Criteria not written* for the duplicate-track question**, which
  `VAL-27` now answers. The sentence is stale. Left alone, because it is an existing criterion.
- **`ID-3` asks for the keypoint list to be reversed, and that has no defined meaning on its
  own.** A binding carries `values`, one entry per keypoint, indexed positionally — so reversing
  the keypoints without reversing the values against them is a change of *meaning*, not a
  reorder, and the criterion does not say which is intended. The test reverses both, which is the
  meaning-preserving reading; the criterion should say so.
- **An unfamiliar enumerated value in a *known* field of a later-version file has no stated
  in-memory representation.** `rigged:future-minor` carries `"output_source": ["torque"]` on a
  physics node. `VAL-20` and `EXT-7` say such a value is *preserved opaquely*, but the field is
  one this build interprets and the typed model has nowhere to put it: `OutputSource` is a closed
  enum, as `AxisPolarity` is. `TOOL-4` and `EXT-7` exclude that node and say so in a comment. This
  is the gap most likely to bite task 5.
- **`FLIP-2` has no fixture for a top–bottom flip on its own.** `rigged:flipped-pair` carries
  none, left–right, and both-set. The criterion asks that all three round-trip and be
  distinguishable; the test builds the third in memory. A fourth part in the fixture would be
  better, and the corpus is regenerable.
- **`POL-1` and `VAL-21`'s acceptance halves depend on `rigged:paired`**, which this subsystem's
  *Deferred* list puts out of scope because it serves `authoring-acceptance.md`. Both criteria are
  listed as in scope here. The tests set an axis to *unknown* in memory and round-trip it, which
  is the same claim without the fixture.
- **`CANON-7` compares a decoded payload against *the imported artwork*, and there is no import
  path in S2** and no separate source artwork beside `rigged`. Only the storage half — payloads
  carried through a round trip without re-encoding, and stored at the content rectangle the record
  declares — is assertable here.
- **`LIM-3`, `LIM-2` and `LIM-7` all want peak resident bytes bounded during a failed load.** The
  acceptance suite has no counting allocator. [stack.md](../stack.md) names one for the
  CPU-allocation budget; if it lands it serves these too, and until then *checked before
  allocating* is asserted only as *refused as a limit*.
- **`EXEC-1` needs a harness that fails on process spawn, dynamic library load, outbound
  connection and filesystem access outside the container.** None exists. What the test carries is
  the observable half: every fixture reaches its catalogued outcome, and the directory the puppet
  sits in is unchanged, gains nothing and keeps its modification time. The rest belongs with task
  8's hostile-input work.
- **[format.md](../format.md) never says what `composition_mode`'s one value is.** It says the
  field is always written and that the format defines one mode, and names neither. The fixtures
  use `"sum"` and the contract declares `CompositionMode::Sum`; the document should carry it,
  since `DET-6` asserts *its value is the one meaning the fixed order specified now*.
- **Which fields have defaults is stated only for a physics node and for the puppet's gravity.**
  *A defaulted field is absent when it holds its default* is a rule with no list behind it, and
  `rigged` writes `"source": ""` on its channel mapping and an identity `rotation`/`scale` on
  every transform. Either those are defaults and the fixtures break the rule, or they are not and
  the document should say so. This decides whether
  `canonical::saving_an_unmodified_puppet_reproduces_the_file_it_came_from` can pass at all.
- **[format.md](../format.md)'s *Validation* list does not mention four rules the criteria
  require**: exactly one neutral keypoint per parameter (`VAL-15`), each axis declaring a polarity
  (`VAL-17`), a track's keyframe times, values and blend duration (`VAL-26`), and extension data
  declaring a version and a name (`EXT-3`). The last three live elsewhere in the document; the
  neutral-keypoint rule is nowhere in it.
- **A mesh group's mesh carries `uvs` in the corpus**, and a mesh group references no texture.
  [format.md](../format.md) specifies `uvs` under *A part's mesh* and says they are in the
  texture's authored pixel space, which a mesh group has none of. Either the field is meaningful
  there and the document should say in what space, or it should not be written.
- **`one-part`'s catalogue row and the criteria document still disagree**, as task 1 reported.
  Unchanged by this task; the tests use the fixture as built.

Task 4. Found while implementing the contract against the documents and the tests at the same
moment, and **reported rather than fixed**.

- **`EXT-8`'s test reads the wrong node, and fails at an assertion rather than at an
  `unimplemented!()`.** `extensions::ext_8_the_version_declaration_is_what_licenses_unmarked_material`
  asks `json_node(&doc, 11)["wiggle"]`; both `rigged:ext-unmarked` and `rigged:future-minor`
  carry the unmarked `wiggle` key on node **12**, which is the part. `rigel-fixtures`'s own
  `the_unmarked_material_is_byte_identical_across_the_version_gate` finds it by searching for
  the key rather than by naming a node, and passes. The test is an acceptance test, so this is
  a finding; nothing in task 4 touches it and it failed the same way before.
- **Task 3's handoff claims every one of the suite's failures is at an `unimplemented!()` in the
  crate.** `EXT-8` is not, and was not: it fails in the test. The claim is what makes *check the
  panic site, do not count the failures* worth doing on every task from here.
- **Task 4's own *Done when* could not be met as written** — it names the identity tests, and
  every one of them loads and saves. Corrected in the task entry above rather than left to
  surprise the reviewer.
- **`validate(&Puppet)` cannot apply the version gate the unfamiliar-value rule needs.**
  `docs/format.md` gates an unfamiliar enumerated value on the **declared version**; a `Puppet`
  carries no version, and the task 3 decision *a save decides whether it is upgrading by reading
  the file it is replacing* says deliberately that it must not. So `Rule::EnumeratedValueKnown`
  can only be reached where the version is in hand — `load` and `validate_file` — and
  `validate(&Puppet)` has to admit an `Unfamiliar` value unconditionally, or `VER-1` could load
  a later-minor file and then fail to save it. `VAL-13` does not catch the difference, because
  it compares `load` against `validate_file`, both of which read the declaration. Task 6's to
  settle; stated here so it is settled rather than discovered.
- **`format.md` says *every enumerated field the format defines*, and the contract leaves some
  closed.** A binding carries `"kind": "transform" | "deformation"` and the corpus writes
  it, but `BindingKind` has no unfamiliar inhabitant — and could not usefully have one, because
  `BindingValues` is the parsed shape that `kind` selects. A node's `"kind"` is the other, and
  it is covered by a different mechanism: an unknown node kind is preserved opaquely and is not
  a member of `Puppet::nodes` at all (`EXT-7`). Whether the rule means *every enumerated field
  whose values are inert data* is worth a sentence in the document.
- **`LIM-1` can never pass in S2, and neither can the fixtures that depend on a figure.**
  *Deferred* already records that the just-over fixtures are blocked on S3. What it does not
  record is that `rigged:decompression-bomb` and `rigged:entry-name-overlong` are catalogued
  `rejected`, and `VAL-13` asserts **every** catalogue fixture reaches its catalogued outcome —
  so without `DecompressedPayloadBytes` and `EntryNameBytes` figures, task 5 has no threshold to
  refuse them against and `VAL-13`, `LIM-3` and the hostile-container test stay red too. The
  branch therefore cannot be green at task 8 as *Status* expects, unless provisional figures are
  stated. `crates/rigel-fixtures`'s `PROVISIONAL_NODE_LIMIT` is the precedent for stating one.
- **`format.md` calls the root *a kind that contains* and then names mesh group and composite.**
  Physics contains physics and may not be the root, so the general phrase is not the rule — the
  parenthetical is. `may_be_root` implements the parenthetical. The document would be clearer
  saying *a kind that may contain drawn content*, or simply naming the two.

Task 5. Found while implementing the reader against the documents, the corpus and the tests at
the same moment, and **reported rather than fixed**.

- **`PHYS-4` and `MSG-1` cannot both pass, and the fixtures are why.**
  `stored_shape::phys_4` asserts `load("rigged").report.warnings.is_empty()`, and
  `version::msg_1` asserts `load("rigged:ext-unknown").report.warnings` is **not** empty.
  `rigged:ext-unknown` is `rigged` plus **one more extension namespace**, and `rigged` already
  carries two namespaces no build understands (`com.alt.notes`, `com.example.wiggle`). No rule
  that warns about unrecognised extensions can warn for one file and not the other. The reader
  satisfies `MSG-1`, whose criterion *is* the report; `PHYS-4`'s assertion is incidental to its
  criterion — it uses an empty warning list as a proxy for *driving nothing is not reported as a
  defect* — and it will go red at that line once task 6 lands. Both tests are acceptance tests and
  neither was touched. The cheap repair is a fixture: give `rigged` no extension data no build
  understands, or give `PHYS-4` a narrower assertion.
- **`format.md` puts `x` on *any object*, and the contract models `Extensions` on `Puppet` and
  `Node` only.** The reader reads and reports the marker wherever it appears and keeps it verbatim
  under `Preserved` where there is no typed slot, so nothing is lost. Either the document should
  say `x` lives on those two, or the contract should carry `Extensions` everywhere.
- **`format.md` does not say whether a mesh is required, and the corpus answers *no*.** A mesh
  group leaf with no `mesh` is what `rigged:containment-permitted` and `rigged:near-limit` are
  built from. *A part's mesh* is specified; a mesh group's is not, although the containment table
  says a mesh group *deforms its children with its own mesh*.
- **`format.md` names no required member of an extension's `x` entry beyond the three.** The
  reader requires `data` and refuses an entry without it, which no fixture exercises and the
  document does not state. `EXT-3` names `version` and `name` only.
- **A binding's `kind` is an enumerated field the format defines, and it cannot have the
  unfamiliar inhabitant the rule gives every other one.** Task 4 reported this; the reader now
  has to act on it, and it refuses an unfamiliar binding kind **at every declared version**,
  because `kind` selects the shape of `values` and there is no reading of an unknown kind that
  still yields values. The document should say whether the rule means *every enumerated field
  whose values are inert data*.
- **`platform.md`'s table has no row for six more kinds than task 5's entry above lists.**
  That entry names decompressed payload size, tree
  depth, string length, total load memory, tracks per animation and keyframes per track. Walking
  `LimitKind::ALL` against the table turns up a keypoint count, a response curve's points, a
  chain's joints, an entry name's bytes, a texture dimension and extension data bytes as well —
  none of which the table measures. Each is marked `Basis::Provisional` with the dimension it is
  missing named in its row.
- **The table's texture rows cannot carry a per-entry byte limit, and say so themselves.**
  `platform.md` calls them *a deliberate headroom choice with no evidence behind them* and *the
  least trustworthy numbers here*, and both are **totals over a puppet** rather than the size of
  one entry. `DecompressedPayloadBytes` and `TextureDimension` are therefore provisional despite
  there being rows that look as though they would serve.
- **`LIM-4`'s *four times the stress dimension* is unsatisfiable for `TotalLoadMemory` as
  written.** Four times the stress artwork would be well past the 2 GB resident budget
  `platform.md` gives the editor with a reference puppet open, so the figure is taken from the
  budget instead and marked provisional.
- **`format.md` requires `composition_mode` to be *always written* and says nothing about a file
  that omits it.** The reader defaults it to `sum` rather than refusing, which is the lenient
  reading and is untested by any fixture.
- **`format.md`'s canonical form is a rule about writing, and the corpus relies on it being a rule
  about reading too.** `rigged` writes an identity `rotation` and `scale` on every transform and
  `"source": ""` on its channel mapping, which the absent-when-default rule says it should not.
  The reader accepts both present and absent; the writer in task 7 is the one that has to choose,
  and `saving_an_unmodified_puppet_reproduces_the_file_it_came_from` is what will decide it. Task
  3 reported the same thing from the other side.
- **Nothing in `docs/` declares a limit on JSON nesting depth**, which is a size the loader
  trusts and so is `LIM-8`'s business. It is not a `LimitKind`, because every kind there is a size
  or count of something the format defines and nesting depth is a property of the encoding; it is
  a constant in `json.rs` and a file past it is refused as malformed. Worth a sentence somewhere.

Task 6. Found while implementing the validator against the documents, the corpus and the tests at
the same moment, and **reported rather than fixed**.

- **The ordering rules read as rules about the stored form, and `ID-3` says the in-memory order
  is free.** [format.md](../format.md)'s *Validation* list requires keypoints ordered, and its
  *Canonical serialisation* section makes ordering *part of the canonical form* — a rule about
  **writing**. `identity::id_3` reverses `nodes`, `parameters`, `textures`, `bindings`, every
  parameter's keypoints and the binding values against them, and then expects the save to
  succeed and the reload to come back canonically ordered. Both cannot be true of the same rule
  unless the **writer canonicalises before validating**. Implemented with the rule in
  `validate(&Puppet)`, which is where the specification's list puts it; what that costs is
  stated under *Traps*, because it lands on task 7 and nothing exercises it yet.
- **[format.md](../format.md) does not say what *ordered by position* means for a
  two-dimensional parameter.** `body_lean` declares two axes. The order implemented is
  lexicographic over the position vector, which is what the corpus carries; the document should
  say so, because a third-party writer has nothing to derive it from.
- **[format.md](../format.md) requires every number finite and names no precision, while
  `VAL-18` requires finiteness judged at the precision evaluation uses.** The criterion is what
  makes `rigged:nonfinite-after-narrowing` a defect; the document does not mention the question,
  so a conforming third-party validator would accept that file. This is the clearest case in the
  format of two tools disagreeing about validity, which is exactly what the criterion exists to
  prevent, and it belongs in the document rather than only in a criterion.
- **The *Validation* list says *binding value counts matching their mesh* and the contract says
  they match the parameter's keypoints.** Both are real counts and both are what the mark is
  about; the document names one and `BindingValues::len`'s doc comment names the other. See
  *Decisions made*.
- **The puppet's own gravity has no stated rule, and the validator applies none beyond
  finiteness.** `PhysicsConstant::Gravity` is the **per-node multiplier**; a puppet-level gravity
  below zero is accepted today. Whether negative means *up* or is a defect is not in the
  document and no criterion asks.
- **Nothing says whether a part must reference a texture.** The reserved zero means *no
  reference*, so a part carrying `"texture": 0` validates. `rigged:dangling-texture-ref` names a
  texture that is absent, which is a different thing. No fixture carries the first.
- **`format.md` says `output_source` and `output_scale` are *per axis of that parameter* and no
  rule checks that they are.** A physics node may name a one-axis parameter and carry three
  output sources. Not a rule the document states, not a criterion, and so not implemented —
  reported because it is the kind of disagreement the evaluation pass will meet first.
- **`rigged:id-zero`'s rejection names the reserved zero as its offending identifier**, which is
  true and nearly useless: the offender *is* zero. A `Puppet` does not record which entity
  carried an identifier — `Puppet::identifiers` is a flat list — so naming the owner would mean
  a second walk written only for the message. `VAL-12`'s letter is met; its spirit is thin for
  this one rule.
- **The root rule, again.** Task 4 reported that *a kind that contains* is the general phrase and
  the parenthetical naming mesh group and composite is the actual rule. Confirmed from the other
  side: `may_be_root` is what the validator reads, and physics *contains* and may not be the
  root.

**Traps:**

- **`zip`'s `compression_level` rejects an ordinary deflate level.** With only the
  `deflate-flate2` feature on, `SimpleFileOptions::compression_level(Some(6))` fails at runtime
  with `UnsupportedArchive("Unsupported compression level")`, and so does `Some(9)`. Leaving the
  level unset gives a deterministic default. This cost real time and the error names nothing.
- **A node appended to the end of `nodes` silently breaks depth-first preorder** whenever its
  parent is not the last subtree. Nothing fails; the fixture is simply not canonically ordered,
  and a later byte-identity criterion would have found it instead. `Doc::push_node` inserts at
  the end of the parent's run.
- **`serde_json`'s map is a `BTreeMap` only while `preserve_order` is off.** Any crate anywhere in
  the graph turning that feature on would de-canonicalise every fixture, through feature
  unification, with nothing failing. The canonical-JSON test catches it; the hazard is worth
  knowing before it is diagnosed.
- **A `:` in a fixture file name fails at checkout on Windows, not at build.** The catalogue's
  `rigged:<defect>` becomes `rigged.<defect>` on disk. A test asserts it.

Task 3.

- **A test that sets the panic hook silences every other test in the binary.** `EXEC-5` uses
  `catch_unwind` over 256 mutations, and an early version installed an empty panic hook to keep
  the output readable. The hook is process-global: 80-odd concurrent failures lost their
  `panicked at …` lines and the suite looked like it had failed for no stated reason. The hook is
  gone; the noise is the price.
- **`zip`'s `"deflate-flate2"` feature does not choose a flate2 backend.** Adding `zip` alone as a
  dev-dependency fails to build with *You need to choose a zlib backend* — it only works in
  `rigel-fixtures` because that crate also depends on `flate2` with `rust_backend`, and the
  workspace build unifies them. Building `-p rigel-format` on its own does not. `flate2` with
  `rust_backend` is therefore a dev-dependency here too.
- **`zip` 9's `ZipFile::name()` returns a `Result<Cow<str>, _>`**, not a `&str`. Worth knowing
  before the writer in task 7 reads an archive back.

Task 4.

- **Opening an enum costs `Copy`, and `Copy` was load-bearing nowhere — but only by
  inspection.** `Axis` lost it with `AxisPolarity`. Nothing in `src/` or `tests/` needed it, and
  `cargo build` said so, but a type that was `Copy` and is not reads as an unrelated borrow
  error at the use site rather than as *this enum opened*. Worth knowing before diagnosing one.
- **`AxisPolarity::Unknown` and `AxisPolarity::Unfamiliar` are a sentence apart and mean
  opposite things.** *Unknown* is a value the format defines: the rigger has not decided, and it
  must be written rather than inferred from an absence (`VAL-21`). *Unfamiliar* is a token this
  build has never heard of. A validator that treats them alike either rejects every import or
  accepts every typo. Both doc comments say so; this is the one place the vocabulary genuinely
  invites the error.

Task 5.

- **The `zip` crate stops an entry at its declared uncompressed size.** Reading
  `rigged:decompression-bomb` through `by_index` returns 1024 bytes and then
  `"File is larger than its declared uncompressed size"` — so a loader built on `by_index` never
  learns that the entry expands to a quarter of a gigabyte, and refuses it for the wrong reason.
  `LIM-3` requires the refusal to be a limit. `by_index_raw` plus our own inflate is the way
  through, and it cost real time to find because the safe-looking API is the one that hides it.
- **`ZipArchive::new` opens all four hostile containers happily**, and `len()` quietly collapses
  duplicate names — `rigged:duplicate-entry-names` has six entries and reports five.
  `has_duplicate_names()` is what sees it. `has_overlapping_entries()` returns `Err` rather than
  `Ok(true)` on the overlapping fixture, which is indistinguishable from a genuinely broken
  header, so it is not what the overlap check uses.
- **A field added to `Node`, `Part`, `Parameter`, `Mesh`, `Axis`, `Keypoint`, `Transform` or
  `TextureFlip` breaks the acceptance suite's compilation**, because `identity.rs` and
  `stored_shape.rs` construct each of them by struct literal with no `..` rest. `Puppet` is the
  only type in the model with no literal in the suite, which is why everything the reader had to
  add hangs off it. Worth knowing before designing anything else into those types.
- **`rigged:near-limit` sits at 255 nodes, against `PROVISIONAL_NODE_LIMIT` in
  `rigel-fixtures`, and the real `NodeCount` figure is 2160.** The fixture is catalogued
  `accepted` and stays accepted, so nothing fails — but the figure it was built against and the
  figure the loader enforces are now two different numbers and neither document says so.
- **`cargo test -p rigel-format --test acceptance` reports one test binary twice.**
  `canon_2` re-runs the test binary as a child, so the output carries a second
  `test result: FAILED. 0 passed; 1 failed` line that belongs to the child and not to the suite.
  Counting result lines rather than reading them gets this wrong.

Task 6.

- **The writer must canonicalise before it validates, and nothing proves it yet.**
  `validate(&Puppet)` enforces keypoint order, and `identity::id_3` reverses a parameter's
  keypoints in memory — with the binding values against them, which is the meaning-preserving
  reorder — and then saves. That save succeeds only if `save` sorts keypoints by position and
  permutes each binding's values with them **before** handing the puppet to the validator. It is
  work the writer has to do anyway, because `ID-3` also asserts the reload comes back in
  canonical order; what is easy to miss is that it must happen on the *validating* path too.
  **Every identity test dies at `save`'s `unimplemented!()` today**, so this costs nothing until
  task 7 and then costs an afternoon. The alternative — moving the rule into the reader, where
  the stored order is a fact about the text — was weighed and rejected; see *Tried and rejected*.
- **A parent walk that does not colour nodes never terminates on `rigged:cycle`.** The fixture
  reparents the head of the physics chain under its own child, so walking up from either node
  loops for ever. Colouring each node once is also what keeps a hostile file from costing a
  quadratic pass, and the test that catches the regression builds the two-node cycle by hand.
- **Clippy refuses `!(a > b)`** — `neg_cmp_op_on_partial_ord`, because the two values may be
  incomparable. It is right: with a `NaN` time, every comparison is false and the negation is
  what decides. `ascends` and `follows` are the one place the question is answered, for keypoint
  positions and keyframe times alike.
- **A unit test that puts one node kind under another has to account for the grandparent too.**
  Testing *parent contains child* by building root → parent → child means the root's own pair
  must be permitted, or the verdict carries a containment violation that has nothing to do with
  the pair under test. The table-driven test in `validate.rs` states that condition rather than
  working around it.

**Assumptions:**

- ~~Every JSON key name, the current version, the gravity default, the compression per entry kind
  and the canonical collection orders.~~ **Resolved** — promoted into [format.md](../format.md),
  which now specifies them. They remain `[proposed]` there, as every section of that document is.
- `uvs` are in the texture's **authored** pixel space, which is the reading that makes
  `format.md`'s cropping paragraph — the record carrying the crop rectangle and the authored
  dimensions so the runtime can reconstruct the space — do its stated work.
- `rigged`'s catalogue row describes four parts, not three. See *Contradicts docs*.
- `PROVISIONAL_NODE_LIMIT`, the figure `rigged:near-limit` sits under, is invented and will be
  wrong. The builder is committed so the fixture regenerates when the real figure lands.

Task 3.

- **A rejection carries its broken rules as typed `Violation`s**, which is what
  `support::assert_rejected_naming` matches on — not the rendered message. `Rejection::rule_name`
  is asserted only to be non-empty, by `VAL-12`, so how a rule reads to a person stays the
  implementer's.
- **`rigged:nan` may be refused as malformed JSON rather than for finiteness**, and `VAL-6`'s test
  accepts either for that one fixture while requiring finiteness by name for the infinity pair.
  Assumed per the task 1 finding above.
- **An unknown node kind is not a member of `Puppet::nodes`.** Read out of `EXT-7`'s *no reported
  entity*. See *Contradicts docs*.
- **A physics node's stored object carries only the keys
  [format.md](../format.md)'s table defines**, plus the keys every node carries and the `x`
  marker. `PHYS-1` and `PHYS-8` assert the key set against that list, which is written out in
  `tests/acceptance/stored_shape.rs`; a field added to the format is added there too.
- **`rigged:no-gravity` saves back byte-identically**, which assumes the gravity default is
  `1.0` and the absent-when-default rule applies to it. Both are stated in
  [format.md](../format.md); the fixture assumed the first before the document did.

Task 4.

- **The unfamiliar inhabitant is `Unfamiliar(String)`, the token as it arrived.** The format
  says *carrying the token verbatim* and names no in-memory shape. The writer in task 7 writes
  that string back where the field goes; the reader in task 5 builds it only when the declared
  minor is higher than `FormatVersion::CURRENT`.
- **`Puppet::identifiers` enumerates a chain deformer and each of its joints** as entities in
  their own right, because `ID-8` lists them among the pieces a puppet is made of and they carry
  identifiers.
- **Nothing in `load.rs` was implemented, including the parts that need no file I/O** —
  `Rejection::rule_name`, `offenders`, `violations` and its `Display`. They are reachable only
  from a rejection a reader produced, so implementing them turns no test green, and how a rule
  reads to a person is the validator's surface. Task 6's, unless task 5 wants them sooner.

Task 5.

- **A payload is an image if its PNG header says so.** A payload with a well-formed `IHDR` and a
  corrupt image body reads as readable and is not reported. The alternative is decoding every
  texture at load, which belongs to S5 and would put an image decoder in a crate that builds
  without one.
- **A later-version file's unmarked member on an object with no identifier of its own** is kept
  against the entity that contains it, at a path. No fixture carries one, so the shape is
  untested; the writer in task 7 is the first thing that will exercise it.
- **An archive entry naming a texture the puppet does not carry is malformed.** No fixture carries
  one, and the alternative — ignoring it — would let material ride along inside the file with
  nothing accounting for it.
- **An entry compressed by any method but Stored or Deflate is refused.** The format fixes the
  method per entry kind for *writing*; the reader accepts either for any entry, because
  `rigged:decompression-bomb` deflates a PNG and a third party's tool may too.
- **Provisional figures will be wrong.** Each says in its own row which dimension it is missing,
  and `Basis::Provisional` is the thing to grep for when the measurement lands.

Task 6.

- **Keypoints are ordered strictly**, so two at one position are invalid. No fixture carries a
  repeated position and no criterion names one; interpolating between two keypoints at the same
  place has no defined result, which is the reading taken.
- **A part may reference the reserved zero**, meaning no texture. See *Contradicts docs*.
- **A mesh group with no mesh has no vertices**, so a deformation binding targeting one must
  carry no offsets or be marked stale. No fixture carries that pair; task 5 established that an
  absent mesh is an empty mesh.
- **`validate(&Puppet)` re-checks none of the reader's three rule families.** An extension entry
  with an empty name or version, an axis whose polarity was never declared, and a key the format
  does not define are all facts about the text the reader has already judged, and an absence has
  no typed inhabitant in a puppet assembled in memory. A save of a puppet whose
  extension entry was emptied by a caller is therefore not refused for that.
- **The puppet-level gravity is validated for finiteness and nothing else.** See
  *Contradicts docs*.

**Deferred:**

Task 4 — what was in the task's scope and is not done, with why:

| Left | Why |
|---|---|
| Every figure in `limits::FIGURES` | `LIM-4` ties each to a `stress` dimension, which S3 generates. The rows are there; only the numbers are outstanding. See *Contradicts docs* for what this blocks |
| Every `ValidRange::max` in `validate::RANGES` | Upper bounds are `[open]` in [format.md](../format.md), due with S4 as a measurement. The lower bounds and their strictness are stated and implemented |
| An `Unfamiliar` inhabitant for `BindingKind` | `BindingValues` is the parsed shape `kind` selects, so an unfamiliar kind has no values to carry. Reported as a question for the document rather than answered here |

Task 3 — the in-scope criteria with no test, and what each needs:

| Criterion | Why there is no test, and what it needs |
|---|---|
| `ID-7`, `VAL-14`, `TOOL-2`, `TOOL-3`, `MSG-2`, `LIM-5`, `EXEC-4`, `CHAIN-4`, `PHYS-5` | Not `test` in the criteria document — `judgment` at review, or the human gate task 9 already sets out. Nothing missing |
| `LIM-1` (second half), `LIM-2`, `LIM-6`, `LIM-7`, `LIM-8` | A fixture just over each declared limit, and the figures do not exist until S3 generates `stress`. `LIM-1`'s first half — every limit declared in one place — **is** tested. `LIM-8`'s walk of the format definition is `script` and needs the definition to be walkable |
| `EXEC-3`, `VAL-24` | `script` in the criteria document. `VAL-24` has a behavioural half here — the declaration is total over the kinds — and the script is task 4's |
| `EXEC-1` (the interception half), `EXT-5` (the same) | A harness that fails on spawn, `dlopen`, connect and out-of-container filesystem access. Task 8 |
| `LIM-3` (the memory half) | A counting allocator the acceptance suite can read |
| `CANON-7` (the artwork half) | The import path, which is Studio's |
| `VER-2`, `PHYS-6`, `PHYS-8`, `FLIP-1`, `FLIP-2`, `TEX-4`, `CHAIN-1` — the second half of each | Evaluation or a render surface: S4 and S5. Each test carries the stored-shape half and says in its comment where the other lives |

- **A just-over fixture per declared load limit** (`LIM-1`, `LIM-2`, `LIM-7`). The limits do not
  exist yet and `LIM-4` ties them to a workload S3 generates; a fixture built against a guessed
  figure would be rebuilt anyway. When the figures land they go in one table in
  `crates/rigel-fixtures` and the corpus regenerates.
- **`reference` and `stress`** — S3's generated workloads, out of scope.
- **`rigged:paired`, `rigged:far-axis`, `source-without-polarity`** — they serve
  `authoring-acceptance.md` criteria this subsystem puts out of scope.
- **`rigged:dup-texture`** (`TEX-1`, needs resident-byte accounting, S5), **`rigged:solo-animation`**
  (`ART-18`, Studio), **`rigged:three-deform`** (`DET-4`, S4).
- **Wiring `scripts/fixtures-check.sh` into the push gates.** It belongs with task 2, which
  already edits `.github/workflows/push-gates.yml` and `stack.md`'s gate table; adding a step here
  would leave that table disagreeing with the workflow. Until it is wired, drift between the
  committed corpus and its builder is caught only by running the script.

Task 5 — what is in the reader's scope and is not done, with why:

| Left | Why |
|---|---|
| `Rejection` messages read at a human gate | `MSG-2` is task 9's, and the warning text the reader emits has not been read by a person. It names each unrecognised extension by its human-readable name, promises nothing about preservation and implies no install route, which is what that gate is for |
| A limit on JSON nesting depth as a `LimitKind` | Reported above instead. Adding a variant changes `LimitKind::ALL`, which `LIM-1` and `LIM-8` are both driven over, and the kind is about the encoding rather than about the puppet |
| `TotalLoadMemory` enforced against **resident** bytes | It counts bytes decompressed, which is the part of a load this crate controls. The resident half needs the counting allocator `stack.md` names, which is also what `LIM-3`'s memory half waits on |

Task 6 — what is in the validator's scope and is not done, with why:

| Left | Why |
|---|---|
| Every `ValidRange::max` | Still `[open]` in [format.md](../format.md), pending the integrator-stability sweep due with the evaluation pass. The validator reads the table, so a figure landing there is the whole change; `schema/README.md` says the same of the schema |
| Running the committed schema over the corpus | Needs a JSON Schema implementation in the locked graph, bought to check files the acceptance suite already loads through the validator. What is held instead is the schema's member lists against the reader's. `VAL-14` is the human gate that runs it |
| A rule about `output_source` and `output_scale` matching the parameter's axis count | Not stated in [format.md](../format.md) and not a criterion. Reported in *Contradicts docs* |
| A rejection naming the entity that carried the reserved zero | `Puppet::identifiers` is a flat list, so the owner is not recoverable without a second walk written only for the message. Reported in *Contradicts docs* |

**Tried and rejected:**

The branch keeps these in [*Tried and rejected*](#tried-and-rejected) above, where tasks 1 to 3
put theirs. Task 4 added the provisional load-limit figures, the version field on `Puppet`, and
the parent-only containment lookup.

Task 5 added these.

- **Calling `ZipArchive::validate_header_consistency()`.** It would refuse a file whose local
  headers disagree with the central directory, which is several of the twenty single-byte
  mutations `EXEC-5` currently survives. It is not worth it: the mutations it would catch are
  already harmless — every entry's CRC is checked, so a header that lies about a size produces
  bytes that do not match — and a zip tool writing data descriptors leaves the local sizes zero,
  which would refuse an honestly repacked file at task 9's gate for a reason the specification
  does not state.
- **Preserving a later-version file's unknown material as the whole arrived source object.** It
  is the generic answer and it breaks `FLIP-2`: a test that mutates `texture_flip` in memory and
  compares the puppet against the reloaded one would be comparing a stale copy of the member it
  just changed, because `Puppet` derives `PartialEq`. Keeping only the members the typed
  structures have no slot for is what keeps that comparison honest.
- **Refusing a later-version file whose unknown member sits on an object with no identifier.** The
  honest failure, and it breaks the compatibility rule for exactly the addition that rule exists
  to carry — a later minor adding a field to a transform would make every file unopenable.

Task 6 added these.

- **Moving the keypoint-order rule into the reader.** Stored order is a fact about the source
  text, which is the shape task 5 used for the three rule families it kept, and it would make
  `identity::id_3`'s reversed-in-memory puppet valid with no demand on the writer at all. It
  loses on a worse count: [format.md](../format.md)'s *Validation* list is the specification of
  `validate`, and taking a rule out of it leaves `validate(&Puppet)` admitting a puppet the
  document calls invalid — the exact drift the single-validator rule exists to stop. The writer
  has to sort keypoints for `ID-3` either way, so the cost of keeping it is ordering a collection
  the writer already orders. Recorded as a trap rather than taken.
- **Pulling a JSON Schema implementation into the locked graph to run the committed schema in
  CI.** It is the check that would make the schema certainly correct rather than probably. It
  buys a second opinion about files the acceptance suite already loads through the validator,
  against a dependency the `locked-graph.sh` gate exists to keep small, and it does not catch the
  failure the schema actually has — drifting from the format. Holding its member lists against
  `decode.rs` catches that, costs nothing, and runs with the suite.
- **Stopping the verdict at the first rule broken.** Cheaper, and it makes every rejection
  trivially bounded. It also makes the corpus's one-defect discipline load-bearing in a way it
  was never meant to be: a fixture that happened to break a second rule would be attributed to
  whichever check ran first, and a person repairing a file would get one line at a time.
- **Giving `validate(&Puppet)` the reader's three rule families as well.** An extension entry
  with an empty name reads like something a save should refuse. Two of the three are about a
  member being **absent**, which has no inhabitant in a puppet, and the third is gated on a
  declared version the puppet does not carry — so the version half would have to be dropped,
  which is the inversion task 4 already recorded.
