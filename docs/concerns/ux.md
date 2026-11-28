# UX

Interaction, layout, states, and the user's path through a flow. The dimension where "it works"
and "it is usable" diverge most, and the one a developer in flow is least likely to sweep.

## Universal

### Every state exists

- **Empty state** — `test` / `judgment`
  What the user sees before there is any data. Not a blank area.
- **Loading state** — `test`
  Anything that can take time shows that it is taking time. The user never wonders whether their
  click registered.
- **Error state** — `test`
  Every failure the user can cause, and every failure they cannot, produces something that says
  what happened and what to do. Never a silent no-op.
- **Partial and slow state** — `judgment`
  What is shown when some of the data arrives and some does not.

### Interaction

- **No interactive element is covered by another** — `test`
  Bounding-box intersection between every interactive element and every piece of persistent
  chrome, at the minimum window size. *This is the check that was missing when a hotspot
  shipped beneath an inventory strip.*
- **Nothing overflows its container unintentionally** — `test`
  Scroll width against client width.
- **Destructive actions are confirmable or reversible** — `test`
  One of the two. Never neither.
- **Double submission is prevented** — `test`
  The second click does not produce a second record.
- **A form that fails preserves what was typed** — `test`
- **Validation says what is wrong and where** — `test`
  Inline, at the field. Not a summary that makes the user hunt.

### The journey

- **The flow can be completed without leaving it** — `judgment`
  No step requires information the user has no way to get from where they are.
- **Interruption is survivable** — `judgment`
  Closing the tab mid-flow, coming back, using the browser's back button.
- **Success is visible** — `test`
  The user can tell the thing worked without checking somewhere else.
- **Every entry point to this feature was considered** — `judgment`
  Deep link, bookmark, refresh mid-flow, arriving with stale state.

### Layout

- **Works at the minimum window size** — `test`
  Bounding-box assertions at each, not a glance at one.
- **Long content does not break it** — `test`
  The longest plausible name, title, list. Test with the real maximum, not a short example.
- **Text in the user's language is not hard-coded to one width** — `judgment`

## This project

<!--
Generated at planning by concern-analyst, approved by the user, grown by harvest at the end of
every cycle. Each entry carries its provenance. See README.md.
-->

### Nothing is lost

- **Every mutating operation in Studio is undoable, and undo then redo restores the puppet exactly** — `judgment`
  → `test`. Requires operations to be enumerable, which is a design constraint worth stating out loud — and the same shape a plugin API would eventually drive.
  Seeded at planning, 2026-11-26.
- **Unsaved changes are never lost silently** — `judgment`
  → `test`, covering close, open-another, and a simulated crash.
  Seeded at planning, 2026-11-26.
- **A failed import leaves the puppet unchanged and says why** — `judgment`
  → `test` over fixtures: truncated PSD, zero layers, corrupt layer, unsupported blend mode, oversized canvas, non-image file. Assert file, cause and next action, and an unchanged serialisation.
  Seeded at planning, 2026-11-26.
- **A bulk operation on a stress-sized collection is one undo step and holds the display rate** — `test`
  Fixture: the minute-long, keyframe-per-frame animation. Assert the undo-step count and per-frame work counters, not wall-clock time. *Catches:* a timeline nominally usable at length that needs a thousand clicks — which is the scripting the owner is trying to stop doing.
  Seeded at the animation planning sweep, 2026-11-28.

### Telling the user what happened

- **Every user-visible error is in one catalogue with what, why and next action, all non-empty** — `judgment`
  → `test`. The catalogue is also where the strict-vocabulary lint looks, and where English-only means text lives.
  Seeded at planning, 2026-11-26.
- **A batch merge reports a per-item outcome** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **Any operation over 100 ms shows progress within 100 ms and cancels without effect** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **Opening a file carrying unrecognised data warns, names what is missing, and still allows editing** — `judgment`
  → `test`. The warning must not imply an install route while plugins do not exist. See `docs/requirements/puppet.md`.
  Seeded at planning, 2026-11-26.
- **A silent reduction of a value the author supplied is reported at the thing reduced** — `test`
  Covers clamping now and anything of the same shape later. Assert the report names the parameter and the contributors, and that a within-range sum reports nothing — a permanent warning is an ignored warning.
  Seeded at the animation planning sweep, 2026-11-28.

### Live never stops rendering

- **No dialog or error state stops the frame counter advancing** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **The render surface's structure contains nothing but the render target, in every error state** — `judgment`
  → `test`. Asserts [I2](../invariants.md).
  Seeded at planning, 2026-11-26.
- **One broken scene item never blocks the rest** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **A source going silent and returning is handled without a restart** — `judgment`
  → `test`, covering each of the three scene-selected drop behaviours, the continuous transition into and out of each, and a scene naming an idle animation the puppet does not carry.
  Seeded at planning, 2026-11-26.
- **Where two places can configure one outcome, one wins, it is stated, and the interface says which** — `test`
  Covers the scene's drop behaviour against a puppet's trigger. *Catches:* a streamer who set one thing, sees another, and has nothing to read.
  Seeded at the animation planning sweep, 2026-11-28.

### Saving, and what it changes

- **Any save names what is being written and where, before it is written** — `judgment`
  Specifically: Live editing a channel mapping is modifying a *puppet*, and must say so. Inochi Session's "save to file" is the generating case — a user can reasonably believe they are saving a scene while a puppet is being changed.
  Seeded at planning, 2026-11-26.
- **A dormant scene entry can be relocated from the control surface while the scene renders** — `judgment`
  → `test`. Recovery is offered, never demanded: no modal during scene load, no path where a missing file stalls the render. Cross-reference the frame-counter entry above.
  Seeded at planning, 2026-11-26.

### The interface itself

- **Every operation is reachable and comfortable with mouse and keyboard alone** — `judgment`
  No operation depends on pressure, tilt, hover or precision a mouse cannot comfortably produce. Cross-reference `accessibility.md`: keyboard-only is a stronger claim than this one, and it is deliberately not claimed for canvas manipulation, which is a stated exemption there.
  Seeded at planning, 2026-11-26.
- **Viewport gizmos are not covered by chrome at the minimum window size** — `judgment`
  → `test`: bounding-box intersection between every gizmo and every panel and floating element, including gizmos at the viewport edge. This is the generating case from `README.md`.
  Seeded at planning, 2026-11-26.
- **The longest plausible node, parameter and channel names do not overflow the tree or binding lists** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.
- **The active mode and the current edit target are exposed as inspectable state** — `judgment`
  → `test`: switching mode, parameter or keypoint changes it. This is what makes the editing-between-keypoints trap testable rather than a matter of someone noticing.
  Seeded at planning, 2026-11-26.
- **A parameter's order and mark never change what the puppet evaluates to** — `test`
  Shuffle every order, flip every mark, assert identical output. *Catches:* presentation state leaking into behaviour.
  Seeded at the animation planning sweep, 2026-11-28.
- **Anything hidden from a list is still shown when something depends on it, and stays reachable from where the person is** — `test`
  Fixture: a parameter declaring `mappable: false` that carries a channel mapping. *Catches:* a streamer who can see neither the mapping nor a way to remove it, because the flag can only be changed in the other application.
  Seeded at the animation planning sweep, 2026-11-28.
- **A first-time user reaches a moving puppet from a PSD without outside help** — `judgment`
  Human gate on the first end-to-end build. Stays `judgment` permanently. Watch for every point where the tester stops, searches or asks; each becomes an entry.
  Seeded at planning, 2026-11-26.

### Texture lifetime, copy, paste and symmetry

- **A default the rigger did not choose is inspectable before commit and named in the result** — `judgment`
  → `test` once operations return structured results: the paste component, whether a parameter was shared or duplicated, which mirror axis was used, and which axes a mirror left unreversed. The failure this catches is silent and late — the rigger finds out when both eyes blink together, on stream.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **Every term the interface shows is defined in `vocabulary.md` and explained where it first appears** — `script`
  → a lint over the string catalogue, beside the strict-vocabulary lint. The vocabulary places the onboarding cost on explaining a term at first appearance rather than on a second set of friendlier names, and nothing currently checks that the explanation is there. *Mirror*, *texture flip*, *bilateral* and *unipolar* are the terms this revision adds.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

### A node that draws nothing, and a term the interface will show

- **Every node kind that draws nothing is findable in the tree and identifiable as non-drawing, with its anchor and what it drives inspectable** — `judgment`
  → `test` over a fixture containing a physics node: it has a tree row marked as drawing nothing, selecting it exposes its anchor and its output parameter, and it stays reachable with other kinds collapsed or hidden. Physics is the first kind with no pixels, so the canvas cannot show it and the tree is the only place it exists.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A physics node that can currently do nothing is reported at the moment it becomes so** — `judgment`
  → `test`: fixtures with an anchor nothing moves, with a deleted output parameter, and with an edit that would put two drivers on one parameter. Assert the result names the node and the cause, and that a refused edit leaves the puppet unchanged. A silently inert pendulum is the authoring failure that shows up on stream.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A deletion report names a physics node as a node with its anchor, and names every animation track it removes** — `judgment`
  → `test` over fixtures deleting a parameter that has both physics and tracks, and deleting a node that is a physics anchor. Assert the report names the physics node and its anchor, names each animation and its track, and that undo restores both.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **Physics has a defined value on the first frame after a load, a replacement, a re-anchor and a timeline scrub, and no transform is non-finite at any of them** — `judgment`
  → `test` at each event: every accumulated transform finite, and physics at its rest value on the first frame. The replacement case is the one a viewer sees, so it is also where [I2](../invariants.md) binds.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A mirror or a paste over a subtree containing physics names the physics nodes it reflected, copied, or left alone** — `judgment`
  → `test` on the structured result: each physics node listed with its new anchor, and each refusal named. Extends the existing rule that a default the rigger did not choose is named in the result.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **"Track" is defined in `vocabulary.md` and explained where it first appears in the interface** — `judgment`
  → `script` once the term is added to the first-appearance lint over the string catalogue. A new term the rigger has not met is exactly what that lint exists for.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
