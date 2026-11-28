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
| 2 | **Driving** | One value for every parameter: the base driver's, every playing track's contribution added, then clamped |
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

Every step happens **before stage 2 and therefore before any binding sees anything**, once, rather
than defensively at each later stage. Steps 2 and 3 are defined in
[performing.md](requirements/performing.md#normalisation-is-the-recipients-shaping-is-the-riggers-proposed);
what belongs here is only that they sit in this order.

Latest value wins. The queue never grows — a source sending faster than the frame rate has its
intermediate values discarded, not buffered.

## 2. Driving: one base driver, tracks added, one clamp **[proposed]**

**Every parameter has at most one base driver, declared in the puppet.** It is a channel
mapping, a physics output, or nothing — and nothing means the parameter rests at its neutral
keypoint. Two base drivers on one parameter is a validation failure, not a precedence question.
A parameter whose base driver has no data rests at its neutral keypoint as well, which is not
what a channel value of zero maps to: a response curve need not be centred on zero.

**Because** a precedence rule would resolve the conflict by silently discarding the loser,
turning a reportable authoring error into a behaviour nobody can see in the file. Determinism
here is by construction rather than by tie-break. [decisions.md](decisions.md) carries the
alternative and why it lost.

**The rule constrains parameters, not nodes.** Several bindings already sum on one node, node
transforms are not clamped, and nothing here limits how many contributions a node receives. A
parameter has one base driver and a clamp; a node has neither. This is stated because it is
the question people go and check.

### A track adds to its parameter **[proposed]**

**Every playing track adds its contribution to the parameter it drives.** There is no replacing
mode and no precedence between tracks: contributions sum, so no track is ever discarded. A
track is not a base driver and never was — a parameter may carry one alongside any number of
tracks across the animations playing.

**A keyframe holds a pose, and a track's contribution is the curve's value less the parameter's
neutral keypoint.** Keyframes are stored absolute, as keypoints are, so a track resting at the
neutral pose contributes nothing, and a lone track on a parameter with no base driver evaluates
to the authored value exactly. A track blends in and out over its declared
blend duration, which keeps every transition into and out of a fallback behaviour continuous
([performing.md](requirements/performing.md#what-the-puppet-does-when-its-source-drops-is-a-scene-setting-decided)).

**This is the price: an animation can push a parameter but never force it to a value.** *Head
roll exactly centred regardless of tracking* is inexpressible, because it would mean subtracting
a value the animation does not know. A replacing mode can be added later as a format addition
where absent means additive ([format.md](format.md#what-the-format-does-not-define-yet-proposed)).

**Because** override made the ordinary case unrepresentable. A breathing or idle animation
layered underneath live tracking is bread-and-butter vtubing, and under override a
permanently-playing idle blocks tracking entirely. The fallback override was designed for — a
source drops and an idle plays — is served as well by addition, once a lost source rests its
parameters at neutral. It also removes a precedence rule the project had already refused for
base drivers: *later-started wins* silently discarded the loser exactly as that rule would
have. [decisions.md](decisions.md) carries the supersession.

**Composition happens at the node wherever it can.** Layering a distinct motion — a saccade over
tracked pupil movement, a flinch over a tracked head pose — is better authored as a second
parameter bound to the same node than as a second track on one parameter, because nodes sum
unclamped and parameters clamp. Additive tracks exist for several animations layering on one
parameter, where the alternative is a parallel binding set per layer kept in sync by hand.

### The order of the sum is fixed by data **[proposed]**

**Float addition is commutative but not associative**, so three contributions give different
results depending on grouping, and three on one parameter is ordinary. The order is: the base
driver, then tracks by animation identifier, then by track identifier. It never depends on which
animation started first, on arrival order, or on how a collection happens to iterate. Without a
fixed order, two builds — or one build on two days — disagree in the last bits, which breaks the
bit-exact claim in [requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed).

### The clamp is the last step, and its result is always finite **[proposed]**

**The sum of every contribution is clamped to the range the parameter's axes declare.** A
track's own keyframe values sit inside that range; their sum with a base driver and with other
tracks need not. The clamp comes after the whole sum — clamping each contribution as it is added
gives a different answer — and the clamp at the end of [input](#1-input-proposed) is not this
one: that bounds a mapped channel, this bounds the total.

**A non-finite sum rests the parameter at its neutral keypoint**, not at a range edge. Every
contribution is finite and inside range, but a range may reach the float limit, so a sum can
overflow to infinity and `inf - inf` is NaN. A NaN passes straight through the obvious clamp —
Rust's `f32::clamp` propagates it — and would reach geometry, which [I2](invariants.md)
forbids on every failure path. An edge is a value somebody might have meant; neutral is the
one value every parameter declares as rest.

**The clamp is derived and never stored.** **Reporting it is edge-triggered and coalesced** —
once when a parameter starts clamping, once when it stops, never per frame, because a per-frame
report allocates in the frame loop and floods a log that is meant to stay bounded. A report
carries the parameter's identifier and never its name, because a name is puppet content.

### A track's value at a time **[proposed]**

Per-frame cost is one curve evaluation **per playing track**, not per keyframe. Keyframes are
storage plus the cost of finding the active one, by a cursor or a binary search and never a
scan.

**A Bézier segment is cubic in time as well as in value**, so evaluating at a time means solving
for the curve parameter. The iteration count is **fixed**, never convergence-based, because the
bit-exact claim would otherwise depend on the arithmetic. **A fixed count is not enough**: a
Newton step divides by a derivative that can be zero. The requirement is a finite result inside
the segment for every input, degenerate control points included — bisection, or Newton with a
bisection fallback. The reader has already checked that control times lie inside the segment and
that control values are finite ([format.md](format.md#validation-proposed)), which is what keeps
the curve a function of time.

### Physics reads backwards, once **[proposed]**

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

### Physics cannot run away **[proposed]**

A simulation described by a file that strangers trade is a differential equation with
arbitrary coefficients, and the only honest assumption is that some of them diverge. Every bound
below is a **runtime constant, not puppet data** — a rigger never sets one and a file cannot
raise one.

| Bound | Rule |
|---|---|
| **Timestep** | Fixed, independent of the output rate. Determinism is claimed only under a fixed step ([requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed)) |
| **Catch-up** | The simulated time a single frame may consume is capped. The excess is discarded, never simulated |
| **Bob distance** | Clamped to a multiple of that node's `length` |
| **Bob velocity** | Clamped to a multiple of `length` per second |
| **Non-finite state** | That node resets to rest for the frame |
| **Parameter output** | Clamped to the driven parameter's declared axis range |

**Discarding time rather than simulating it is the bound that matters on stream.** A two-second
stall must not produce two seconds of catch-up motion: that is a visible lurch, and it is
unbounded work inside the frame loop, which [I5](invariants.md) exists to prevent. Hair that did
not move is the right failure.

**The feedback loop is bounded by construction rather than by a limit.** Physics is the one stage
that reads backwards, and the only path from a physics node back to its own anchor runs through a
parameter: physics drives one, a binding on an ancestor reads it, the ancestor moves, the anchor
moves. **A parameter is clamped to its declared axis range**, so the gain around that loop has a
ceiling the data model already carries and nothing additional has to enforce. A chain needs no
equivalent, because a child never influences its parent — a chain is a cascade, not a loop.

**What these bounds buy is the severity of a bad rig, not its absence.** A node whose constants
are unstable at the fixed step still buzzes against its clamp. It does not produce a non-finite
transform, it never reaches the render surface ([I2](invariants.md)), and it cannot cost
unbounded time. That is why the upper bounds on the constants themselves can stay `[open]` in
[format.md](format.md) without holding anything up.

**The pattern this forces, named because it is what riggers will reach for.** Inertia on a
tracked value — a head turn that overshoots and settles — is not physics modulating the tracked
parameter, because that would be two drivers on one parameter. It is a physics node anchored
beneath a node the tracked parameter moves, driving a *second* parameter, with bindings
combining the two. The extra parameter is the cost of every parameter's value having exactly one
origin.

## 3. Binding evaluation **[proposed]**

A binding stores values at its parameter's keypoints. Evaluating it means finding the keypoints
bracketing the parameter's current value and interpolating — two on a one-dimensional axis, four
bilinearly on a two-dimensional one. Linear is the only mode the format defines between
keypoints; another is a format addition and does not change anything in this document. That
is a different question from how a track interpolates between keyframes, which is per segment
and answered under [driving](#a-tracks-value-at-a-time-proposed).

## 4. Composition is additive, and the order of every sum is fixed **[proposed]**

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

**Summation is commutative but not associative, so the order is load-bearing today.** Two
contributions are order-free; three or more give different results by grouping, and three
bindings on one node is the ordinary case this section opened with. Every sum in the pass —
here and on a parameter ([the order of the sum](#the-order-of-the-sum-is-fixed-by-data-proposed)) —
is therefore taken in an order fixed by the data and never by how a collection iterates.

**The bindings on one node are summed in ascending binding identifier.** Every other sum in the
pass already keys off an identifier, nothing but the file decides it, and reordering the stored
collection cannot change the result — which is what [ID-3](requirements/puppet-acceptance.md)
already asserts about the file's collection order. A physics node's displacement joins after
the bindings, because it is the one contribution that is computed rather than stored and so has
no identifier of its own to sort by.

A second composition mode would make the order matter further, and
[requirements/puppet.md](requirements/puppet.md#composition-is-deterministic-now-and-configurable-later-proposed)
defers that rather than ruling it out.

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
