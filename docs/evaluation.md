# Evaluating a puppet

In what order a puppet is evaluated, and what composes with what. This is the contract Studio's
viewport and Live's output both honour, and the reason a rig that looks right while being
authored looks right while being performed.

[requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed)
commits to the order being specified rather than emergent. This document is that specification.

## The order **[proposed]**

One pass per frame, each stage reading only what the stages above it produced. **The pass never
iterates.** Its one backward read is backward in *time* rather than within the pass: a physics
node reads the previous frame's transform, which stage 2 describes.

| | Stage | Produces |
|---|---|---|
| 1 | **Input** | A value for every channel a source is sending |
| 2 | **Driving** | One value for every parameter |
| 3 | **Binding evaluation** | For each binding, its interpolated contribution |
| 4 | **Composition** | One accumulated transform and one accumulated deformation per node |
| 5 | **Tree transform** | Each node's final transform, accumulated from its ancestors. A physics node's own simulated displacement is part of it |
| 6 | **Group deformation** | Mesh groups and chain deformers applied to the geometry beneath them |
| 7 | **Render** | The frame |

The same pass runs in Studio and in Live. Studio differs only in what drives stage 2 — a rigger
dragging a parameter rather than a tracker — and in drawing gizmos **outside** the render
surface, never into it ([I2](invariants.md)).

## 1. Input **[proposed]**

A channel value passes through four steps, in this order, and the order is the whole content of
this stage:

| | Step | |
|---|---|---|
| 1 | **Finite check** | Reject anything that is not a finite number, as it arrives |
| 2 | **Normalisation** | The source's own calibration, from Live's configuration, onto the channel's specified range |
| 3 | **Response curve** | The mapping's authored shaping, from the normalised channel onto the parameter's axis |
| 4 | **Clamp** | To the parameter's range |

**The first and last positions are the load-bearing ones.** The finite check is first because
a NaN entering normalisation propagates through everything after it and arrives as geometry.
The clamp is last because normalisation and the curve are both transformations that can leave
the range — so clamping earlier would protect raw input and nothing else. Between them, a
source cannot deform a puppet past its authored extremes however it is calibrated or shaped,
and a NaN never reaches the renderer.

All four happen **before stage 2 and therefore before any binding sees anything**, once, rather
than defensively at each later stage. Steps 2 and 3 are defined in
[performing.md](requirements/performing.md#normalisation-is-the-recipients-shaping-is-the-riggers-proposed);
what belongs here is only that they sit in this order.

Latest value wins. The queue never grows — a source sending faster than the frame rate has its
intermediate values discarded, not buffered.

## 2. Driving: one parameter, one driver **[proposed]**

**Every parameter has exactly one base driver, declared in the puppet.** It is a channel
mapping, a physics output, or nothing — and nothing means the parameter rests at its neutral
keypoint. Two base drivers on one parameter is a validation failure, not a precedence question.

**Because** a precedence rule would resolve the conflict by silently discarding the loser,
turning a reportable authoring error into a behaviour nobody can see in the file. Determinism
here is by construction rather than by tie-break. [decisions.md](decisions.md) carries the
alternative and why it lost.

**Animations are the deliberate exception.** A playing animation overrides whatever drives a
parameter, blending in and out over a duration the animation declares, because
[performing.md](requirements/performing.md#what-the-puppet-does-when-its-source-drops-is-a-scene-setting-decided)
requires every transition into and out of a fallback behaviour to be continuous. Where two
animations target one parameter, the later-started one wins; it is stated here so that it is
not discovered.

Physics reads the previous frame's accumulated transform **of its own node**, which is what
makes hair follow a head turn. That is the one backward read, it is a frame old deliberately,
and it is why the pass never iterates. A physics node reads no node but its own, so physics adds
no reference kind and leaves a deletion nothing to dangle.

**A physics node's simulation also displaces its own node's transform**, and that displacement
joins the componentwise sum at stage 4 like any other contribution. This is what makes a chain
work: a physics node may contain only physics, so a child pendulum hangs from a parent that
already swings. **A chain lags by one frame however long it is** — each node reads a previous
frame in which its parent's displacement had already been applied, so the delay does not
accumulate down the chain. On the first frame after a load or a replacement there is no previous
frame, and every physics node reads its rest transform.

**The pattern this forces, named because it is what riggers will reach for.** Inertia on a
tracked value — a head turn that overshoots and settles — is not physics modulating the tracked
parameter, because that would be two drivers on one parameter. It is a physics node anchored
beneath a node the tracked parameter moves, driving a *second* parameter, with bindings
combining the two. The extra parameter is the cost of every parameter's value having exactly one
origin.

## 3. Binding evaluation **[proposed]**

A binding stores values at its parameter's keypoints. Evaluating it means finding the keypoints
bracketing the parameter's current value and interpolating — two on a one-dimensional axis, four
bilinearly on a two-dimensional one. Linear is the only mode the format defines today; another
is a format addition and does not change anything in this document.

## 4. Composition is additive, and that is why determinism is currently free **[proposed]**

Several bindings routinely touch one node — head roll, pitch and yaw at once is ordinary rigging
rather than an edge case. They combine by **summation**:

- **Deformations** are per-vertex offsets from the [rest pose](vocabulary.md). Offsets add.
- **Transforms sum componentwise** — translation, rotation angle and scale each accumulated on
  their own — and the matrix is built once, afterwards, from the accumulated components.

**Transforms are never composed by multiplying matrices at this stage.** Matrix multiplication
is order-dependent, so two bindings rotating one node would give a different result depending on
which was applied first, and the rigger's feedback loop — adjust, look, adjust again — would be
sitting on top of something they cannot see or predict. This is the one most likely to be
"fixed" by someone who knows graphics; [decisions.md](decisions.md) says why it is deliberate.

**What would reopen this.** Componentwise summation and matrix composition agree for small
angles and diverge as rotations grow. The signal is therefore concrete rather than a matter of
taste: a rigger composing large rotations on one node — a limb, not a head tilt — and finding
the result visibly wrong in a way matrix composition would have got right. That is evidence, and
it should reopen the choice rather than be worked around. Nothing in the format depends on the
answer, so the cost of changing it is the runtime and the rigs already authored, not the files.

**The honest observation worth recording:** summation is commutative, so the evaluation order
this document specifies does not currently change any result. That is not a reason to leave it
unspecified. It becomes load-bearing the moment a second composition mode exists, which
[requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed)
defers rather than rules out — and by then there would be files in the wild whose appearance
depends on an order nobody wrote down. Specifying it now costs nothing and cannot be done later.

## 5–6. Tree transform and group deformation **[proposed]**

Transforms accumulate from the root down. A mesh group deforms everything beneath it with its
own mesh; a chain deformer deforms the geometry along it. Both are producers of per-vertex
offsets and both are applied after the composition stage, so a node beneath a mesh group has
already finished composing its own deformations before the group's are applied to the result.

Nesting is therefore evaluated outermost-last, and the order among siblings is the tree's own —
stable, stored, and part of what canonical serialisation pins.

## 7. Render **[proposed]**

Draw order is the tree's order. A composite renders its children to an offscreen buffer so that
blending and masking apply to the group; a mask restricts where another part draws.

Offscreen passes are the dominant cost on the reference machine's integrated graphics, which is
why they are a counted quantity rather than an implementation detail —
[runtime.md](runtime.md) covers the counters and [platform.md](requirements/platform.md) the
budget they are measured against.

## What the frame loop may not do **[proposed]**

These are the rules that make the budget achievable, and each is counted rather than remembered:

| Rule | Why |
|---|---|
| **Nothing resolves by name.** Channel and parameter names resolve to handles when a source connects and when a puppet loads | A name resolution per frame is work that grows with the rig and buys nothing. The counter being zero is the enforcement ([I3](invariants.md)) |
| **No GPU resource creation and no pipeline compilation** | [I5](invariants.md). Every render state a puppet can reach is prepared before the first presented frame |
| **No CPU allocation** | A budget rather than an invariant, settled as one in [invariants.md](invariants.md) under I5: this is a slope, and invariants are for cliffs |
| **Animation lookup seeks; it does not scan** | The same shape as the name rule — cost that grows with the content for no gain |

## What a stale binding does **[proposed]**

Nothing. A binding whose mesh no longer matches it is skipped at stage 3 and contributes no
deformation, while remaining in the file and in the interface. It is a flagged state the rigger
resolves, and the runtime's only obligation is to be unsurprising about it: the puppet renders
as though that binding were absent, rather than refusing to load or producing distorted
geometry.
