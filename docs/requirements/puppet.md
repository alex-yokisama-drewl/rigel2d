# Requirements — the puppet

What a puppet is, and what its file must guarantee. This is the contract between the editor and
the player, and the reason both are specified before either is built.

## What a puppet is **[proposed]**

A puppet is a layered 2D character rig: artwork cut into parts, arranged in a tree, deformed in
response to named inputs so that a static drawing appears to move in three dimensions.

Its pieces, named in [vocabulary.md](../vocabulary.md):

- a **node tree** of parts, mesh groups, composites and physics, which fixes draw order and
  nesting. A **part is a leaf**: parts draw, mesh groups and composites contain, and physics
  anchors. Keeping each kind to one job is what makes the tree's shape say where deformation and
  compositing can happen, and it is why a cascade reaches the parts under a deleted group and
  stops there. Art that wants to sit under a shared transform goes in a mesh group, which is what
  one is for. **What each kind may contain is declared per kind by the format** — a part contains
  nothing, a mesh group or a composite contains any kind, a physics node contains only physics —
  so that a kind added later arrives with a stated answer rather than a validator exception, and
  so that a deletion cascade can never reach artwork through a physics node;
- **textures**, referenced by parts. A part's reference carries a **texture flip** per axis, so two parts
  may share one drawing with one of them reversed — the ordinary case for a pair of eyes;
- **parameters**, each a one- or two-dimensional axis with keypoints along it. A parameter
  **declares its axis range** — a minimum and a maximum per dimension — and carries **exactly
  one neutral keypoint**. The validity rule that keypoints are within range presupposes the
  first, and easing to neutral in [performing.md](performing.md) presupposes the second. It
  also **declares axis polarity per dimension** — bilateral, unipolar, or **unknown** — which a
  mirror reads to know whether reversing that axis has a meaning, and which it never writes.
  *Unknown* is a declared value, not a missing one, because the alternative is to guess, and a
  guess is wrong for every head-turn parameter it touches and invisible once made. Its route in
  is **a writer that is not ours** — the format is one third-party tools generate
  ([below](#the-file-is-readable-and-writable-by-tools-that-are-not-ours-decided)), and a
  generator with no notion of polarity needs a way to say so honestly rather than pick. Studio's
  own paths do not produce it: a merge takes a puppet that already declares polarity, and
  importing artwork creates no parameters. **The value is therefore dormant in anything Studio
  writes**, which is the expected state and not a sign it is unused. A parameter also has a
  place in an order the rigger sets and a mark saying whether Live offers it for mapping
  ([below](#parameters-are-ordered-and-marked-for-mapping-proposed));
- **bindings**, each mapping a parameter to something that changes — a node transform, or
  per-vertex offsets on a mesh — with values stored at keypoints and interpolated between;
- **physics**, a kind of node: simulated motion anchored in the tree, which reads its own
  accumulated transform from the previous frame, displaces that node's own transform by the
  result, and may drive one parameter. **Driving a parameter is optional**, because a physics
  node is equally a link in a pendulum chain — which is why the reference to the parameter it
  drives is incidental rather than constitutive, [below](#deletion-is-report-then-a-confirmed-cascade-decided).
  **A chain lags by one frame however long it is**, because each node reads a previous frame that
  already carried its parent's displacement;
- **animations**, which drive parameters over time, each composed of **tracks**. One track
  drives exactly one parameter, and an animation carries at most one track per parameter. Each
  declares how it ends and what starts it — [below](#an-animation-adds-to-its-parameter-proposed);
- **chain deformers**, each a chain of **joints** deforming the geometry along it — see
  [below](#a-chain-deformer-is-a-permanent-structure-not-a-bulk-edit-tool-proposed);
- **a gravity**, one magnitude for the whole puppet which every physics node scales by its own
  multiplier. Physics is the only thing that reads it, and one puppet-wide field is what lets a
  rigger stop every pendulum at once. **A puppet that carries none uses the default rather than
  being invalid**, which is what lets a generator that has never heard of physics write a valid
  file;
- **masks**, each a reference from one part to another restricting where the first draws, as
  [vocabulary.md](../vocabulary.md) defines;
- **channel mappings**, each feeding one parameter from a named input channel. These are puppet
  data although they are edited in Live rather than Studio, and **Live writes them into the
  puppet file** by saving it, so that a delivered puppet reacts for whoever receives it — see
  [performing.md](performing.md#channel-mappings-are-puppet-data-and-live-writes-them-decided).
- **mirror axes**, one left–right and one top–bottom, which a mirror reflects across. Authoring
  data, defaulting to the puppet's centre. The player loads it and does not act on it, exactly as
  it does with axis polarity — see [One file](#one-file-decided).

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
  apply is defined, not emergent — and so is the order of every sum, because float addition is
  not associative and three contributions give different results by grouping
  ([evaluation.md](../evaluation.md#the-order-of-the-sum-is-fixed-by-data-proposed)). The claim is three distinct claims, because one unqualified
  sentence would promise more than can be kept:
  - **The same build on the same platform is bit-exact.** The same puppet and the same inputs
    produce the same geometry, every time, in the editor and in the player alike.
  - **Physics is deterministic given a fixed timestep.** It is not claimed to be reproducible
    under a variable frame interval. That is a property of time-stepped simulation, not a defect.
  - **Across platforms, the same geometry within a stated tolerance**, not bit-exact. The
    supported tiers span three operating systems, and bit-exactness across them is not promised.
    **The tolerance figure is `[open]`** — nothing is set, and nothing may assume a number until
    it is.
- **Composition modes are `[open]`** — a later addition, once there is evidence about which
  ones are worth having. **What is settled is that the stored form carries a composition mode
  from the first version**, with one defined value meaning the fixed order specified here.
  Adding a mode later then adds a value and cannot alter any puppet authored before it. It costs
  one field today and cannot be added once the format ships.

**Because:** what makes the current situation workable in practice is that a rigger can adjust
values by testing until it looks right. That only works if the result is *repeatable*. An
undefined evaluation order does not merely produce surprising output; it destroys the feedback
loop the rigger is relying on, and it is how the editor's preview comes to disagree with the
player's output — the failure the shared runtime exists to prevent. Determinism is the part
that must be right first, and it is cheaper than a composition model.

## An animation adds to its parameter **[proposed]**

**Every track adds its contribution to the parameter it drives.** There is no replacing mode and
no precedence between tracks, so no track is ever discarded: an idle animation plays underneath
live tracking, and a lost source's fallback layers on rather than blocking. **A track's
keyframes store absolute values, as keypoints do, and its contribution is that value less the
parameter's neutral keypoint**, so a track at the neutral pose contributes nothing. An animation
can therefore push a parameter and never force it to a value; that is accepted deliberately.
The mechanics, the fixed order of the sum and the reasons are in
[evaluation.md](../evaluation.md#a-track-adds-to-its-parameter-proposed), and the supersession of
the earlier override rule is in [decisions.md](../decisions.md).

**The one-driver rule binds parameters and not nodes.** Several bindings sum on one node and node
transforms are not clamped; a parameter has at most one base driver and is clamped. A track is
not a base driver.

### A parameter clamps to its declared range **[proposed]**

**The sum of every contribution to a parameter is clamped to the range its axes declare.** A
track's keyframe values are required to sit inside that range; their sum with a base driver and
with other tracks need not.

- **A non-finite sum rests the parameter at its neutral keypoint**, never at a range edge and
  never reaching geometry ([I2](../invariants.md)).
- **The clamp is derived and never stored.** No file records that a parameter was clamped.
- **Studio surfaces clamping while authoring**, so the rigger can see a layer being eaten rather
  than discover it on stream. Clamping with no way to see it is the silent discard this project
  refuses everywhere else.
- **Reporting is edge-triggered and coalesced** — once when a parameter starts clamping, once when
  it stops, never per frame — and a report names the parameter by identifier, never by name,
  because a name is puppet content.
- **Studio can predict only part of it.** It has no tracking input, so from the file alone it can
  report where a parameter's animation extent plus its driver's declared range could exceed the
  parameter's range. The live case belongs to Live.

**How the runtime surfaces a clamp to a streamer mid-stream is `[open]`.** It is never on the
render surface ([I2](../invariants.md)). Beyond that it belongs with how a streamer learns
something broke in [performing.md](performing.md), which is `[open]` and now gates this. Nothing
may assume an answer.

### A track declares how it interpolates, and an animation how it ends **[proposed]**

**A track declares its interpolation per segment**: linear, stepped, inverse stepped or Bézier.
They are Cubism's own, because importing animation that already exists is a case this project
serves. A single track may mix them. **A Bézier segment's control times lie inside the segment
and every control value is finite**, and evaluating one yields a finite value inside the segment
for every input, degenerate control points included. **Interpolation between a parameter's
keypoints stays linear and only linear** — a different question, unchanged.

**An animation declares what happens at its last keyframe**: it **loops**, **holds** — keeps
contributing its final values until stopped — or **releases**, stopping its contribution and
blending out over its declared duration. It is not a type of animation but one of the answers to
a single question.

### An animation may be started by a scene event **[proposed]**

**A trigger lives on the animation it starts and is only a start event.** It names the
**scene event** it waits for, defined in [vocabulary.md](../vocabulary.md), and nothing else: no
target, no command, no condition, no delay, nothing evaluated. **A puppet references an event by
name**, as it does a channel, which is outside what [I3](../invariants.md) binds. **An event name
nothing raises never fires and is not a validation failure**, because the scene or plugin that
raises it may not be present, and a puppet carrying triggers for events nothing raises is a normal
puppet. Matching a name against raised events is a lookup and not evaluation, so
[I1](../invariants.md) is untouched. The stored shape, and why it makes sequencing inexpressible,
is in [format.md](../format.md#a-trigger-proposed).

### Losing a source rests its parameters at neutral **[proposed]**

**A parameter whose base driver has no data rests at its neutral keypoint**, not at whatever a
channel value of zero maps to: a response curve need not be centred on zero, so *all zeros* would
leave a puppet subtly off-pose. **The scene decides what losing a source means and the puppet
decides what to do about it** — the scene owns sources, so interpreting their absence is its job;
the puppet owns its animations, so reacting is its. Resting at neutral is what *ease to neutral*
means in [performing.md](performing.md), which owns the scene's drop behaviours, *hold the
last pose* among them.

**The signal carries no payload.** It says a source is lost and nothing else — no source name, no
port, no address, no scene identity — so a puppet file cannot learn which trackers or ports a
person has.

**With several sources, whether loss is per source or per puppet is `[open]`.** The puppet never
learns which source, so the scene must decide what counts as lost. Nothing may assume an answer.

### Long animations are ordinary **[proposed]**

**A puppet may carry several animations of a minute or more, at a thousand keyframes and
upwards**, and both extremes must load and play: a long loop holding two keyframes, and one
holding a keyframe per frame. Per-frame cost is one curve evaluation per playing track, not per
keyframe, which is why this is cheap to promise ([evaluation.md](../evaluation.md#a-tracks-value-at-a-time-proposed)).

### A limit stops a hostile file; a budget describes what is tuned for **[decided]**

A **limit** exists so a file traded between strangers cannot exhaust memory, and is set from
memory safety. A **budget** describes the workload the frame targets are measured against, and is
set from what a rig realistically contains. **A puppet may sit inside every limit and outside the
budget: it loads, it plays, and it misses the frame budget on the reference machine, and that is
the author's choice, not a refusal** — the same bargain as opening fifty browser tabs. The figures
are in [format.md](../format.md#load-limits-proposed).

### Parameters are ordered and marked for mapping **[proposed]**

**A rigger arranges the puppet's parameters, that order is stored, and Live presents them in it.**
Live's mapping list otherwise shows every parameter in an order nobody chose. **An order is not
an identity**: every stored reference is still by identifier ([I3](../invariants.md)) and nothing
resolves through the order.

**A parameter carries a `mappable` flag, set in Studio and stored, saying whether Live offers it
as a mapping target.** No rule can infer it from the puppet's shape, because under additive
composition a parameter may be both mapped and animated, so *is it animated* does not answer
*should it be offered*. Only the author knows.

**It is called `mappable` rather than a mark** because the format already has *marked* extension
data and a binding's *stale* mark, and a third sense of one word is the collision
[vocabulary.md](../vocabulary.md) exists to prevent.

**A parameter that carries a channel mapping is mappable whatever the flag says.** The effective
value is the stored flag or the presence of a mapping, which makes the word honest: there is no
state in which a file calls a parameter unmappable while something is in fact mapped to it. That
state is the one every concern sweep found bugs around — a streamer facing a mapping they can
neither locate nor remove, because the flag can only be changed in the other application.
Deriving the value makes it unreachable instead of making it somebody's job to remember.

**The derived value is never stored.** Only what the author set is written, so removing a
mapping hides the parameter again rather than silently losing the rigger's intent.

**It hides and does not forbid**: presentation, not validation. A file whose flag disagrees with
its mappings is not malformed — that disagreement has a defined reading.

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
**Textures are deduplicated by content hash at import**, so importing the same artwork for two
parts yields one texture entity.

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
  it. Packing that inflates is worse than not packing. The comparison is against the artwork
  **as delivered**, not as cropped: against the cropped form the requirement would be nearly
  tautological, and as delivered it is the real guarantee.
- **Parts are cropped to their own content before packing.** The same measurement found eight
  parts exported at an identical 920×780 with stored sizes from 0.03 MB to 1.16 MB — mostly
  transparent padding, each paying 2.74 MB resident regardless. Cropping is most of the
  available win and requires nothing clever.

### A texture's life is bound to the parts that reference it **[decided]**

**Nothing in Studio deletes a texture.** No operation takes a texture as its subject. A texture
arrives by import and leaves when nothing draws it, and the rigger never has to think about the
difference between the drawing and the part that uses it.

**Because:** every route to a texture runs through a part. Art is imported, which makes both at
once; a part is copied, which shares the texture; art is re-imported, which replaces what a part
draws. There is no workflow in which a rigger has a texture and no part for it, so an operation
to remove one is an operation to create exactly the problem it then has to solve. Removing it
also removes the cascade *from* a texture, which was the only cascade that could reach a
puppet's root.

- **A texture is collected the moment no part references it**, whatever edit caused that: the
  last part referencing it was deleted, a re-import repointed the only part that drew it, a
  paste was undone. **Not on save**, and not on deletion alone. A deletion-only rule leaks —
  re-importing art repoints a part without deleting one, so iterating on artwork would
  accumulate dead textures, and once saved they become indistinguishable from the arrivals
  below.
- **A texture that was unreferenced when the file was loaded is never collected.** Both halves of
  that sentence carry weight. Because collection is immediate, an orphan can no longer be
  *created* and then saved, so this case only ever means a file written by a third-party tool or
  one whose references live in extension data this build cannot read.
- **Collection belongs to the operation that caused it** — the same undo step, named in the same
  result. Undo restores the payload byte-identically under its original identifier, and a
  collected identifier is retired like any other ([Every entity has a stable
  identity](#every-entity-has-a-stable-identity-proposed)). Deduplication by content hash must
  drop a collected texture too, or a later import matches a payload that is no longer there.
- **Where the puppet carries material this build cannot read, a collecting edit reports it and
  asks.** The protection above covers a texture an extension *solely* references; it does not
  cover one referenced by both a part and an extension, which is not unreferenced at load and so
  is collected when that part goes. That case cannot be detected, only disclosed — so in a puppet
  carrying anything opaque, an edit that would collect says so and the rigger confirms. This is
  the same *report, then confirm* shape [deletion](#deletion-is-report-then-a-confirmed-cascade-decided)
  already uses, rather than a second mechanism.
- **Artwork nothing draws can be removed deliberately.** One command, over unreferenced textures
  only, reporting what it will take and how large it is, and undoable. It is the exception to the
  first rule in this section and it is narrow on purpose: it cannot cascade, because nothing
  references what it removes, and the identifiers it frees are retired like any other. Without it a
  rigger who receives a puppet carrying unused artwork
  has no way to remove it, which is a delivery-size problem and a privacy one — a puppet
  redelivered to a client would carry another client's drafts under the sender's name, on every
  save.
- **An unreferenced texture is valid, and costs nothing to hold.** The validity rules take no
  view on whether a texture is drawn. It is never decoded, never uploaded and never counted
  against resident texture memory, so a file carrying unused artwork cannot inflate the frame
  budget — only the file.

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

The puppet itself, and every piece listed in
[What a puppet is](#what-a-puppet-is-proposed), carries an identifier that does not change when
it is renamed, reordered or saved. Every reference between them is by that identifier — never by
name, never by position in an array, never by offset in the file. A mask is a reference between
parts and is covered without exception. Identifiers are unique within a puppet and are never
reused for a different entity after a deletion.

**The puppet's identifier identifies the puppet, not the file, and a copy keeps it.** A scene
uses the path to find the file and the identifier to confirm it is the right puppet, so two
files sharing one — a backup, say — is normal and not a defect. A filesystem copy cannot be
intercepted, so a rule forbidding duplicates would be unenforceable. Studio's *save as* issues a
new identifier, because that is the deliberate act of making a different puppet.

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
  the build does not understand is carried through to the next save byte-for-byte — including
  unmarked material, which the declared version licenses, as
  [below](#unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided) says. Silently
  dropping it destroys the work of someone on a newer build, and that loss is invisible until
  the file reaches them again.
- A file whose version breaks compatibility is **refused with a clear message**. It is never
  opened partially and then saved over.
- Opening never writes. The first save that upgrades a file keeps a backup of the original.
  **An upgrading save writes the file at the current version, and the backup holds the
  pre-upgrade bytes exactly.** Byte-identity across builds necessarily breaks at an upgrade;
  that is not a failure of [canonical serialisation](#the-same-puppet-saves-to-the-same-bytes-proposed).

### Unrecognised data is marked as such by whoever wrote it **[decided]**

Preservation only works if the loader can tell *"data I do not understand"* apart from *"this
file is malformed"*. So anything outside the core format carries an explicit marker naming the
extension it belongs to, and the validator requires that marker — in a file whose declared
version this build knows. A file declaring a *later* version is licensed by that declaration to
carry material this build cannot place, marked or not.

| Rule | |
|---|---|
| Extension data is namespaced and declares its origin and version | An identifier for the extension, the version it expects, and something human-readable to show a user who does not have it. **A puppet declares; it never installs** — see [invariants.md](../invariants.md) I6 |
| Extension data is data | It is structured values — never a script, a command, a path or a URL to resolve. The no-execution rule covers it without exception |
| In a file whose declared version this build knows, unmarked data that the core format does not define is **invalid**, not an extension | Otherwise a typo and an extension are indistinguishable, and the validator can reject neither |
| In a file declaring a **later** version, unmarked material this build does not recognise is accepted and preserved opaquely, exactly as marked extension data is | A core field added by a later version is unmarked, because it is core at the version that declared it. The version declaration is what licenses it. Preserved material is data and is never interpreted, which is why accepting it is safe — [I1](../invariants.md) is untouched |
| Preserved data round-trips byte-stably | It takes part in canonical serialisation below, so carrying someone's extension through a save does not make the file churn |

**Why the gate falls this way:** the governing rule above is that a build never refuses merely
because part of a file is unfamiliar. Requiring a marker on everything would reject exactly the
newer file that rule promises to open; gating on the declared version is what follows from it.
This section and the one above were previously in conflict on this point, and were reconciled
this way rather than by reversing either.

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
| Every texture, parameter and node reference resolves | A reference naming nothing is a malformed file. **Distinct from a texture whose payload is present but unreadable**, which loads with a visible placeholder ([authoring.md](authoring.md)) rather than failing the puppet  **A texture that nothing references is valid**, and there is deliberately no rule against one — a validator that invented one would reject files this format permits and the editor is required to preserve |
| Keypoints are ordered and within range | The range is the one the parameter declares |
| An animation carries at most one track per parameter | More than one is a malformed file. Two tracks driving one parameter over one timeline is the same ambiguity the one-driver rule refuses, arriving by another route |
| A track's keyframes are ordered in time, and its values within its parameter's range | The keypoint rule above, applied to the other thing that stores values against a parameter. Animation lookup seeks rather than scans, so unordered times give wrong answers rather than slow ones |
| A Bézier segment's control times lie inside the segment, and every control value is finite | Outside, the curve stops being a function of time and the solve can leave the segment. Cubism guarantees it; a stranger's writer need not, so the reader checks. The range rule above does not reach control values |
| Each of a parameter's axes declares an **axis polarity** | Bilateral, unipolar or unknown, per dimension. An unfamiliar value follows the enumerated-values row below. A **missing** polarity is invalid in a file whose declared version this build knows — *unknown* is how a file says nobody has decided, and it is a value that must be written rather than an absence that may be inferred. The distinction is what stops a guessed polarity from being indistinguishable from a declared one |
| Every parameter has exactly one neutral keypoint | None and more than one are both invalid. *Easing to neutral* ([performing.md](performing.md)) has no meaning without it, and a rule stated and never checked is a hope |
| Each node's children are of a kind its own kind may contain | Declared **per kind** ([What a puppet is](#what-a-puppet-is-proposed)): a part contains nothing, a mesh group or a composite contains any kind, a physics node contains only physics. A rule written as prose naming kinds is one a fifth kind escapes silently; a declaration is one it has to answer |
| Every parameter has at most one **base driver** | A channel mapping, a physics output, or none ([decisions.md](../decisions.md)). Two is a malformed file rather than a precedence question, which is what [evaluation.md](../evaluation.md) relies on. **A track is not a base driver** — a track adds to whatever drives its parameter, which is why a parameter may carry any number alongside its driver |
| A binding's stale mark agrees with its value counts: counts that match the mesh topology it applies to and no mark, or counts that mismatch and the mark | Staleness is the flagged outcome of [artwork changing under a rig](#adding-artwork-when-the-art-and-the-rig-disagree-proposed) — a state the rigger resolves, not a malformed file. **The derived state is authoritative and the mark must agree with it**, so a mismatch that is unmarked is invalid, and so is a match that is marked. A file from a stranger can lie; a derivation cannot. The mark stays useful as the cheap thing a loader reads first |
| Every number is finite | NaN and infinity are rejected, not clamped silently. **Finiteness is judged at the precision the runtime evaluates in**, not only at the precision the file was parsed in: a literal that is finite as a 64-bit value and infinite once narrowed is rejected, or two tools reading one file disagree about whether it is valid. **This rule does not carry the version gate the row below does**, and the difference matters: it applies to every value this build interprets as a number, in a file of any declared version, so a later version's NaN in a node transform is still rejected rather than preserved into the frame loop. What it does not reach is material preserved opaquely — in data whose shape we do not know there is nowhere to look for a number, and almost any eight bytes read as one are a NaN |
| Enumerated values are known, or explicitly marked as extensions | In a file whose declared version this build knows. A later version's unfamiliar value is preserved, under the [version gate](#unrecognised-data-is-marked-as-such-by-whoever-wrote-it-decided). **Inert is stated per field, because a build must not guess**: an unfamiliar interpolation mode makes its whole **track** contribute nothing — not the segment alone, which would pop at the boundary — and an unfamiliar end behaviour means the animation never starts. A trigger's event name is not an enumerated field, so an unrecognised one never fires at any version, as above |

Every size and count the loader trusts has a **load limit**, declared in one place, checked
**before allocating** — decompressed payload size, texture dimensions, the count of every kind
the file stores (vertices, nodes with physics nodes among them, parameters, animations, tracks
and keyframes), tree depth, string length, total load memory. **The enumeration is the format's,
not this list's**: a kind added later with no limit is the failure this rule exists to prevent,
so what binds is that every stored collection has one. **They bind every entry in the file** — one nothing
references, and one this build only carries without parsing — because data nothing renders is exactly where
a limit evasion would be put, and it would have no visible effect until it had already been allocated.
Exceeding one fails the load with a readable error in bounded time and memory. These limits are a defence against a corrupt or
hostile file and are multiples of the stress puppet in
[platform.md](platform.md#the-reference-puppet-and-the-stress-puppet-proposed); they are **not** authoring
limits, and they must not reappear as a choice presented to the person rigging.

**The multiple is 4× the stress workload on each dimension, except where a limit is set from
memory rather than from the workload** — the animation keyframe limits, which
[a limit stops a hostile file](#a-limit-stops-a-hostile-file-a-budget-describes-what-is-tuned-for-decided)
explains. The multiple is provisional, to be revised the first time a legitimate file is refused.

**The memory bound is assertable and the time bound is not.** Peak resident bytes during a
failed load stay below the declared payload limit, which is exact. For time there is no
reference machine, so a wall-clock figure would gate nothing; the honest form is that the work
is linear in the declared size, with no unbounded loop. This is the same split between an
asserted half and a recorded half that
[platform.md](platform.md#how-this-is-verified-proposed) makes for the frame budget.

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

**Preservation and canonical ordering are about different things and do not conflict.**
Preservation concerns *contents*: the bytes of preserved material are written back verbatim.
Ordering concerns *position*: where that material sits in the file is set by the canonical
ordering, keyed on its marker — an extension's namespace identifier or, for unknown core
material, its field identifier — which a build reads without understanding the contents. The
same puppet therefore saves to the same bytes whatever order the preserved material arrived in.

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

**A stale deformation is skipped in rendering.** The part renders at its rest geometry for that
binding, as though the deformation were absent. Other, non-stale bindings on the same node are
unaffected. **Because:** it is predictable and cheap, and the rigger already has the flag telling
them why the part looks unposed.

| Case | Behaviour |
|---|---|
| Art re-exported and re-imported after an edit | Matched to existing parts by stored import provenance rather than by name alone. Art that no longer appears leaves its part in place, flagged as having lost its source |
| Replacement art has different dimensions | Remapped. Deformations are offsets in the texture's coordinate space, so a dimension change is a defined scaling of that space |
| Replacement art matches one of several parts sharing a texture | **Split.** The matched part gets a new texture entity; every other part keeps the original, with its pixels, its deformations and its evaluated geometry unchanged. The result names both halves — which part changed and which did not |
| A mesh is regenerated or edited by hand | Retained and flagged. Topology changed, so there is no correspondence to carry offsets across and any remap would be a guess at the rigger's intent |

Deleting a rigger's deformation work because a texture changed size is the worst thing this
application could do, which is why the fallback is retention rather than a best-effort remap.

What follows, as constraints rather than design:

- **A deformation can outlive the mesh it was authored against.** A binding carries enough to
  tell whether it still matches, and a mismatch is a flag rather than the malformed-file case
  below. Staleness is mechanical, derived from the mismatch and never an annotation a rigger
  writes; the stored mark must agree with the derivation, as the validity rules say.
- **Stale deformation data round-trips**, like any other data a build carries but does not act
  on.
- **Import provenance is stored per part**, which [One file](#one-file-decided) already
  requires. It is a **stable identifier taken from the source document** — a PSD layer
  identifier where the format provides one — together with a **content hash of the imported
  pixels**. Matching prefers the source identifier, falls back to the content hash, then to the
  name. No path, username or machine name is stored, and nothing in it is ever resolved
  ([I1](../invariants.md)). **A part created by copying carries none**, because a copy has no source
  document — which is also what makes *matches one of several parts sharing a texture*, above, well
  defined rather than ambiguous between two parts claiming the same layer.

### Deletion is report, then a confirmed cascade **[decided]**

Deleting a **node** that holds a subtree, or a **parameter** that bindings reference, first
reports everything affected; the user confirms; the dependent entities go with it. **A texture
is not among the things that can be deleted** — see [A texture's life is bound to the parts that
reference it](#a-textures-life-is-bound-to-the-parts-that-reference-it-decided). What is refused is the *quiet* cascade, not the cascade — a
dangling reference is a malformed file, and one validator gates save, so a report alone cannot
be the end of it. The operation is undoable like any other ([authoring.md](authoring.md)), which
is what makes a confirmed cascade acceptable rather than dangerous.

**The cascade is transitive, and it walks dependents rather than referents.** One level does not
satisfy the validator: deleting a mesh group removes the parts beneath it, and a **binding** that
deformed one of those parts cannot exist without it either — so stopping after the first level
leaves exactly the dangling reference the cascade exists to avoid. The set is closed — but in
one direction only.

**The direction is what stops it taking the puppet.** The closure follows *what depends on the
deleted entity*, never *what the deleted entity reaches*. Containment is a stored reference like
any other, so a closure that walked every reference would go from a part to the mesh group
holding it and from there to the root: deleting one part would delete everything. **A
container is not a dependent of what it contains** — a mesh group with one fewer child is still
a mesh group, and no closure walks upward into the root. The root can still be reached the other
way, by a constitutive chain running down to it in a puppet small enough that the root is also a
part; that case is refused rather than excepted, below.

**Downward, containment is constitutive: removing a node removes the subtree under it.** A child
cannot exist without a parent to hang from, so the closure continues down. A **mesh group** whose
every child was deleted individually goes with them only if it is itself a dependent — it is not,
so it survives empty, by the rule above.
Where the closure does run down is from a node deleted directly, or reached as a dependent: its
children go, and theirs.

**Re-parenting children instead is a different operation and gets a different name.** Dissolving
a group keeps its children and removes only the group; deleting it removes both. Conflating them
is how a rigger loses a branch they meant to keep, and the report names the whole subtree before
either happens.

The report names the whole set before anything changes.

- Deleting a **node** takes the subtree under it, and the **bindings** on every node in that
  subtree. Textures the deleted parts were the last to reference are collected in the same step,
  and the report names them.
- Deleting a **parameter** removes what cannot exist without it: its **bindings**, the **channel
  mappings** that feed it, and in every **animation** that drove it, the **track** that did so.
  A track is authored keyframes, so **the report names each one by animation** rather than saying
  the animation survives. The animation itself **survives**, driving the rest, and survives with
  no tracks left. A **physics node** that drove the parameter also survives, now driving nothing
  and named in the report, because a physics node is whole without a parameter. The nodes those
  bindings drove survive untouched too — a node is not defined by being driven.

**A dependent is removed only where the reference is constitutive; otherwise it just loses the
reference.** Both leave no dangling reference, which is all the validator asks, so the rule is
chosen on what it costs the rigger rather than on validity.

| The reference is | Then | Examples |
|---|---|---|
| **Constitutive** — the dependent cannot exist without it | The dependent is removed, and the closure continues through it | A **part** without its **texture**; a **binding** or a **channel mapping** without its **parameter**; a **track** without the parameter it drives |
| **Incidental** — the dependent is whole without it | The reference drops and the dependent stays. The closure stops here, because nothing was removed | A part that was **masked** by a removed part, which survives unmasked; a **physics node** whose driven parameter went, which survives as a link in its chain |

**A part without its texture stays declared constitutive although nothing can now delete a
texture.** The declaration is what the closure is computed from, and the kind is still exercised
from the other end — by a re-import split, and by the collection rule. A kind that stops being
reachable from the interface does not stop being a kind.

**Which kind each reference is belongs to the format**, declared per reference kind rather than
decided per deletion. That is what keeps the rule checkable, and what stops a new entity kind
arriving with no stated answer.

**No edit produces a puppet that would not load.** The result is computed, validated against the
integrity rules *and* the load limits, and only then is anything offered to the user or allowed
to replace what is on screen. A refusal says which rule would break.

This is stated over **every** edit rather than over deletion alone, because deletion is not the
only operation that can reach an invalid result. A paste can carry a near-limit puppet past a
load limit, so that Studio would write a file it then refuses to open. A mirror computes a
reflection, and a reflection of a finite coordinate about a distant axis can overflow to
infinity, which *Every number is finite* rejects — so it is refused, never silently clamped. A
mirror that would move a keypoint outside its parameter's declared range is refused on the same
ground.

The smallest case is still the generating one: in a single-part puppet the part *is* the root, so
deleting it would leave a tree the validity rules reject — and that is a beginner's first import,
not a corner. Stating it as *refuse what would not validate* rather than as an exception for the
root means it also catches the same shape for entity kinds and operations nobody has thought of
yet.

**The incidental case is the one that justifies the whole distinction.** Destroying a part
because the part masking it was deleted, or destroying an animation because one of the
parameters it drives went, would lose authored work that nothing requires us to lose.

**A container left empty still survives.** An animation whose only track went — because the
parameter that track drove was deleted — remains, driving nothing. Removing it would be a
deletion the user never confirmed, which is the quiet cascade this whole rule refuses, and an
empty animation is visible, so the rigger can remove it themselves.

**An animation is a container here, not a dependent.** The reference to the parameter belonged to
the **track**, and the track was constitutive and went with it; the animation never held a
reference to drop. That is the same asymmetry a mesh group has with its children, and it is why
the rule takes no exception for a container becoming empty — every such exception returns once
per entity kind.

**The channel mapping is the case to be careful with.** Live writes mappings into a puppet file,
so a deletion in the editor discards work the player wrote. That is a report the user must see
in those terms, not a line item among others.

**Stating it as a closure rather than a list** is what keeps the rule true as entity kinds are
added, instead of needing a new clause each time something comes to reference a parameter.

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
- **There is no separate authoring payload, and the player loads the whole structure.** Data
  only Studio acts on — import provenance, the mirror axes, axis polarity — sits beside
  everything else, and the player ignores what it cannot use. **Because** that set is an import
  identifier and a content hash per part, plus two axes: tens of kilobytes against a stress
  workload budgeted at hundreds of megabytes of resident texture
  ([platform.md](platform.md#the-reference-puppet-and-the-stress-puppet-proposed)). Splitting it
  into its own payload saves the player a rounding error and costs a category of its own — data
  that is recognised but deliberately unread, a byte-carry path through every save Live makes,
  and a validator that answers differently in each application. The earlier rule that the format
  declares each kind runtime or authoring-only is **withdrawn**;
  [decisions.md](../decisions.md) carries why, and what would reopen it.

What a puppet stores is the **cut-up parts**, which are what rendering needs anyway, together
with enough import provenance to match returning artwork on re-import, as
[above](#adding-artwork-when-the-art-and-the-rig-disagree-proposed) describes. **The layered source document is not embedded.** It is the artist's working file
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


