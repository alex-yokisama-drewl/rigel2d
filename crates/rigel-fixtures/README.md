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
Windows, so the file is `rigged.<defect>.rigel2d`. `.rigel2d` is the extension `docs/format.md`
specifies.

## The encoding

The concrete encoding — key names, ordering, defaults, compression and the rest — is specified in
[`docs/format.md`](../../docs/format.md), and this crate follows it. A key is the term
`docs/vocabulary.md` sanctions for the thing, in `snake_case`. A disagreement between this crate
and that document is a defect in the crate.

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

## The load limits

**Most of them are no longer here.** S2 landed `crates/rigel-format/src/limits.rs`, which is the
one place a load limit is declared, and it carries a figure for every kind. A fixture built here
against a limit reads that figure rather than inventing one.

The figures this crate still spells out for itself are the `PROVISIONAL_*` constants in
`src/puppet.rs`, each carrying on its own doc comment why it is here and what pins it:

- `PROVISIONAL_TRIGGERS_PER_ANIMATION` and `PROVISIONAL_COORDINATE_COUNT` are **in step** with
  `limits.rs`. Each names a collection whose limit landed in S2b, each has its just-over fixture
  — `rigged:triggers-over-limit`, `rigged:coordinates-over-limit` — and a unit test here pins the
  fixture to the constant, so the constant and the row in `limits.rs` can only drift if both are
  edited. `limits.rs` names the constant in its own note for the same reason.
- `PROVISIONAL_NODE_LIMIT` is **stale**. It is 256; `limits.rs` sets `NodeCount` at 2160, so
  `rigged:near-limit` is nowhere near the node limit and no longer tests what its name says.
  Regenerating it means two thousand nodes in the committed corpus, which is a decision rather
  than an edit. Reported in `docs/wip/s2b-animation-shape.md`'s handoff.
