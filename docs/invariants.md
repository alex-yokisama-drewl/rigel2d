# Invariants

What must never be broken, and what a violation looks like.

This document and the invariant list in `CLAUDE.md` are **one list in two halves and must never
disagree**: CLAUDE.md carries the rule and its reason in one line; this document carries the
failure mode, numbered to match. `scripts/docs-check.sh` checks that the numbers correspond.

A violation found in review outranks every other finding.

## Format

Each entry:

```markdown
## I<n>. <The rule, stated as what must always be true>

**Why:** <what the rule protects. A rule without one gets "fixed".>
**Violation looks like:** <what you would observe. Concrete.>
**Checked by:** <a test, a script, or "judgment" — and where it lives.>
```

An invariant with no check is a hope. Where one cannot be automated, say so explicitly rather
than leaving the line blank.

## Invariants

<!--
Added during tech-design and as subsystems are built. Numbering is stable: a retired invariant
keeps its number and is marked retired rather than renumbered, because CLAUDE.md and review
reports reference these numbers.
-->

## I1. Rigel2D never executes anything a puppet, scene or source supplies

**Why:** puppets are traded between strangers — a rigger delivers to a streamer, the community
shares files. Opening one must not be able to harm the machine, and for software with no server
and no accounts that single sentence is most of the threat model.
**Violation looks like:** a field from a file or the port evaluated as a script, a command or a
format string; a path or URL inside a file resolved outside the container it arrived in; any
dynamic-load call reachable from a parsing path.
**Checked by:** judgment — review, plus a lint over the parsing paths. This one cannot be a test,
because it is a negative over all code rather than a behaviour to assert.

**Scope, and the residual risk.** This binds *Rigel2D's own code*. It is not a promise about
what runs on the machine once plugins exist: a plugin can store code in a puppet's extension
data and execute it itself, and nothing in the format prevents that. That risk is accepted as
part of allowing plugins (`decisions.md`), and it is the reason installation must be explicit
and visible — see [I6](#i6-a-plugin-never-arrives-inside-a-puppet-a-scene-or-a-source). It is
not a reason to weaken the rule for our own code, which is where the rule does its work.

## I2. Nothing but rendered puppet content ever reaches the render surface

**Why:** the render surface is what the stream shows. An error, a dialog, a tooltip, a
notification or debug output appearing there is seen by an audience, and it is the worst failure
Live can produce — worse than whatever caused it.
**Violation looks like:** any element other than the render target in the render surface's
structure, in any state, including every failure path.
**Checked by:** test — a structural assertion over the render surface in each error state.
`docs/concerns/ux.md`.

## I3. Every reference in a stored puppet or scene is by stable identifier

Never by name, never by index, never by file position.

**Why:** name- and index-based references break on exactly the operations this project exists to
make easy — rename, reorder, merge, re-import — and identity cannot be retrofitted into a format
already in the wild.
**Violation looks like:** a stored reference that resolves by position or by a user-visible name;
a rename or a reorder that breaks a binding, a mapping or a scene's reference to a puppet.
**Checked by:** test — rename and reorder a fixture, save, reload, assert every reference still
resolves; plus the validator rejecting name-based and index-based references.
`docs/concerns/data.md`.

**In memory this does not apply, and must not.** Resolving identifiers down to handles or
indices once at load is required, because nothing in the frame loop may resolve by name
(`docs/concerns/performance.md`). This invariant is about what is written to disk.

**What it binds is references to things the file contains.** Two stored references point outward
instead, and neither is an exception to the rule so much as outside what the rule is about.
Both are deliberate and both are specified elsewhere:

| Reference | Why a name or a path, and where it is settled |
|---|---|
| A channel mapping names a **channel** | A channel is supplied by a source the puppet has never met, on a machine the rigger has never seen. A shared name is the only thing the two ends can agree on in advance, which is why the channel vocabulary is Rigel2D's own — `docs/format.md`, `docs/requirements/performing.md` |
| A scene item names a **puppet file** by absolute path, *and* by that puppet's identifier | The path finds the file; the identifier is the reference and is what survives the puppet being edited. The path is a locator, not a reference — `docs/scene.md` |

A reference to anything in the pieces list of
[What a puppet is](requirements/puppet.md#what-a-puppet-is-proposed) has no such excuse and is
covered without exception — a mask, being a part-to-part reference, included.

## I4. A load that fails never replaces what is already displayed

**Why:** Live is on a stream. Replacing a good puppet with a partial one, a placeholder or an
error texture is visible to an audience.
**Violation looks like:** anything other than the previous content rendering after a failed load;
the rendered output changing when a hostile fixture is loaded alongside a good puppet.
**Checked by:** test — load a hostile fixture while a good puppet renders and assert the rendered
buffer is unchanged. `docs/concerns/security.md`.

**What this binds.** Loading an *additional* puppet is additive and cannot displace anything, so
it is outside this rule. What it binds is the replace path: reloading a puppet that changed on
disk, and swapping one in a scene.

## I5. No GPU resource creation or pipeline compilation happens in the frame loop

**Why:** creating a texture or compiling a shader mid-frame is a visible hitch, and on some
drivers a stall of hundreds of milliseconds. Steady state means every resource the puppet can
need already exists before the first presented frame.
**Violation looks like:** a non-zero resource-creation or pipeline-preparation counter after
frame one, for any blend, mask or composite combination the puppet uses.
**Checked by:** test — *no first-use work after the first presented frame*,
`docs/concerns/performance.md`.

**CPU allocation in the frame loop is a budget, not an invariant**, and it stays one now for a
better reason than the original. The original was that stating it absolutely would be a stack
decision wearing an invariant's clothes — natural without a garbage collector, contorted and
hard to verify with one. The stack removed that objection: zero is natural in Rust and a
counting global allocator makes the count exact (`docs/stack.md`).

It remains a budget because **this invariant's violation is a cliff and an allocation is a
slope.** A mid-frame shader compile stalls for hundreds of milliseconds on some drivers, and one
occurrence ruins a frame. An allocation in a language with no collector costs microseconds, is
bad only in aggregate, and has no pause behind it. Invariants are for cliffs; budgets are for
slopes, and the counter gate catches a regression either way. What promotion would have bought
is agent visibility in `CLAUDE.md` and review severity, at the price of a rule admitting no
justified exception.

## I6. A plugin never arrives inside a puppet, a scene or a source

Installing a plugin is always an explicit act by the user, through an installation process, out
of band from file sharing.

**Why:** puppets are traded between strangers; installed software is not. That distinction is
what makes [I1](#i1-rigel2d-never-executes-anything-a-puppet-scene-or-source-supplies) worth
stating at all. A file able to carry a plugin collapses it, and most of the threat model with
it.
**Violation looks like:** any load path that can install, register or load code from file content
or port input; opening a puppet causing a plugin to appear.
**Checked by:** judgment and review now; a script once a plugin loader exists, asserting its only
input is the installation directory.

**A puppet may declare, never install.** It may say that it carries data for a named plugin at a
named version, so the user can be told what is missing and what would read it. Declaring is not
installing, and the declaration is inert data like any other.

## I7. A failed save never damages the previous file

**Why:** for an authoring tool, losing the puppet is worse than a bad frame. A partially written
file is indistinguishable from a corrupt one at the next load, and the loss is discovered later,
by someone who cannot undo it.
**Violation looks like:** a write to a puppet, scene or autosave path that does not go through
the atomic-write function; a file truncated or half-written after a failure mid-save.
**Checked by:** script — every write goes through one function, failing on any write outside it;
plus a test injecting failure at each step and asserting the prior file is intact.
`docs/concerns/data.md`.
