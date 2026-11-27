# Testing and verification rules

How work is verified, and what is allowed to depend on a human looking at it.

## Deterministic first **[decided]**

If a check can be a script or an assertion, it is one. An agent's attention is the most expensive
and least repeatable verification available, and it is spent on the things that genuinely need
judgment.

The ordering, cheapest and most reliable first:

1. **Type checker, linter, formatter** — run on every edit by hook, not by instruction.
2. **Unit tests** — written by whoever wrote the code. They test what the code does.
3. **Acceptance tests** — written from the requirements by an agent that has not seen the
   implementation. They test what was asked for. These are different jobs and the distinction is
   load-bearing: tests written by the author of the code encode the code's assumptions.
4. **Checklist assertions** — `docs/concerns/` entries tagged `script` or `test`, run with the
   suite.
5. **Agent judgment** — review, for what the above cannot express.
6. **Human gate** — see below.

## Visual and UI verification **[decided]**

**Assert on structure, never on pixels.** A screenshot parsed by a model is expensive,
non-deterministic, and fails to catch the things it is reached for.

Everything below is a plain assertion in the project's browser-driving harness:

| Question | Assert on |
|---|---|
| Is this element covered by that one? | Bounding box intersection |
| Is the text readable on this background? | Computed colour, contrast ratio |
| Is this the right colour, size, position? | Computed style |
| Is this reachable by keyboard? Announced correctly? | Accessibility tree, focus order |
| Did the layout break at this width? | Bounding boxes at the named breakpoints |
| Is anything overflowing its container? | Scroll width versus client width |

The generating case: a hotspot rendered beneath a persistent inventory strip, shipped and found
by a human playing the game. A bounding-box intersection assertion between every interactive
element and every piece of persistent chrome would have caught it on the first run, and costs
nothing per run thereafter. `scripts/ui-assertions.md` carries the patterns.

**Screenshots are for the human gate only** — for a person to look at, not for an agent to parse.

## The visual-check policy is per project **[decided]**

Whether automated visual checking is attempted at all is **project configuration, not an agent's
judgment call**. State it in `CLAUDE.md`:

- `visual-checks: automated` — the harness exists; assertions are expected with every UI change.
- `visual-checks: human` — no harness; UI changes go to a human gate and say what to look at.

An agent must not decide for itself that a visual check is not worth automating.

## The human gate **[decided]**

Some things need a person. The gate is a **task state**, not an interruption: a task is marked
`awaiting approval` in its feature document, so a session arriving mid-gate can see where things
stand without reconstructing it.

Rules:

- **Never mark a task done before approval**, and never batch tasks into one approval to save a
  round trip.
- **"Looks right" is not approval.** The bar is "all good, no more fixes needed" — nothing
  functional changes after that point.
- **Say what to look at.** A gate that says "please check the page" wastes the gate. Name the
  thing, and name what would count as wrong.
- **Gate only what a person can actually settle.** If the owner could answer only by re-running
  what you already ran, or by owning hardware they do not have, it is not a gate — it is a round
  trip, and the evidence you already hold is the answer. Say what remains genuinely unverified
  instead, and gate it on someone who can verify it.
- **A gate is not for approving a technical choice.** Those are the agent's (`CLAUDE.md`,
  *Who decides what*) and are reported, not submitted.
- **Every catch at a gate becomes a checklist entry before the phase closes.** This is the rule
  that makes the process improve rather than merely run. A bug found twice is a process failure.

## Test results are state on disk **[decided]**

`scripts/test-record.sh` runs the suite and records the outcome keyed by commit SHA. Anything
that wants to know what passed **reads the file**; it does not re-run the suite to find out, and
agents do not coordinate to avoid duplicate runs.

A result for a different SHA than `HEAD` is stale and says so. Stale is information, not an
error.
