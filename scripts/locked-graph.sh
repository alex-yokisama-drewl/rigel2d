#!/usr/bin/env bash
# Gate 5: the locked dependency graph carries no network client, and every crate in
# it comes from crates.io with a checksum.
#
# Asserts two entries in docs/concerns/security.md — *The locked dependency graph
# contains no network-client crate, and no git or alternative-registry source*, and
# the checksum half of *Dependencies are locked with hashes*.
#
# WHY THIS EXISTS. scripts/no-outbound.sh greps first-party source, so it cannot see
# the dependency graph at all: a crate that pulls in an HTTP client transitively is
# invisible to it, and docs/concerns/security.md records that as the likeliest way
# the no-outbound decision actually breaks. This is the other half. Together they
# cover "we did not write one" and "nothing we build on brought one".
#
# A git or alternative-registry source is refused for a different reason: a crates.io
# release is immutable and checksummed, a git ref is neither.
#
# SCOPE. This reads Cargo.lock, so it describes what would be built, not what runs.
# A crate can reach the network without any name here matching — this is a deny-list,
# like scripts/no-outbound.sh, and it catches the named forms and no others.
#
#   usage: scripts/locked-graph.sh [--self-test]
set -uo pipefail

CRATES_IO="registry+https://github.com/rust-lang/crates.io-index"

# Each entry: <crate>###<what it is>. A TLS stack is here although it is not itself a
# client: a project that opens no outbound connection has no use for one, so its
# arrival is the signal, not the proof.
denied=(
  'reqwest###HTTP client'
  'ureq###HTTP client'
  'isahc###HTTP client'
  'attohttpc###HTTP client'
  'surf###HTTP client'
  'curl###HTTP client'
  'curl-sys###HTTP client'
  'hyper###HTTP client and server'
  'hyper-util###HTTP client and server'
  'h2###HTTP/2 transport'
  'quinn###QUIC transport'
  'tonic###gRPC client'
  'tungstenite###WebSocket client'
  'tokio-tungstenite###WebSocket client'
  'native-tls###TLS stack, which this project has no outbound use for'
  'openssl###TLS stack, which this project has no outbound use for'
  'openssl-sys###TLS stack, which this project has no outbound use for'
  'rustls###TLS stack, which this project has no outbound use for'
)

scan() {
  local lock="$1" found=0 entry name what

  if [ ! -f "$lock" ]; then
    echo "locked-graph: FAILED — no $lock"
    echo "  This gate reads the lock file. Its absence is not a pass: an unlocked build"
    echo "  resolves dependencies at build time, which is the thing being prevented."
    return 1
  fi

  # One line per package: name<TAB>source<TAB>checksum. A package with no source is a
  # workspace member of ours and carries no checksum, which is correct and not a finding.
  local table
  table="$(awk -v RS='' -F'\n' '
    /^\[\[package\]\]/ {
      name=""; source=""; checksum=""
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^name = /)     { name     = $i; sub(/^name = "/,     "", name);     sub(/"$/, "", name) }
        if ($i ~ /^source = /)   { source   = $i; sub(/^source = "/,   "", source);   sub(/"$/, "", source) }
        if ($i ~ /^checksum = /) { checksum = $i; sub(/^checksum = "/, "", checksum); sub(/"$/, "", checksum) }
      }
      if (name != "") printf "%s\t%s\t%s\n", name, source, checksum
    }' "$lock")"

  if [ -z "$table" ]; then
    echo "locked-graph: FAILED — no package found in $lock"
    echo "  The lock file parsed to nothing, so this gate is no longer reading anything."
    return 1
  fi

  fail() {
    if [ "$found" -eq 0 ]; then
      echo "locked-graph: FAILED — $1"
      echo
    fi
    found=1
  }

  local pname psource pchecksum
  while IFS=$'\t' read -r pname psource pchecksum; do
    [ -n "$pname" ] || continue

    for entry in "${denied[@]}"; do
      name="${entry%%###*}"
      what="${entry##*###}"
      if [ "$pname" = "$name" ]; then
        fail "the locked graph must contain no network client"
        echo "  $pname — $what"
      fi
    done

    case "$psource" in
      '') : ;;                                   # a workspace member of ours
      git+*)
        fail "every dependency must come from crates.io"
        echo "  $pname — git source: $psource"
        echo "      ^ a git ref is mutable and unchecksummed; a crates.io release is neither"
        ;;
      "$CRATES_IO")
        if [ -z "$pchecksum" ]; then
          fail "every registry dependency must carry a checksum"
          echo "  $pname — from crates.io with no checksum line"
        fi
        ;;
      *)
        fail "every dependency must come from crates.io"
        echo "  $pname — alternative registry: $psource"
        ;;
    esac
  done <<< "$table"

  if [ "$found" -ne 0 ]; then
    echo
    echo "If one of these is genuinely required, that is a decision to take in docs/decisions.md"
    echo "and to describe in docs/concerns/security.md — not an exemption to add here."
    return 1
  fi

  echo "locked-graph: clean — $(printf '%s\n' "$table" | wc -l) locked package(s) checked"
  echo "locked-graph: scope is what the lock file names; what a crate does at runtime is not covered"
  return 0
}

self_test() {
  local tmp rc=0
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  pkg() { printf '[[package]]\nname = "%s"\nversion = "1.0.0"\n%s\n\n' "$1" "$2"; }
  local ok_src="source = \"$CRATES_IO\"
checksum = \"abc\""

  # allowed: a workspace member with no source, and an ordinary checksummed crate
  { pkg "rigel-format" ""; pkg "serde_json" "$ok_src"; } > "$tmp/ok.lock"
  if ! scan "$tmp/ok.lock" >/dev/null 2>&1; then
    echo "self-test FAILED: an ordinary locked graph was refused"
    rc=1
  fi

  # forbidden, one per lock file
  { pkg "rigel-format" ""; pkg "reqwest" "$ok_src"; }                                   > "$tmp/client.lock"
  { pkg "rigel-format" ""; pkg "rustls" "$ok_src"; }                                    > "$tmp/tls.lock"
  { pkg "zip" "source = \"git+https://github.com/x/zip?branch=main#deadbeef\""; }        > "$tmp/git.lock"
  { pkg "zip" "source = \"registry+https://example.com/index\"
checksum = \"abc\""; }                                                                  > "$tmp/altreg.lock"
  { pkg "zip" "source = \"$CRATES_IO\""; }                                              > "$tmp/nosum.lock"

  local f
  for f in client tls git altreg nosum; do
    if scan "$tmp/$f.lock" >/dev/null 2>&1; then
      echo "self-test FAILED: $f passed the gate"
      rc=1
    fi
  done

  # a missing lock file, and an empty one, must fail rather than pass
  if scan "$tmp/absent.lock" >/dev/null 2>&1; then
    echo "self-test FAILED: a missing lock file passed the gate"
    rc=1
  fi
  : > "$tmp/empty.lock"
  if scan "$tmp/empty.lock" >/dev/null 2>&1; then
    echo "self-test FAILED: a lock file with no package passed the gate"
    rc=1
  fi

  [ "$rc" -eq 0 ] && echo "locked-graph: self-test passed — every forbidden source and crate is caught and an ordinary graph permitted"
  return "$rc"
}

cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1
if [ "${1:-}" = "--self-test" ]; then
  self_test
else
  self_test >/dev/null || { echo "locked-graph: the gate's own self-test failed — run --self-test"; exit 1; }
  scan Cargo.lock
fi
