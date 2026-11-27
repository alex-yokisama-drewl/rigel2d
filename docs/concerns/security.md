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
  → `script`: `scripts/no-outbound.sh`, run as gate 5. Same caveat as above: the assertion covers our code, so its scope needs restating before plugins ship. **It also reads first-party source and cannot see the dependency graph.** A crate that pulls in an HTTP client transitively is invisible to it, and that is the likeliest way this decision actually breaks. A pass means "we did not write one", never "none can happen". See `docs/decisions.md`.
  Seeded at planning, 2026-11-26.

### Supply chain

- **Dependencies are locked with hashes, CI actions pinned to commit hashes, scan clean or triaged** — `judgment`
  → `script`. The manifest now exists, and only part of this is covered: `--locked` on every cargo invocation in CI, `Cargo.lock` committed, and `actions/checkout` pinned to a commit hash by hand. Not covered: any check that the pinning stays, a vulnerability scan, and a place to record a triage decision. There are no registry dependencies yet, so the checksum half is satisfied vacuously.
  Seeded at planning, 2026-11-26.
- **Every `uses:` in `.github/workflows/` is pinned to a 40-hex commit SHA** — `script`
  A check over the workflow files failing on any action ref that is not a full commit hash. The pinning is a comment convention today, and one `@v4` would pass every gate. A tag can be moved; a hash cannot.
  Seeded at the S1 review, 2026-11-27.
- **The locked dependency graph contains no network-client crate, and no git or alternative-registry source** — `script`
  `cargo deny check bans sources`, or a `cargo metadata` check against a deny-list, in CI. This is the only thing that can see what gate 5 cannot, and it binds from the first registry dependency.
  Seeded at the S1 review, 2026-11-27.
- **Signing keys and release tokens are unreachable from pull-request jobs** — `judgment`
  → `script` once releases exist.
  Seeded at planning, 2026-11-26.

### Texture lifetime, copy, paste and symmetry

- **Load limits bind every entry in a file, referenced or not and parsed or only carried** — `judgment`
  → `test`: an oversized texture that no part references, and an authoring-only entry whose declared size and actual size disagree, each refused on Studio's path and on the player's carry-without-parsing path. Data nothing renders is where a limit evasion would be put, because it has no visible effect until it has been allocated.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **Data moved between puppets is renumbered, and no reference crosses by numeric coincidence** — `judgment`
  → `test`: two fixtures whose identifier spaces overlap; copy a part whose mask names a part that was not copied. Assert the reference is dropped or the paste refused, never resolved against whatever holds that number in the destination. Identifiers are unique within a puppet and nothing more, so overlap is the normal case.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.

- **Finiteness is judged at the precision the runtime evaluates in, and after an operation rather than only at parse** — `judgment`
  → `test`: a literal finite as a 64-bit value and infinite once narrowed; and a mirror whose reflection of a finite coordinate about a distant axis overflows. The first makes two tools disagree about whether one file is valid; the second produces an unloadable puppet from a legal one.
  Seeded at planning, 2026-11-26, on the texture-lifetime and symmetry revision.
