#!/usr/bin/env bash
# PreToolUse and PostToolUse on the subagent and skill tools — appends to .calibration/trace.jsonl.
#
# CALIBRATION ONLY. Remove with the rest of the apparatus before real work: see the suite's
# design/calibration.md. Nothing outside .calibration/ depends on this.
#
# The record is written by a hook rather than by the model on purpose: self-reported logging is
# unreliable in a specific direction — a model logs best when things went well and forgets when
# it is confused, which is exactly when the record matters.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 0
. "$(dirname "$0")/_lib.sh"

mkdir -p .calibration || exit 0
read_event

phase="${1:-unknown}"
ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
branch="$(git branch --show-current 2>/dev/null || echo '')"

if command -v jq >/dev/null 2>&1; then
  printf '%s' "$HOOK_INPUT" \
    | jq -c --arg ts "$ts" --arg phase "$phase" --arg branch "$branch" \
        '{ts: $ts, phase: $phase, branch: $branch,
          tool: .tool_name,
          subagent: (.tool_input.subagent_type // .tool_input.skill // null),
          description: (.tool_input.description // null),
          prompt: (.tool_input.prompt // null),
          response: (.tool_response // null)}' \
    >> .calibration/trace.jsonl 2>/dev/null
else
  printf '{"ts":"%s","phase":"%s","branch":"%s","raw":%s}\n' \
    "$ts" "$phase" "$branch" "$(printf '%s' "$HOOK_INPUT" | tr -d '\n')" \
    >> .calibration/trace.jsonl 2>/dev/null
fi

exit 0
