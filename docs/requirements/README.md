# Requirements

What the system must do. Declarative throughout — behaviour, rules and constraints, never
implementation.

## Rules **[decided]**

- **One file per area**, named for the area. `<area>.md` holds the requirements;
  `<area>-acceptance.md` holds the criteria and the tests that check them, written by
  `acceptance-planner` from these requirements without sight of any implementation.
- **Every section carries a status tag** — `[decided]`, `[proposed]`, `[open]`.
- **Deferral is written down as `[open]`, never left as a gap.** "Per-datatype admin screens —
  `[open]`, specified per datatype at implementation" is a recorded decision. A gap is a thing
  an implementer silently fills with a guess.
- **Technical constraints that arrived with the requirements are recorded as constraints with
  their reasons**, not as derived design. A later agent needs to know whether a choice may be
  challenged; an inherited constraint with no reason gets "fixed".
- **Detail goes where a wrong guess is expensive to undo.** That a capability exists, who can
  reach it, what stays editable, what is audited — those invalidate the data model if wrong. The
  layout of a particular screen does not. Over-specification has a running cost: every detail is
  context someone pays to load and a claim implementation will contradict.

## Files

<!-- Listed in ../README.md's index like any other document. -->

| File | Answers |
|---|---|
| [puppet.md](puppet.md) | What is a puppet, and what must its file guarantee? |
| [authoring.md](authoring.md) | What must the editor do, and what must it feel like? |
| [performing.md](performing.md) | What must the player do, from tracking input to streaming output? |
| [platform.md](platform.md) | What must it run on, and how well must it run there? |

All are in draft. **No acceptance criteria exist yet** — no `<area>-acceptance.md` file has
been written. They are commissioned from `acceptance-planner` once the `[open]` sections that
would change them are resolved.
