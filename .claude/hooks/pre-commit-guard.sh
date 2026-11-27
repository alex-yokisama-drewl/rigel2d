#!/usr/bin/env bash
# PreToolUse on Bash — refuses commits that break the process rules that fail silently.
#
# These are hooks rather than instructions because "a step after the merge is a step that gets
# skipped". An instruction is a hope.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 0
. "$(dirname "$0")/_lib.sh"

read_event
cmd="$(jget '.tool_input.command')"
[ -z "$cmd" ] && exit 0
case "$cmd" in *"git commit"*) ;; *) exit 0 ;; esac

in_git_repo || exit 0

branch="$(git branch --show-current 2>/dev/null)"
default="$(resolve_default_branch)"

# 1. Never commit directly to the default branch.
if [ "$branch" = "$default" ]; then
  block "Refused: direct commit to the default branch ($default).

Work belongs on a branch — \`feature/<slug>\` or \`fix/<slug>\`. The only things that reach
$default directly are agreed via a PR.

If this is a phase plan being committed as part of \`plan-phase\`, that is the one documented
exception: say so and commit with --no-verify, or branch and open a PR."
fi

# 2. Code and its feature document land together.
staged="$(git diff --cached --name-only 2>/dev/null)"
[ -z "$staged" ] && exit 0

# This branch's own feature document, derived from the branch name.
#
# It used to be whatever `find docs/wip` returned first, which is wrong three ways: it could
# name another branch's document; once a finish commit staged the deletion of the real one
# `find` stopped seeing it and the guard demanded `open-questions.md`, blocking the very commit
# docs/documentation.md mandates; and on a branch with no feature document at all it demanded
# one anyway. The rule itself is unchanged — a feature branch still cannot commit code without
# its document — it now lands on the right file.
case "$branch" in
  feature/*|fix/*) slug="${branch#*/}" ;;
  *)               slug="" ;;
esac

wipdoc=""
if [ -n "$slug" ] && [ -f "docs/wip/$slug.md" ]; then
  wipdoc="docs/wip/$slug.md"
fi

# A staged change to it satisfies the rule, including the deletion that retires it at finish.
if [ -n "$slug" ] && printf '%s\n' "$staged" | grep -qFx "docs/wip/$slug.md"; then
  wipdoc=""
fi

if [ -n "$wipdoc" ]; then
  # Is anything outside docs/ being committed?
  code_changed="$(printf '%s\n' "$staged" | grep -v '^docs/' | head -1)"
  doc_staged="$(printf '%s\n' "$staged" | grep -Fx "$wipdoc" | head -1)"
  if [ -n "$code_changed" ] && [ -z "$doc_staged" ]; then
    block "Refused: code is staged but $wipdoc is not.

The feature document and the task's code land in the same commit. A document update left for
afterwards is a step that gets skipped, and it fails silently — the next session reads a
document that disagrees with the branch.

Update $wipdoc (task status, decisions, handoff slots) and stage it."
  fi
fi

exit 0
