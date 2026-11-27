# The puppet file

What is in a puppet, how it is encoded, and what a reader must do with it. The requirements this
satisfies are in [requirements/puppet.md](requirements/puppet.md); this is the shape that
satisfies them.

A scene is a separate, much thinner file and is specified in [scene.md](scene.md).

## The container is an archive with a manifest **[proposed]**

A puppet is a zip archive. One entry carries the format version, one carries the whole
structure as JSON, and textures are separate entries. A puppet file's extension is **`.rigel2d`**.

| Entry | Holds |
|---|---|
| `rigel2d.json` | The format name and version, and nothing else |
| `puppet.json` | The node tree — parts, mesh groups, composites and physics — with parameters, bindings, animations, channel mappings, texture records, and the data only Studio acts on |
| `textures/<identifier>.png` | One entry per distinct texture, cropped to its own content |

**Because** the requirement is that a third party can inspect, change and generate a puppet with
ordinary tooling. An archive delivers that literally: `unzip` gives a reader one JSON file a text
editor opens and images any viewer displays. It also delivers the structural requirement that
plain JSON does not — binary payloads addressable without parsing the whole file.

**There is no separate authoring entry.** An earlier draft of this table gave Studio-only data a
payload of its own for the player to skip.
[puppet.md](requirements/puppet.md#one-file-decided) withdraws that and
[decisions.md](decisions.md) carries why. One structural entry is also the stronger form of the
third-party claim: a reader opens one file rather than learning which of two holds what.

What lost, and why, is in [decisions.md](decisions.md).

### The version is readable on its own **[proposed]**

`rigel2d.json` exists so that deciding whether to refuse a file costs one small bounded read,
before any load limit is at risk and before the manifest is parsed. It carries a major and a
minor number and nothing a reader needs to interpret. The current format version is **1.1**.

| | |
|---|---|
| Major differs | **Refuse**, with a message naming the version. Never open partially |
| Minor is higher than this build knows | **Open, warn, stay editable.** Everything unrecognised is preserved — see below |
| Minor is lower or equal | Open normally |

This is the whole of [the compatibility
rule](requirements/puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest-decided)
expressed as two numbers. A build must be able to render and save a structure it has no code
for, so *can I render this* and *can I save this* are one question here and not two.

### An unfamiliar value in a field this build knows **[proposed]**

The hard case, and the one that is not extension data. A later minor version may add a value to
an enumerated field the core format already defines — a physics node's `output_source`, a
parameter axis's polarity, a deformation's composition mode. The field is one this build
interprets; the value is one it has never heard of.

**Every enumerated field the format defines therefore has one more inhabitant than it lists: the
unfamiliar value, carrying the token verbatim.** It is not a parse failure and not a repair.

| | |
|---|---|
| At the **current** version | **Invalid.** A writer at this version had the whole list and used something else, so a typo and an addition are indistinguishable — the same argument that makes unmarked data outside `x` invalid |
| At a **later declared** version | **Loads.** Inert wherever it would have been interpreted, and written back verbatim |

*Inert* is what [requirements/puppet-acceptance.md](requirements/puppet-acceptance.md)'s `EXT-7`
asserts by removal: deleting the material from the file must change no evaluated geometry, no
draw order and no reported entity. So a physics node whose `output_source` is unfamiliar drives
nothing on that axis — it does not fall back to a default, because guessing is the one thing a
build with no code for a value must not do.

**Because** the alternative is a closed enum, and a closed enum forces a reader to choose between
refusing a file a later build wrote and silently substituting a value nobody asked for. The first
breaks [the compatibility rule](requirements/puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest-decided),
which requires that a structure this build has no code for still renders and saves; the second
corrupts a puppet quietly, which is worse. Carrying the token costs one variant per field.

This is the same shape as extension data under `x` and is gated the same way — on the **declared
version**, never on recognisability. What differs is only that the core format named the field.

### The upgrading save keeps the previous bytes **[proposed]**

When a build opens a file declaring a lower minor and saves it, it writes the file at the current
version and preserves the bytes it read as a sibling file in the same directory, named by
appending `.bak-<major>.<minor>` to the original file name. The suffix is the version the backup
*holds*, not the version written: `hero.rigel2d` upgraded from 1.0 becomes `hero.rigel2d` at 1.1
beside `hero.rigel2d.bak-1.0`.

**Because** an upgrade is the one save that cannot be undone by saving again — the previous build
can no longer open what replaced it — and [I7](invariants.md) makes losing the puppet worse than
a bad frame. A sibling rather than a hidden location, because the person has to be able to find
it. It is also what `FILE-2` in [puppet-acceptance.md](requirements/puppet-acceptance.md) means
when it says the directory afterwards holds the puppet "and nothing else but the version-upgrade
backup".

## Identifiers are integers, not UUIDs **[proposed]**

Every entity carries an unsigned 64-bit integer, unique within the puppet. The puppet records a
high-water mark that never decreases, which is what makes an identifier never reused after a
deletion. Identifier zero is reserved and means *no reference*.

**Because** the governing requirement is that tools which are not ours can write a valid
puppet. Emitting `1, 2, 3` needs nothing; emitting UUIDs needs a library in every tool that
touches the format. Integers also diff legibly, which matters for a format whose whole case is
that version control and third-party tooling work on it.

UUIDs are the obvious alternative; [decisions.md](decisions.md) carries why they lost.

**The trap this leaves.** Two tools editing one puppet without seeing each other's work can
both allocate from the same high-water mark and produce colliding identifiers. There is no
concurrent-editing story and no merge of divergent copies, so nothing exercises this today —
but a tool that writes a puppet must take the high-water mark from the file it read, never from
a count of what it found.

## Extension data **[proposed]**

Anything outside the core format lives under a reserved `x` key, present on any object, keyed by
the extension's own identifier:

```json
"x": {
  "com.example.wiggle": {
    "version": "1.2",
    "name": "Wiggle Deformer",
    "data": { }
  }
}
```

| Rule | Why |
|---|---|
| `name` is human-readable and is what a user without the extension is shown | The warning on open has to name something a person recognises |
| `data` is structured values only | Never a string that is executed, resolved or interpreted as a path or URL. [I1](invariants.md) |
| A key the core format does not define, outside `x`, is **invalid** | Without this a typo and an extension are indistinguishable and the validator can reject neither |
| `x` survives a save untouched, byte for byte | Including through Live, which writes puppets for channel mappings |

**A puppet that declares it carries data for a named extension does so with an ordinary `x`
entry.** The marker already carries the extension's identifier, the version it expects and a
human-readable name, and the format defines no second mechanism. [I6](invariants.md) permits a
file to declare what it needs and never to install it, so a declaration is exactly this and
nothing more.

The loader therefore keeps the parsed source alongside its typed structures rather than
discarding it. That is a cost paid on every load, and it is the mechanism the whole
preserve-what-you-do-not-understand rule rests on.

## Canonical serialisation, and where zip fights it **[proposed]**

The same puppet must save to the same bytes. JSON is the easy half: keys sorted, no insignificant
whitespace, a fixed number format, UTF-8.

**Ordering is part of the canonical form.** `nodes` is in depth-first preorder, and order among
siblings is draw order — the one thing order is allowed to decide. Every other collection is
ordered by identifier, and keypoints by position along their axis.

**A defaulted field is absent when it holds its default.** Without the rule a writer that omits a
default and one that spells it out produce different bytes for the same puppet. The one
deliberate exception is `composition_mode`, under
[an animation and its tracks](#an-animation-and-its-tracks-proposed).

**The archive is the half that goes wrong**, because a zip writer's defaults are all
non-canonical. Every one of these must be pinned, and a writer that forgets one produces a file
that churns in version control for no reason:

- entry order fixed, sorted by path;
- modification times fixed to 1980-01-01 00:00:00 — the earliest a zip date field expresses, so
  there is no question of it having come from a clock;
- no directory entries;
- compression fixed per entry kind: Deflate for JSON, Stored for PNG. A PNG is already deflated
  inside its own container, and storing it keeps the payload extractable in one step, which is
  what `TOOL-3` rests on. The Deflate level is fixed by the locked dependency set;
- no extra fields, and external attributes pinned to a fixed value, `0644`, which carries no
  umask — an entry written through a Unix version-made-by cannot have none;
- file names UTF-8, with no platform path separators.

The last two are also how a machine's identity leaks into a delivered file, which
[the requirement](requirements/puppet.md#the-same-puppet-saves-to-the-same-bytes-proposed) rules out for
its own reasons.

## The node tree **[proposed]**

Four kinds of node — **part**, **mesh group**, **composite** and **physics**. Every node carries
an identifier, a name and a local transform. What separates the kinds is what each draws and what
each may contain.

**What a kind may contain is a property of the kind, declared in one table**, and the validator
reads that table rather than carrying rules that name kinds.

| Kind | Draws | May contain |
|---|---|---|
| Part | A mesh with a texture | Nothing — a part is a leaf |
| Mesh group | Nothing of its own | Any kind |
| Composite | Its children, to an offscreen buffer | Any kind |
| Physics | Nothing | Physics only |

**Because** a containment rule spelled out in code has to be found and edited by whoever adds a
fifth kind, and whoever forgets leaves a kind that may contain anything. A table turns that
omission into a missing row, which a check can refuse. It is the same shape as the
constitutive-or-incidental declaration carried per reference kind, and exists for the same reason
([requirements/puppet.md](requirements/puppet.md#deletion-is-report-then-a-confirmed-cascade-decided)):
a kind added later arrives with a stated answer instead of inheriting a default nobody chose.

**The tree is stored as a flat `nodes` array, each node carrying `parent`, with a top-level
`root` naming one node.** **Because** a nested tree cannot express a cycle or a second root, so
the acyclicity and one-root rules in [Validation](#validation-proposed) would have nothing to
reject and no file could carry the defect. Containment becomes a reference by identifier, which
is what [I3](invariants.md) asks for anyway.

**The root is a kind that contains** — a mesh group or a composite. **Because** adding artwork to
a configured puppet must preserve its configuration
([requirements/puppet.md](requirements/puppet.md#one-file-decided)), and a puppet rooted at a part
would have to be re-rooted the moment a second part arrived. Refusing the one-node case costs a
rigger nothing and removes a re-rooting path from every import.

**Physics containing only physics is what keeps a deletion cascade off artwork.** A pendulum
chain is an ordinary subtree, so nesting supplies it with no new mechanism; and because no part
can sit beneath a physics node, no cascade that reaches one can reach a part through it.

### A part's mesh **[proposed]**

A part's mesh carries `vertices`, `uvs` and `triangles`. **`uvs` are in the texture's authored
pixel space**, not the cropped bitmap's. That is the statement about texture coordinates that
makes the paragraph on cropping in [Textures](#textures-proposed) do its work: the runtime
reconstructs the authored space from the crop rectangle and the authored dimensions, and these
coordinates are what it reconstructs it for.

## A physics node **[proposed]**

One node is one anchor and one bob; a chain is nesting, so every constant here is **per node**.
The parameterisation is inochi2d's, pruned — what was taken, what was dropped, and the evidence
for each is in
[research/2026-11-27-physics-parameterisation.md](research/2026-11-27-physics-parameterisation.md).

| Field | Meaning | Unit | Default |
|---|---|---|---|
| `length` | Rest distance from the anchor to the bob | puppet pixels | 100 |
| `gravity` | Multiplier on the puppet's gravity; zero turns it off for this node alone | ratio | 1.0 |
| `frequency` | Resonant frequency of the restoring spring — the stiffness knob | Hz | 1.0 |
| `angle_damping` | Damping of swinging motion | ratio | 0.5 |
| `length_damping` | Damping of stretch | ratio | 0.5 |
| `output` | The parameter this node drives, or the reserved zero for none | identifier | 0 |
| `output_source` | Per axis of that parameter, which simulated quantity drives it — `angle`, `length`, `x` or `y` | enum, per axis | `angle` |
| `output_scale` | Gain from that quantity onto that axis | ratio, per axis | 1.0 |

**The puppet carries one gravity**, a magnitude along puppet-down, which every node's `gravity`
multiplies. It defaults to `1.0`, matching the per-node multiplier's default, and is absent when
it holds that, by [the stored-form rule](#canonical-serialisation-and-where-zip-fights-it-proposed).
**Because** a rigger sets the world once and then says only which strands are heavy;
and because one field at zero stops every pendulum in the puppet, which is what makes a physics
problem bisectable. A direction, and wind, are what a later minor version adds if they are ever
wanted.

**The arm rests along the node's own down-axis, so a pendulum's neutral direction is simply the
node's own rotation.** There is no neutral-angle field: every node already carries a local
transform, and a rigger already has a gizmo for it. The restoring spring pulls the arm back to
that direction, gravity pulls it toward puppet-down, and where the two disagree the pendulum
rests between them.

**This is what makes a spike.** Set `gravity` to zero and the node returns exactly to the
direction it was authored at — springing when its anchor moves, settling back where it started.
Short hair, spikes, antennae, anything meant to read as stiff. Leave `gravity` at one with the
node unrotated and the same model is ordinary long hair. Two rigging cases, one model, one knob
between them, which is why neither gets a mode of its own. If decoupling the rest direction from
the node's orientation is ever actually wanted, that is a field a later minor version adds.

**There is one model, a spring pendulum.** inochi2d also stores a rigid-pendulum switch, and its
rigid model ignores `frequency` and `length_damping` — a control that silently disables two
others, which is the clearest beginner trap in either incumbent's field set. A high `frequency`
approximates the rigid case closely enough.

**Both dampings are stored, although a hair rig rarely wants stretch.** Whether a rigger sees one
damping or two is Studio's question, and Studio may derive the radial one from the tangential
one. Whether the file carries both is this document's, and it carries both: removing a stored
field later is a format break, while hiding one in the editor is not.

### What the simulation drives **[proposed]**

Two outputs, and the first is what makes a chain work with no chain mechanism in the format.

- **The node's own transform is displaced** — rotated about its anchor by the bob's angle, and
  translated along the arm by the difference between the current length and `length`. A child
  physics node therefore hangs from a parent that is already swinging, which is the whole of
  chaining ([evaluation.md](evaluation.md)).
- **One parameter may be driven.** Each axis of that parameter names the quantity that drives it
  and the gain onto it. inochi2d spends an enum on the four orderings of angle, length, x and y;
  naming the source per axis covers all four, covers a one-dimensional parameter without a
  special case, and does not grow when a fifth quantity is wanted.


**Ranges are `[open]` beyond sign and finiteness.** The validator requires `length` and
`frequency` above zero, `gravity` and both dampings at or above zero, and every value finite.
Upper bounds are a question about integrator stability at the fixed timestep, and neither
incumbent declares any — a sweep over frequency, damping and length settles them, which is a
measurement due with the evaluation pass rather than more research.

**That they are open blocks nothing**, because the runtime bounds the simulation itself rather
than trusting the constants: see
[evaluation.md](evaluation.md#physics-cannot-run-away-proposed). A constant outside any range we
would have set produces an ugly rig, not a crash.

## An animation and its tracks **[proposed]**

**An animation stores its tracks as a list, each naming the parameter it drives**, and two
tracks naming one parameter is a validation failure rather than a shape the format cannot
express. The alternative — keying the collection by parameter identifier, which makes the
duplicate inexpressible and needs no rule — loses on legibility: it makes the identifier serve as
both key and reference, and a third party reading the JSON is the case this format exists to
serve.

A track carries a **`blend_duration`**. [security.md](concerns/security.md), by way of `VAL-26`,
requires a track's keyframe times, values and blend durations to be validated, and nothing
defined one until here.

**`composition_mode` is always written, even though it holds the only value the format
defines.** It is the single exception to the absent-when-default rule, and deliberate: a mode
added in a later version must not change how a puppet authored before it composes. Further modes
are [what the format does not define yet](#what-the-format-does-not-define-yet-proposed), which
owns that question.

## Textures **[proposed]**

One archive entry per distinct texture, PNG, cropped to its own content. PNG because it is
lossless, carries the alpha channel this project is built around, and is readable by everything.

**Cropping must not move anything.** A part's mesh coordinates are in the artwork's authored
space, not in the cropped bitmap's. The texture record therefore carries the crop rectangle and
the authored dimensions, and the runtime reconstructs the authored space on load. Getting this
wrong shifts every mesh built on that texture, and it would do so silently.

This is also what makes the
[dimension-change remap](requirements/puppet.md#adding-artwork-when-the-art-and-the-rig-disagree-proposed)
well-defined: a deformation is an offset in the authored space, so replacing artwork at a
different size is a scaling of a space the file already names.

How textures reach the GPU is not this document's concern and is deliberately not visible here —
see [runtime.md](runtime.md).

## A channel mapping **[proposed]**

A mapping stores the channel's name, the identifier of the parameter it feeds, and a response
curve. It may also name a source, and that field is empty in almost every puppet. See
[performing.md](requirements/performing.md#channels-have-default-names-and-those-names-are-what-travels-proposed).

**A channel is referenced by name, which is outside what [I3](invariants.md) binds rather than
an exception to it.** Everything *inside* a puppet is referenced by identifier; a channel is not
inside the puppet. It is supplied by a source the puppet has never met, on a machine the rigger
has never seen, and a shared name is the only thing the two ends can agree on in advance. That
is why the channel vocabulary is Rigel2D's own rather than a tracker's — the name *is* the
contract, and the requirement that mappings travel with the puppet rests on it entirely.

**The curve's input domain is the normalised channel, not raw values**, which is what makes a
delivered mapping behave on someone else's face. Normalisation itself is *not* in the puppet —
it is per machine and belongs to Live's configuration, for the reasons in
[performing.md](requirements/performing.md#normalisation-is-the-recipients-shaping-is-the-riggers-proposed).
A format that stored it would be storing the rigger's webcam.

Names resolve to handles when a source connects, so nothing resolves by name in the frame loop
([evaluation.md](evaluation.md)).

## Stale bindings are a state, not a failure **[proposed]**

A binding carries enough to tell whether it still matches the mesh it applies to. When it does
not, it is marked stale and kept. A stale binding loads, renders as though absent, saves
unchanged, and is reported to the rigger.

The validator treats staleness as legal and a value-count mismatch *without* the mark as
malformed. That distinction is the whole reason the mark exists: without it, retained work and
a corrupt file look identical.

**A binding carries `values`, one entry per keypoint of its parameter, and a `stale` mark written
only when set.** **Because** the value-count rule needs a count to exist before it can be
compared.

## Validation **[proposed]**

The same validator gates loading and saving, and it must be runnable against the specification
without our code — a committed JSON Schema for what a schema can express, plus the structural
rules it cannot:

- identifiers unique, and none above the high-water mark;
- the tree acyclic with one root, and that root a kind that contains;
- every node's children of a kind its parent's kind may contain, read from the table above;
- every reference resolves, or is the reserved zero;
- keypoints ordered and within their parameter's range;
- **every parameter carrying at most one base driver** — a channel mapping, a physics output, or
  none. Two is the validation failure [evaluation.md](evaluation.md) relies on;
- binding value counts matching their mesh, or marked stale;
- every number finite;
- enumerated values known, or under `x`, or unfamiliar in a file declaring a later minor version
  — see [an unfamiliar value in a field this build knows](#an-unfamiliar-value-in-a-field-this-build-knows-proposed).

A generated puppet doubles as the conformance fixture proving a third party can write a valid
one, which [platform.md](requirements/platform.md) already requires for its own reasons.

### Load limits **[proposed]**

Every size and count the loader trusts is limited, in one place, checked **before allocating**.
The limits are generous multiples of the stress puppet and are not authoring limits — the
distinction, and the reason it matters, is in
[vocabulary.md](vocabulary.md) and
[requirements/puppet.md](requirements/puppet.md#a-puppet-that-fails-its-integrity-rules-is-rejected-not-repaired-proposed).

The archive makes one of these sharper than it would otherwise be: **a zip entry declares its
decompressed size, and that declaration is attacker-controlled.** The limit is enforced against
bytes actually decompressed, with the declared size used only to refuse early. A loader that
trusts the header has a decompression bomb.

## What the format does not define yet **[proposed]**

Each of these is a settled position rather than an open question — the format defines one thing
today and the addition, if it ever comes, is a higher minor version that older builds carry
through untouched. None of them changes [evaluation order](evaluation.md).

| | Today | If it is ever wanted |
|---|---|---|
| Interpolation between keypoints | Linear, and only linear | Another mode, as a format addition |
| Composition of deformations | Summation, the single mode | Deferred in [requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed), which owns that question |
| Channel-mapping setups per puppet | One | Named profiles, per [requirements/performing.md](requirements/performing.md) |
