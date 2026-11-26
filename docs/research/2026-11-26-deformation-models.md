# How do other 2D rigging systems model deformation, and does this project need a skeleton?

**Researched:** 2026-11-26 · **Feeds:** what deformation model the puppet format uses, in [puppet.md](../requirements/puppet.md) · **Verdict:** Keep the parameter-and-deform model as the core and add no skeleton now, but the evidence says the model's weak points are *composition* and *long chains* (hair, tails, limbs), not bones as such. Specify composition explicitly, and treat a chain or path deformer as the first skeleton-like primitive to reserve a place for.

## The decision this feeds

Whether the puppet format gains bones and vertex skinning beneath the parameter-and-deform model. The options on the table: inochi2d's model unchanged (node tree, parameters, bindings to node transforms or per-vertex offsets, physics); that model plus a skeleton; or something in between. The owner's instinct is that rotation plus mesh deformation suffices in 2D.

## Verdict

**The instinct is largely supported, with two caveats the evidence makes concrete.**

1. **Cubism has no bones.** It is the strongest evidence available: the commercial incumbent for 2D vtubing covers faces, head turns, bodies and limbs with a nested tree of two deformer kinds (warp, rotation) plus per-vertex keyforms driven by parameters. Where it needed something skeleton-like, for long thin meshes such as hair, it added a *limited* skinning feature built from chains of rotation deformers, and its own manual says a full skinning function "will be supported in the future". So the incumbent's answer is "no skeleton, except a narrow bone-chain for hair".
2. **The real weakness is how deformations compose, not the absence of bones.** In every parameter-keyform system, several parameters acting on one mesh multiply (a keyform grid) unless the rig is split across a hierarchy of deformers or uses additive shape differences. Cubism's manual is organised around this problem: at most two parameters per object, three if necessary, and Blend Shapes as an additive escape from the multiplication. Bones do not remove this problem; they relocate it.

**Recommendation.** Do not build a skeleton or vertex skinning into the first format. Do three things instead, all inside the existing `[open]` constraint that the format must not preclude one:

- **Make composition a specified, first-class part of the format**: an explicit evaluation order, and a stated blend rule per binding (replace, add, or multiply). This is the area inochi2d is reported weakest in, and it is what Cubism spent its Blend Shape feature on.
- **Reserve a place for a chain or path deformer** (a sequence of pivots deforming a mesh, with per-vertex weights). It is the feature Cubism added for hair, it is the smallest step toward skinning, and inochi2d's own spec already lists a `PathDeform` node (see Evidence).
- **Note that a bone is a node with a transform and no drawing.** The node tree already carries transforms and hierarchy, and every entity has a stable identity. A later "skin" binding (per-vertex weights referencing nodes) would then be an addition, not a redesign. That is the sense in which the format would not preclude a skeleton.

Revisit if the project's target widens to full-body, tracked-arms or game-character use, which is where skeletons clearly earn their place (see the comparison).

## Evidence, per system

Source dates: the Live2D, Spine, Rive and Synfig pages are living documentation, read on 2026-11-26 unless a date is given. **Cubism Core is closed, so Cubism's internals are inferable only from its documentation.** Where I could not confirm something it is marked **[unconfirmed]**.

### Live2D Cubism (the incumbent)

**Does Cubism use bones? No.** I found no bone or skeleton tool in the editor manual, in the 5.1 and 5.3 "what's new" pages, or in the skinning page. A community feature request asks for bones ("Bones would be nice to do initial rigid animation and after that use deformers to do further animation", [community.live2d.com/discussion/863](https://community.live2d.com/discussion/863/sketches-and-bones), no staff reply visible); another asks whether a bone can be glued to a mesh for Unity export ([discussion/1105](https://community.live2d.com/discussion/1105/bone-live2d-in-unity), August 2021, no reply visible).

**Primitives**

| Primitive | What it is | Source |
|---|---|---|
| Warp deformer | A grid of control points; deforms every mesh and deformer inside it | [Deformers](https://docs.live2d.com/en/cubism-editor-manual/deformer/) |
| Rotation deformer | An angle about an origin; rotates contents "without changing its shape" | same |
| ArtMesh | The drawn mesh; carries its own per-vertex keyforms | same |
| Parameter plus keyforms | Each deformer or ArtMesh stores keyforms at parameter points; a two-parameter object stores the grid | [Keyforms](https://docs.live2d.com/en/cubism-editor-manual/keyform-parent-chilid-relation/) |
| Blend Shape | A shape *difference* with a weight, added on top of the normal keyforms | [Blend Shape](https://docs.live2d.com/en/cubism-editor-manual/blend-shape/) |
| Glue | A per-vertex binding that ties vertices of two meshes together, with weights | [Skinning](https://docs.live2d.com/en/cubism-editor-manual/skinning/) |
| Skinning | Several rotation deformers on one ArtMesh; the mesh is auto-split between them and glued. Deform paths convert to a rotation-deformer chain and generate parameters | [Skinning](https://docs.live2d.com/en/cubism-editor-manual/skinning/), [Deform paths](https://docs.live2d.com/en/cubism-editor-manual/deformpath/) |

**Nesting.** Per the [parent-child combinations page](https://docs.live2d.com/en/cubism-editor-manual/combintion-of-parent-child-relation): warp in warp shapes and moves; rotation parent with warp child changes shape while rotating (necks); rotation in rotation is "used to express the articulation of legs and arms"; warp parent with rotation child moves and rotates the child rather than deforming it (breathing, shoulder shrugs). A child warp that overhangs its parent still works but costs more. The [tutorial](https://docs.live2d.com/en/cubism-editor-tutorials/deformer/) rigs an arm as one rotation deformer with a parameter, and tilts the body with a warp deformer.

**Evaluation order, as documented.** The parent-child combination page gives: the parent warp's own parameter interpolation, then the child's, then the parent's transformation of its children (as relayed in search results; I did not re-read that sentence on the page, so treat the order as **[unconfirmed]** beyond "parent transforms children"). The keyform blending formula and the internal order inside Core are **not published**; the Core API reference PDFs list drawable vertex output only.

**Driving.** Parameters are the single interface. Motions are parameter timelines; physics and sway drive parameters. This is the same shape as the project's vocabulary.

**Composition: documented behaviour and costs.**

- Keyforms on one object multiply across its parameters; the manual's example is 3+3+9 = 15 patterns when form, angle and X/Y are split across deformers, against 81 on one ([keyforms page](https://docs.live2d.com/en/cubism-editor-manual/keyform-parent-chilid-relation/)). Splitting into a hierarchy is the prescribed remedy.
- Blend Shapes add differences without multiplying; the [performance page](https://docs.live2d.com/en/cubism-editor-manual/performance/) says they "can reduce the cost" for objects with many multiplied parameters. They cannot be used with multiple-keys or extended interpolation, and weights need limit settings when several are active.
- A large rotation of an ArtMesh or warp deformer "can cause shrinkage from linear interpolation"; the manual says to use a rotation deformer instead ([Deformers](https://docs.live2d.com/en/cubism-editor-manual/deformer/)). This is a direct statement of the central weakness of interpolating offsets, and it is the argument for keeping rotation as a transform rather than as keyed vertices.
- Warp deformers now auto-extend to children that exceed them (5.3, [what's new](https://docs.live2d.com/en/cubism-editor-manual/new-function5-3/)), which shows how much friction the nesting rules caused.

**Complaints found.** Thin, and I could not read most thread replies (sign-in walls). Found: a user cannot make a child rotation deformer ignore its parent's angle ([community.live2d.com](https://community.live2d.com/discussion/comment/2455/) and neighbours); a warp deformer containing rotation deformers does not scale them as expected ([discussion/comment/2458](https://community.live2d.com/discussion/comment/2458)); an error creating a rotation deformer for a head tilt when the contents have different parent deformers ([discussion/833](https://community.live2d.com/discussion/833/trouble-with-rotation-deformers)); head turns "take quite some time" and diagonals need a separate "Synthesize Corner" step. **I found no documented complaint specifically about missing bones from working riggers beyond the two requests above.** That is a finding about my search reach, not proof none exist.

**Cost per frame.** Published guidance only: parameters per object dominate (up to two, three if necessary, more than four discouraged); draw-order switches between textures, blend modes and clipping cost; overhanging deformers cost; deformer count and hierarchy depth are listed as factors with no numbers ([performance](https://docs.live2d.com/en/cubism-editor-manual/performance/)). No per-vertex figures are published. A search snippet from live2d.com reported the free edition caps deformers (30 combined) and Pro lifts the warp division cap; I did not open that page, so **[unconfirmed]**.

### Spine

- **Primitives.** Bones (a transform hierarchy), weighted meshes (each vertex weighted to bones, weights sum to 100%), and deform keys, which store only vertex positions and interpolate "in a straight line" ([Meshes](https://en.esotericsoftware.com/spine-meshes), [Weights](https://en.esotericsoftware.com/spine-weights)). Constraints on top: IK, transform, path, slider ([spine-cpp docs](https://tr.esotericsoftware.com/spine-cpp)), and physics constraints for secondary motion on hair and clothing ([physics constraints](https://en.esotericsoftware.com/spine-physics-constraints); introduction version not stated on the page).
- **Driving.** A timeline per animation, plus skins; there is no parameter space. A runtime sets bone values directly (aim at the cursor) or moves IK targets.
- **Composition.** Bones compose through the hierarchy and constraints in a defined order (root first, then constraints, per the runtime docs). The Meshes page does **not** say how deform keys combine with weights, and the official advice is that deform keys "should be avoided or kept to a minimum" with weights preferred. Deform keys also lose rotation and scale, so they are fragile. This is the same shape of problem as the project's: per-vertex offsets are the escape hatch, and the vendor discourages leaning on it.
- **Easy and hard.** Bodies, limbs, props, anything articulated: easy. Faces and faux-3D head turns: Spine users report doing these with one or two bones for a faux-3D effect, while per-vertex work is manual ([Spine forum](https://esotericsoftware.com/forum/d/14671), summarised; I read only a snippet).
- **History that matters.** FFD was added on 2014-01-15 because bones alone limited liveliness and made animators draw more art; the post says FFD "doesn't replace bones" and that skinning was next ([blog](https://en.esotericsoftware.com/blog/ffd-now-available)). Spine started with bones and added deformation; Cubism started with deformation and added a thin skinning layer. They converged from opposite ends.
- **Cost.** Per-vertex cost scales with the number of influencing bones; small weights "still cause extra vertex transforms to be calculated", and the pruning tool exists to cut them ([Weights](https://en.esotericsoftware.com/spine-weights)). No published numbers on that page.

### DragonBones

Skeletal animation with skinning, mesh and FFD, and IK constraints, per its own site ([dragonbones.github.io/en/animation.html](https://dragonbones.github.io/en/animation.html), dated 2012-2017). Runtime docs have a shared-mesh option that inherits FFD from a master mesh. **Thin evidence**: I found nothing on how FFD composes with bound meshes, nothing on complaints, nothing on cost. It is a Spine-class system and adds no new idea; I did not pursue it further. The project appears dormant on its documentation dates; **[unconfirmed]**.

### Rive

Bones that "bend vector paths and images by binding them to bones", meshes with weights, and constraints (IK, distance, follow path, translation, rotation, scale, transform, scroll) ([Rive docs](https://rive.app/docs/editor/manipulating-shapes/manipulating-shapes.md)). Driven by timelines and a **state machine** with data binding, plus Solos to switch hierarchies or create skins ([Solos](https://rive.app/changelog/solos-released)). The runtime source has a `Skin` class depending on bones and tendons ([rive-cpp skin.cpp](https://skia.googlesource.com/external/github.com/rive-app/rive-cpp/+/HEAD/src/bones/skin.cpp)); I only saw dependency-building fragments, so per-vertex cost and influence limits are **[unconfirmed]**. Rive's own Rive-versus-Spine page is marketing and I did not rely on it. What it adds that is genuinely different: the state machine as the driver, which is a layer above this project's parameters and animations, not a deformation primitive.

### Moho / Anime Studio

Bones for vector and image layers, plus **smart bones**: a bone angle acts as a dial that drives "scene elements in ways you define" ([tutorial index](https://www.animestudiotutor.com/bones/animating_a_head_turn_with_smart_bones/), summarised from search; the Smart Bone page I fetched did not explain internals). A smart bone is, in effect, a parameter with keypoints, hosted on a bone. Head turns are routinely rigged this way, including morphing a head shape between front and three-quarter views. This is the clearest cross-system evidence that **a skeleton does not solve head turns; a parameter-like driven-pose mechanism does**, even in a bone-based tool. Internal action blending is **[unconfirmed]**.

### Synfig

A Skeleton layer holds bones and links vector layers to them; a separate Skeleton Deformation layer deforms **raster** pixels over a bounding rectangle with a mesh resolution ([Skeleton layer](https://synfig.readthedocs.io/en/stable/layers/skeleton.html), [Skeleton Deformation](https://synfig.readthedocs.io/en/latest/layers/skeleton_deformation.html)). Driven by a timeline. A forum report says layer order matters with multiple deformation layers and the workaround is one deformation layer per part ([Synfig forum](https://forums.synfig.org/t/solved-problem-with-link-to-bone-pivote-points-i-dont-know/4025), anecdotal). Relevant as a cautionary example of composition being implicit in layer order, which is the failure this project wants to avoid. Not a real-time or vtubing system; cost not published.

### Blender grease pencil

The 3.x manual lists Armature, Hook and Lattice among grease pencil deform modifiers ([Armature](https://docs.blender.org/manual/nb/dev/grease_pencil/modifiers/deform/armature.html)); I could not read the current (4.3 and later) list, and an attempt to fetch it returned 403. Mesh Deform and shape keys do not appear in the 3.x lists. An undated 80.lv article describes grease pencil shape keys as still in development, with problems combining them with armatures and drivers ([80.lv](https://80.lv/articles/the-long-awaited-shape-keys-for-blender-s-grease-pencil-tested-on-a-character-rig)). A Blender Artists thread reports lattice plus armature deformation not blending cleanly at intermediate weights ([thread](https://blenderartists.org/t/lattice-armature-causing-unexpected-translation/1256823)). Useful only as a second data point that **stacking independent deformers is where things go wrong**; it is stroke-based 3D-space tooling, not a puppet format, and I would not borrow from it.

### Others that are genuinely different

- **Animaze** (a vtubing runtime that is not Cubism): GPU skinning only, at most 255 bones per mesh and four influences per vertex, extra weights dropped and renormalised ([Animaze technical reference](https://www.animaze.us/manual/technicalreference/deformation)). The useful part is the concrete cost shape: bounded influences make skinning a fixed, cheap vertex-shader cost. It also confirms that a live-performance runtime has chosen skinning.
- **Inochi2D itself**: the JSON spec wiki (marked out of date) lists a `PathDeform` node with `joints` and per-drawable binding indices ([wiki](https://github.com/Inochi2D/inochi2d/wiki/JSON-Data-Specification)), and the Inox2D README lists Parameters, Deforms (mesh vertex offsets) and Values (node transform offsets). **This contradicts the brief's working assumption that inochi2d has "no skeleton" in any sense**: a joint-chain deformer exists in its node vocabulary. How it composes with bindings I could not confirm. The reported weakness in composition is therefore **not independently confirmed by me**; the inochi2d docs site returned no detail and I did not read its source.
- **Toon Boom Harmony** appeared in results with bone deformers and other deformer kinds ([docs](https://docs.toonboom.com/help/harmony-22/advanced/getting-started/deformation.html)); I did not read it. Harmony is the closest conventional-animation analogue to Cubism's nested deformers and is a candidate for follow-up if composition design needs more precedent.

## Comparison across the dimensions

| | Cubism | Spine | Rive | Moho | Animaze (skinning) | Synfig |
|---|---|---|---|---|---|---|
| Core primitive | Nested warp and rotation deformers; per-vertex keyforms | Bones; weighted meshes; deform keys | Bones; weighted meshes; constraints | Bones; smart bones; warp meshes | Bone skinning on GPU | Bones; raster deformation layer |
| Driven by | Parameters; timelines of parameters; physics | Timeline; skins; direct bone sets; constraints | Timeline; state machine; data binding | Timeline; bone angles as dials | Not documented in what I read | Timeline |
| Composition | Hierarchy (child in parent space); keyform grid multiplies; Blend Shape adds | Bone hierarchy then constraints; deform key order **[unconfirmed]** | **[unconfirmed]** | Smart-bone actions on bones; internals **[unconfirmed]** | Four weights per vertex | Layer order |
| Faces and expressions | Strong (the design centre) | Weak without deform keys | Weak without meshes | Strong via smart bones | n/a | Weak |
| Limbs and bodies | Adequate: rotation deformers in a chain | Strong | Strong | Strong | Strong | Adequate |
| Head turn, faux-3D | Strong; costly in keyforms | Faux-3D via a few bones | **[unconfirmed]** | Strong via smart bones | n/a | Weak |
| Hair, cloth, tails | Limited skinning on rotation chains; physics and sway | Physics constraints on bones | **[unconfirmed]** | **[unconfirmed]** | n/a | n/a |
| Frame cost | Per-object parameters multiply; no numbers published | Per-vertex per-influence | **[unconfirmed]** | **[unconfirmed]** | Bounded, GPU | **[unconfirmed]** |

**Composition, the question the project most wants answered.** The systems fall into three strategies, and each costs something:

| Strategy | Used by | Buys | Costs |
|---|---|---|---|
| Hierarchy: each stage deforms in its parent's space | Cubism, Spine bones, Rive | Independent controls that do not multiply; a stage is editable alone | Order is baked into the tree; reparenting changes the result; children that leave their parent cost more or misbehave (Cubism overhang and scaling reports) |
| Grid of keyforms: all parameters on one object | Cubism per object | Any combination can be authored exactly | Keyforms grow as the product of axis sizes; Cubism caps practice at two parameters |
| Additive differences | Cubism Blend Shape | Linear cost; order-free; good for expressions | Cannot represent rotation (differences interpolate linearly); needs weight limits when several fire; excluded from some features |
| Implicit by stack or layer order | Synfig, grease pencil | Cheap to build | Reported as the source of surprises |

**Inference, not a sourced claim:** interpolating a binding across an N-axis grid cell touches 2^N keyforms per vertex, so the per-vertex cost of a multi-axis deformation grows with axes, which is consistent with Cubism's advice to keep parameters per object small. A skinned vertex touches a bounded number of bones (four in Animaze) regardless of how many animations are active. For a fixed frame budget on weak integrated graphics, skinning is the cheaper primitive per vertex *when it applies*; the open question is authoring, not runtime.

## Two direct answers

**1. Does Cubism use bones?** No. Its deformers are warp (a control-point grid) and rotation (angle about an origin), nested in a tree, plus per-vertex keyforms on ArtMeshes, all driven by parameters. Nesting and the four parent-child combinations are in the Evidence section. A limited skinning function (rotation-deformer chains with glue, for long thin meshes) was added later and is explicitly described as a stopgap.

**2. Where does a skeleton earn its place in 2D?**

*Evidence that it does:*
- **Long thin things.** Cubism's one concession to skinning is for hair and strings, and it points toward "full skinning" later. Keyed offsets on a warp grid bend such shapes badly; chains of pivots do it naturally.
- **Large rotations.** Offsets interpolate linearly and shrink; Cubism says to use a rotation deformer instead. A rotation is a skeleton fragment, and the project's node transforms already provide it.
- **Reuse across motion.** Spine's model of one skeleton under many timelines suits game characters; its vendor discourages per-vertex keys because they are fragile and heavy. Animaze, a live runtime, chose GPU skinning.
- **Physics on chains.** Spine drives secondary motion through bones; chain-shaped physics maps onto a skeleton directly.

*Evidence that it does not, or not alone:*
- **Faces and head turns.** The industry incumbent does these with parameters, and Moho, a bone-based tool, added smart bones (dial-driven poses) precisely because bones do not give head turns. Spine added FFD because bones alone were limiting.
- **Vtubing specifically.** The incumbent's entire market is served without bones. A forum thread says a streamer would choose Spine if it had vtuber runtimes ([Spine forum, summarised](https://fr.esotericsoftware.com/forum/d/26595-vtubing-with-spine-models)); the gap there is tooling and tracking integration, not deformation.
- **A third-party tutorial claims rotation of the head is "off the table" for 2D rigs** ([Rokoko](https://rokoko.com/insights/vtuber-rigging-tutorial)). That contradicts the Cubism workflow and the working practice documented above, and I treat it as an overstatement.

**What I could not find:** a practitioner account, from a working vtuber rigger, of parameter-and-deform breaking down on a limb and bones fixing it. The nearest are Cubism's own hair skinning and the two feature requests. The "limbs and body motion are the weakest part" line in puppet.md is therefore **plausible but not evidenced by anything I found**.

## Recommendation, and the risks of taking it

**Take:** parameter-and-deform core, no skeleton in the first format, with composition specified (order and blend rule per binding), a reserved slot for a chain or path deformer, and the format shaped so a weight-based "skin" binding on node transforms could be added later.

**Why.** It matches what the incumbent ships for the basics; the cost of skinning is authoring (weight painting, bone placement, a whole editing mode), which contradicts the project's trade of "cover the basics excellently"; and the largest, best-evidenced weakness is composition, which a skeleton would not fix.

**Risks.**
- **Hair and tail quality.** Without a chain primitive, long thin parts fall back on warp grids and physics-driven rotation. That is Cubism's pre-skinning state, which users were evidently unhappy enough with to get a feature added.
- **Full-body and tracked-arm use.** If the target audience extends to full-body vtubers or game use, parameter-per-pose for arms multiplies keyforms and the absent skeleton becomes the main complaint. This is the likeliest reason to reverse the decision.
- **Evidence gaps on the thing being improved.** I could not confirm how inochi2d composes bindings, so the claim that it is weakest there rests on the brief and not on my findings. Confirm by reading its source before designing the improvement.
- **Reserving a slot is not free.** A format that reserves a chain deformer and a skin binding without implementing them risks designing them wrongly. The cheap form is a stated extension mechanism (unknown node and binding kinds are preserved, as puppet.md already requires) rather than speculative fields.

## What would change this

- **Reading inochi2d's source** (binding application order and the `PathDeform` implementation) would confirm or retire the composition complaint. Highest-value follow-up.
- **A practitioner account** of bones solving a limb problem in a Cubism-class rig, or the reverse, would settle the open question puppet.md poses. Live2D's community forum (sign-in walls blocked most replies) and the nijigenerate and Inochi2D community channels are the places to look.
- **Cubism's own direction.** Its skinning page promises fuller skinning "in the future"; if a later Cubism release ships it, that is strong evidence the incumbent concluded the model needs bones. Re-check the Cubism "what's new" pages after each release; the pages read here (5.1, 5.3) did not mention it.
- **Rive, Moho and DragonBones internals** are unconfirmed in this report. They matter only if the design wants a precedent from them.
- **Target audience.** A decision to serve full-body or game use changes the answer.
- Re-check the Spine physics constraint availability and the Blender grease pencil modifier list if either becomes a precedent; both were only partly read.

## Rejected options

- **Full bone-and-skinning model as the core (Spine, Rive, DragonBones style).** Loses on the project's stated trade (large advanced feature, a whole editing mode) and on the evidence that faces and head turns, the vtubing core, are done with parameters even in bone-based tools. Not rejected as a *later addition*.
- **Pure grid-of-keyforms per object with no hierarchy.** Keyform counts grow as the product of axes; Cubism's own manual steers authors away from it.
- **Additive differences only (Blend Shape as the sole mechanism).** Cannot represent rotation, which interpolates linearly and shrinks; Cubism keeps rotation deformers for that reason.
- **Composition by layer or stack order left implicit (Synfig, grease pencil style).** Reported as the source of surprises in both; contradicts the project's goal of explicit, validator-checkable semantics.
- **A state machine as the deformation driver (Rive).** It is a driver above the deformation layer, not a deformation model; it may belong to the player's scene design, not this decision.
- **Blender grease pencil as a precedent.** Stroke-based, 3D-space, partly in flux; contributes only a cautionary note about stacking deformers.
