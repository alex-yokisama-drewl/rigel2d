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

**Current task:** 2 — `scripts/test-record.sh` records a result
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

- *(none yet)*

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

### 2. `scripts/test-record.sh` records a result — `todo`

The script already exists and reads `.claude/test-cmd`, which already holds
`cargo test --workspace`. Neither has ever run against a workspace. Make it work and record a
result.

**Done when:** the script runs the suite, writes a result keyed by `HEAD`'s SHA in the shape
`.claude/rules/testing.md` describes, and a second run against a different commit reports the
earlier result as stale rather than as an error. The suite is empty and the result says so
honestly — that is what makes "confirmed failing" mean something from S2. No user gate.

### 3. The Linux push-gate workflow — gates 1, 2 and 4 — `todo`

`cargo fmt --check` and `cargo clippy -- -D warnings`; `cargo test --workspace`;
`scripts/docs-check.sh`. In the order [stack.md](../stack.md#on-push-proposed) sets, each
blocking a merge. CI actions pinned to commit hashes, per the supply-chain concern entry.

**Done when:** a push to this branch runs all three and they pass, observed on GitHub rather
than reasoned about. **Needs the user's eyes**: the branch protection that makes a gate actually
block a merge is a repository setting, not a file, and only the repository owner can set it. What
to look at: the gates are *required* on `master`, not merely present and green. A gate that runs
and reports but does not block is the failure this task exists to prevent.

### 4. Gate 5 — the no-outbound-connection assertion — `todo`

A script over our own source, per `concerns/security.md`. It must fail on an attempt to open an
outbound connection and say which file did it. Its scope is our code and not a dependency's;
[decisions.md](../decisions.md) and `wip/open-questions.md` both record that the plugin and
dependency cases are deliberately unresolved, so the script states its own scope rather than
implying a stronger claim.

**Done when:** the script is in the workflow, passes against the empty workspace, and **fails
against a deliberate violation added and reverted in the same session** — an assertion never
observed failing is not an assertion. No user gate.

### 5. Windows and macOS build jobs — `todo`

The build exercised on both, per [stack.md](../stack.md#on-push-proposed). They say that it
builds and nothing more; [platform.md](../requirements/platform.md) is explicit that no stronger
macOS claim is founded, and `concerns/ops.md`'s *Environments* says hosted Windows and macOS
runners establish that it builds and that structural assertions pass.

**Done when:** both jobs run green on a push, and neither claims more than a build in its name or
its output. No user gate.

### 6. Correct *Running locally* and *Running it* — `todo`

[stack.md](../stack.md#running-locally-proposed) asks for the Linux package list "established on
a clean machine rather than remembered", and carries an **Unverified** warning that this task
removes. `CLAUDE.md`'s *Running it* table currently says the workspace and `rust-toolchain.toml`
do not exist; after task 1 they do, and both documents move together.

The walk is done in a container with nothing preinstalled, which establishes the package list
honestly. **A container is not a desktop**: it says nothing about a real install with a display
server and a GPU driver.

**Done when:** both documents describe steps that were walked, and the
`ops` entry *The project runs locally from its documented setup* has something true to bind to.
**Needs the user's eyes.** What to look at: the package list, on your own clean install if you
have one to spare — a container cannot tell you whether a real desktop needs something it already
had. What would count as wrong: a package that turns out to be unnecessary, or a missing one that
the container happened not to need.

### 7. Discard the S0 spike scaffolding — `todo`

[plan.md](plan.md) records `spike/alpha-sender/` as throwaway scaffolding S1 discards. Deleting
it is only half the task: [README.md](../README.md)'s status paragraph and the *Reproducing it*
section of
[research/2026-11-27-transparency-spike.md](../research/2026-11-27-transparency-spike.md) both
point at the directory, and macOS and a second Windows machine are still unmeasured, so the
pointer has to become one a future measurer can follow rather than simply vanish.

**Done when:** the directory is gone, both documents name the branch and the merge commit the
instrument lives in instead of a path that no longer resolves, and `scripts/docs-check.sh` is
clean. No user gate.

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**
The workspace: `Cargo.toml` at the root, `rust-toolchain.toml` pinning 1.95.0, a committed
`Cargo.lock`, and the four crates [stack.md](../stack.md#workspace-proposed) names, each a shell
with the specified dependency graph wired and nothing implemented.

**Decisions made:**
- **Gate set, glibc deferral, toolchain pin, committed lockfile and CI provider** — all five are
  in *Decisions* above with their reasons. Two touch `[proposed]` sections and say so.

**Contradicts docs:**
- `CLAUDE.md`'s *Running it* table and `stack.md`'s *Running locally* both describe a path nobody
  has walked. Task 6 is the correction; recorded here because it is a known contradiction on
  entry rather than one discovered during the work.

**Traps:**

**Assumptions:**

**Deferred:**
- **The glibc 2.31 floor**, to the release job. See *Decisions*.
- **Gate 3, the counter gate**, to S5 and S6, which create its subject and its baseline.

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
