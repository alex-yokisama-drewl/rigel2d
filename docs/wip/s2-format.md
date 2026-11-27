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

**Current task:** 1 `done` and committed. **Next: the encoding promotion below, then task 2.**
**Blocked on:** nothing — but *see "Before task 3" under Tasks*. Task 3 cannot be dispatched
until the encoding the fixture builder settled is in [format.md](../format.md).

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

## Before task 3 — the encoding must land in `format.md`

**Read this before dispatching anything.** Task 1 turned up the one thing that changes this
plan, and it is recorded here rather than inside a task because it sits between them.

[format.md](../format.md) names JSON keys for the physics fields and for extension data **and
for nothing else**. Encoding `rigged` therefore meant settling the key names, the tree encoding,
the collection orders, the current version number, the compression per entry kind and the gravity
default. Those are now settled — by a fixture builder, in `crates/rigel-fixtures/README.md`,
which is not a document anybody is told to read.

That is the wrong home for them, for the reason [vocabulary.md](../vocabulary.md) gives about
terms invented by an implementer: they do not propagate, they collide. The next agent to touch
this subsystem reads `format.md`, finds no key names, and invents a second set.

**So the next commit on this branch promotes that table into [format.md](../format.md)**, under
the `[proposed]` sections it belongs to, and the builder's README becomes a pointer. It is a
`doc-writer` job with the routing done first; the full list is in *Decisions made* and
*Assumptions* below.

It is listed before task 2 only because task 3 depends on it — a test that loads a file has to
know the encoding, so the acceptance tests cannot be commissioned until this lands. Task 2 is
independent of it and may go either side.

**The other `Contradicts docs` findings are routed at task 10, not now**, with one exception
already closed: every member crate now inherits the workspace `license`, so that entry is fixed
rather than outstanding.

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

**Decisions made:**

- **The current format version is `1.1`.** `rigged:old-version` must declare "an earlier version
  this build upgrades", and an earlier *minor* exists only if the current one is not the first.
- **The node tree is a flat `nodes` array with `parent` references and a top-level `root`.** See
  *Tried and rejected*. Canonical order is depth-first preorder, and order among siblings is draw
  order; every other collection is ordered by identifier, keypoints by position.
- **Every JSON key not named by `format.md` is the term `vocabulary.md` sanctions, in
  `snake_case`.** `format.md` names keys for the physics fields and for extension data and for
  nothing else, so encoding `rigged` meant choosing the rest. The full list is the table in
  `crates/rigel-fixtures/README.md` and **it belongs in `format.md`** — nobody but this builder
  has settled any of it. This touches `format.md`'s `[proposed]` sections throughout.
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

**Assumptions:**

- Every JSON key name, the current version, the gravity default, the compression per entry kind
  and the canonical collection orders, as *Decisions made* records. All unresolved: they belong
  in `format.md` and were settled by this builder alone.
- `uvs` are in the texture's **authored** pixel space, which is the reading that makes
  `format.md`'s cropping paragraph — the record carrying the crop rectangle and the authored
  dimensions so the runtime can reconstruct the space — do its stated work.
- `rigged`'s catalogue row describes four parts, not three. See *Contradicts docs*.
- `PROVISIONAL_NODE_LIMIT`, the figure `rigged:near-limit` sits under, is invented and will be
  wrong. The builder is committed so the fixture regenerates when the real figure lands.

**Deferred:**

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
