---
name: requirements
description: Turns whatever the user has - a full document, scattered notes, or one sentence - into declarative requirements in docs/requirements/. Reached from the workflow skill. Asks until there is enough, sweeps the concern dimensions, and records deferrals as [open] rather than leaving gaps.
---

# Requirements

Produces `docs/requirements/<area>.md`: what the system does and how it behaves, never how it is
built.

This is a conversation before it is a file. It ends when there is enough to design against, not
when everything is known.

## 1. Take what exists

The user may hand over a finished document, a few notes, or a sentence. All three are the same
starting point: read it, restate it in the project's vocabulary, and show the restatement. A
misunderstanding caught here costs a sentence.

If `docs/vocabulary.md` has no word for something central, that is a vocabulary decision and it
is made now, with the user. Terms invented later by an implementer do not propagate.

## 2. Sweep the concerns

This is the step that distinguishes this from writing down what you were told.

Dispatch `concern-analyst` in **planning mode**, once per dimension in `docs/concerns/`, in
parallel. Each gets: the draft requirements, the design docs if any, the scope, and its
dimension name.

Each returns missing criteria, questions for the user, and proposed project-specific checklist
entries.

Then:

- **Missing criteria** → fold into the requirements. This is the shift-left: the implementer
  will be told to emit the analytics event rather than caught later for omitting it.
- **Questions** → ask the user. Do not answer them yourself.
- **Proposed checklist entries** → show the user, get approval, add the approved ones to
  `docs/concerns/<dimension>.md` under *This project* with their provenance. **Do not add
  unapproved entries.** These become binding checks, and a list the user did not agree to is a
  list nobody believes.

A general checklist cannot know that this project's join codes can be brute-forced or that its
coordinates are hand-authored and unvalidated. That knowledge arrives here, once per project, and
then persists.

## 3. Ask until there is enough

Ask in batches, not one at a time. Prefer questions whose answers change what gets built.

**Depth rule: write detail where a wrong guess is expensive to undo.** That the admin UI exists,
who can reach it, which fields stay editable after publish, whether changes are audited — those
are requirements, because getting them wrong invalidates the data model. The field layout of each
editor is not, because getting it wrong costs an afternoon.

Over-specification has a running cost and not just a writing cost: every detail is context
someone pays to load, and a claim that implementation will contradict. A requirements document
wrong in twenty small ways stops being consulted.

**Deferral is written down, never left as a gap.** "Per-datatype admin screens — `[open]`,
specified per datatype at implementation" is a recorded decision. A gap is a thing an
implementer silently fills with a guess.

## 4. Write it

`docs/requirements/<area>.md`. Declarative throughout — behaviour, rules, constraints. Where the
user arrived with a technical preference, record it as a **constraint with its reason**
("Postgres: ops will not run a second datastore"), not as a derived design decision. The reason
is what tells a later agent whether it may be challenged.

Every section carries `[decided]`, `[proposed]` or `[open]`. Add the document to
`docs/README.md`.

## Done when

- Every concern dimension has been swept and its output resolved.
- Every open question is either answered or tagged `[open]` with the deferral stated.
- The user agrees it is enough to design against.
- It is committed.

Then route to `tech-design`.
