# Phase 1 — the spike and the runtime

The current phase: which subsystems get built, in what order where order matters.

**Status:** agreed 2026-11-26. **S0 and S1 are done** (2026-11-27); S2 onward are not started.
The workspace builds, the push gates run and block a merge, and no production code is
implemented yet — the crates are shells.

**The requirements round that gated S2 is done** — 2026-11-27, on
`requirements/s2-format-gaps`. What it settled is below under *What blocked S2*. **S2 is clear to
start**, with one piece of design work due first: [format.md](../format.md) must say what a
physics node stores and how the per-kind containment declaration is written. Neither is a
requirements question and neither was settled by that round.

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
| S2 | **`rigel-format`: read, validate, write** | A puppet archive round-trips byte-identically; the validator rejects a hand-written malformed corpus and accepts stale bindings as a state; load limits are enforced before allocating; extension data survives untouched; every write goes through the one atomic path | S1, **and the requirements gaps below** |
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
| **Gate 2 is green over an empty suite** | Intended at S1 and wrong from S2. The recorded result carries `tests_run`, and the approved `ops` entry *Gate 2 fails when the workspace runs zero tests* is the check that makes it bind. **S2 writes that check**, because S2 is the first subsystem with tests to lose |
| **Gate 5 cannot see the dependency graph** | It reads first-party source. The first registry dependency is the point at which "gate 5 green" and the claim in [decisions.md](../decisions.md) start to diverge, and the approved `security` entries — the SHA-pinning check and the locked-graph check — are what close it. **Due with the first dependency, not later** |
| **The Linux package list is a floor** | Established against crates with no dependencies. Whichever subsystem first pulls in windowing or the GPU establishes its packages the same way and extends [stack.md](../stack.md#running-locally-proposed) — verbatim in a bare environment, which is the standard the `ops` entry now sets |
| **A bare nested test directory is silently skipped** | [testing.md](../testing.md#harness-proposed) prescribes `tests/acceptance/` and `tests/counters/`; cargo discovers `tests/*.rs` and `tests/<dir>/main.rs` only. The first subsystem to add one needs a `main.rs` or an explicit `[[test]]` |
| **The backend is pinned on Windows** | Never left to wgpu's default selection, which chose the one backend that crashed on the machine measured. [stack.md](../stack.md#what-the-transparency-spike-settled-decided) carries the reason, alongside the surface-format and colour-space condition from the same spike |
| **Read the vendored wgpu source before porting** | Due in S5, where the render stage meets wgpu. The spike's port found the API differs across majors enough to make guesses worthless; [stack.md](../stack.md#moving-to-a-new-wgpu-major-decided) carries what it cost |
| **The S0 instrument is in git, not the tree** | S1 discarded `spike/alpha-sender/` as planned. [research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md) carries the commit and the commands to recover it, which matters because macOS and a second Windows machine are both still unmeasured |

### Raised by the S1 review and not settled

Nothing here blocks a subsystem. The licence is the owner's; the unadopted entries are for
`finish-cycle`.

| | |
|---|---|
| **The project has no licence** | S1 wrote `MIT OR Apache-2.0` into the workspace manifest without authority and it was removed; nothing in `docs/` settles one, and a manifest is not where that is decided. There is no `LICENSE` file. Due before anything is published |
| **Proposed checklist entries not adopted** | The S1 review proposed more `docs/concerns/` entries than were approved — a secret scan in CI, a committed fixture set for gate 5, a `Cargo.lock` checksum assertion, a `panic = "abort"` grep, and a container job for the documented setup. They are in the review's record on `feature/s1-workspace-and-gates`. `finish-cycle` should revisit them rather than re-derive them |

## Acceptance criteria

Commissioned from `acceptance-planner` per area, which must not be shown an implementation.
Timing differs by area because the `[open]` sections that would change the criteria resolve at
different points.

| Area | When | Why then |
|---|---|---|
| [puppet.md](../requirements/puppet.md) | **Done** — [puppet-acceptance.md](../requirements/puppet-acceptance.md) | Its `[open]` on the compatibility promise is a signal from the owner and changes no criterion |
| [platform.md](../requirements/platform.md) | With S6 | The counter half can be commissioned now. The soak length is settled at four hours and the deadline-miss figures stand as written, provisional until the calibration sitting |
| [performing.md](../requirements/performing.md) | **Due now** | S0 is done and the `[open]` that decides the output path is resolved |
| [authoring.md](../requirements/authoring.md) | **Partly done** — the operations settled on 2026-11-26 are in [authoring-acceptance.md](../requirements/authoring-acceptance.md); the rest is next cycle | Studio is not in this phase |

The planner's most useful output is the gaps it finds in the requirements. Those route back into
`docs/requirements/` before the subsystem they affect starts.

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
| What blocks S2 | S2 | *What blocks S2* above, and the table it points at |

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
