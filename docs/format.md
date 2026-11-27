# The puppet file

What is in a puppet, how it is encoded, and what a reader must do with it. The requirements this
satisfies are in [requirements/puppet.md](requirements/puppet.md); this is the shape that
satisfies them.

A scene is a separate, much thinner file and is specified in [scene.md](scene.md).

## The container is an archive with a manifest **[proposed]**

A puppet is a zip archive. One entry carries the format version, one carries the whole
structure as JSON, and textures are separate entries.

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

**Still to be designed here**, as a consequence and not settled by this table: what a physics
node stores — its local offset, its simulation constants, and an optional reference to the
parameter it drives — and the per-kind containment declaration that
[puppet.md](requirements/puppet.md#what-a-puppet-is-proposed) now requires, which is what keeps
*a physics node contains only physics* a row in a table rather than a special case in the
validator.

What lost, and why, is in [decisions.md](decisions.md).

### The version is readable on its own **[proposed]**

`rigel2d.json` exists so that deciding whether to refuse a file costs one small bounded read,
before any load limit is at risk and before the manifest is parsed. It carries a major and a
minor number and nothing a reader needs to interpret.

| | |
|---|---|
| Major differs | **Refuse**, with a message naming the version. Never open partially |
| Minor is higher than this build knows | **Open, warn, stay editable.** Everything unrecognised is preserved — see below |
| Minor is lower or equal | Open normally |

This is the whole of [the compatibility
rule](requirements/puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest-decided)
expressed as two numbers. A build must be able to render and save a structure it has no code
for, so *can I render this* and *can I save this* are one question here and not two.

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

The loader therefore keeps the parsed source alongside its typed structures rather than
discarding it. That is a cost paid on every load, and it is the mechanism the whole
preserve-what-you-do-not-understand rule rests on.

## Canonical serialisation, and where zip fights it **[proposed]**

The same puppet must save to the same bytes. JSON is the easy half: keys sorted, no insignificant
whitespace, a fixed number format, UTF-8.

**The archive is the half that goes wrong**, because a zip writer's defaults are all
non-canonical. Every one of these must be pinned, and a writer that forgets one produces a file
that churns in version control for no reason:

- entry order fixed, sorted by path;
- modification times fixed to a constant, never the clock;
- no directory entries;
- compression method and level fixed per entry kind;
- no extra fields, and no external attributes carrying the writing machine's umask;
- file names UTF-8, with no platform path separators.

The last two are also how a machine's identity leaks into a delivered file, which
[the requirement](requirements/puppet.md#the-same-puppet-saves-to-the-same-bytes-proposed) rules out for
its own reasons.

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

## Validation **[proposed]**

The same validator gates loading and saving, and it must be runnable against the specification
without our code — a committed JSON Schema for what a schema can express, plus the structural
rules it cannot:

- identifiers unique, and none above the high-water mark;
- the tree acyclic with one root;
- every reference resolves, or is the reserved zero;
- keypoints ordered and within their parameter's range;
- **every parameter carrying at most one base driver** — a channel mapping, a physics output, or
  none. Two is the validation failure [evaluation.md](evaluation.md) relies on;
- binding value counts matching their mesh, or marked stale;
- every number finite;
- enumerated values known, or under `x`.

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
