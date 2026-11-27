#!/usr/bin/env bash
# Gate 5: our own code never opens an outbound network connection.
#
# Asserts the claim in docs/decisions.md and the entry *No outbound network
# connection is opened* in docs/concerns/security.md.
#
# SCOPE, stated rather than implied. This is a deny-list over first-party Rust
# source. It cannot see what a dependency does, and docs/wip/open-questions.md
# records that the dependency and plugin cases are deliberately unresolved. A
# pass here means "we did not write one", never "none can happen".
#
# Listening is not connecting. Live binds a port and waits for a source to send
# to it (docs/requirements/performing.md); docs/concerns/ops.md calls source
# reconnection passive listening. So bind, listen, accept and recv are allowed
# and only outbound initiation is refused — which is why this is a list of
# specific calls rather than a ban on std::net.
set -uo pipefail
cd "${CLAUDE_PROJECT_DIR:-$PWD}" || exit 1

roots=()
for d in crates/*/src; do [ -d "$d" ] && roots+=("$d"); done

if [ ${#roots[@]} -eq 0 ]; then
  echo "no-outbound: no first-party source yet — nothing to check" >&2
  exit 0
fi

# Each entry: <regex>|<what it is>
patterns=(
  'TcpStream::connect|opens an outbound TCP connection'
  'UdpSocket::connect|fixes a remote peer for an outbound datagram'
  'SocketAddr[^;]*::connect|outbound connect'
  '\breqwest\b|HTTP client'
  '\bureq\b|HTTP client'
  '\bisahc\b|HTTP client'
  '\battohttpc\b|HTTP client'
  '\bsurf\b|HTTP client'
  '\bhyper::client|HTTP client'
  '\bcurl\b|HTTP client'
  'tokio::net::TcpStream|outbound TCP via tokio'
  'async_std::net::TcpStream|outbound TCP via async-std'
)

found=0
for entry in "${patterns[@]}"; do
  re="${entry%%|*}"
  what="${entry#*|}"
  while IFS= read -r hit; do
    [ -n "$hit" ] || continue
    if [ "$found" -eq 0 ]; then
      echo "no-outbound: FAILED — our own code must not open an outbound connection"
      echo
    fi
    found=1
    echo "  $hit"
    echo "      ^ $what"
  done < <(grep -rnE --include='*.rs' "$re" "${roots[@]}" 2>/dev/null)
done

if [ "$found" -ne 0 ]; then
  echo
  echo "See docs/decisions.md. If a connection is genuinely required, that is a decision to"
  echo "reopen there and in docs/concerns/security.md — not an exemption to add here."
  exit 1
fi

files=$(find "${roots[@]}" -name '*.rs' | wc -l)
echo "no-outbound: clean — $files first-party Rust file(s) checked, none initiates a connection"
echo "no-outbound: scope is our own source; a dependency's behaviour is not covered"
