# Security

Authentication, authorisation, input, secrets, exposure. The dimension where the failure mode is
silent until it is catastrophic, which is why the entries lean toward `test` over inspection.

## Universal

### Authorisation

- **Every endpoint and action checks authorisation server-side** — `test`
  A hidden button is not an authorisation check.
- **Authorisation is checked on the object, not only the route** — `test`
  The classic defect: a user may reach `/orders/:id` and sees any order. Test with a valid
  session and another user's identifier.
- **Enumeration does not leak existence** — `test`
  "Not found" and "not yours" are the same response where the distinction is itself
  information.
- **Privilege cannot be escalated through a mass-assigned field** — `test`
  A role, an owner or a flag accepted from request input.

### Input

- **Every input is validated server-side** — `test`
  Client validation is a convenience.
- **Queries are parameterised** — `script`
  Grep for string-built queries.
- **Output is escaped for its context** — `test`
  HTML, attribute, URL, SQL, shell are different contexts.
- **Uploads are constrained by type, size and count, and stored outside the served root** — `test`
- **Redirect targets are validated against an allow-list** — `test`

### Identity and session

- **Credentials are hashed with a current algorithm** — `script`
- **Sessions are invalidated on logout and on password change** — `test`
- **State-changing requests carry a CSRF defence** — `test`
- **Authentication and anything brute-forceable is rate-limited** — `test`
  Including anything with a guessable short code.

### Secrets and exposure

- **No secret is in the repository** — `script`
  Scanned in CI, not reviewed by eye.
- **No secret reaches a log, an error page, an analytics event, or the client bundle** — `script`
- **Errors shown to users carry no internal detail** — `test`
  Stack traces, query text, paths, versions.
- **Dependencies are scanned, and findings are triaged rather than accumulated** — `script`

### Data in transit and at rest

- **Transport is encrypted everywhere, including between internal services** — `judgment`
- **What is encrypted at rest, and what is not, is a recorded decision** — `judgment`

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the threat model, the roles, and anything exempt here on the first planning sweep.
-->

### Untrusted input

- **Every parser of untrusted input survives hostile input** — `judgment`
  → `test` once parsers exist: a time-boxed fuzz run in CI over the puppet, scene, PSD, image and port-message parsers. No crash, no hang, bounded memory.
  Seeded at planning, 2026-11-26.
- **A committed set of hostile fixtures is rejected cleanly** — `judgment`
  → `test`. Truncated, cyclic, duplicate identifiers, out-of-range index, NaN, oversized dimensions, decompression bomb, traversal-style entry names, unknown sections.
  Seeded at planning, 2026-11-26.
- **Load limits are defined in one place and each is exercised by a fixture just over it** — `judgment`
  → `test`, where a limit with no fixture fails the check.
  Seeded at planning, 2026-11-26.
- **A failed load leaves the displayed puppet unchanged** — `judgment`
  → `test`: load a hostile fixture while a good puppet renders, and assert the rendered buffer is unchanged. Asserts [I4](../invariants.md).
  Seeded at planning, 2026-11-26.

### The input port

- **The port binds to loopback, and loading a scene never changes that** — `judgment`
  → `test`. Off-machine listening is deferred, not rejected; when it arrives it is an explicit in-app opt-in and still never enabled from a file.
  Seeded at planning, 2026-11-26.
- **A source can set channel values and trigger animations, and nothing else** — `judgment`
  → `test`: malformed, oversized and many-channel messages leave scene state, loaded puppets and output configuration unchanged, within the memory cap.
  Seeded at planning, 2026-11-26.
- **Channel values are finite and clamped before binding** — `judgment`
  → `test`.
  Seeded at planning, 2026-11-26.

### What the applications may not do

- **Nothing supplied by a puppet, scene or source is ever executed** — `judgment`
  Review and a lint over the parsing paths; no field is evaluated as a script, a command or a format string, and no path or URL inside a file is resolved outside its container. **A plugin is not covered by this** — a plugin never arrives inside a file ([I6](../invariants.md)), which is what keeps the rule true for our code. Asserts [I1](../invariants.md).
  Seeded at planning, 2026-11-26.
- **Live requests no screen-capture or window-capture capability** — `judgment`
  → `script` scanning platform manifests and entitlements, failing on any. **Revisit before plugins ship**: the scan covers our own manifests, and a plugin is not our code.
  Seeded at planning, 2026-11-26.
- **No outbound network connection is opened** — `judgment`
  → `script`: `scripts/no-outbound.sh`, run as gate 5. Same caveat as above: the assertion covers our code, so its scope needs restating before plugins ship. **It also reads first-party source and cannot see the dependency graph.** A crate that pulls in an HTTP client transitively is invisible to it, and that is the likeliest way this decision actually breaks. A pass means "we did not write one", never "none can happen". See `docs/decisions.md`. **The other half now exists** — `scripts/locked-graph.sh`, below — so the pair covers both "we did not write one" and "nothing we build on brought one".
  Covered, S2 task 2, 2026-11-27.
  Seeded at planning, 2026-11-26.

### Supply chain

- **Dependencies are locked with hashes, CI actions pinned to commit hashes, scan clean or triaged** — `judgment`
  → `script`. Now mostly covered: `--locked` on every cargo invocation in CI, `Cargo.lock` committed, `scripts/actions-pinned.sh` checking that the pinning stays, and `scripts/locked-graph.sh` requiring a checksum on every registry dependency — no longer vacuous, since S2 added the first ones. **Still not covered: a vulnerability scan, and a place to record a triage decision.**
  Seeded at planning, 2026-11-26.
- **Every `uses:` in `.github/workflows/` is pinned to a 40-hex commit SHA** — `script`
  → `script`: `scripts/actions-pinned.sh`, run as gate 5. Fails on any ref that is not a 40-hex commit hash, and on a tree with no workflow file, so the check cannot pass by having stopped looking. A local `./` action is this repository's own code and is exempt. Scope: the refs named here, never what a pinned action does or what it calls in turn.
  Covered, S2 task 2, 2026-11-27.
  Seeded at the S1 review, 2026-11-27.
- **The locked dependency graph contains no network-client crate, and no git or alternative-registry source** — `script`
  → `script`: `scripts/locked-graph.sh`, run as gate 5. Reads `Cargo.lock` and refuses a named network client, a TLS stack, a `git+` source, an alternative registry, and a registry dependency carrying no checksum. A TLS stack is refused although it is not itself a client: a project that opens no outbound connection has no use for one, so its arrival is the signal. A deny-list, like `no-outbound.sh`, so it catches the named forms and no others. Chosen over `cargo deny` to add no dependency to the gate itself.
  Covered, S2 task 2, 2026-11-27.
  Seeded at the S1 review, 2026-11-27.
- **Signing keys and release tokens are unreachable from pull-request jobs** — `judgment`
  → `script` once releases exist.
  Seeded at planning, 2026-11-26.

### Texture lifetime, copy, paste and symmetry

- **Load limits bind every entry in a file, referenced or not and parsed or only carried** — `judgment`
  → `test`: an oversized texture that no part references, and a structural entry whose declared size and actual size disagree, each refused on Studio's path and on the player's. Data nothing renders is where a limit evasion would be put, because it has no visible effect until it has been allocated. Revised 2026-11-27: there is no longer a carry-without-parsing path, since the player loads the whole structure.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **Data moved between puppets is renumbered, and no reference crosses by numeric coincidence** — `judgment`
  → `test`: two fixtures whose identifier spaces overlap; copy a part whose mask names a part that was not copied. Assert the reference is dropped or the paste refused, never resolved against whatever holds that number in the destination. Identifiers are unique within a puppet and nothing more, so overlap is the normal case.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **Finiteness is judged at the precision the runtime evaluates in, and after an operation rather than only at parse** — `judgment`
  → `test`: a literal finite as a 64-bit value and infinite once narrowed; and a mirror whose reflection of a finite coordinate about a distant axis overflows. The first makes two tools disagree about whether one file is valid; the second produces an unloadable puppet from a legal one.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

### Physics, tracks and the limits they need

- **Physics cannot produce a non-finite or out-of-range value from a finite file** — `judgment`
  → `test`: fixtures with zero or negative simulation constants, extreme stiffness, a high-gain loop from output back to anchor, and a stalled frame interval. Run many frames and assert every parameter and transform stays finite and within the driven parameter's range. A non-finite state must reset to rest and never reach geometry, which is the cliff [I2](../invariants.md) guards.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **Physics simulation constants are range-validated at load, before anything is allocated for them** — `judgment`
  → `test`, one rejected fixture per constant that admits a degenerate value. A finite file does not imply a finite simulation, and nothing downstream re-checks.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A track's keyframe times are finite and strictly ordered, its values finite and within its parameter's range, and its blend durations finite and non-negative** — `judgment`
  → `test`, one rejected fixture per defect. Ordering is load-bearing rather than cosmetic: animation lookup seeks rather than scans, so unordered times give wrong results silently instead of failing.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **Every stored collection and string has a declared limit, and the limit set and the schema agree** — `judgment`
  → `script`: walk the format definition and fail on any collection or string with no limit, and on any limit with no just-over fixture. Physics nodes, tree depth through a chain, animations, tracks and keyframes are all newly countable and none is currently bounded. A kind added later with no limit then fails the check rather than shipping unbounded.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **The archive's directory is bounded and sanity-checked before any entry is read** — `judgment`
  → `test`: fixtures with an over-long entry name, duplicate entry names, overlapping local-header data, and an entry name the format does not define. Each is refused before anything is decompressed, with peak memory below the payload limit.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
- **A saved file contains no path, username or machine name from the writing machine** — `judgment`
  → `script`: save a fixture with the harness's home directory and user name set to unique strings, then scan every entry's decompressed bytes and fail on any occurrence. The rule is already stated for import provenance and for zip metadata; nothing checks it.
  Seeded at planning, 2026-11-27, on the physics-and-track revision.
