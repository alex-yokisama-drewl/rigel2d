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

# How many tests actually ran. A green result over an empty suite is not evidence, and without
# this the record says "passed" where it means "nothing was checked" — the failure
# docs/concerns/README.md names: a check that does not exist reads as covered.
#
# Counts passed + failed + ignored, not passed alone: a run where one test fails must not read
# as a run where one test vanished.
#
# `|| true` is load-bearing. set -e is back on by here and pipefail is set, so a grep that
# matches nothing would abort the script BEFORE the record is written — which is exactly the
# compile-error case, the one result most worth having. Null means the runner is not one we can
# read; it never means the script gave up.
# Every stage can legitimately match nothing, and under pipefail any of them would abort the
# script, so the whole pipeline is neutralised rather than just its first stage.
ran="$( { grep -oE '^test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed; [0-9]+ ignored' "$out" \
          | grep -oE '[0-9]+ (passed|failed|ignored)' \
          | grep -oE '^[0-9]+' \
          | awk '{s+=$1} END {if (NR) print s; else print "null"}'; } || true )"
[ -n "$ran" ] || ran=null
tail_out="$(tail -40 "$out" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk '{printf "%s\\n", $0}')"

cat > ".test-results/$sha.json" <<JSON
{
  "sha": "$sha",
  "dirty": $dirty,
  "command": "$(printf '%s' "$cmd" | sed 's/\\/\\\\/g; s/"/\\"/g')",
  "status": $status,
  "passed": $([ "$status" -eq 0 ] && echo true || echo false),
  "tests_run": $ran,
  "started": "$start",
  "finished": "$end",
  "tail": "$tail_out"
}
JSON

rm -f "$out"
echo
echo "recorded: .test-results/$sha.json (status $status)"
[ "$dirty" = true ] && echo "note: working tree was dirty — this result does not describe $sha alone"
[ "$ran" = "0" ] && echo "note: the suite is empty — a pass here means nothing was checked, not that anything works"
[ "$ran" = "null" ] && echo "note: could not read a test count from this runner's output — the record says null, not zero"
exit "$status"
