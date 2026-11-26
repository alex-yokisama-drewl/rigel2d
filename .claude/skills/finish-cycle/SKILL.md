---
name: finish-cycle
description: Closes a development phase - documentation sweep, harvesting human-gate catches into the concern checklists, pruning, the status report, and (on a calibration project) reading the trace. Reached from the workflow skill when every subsystem in the plan is done.
---

# Finish a cycle

The phase's subsystems are all `done`. This closes it.

Every step here is skippable-feeling and none is optional. The work already functions; what
remains is what keeps the *next* phase from starting against a rotten doc set.

## 1. Documentation sweep

**Mechanical first.** `scripts/docs-check.sh`. Fix everything it reports before dispatching
anything — a mechanical failure usually explains what a reviewer would otherwise report as a
contradiction.

**Then judgment.** Dispatch `doc-reviewer`. It finds what a script cannot: contradictions,
rules stated twice, prose that drifted from the code, accumulated history, stale research.

Route its findings — you hold the doc map — and dispatch `doc-writer` with the routed items and
their target files.

Doc rot is not hygiene. A contradiction left here becomes the input to the next phase's briefs,
and an agent briefed from a contradictory doc set produces work that is wrong in a way nobody
can trace back.

## 2. Harvest the catches

**This is the step that makes the process improve rather than merely run.**

Collect every *Caught at gate* entry from the phase's feature documents — everything a human
found that automation did not. For each:

- Which concern dimension does it belong to?
- What check would have caught it? Prefer `script` or `test`; accept `judgment` only when
  neither is possible.
- Write it into `docs/concerns/<dimension>.md` under **This project**, with its provenance:
  caught at a gate, on this date, on this feature.

Then **write the automation**, where the entry says `script` or `test`. An entry promising a
check that does not exist is worse than no entry, because it reads as covered.

A bug found twice is a process failure. This step is what prevents the second time.

## 3. Prune

A checklist that only grows stops being read, which fails exactly like having none.

Go through `docs/concerns/` *This project* sections. An entry that has **never fired** and has
**no incident behind it** is a candidate for deletion — it was speculative, and provenance is
what lets you tell. Propose deletions to the user; do not delete unilaterally.

Also prune `docs/research/` entries that are stale and no longer feed a live decision, and
`docs/decisions.md` entries that no longer meet its admission test.

## 4. Update the index

`docs/README.md` carries the project's status, and **that is the report surface.** Rewrite its
status section:

- what is now built;
- what was deliberately **not** built, as a refusal or a deferral on record rather than a
  pending task;
- what the next phase inherits.

**Do not create a per-phase report file.** It would be a third place truth lives, it accumulates,
and it is history — which the doc set deliberately refuses. The durable report is this section;
the narrative belongs in your message to the user.

## 5. Retire the plan

`docs/wip/plan.md` is deleted when the phase ends. An empty `docs/wip/` is the normal state
between phases, not a missing file.

## 6. Report to the user

In chat, not in a file. What was built, what was refused and why, what the next phase inherits,
and what the phase cost against what it was expected to cost.

## 7. Read the trace — calibration projects only

Skip if `.calibration/` does not exist.

Read `.calibration/trace.jsonl` against the questions in the suite's `design/calibration.md`,
and write the answers into `docs/wip/calibration.md`:

| Signal | Decision it feeds |
|---|---|
| `BLOCKED` rate and causes | Brief quality — is the main session under-specifying, and about what? |
| Rework after review | Where defects enter the pipeline |
| Human-gate catches | Which concern dimension is weakest |
| Dispatches returning nothing actionable | Which agents to delete |
| Tokens per agent per role | Model assignment |
| Resume-after-`BLOCKED` vs re-dispatch | Whether the halt protocol is honoured at all |
| Skills that never fired | Routing problems, or dead lanes |

**Anything that never fired is a candidate for deletion rather than refinement.** The likely
failure of a workflow this size is not that a piece is wrong, but that half of it goes unused
and nobody can tell which half.

## Done when

- `docs-check.sh` is clean and the reviewer's findings are applied.
- Every gate catch is a checklist entry, and its automation exists.
- Pruning is proposed and resolved.
- `docs/README.md` states the current status.
- `docs/wip/` is empty.
- The user has the report.
