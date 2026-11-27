# Does transparent output actually work, and what does each route do to our frames?

**Measured:** 2026-11-27 · **Feeds:** *Which platforms need an OBS plugin* in
[performing.md](../requirements/performing.md), *What the transparency spike settled* in
[stack.md](../stack.md#what-the-transparency-spike-settled-decided), and the frame convention
in [runtime.md](../runtime.md) ·
**Verdict:** Transparency works, on Linux and on Windows, and has now been observed rather than
inferred. Every route measured carries alpha correctly. **None of them carries colour correctly
without being told what it is being sent**, and no host documents its assumption or reports a
mismatch. The Linux route this project was leaning toward cannot be assembled on the target
distribution at all.

This is the S0 spike that [plan.md](../wip/plan.md) put before every other subsystem, and the
reason it was placed there — that a bad answer would reopen the stack choice — turned out to be
half right. The stack survives. What it exposed instead is a correctness hazard in the output
path that nothing in the documentation anticipated.

## How this was measured

A test frame of six vertical bands at fixed alphas, with every expected value **written down
before any route was built** ([the reference table](../../spike/alpha-sender/REFERENCE.md)).
The alphas are 0, 64, 128, 192 and 255 so that no value lands on a rounding boundary. Nothing
here was judged by eye; each result is named pixels compared as integers.

**The instrument has a blind spot, and it bounds what follows.** Every band is full-intensity
white, red or green, so in premultiplied form the stored colour equals the alpha at each probe,
and in straight form every channel is 0 or 255, where the transfer function is the identity. The
opaque and fully-transparent bands discriminate nothing by design. **So no probe has a colour
that is neither 0, 255, nor equal to its own alpha**, and neither platform's measurements
identify its behaviour on their own — what pins each is the pair of runs taken together. A band
of mid-intensity grey at an unequal alpha would break both degeneracies at once, and should be
added before anything is measured on a second machine or on macOS.

That discipline earned itself twice. Both colour faults below are invisible to a person looking
at the output — one reads as *slightly bright artwork*, the other as *slightly dark edges* — and
both would have passed a human gate.

## What each route does

| Route | Alpha | Colour | Reads the buffer as |
|---|---|---|---|
| Linux, `obs-vkcapture` | Bit-exact | sRGB-encoded on output | Linear light, already premultiplied |
| Windows, OBS Game Capture | Bit-exact | Alpha applied a second time | sRGB-encoded, straight alpha |
| Linux, X11 compositor (Mutter) | Honoured — a desktop capture carries no alpha of its own to compare | As predicted, but **not** the same answer as OBS | Encoded values, blended in encoded space |
| Linux, PipeWire + `obs-pwvideo` | Unmeasured | Unmeasured | Sender will not build — see below |
| macOS, Syphon | Not attempted | Not attempted | No Mac was reachable |

Two fitted behaviours, each established on one run and then used to predict a **different run on
the same platform** before it was taken — the Linux double-encode, and the Windows straight-alpha
run. Both predictions held to within one 8-bit unit.

- **Linux:** `result = encode(rgb)` — premultiplied assumed, stored values read as linear.
- **Windows:** `result = encode(decode(rgb) × alpha)` — straight assumed, values read as encoded.

The pixels establish *what transform was applied*, not which component applied it: separating
OBS's compositing from its screenshot export rests on a single preview capture over a background
whose colour was not recorded. **The decision does not depend on resolving that** — sending
straight alpha is correct whether the extra multiply happens in the composite or only at export.

Neither predicts the other platform, and neither is claimed to. What makes each more than a
three-point fit is the second run on its own platform, where a rival with no alpha term fits the
first run equally well and the second excludes it.

**Fed what each expects, the two capture routes produce identical output** — and that is an
identity rather than a coincidence of three probes: Linux stores `L·a` and emits `E(L·a)`,
Windows stores `E(L)` and emits `E(D(E(L))·a) = E(L·a)`, equal for every `L` and `a`. So the
divergence between them is fixable by conversion at the output boundary, which is what
[runtime.md](../runtime.md) now requires.

**The desktop compositor is not one of them.** It blends our window over a background we neither
control nor see, in encoded space where OBS works in linear, so no stored value satisfies both:
white at half alpha reaches the desktop as 128 and the stream as 188 over black, 192 and 205 over
mid-grey. The consequence — that anything meant to match the stream composites inside the frame
rather than leaving it to the host — is a requirement in `runtime.md`, not a conversion.

## The findings that outlive the routes

**Alpha was never the problem.** It survived every route measured, bit-exact at every probe. The
question the spike was commissioned to answer has a clean yes, and the risk was somewhere else
entirely.

**Each host silently assumes an encoding, and they disagree.** A renderer correct for one is
wrong for the other, by an amount that reads as a plausible picture rather than a fault — which
is not the same as a small one. A half-transparent white measured 92 where it should have been
188, a factor of two in luminance, and the mismatch also shifts opaque mid-tones, so it is not
confined to edges.
Neither host reports a mismatch, and the transports carry no colour-space tag that would settle
it. The remedy that looks obvious on Linux — request an sRGB surface — makes it worse in exactly
the predictable way, applying the encode twice.

**The editor's viewport and the player's output reach the screen through different hosts.**
A desktop compositor blends in encoded space; OBS works in linear light. The guarantee in
[runtime.md](../runtime.md) that a rig cannot look right while authored and wrong while performed
is therefore a property of these conversions, not of the shared runtime alone.

**Game Capture fails silently on a dual-GPU laptop.** Its hook shares a texture with the captured
process, which cannot cross GPUs; with the player on the integrated GPU and OBS on the discrete
one, the window lists, the source stays blank, and nothing anywhere reports an error. The project
owner had independently seen the same with real games. For an audience of beginners this is a
poor default: there is no way to tell a GPU-pairing problem from a broken application.

**wgpu selects a backend that crashes.** On the Windows machine tested, the default selection is
Vulkan, which dies with an access violation — in the offscreen path as much as the windowed one,
so it is not a transparency fault. DX12 and OpenGL both render correctly. **A build that does not
pin its backend on Windows picks the one that fails**, and nothing in [stack.md](../stack.md)
currently says it must.

**No wgpu backend on Windows offers an alpha-capable swapchain.** DX12 reports `Opaque` alone,
Vulkan `Opaque` and `Inherit`. This follows from presenting through `CreateSwapChainForHwnd`;
desktop transparency on Windows needs DirectComposition or a layered window, and wgpu exposes
neither. **It does not block output**, because `Opaque` means alpha is ignored for desktop
compositing rather than discarded from the buffer, and Game Capture reads the buffer. What it
does block is a transparent *preview* on the desktop, which is a platform difference in the
editor that no document anticipates.

## The route that cannot be assembled

`obs-pwvideo` is sound: it builds per-user with no root, and its format table maps `BGRA` and
`RGBA` to alpha-bearing OBS formats where built-in OBS passes `GS_BGRX` and discards alpha. That
structural advantage — the research's central claim about this route — is confirmed from the
shipped source.

**Its sending half does not build on Ubuntu 24.04 LTS.** libfunnel needs `pw_stream_is_lazy`,
`PW_KEY_NODE_SUPPORTS_LAZY` and `SPA_META_SyncTimeline`, none of which exist in the PipeWire
1.0.5 headers the LTS ships, and it declares no minimum version, so the failure arrives as a wall
of compile errors. The commit before the sync-timeline work fails on the lazy-scheduling API
instead. Ubuntu LTS is what this audience runs; "upgrade PipeWire system-wide first" is not a
beginner instruction.

Two further costs found before that wall: the producer must declare `media.type`, `media.class`
and `media.role` or the plugin filters it out in silence, and **nothing links producer to
consumer** — frames flowed only after a manual `pw-link`, which libfunnel's own README
corroborates by describing "hacky pw-cli shenanigans".

**So the research's ordering is inverted by measurement.** It named this route technical-fit
primary and `obs-vkcapture` the fallback. `obs-vkcapture` works end to end today.

## What a plugin of our own would cost

Costed against both reference implementations rather than estimated.

| | Lines |
|---|---|
| `obs-vkcapture`, receiver half | 1,541 |
| `obs-vkcapture`, sender half (Vulkan layer, GL injection, ELF hacks) | 4,182 |
| `obs-pwvideo`, receiver including its own transport | 2,925 |

**The plugin is the small half.** Most of that sender intercepts *other people's* swapchains,
because it captures applications that know nothing about it; our player exports its own texture
deliberately. And a plugin adds no sender work to the bill, because
[performing.md](../requirements/performing.md) already records the sender as per-platform under
every option.

The recurring cost is measured, not guessed: `OBS_DECLARE_MODULE()` bakes `LIBOBS_API_VER` in at
compile time and libobs increments its major version on breaking changes, so a plugin is rebuilt
and re-released per OBS major — roughly annual — on top of the Windows folder-layout change in
OBS 33, Windows signing and macOS notarisation.

**The argument that did not exist before this spike:** both colour hazards above come from a host
guessing what we sent. A plugin of our own controls the import and can declare it, which removes
the per-platform conversion rather than compensating for it.

## What is still unknown

- **macOS, entirely.** Syphon alpha is unobserved, as before. Both measured platforms turned out
  to carry a premultiplication assumption that had to be discovered, so approaching macOS
  expecting none would be the wrong prior.
- **`obs-pwvideo`'s dmabuf import in practice.** Its format table says alpha survives; the
  sending half would not build to prove it.
- **Whether the Windows findings generalise.** One machine, with Intel HD Graphics 630 (driver
  31.0.101.2140) and an NVIDIA GeForce GTX 950M (DX12 driver 27.21.14.5167, Vulkan 451.67),
  against OBS 33.0.1. The Vulkan crash in particular may be that driver rather than Windows.
- **Whether a second GPU generation agrees** on the alpha modes offered. The swapchain result is
  architectural and should hold; it has been seen once.

## Reproducing it

`spike/alpha-sender/` carries the instrument. `REFERENCE.md` holds the expected values and the
reasoning for each. `compositor-test.py` measures a real compositor, `obs-test.py` drives OBS
through its bundled websocket, and `probe-windows.py` scores images returned from a machine this
session cannot reach. The sender takes `--backend`, `--adapter`, `--straight` and `--srgb`, each
of which exists because some host's assumption had to be varied to be measured.

Traps that would cost a future measurer real time again:

- **A capture route that picks its own surface can grab the wrong window and return a frame that
  is internally consistent.** `obs-vkcapture` captured the backdrop rather than the test window
  and returned flat magenta that scored as coherent. Only a check on frame size caught it.
  **Assert the captured frame's size before trusting any probe.** The sender's `--solo`
  suppresses the backdrop for capture runs.
- **Two consecutive probe runs returning byte-identical numbers mean a stale screenshot, not a
  result.** A real capture of a changed scene cannot be identical. Distrust any measurement that
  repeats exactly.
- **`systemd-run --user --scope` does not escape a snap.** A scope stays in the caller's mount
  namespace, and only appears to work for a command such as `--version` that never loads the
  heavy libraries; OBS died with a `symbol lookup error` against the snap's `libpthread`. A
  transient **service** — the same command without `--scope` — is started by the user's systemd
  manager, outside the confinement, and works. Both sender and consumer must be outside it,
  because a snap also has a private `/tmp`.
