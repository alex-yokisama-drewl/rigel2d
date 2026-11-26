# Requirements — the editor

What the application that builds puppets must do. The puppet model itself is in
[puppet.md](puppet.md); this is about the work of authoring one.

**The editor is the larger of the two applications and this document is incomplete.** Sections
tagged `[open]` are known gaps, not oversights.

## The problem this application exists to solve **[decided]**

Inochi Creator's feature set is adequate; its interaction is not. The gap this project targets
is between *the tool can do this* and *a beginner can do this without being told how*.

That makes interaction quality a requirement here rather than a polish item, and it sets the
standard for arguments about scope: **a feature that cannot be made intuitive is better left
out.**

## Bringing artwork in **[proposed]**

- Layered PSD import, mapping layers onto parts, is the primary path.
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
- A PSD whose layers are partly unusable imports the usable ones and **lists what it skipped**.
  Silently dropping them is a data-loss bug from the person's side of the screen.
- **A multi-item merge reports an outcome per item.** One bad file does not cancel the rest or
  hide what happened to them — which is the whole value of merging many at once.
- A puppet that will not load says where and why, in terms a person can act on rather than a
  parser's own words. A puppet with a missing texture shows a visible placeholder and says so.
- When a puppet is partly not understood — which the format permits, see
  [puppet.md](puppet.md#the-file-is-readable-and-writable-by-tools-that-are-not-ours) — **the
  person is told what was ignored.**

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

## What the editor deliberately does not do **[decided]**

**No plugin API, and no scripting inside a puppet.** Extensibility comes from open seams
instead: a file format third-party tools can read and write ([puppet.md](puppet.md)), and an
input protocol anything that can open a socket can speak
([performing.md](performing.md)). Extending the system means writing a program that produces a
file or sends channels, not loading code into ours.

**Because:** an API surface is a permanent maintenance cost and a compatibility promise, and it
is scope against a project whose stated trade is depth over breadth. It is also what makes
*nothing a puppet, scene or source supplies is ever executed* available as an invariant — and
that single sentence is most of the threat model for software whose files are traded between
strangers.

## Deferred **[open]**

Recorded rather than left as gaps, and tracked in `docs/wip/open-questions.md`: whether the
interface is translated into more than one language, and whether a pen tablet is a target input
device — pressure, hover, and no right-click change the interaction model, and artists often
work on one.
