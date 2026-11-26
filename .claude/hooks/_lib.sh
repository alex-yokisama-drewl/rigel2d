#!/usr/bin/env bash
# Shared helpers for the workflow hooks. Sourced, never run directly.
#
# Hooks receive a JSON event on stdin. Everything here degrades to a no-op rather than
# failing: a hook that errors is a hook that gets disabled, and a disabled guard is worse
# than a missing one.

# Read stdin once into HOOK_INPUT.
read_event() { HOOK_INPUT="$(cat)"; }

# jget <jq-path> — extract a value from HOOK_INPUT. Empty string when absent.
jget() {
  if command -v jq >/dev/null 2>&1; then
    printf '%s' "$HOOK_INPUT" | jq -r "$1 // empty" 2>/dev/null
  elif command -v python3 >/dev/null 2>&1; then
    printf '%s' "$HOOK_INPUT" | python3 -I -c '
import sys, json
path = sys.argv[1].lstrip(".").split(".")
try:
    v = json.load(sys.stdin)
except Exception:
    sys.exit(0)
for p in path:
    if not isinstance(v, dict) or p not in v:
        sys.exit(0)
    v = v[p]
print(v if isinstance(v, str) else json.dumps(v))
' "$1" 2>/dev/null
  fi
}

# The repository's default branch, best effort.
default_branch() {
  git symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|^origin/||' \
    || true
}
resolve_default_branch() {
  local b; b="$(default_branch)"
  if [ -z "$b" ]; then
    for c in main master trunk; do
      git show-ref --verify --quiet "refs/heads/$c" && { b="$c"; break; }
    done
  fi
  printf '%s' "${b:-main}"
}

in_git_repo() { git rev-parse --git-dir >/dev/null 2>&1; }

# Refuse the tool call. stderr reaches the model; exit 2 is the block signal.
block() { printf '%s\n' "$*" >&2; exit 2; }
