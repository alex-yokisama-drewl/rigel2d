# Accessibility

Keyboard, assistive technology, contrast, motion. Almost entirely mechanically checkable, which
makes it the dimension with the least excuse for `judgment` entries.

## Universal

### Keyboard

- **Every interactive element is reachable by keyboard** — `test`
  Tab through the flow. Nothing is mouse-only.
- **Focus order follows visual order** — `test`
- **Focus is always visible** — `test`
  A focused element has a visible indicator that is not removed by a style reset.
- **Focus is trapped in a modal and restored on close** — `test`
  Returns to the element that opened it.
- **Escape closes what it opened** — `test`

### Assistive technology

- **Every control has an accessible name** — `test`
  Accessibility tree, not a visual label that is not associated.
- **Every image has alt text, or is marked decorative** — `test`
- **Form fields are associated with their labels and their errors** — `test`
- **Dynamic changes are announced** — `test`
  A live region for anything that appears without a navigation.
- **Heading levels are sequential** — `script`
  No jump from h1 to h3.
- **Landmarks exist** — `script`

### Visual

- **Text contrast meets the project's stated level** — `test`
  Computed colour, both themes if there are two. Never judged from a screenshot.
- **Non-text contrast** — `test`
  Borders, icons, focus indicators, chart series.
- **Nothing is conveyed by colour alone** — `judgment`
  A red border also carries text.
- **Interactive targets meet the minimum size** — `test`
  Bounding box.
- **The layout survives text scaling** — `test`
  At 200%, at the minimum window size.

### Motion

- **`prefers-reduced-motion` is honoured** — `test`
- **Nothing flashes more than three times a second** — `judgment`
- **No motion is required to understand state** — `judgment`

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the project's target level and any exemptions here on the first planning sweep.
-->

### Target level **[decided]**

**WCAG 2.2 level AA, applied to desktop software by analogy (WCAG2ICT), for the interface
outside the custom canvases.** Chosen because the audience is beginners, the project has already
refused to treat interaction quality as polish
([authoring.md](../requirements/authoring.md)), and nearly all of AA is checkable as structure,
which fits the harness in [testing.md](../testing.md).

**Canvas manipulation is a stated exemption, not an oversight.** Placing vertices, shaping curves
and dragging gizmos require a mouse or touchpad. That fails AA's keyboard-operability
requirement and its single-pointer alternative to dragging, so **the project does not claim AA
overall** — it claims it for the chrome and says plainly where it stops, in the same shape as the
macOS tier in [platform.md](../requirements/platform.md).

What the level commits to, for the chrome: text contrast of 4.5:1 and non-text of 3:1; every
control exposed with a role and an accessible name; every chrome command operable from the
keyboard; a visible focus indicator; a minimum target size of 24 logical px; scaling to 200%
without loss; nothing conveyed by colour or hover alone.

**What it does not commit to:**

| Not committed to | Why |
|---|---|
| That the applications work with a screen reader | `egui_kittest` asserts the tree egui builds, not that the platform adapter — UIA on Windows, AT-SPI on Linux — delivers it. The honest claim is *exposes a correct tree* |
| Spatial authoring by assistive technology | See the canvas exemption above |
| macOS beyond "the structural assertions pass" | [platform.md](../requirements/platform.md) sets that tier |

**Exemptions**, each with its reason:

| Exempt | Why |
|---|---|
| The render surface | It is the puppet's artwork, and with no toolkit linked ([I2](../invariants.md)) it has no tree to populate. Only its OS window title is in scope |
| The puppet inside Studio's viewport | Content, not interface. Gizmos drawn over it are interface and are not exempt |
| Keyboard operation of canvas elements | The stated exemption above |
| Individual mesh vertices, from registration and from target size | Dense by nature, and not keyboard-reachable, so nothing reads them. Gizmos, keypoints and timeline markers are **not** exempt |
| Puppet animation playback | Content, so the reduced-motion and flashing entries do not constrain it |

### The canvas registration convention

Registration survives the keyboard exemption. Its main job is the overlap assertion — a gizmo
shipped underneath a panel, which is the generating case in [README.md](README.md) — and that is
unaffected by whether the element can be focused.

- **Nothing becomes interactive or persistent in a canvas, or a panel, except through the one registration module** — `judgment`
  → `script`: lint fails on raw `ui.interact`, `allocate_rect` with a click or drag sense,
  `SidePanel`, `TopBottomPanel`, `Window` and `Area` outside that module, which is the only place
  recording role, name and chrome status. Registration is then by construction rather than by
  remembered effort.
  Seeded at planning, 2026-11-26.

- **Every gizmo, keypoint and timeline marker a canvas can hit-test appears in the tree with a role and a non-empty name** — `judgment`
  → `test`: compare the tree against the canvas's own hit-test list, which is an independent
  source, not against what registered. Comparing the tree with itself is the circularity recorded
  as `[open]` in [testing.md](../testing.md). Individual mesh vertices are exempt.
  Seeded at planning, 2026-11-26.

- **A canvas stays one Tab stop, and its tree size is bounded however large the puppet** — `judgment`
  → `test`: Tab presses from the first to the last chrome control are identical on the reference
  and stress puppets, and per-canvas tree-node counts have a committed ceiling beside the counter
  baseline. The ceiling is what stops vertex registration creeping back in.
  Seeded at planning, 2026-11-26.

### Keyboard

- **Every operation in the enumerable operation set, except spatial canvas manipulation, has a keyboard path** — `judgment`
  → `test`: iterate the operation set that undo already requires
  ([authoring.md](../requirements/authoring.md)) and assert each is reachable by a shortcut or a
  focusable menu node, with the exempt operations named in one list rather than skipped silently.
  A list that grows is then visible.
  Seeded at planning, 2026-11-26.

- **Escape during a drag restores the pre-drag state exactly and adds nothing to the undo stack** — `judgment`
  → `test`: specialises the Universal Escape entry to the canvas case. Applies although the drag
  itself is pointer-only.
  Seeded at planning, 2026-11-26.

- **Focus on a chrome control is drawn by the registration module** — `judgment`
  → `test`: focus the control and assert a focus-indicator shape in egui's output at its rect.
  Shapes are data, not pixels. The Universal focus-visible entry cannot see custom-painted focus.
  Seeded at planning, 2026-11-26.

- **No information exists only on hover** — `judgment`
  → `script`: [authoring.md](../requirements/authoring.md) rules out hover-dependence for
  operations; this extends it to information. Raw `on_hover_text` is banned outside a wrapper
  that also sets the node's label or description.
  Seeded at planning, 2026-11-26.

### Visual

- **No colour literal is painted in interface code** — `judgment`
  → `script`: lint for `Color32::` literals, `RichText::color`, `override_text_color` and
  per-call strokes outside the theme module and an allowlist of puppet-content paint paths. This
  is the reviewable defect named in [testing.md](../testing.md), made mechanical.
  Seeded at planning, 2026-11-26.

- **Canvas palettes live beside `Visuals` and are covered by the same exhaustive contrast test; a colour field in neither a declared pairing nor the exempt list fails** — `judgment`
  → `test`: text pairings drawn from one table, over every theme that exists rather than a fixed
  number of them, so adding a theme later costs nothing. The canvas half is what
  [testing.md](../testing.md) misses by checking `Visuals` alone.
  Seeded at planning, 2026-11-26.

- **Gizmos, keypoints and vertices drawn over artwork use an outline and fill pair that guarantees 3:1 against any background** — `judgment`
  → `test`: compute, over all background luminances, that the better of the two colours meets
  3:1. Contrast against arbitrary user artwork is otherwise unassertable, because the artwork is
  not ours to constrain.
  Seeded at planning, 2026-11-26.

- **Chrome controls, gizmos, keypoints and timeline markers meet the 24 logical px minimum target size** — `judgment`
  → `test`: `bounding_box()` over every registered interactive node. Individual mesh vertices are
  exempt.
  Seeded at planning, 2026-11-26.

- **At 200% UI scale, panels collapse rather than the minimum window size growing, and nothing overlaps or overflows at either scale** — `judgment`
  → `test`: re-run the overlap and panel-state overflow assertions at 100% and 200%, with the
  minimum window size as one named constant. A minimum window that scaled with the setting would
  not fit a 1366×768 laptop, which is the hardware this project is aimed at. The collapsed states
  are part of what is asserted. Revisit when Studio's layout is designed.
  Seeded at planning, 2026-11-26.

- **All interface animation duration comes from one setting, and reduced motion sets it to zero** — `judgment`
  → `script`: lint bans `animate_*_with_time` with a literal duration outside the wrapper, plus a
  unit test that the setting zeroes the animation. The preference is **the application's own
  setting, not the operating system's** — reading the OS preference needs a platform-specific
  path on each platform, against [platform.md](../requirements/platform.md)'s rule that macOS
  costs packaging rather than reimplementation.
  Seeded at planning, 2026-11-26.

### Assistive technology

- **Every flagged state carries its meaning as text in the tree, not only as colour** — `judgment`
  → `test`: one fixture per state variant, asserting the node label equals the catalogue string.
  The states are dormant entry, stale binding or deformation, unrecognised data, source
  collision, source dropped, and the active edit target. Stronger than the Universal colour-alone
  entry, which is `judgment` permanently.
  Seeded at planning, 2026-11-26.

- **Announced events come from one list, and per-frame displays are never live regions** — `judgment`
  → `test`: iterate the list and assert a live property on each, and assert that channel meters
  and any frame-time readout are *not* live — announcing a value that changes every frame buries
  everything else. The events are mode change, edit-target change, load failure, per-item merge
  outcome, source drop and return, and dormant entry.
  Seeded at planning, 2026-11-26.

- **The two Live windows have distinct, non-empty OS titles, and the control surface offers a named, keyboard-focusable way to raise the render surface** — `judgment`
  → `test`: assert the title constants, and assert the activation by the command it emits rather
  than by a real window. The render surface is undecorated and has no tree, so the OS title,
  taskbar and alt-tab entry are the only identity it can carry. Serves *losing one behind the
  other is recoverable* in [performing.md](../requirements/performing.md).
  Seeded at planning, 2026-11-26.

- **The platform adapter delivers the tree, checked by a person each release** — `judgment`
  Walk Studio's first run and Live's control surface with Narrator or NVDA on Windows and Orca on
  Linux, and confirm names, roles and announcements arrive. **Stays `judgment` permanently** —
  nothing automated reaches the adapter. Every catch becomes an entry.
  Seeded at planning, 2026-11-26.
