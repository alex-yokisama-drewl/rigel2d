# Ops

Deployment, configuration, failure and recovery. What it takes to run the thing and to find out
what it is doing.

## Universal

### Configuration

- **Every setting that differs by environment is configuration, not a constant** — `script`
  Grep for hard-coded hosts, keys, paths and limits.
- **Missing configuration fails loudly at start-up** — `test`
  Never silently at the first request, and never with a default that works in development and
  is wrong in production.
- **Configuration is documented where it is defined** — `script`
  An example file, complete, with every key present.

### Deployment

- **A deploy is repeatable from a clean checkout** — `judgment`
- **A deploy can be rolled back** — `judgment`
  And the step that cannot — usually a migration — is known in advance. Cross-reference
  `data.md`.
- **The project runs locally from its documented setup** — `judgment`
  The block in `docs/stack.md` is run verbatim in a bare environment, with no step supplied by
  hand — no package index refresh, no elevated user, no exported PATH. A walk that quietly adds
  any of them passes while being wrong, which is how a first attempt was labelled walked while
  lines of the block failed a literal run. A scheduled or manual container job running the block
  would restore a `script` tag; its cadence is undecided, and until it exists no check does.

### Failure

- **Every external dependency has a defined behaviour when it is down** — `test`
  Fail, degrade, queue, or retry. Name which.
- **Retries are bounded and backed off** — `test`
- **Nothing retries a non-idempotent operation** — `judgment`
- **A failure in a non-essential path does not take down an essential one** — `judgment`

### Observability

- **Errors reach somewhere a person will see them** — `judgment`
  Not only a log file nobody reads.
- **Logs carry enough to trace one request end to end** — `test`
  A correlation identifier.
- **Logs carry no secret and no personal data beyond what is permitted** — `script`
  Cross-reference `security.md`.
- **There is a health check that means something** — `test`
  One that fails when the service cannot do its job, not one that returns 200 while the
  database is unreachable.
- **The things worth alerting on are stated** — `judgment`
  An alert on everything is an alert on nothing.

### Cost

- **Anything billed per use has a bound** — `judgment`
  An unbounded loop against a paid API is a financial incident.

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the environments, the deploy mechanism and the alerting destination here on the first
planning sweep.
-->

### Environments **[proposed]**

There is no staging and no production service. What exists:

| Environment | What it can establish |
|---|---|
| The maintainer's development machine, Linux | The only place the real-driver render path runs routinely |
| Hosted CI, Linux | The push gates in [stack.md](../stack.md), with no GPU and a noop device |
| Hosted CI, Windows and macOS | That it builds and that structural assertions pass, nothing more |
| The maintainer's Windows laptop | Real-hardware checks, run by hand and occasionally. Reachable over SSH, which reaches building and structural assertions but not the render path — see the caution below |
| A borrowed weak laptop, rarely, late | Counter-to-milliseconds calibration only |
| The user's machine | **Production.** Uncountable, no operator, no console |

**Before the work is public, Windows is trusted rather than continuously tested.** Linux is
tested routinely because it is where development happens; Windows gets occasional checks by hand.
That is weaker than the Supported tier in [platform.md](../requirements/platform.md) reads, and
is recorded here rather than left as an implication. It tightens when the work goes public.

**Limit on automating against the Windows laptop.** An SSH session on Windows is
non-interactive and sits outside the desktop session, where a swapchain and OBS behave
differently from what runs in the user's own session. Building and structural assertions could
be automated over SSH. Anything touching a swapchain, the compositor or a capture route cannot,
because measuring it there risks measuring the wrong thing entirely; the owner runs it by hand
in their own desktop session, as S0's Windows leg was run as a self-asserting bundle. Established
by S0, 2026-11-27.

**Caution on a self-hosted runner.** One reachable over SSH would let the structural assertions
and the counter gate run on real Windows hardware, which no hosted runner can offer. It must not
accept work from outside contributors: a self-hosted runner on a personal machine executing CI
from a public repository runs a stranger's code on that machine. If it is set up, it is gated to
the maintainer's own branches.

### Deploy mechanism **[proposed]**

A person downloads a build and installs it by hand. There is no update path
([decisions.md](../decisions.md)), so a release reaches a user only if they go looking. Studio
and Live are separate artifacts.

| | |
|---|---|
| Linux | A plain tarball and an AppImage. **Not a sandboxed format** — a sandbox breaks the absolute puppet paths a scene stores ([scene.md](../scene.md)) and the per-user configuration location |
| Windows | A zip, and an installer only if one proves necessary. Windows 10 64-bit is the floor |
| macOS | A disk image at the best-effort tier, deployment target macOS 11 |

The Linux floor is glibc 2.31, which is Ubuntu 20.04 and Debian 11. It is a ceiling on the CI
runner image as much as a claim about users: building on a newer runner silently excludes older
distributions, which is the entry below.

### Alerting destination **[proposed]**

There is no pager and nothing is sent anywhere. Three channels replace it:

| Channel | How |
|---|---|
| The user | The control surface and a local log. Never the render surface ([I2](../invariants.md)). Which failures interrupt a streamer is `[open]` |
| The developers | A report filed by hand on the public issue tracker, carrying a diagnostics report the user chose to attach. **The application never sends it** |
| The maintainer | CI failure blocks a merge |

Releases are published artifacts with checksums, on the same forge as the issue tracker. Opening
the user's browser at that page is a hand-off to the operating system and does not make the
application open a connection, so it does not touch the rule in
[decisions.md](../decisions.md) — stated because it reads like it might.

### Building and shipping

- **A release is built by CI from a tag, from the locked dependency set and the pinned toolchain, and both applications come out of the same tag** — `judgment`
  → `script`: the release job builds with `--locked`, fails without `rust-toolchain.toml`, and
  fails unless `scripts/test-record.sh` holds a passing result for the tag's commit SHA. A
  toolchain bump is its own commit and re-runs the counter gate.
  **The record is not yet readable across jobs.** CI runs the suite through
  `scripts/test-record.sh`, so the gate job writes a result. The results directory is gitignored
  and nothing uploads it, so the record exists only on the gate job's runner; a later job on a
  different runner cannot read it, and whoever builds the release job has to carry the record
  across as well as check it.
  Seeded at planning, 2026-11-26.

- **Each artifact runs on a clean machine with nothing installed beyond the platform's own components** — `judgment`
  → `script`: check the artifact's imported libraries against a per-platform allow-list. Windows
  must not need the Visual C++ runtime, so the CRT is static; Linux must stay under the glibc
  symbol-version ceiling above; macOS must set its deployment target. A missing runtime on
  Windows is a silent no-start with no console, which is the worst failure to diagnose by hand.
  **The push gates do not enforce the Linux floor and are not evidence of it**: they run on
  `ubuntu-latest`, and the symbol-version check is what enforces it. Ubuntu 20.04 hosted runners
  are retired, so the release job needs a container or another floor-compatible build
  environment; it cannot reuse the gate job's runner choice.
  Seeded at planning, 2026-11-26.

- **The built artifact reaches a first presented frame and exits cleanly, with an empty per-user directory and a read-only install location** — `judgment`
  → `script`: a release-job smoke launch under a virtual display with a software Vulkan device.
  A release job rather than a push gate, so [stack.md](../stack.md)'s gate list is untouched. It
  is also the first time the shipped binary runs anywhere in CI.
  Seeded at planning, 2026-11-26.

- **Every published artifact has a checksum generated by the release job, and the download instructions say what the platform's unsigned-binary warning looks like and how to proceed** — `judgment`
  → `script` for checksum presence and match; `docs-check.sh` checks the instruction text for
  presence only. If Windows signing is funded, a failing `signtool verify` joins the release job.
  Seeded at planning, 2026-11-26.

- **An older release stays downloadable and can open what a newer one wrote — puppet, scene and configuration** — `judgment`
  → `script` once a prior release exists: CI builds the previous tag's standalone validator and
  runs it over files the current build writes. **This is the desktop meaning of rollback** —
  nothing can be withdrawn from a user's disk, so the guarantee has to run the other way.
  Seeded at planning, 2026-11-26.

### Configuration

- **A configuration file that exists but cannot be read is kept, never overwritten, and the user is told once in the control surface; a missing one falls back silently** — `judgment`
  → `test`: zero-length, truncated, wrong-type and future-version fixtures all start on defaults,
  leave the file intact or moved aside, and report it. The control surface offers a reset,
  because a beginner will not find the per-user directory. Live's configuration holds per-source
  normalisation, which is per-person calibration that a silent overwrite destroys.
  Seeded at planning, 2026-11-26.

- **Configuration goes through the atomic write path, carries a version, and preserves keys it does not understand** — `judgment`
  → `script`: extend the [I7](../invariants.md) write-path script to configuration, with an
  explicit allow-list for append-only logs. [runtime.md](../runtime.md) settles that
  configuration is inside the one write path and an append-only log is not.
  Seeded at planning, 2026-11-26.

- **Restored window placement, output device and port are validated against the machine as it is now** — `judgment`
  → `test`: a pure function from saved placement and attached monitors to a placement, with
  fixtures for a monitor removed, a resolution change, and both windows saved off-screen. Extends
  *losing one behind the other is recoverable* to a window restored off-screen.
  Seeded at planning, 2026-11-26.

- **Every configuration key is documented in `stack.md`, none appears in the scene schema, and a key's absence is never an error** — `judgment`
  → `script`: `docs-check.sh` compares the configuration struct's keys to the document and to the
  scene schema. The second half is [scene.md](../scene.md)'s rule that a scene holds placement,
  not behaviour.
  Seeded at planning, 2026-11-26.

### Failure on a user's machine

- **A panic in any thread but the render loop never stops rendering, a panic during a replace-load leaves the previous puppet rendering, and no panic output reaches the render surface** — `judgment`
  → `test`: injected panics in the loader, the control-surface thread, the logger and the
  autosave writer, each asserting the rendered buffer is unchanged. Paired `script`: the release
  profile does not set `panic = "abort"`. **This extends [I4](../invariants.md) from failed loads
  to panics and adds panic to [I2](../invariants.md)'s error states** — an entry reaching into
  invariant territory, which is why it is written here for the owner to see rather than folded
  into one of them.
  Seeded at planning, 2026-11-26.

- **Start-up with no usable adapter, no display, or a port already taken yields a message naming the cause and the next step, never a silent exit** — `judgment`
  → `test`: injected adapter, display and bind failures. A port in use leaves Live rendering with
  its source flagged, per *nothing stops the puppet rendering*.
  Seeded at planning, 2026-11-26.

- **Losing the GPU device mid-run, as when a game triggers a driver reset, recovers or fails cleanly, never a frozen frame** — `judgment`
  → `test` with injected device loss. Recovery reuses the load-time preparation path, so the
  [I5](../invariants.md) assertion must count recovery as a load rather than as steady state, or
  it fires on a legitimate rebuild.
  Seeded at planning, 2026-11-26.

- **A full, read-only or missing disk fails every write without stopping rendering or losing what is in memory** — `judgment`
  → `test`: injected write failure for incremental scene writes, autosave, configuration and log.
  Rendering continues, the control surface reports it, the in-memory scene is retained, and the
  retry waits for the next change rather than spinning.
  Seeded at planning, 2026-11-26.

- **A save whose file changed on disk since it was loaded never silently discards the other application's work** — `judgment`
  → `test`: compare a content hash against the one recorded at load or last save. Where the only
  difference is channel mappings, re-read them and keep the rest — the targeted merge is
  tractable because mappings are a bounded part of the format and every reference is by stable
  identifier ([I3](../invariants.md)). Where it is not, warn and let the user overwrite or
  cancel. **There is no locking**: the file is the user's and a text editor can change it too.
  Seeded at planning, 2026-11-26.

- **Relaunch after a crash during a load does not crash again, and restores everything else** — `judgment`
  → `test`: a load-in-progress marker written before the restore and cleared at the first
  presented frame, with a panic-inducing fixture. The item being loaded when the last launch died
  comes back **dormant**, which is the mechanism
  [performing.md](../requirements/performing.md) already defines for an item that cannot be
  loaded. Recovering the rest of the setup is the point; re-attempting the item that killed it is
  how a loop starts.
  Seeded at planning, 2026-11-26.

- **A file that cannot be opened is reported as such, distinctly from a file that is refused** — `test`
  A missing path, a directory and an unreadable file each produce a different rule name from a malformed or hostile file. *Would have caught:* the validator reporting a mistyped path identically to a hostile puppet, leaving a script no way to tell them apart. Fixed; this entry keeps it fixed.
  Harvested at the S2 review, 2026-11-28.

### Observability with no operator

- **Both applications keep a local log that is bounded, rotated, and never written synchronously or allocated for inside the frame loop** — `judgment`
  → `test`: the soak asserts log bytes stay bounded, and the counter gate asserts zero frame-loop
  heap allocations with logging at its default level. The log is required because
  [analytics.md](analytics.md) and [decisions.md](../decisions.md) both depend on one, and it is
  specified in [platform.md](../requirements/platform.md) — *Local log and diagnostics report*,
  which this sweep is what prompted.
  Seeded at planning, 2026-11-26.

- **The log and the diagnostics report carry no personal data: home-directory paths are redacted, and no puppet or scene content, channel value or camera-derived value appears** — `judgment`
  → `test`: sentinel values pushed through the input port and a sentinel username in a loaded
  path, then both outputs scanned. Scene items store absolute paths, and tracking input is face
  data.
  Seeded at planning, 2026-11-26.

- **A diagnostics report is one action in the control surface and carries build, commit, format version, operating system, graphics backend, adapter and the recent log; the application never sends it** — `judgment`
  → `test` on the report's content. This is the only route by which a problem reaches the
  developers.
  Seeded at planning, 2026-11-26.

- **After an unclean shutdown the next launch says so and offers the report, non-modally in Live** — `judgment`
  → `test` with a simulated kill. Shares its mechanism with Studio's crash-recovery prompt, and
  Live must not wait on it — a stream does not wait for a dialog.
  Seeded at planning, 2026-11-26.

- **Each failure class reaches the control surface and the log and never the render surface, and source status reflects data arriving rather than a socket being open** — `judgment`
  → `test`: inject each class and assert both destinations, with an injected clock for source
  freshness. The classes are source dropped, output lost, device lost, write failed, load failed
  and dormant item. Which of them also interrupt the streamer is `[open]`.
  Seeded at planning, 2026-11-26.

### CI machinery

- **A counter ceiling rises only in a commit that also changes that ceiling's recorded reason, and the baseline records whether it is still provisional** — `judgment`
  → `script`: a diff-aware check over the committed baseline file. *A stated reason* is
  unenforceable prose until something reads it.
  Seeded at planning, 2026-11-26.

- **The required status checks match the workflow's job names exactly** — `script`
  A check comparing GitHub's required-status-check names for `master` against the job names the workflow actually produces, including expanded matrix names, and asserting `strict` is on and admins are enforced. A renamed job otherwise silently stops being required while every run stays green.
  Seeded at the S1 review, 2026-11-27.

- **Gate 2 fails when the workspace runs zero tests** — `script`
  → `script`: `scripts/tests-ran.sh`, run as gate 2's second step. Gate 2 now runs the suite through `scripts/test-record.sh`, and this reads the `tests_run` that record carries. Fails on zero, and equally on `null` or a missing field — a count that cannot be read is the same silent decay as a zero, not a pass. Bound from S2, the first subsystem with tests to lose.
  Covered, S2 task 2, 2026-11-27.
  Seeded at the S1 review, 2026-11-27.

- **A test count never falls without a recorded reason** — `script`
  The zero-test gate fails only at zero, so losing an entire suite leaves it green as long as something still runs. Compare the recorded count against a committed floor that rises with new tests and falls only in a commit that edits its stated reason. *Would have caught:* a dropped test-module declaration, or a nested test directory the build silently skips.
  Harvested at the S2 review, 2026-11-28.

- **Every continuous-integration job sets an explicit timeout** — `script`
  Extend the existing workflow-checking script, or add a sibling. *Would have caught:* a hanging job holding a metered runner for the platform default, in a workflow that already manages metered-runner cost by other means.
  Harvested at the S2 review, 2026-11-28.

- **A fuzz failure's input is recoverable from the CI run** — `judgment`
  → `script` asserting the workflow uploads the failing input on failure. **A mitigated gap, not a violation:** the failing input is written to disk and nothing uploads it, but the run is deterministic and prints its seed, so the failure can be reproduced from the log. Recovering the input directly is a convenience until the seed stops being enough.
  Harvested at the S2 review, 2026-11-28.

### Universal entries that do not apply here **[decided]**

Recorded so that a later sweep does not rediscover them. A service-shaped checklist against
desktop software leaves these with nothing to bind.

| Universal entry | Here |
|---|---|
| Per-environment settings, hosts and keys | No environments, hosts, keys or secrets. What survives is *no hard-coded absolute paths*; the port is an overridable default |
| Missing configuration fails loudly at start-up | **Inverted.** [stack.md](../stack.md) requires a missing file to fall back silently. Only an unreadable file is a failure |
| Configuration has a complete example file | **Replaced.** The application writes the file and nobody hand-edits it, so the documentation is derived from the struct |
| Rollback of a deploy | **Reinterpreted** as older releases staying downloadable and readable. There is no deploy to reverse |
| Retries are bounded; nothing retries a non-idempotent operation | Nothing initiates a call to a remote party. Source reconnection is passive listening, and write-beside-then-rename is idempotent |
| Correlation identifier per request | There are no requests. At most a per-launch token on each log line, which is not an **identifier** in the vocabulary's sense |
| Health check | Survives as truthful source status and the diagnostics report |
| Per-use billing bound | Nothing is billed. The analogues belong to [performance.md](performance.md) and to the log-bound entry above |
