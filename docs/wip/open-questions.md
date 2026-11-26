# Open questions and pending approvals

What requirements work is waiting on. Deleted when the requirements are agreed and the phase
plan is written; nothing here is permanent.

Two halves: **questions for the user**, and **checklist entries proposed by the concern sweep
and awaiting approval**. The second half must not be copied into `docs/concerns/` until
approved — an agent-generated list nobody agreed to is a list nobody believes
(`docs/concerns/README.md`).

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
| Must sources be reachable from another machine in the first release? Phone-based tracking needs it | If yes: is a visible indicator enough, or is an address allow-list or a pairing step wanted? |
| Is OpenSeeFace's protocol our native protocol, or one source implementation behind a protocol of our own? | Decides whether the port is reachable from a web page, which decides whether it needs origin checks |
| Can a scene be edited while live? Is there a locked state? | |

*Resolved 2026-11-26: what the puppet does when its source drops is a scene setting, choosing
among hold, ease to neutral, and idle animation — in `docs/requirements/performing.md`. The
default for a new scene is still `[open]`; ease to neutral is the working assumption.*

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

| Question | Why it matters |
|---|---|
| Is a pen tablet a target input device for Studio — pressure, hover, no right-click? | Artists often work on one, and it changes the interaction model |

*Answered 2026-11-26: the project is **Rigel2D**; the applications are **Studio** and **Live**,
in `docs/vocabulary.md` and `CLAUDE.md`. UI text is English only — no localisation in scope.*

*Answered 2026-11-26, now in `docs/decisions.md`: no plugin API and no in-puppet scripting; no
outbound network connection ever; the interface uses the project's strict vocabulary.*

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

# Proposed concern checklist entries, awaiting approval

From the planning sweep of 2026-11-26. Triaged down from what the analysts returned; the ones
dropped were either duplicates across dimensions or gated on questions above.

Provenance for every entry below is "seeded at planning, 2026-11-26".

**A note on tags.** `docs/concerns/README.md` says an entry tagged `script` or `test` whose
check does not exist is worse than no entry. Nothing is built, so *every* entry here would be
`judgment` on the day it is added. The tag shown is the intended end state, and each names what
promotes it. Deciding how to handle that gap — add them as `judgment` now, or hold them until
their phase — is itself a question for the user.

## data

- **Every entity has a stable identifier, and every reference is by identifier** — `test`
  Rename and reorder a fixture puppet, save, reload, assert every reference still resolves. The
  validator rejects name-based and index-based references.
- **One shared validator gates load and save, and runs as a standalone command** — `test`
  Hostile fixtures — cycle, dangling reference, duplicate identifier, NaN, unordered keypoints,
  binding count mismatch — are all rejected. Save refuses to write an invalid puppet.
- **Every write to a puppet, scene or autosave path goes through one atomic-write function** — `script`
  Fails on any write outside it. A paired test injects failure at each step and asserts the
  prior file is intact.
- **Saving an unmodified puppet is byte-identical, with no timestamps, paths or machine names** — `test`
- **Unknown fields and entities survive a load-then-save round trip** — `test`
- **Every released format version has a committed fixture the current build loads; fixtures are append-only** — `script`
- **Adding artwork preserves configuration** — `test`
  Add images, a PSD and a merged puppet to a configured fixture. Assert every pre-existing
  identifier, binding, keypoint value and deformation is unchanged. Includes the
  changed-dimensions and regenerated-mesh cases once those are decided.
- **A random sequence of edits followed by undo-all returns a serialisation equal to the original** — `test`
  Property test over the operation set, including compound operations and import.
- **A file written by an independent generator, using no project code, validates and loads** — `test`
  The reference puppet generator satisfies this. It is the machine-checkable form of "third
  parties can write it".

## security

- **Every parser of untrusted input survives hostile input** — `test`
  Time-boxed fuzz run in CI over puppet, scene, PSD, image and port-message parsers. No crash,
  no hang, bounded memory.
- **A committed set of hostile fixtures is rejected cleanly** — `test`
  Truncated, cyclic, duplicate identifiers, out-of-range index, NaN, oversized dimensions,
  decompression bomb, traversal-style entry names, unknown sections.
- **Load limits are defined in one place and each is exercised by a fixture just over it** — `test`
  A limit with no fixture fails the check.
- **A failed load leaves the displayed puppet unchanged** — `test`
  Load a hostile fixture while a good puppet renders; assert the rendered buffer is unchanged.
- **The input port binds to loopback by default, and loading a scene never changes that** — `test`
- **A source can set channel values and trigger animations, and nothing else** — `test`
  Malformed, oversized and many-channel messages leave scene state, loaded puppets and output
  configuration unchanged, within the memory cap.
- **Channel values are finite and clamped before binding** — `test`
- **The player requests no screen-capture or window-capture capability** — `script`
  Scan platform manifests and entitlements; fail on any.
- **Dependencies are locked with hashes, CI actions pinned to commit hashes, scan clean or triaged** — `script`
- **Signing keys and release tokens are unreachable from pull-request jobs** — `script`

## performance

- **Every per-frame counter has a committed baseline ceiling** — `script`
  Promoted once the runtime exposes headless counters. Fails on any counter above baseline
  unless the baseline changes in the same commit with a reason.
- **Per-frame counters scale no worse than linearly from the reference puppet to the stress puppet** — `test`
  Catches accidental quadratic behaviour in deformation composition and physics.
- **Nothing in the frame loop resolves by name or scans linearly** — `test`
  The name-resolution counter is zero; animation lookup is independent of animation length.
- **Parts sharing one texture leave resident texture bytes and upload count unchanged** — `test`
  Asserts the `[decided]` texture-reuse requirement in `puppet.md`.
- **No first-use work after the first presented frame** — `test`
  Resource creations and pipeline preparation after frame one are zero, for every blend, mask
  and composite combination the puppet uses.
- **Startup and load budgets hold with every derived cache empty** — `test`
- **The input path is latest-wins and bounded, and its latency in frames is asserted** — `test`
  Injected clock. Includes a flood case and a many-distinct-channel-names case.
- **Edit cost, undo step size and bytes written per save are independent of total puppet size** — `test`
- **Import peak memory is independent of PSD layer count** — `test`
- **A soak run at the stated stream length leaves counters and resident bytes flat** — `test`

## ux

- **Every mutating editor operation is undoable, and undo then redo restores the puppet exactly** — `test`
  Requires operations to be enumerable, which is a design constraint worth stating out loud.
- **A failed import leaves the puppet unchanged and says why** — `test`
  Fixtures: truncated PSD, zero layers, corrupt layer, unsupported blend mode, oversized canvas,
  non-image file. Assert file, cause and next action, and an unchanged serialisation.
- **A batch merge reports a per-item outcome** — `test`
- **Any operation over 100 ms shows progress within 100 ms and cancels without effect** — `test`
- **Every user-visible error is in one catalogue with what, why and next action, all non-empty** — `test`
- **No dialog or error state stops the player's frame counter advancing** — `test`
- **A source going silent and returning is handled without a restart** — `test`
  Covers each of the three scene-selected drop behaviours, the continuous transition into and
  out of each, and the case where a scene names an idle animation the puppet does not carry.
- **The render surface's structure contains nothing but the render target, in every error state** — `test`
- **One broken scene item never blocks the rest** — `test`
- **Unsaved changes are never lost silently** — `test`
  Close, open-another, and simulated crash.
- **The active mode and the current edit target are exposed as inspectable state** — `test`
  Switching mode, parameter or keypoint changes it. This is what makes the
  editing-between-keypoints trap testable rather than a matter of someone noticing.
- **Viewport handles are not covered by chrome at the minimum window size** — `test`
  Bounding-box intersection between every handle and every panel and floating element, including
  handles at the viewport edge. This is the generating case from `docs/concerns/README.md`.
- **The longest plausible node, parameter and channel names do not overflow the tree or binding lists** — `test`
- **A first-time user reaches a moving puppet from a PSD without outside help** — `judgment`
  Human gate on the first end-to-end build. Watch for every point where the tester stops,
  searches or asks; each becomes an entry.

## Dimensions not yet swept

`accessibility` and `ops` were deliberately held back: both depend on decisions that are still
`[open]` — the UI toolkit for one, packaging and distribution for the other. Sweep them once
the stack is chosen.
