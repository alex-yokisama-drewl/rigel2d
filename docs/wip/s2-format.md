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

**Current task:** tasks 1, 2 and 3 are `done`, as is the encoding promotion that stood between
tasks 1 and 3 — see the section above the task list. **Next: task 4, the core types.**
**Blocked on:** nothing.

**Gate 2 is red from here**, as designed: task 3 wrote the acceptance tests against a declared
surface whose every body is `unimplemented!()`, and they fail. Tasks 4 to 7 turn them green.

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

### 4. Core types, identifiers and the containment table — `todo`

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

**Why `may_contain`, `may_be_root`, `declared_range` and `declared_limit` return `Option`**, which
looks over-engineered until you need it: `VAL-24`, `VAL-25` and `LIM-1` each require an
*undeclared* answer to fail a check rather than default to false. A `bool` cannot distinguish
"declared, and forbidden" from "nobody declared this", which is the whole point of each of the
criteria named above.

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

**Deferred:**

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
