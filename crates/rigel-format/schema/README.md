# The committed JSON Schema, and where it stops

`docs/format.md`, *Validation*, requires the validator to be runnable against the specification
**without our code**: a JSON Schema for what a schema can express, plus the structural rules it
cannot. These files are the first half. This document is the boundary between them, written down
so that a third party who runs the schema knows exactly what they have and have not checked.

| File | Describes |
|---|---|
| `rigel2d.schema.json` | the `rigel2d.json` archive entry |
| `puppet.schema.json` | the `puppet.json` archive entry |

Both are JSON Schema draft 2020-12, and both describe **format version 1.1**. Neither describes
the archive around them — entry names, compression, ordering and the load limits are the
container's rules and are in `docs/format.md`.

## The boundary

A schema states what one value may be, and what members one object may carry. Everything in the
format that is a **relation between two values** is outside it, and that is most of the
validation list — because most of what makes a puppet invalid is a reference, a count or an
order rather than a shape.

Each rule the validator names, and which side of the line it falls on. The rule names are
`Rule`'s in `../src/validate.rs`, so a rule added there and not here is visible as a missing row.

| Rule | In the schema | Why |
|---|---|---|
| `identifier-zero-reserved` | yes | `entity_identifier` has a minimum of one; the reserved zero is admitted only where a *reference* may be empty |
| `axis-polarity-declared` | yes | the member is required, which is the whole rule: an absence is not the value `unknown` |
| `enumerated-value-known` | partly | every enumerated field carries its list. What a schema cannot express is the **version gate** — an unfamiliar value is legal in a file declaring a later minor, and a schema pinned to 1.1 has no way to say so |
| `unmarked-data` | yes | `additionalProperties: false`, and `unevaluatedProperties: false` on a node, whose members depend on its kind. The same gate applies: a later minor licenses the material |
| `extension-declares-version`, `extension-declares-name` | yes | both are required members of an `x` entry |
| `numbers-finite` | yes | every interpreted number is bounded at the single-precision limits, which is the rule stated rather than approximated: finiteness is judged at the precision evaluation uses, so a value finite as a double and infinite as a single is out of range here too |
| `blend-duration-non-negative` | yes | a minimum of zero |
| `physics-constant-within-range` | lower bounds | `length` and `frequency` above zero, `gravity` and the dampings at or above it. Upper bounds are `[open]` in `docs/format.md` pending the integrator-stability sweep, so neither the schema nor the validator states one |
| `identifiers-unique` | no | identifiers are unique across every kind of entity at once, which is a relation over the whole document |
| `identifier-within-high-water-mark` | no | compares each identifier against a value stored elsewhere in the file |
| `tree-acyclic` | no | the tree is a flat array with parent references, so acyclicity is a walk |
| `exactly-one-root` | no | counts the nodes with no parent and compares against what the puppet names |
| `root-is-a-kind-that-contains` | no | reads the root's `kind` through a reference |
| `containment` | no | compares a node's kind with its parent's, through a reference |
| `reference-resolves` | no | every stored reference is an integer the schema cannot follow |
| `keypoints-ordered`, `keyframes-ordered` | no | an order over array elements, not a constraint on one |
| `keypoint-within-declared-range`, `keyframe-within-declared-range` | no | the range is declared by the parameter, elsewhere in the document |
| `exactly-one-neutral-keypoint` | no | a count over a sibling array |
| `at-most-one-base-driver` | no | counts channel mappings and physics outputs that name one parameter |
| `binding-value-count-agrees-with-stale-mark` | no | compares a count against a mesh reached through a reference, in both directions |
| `at-most-one-track-per-parameter` | no | a uniqueness rule over a referenced value |

**A file this schema accepts is not yet a valid puppet.** It is a file whose *shapes* are right.

## The load limits are not here either

Every size and count the loader trusts is limited, in one place, and checked **before
allocating** — `limits::FIGURES` in `../src/limits.rs`. Some of those would fit a `maxItems`,
and they are deliberately not written here: a limit is enforced against bytes and entries
actually produced, several figures are still provisional, and a second copy in a file nothing
checks would be the drift this project keeps scripts to prevent.

## Three places the schema is stricter than this build's reader

Each is a question for `docs/format.md` rather than a decision taken here, and each is recorded
in `docs/wip/s2-format.md`.

- **`composition_mode` is required.** `docs/format.md` says it is always written, and every
  fixture writes it. The reader defaults it to `sum` when it is absent, which is the lenient
  reading and which no fixture exercises.
- **An `x` entry requires `data`.** `EXT-3` names `version` and `name`; the document names no
  third required member, and the reader requires `data` as well.
- **A node's `kind` is one of four.** That is the rule at version 1.1. A file declaring a later
  minor may carry a kind this build has never heard of, which loads and is preserved opaquely —
  so running this schema over such a file reports material the format expects it to carry.

The schema is **not** stricter about extension `data`: its contents are deliberately
unconstrained. *Structured values only, never a string that is executed, resolved or interpreted
as a path or URL* binds what reads the data, not what a file may hold — [I1](../../../docs/invariants.md)
is a promise about our code. `rigged:ext-hostile` is a valid puppet carrying path-, URL- and
command-shaped strings in every string-bearing field, and it must stay one.

## Keeping it from drifting

`../src/validate.rs` carries a unit test holding each object's member list in `puppet.schema.json`
against the key sets in `../src/decode.rs`, which are what the reader accepts. A member added to
one and not the other fails that test rather than being found by whoever runs the schema next
year.

What no test here covers is the schema being *run*: validating a document against it needs a
JSON Schema implementation, and adding one to the locked dependency graph to check a file the
acceptance suite already loads would buy very little. Running it is `VAL-14`'s human gate, where
a person writes a validator from the specification and sees whether they reach the same verdict.
