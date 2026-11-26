# Open questions

The agenda: what requirements work is waiting on, grouped by what it blocks. Deleted when the
requirements are agreed and the phase plan is written.

**This file carries no content.** Each line names a question and points at the `[open]` section
that holds it — that section is the question's home, because it keeps the context the question
needs and it survives this file being deleted. `scripts/docs-check.sh` lists every `[open]` tag
in the documentation set, including the ones inside otherwise-settled sections; this is the
subset that needs a decision from the project owner rather than from design work.

---

## Waiting on the owner

| Question | Lives in |
|---|---|
| How a streamer learns something broke, when the render surface stays clean and the control surface may be behind a game | `requirements/performing.md` — *How a streamer learns something broke* |
| Whether a scene can be edited while streaming, and whether there is a locked state | `requirements/performing.md` — *Editing a scene while streaming* |
| Whether Studio keeps rolling backups, and how many | `requirements/authoring.md` — *Rolling backups* |
| What form crash recovery takes — a prompt, an autosave journal, or both | `requirements/authoring.md`, inline |
| Whether Live yields or holds under contention; whether it degrades, warns or refuses over budget; the deadline-miss rate and soak length | `requirements/platform.md` — *Behaviour under contention and over budget* |
| Whether Windows code signing is funded, and whether macOS notarisation follows | `requirements/platform.md` — *Windows code signing* |

## Waiting on design work, not on an answer

| Question | Lives in |
|---|---|
| Which platforms need an OBS plugin | `requirements/performing.md`. Deferred to the spike deliberately; the spike is the trigger |
| How a dormant scene entry is relocated in practice | `requirements/performing.md`. Shape settled, mechanism open |
| Composition modes for deformations | `requirements/puppet.md`. Waits on evidence about which are wanted |
| Randomness, and first run | `requirements/authoring.md` |

## Waiting on a signal, not a decision

| | |
|---|---|
| When the format compatibility promise becomes hard | `requirements/puppet.md`. It starts when the project is shared with a community, and the owner says when |

## Due before plugins ship, not before the first release

Opened by the plugin decision in `decisions.md`. No requirements document owns these yet,
because the API surface is deliberately undesigned.

| | |
|---|---|
| What a plugin can do, and where it runs | Revisit once the deformation model and the operation set have stopped moving |
| Whether *never opens an outbound network connection* means the applications, or everything they load | The claim is asserted in CI over our own code, and a plugin is not our code. Same shape as the no-screen-capture claim |
| How a user learns what would make an unrecognised extension work | The warning on open names what is missing; until plugins exist it must not imply an install route |

## Carried, not blocking

| | |
|---|---|
| Read inochi2d's source to see how its bindings actually compose | Deferred by the owner, not dropped. Composition is the named weak point and the incumbent's approach is the most useful remaining input |

## Dimensions not yet swept

`accessibility` and `ops` were held back deliberately, pending the stack. **The stack is now
chosen ([stack.md](../stack.md)), so both are unblocked** and should be swept before the phase
plan is written. The accessibility sweep has a specific thing to bite on: the custom-canvas
registration convention in [testing.md](../testing.md) is what its entries would verify.
