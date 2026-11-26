# Feature: <name>

**Branch:** `feature/<slug>`
**Phase subsystem:** <which entry in wip/plan.md this delivers>

<One paragraph: what this feature does and what "done" means.>

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** <number and name, or "not started" / "all tasks done — ready to finish">
**Blocked on:** <nothing, or what>

---

## Assignment

### Goal
<One sentence.>

### Out of scope
<Explicit. Scope creep is the second most common dispatch failure.>

### Doc set
<The documents an implementer reads, and no others. Always CLAUDE.md, docs/vocabulary.md,
docs/invariants.md, plus:>
- <path> — <why it is needed>

### Acceptance criteria
<From docs/requirements/<area>-acceptance.md — which apply here. Reference, do not restate.>

### Concern criteria
<From the planning sweep: the docs/concerns/ entries that apply to this scope. These are
specifications, not later checks — analytics events, overlap assertions, audit rules.>

### Already ruled out
<What has been considered and rejected before this work started, with reasons. Without this an
implementer re-treads them.>

---

## Decisions

Made for this feature. Anything meeting **[decisions.md](../decisions.md)'s admission test moves
there immediately** — a narrow set. This list is everything else, and it survives every task
cleanup.

- **<decision>** — <why, in one or two lines>

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- **<approach>** — <why it lost>

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

- **<what was caught>** — <which dimension, and what check would have caught it>

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. <name> — `todo`
<What it covers.>
**Done when:** <what makes it complete, and whether the user needs to see it. If so: what to look
at, and what would count as wrong. Provisional — adjust as you learn.>

### 2. <name> — `todo`
<>
**Done when:** <>

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**
<What exists now that did not before.>

**Decisions made:**
<Each with its reason. Flag any that touched a [proposed] or [open] section.>

**Contradicts docs:**
<Anything a document states that turned out to be false. Name the document and the claim. The
most valuable slot here — this is the only point in the process where someone reads a document
and the code it describes at the same moment.>

**Traps:**
<What cost real debugging time and would cost it again.>

**Assumptions:**
<Made under ambiguity, still unresolved.>

**Deferred:**
<Scoped out, and why.>

---

<!--
Cleanup rule for a completed task: keep the heading, the `done` status, and anything a fresh
session still needs. Delete the working detail. Decisions and handoff entries are never deleted
here — they move out at finish, to the documents that state the resulting rules.
-->
