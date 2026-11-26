# Phase 1 — the spike and the runtime

The current phase: which subsystems get built, in what order where order matters.

**Status: agreed 2026-11-26.** Nothing is built. S0 is in flight; every other subsystem is
not started.

## Where this phase ends, and why there

This phase ends at the runtime. It does not reach Studio or Live.

[README.md](../README.md) records that Studio's and Live's own designs are deliberately not
written — the design pass covered the format, the evaluation pass, the scene, the runtime's
boundaries and the stack, in the build order [overview.md](../overview.md) sets out. There is
nothing to decompose the two applications from, and a plan that listed them would be listing
guesses. Studio's design is a `tech-design` pass in the next cycle, and the concern entries this
phase seeded are waiting for it.

What this phase delivers is the library both applications embed, proven against generated
workloads, with the one question that could reopen the stack settled first.

## Subsystems

Ordered. `Blocks on` names what genuinely must come first; everything else may move.

| | Subsystem | Done when | Blocks on |
|---|---|---|---|
| S0 | **Transparency and alpha-output spike** | A known half-transparent frame has gone through each candidate route into a real OBS on Linux and Windows, and the pixel values OBS emits are asserted — premultiplied against straight alpha explicitly among them. macOS is attempted at its tier. The `[open]` in [performing.md](../requirements/performing.md#which-platforms-need-an-obs-plugin-open) is resolved, the Game Capture capability tension is answered, and a dated file lands in [research/](../research/) | Nothing |
| S1 | **Workspace and the push gates** | Every crate in [stack.md](../stack.md#workspace-proposed)'s workspace table builds from a clean checkout, the push gates run, `scripts/test-record.sh` records a result, and [stack.md](../stack.md#running-locally-proposed)'s *Running locally* steps are corrected to what was actually walked — including the Linux package list it asks for | S0 |
| S2 | **`rigel-format`: read, validate, write** | A puppet archive round-trips byte-identically; the validator rejects a hand-written malformed corpus and accepts stale bindings as a state; load limits are enforced before allocating; extension data survives untouched; every write goes through the one atomic path | S1 |
| S3 | **The workload generator and the fixture corpus** | The reference and stress puppets are generated from the committed dimension table in [platform.md](../requirements/platform.md), both validate, and the generated puppet doubles as the conformance fixture [puppet.md](../requirements/puppet.md) requires | S2 |
| S4 | **The evaluation pass** | A generated puppet evaluates to a posed tree for given channel input, in the order [evaluation.md](../evaluation.md) sets; one parameter has one driver, enforced by the validator; stale bindings behave as specified; the name-resolution counter is zero | S2 |
| S5 | **The render stage: command stream and backend** | Evaluation output produces an explicit command stream countable with no GPU present; the wgpu backend submits it; the warm-up pass begins every configuration the loaded puppet can reach, enumerated from puppet data; GPU resource creations after the first presented frame are zero on both workloads | S3, S4 |
| S6 | **The counter gate, growth and soak** | Counters have a committed baseline, CI fails above it, growth is asserted per dimension on the stress workload, and the soak shows counters and resident bytes flat | S5, and the soak question below |

### What does not block what

S3 and S4 both depend only on S2 and are independent of each other. S3 is placed first because it
gives S4 its fixture, not because S4 needs it to exist. Ordering them the other way costs only a
hand-built fixture.

S0 blocks everything, and that is the point rather than an inconvenience — see below.

### Why S0 is first, and what it may cost

[stack.md](../stack.md#what-this-choice-is-still-exposed-to-open) names transparency as the first
of what the stack choice is still exposed to, and is explicit that nothing should be built
assuming it has gone well. It has not been observed working on any platform; every claim about it
is drawn from reading source and documentation. It gates the OBS delivery path
([performing.md](../requirements/performing.md)) and the stack's own windowing assumption at the
same time, and one piece of work settles both.

**If it fails, the stack choice reopens** and S1 onward are built on a different foundation. So
S0 carries its own throwaway scaffolding rather than waiting for S1 to formalise a workspace that
may not survive it. S1 then formalises whatever S0 established.

S0 is a spike. It produces evidence and a resolved `[open]`, not production code, and its
scaffolding is expected to be discarded.

## Acceptance criteria

Commissioned from `acceptance-planner` per area, which must not be shown an implementation.
Timing differs by area because the `[open]` sections that would change the criteria resolve at
different points.

| Area | When | Why then |
|---|---|---|
| [puppet.md](../requirements/puppet.md) | **Done** — [puppet-acceptance.md](../requirements/puppet-acceptance.md) | Its `[open]` on the compatibility promise is a signal from the owner and changes no criterion |
| [platform.md](../requirements/platform.md) | With S6 | The counter half can be commissioned now; the soak length and the tolerated deadline-miss rate are `[open]` |
| [performing.md](../requirements/performing.md) | After S0 | The spike resolves the `[open]` that decides the output path |
| [authoring.md](../requirements/authoring.md) | **Partly done** — the operations settled on 2026-11-26 are in [authoring-acceptance.md](../requirements/authoring-acceptance.md); the rest is next cycle | Studio is not in this phase |

The planner's most useful output is the gaps it finds in the requirements. Those route back into
`docs/requirements/` before the subsystem they affect starts.

**S2 is gated on the gaps the puppet criteria found, not just on S1.** Writing criteria against
[puppet.md](../requirements/puppet.md) turned up specification defects that an implementer would
otherwise settle by accident — including two `[decided]` sections that contradict each other on
whether a newer version's unmarked core field opens or is rejected. They are listed in the
closing table of [puppet-acceptance.md](../requirements/puppet-acceptance.md). **Resolving them
is requirements work and comes before S2 starts**; it is the reason the criteria were
commissioned early rather than written alongside the code.

**The criteria and the tests separate here, and on a greenfield phase they must.** The
`plan-phase` lane closes on *acceptance criteria and tests exist and have been confirmed
failing*. A criteria document can be written now, from the requirements alone. A runnable test
cannot: there is no crate to put it in and no command to run it until S1 exists, so nothing can
be confirmed failing before then. The split is therefore criteria now, tests once S1 lands — and
S1's own completion includes a test suite that runs and a recorded result, which is what makes
"confirmed failing" mean anything.

## Due before the subsystem that needs them

Not blocking the plan. Each is due before its subsystem starts, and named here so it is not
discovered late.

| Question | Due before | Lives in |
|---|---|---|
| The tolerated deadline-miss rate, and the stream length the soak targets | S6 | [platform.md](../requirements/platform.md) — *Behaviour under contention and over budget*. The other two bullets in that section are Live's behaviour and belong to a later cycle |
| Whether Studio refuses or asks when a puppet changed on disk since it loaded | S2 | Unowned. Live writes puppets for channel mappings ([runtime.md](../runtime.md)) while Studio may hold the same puppet open. No document covers the collision |
| Whether configuration is inside the one write path, and how logs are exempted from it | S2 | [runtime.md](../runtime.md) — *One write path* names puppet, scene and autosave only |

## Carried out of this phase

Seeded by the planning sweeps on 2026-11-26 and not actionable until the applications have a
design. They are recorded so the Studio cycle inherits them rather than re-deriving them.

- The accessibility entries, which bite on Studio's custom canvases and Live's control surface.
- The ops entries covering packaging, release and failure on a user's machine.
- The documentation defects both sweeps found, which are being routed separately and are not
  waiting on this plan.
