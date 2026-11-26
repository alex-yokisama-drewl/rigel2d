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
| **Puppet** | A complete rig: node tree, textures, parameters, bindings, physics, animations. The thing both applications work on | Not "model" — that word is doing other work. Not "character", which is the art |
| **Node** | One entry in a puppet's tree. Part, mesh group and composite are kinds of node | Not "object" and not "layer" |
| **Part** | A node that draws a mesh with a texture on it | Not the texture. A part *references* a texture, and several parts may reference one ([puppet.md](requirements/puppet.md)) |
| **Texture** | An image a part draws with. An entity in its own right, referenced rather than owned | Not a part, not a sprite, not an atlas |
| **Mesh group** | A node that deforms its children with its own mesh | |
| **Composite** | A node that renders its children to a buffer so blending and masking apply to the group | |
| **Parameter** | A named one- or two-dimensional axis belonging to a puppet, with keypoints along it. The puppet's own controls | **Not an input channel.** A channel is bound *to* a parameter; they are different things and conflating them is the error this entry exists to prevent |
| **Keypoint** | A position on a parameter's axis at which binding values are stored. Values between keypoints are interpolated | Not a keyframe, which is on a timeline |
| **Binding** | A mapping from a parameter to something that changes with it — a node transform, or per-vertex offsets | |
| **Deformation** | Per-vertex offsets applied to a mesh | Not a transform, which moves a whole node |
| **Animation** | Parameter values driven over time, on a timeline | |
| **Physics** | Simulated motion that drives parameters, rather than input driving them | |
| **Scene** | The player's saved document: which puppets, placed where, fed by which sources, output how | Not a puppet. A scene references puppets |
| **Source** | A feed of input arriving over a port, supplying channels. A face tracker is one kind | Not a tracker specifically. Anything that can open a socket is a source |
| **Channel** | One named value arriving from a source | Not a parameter. See above |
| **Mask** | A part that restricts where another part draws | Not a composite, which groups children for blending |
| **Chain deformer** | A sequence of linked joints deforming geometry along it, a permanent part of a rig. For hair, tails, ropes | Not a skeleton, and it does not generalise into one |
| **Channel mapping** | A scene's assignment of a channel to a parameter | **Not a binding.** A binding belongs to a puppet and maps a parameter to geometry; a mapping belongs to a scene and feeds the parameter |
| **Render surface** | The surface the player draws the puppet on, containing nothing else | Not the control surface |
| **Control surface** | The player's interface. Never captured, never part of the output | Not the render surface |
| **Frame time** | Wall clock from the start of a frame's CPU work to that frame's presentation | Not frame rate. Pacing is what matters, and an average hides it |
| **Deadline miss** | A frame whose frame time exceeded the output interval | |
| **Steady state** | Everything from the first presented frame onward. No warm-up is excluded | |
| **Reference puppet** | The generated workload the budgets are stated against, at the corpus median | Not a typical puppet, and not a limit — larger puppets must work |
| **Stress puppet** | The same, scaled past the largest rig measured, for growth testing | |
| **Counter** | A per-frame or per-operation quantity the runtime reports headlessly, asserted in CI against a committed ceiling | Not a timing. Counters are exact; timings are not |
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
| Session | Inochi Session's product name, and also a period of use, and also a streaming session | **Live** for the application |
| Object | Too loose to name a tree entry, and it will collide with whatever the implementation calls its base type | **Node**, or the specific kind |
| Layer | A PSD layer and a draw-order layer are different things, and import maps one onto the other | **PSD layer** on the import side, **part** on the puppet side |
| Sprite | Conflates the part with the texture, which this project deliberately separates | **Part** or **texture**, whichever is meant |
| Project | There is no project file — a puppet is one file (`decisions.md`). The word now only invites the question back | **Puppet**, or **the repository**, whichever is meant |
