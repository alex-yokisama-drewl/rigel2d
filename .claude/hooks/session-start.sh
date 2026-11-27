#!/usr/bin/env bash
# SessionStart — prints orientation. Step 0 of the workflow skill, done deterministically so it
# cannot be skipped or half-remembered. Output is added to the session's context.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 0
. "$(dirname "$0")/_lib.sh"

in_git_repo || exit 0

branch="$(git branch --show-current 2>/dev/null)"
default="$(resolve_default_branch)"
dirty="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"

echo "## Orientation"
echo
echo "- Branch: \`${branch:-detached}\`${branch:+$([ "$branch" = "$default" ] && echo '  ← default branch; work belongs on a branch')}"
echo "- Uncommitted files: $dirty"

if [ -d docs/wip ]; then
  wip="$(find docs/wip -maxdepth 1 -name '*.md' -printf '%f\n' 2>/dev/null | sort)"
  if [ -z "$wip" ]; then
    echo "- Work in flight: none (normal between phases)"
  else
    echo "- Work in flight:"
    printf '%s\n' "$wip" | sed 's/^/  - docs\/wip\//'
  fi
fi

if [ -d .test-results ]; then
  head_sha="$(git rev-parse --short HEAD 2>/dev/null)"
  latest="$(ls -t .test-results/*.json 2>/dev/null | head -1)"
  if [ -n "$latest" ]; then
    sha="$(basename "$latest" .json)"
    if [ "$sha" = "$head_sha" ]; then
      ran="$(sed -n 's/.*"tests_run": *\([^,]*\).*/\1/p' "$latest" 2>/dev/null | head -1)"
      case "$ran" in
        0)    note=" — **0 tests ran**, so a pass means nothing was checked" ;;
        null) note=" — test count unreadable; open the file before trusting it" ;;
        "")   note="" ;;
        *)    note=" — $ran tests ran" ;;
      esac
      echo "- Tests: recorded for HEAD ($sha)$note — read \`$latest\`, do not re-run to find out"
    else
      echo "- Tests: last record is for $sha, HEAD is $head_sha — **stale**"
    fi
  fi
fi

echo
echo "Route through the \`workflow\` skill rather than guessing a lane."
