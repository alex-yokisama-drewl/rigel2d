# Dispatch rules

The contract between the main session and a subagent. Two halves, because it has two readers:
**Writing a brief** is for the session dispatching; **Returning a result** is for the agent.

Read this before dispatching anything. Every agent definition restates the status block inline —
that duplication is deliberate, because a link is a thing an agent can skip.

---

# Writing a brief

A subagent starts with an empty context. It sees your brief and nothing else: not the
conversation, not what you already ruled out, not what the user said five minutes ago. It cannot
ask you a question mid-run.

**If you cannot write a clear brief, you do not understand the task well enough to delegate it.**
Do it yourself instead.

## Required in every brief

| Part | Why |
|---|---|
| **The goal**, in one sentence | What a successful return looks like |
| **Scope** — what is in, what is explicitly out | Scope creep is the second most common failure |
| **What you have already ruled out**, and why | Without it, the agent re-treads your dead ends |
| **Where to look** — specific files, docs, directories | Pointing beats pasting. Never paste a file an agent can read |
| **The fixed doc set** | See below |
| **What to return** | A verdict, a path, a list of findings. Name the shape |

## The fixed doc set

Every agent working inside the project gets these named in its brief, and reads them before
anything else:

- `CLAUDE.md` — current state and invariants
- `docs/vocabulary.md` — the terms. **Load-bearing.** An agent using the wrong word for a thing
  produces work that does not compose with anyone else's
- `docs/invariants.md` — what must never break

Then name the specific docs relevant to this task. Naming them is your job; an agent told to
"read the docs" reads the wrong ones or all of them.

## What never goes in a brief

- **Your conclusion**, when the agent's job is to reach one. An agent handed a hypothesis returns
  it confirmed. Give a reviewer the code and the question, never your reading of it.
- **A file's contents**, when a path would do.
- **An unresolved ambiguity.** Resolve it, or state the assumption explicitly so the agent can
  flag it back.

## After the return

Verify. A `STATUS: COMPLETE` is a claim, not a guarantee, and the whole protocol is a prompt
convention that a model mid-task can ignore.

**On `BLOCKED`:** answer and resume with `SendMessage`, which keeps the agent's context. Do not
re-dispatch — that discards everything it learned and pays to rebuild it. If the block names a
question you cannot answer either, ask the user. Do not relay your own guess as fact.

**Most blocks resolve with "yes, do that"** against the agent's own `Default:` line, which costs
one cheap message.

---

# Returning a result

*(This half is addressed to the agent.)*

## Halt conditions

**Returning `BLOCKED` with a precise question is a successful outcome. Guessing is the failure.**

Stop — do not guess, do not work around — when:

- information you need is not in your brief and not on disk;
- **an approach has failed twice.** Do not try a third variant of it;
- the task as briefed contradicts what you find in the repository;
- finishing would require acting outside the scope you were given;
- you would need a destructive or hard-to-reverse action.

The two-failures rule is a counter, not a judgment call. A judgment-based version never fires,
because from inside the work the next variant always looks like the one that works.

## Status block

End with exactly one of these.

```
STATUS: COMPLETE
<the result, in the shape the brief asked for>
```

```
STATUS: BLOCKED
Question: <one specific, answerable question>
Tried: <what you attempted, and how it failed>
Default: <what you would do if told "use your judgment">
Done so far: <work already completed, so it is not repeated on resume>
```

```
STATUS: PARTIAL
Done: <what completed cleanly>
Blocked on: <the rest, in the BLOCKED shape above>
```

`Default:` is the most useful line you write. Most blocks are resolved by approving it, which
costs one message instead of a conversation. Writing it also forces the ambiguity to be precise
enough to hold a position on — and often reveals there was no real question.

Use `PARTIAL` whenever some work finished cleanly. One ambiguity should not zero out a run.

## Returning large output

The return channel is a summary. If your result is longer than a few paragraphs — a plan, a diff,
a report, a test suite — **write it to a file and return the path.** Say what is in it in two
lines.

---

# The handoff

Any agent that **changes the repository** ends its work with a handoff in these slots, written
into its feature document. Leave a slot empty rather than omitting it; an absent slot is
ambiguous between "nothing to report" and "forgot".

| Slot | Contents |
|---|---|
| **Built** | What exists now that did not before |
| **Decisions made** | Each with its reason. Flag any that touched a `[proposed]` or `[open]` section |
| **Contradicts docs** | Anything a document states that turned out to be false. See below |
| **Traps** | What cost real debugging time and would cost it again |
| **Assumptions** | Made under ambiguity and still unresolved |
| **Deferred** | Scoped out, and why |
| **Tried and rejected** | So nobody re-proposes it |

**You do not decide where any of this belongs in the documentation.** Report it in these slots.
The main session holds the doc map and routes each item; `doc-writer` performs the edits. This
is why you were not given the whole doc set to read.

**`Contradicts docs` is the most valuable slot.** You are the only participant that reads a
document and the code it describes at the same moment. Report every mismatch, including ones that
look too small to matter and ones you worked around. Say which doc and which claim.
