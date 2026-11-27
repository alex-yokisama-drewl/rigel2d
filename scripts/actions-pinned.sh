#!/usr/bin/env bash
# Gate 5: every third-party GitHub Action is pinned to a commit hash.
#
# Asserts the entry *Every `uses:` in `.github/workflows/` is pinned to a 40-hex
# commit SHA* in docs/concerns/security.md.
#
# WHY A HASH AND NOT A TAG. A tag is a mutable pointer in somebody else's
# repository. Whoever controls it can move it, and the next run of this workflow
# then executes code nobody here reviewed, with a checkout of this repository in
# front of it. A commit hash cannot be moved. The pinning was a comment convention
# until this gate existed, and one `@v4` would have passed every other check.
#
# SCOPE, stated rather than implied. This reads workflow files. It says nothing
# about what the pinned action does, and nothing about an action a pinned action
# itself calls. A pass means "no mutable ref is named here".
#
#   usage: scripts/actions-pinned.sh [--self-test]
set -uo pipefail

# A local action — `uses: ./path` — is this repository's own code and is covered by
# everything else that reads this tree. Anything else must carry a 40-hex sha.
scan() {
  local root="$1" files found=0 file line ref

  files="$(find "$root" -type f \( -name '*.yml' -o -name '*.yaml' \) 2>/dev/null | sort)"

  if [ -z "$files" ]; then
    echo "actions-pinned: FAILED — no workflow file found under $root"
    echo "  This gate reads workflow files. Finding none means the layout moved and the gate"
    echo "  is no longer looking at anything, which is not the same as a pass."
    return 1
  fi

  while IFS= read -r file; do
    while IFS= read -r line; do
      # The ref is whatever follows the last @, with any trailing comment removed.
      ref="$(printf '%s' "$line" | sed 's/#.*$//' | sed 's/.*[[:space:]]uses:[[:space:]]*//; s/[[:space:]]*$//')"
      [ -n "$ref" ] || continue
      case "$ref" in
        ./*|."/"*) continue ;;                       # this repository's own action
      esac
      case "$ref" in
        *@[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f])
          continue ;;
      esac
      if [ "$found" -eq 0 ]; then
        echo "actions-pinned: FAILED — every third-party action must be pinned to a commit hash"
        echo
      fi
      found=1
      echo "  ${file#"$root"/}: $ref"
      case "$ref" in
        *@*) echo "      ^ the ref after @ is not a 40-hex commit sha" ;;
        *)   echo "      ^ no ref at all, so this tracks the action's default branch" ;;
      esac
    done < <(grep -nE '^[[:space:]]*-?[[:space:]]*uses:' "$file" 2>/dev/null | sed 's/^[0-9]*://')
  done <<< "$files"

  if [ "$found" -ne 0 ]; then
    echo
    echo "Resolve the tag to its commit and pin that, keeping the tag as a trailing comment:"
    echo "  uses: actions/checkout@<40-hex> # v5"
    return 1
  fi

  echo "actions-pinned: clean — $(printf '%s\n' "$files" | wc -l) workflow file(s) checked"
  echo "actions-pinned: scope is the refs named here; what a pinned action does is not covered"
  return 0
}

# A check over text decays silently when edited: a pattern that stops matching still
# passes. This plants each forbidden form and each allowed one and asserts the verdict.
self_test() {
  local tmp rc=0 sha="fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  local forbidden=(
    "      - uses: actions/checkout@v5"
    "      - uses: actions/checkout@main"
    "      - uses: actions/checkout"
    "      - uses: actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c0 # 39 hex"
    "      - uses: docker://alpine:3"
  )
  local allowed=(
    "      - uses: actions/checkout@$sha # v5"
    "      - uses: actions/checkout@$sha"
    "      - uses: ./.github/actions/local"
  )

  local i=0 body
  for body in "${forbidden[@]}"; do
    i=$((i + 1))
    rm -rf "$tmp/w"; mkdir -p "$tmp/w"
    printf 'jobs:\n  j:\n    steps:\n%s\n' "$body" > "$tmp/w/a.yml"
    if scan "$tmp/w" >/dev/null 2>&1; then
      echo "self-test FAILED: forbidden case $i passed the gate — $body"
      rc=1
    fi
  done

  i=0
  for body in "${allowed[@]}"; do
    i=$((i + 1))
    rm -rf "$tmp/w"; mkdir -p "$tmp/w"
    printf 'jobs:\n  j:\n    steps:\n%s\n' "$body" > "$tmp/w/a.yml"
    if ! scan "$tmp/w" >/dev/null 2>&1; then
      echo "self-test FAILED: allowed case $i was refused — $body"
      rc=1
    fi
  done

  # an empty tree must fail, not pass
  rm -rf "$tmp/w"; mkdir -p "$tmp/w"
  if scan "$tmp/w" >/dev/null 2>&1; then
    echo "self-test FAILED: a tree with no workflow file passed the gate"
    rc=1
  fi

  [ "$rc" -eq 0 ] && echo "actions-pinned: self-test passed — every mutable ref is caught and every pinned one permitted"
  return "$rc"
}

cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1
if [ "${1:-}" = "--self-test" ]; then
  self_test
else
  self_test >/dev/null || { echo "actions-pinned: the gate's own self-test failed — run --self-test"; exit 1; }
  scan .github/workflows
fi
