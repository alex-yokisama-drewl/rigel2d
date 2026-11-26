---
name: workflow
description: The development workflow for this project - orienting, then routing to the right lane. Use whenever work is about to start, continue or finish - "let's build X", "what's next", "pick up where we left off", "this is done", "fix this bug", "here are the requirements", "plan the next phase". Start here rather than guessing a lane. Not for questions about existing code, which need no workflow.
---

# Workflow

Every piece of work enters through here. The routing table is the skill — the lanes are separate
files and this chooses between them.

## Step 0 — orient before doing anything

The session-start hook has already printed the branch, the state of `docs/wip/` and the last
recorded test result. If that output is not in front of you, get it:

```
git branch --show-current && git status --short
ls docs/wip/
```

Then read, in this order:

1. `CLAUDE.md`
2. `docs/vocabulary.md` — before anything else that involves naming. Load-bearing.
3. If the user named a thing they are working on: **the matching wip document, and only that
   one.** Ignore the others. A session works on one feature at a time.

## Route

| What you find, or what was asked | Lane |
|---|---|
| No requirements for this area, or only an informal description | **`requirements`** |
| Requirements exist, no technical design for this area | **`tech-design`** |
| Design exists, no `docs/wip/plan.md` | **`plan-phase`** |
| A plan, and a subsystem ready to start | **`feature`** |
| A wip document for the named feature | **`feature`**, resume branch. Say where things stand before touching anything |
| Its tasks all `done` | **`feature`**, finish branch |
| Every subsystem in the plan `done` | **`finish-cycle`** |
| A defect or change below the threshold | **`small-fix`** |
| A question the codebase can answer | Dispatch `explorer`. No lane |
| A question the codebase cannot answer | **`research`** |

**Never skip to implementation because the request sounds simple.** The routing table is the
skill. If a request sounds simple, the lane is probably `small-fix`, which is a lane and not an
absence of one.

## When two lanes seem to apply

Take the earliest. A request to build something whose requirements are vague is a `requirements`
request wearing a feature's clothes, and building it first means discovering that later and at
greater cost.

## Dispatching

Anything you hand to a subagent follows `.claude/rules/dispatch.md`. Read it before your first
dispatch in a session. The parts that get skipped and should not:

- **Name the fixed doc set** in every brief — `CLAUDE.md`, `docs/vocabulary.md`,
  `docs/invariants.md`, plus the specific documents that task needs.
- **State what you have already ruled out.** The agent cannot see this conversation.
- **Never hand an agent your conclusion** when its job is to reach one.
- **On `BLOCKED`, resume with `SendMessage`** rather than dispatching again. The agent keeps its
  context; a new dispatch throws it away.

## What does not go through a lane

- Answering a question. Dispatch `explorer` or read the code.
- Reading documentation to the user.
- Anything the user explicitly said to just do.

Doing a small thing directly is fine. Doing a *change* directly, without a lane, is how the
process stops being the process.
