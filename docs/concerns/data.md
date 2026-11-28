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
- **A rule the specification states for the canonical writer is not enforced by the reader, and each such rule has a valid-but-non-canonical fixture the reader accepts and the writer canonicalises** — `test`
  Table-driven over the canonical-serialisation list: unsorted entries, other modification times, the other compression method per entry kind, an extra field, other external attributes, a directory marker. Project-specific because the whole fixture corpus is written by our own canonical writer, so every fixture satisfies every writer rule and no test can see the difference. *Would have caught:* a puppet repacked with ordinary tooling being refused. It also covers the sibling rules nobody had tested.
  Harvested from the S2 third-party gate, 2026-11-28.
- **A tolerance added for third-party tooling is tested against the output of more than one such tool** — `test`
  *Would have caught:* the first directory-marker fix, validated only against the tool that produced the original report and broken for another. One sample is not a boundary.
  Harvested at the S2 review, 2026-11-28.
- **A fixture exempted from a corpus-wide assertion is exempt from the named assertion only, and must still satisfy the rest** — `test`
  *Would have caught:* an exemption written per fixture rather than per assertion, under which an unrelated stray entry in that fixture would pass unnoticed.
  Harvested at the S2 review, 2026-11-28.
- **Preserved unknown material follows its entity through in-memory edits, including entities addressed by position** — `test`
  Material from a later-version file that the writer keys by position within a collection, after a keypoint, keyframe or axis is inserted or deleted. *Would have caught:* a stranger's data silently re-attaching to a different entity.
  Harvested at the S2 review, 2026-11-28.
- **A value stored by position survives its neighbours being inserted, deleted or split** — `test`
  A segment's interpolation mode lives on the earlier keyframe, and keyframes have no identifier. *Catches:* a mode silently moving to the next segment after an insert. Extends the preserved-material entry above from material a build does not understand to material it does.
  Seeded at the animation planning sweep, 2026-11-28.
- **Every enumerated field declares what a build that does not know its value does, and carries a fixture of one** — `test`
  The unfamiliar-data entries above cover the fields the format already lists; this one binds the next field added. *Catches:* an interpolation mode, end behaviour or trigger event added with no answer for a stranger's build — which is the gap the animation draft had until it was swept.
  Seeded at the animation planning sweep, 2026-11-28.
- **Contributions to one value are summed in an order fixed by data, never by runtime state** — `test`
  Permute the file's collection order and the runtime start order over three or more contributions whose float sum differs by grouping, and assert bit-identical output. *Catches:* the belief that summation being commutative makes order irrelevant. It is commutative and **not associative**, so three contributions depend on grouping — and `evaluation.md` calls three bindings on one node ordinary rigging. Applies equally to any fixed iteration count.
  Seeded at the animation planning sweep, 2026-11-28.
- **Presentation data is stored as a field, never as a collection's order, and every save that does not own it leaves it untouched** — `test`
  Covers a parameter's stored order and its `mappable` flag. *Catches:* a canonical sort by identifier erasing a rigger's arrangement, and Live's save path dropping what Studio set.
  Seeded at the animation planning sweep, 2026-11-28.
- **A change to how stored data composes states what previously saved files now do** — `judgment`
  Becomes a `test` once an evaluator exists. *Catches:* a semantics change with no data migration to prompt anyone — nothing in the file differs, so nothing flags it. Currently cheap because nothing has shipped; the entry exists for when that stops being true.
  Seeded at the animation planning sweep, 2026-11-28.
- **A canonical-writer rule is tested against a puppet built in memory, never only one decoded from a fixture** — `test`
  *Catches:* a rule the decoder happens to satisfy, so a fixture-fed test passes without the writer being exercised at all. Triggers were the generating case: the writer never sorted them, the reader did, and the test could not see it.
  Harvested at the S2b review, 2026-11-28.
- **A preserved member's path may name a position in a collection, and any operation that reorders that collection must repath its preserved members in the same step** — `script` for the format definition, `test` for the edit
  A collection whose order can change without the writer repathing is the defect. *Catches:* a stranger's preserved data re-attaching to a different entity when a collection is reordered or an item inserted. An animation's triggers are the first collection whose canonical order, by event name, differs from the order it was read in, which is what made this concrete: their preserved material is addressed as `triggers/<n>` and the writer repaths it when it sorts. Keying the path by event name instead was rejected — it would add a second addressing scheme for one collection, and it keys on a value whose uniqueness is a validation rule while decode runs first, so a duplicate-event file would collide two preserved rows during a decode that is about to refuse anyway.
  **The `test` half is covered** — `a_triggers_preserved_material_follows_its_event_through_the_writers_sort` reorders a trigger in memory, saves, and re-resolves the material by **event name** rather than by the seat it arrived at. Asserting it by path instead proves nothing, because a fixture whose triggers already arrive canonical makes the sort the identity. **The `script` half does not exist**: nothing walks the format definition for a positional path in a collection ordered by value, so a collection added later with the same shape arrives unchecked.
  Harvested at the S2b review, 2026-11-28.
- **Every collection with a stored canonical order is sorted by the writer** — `script`
  *Catches:* a collection added whose ordering is done in the reader, so a writer that is not the reader produces non-canonical files.
  **No script implements this yet.** Nothing in `scripts/` or `tests/` does the work. Tagged `script` for what it is meant to be, not for what exists.
  Harvested at the S2b review, 2026-11-28.

### Writing to disk

<!-- Both applications write puppet files: Live writes channel mappings. These entries bind Live
     as much as Studio. -->

- **Every write to a puppet, scene or autosave path goes through one atomic-write function** — `script`
  A grep failing on any write outside the function, with an allow-list for the single write path and the fixture builder. A paired test injects failure at each step and asserts the prior file is intact.
  Seeded at planning, 2026-11-26. Function exists as of S2; script now required.
- **A failed or refused save leaves the destination directory listing unchanged and never replaces an existing sibling** — `test`
  Run for each injected failure step, for upgrading and non-upgrading saves, and with a backup sibling already present. *Would have caught:* an upgrading save overwriting an existing version-upgrade backup, the one file kept precisely because an upgrade cannot be undone by saving again.
  Harvested at the S2 review, 2026-11-28.
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
- **A new reference kind declares whether it is constitutive or incidental, and the deletion report names every record a cascade removes** — `script` over the format definition
  Extends the declaration check above from node kinds to every reference kind. *Catches:* a reference kind added with no cascade rule. Note a trigger is **not** an instance of this: it lives on the animation it starts, carries no target, and goes with that animation.
  Seeded at the animation planning sweep, 2026-11-28.
- **The derived mirror axes are computed from drawn geometry alone** — `judgment`
  → `test`: change a physics node's local offset and assert both derived axes are unchanged. Physics has an offset and draws nothing, so it is the first kind that could wrongly move them.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
