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
macOS, where nobody on the project has hardware to reproduce on.

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
