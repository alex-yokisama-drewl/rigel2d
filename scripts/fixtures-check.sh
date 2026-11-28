#!/usr/bin/env bash
# The committed puppet fixture corpus is reproducible from its builder.
#
# The corpus is committed so that there is a real file to `unzip`, read and
# repack — the third-party gate needs one, and so does anyone reading
# docs/format.md against a file. The builder is committed beside it so a fixture
# is regenerated rather than hand-patched. This check is what stops the two
# drifting: it rebuilds into a temporary directory and asserts the committed
# artifacts are reproduced byte for byte.
#
# A failure here is one of two things. Either the builder changed and the corpus
# was not regenerated — rerun the command this script prints — or the builder is
# not deterministic, which is a defect in the builder.
#
# Exit 0 clean, 1 with findings.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1

COMMITTED=crates/rigel-format/tests/fixtures

fail=0
note() { printf '  %s\n' "$*"; }
section() { printf '\n%s\n' "$*"; }
problem() {
  fail=1
  note "$*"
}

section "Fixture corpus"

if [ ! -d "$COMMITTED" ]; then
  echo "no $COMMITTED directory"
  exit 1
fi

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

if ! cargo run --quiet --locked -p rigel-fixtures -- "$scratch" >/dev/null; then
  echo "the fixture builder did not run"
  exit 1
fi

# Committed but not produced: a fixture the builder no longer knows about, which
# nothing can regenerate.
while IFS= read -r path; do
  name="$(basename "$path")"
  [ -e "$scratch/$name" ] || problem "committed but not produced: $name"
done < <(find "$COMMITTED" -maxdepth 1 -type f | sort)

# Produced but not committed, and produced differently.
while IFS= read -r path; do
  name="$(basename "$path")"
  if [ ! -e "$COMMITTED/$name" ]; then
    problem "produced but not committed: $name"
  elif ! cmp -s "$path" "$COMMITTED/$name"; then
    problem "committed bytes differ from the builder's: $name"
  fi
done < <(find "$scratch" -maxdepth 1 -type f | sort)

if [ "$fail" -eq 0 ]; then
  count="$(find "$COMMITTED" -maxdepth 1 -type f ! -name catalogue.json | wc -l | tr -d ' ')"
  note "$count committed fixtures reproduce byte for byte"
else
  note ""
  note "to regenerate: cargo run -p rigel-fixtures -- $COMMITTED"
fi

exit "$fail"
