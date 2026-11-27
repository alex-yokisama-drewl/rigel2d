# Requirements — the editor

What Studio must do. What a puppet *is* belongs to [puppet.md](puppet.md); this is about the
work of authoring one.

**The editor is the larger of the two applications and this document is incomplete.** Sections
marked as open are known gaps, not oversights.

## The problem this application exists to solve **[decided]**

Inochi Creator's feature set is adequate; its interaction is not. The gap this project targets
is between *the tool can do this* and *a beginner can do this without being told how*.

That makes interaction quality a requirement here rather than a polish item, and it sets the
standard for arguments about scope: **a feature that cannot be made intuitive is better left
out.**

## Bringing artwork in **[proposed]**

- Layered PSD import, mapping PSD layers onto parts, is the primary path.
- Individual image files import directly.
- **Merging into an existing puppet accepts more than one thing at a time** — several images, a
  PSD, or another puppet. Inochi Creator's one-at-a-time merge is a specific friction worth
  naming: the work it makes tedious is routine.
- **Artwork nothing draws can be removed deliberately**, as one command over unreferenced textures
  only, reporting what it will take and how large it is. It is the one operation whose subject is a
  texture, and it exists because a rigger who receives a puppet carrying unused artwork otherwise has
  no way to remove it — see [puppet.md](puppet.md#a-textures-life-is-bound-to-the-parts-that-reference-it-decided).
- **Merging another puppet copies what it takes.** Parts arrive with fresh identifiers and no
  tie to the puppet they came from, so the result stays self-contained. A live reference would
  mean a stored reference resolving outside the file it arrived in, which
  [One file](puppet.md#one-file-decided) and [I1](../invariants.md) both rule out. The cost is that
  updating shared artwork means merging it again wherever it was used.

## Editing the tree **[decided]**

- **Anything in the tree can be copied and pasted**, including a node with its subtree, its
  bindings and its deformations. Its absence in Inochi Creator is the single most-cited
  friction in this project's motivating complaints.
- **Symmetry is a first-class operation.** Mirroring a deformation, a mesh, or a subtree from
  one side of a character to the other is routine work and must be one action, not a manual
  re-authoring.

**Symmetry is an operation, not stored structure.** Mirroring produces independent geometry
that is edited on its own thereafter; nothing in a puppet records that two subtrees are each
other's mirror. A persistent link would be puppet data needing its own identity, its own
validation and a defined behaviour when the two sides are deliberately made to differ — and
riggers make them differ routinely, so the link would be fighting the common case. The cost is
that changing one side means mirroring again.

**A part's texture flip is not a link either.** It is stored state on one part, saying its
texture is sampled reversed, and it is what lets a pair of eyes share one drawing
([puppet.md](puppet.md#what-a-puppet-is-proposed)). It records nothing about any other part.

### Copy takes everything; paste chooses what lands **[decided]**

A rigger does not know which pieces of a node they want until they know where it is going. So
copy is one command over a whole node, and **paste** is where the choice is made:

| Paste | Lands | The case it is for |
|---|---|---|
| **Whole node** | The subtree, its meshes, its bindings and its deformations | Duplicating a finished piece of a rig |
| **Mesh only** | The mesh, onto an existing part, replacing that part's mesh | Heterochromia: two eyes, two drawings, one mesh |
| **Rigging only** | Bindings and deformations, onto an existing node | One eye is rigged, the other's art is already placed |

Each is available plain or **mirrored**, and each is one undo step.

**Mesh-only paste does not carry the texture reference** — carrying it would make the operation a
whole-node paste. The target keeps its own texture, and the mesh is scaled into that texture's
coordinate space by the same defined rule replacement art of different dimensions already uses
([puppet.md](puppet.md#adding-artwork-when-the-art-and-the-rig-disagree-proposed)). This is why
Studio needs no operation that repoints a part at a different texture: the case that wants one is
this paste, where the coordinate-space question is answered instead of hidden.

**Pasting rigging creates bindings, never parameters.** A binding maps one parameter to one node;
pasting rigging onto the other eye adds bindings, and the parameters they name already exist. A
parameter that drives five other nodes is untouched by gaining a sixth binding. Everything below
follows from that, and it is the reason the operation composes with a rig in which one parameter
drives many nodes.

### Which parameters a paste shares, and which it duplicates **[decided]**

Pasted bindings can name the **existing** parameters, so both sides move together, or
**duplicates** of them, so the sides move independently. Both are wanted and neither is right for
a whole operation at once: a mirrored eye needs its own parameters or it can never wink, and a
mirrored earring needs the *head's* tilt parameter or it stops following the head.

**So the choice is made per parameter, by whether that parameter reaches outside what is being
pasted.** A parameter used only by nodes inside the pasted set is duplicated. A parameter also
used by nodes outside it is shared.

| Mirroring | Parameter | Reaches outside? | |
|---|---|---|---|
| An eye | `EyeLOpen` | No — only the eye uses it | **Duplicated**, so the pair can blink separately |
| An earring | `HeadTiltX` | Yes — the head and the hair use it too | **Shared**, so the earring still follows the head |
| An earring | `EarringSway` | No | **Duplicated** |

**Because** it gets both cases right with nothing to choose and nothing to undo, and because the
question it asks — *is this parameter this subtree's own, or the puppet's?* — is the question the
rigger would have been answering by hand. The rigger can still override it; what they do not have
to do is decide it on every paste.

**A duplicated parameter carries its physics, and neither channel mappings nor animations.** Physics
is rigging and belongs with it. A channel mapping is data Live wrote, naming a channel; an
animation is authored work about the parameter it was written against. Carrying either onto an
axis that did not exist a moment ago would be a guess at which channel should feed it, or at
whether a timeline meant both sides — so a duplicate that carried no physics arrives **with no
driver at all**, and **the result names every parameter in that state.**

*Driver* is [evaluation.md](../evaluation.md)'s term and it is exclusive: a parameter has exactly
one, a channel mapping or physics, and two is a validation failure. So the reported set is the
duplicates that have neither — a duplicate that carried its physics has its driver and is not
reported, because it will move on stream. A puppet that moves in Studio and is dead on stream is
the failure this report exists to prevent, and a report that named working parameters too would
be read past.

### Mirroring reads a parameter and never writes one **[decided]**

Mirroring a **binding** reflects that binding's own stored values, and where the parameter's axis
is bilateral, reverses which keypoint holds which value. **The parameter itself is never
modified.** Mirroring the left eye therefore cannot change how any other node bound to the same
parameter behaves.

**This is stated as a rule because it is reliably got wrong.** The natural reading of "flip the
rigged eye" is that the axis is reversed, and an axis belongs to a parameter that other nodes
share — so the operation looks as though it must reach them. It does not, because what is stored
per node is the binding, and the reversal is a remapping of that binding's values across the
parameter's keypoints.

- **Axis polarity decides whether reversal applies at all.** A bilateral axis reverses; a unipolar
  one does not, because reversing *eye open* would open the eye when it should shut.
- **A parameter whose polarity is unknown reverses nothing, and the result says so**, naming each
  parameter left alone so the rigger can set its polarity and mirror again. This is the case for a
  puppet from a third-party tool. Guessing would be wrong for every head-turn parameter in it, and
  invisible.
- **A mirror toggles the texture flip on the axis it mirrored**, for every part it reflects. A
  mirrored mesh over unreversed artwork puts the highlight on the wrong side of the second eye,
  which is the case the whole sharing arrangement exists to serve — so reflecting a part reflects
  how it samples its texture, and nothing has to be flipped by hand afterwards. Toggling twice
  returns the original, which is what makes the rule below consistent with this one.
- **A mirror preserves vertex index correspondence.** The reflected mesh has the same vertices in
  the same order, so deformations transfer onto it by position in the list. A mirror that
  renumbered them would scramble every offset it carried across, silently and unrecoverably.
- **Mirroring a node in place twice returns the original**, byte for byte — the reflection, the
  reversed axes and the toggled texture flips all undo each other. This is about mirroring *in
  place*, not about a mirrored paste, which creates a second node and has nothing to return to.
- Inochi Creator's mirror-within-one-parameter and "flip the whole rigged eye" are the same
  operation at two scopes — one binding, or every binding in a subtree.

### Mirror axes **[decided]**

A puppet carries two, defaulting to its centre: the **left–right mirror axis**, which swaps left
and right, and the **top–bottom mirror axis**, which swaps top and bottom. They are named for what
they swap rather than for the orientation of the line, because those two namings invert each other
([vocabulary.md](../vocabulary.md)).

The default is **derived**, not stored, so it cannot go stale as the puppet grows; an axis the
rigger moves is stored from then on, and moving one is an undoable edit that changes no existing
node. Every mirrored operation's result names the axis it used. Anything the two axes do not cover
is a mirror followed by a move, which is the ordinary cost of not having a third mechanism.

### Pasting into a different puppet **[decided]**

Sharing a parameter has no meaning across files, so a paste into another puppet **duplicates,
always**, and says so rather than offering a mode it cannot honour.

**Artwork arriving by paste deduplicates against the destination**, by content hash, exactly as an
import does. Pasting a part into a puppet that already holds the same drawing yields one texture,
not two — otherwise the cheapest route to duplicating artwork would be the one the rigger reaches
for most.

**A mirrored paste uses the destination's mirror axes**, because the axes belong to the puppet being
written and not to the one the fragment came from.

**Every identifier is reissued from the destination.** Identifiers are unique within a puppet and
nothing more ([I3](../invariants.md)), so two unrelated puppets using the same small numbers is the
normal case. A pasted reference that resolved by number in the destination would be a wrong
reference in a file that never looked malformed. References from the pasted set to something that
was not copied follow the kinds the format declares: constitutive, and the paste is refused naming
what is missing; incidental, and the reference drops.

### Every compound edit reports what it did **[decided]**

Paste, mirror, import and the collection of artwork nothing draws each return a structured result
naming what was **created, shared, flagged, collected and left unchanged**. This is what makes the
rest of this section checkable before either application's interface exists, and it is what the
person is shown.

**An operation that changes something the rigger did not point at must name it.** That is the
common shape behind the failures this section is built to avoid: a mirror that reverses an axis,
a paste that couples two sides through one parameter, a re-import that splits artwork two parts
were sharing, a mesh paste that leaves deformations stale. Each is correct behaviour and each is
invisible, and a beginner finds out on stream.

## Meshing **[proposed]**

Automatic mesh generation from artwork, and hand editing of the result. Both are needed; neither
is sufficient alone.

## The timeline **[proposed]**

Animation editing must support **zooming and scrolling the time axis**, at any length of
animation. Inochi Creator's fixed timeline scale is a named frustration, and it has a second-
order effect worth recording: it caps the length of animation that can be authored at all, which
is what forced the workaround described under randomness below.

## Randomness **[open]**

Real-time randomised motion — idle variation that does not repeat — is wanted and **not
committed to**.

Recorded because the workaround names the requirement precisely: with no real-time randomness
and a timeline that cannot be navigated at length, the only route to varied idle motion was an
external script generating a very long pre-randomised animation, regenerated between streams.
Either half of that problem being fixed reduces the need for the other.

## Work is never lost **[decided]**

The editor holds hours or weeks of a person's labour in a file. Everything in this section
follows from that, and none of it is optional.

- **Every operation that changes a puppet is undoable**, including paste, mirror, import and
  merge. An import or a merge undoes as one step, not as the many steps it was made of. Undo
  followed by redo restores the puppet exactly.
- **Compound operations are atomic.** A merge, import, paste or mirror either completes or
  leaves the puppet exactly as it was. A failed import leaves no orphaned textures behind.
- **Saving is atomic** — written beside the original and renamed over it, so that a crash, a
  full disk or a power loss at any moment leaves the previous file intact and loses nothing
  from memory.
- **Unsaved work is never lost silently**, whether by closing, opening something else, or
  crashing. Whether that means a prompt, a recoverable autosave journal, or both is `[open]`;
  that autosave never overwrites the puppet file is not.

## Failure says what happened and changes nothing **[decided]**

- A malformed, truncated, empty or unsupported PSD produces a message naming the file, the
  cause and a next action, and **leaves the puppet unchanged**.
- A PSD whose PSD layers are partly unusable imports the usable ones and **lists what it skipped**.
  Silently dropping them is a data-loss bug from the person's side of the screen.
- **A multi-item merge reports an outcome per item.** One bad file does not cancel the rest or
  hide what happened to them — which is the whole value of merging many at once.
- A puppet that will not load says where and why, in terms a person can act on rather than a
  parser's own words. A puppet with a missing texture shows a visible placeholder and says so.
- When a puppet is partly not understood — which the format permits, see
  [puppet.md](puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest-decided) — **the
  person is told what is unrecognised and what it belongs to.** Nothing is ignored: unrecognised
  data is carried through to the next save untouched.

## The person can always tell what an edit will do **[proposed]**

The hardest-won requirement in this document, because it is the trap every beginner falls into
and it is invisible in a feature list: **editing a deformation while a parameter sits between
keypoints must have defined and visible behaviour.** What is being edited — the rest pose, or a
keypoint's value — is never ambiguous, and which node, parameter and keypoint is active is
always on screen.

An operation that is unavailable says why rather than failing to respond.

## First run **[open]**

Opening the editor with no puppet shows a way to begin, not an empty workspace. A first-time
user can get from a PSD to a puppet that moves in the viewport without outside help; what form
that takes — a bundled sample puppet, a guided path, in-context hints — is undecided.

## Mouse and keyboard are the whole interaction model **[decided]**

Every operation in Studio is **convenient** with a mouse and a keyboard. Not merely possible —
convenient, meaning no operation depends on precision, pressure, hover or a gesture that a mouse
cannot comfortably produce.

A pen tablet is not a target device. Tablets still work, because the operating system presents
one as a pointer, but nothing is designed for pressure, tilt, hover or barrel buttons, and no
interaction assumes them.

**Because:** this is the audience requirement, not a simplification. Beginners and smaller
streamers are the people this project is for, and a tool whose rigging workflow is merely
tolerable without a tablet quietly excludes them. Holding the baseline at mouse and keyboard
also keeps one interaction model to design, document and test rather than two that drift.

**Tablet-specific interaction is a candidate for the plugin API** once it exists — it is the
first concrete example of the "build it in or not at all" problem that deferring the API keeps
open rather than closing. See below.

## Extensibility: plugins are intended, the API is deferred **[decided]**

**No scripting inside a puppet, ever.** No field in a puppet, a scene or a source is evaluated
as code. That is permanent, and it is not what this section defers.

**A plugin API is intended, and deliberately not designed yet.** Studio and Live expect to load
third-party code eventually — a custom deformer, a node kind, an import format, an interface
panel are all things open seams cannot reach, and a tool like this is better for having them.
What is deferred is the API surface, because an API is a compatibility promise over internals,
and this project's internals are the part it most wants freedom to change. The deformation model
is precisely where a premature API would bite hardest.

So the rule for now is **build so that plugins stay possible, and build nothing for them yet.**
The test for whether a piece of groundwork belongs in the first release:

> Is it impossible, or very expensive, to retrofit once files are in the wild and users have
> habits?

| In scope now, because retrofitting is painful | |
|---|---|
| Marked extension data in the format, preserved on save | Retrofitting means every file already written lacks the marker, and older builds have already destroyed what they did not recognise. See [puppet.md](puppet.md) |
| A stable identifier on every entity | A plugin has to name what it acts on. Already `[decided]` for unrelated reasons, which is the sign it is the right shape |
| An enumerable operation set | Already required so that undo can be exhaustive. The same shape is what a plugin API would drive, so the two reinforce each other rather than compete |
| Installation is never a file | Below. A security line, not a convenience |

**Out of scope now:** the API surface itself, any ABI or stability promise, a sandbox or
capability model, and — the tempting one — abstraction layers with exactly one implementation
behind them, written "for the plugins". That is the failure mode this deferral exists to avoid,
and it costs more than the retrofit it is trying to prevent.

**The line that holds whenever plugins arrive: a plugin never arrives inside a puppet, a scene
or a source.** Installing one is an explicit act by the user, out of band from file sharing.
This is what keeps [I1](../invariants.md) true of our own code, which is where it does its work.
It is **not** a claim that nothing supplied by a file can ever execute once plugins exist — a
plugin can store code in extension data and run it itself, and that residual risk is accepted
(`invariants.md` I1). What a puppet able to *carry* a plugin would destroy is the distinction
itself: puppets are traded between strangers, installed software is not, and collapsing the two
takes most of the threat model with it.

**Until plugins exist, extensibility is open seams:** a file format third-party tools read and
write ([puppet.md](puppet.md)), and an input protocol anything that can open a socket can speak
([performing.md](performing.md)). That remains the right answer for a great deal, and a plugin
API never replaces it.

## Rolling backups **[open]**

Whether Studio keeps previous saves alongside the current one, and how many, is undecided. It
interacts with the crash-recovery question above and with
[I7](../invariants.md#i7-a-failed-save-never-damages-the-previous-file), which protects the
previous file during a save but says nothing about keeping older ones.

