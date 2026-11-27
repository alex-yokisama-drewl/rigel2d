---
name: feature
description: The per-feature loop - starting a feature, working its tasks, and finishing it. Reached from the workflow skill when a phase plan exists. Covers the assignment document, dispatching the implementer, review, and landing the work with its documentation.
---

# Feature

One subsystem from `docs/wip/plan.md`, built on its own branch and landed as a unit.

Two documents, defined in `docs/documentation.md`:

- **`docs/wip/plan.md`** — the phase. Reaches the default branch. Moved on by the feature that
  completes a subsystem, in the same commit that deletes that feature's document.
- **`docs/wip/<slug>.md`** — this feature. Lives **only on `feature/<slug>`**, and is the
  assignment, the progress record and the handoff in one file.

Both answer one question: **can a session that has never seen this work pick it up from this
file alone?** That is also the test a subagent brief has to pass, which is why the assignment and
the brief are nearly the same document.

## Route

| State | Go to |
|---|---|
| No document for this feature | **A. Start** |
| A document, tasks outstanding | **B. Work** — say where things stand before touching anything |
| A task `awaiting approval` | **B.3** — the gate |
| All tasks `done` | **C. Finish** |

---

## A. Start

Four things, in order. Do not write code partway through this list.

**1. Settle the outstanding decisions.** Before any branch exists. Check the docs touching this
area for `[open]` and `[proposed]` sections — the unresolved questions are already there. Settle
them with the user.

**2. Branch.** `git checkout <default> && git checkout -b feature/<slug>`.

**3. Write `docs/wip/<slug>.md`** from `templates/feature.md`.

**This document is the implementer's brief.** Write it as `.claude/rules/dispatch.md` says a
brief must be written, because that is what it becomes:

- the goal, and what is explicitly out of scope;
- **the named doc set** the implementer gets — `CLAUDE.md`, `docs/vocabulary.md`,
  `docs/invariants.md`, plus the specific documents for this work. Naming them is your job; the
  implementer must not go reading the whole set, and the brief is what makes that unnecessary;
- the acceptance criteria that apply, from `docs/requirements/<area>-acceptance.md`;
- **the concern criteria** that apply — the `docs/concerns/` entries the planning sweep produced
  for this scope. Stating them here is what makes analytics a specification rather than a later
  catch;
- what you have already ruled out, and why;
- the decisions from step 1.

A decision meeting `docs/decisions.md`'s admission test goes there **immediately**, not at the
end of the feature.

**4. Decompose into commit-sized tasks and agree them.** For each task write **how it ends** —
what makes it done, and whether it needs the user's eyes. That answer is provisional, and the
point of writing it in advance is that it is not decided in the moment when you are most eager
to continue.

Commit the document, then begin.

---

## B. Work

One task at a time, in the agreed order.

**1. Build.** Either directly, or by dispatching `module-dev` with the assignment and the named
doc set. Dispatch when the work is self-contained enough that the scoped context is a saving;
build directly when you would spend more briefing than implementing.

**Never dispatch two implementers at once against this working tree.**

**2. Verify.** Formatter, linter, type checker, the suite. `scripts/test-record.sh` so the result
is recorded against the commit and nothing re-runs it later to find out.

**3. The gate.** If the task needs the user's eyes, mark it `awaiting approval` in the document
and say **what to look at and what would count as wrong**. A gate that says "check the page"
wastes the gate.

Wait for "all good, no more fixes needed". Not "looks right". Nothing functional changes after
that point.

**Anything the user catches at a gate becomes a checklist entry.** Not later — note it in the
feature document now, under *Caught at gate*, so `finish-cycle` can harvest it. This is the rule
that makes the process improve rather than merely run.

**4. Update the document.** Mark the task `done`, strip working detail that no longer serves
resumption, keep the decisions and the handoff slots.

**5. Commit.** Code and document together, one commit per task. The pre-commit hook will refuse
if the document is untouched.

### While working

- **A decision changes what the software is** — behaviour a user sees, a trade-off, scope → ask.
- **A decision touches an invariant** → stop and ask. Always.
- **Any other decision, including one that touches a `[proposed]` or `[open]` section** → make
  it, proceed, record it with its reason, and flag in the handoff that the section was touched.
  See *Who decides what* in `CLAUDE.md`.
- **An approach fails twice** → stop trying variants. Record it under *Tried and rejected* with
  one line on why. That section survives every cleanup and is what stops a fresh session
  re-proposing it.
- **The plan is wrong** → change it. Add, split or drop tasks as you learn, and tell the user
  what changed. An outdated commit plan is worse than a revised one.
- **An implementer returns `BLOCKED`** → answer and resume with `SendMessage`. Do not
  re-dispatch; that discards its context. If you cannot answer either, ask the user — do not
  relay a guess as fact.
- **An implementer reports `Contradicts docs`** → do not let it pass. Record it; it is routed at
  finish, and it is the cheapest drift detection the project has.

---

## C. Finish

Nothing here is optional, and this is the step most likely to be skipped because the feature
already works.

**1. Review.** Dispatch in parallel:

- `code-reviewer` with the diff and the assignment — and **not** your reading of the code;
- `concern-analyst` in **review mode**, once per dimension that applies to this scope.

Fix what is blocking. Record what you consciously decline, with the reason.

**2. Route the handoff.** Take the implementer's handoff slots and decide, per item, where it
belongs: this trap into the subsystem's document, this decision into `decisions.md` if it meets
the admission test, this `Contradicts docs` item means a named document is now wrong.

**You** do the routing — you hold the doc map. Then dispatch `doc-writer` with the routed items
and the named target files. It loads the documents and makes the edits.

**3. Verify the documentation.** `scripts/docs-check.sh`. Fix what it reports.

**4. Update `plan.md` on the branch.** Mark the subsystem done, strip detail that no longer
serves the next session, and add what the next subsystem inherits — scaffolding it must delete,
a question this feature opened.

**5. Delete `docs/wip/<slug>.md`.** This and step 4 go in the **same commit**, the last before
merge: one commit retires the feature document and moves the phase on. A step after the merge is
a step that gets skipped, and it fails silently.

**6. Push and open a PR.** Merge commit, not squash — the per-task commits are each an agreed,
approved unit. Summarise what changed rather than restating the diff.

If a feature is abandoned, delete its document and its branch, and say so in `plan.md` on a
documentation-only branch. A stale document in `docs/wip/` reads as live work.

---

## Constraints that override convenience

- **Never promote a `[proposed]` to `[decided]`** because you built on it. Ask.
- **Never mark a task done before approval**, and never batch tasks into one approval.
- **Never let a document and the branch disagree.** That is why each task's document update is
  inside the task's commit, and why `plan.md` moves on in the commit that deletes the feature
  document rather than in a follow-up.
- **Never edit an acceptance test to make it pass.** They were written without sight of the
  implementation, which is the whole point. If one is wrong, that is a finding.
