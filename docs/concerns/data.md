# Data

Shape, migration, integrity, retention, deletion. The dimension where mistakes are least
reversible, because the data outlives every decision made about it.

## Universal

### Shape

- **Every field's meaning is unambiguous, and nullable means something** — `judgment`
  A nullable column where null and empty differ is a defect waiting to be interpreted two ways.
- **Constraints are in the database, not only in the application** — `script`
  Foreign keys, uniqueness, not-null, checks. The application is one of several things that
  will write to this.
- **Time is stored with its zone, and the convention is stated** — `script`
- **Money and quantities are not stored as floats** — `script`
- **Enumerated values are constrained** — `script`
  Not free text that happens to hold one of four strings.

### Migration

- **Every migration is reversible, or its irreversibility is recorded** — `judgment`
- **Migrations run against production-sized data in reasonable time** — `judgment`
  A table rewrite that locks for an hour is discovered at the worst moment.
- **A schema change is compatible with the code currently deployed** — `judgment`
  Deploy order: expand, migrate, contract. Never a change that breaks the running version.
- **A migration that transforms data is tested against realistic data** — `test`
  Including the rows that are wrong, old, or partially filled.

### Integrity

- **Concurrent writes to the same record have a defined outcome** — `test`
  Last-write-wins is an acceptable answer; having no answer is not.
- **A multi-step change is atomic, or its partial states are valid** — `test`
- **Invariants that the schema cannot express are tested** — `test`
  Cross-reference `docs/invariants.md`.

### Lifecycle

- **Deletion is defined: soft, hard, or cascading** — `judgment`
  And what happens to what referenced it.
- **Retention is stated per kind of data** — `judgment`
  Including logs, events and backups.
- **Personal data can be found and removed for one subject** — `judgment`
  Where the project is subject to that. It is far cheaper to design in than to retrofit.
- **Backups exist and restoring from one has been tried** — `judgment`
  An untested backup is a belief.

### Export and import

- **Data can be got out in a form someone else could use** — `judgment`
- **Imported data is validated as untrusted input** — `test`
  Cross-reference `security.md`.

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record retention rules, the deletion policy, and the scale numbers here on the first planning
sweep.
-->

### The format and what it promises

- **Every entity has a stable identifier, and every reference is by identifier** — `judgment`
  → `test` once a fixture puppet and a loader exist: rename and reorder it, save, reload, assert every reference still resolves. The validator rejects name-based and index-based references.
  Seeded at planning, 2026-11-26.
- **One shared validator gates load and save, and runs as a standalone command** — `judgment`
  → `test` once the validator exists. Hostile fixtures — cycle, dangling reference, duplicate identifier, NaN, unordered keypoints, binding count mismatch — are all rejected, and save refuses to write an invalid puppet.
  Seeded at planning, 2026-11-26.
- **Unmarked data the core format does not define is rejected; marked extension data round-trips untouched** — `judgment`
  → `test` once extension data is implemented. This is what makes preservation and validation coexist: without the marker a typo and an extension are indistinguishable. See `docs/requirements/puppet.md`.
  Seeded at planning, 2026-11-26.
- **Unknown fields and entities survive a load-then-save round trip** — `judgment`
  → `test`. Covers both the newer-format case and marked extension data from a writer this build does not know.
  Seeded at planning, 2026-11-26.
- **Every released format version has a committed fixture the current build loads; fixtures are append-only** — `judgment`
  → `script` once there is a released version. Note the compatibility promise does not harden until the project is shared with a community.
  Seeded at planning, 2026-11-26.

### Writing to disk

<!-- Both applications write puppet files: Live writes channel mappings. These entries bind Live
     as much as Studio. -->

- **Every write to a puppet, scene or autosave path goes through one atomic-write function** — `judgment`
  → `script` once the function exists, failing on any write outside it. A paired test injects failure at each step and asserts the prior file is intact.
  Seeded at planning, 2026-11-26.
- **Saving an unmodified puppet is byte-identical, with no timestamps, paths or machine names** — `judgment`
  → `test`. Preserved extension data takes part in canonical serialisation, or carrying it makes files churn.
  Seeded at planning, 2026-11-26.

### Editing without losing work

- **Adding artwork preserves configuration** — `judgment`
  → `test`: add images, a PSD and a merged puppet to a configured fixture, then assert every pre-existing identifier, binding, keypoint value and deformation is unchanged. Extend to the changed-dimensions and regenerated-mesh cases once those are decided.
  Seeded at planning, 2026-11-26.
- **A random sequence of edits followed by undo-all returns a serialisation equal to the original** — `judgment`
  → `test`, as a property test over the operation set, including compound operations and import.
  Seeded at planning, 2026-11-26.
- **A file written by an independent generator, using no project code, validates and loads** — `judgment`
  → `test`. The reference puppet generator satisfies this; it is the machine-checkable form of "third parties can write the format".
  Seeded at planning, 2026-11-26.
