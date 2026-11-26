# Requirements — the player

What the application that performs a puppet live must do. The puppet model is in
[puppet.md](puppet.md).

The player renders a puppet driven by live input and delivers the result to streaming software.
Everything here is in service of one sitting: start it, have it already set up, stream.

## Transparent output is native and on by default **[decided]**

The player outputs with an alpha channel on every supported platform, and transparency is on by
default rather than configured.

**Because:** the alternative in practice is a chroma key filter in OBS, which constrains what
colours a puppet may contain and degrades edges — a limitation about output plumbing landing on
the artwork, which is the wrong place for it. Inochi Session's transparency is a Windows-only
plugin with no Linux equivalent, which makes this a gap rather than a preference.

### Only Linux needs an OBS plugin **[open]**

**The choice among the options below is deferred to the spike, deliberately.** Alpha has not
been observed working end to end on any candidate path, so picking one now would be picking on
the strength of source reading alone. The spike described at the end of this section is the
trigger: it decides this, and nothing should be built that assumes an answer before it runs.
What is already settled, and does not wait for the spike, is that output sits behind a narrow
interface that can host more than one backend.

The first draft of this section required alpha output "without a plugin". Research has
established what each platform actually offers
([2026-11-26-linux-alpha-output.md](../research/2026-11-26-linux-alpha-output.md)):

| Platform | Plugin-free route to alpha | |
|---|---|---|
| macOS | **Yes** | OBS ships a Syphon client with a transparency setting |
| Windows | **Yes** | Built-in Game Capture with "Allow Transparency". Spout, despite being the platform's convention, is a third-party plugin and never shipped in OBS. **See the caveat below** |
| Linux | **No** | OBS's built-in PipeWire path is portal-based and declares the texture without an alpha channel, so transparency is discarded on arrival |

**Linux is the odd one out, not the general case.** This matters because it rules out the
tempting symmetry: writing one plugin of our own for all three platforms would *add* an install
step on the two platforms that do not need one, in the name of consistency. Uniformity is not
worth that.

So the decision is narrower than it first appeared:

| Option | |
|---|---|
| Depend on a third-party Linux plugin | No work for us. The two differ in risk: `obs-vkcapture` is actively released and the research rates it the safer dependency, while the `obs-pwvideo` route also pulls in `libfunnel`, which is single-author, has no tagged release, and declares its API unstable |
| **Write our own, Linux first, built to extend** | We own the Linux story. The sender is per-platform either way; `libobs` has one texture-import call per platform, so extending later is open without being committed to now |
| Write our own for all three | Consistency at the cost of an install step where none is needed |

Whichever is chosen, the recurring cost of owning an OBS plugin is **coupling to OBS releases**,
not the code: OBS refuses to load plugins built for a newer release, offers no stability
promise, changed its Windows plugin folder layout in version 33, and requires signing on
Windows and notarisation on macOS.

**An unresolved tension in the Windows route.** Game Capture works by hooking the player's
swapchain, which is OBS reaching into our process rather than the player handing over frames.
Whether that is compatible with "the player requests no screen-capture or window-capture
capability" below turns on whose capability it is — arguably OBS's, since the player requests
nothing and only presents its own window. It is not settled, and the spike should answer it
rather than assume.

**Settled regardless:** output sits behind a narrow interface that can host more than one
backend, because the candidates differ in maturity and any of them may fail.

**A spike comes before design lock, and it covers all three platforms.** Alpha has not been
observed working end to end on *any* path — not Linux, not Windows Game Capture, not macOS
Syphon. Every conclusion above is drawn from reading source code and documentation. The spike
pushes a known half-transparent frame through each route into a real OBS and asserts on the
pixel values OBS emits; premultiplied-versus-straight alpha is the likeliest place for it to be
quietly wrong, and the macOS path has no premultiplied handling at all, which is a hint rather
than a conclusion.

## Nothing stops the puppet rendering **[decided]**

No dialog, error, file picker or interface state in the control surface stalls the render loop.
A puppet frozen on stream is the worst outcome this application can produce, and it is worse
than whatever error caused it.

Consequences that follow:

- **A failed load changes nothing visible.** A puppet or scene is fully parsed and validated
  before it replaces what is displayed. A failure leaves the previous puppet rendering —
  never a partial puppet, a placeholder, or an error texture.
- **The same applies to a puppet changing on disk.** If the editor saves while the player is
  live, the player validates the new file and keeps the last good puppet if the new one is
  invalid.
- **One broken item never blocks a scene.** A missing puppet, a channel mapping that resolves to nothing, or an
  absent channel loads as a dormant entry, flagged in the control surface; the rest of the scene renders. A dormant
  entry survives the next save, because the user may yet restore what is missing.

## The render surface carries nothing but the puppet, including when things fail **[decided]**

Extending the clean-surface requirement to the failure paths, which is where it is most likely
to be violated and most expensive when it is: errors, dialogs, notifications, tooltips and debug
output never reach the render surface.

**The player requests no screen-capture or window-capture capability.** Output plumbing that
works by capturing a window or the desktop creates the route by which another window's contents
appear on a live stream. This constrains the Linux transparency research above, and is a reason
to prefer a mechanism where the player hands over its own frames.

## How a streamer learns something broke **[open]**

The render surface must stay clean and the control surface may be behind a full-screen game. So
the question of how a live failure — a source dropping, an output failing — reaches a person who
is not looking at the player is a requirement and not a detail. Candidates include a sound, an
OS notification, and a tray state. Undecided.

## What the puppet does when its source drops is a scene setting **[decided]**

A scene chooses what happens when a source stops sending, from a fixed set: **hold the last
pose**, **ease to a neutral pose**, or **play an idle animation**. It is a scene setting because
the right answer depends on the puppet and on what the streamer is doing — a rig with no
authored idle cannot offer the third, and a streamer who steps away wants something different
from one whose tracker glitched mid-sentence.

Constraints on all three:

- **It never freezes mid-expression, jumps, or crashes.** Whichever behaviour is selected, the
  transition into it is continuous.
- **Reconnection is automatic and needs no restart**, and the transition back out is continuous
  too.
- **An idle animation that the puppet does not carry falls back to easing to neutral**, rather
  than failing the scene load. A scene is a file strangers share, so it may name a behaviour the
  puppet it is pointed at cannot perform.
- **Easing to neutral means each parameter returning to its own authored neutral keypoint**, not
  to zero.

**A new scene defaults to easing to neutral.** It is unambiguous on stream, and it is the one
behaviour every puppet can perform without carrying authored content for it.

## A source can drive the puppet and nothing else **[decided]**

The input port is the application's only network surface and it is treated as untrusted.

- **It binds to loopback, and the first release does nothing else.** Off-machine sources —
  which phone-based tracking needs — are **not rejected, only deferred**; the design must not
  preclude them. When they arrive the shape is already settled: an explicit opt-in made in the
  running application, never on by default, with a visible indication while it is on. Whether
  that indicator is sufficient on its own, or wants an address allow-list or a pairing step, is
  decided then and not now.
- **A scene never enables off-machine listening.** Scenes are files that strangers share, and
  "open my port to the network" must not be able to arrive inside one.
- **A source may set channel values and trigger animations. It may do nothing else** — not load
  a puppet, not change the scene, not touch the filesystem, not reconfigure output.
- **Channel values are finite and clamped to the parameter's range before any binding is
  evaluated.** A source cannot deform a puppet past its authored extremes, and a NaN never
  reaches the renderer.
- **The port is bounded** — message size, the number of distinct channel names, and the length
  of a name. Channel names being an open vocabulary is the point of the design and also the way
  a flood exhausts memory, so the cap is what makes the openness safe.

## A scene survives being interrupted **[proposed]**

Scene changes are written as they happen, not only on exit, and written atomically — a crash or
a kill mid-stream cannot corrupt a scene. After a crash, relaunching restores the scene and
resumes with no setup.

A scene refers to a puppet by a relative path together with the puppet's own identifier, and
binds to parameters by identifier ([puppet.md](puppet.md#every-entity-has-a-stable-identity)),
so that editing a puppet does not break the scene that uses it. What moves between machines and
what does not — output device, window placement, absolute paths — are kept separate, and a
missing output device falls back rather than failing the load.

## Deferred **[open]**

Recorded rather than left as gaps, and tracked in `docs/wip/open-questions.md`: whether
OpenSeeFace's protocol is our native one or sits behind a protocol of our own, which decides
whether the port is reachable from a web page; and whether a scene can be edited while
streaming.

Off-machine sources are deferred rather than open — the position is above, under *A source can
drive the puppet and nothing else*.
