# Phase 1 — the spike and the runtime

The current phase: which subsystems get built, in what order where order matters.

**Status:** agreed 2026-11-26. **S0, S1 and S2 are done** — S0 and S1 on 2026-11-27, S2 on
2026-11-28. `rigel-format` is the first subsystem with production code: a puppet file can be
read, validated and written, and the rest of the phase builds on it.

**S3 and S4 are both clear to start**, and are independent of each other.

## Where this phase ends, and why there

This phase ends at the runtime. It does not reach Studio or Live.

[README.md](../README.md) records that Studio's and Live's own designs are deliberately not
written — the design pass covered the format, the evaluation pass, the scene, the runtime's
boundaries and the stack, in the build order [overview.md](../overview.md) sets out. There is
nothing to decompose the two applications from, and a plan that listed them would be listing
guesses. Studio's design is a `tech-design` pass in the next cycle, and the concern entries this
phase seeded are waiting for it.

What this phase delivers is the library both applications embed, proven against generated
workloads, with the one question that could reopen the stack settled first.

## Subsystems

Ordered. `Blocks on` names what genuinely must come first; everything else may move.

| | Subsystem | Done when | Blocks on |
|---|---|---|---|
| S0 | **Transparency and alpha-output spike** | **Done 2026-11-27** — measured on Linux and Windows, recorded in [research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md). The `[open]` in [performing.md](../requirements/performing.md#which-platforms-need-an-obs-plugin-decided) is resolved and the Game Capture capability tension answered. **macOS was not attempted** — no Mac was reachable — so Syphon alpha stays unobserved, which [platform.md](../requirements/platform.md) permits at its tier | Nothing |
| S1 | **Workspace and the push gates** | **Done 2026-11-27** — the workspace builds from a clean clone with `--locked`, the gates that have a subject run and are required on `master`, `scripts/test-record.sh` records a result, and *Running locally* and *Running it* were corrected from a verbatim container walk. Gate 3 waits for a counter baseline; [stack.md](../stack.md#on-push-proposed) says which gates are live and why | S0 |
| S2 | **`rigel-format`: read, validate, write** | **Done 2026-11-28** — a puppet archive round-trips byte-identically, the hand-authored malformed corpus is rejected with stale bindings accepted as a state, load limits bite before allocating, extension data survives untouched, and every write goes through the one atomic path. The third-party gate was run by the owner and found one defect, now fixed. A review sweep found the reader's work and memory unbounded in four shapes and a late task closed them; *What S3 and S4 inherit* carries what that leaves | S1 |
| S3 | **The workload generator and the fixture corpus** | The reference and stress puppets are generated from the committed dimension table in [platform.md](../requirements/platform.md), both validate, and the generated puppet doubles as the conformance fixture [puppet.md](../requirements/puppet.md) requires | S2 |
| S4 | **The evaluation pass** | A generated puppet evaluates to a posed tree for given channel input, in the order [evaluation.md](../evaluation.md) sets; one parameter has one driver, enforced by the validator; stale bindings behave as specified; the name-resolution counter is zero | S2 |
| S5 | **The render stage: command stream and backend** | Evaluation output produces an explicit command stream countable with no GPU present; the wgpu backend submits it; the warm-up pass begins every configuration the loaded puppet can reach, enumerated from puppet data; GPU resource creations after the first presented frame are zero on both workloads | S3, S4 |
| S6 | **The counter gate, growth and soak** | Counters have a committed baseline, CI fails above it, growth is asserted per dimension on the stress workload, and the soak shows counters and resident bytes flat | S5 |

### What does not block what

S3 and S4 both depend only on S2 and are independent of each other. S3 is placed first because it
gives S4 its fixture, not because S4 needs it to exist. Ordering them the other way costs only a
hand-built fixture.

S0 blocked everything, and that was the point rather than an inconvenience — see below.

### Why S0 was first

S0 went before everything else because a bad answer would have reopened the stack choice. It did
not: the stack survives, and what the spike found instead was a colour hazard no document had
anticipated. The conditions it imposed are in
[stack.md](../stack.md#what-the-transparency-spike-settled-decided), the rule it produced is in
[runtime.md](../runtime.md), the route it decided is in [decisions.md](../decisions.md), and the
measurements are in
[research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md).

### What S2 onward inherit

| | |
|---|---|
| **Gate 2 is green over an empty suite** | **Closed by S2.** `scripts/tests-ran.sh` fails on a zero, null or missing count, carries its own self-test, and runs in CI after the recording step. What it does *not* yet catch is a count that merely falls, which is now an `ops` entry |
| **Gate 5 cannot see the dependency graph** | It reads first-party source. The first registry dependency is the point at which "gate 5 green" and the claim in [decisions.md](../decisions.md) start to diverge, and the approved `security` entries — the SHA-pinning check and the locked-graph check — are what close it. **Due with the first dependency, not later** |
| **The Linux package list is a floor** | Established against crates with no dependencies. Whichever subsystem first pulls in windowing or the GPU establishes its packages the same way and extends [stack.md](../stack.md#running-locally-proposed) — verbatim in a bare environment, which is the standard the `ops` entry now sets |
| **A bare nested test directory is silently skipped** | [testing.md](../testing.md#harness-proposed) prescribes `tests/acceptance/` and `tests/counters/`; cargo discovers `tests/*.rs` and `tests/<dir>/main.rs` only. The first subsystem to add one needs a `main.rs` or an explicit `[[test]]` |
| **The backend is pinned on Windows** | Never left to wgpu's default selection, which chose the one backend that crashed on the machine measured. [stack.md](../stack.md#what-the-transparency-spike-settled-decided) carries the reason, alongside the surface-format and colour-space condition from the same spike |
| **Read the vendored wgpu source before porting** | Due in S5, where the render stage meets wgpu. The spike's port found the API differs across majors enough to make guesses worthless; [stack.md](../stack.md#moving-to-a-new-wgpu-major-decided) carries what it cost |
| **The S0 instrument is in git, not the tree** | S1 discarded `spike/alpha-sender/` as planned. [research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md) carries the commit and the commands to recover it, which matters because macOS and a second Windows machine are both still unmeasured |

### What S3 and S4 inherit

| | |
|---|---|
| **The reader's cost is bounded by figures that are mostly provisional** | S2 found and closed four shapes where the reader's work or memory was unbounded — a quadratic string scan, a quadratic duplicate-member scan, an uncounted parse tree and three unbounded collections. The figures that now bound them are in `crates/rigel-format/src/limits.rs`, and all but `AxisCount` are `Provisional`. **S3 is where several stop being guesses**, because it commits the workload dimensions the limits table is supposed to be derived from |
| **Every `ValidRange::max` is still `[open]`** | Pending the integrator-stability sweep due with the evaluation pass. The validator reads the table, so a figure landing there is the whole change — no code moves. **S4 owns this** |
| **The fuzz target bounds one input, not the campaign** | `drive` runs on a worker thread with a per-input timeout, which is what catches a hang. A hung thread cannot be killed, so the run fails and the thread dies with the process. A subsystem adding a slower loader path should check the figure still has margin |
| **The corpus is written by our own canonical writer** | So every fixture satisfies every writer rule, and no fixture can show that the *reader* accepts a container ordinary tooling wrote. This is what let a defect reach a human gate. The `data` entry that closes it is table-driven over the canonical-serialisation list and is not yet written |
| **A texture's pixel area is unbounded** | Each side is capped; the product is not, so a legal texture is a gigabyte of pixels at upload. The format crate only reads the header. **Deferred by the owner rather than owed to a subsystem** — it is a hardening item, and nothing should block on it |

### Raised by the S1 review and not settled

Nothing here blocks a subsystem. The unadopted entries are for `finish-cycle`.

**The licence is settled** — `MIT OR Apache-2.0`, chosen by the owner on 2026-11-27 and carried
by `LICENSE-MIT` and `LICENSE-APACHE` at the repository root. S1 had written the same string into
the workspace manifest without authority and it was removed; the manifest now declares it because
the licence exists, rather than in place of one. The repository was already public and therefore
already published, which is why this was settled ahead of the subsystem rather than before a
release.

| | |
|---|---|
| **Proposed checklist entries not adopted** | The S1 review proposed more `docs/concerns/` entries than were approved — a secret scan in CI, a committed fixture set for gate 5, a `Cargo.lock` checksum assertion, a `panic = "abort"` grep, and a container job for the documented setup. They are in the review's record on `feature/s1-workspace-and-gates`. `finish-cycle` should revisit them rather than re-derive them |

## Acceptance criteria

Commissioned from `acceptance-planner` per area, which must not be shown an implementation.
Timing differs by area because the `[open]` sections that would change the criteria resolve at
different points.

| Area | When | Why then |
|---|---|---|
| [puppet.md](../requirements/puppet.md) | **Done**, and refreshed 2026-11-27 — [puppet-acceptance.md](../requirements/puppet-acceptance.md) | Its `[open]` on the compatibility promise is a signal from the owner and changes no criterion. The refresh was due because the physics, track and authoring-payload decisions moved the requirement under criteria already written |
| [platform.md](../requirements/platform.md) | With S6 | The counter half can be commissioned now. The soak length is settled at four hours and the deadline-miss figures stand as written, provisional until the calibration sitting |
| [performing.md](../requirements/performing.md) | **Due now** | S0 is done and the `[open]` that decides the output path is resolved |
| [authoring.md](../requirements/authoring.md) | **Partly done** — the operations settled on 2026-11-26 are in [authoring-acceptance.md](../requirements/authoring-acceptance.md); the rest is next cycle | Studio is not in this phase |

The planner's most useful output is the gaps it finds in the requirements. Those route back into
`docs/requirements/` before the subsystem they affect starts.

### The tests, and why they are not written yet **[decided]**

[requirements/README.md](../requirements/README.md) draws the line: *a criterion is confirmed
failing when its test is written, not when it is specified.* The criteria are design-blind and
durable. The tests that implement them are not yet writable, and the reason is concrete rather
than a matter of sequencing.

`crates/rigel-format/src/lib.rs` is a doc comment. There is no type to call, no error to match
on, and no `rigged` fixture — and `rigged` is a puppet archive, so building one by hand is the
format's own job. An agent writing the tests today would have to invent the crate's public API
and its encoding, which is the design and the implementation both, and would destroy the
independence the tests exist to provide.

**The tests are therefore written at the start of S2, before the implementer runs** — the
ordinary `feature` shape, `acceptance-planner` and then `module-dev`. Two things have to hold,
and both are easy to lose:

- **The implementer never writes the acceptance tests.** Tests written by the author of the code
  encode the code's assumptions ([testing.md](../testing.md)). The criteria are already written
  design-blind, so the test-writing agent *implements* them rather than deriving them, and it
  may read [format.md](../format.md) — a test that loads a file has to know the encoding.
- **`rigged` is hand-authored and uses no project code.** A fixture produced by our writer and
  then read by our reader asserts only that we agree with ourselves. This is the rule
  [puppet-acceptance.md](../requirements/puppet-acceptance.md) already puts on the generated
  workloads, and it holds here for the same reason.

### What blocked S2 — settled 2026-11-27

**S2 was gated on requirements gaps, not just on S1.** Writing criteria against
[puppet.md](../requirements/puppet.md) turned up places where the requirement did not say enough
for a criterion to exist without inventing the answer, recorded in the *Criteria not written*
table at the foot of
[puppet-acceptance.md](../requirements/puppet-acceptance.md#criteria-not-written-open). Three of
its rows bore on the format. Commissioning the criteria early is what surfaced them before any
code could settle them by accident, which was the point of the early commission.

| Row | How it was settled |
|---|---|
| **Which kinds of data are authoring-only** | **Dissolved rather than answered.** The authoring-only set turned out to be an import identifier and a content hash per part plus two axes — tens of kilobytes against a workload budgeted at hundreds of megabytes of texture — so the split was withdrawn and the player loads the whole structure. [FILE-6](../requirements/puppet-acceptance.md#n-one-file-decided) and FILE-7 are retired; [decisions.md](../decisions.md) carries why |
| **Whether physics is a node** | **It is**, and it also displaces its own node's transform and contains only physics, which makes a pendulum chain an ordinary subtree. It reads its own previous-frame transform, so the one-frame delay is deliberate and does not accumulate down a chain. Its reference to the parameter it drives is incidental, so a deletion cascade can never reach artwork through one |
| **The term for the part of an animation that drives one parameter** | **Track.** Defined in [vocabulary.md](../vocabulary.md), which also records *lane* — inochi2d's word for it — as a term this project does not use |

The table's remaining rows still do not block S2: the determinism tolerance is a figure the
evaluation pass needs, the unparseable-extension texture case is disclosed rather than fixable,
and the unknown-polarity import path decides whether a criterion is live or dormant on a path
Studio owns.

**Raised by that round and not blocking S2.** The workload table has no dimension for tracks per
animation or keyframes per track, because the corpus counted `.motion3.json` files rather than
the curves inside them. Until that research is done, animation cost is unbounded by the table and
no load limit covers it. It is in [open-questions.md](open-questions.md) and is due before S4.

## Due before the subsystem that needs them

Not blocking the plan. Each is due before its subsystem starts, and named here so it is not
discovered late.

| Question | Due before | Lives in |
|---|---|---|
| Tracks per animation and keyframes per track, which no workload dimension covers | S4 | [open-questions.md](open-questions.md) |
| Whether a texture has a pixel-area limit, and what figure | Not a subsystem — **deferred by the owner on 2026-11-28**, on the principle that the tool works first and is hardened second | [open-questions.md](open-questions.md) |

The rows that stood here on the soak length, on a puppet changed on disk, and on configuration
in the write path have all been answered in the documents that own them, and were struck when
that was noticed rather than left to be rediscovered.

## Carried out of this phase

Seeded by the planning sweeps on 2026-11-26 and not actionable until the applications have a
design. They are recorded so the Studio cycle inherits them rather than re-deriving them.

- The accessibility entries, which bite on Studio's custom canvases and Live's control surface.
- The ops entries covering packaging, release and failure on a user's machine.
- The documentation defects both sweeps found, which are being routed separately and are not
  waiting on this plan.
