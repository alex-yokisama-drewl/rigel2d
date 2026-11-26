# Open questions

The agenda: everything still open, grouped by what each one waits on.

**This file was due for deletion once the phase plan was written, and is deliberately kept.**
[documentation.md](../documentation.md) says it goes when the requirements are agreed and the
plan exists, which is now. It survives because the grouping below is the thing being kept: the
questions waiting on the project owner are still waiting, and `scripts/docs-check.sh` lists every
`[open]` tag without saying which of them anyone can answer. It goes when that group empties.

**This file carries no content.** Each line names a question and points at the `[open]` section
that holds it — that section is the question's home, because it keeps the context the question
needs and it survives this file being deleted. This file adds only the grouping.

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
| Which data the format declares authoring-only | `requirements/puppet.md` — *One file*. That the format declares it is settled; the membership is a format design question for the format subsystem. **The mirror axes are the first settled member**, declared authoring-only where they are introduced in *What a puppet is* |
| Whether a `Physics nodes` row in a counter table makes physics a node | `requirements/platform.md`'s workload table counts them as nodes; `vocabulary.md` and the pieces list in `requirements/puppet.md` say the node kinds are parts, mesh groups and composites. The deletion rule no longer depends on the answer, but the two documents still disagree |
| How a dormant scene entry is relocated in practice | `requirements/performing.md`. Shape settled, mechanism open |
| Composition modes for deformations | `requirements/puppet.md`. Waits on evidence about which are wanted |
| Randomness, and first run | `requirements/authoring.md` |

## Waiting on a signal, not a decision

| | |
|---|---|
| When the format compatibility promise becomes hard | `requirements/puppet.md`. It starts when the work is shared with a community, and the owner says when |

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

## Dimensions swept

Every dimension in [concerns/](../concerns/) has now been swept. `accessibility` and `ops` were
held back pending the stack and were swept on 2026-11-26, once [stack.md](../stack.md) settled.
Their entries are approved and in place.

The accessibility sweep bit where this file predicted it would: the custom-canvas registration
convention in [testing.md](../testing.md) turned out to describe a test that cannot fail, which
is the one piece of it still open above.

**What the sweeps settled, rather than opened.** The owner's answers on 2026-11-26 closed the
questions about release packaging and the operating-system floors, who tests Windows and on
what, whether there is a local log and a diagnostics report, what a save does when the file
changed underneath it, what a relaunch restores after a crash during a load, whether a fallback
to defaults may overwrite, and whether application configuration is inside the one write path.
Each now lives as settled text in the document that owns it.

**One answer narrowed a claim rather than widening one.** Canvas manipulation does not have to be
keyboard-operable, which means the project does not claim WCAG AA overall — it claims it for the
chrome and states the exemption, in the shape [platform.md](../requirements/platform.md) uses for
the macOS tier. That is recorded in `concerns/accessibility.md`.
