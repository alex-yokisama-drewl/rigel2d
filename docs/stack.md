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

### The render surface is a window the toolkit does not own **[proposed]**

This is part of the choice rather than a refinement of it.

Live's render surface is a bare winit window with a wgpu surface, transparent and undecorated,
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

**Unverified.** Nothing is built yet, so these steps describe the intended shape rather than a
path anyone has walked. The ops checklist entry that asserts this document works when followed
literally cannot pass until there is something to run; the first feature to land code is what
makes these steps real, and it must correct them here.

```
cargo build --workspace              # with rustup installed: the compiler is the one rust-toolchain.toml pins
cargo run -p rigel-studio
cargo run -p rigel-live
```

Linux additionally needs the usual Wayland or X11 development headers and a Vulkan loader; the
exact package list belongs here once it has been established on a clean machine rather than
remembered.

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

1. **Transparency has not been observed anywhere.** winit's own documentation calls transparency
   "more of a hint for the system" on some platforms and gives no per-platform notes. **This is
   the same spike [performing.md](requirements/performing.md) already requires** for the OBS
   alpha path — both need a decoration-free window emitting correct premultiplied alpha into a
   real compositor on Linux, Windows and macOS, so there is one piece of work here, not two. If it
   needs platform-specific branches, macOS-as-a-build-target weakens for every Rust option and
   Qt's maturity starts to be worth its other costs.
2. **egui's accessibility tree contains only what egui populates**, so a custom-painted canvas —
   Studio's mesh editor, its timeline, its parameter graph — contributes nothing unless its
   author says so. Studio is mostly custom canvases, which makes this a question about Studio's
   verifiability rather than about a toolkit feature. [testing.md](testing.md) carries the
   convention that answers it, and Slint is the named fallback if the convention proves
   unworkable.
3. **wgpu creates render passes and framebuffers lazily, on first use, inside the frame loop.**
   Pipelines and shaders — the expensive part — are eager, which is why wgpu passes at all. These
   two are cheap metadata objects and a warm-up pass during load is the mitigation
   [platform.md](requirements/platform.md) already prescribes, but they are a non-zero
   resource-creation counter after frame one, which is [I5](invariants.md) as stated.

**Re-check this document at the start of each cycle.** wgpu's major versions move fast and the
`wgpu-hal` internals the third point rests on are not a stable API.
