# Feature: S0 — the transparency and alpha-output spike

**Branch:** `feature/s0-transparency-spike`
**Phase subsystem:** S0 in [plan.md](plan.md), which blocks every other subsystem

Pushes a known half-transparent frame through every candidate route into a real OBS and asserts
on the pixel values OBS emits. Done means the `[open]` in
[performing.md](../requirements/performing.md#which-platforms-need-an-obs-plugin-open) is
resolved on evidence rather than on source reading, the Game Capture capability tension is
answered, point 1 of [stack.md](../stack.md#what-this-choice-is-still-exposed-to-open) is
settled, and a dated file lands in [research/](../research/).

> This file lives only on its branch and is deleted in the last commit before merge. It exists
> so a session that has never seen this work can pick it up from this file alone — and so an
> implementer with no other context can be briefed from it. Strip anything that stops serving
> either. Git holds the history.

## Status

**Current task:** 6 — Windows, Game Capture
**Blocked on:** task 6 — the owner running the Windows bundle. Everything else on Linux is done.

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

### 6. Windows: Game Capture — `todo`

Cross-compile the sender to `x86_64-pc-windows-gnu`, and build a self-asserting bundle: a script
that configures an OBS scene collection, records, extracts the probe values and writes them to a
file. The owner runs it in their own desktop session and returns the file.

Also answers the capability tension: Game Capture hooks the player's swapchain, which is OBS
reaching into our process. `performing.md` flags as unresolved whether that is compatible with
*the player requests no screen-capture or window-capture capability*, and says the spike should
answer rather than assume.

**Done when:** the returned file's probe values are asserted against the reference table, and the
capability question has an answer with its reasoning. **Needs the owner**, by construction — they
run it. What would count as wrong: a bundle that needs them to judge anything, rather than to run
one command and send back a file.

### 7. macOS: attempted at its tier — `todo`

Syphon client with its transparency setting, if a machine is reachable.

**Done when:** either probed as above, or recorded as not attempted with the reason. `plan.md`
asks only that it be attempted at its tier, and `platform.md` has macOS `[open]` and possibly
dropped — so *not attempted* is a legitimate outcome and must be written down as one rather than
left silent.

### 8. The research file, and resolving what this spike gates — `todo`

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

**Contradicts docs:**
- `CLAUDE.md` says the compiler is "the one `rust-toolchain.toml` pins". **There is no such
  file.** The surrounding text does say the commands are unverified because nothing is built, but
  naming a file that does not exist reads as a statement of fact rather than an intention. S1
  creates it; until then the sentence is wrong.
- `CLAUDE.md` gives `cargo build --workspace` as the install step. **There is no workspace**, and
  by `plan.md`'s own design there will not be one until S1 — S0 deliberately carries its own
  scaffolding. The spike crate is standalone and not a member of anything.

**Traps:**
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

**Deferred:**
Route B's alpha measurement. The sending half does not build on the target platform, so measuring
it would require a newer PipeWire or a purpose-built producer — not worth doing before a route is
chosen. Everything else about the route is settled in task 4.
