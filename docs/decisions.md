# Decision log

Why an odd-looking thing is the way it is, and what was rejected. **Short by design.**

## What belongs here **[decided]**

One test, and it governs:

> Would a newcomer take this for a mistake, and break something by "fixing" it?

If no, it does not belong here, however much discussion it took. In particular, these do **not**
belong:

- obvious decisions, and decisions with no live alternative;
- arbitrary choices where either option would have been fine;
- documentation and process decisions;
- deferrals — an `[open]` tag already records those;
- anything the rule's own document explains adequately.

A phase can end with nothing added to this log. That is the normal case, not an omission.

## Lifecycle **[decided]**

- Entries are **not edited to stay current.** A reversal is a new entry that supersedes the old
  one; the old one stays, because the reasoning that produced it is what a reader needs in order
  not to repeat it.
- Entries **are pruned** when they stop meeting the admission test — usually because the thing
  they protect no longer exists.
- An entry is written **the moment a decision is recognised as meeting the test**, not at the
  end of a feature. A decision recognised during implementation moves here immediately.

## Format

```markdown
## <YYYY-MM-DD> — <the decision, in a few words>

**Decided:** <what was decided.>
**Because:** <the reasoning that would otherwise be lost.>
**Rejected:** <each alternative, with the specific reason it lost. The alternative a
reader would reach for next belongs here above all others.>
**Consequences:** <what this makes harder, where the cost shows up.>
```

## Entries

## 2026-11-27 — One physics model, and no rigid-pendulum switch

**Decided:** a physics node stores one spring pendulum's constants — a rest length, a gravity
multiplier, a resonant frequency, and separate angle and length damping — plus an optional
parameter output whose every axis names the simulated quantity driving it and the gain onto it.
No model-type switch, no map-mode enum, no local-only flag. The puppet carries one gravity
magnitude that each node's multiplier scales. The evidence is in
[research/2026-11-27-physics-parameterisation.md](research/2026-11-27-physics-parameterisation.md).
**Because:** inochi2d's constants are physical quantities that map one-to-one onto a single node,
and this format's node is one anchor and one bob. Cubism's are dimensionless factors tied to a
hard-coded 30 fps reference, meaningful only inside a chain, with no stiffness knob at all — and
a chain here is nesting, so a per-chain parameterisation has nothing to attach to.
**Rejected:** inochi2d's `model_type`, because its rigid model ignores `frequency` and
`length_damping`, and a control that silently disables two others is the clearest beginner trap
in either incumbent's field set; a high frequency approximates the rigid case. Its `map_mode`, an
enum over the four orderings of angle, length, x and y, because naming the source per axis covers
all four and covers a one-dimensional parameter without a special case. Its `local_only`, which
tree placement already determines here. And Cubism's parameterisation entire.
**Consequences:** both dampings are stored although a hair rig rarely wants stretch, because
removing a stored field later is a format break while hiding one in the editor is not — so Studio
is expected to show fewer knobs than the file holds, and that gap will read as an oversight.
Upper bounds on the constants are unset: sign and finiteness are all the validator enforces until
an integrator stability sweep measures them, which is due with the evaluation pass.

## 2026-11-27 — The puppet file has no separate authoring payload

**Decided:** everything a puppet stores lives in one structural entry. Data only Studio acts on —
import provenance, the mirror axes, axis polarity — sits beside the rest, and the player ignores
what it cannot use. This withdraws the earlier rule that the format declares each kind runtime or
authoring-only, and removes the `authoring.json` entry.
**Because:** the authoring-only set is an import identifier and a content hash per part, plus two
axes — tens of kilobytes against a stress workload budgeted at hundreds of megabytes of resident
texture. The split was specified before anyone had measured the thing being split.
**Rejected:** a separate entry the player skips, which is the obvious design and was this
project's own position until today. It saves the player a rounding error and buys a category of
complexity in exchange: data that is recognised but deliberately unread, a byte-for-byte carry
path through every save Live makes, a validator that answers differently in Studio and in the
player, and a second place anyone reading the format has to learn about.
**Consequences:** the player loads fields it will never use, which looks like an oversight and is
not. Anything genuinely worth skipping later has to earn its own entry on measured size rather
than on category. The archive itself is untouched — binary payloads addressable without parsing
the whole file was always the stronger half of that decision and stands without this one.

## 2026-11-27 — Physics is a node that moves itself, and contains only physics

**Decided:** physics is a fourth node kind. It anchors in the tree with its own local offset,
reads its own node's accumulated transform from the previous frame, displaces that node's
transform by the simulation result, and may drive one parameter — optionally, because a physics
node is equally a link in a chain. It may contain physics nodes and nothing else. **What each
kind may contain is declared per kind by the format**, rather than enforced by a rule that names
physics.
**Because:** a pendulum's pivot is rarely its parent's origin, so physics needs a parent and a
local offset, which is exactly what a node is. Keeping it outside the tree means storing an
anchor identifier and a local transform and resolving them against the parent chain —
re-implementing a node in order to preserve the sentence that a part is the only leaf kind.
Anchoring in the tree also asks nothing of a rigger: a physics node placed where the hair
attaches works. Containing only physics is what makes a pendulum chain an ordinary subtree while
keeping artwork out of reach of a cascade through one.
**Rejected:** the Cubism model, in which physics is a transfer function from named input
parameters to output parameters with no position in the tree. It is strictly forward — no cycle,
no delay — and it was the recommendation this decision overrode. It lost on who pays: naming and
weighting input parameters is a documented fiddly corner of Cubism, and this project is for
beginners. Also rejected: physics containing any kind, which would let a parameter's deletion
cascade through a physics node into artwork; and physics as a leaf, which makes a chain cost a
parameter, a binding and a helper node per link.
**Consequences:** physics is permanently one frame behind, which is worst on the weak hardware
this project targets and will read as a bug to be fixed. A chain of any length is still one frame
behind rather than one frame per segment, because each node reads a previous frame that already
carried its parent's displacement. A node kind that draws nothing now exists in a draw-ordered
tree. And the reference from a physics node to the parameter it drives is **incidental, not
constitutive** — deleting that parameter leaves the node standing as a chain link.

## 2026-11-27 — We write the OBS plugin for Linux ourselves, rather than depending on one

**Decided:** On Linux the player delivers frames through a plugin of our own, built so that it
can be extended to the other platforms later. Windows keeps stock OBS Game Capture; macOS is
unmeasured and keeps the Syphon client on paper. Settled against measurement
([research/2026-11-27-transparency-spike.md](research/2026-11-27-transparency-spike.md)).
**Because:** alpha turned out to survive every route measured, so the risk was never the thing
the spike was commissioned to find. What does not survive is colour. Each host silently assumes
an encoding of the buffer it is handed, the two measured hosts assume *different* ones, and
neither reports a mismatch — so a renderer correct for one is wrong for the other by an amount
that reads as a plausible picture rather than a fault. Both were invisible to the eye and were
caught only by comparing against values written down in advance. A plugin of our own cannot stop
a conversion being necessary, but it makes the conversion ours and explicit against a format we
define, instead of a host inferring one from a buffer carrying no colour-space tag.
**Rejected:** depending on **`obs-vkcapture`**, which a reader will reach for first because it is
the only Linux route measured working end to end, with alpha bit-exact. It loses on ownership
rather than on function: the route carrying our output would belong to a third party, it is not
in Ubuntu's repositories so a beginner builds it from source, and the colour conversion stays
ours to maintain either way. Also rejected: **PipeWire with `obs-pwvideo`**, which the earlier
research named the technical-fit primary — its receiver is sound and its format table genuinely
carries alpha, but libfunnel does not build against the PipeWire the target distribution ships,
and nothing links producer to consumer without manual intervention. And: **writing our own for
every platform**, which still fails *uniformity is not worth an install step where none is
needed* — though less clearly than when that was written, because the Windows route is now known
to fail silently when the player and OBS land on different GPUs.
**Consequences:** a plugin is a compatibility surface against an API with no stability promise.
`OBS_DECLARE_MODULE()` bakes `LIBOBS_API_VER` in at compile time and libobs increments its major
version on breaking changes, so this is a rebuild and a re-release per OBS major — roughly
annual — before any of Windows signing or macOS notarisation, which arrive if it is ever
extended. Against that, the plugin is the *small* half of the work: the sender is per-platform
under every option, and most of a reference implementation's bulk is swapchain interception that
a player exporting its own texture does not need.


## 2026-11-26 — Studio has no operation that deletes a texture

**Decided:** A texture is not a thing the rigger deletes. It arrives by import and is collected
the moment no part references it, whatever edit caused that. One narrow exception exists — a
command over unreferenced textures only, so that artwork inherited in a received file can be
removed. This supersedes the confirmed cascade *from* a texture, which this project previously
specified in full and wrote acceptance criteria for.
**Because:** every route to a texture runs through a part, so there is no state in which a rigger
holds a texture with no part for it. An operation to delete one therefore only creates the problem
it then has to solve, and the cascade it needed was the only one that could reach a puppet's root
— a single-part puppet is a beginner's first import, and deleting its texture would have taken the
whole tree. Collection also makes an orphaned texture impossible to produce rather than something
the editor must be careful not to leave behind.
**Rejected:** the confirmed cascade from a texture, which is what this document's own earlier
position was and what a reader will assume was considered. It is correct and it is machinery
serving an operation nobody needs. Also rejected: refusing deletion when a texture is in use,
which leaves the rigger stuck with no route forward; and sweeping unreferenced textures at save
time, which cannot see a reference held inside extension data this build cannot parse, and so
would destroy data the format promises to carry untouched.
**Consequences:** the part-without-its-texture reference kind stays declared constitutive although
nothing can reach it as an operation any more, because the closure is computed from the
declaration and a re-import split still exercises it. Collection cannot see a reference held by an
extension, so a texture referenced by both a part and an extension is lost when that part goes;
this is disclosed rather than solved — an edit that would collect, in a puppet carrying anything
opaque, says so and the rigger confirms. And *refuse what would not validate* had to be
generalised from deletion to every edit, since deletion is no longer the only operation that can
reach an invalid result.

## 2026-11-26 — A mirror rewrites bindings and never writes a parameter

**Decided:** Mirroring a binding reflects that binding's own stored values and, where the
parameter's axis is bilateral, reverses which keypoint holds which value. The parameter is read —
for its axis polarity — and never modified. Pasting rigging likewise creates bindings, never
parameters.
**Because:** the alternative reading is the one everybody arrives at, and it is wrong in a way
that looks right. "Flip the rigged eye" sounds like reversing an axis; an axis belongs to a
parameter; parameters are shared by many nodes; so the operation appears to have to reach every
node bound to it, and the feature appears to conflict with the ordinary rig in which one parameter
drives many nodes. It does not, because what is stored per node is the binding. Three separate
readers of this design reached the wrong conclusion independently before it was written down,
which is the evidence that it needed to be.
**Rejected:** reversing the parameter's axis, which would silently re-mean every other binding on
it and every channel mapping Live wrote into it. Also rejected: treating "copy the rigging" as
copying parameters, which is what makes the feature look as though it conflicts with shared
parameters in the first place.
**Consequences:** axis polarity has to be stored per parameter axis from the first version of the
format, because without it a mirror must ask the rigger which axes to reverse on every use. A
parameter whose polarity is unknown — from a third-party tool — reverses nothing and is named in
the result rather than guessed at.


## 2026-11-26 — A puppet is one file, with no separate project or export

**Decided:** No editing project distinct from a runtime form of the same thing. One file, always re-editable. It
stores the cut-up parts plus import provenance; the layered source document stays outside it.
**Because:** measurement of two real inochi2d rigs found its project and export files carrying
byte-identical rig data but for one key, and the export was the larger file for one of the two — so the
separation was not buying a smaller or simpler runtime file. What it costs is concrete: a second
file to lose, a pair to keep in sync, and the unrecoverable "exported, then lost the project".
**Rejected:** the project-plus-export split, which is what inochi2d does and what a reader will
assume was the considered choice. Also rejected: one format with a strippable export, which
keeps the simple path simple but reintroduces a derived file that can be mistaken for a working
copy and overwritten.
**Consequences:** a delivered puppet is always re-editable by whoever receives it. A rigger
cannot hand over something that renders but cannot be taken apart — which is a deliberate
consequence of an open format, not an oversight, and riggers should be told so rather than
offered a protection the format cannot provide.

## 2026-11-26 — No plugin API, and no scripting inside a puppet

**Decided:** Neither application loads third-party code. Extensibility is provided by open
seams — a file format others can read and write, and an input protocol anything can speak.
**Because:** it keeps *nothing a puppet, scene or source supplies is ever executed* available as
an invariant. Puppets are traded between strangers, so that sentence is most of the threat model
for a tool with no server and no accounts. An API is also a permanent compatibility promise,
against a project whose stated trade is depth over breadth.
**Rejected:** a plugin API, which a reader will propose the first time someone asks for a
feature that does not fit. It loses on the invariant above: a sandbox for untrusted plugin code
is a large and ongoing security problem that the open-seams approach does not have at all,
because an external program producing a file or sending channels runs with its own permissions
and ours are not involved.
**Consequences:** some extensions are impossible rather than merely unsupported — anything
needing to run inside the editor's process, such as a custom node type or a custom deformer.
When that is asked for, the answer is to build it into the application or not at all.

## 2026-11-26 — The applications never open an outbound network connection

**Decided:** No telemetry, no update check, no crash reporting. Neither application opens an
outbound network connection, and that is asserted in CI rather than intended. This is about
connections *out*: the player's input port listens, and local transports such as the display
server and any output plumbing are not network connections and are not covered.
**Because:** the strongest privacy posture available, a threat model with one network surface
instead of three, and a genuine selling point for an audience that is wary of streaming
software phoning home. It also costs nothing to maintain, where each of the alternatives is a
service someone has to run.
**Rejected:** an opt-in update check, which is the one a reader will reach for and which has a
real argument behind it — without it, a security fix reaches only the people who come looking
for a new version. It lost because "no outbound connections, ever" is checkable and explicable
in one sentence, where "one outbound connection, under these conditions, honouring this
preference" is neither.
**Consequences:** users learn about releases from wherever the project publishes them, not from
the application. Crash diagnosis depends on what users report by hand — which bites hardest on
macOS, where nobody on the project has hardware to reproduce on. A bad or insecure release
cannot be recalled from machines that already have it, and nothing tells a user that a fix
exists, so a security fix reaches only those who go looking. This is a cost of the decision, and
the decision stands.

## 2026-11-26 — The interface uses the project's strict vocabulary

**Decided:** User-facing text uses the same terms as the code and the documentation —
*parameter*, *binding*, *keypoint*, *deformation*. Onboarding explains them where they first
appear rather than renaming them.
**Because:** one vocabulary means a tutorial, a bug report, a forum answer and the source all
use the same words. Two vocabularies split the community's shared language permanently, and the
project's whole premise is that beginners can get productive — which depends more on their
being able to follow other people's explanations than on the first hour being gentle.
**Rejected:** friendlier interface labels mapped to strict terms underneath. It is the obvious
accommodation for a beginner audience and it is why this entry exists: the cost is invisible at
the moment of choosing and permanent afterwards.
**Consequences:** the first hour is steeper, and the interface carries an explanatory burden
that friendlier labels would have avoided. A lint over user-facing strings enforces this.

## 2026-11-26 — One repository owns the authoring app, the renderer app and their shared core

**Decided:** The format specification, the runtime library, the authoring application and the
renderer application all live in this repository and are specified together, even though the
authoring application is built first.
**Because:** The tight coupling is each application against the format and runtime, not the two
applications against each other. The authoring app's viewport *is* the renderer; if its preview
is a separate code path from the renderer app's output, rigs look correct while being authored
and wrong when performed. Specifying the format with only an editor in view produces a format
that is pleasant to edit and awkward to evaluate at frame rate.
**Rejected:** Separate repositories per application, mirroring Inochi Creator and Inochi Session
— the obvious shape and the one a reader will reach for. It trades a cheap, reversible cost
(splitting a repository later with `git subtree split`) for an expensive, permanent one (format
and runtime divergence between two codebases with no single source of truth).
**Consequences:** Release cadence and distribution for the two applications must be separated at
the build-artifact level rather than falling out of the repository layout. Requirements work
covers both applications before either is built, which front-loads conversation.

## 2026-11-26 — The puppet format is our own, with no inochi2d interoperability

**Decided:** No reading or writing of inochi2d's `.inp`/`.inx` formats, in either direction. The
deformation model is designed on its own merits.
**Because:** Interoperability would constrain the deformation and parameter model to match
inochi2d's, which is the part of the design with the most room for improvement and the part this
project exists to reconsider.
**Rejected:** Import-only support, which would have supplied a corpus of real rigs for testing
and an on-ramp for existing riggers. Rejected because an importer is only writable once the
deformation model is settled, and until then it exerts quiet pressure to make that model match
the one being imported.
**Consequences:** No existing models to test against — test assets must be authored. No migration
path for anyone with an existing inochi2d rig, and no path for a rig authored here to be
performed in Inochi Session.


## 2026-11-26 — An older build opens a newer-but-additive puppet read-only

**Decided:** When a puppet's format version adds things the running build does not know, the
build opens it for rendering and inspection but refuses to save it. The format is explicitly
unstable until the release tagged 1.0; the promise begins there.
**Because:** preserving unknown fields across a save is a promise about data the build cannot
validate, cannot render, and cannot show the user. A build that round-trips what it does not
understand will eventually drop or corrupt some of it, and the failure is silent and arrives on
someone else's machine — the rigger who opened a collaborator's file in last month's build finds
out when the work is gone. Read-only makes the limitation visible at the moment it applies, and
it is checkable: the save path either refuses on a version it does not know or it does not.
**Rejected:** opening read-write and preserving unknown fields on save, which is what this
document's own requirements proposed until today and is what a reader will reach for — it is
friendlier, and it is what most formats with a version field do. It lost on the silent-loss
failure mode above, and because the friendliness is worth least to this project's audience:
puppets move between a rigger and a streamer as finished deliverables far more often than two
people edit one file on different builds.
**Consequences:** a collaborator on an older build cannot contribute edits until they update,
and "update to open this" becomes a message the project has to write well. The proposed data
checklist entry about unknown fields surviving a load-then-save round trip narrows accordingly:
it applies within a version range the build understands, not across one it does not. A build
must also be able to render a format it cannot save, so rendering and saving need separate
capability checks rather than one "do we support this version" flag.

## 2026-11-26 — Unrecognised data is preserved, not a reason to stop (supersedes the entry above)

**Decided:** Reverses *"An older build opens a newer-but-additive puppet read-only"*, written
earlier the same day — **both halves of it**, the read-only rule and the claim that the promise
begins at a release tagged 1.0. The hard promise does not start at a version number at all; it
starts when the project is shared with a community, and the owner says when. A build opens a file carrying data it does not understand, stays fully
editable, and writes that data back untouched on save. It warns on open, naming what is missing,
and lets the user work. The rule is wider than format versioning: it covers any marked extension
data, from any writer.
**Because:** read-only protects the unknown data perfectly and stops the work, and stopping the
work is the more common harm. The realistic case is someone needing a small edit to a file that
carries something they do not have; sending them away to install something is the wrong default
when the file will almost certainly survive the edit. Preserving someone else's work is the
stronger argument, and the honest handling of the residual risk is a warning rather than a
refusal — the user is told what they are carrying and that those features may not behave, and
then trusted.
**Rejected:** read-only on unrecognised data, for the reason above — the superseded entry
carries its own reasoning and is left in place. Also rejected: opening silently and dropping
what is not understood, which is the easy implementation and the one that loses work invisibly,
on someone else's machine, long after the save that caused it.
**Consequences:** the format needs a marker on all extension data, and the validator must
require it — without that, a typo and an extension are indistinguishable and neither can be
rejected. Preserved data must take part in canonical serialisation, or carrying it makes files
churn. A build must be able to render and save a structure it has no code for, which means the
loader keeps an untyped representation alongside the typed one rather than parsing into typed
structures and discarding the source. **This entry defines extension data without deciding
whether the applications ever load third-party code** — that is still governed by *"No plugin
API, and no scripting inside a puppet"* above, which the extension-data mechanism does not by
itself disturb: data is never executed either way.

## 2026-11-26 — Plugins are intended; the API is deferred (supersedes "No plugin API")

**Decided:** Reverses the *"No plugin API"* half of the earlier entry and leaves the
*"no scripting inside a puppet"* half untouched and permanent. Studio and Live will load
third-party code eventually. The API surface is not designed now, and nothing is built for it
now beyond what would be impossible to retrofit.
**Because:** the earlier entry argued the case on security, and that argument does not survive
contact with the distinction it missed: the threat is code arriving *in a traded file*, not code
a user deliberately installs. Puppets are shared between strangers; plugins are not. The
argument that does survive is the one the old entry made almost in passing — an API is a
permanent compatibility promise over internals — and that is a reason to defer, not to refuse.
This project's deformation model is the part it most wants freedom to reconsider, and it is
exactly what a premature API would freeze. Tools of this kind are better for having plugins;
Blender, Krita and OBS all added theirs after the internals settled.
**Rejected:** holding the line at "build it in or not at all", which is defensible for a focused
tool and is what the superseded entry chose — it loses because it answers "can we afford the
API?" with "never" when the real answer is "not yet". Also rejected: designing the API now so it
is ready. An API with no plugins written against it is a guess, and the guess becomes a promise
the moment anything ships.
**Consequences:** the groundwork that *is* in scope now is narrow and listed in
`docs/requirements/authoring.md` — marked extension data, stable identifiers, an enumerable
operation set, and the rule that a plugin never arrives inside a file. The standing temptation
is abstraction layers with one implementation behind them, written "for the plugins"; that costs
more than the retrofit it avoids. **Two existing claims need revisiting before plugins ship, not
before then:** *"the applications never open an outbound network connection, asserted in CI"*
is an assertion over our own code and a plugin is not our code, so either the claim narrows to
the applications themselves or plugins are constrained from networking; and the security
checklist's *"the player requests no screen-capture capability"* has the same shape.

## 2026-11-26 — A puppet is a zip archive, not a single JSON document

**Decided:** The container is an archive: a tiny version entry, the structure as JSON, one image
entry per texture, and authoring data in its own entry.
**Because:** two of the format requirements are structural rather than aesthetic — binary
payloads must be addressable without parsing the whole file, and the player must be able to skip
authoring data rather than load it. An archive delivers both by construction.
**Rejected:** plain JSON with textures base64-encoded inside, which `requirements/puppet.md`
explicitly allows and which a reader will reach for because it is simpler and more obviously
inspectable. It loses three ways: it inflates the payload by about a third, a file with a
hundred megabytes of base64 in it cannot be opened in the text editor that was the whole reason
for choosing JSON, and reaching any part of it means parsing all of it. Also rejected: JSON
beside a directory of images, which is the same design minus *One file*.
**Consequences:** every zip writer default is non-canonical, so canonical serialisation now has
a second half that is easy to get wrong and invisible when you do — `format.md` lists what must
be pinned. A decompression-bomb check is also required, because an entry's declared size is
attacker-controlled.

## 2026-11-26 — Entity identifiers are integers, not UUIDs

**Decided:** An unsigned 64-bit integer per entity, unique within the puppet, allocated from a
stored high-water mark that never decreases.
**Because:** the governing requirement is that tools which are not ours can write a valid
puppet. Emitting `1, 2, 3` requires nothing of a third-party tool; emitting UUIDs requires a
library in every one of them. Integers also diff legibly, and legible diffs are most of the case
for an open format.
**Rejected:** UUIDs, which are the obvious choice and have a real advantage — two puppets merge
without renumbering. That advantage is worth little here, because merging another puppet copies
what it takes and assigns fresh identifiers on the way in under either scheme.
**Consequences:** two tools editing one puppet without seeing each other's work can allocate
colliding identifiers. Nothing exercises this today — there is no concurrent editing and no
merge of divergent copies — but it is a real edge the UUID scheme would not have had, and a
writer must take the high-water mark from the file rather than from a count of what it found.

## 2026-11-26 — Transform bindings sum componentwise and are never composed as matrices

**Decided:** Where several bindings drive one node's transform, translation, rotation angle and
scale accumulate separately and the matrix is built once from the accumulated values.
**Because:** matrix multiplication is order-dependent, so composing transforms as matrices makes
the result depend on binding order. Head roll, pitch and yaw all touching one node is ordinary
rigging, not an edge case, and what makes it workable in practice is that the rigger adjusts
values until it looks right. That loop needs the result to be repeatable and predictable; an
order-dependent one destroys it, and it is also how the editor's viewport comes to disagree with
the player's output.
**Rejected:** composing transforms as matrices, which is what a newcomer will reach for because
it is what every graphics text does and because it is more "correct" in the sense of being a
proper composition of rigid transforms. It is the right answer for a scene graph, where one
transform has one parent, and the wrong answer here, where many bindings contribute to one
transform.
**Consequences:** a composed rotation beyond the small angles rigging actually uses will not
match what matrix composition would have produced. That is accepted; it is also what the
incumbent tools do, so it will not surprise a rigger.

## 2026-11-26 — One parameter has one driver, enforced by the validator

**Decided:** Each parameter declares exactly one base driver — a channel mapping, a physics
output, or nothing. Two is a validation failure. Animations are the one exception and override
temporarily, blending in and out.
**Because:** the alternative is a precedence rule, and a precedence rule resolves the conflict by
silently discarding the loser. A rigger who has mapped a channel onto a parameter that physics
also drives has made a mistake, and the useful moment to tell them is when the file is
validated, not invisibly at frame time.
**Rejected:** a documented precedence order, which is the obvious fix and looks more permissive.
It is more permissive about exactly the thing that should not be permitted, and it turns a
reportable authoring error into a silent behaviour nobody can see in the file.
**Consequences:** a puppet that a third-party tool generates with two drivers on a parameter is
refused rather than resolved, so the error message has to be good enough to act on. Anyone who
genuinely wants a blend of two drivers has to express it as physics reading the other value,
which is the one backward read the evaluation pass allows.

## 2026-11-26 — Rust, wgpu and egui

**Decided:** Rust as the language, wgpu as the GPU abstraction, egui through `egui-wgpu` as the
UI toolkit, on winit. Evidence in `research/2026-11-26-stack.md`; the rows and their reasons are
in `stack.md`.
**Because:** three constraints did the deciding, and each eliminated candidates outright. I5
needs an abstraction that creates pipelines when asked — wgpu calls `vkCreateGraphicsPipelines`
inside `create_render_pipeline`, and bgfx calls it inside `submit()` on a cache miss, with no way
to force it earlier. The 8 ms 99th-percentile budget needs no collector in the frame loop. And
the project asserts on accessible names and roles as well as geometry, which among the Rust
toolkits only egui's AccessKit-based harness supplies headlessly.
**Rejected:** bgfx, structurally, on I5. Tauri and Electron, on three independent failures — the
implementation decides when pipelines compile, a JavaScript frame loop cannot hold a
zero-allocation budget, and Linux transparency is a recurring compositor bug surface on a tier
that blocks a release. Dear ImGui in any language, which has no accessibility concept at all and
would make that gap permanent. D, which is what the incumbent uses and is therefore the one a
reader will ask about: its collector against this budget, and in practice D here means Dear
ImGui. **Qt was not rejected on capability** — it has the strongest answer to structural UI
assertions of any native option and the most mature transparency story — but loses on MoltenVK
against a no-Mac-to-test-on constraint and on being a far heavier dependency for a project with
no server, no networking and no localisation. **Slint is the named fallback, not a rejection:**
its testing API is better than egui's, and it loses on wgpu interop pinned to one major version
behind unstable feature flags, on carrying a second renderer in the editor, and on GPLv3
constraining the deferred plugin story.
**Consequences:** nothing here was observed — the verdict rests on documented behaviour and on
reading library source, and transparency in particular has not been seen working anywhere. The
exposures and the spikes that settle them are in `stack.md`, and the transparency one is the
same spike `requirements/performing.md` already requires for the OBS alpha path.

## 2026-11-26 — The render surface is a window the UI toolkit does not own

**Decided:** Live's render surface is a bare winit window with a wgpu surface and no widget
toolkit linked into its draw path. The control surface, and all of Studio, are egui on a separate
window sharing the same wgpu device.
**Because:** it converts I2 from a rule someone must remember into a thing that cannot happen —
no error, dialog, tooltip or debug output can reach the render surface, because the toolkit has
no access to that window. It also keeps egui out of the player's frame budget: egui lays out
every frame, its own documentation puts that at 1–2 ms, and the player's solo CPU budget is 4 ms.
**Rejected:** one window with the render surface as a region inside the toolkit's layout. This is
the obvious shape, it is what most applications do, and it is what a newcomer will refactor
toward because two windows look like an accident. It loses both ways at once: I2 becomes a thing
enforced by care on every failure path, and the toolkit's per-frame layout lands inside the frame
loop that the budget binds.
**Consequences:** two windows to place, move and remember the geometry of, and a user who can
lose one behind the other. Studio's viewport is a paint callback into the shared device rather
than a window of its own, so the split costs nothing there.

## 2026-11-26 — Normalisation belongs to the source, the response curve to the puppet

**Decided:** A channel mapping stores a response curve, defined over the *normalised* channel.
Normalisation — compensating for this person, this camera, this tracker — is held per source in
Live's configuration and is never puppet data.
**Because:** one word, "range", covers two transformations that vary with different things.
Shaping varies with the puppet and is authored work a recipient wants delivered. Normalisation
varies with whose face is in front of the camera, and a specified range does not close that gap
— a real user's hardest squint reads 0.80 against a nominal maximum of 1, and natural head
movement reaches about two-thirds of a nominal angular range
(`research/2026-11-26-tracker-interchange.md`). Stored per puppet, that compensation encodes the
rigger's face at the rigger's desk and is wrong for everyone else; stored per source, the
recipient calibrates once and every puppet they are handed thereafter arrives correct.
**Rejected:** one stored input range per mapping, carrying both. This is what VTube Studio does,
it is the obvious design, and it is what a reader will reach for — and its users re-tuning a
purchased model is the observable consequence. Also rejected: a channel name and a parameter
with nothing else, which was this project's own position until today. It assumes a name pins the
numbers as well as the meaning; widely used trackers disagree on sign, zero point, unit and
range for a single quantity, and even where a vocabulary pinned a range, transports carrying it
rescaled it under the same names.
**Consequences:** the channel vocabulary has to pin unit, range, zero point, sign and semantic
class per channel, not just a name — a much larger specification than "a list of channel names"
implies, and it is now on the critical path rather than beside it. Live needs a calibration
affordance it would not otherwise have had. A user moving machines recalibrates, which is
correct and will still surprise someone. And the evaluation pass gains an order that has to be
stated rather than assumed: finite check, normalise, curve, clamp.
