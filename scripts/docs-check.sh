#!/usr/bin/env bash
# Mechanical documentation checks.
#
# These are the defects that fail silently and that a reviewer catches only sometimes. Run
# before finishing any feature, and before dispatching doc-reviewer — a mechanical failure
# usually explains what would otherwise be reported as a contradiction.
#
# Exit 0 clean, 1 with findings.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1

fail=0
note() { printf '  %s\n' "$*"; }
section() { printf '\n%s\n' "$*"; }
problem() { fail=1; note "$*"; }

[ -d docs ] || { echo "no docs/ directory"; exit 1; }

# ---------------------------------------------------------------- index completeness
section "Index"
missing=0
while IFS= read -r f; do
  rel="${f#docs/}"
  case "$rel" in wip/*) continue ;; esac
  grep -q "($rel)" docs/README.md 2>/dev/null && continue
  # A document inside a subdirectory may be covered by a link to that directory.
  dir="${rel%/*}"
  [ "$dir" != "$rel" ] && grep -q "($dir/)" docs/README.md 2>/dev/null && continue
  problem "not in docs/README.md: docs/$rel"
  missing=1
done < <(find docs -name '*.md' ! -name 'README.md' | sort)
[ "$missing" -eq 0 ] && note "every document is indexed"

# ---------------------------------------------------------------- link resolution
section "Links"
broken=0
while IFS= read -r src; do
  base="$(dirname "$src")"
  sed 's/`[^`]*`//g' "$src" 2>/dev/null \
  | grep -o '](\([^)#][^)]*\))' \
  | sed 's/^](//; s/)$//' \
  | while IFS= read -r target; do
      case "$target" in http*|mailto:*|'#'*) continue ;; esac
      target="${target%%#*}"
      [ -z "$target" ] && continue
      [ -e "$base/$target" ] && continue
      printf 'BROKEN\t%s\t%s\n' "$src" "$target"
    done
done < <(find docs -name '*.md'; ls CLAUDE.md 2>/dev/null) > /tmp/.docs-check-links.$$ 2>/dev/null

if [ -s /tmp/.docs-check-links.$$ ]; then
  while IFS=$'\t' read -r _ src target; do
    problem "broken link in $src → $target"
    broken=1
  done < /tmp/.docs-check-links.$$
fi
rm -f /tmp/.docs-check-links.$$
[ "$broken" -eq 0 ] && note "every relative link resolves"

# ---------------------------------------------------------------- anchor resolution
# A link to a heading that does not exist lands the reader at the top of the right
# file, which is the kind of defect nobody reports and nobody notices. Checked here
# because the status tag is part of the heading text and therefore part of the
# anchor: promoting a [proposed] to [decided] changes every anchor into that
# section. That is a real cost, and the point of this check is that it is paid
# loudly, at the moment of promotion, rather than silently afterwards.
section "Anchors"
slug() {
  printf '%s\n' "$1" \
  | sed -E 's/`([^`]*)`/\1/g; s/\[([^]]*)\]\([^)]*\)/\1/g; s/\*+//g' \
  | tr '[:upper:]' '[:lower:]' \
  | sed -E 's/[^a-z0-9 _-]+//g; s/ +/-/g; s/^-+//; s/-+$//'
}

anchors_of() {
  grep -E '^#+ ' "$1" 2>/dev/null | sed -E 's/^#+ +//' | while IFS= read -r h; do slug "$h"; done
}

bad_anchor=0
while IFS= read -r src; do
  base="$(dirname "$src")"
  sed 's/`[^`]*`//g' "$src" 2>/dev/null \
  | grep -oE '\]\([^)]*#[^)]*\)' \
  | sed -E 's/^\]\(//; s/\)$//' \
  | while IFS= read -r target; do
      case "$target" in http*|mailto:*) continue ;; esac
      frag="${target#*#}"
      path="${target%%#*}"
      [ -z "$frag" ] && continue
      if [ -z "$path" ]; then tgt="$src"; else tgt="$base/$path"; fi
      [ -e "$tgt" ] || continue   # already reported by the link check above
      case "$tgt" in *.md) ;; *) continue ;; esac
      anchors_of "$tgt" | grep -xF "$frag" >/dev/null && continue
      printf 'ANCHOR\t%s\t%s\n' "$src" "$target"
    done
done < <(find docs -name '*.md'; ls CLAUDE.md 2>/dev/null) > /tmp/.docs-check-anch.$$ 2>/dev/null

if [ -s /tmp/.docs-check-anch.$$ ]; then
  while IFS=$'\t' read -r _ src target; do
    problem "no such heading: $src → $target"
    bad_anchor=1
  done < /tmp/.docs-check-anch.$$
fi
rm -f /tmp/.docs-check-anch.$$
[ "$bad_anchor" -eq 0 ] && note "every heading link resolves"

# ---------------------------------------------------------------- work in flight
section "Work in flight"
branch="$(git branch --show-current 2>/dev/null || echo '')"
default="$(git symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null \
  | sed 's|^origin/||')"
[ -z "$default" ] && for c in main master trunk; do
  git show-ref --verify --quiet "refs/heads/$c" 2>/dev/null && { default="$c"; break; }
done
if [ -n "$branch" ] && [ "$branch" = "${default:-main}" ]; then
  stray="$(find docs/wip -maxdepth 1 -name '*.md' ! -name 'plan.md' ! -name 'calibration.md' ! -name 'open-questions.md' \
    2>/dev/null)"
  if [ -n "$stray" ]; then
    printf '%s\n' "$stray" | while IFS= read -r f; do
      echo "  feature document on the default branch: $f"
    done
    problem "feature documents must never reach ${default:-main}"
  else
    note "no feature document on the default branch"
  fi
else
  note "on $branch — not checked"
fi

# ---------------------------------------------------------------- invariants agree
section "Invariants"
if [ -f docs/invariants.md ] && [ -f CLAUDE.md ]; then
  a="$(grep -oE '^## (I[0-9]+)\.' docs/invariants.md 2>/dev/null | grep -oE 'I[0-9]+' | sort -u)"
  b="$(grep -oE '\bI[0-9]+\b' CLAUDE.md 2>/dev/null | sort -u)"
  only_doc="$(comm -23 <(printf '%s\n' "$a") <(printf '%s\n' "$b") | tr '\n' ' ')"
  only_cla="$(comm -13 <(printf '%s\n' "$a") <(printf '%s\n' "$b") | tr '\n' ' ')"
  [ -n "${only_doc// /}" ] && problem "in invariants.md but not CLAUDE.md: $only_doc"
  [ -n "${only_cla// /}" ] && problem "in CLAUDE.md but not invariants.md: $only_cla"
  [ -z "${only_doc// /}" ] && [ -z "${only_cla// /}" ] && note "the two halves agree"
else
  note "skipped — invariants.md or CLAUDE.md missing"
fi

# ---------------------------------------------------------------- concern entries
section "Concern checklists"
untagged=0
for f in docs/concerns/*.md; do
  [ -e "$f" ] || continue
  case "$f" in */README.md) continue ;; esac
  while IFS= read -r line; do
    printf '%s' "$line" | grep -qE '`(script|test|judgment)`' && continue
    problem "entry without a verification tag in $f: $(printf '%s' "$line" | cut -c1-70)"
    untagged=1
  done < <(grep -E '^- \*\*' "$f" 2>/dev/null)
done
[ "$untagged" -eq 0 ] && note "every entry names how it is verified"

judg="$(grep -rhoE '`judgment`' docs/concerns/ 2>/dev/null | wc -l | tr -d ' ')"
auto="$(grep -rhoE '`(script|test)`' docs/concerns/ 2>/dev/null | wc -l | tr -d ' ')"
note "verification mix — automated: $auto, judgment: $judg (the second should fall over time)"

# ---------------------------------------------------------------- open questions
# Two forms of the tag, and both must be listed. A heading-level [open] is hard to
# miss when reading. An inline one, inside a section tagged [decided] or [proposed],
# is easy to miss and is the reason this check exists at all: a hand-written index
# used to compensate for it, and drifted.
#
# Files that *define* or *collect* the tag rather than use it are excluded, or they
# drown the list.
section "Open questions"
oq_excl='docs/documentation.md|docs/README.md|docs/requirements/README.md|docs/wip/open-questions.md|docs/decisions.md|docs/research/'

oq_head="$(grep -rnE '^#+ .*\[open\]' docs/ 2>/dev/null | grep -vE "^($oq_excl)")"
oq_line="$(grep -rnE '\[open\]' docs/ 2>/dev/null | grep -vE "^($oq_excl)" | grep -vE ':[0-9]+:#+ ')"

if [ -n "$oq_head" ]; then
  printf '  unsettled sections\n'
  printf '%s\n' "$oq_head" | sed 's/^/    /'
fi
if [ -n "$oq_line" ]; then
  printf '  unsettled points inside settled sections\n'
  printf '%s\n' "$oq_line" | sed 's/^/    /'
fi
if [ -z "$oq_head" ] && [ -z "$oq_line" ]; then
  note "none"
else
  note "(informational — nothing should be built that assumes an answer to these)"
fi

printf '\n'
if [ "$fail" -eq 0 ]; then echo "docs-check: clean"; else echo "docs-check: findings above"; fi
exit "$fail"
