# rigel2d

A free and open-source 2D vtubing toolkit. **The editor** builds a layered drawing into an
animated puppet; **the player** performs that puppet live from tracking input into streaming
software. Same ground as Inochi Creator and Inochi Session, aimed at beginners and smaller
streamers on weak hardware. Both applications and the runtime they share live in this
repository. The workspace builds and the push gates run; no production code is implemented yet.
Requirements cover every area, with the gaps that gate the next subsystem recorded in
`docs/wip/plan.md`; technical design has settled the format, the evaluation pass, the scene, the
runtime's boundaries and the stack. Phase 1 is in flight — **start at `docs/wip/plan.md`**.

The project is **Rigel2D**; the two applications are **Studio** and **Live**.

## Working here

**Route every piece of work through the `workflow` skill.** It orients and picks a lane —
requirements, design, phase planning, feature, small fix, research, or finishing a cycle. Do
not guess a lane, and do not skip to implementation because a request sounds simple.

| | |
|---|---|
| Documentation index | `docs/README.md` — if it is not listed there, it does not exist |
| Documentation rules | `docs/documentation.md` — governs where things go |
| Dispatch contract | `.claude/rules/dispatch.md` — read before the first dispatch of a session |
| Verification rules | `.claude/rules/testing.md` |
| Cross-cutting checklists | `docs/concerns/` |

**visual-checks: `automated`.** Whether automated visual verification is
attempted is project configuration, not an agent's judgment call.

## Who decides what

**Technical decisions are the agent's.** Data model, gates, dependencies, test strategy, when
to commit, when to merge, how quality is assured. Decide them, implement them, and say plainly
what was chosen and why. A technical choice is **announced, not submitted** — a decision
recorded with its reason is reversible, and one deferred to the owner is only a slower decision.

**Product decisions are the owner's**, and are worth interrupting for: how something should
feel to use, which trade-off to take, what is in scope, anything that changes what the software
is. The `requirements` and `tech-design` lanes are where that time is spent.

Still the owner's, inside technical work:

- **An invariant.** Changing one is never an agent's call.
- **A question only they can answer** — hardware, accounts, money, or a fact not on disk.
- **A human gate**, which exists for what a person must look at. It is not a way to get a
  technical choice approved, and a gate the owner can only answer by re-running what the agent
  already ran is a defect in the gate. See `.claude/rules/testing.md`.

**Do not open a question the code, the documents or a measurement can answer.** Go and find out.

## Running it

| | |
|---|---|
| Install | `cargo build --workspace`, with rustup installed; the compiler is the one `rust-toolchain.toml` pins. Debian-family Linux needs `curl build-essential` and nothing else yet — the walked steps are in `docs/stack.md` |
| Run locally | `cargo run -p rigel-studio`, `cargo run -p rigel-live` — both are shells that print and exit |
| Tests | `scripts/test-record.sh` (reads `.claude/test-cmd`) |
| Documentation check | `scripts/docs-check.sh` |

Full detail in `docs/stack.md`.

## Vocabulary

Summarised from `docs/vocabulary.md`, which governs. **Strict** — a term used in a sense that
document does not sanction is a defect, and it is how independently-written code becomes
incompatible.

| Term | Means |
|---|---|
| **Puppet** | A complete rig: node tree, textures, parameters, bindings, animations |
| **Node** | One entry in a puppet's tree. Part, mesh group, composite, physics are kinds. What each kind may contain is declared per kind. Never "object" or "layer" |
| **Part** | A node that draws a mesh with a texture. **References** a texture; never owns one |
| **Texture** | An image, an entity in its own right. Many parts may reference one |
| **Physics** | A node kind. Reads its own previous-frame transform, displaces that node's transform, may drive one parameter. Contains only physics |
| **Parameter** | A puppet's own named axis with keypoints. **Not** an input channel |
| **Binding** | Maps a parameter to a node transform or to per-vertex offsets. Edited in Studio |
| **Channel mapping** | Maps a channel to a parameter. Puppet data, edited in Live |
| **Track** | The part of an animation that drives one parameter. Never a "lane" |
| **Deformation** | Per-vertex mesh offsets. Not a transform, which moves a whole node |
| **Identifier** | The stable value every stored reference names an entity by. Never a name, an index or a file offset |
| **Handle** | What an identifier resolves to at load, for the frame loop. Never written to a file |
| **Extension data** | Marked, namespaced values the core format does not define. Preserved untouched, never executed |
| **Scene** | Live's saved document: a background, which puppets and images, and where each sits. Deliberately thin |
| **Source** | A feed of input over a port, supplying channels. A face tracker is one kind |
| **Channel** | One named value from a source. Bound *to* a parameter |
| **Runtime** | The library that evaluates and renders a puppet, embedded in both applications |
| **Rigel2D** | The project and toolkit as a whole, not either application |
| **Studio** / **Live** | The two applications — the one that builds puppets, the one that performs them |
| **The editor** / **the player** | Descriptive terms for Studio and Live. Still sanctioned |

Never say: *performance* (means two things here), *model*, *session*, *object*, *layer*,
*sprite*, *project*, *lane*. `docs/vocabulary.md` says what to say instead, and why.

## Invariants

One line each, numbered to match `docs/invariants.md`, which carries the failure modes. **These
two lists are one list in two halves and must never disagree** — `scripts/docs-check.sh`
verifies the numbering corresponds.

<!-- Format: In. <the rule> — <why, in a clause> -->

- **I1.** **Rigel2D never executes anything a puppet, scene or source supplies** — files are traded between strangers, so this is most of the threat model. Binds our code, not plugins.
- **I2.** **Nothing but rendered puppet content ever reaches the render surface** — it is what the stream shows, on every failure path.
- **I3.** **Every reference in a stored puppet or scene is by stable identifier** — never name, index or position. Binds references to what the file *contains*; a channel and a puppet file are named outward. In memory, resolved handles are required instead.
- **I4.** **A load that fails never replaces what is already displayed** — binds the replace path; loading an additional puppet is additive.
- **I5.** **No GPU resource creation or pipeline compilation in the frame loop** — a mid-frame texture or shader is a visible hitch. CPU allocation is a budget, not this.
- **I6.** **A plugin never arrives inside a puppet, a scene or a source** — installing is always explicit. A file may declare what it needs, never install it.
- **I7.** **A failed save never damages the previous file** — losing the puppet is worse than a bad frame.
