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

### The frame loop

- **Every per-frame counter has a committed baseline ceiling** — `judgment`
  → `script` once the runtime exposes headless counters, failing on any counter above baseline unless the baseline changes in the same commit with a reason.
  Seeded at planning, 2026-11-26.
- **Per-frame counters scale no worse than linearly from the reference puppet to the stress puppet** — `judgment`
  → `test`. Catches accidental quadratic behaviour in deformation composition and physics.
  Seeded at planning, 2026-11-26.
- **Nothing in the frame loop resolves by name or scans linearly** — `judgment`
  → `test`: the name-resolution counter is zero, and animation lookup is independent of animation length.
  Seeded at planning, 2026-11-26.
- **No first-use work after the first presented frame** — `judgment`
  → `test`: resource creations and pipeline preparation after frame one are zero, for every blend, mask and composite combination the puppet uses.
  Seeded at planning, 2026-11-26.
- **A soak run at the stated stream length leaves counters and resident bytes flat** — `judgment`
  → `test`. The length is four hours, set in `docs/requirements/platform.md`.
  Seeded at planning, 2026-11-26.

### Memory and loading

- **Parts sharing one texture leave resident texture bytes and upload count unchanged** — `judgment`
  → `test`. Asserts the `[decided]` texture-reuse requirement in `puppet.md`.
  Seeded at planning, 2026-11-26.
- **Startup and load budgets hold with every derived cache empty** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **Import peak memory is independent of PSD layer count** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.

### Input and editing

- **The input path is latest-wins and bounded, and its latency in frames is asserted** — `judgment`
  → `test` with an injected clock, including a flood case and a many-distinct-channel-names case.
  Seeded at planning, 2026-11-26.
- **Edit cost, undo step size and bytes written per save are independent of total puppet size** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.

### Texture lifetime, copy, paste and symmetry

- **An operation that only re-references artwork creates no texture, uploads nothing, and retains no pixels in its undo step** — `judgment`
  → `test`: copy, each paste mode, mirrored paste. Assert textures created, bytes uploaded, payload bytes written and pixel bytes retained for undo are all zero. A build that copies pixels per paste silently defeats the sharing these operations exist for.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **A texture nothing draws is neither decoded, uploaded, nor counted against resident texture memory** — `judgment`
  → `test`: load a puppet carrying an unreferenced texture of declared size; assert it round-trips byte-identically and appears in no upload count or residency figure. Otherwise a file carrying unused artwork inflates the player's memory budget with art nobody can see.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.
