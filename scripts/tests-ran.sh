#!/usr/bin/env bash
# Gate 2's second half: the suite that just passed was not empty.
#
# Asserts the entry *Gate 2 fails when the workspace runs zero tests* in
# docs/concerns/ops.md.
#
# WHY. `cargo test` over a workspace with no test exits zero. From CI that is
# indistinguishable from a suite that ran and passed, so a green gate 2 can mean
# "nothing was checked" — the failure docs/concerns/README.md names, where a check
# that does not exist reads as covered. It was the intended state while the crates
# were shells; from the first subsystem with tests to lose it is a hazard.
#
# It reads the record scripts/test-record.sh wrote rather than re-running anything,
# because .claude/rules/testing.md makes test results state on disk: whatever wants
# to know what passed reads the file.
#
#   usage: scripts/tests-ran.sh [path to a result file]
set -uo pipefail

check() {
  local file="$1" ran

  if [ ! -f "$file" ]; then
    echo "tests-ran: FAILED — no result at $file"
    echo "  Run scripts/test-record.sh first. Its absence is not a pass: this gate exists to"
    echo "  distinguish a suite that ran from one that did not, and no record is the latter."
    return 1
  fi

  ran="$(sed -n 's/.*"tests_run"[[:space:]]*:[[:space:]]*\([0-9]*\|null\).*/\1/p' "$file" | head -1)"

  case "$ran" in
    '')
      echo "tests-ran: FAILED — $file carries no tests_run field"
      echo "  The record's shape changed and this gate is no longer reading anything."
      return 1 ;;
    null)
      echo "tests-ran: FAILED — $file records tests_run as null"
      echo "  The runner's output could not be counted, so this gate cannot tell a suite that"
      echo "  ran from one that did not. That is the same silent decay as a zero, not a pass."
      return 1 ;;
    0)
      echo "tests-ran: FAILED — the workspace ran zero tests"
      echo "  A pass over an empty suite means nothing was checked. If tests were lost, that is"
      echo "  the finding; if a crate's tests stopped being discovered, see the nested-test-"
      echo "  directory trap in docs/testing.md — cargo finds tests/*.rs and tests/<dir>/main.rs"
      echo "  only, and skips a bare nested directory without reporting it."
      return 1 ;;
  esac

  echo "tests-ran: clean — $ran test(s) ran"
  return 0
}

self_test() {
  local tmp rc=0
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  printf '{ "tests_run": 15 }\n'   > "$tmp/ok.json"
  printf '{ "tests_run": 0 }\n'    > "$tmp/zero.json"
  printf '{ "tests_run": null }\n' > "$tmp/null.json"
  printf '{ "passed": true }\n'    > "$tmp/nofield.json"

  check "$tmp/ok.json" >/dev/null 2>&1 || { echo "self-test FAILED: a non-empty suite was refused"; rc=1; }

  local f
  for f in zero null nofield absent; do
    if check "$tmp/$f.json" >/dev/null 2>&1; then
      echo "self-test FAILED: $f passed the gate"
      rc=1
    fi
  done

  [ "$rc" -eq 0 ] && echo "tests-ran: self-test passed — zero, null, a missing field and a missing file all fail"
  return "$rc"
}

cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1
if [ "${1:-}" = "--self-test" ]; then
  self_test
else
  self_test >/dev/null || { echo "tests-ran: the gate's own self-test failed — run --self-test"; exit 1; }
  check "${1:-.test-results/$(git rev-parse --short HEAD).json}"
fi
