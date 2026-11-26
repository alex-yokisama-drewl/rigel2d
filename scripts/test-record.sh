#!/usr/bin/env bash
# Runs the project's test suite and records the outcome against the current commit.
#
# Test results are state on disk, not a protocol between agents. Anything that wants to know
# what passed reads .test-results/<sha>.json; nothing re-runs the suite to find out.
#
# Configure by creating .claude/test-cmd with the command to run, one line. Without it this
# script does nothing — guessing a project's runner and reporting a false pass is worse.
#
#   usage: scripts/test-record.sh [extra args passed to the test command]
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1

conf=".claude/test-cmd"
if [ ! -f "$conf" ]; then
  echo "no $conf — create it with the project's test command on one line" >&2
  exit 2
fi
cmd="$(head -1 "$conf")"
[ -n "$cmd" ] || { echo "$conf is empty" >&2; exit 2; }

sha="$(git rev-parse --short HEAD 2>/dev/null || echo 'nogit')"
dirty=false
[ -n "$(git status --porcelain 2>/dev/null)" ] && dirty=true

mkdir -p .test-results
out="$(mktemp)"
start="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

set +e
# shellcheck disable=SC2086
eval "$cmd" "$@" 2>&1 | tee "$out"
status="${PIPESTATUS[0]}"
set -e

end="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
tail_out="$(tail -40 "$out" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk '{printf "%s\\n", $0}')"

cat > ".test-results/$sha.json" <<JSON
{
  "sha": "$sha",
  "dirty": $dirty,
  "command": "$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')",
  "status": $status,
  "passed": $([ "$status" -eq 0 ] && echo true || echo false),
  "started": "$start",
  "finished": "$end",
  "tail": "$tail_out"
}
JSON

rm -f "$out"
echo
echo "recorded: .test-results/$sha.json (status $status)"
[ "$dirty" = true ] && echo "note: working tree was dirty — this result does not describe $sha alone"
exit "$status"
