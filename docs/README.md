# Documentation index

Every document in the project is listed here. If it is not on this list, it does not exist —
see [documentation.md](documentation.md).

**Status: nothing is built yet.** One requirements pass is complete across every area listed
below, with the planning concern sweep folded in and the research findings filed. No design, no
plan, no code.

What is settled is in [decisions.md](decisions.md), which is the list to read rather than one
restated here — it supersedes its own entries in place, so a summary elsewhere goes stale.
[invariants.md](invariants.md) carries what must never break, with its one-line half in
`CLAUDE.md`.

**A session picking this up starts at `docs/wip/open-questions.md`**, which holds what is still
undecided. The largest remaining `[open]` is how Live delivers transparent frames into OBS on
Linux, which a spike settles before Live is designed.

## Start here

| Doc | Answers |
|---|---|
| [overview.md](overview.md) | What is this project, and what is it deliberately not? |
| [vocabulary.md](vocabulary.md) | What do the project's terms mean? Load-bearing — read before anything else |
| [invariants.md](invariants.md) | What must never be broken, and what a violation looks like |
| [documentation.md](documentation.md) | Where documents live, how they are tagged, how they stay true |

## Requirements

| Doc | Answers |
|---|---|
| [requirements/](requirements/) | What the system must do, per area. Declarative |
| [requirements/puppet.md](requirements/puppet.md) | What is a puppet, and what must its file guarantee? |
| [requirements/authoring.md](requirements/authoring.md) | What must the editor do, and what must it feel like? |
| [requirements/performing.md](requirements/performing.md) | What must the player do, from tracking input to streaming output? |
| [requirements/platform.md](requirements/platform.md) | What must it run on, and to what frame budget? |

## Design

<!-- One line per design document, by the reader question it answers. -->

| Doc | Answers |
|---|---|
| [format.md](format.md) | What is in a puppet file, how is it encoded, and what must a reader do? |
| [evaluation.md](evaluation.md) | In what order is a puppet evaluated, and what composes with what? |
| [scene.md](scene.md) | What does Live store about a sitting, and what does it deliberately not? |
| [runtime.md](runtime.md) | What does the runtime own, where are its boundaries, and how is it measured? |

## Working on it

| Doc | Answers |
|---|---|
| [stack.md](stack.md) | What is it built on, how do I run it locally, what runs on push? |
| [testing.md](testing.md) | How is each kind of thing tested? |
| [decisions.md](decisions.md) | Why is this odd-looking thing the way it is, and what was rejected? Short by design |
| [concerns/](concerns/) | The cross-cutting checklists, and how they grow |
| [research/](research/) | Dated findings, each against the decision it fed |
| [research/2026-11-26-linux-alpha-output.md](research/2026-11-26-linux-alpha-output.md) | How does the player get transparent frames into OBS on Linux? |
| [research/2026-11-26-inochi-file-measurements.md](research/2026-11-26-inochi-file-measurements.md) | What is actually inside an inochi2d model file? |
| [research/2026-11-26-deformation-models.md](research/2026-11-26-deformation-models.md) | How do other 2D systems model deformation, and is a skeleton needed? |
| [research/2026-11-26-puppet-corpus.md](research/2026-11-26-puppet-corpus.md) | How large is a real 2D vtuber puppet, measured across a corpus? |

`docs/wip/` is **empty between phases**, which is the normal state rather than a missing file.
A `plan.md` exists only while a phase is in flight and a `<feature>.md` only on its own branch
([documentation.md](documentation.md)).

## Conventions

- **Documents are an overview, not a substitute for the code.** They carry the rules, the shape
  of a subsystem, and why it is that way; anything a reader would get faster from the source is
  left to the source, with a pointer.
- **Vocabulary is strict.** [vocabulary.md](vocabulary.md) governs code, comments, documents and
  conversation.
- **Status tags** — `[decided]` / `[proposed]` / `[open]`. `[proposed]` means *in force*, not
  pending. Legend in [documentation.md](documentation.md).
- **CLAUDE.md** at the repository root is the agent entrypoint: how to run things, the
  vocabulary summary, and the invariants in one-line form. Never a second index.
