# Routed — checklist entries from the animation sweep

**Working document.** The seven planning sweeps of 2026-11-28 proposed around forty-five
entries; many were the same finding reached from different directions. This is the deduped set,
curated and approved, with the destination decided. A writer places these; it does not choose
where they go or whether they belong.

Every entry carries the provenance line **"Seeded at the animation planning sweep,
2026-11-28"**, goes under *This project* in its file, and matches the surrounding entry format —
bolded claim, the tag, then the indented explanation naming the failure it catches.

The owner's standing instruction on these: **bias toward keeping.** A check that turns out
unnecessary is cheap to delete; one that was never written is the expensive kind. Where an
entry looked marginal it was kept.

---

## `docs/concerns/data.md`

- **Contributions to one value are summed in an order fixed by data, never by runtime state** — `test`
  Permute the file's collection order and the runtime start order over three or more
  contributions whose float sum differs by grouping, and assert bit-identical output.
  *Catches:* the belief that summation being commutative makes order irrelevant. It is
  commutative and **not associative**, so three contributions depend on grouping — and
  `evaluation.md` calls three bindings on one node ordinary rigging. Applies equally to any
  fixed iteration count.

- **Every enumerated field declares what a build that does not know its value does, and carries a fixture of one** — `test`
  The existing unfamiliar-value entry covers the fields the format already lists; this one binds
  the next field added. *Catches:* an interpolation mode, end behaviour or trigger event added
  with no answer for a stranger's build — which is the gap the animation draft had until it was
  swept.

- **A value stored by position survives its neighbours being inserted, deleted or split** — `test`
  A segment's interpolation mode lives on the earlier keyframe, and keyframes have no
  identifier. *Catches:* a mode silently moving to the next segment after an insert. Extends
  the existing preserved-material entry from material a build does not understand to material
  it does.

- **Presentation data is stored as a field, never as a collection's order, and every save that does not own it leaves it untouched** — `test`
  Covers parameter order and the mapping mark. *Catches:* a canonical sort by identifier erasing
  a rigger's arrangement, and Live's save path dropping what Studio set.

- **A new reference kind declares whether it is constitutive or incidental, and the deletion report names every record a cascade removes** — `script` over the format definition
  *Catches:* an animation deleted leaving a trigger that names nothing.

- **A change to how stored data composes states what previously saved files now do** — `judgment`
  Becomes a `test` once an evaluator exists. *Catches:* a semantics change with no data
  migration to prompt anyone — nothing in the file differs, so nothing flags it. Currently
  cheap because nothing has shipped; the entry exists for when that stops being true.

## `docs/concerns/security.md`

- **A trigger is data selecting from a set, and no field of it is evaluated** — `test`
  Fixtures carrying an expression-like string, an unknown event, and a target named rather than
  identified. *Catches:* a trigger growing a condition, a delay expression or a runtime-looked-up
  hook, each of which turns a puppet into a program and breaks I1. This is the entry that keeps
  the line where the sweep found it.

- **Animation evaluation is finite for every finite file** — `test`
  Property test over keyframe times, Bézier control points, durations, a stalled frame interval
  and the representable extremes. *Catches:* `f32::clamp` propagating NaN to geometry, `inf −
  inf` in a sum of in-range contributions whose range reaches the float limit, a zero derivative
  in a Newton step, and a zero-duration loop that never exits.

- **Executing a trigger cannot raise an event, and a flapping source cannot start work without bound** — `test`
  *Catches:* a restart storm from a hostile or glitchy source on loopback, and trigger-to-trigger
  chaining, which is where the state machine we ruled out would arrive by the back door.

- **An aggregate limit exists wherever per-item limits multiply** — `script`
  Extend the limits completeness walk to require it. *Catches:* animation × track × keyframe
  limits permitting hundreds of millions of keyframes, bounded only by the parse-tree cap.

- **A scene event carries no payload** — `test`
  *Catches:* a puppet file learning which trackers, ports or addresses a person has. Cheap now,
  a breaking change once files exist.

## `docs/concerns/performance.md`

- **Seeking a keyframe is logarithmic in the worst case, and a time jump never scans** — `test`
  Fixture with a forward jump, a loop wrap, a backward seek and a first evaluation, at both
  two keyframes and one-per-frame. *Catches:* a forward-advancing cursor that walks every
  keyframe it skipped — which passes every steady-state test and fails precisely on a deadline
  miss, when the budget is already blown.

- **Issuing a playback command or firing a trigger allocates nothing** — `test`
  *Catches:* per-track playback state created lazily at play time, which is CPU allocation in
  the frame loop at the exact moment a source drops.

- **Work per frame is counted by playing state, and held animations have a stated ceiling** — `test`
  *Catches:* hold animations accumulating with nothing stopping them, quietly raising per-frame
  cost over a stream.

- **A looping playhead is wrapped, not accumulated, and the soak asserts it** — `test`
  *Catches:* float drift that grows with elapsed time and appears only hours into a stream.

- **Resident bytes for animation are linear in keyframes at a stated per-keyframe size** — `test`
  *Catches:* storage that fits the payload limit and not the memory budget — the distinction
  between a limit and a budget, made checkable.

## `docs/concerns/ops.md`

- **An advisory is edge-triggered and coalesced; its cost never scales with frames or messages** — `test`
  *Catches:* a clamp reported once per frame, and a flooding source evicting every useful line
  from a rotated log so the diagnostics report is one repeated message.

- **A condition the runtime absorbs silently is visible somewhere a person can reach** — `test`
  Covers clamping, resting at neutral on a lost source, and a trigger firing. *Catches:* the
  stream looking wrong with nothing anywhere saying why.

- **Everything that can start without the person asking can be stopped by the person, from the surface they are already using** — `test`
  *Catches:* a `hold` animation with no exit, and the restart-the-player outcome. Assert the
  playing set names what started each animation, and that stopping returns evaluation to the
  base drivers.

- **A trigger that cannot act is reported, never silent, and never reaches the render surface** — `test`
  Fixtures: a dangling target, an event nothing raises, a scene naming an idle the puppet does
  not have. *Catches:* a puppet quietly not doing what its author wired, and I2 breached by a
  diagnostic drawn on the stream.

- **Losing and regaining a source leaves no residue, and a flapping source cannot restart animations without bound** — `test`
  *Catches:* an idle started on a drop still layered over live tracking after the tracker came
  back — the failure the release-on-clear rule exists to prevent, asserted rather than assumed.

- **The load-in-progress marker is cleared only after start-up triggers have run and a frame has presented** — `test`
  *Catches:* a crash loop that begins after the marker clears, so the existing relaunch
  protection never engages.

- **A local log records the shape of what was loaded, and carries nothing the user did not intend to share** — `test` and `script`
  Per animation: duration, track count, keyframe count, by identifier and never by name.
  *Catches:* workload assumptions that cannot be revisited because real inputs were never
  visible, and a log invited into a bug report leaking file paths or puppet names.

## `docs/concerns/ux.md`

- **A parameter's order and mark never change what the puppet evaluates to** — `test`
  Shuffle every order, flip every mark, assert identical output. *Catches:* presentation state
  leaking into behaviour.

- **Anything hidden from a list is still shown when something depends on it, and stays reachable from where the person is** — `test`
  Fixture: a mapped parameter marked not offered. *Catches:* a streamer who can see neither the
  mapping nor a way to remove it, because the mark can only be changed in the other application.

- **A bulk operation on a stress-sized collection is one undo step and holds the display rate** — `test`
  Fixture: the minute-long, keyframe-per-frame animation. *Catches:* a timeline nominally usable
  at length that needs a thousand clicks — which is the scripting the owner is trying to stop
  doing.

- **Where two places can configure one outcome, one wins, it is stated, and the interface says which** — `test`
  Covers the scene's drop behaviour against a puppet's trigger. *Catches:* a streamer who set
  one thing, sees another, and has nothing to read.

- **A silent reduction of a value the author supplied is reported at the thing reduced** — `test`
  Covers clamping now and anything of the same shape later. Assert the report names the
  parameter and the contributors, and that a within-range sum reports nothing — a permanent
  warning is an ignored warning.

## `docs/concerns/accessibility.md`

- **A dense ordered collection is reached by a keyboard walk, not by registering every element** — `test`
  Amends the existing "appears in the tree" entry for keyframes. Walk *next* from the first and
  compare against the canvas's own hit-test list, which is the independent second source
  `testing.md` requires. *Catches:* both failures at once — keyframes reachable only by mouse,
  and a tree node per keyframe at a thousand.
  **Keyframes are not vertices.** A vertex is an unordered point meaning nothing to a person; a
  keyframe has a time and a value a rigger reasons about, and *next keyframe* is a natural
  operation. The existing vertex exemption does not extend to them.

- **Tree size and tab-stop count do not grow with the size of the thing being edited** — `test`
  *Catches:* registration creeping back at scale, asserted against a committed ceiling.

- **Every drag in chrome has a non-drag alternative, and the drags are enumerated in one list** — `test`
  Parameter reorder, keyframe move, playhead scrub. Canvas drags are exempt by name in the same
  list. *Catches:* reordering that works only by mouse.

- **An element in a dense collection has an accessible name carrying its own values, not its kind** — `test`
  A keyframe reads as track, time, value and interpolation to the next. *Catches:* a thousand
  elements all announcing "keyframe".

- **A per-frame value is never a live region; a discrete event is announced once** — `test`
  Playhead time and clamp flips are not live; play, pause, stop, seek and end-reached are.
  *Catches:* a screen reader flooded for the duration of playback.

- **A virtualised list exposes position and set size on its visible rows** — `test`
  *Catches:* rows that exist only once scrolled to, and a reorder announcement with nothing to
  say.

- **Displayed order, stored order and focus order are the same for an author-ordered list** — `test`
  *Catches:* unordered-container iteration silently discarding the arrangement the author made.

- **An event that can arrive in a burst is coalesced into one announcement** — `test`
  *Catches:* a flapping source flooding the live region.
