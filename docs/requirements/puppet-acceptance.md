# Acceptance criteria — the puppet

The criteria that decide whether what gets built satisfies [puppet.md](puppet.md). Written from
that document alone, before any implementation existed and without sight of
[format.md](../format.md) — which is the point, and
[testing.md](../testing.md#who-writes-what-decided) says why: a criterion derived from the design
can only establish that the design was implemented, so a build that satisfies the design and
misses the requirement would pass.

## How to read this **[decided]**

**These are criteria, not tests.** Nothing is built — [plan.md](../wip/plan.md) places the
workspace and the format crate ahead of any of this — so none of it has been run, and no
criterion here has been confirmed failing. Each is written to the point where the test that
implements it is obvious: a fixture, an operation on it, and an assertion. A later pass turns
them into `tests/acceptance/`, the location [testing.md](../testing.md#harness-proposed) gives,
where they run in the default `cargo test --workspace`. **None of them belongs in the soak** —
that job is four hours of simulated stream, is ignored by default, and answers a question about
the runtime over time rather than about what a puppet file guarantees.

Each criterion carries a `How` — the tags are [concerns/](../concerns/README.md)'s: `test` is an
assertion in the suite, `script` a check in `scripts/`, `judgment` an agent reading code or
specification at review, `gate` a person looking at something. Where it is `gate`, what the
person looks at and what would count as wrong is stated with it; a gate that says "check it"
wastes the gate.

**A criterion is never firmer than the requirement it traces to.** Each section below carries the
status tag of the section of [puppet.md](puppet.md) it is derived from, and links to it. Where a
requirement could not be made checkable, the row says so and the reason is in
[Criteria not written](#criteria-not-written-open) rather than being filled with a guess.

Identifiers are stable. A criterion that is retired keeps its identifier and is marked retired,
because assignments and review reports cite them.

## The fixtures **[proposed]**

Described by what they contain, never by how they are encoded — the encoding is
[format.md](../format.md)'s and these criteria must not depend on it.

| Fixture | What it carries |
|---|---|
| `rigged` | The base valid puppet. A root composite; a mesh group with two part children; a third part at root referencing the same **texture** as one of those two; one part acting as a mask for another; three textures of distinct dimensions, one of them content surrounded by transparent padding of known extent; four **parameters** — two one-dimensional, one two-dimensional, one with a single keypoint — each declaring its axis range and carrying exactly one neutral keypoint; against one node both a transform **binding** and a **deformation** binding; a **chain deformer** on one part; **physics** driving one parameter; an **animation** driving two; a **channel mapping** from one named channel to one parameter, with a response curve; **import provenance** on every part; **extension data** in a namespace no build understands, at two places in the tree |
| `reference`, `stress` | The two generated workloads, at the dimensions in [platform.md](platform.md#the-reference-puppet-and-the-stress-puppet-proposed), produced by the committed generator **using no project code**. The same artifact is the conformance fixture for [TOOL-1](#c-the-file-is-readable-and-writable-by-tools-that-are-not-ours-decided) |
| `rigged:future-minor` | `rigged` declaring a format version **later** than this build knows, carrying an unmarked core field, an unknown node kind, and an unfamiliar value in a known enumerated field. The version declaration is what licenses all three. Variants named `rigged:future-minor-…` declare the same later version |
| `one-part` | The smallest valid puppet: one root part, one texture, nothing else. The beginner's first import, and the fixture on which a cascade meets the one-root rule |
| `rigged:deep` | `rigged` with a **mesh group** nested inside the first, carrying a part of its own which carries a binding. The fixture for a cascade that runs down a subtree |
| `rigged:<defect>` | `rigged` with exactly one change, named per criterion. One defect per fixture, so a rejection cannot be attributed to the wrong rule |

**Every fixture declares the current format version — the one this build knows — unless its name
says otherwise.** That is part of the fixture, not incidental to it: the validator's rules on
unmarked data and on unfamiliar enumerated values are gated on the declared version, so the same
material in a file declaring a later version is **accepted**. A defect fixture that declared a
later version would therefore pass, and the criterion would silently stop testing anything.

Defect fixtures are listed where they are used. Every fixture in this document is committed, and
[LIM-1](#h-load-limits-proposed) makes a declared load limit with no fixture a failure.

## A. Every entity has a stable identity **[proposed]**

Traces to [Every entity has a stable identity](puppet.md#every-entity-has-a-stable-identity-proposed)
and asserts [I3](../invariants.md#i3-every-reference-in-a-stored-puppet-or-scene-is-by-stable-identifier).

| ID | Operation and assertion | How |
|---|---|---|
| **ID-1** | Load `rigged`, record the **identifier** of the puppet and of every piece [What a puppet is](puppet.md#what-a-puppet-is-proposed) lists — enumerated from that list rather than from a list kept here, so that a piece added there is covered without an edit. Save, reload. Assert the set recovered is exactly the set recorded, member for member | `test` |
| **ID-2** | Rename every named entity in `rigged` to a fresh, distinct name. Save, reload. Assert every identifier is unchanged and every reference resolves to the same entity it did before | `test` |
| **ID-3** | Reverse sibling order at every level of the tree, and the order of the parameter, texture, binding and keypoint lists. Save, reload. Assert every identifier unchanged, every reference resolving to the same entity, and draw order following the tree's new order — the one thing order is allowed to decide | `test` |
| **ID-4** | Delete a part and a parameter, save; create a new part and a new parameter, save; reload and repeat. Assert no newly issued identifier equals one previously deleted, within one run of the editor and across the reload | `test` |
| **ID-5** | `rigged:dup-id-same-kind` (two parameters sharing an identifier) and `rigged:dup-id-cross-kind` (a node and a texture sharing one) are both rejected by the validator | `test` |
| **ID-6** | Record `rigged`'s own identifier; add a part, rename the puppet, save. Assert the puppet's identifier is unchanged — this is what a **scene** confirms the file against, and what must survive the puppet being edited | `test` |
| **ID-7** | No stored reference resolves through a name or through a position. Read at review against the format specification, because a file cannot express a reference the format does not define — so there is nothing for a fixture to carry. [ID-2](#a-every-entity-has-a-stable-identity-proposed) and ID-3 are the behavioural half | `judgment` |
| **ID-8** | Every piece named in [What a puppet is](puppet.md#what-a-puppet-is-proposed) is present and equal after load, save, load — **masks**, **chain deformers** and their joints, **physics** and **channel mappings** as much as the node tree, textures, parameters, bindings and animations. The catch-all that fails when a kind is forgotten entirely. A mask is a reference rather than a free-standing entity, so it is asserted here as a reference that still resolves | `test` |
| **ID-9** | The puppet's identifier identifies the puppet, not the file. Copy the file; assert the copy's identifier is identical to the original's and that a scene reference to either resolves. Then **save as** a new file; assert that one carries a **new** identifier. Two files sharing an identifier is normal; the deliberate act of making a different puppet is what issues a new one | `test` |

Identifier reuse by a *third-party* writer is not detectable by any criterion here — see
[Criteria not written](#criteria-not-written-open).

## B. A texture is not a part **[decided]**

Traces to [A texture is not a part](puppet.md#a-texture-is-not-a-part-decided) and
[How textures are stored is not how they reach the GPU](puppet.md#how-textures-are-stored-is-not-how-they-reach-the-gpu-proposed).

| ID | Operation and assertion | How |
|---|---|---|
| **TEX-1** | Load `rigged`, where two parts reference one texture. Assert one texture entity, one stored payload, one upload, and resident texture bytes equal to the distinct textures' footprints. Compare against `rigged:dup-texture`, a file in which the second part references a separate texture entity holding identical artwork — writable by a third party even though an import would not produce it: assert `rigged`'s resident bytes and upload count are strictly lower | `test` |
| **TEX-2** | The saved bytes are independent of any GPU-layout decision. Save `rigged` against the noop device and against a real one; assert the files are byte-identical | `test` |
| **TEX-3** | Load `reference`. Assert resident texture bytes do not exceed the source artwork **as delivered**, at the declared 25 Mpx — not the cropped form, against which the comparison would be nearly tautological. Cropping is how the guarantee is met, not what it is measured against | `test` |
| **TEX-4** | `rigged` carries a texture whose content occupies a known sub-rectangle of a padded image. Assert the stored payload and the resident footprint are the content bounding box, **and that the part's evaluated geometry places the artwork where it was before cropping** — a crop that moves the art is the failure this half catches | `test` |
| **TEX-5** | The saved file holds exactly one payload per distinct texture entity and nothing else that decodes as an image. Together with [FILE-4](#l-one-file-decided) this is how "the layered source is not embedded" is asserted | `test` |
| **TEX-6** | No packing decision is visible to the person rigging: no atlas size, no dimension cap, no duplication presented as a choice. Read at review and at Studio's interface gate when there is one | `judgment` |
| **TEX-7** | **Deduplication is at import.** Import the same artwork twice, for two different parts. Assert one texture entity results, both parts reference it, and the saved file holds one payload. Importing a second image whose pixels differ by one produces a second entity — a content hash, not a filename or a size | `test` |

## C. The file is readable and writable by tools that are not ours **[decided]**

Traces to [The file is readable and writable by tools that are not ours](puppet.md#the-file-is-readable-and-writable-by-tools-that-are-not-ours-decided).

| ID | Operation and assertion | How |
|---|---|---|
| **TOOL-1** | `reference` and `stress`, written by the committed generator with no project code, validate and load, and the loaded puppet reproduces every dimension declared for that workload | `test` |
| **TOOL-2** | A person, given only the format specification and ordinary command-line tooling, inspects `rigged`, changes one parameter's name and one keypoint value, and writes it back. Assert the build loads the result, the two changes are present, and the rest is identical to a build-made save of the same two changes. **Wrong looks like:** needing a Rigel2D tool at any step, having to read the whole file to reach one texture's payload, or the build rejecting the hand-made file | `gate` |
| **TOOL-3** | Within TOOL-2, one texture's payload is extracted and decoded on its own, without the structure of the rest of the file being parsed. **Wrong looks like:** the payload only being reachable by walking the file from the start | `gate` |
| **TOOL-4** | `rigged:future-minor` loads — its unmarked core field, unknown node kind and unfamiliar enumerated value notwithstanding — and every entity the build does know is present and correct. The preservation half is [VER-1](#d-a-build-works-with-what-it-understands-and-preserves-the-rest-decided) | `test` |

## D. A build works with what it understands and preserves the rest **[decided]**

Traces to [A build works with what it understands and preserves the rest](puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest-decided).

| ID | Operation and assertion | How |
|---|---|---|
| **VER-1** | Load `rigged:future-minor`, edit a value this build understands, save. Assert the **contents** of the unfamiliar material — unmarked as much as marked — are byte-identical to what arrived, that the edit is present, and that the file remains editable throughout. Where that material *sits* is [EXT-4](#e-unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided), and the two are different questions | `test` |
| **VER-2** | `rigged:breaking-version`, declaring a break in compatibility, is refused with an error naming the version. Assert the file's bytes and modification time are unchanged, and that the puppet already loaded, if any, still renders unchanged — [I4](../invariants.md#i4-a-load-that-fails-never-replaces-what-is-already-displayed) | `test` |
| **VER-3** | Opening never writes. Load `rigged`, `rigged:future-minor` and every defect fixture; assert bytes and modification time unchanged after each, and after a run of the editor that saves nothing | `test` |
| **VER-4** | `rigged:old-version` declares an earlier version this build upgrades. Save. Assert a backup exists holding the pre-upgrade bytes exactly, and that the saved file declares the current version. Byte-identity with the pre-upgrade file necessarily breaks here, and that is not a canonical-serialisation failure — [CANON-1](#j-the-same-puppet-saves-to-the-same-bytes-proposed) is scoped to one build | `test` |
| **VER-5** | Inject a failure at each step of the upgrading save of `rigged:old-version`, including the backup write. Assert the original file is intact after every one — [I7](../invariants.md#i7-a-failed-save-never-damages-the-previous-file) | `test` |
| **VER-6** | Load, save, load `rigged:future-minor`. Assert the two serialisations are byte-identical, so that carrying someone else's data does not make the file churn | `test` |

## E. Unrecognised data is marked as such by whoever wrote it **[decided]**

Traces to [Unrecognised data is marked as such by whoever wrote it](puppet.md#unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided)
and asserts [I1](../invariants.md#i1-rigel2d-never-executes-anything-a-puppet-scene-or-source-supplies)
and [I6](../invariants.md#i6-a-plugin-never-arrives-inside-a-puppet-a-scene-or-a-source).

| ID | Operation and assertion | How |
|---|---|---|
| **EXT-1** | `rigged`'s extension data declares an extension identifier, the version it expects, and a human-readable name. The validator accepts it and the loader preserves it | `test` |
| **EXT-2** | `rigged:ext-unmarked` carries the same values with the marker removed, **declaring the current version**. Assert it is **rejected as malformed**, not preserved — without this a typo and an extension are the same thing. The declared version is what makes this a defect rather than a licensed unknown | `test` |
| **EXT-3** | `rigged:ext-no-version` and `rigged:ext-no-label` are each rejected | `test` |
| **EXT-4** | **Contents verbatim, position canonical** — the two halves that look like a contradiction and are not. Present the same extension data at a different position in the input; assert the saved bytes are identical to `rigged`'s, so position is set by the canonical ordering, keyed on the marker — an extension's namespace identifier, or for unknown core material its field identifier — which a build can read without understanding the contents. Assert the contents themselves are byte-identical to what arrived | `test` |
| **EXT-5** | `rigged:ext-hostile` carries, inside extension data and inside every string-bearing core field, values shaped like an absolute path, a traversal path, a URL, a shell command and a format string. Assert each is preserved verbatim through load and save, and that the load performs no process spawn, no dynamic library load, no network connection, and no filesystem access outside the puppet's own container — asserted by a harness that fails on the call, not by inspection | `test` |
| **EXT-6** | `rigged:ext-declares-plugin` declares that it carries data for a named extension at a named version. Assert nothing is installed, registered or loaded, and that the declaration is reported to the user as missing | `test` |
| **EXT-7** | `rigged:future-minor`'s unmarked material — the core field, the node kind, the unfamiliar enumerated value — is **accepted and preserved opaquely**, exactly as marked extension data is. Assert the load succeeds, that none of it is interpreted (removing it from the fixture changes no evaluated geometry, no draw order and no reported entity), and that it survives load, save, load byte-stably | `test` |
| **EXT-8** | The gate is the **declared version**, not recognisability. Present byte-for-byte the same unmarked material in `rigged:ext-unmarked` and in `rigged:future-minor`. Assert the first is rejected and the second accepted. One differential assertion carries the whole reconciliation, and it is the one that fails if a build gates on anything else | `test` |
| **EXT-9** | The finite-number rule is **not** gated by the declared version, in two halves. `rigged:future-minor-nonfinite` carries, inside its opaquely preserved material, bytes that would read as a non-finite number: assert it is accepted, never examined, and preserved byte-stably. `rigged:future-minor-nan-interpreted` puts a non-finite value in a **known numeric field** of a later-version file: assert it is **rejected**, because the value is one this build interprets. The second half is the one an implementer gets wrong, by reading "later version" as "accept the file", and it is the difference between rejecting a NaN and letting one reach the frame loop | `test` |

## F. What the user is told **[decided]**

Traces to [What the user is told](puppet.md#what-the-user-is-told-decided). Asserted on the load
outcome the runtime reports, **not on any interface** — neither application's interface is
designed, and [I2](../invariants.md#i2-nothing-but-rendered-puppet-content-ever-reaches-the-render-surface)
binds where it may surface in Live.

| ID | Operation and assertion | How |
|---|---|---|
| **MSG-1** | Load `rigged:future-minor` and `rigged:ext-unknown`. Assert the reported load outcome names the version gap, names each unrecognised extension by its human-readable name and declared version, and carries the warning that the features behind them may not work in this build | `test` |
| **MSG-2** | A person reads the message for both fixtures. **Wrong looks like:** a promise that the file will be preserved intact, an internal extension identifier shown where the human-readable name belongs, or a message that reads as an error for a file that opened fine | `gate` |

## G. A puppet that fails its integrity rules is rejected, not repaired **[proposed]**

Traces to [A puppet that fails its integrity rules is rejected, not repaired](puppet.md#a-puppet-that-fails-its-integrity-rules-is-rejected-not-repaired-proposed).

Each fixture is `rigged` with exactly one defect, **declaring the current format version**, and
each is **rejected**. The declaration is load-bearing for VAL-7 and VAL-8, whose rules are gated
on it; the licensed counterpart is [EXT-7](#e-unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided).

| ID | Fixture | The rule it breaks |
|---|---|---|
| **VAL-1** | `:cycle` | The tree is acyclic |
| **VAL-2** | `:no-root`, `:two-roots` | One root |
| **VAL-3** | `:dangling-node-ref`, `:dangling-texture-ref`, `:dangling-param-ref` | Every reference resolves |
| **VAL-4** | `:keypoints-unordered`, `:keypoint-outside-declared-range` | Keypoints are ordered and within the range **the parameter declares**, which is why a parameter declares one |
| **VAL-5** | `:binding-count-mismatch-unmarked`, `:binding-count-match-marked-stale` | The stale mark agrees with the value counts. **Both disagreements are invalid**, because the derived state is authoritative and the mark must agree with it — a file from a stranger can lie, a derivation cannot. A criterion that checks only the unmarked mismatch accepts a file claiming staleness that is not there |
| **VAL-6** | `:nan`, `:positive-infinity`, `:negative-infinity` | Every number is finite — rejected, never clamped, in every value this build interprets as a number, **in a file of any declared version**. This rule does not carry the version gate the two rows below do. What it does not reach is opaquely preserved material, which is [EXT-9](#e-unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided) |
| **VAL-7** | `:unknown-enum-unmarked` | Enumerated values are known, or marked as extensions — in a file whose declared version this build knows |
| **VAL-8** | `:unmarked-core-data` | Unmarked data the core format does not define is invalid — in a file whose declared version this build knows |
| **VAL-15** | `:no-neutral-keypoint`, `:two-neutral-keypoints` | Every parameter has exactly one neutral keypoint. Both none and more than one are invalid, and *easing to neutral* has no meaning without it |
| **VAL-16** | `:part-with-child` | No part has children. A part draws and is a leaf; mesh groups and composites are the kinds that contain |

Duplicate identifiers are [ID-5](#a-every-entity-has-a-stable-identity-proposed).

| ID | Operation and assertion | How |
|---|---|---|
| **VAL-9** | `rigged:binding-count-mismatch-marked-stale` — the **valid** stale case: counts mismatch and the mark is present, so mark and derivation agree. Assert it validates, loads, renders and saves, and that the mark survives the round trip. Conflating this with VAL-5 rejects a file the format defines as valid | `test` |
| **VAL-10** | `rigged:texture-payload-corrupt` — a payload present but undecodable. Assert the puppet is **not** rejected, the texture is reported unreadable, the part keeps its reference, and a save writes the undecodable payload back byte-for-byte rather than re-encoding something it could not read | `test` |
| **VAL-11** | For every fixture in this section, assert the file on disk is byte-identical after the rejected load. A load never silently repairs and writes back | `test` |
| **VAL-12** | Every rejection names the rule broken and at least one offending identifier, in bounded output | `test` |
| **VAL-13** | One validator gates load and save, and runs as a standalone command. Assert the standalone verdict equals the loader's for every fixture in the catalogue, and that a save refuses to write a puppet the validator rejects | `test` |
| **VAL-14** | Validity is defined precisely enough to write a validator against the specification without our code. **Wrong looks like:** a rule in the specification whose outcome the reader cannot determine from the file alone | `gate` |

## H. Load limits **[proposed]**

Traces to the load-limit paragraph in
[A puppet that fails its integrity rules](puppet.md#a-puppet-that-fails-its-integrity-rules-is-rejected-not-repaired-proposed).
A **load limit** is not a ceiling and not an authoring limit.

| ID | Operation and assertion | How |
|---|---|---|
| **LIM-1** | Every load limit is declared in one place, and every declared limit has a fixture just over it. A limit with no fixture fails the check | `script` |
| **LIM-2** | For each limit — decompressed payload size, texture dimensions, vertex, node and parameter counts, tree depth, string length, total load memory — the just-over fixture fails the load with a readable error, and **peak resident bytes during the failed load stay below the declared payload limit**. That bound is exact and is the half that can be asserted | `test` |
| **LIM-3** | `rigged:decompression-bomb` declares a size within the limit and expands far beyond it. Assert the load fails **without having allocated the expansion**: peak resident bytes during the attempt stay below the declared limit, which is what "checked before allocating" means in practice | `test` |
| **LIM-4** | Every limit is **4× the corresponding `stress` dimension**, and `stress` loads successfully. The multiple is stated and provisional, so assert the figure rather than "at or above" — a drift to 1.01× would pass the weaker form while turning a defence into an authoring limit | `test` |
| **LIM-5** | No load limit appears as a choice presented to the person rigging. Read at review and at the interface gate | `judgment` |
| **LIM-6** | The time bound is **linear work, not wall clock**. With no reference machine a wall-clock figure would gate nothing, so assert instead that the work done on a rejected load grows linearly in the declared size across two fixtures an order of magnitude apart, and that no loop in the rejection path is unbounded | `test` |

## I. Nothing in a puppet is executed **[proposed]**

Traces to [Nothing in a puppet is executed](puppet.md#nothing-in-a-puppet-is-executed-proposed) and
asserts [I1](../invariants.md#i1-rigel2d-never-executes-anything-a-puppet-scene-or-source-supplies),
which says in terms that it cannot be carried by a test alone.

| ID | Operation and assertion | How |
|---|---|---|
| **EXEC-1** | Load every fixture in the catalogue under a harness that fails on process spawn, dynamic library load, outbound connection, or filesystem access outside the puppet's own container and the application's own directories. Assert none occurs, on the failing paths as much as the succeeding ones | `test` |
| **EXEC-2** | [EXT-5](#e-unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided) is the data half: hostile-shaped strings are preserved verbatim and never resolved or interpolated | `test` |
| **EXEC-3** | No dynamic-load, process-spawn or evaluation-shaped call is reachable from a parsing path | `script` |
| **EXEC-4** | The negative over all code, which neither of the above establishes | `judgment` |
| **EXEC-5** | A time-boxed fuzz run over the puppet parser: no crash, no hang, bounded memory | `test` |

## J. The same puppet saves to the same bytes **[proposed]**

Traces to [The same puppet saves to the same bytes](puppet.md#the-same-puppet-saves-to-the-same-bytes-proposed).

| ID | Operation and assertion | How |
|---|---|---|
| **CANON-1** | Load `rigged`, save to A; load A, save to B. Assert A and B are byte-identical | `test` |
| **CANON-2** | Save `rigged` under differing username, hostname, working directory, destination path, locale and time zone, with the clock advanced between saves. Assert all outputs are byte-identical, and that no output contains the test's username, hostname or any path it was saved to | `test` |
| **CANON-3** | Build the same puppet twice, adding the same entities with the same identifiers in two different orders. Assert the saved bytes are identical — ordering is the build's rule, not the input's | `test` |
| **CANON-4** | Re-import byte-identical source artwork; assert every stored texture payload is unchanged. Change one pixel of one image and re-import; assert only that texture's payload changes | `test` |
| **CANON-5** | Open `rigged` in the player, change a **channel mapping**, save. Assert everything other than the channel mappings is byte-identical. Live writes mappings into the puppet file by saving it, so this is the path that most easily churns a file and the one where a second application's serialiser can diverge from the first's | `test` |
| **CANON-6** | A random sequence of edits followed by undo-all yields bytes equal to the original. A property test over the operation set, including compound operations and import | `test` |
| **CANON-7** | Texture payloads are lossless: decode a stored payload and assert it equals the imported artwork pixel for pixel, in the cropped region | `test` |

## K. Adding artwork: when the art and the rig disagree **[proposed]**

Traces to [Adding artwork: when the art and the rig disagree](puppet.md#adding-artwork-when-the-art-and-the-rig-disagree-proposed).
**Stale** is a flagged state, never a validation failure.

| ID | Operation and assertion | How |
|---|---|---|
| **ART-1** | **Import provenance** matches in a stated order: the source identifier, then the content hash, then the name. Re-import `rigged`'s artwork three ways — every PSD layer renamed; the layer identifiers stripped so only the pixels match; the pixels altered and the identifiers stripped so only the name matches — and assert each part is matched to its own artwork in each case, with every identifier, binding, keypoint value and deformation unchanged. The middle case is the one a name-based implementation passes by accident and an identifier-only one fails | `test` |
| **ART-2** | Re-import with one PSD layer absent. Assert that part remains in place, is flagged as having lost its source, is reported, and that the flag survives save and load. Assert nothing else changed | `test` |
| **ART-3** | Replace one part's artwork with the same image at twice the dimensions. Assert the deformation offsets are scaled by the defined factor of the texture's coordinate space, the binding is **not** marked stale, and the evaluated geometry at each keypoint is unchanged | `test` |
| **ART-4** | Regenerate, with a different vertex count, the mesh of the part whose node carries both a transform binding and a deformation binding — so that one binding goes stale and one does not, which [ART-11](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) needs. Assert every deformation on it is retained with its authored values exactly as they were, that the stale mark is **derived from the mismatch** and agrees with it rather than being an annotation anything else writes, that it is reported; that the puppet loads, renders and saves; and that the stale data round-trips byte-stably | `test` |
| **ART-5** | After ART-4, the validator accepts the puppet and a save succeeds. A stale deformation is not a malformed file | `test` |
| **ART-6** | Delete a **texture** that two ordinary parts reference, one of which a **binding** deforms. Assert the report names both parts **and that binding**, before anything changes; that declining the confirmation leaves the puppet equal to what it was, in memory and in bytes; and that on confirmation the texture, both parts and that binding are gone and nothing else is — every other identifier, binding, keypoint value and deformation unchanged. The binding is the second level: a build that stops after the parts leaves exactly the dangling reference the cascade exists to avoid. Assert also that **nothing below the parts goes**, which is now a statement about the format rather than a hedge — a part is a leaf, so the only continuation from one is through its own dependents. Then assert the validator accepts the result and a save succeeds | `test` |
| **ART-7** | Delete a **parameter** and assert the cascade reaches everything that depends on it, not only the first kind. The report names its **bindings**, the **channel mappings** feeding it and the **physics** driving it — all three constitutive — and on confirmation each is gone. Assert the **nodes those bindings drove survive untouched**, same identifiers and their other bindings intact. Assert the **animation survives**, with its identifier, still driving the parameters it drove before and to the same values at the same sample times, and that nothing in it still refers to the deleted parameter. A criterion that checks bindings alone passes against a build that leaves two kinds of reference dangling; one that takes the animation passes against a build that destroys authored work | `test` |
| **ART-8** | A person looks at a puppet after ART-3 and after ART-4. **Wrong looks like:** the remapped part's artwork sitting in a different place or at a different scale relative to its neighbours; a part carrying a stale deformation looking torn or displaced rather than simply unposed for that one deformation | `gate` |
| **ART-9** | Undo the confirmed cascade of ART-6, of ART-7, of ART-10 and of ART-17 — the transitive and subtree cases as much as the one-level one. Assert the serialisation equals the pre-delete one byte for byte in each. A cascade is acceptable only because it is undoable, which makes this criterion load-bearing rather than incidental | `test` |
| **ART-10** | An **incidental** reference drops; the dependent stays. `rigged` has a part acting as a **mask** for another. Delete the mask part's texture and confirm. Assert the mask part is gone, that **the masked part survives unmasked** — same identifier, its own texture, bindings and deformations intact, no mask reference remaining — and that the closure stopped there, nothing beyond it touched. Deleting the masked part would satisfy the validator too, and lose work nothing required losing | `test` |
| **ART-11** | A **stale** deformation is skipped. After ART-4, evaluate the puppet across the stale binding's keypoints and assert the part's geometry equals that of a control puppet in which the stale binding has been deleted outright — rest geometry for that binding, as though it were absent. On the same node, assert a non-stale binding still applies, by comparing against the same control | `test` |
| **ART-12** | Containment is **asymmetric**, and one criterion asserts both halves so that neither can be implemented without the other. Upward: after ART-6 the deleted parts' parent mesh group survives with its identifier and one fewer child, the root is not reached, and every node not depending on the deleted texture is untouched — a container is not a dependent of what it contains, and a closure walking referents instead would reach the root and take the puppet. Downward: removing a node removes the subtree under it, asserted by [ART-17](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed). A build that gets one direction right and the other wrong fails here | `test` |
| **ART-13** | A **channel mapping** caught by a cascade is reported as its own case, not as one line item among others. Assert the reported set distinguishes the mappings, and marks them as data the player wrote, from the entities the editor itself authored. Asserted on the structured result of the operation, as [MSG-1](#f-what-the-user-is-told-decided) is, because neither application's interface exists yet | `test` |
| **ART-14** | A person deletes a parameter that a channel mapping feeds. **Wrong looks like:** the mapping appearing in an undifferentiated list of affected things, or the user confirming without having been told that mappings made in Live are among what goes | `gate` |
| **ART-15** | Every reference kind the format defines declares whether it is **constitutive** or **incidental**. A reference kind with no declaration fails the check — this is the rule that stops a new entity kind arriving with no stated answer, and it is worth nothing if it is not enforced mechanically | `script` |
| **ART-16** | For each declared reference kind, delete the target and assert the behaviour matches the declaration: constitutive, and the dependent is removed and the closure continues through it; incidental, and the dependent survives without the reference and the closure stops. **Containment is one of the kinds**, with the child as the dependent and the declaration constitutive. Table-driven from the same declaration ART-15 checks, so a kind added later is covered without a new criterion | `test` |
| **ART-17** | Removing a **node** removes the subtree under it. In `rigged:deep`, delete the inner mesh group directly. Assert the report names the group, its child part and that part's binding before anything changes; that on confirmation all three are gone; and that the group's own ancestors, up to and including the root, are untouched. Downward containment is constitutive; upward it is not, and [ART-12](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) is the other half | `test` |
| **ART-18** | An incidental dependent left **empty** still survives. In `rigged:solo-animation`, whose animation drives one parameter rather than two, delete that parameter. Assert the animation remains with its identifier, drives nothing, and that it survives save and reload. Removing it would be a deletion the user never confirmed; a build that tidies it away passes every other criterion here | `test` |
| **ART-19** | A cascade whose result would not validate is **refused, and says why**. In `one-part`, delete the texture: the part is constitutive on it and the part is the root, so the closure would leave a tree with no root. Assert the operation is refused, that the report names the validity rule that would break, that **no confirmation is requested** — the set is computed and validated before anything is offered — and that the puppet is unchanged in memory and in bytes | `test` |
| **ART-20** | The refusal is **general, not a root exception**. For each constitutive reference kind the format declares, construct the smallest puppet whose cascade would break a validity rule, and assert refusal naming that rule. Table-driven from the same declaration [ART-15](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) checks, so a kind added later is covered without a new criterion — which is the reason the requirement gives for stating it this way | `test` |
| **ART-21** | A container emptied by a cascade **survives empty**. Delete the texture drawn by every child of one mesh group. Assert the parts go, the group remains with its identifier and no children, and the result validates. The group is not a dependent of what it contained, so nothing removes it — and tidying it away would be a deletion the user never confirmed | `test` |

## L. One file **[decided]**

Traces to [One file](puppet.md#one-file-decided).

| ID | Operation and assertion | How |
|---|---|---|
| **FILE-1** | Copy `rigged` alone into an empty directory and load it. Assert it loads and the in-memory puppet is equal to the one loaded from its original location. Nothing is resolved from outside the file | `test` |
| **FILE-2** | Save `rigged`. Assert the directory afterwards holds the puppet and nothing else but the version-upgrade backup of [VER-4](#d-a-build-works-with-what-it-understands-and-preserves-the-rest-decided) where one is due — no temporary left behind, no derived artifact that could be mistaken for a working copy | `test` |
| **FILE-3** | Add images, a layered source document, and a merged puppet to `rigged`. Assert every pre-existing identifier, binding, keypoint value and deformation is unchanged | `test` |
| **FILE-4** | After importing a layered source document, assert no stored payload is that document or any part of it; the stored payloads are exactly the distinct cropped textures of [TEX-5](#b-a-texture-is-not-a-part-decided) | `test` |
| **FILE-5** | Import provenance is stored per part and round-trips: a **source identifier** taken from the source document where it provides one, and a **content hash of the imported pixels**. Assert it contains no path, username or machine name — the constraint [CANON-2](#j-the-same-puppet-saves-to-the-same-bytes-proposed) puts on it — and that nothing in it is ever resolved, which [EXEC-1](#i-nothing-in-a-puppet-is-executed-proposed) covers | `test` |
| **FILE-6** | The format declares, per kind of data, whether it is runtime or **authoring-only**, and the player skips exactly the authoring-only set. Load `reference` in the player and in the editor; assert the player touches no kind declared authoring-only and every kind declared runtime, rather than merely reading fewer bytes. *Which* kinds are authoring-only is still [open](#criteria-not-written-open); the declaration is what makes the criterion assertable without that answer | `test` |
| **FILE-7** | Every kind of data the format defines carries the runtime-or-authoring declaration. A kind with no declaration fails the check, the same shape as [ART-15](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) | `script` |

## M. Composition is deterministic **[proposed]**

Traces to [Composition is deterministic now and configurable later](puppet.md#composition-is-deterministic-now-and-configurable-later-proposed).

| ID | Operation and assertion | How |
|---|---|---|
| **DET-1** | Evaluate `rigged` at a fixed set of parameter values, with three bindings touching one node. Assert the evaluated vertex positions are **bit-identical** across repeated evaluations, across processes, and across a save-and-reload. One build, one platform, which is the scope of the bit-exact claim | `test` |
| **DET-2** | Apply the same parameter values in different orders of assignment. Assert identical geometry — the order values arrive in is not the order deformations apply in | `test` |
| **DET-3** | Evaluate the same puppet and inputs through the runtime as embedded in Studio and as embedded in Live, on one platform. Assert bit-identical geometry. This is the disagreement the shared **runtime** exists to prevent | `test` |
| **DET-4** | `rigged:three-deform` is small enough to work by hand: one node, three deformations with known offsets. Assert the evaluated result equals a committed expectation **derived from the specified order by hand, not from the implementation**. Without this, DET-1 only establishes that the build repeats itself | `test` |
| **DET-5** | With a fixed sequence of timesteps injected, **physics** output is bit-identical across runs. Nothing here asserts physics under a variable frame interval, because the requirement does not claim it | `test` |
| **DET-6** | The stored form carries a **composition mode** from the first version. Assert `rigged` carries it, that its value is the one meaning the fixed order specified now, that it round-trips, and that [DET-4](#m-composition-is-deterministic-proposed)'s hand-derived expectation holds for that value. A mode added later is then a new value, and cannot alter a puppet authored before it — which is the whole reason the field exists today | `test` |

## N. The deformation model **[proposed]**

Traces to [No general skeleton](puppet.md#no-general-skeleton-proposed) and
[A chain deformer is a permanent structure](puppet.md#a-chain-deformer-is-a-permanent-structure-not-a-bulk-edit-tool-proposed).

| ID | Operation and assertion | How |
|---|---|---|
| **CHAIN-1** | `rigged`'s chain deformer survives save and load with its joints and their identifiers intact, and evaluation after reload is bit-identical | `test` |
| **CHAIN-2** | Move one joint of the chain deformer and move it back; save. Assert the bytes equal the original. A chain deformer that had baked itself into per-vertex offsets at creation fails this, which is the bulk-edit-tool behaviour the requirement rejects | `test` |
| **CHAIN-3** | Move one joint and assert the evaluated geometry along the chain changes. Adjusting it later adjusts the rig | `test` |
| **CHAIN-4** | The deformation model offers no weight-painted vertex skinning and no general bone primitive; the chain deformer is the only bone-like one. A negative over the design, read at review | `judgment` |

## Concern entries carried **[decided]**

The entries in [concerns/](../concerns/) tagged for this area, and where each lands. Entries not
listed here belong to other areas or to subsystems that do not exist yet.

| Concern entry | Criteria |
|---|---|
| data — stable identifier, every reference by identifier | ID-1 … ID-7 |
| data — one validator gates load and save, standalone | VAL-13, and the stale pairing at VAL-5/VAL-9 |
| data — unmarked data rejected, marked extension data round-trips | EXT-1 … EXT-4, EXT-7, EXT-8, VAL-7, VAL-8 |
| data — unknown fields and entities survive a round trip | VER-1, VER-6, TOOL-4, EXT-7 |
| data — atomic write, failure injected at each step | VER-5 |
| data — saving an unmodified puppet is byte-identical | CANON-1 … CANON-3, CANON-5 |
| data — adding artwork preserves configuration | FILE-3, ART-1 … ART-4 |
| data — edits then undo-all equals the original | CANON-6, ART-9 |
| data — a file written without project code validates and loads | TOOL-1 |
| data — imported data validated as untrusted input | VAL-1 … VAL-12, VAL-15, VAL-16, LIM-2 … LIM-4, LIM-6 |
| security — hostile fixtures rejected cleanly | VAL-1 … VAL-11, VAL-15, VAL-16, LIM-3 |
| security — load limits in one place, a fixture just over each | LIM-1, LIM-2 |
| security — a failed load leaves the displayed puppet unchanged | VER-2 |
| security — every parser of untrusted input survives hostile input | EXEC-5 |
| security — nothing supplied by a file is executed | EXEC-1 … EXEC-4, EXT-5, EXT-6 |
| performance — parts sharing a texture leave resident bytes unchanged | TEX-1, TEX-7 |

## Criteria not written **[open]**

Each is a place where [puppet.md](puppet.md) does not say enough for a criterion to be written
without inventing the answer. They are recorded rather than guessed, and they are routed as
requirements work, not filled in here.

| What could not be written | What it needs |
|---|---|
| The **cross-platform** determinism tolerance | The shape is settled and the criterion is one line away: evaluate `reference` at a committed set of parameter values on each supported tier and assert the maximum per-vertex deviation is within the tolerance. **Only the figure is missing**, and it is `[open]` in the requirement |
| **Which** kinds of data are authoring-only | The format declares it per kind, which is what [FILE-6](#l-one-file-decided) and [FILE-7](#l-one-file-decided) assert. The set itself is not written down, so no criterion can name a kind and say which side it falls on |
| The name for the part of an **animation** that drives one parameter | [vocabulary.md](../vocabulary.md) does not define one, and a term is settled with the owner rather than invented by whoever needs it first. [ART-7](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) and [ART-18](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed) both have to talk around it |
| Whether **physics** is a node | The pieces list says the node tree is parts, mesh groups and composites, and names physics separately; [platform.md](platform.md#the-reference-puppet-and-the-stress-puppet-proposed)'s workload table counts *physics nodes*. [ART-16](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed)'s containment declaration and [ART-17](#k-adding-artwork-when-the-art-and-the-rig-disagree-proposed)'s subtree case both need the answer eventually, and neither depends on it as written |

**Identifier reuse by a writer that is not ours is not here, because it is not a gap.**
[testing.md](../testing.md) records it as a recorded limitation: duplicates within one file are
detectable and rejected, reuse across time by a stranger's tool is undetectable without history
nobody keeps.
