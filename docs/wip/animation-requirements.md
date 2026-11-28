# Draft — animation requirements

**Working document.** Settled in conversation with the owner on 2026-11-28, swept for concerns,
then distributed into [puppet.md](../requirements/puppet.md),
[performing.md](../requirements/performing.md) and
[authoring.md](../requirements/authoring.md) and deleted. It exists so the concern sweep has one
subject rather than three, and so the restatement can be corrected before it is spread across
documents.

**Why now.** S3 needs to know what an animation contains before it can generate one, and the
owner's practical experience with Inochi contradicted two things the design assumed.

---

## 1. A track composes additively **[proposed]**

**Every animation track adds its contribution to the parameter it drives.** There is no replacing
mode and no precedence between tracks: contributions sum, so no track is ever discarded.

**The summation order is fixed by data, and this is not optional.** Float addition is
commutative but **not associative**, so three or more contributions give different results
depending on grouping — and three contributions on one parameter is ordinary. The order is:
the base driver, then tracks by animation identifier and then by track identifier. It never
depends on which animation started first or on how a collection happens to iterate.

**[evaluation.md](../evaluation.md) currently states the opposite and is wrong.** It says
"summation is commutative, so the evaluation order this document specifies does not currently
change any result". That holds for two contributions and fails for three, which that same
document calls ordinary rigging — "head roll, pitch and yaw at once". The claim must be
corrected there, not only here; the determinism it supports is real but only once the order is
fixed.

This supersedes the current rule, under which an animation *overrode* whatever drove a parameter
and the later-started animation won where two targeted one parameter.

**Because** the override rule made the ordinary case unrepresentable: a breathing or idle
animation layered *underneath* live tracking is bread-and-butter vtubing, and under override a
permanently-playing idle would block tracking entirely. The fallback it was designed for — a
source drops, play an idle — is served just as well by addition, once a dropped source rests its
parameters at neutral (§5).

**It also removes a precedence rule the project had already rejected elsewhere.**
[decisions.md](../decisions.md) refuses a precedence order between base drivers because it
"resolves the conflict by silently discarding the loser"; *later-started wins* was exactly that,
disclosed rather than argued. Summation discards nothing.

**A base driver is still singular.** One channel mapping, one physics output, or none. Tracks are
not base drivers and never were; this changes how a track composes, not how many drivers a
parameter has.

### A keyframe holds a pose; its contribution is that pose less the neutral **[decided]**

**A keyframe stores an absolute value, the same way a keypoint does, and what the track
contributes is that value minus the parameter's neutral keypoint.**

This is the hinge the rest of the round turns on, and the alternative — a keyframe storing the
contribution directly — breaks four things at once. A track resting at the neutral pose would
add the neutral value on top of its base driver and shove the puppet permanently off-pose. A
rigger scrubbing to author a pose sees absolute values everywhere else in Studio. Imported
Cubism curves are absolute and would need converting. And the existing rule that a track's
keyframe values sit inside its parameter's range would no longer make sense, because a
contribution's natural range is not the parameter's.

It also composes correctly at the edges: a single track on a parameter with no base driver
evaluates to `neutral + (value − neutral)`, which is the authored value exactly.

### What this gives up **[decided]**

**An animation can push a parameter but can never force it to a specific value.** On a bilateral
axis, *head roll exactly centred regardless of tracking* cannot be expressed, because it would
mean subtracting a value the animation does not know. A unipolar axis fares better only
because its range is one-sided, not because saturation is guaranteed — a track's keyframes must
sit inside the parameter's range, so the largest push it can make is bounded by that range.

Accepted deliberately. The capability is speculative, the simplification is immediate, and a
replacing mode can be added later as a format addition where **absent means additive** — a clean
addition rather than a migration.

### Composition happens at the node wherever it can **[proposed]**

Several bindings already touch one node and sum there, and node transforms are **not** clamped.
Parameters are clamped (§2). So layering a distinct motion — a saccade over tracked pupil
movement, a flinch over a tracked head pose — is better authored as **a second parameter bound to
the same node** than as a second track on one parameter.

Additive tracks exist for what that cannot do economically: several animations layering on *one*
parameter, where the second-parameter route would mean a parallel binding set per layer, kept in
sync by hand.

**Stated because both the owner and the agent had to go and check** whether the one-driver rule
constrained nodes or parameters. It constrains parameters. [evaluation.md](../evaluation.md)
should say so where it states the rule.

## 2. A parameter clamps to its declared range **[proposed]**

**The sum of every contribution to a parameter is clamped to the range its axes declare.** A
track's own keyframe values are already required to sit inside that range; their sum with a base
driver and with other tracks need not.

**Studio surfaces clamping while authoring** — the rigger has to be able to see that a layer is
being eaten rather than discover it on stream. Clamping at frame time with no way to see it is
the silent discard this project refuses everywhere else.

**The clamp is the last step, and its result is always finite.** Every contribution is finite
and inside the parameter's range, but a range may reach the float limit, so a sum can overflow
to infinity and `inf - inf` is NaN. A NaN passes straight through the obvious clamp — Rust's
`f32::clamp` propagates it — and would reach geometry, which [I2](../invariants.md) forbids on
every failure path. **A non-finite sum rests the parameter at its neutral keypoint**, not at a
range edge, because an edge is a value somebody might have meant.

**The clamp is derived and never stored.** No file records that a parameter was clamped.

**Reporting is edge-triggered and coalesced** — once when a parameter starts clamping, once when
it stops, never per frame. A per-frame report allocates in the frame loop, which the budget puts
at zero, and floods a log that is supposed to stay bounded. What is reported carries the
parameter's **identifier**, never its name, because a name is puppet content.

**Studio can only predict part of it.** Studio has no tracking input, so it cannot know what a
base driver will contribute; from the file alone it can report where a parameter's animation
extent plus its driver's declared range could exceed the parameter's range. That is a pure
function and worth having. The live case belongs to Live.

**`[open]`: how the runtime surfaces a clamp to a streamer mid-stream.** It is never on the
render surface. Beyond that it belongs with *how a streamer learns something broke* in
[performing.md](../requirements/performing.md), which is already `[open]` and now gates this.

## 3. A track declares how it interpolates between keyframes **[proposed]**

The modes are Cubism's own, because importing animation that already exists is a case this
project serves:

| Mode | Between two keyframes |
|---|---|
| **Linear** | Straight interpolation |
| **Stepped** | Holds the earlier value until the later keyframe |
| **Inverse stepped** | Takes the later value immediately |
| **Bézier** | A cubic through two control points |

**Declared per segment, not per track**, matching the corpus: a single track mixes modes, and a
format that could not express that could not import one.

**Bézier evaluation is iterative and must be bounded.** A Bézier segment is cubic in *time* as
well as in value, so evaluating at a given time means solving for the curve parameter. The
iteration count is **fixed**, never convergence-based, because this project claims bit-exact
determinism per platform and a convergence test makes the result depend on the arithmetic.

**Because** stepped is how discrete state swaps are expressed and is present in shipped sample
rigs; Bézier is what lets a rigger author smooth motion without baking a keyframe per frame,
which the owner currently does with scripts.

**A Bézier segment's control times lie inside the segment**, and every control value is finite.
Outside, the curve stops being a function of time: the solve has several roots and the iteration
can leave the segment entirely. Cubism guarantees this; a stranger's writer need not, so the
reader checks it.

**The solver is guarded, not merely fixed-count.** A fixed number of Newton steps still divides
by a derivative that can be zero, which gives NaN. The requirement is a finite result inside the
segment for **every** input, degenerate control points included — which means bisection, or
Newton with a bisection fallback.

**A mode is stored on the earlier keyframe of its segment, and the last keyframe carries none.**
Storage by position is the trap this project already has an entry for: inserting or deleting a
keyframe must carry each segment's mode — and any preserved material a build does not
understand — with the right segment. Linear is the absent form, so a linear track stores no
modes at all.

**Interpolation between a parameter's keypoints stays linear and only linear.** That is a
different question with a different answer, unchanged here.

## 4. An animation declares what happens when it reaches its end **[proposed]**

| End behaviour | At the last keyframe |
|---|---|
| **Loop** | Restarts |
| **Hold** | Keeps contributing its final values until stopped |
| **Release** | Stops contributing, blending out over its declared duration |

**Because** the owner needs a puppet to transition into a state and stay there with an option to
return, and that is *hold*; a one-off emote that returns is *release*; a breathing idle is *loop*.
Expressed as what happens at the end rather than as a category, because that is the thing that
actually differs.

**Not a type field.** Looping is not a different kind of animation, it is one of three answers to
one question.

## 5. Losing a source rests its parameters at neutral **[proposed]**

**The scene decides what losing a source means; the puppet decides what to do about it.** The
scene owns sources, so interpreting their absence is its job; the puppet owns its animations, so
reacting is the puppet's.

**A parameter whose base driver has no data rests at its neutral keypoint** — not at whatever a
channel value of zero maps to. Every parameter declares exactly one neutral keypoint and that is
its rest; a response curve need not be centred on zero, so *all zeros* would leave a puppet
subtly off-pose.

**The scene's three drop behaviours stand, and this section describes one of them.**
[performing.md](../requirements/performing.md) offers *hold the last pose*, *ease to neutral*
and *play an idle*. Resting at the neutral keypoint is what *ease to neutral* means; *hold the
last pose* suppresses it. An earlier draft of this section read as deleting *hold*, which was
not intended and is the kind of contradiction a reader has to reconcile alone.

**The signal carries no payload.** It says a source is lost and nothing else — no source name,
no port, no address, no scene identity. A puppet file must not be able to learn which trackers
or ports a person has, and that is far cheaper to fix now than once files exist.

**`[open]`: the signal needs a name, and `docs/vocabulary.md` needs it before any code uses
one.** This is the one genuinely new concept in this round, and an unnamed concept is how
implementers invent three words for it.

**`[open]`: with several sources, whether loss is per source or per puppet.** The puppet never
learns which source, so the scene must decide what counts as lost.

## 6. An animation is started by a trigger, and triggers are puppet data **[proposed]**

**Animation configuration lives in the puppet**, alongside channel mappings, for the same reason:
it is how this puppet behaves, and it should not have to be redone per scene or per machine.

These are deliberately separate, and conflating them is how a format grows a state machine:

| | What it is | Where |
|---|---|---|
| **Commands** | Play, pause, stop, seek — the verbs | Runtime |
| **End behaviour** | §4 | Puppet |
| **Triggers** | What issues a command, and when | Puppet |

**A trigger lives on the animation it starts, and is only a start event.** It carries no target,
because the animation that owns it is the target, and no command, because the command is always
*start*. The events are *on load* and *on source lost*.

**Because the shape enforces the scope decision.** A trigger that cannot name another animation
cannot sequence one, so the state machine ruled out above is structurally inexpressible rather
than forbidden by a sentence somebody later reads as advice. It also deletes two enumerated
fields and the two unfamiliar-value questions that came with them.

**Stopping is a rule, not a trigger.** An animation started by an event that can clear is
released when it clears, blending out over its declared duration — so *source regained* needs no
trigger of its own, and a tracker that recovers never leaves an idle layered on top of live
tracking. *On load* does not clear, so an animation it starts runs to its end behaviour.

**A person can always stop a playing animation.** Live's control surface lists what is playing
and what started it, and can stop any of them or all of them, by keyboard, without reloading.
This is a recovery path rather than a state machine: it issues *stop*, it never decides what
plays next. **It is also what stops a `hold` animation**, which otherwise has no exit.

**`[open]`: whether a source may issue commands over its port, and how it would name an
animation.** [performing.md](../requirements/performing.md) already says a source may "trigger
animations", which this section does not yet reconcile. Whatever the answer, a source addresses
an animation by an outward name like a channel, never by a stored identifier
([I3](../invariants.md)).

**A trigger is a selection from a closed set and nothing it carries is evaluated.** No
condition, no comparison, no delay expression, no free string looked up at run time. The first
of those turns a puppet into a program, which [I1](../invariants.md) forbids.

**Tension to resolve, not to drift past:**
[performing.md](../requirements/performing.md) already decides that *what the puppet does when
its source drops* is a **scene** setting. §5 splits that — the interpretation is the scene's, the
reaction is the puppet's — and that split has to be written into that section rather than left
for a reader to reconcile.

## 7. Long animations are ordinary **[proposed]**

**A puppet may carry several animations of a minute or more, at a thousand keyframes and
upwards.** Both extremes must load and play: a long loop holding two keyframes, and a long loop
holding one keyframe per frame.

**Authoring them in Studio is a requirement, not a workaround.** The owner reaches for scripts
today because Inochi's timeline becomes impractical somewhere around a hundred frames. Studio is
expected to do better, and *better* here means a timeline that stays usable at a thousand-plus.

**This is cheap to promise and the reason should be recorded, because it will look expensive.**
Per-frame cost is one curve evaluation **per playing track**, not per keyframe. Keyframes are
storage, plus the cost of finding the active one — a cursor or a binary search, not a scan. The
storage is already bounded by the payload and parse-tree limits.

### A limit stops a hostile file; a budget describes what is tuned for **[decided]**

These are different numbers and conflating them is what made an earlier draft of this section
wrong. A **limit** exists so a file traded between strangers cannot exhaust memory, and is set
from memory safety. A **budget** describes the workload the frame targets are measured against,
and is set from what a rig realistically contains. A puppet may sit inside every limit and
outside the budget: it loads, it plays, and it misses the frame budget on the reference
machine. **That is the author's choice, not a refusal** — the same bargain as opening fifty
browser tabs.

| | Figure | Why |
|---|---|---|
| `KeyframesPerTrack` | raised to 65536 | One keyframe per frame is about eighteen minutes at 60fps. The old figure of 4096 is reached at 68 seconds, which the minute-plus animations in this section already exceed |
| Total keyframes per puppet | a new limit, set from memory with the per-keyframe size stated | Per-track limits multiply: the existing animation, track and keyframe figures together permit hundreds of millions of keyframes, bounded only by the parse-tree cap. The aggregate is the one that decides whether a load can exhaust memory |

**The stress workload carries one animation at extreme density and the rest at corpus density.**
Every one of its animations at one keyframe per frame would be roughly 17 million keyframes —
several hundred megabytes against a player budget of 700 MB that already spends 300 MB on
textures. A budget measured against a workload nobody authors is not a budget, so the extreme
is promised **per animation**, not simultaneously across all of them.

## 8. Studio orders parameters, and the order is kept **[proposed]**

**A rigger arranges the puppet's parameters and that arrangement is stored.** Live presents them
in that order.

**Because** Live's mapping list shows every parameter in an order nobody chose, and finding the
handful that want a channel means scrolling past the rest. This is the owner's experience of
Inochi Session and it is a small fix to a daily irritation.

**An order is not an identity.** Every stored reference is still by identifier
([I3](../invariants.md)); the order is presentation and nothing resolves through it.

## 9. A parameter declares whether it is offered for mapping **[proposed]**

**A parameter carries a mark saying whether Live offers it as a mapping target**, set in Studio
and stored in the puppet.

**Because** a parameter only ever driven by animation is not a candidate for a channel, and no
rule can infer that: under additive composition a parameter may legitimately be both mapped and
animated, so *is it animated* does not answer *should it be offered*. Only the author knows.

**The mark hides, it does not forbid.** It is presentation, not validation — a file whose mark
disagrees with its mappings is not malformed.

## 10. A build that does not understand a value does not guess **[proposed]**

The format's existing rule is that an unfamiliar enumerated value in a file declaring a later
minor version loads, is carried back verbatim, and is **inert wherever it would have been
interpreted** — never replaced by a default, because guessing is the one thing a build with no
code for a value must not do. This round adds enumerated fields, and each needs its answer
stated rather than left to an implementer:

| Field | Inert means |
|---|---|
| Interpolation mode | The **track** contributes nothing. Not the segment alone: a track that contributes across part of its timeline and vanishes across the rest pops at the boundary, which looks like a bug rather than like a missing feature |
| End behaviour | The animation **never starts**. Treating it as loop, hold or release is the guess the rule forbids |
| Trigger event | The trigger **never fires**, and the other triggers on that animation are unaffected |

At the **current** declared version each of these is invalid rather than inert, which is the
existing rule applied unchanged: a writer at this version had the whole list and used something
else, so a typo and an addition are indistinguishable.

**Nothing has shipped, so there is no migration.** The format is at 1.1 and no file exists
outside this repository's own fixtures, so the behaviour change in §1 has no puppets in the wild
to break. The fixtures are regenerated. This is worth stating because it is the reason this
round can change composition semantics at all, and that reason expires at the first release.

---

## Deferred, recorded so it is not rediscovered

| | Why |
|---|---|
| A replacing composition mode for tracks | §1. Add as a format addition where absent means additive, if a case appears that additive genuinely cannot express |
| Mutual exclusion between animations, and sequencing | §6. A state machine belongs to whatever issues commands |
| Interpolation modes between a parameter's keypoints | Unchanged: linear only, and a format addition if ever wanted |
| Whether a source may issue animation commands over its port | §6. [performing.md](../requirements/performing.md) already implies it may; nothing here settles how an animation would be named |
| Whether Studio predicts clamping statically, across a whole timeline | §2. Studio has no tracking input, so only the animation extent is knowable from the file. The playhead case is cheap and is in scope; the whole-timeline case is `O(duration × tracks)` and is not |
