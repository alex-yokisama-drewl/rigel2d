# The scene file

What Live stores about a sitting, and what it deliberately does not. The requirements are in
[requirements/performing.md](requirements/performing.md#what-a-scene-is-and-what-it-is-not-decided); this
is the shape.

A scene is thin on purpose, and most of this document is about keeping it that way.

## Shape **[proposed]**

A single JSON file: a background, the items in the scene, and where each sits. Items are puppets
and images. Nothing else.

Each item carries an absolute path, the referenced puppet's own identifier, and a placement. The
identifier is what survives the puppet being edited; the path is what finds it. Both are needed
and neither is sufficient —
[performing.md](requirements/performing.md#a-scene-survives-being-interrupted-proposed) carries why the
path is absolute rather than relative, and what that costs.

**An image item is a puppet with one part, assembled in memory on load.** It is never written to
disk as a puppet and the user never sees one. This keeps Live with exactly one thing it knows
how to render and one validation path, which is worth more than the small cost of the wrapper.

What a scene holds and what it deliberately excludes — including that nothing about the machine
is in it — is [performing.md](requirements/performing.md#what-a-scene-is-and-what-it-is-not-decided)'s,
not restated here.

## Writes are incremental and atomic **[proposed]**

A scene is written as it changes, through the same atomic path as a puppet
([I7](invariants.md)). Live is already a writer of puppet files for channel mappings, so there
is one write path in the application and not two.

The requirement this serves — resuming after a crash with no setup — is
[performing.md](requirements/performing.md#a-scene-survives-being-interrupted-proposed)'s. What
follows for the design is that a scene write happens often and must therefore be cheap, which
is a second reason the file is small.

## A dormant item **[proposed]**

[performing.md](requirements/performing.md#nothing-stops-the-puppet-rendering-decided) says when
an item becomes dormant and why nothing about it may block a load. What the file carries is the
design half: **a dormant item is stored exactly as a live one.** Nothing records that it failed
to resolve, because dormancy is a property of this run and not of the scene — the file the user
restores tomorrow must be the file they had yesterday.

## Versioning **[proposed]**

The same two-number scheme as [the puppet file](format.md#the-version-is-readable-on-its-own-proposed),
for the same reasons, and the same rule about preserving marked extension data through a save.
A scene is a file strangers share, so it gets the identical treatment; nothing here is relaxed
because the file is small.
