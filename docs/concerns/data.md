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
  → `test` once the validator exists. Hostile fixtures — cycle, dangling reference, duplicate identifier, NaN, unordered keypoints, an *unmarked* binding count mismatch — are all rejected, and save refuses to write an invalid puppet. A mismatch **marked stale** is legal and must load, render and save; that is the paired assertion, and conflating the two would reject a file the format defines as valid.
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
  → `test`: add images, a PSD and a merged puppet to a configured fixture, then assert every pre-existing identifier, binding, keypoint value and deformation is unchanged. Also the two cases `docs/requirements/puppet.md` settles: replacement art at different dimensions is remapped, and a regenerated mesh leaves its deformations retained and marked stale. Neither may discard anything.
  Seeded at planning, 2026-11-26.
- **A random sequence of edits followed by undo-all returns a serialisation equal to the original** — `judgment`
  → `test`, as a property test over the operation set, including compound operations and import.
  Seeded at planning, 2026-11-26.
- **A file written by an independent generator, using no project code, validates and loads** — `judgment`
  → `test`. The reference puppet generator satisfies this; it is the machine-checkable form of "third parties can write the format".
  Seeded at planning, 2026-11-26.

### Texture lifetime, copy, paste and symmetry

- **An operation that copies or merges entities mints a new identifier for every one of them, and remaps references internal to the copied set** — `judgment`
  → `test` once paste exists: copy a subtree containing a mask between two of its own parts, paste, assert the pasted mask names the pasted part and not the original. Assert no issued identifier equals any live or retired one. Paste, merge, parameter duplication and the re-import split all take this path.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **A field with a default has exactly one stored form for that default** — `judgment`
  → `test`: texture flip, axis polarity and the mirror axes each serialise one way when unset — absent or explicit, never both. A default that serialises two ways makes the same puppet save to different bytes, which is the canonical-serialisation promise.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **An operation that mirrors or reverses data returns the original bytes when applied twice, and bounds the set it modifies** — `judgment`
  → `test`: mirror a rigged node twice and assert byte equality with the original. Assert every binding outside the selection evaluates identically, which is the assertable form of *a mirror never writes a parameter*.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

### Physics as a node, and tracks

- **Physics state is never serialised, and a load or a replacement starts from rest** — `judgment`
  → `test`: save mid-simulation and assert the bytes equal a save at rest; load the same puppet twice and assert an identical first frame. A load that fails must leave no simulation state behind ([I4](../invariants.md)).
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A physics node whose output drives a binding on one of its own ancestors validates, loads, evaluates deterministically and saves** — `judgment`
  → `test`, paired with a rejection case as the tree-acyclicity criteria are. The dataflow cycle is deliberate and broken in time by the one-frame read; a validator that rejects it rejects a valid file, which is the failure this entry exists to catch.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **No operation that copies, pastes or mirrors a physics node leaves a parameter with two base drivers** — `judgment`
  → `test`, table-driven over the operation set, including a paste between puppets with overlapping identifier spaces. A refusal, never a repair.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A deletion names every parameter it leaves driverless and every physics node it leaves driving nothing** — `judgment`
  → `test` on the structured result. Deleting a parameter now leaves its physics node standing as a chain link, which is a state no edit could previously produce.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **Deleting a parameter removes its track in every animation, names each one by animation, and undo restores them byte-identically** — `judgment`
  → `test`. A track is authored keyframes, so a report saying only that the animation survives hides lost work.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **Every node kind carries a containment declaration and a constitutive-or-incidental declaration for each reference it holds; a kind missing either fails the check** — `judgment`
  → `script` over the format definition. This is what keeps *a physics node contains only physics* a row in a table rather than a special case, and what makes the next kind arrive with a stated answer.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **The derived mirror axes are computed from drawn geometry alone** — `judgment`
  → `test`: change a physics node's local offset and assert both derived axes are unchanged. Physics has an offset and draws nothing, so it is the first kind that could wrongly move them.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
