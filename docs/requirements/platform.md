# Requirements — platform and frame budget

What this must run on, and how well. Cross-cutting: it constrains both applications and the
runtime they share.

## Support tiers **[proposed]**

| Tier | Platforms | Means |
|---|---|---|
| Supported | Linux, Windows | Tested before release. A defect here blocks a release |
| Best-effort | macOS | Built and released. Tested only by whatever is automatable and by users |

macOS is wanted and is not worth a significant amount of work. The way to get it cheaply is to
keep it a **build target rather than a port**: choose a portable GPU abstraction and avoid
platform-specific paths in the runtime, so that macOS costs packaging rather than
reimplementation. Where that stops being cheap — platform-specific output plumbing, code
signing and notarisation — macOS is `[open]` and may be dropped.

**Constraint, with its reason:** there is no Mac available to test on. Automated checks on
hosted macOS runners can verify that it builds and that structural assertions pass; they cannot
verify window compositing, transparency against a real compositor, or whether it feels right.
Any macOS claim stronger than "it builds and the tests pass" is unfounded, and the tier above
says so honestly rather than implying coverage that does not exist.

## The speed requirement, stated as a budget **[proposed]**

The goal in plain terms is *smooth on a weak machine, while leaving room for everything else the
machine is doing*. Both halves matter, and the second is the one usually dropped: this software
never runs alone. It runs beside a game and beside an encoder, and those have the user's
attention.

So the budget is not "fast enough" but **a fixed share of a defined machine under a defined
load**.

**The requirement falls on the player, not equally on both. [decided]** The player runs while
someone is streaming in front of an audience, and a dropped frame there is visible and unrecoverable.
The editor is a tool someone sits in front of; an operation that takes a moment longer is
annoying, and so is a slow computer in general. Where the two conflict — a shared runtime
decision that helps one and costs the other — the player wins. The editor's figures below are
targets; the player's are the requirement.

### The reference machine **[proposed]**

A beginner streamer's computer, not a developer's. Named concretely, because a budget against an
unnamed machine cannot be reproduced or bought:

| | |
|---|---|
| CPU | Intel Core i5-8250U, or AMD Ryzen 5 3500U |
| GPU | Its integrated graphics — Intel UHD 620, or AMD Vega 8 |
| RAM | 8 GB, **dual channel** — integrated graphics is bandwidth-bound and single-channel roughly halves it |
| Storage | SATA SSD |
| Output | 1080p at 60 Hz |
| Power | Mains, after thermals have settled. Battery and cold-start numbers are not the budget |

**The co-tenant load is part of the machine definition.** Budgets are stated twice: once solo,
and once against a fixed synthetic CPU and GPU load standing in for a game and an encoder. The
contended figure is the one that reflects reality; the solo figure is the one that localises a
regression.

### Budgets are per puppet **[decided]**

Every figure below is stated against **one** puppet. A scene holding several costs roughly its
share for each, and that is the expected and acceptable outcome — the same bargain as opening
more browser tabs. Live is benchmarked on one puppet of average size rendering smoothly; it does
not promise a fixed ceiling across a whole scene, and it does not degrade a puppet to stay
inside one.

**Because:** a whole-scene budget would make the common case pay for the rare one, and the
number it produced would be arbitrary. Per-puppet costs are the ones a user can reason about
and act on.

### The reference puppet and the stress puppet **[proposed]**

A budget needs a defined workload, and on integrated graphics the dominant costs are fill rate,
offscreen passes and memory bandwidth rather than triangle count. The workload is therefore
described by what actually drives cost.

Calibrated against a corpus of freely available rigs — Live2D Cubism's own sample models and
inochi2d's example models by an experienced rigger — measured in
[2026-11-26-puppet-corpus.md](../research/2026-11-26-puppet-corpus.md). **The reference sits at
the corpus median; the stress sits above the largest rig measured on every dimension.**

| | Reference | Stress | |
|---|---|---|---|
| Parts | 85 | 300 | Drawn parts. **The comparable figure across formats** |
| Nodes | 190 | 500 | Every tree entry, drawing or not |
| Vertices | 3,200 | 9,600 | |
| Triangles | 4,700 | 14,100 | |
| Parameters | 42 | 150 | |
| Bindings | 260 | 780 | **Understated.** See below |
| Masks and composites | 11 | 80 | Measured as mask references in the corpus, which is a narrower unit |
| Offscreen passes per frame | 11 | 80 | Follows the row above, though a mask and an offscreen pass are not the same thing |
| Shaded pixels per frame | 3 M | 9 M | Not derivable from a file; belongs to the runtime counters |
| Physics nodes | 12 | 40 | |
| Animations | 8 | 32 | The one row comfortably above the corpus at both ends |
| Source artwork | 25 Mpx | 75 Mpx | |
| GPU-resident texture bytes | 100 MB | 300 MB | |

**The node row is only meaningful because we define the convention.** Cubism rigs through
hierarchies of deformers that draw nothing; inochi2d does comparable work with per-part
bindings. Node counts are not comparable across formats, so *parts* is the row to reason with
and *nodes* is a figure for our own tree.

**Bindings are understated.** One Cubism keyform set carries deformation, opacity and draw
order together where inochi2d counts three bindings, so the corpus measurement undercounts the
Cubism half and the true spread is wider than the row shows.

**The texture rows are the weakest in the table.** The Cubism samples cannot bound them — each
ships one or two 2048² packed atlases, which is an export setting, and the layered source never
leaves the artist's file. The honest datapoint is inochi2d's unpacked per-part artwork at
20–23 Mpx and 78–87 MB resident, which makes the reference figures plausible and the stress
figures **a deliberate headroom choice with no evidence behind them.** Treat them as the least
trustworthy numbers here.

**Nothing in the corpus is a rig someone paid for**, which is the population this project
actually serves and the one systematically unobtainable for free. Live2D's samples are built to
demonstrate features, so they use more deformers, parameters and masks than a rig built to do a
job; part of the spread is also a decade of format growth rather than rigger skill. One
commissioned rig, obtained by asking a rigger directly, would be worth more than the whole
corpus, because it is the only way to learn whether these samples are the ceiling of commercial
work or the floor.

Both workloads are produced by a **committed generator** taking the declared dimensions above, not
authored by hand — necessarily, because authoring one needs an editor and the editor is built
after the runtime that these budgets constrain. A generated puppet also doubles as the
conformance fixture proving a third party can write a valid puppet without our code
([puppet.md](puppet.md)).

### The budgets **[proposed]**

Frame time, deadline miss, steady state and counter are defined in
[vocabulary.md](../vocabulary.md). Two consequences of those definitions drive the table: CPU
and GPU time are reported separately because on integrated graphics they contend for one pool,
and no warm-up is excluded from steady state because a hitch in the first minute of a stream is
still a hitch on stream.

| Budget | Solo | Contended | Why this one |
|---|---|---|---|
| Frame time, 99th percentile | 8 ms | 12 ms | Half the 16.6 ms frame solo. The remainder is the headroom requirement, made checkable |
| Deadline misses | 1 per hour | 60 per hour | **Frame pacing, not average frame rate.** A maximum-frame-time budget is unachievable under OS scheduling noise and so would be ignored; a miss rate is both honest and measurable |
| CPU per frame | 4 ms | 6 ms | Per frame rather than as a share of the machine, so that it binds |
| Resident bytes, player | 700 MB | | RAM and GPU memory are one pool on this machine, so they are budgeted as one number |
| Resident bytes, editor | 2 GB | | With the reference puppet and its source artwork open, on an 8 GB machine |
| GPU resource creation or pipeline compilation in the frame loop | None | | **Invariant — see `docs/invariants.md` I5.** A mid-frame texture or shader compile is a visible hitch, and on some drivers a stall of hundreds of milliseconds |
| CPU allocation in the frame loop | None | | A budget rather than an invariant. The language is now chosen ([stack.md](../stack.md)) and makes zero natural, and a counting global allocator makes the counter exact rather than approximated — so this is now a candidate for promotion to an invariant, which is the project owner's call and not a design one. A pause from allocation or collection is exactly the stutter the percentile budgets exist to prevent |
| First-use work after the first presented frame | None | | The broader form of the row above, and how I5 is actually asserted: every render state a puppet can reach is prepared before it is first needed, or paid for during load |
| Startup to first frame | 3 s cold, 1 s warm | | Cold means every derived cache empty, which is every beginner's first run |
| Puppet load | 2 s | | Including validation and texture upload |
| Input latency | 2 frames | | From a packet arriving to the frame it affects. Latest value wins; the queue never grows |

The editor's responsiveness splits by interaction kind, because one figure is wrong for two of
these three:

| Interaction | Budget |
|---|---|
| Continuous manipulation — dragging a vertex, scrubbing a parameter or the timeline, panning the viewport | Display rate. 100 ms here is unusable |
| Discrete command — paste, mirror, undo | 100 ms |
| Long operation — automatic meshing, PSD import, save | Progress shown within 100 ms, cancellable, off the interaction thread. Cancelling leaves the puppet unchanged |

Both applications idle at near-zero cost when nothing is changing, and the player never renders
faster than its output is consumed. Free-running at the GPU's maximum rate to produce frames
nobody sees is a defect, not speed.

### Cost grows no worse than linearly **[proposed]**

Each per-frame counter, measured on the stress puppet, is no more than its reference value
multiplied by the growth in **the workload dimension that drives it** — vertices for
deformation work, parts for draw calls, masks for offscreen passes. Stated per dimension rather
than as one figure, because the table above does not scale uniformly: masks grow by observation
and the rest by a multiple.

This is what catches accidental quadratic behaviour in deformation composition and physics,
which is the failure mode that makes a tool pleasant on small rigs and useless on real ones.

## How this is verified **[proposed]**

Wall-clock timing is not reproducible across machines, so the budgets split into an asserted
half and a recorded half. Both halves must carry weight: a split that puts every number in the
recorded half gates nothing.

**The runtime exposes per-frame counters headlessly**, computable from a frame's command stream
without a GPU present. This is a requirement on the runtime's design and not an afterthought: it
is what lets hosted CI, which has no real GPU, assert the things that cause the timing problems.

| Counted per frame | Counted per operation |
|---|---|
| Draw calls, state changes, offscreen passes | Puppet load steps and bytes read |
| Vertices deformed, binding evaluations | Bytes written per save |
| Bytes uploaded, estimated shaded pixels | Bytes retained per undo step |
| Heap allocations, GPU resource creations | Peak bytes during PSD import |
| Name resolutions — must be zero | |

Each counter has a ceiling in a **committed baseline file**, derived from one calibration run on
the reference machine. CI fails on any counter above its baseline unless the baseline changes in
the same commit with a stated reason. This is the primary gate, and it is exact.

Nothing resolves anything by name inside the frame loop. Channel names and parameter names
resolve to direct references when a source connects and when a puppet loads; animation lookup
seeks rather than scanning. The name-resolution counter being zero is how this is enforced
rather than remembered.

**No reference machine is available for routine testing.** A weaker laptop can be borrowed on
a small number of occasions, late in development. This is a constraint with consequences that
must be stated rather than glossed:

- The wall-clock half of the strategy **does not run per commit**. There is no continuous
  frame-time series and no routine contended benchmark; the figures in the budget table are
  targets that nothing checks day to day.
- What occasional access buys is **calibration, not regression detection**. The right use of a
  rare sitting is to measure the counter-to-milliseconds relationship once — establishing what
  the per-frame counter ceilings actually cost in time on real hardware — so that the counters
  CI *can* run stop being arbitrary and start standing in for the budget. Spending a rare
  sitting hunting a regression instead would waste it.
- **This means the counter ceilings should be set provisionally and revised at the first
  sitting on real hardware**, rather than treated as settled now.
- **The per-frame counters therefore carry the entire requirement.** This is workable — they
  are exact, they catch the regressions that cause timing problems, and they run in CI without
  a GPU — but they measure work done, not time taken. A change that makes each unit of work
  slower without doing more of it passes every check.
- Consequently **the claim "runs well on weak hardware" is unverified** until someone with such
  hardware reports otherwise. The honest form of the claim is "designed to a budget, with the
  work-done half of that budget enforced".

The cheapest way to close this is a secondhand machine of the class above. For a project whose
premise is weak-hardware operation, it is the highest-value hardware there is to own, and the
gap above is what its absence costs.

A **soak run** at accelerated time, headless, with synthetic input, shows counters and resident
bytes flat over a stream's length. This is what catches the leak that only matters after three
hours, which is exactly when it is most expensive to discover.

Per [testing.md](../testing.md) and `.claude/rules/testing.md`: the counter half carries the
requirement, and judgment covers what it cannot reach. **This strategy belongs in
[testing.md](../testing.md)** once that document is written; it is stated here because that one
is still an empty template.

## English only **[decided]**

User-facing text is English, and localisation is out of scope. No translation layer, no locale
files, no string catalogue built for extraction.

**Because:** the project's stated trade is depth over breadth, and a second language multiplies
the surface of every interface change for an audience the project has not yet got. This is a
scope decision rather than a technical one, and it is reversible at a real but bounded cost —
which is the reason not to build the machinery for it now.

It does not license the opposite mistake: text still lives where a person can find it rather
than being scattered through the interface code, because the user-facing string lint that
enforces the strict vocabulary ([decisions.md](../decisions.md)) needs somewhere to look.

## Behaviour under contention and over budget **[open]**

Three connected questions, none decided, and nothing may assume an answer to any of them:

- Under contention, whether Live **yields** — dropping frames so the game wins — or **holds**,
  so the stream stays smooth and the game suffers.
- When a puppet exceeds the budget on a weak machine, whether Live **degrades, warns or
  refuses**, and whether it may hold textures below their authored resolution. This interacts
  with *no arbitrary dimension ceilings* in [puppet.md](puppet.md). Distinct from a **load
  limit**, which is a defence against a hostile file rather than a cap on authoring.
- The tolerated deadline-miss rate and the stream length the soak run targets. Both figures in
  the budget table above are guesses and are marked as such.

## Windows code signing **[open]**

A certificate costs money. Whether to fund one or ship unsigned with published checksums is
undecided. macOS notarisation is `[open]` for the same reason, under *Support tiers* above.

