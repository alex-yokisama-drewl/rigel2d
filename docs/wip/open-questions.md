# Open questions and pending approvals

What requirements work is waiting on. Deleted when the requirements are agreed and the phase
plan is written; nothing here is permanent.

What remains is **questions for the user** and **the proposed invariants**. The concern
checklist entries that used to sit here have been approved and moved into `docs/concerns/`.

---

# Questions

Grouped by what they block. The `[open]` sections in `docs/requirements/` point here.

## Blocking the file-layout decision

*Resolved 2026-11-26: a puppet is one file, recorded in `docs/decisions.md`. The source document
stays outside it, which disposes of the delivery question too.*

What remains, and is about rig data rather than file layout:

| Question | Why it matters |
|---|---|
| When replacement art has different dimensions, or a mesh is regenerated, what happens to existing deformations — remap, keep and flag, or discard with an undoable warning? | Deformations are tied to mesh topology and texture coordinate space. Getting this wrong destroys rigging work |
| Should the editor keep rolling backups of previous saves, and how many? | |

## Blocking the format compatibility promise

*Resolved 2026-11-26, after a reversal the same day. A build opens a file carrying data it does
not understand, stays editable, warns on open naming what is missing, and writes the unknown
data back untouched. Extension data must be explicitly marked by whoever wrote it, so the
validator can tell an extension from a malformed file. The hard promise does not start at a
version number — it starts when the project is shared with a community, and the owner will say
when. Compatibility is a design consideration on every 0.x release meanwhile. In
`docs/requirements/puppet.md`, with both decision entries in `docs/decisions.md`.*

**This raised a question that is now the largest thing open — see "Blocking the extension
story" below.**

## Deformation model — largely settled

*2026-11-26. `docs/requirements/puppet.md` carries the position as `[proposed]` sections: no
general skeleton; composition deterministic now with modes `[open]`; a chain deformer built as
a permanent structure rather than a bulk-edit tool. They reflect the owner's position and are
in force.*

| Carried to a later session | |
|---|---|
| Read inochi2d's source to see how its bindings actually compose | Deferred by the owner, not dropped. Composition is the named weak point and the incumbent's approach is the most useful remaining input |

## Blocking the player's design

| Question | Why it matters |
|---|---|
| How does a streamer learn something broke, when the render surface must stay clean and the control surface may be behind a full-screen game? | A sound, an OS notification, a tray state. Without an answer, a live failure is silent |
| Is OpenSeeFace's protocol our native protocol, or one source implementation behind a protocol of our own? | Decides whether the port is reachable from a web page, which decides whether it needs origin checks |
| Can a scene be edited while live? Is there a locked state? | |

*Resolved 2026-11-26: what the puppet does when its source drops is a scene setting, choosing
among hold, ease to neutral, and idle animation, defaulting to ease to neutral — in
`docs/requirements/performing.md`.*

*Deferred 2026-11-26, deliberately: the Linux OBS plugin choice waits on the spike, because
alpha is unobserved end to end on every candidate path. The spike is the trigger. Output behind
a narrow interface is settled regardless.*

## The extension story

*Resolved 2026-11-26: plugins are intended; the API surface is deferred. Groundwork in scope now
is only what cannot be retrofitted — marked extension data, stable identifiers, an enumerable
operation set, and the permanent rule that a plugin never arrives inside a puppet, scene or
source. In `docs/requirements/authoring.md`, superseding the no-plugin-API entry in
`docs/decisions.md`.*

Not blocking anything now, and **due before plugins ship rather than before the first release:**

| Question | Why it matters |
|---|---|
| What can a plugin do, and where does it run? | The API surface. Deferred on purpose; revisit once the deformation model and the operation set have stopped moving |
| Does *"never opens an outbound network connection"* mean the applications, or the applications plus everything they load? | The claim's whole justification was that it is checkable in one sentence and asserted in CI. A plugin is not our code, so the assertion's scope has to be restated or plugins constrained. Same shape as the no-screen-capture claim |
| How does a user learn what would make an unrecognised extension work? | The warning on open names what is missing. Until plugins exist it cannot promise an install route, and the wording should not imply one |

## Blocking scope and the threat model

*Answered 2026-11-26: a pen tablet is not a target device. Every operation in Studio must be
**convenient** with mouse and keyboard, not merely possible — the audience requirement, since a
workflow that is only tolerable without a tablet excludes the people this project is for.
Tablet-specific interaction becomes a candidate for the plugin API once that exists. In
`docs/requirements/authoring.md`.*

*Answered 2026-11-26: off-machine sources are deferred, not rejected — the first release binds
to loopback and does nothing else, and the design must not preclude adding them. The shape is
settled for when they arrive (explicit in-app opt-in, never on by default, visible while on);
whether the indicator alone suffices or an allow-list or pairing step is wanted is decided then.
In `docs/requirements/performing.md`, which also resolves a contradiction — that section was
`[decided]` while its own Deferred list and this document both called it open.*

*Answered 2026-11-26: the project is **Rigel2D**; the applications are **Studio** and **Live**,
in `docs/vocabulary.md` and `CLAUDE.md`. UI text is English only — no localisation in scope.*

*Answered 2026-11-26, now in `docs/decisions.md`: no in-puppet scripting, ever; no outbound
network connection; the interface uses the project's strict vocabulary. The no-plugin-API half
of that entry has since been superseded — see "The extension story" above.*

## Blocking the speed verification strategy

*Answered 2026-11-26: no machine is available for routine testing, though a weaker laptop can
be borrowed on a few occasions late in development. The consequences — that the counters carry
the requirement day to day, that the weak-hardware claim is otherwise unverified, and that a
rare session is best spent calibrating counters against milliseconds rather than hunting
regressions — are recorded in `docs/requirements/platform.md`.*

| Question | Why it matters |
|---|---|
| Under contention, should the player yield — drop frames so the game wins — or hold, so the stream stays smooth and the game suffers? | |
| When a puppet exceeds the budget on a weak machine, does the player degrade, warn, or refuse? May it hold textures at lower resolution than authored, and who decides? | Interacts with "no arbitrary dimension ceilings" in `puppet.md` |
| How many missed frames per hour is acceptable, and what stream length is the soak target? | Both figures in the budget table are guesses |
| Windows code signing costs money. Ship unsigned with checksums, or fund a certificate? | macOS notarisation is already `[open]` |

---

# Proposed invariants, awaiting approval

`docs/invariants.md` is empty. These restate requirements in the form "what must never break".
They are the strongest claim the project makes, so they are proposed rather than written.

**Most of what they restate is `[proposed]`, not `[decided]`** — nothing executed, stable
identifiers and no allocation in the frame loop all are. Approving an invariant therefore
settles the requirement behind it at the same time, which is a larger step than it looks.

1. **Nothing a puppet, scene or source supplies is ever executed.** Unblocked — the plugin API
   is ruled out (`docs/decisions.md`).
2. **The render surface shows only the puppet**, on every path including every failure path.
3. **Every reference is by stable identifier**, never by name, index or file position.
4. **A failed load never replaces what is displayed.**
5. **No allocation or GPU resource creation happens in the frame loop.**

---

# Concern checklist entries — approved and moved

*Approved 2026-11-26 and written into `docs/concerns/` under each dimension's *This project*
section, every entry tagged `judgment` with the check that would promote it named inline.*

**Why they are all `judgment` today:** `docs/concerns/README.md` says an entry tagged `script` or
`test` whose check does not exist is worse than no entry, because it reads as covered. Nothing is
built, so nothing can be tagged otherwise yet. Each entry names its intended end state and what
has to exist first; promoting them is work for the phase that builds the thing.

## Dimensions not yet swept

`accessibility` and `ops` were deliberately held back: both depend on decisions that are still
`[open]` — the UI toolkit for one, packaging and distribution for the other. Sweep them once
the stack is chosen.
