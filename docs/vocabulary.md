# Vocabulary

The project's terms. **Load-bearing: read before anything else.**

This document governs code, comments, documentation and conversation. It matters more here than
in a single-author project, because agents working from different slices of the documentation
produce incompatible work when they do not share names. A term used in a sense this document
does not sanction is a real defect, and `code-reviewer` reports it as one.

## Rules **[decided]**

- **A term is defined once, here.** No other document redefines one; they use it.
- **A casual word is never an architectural type.** If a word is used loosely in conversation,
  it must not also name a class, a table or a field — someone will eventually read the loose
  sense into the strict one.
- **A new term is settled with the user**, during `requirements` or `tech-design`, before it
  appears in code. Terms invented by an implementer do not propagate; they collide.
- **Renaming a term means renaming it everywhere**, in one change: code, documents, comments,
  tests.
- **It governs user-facing text too.** The interface says *parameter*, *binding*, *keypoint*,
  *deformation* — the same words as the code, the documentation and the community. A second,
  friendlier vocabulary for the interface would split every tutorial, bug report and forum
  answer across two sets of names, which is the incompatibility this document exists to
  prevent. The onboarding cost is paid in the interface, by explaining a term where it first
  appears, rather than by renaming it.

## Terms

<!--
Format: the term in bold, then what it is, then — where the confusion is likely — what it is
NOT. The negative half is usually the useful half.
-->

| Term | Means | Not |
|---|---|---|
| **Puppet** | A complete rig: node tree, textures, parameters, bindings, animations. The thing both applications work on | Not "model" — that word is doing other work. Not "character", which is the art |
| **Node** | One entry in a puppet's tree. Part, mesh group, composite and physics are kinds of node. **Which kinds each kind may contain is declared per kind by the format**, so a kind added later arrives with a stated answer rather than a validator exception | Not "object" and not "layer" |
| **Part** | A node that draws a mesh with a texture on it | Not the texture. A part *references* a texture, and several parts may reference one ([puppet.md](requirements/puppet.md)) |
| **Texture** | An image a part draws with. An entity in its own right, referenced rather than owned | Not a part, not a sprite, not an atlas |
| **Texture flip** | Stored state on a **part's** reference to a texture: the texture is sampled reversed along the left–right axis, the top–bottom axis, or both. What lets two parts share one drawing with one of them reversed | **Not a mirror**, which is an operation producing new independent data. A texture flip creates nothing and belongs to the part that holds it |
| **Mesh group** | A node that deforms its children with its own mesh | |
| **Composite** | A node that renders its children to a buffer so blending and masking apply to the group | |
| **Parameter** | A named one- or two-dimensional axis belonging to a puppet, with keypoints along it. The puppet's own controls | **Not an input channel.** A channel is bound *to* a parameter; they are different things and conflating them is the error this entry exists to prevent |
| **Keypoint** | A position on a parameter's axis at which binding values are stored. Values between keypoints are interpolated | Not a keyframe, which is on a timeline |
| **Neutral keypoint** | The keypoint on a parameter's axis that is its authored resting value. *Easing to neutral* means every parameter returning to its own ([performing.md](requirements/performing.md)) | Not zero, which is merely a number on the axis. Not the rest pose, which is geometry rather than a parameter position |
| **Axis polarity** | What a parameter declares about each of its axes: **bilateral**, meaning the negative half is the mirror of the positive half — head turn, body lean — **unipolar**, meaning the axis has no mirror sense, as eye-open and mouth-open do not, or **unknown**, meaning nobody has decided. Declared **per axis**, so a two-dimensional parameter may be one of each. *Unknown* is a value that must be written, never an absence to be inferred — the route in is a writer that is not ours ([puppet.md](requirements/puppet.md#what-a-puppet-is-proposed)) | Not the range, which is the numbers an axis spans. Polarity says what reversing the axis would *mean*, and a mirror reads it without ever writing it |
| **Binding** | A mapping from a parameter to something that changes with it — a node transform, or per-vertex offsets | |
| **Deformation** | Per-vertex offsets applied to a mesh | Not a transform, which moves a whole node |
| **Rest pose** | A puppet's geometry with no deformation applied: the meshes as authored. What deformations are offsets *from* | Not the neutral keypoint, above. One is geometry, the other a position on an axis, and editing at the wrong one is the trap [authoring.md](requirements/authoring.md) names |
| **Mirror** | The operation that produces new, independent data reflected across a mirror axis. Applied to a mesh, to a binding, or to a whole subtree containing both. Mirroring a **binding** reflects *that binding's* stored values and, where the axis is bilateral, reverses which keypoint holds which value | **A mirror never writes a parameter.** It reads axis polarity and rewrites bindings, so mirroring one node cannot change how any other node bound to the same parameter behaves. Not a texture flip, which is stored state rather than an operation |
| **Mirror axis** | A line in puppet space that a mirror reflects across. A puppet carries two — the **left–right mirror axis**, which swaps left and right, and the **top–bottom mirror axis**, which swaps top and bottom. Authoring-only data, defaulting to the puppet's centre | Named for **what it swaps**, never for whether the line itself is vertical or horizontal. Those two namings invert each other, and that inversion is what this entry exists to prevent |
| **Stale** | Of a binding or a deformation: authored data kept although the mesh it applies to no longer matches it. A flagged state the rigger resolves at their own pace | **Not invalid.** A stale binding loads, renders and saves; a malformed one does not ([puppet.md](requirements/puppet.md)) |
| **Animation** | Parameter values driven over time, on a timeline. Composed of **tracks** | |
| **Track** | The part of an animation that drives exactly one parameter over time. An animation is a set of tracks, at most one per parameter | **Not a channel**, which arrives from a source. Not a **response curve**, which shapes a channel onto an axis. Not a *lane*, which is inochi2d's word for the same thing |
| **Physics** | A kind of node. Simulated motion anchored in the tree, which reads its own accumulated transform from the previous frame, displaces that node's own transform by the result, and may drive one parameter. **Contains only physics**, which is what makes a pendulum chain a subtree | Not something that lives beside the node tree — it is in it. **Not one frame behind per segment**: a chain of any length lags by one frame, because each node reads a previous frame that already carried its parent's displacement |
| **Anchor** | The point a physics node's simulation hangs from: that node's own origin, after its ancestors' transforms have been applied. What the node reads from the previous frame | Not its parent node. A physics node's anchor is its own position, which is why it needs no reference to anything to find it |
| **Bob** | The simulated mass at the far end of a physics node's arm. Its offset from the anchor is what displaces the node's transform, and what may drive a parameter | |
| **Scene** | Live's saved document: a background, which puppets and images, and where each sits | Not a puppet, and deliberately thin. It holds placement, not behaviour — a scene references puppets, never contains one. **`[open]`:** whether that line overstates it, since the scene already decides what a dropped source means ([performing.md](requirements/performing.md#losing-a-source-and-scene-events-proposed)) and raises the events that wiring runs on. Nothing may assume either wording until it is settled |
| **Scene event** | A named occurrence the scene raises, which a puppet's animation may listen for — *on load* and *source lost* are the two with a use today. **Named outward, like a channel**: a puppet references one by name and never by identifier, which is why [I3](invariants.md), a rule about references to what the file contains, does not apply to it. The set of names is open, so an unrecognised one never fires and is not a validation failure. It carries no payload — no source, port or address | **Not a command.** The scene says what happened and never what to do about it; what to do is the puppet's to decide. Not a **trigger**, below, which is the puppet's side of the pairing |
| **Trigger** | Puppet data, stored on the animation it starts: the **scene event** that animation waits for. It carries **no target**, because the animation that owns it is the target, and **no command**, because the command is always *start*. Nothing it carries is evaluated — no condition, no comparison, no delay expression, no string looked up at run time | **Not a command**, nor a thing that issues one: it selects an event to start on and never says what happens next. **Not a rule for stopping** — an animation started by an event that can clear is released when it clears, and a person can always stop one from the control surface. **Not a hook and not a script.** One that could name another animation would be sequencing, and one that could carry a condition would be a small language; [I1](invariants.md) forbids both, and the shape of the entry is what enforces it rather than a sentence elsewhere |
| **Source** | A feed of input arriving over a port, supplying channels. A face tracker is one kind | Not a tracker specifically. Anything that can open a socket is a source |
| **Channel** | One named value arriving from a source | Not a parameter. See above |
| **Normalisation** | Bringing a source's raw values onto a channel's specified range for *this* person, camera and tracker. Held per source in Live's configuration, applied before any mapping sees a value | **Not shaping.** It compensates for who is sitting in front of the camera, carries no authored intent, and is never puppet data |
| **Response curve** | The authored shaping a channel mapping applies, taking the normalised channel onto a parameter's axis | **Not normalisation.** It is the rigger's intent, it travels with the puppet, and it is defined over a calibrated input — never over raw tracker numbers |
| **Semantic class** | What kind of quantity a channel carries: an absolute physical measurement, a modelled activation, or a value already relative to the sender's own running baseline. Declared per channel by the protocol | Not the unit. Auto-ranging a measurement in degrees, or calibrating a baseline-relative value a second time, is the error this exists to prevent |
| **Mask** | A part that restricts where another part draws | Not a composite, which groups children for blending |
| **Chain deformer** | A sequence of linked joints deforming geometry along it, a permanent part of a rig. For hair, tails, ropes | Not a skeleton, and it does not generalise into one |
| **Identifier** | The stable value every entity in a puppet or scene carries and every stored reference names it by. Unique within a puppet, unchanged by rename, reorder or save, never reused after a deletion ([I3](invariants.md)) | Not a name, which is for people and changes. Not a position in an array, and not an offset in the file |
| **Handle** | What an identifier resolves to once, at load: the in-memory reference the frame loop uses, because nothing in it may resolve by name | **Never written to a file.** The stored form and the in-memory form are deliberately different things, which is the whole of [I3](invariants.md) |
| **Extension data** | Structured values in a puppet, scene or source that the core format does not define, carrying a marker naming the extension they belong to and the version it expects. Preserved untouched by a build that does not understand them | Never a script, a command, a path or a URL ([I1](invariants.md)). Unmarked data the core format does not define is **invalid**, not an extension |
| **Import provenance** | What a part records about where its artwork came from, so re-exported art is matched back to it rather than to its name | Not the artwork, and not a path the loader resolves — see [I1](invariants.md) |
| **Load limit** | A maximum the loader enforces on a file **before allocating**: payload bytes, texture dimensions, entity counts, tree depth, string length. A defence against a corrupt or hostile file | **Not a ceiling**, which is a counter's committed maximum in CI. Not an authoring limit either — it is a generous multiple of the stress puppet and never appears as a choice to the person rigging |
| **Channel mapping** | A puppet's assignment of a channel to one of its parameters | **Not a binding.** Both belong to the puppet, and the pair is the whole chain: a mapping feeds a parameter from a channel, a binding takes that parameter to geometry. Studio edits bindings, Live edits mappings |
| **Render surface** | The surface the player draws the puppet on, containing nothing else | Not the control surface |
| **Control surface** | The player's interface. Never captured, never part of the output | Not the render surface |
| **Gizmo** | An on-canvas manipulator in Studio's viewport — a draggable point, an axis, a bounding box | **Not a handle**, which is a resolved reference and has nothing to do with the interface |
| **Frame time** | Wall clock from the start of a frame's CPU work to that frame's presentation | Not frame rate. Pacing is what matters, and an average hides it |
| **Deadline miss** | A frame whose frame time exceeded the output interval | |
| **Steady state** | Everything from the first presented frame onward. No warm-up is excluded | |
| **Reference puppet** | The generated workload the budgets are stated against, at the corpus median | Not a typical puppet, and not a limit — larger puppets must work |
| **Stress puppet** | The same, scaled past the largest rig measured, for growth testing | |
| **Counter** | A per-frame or per-operation quantity the runtime reports headlessly, asserted in CI against a committed ceiling | Not a timing. Counters are exact; timings are not |
| **Runtime** | The library that evaluates and renders a puppet, embedded in both applications so that Studio's viewport and Live's output are one code path | Not either application. Not the frame loop alone, which is one part of it |
| **Rigel2D** | The project and the toolkit as a whole — the format, the runtime and both applications | Not either application on its own, and not the repository when the repository is what is meant |
| **Studio** | The application that builds puppets. *Rigel2D Studio* where the full name is wanted | Inochi Creator's counterpart |
| **Live** | The application that performs a puppet. *Rigel2D Live* where the full name is wanted | Inochi Session's counterpart. **See the caution below** |
| **The editor** / **the player** | Descriptive terms for Studio and Live, still sanctioned | Use them where the sentence is about the role rather than the product. No longer placeholders |

**Caution: `Live` the product and `live` the adjective.** This document's whole purpose is to
stop one word carrying two senses, and this name does. "A scene edited while live" and "a scene
edited in Live" are different claims. The rule: **capital-L `Live` is the application, never
anything else**, and where a sentence could be read either way, name the application in full as
*Rigel2D Live* or reword to *while streaming*.

## Words we deliberately do not use

<!--
Words that are ambiguous in this domain, and what to say instead. This section prevents more
confusion than the table above, because it catches the terms nobody thought to define.
-->

| Avoid | Because | Say |
|---|---|---|
| Performance | Means both *a stage performance* and *how fast it runs*, and this project cares a great deal about both | **Live**, or "the player", for the first; "frame budget", "frame time" for the second |
| Model | Means a character rig to a vtuber and a data model to a programmer. Both senses are live here | **Puppet** for the character; name the specific structure otherwise |
| Session | Inochi Session's product name, and also a period of use, and also a streaming session | **Live** for the application. *An agent's working session* is a separate, sanctioned sense — it is not about the software being built |
| Object | Too loose to name a tree entry, and it will collide with whatever the implementation calls its base type | **Node**, or the specific kind |
| Layer | A PSD layer and a draw-order layer are different things, and import maps one onto the other | **PSD layer** on the import side, **part** on the puppet side |
| Sprite | Conflates the part with the texture, which this project deliberately separates | **Part** or **texture**, whichever is meant |
| Lane | inochi2d's word for the part of an animation that drives one parameter. Two vocabularies for one thing is the whole failure this document exists to prevent | **Track** |
| Project | There is no project file — a puppet is one file (`decisions.md`), so the word invites a question with no answer | **Puppet**, or **the repository**, whichever is meant. *The project* as the undertaking, and *the project owner*, are sanctioned — what is banned is a project **file**, document or artifact |
