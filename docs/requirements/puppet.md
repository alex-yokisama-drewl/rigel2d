# Requirements — the puppet

What a puppet is, and what its file must guarantee. This is the contract between the editor and
the player, and the reason both are specified before either is built.

## What a puppet is **[proposed]**

A puppet is a layered 2D character rig: artwork cut into parts, arranged in a tree, deformed in
response to named inputs so that a static drawing appears to move in three dimensions.

Its pieces, named in [vocabulary.md](../vocabulary.md):

- a **node tree** of parts, mesh groups and composites, which fixes draw order and nesting;
- **textures**, referenced by parts;
- **parameters**, each a one- or two-dimensional axis with keypoints along it;
- **bindings**, each mapping a parameter to something that changes — a node transform, or
  per-vertex offsets on a mesh — with values stored at keypoints and interpolated between;
- **physics**, which drives parameters from simulated motion rather than from input;
- **animations**, which drive parameters over time;
- **channel mappings**, each feeding one parameter from a named input channel. These are puppet
  data although they are edited in Live rather than Studio, so that a delivered puppet reacts
  for whoever receives it — see
  [performing.md](performing.md#channel-mappings-are-puppet-data-and-live-writes-them-decided).

Inochi2d's model is the starting point and its vocabulary of pieces is largely right. This
project departs from it deliberately in several places recorded below — textures as entities
in their own right, deterministic composition, a chain deformer that persists — and those
departures are the point rather than incidental.

## No general skeleton **[proposed]**

No bones with weight-painted vertex skinning. The deformation model stays parameter-and-deform: node
transforms, per-vertex offsets keyed to parameter keypoints, and nesting. One bone-like
primitive is wanted and is described further down; it is not a general skeleton.

**Because the incumbent that costs money does not have one either.** Live2D Cubism has no bone
or skeleton primitive
([2026-11-26-deformation-models.md](../research/2026-11-26-deformation-models.md)). Its
deformers are a control-point grid and a rotation about an origin, nested in a tree and driven
by parameters — structurally the same answer. Its one skinning-adjacent feature was added for
hair and is a chain of rotation deformers — a limited stopgap, and its manual describes fuller
skinning as future work. So the evidence is that general skinning is not *required* for 2D
vtubing, not that nobody wants it: Animaze, a live vtubing runtime, chose GPU skinning.

**The risk, stated plainly:** this holds for faces, expressions and upper body, which is what
this audience does. It fails if scope widens to full-body rigs with tracked arms, where posing
a limb through parameter keypoints multiplies the authoring work and a missing skeleton becomes
the headline complaint.

## Composition is deterministic now and configurable later **[proposed]**

Several deformations routinely apply to one node at once — the generating case is head roll,
pitch and yaw, three parameters all touching the same node, which is ordinary rigging rather
than an edge case.

There is no good universal rule for combining them, and this project does not pretend to one.
What it commits to instead:

- **Evaluation is deterministic and specified in the format.** The order in which deformations
  apply is defined, not emergent. The same puppet and the same inputs produce the same geometry,
  every time, in the editor and in the player alike.
- **Composition modes are `[open]`** — a later addition, once there is evidence about which
  ones are worth having.

**Because:** what makes the current situation workable in practice is that a rigger can adjust
values by testing until it looks right. That only works if the result is *repeatable*. An
undefined evaluation order does not merely produce surprising output; it destroys the feedback
loop the rigger is relying on, and it is how the editor's preview comes to disagree with the
player's output — the failure the shared runtime exists to prevent. Determinism is the part
that must be right first, and it is cheaper than a composition model.

## A chain deformer is a permanent structure, not a bulk-edit tool **[proposed]**

A chain of linked joints deforming the geometry along it, for long thin parts — hair, tails,
ropes. Wanted, and worth building.

**Built as a persistent part of the rig.** This is the whole point of the entry. Inochi2d's path
deformer does the same job and works, but is unwieldy in use, and the reason is structural: it
behaves as a **single-use tool for bulk-setting deformation** rather than as a lasting feature
of the rig. What a rigger expects from something shaped like bones is that it stays, and that
adjusting it later adjusts the rig. Making it permanent is most of what makes it usable.

Cubism arrived at a similar primitive for the same parts from the other direction — its hair
feature is a chain of rotation deformers. Convergence from two directions is encouraging rather
than conclusive; Cubism's is a documented stopgap, and how inochi2d's path deformer composes
internally has not been confirmed.

It is the only bone-like primitive in the deformation model, and it does not generalise into a
skeleton.

## A texture is not a part **[decided]**

A texture is an entity in its own right. A part references a texture; many parts may reference
the same one. Reusing a drawing must cost one import and one texture, however many parts use it.

**Because:** in inochi2d the two are conflated, so using the same shape six times means
importing it six times and paying for it six times in the packed atlas. The cost lands on
exactly the work beginners do most — building a puppet out of repeated simple shapes.

How textures are packed for the GPU is an implementation concern and must stay invisible to the
person authoring. Packing may not impose a cost that authoring can see: no duplication, no
manual atlas sizing, no arbitrary dimension caps presented as a choice.

**Packing must not cost memory.** Measurement of two real inochi2d rigs
([2026-11-26-inochi-file-measurements.md](../research/2026-11-26-inochi-file-measurements.md))
found its export packing one model's 26.9 Mpx of artwork into an 8192×8192 sheet — 67.1 Mpx,
and 256 MB of GPU memory to hold 103 MB of artwork. The other fared worse by ratio. The waste
is invisible in the stored file, because empty space compresses, and arrives in full the moment
the puppet loads. So:

- **Resident texture memory does not exceed the source artwork**, and the ambition is to beat
  it. Packing that inflates is worse than not packing.
- **Parts are cropped to their own content before packing.** The same measurement found eight
  parts exported at an identical 920×780 with stored sizes from 0.03 MB to 1.16 MB — mostly
  transparent padding, each paying 2.74 MB resident regardless. Cropping is most of the
  available win and requires nothing clever.

## The file is readable and writable by tools that are not ours **[decided]**

A third party must be able to inspect a puppet, change it, and generate one, using ordinary
tooling and without our code. This is a stated goal in its own right — it is what makes
community tooling and format longevity possible, and it is the main thing inochi2d's container
format gets wrong by wrapping JSON sections in a binary envelope that no text tool can open.

What follows from it, as constraints rather than design:

- the structure is self-describing, and its meaning is discoverable from the file;
- binary payloads — textures above all — are addressable as separate units rather than embedded
  in a way that forces a reader to parse the whole file;
- a reader that does not understand part of a puppet can still read the rest.

The specific encoding is [tech-design](../README.md)'s call. Plain JSON satisfies this; so does
an archive with a manifest. A single opaque binary blob does not.

## Every entity has a stable identity **[proposed]**

The puppet itself, and every node, texture, parameter, binding, keypoint and animation in it,
carries an identifier that does
not change when it is renamed, reordered or saved. Every reference between them is by that
identifier — never by name, never by position in an array, never by offset in the file.
Identifiers are unique within a puppet and are never reused for a different entity after a
deletion.

**Because:** this is the mechanism behind "adding artwork preserves configuration", stated below. It is
also what lets a scene bind to a puppet's parameters and survive the puppet being edited, and
what lets a merge renumber what it brings in instead of refusing it. Name-based or index-based references
break on exactly the operations this project exists to make easy, and retrofitting identity into
a format already in the wild is not possible.

## A build works with what it understands and preserves the rest **[decided]**

The governing rule, and it is wider than versioning: **a build never discards data it does not
understand, and never refuses to work merely because some of a file is unfamiliar.** It operates
on what it recognises, leaves everything else exactly as it found it, and writes it back on
save.

The one thing it does refuse is a file whose *version* declares a break in compatibility, below.
That is a different case: not "part of this file is unfamiliar" but "this file is not the format
I read".

A puppet declares its format version, readable before anything else is parsed. The rules:

- A file whose version adds things this build does not know **opens, and stays editable**. What
  the build does not understand is carried through to the next save byte-for-byte. Silently
  dropping it destroys the work of someone on a newer build, and that loss is invisible until
  the file reaches them again.
- A file whose version breaks compatibility is **refused with a clear message**. It is never
  opened partially and then saved over.
- Opening never writes. The first save that upgrades a file keeps a backup of the original.

### Unrecognised data is marked as such by whoever wrote it **[decided]**

Preservation only works if the loader can tell *"data I do not understand"* apart from *"this
file is malformed"*. So anything outside the core format carries an explicit marker naming the
extension it belongs to, and the validator requires that marker.

| Rule | |
|---|---|
| Extension data is namespaced and declares its origin and version | An identifier for the extension, the version it expects, and something human-readable to show a user who does not have it. **A puppet declares; it never installs** — see [invariants.md](../invariants.md) I6 |
| Extension data is data | It is structured values — never a script, a command, a path or a URL to resolve. The no-execution rule covers it without exception |
| Unmarked data that the core format does not define is **invalid**, not an extension | Otherwise a typo and an extension are indistinguishable, and the validator can reject neither |
| Preserved data round-trips byte-stably | It takes part in canonical serialisation below, so carrying someone's extension through a save does not make the file churn |

**Because:** the alternative — refusing to open, or opening read-only — protects the unknown
data perfectly and stops the work. The likelier case is someone needing a small change to a file
carrying something they do not have, and the right answer is to let them. This was decided
against read-only after a reversal; the alternatives and why they lost are in
[decisions.md](../decisions.md).

### What the user is told **[decided]**

**On opening a file carrying anything unrecognised, the build says so**, names what is missing —
the format version, or each extension by its human-readable name — and warns that the features
behind it may not work correctly in this build. Then it lets the user work.

This is the honest position: the file will very probably survive the edit intact, and the user
is told what they are carrying and what they might install. It is not a guarantee, and it is not
presented as one.

### When this becomes a hard promise **[open]**

Compatibility is a design consideration from the start — the format is built so that these rules
*can* hold on every 0.x release, and gratuitous breakage is avoided. But **the hard promise does
not start at a version number.** It starts when the project is shared with a community, and the
project owner will say when that is.

Until then, a breaking change is allowed where it is genuinely warranted. What is not allowed is
designing as though compatibility will be bolted on later: these rules exist before the first
release because they cannot be added afterwards.

## A puppet that fails its integrity rules is rejected, not repaired **[proposed]**

Because third-party tools write puppets ([above](#the-file-is-readable-and-writable-by-tools-that-are-not-ours-decided)),
files arrive from anywhere and may be malformed, hostile, or simply buggy. Validity is defined
precisely enough that a validator can be written against the specification and run without our
code, and the same validator gates loading and saving.

| Rule | |
|---|---|
| Identifiers are unique | |
| The tree is acyclic, with one root | |
| Every texture, parameter and node reference resolves | A reference naming nothing is a malformed file. **Distinct from a texture whose payload is present but unreadable**, which loads with a visible placeholder ([authoring.md](authoring.md)) rather than failing the puppet |
| Keypoints are ordered and within range | |
| Binding value counts match the mesh topology they apply to, or the binding is marked stale | Staleness is the flagged outcome of [artwork changing under a rig](#adding-artwork-when-the-art-and-the-rig-disagree-proposed) — a state the rigger resolves, not a malformed file |
| Every number is finite | NaN and infinity are rejected, not clamped silently |
| Enumerated values are known, or explicitly marked as extensions | |

Every size and count the loader trusts has a **load limit**, declared in one place, checked
**before allocating** — decompressed payload size, texture dimensions, vertex and node and parameter
counts, tree depth, string length, total load memory. Exceeding one fails the load with a
readable error in bounded time and memory. These limits are a defence against a corrupt or
hostile file and are generous multiples of the stress puppet in
[platform.md](platform.md#the-reference-puppet-and-the-stress-puppet-proposed); they are **not** authoring
limits, and they must not reappear as a choice presented to the person rigging.

A load never silently repairs a file and then writes the repair back.

## Nothing in a puppet is executed **[proposed]**

A puppet is data. No field in it is evaluated as a script, a command, or a format string, and no
path or URL inside one is resolved outside the container it arrived in.

**Because:** puppets are traded between strangers — a rigger delivers to a streamer, and the
community shares files. Opening one must not be able to harm the machine.

This is [I1](../invariants.md), and it binds **Rigel2D's own code**. It is not a promise about
everything that may run on the machine once plugins exist: a plugin can store code in a puppet's
extension data and execute it itself, and the format cannot prevent that. That residual risk is
accepted as part of allowing plugins, and is why a plugin may never arrive inside a shared file
([I6](../invariants.md)) — installation stays an explicit act, out of band from file sharing.

## The same puppet saves to the same bytes **[proposed]**

Serialisation is canonical: stable ordering, no timestamps, no absolute paths, no usernames, no
machine names. Texture payloads are lossless and byte-identical across saves unless the artwork
itself changed.

**Because:** it makes version control and third-party diffing work, which is most of the point
of an open format — and it means a delivered puppet carries nothing about the machine that made
it.

## Adding artwork: when the art and the rig disagree **[proposed]**

"Adding artwork preserves configuration" is settled. This is the behaviour when returning
artwork no longer matches the rig built on it — the cases where a wrong answer destroys work.

**Remap where a remap is defined; otherwise retain and flag.** Nothing is discarded and nothing
is silently reinterpreted. A deformation whose mesh moved under it is kept exactly as authored,
marked as no longer matching, and reported; the rigger resolves it at their own pace. The puppet
stays loadable and editable throughout — a stale deformation is a flagged state, not a
validation failure.

| Case | Behaviour |
|---|---|
| Art re-exported and re-imported after an edit | Matched to existing parts by stored import provenance rather than by name alone. Art that no longer appears leaves its part in place, flagged as having lost its source |
| Replacement art has different dimensions | Remapped. Deformations are offsets in the texture's coordinate space, so a dimension change is a defined scaling of that space |
| A mesh is regenerated or edited by hand | Retained and flagged. Topology changed, so there is no correspondence to carry offsets across and any remap would be a guess at the rigger's intent |

Deleting a rigger's deformation work because a texture changed size is the worst thing this
application could do, which is why the fallback is retention rather than a best-effort remap.

What follows, as constraints rather than design:

- **A deformation can outlive the mesh it was authored against.** A binding carries enough to
  tell whether it still matches, and a mismatch is a flag rather than the malformed-file case
  below.
- **Stale deformation data round-trips**, like any other data a build carries but does not act
  on.
- **Import provenance is stored per part** — the originating PSD layer, which
  [One file](#one-file-decided) already requires.

Deletion has the same character: deleting a texture that parts use, or a parameter that bindings
reference, reports what it affects rather than cascading quietly.

## One file **[decided]**

A puppet is one file. There is no separate editing project and no separate exported runtime
form. Any puppet, from anywhere, opens in the editor.

**Because:** there is nothing a project file would hold that a puppet should not. Measurement of
two real inochi2d rigs found the separation buying nothing — the figures are in
[2026-11-26-inochi-file-measurements.md](../research/2026-11-26-inochi-file-measurements.md),
and what it costs and which alternatives lost are in [decisions.md](../decisions.md).

Consequences that follow and are settled with it:

- **Adding artwork to a configured puppet preserves its configuration.** No export step can
  desynchronise from a source.
- **There is no derived artifact that can be mistaken for a working copy**, which was the main
  risk of the alternative.
- **The runtime skips what it does not need.** Authoring data is addressable separately, so the
  player does not pay to load it.

What a puppet stores is the **cut-up parts**, which are what rendering needs anyway, together
with enough import provenance — the originating PSD layer's name, say — to match returning artwork
on re-import. **The layered source document is not embedded.** It is the artist's working file
and belongs beside the puppet, not inside it, which also disposes of most of the question about
what travels when a rigger delivers to a client.

## How textures are stored is not how they reach the GPU **[proposed]**

These are two questions and conflating them is what produced the waste measured above.

**Storage: one entry per distinct texture, cropped to its own content.** This is better than a
packed sheet on nearly every axis — no padding, each image compresses on its own terms, a small
edit rewrites one entry rather than the whole payload, reuse is free because a texture is
already a referenced entity, and third-party tools can read one image without understanding the
packing. It also follows from the format requirement that binary payloads be addressable as
separate units.

**GPU layout: an internal decision, invisible in the file, deferred.** Whether the runtime packs
textures into a sheet, uses a texture array, or simply binds each one is a question about draw
call batching, and the honest answer is that **at this scale it probably does not matter.**
Batching earns its keep with thousands of sprites; a puppet has tens of parts, so a bind per
distinct texture is a few dozen state changes per frame. Meanwhile composites and masks force
offscreen passes, which cost far more and which packing does nothing about — so a packed sheet
optimises the cheap thing while the expensive thing goes unaddressed.

Packing also brings costs that are easy to overlook: filtering at cell edges samples neighbouring
cells, so gutters are needed, which is more padding; every artwork change repacks everything;
and maximum texture dimensions force multiple pages above a threshold anyway.

So the runtime binds per texture until the per-frame counters in
[platform.md](platform.md#how-this-is-verified-proposed) show that binds matter. **This is exactly the
kind of question the counters exist to settle**, and settling it by measurement later costs
nothing, because the file format does not depend on the answer.


