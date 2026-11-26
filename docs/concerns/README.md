# Concern checklists

Cross-cutting concerns, one file per dimension. These are the project's accumulated knowledge
about what gets forgotten.

## Why these exist

A developer building a form does not remember that it needs analytics. An agent told to
"consider the UX implications" catches a different random subset every run. A checklist entry
catches its item every run, at no marginal cost.

The generating case: a hotspot rendered beneath a persistent inventory strip, shipped and found
by a person playing the game. It was trivially catchable — a bounding-box intersection
assertion — and was missed because nobody had written down that overlap is a thing you check.

## How they are used

Each file has two readers, which is why each entry must serve both:

| When | Who | What happens |
|---|---|---|
| **Planning** | `concern-analyst`, planning mode | Entries become acceptance criteria in the requirements, and in the implementer's assignment. A specification, not a later catch |
| **Review** | `concern-analyst`, review mode | Entries are verified against what was built |

Dispatch one analyst per dimension; they run in parallel and each reads one file.

## Structure of a file

Two sections:

- **Universal** — true of most projects. Shipped with the workflow. Changes rarely, and a change
  belongs upstream in the suite rather than here.
- **This project** — specific to this codebase. Generated at planning time by `concern-analyst`,
  **approved by the user**, and grown by harvest at the end of every cycle.

## The three rules

**1. Every entry names how it is verified.**

| Tag | Meaning |
|---|---|
| `script` | A check in `scripts/`, run in CI or by `docs-check.sh` |
| `test` | An assertion in the test suite |
| `judgment` | Someone has to think about it |

An entry with no verification path is an opinion. **Prefer `script` and `test`.** The proportion
of `judgment` entries measures how much of this process still depends on an agent having a good
day; it should fall over the life of the project. An entry tagged `script` or `test` whose check
does not exist yet is worse than no entry, because it reads as covered — write the check or tag
it `judgment` until you do.

**2. Every *This project* entry carries provenance.**

Where it came from: seeded at planning on a date, or caught at a gate on a date, on which
feature. Without provenance you can never prune, because you cannot tell which entries were
speculative and which were paid for in a real bug.

**3. The list has a budget.**

A checklist that only grows stops being read, which fails exactly like having none. At the end
of every cycle, `finish-cycle` proposes deleting *This project* entries that have never fired
and have no incident behind them.

## Adding entries

**Agents propose; the user approves.** `concern-analyst` returns proposed entries and does not
write them. These become binding checks, and a list generated and self-approved by an agent is a
list nobody believes.

The one path that does not wait for a planning sweep: **anything a human catches at a gate
becomes an entry before the phase closes**, harvested by `finish-cycle`. That is the mechanism by
which this process gets better rather than merely organised. A bug found twice is a process
failure.

## Entry format

```markdown
- **<what must be true>** — `script|test|judgment`
  <How it is checked, or what to look at. One line.>
  <Provenance, for This project entries only.>
```

## The dimensions

| File | Covers |
|---|---|
| [ux.md](ux.md) | Interaction, layout, states, the user's journey through a flow |
| [accessibility.md](accessibility.md) | Keyboard, screen reader, contrast, motion, target size |
| [analytics.md](analytics.md) | What is measured, and whether a question can be answered later |
| [security.md](security.md) | Authentication, authorisation, input, secrets, exposure |
| [performance.md](performance.md) | Cost at scale, in queries, payload, and perceived latency |
| [data.md](data.md) | Shape, migration, integrity, retention, deletion |
| [ops.md](ops.md) | Deployment, configuration, failure, observability, rollback |

Add a dimension when something is repeatedly forgotten and fits none of these. Adding one means
adding it here and to this table.
