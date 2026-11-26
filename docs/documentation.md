# Documentation guide

Where documentation lives, how it is structured, and how it stays true. Read this before adding
a document, moving one, or writing down a decision. It governs, and it outranks an agent's
instincts about documentation.

## Status tags **[decided]**

Every substantive section carries one:

- **[decided]** — settled. Change only with a reason, and record the reason in
  [decisions.md](decisions.md).
- **[proposed]** — settled enough to build against, but likely to want revision once something
  has been built on it. **In force, not pending:** code follows it today. The tag records
  *confidence*, not *authority*.
- **[open]** — genuinely undecided. Nothing should be built that assumes an answer.

Variants that appear and mean what they say: `[decided — decide later]`,
`[decided — supported, unused]`, `[decided — temporary scaffolding]`.

**Never promote a `[proposed]` to `[decided]` without the user saying so.** Agents in
particular: a tag is a claim about the project's confidence, not about yours. Having built on a
section is not evidence that it is settled.

## Where a document lives **[decided]**

| Location | What goes there |
|---|---|
| `CLAUDE.md` | Agent entrypoint. Invariants, vocabulary summary, pointer to the index. Always loaded, so it stays short. **Orientation, never history** |
| `docs/README.md` | **The** index, and the project's status. The only list of documents that exists |
| `docs/*.md` | Everything else. Flat — no subdirectories except the ones below |
| `docs/requirements/` | What the system must do. Declarative, per area |
| `docs/concerns/` | The cross-cutting checklists. See its own README |
| `docs/research/` | Dated findings against decisions |
| `docs/wip/` | Work in flight. Different lifecycle — see below |
| Next to a module | Only by explicit confirmation, and only as a pointer |

**`CLAUDE.md` states the current state, never how it was reached [decided].** It answers *what
is true now, and what must I not break*. No phase log, no "since then", no paragraph per
feature. History belongs in [README.md](README.md) and the alternatives in
[decisions.md](decisions.md); CLAUDE.md is loaded into every session and does not have room to
grow. **When something lands, edit what CLAUDE.md already says rather than appending to it.**

A module may carry a README only when the material is genuinely local and would be noise in
`docs/`. **Confirm with the user in every such case.** When one is agreed, it points into
`docs/` rather than restating it. Two places to look is the failure the flat layout exists to
avoid.

## `docs/wip/` — work in flight **[proposed]**

Everything else in `docs/` describes the project. `docs/wip/` describes **what is currently
being built**, and is the only deliberately temporary part of the set.

| File | Scope | Lives on | Deleted |
|---|---|---|---|
| `wip/plan.md` | The current phase: which subsystems, in what order | The default branch, edited on branches | When the phase ends |
| `wip/<feature>.md` | One feature: assignment, decisions, progress, handoff | **Its feature branch only** | In the last commit before merge |
| `wip/calibration.md` | Workflow observations. Calibration projects only | The default branch | When calibration ends |
| `wip/open-questions.md` | Questions requirements work is waiting on, and approvals pending | The default branch | When the requirements are agreed |

Three rules make this work:

1. **A feature document never reaches the default branch.** Created on its branch and deleted
   there, so no two branches conflict over one. Resuming is `git checkout <branch>` and reading
   its file.
2. **`plan.md` is moved on by the feature that completes a subsystem**, in the same commit that
   deletes that feature's document. The merge that lands the work also lands the phase update,
   and a reviewer sees both. Two branches in flight can conflict on it; that is a one-line
   conflict git shows you, which is a better failure than a follow-up commit everyone forgets.
3. **Both are for resumption, not history.** The test: *can a session that has never seen this
   work pick it up from this file alone?* Anything that does not serve that gets stripped.

The same test is what a subagent brief must pass, which is why a feature document doubles as the
implementer's assignment.

## One document, one question **[decided]**

The unit is **a question a reader arrives with**, not a subsystem. A document answering two
unrelated questions should split; two answering halves of one should merge.

Granularity has a floor. Splitting past it fragments context — a reader, human or agent, ends up
loading fewer of the relevant documents, not more. Under about forty lines with no distinct
reader question, it belongs inside its neighbour.

## How much detail **[decided]**

**A document is an overview, and the code is the detail.** Someone working on a subsystem will
read its code either way, so a document narrating the code is a second copy that goes stale —
and the reader now has two sources and no way to tell which is current.

A document carries:

- **the rules**, including the ones no reader would infer and the ones a plausible change would
  break;
- **the shape** of the subsystem — what the pieces are and how they relate;
- **the reason a rule exists**, in a clause or a sentence, because a rule without one gets
  "fixed";
- **the traps**, where something cost real debugging time and would cost it again.

It leaves out: what a function does, what a field holds, how a loop is written, how many tests
there are, and step-by-step behaviour a reader can follow in the source. Name the class or the
file instead, and let the reader go there.

**Length is a symptom worth checking.** Several hundred lines usually means one of three things
has crept in: prose explaining code, a rule stated in two documents, or the document's own
history. All three are cuttable without losing anything.

## Do not state counts of what you also show **[decided]**

Durable text must not state the number of things it also lists, or restate values a table
already carries. A count in prose drifts the moment the list is edited, and makes the list
expensive to edit — every addition becomes two edits, and the second is the one that gets
forgotten. Point at the thing, or restructure so its shape is visible.

This applies to every document here and to anything published from them. It does not apply to a
message in chat, which is read once.

## Current state versus history **[decided]**

The commonest way a document set rots is the same rationale living in two places with only one
getting updated.

| | [decisions.md](decisions.md) | Everything else |
|---|---|---|
| Tense | What was decided, when, and **what was rejected** | What is true **now** |
| Lifecycle | Not edited to stay current — a reversal is a new entry. Pruned when they fail the admission test | Rewritten freely; no history kept |
| Contains | The alternatives, the trade, the date | The rule and enough rationale to apply it |
| Admits | Only what a newcomer would take for a mistake and "fix" | Everything true, surprising or not |

**The decision log is deliberately short.** Most decisions do not belong in it — not obvious
ones, not arbitrary ones, not documentation or process ones, not deferrals. Its own admission
test governs; read it before adding.

A `[decided]` section states its rule and *why the rule is the way it is* — that much travels
with the rule or the next reader "fixes" it. It must not re-argue the alternatives; that is the
decision log's job, and it links there.

## Updating **[decided]**

- **A decision made in conversation is not made until it is written down.** Update the document
  in the same change that makes the decision, not afterwards.
- **Adding a document means adding a line to [README.md](README.md).** A document not in the
  index does not exist, because nothing will find it.
- **Renaming or deleting means fixing every inbound link.**
  `grep -rn '](oldname.md)' docs/ CLAUDE.md`
- **Changing an invariant means updating [invariants.md](invariants.md) *and* CLAUDE.md.** They
  are one list in two halves and must never disagree.
- Vocabulary is defined once, in [vocabulary.md](vocabulary.md). CLAUDE.md summarises it; no
  other document redefines a term.

`scripts/docs-check.sh` enforces the mechanical half of this list. Run it before finishing
anything.

## Who writes what **[decided]**

An implementer reports facts in the handoff slots of its feature document and **does not decide
where they belong**. Deciding destinations requires the whole doc map, and giving an implementer
the whole doc set destroys the context isolation that made dispatching it worthwhile.

| Step | Who |
|---|---|
| Report what happened, in fixed slots | The implementer |
| Decide which document each fact belongs in | The main session |
| Load the target documents and edit them | `doc-writer` |
| Find contradictions and drift afterwards | `doc-reviewer`, and `docs-check.sh` first |

The handoff's **Contradicts docs** slot is the most valuable thing the process produces: the
implementer is the only participant that reads a document and the code it describes at the same
moment.

## What not to document **[decided]**

- Anything the code already states plainly. A document listing function signatures will be wrong
  within a month.
- Changelogs of the documentation itself.
- Speculative future design. That is `[open]`, or it is not written.
