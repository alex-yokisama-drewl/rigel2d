# Feature: S1 — the workspace and the push gates

**Branch:** `feature/s1-workspace-and-gates`
**Phase subsystem:** S1 in [plan.md](plan.md), which blocks every subsystem after it

Turns an empty repository into one that builds, tests and gates. Done means every crate in
[stack.md](../stack.md#workspace-proposed)'s workspace table builds from a clean checkout, the
push gates that have a subject run and block a merge, `scripts/test-record.sh` records a result
keyed by commit SHA, and the *Running locally* steps in [stack.md](../stack.md) and the
*Running it* table in `CLAUDE.md` are corrected to what was actually walked rather than what was
intended.

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** all tasks done, and the review of 2026-11-27 is applied. Resume at **C. Finish, step 2** — routing the handoff
**Blocked on:** nothing

---

## Assignment

### Goal

Establish the workspace, the pinned toolchain and the push gates, so that every subsystem after
this one lands against a suite that runs and a gate that blocks.

### Out of scope

- **Any production behaviour.** The crates are empty shells with the dependency graph wired.
  Parsing, validation, evaluation and rendering are S2 onward, and a crate that does something
  here is scope creep that S2 then has to unpick.
- **The counter gate** — gate 3 in [stack.md](../stack.md#on-push-proposed)'s table. It has no
  subject until the runtime produces a command stream and the workload generator produces a
  puppet to run it against. [plan.md](plan.md) puts the baseline in S6.
- **The release job**, and everything under *Building and shipping* in
  [ops.md](../concerns/ops.md). No S1 output reaches a user. See the glibc decision below.
- **Acceptance tests.** [plan.md](plan.md) commissions them per area; S1 supplies the crate and
  the command that let one be confirmed failing, which is the thing that was missing.
- **The structural UI assertions.** `egui_kittest` needs a surface to assert against and there
  is none. The row stays in [testing.md](../testing.md) and binds from the first UI.

### Doc set

- `CLAUDE.md` — current state and invariants. **Its *Running it* table is wrong and this feature
  is what makes it right**
- `docs/vocabulary.md` — **load-bearing**, read first
- `docs/invariants.md` — no invariant is implemented here, but the gates are where several are
  eventually asserted
- `docs/stack.md` — *Workspace*, *Running locally*, *On push*, *Test command*. This is the
  document S1 delivers against, and the one it corrects
- `docs/testing.md` — *Harness*, which says what kind of test lives where, and that no test
  requires a GPU
- `docs/concerns/security.md` — *What the applications may not do* and *Supply chain*; the first
  is gate 5, the second becomes live the moment this feature creates a dependency manifest
- `docs/concerns/ops.md` — *Environments* for what CI can and cannot establish, and the glibc
  ceiling under *Building and shipping*
- `.claude/rules/testing.md` — the verification ordering, and that test results are state on disk

### Acceptance criteria

**None apply, and that is correct rather than an omission.** The acceptance documents cover the
puppet format and the editor's operations. S1 produces no behaviour to assert on; what it
produces is the place an assertion can live and the command that runs it.

### Concern criteria

From [concerns/](../concerns/), the entries that bind here:

- **`security`, *No outbound network connection is opened*** — tagged `judgment` with a `script`
  wanted. Gate 5 is that script. It passes vacuously against empty crates, which is the point:
  it is in place before the first dependency that could break it.
- **`security`, *Dependencies are locked with hashes, CI actions pinned to commit hashes***  —
  tagged "once there is a dependency manifest". **This feature creates the first one**, so the
  entry stops being deferred here. CI actions are pinned to commit hashes, not tags.
- **`ops`, *A deploy is repeatable from a clean checkout*** — the workspace builds from a fresh
  clone with nothing cached.
- **`ops`, *The project runs locally from its documented setup*** — tagged `test`, and the
  reason task 6 exists. The entry says the instructions are followed literally on a clean
  environment, not remembered.

### Already ruled out

Do not re-propose these; each was settled with the project owner before this branch existed.

- **Wiring all five push gates now, with gate 3 failing until the runtime exists.** A permanently
  red required check across several subsystems trains everyone to ignore a red build.
- **Wiring only gates 1 and 4.** The remaining gates would then arrive scattered across five
  subsystems, which is where a gate gets skipped.
- **Enforcing the glibc 2.31 floor in a container now.** Deferred — see the decision below.
- **Raising the Linux floor to glibc 2.35** to match a runner image that still exists. That is a
  requirements change affecting who can run the result, not a build convenience, and it was not
  made.
- **A floating `stable` toolchain.** [ops.md](../concerns/ops.md)'s release entry requires a
  pinned toolchain and `--locked`, and a floating channel breaks CI on someone else's schedule.

---

## Decisions

Made for this feature. Anything meeting [decisions.md](../decisions.md)'s admission test moves
there immediately — a narrow set. This list is everything else, and it survives every task
cleanup.

- **S1 wires the push gates that have a subject: 1, 2, 4 and 5.** Gate 3, the counters, waits
  for S5 and S6, where both the command stream and the committed baseline are created.
  **Touches [stack.md](../stack.md#on-push-proposed)'s *On push*, which is `[proposed]`** — the
  table is not edited. It states the gate set the project commits to; S1 records which of them
  are live, and the rest arrive with their subjects.
- **The glibc 2.31 floor is not enforced by the push gates.** It is a property of a shipped
  artifact and no S1 output is shipped. The symbol-version check that
  [ops.md](../concerns/ops.md) already specifies under *Building and shipping* is what enforces
  it, in the release job. **Touches an `[proposed]` ops entry** — deferred deliberately, and
  recorded so it reads as a decision rather than an oversight. Push gates run on
  `ubuntu-latest`.
- **The toolchain is pinned to 1.95.0**, the compiler this was developed against, in
  `rust-toolchain.toml`. Edition 2024.
- **`Cargo.lock` is committed.** The workspace produces binaries, and
  [ops.md](../concerns/ops.md) requires release builds to pass `--locked`.
- **CI is GitHub Actions**, matching the forge the repository and the issue tracker are already
  on. [ops.md](../concerns/ops.md)'s *Environments* assumes hosted CI and names no provider.

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- **`|` as the field separator in `no-outbound.sh`'s pattern table.** Half the regexes contain
  alternation, so `${entry%%|*}` split them through the middle of themselves and two patterns
  silently matched nothing. The gate's own self-test is what caught it. The delimiter is `###`.
- **A `license` key in the workspace manifest.** `MIT OR Apache-2.0` was written in without
  authority — nothing in `docs/` settles a licence, and a manifest is not where that gets
  decided. Removed, with the reason in the file. **For the owner**: this is a real open
  question, not a tidy-up.

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section, and this
document is deleted at merge, so anything here is routed before that happens.

- *(none yet)*

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The workspace, the toolchain pin, and four crates that build — `done`

The crates live under `crates/`. `rigel-format` has no dependencies at all, which is the
property [stack.md](../stack.md#workspace-proposed) asks for and the thing to re-check whenever
something is added to it.

### 2. `scripts/test-record.sh` records a result — `done`

The script and `.claude/test-cmd` both already existed; neither had run against a workspace.
Staleness was already handled — the session-start hook compares the newest record's SHA against
`HEAD` and says so.

**What was missing:** the record said `passed: true` over a suite with nothing in it. It now
carries `assertions_run`, and the script says on a zero that a pass means nothing was checked.
Read that field before trusting a green record, in CI or at a session start.

### 3. The Linux push-gate workflow — gates 1, 2 and 4 — `done`

`.github/workflows/push-gates.yml`. One job with ordered steps rather than parallel jobs,
because the ordering is the point — a formatting mistake fails in seconds instead of after the
suite. Steps are named for the gate numbers in [stack.md](../stack.md#on-push-proposed), so the
correspondence to that table can be checked rather than trusted. `actions/checkout` is pinned to
a commit hash; no toolchain action is used, because `rust-toolchain.toml` and the runner's
rustup do the job with one less third-party dependency.

**Approved 2026-11-27**, and branch protection applied to `master` on the owner's instruction:
the three checks required, `strict` so a branch must be current, admins included because
[ops.md](../concerns/ops.md) says CI failure blocks a merge for the maintainer too.

**`required_linear_history` is off, deliberately.** The `feature` lane lands each subsystem as a
merge commit because the per-task commits are each an agreed unit; turning linear history on
would forbid exactly that.

**Protection names the checks as strings.** Renaming a job in the workflow silently stops that
check being required — the protection does not fail, it just stops covering anything. Rename and
re-point together, or do not rename.

### 4. Gate 5 — the no-outbound-connection assertion — `done`

`scripts/no-outbound.sh`, a deny-list over first-party Rust source, in the workflow.

**Listening is not connecting**, and that is the thing to preserve if the script is ever
rewritten. Live binds a port and waits for a source
([performing.md](../requirements/performing.md)); `bind`, `listen`, `accept` and `recv` are
allowed, and only outbound initiation is refused. A blanket ban on `std::net` would forbid the
input port Live is specified to open.

The script prints its own scope on every run: it cannot see what a dependency does, so a pass
means "we did not write one", never "none can happen".

Observed failing on a planted `TcpStream::connect` and passing on a planted `UdpSocket::bind`,
both reverted.

### 5. Windows and macOS build jobs — `done`

A `builds` job matrixed over both runners, separate from `gates` so each is its own check.

**Named `builds only — <os>` deliberately.** A job called "windows tests" would be read as a
stronger claim than it makes, and [platform.md](../requirements/platform.md) is explicit that no
stronger macOS claim is founded. The structural UI assertions that `stack.md` expects here join
when there is a surface to assert against.

### 6. Correct *Running locally* and *Running it* — `done`

**Walked** on a clean `ubuntu:24.04` container. The documented path needed `curl` and
`build-essential` and nothing else: the first walk failed on `linker cc not found`, which is the
kind of thing remembering does not produce. Every command in the block was run, including both
`cargo run` targets.

**The list is a floor.** The crates have no dependencies, so nothing yet pulls in the Wayland or
X11 headers or the Vulkan loader. Both documents say so rather than implying the list is final.

**The gate on this task was withdrawn as mis-specified.** It asked the owner to check the list
against a clean install they do not have; the only thing they could have done is run a container,
which is what had already been run. A gate whose answer can only be re-derived from the evidence
already in hand is not a gate — it is a round trip. The container walk stands as the evidence,
with its limits stated in the document.

**What is genuinely unverified, and now says so in [stack.md](../stack.md#running-locally-proposed):**
a container has no display server, no GPU driver and no compositor. Nothing here speaks for
those, and the S0 spike is the evidence that they are where the failures live. That is a real
gate for whichever subsystem first depends on them — on the owner's own desktop, where the
question is answerable.

### 7. Discard the S0 spike scaffolding — `done`

[plan.md](plan.md) records `spike/alpha-sender/` as throwaway scaffolding S1 discards. Deleting
it is only half the task: [README.md](../README.md)'s status paragraph and the *Reproducing it*
section of
[research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md) both
point at the directory, and macOS and a second Windows machine are still unmeasured, so the
pointer has to become one a future measurer can follow rather than simply vanish.

Both documents now give the git coordinate — commit `dafc348`, or the branch
`feature/s0-transparency-spike` — with the `git show` and `git checkout` lines to recover it.
Recovery is framed as expected rather than exceptional, because macOS is unmeasured and the
Windows findings rest on one machine.

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**
The workspace: `Cargo.toml` at the root, `rust-toolchain.toml` pinning 1.95.0, a committed
`Cargo.lock`, and the four crates [stack.md](../stack.md#workspace-proposed) names, each a shell
with the specified dependency graph wired and nothing implemented.

`scripts/test-record.sh` now records `assertions_run` alongside the pass flag, and says on a zero
that a pass means nothing was checked.

The push gates: `.github/workflows/push-gates.yml`, carrying gates 1, 2, 4 and 5 on Linux in the
order [stack.md](../stack.md#on-push-proposed) sets, and a separate `builds` job on Windows and
macOS that claims only a build. `scripts/no-outbound.sh` is gate 5.

`spike/alpha-sender/` is gone, and the documents that pointed at it now point into git.

**Decisions made:**
- **Gate set, glibc deferral, toolchain pin, committed lockfile and CI provider** — all five are
  in *Decisions* above with their reasons. Two touch `[proposed]` sections and say so.

**Contradicts docs:**
- `CLAUDE.md`'s *Running it* table and `stack.md`'s *Running locally* both described a path
  nobody had walked. Corrected in task 6, from a container walk.
- **`stack.md`'s *On push* gives gate 1 as `cargo clippy -- -D warnings`.** On a workspace root
  that lints neither the other members nor any test target. The workflow runs
  `--workspace --all-targets`, which is what the gate is understood to mean. The document's
  shorthand is weaker than its intent.
- **`concerns/ops.md`'s *The project runs locally from its documented setup* is tagged `test`,
  and no test exists.** The container walk in task 6 was done by hand. `concerns/README.md` is
  explicit that a `script` or `test` tag whose check does not exist "reads as covered", so this
  entry is currently making a promise nothing keeps. Automating the walk is a CI job that
  installs an OS and a toolchain on every run, which is why it was not simply added here — it
  needs a decision about cadence.

**Traps:**
- **A durable file that names a phase subsystem goes stale when the phase ends.** The workflow
  comment explaining gate 3's absence pointed at `docs/wip/plan.md` and named S6. The reason is
  phase-independent — no command stream, no committed baseline — and it now says that instead.
- **A `set -e` script died on a `grep` that matched nothing, and so wrote no record at all.**
  `scripts/test-record.sh` has `pipefail` set and `set -e` restored before the parse, so when
  `cargo` emitted no `test result:` line — which is exactly what a compile error does — the
  assignment aborted the script before the record was written. The previous commit's record then
  stood, saying `passed: true`, and the session-start hook reported it as merely stale, which is
  indistinguishable from nobody having run anything. **A broken build read as last known good.**
  Every stage of the pipeline is now neutralised, not just the first — the second and third
  greps can match nothing too, which is how the first attempt at this fix still failed.
- **A deny-list gate decays silently, so it needs its own test.** `no-outbound.sh --self-test`
  plants every forbidden form and every allowed one and asserts the verdict, and runs before the
  real scan on every invocation. It immediately caught a delimiter bug that had disabled two
  patterns. A gate that is only verified the day it is written is a gate that will one day be
  green for the wrong reason.
- **The documented build needed a package nobody had written down.** `cargo build` on a clean
  Ubuntu fails with `linker cc not found`; `build-essential` was never in `stack.md` because
  every machine that had ever run the command already had it. This is the whole argument for the
  `ops` entry that says the instructions are followed literally on a clean environment rather
  than remembered.
- **A recorded `passed: true` meant "the runner exited 0", not "anything was checked."** With an
  empty workspace the suite is green and the session-start hook reports it as a result for
  `HEAD`, so a session would read "tests pass" where nothing existed to pass. This is the failure
  [concerns/README.md](../concerns/README.md) names — a check that does not exist reads as
  covered — reached through the result file rather than through a checklist entry. The record
  now carries `assertions_run`. **Anything reading a result reads that field too.**

- **"Walked" is a claim that has to mean *verbatim*.** The first version of `stack.md`'s
  *Running locally* was labelled as walked while the walk had quietly used `apt-get update`, run
  as root, and exported PATH by hand — none of which were in the block. Three of its lines failed
  on a literal run. It is now executed verbatim in a bare container, with only a `sudo` shim,
  and the lines that exist solely because it failed without them say so.

**Assumptions:**
- **The package list is a floor established against a workspace with no dependencies.** It is
  correct today and will be incomplete the moment windowing or GPU crates land.
- **Gate 5 sees source, never the dependency graph.** Once anything is depended on, "gate 5
  green" and the claim in `decisions.md` start to diverge, and nothing currently measures the
  gap. The script says so on every run.

**Deferred:**
- **The Linux package list beyond `curl build-essential`**, to whichever subsystem first depends
  on windowing or the GPU. Nothing pulls those in yet, so a list naming them now would be
  remembered rather than established — the exact failure task 6 exists to prevent.
- **The glibc 2.31 floor**, to the release job. See *Decisions*.
- **Gate 3, the counter gate**, to S5 and S6, which create its subject and its baseline.

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
