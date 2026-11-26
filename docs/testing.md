# Testing

How each kind of thing is tested here.

`.claude/rules/testing.md` carries the rules that apply to every project — the deterministic-
first ordering, the visual-check policy, the human gate. **This document carries what is
specific to this one:** the harness, the idioms, and what is deliberately not tested.

## Harness **[open]**

| Kind | Tool | Lives in |
|---|---|---|
| Unit | | |
| Integration | | |
| Browser / end-to-end | | |

## Who writes what **[decided]**

- **Unit tests** — whoever wrote the code. They test what the code does.
- **Acceptance tests** — `acceptance-planner`, from the requirements, **without sight of the
  implementation**. They test what was asked for.

These are different jobs. Tests written by the author of the code encode the code's
assumptions, which is why the second kind is written first and never edited to make an
implementation pass. An acceptance test that seems wrong is a finding, not an obstacle.

## Visual verification **[open]**

**`visual-checks:`** `automated` | `human` — state which, and state it in `CLAUDE.md` too.

Where automated: the harness, the helpers, and where the assertion patterns live. See
`scripts/ui-assertions.md` for the forms. Assert on bounding boxes, computed styles and the
accessibility tree; never on pixels.

## Not tested, and why **[open]**

<Each with its reason. A gap on record is a decision; an unmentioned gap is a surprise.>
