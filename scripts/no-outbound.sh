#!/usr/bin/env bash
# Gate 5: our own code never opens an outbound network connection.
#
# Asserts the claim in docs/decisions.md and the entry *No outbound network
# connection is opened* in docs/concerns/security.md.
#
# SCOPE, stated rather than implied. This is a deny-list over first-party Rust
# source. A pass means "we did not write one", never "none can happen":
#
#   * It cannot see what a dependency does. A crate that pulls in an HTTP client
#     transitively is invisible here, and that is the likeliest way this decision
#     actually gets broken. docs/concerns/security.md records the dependency and
#     plugin cases as deliberately unresolved. Closing that needs a check over the
#     locked graph, not over source text.
#   * It is a deny-list, so it catches the forms named below and no others.
#
# LISTENING IS NOT CONNECTING, and that is the distinction to preserve if this is
# ever rewritten. Live binds a port and waits for a source to send to it
# (docs/requirements/performing.md), and docs/concerns/ops.md calls source
# reconnection passive listening. So bind, listen, accept, recv and recv_from are
# allowed; only outbound initiation is refused. A blanket ban on std::net would
# forbid the input port Live is specified to open.
#
# send_to IS refused, although it opens no connection: it is the shortest path
# from "we hold a bound socket" to "we are sending to a remote host". If a source
# handshake ever makes a reply-to-sender legitimate, that is a decision to take in
# docs/decisions.md and to describe here — not an exemption to add quietly.
#
#   usage: scripts/no-outbound.sh [--self-test]
set -uo pipefail

# Each entry: <regex>###<what it is>. The delimiter is not `|`, because a regex
# containing alternation would be split through the middle of itself.
patterns=(
  'TcpStream::connect###opens an outbound TCP connection'
  'TcpSocket::connect###opens an outbound TCP connection'
  'UdpSocket::connect###fixes a remote peer for outbound datagrams'
  'Socket::connect###outbound connect on a raw socket'
  '\.connect\(###outbound connect on a socket value'
  '\.send_to\(###sends a datagram to a remote address'
  '\breqwest::###HTTP client'
  '\bureq::###HTTP client'
  '\bisahc::###HTTP client'
  '\battohttpc::###HTTP client'
  '\bsurf::###HTTP client'
  'hyper::client###HTTP client'
  '\bcurl::###HTTP client'
  'use[[:space:]]+(reqwest|ureq|isahc|attohttpc|surf|curl)\b###HTTP client'
  'Command::new\("(curl|wget)"###shells out to a downloader'
  'libc::connect###the connect syscall'
  'rustix::net::connect###the connect syscall'
  'nix::sys::socket::connect###the connect syscall'
)

# Every tracked or untracked Rust file in the tree, not just crates/*/src: build.rs
# runs arbitrary code at build time, tests/ benches/ examples/ are first-party too,
# and a workspace member need not live under crates/.
collect() {
  find "$1" \
    -type d \( -name target -o -name .git \) -prune -o \
    -name '*.rs' -print 2>/dev/null | sort
}

scan() {
  local root="$1" files found=0 hit re what
  files="$(collect "$root")"

  if [ -z "$files" ]; then
    echo "no-outbound: FAILED — no Rust file found under $root"
    echo "  This gate greps source. Finding nothing means the layout moved and the gate is"
    echo "  no longer looking at anything, which is not the same as a pass."
    return 1
  fi

  for entry in "${patterns[@]}"; do
    re="${entry%%###*}"
    what="${entry##*###}"
    while IFS= read -r hit; do
      [ -n "$hit" ] || continue
      if [ "$found" -eq 0 ]; then
        echo "no-outbound: FAILED — our own code must not open an outbound connection"
        echo
      fi
      found=1
      echo "  $hit"
      echo "      ^ $what"
    done < <(printf '%s\n' "$files" | tr '\n' '\0' | xargs -0 grep -nE "$re" 2>/dev/null)
  done

  if [ "$found" -ne 0 ]; then
    echo
    echo "See docs/decisions.md. If a connection is genuinely required, that is a decision to"
    echo "reopen there and in docs/concerns/security.md — not an exemption to add here."
    return 1
  fi

  echo "no-outbound: clean — $(printf '%s\n' "$files" | wc -l) first-party Rust file(s) checked"
  echo "no-outbound: scope is our own source; a dependency's behaviour is not covered"
  return 0
}

# A deny-list decays silently when edited: a pattern that stops matching still passes.
# This plants each forbidden form and each allowed one and asserts the verdict, so the
# gate's own behaviour is checked on every run rather than the day it was written.
self_test() {
  local tmp rc=0
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  local forbidden=(
    'use std::net::TcpStream; fn f(){ let _ = TcpStream::connect("h:1"); }'
    'fn f(s: &std::net::UdpSocket){ let _ = s.connect("h:1"); }'
    'fn f(s: &std::net::UdpSocket){ let _ = s.send_to(b"x", "h:1"); }'
    'fn f(){ let _ = reqwest::blocking::get("https://x"); }'
    'use ureq;'
    'fn f(){ std::process::Command::new("curl").arg("https://x").spawn().ok(); }'
    'fn f(){ unsafe { libc::connect(0, std::ptr::null(), 0) }; }'
  )
  local allowed=(
    'fn f(){ let _ = std::net::UdpSocket::bind("127.0.0.1:0"); }'
    'fn f(s: &std::net::UdpSocket){ let mut b=[0u8;4]; let _ = s.recv_from(&mut b); }'
    'fn f(){ let _ = std::net::TcpListener::bind("127.0.0.1:0"); }'
    '//! Rigel2D uses no HTTP client: no reqwest here, and it never shells out to curl.'
  )

  local i=0 body
  for body in "${forbidden[@]}"; do
    i=$((i + 1))
    rm -rf "$tmp/c"; mkdir -p "$tmp/c"
    printf '%s\n' "$body" > "$tmp/c/x.rs"
    if scan "$tmp/c" >/dev/null 2>&1; then
      echo "self-test FAILED: forbidden case $i passed the gate — $body"
      rc=1
    fi
  done

  i=0
  for body in "${allowed[@]}"; do
    i=$((i + 1))
    rm -rf "$tmp/c"; mkdir -p "$tmp/c"
    printf '%s\n' "$body" > "$tmp/c/x.rs"
    if ! scan "$tmp/c" >/dev/null 2>&1; then
      echo "self-test FAILED: allowed case $i was refused — $body"
      rc=1
    fi
  done

  # build.rs must be reached, not just src/
  rm -rf "$tmp/c"; mkdir -p "$tmp/c"
  printf 'fn main(){ let _ = reqwest::blocking::get("https://x"); }\n' > "$tmp/c/build.rs"
  if scan "$tmp/c" >/dev/null 2>&1; then
    echo "self-test FAILED: a violation in build.rs passed the gate"
    rc=1
  fi

  # an empty tree must fail, not pass
  rm -rf "$tmp/c"; mkdir -p "$tmp/c"
  if scan "$tmp/c" >/dev/null 2>&1; then
    echo "self-test FAILED: a tree with no Rust file passed the gate"
    rc=1
  fi

  [ "$rc" -eq 0 ] && echo "no-outbound: self-test passed — every forbidden form is caught and every allowed one permitted"
  return "$rc"
}

cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1
if [ "${1:-}" = "--self-test" ]; then
  self_test
else
  self_test >/dev/null || { echo "no-outbound: the gate's own self-test failed — run --self-test"; exit 1; }
  scan .
fi
