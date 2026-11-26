---
name: tech-design
description: Settles stack, boundaries, contracts and data shape into the project's design docs, and establishes the vocabulary. Reached from the workflow skill once requirements exist for an area.
---

# Technical design

Produces the documents an implementer builds against — and, more importantly in a multi-agent
project, the **shared vocabulary** that lets independently-briefed agents produce work that
composes.

## What a design document is for here

Two agents given different slices of the same system produce incompatible code when they do not
share names and boundaries. That tells you what belongs in these documents and what does not:

| In | Out |
|---|---|
| Names, and what they mean | Algorithms |
| Boundaries — what owns what | Implementation detail the code states better |
| Contracts between pieces | Anything a reader gets faster from the source |
| Data shape, and what changes it | How a function is written |
| Invariants, and why each exists | Narration of behaviour |

**A document is an overview; the code is the detail.** Someone working on a subsystem reads its
code either way, so a document that narrates the code is a second copy that goes stale and leaves
the reader with two sources and no way to tell which is current.

## 1. Vocabulary first

Before anything else, settle `docs/vocabulary.md` for this area. Every term the requirements
introduced, and every term the design will. One definition each, nowhere restated.

This is the highest-leverage part of the whole workflow and it is tempting to skip because it
feels like bookkeeping. Terms disagreed on at this point become incompatible code later, found
during review at many times the cost.

## 2. Constraints you inherit

Requirements may already dictate technical choices. Carry them across as constraints **with
their reasons** and mark them as inherited, not derived. A later agent needs to know whether a
choice may be challenged — an inherited constraint with no reason gets "fixed".

## 3. Research what you cannot decide

When a choice turns on something nobody knows — whether an API supports a thing, what a service
costs, how a library behaves under load — route to `research` rather than guessing or deferring
silently.

Do not dispatch research for background reading. The `research` skill will not frame a question
that has no decision attached, and that constraint is deliberate.

## 4. Depth

Same rule as requirements: **detail where a wrong guess is expensive to undo.** Data shape,
boundaries, contracts and permission models are expensive, because everything downstream assumes
them. Internal structure of a single module is cheap, because the implementer will choose better
with the code in front of them.

Where something is genuinely undecided, tag it `[open]` and say what would settle it. Nothing
should be built that assumes an answer to an `[open]` question.

## 5. Write it

Into `docs/`, following `docs/documentation.md` — one document per reader question, flat, in the
index, every section tagged. New invariants go in `docs/invariants.md` **and** `CLAUDE.md`; they
are one list in two halves.

Decisions go in `docs/decisions.md` only if they meet its admission test: a newcomer would take
it for a mistake and break something by "fixing" it. Most decisions do not meet it. Read that
document's own test rather than promoting by feel.

## Done when

- Vocabulary is settled for the area.
- Every requirement has somewhere to live in the design, or is explicitly out of scope.
- Inherited constraints are recorded with reasons.
- Open questions are tagged `[open]` with what would settle them.
- The user agrees. Committed.

Then route to `plan-phase`.
