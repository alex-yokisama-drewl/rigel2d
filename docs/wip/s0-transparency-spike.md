# Feature: S0 — the transparency and alpha-output spike

**Branch:** `feature/s0-transparency-spike`
**Phase subsystem:** S0 in [plan.md](plan.md), which blocks every other subsystem

Pushes a known half-transparent frame through every candidate route into a real OBS and asserts
on the pixel values OBS emits. Done means the `[open]` in
[performing.md](../requirements/performing.md#which-platforms-need-an-obs-plugin-decided) is
resolved on evidence rather than on source reading, the Game Capture capability tension is
answered, point 1 of [stack.md](../stack.md#what-this-choice-is-still-exposed-to-open) is
settled, and a dated file lands in [research/](../research/).

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** all tasks done. Resume at **C. Finish, step 2** in the `feature` skill — review is already done (step 1); what remains is routing the handoff below, moving `plan.md` on, deleting this file in that same commit, and opening the PR
**Blocked on:** nothing. Whether macOS can be attempted at all is a question for the owner.

---

## Assignment

### Goal

Determine, by measurement rather than by reading documentation, whether each candidate route
delivers correct alpha from the player into OBS, on Linux and on Windows — and answer the
questions that ride along with it, which the tasks below name one by one.

### Out of scope

- **Production code.** This is a spike. Its scaffolding is expected to be discarded, and S1
  formalises whatever survives. No workspace, no crate layout, no push gates — those are S1.
- **Choosing the route.** The spike produces evidence; the choice among the options in
  `performing.md` is the project owner's and happens after.
- **Writing an OBS plugin.** Task 5 sketches what one would carry and cost. It does not build
  one.
- **Anything about puppets.** No format, no evaluation, no rendering of actual puppet content. A
  flat test frame with known pixel values is the whole of the artwork here.
- **macOS beyond its tier.** Attempted if a machine is reachable, recorded as not attempted if
  not. [platform.md](../requirements/platform.md) already has macOS `[open]` and possibly
  dropped.

### Doc set

- `CLAUDE.md` — current state and invariants
- `docs/vocabulary.md` — **load-bearing**, read first
- `docs/invariants.md` — **I2 and I5 bear directly**: nothing but rendered puppet content reaches
  the render surface, and no GPU resource creation in the frame loop
- `docs/requirements/performing.md` — *Transparent output is native and on by default*, *Which
  platforms need an OBS plugin* with its options table and the Game Capture caveat, and *The
  render surface carries nothing but the puppet* for the capability constraint
- `docs/stack.md` — *Built on*, *The render surface is a window the toolkit does not own*, and
  *What this choice is still exposed to*, whose point 1 is half of this spike
- `docs/research/2026-11-26-linux-alpha-output.md` — the candidates, the comparison table, and
  what would change the verdict
- `.claude/rules/testing.md` — what may be asserted and what needs a human gate

### Acceptance criteria

**None apply, and that is correct rather than an omission.** The acceptance documents cover the
puppet format and the editor's operations.
[plan.md](plan.md) states that S0 produces evidence and a resolved `[open]`, not production code,
and that runnable tests cannot exist before S1 gives them a crate and a command. The assertions
here are the spike's own, and they live with its throwaway scaffolding.

### Concern criteria

From `docs/concerns/`, the entries that bear on this scope:

- **`security`** — the player requests no screen-capture or window-capture capability. Task 3's
  `obs-vkcapture` route and task 6's Game Capture route both need checking against it, and the
  second is the unresolved tension named in `performing.md`.
- **`performance`** — no GPU resource creation after the first presented frame ([I5](../invariants.md)).
  The sender is throwaway, but a route that can only work by creating resources per frame is
  evidence about the route, so it is recorded rather than waved through.
- **`ops`** — what a beginner has to install for each route, which is a column in the research's
  comparison table and a real cost in the owner's decision.

### Already ruled out

Do not re-propose these; each lost with a stated reason in the documents above.

- **OBS's built-in PipeWire screen or window capture.** Portal-based, and it imports the dmabuf as
  `GS_BGRX`, so alpha is discarded on arrival. It also needs a capture capability the player is
  forbidden to request.
- **V4L2 loopback virtual camera.** Not zero-copy, needs root and a kernel module, alpha unknown.
- **Chroma key in OBS.** Rejected in `performing.md` — it constrains what colours a puppet may
  contain and degrades edges, putting an output-plumbing limitation onto the artwork.
- **Writing our own plugin for every supported platform.** `performing.md` answers it: it adds an
  install step on the two platforms that need none, and *uniformity is not worth that*. Task 5
  costs the Linux-first variant, which is a different option.
- **SSH into the owner's Windows machine to run the Windows leg.** An SSH session on Windows is
  non-interactive and outside the desktop session, where a swapchain and OBS behave differently
  from what this spike measures — so it would risk testing the wrong thing. The Windows leg is a
  self-asserting bundle the owner runs in their own session instead.

---

## Decisions

Made for this feature. Anything meeting [decisions.md](../decisions.md)'s admission test moves
there immediately — a narrow set. This list is everything else, and it survives every task
cleanup.

- **The spike covers every option in `performing.md`'s table** — the owner chose to decide
  after seeing evidence, and `performing.md` defers the choice to this spike deliberately. The
  third option (our own plugin for every platform) is already argued against there, so task 5
  costs only the Linux-first variant.
- **`obs-vkcapture` is tested before `obs-pwvideo`**, although the research names the latter as
  its technical-fit primary. A known-good result first makes a failure on the other route
  interpretable; with no baseline, a failure is ambiguous between the route and the sender.
- **Pixel assertions are numeric, at known coordinates** — not a model reading a screenshot.
  `.claude/rules/testing.md` reserves screenshots for the human gate, and this spike's
  measurement is the pixel values themselves, which is the deterministic end of that rule rather
  than an exception to it.
- **Premultiplied versus straight alpha is asserted explicitly on every route**, not inferred from
  the image looking right. `performing.md` names it as the likeliest place for a route to be
  quietly wrong.

## Tried and rejected

Approaches discarded during this branch. Not worth a permanent record; worth keeping until the
branch ends so nobody re-proposes them.

- *(none yet)*

## Caught at gate

What the user caught that automation did not. **Each becomes a checklist entry in
`docs/concerns/` before the phase closes** — `finish-cycle` harvests this section.

- **The harness left OBS in an unclean-exit state, and left `obs-websocket` enabled.** The owner
  saw OBS report it had stopped unexpectedly. Two separate faults behind one symptom: the harness
  stopped OBS with SIGTERM, which OBS treats as a crash and answers with a Safe Mode prompt on
  next launch — and Safe Mode disables third-party plugins, so the capture plugin would have
  silently vanished and the next run would have reported the input kind as merely unavailable.
  Separately, the restore path for `obs-websocket` only ran when the harness itself had flipped
  the setting, so a setting enabled once by hand stayed enabled for good.
  **Dimension: `ops`.** The check that would have caught both: *a harness that changes a
  user's application state restores it unconditionally from a recorded prior state, and verifies
  the process it started is actually gone rather than trusting the unit to be retired.*

---

## Tasks

One commit each. Status: `todo` / `awaiting approval` / `done`.

### 1. The test frame and the sender — `done`

`spike/alpha-sender/` renders the frame and scores it against `spike/alpha-sender/REFERENCE.md`,
which was written first. `--dump <png>` runs offscreen with no window; `--straight` emits the
other interpretation.

Both variants reproduce the table exactly on Intel ARL via Vulkan — every probe, no rounding
anywhere, which is what the exactly-representable 8-bit values were chosen to achieve. The
`RGB > A` test independently reports `false` for premultiplied and `true` for straight, so the
instrument discriminates by two unrelated means rather than one.

**What this licenses:** a route whose numbers disagree with the table is the route's fault, not
the sender's. That was the point of doing it before touching OBS.

### 2. Linux: does the window composite correctly at all — `done`

**Transparency works on Linux/X11 under Mutter, exactly as predicted.** Run it with
`python3 spike/alpha-sender/compositor-test.py`.

Every probe matched the composited column of `REFERENCE.md` on the nose, P4 included — fully
transparent pixels show the magenta backdrop rather than black, so the alpha channel reaches the
compositor. P1–P3 matching exactly also settles the caveat that table raised: **Mutter blends in
encoded 8-bit space**, not linear light, so no colour-space correction is needed anywhere.

**This is the first time transparency has been observed working anywhere in this project.**
`stack.md` records that every claim about it was drawn from reading source and documentation.
For Linux/X11 that is no longer true. Windows and macOS remain unobserved.

**The surface offers only `[PreMultiplied, Inherit]`.** `PostMultiplied` is not available, so
straight alpha cannot be requested from the surface on this platform at all — a route needing it
would have to convert in the shader. Premultiplied being the one offered is what the PipeWire
dmabuf contract wants, so this is favourable, but it means `--window --straight` cannot measure
what its name suggests and says so rather than reporting a misleading pass.

### 3. Linux route A: `obs-vkcapture` — `done`

**Alpha survives exactly. Colour does not.** Reproduced twice with the capture confirmed to be
the right window.

| | |
|---|---|
| Alpha at every probe | Bit-exact: 255, 128, 64, 192, 0, 255 |
| Colour at every probe | The **sRGB encoding** of what was sent: 128→188, 64→137, 192→225 |
| With `allow_transparency` off | Alpha forced opaque, **colour still encoded**. The shift is not part of the transparency path |

**The mechanism is pinned, not guessed.** `vkcapture.c` reads the Vulkan swapchain's
`VkColorSpaceKHR` and reports a colour space to OBS, so the transport is not colour-space-blind.
But OBS samples the imported dmabuf without decoding and applies one sRGB encode on output,
whatever the format says. Requesting a `Bgra8UnormSrgb` surface — the obvious remedy — made it
**worse in exactly the predicted way**: 223, 194, 242, which is the same encode applied twice.
The prediction was written down before that run.

**So the remedy is the opposite of the obvious one: render in linear light into a linear
surface**, and let OBS's single encode produce the correct display output.

**The finding that outlives this route.** The same bits mean different things to the two
consumers measured so far — Mutter blends them as display-encoded values (task 2), OBS treats
them as linear light. A frame correct for one is wrong for the other, and nothing in the
transport settles it. That is a colour-space hazard sitting underneath the shared-runtime
guarantee in [runtime.md](../runtime.md), which exists so that Studio's viewport and Live's
output cannot disagree — and the viewport goes through the compositor while the output goes
through OBS. **No document states this**, and it is not a vkcapture defect.

**Beginner setup, measured.** Not in Ubuntu 24.04's repositories at all, so it is built from
source — but it needs **no root**: `libobs` headers ship inside `obs-studio` itself, and the
Vulkan layer JSON installs into `~/.local/share/vulkan/implicit_layer.d`, which the loader
searches by default. One undeclared build dependency: SIMDe, which `obs-studio` does not pull in
and which libobs's own CMake config requires.

### 4. Linux route B: PipeWire dmabuf + `obs-pwvideo` — `done`

**Answered, though not the way the task expected: the route cannot be assembled on the target
platform today.** Alpha was never measured because the sending half does not build.

**The receiver is sound.** `obs-pwvideo` builds and installs per-user with no root, and
`formats.c` maps `BGRA` to `GS_BGRA` and `RGBA` to `GS_RGBA` — **alpha-bearing**, where built-in
OBS passes `GS_BGRX` at the same call and discards alpha. The research could only infer that;
it is now read from the shipped source. It accepts both `SPA_DATA_MemPtr` and `SPA_DATA_DmaBuf`.

**The sender does not build on Ubuntu 24.04 LTS.** libfunnel needs `pw_stream_is_lazy`,
`PW_KEY_NODE_SUPPORTS_LAZY` and `SPA_META_SyncTimeline`, and **all three are absent from the
PipeWire 1.0.5 headers the LTS ships**. Its `meson.build` declares no minimum version, so the
failure arrives as a wall of compile errors rather than a readable message. Going back before the
sync-timeline commit does not help — the previous commit fails on the lazy-scheduling API
instead, which is why the attempt stopped there rather than bisecting further into an untagged
history of a library whose own README calls the API unstable.

**Two further costs, found before the build wall.**

- **The producer must declare `media.type=Video`, `media.class=Stream/Output/Video` and
  `media.role` unset or `Production`**, or the plugin never lists it. A node published as
  `Video/Source` — GStreamer's default — is filtered out silently.
- **Nothing links producer to consumer.** The plugin found the target and the stream reached
  `paused`; frames flowed only after a manual `pw-link`, after which format and size negotiated
  correctly at 768×256. libfunnel's own README corroborates this, describing "hacky pw-cli
  shenanigans" for redirecting streams.

**The verdict that matters.** The research named this route technical-fit primary and
`obs-vkcapture` the fallback. On measured evidence that is backwards: vkcapture works end to end
today with alpha bit-exact, while this route needs a PipeWire newer than the LTS ships, manual
graph linking, and undocumented producer properties — against a maintenance note saying the
author no longer invests beyond their own use cases.

**What remains genuinely unknown** is whether alpha survives `obs-pwvideo`'s dmabuf import in
practice. The format table says it should. Settling it needs either a newer PipeWire or a
purpose-built producer, and neither is worth doing before the owner has chosen a route.

### 5. Our own Linux-first plugin: shape and cost — `done`

Costed against the two reference implementations built in tasks 3 and 4, and against the libobs
headers installed on this machine. No plugin was written.

**The plugin is the small half, and that inverts the usual objection.** Line counts of working
code that does this job:

| | Lines |
|---|---|
| `obs-vkcapture`, receiver half (`vkcapture.c` + `wlcursor.c`) | 1,541 |
| `obs-vkcapture`, sender half (Vulkan layer, GL injection, ELF hacks) | 4,182 |
| `obs-pwvideo`, receiver including its own PipeWire transport | 2,925 |

**Most of vkcapture's sender does work we would never need.** It intercepts *other people's*
swapchains — a Vulkan layer, `dlsym` interception, ELF patching — because it captures
applications that know nothing about it. Our player exports its own texture deliberately, so our
sender is "allocate a dmabuf, render into it, hand over the fd, format and modifier". The 4,182
figure is not our sender's cost; it is the cost of not controlling the application.

Nor does writing a plugin add the sender to our bill:
[performing.md](../requirements/performing.md) already records that the sender is per-platform
whichever route is chosen, so that work exists under every option in the table.

**What we would actually write:** an `obs_source_info` implementing `get_name`, `create`,
`destroy`, `get_width`, `get_height`, `get_defaults`, `update`, `get_properties` and
`video_render`, plus a transport receiver and one call to
`gs_texture_create_from_dmabuf`. On the evidence above, order of a thousand to fifteen hundred
lines of C for Linux, less than vkcapture's receiver because we need no Wayland cursor capture.

**The per-platform import calls are confirmed in the installed headers**, not inferred:
`gs_texture_create_from_dmabuf` (Linux), `gs_texture_open_shared` and `gs_texture_open_nt_shared`
(Windows), `gs_texture_create_from_iosurface` (macOS). So one plugin with a thin per-platform
import layer is real, and extending from Linux to the others later is open.

**The recurring cost, measured.** `OBS_DECLARE_MODULE()` bakes `LIBOBS_API_VER` into the module
at compile time and exports it as `obs_module_ver()`; this machine's libobs is 32.2.0 and
declares major versions to increment on breaking API changes. So **a plugin needs rebuilding and
re-releasing per OBS major version**, which has been roughly annual. On top of that, per the
research: the Windows plugin folder layout changed in OBS 33, Windows wants signing and macOS
notarisation.

**The argument the research could not make.** Task 3 found that the problem on the vkcapture path
is not alpha but colour: OBS treats the imported dmabuf as linear light and encodes once on
output, so the player must render in linear light to compensate. **A plugin of our own controls
that import**, and could declare the colour space correctly rather than having the renderer
compensate for someone else's assumption. That is a real engineering argument for option 2 that
did not exist before this spike, because the behaviour was not known.

**What it does not change.** `performing.md` rejects writing our own for every platform —
*uniformity is not worth that* — and nothing here disturbs that. This costs the **Linux-first**
variant only.

### 6. Windows: Game Capture — `done`

Run on an Optimus laptop: Intel HD Graphics 630 (driver 31.0.101.2140) and NVIDIA GeForce
GTX 950M (DX12 driver 27.21.14.5167, Vulkan driver 451.67).

**Rendering works; the Vulkan backend does not.** DX12 and OpenGL both produced the test frame
with every probe bit-exact. The **Vulkan** backend dies with `0xC0000005`, an access violation,
in the offscreen path as well as the windowed one — so it is not a transparency fault. On this
hardware wgpu selects Vulkan by default, which is why the first run looked like a total failure.
**A build that does not pin its backend on Windows picks the one that crashes here.**

**No backend on this machine offers an alpha-capable swapchain.**

| Backend | Surface alpha modes |
|---|---|
| DX12 | `[Opaque]` |
| Vulkan | `[Opaque, Inherit]` |

Neither offers `PreMultiplied` or `PostMultiplied`. This is the expected consequence of wgpu
presenting through `CreateSwapChainForHwnd`, which is opaque-only on Windows; desktop
transparency there needs DirectComposition or a layered window, and wgpu exposes neither.

**Game Capture works, and the GPU-pairing theory was right.** Putting the sender on the NVIDIA
GPU with `--adapter nvidia` made the capture appear immediately. Alpha is plainly visible: the
scene behind shows fully through the transparent band and partially through the three
semi-transparent ones.

**Alpha reaches OBS intact. The compositing is wrong.** From a `Screenshot (Source)` of the
captured frame, composited over black:

| Probe | Sent (premultiplied) | OBS emitted |
|---|---|---|
| P1 | 128 | 92 |
| P2 | 64 | 30 |
| P3 | 192 | 169 |

**The mechanism fits to within one unit on all three:** OBS multiplied by alpha a *second* time,
in linear light — `encode(decode(premultiplied) × alpha)` predicts 93, 30, 169. For that model to
fit, OBS must be holding our exact alpha values, so the transport is faultless and the error is
in the blend. It is what OBS does when it believes a frame carries **straight** alpha while we
are sending premultiplied.

**It is not an artefact of the screenshot path.** The earlier preview screenshot, composited over
a scene background, reads 94 at P1 where the same model predicts 95.5. OBS's ordinary
compositing does this, so a semi-transparent puppet would render too dark on stream.

**This is the failure `performing.md` predicted.** It names premultiplied-versus-straight as "the
likeliest place for it to be quietly wrong", and it is wrong, quietly, on a real platform — and
invisible to anyone judging by eye.

**The two platforms disagree, which is the finding that outlives both routes.**

| Route | What OBS assumes about the buffer | So the player must send |
|---|---|---|
| Linux, `obs-vkcapture` | Linear light, premultiplied | Linear, premultiplied |
| Windows, Game Capture | Straight alpha | Straight alpha |

Neither is documented. A renderer that satisfies one is wrong on the other, which is a second
colour-correctness hazard under [runtime.md](../runtime.md)'s shared-runtime guarantee — the
first being the sRGB shift in task 3. **Both are fixable by a plugin of our own**, which controls
the import and could declare what it is sending.

**Confirmed by a second run, predicted before it was taken.** Sending **straight** alpha gives
188, 137, 225 — the model's prediction exactly, on all three probes. *Allow Transparency* was
ticked throughout, and unticking it removes transparency altogether, so the toggle behaves as
documented and the measured path is the intended one.

**Each host wants a different encoding of the same frame, and fed correctly both agree.**

| Route | Treats the buffer as | Send it | Output |
|---|---|---|---|
| Linux, `obs-vkcapture` | Linear light, already premultiplied | Premultiplied, linear | 188, 137, 225 |
| Windows, Game Capture | sRGB-encoded, straight alpha | Straight, sRGB-encoded | 188, 137, 225 |

**This is the result that matters: the two platforms converge on identical output** once each is
sent what it expects — and on the linear-light composite, which is the colorimetrically correct
one. [runtime.md](../runtime.md)'s guarantee that the editor's viewport and the player's output
cannot disagree is therefore achievable; what it costs is a **per-platform output adaptation**,
small and precisely specified, which no document currently mentions.

Two models, each fitted to three probes, each predicting the other platform's numbers before
they were measured:

- Linux: `result = encode(rgb)` — premultiplied assumed, values read as linear.
- Windows: `result = encode(decode(rgb) × alpha)` — straight assumed, values read as encoded.

**What this does to the plugin decision.** A plugin of our own removes the adaptation entirely by
declaring what it sends, on every platform. That is now an argument from measurement rather than
from preference.

---

**Earlier, before the GPU pairing was found — kept because the failure mode matters.**

**Game Capture captured nothing — and not only from us.** The window appears in OBS's list and
the source renders empty, with `Allow Transparency` making no difference. **The owner reports the
same behaviour with real games on this machine**, so it predates anything here.

The machine is an Optimus laptop carrying both GPUs, and the Game Capture hook shares a texture
with the captured process, which it cannot do across two different GPUs. The sender ran on the
Intel GPU; if OBS runs on the NVIDIA one, capture is empty. A `--adapter` switch now exists so
each side can be put on the same GPU and the theory tested rather than argued.

**This is decision-relevant even if the pairing fixes it.** `performing.md` picks Game Capture
as the Windows route precisely because it needs no plugin. A route that fails silently — the
window listed, the source blank, no error — on a common laptop class is a poor default for an
audience of beginners, who have no way to tell a GPU-pairing problem from a broken application.
It is a mark against the route and a point in favour of owning the Windows side too.

**A second, unrelated symptom, recorded because it was offered and is worth not re-diagnosing.**
OBS Window Capture on that machine returns a stale image of whatever sat behind the window, not
updating. That is the legacy BitBlt capture method copying a screen region rather than reading
the window; the fix is the *Windows 10 (1903 and up)* method. It cannot serve this spike —
window capture reads the composited desktop, where the alpha is already gone — but it is why
window capture is not an alternative route here.

**The distinction that keeps this from being fatal.** Two different things were being conflated:

- **Desktop transparency** — the window blending over what is behind it. **Not available**
  through a wgpu swapchain on Windows.
- **Alpha in the rendered surface** — what a capture route reads. **Produced correctly**, proven
  by the offscreen dumps.

`Opaque` means the alpha channel is ignored *for desktop compositing*, not that it is discarded
from the buffer. So Game Capture may still read correct alpha from a window that looks opaque on
screen. **That is the one measurement still outstanding, and it decides the Windows story.**

**Unplanned finding: the output is byte-identical across backends and platforms.** Windows DX12,
Windows OpenGL and Linux Vulkan all produced the same PNG, MD5 `4ceef970…`. **This does not
settle the cross-platform tolerance that [puppet.md](../requirements/puppet.md) leaves `[open]`**
— the test frame is a flat-fill fragment shader with no interpolation, no trigonometry and no
accumulation, which is the easiest case there is. Real deformation is where cross-platform
divergence actually appears. What it does show is that rasterisation and format conversion are
exact across all three, so the tolerance question is about the evaluation maths and not about the
render path.

### 7. macOS: attempted at its tier — `done`

**Not attempted: no Mac is reachable.** Recorded as an outcome rather than left silent, which is
what the task asked for.

`plan.md` requires only that macOS be attempted at its tier, and
[platform.md](../requirements/platform.md) already carries macOS as `[open]` and possibly
dropped. **Syphon alpha therefore remains unobserved**, as it was before this spike — the one
platform whose claim still rests entirely on reading source and documentation.

Worth noting against the day it is tried: the research observed that the macOS path has no
premultiplied handling at all. Both measured platforms turned out to have a premultiplication
assumption that had to be discovered, so arriving at macOS expecting none would be the wrong
prior.

### 8. The research file, and resolving what this spike gates — `done`

A dated file in `docs/research/` carrying the measurements. Then resolve
`performing.md`'s `[open]` to what the evidence supports, answer the Game Capture tension in the
same document, and amend point 1 of `stack.md`'s exposure list.

**Done when:** `scripts/docs-check.sh` is clean, `docs/wip/open-questions.md` no longer lists
this as the largest open item, and the owner has each option costed well enough to choose
between them. **Needs the owner** — the choice of route is theirs, and this task presents it
rather than making it.

---

## Handoff

Filled as work proceeds; routed to the project documentation at finish. Leave a slot empty rather
than deleting it.

**Built:**
`spike/alpha-sender/obs-test.py` — drives OBS through its bundled obs-websocket: builds a scene
and a capture source, saves that source's own render as a PNG, and probes it. Measures the
source rather than the program output, so no scene background is composited behind the frame.
Reusable for the Windows leg, which is why it takes the input kind and settings as arguments.
`spike/alpha-sender/` — throwaway instrument for the spike: the test frame, the reference table
derived before any route was tried, and a probe report that scores a frame against it. Offscreen
dump only so far; no window yet.

`spike/alpha-sender/compositor-test.py` — runs the windowed mode, screenshots the real screen and
probes the composited result. It confirms the backdrop is where the sender reported before
trusting any probe inside it, so a result taken under an overlapping window is reported as
invalid rather than as a transparency failure.

**Decisions made:**
- **Our own OBS plugin for Linux, built to extend** — the owner's call on the evidence. Already
  routed: `decisions.md` carries the entry, `performing.md` the rule. Nothing to do.
- **The frame has one convention and every consumer converts from it** — already routed to
  `runtime.md` as `[proposed]`, with *which* convention left `[open]`. Nothing to do.
- **`obs-vkcapture` is tested before `obs-pwvideo`**, inverting the research's ordering, so that a
  known-good result makes a later failure interpretable. Local to the spike; needs no home.
- **The Windows leg is a bundle the owner runs, not an SSH session.** Local to the spike, but the
  reasoning generalises: a GUI-and-GPU measurement cannot be taken from a non-interactive session.
  Route only if a later cycle needs it.

**Contradicts docs:**
- `CLAUDE.md` says the compiler is "the one `rust-toolchain.toml` pins". **There is no such
  file.** The surrounding text does say the commands are unverified because nothing is built, but
  naming a file that does not exist reads as a statement of fact rather than an intention. S1
  creates it; until then the sentence is wrong.
- `CLAUDE.md` gives `cargo build --workspace` as the install step. **There is no workspace**, and
  by `plan.md`'s own design there will not be one until S1 — S0 deliberately carries its own
  scaffolding. The spike crate is standalone and not a member of anything.

**Traps:**
- **`--backend` was wired into the windowed path only, not the offscreen dump.** So the first
  Windows bundle could not test DX12 without a window, and the one test that would have separated
  "rendering is broken" from "transparency is broken" always ran on the backend that was
  crashing. A diagnostic that cannot vary the thing under suspicion is not a diagnostic.
- **The two windows were never positioned relative to each other.** Only their sizes were set,
  so the window manager placed each independently and the visible margin was whatever it happened
  to be. On Linux this went unnoticed because the harness used the positions the sender reports;
  it surfaced only when the Windows probe, which has no coordinates and locates the frame by
  finding the backdrop, read a bounding box of the wrong shape. Its own aspect-ratio check caught
  it. The sender now sets the test window's position explicitly.
- **A stale screenshot read as a failing result.** Two consecutive probe runs returned byte-identical
  numbers, which was the tell — a real capture of a changed scene cannot be identical. Worth
  distrusting any measurement that repeats exactly.
- **A capture route silently grabbed the wrong window.** The sender showed a backdrop and a test
  window; vkcapture captured the backdrop and returned a frame of flat magenta that was
  *internally consistent*. Only the harness's check on frame size caught it. `--solo` now
  suppresses the backdrop for capture runs. Any route that picks a surface for itself can do
  this, so **assert the captured size before trusting any probe**.
- **`systemd-run --user --scope` does not escape a snap.** A scope stays in the caller's mount
  namespace; it only appears to work for a command like `--version` that never loads the heavy
  libraries. OBS died with `symbol lookup error ... /snap/core20/.../libpthread.so.0`. A
  transient **service** — same command without `--scope` — is started by the user's systemd
  manager, outside the confinement, and works. Both sender and consumer must be outside, since a
  snap also has a private `/tmp`.
- **wgpu 30 added `SurfaceConfiguration::color_space`, and its default `Auto` resolves to
  `Srgb`.** It sits directly on top of what this spike measures. With a linear `Bgra8Unorm`
  format the shader's values still reach the surface unencoded — the colour space only tells the
  compositor how to read them — but anyone assuming the old two-field model will miss it.
- **`SurfaceTexture::present` is gone in wgpu 30**; presentation moved to `Queue::present`, and a
  frame that is merely dropped is *discarded* rather than shown. A port that drops the call
  compiles and renders nothing, with no error.
- **wgpu 30's API differs from every older version enough to make guesses worthless.**
  `PipelineLayoutDescriptor` has `immediate_size` where it had `push_constant_ranges`;
  `multiview` became `multiview_mask` and is now required on `RenderPassDescriptor` too;
  `PollType::Wait` is a struct variant needing `submission_index` and `timeout`;
  `get_mapped_range` returns a `Result`; `InstanceDescriptor` has no `Default`, though
  `Instance::default()` exists. Reading the vendored source under
  `~/.cargo/registry/src/*/wgpu-30*/src/api/` settled all of it in one pass, where compiling and
  fixing one error at a time would have taken several. **Do that first in S5.**
- This is first-hand support for `stack.md`'s own warning that wgpu's major versions move fast.
  It is evidence for that note, not a contradiction of it.
- **An sRGB surface format would have falsified every number silently** — about 188 where the
  table says 128, which looks like a plausible result rather than a bug. The sender pins
  `Bgra8Unorm`. `REFERENCE.md` records it as the first thing to re-check if a route lands close
  to the table but not equal.

**Assumptions:**
The Windows findings so far come from **one machine with Intel HD Graphics 630**, Kaby Lake
hardware whose Windows Vulkan driver is legacy and no longer updated. Nothing about Windows in
general should be concluded from it until a second backend, and ideally a second machine, agree.

**Deferred:**
- **The instrument's missing band**, specified at the foot of `spike/alpha-sender/REFERENCE.md`.
  Deliberately not added, because changing the frame's geometry would break the correspondence
  between the committed instrument and the committed measurements. Whoever measures a second
  machine or macOS adds it first.
Route B's alpha measurement. The sending half does not build on the target platform, so measuring
it would require a newer PipeWire or a purpose-built producer — not worth doing before a route is
chosen. Everything else about the route is settled in task 4.
