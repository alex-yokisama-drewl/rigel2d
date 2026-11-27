# Stack

What it is built on, how to run it locally, and what runs on push.

The evidence behind every row below is in
[research/2026-11-26-stack.md](research/2026-11-26-stack.md), which marks each claim as read in a
library's own source, reported by a third party, or inferred from an API's shape. **Nothing in it
was observed** — no code was built or run — so the rows here are chosen on documented behaviour
and are not yet confirmed by use. [decisions.md](decisions.md) carries what lost.

## Built on **[proposed]**

| Layer | Choice | Why |
|---|---|---|
| Language | Rust | No collector to put a pause inside an 8 ms 99th-percentile budget, and a counting `GlobalAlloc` makes the per-frame heap-allocation counter [platform.md](requirements/platform.md) demands exact rather than approximated |
| GPU abstraction | wgpu | Creates pipelines when asked, synchronously, which is what [I5](invariants.md) needs. Its Metal backend is first-party, which is what makes macOS a build target rather than a port |
| Windowing | winit | One API across the platforms, and it lets the render surface be a window no toolkit owns |
| UI toolkit | egui, through `egui-wgpu` | Its test harness reads an AccessKit tree headlessly, which is the only Rust answer that checks the roles, accessible names and geometry of the tree egui builds. **It says nothing about delivery**: whether the platform adapter (UIA on Windows, AT-SPI on Linux) hands that tree to assistive technology cannot be automated here. MIT/Apache-2.0, which leaves the deferred plugin question open |
| Plugin host, when it arrives | Undecided, WebAssembly likely | Not built now. Recorded because Rust having no stable ABI is usually called a weakness and here is nearly irrelevant: [I1](invariants.md) wants a sandbox, and a WebAssembly host commits to no native ABI today |

### What the transparency spike settled **[decided]**

Transparency was the first thing this choice was exposed to, and it has been measured rather
than reasoned about ([research/2026-11-27-transparency-spike.md](research/2026-11-27-transparency-spike.md)).
**A decoration-free winit window emits correct premultiplied alpha into a real compositor on
Linux/X11, and alpha reaches OBS on both Linux and Windows. The stack survives.** What came out
of it is a set of conditions:

- **The backend must be pinned on Windows.** wgpu's default selection chose Vulkan on the machine
  tested, which crashes with an access violation — offscreen as well as windowed, so it is not a
  transparency fault. DX12 and OpenGL both render correctly. A build that leaves the choice to
  wgpu picks the one that failed there. **Whether the crash is that driver or Windows generally
  is unknown**: it was one machine, with an Intel HD Graphics 630 and a GeForce GTX 950M.
- **No wgpu backend on Windows offers an alpha-capable swapchain.** DX12 reported `Opaque` alone,
  Vulkan `Opaque` and `Inherit`, because presentation goes through `CreateSwapChainForHwnd`;
  desktop transparency there needs DirectComposition or a layered window and wgpu exposes
  neither. **Output is unaffected** — `Opaque` ignores alpha for desktop compositing rather than
  discarding it from the buffer, and the capture route reads the buffer.
- **The surface format and its colour space must be pinned explicitly, on every platform.** The
  spike pinned `Bgra8Unorm`. An sRGB surface applies the transfer function a second time and
  falsifies every measured value silently — a half-transparent white reads about 188 where the
  reference table says 128, which looks like a plausible result rather than a bug. wgpu 30 added
  `SurfaceConfiguration::color_space`, whose default `Auto` resolves to `Srgb`, so the hazard is
  now reachable without anyone asking for an sRGB *format*. With a linear format the shader's
  values still reach the surface unencoded — the colour space only tells the compositor how to
  read them — but anyone carrying over the older two-field mental model will miss it.

**What is affected is the editor's preview on Windows**, which cannot be transparent on the
desktop. That is a platform difference in Studio and it is not yet designed for; it belongs to
the Studio cycle.

macOS is unobserved, and so is the windowing assumption there.

### Moving to a new wgpu major **[decided]**

First-hand from the spike's port to wgpu 30, and the support for the re-check note at the foot
of this document. This note names its version so that it ages visibly.

- **Read the vendored source in one pass before porting**, under
  `~/.cargo/registry/src/*/wgpu-<major>*/src/api/`. wgpu 30's API differed from older versions
  enough to make guesses worthless: `PipelineLayoutDescriptor` has `immediate_size` where it had
  `push_constant_ranges`; `multiview` became `multiview_mask` and is now required on
  `RenderPassDescriptor` too; `PollType::Wait` is a struct variant needing `submission_index` and
  `timeout`; `get_mapped_range` returns a `Result`; `InstanceDescriptor` has no `Default`, though
  `Instance::default()` exists. Reading settled all of it at once, where compiling and fixing one
  error at a time would have taken several passes. **Do this before the port, not during it.**
- **The one that fails silently:** `SurfaceTexture::present` is gone in wgpu 30; presentation
  moved to `Queue::present`, and a frame that is merely dropped is *discarded* rather than shown.
  A port that drops the call compiles and renders nothing, with no error.

### The render surface is a window the toolkit does not own **[proposed]**

This is part of the choice rather than a refinement of it.

Live's render surface is a bare winit window with a wgpu surface, undecorated, and carrying
alpha — transparent **on the desktop** on Linux, but not on Windows, where no wgpu backend offers
an alpha-capable swapchain (above). The alpha reaches the capture route either way,
with no widget toolkit linked into its draw path. Live's control surface, and all of Studio, are
egui on a **separate** window sharing the same wgpu device, so Studio's viewport is a paint
callback into that device rather than a second renderer.

What follows is the reason for the split:

- **[I2](invariants.md) stops being a rule to remember.** Nothing but puppet content reaches the
  render surface because the toolkit has no access to that window at all.
- **egui's per-frame cost stays out of the player's budget.** egui lays out every frame and its
  own documentation puts that at 1–2 ms, against a 4 ms solo CPU budget. On a separate window
  with its own repaint cadence, idle unless something is being adjusted, it is not in the frame
  loop that the budget binds.

### Workspace **[proposed]**

| Crate | Owns |
|---|---|
| `rigel-format` | Parsing, validation, canonical writing. No GPU, no window, no toolkit |
| `rigel-runtime` | Evaluation and render, on `rigel-format`. See [runtime.md](runtime.md) |
| `rigel-studio` | The editor |
| `rigel-live` | The player |

`rigel-format` is separate so that the validator and the workload generator build and run without
a GPU or a window — which is what lets CI exercise them, and what makes a standalone validator a
thing a third party can run.

## Running locally **[proposed]**

**Run verbatim on a bare `ubuntu:24.04` container**, rather than remembered. Debian-family
package names; another distribution needs its own equivalents.

```
sudo apt-get update
sudo apt-get install -y curl build-essential   # rustup arrives over https; cargo needs a linker
curl -sSf https://sh.rustup.rs | sh -s -- -y   # the defaults are what we want
. "$HOME/.cargo/env"                           # rustup edits your login profile, not this shell
cargo build --workspace                        # the first cargo run installs the pinned toolchain
cargo run -p rigel-studio
cargo run -p rigel-live
```

rustup installs the pinned toolchain itself on the first `cargo` command, so no version is named
here and none goes stale.

**Three of those lines exist because the block failed without them**, and each is the kind of
thing that survives being remembered rather than run: `apt-get update` because a bare image
carries no package lists; `sh -s -- -y` because the installer is otherwise interactive; and
sourcing `cargo/env` because rustup writes PATH into your login profile and the very next line
runs `cargo`. The container has no `sudo`, so the walk supplied a shim that execs its arguments
and the text above ran unmodified.

**That package list is a floor and it will grow.** It was established against a workspace whose
crates have no dependencies at all. The Wayland or X11 development headers and the Vulkan loader
are pulled in by the windowing and GPU dependencies, which do not exist yet; whichever subsystem
adds them establishes their packages the same way and extends the list here.

**A container is not a desktop.** The walk establishes that the build and these commands work
from nothing. It says nothing about a display server, a GPU driver or a compositor — which is
where the render surface described above actually lives, and where the S0 spike found that the
interesting failures are.

## Configuration **[proposed]**

Neither application requires configuration to start, and that is a property worth keeping: a
beginner's first run must not involve a settings file.

What exists is application configuration in the platform's standard per-user location — the
output device, window placement, and the input port, and in Live the per-source normalisation and
channel renames. **None of it is scene content**, for the reason [scene.md](scene.md) gives. A
missing or unreadable configuration file falls back to defaults and never blocks startup, which
is the same rule as
[a missing output device](requirements/performing.md#nothing-stops-the-puppet-rendering-decided).

There are no secrets, no accounts and no service endpoints, because
[neither application opens an outbound connection](decisions.md).

### What a fallback to defaults may overwrite **[proposed]**

A configuration file that is **missing** falls back to defaults silently. One that **exists but
cannot be read** is kept and never overwritten: the application starts on defaults and says so
once in the control surface, which also offers a reset.

**Because** Live's configuration holds per-source normalisation, which is per-person calibration
that is expensive to redo, and a silent overwrite would destroy it. The reset exists because a
beginner will not find the per-user directory. The checks are in [ops.md](concerns/ops.md).

## On push **[proposed]**

In order, each blocking a merge:

| | Gate |
|---|---|
| 1 | `cargo fmt --check`, `cargo clippy -- -D warnings` |
| 2 | `cargo test --workspace` — unit and acceptance tests, and the structural UI assertions |
| 3 | The per-frame counters against their committed baseline, on the reference and stress puppets |
| 4 | `scripts/docs-check.sh` |
| 5 | The no-outbound-connection assertion over our own code |

Gate 3 is the one that carries the frame budget, and it runs without a GPU — see
[runtime.md](runtime.md#counters-are-the-requirement-proposed). Gates 1, 2 and 4 run on Linux;
the build is additionally exercised on Windows and on a hosted macOS runner, which can say that
it builds and that structural assertions pass and nothing more
([platform.md](requirements/platform.md) is explicit that no stronger macOS claim is founded).

### What a release requires **[proposed]**

A release is built by CI from a tag. The conditions it must meet are the entries under
*Building and shipping* in [ops.md](concerns/ops.md), and are not restated here.

| Platform | Packaging |
|---|---|
| Linux | A plain tarball and an AppImage. **Not a sandboxed format**: a sandbox breaks the absolute puppet paths a scene stores ([scene.md](scene.md)) and the per-user configuration location above |
| Windows | A zip, with an installer only if one proves necessary |
| macOS | A disk image, at its best-effort tier |

Artifacts are published with checksums on the same public forge as the issue tracker.

## Test command **[proposed]**

`.claude/test-cmd` holds it, which is what `scripts/test-record.sh` reads. The harness and the
gates are [testing.md](testing.md).

## What this choice is still exposed to **[open]**

Each of these would reverse or amend the choice, and each has a spike that settles it. Nothing
should be built that assumes any of them has gone well.

1. **egui's accessibility tree contains only what egui populates**, so a custom-painted canvas —
   Studio's mesh editor, its timeline, its parameter graph — contributes nothing unless its
   author says so. Studio is mostly custom canvases, which makes this a question about Studio's
   verifiability rather than about a toolkit feature. [testing.md](testing.md) carries the
   convention that answers it, and Slint is the named fallback if the convention proves
   unworkable.
2. **wgpu creates render passes and framebuffers lazily, on first use, inside the frame loop.**
   Pipelines and shaders — the expensive part — are eager, which is why wgpu passes at all. These
   two are cheap metadata objects and a warm-up pass during load is the mitigation
   [platform.md](requirements/platform.md) already prescribes, but they are a non-zero
   resource-creation counter after frame one, which is [I5](invariants.md) as stated.

**Re-check this document at the start of each cycle.** wgpu's major versions move fast and the
`wgpu-hal` internals the second point rests on are not a stable API.
