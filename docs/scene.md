# The scene file

What Live stores about a sitting, and what it deliberately does not. The requirements are in
[requirements/performing.md](requirements/performing.md#what-a-scene-is-and-what-it-is-not); this
is the shape.

A scene is thin on purpose, and most of this document is about keeping it that way.

## Shape **[proposed]**

A single JSON file: a background, the items in the scene, and where each sits. Items are puppets
and images. Nothing else.

Each item carries an absolute path, the referenced puppet's own identifier, and a placement. The
identifier is what survives the puppet being edited; the path is what finds it. Both are needed
and neither is sufficient —
[performing.md](requirements/performing.md#a-scene-survives-being-interrupted) carries why the
path is absolute rather than relative, and what that costs.

**An image item is a puppet with one part, assembled in memory on load.** It is never written to
disk as a puppet and the user never sees one. This keeps Live with exactly one thing it knows
how to render and one validation path, which is worth more than the small cost of the wrapper.

**Nothing about the machine is in the file.** The output device, window placement and the input
port are application configuration, kept elsewhere. That is what makes *a scene never enables
off-machine listening* true by construction rather than by a check someone can forget.

## Writes are incremental and atomic **[proposed]**

A scene is written as it changes, not on exit, through the same atomic path as a puppet
([I7](invariants.md)). Live is already a writer of puppet files for channel mappings, so there
is one write path in the application and not two.

Relaunching after a crash restores the scene and resumes with no setup. This is the whole reason
the file is written eagerly, and it is why a scene write must be cheap enough to do often — a
scene is small for this reason as well as for portability.

## A dormant item **[proposed]**

An item whose puppet is missing, or whose channel mapping resolves to nothing, loads as
**dormant**: present in the scene, flagged in the control surface, drawing nothing, and
surviving the next save because the user may yet restore what is missing.

The rest of the scene renders. Nothing about this is a modal, a prompt or a blocking question
during load — by the time anyone could answer one, the scene is already up, which is the point.
The mechanism by which a dormant item is pointed at a moved file is `[open]` in
[performing.md](requirements/performing.md#a-scene-survives-being-interrupted); what is settled
is that it happens in the control surface, at the user's pace, while the scene renders.

## Where scenes live **[proposed]**

In a directory Live manages, reached through Live's interface rather than a file dialog. The
file is still plain, still validated, still shareable by hand — and a scene arriving from
elsewhere is untrusted input like any other file, validated on the same terms as a puppet before
anything it names is opened.

## Versioning **[proposed]**

The same two-number scheme as [the puppet file](format.md#the-version-is-readable-on-its-own),
for the same reasons, and the same rule about preserving marked extension data through a save.
A scene is a file strangers share, so it gets the identical treatment; nothing here is relaxed
because the file is small.
