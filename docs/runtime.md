# The runtime

What the runtime owns, where its boundaries are, and how it is measured. The runtime is the
library both applications embed, so that Studio's viewport and Live's output are one code path
and a rig cannot look right while being authored and wrong while being performed.

The evaluation pass itself is [evaluation.md](evaluation.md); this is about ownership, the
boundaries and the counters.

## What it owns **[proposed]**

| Owns | Does not own |
|---|---|
| Parsing, validating and holding a puppet | Where the file came from, or the dialog that chose it |
| The evaluation pass and the render | The window, the control surface, any interface |
| GPU resources for a loaded puppet, and their lifetime | Which GPU, or how the surface is presented |
| The atomic write path for puppets and scenes | When to save, or what the user was asked |
| The counters | What CI does with them |

**The applications own every decision a person makes; the runtime owns everything that follows
from one.** That line is what keeps the two applications from diverging: Studio and Live differ
in what they ask the user and in what they do with the result, not in how a puppet evaluates or
draws.

## Loading never disturbs what is already there **[proposed]**

A load produces a complete, validated puppet or it produces nothing. There is no partially
constructed puppet reachable from anywhere, and no stage at which a caller holds half of one.

This is [I4](invariants.md), and the design that satisfies it is that **loading and installing
are separate steps with a complete puppet between them**. The caller loads — which may fail, at
any point, for any reason — and only then installs. An install cannot fail.

Three consequences that are easy to miss:

- **The peak cost of a reload is two puppets resident**, because the new one is complete before
  the old one is released. The resident-bytes budget in
  [platform.md](requirements/platform.md) is stated against one puppet, so a reload transiently
  exceeds it. That is accepted and is the price of the invariant.
- **Loading an additional puppet is not a replace** and is outside the rule entirely.
- **Validation is not a pre-pass over the file followed by a trusting parse.** The same
  validator gates both, and the load limits are checked before allocating rather than after
  reading.

## One write path **[proposed]**

Every write to a puppet, a scene or an autosave goes through one function: write beside the
target, fsync, rename over. [I7](invariants.md) is checked by a script asserting that no write
happens anywhere else, which is why it is one function and not a convention.

**Live writes puppets**, for channel mappings, so this binds Live as much as Studio. A build
that writes a puppet must also carry through everything it did not parse — extension data,
authoring data, a higher minor version's additions — which makes the write path and the
preserve-what-you-do-not-understand rule the same mechanism rather than two.

## Counters are the requirement **[proposed]**

[platform.md](requirements/platform.md#how-this-is-verified) explains why: there is no reference
machine for routine testing, so wall-clock figures are targets that nothing checks day to day
and the counters carry the budget on their own. That makes headless counting a constraint on
the runtime's design rather than an instrumentation detail added later.

**The runtime must be able to produce and count a frame without a GPU present.** The design that
delivers this is that the render stage emits an explicit command stream, which a backend then
submits — so counting is reading the stream, and CI needs no device. The alternative, calling a
graphics API directly from the evaluation pass and counting through the API, ties every counter
to a backend that hosted CI cannot run.

Counted quantities are listed in
[platform.md](requirements/platform.md#how-this-is-verified) and not repeated here. Two carry
rules rather than merely numbers:

- **Name resolutions must be zero.** Identifiers resolve to handles once, at load and at source
  connection. A non-zero count is a defect, not a regression.
- **GPU resource creations must be zero after the first presented frame.** This is
  [I5](invariants.md), and it is asserted as *no first-use work*: every render state any puppet
  can reach is prepared during load.

The last one has a sharp edge worth stating. *Every state a puppet can reach* means every
combination of blend mode, mask and composite the loaded puppet actually uses — enumerated from
the puppet during load, not discovered when a parameter first moves far enough to show a part
that was hidden. A puppet whose rarely-used expression needs a pipeline nobody compiled is
exactly the hitch I5 exists to prevent, and it appears minutes into a stream rather than at
load.

## Textures reach the GPU however the counters say **[proposed]**

One bind per distinct texture, until the counters show that binds matter. This is already
settled in
[requirements/puppet.md](requirements/puppet.md#how-textures-are-stored-is-not-how-they-reach-the-gpu)
and is repeated here only to name where the question gets answered: it is a runtime decision,
invisible in the file, and the file format does not depend on the answer — so settling it later
by measurement costs nothing.

## The render surface boundary **[proposed]**

The runtime draws puppet content and nothing else, into a surface it is handed. It has no
capability to draw anything else, which is how [I2](invariants.md) is structural rather than
observed: errors, dialogs and debug output cannot reach the render surface because the runtime
has no means of putting them there and the applications have no access to it.

Studio's gizmos are drawn by Studio, outside the render surface, over a viewport that contains
the same rendered content Live would produce.

## The input boundary **[proposed]**

A source may set channel values and trigger animations. The runtime exposes nothing else to it
— no load, no scene change, no filesystem, no output reconfiguration — and the port's bounds on
message size, channel-name count and name length are enforced at the boundary rather than
deeper in.

Clamping and the finite check happen here too, once, before stage 2 of the evaluation pass. See
[evaluation.md](evaluation.md#1-input-proposed).

## What the stack decides **[open]**

Three parts of this document are shaped by a choice not yet made, and nothing should be built
that assumes an answer:

- **Whether zero CPU allocation in the frame loop is natural or contorted**, which
  [platform.md](requirements/platform.md) already flags as the reason that figure is a budget
  rather than an invariant.
- **Whether the command stream is counted through a null backend the graphics abstraction
  provides, or through our own stream above it.** Either satisfies the requirement above; which
  one it is changes where the boundary sits and how much of the render stage is ours.
- **How structural UI assertions are written.** `scripts/ui-assertions.md` is written for a
  browser and a DOM, which is a hidden stack constraint rather than a neutral one —
  [testing.md](testing.md) resolves it once the toolkit is chosen.
