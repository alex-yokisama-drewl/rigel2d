---
name: research
description: Frames a research question against a pending decision, dispatches the researcher agent, and files the dated result. Reached from the workflow skill or from tech-design when a choice turns on something nobody knows. Framing happens here because it is the hard part; the searching is the agent's.
---

# Research

Two jobs, and only one of them is the agent's. Framing the question is the hard part and it needs
the conversation; searching is mechanical once the question is right.

## The rule that makes research useful

**A research question is not ready until it names the decision it feeds.**

Research with no decision attached produces a report nobody reads. That is the common failure —
not bad research, unused research. If you cannot name the decision, you do not want research; you
want to think, or to ask the user what they are trying to settle.

## 1. Frame it

Work out, with the user where it is not obvious:

| Part | Why |
|---|---|
| **The decision** this feeds | Without it the report has no shape and no reader |
| **The options** currently live | A verdict is a choice among these |
| **What would change the decision** | Tells the agent which evidence is decisive and which is background |
| **A depth budget** | Diminishing returns are real; most decisions are not close |

If the decision is already effectively made and research would only justify it, say so and skip
it.

## 2. Dispatch

`researcher`, with those four parts. Nothing else — it does not need the project context, and
giving it the conversation is how a researcher returns your own assumption with citations.

Dispatch several in parallel for genuinely independent questions. Do not split one question
across agents.

## 3. Take the verdict

It returns a path to `docs/research/<topic>.md` and a one-line verdict.

- **Read the verdict and the evidence.** A verdict you accept without reading the evidence is a
  guess with extra steps.
- **If it returned a survey rather than a verdict**, resume it with `SendMessage` and ask for the
  call. Do not do its job.
- **If it says the evidence does not support a verdict**, that is an answer. Decide under
  uncertainty, explicitly, and record what would change it.

## 4. Land the decision

The research file is not the decision. Make the decision with the user, and write it where
decisions live — the design document that states the resulting rule, and `docs/decisions.md` only
if it meets that document's admission test.

Link the research file from wherever the decision landed. An unlinked research file is
re-researched.

## Why research files carry dates

`docs/research/` stales on a different clock than design does. A pricing answer or an API
capability from four months ago is a liability sitting next to design decisions that are still
true. The date in the header is what lets a reader tell the difference at a glance, and
`doc-reviewer` flags old ones every cycle.
