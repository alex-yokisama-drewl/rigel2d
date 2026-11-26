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
| Soak | Accelerated time, headless, synthetic input | `tests/soak/` in `rigel-runtime` |

`.claude/test-cmd` holds `cargo test --workspace`. The counter gate runs as its own job: it
compares against a committed baseline rather than passing or failing on its own terms, and
[platform.md](requirements/platform.md#how-this-is-verified-proposed) explains why that half carries the
whole frame budget.

**No test requires a GPU.** This is a design constraint rather than a convenience —
[runtime.md](runtime.md) describes the command stream the counters are read from, and wgpu's noop
backend supplies the device so the load path still runs real code.

## Visual verification **[proposed]**

**`visual-checks: automated`**, as `CLAUDE.md` states.

`scripts/ui-assertions.md` carries the patterns, written against a browser and Playwright. **The
project is not a browser**, so the translation is set out here once rather than rediscovered per
test.

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
state. The assertion therefore moves — contrast is checked exhaustively over `Visuals`, for both
themes, with no widget tree involved at all.

**This is wrong in exactly one case**, and it is the case to watch: a widget painted with a
colour overridden at the call site is invisible both to the theme assertion and to the
accessibility tree. **An overridden colour in interface code is therefore a reviewable defect**
unless it is a puppet's own content, which is not interface.

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
  accessible name, through egui's widget-info call. A gizmo, a keypoint, a timeline marker and a
  draggable vertex are each an element.
- **Every piece of persistent chrome is marked as such**, so the overlap assertion has both
  sides of its comparison.
- **An element that cannot be registered cannot be shipped.** This is the rule that keeps the
  coverage honest, and it is the one that will be under pressure, because registering is work
  and skipping it is invisible until a gizmo ships under a panel.

A test asserts that every interactive element in each canvas appears in the tree, so that
*forgetting* fails rather than silently narrowing what the other assertions cover. Tagged
`test`, and it is the entry that makes the rest of the convention self-enforcing.

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
| Wall-clock frame time, and the contended budget | No reference machine is available for routine testing. The per-frame counters carry the requirement instead, and [platform.md](requirements/platform.md#how-this-is-verified-proposed) is explicit that this leaves "runs well on weak hardware" unverified rather than merely unmeasured |
| Anything on macOS beyond building | No Mac to test on. A hosted runner says it builds and that structural assertions pass; window compositing, transparency against a real compositor, and whether it feels right are all unclaimed |
| Rendered pixels | Asserted on structure, never on pixels — the project-wide rule. Screenshots go to a human gate, for a person to look at |
| A puppet's visual correctness | Judgment, at a gate. Whether a rig looks right is not a thing an assertion reaches |
| The OBS output path | Until the spike in [performing.md](requirements/performing.md#which-platforms-need-an-obs-plugin-open) runs, there is nothing to test against. The spike's own assertions are on the pixel values OBS emits, which is the one place pixels are the right unit |
| Third-party tools reading our files | The generated conformance fixture proves a valid puppet can be written without our code; it does not prove anyone else's tool reads one |
