# `rigel-fixtures`

The hand-authored puppet fixture corpus, and the builder that produces it.

**This crate depends on no other crate in this workspace, and it must stay that way.** A fixture
produced by our writer and then read back by our reader asserts only that we agree with
ourselves. These files are authored from [`docs/format.md`](../../docs/format.md) with
third-party zip, JSON and PNG libraries, so that `rigel-format` is tested against the
specification rather than against itself. Reaching for a type from `rigel-format` here is the
one failure this crate exists to prevent.

```
cargo run -p rigel-fixtures -- crates/rigel-format/tests/fixtures   # regenerate
./scripts/fixtures-check.sh                                         # committed bytes still match
```

Both the corpus and the builder are committed. The corpus, because the third-party gate needs a
real file to `unzip`, read and repack, and because a fixture you can open is worth more than one
you have to run a program to see. The builder, because a fixture is then regenerated rather than
hand-patched. `scripts/fixtures-check.sh` is what stops the two drifting.

`catalogue.json`, written beside the fixtures, maps each catalogue name to its file, what a
conforming loader does with it, and the one thing that makes it what it is. A test reaches a
fixture through that rather than guessing a path.

**Fixture file names.** `docs/requirements/puppet-acceptance.md` writes a variant as
`rigged:<defect>`. A `:` cannot appear in a Windows path and the push gates check the tree out on
Windows, so the file is `rigged.<defect>.rigel2d`. `docs/format.md` names no file extension;
`.rigel2d` is this corpus's convention and decides nothing.

## What the specification did not say, and what was chosen

`docs/format.md` settles the container, the version gate, identifiers, extension data, the node
kinds, the physics parameterisation, the texture record and the validation rules. It names JSON
keys for physics fields and for extension data, and for nothing else — so encoding `rigged` meant
choosing the rest. **Every choice below is provisional and belongs in `docs/format.md` or in a
decision record; none of it was settled by anyone but this builder**, and `docs/wip/s2-format.md`
carries the same list in its handoff.

The naming rule used throughout: a key is the term `docs/vocabulary.md` sanctions for the thing,
in `snake_case`. The third-party gate's bar is that `rigged` is understandable from the
specification and ordinary tooling, and a key named anything else fails it.

| Chosen | Why this and not something else |
|---|---|
| The current format version is `1.1` | `rigged:old-version` has to declare "an earlier version this build upgrades", and an earlier *minor* only exists if the current one is not the first |
| The tree is a flat `nodes` array, each node carrying `parent`, with a top-level `root` naming one | A nested tree cannot express a cycle or a second root, so `rigged:cycle`, `rigged:no-root` and `rigged:two-roots` would have no fixture and the acyclicity and one-root rules nothing to reject. Containment is then a reference by identifier, which is what [I3](../../docs/invariants.md) asks for |
| `nodes` is in depth-first preorder, and order among siblings is draw order | The one thing order is allowed to decide. Every other collection is ordered by identifier, and keypoints by position |
| Defaulted fields are absent when they hold their default | Stated for the puppet's gravity, and `docs/format.md` says that is "the stored-form rule every other defaulted field here follows". So a physics node at its defaults carries almost nothing, and `rigged`'s carry non-default values to be visible at all |
| The puppet's gravity defaults to `1.0` | The multiplier each node applies to it defaults to `1.0`, and no other figure is stated anywhere |
| A part's mesh carries `vertices`, `uvs` and `triangles`, with `uvs` in the texture's **authored** pixel space | `docs/format.md` says mesh coordinates are in the artwork's authored space and that the runtime reconstructs that space from the crop rectangle and the authored dimensions. That is a statement about the texture coordinates |
| A binding carries `values`, one entry per keypoint of its parameter, and a `stale` mark written only when set | The value-count rule is stated against the mesh; this is the shape that makes a count exist to compare |
| A track carries `blend_duration` | `docs/concerns/security.md`, by way of `VAL-26`, requires a track's blend durations to be validated. Nothing in `docs/format.md` defines one |
| `composition_mode` is always written | `DET-6` asserts `rigged` carries it. It is the one defaulted-looking field that is not omitted, because a mode added later must not alter a puppet authored before it |
| An extension declaring a plugin is an ordinary `x` entry | `I6` says a puppet may declare that it carries data for a named plugin at a named version. The `x` marker already carries an identifier, a version and a human-readable name, and `docs/format.md` defines no second mechanism |
| Compression is Deflate for JSON, Stored for PNG | `docs/format.md` requires the method and level to be fixed per entry kind without naming either. A PNG is already deflated inside its own container, so storing it keeps the payload extractable in one step. The level is the zip crate's default, pinned by `Cargo.lock` |
| Entry modification times are 1980-01-01 00:00:00, external attributes are `0644` | The earliest a zip date field expresses, so there is no question of it having come from a clock; a pinned permission bit carries no umask |

## Two places the documents disagree

Both are reported in `docs/wip/s2-format.md`'s handoff. Neither is this crate's to settle.

**`one-part`.** The fixture catalogue calls it "the smallest valid puppet: one root part, one
texture, nothing else". `docs/format.md` says the root is a kind that contains — a mesh group or
a composite — and gives its reasoning in terms of what an import produces. A root part is
therefore invalid, and the two claims cannot both hold. The fixture here keeps *valid*, which is
the property the catalogue leans on, and wraps the part in a composite. `root-is-part` is the
same file without the wrapper, which is the fixture for the rule itself.

**`rigged:nan`.** JSON has no literal for a quiet NaN, so a file carrying one is not strict JSON.
A writer that is not ours emits the bare token — Python's `json` module does, by default — and
that is what this fixture carries. The consequence is that this one rejection may come from the
parser rather than from the finiteness rule, so it cannot name an offending identifier the way
`VAL-12` asks. `rigged:positive-infinity` and `rigged:negative-infinity` use `1e400`, which is
strict JSON and overflows on parse, so those two can.

## The load limits are not here

`rigged:near-limit` sits one node under `PROVISIONAL_NODE_LIMIT`, a figure this builder invented.
`LIM-4` ties every load limit to four times the corresponding `stress` workload dimension, and
that workload is S3's. The just-over fixture per declared limit that `LIM-1`, `LIM-2` and `LIM-7`
want is therefore deferred: the limits do not exist yet, and a fixture built against a guessed
figure would have to be rebuilt anyway. When the figures land, they go in one table here and the
fixtures regenerate.
