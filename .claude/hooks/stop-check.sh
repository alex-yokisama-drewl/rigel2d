#!/usr/bin/env bash
# Stop — warns when a task is marked done but the work is not committed.
#
# Advisory, never blocking: stopping is the user's prerogative and a hook that fights it gets
# turned off.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 0
. "$(dirname "$0")/_lib.sh"

in_git_repo || exit 0
[ -z "$(git status --porcelain 2>/dev/null)" ] && exit 0

wipdoc="$(find docs/wip -maxdepth 1 -name '*.md' ! -name 'plan.md' ! -name 'calibration.md' \
  -printf '%p\n' 2>/dev/null | head -1)"
[ -n "$wipdoc" ] || exit 0

if grep -qE '^###.*—[[:space:]]*`done`' "$wipdoc" 2>/dev/null; then
  echo "Note: $wipdoc marks a task \`done\` and the working tree is not clean." >&2
  echo "A task's code and its document update land in one commit." >&2
fi
exit 0
