# Open questions

The agenda: everything still open, grouped by what each one waits on.

**Kept past its deletion date on purpose.** [documentation.md](../documentation.md) retires it
once the requirements are agreed and a plan exists, which is now. What is being kept is the
grouping: `scripts/docs-check.sh` lists every `[open]` tag but cannot say which of them anyone
but the owner can answer. It goes when the first group below empties.

**This file carries no content.** Each line names a question and points at the `[open]` section
that holds it — that section is the question's home, because it keeps the context the question
needs and survives this file being deleted. This file adds only the grouping.

---

## Waiting on the owner

| Question | Lives in |
|---|---|
| How a streamer learns something broke, when the render surface stays clean and the control surface may be behind a game | `requirements/performing.md` — *How a streamer learns something broke* |
| Whether a scene can be edited while streaming, and whether there is a locked state | `requirements/performing.md` — *Editing a scene while streaming* |
| Whether Studio keeps rolling backups, and how many | `requirements/authoring.md` — *Rolling backups* |
| What form crash recovery takes — a prompt, an autosave journal, or both | `requirements/authoring.md`, inline |
| Whether Live yields or holds under contention, and whether it degrades, warns or refuses over budget | `requirements/platform.md` — *Behaviour under contention and over budget*. Live's behaviour, and not needed until Live is built |
| Whether Windows code signing is funded, and whether macOS notarisation follows | `requirements/platform.md` — *Windows code signing* |

## Waiting on design work, not on an answer

| Question | Lives in |
|---|---|
| Where the independent second list comes from that lets the custom-canvas test fail | `testing.md` — *The convention custom canvases need*. `concerns/accessibility.md` names the canvas's own hit-test list as the intended source; nothing is built. The other half of this question, how dense elements are summarised, was settled by the keyboard exemption — vertices are no longer registered at all |
| What the cross-platform determinism tolerance is | `requirements/puppet.md` — the determinism section. Bit-exactness is claimed per platform; across platforms the figure is unset |
| How a dormant scene entry is relocated in practice | `requirements/performing.md`. Shape settled, mechanism open |
| How many tracks an animation carries, and how many keyframes a track carries | `requirements/platform.md`'s workload table. The corpus counted `.motion3.json` files rather than the curves inside them, so the dimension has no figures and no load limit covers it. Reading curve counts out of the same corpus is a research task, not a decision |
| Composition modes for deformations | `requirements/puppet.md`. Waits on evidence about which are wanted |
| Randomness, and first run | `requirements/authoring.md` |
| How `ART-19` is restated now that its premise is gone | `requirements/puppet-acceptance.md` — ART-19. Belongs to the phase that owns Studio editing |
| What relates a keypoint's coordinate count to its parameter's axis count, and a keyframe's value count to the same | `format.md` — *Validation*. No rule exists, so the validator zips the two and a surplus coordinate is checked against no range at all |
| What textual form the writer spells a number in | `format.md` — *Canonical serialisation*. Unstated, so byte-identical round-tripping is checkable only against our own writer's choices |

## Waiting on a signal, not a decision

| | |
|---|---|
| When the format compatibility promise becomes hard | `requirements/puppet.md`. It starts when the work is shared with a community, and the owner says when |
| Whether a texture has a pixel-area limit, and what figure | `format.md` — *Load limits*. Each side is capped and the product is not, so a texture at the maximum on both is a gigabyte of pixels once uploaded. **Deferred by the owner on 2026-11-28**: make the tool work, then stop people breaking it. The signal is a build that actually uploads a texture, not a date |

## Due before plugins ship, not before the first release

Opened by the plugin decision in `decisions.md`. No requirements document owns these yet,
because the API surface is deliberately undesigned.

| | |
|---|---|
| What a plugin can do, and where it runs | Revisit once the deformation model and the operation set have stopped moving |
| Whether *never opens an outbound network connection* means the applications, or everything they load | The claim is asserted in CI over our own code, and a plugin is not our code. The ops sweep raised the same gap for ordinary dependencies, which run in the binary too — a refinement for the security dimension |
| How a user learns what would make an unrecognised extension work | The warning on open names what is missing; until plugins exist it must not imply an install route |

## Carried, not blocking

| | |
|---|---|
| Read inochi2d's source to see how its bindings actually compose | Deferred by the owner, not dropped. Composition is the named weak point and the incumbent's approach is the most useful remaining input |

## Not on this list

`scripts/docs-check.sh` derives the full `[open]` set from the tags and is the complete list.
This file is narrower on purpose: it carries the questions that are *waiting on someone*, so
that the first group above can be read as an agenda. A question that is open but not blocked —
the runtime's colour convention, the stack's remaining exposures, several rows of
[puppet-acceptance.md](../requirements/puppet-acceptance.md)'s *Criteria not written* — is real
and is not here.

**The ones that gate the next subsystem are in [plan.md](plan.md)**, under *What blocks S2*.
