# Requirements — the player

What Live must do. What a puppet *is* belongs to [puppet.md](puppet.md).

The player renders a puppet driven by live input and delivers the result to streaming software.
Everything here is in service of one sitting: start it, have it already set up, stream.

## Transparent output is native and on by default **[decided]**

The player outputs with an alpha channel on every supported platform, and transparency is on by
default rather than configured.

**Because:** the alternative in practice is a chroma key filter in OBS, which constrains what
colours a puppet may contain and degrades edges — a limitation about output plumbing landing on
the artwork, which is the wrong place for it. Inochi Session's transparency is a Windows-only
plugin with no Linux equivalent, which makes this a gap rather than a preference.

### Which platforms need an OBS plugin **[open]**

**The choice among the options below is deferred to the spike, deliberately.** Alpha has not
been observed working end to end on any candidate path, so picking one now would be picking on
the strength of source reading alone. The spike described below is the
trigger: it decides this, and nothing should be built that assumes an answer before it runs.
What is already settled, and does not wait for the spike, is that output sits behind a narrow
interface that can host more than one backend.

What each platform offers **on paper** — every row below is drawn from reading source and
documentation, and none of it has been observed working
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
| Depend on a third-party Linux plugin | No work for us. The two differ in risk. The research recommends the `obs-pwvideo` route as primary on technical fit, while noting it is a single-maintainer, best-effort project and pulls in `libfunnel`, which is single-author, has no tagged release and declares its API unstable. It recommends `obs-vkcapture` as the fallback, far more widely deployed and actively released, at the cost of a launch wrapper and a swapchain that actually presents |
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
- **A missing output device falls back rather than failing.** Output configuration is application
  configuration rather than scene content, so this is not a scene-load concern — but a device that
  has gone away since last launch must not stop Live starting or stop the puppet rendering.
- **One broken item never blocks a scene.** A missing puppet, a channel mapping that resolves to nothing, or an
  absent channel loads as a dormant entry, flagged in the control surface; the rest of the scene renders. A dormant
  entry survives the next save, because the user may yet restore what is missing.
- **Recovery is offered, never demanded.** Pointing a dormant entry at a moved file happens in the control surface,
  at the user's pace, while the scene renders. It is not a modal asked during load, and there is no "cancel the scene
  load" — by the time anyone could answer, the scene is already up, which is the point. A stream does not wait for a
  file dialog.

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

## The input protocol is ours, and the channel vocabulary with it **[decided]**

Rigel2D defines the protocol and the set of channel names it carries. OpenSeeFace is **one
source implementation behind it**, an adapter like any other, not the native protocol.

**Because:** channel mappings live in the puppet (below). A puppet's mapping is therefore only
worth storing if a channel name means the same thing whoever is sending it — otherwise handing
someone a rigged puppet hands them a mapping that is dead weight against their tracker, and the
main benefit of storing it disappears. A defined vocabulary is what makes a delivered puppet
work on arrival.

It also fits the extensibility story: anything that can open a socket can speak this protocol,
and adapting a tracker that speaks something else is writing a small program, not loading code
into ours.

**The transport is a tech-design question, not a requirement.** One consequence is worth
carrying into it: a transport a web page can reach needs origin checks, and one it cannot
removes that concern entirely rather than mitigating it.

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

## Channel mappings are puppet data, and Live writes them **[decided]**

The mapping from a channel to a parameter is stored **in the puppet**, not in the scene, and it
is **edited in Live** rather than in Studio.

**Why it is in the puppet:** mapping a tracker onto a rig is real work, done once. Storing it
with the puppet means a delivered puppet moves and reacts for whoever receives it, which is the
whole beginner story. Storing it in a scene would mean redoing it for every scene, and losing it
when a puppet is handed on.

**Why Live edits it:** mapping is tuned by watching yourself move, which only Live can show.
Live receives the tracking data and renders the result, so it is the only place the feedback
loop is closed.

**Live does not need to understand the whole puppet format to do this.** It loads what it
renders and what it maps, and carries everything else through untouched — the same rule that
lets any build open a file carrying data it does not recognise
([puppet.md](puppet.md#a-build-works-with-what-it-understands-and-preserves-the-rest)). The
mechanism was built for format versions and extensions; Live writing mappings is the same
problem and gets the same answer for free.

Consequences that follow:

- **Live writes puppet files, so [I7](../invariants.md#i7-a-failed-save-never-damages-the-previous-file) binds Live too.** Every write goes through the one atomic-write path. Live is not a read-only application and must not be built as though saving were Studio's problem.
- **It must be unmistakable what saving does.** Inochi Session's equivalent is confusing — "save to file" does not make clear *what* is written or *where*, and a user can reasonably think they are saving a scene when they are modifying a puppet. The interface names the puppet being changed, and what in it changes, before it changes.
- **Two mapping setups for one puppet is not supported**, and is expected to be rare. If it is wanted later, the answer is named mapping profiles in the puppet, or a plugin — not moving mappings back into the scene.

## What a scene is, and what it is not **[decided]**

A scene holds a background, the puppets and images in it, and where each one sits. That is all,
deliberately.

**Nothing about the machine is scene content.** The output device, window placement and the
input port are application configuration. Two reasons: a scene stays portable, so the file being
findable and shareable costs nothing; and *a scene never enables off-machine listening* becomes
true by construction rather than by a check that could be forgotten.

**Anything more elaborate than placement is a puppet.** An animated or interactive background is
a puppet with its own rig, and the scene merely places it. This is a workaround in form and the
right shape in substance: Live gets exactly one thing it knows how to render, and scene
complexity has somewhere to go that is already specified and already validated.

**Live accepts plain image files in a scene**, without the user having to make a puppet first.
Whether an image is internally a one-part puppet or its own kind of scene item is a tech-design
question and deliberately left there — it does not change what the user does.

### Where scenes live **[decided]**

Scenes are stored in a directory Live manages, and the user reaches them through Live's
interface rather than through a file dialog. They are not documents in the sense a puppet is
one: nobody is expected to curate a folder of them.

The file is still findable, still a plain validated file, and can still be shared by hand. That
is a property of being honest about storage, not an invitation — and a scene that arrives from
elsewhere is untrusted input like any other file.

## A scene survives being interrupted **[proposed]**

Scene changes are written as they happen, not only on exit, and written atomically — a crash or
a kill mid-stream cannot corrupt a scene. After a crash, relaunching restores the scene and
resumes with no setup.

A scene refers to a puppet by an **absolute path together with the puppet's own identifier**
([puppet.md](puppet.md#every-entity-has-a-stable-identity)), so that editing a puppet does not
break the scene that uses it. Absolute rather than relative, because a scene lives in a
directory Live manages while puppets live wherever the user keeps them, and a relative path
between those two is long and snaps the moment anything is reorganised.

The cost is that a moved or renamed puppet breaks the reference — the common case, far more
than a scene moving between machines. **Recovery is how that is handled, and the exact mechanism
is `[open]`.** What is settled is the shape: the scene still loads, the missing item is dormant,
and the control surface offers to locate it, skip it, or skip all like it. It cannot be a modal
asked during load, for the reason in
[Nothing stops the puppet rendering](#nothing-stops-the-puppet-rendering-decided) above.

## Editing a scene while streaming **[open]**

Whether a scene can be changed while Live is performing it, and whether there is a locked state
that prevents it, is undecided. Nothing may assume either answer.

Off-machine sources are **deferred rather than open** — the position is settled above, under
*A source can drive the puppet and nothing else*.

