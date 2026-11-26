# Performance

Cost at scale, in queries, payload and perceived latency. Mostly a question of what happens when
the data is a thousand times larger than it is in development.

## Universal

### Queries

- **No query inside a loop** — `test`
  The N+1. Assert on query count for the endpoint, not on wall-clock time — a count assertion is
  deterministic and a timing assertion is flaky.
- **Every query a user can trigger hits an index** — `script`
  Explain-plan check for the paths that matter.
- **Every list is paginated, bounded, or provably small** — `test`
  "Provably small" means a constraint enforces it, not that it is small today.
- **Queries are bounded by a timeout** — `judgment`

### Volume

- **Behaviour at realistic scale is known** — `judgment`
  Name the number: how many rows, how many concurrent users, how large a file. An unstated
  assumption about scale is the commonest source of a surprise in production.
- **Nothing unbounded is held in memory** — `judgment`
  Reading a whole table, a whole file, or a whole upload.
- **Background work is idempotent and retryable** — `test`

### Payload and perceived speed

- **Page or response weight is budgeted** — `script`
  A number, checked in CI. Weight only ever grows without one.
- **Images are sized and served at the dimensions used** — `script`
- **Anything slow reports progress** — `test`
  Cross-reference `ux.md`. Perceived latency is mostly a UX problem.
- **The common path does not wait on the uncommon one** — `judgment`

### Caching

- **What is cached, for how long, and what invalidates it, is written down** — `judgment`
  A cache with no stated invalidation is a correctness bug with a delay on it.
- **A cache miss is not catastrophically slower than no cache** — `judgment`

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the realistic scale numbers and the budgets here on the first planning sweep — the
Universal entries reference them.
-->
