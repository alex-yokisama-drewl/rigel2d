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

**Current task:** not started
**Blocked on:** nothing

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

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- *(none yet)*

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

- *(none yet)*

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The hand-authored fixture corpus — `todo`

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

### 2. Dependency and workflow gates — `todo`

Due with the first registry dependency, not later ([plan.md](plan.md)). A check that every
`uses:` in `.github/workflows/` is a 40-hex commit SHA, and a check that the locked dependency
graph contains no network-client crate and no git or alternative-registry source. Both as
scripts, wired into gate 5's job.

**Done when:** both scripts fail on a deliberately broken input and pass on the tree; gate 5's
job runs them; the `security` entries they close are marked covered.
[decisions.md](../decisions.md)'s no-outbound-connection claim stops diverging from what gate 5
can actually see, which is the point of doing this now. No user gate.

### 3. Acceptance tests — `todo`

Dispatched to `acceptance-planner`, **which must not be shown an implementation** — there is
none, which is why this task is here rather than after task 7. It implements the in-scope
criteria into `tests/acceptance/`, adds the `main.rs` that stops cargo silently skipping the
directory, and writes the two criteria that
[puppet-acceptance.md](../requirements/puppet-acceptance.md#criteria-not-written-open) records as
writable and unwritten.

Also the `ops` check that gate 2 fails on zero tests — it has a subject from this commit onward.

**Done when:** every in-scope criterion has a test or a stated reason it has none; the suite
compiles; the tests **fail**, which is what confirms them. No user gate.

### 4. Core types, identifiers and the containment table — `todo`

The in-memory puppet: entities, the unsigned 64-bit identifier with its high-water mark and
reserved zero, handles, and the per-kind containment declaration as **data the validator reads**
rather than rules naming kinds. Mutation sufficient for `ID-2`, `ID-3` and `ID-4` and no more.

**Done when:** `VAL-24` passes — a node kind with no containment declaration fails a check rather
than inheriting a default. No user gate.

### 5. The reader — `todo`

Archive and manifest, the version gate (`rigel2d.json` read on its own, before any load limit is
at risk), load limits enforced **before allocating**, and the parsed source kept alongside the
typed structures so unknown material survives. Treats the file as hostile throughout.

**Done when:** the version, extension, limit and preservation criteria pass; `I4` holds
structurally — a failed load cannot have replaced anything, because the loader produces a value
rather than mutating one. No user gate.

### 6. The validator — `todo`

The single gate on load and save, plus the committed JSON Schema for what a schema can express
and the standalone command `VAL-13` needs. Every rejection names the rule broken and at least one
offending identifier, in bounded output.

**Done when:** the whole malformed corpus is rejected and stale bindings are accepted as a state;
the standalone verdict equals the embedded one on every fixture. No user gate.

### 7. The writer — `todo`

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

**Decisions made:**

**Contradicts docs:**

**Traps:**

**Assumptions:**

**Deferred:**
