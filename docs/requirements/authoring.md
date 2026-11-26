# Requirements — the editor

What Studio must do. What a puppet *is* belongs to [puppet.md](puppet.md); this is about the
work of authoring one.

**The editor is the larger of the two applications and this document is incomplete.** Sections
tagged `[open]` are known gaps, not oversights.

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
- Merging another puppet implies that parts of a rig can be reused across puppets. Whether that
  reuse is a copy or a live reference is `[open]`.

## Editing the tree **[decided]**

- **Anything in the tree can be copied and pasted**, including a node with its subtree, its
  bindings and its deformations. Its absence in Inochi Creator is the single most-cited
  friction in this project's motivating complaints.
- **Symmetry is a first-class operation.** Mirroring a deformation, a mesh, or a subtree from
  one side of a character to the other is routine work and must be one action, not a manual
  re-authoring.

How symmetry is expressed — a one-off mirror operation, or a persistent link that keeps two
sides in step — is `[open]`.

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
  [puppet.md](puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest) — **the
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

## Deferred **[open]**

Recorded rather than left as gaps, and tracked in `docs/wip/open-questions.md`: whether reuse
of a part is a copy or a live reference; how the two sides of a symmetrical rig are kept in
step; what form crash recovery takes; what happens to existing deformations when replacement
art has different dimensions or a mesh is regenerated; and whether Studio keeps rolling backups
of previous saves.
