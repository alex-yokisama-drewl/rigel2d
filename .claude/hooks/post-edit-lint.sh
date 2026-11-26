#!/usr/bin/env bash
# PostToolUse on Edit|Write — formats and lints the file just changed.
#
# Configure by creating .claude/hooks/lint.conf with one line per rule:
#     <glob>\t<command, with {} for the file path>
# Example:
#     *.ts	npx prettier --write {} && npx eslint --fix {}
#     *.php	./vendor/bin/pint {}
# Without that file this hook does nothing, by design — guessing a project's toolchain and
# running the wrong thing is worse than running nothing.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 0
. "$(dirname "$0")/_lib.sh"

conf="$(dirname "$0")/lint.conf"
[ -f "$conf" ] || exit 0

read_event
file="$(jget '.tool_input.file_path')"
[ -n "$file" ] && [ -f "$file" ] || exit 0

while IFS=$'\t' read -r glob cmd; do
  case "$glob" in ''|'#'*) continue ;; esac
  # shellcheck disable=SC2254
  case "$(basename "$file")" in
    $glob) eval "${cmd//\{\}/\"$file\"}" >/dev/null 2>&1 ;;
  esac
done < "$conf"

exit 0
