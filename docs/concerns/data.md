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
