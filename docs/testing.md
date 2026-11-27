# Testing

How each kind of thing is tested here.

`.claude/rules/testing.md` carries the rules that apply to every project — the deterministic-
first ordering, the visual-check policy, the human gate. **This document carries what is
specific to this one:** the harness, the idioms, and what is deliberately not tested.

## Harness **[proposed]**

| Kind | Tool | Lives in |
|---|---|---|
| Unit | `cargo test`, in-crate | Beside the code it tests |
| Integration | `cargo test`, workspace | `tests/` in each crate |
| Acceptance | `cargo test`, workspace | `tests/acceptance/`, written from the requirements by `acceptance-planner` |
| Structural UI | `egui_kittest`, headless | `tests/ui/` in `rigel-studio` and `rigel-live` |
| Counters | The generated workloads against a committed baseline | `tests/counters/` in `rigel-runtime` |
| Growth | The same counters on the stress workload, against the dimension that drives each | `tests/counters/`, beside the baseline gate |
| Soak | Accelerated time, headless, synthetic input, over four hours of simulated stream | `tests/soak/` in `rigel-runtime`. **Ignored by default** — see below |

**Trap: a bare nested test directory is silently skipped.** Cargo auto-discovers `tests/*.rs` and
`tests/<dir>/main.rs` only, so `tests/acceptance/` or `tests/counters/` holding other files is not
run by `cargo test --workspace`, and nothing reports it. Each such directory needs a `main.rs` or
an explicit `[[test]]` entry.

`.claude/test-cmd` holds the command, and is the one place it is written down — the gate list in
[stack.md](stack.md#on-push-proposed) and CI both run it with `--locked`. The counter gate runs as its own job: it
compares against a committed baseline rather than passing or failing on its own terms, and
[platform.md](requirements/platform.md#how-this-is-verified-proposed) explains why that half carries the
whole frame budget.

**The soak is its own job too, and is excluded from the default run.** It is marked ignored by
default and run on its own schedule — nightly, and before a release.
[platform.md](requirements/platform.md) sets the length, and carries the arithmetic for why a
suite that slow would simply be switched off.

The growth row is the one that catches accidental quadratic behaviour in deformation
composition and physics — the failure that makes a tool pleasant on small rigs and useless on
real ones. It is asserted per dimension rather than as one figure, because
[platform.md](requirements/platform.md) is explicit that the workload table does not scale
uniformly.

**No test requires a GPU.** This is a design constraint rather than a convenience —
[runtime.md](runtime.md) describes the command stream the counters are read from, and wgpu's noop
backend supplies the device so the load path still runs real code.

## Visual verification **[proposed]**

**`visual-checks: automated`**, as `CLAUDE.md` states.

`scripts/ui-assertions.md` carries the patterns, written against a browser and Playwright. **The
project is not a browser**, so the translation is set out here once rather than rediscovered per
test.

**There are no breakpoints.** `scripts/ui-assertions.md` asks for the widths to be named here,
and for a desktop application the answer is that there is one width that matters: the
**minimum window size**, below which the window cannot be resized. Overlap and overflow are
asserted there and at a comfortable default. The actual figure is set when Studio's layout is
designed and belongs in this document when it is.

| Pattern | Written in a browser as | Here |
|---|---|---|
| Overlap | `boundingBox()` intersection | `bounding_box()` on two AccessKit nodes, intersected |
| Size and position | `getBoundingClientRect` | `bounding_box()` |
| Keyboard and focus | Tab, then `document.activeElement` | Simulated input, then `is_focused()` |
| Accessible name and role | Accessibility-tree snapshot | `role()` and `label()` on the node |
| Contrast and computed colour | `getComputedStyle` | **Not per-element.** See below |
| Overflow | `scrollWidth` vs `clientWidth` | **Not from the tree.** See below |

### The two patterns that do not translate **[proposed]**

**Computed colour has no per-element query, and does not need one.** The browser pattern exists
because CSS cascades, so an element's colour is only knowable after resolution. egui has no
cascade: a widget's colour comes from the one `Visuals` struct, by widget kind and interaction
state. The assertion therefore moves — contrast is checked exhaustively over `Visuals`, with no
widget tree involved at all. There is one theme, dark, and the test iterates over whatever themes
exist, so adding one later costs nothing.

**This leaves a large case uncovered**, and in Studio it is the main one: a widget painted with a
colour chosen at the call site is invisible both to the theme assertion and to the accessibility
tree. The mesh editor, the timeline and the parameter graph are custom canvases that paint their
own colours, so a contrast assertion over `Visuals` alone misses all of them. **Canvas palettes
therefore have to be inside the contrast test**, not only guarded by review. Beyond the canvases,
**an overridden colour in interface code is a reviewable defect** unless it is a puppet's own
content, which is not interface.

**Overflow is asserted from application state**, not from the tree. An egui `ScrollArea` exposes
its content size and inner rect to the code that built it, so the test reaches into the panel's
own state rather than querying a widget.

### The convention custom canvases need **[proposed]**

`scripts/ui-assertions.md` says the overlap check "needs a convention, and the convention is the
real work", and asks for it to be established here. This is that convention, and it matters more
in Studio than the note anticipated.

**egui's accessibility tree contains only what egui puts in it.** A custom-painted canvas
contributes nothing unless its author says so — and Studio's mesh editor, timeline and parameter
graph are all custom canvases. Without a convention, the automated visual-check policy would
cover the chrome around Studio's real work and none of the work itself, which is the silent
shrinking coverage that note warns about.

The rule:

- **Every interactive element in a custom canvas registers itself**, with a role and an
  accessible name, through egui's widget-info call. A gizmo, a keypoint, a timeline marker and
  a piece of chrome are each an element. **Individual mesh vertices are not registered**:
  canvas manipulation does not need to be keyboard-operable
  ([accessibility.md](concerns/accessibility.md)), so the set to register is bounded and does
  not scale with the vertex count.
- **Every piece of persistent chrome is marked as such**, so the overlap assertion has both
  sides of its comparison.
- **An element that cannot be registered cannot be shipped.** This is the rule that keeps the
  coverage honest, and it is the one that will be under pressure, because registering is work
  and skipping it is invisible until a gizmo ships under a panel.

A test asserts that every interactive element in each canvas appears in the tree, so that
*forgetting* fails rather than silently narrowing what the other assertions cover. Tagged
`test`, and it is the entry that makes the rest of the convention self-enforcing.

**Where the independent second list comes from is undecided** **[open]**. The test as described
cannot fail on its own: if the tree is the only list of a canvas's interactive elements, a
forgotten element is absent from the tree and from the expectation alike, and the test passes.
It needs a second list to compare the tree against.
[accessibility.md](concerns/accessibility.md) names the canvas's own hit-test list as the
intended source, but nothing is built, so nothing may assume it works. Until then the test is
not known to be able to fail.

**If this proves unworkable in practice, Slint is the named fallback** and the reasons are in
[stack.md](stack.md) — its testing API is stronger, at a cost in coupling and licensing. That is
a cycle-scale reversal, not a tweak, so the prototype that settles it should come early.

## Who writes what **[decided]**

- **Unit tests** — whoever wrote the code. They test what the code does.
- **Acceptance tests** — `acceptance-planner`, from the requirements, **without sight of the
  implementation**. They test what was asked for.

These are different jobs. Tests written by the author of the code encode the code's
assumptions, which is why the second kind is written first and never edited to make an
implementation pass. An acceptance test that seems wrong is a finding, not an obstacle.

## Not tested, and why **[proposed]**

Each with its reason. A gap on record is a decision; an unmentioned gap is a surprise.

| Not tested | Why, and what stands in |
|---|---|
| Wall-clock frame time, and the contended budget | No reference machine is available for routine testing. The counters carry the requirement instead — [platform.md](requirements/platform.md#how-this-is-verified-proposed) owns the reasoning and what it costs |
| Anything on macOS beyond building | No Mac to test on — [platform.md](requirements/platform.md) sets the tier and what may be claimed. A hosted runner says it builds and that structural assertions pass |
| Rendered pixels | Asserted on structure, never on pixels — the project-wide rule. Screenshots go to a human gate, for a person to look at |
| A puppet's visual correctness | Judgment, at a gate. Whether a rig looks right is not a thing an assertion reaches |
| The OBS output path | The spike has run ([2026-11-27-transparency-spike.md](research/2026-11-27-transparency-spike.md)) and its assertions are on the pixel values OBS emits, which is the one place pixels are the right unit. Nothing is testable in CI until the plugin [performing.md](requirements/performing.md#which-platforms-need-an-obs-plugin-decided) chose exists, because the route under test is a running OBS on a real desktop |
| Identifier reuse by a third-party writer | Duplicate identifiers within a file are detectable and rejected. Reuse across time by a writer that is not ours is undetectable without history nobody keeps. A recorded limitation, not a requirement |
| Third-party tools reading our files | The generated conformance fixture proves a valid puppet can be written without our code; it does not prove anyone else's tool reads one |
