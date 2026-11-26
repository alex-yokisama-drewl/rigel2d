# How does the player deliver alpha-channel frames into OBS Studio on Linux (and how does that compare with Windows and macOS)?

**Researched:** 2026-11-26 (follow-up on Windows, macOS and a custom plugin added the same day) · **Feeds:** how the player delivers transparent frames to streaming software, in `docs/requirements/performing.md` · **Verdict:** An OBS plugin is a Linux-only wart today: stock OBS already receives alpha on macOS (Syphon Client) and on Windows (Game Capture, not Spout). On Linux every viable path needs a plugin. The best existing fit is a PipeWire dmabuf stream received by `obs-pwvideo`, with obs-vkcapture as the fallback. Writing our own plugin is a deliberate choice to make the story uniform, not a reflection of how things already are. Alpha is unverified end to end on every path.

## Verdict

**Revised after the Windows and macOS follow-up.** Linux is the only platform where stock OBS has no way to receive our frames with alpha. The earlier text of this report treated the plugin as unavoidable; that stays true for Linux and is false for Windows and macOS (see "Follow-up: Windows, macOS and a plugin of our own" below).

**"Without a plugin" cannot be met on Linux.** Every mechanism found that carries alpha needs an OBS-side component the user installs. OBS's built-in PipeWire sources are portal-based (screen/window capture and cameras), and their dmabuf import declares the texture as `GS_BGRX`, i.e. no alpha. `performing.md` said, before this finding, the output is native and plugin-free on every platform. That is achievable on macOS, achievable on Windows only through Game Capture (not Spout, which is a third-party plugin in OBS), and not achievable on Linux. The user must either reword the requirement to allow a plugin on Linux, or choose to ship our own plugin everywhere. That is a product decision, not a research question.

**Recommended primary for Linux if we do not write our own plugin: a PipeWire video stream of zero-copy dmabuf frames, received by `obs-pwvideo`.** The pattern is what libfunnel packages ("Spout2 / Syphon, but for Linux"). Reasons:

- It is not screen capture. The player publishes its own stream; no portal, no window or desktop capture, so the security constraint is met by construction.
- It needs no window at all. EGL and Vulkan integrations work headless, so "the render surface carries no interface" becomes a property of the architecture, not a discipline.
- It is zero-copy, and works the same under X11 and Wayland because PipeWire is the transport, not the display server.
- Alpha is a documented contract: premultiplied RGBA, with the format list negotiated. The consuming plugin's format table maps the alpha-bearing formats to alpha-bearing OBS texture formats, and its dmabuf import uses that mapping rather than hardcoding `GS_BGRX` as built-in OBS does.
- It lets the player pace frames to OBS's frame rate (libfunnel double-buffered mode, which its docs recommend for OBS), which fits the rule that the player never renders faster than output is consumed.

**Recommended fallback: obs-vkcapture.** Far more widely deployed and actively released, with an explicit "Allow Transparency" switch on its OBS source. It fits a second-window design well. Its costs are a launch wrapper or env var for the layer path, a swapchain that actually presents, and, if we skip the injection and speak its socket protocol directly, dependence on an unversioned internal protocol.

**If the user prefers one uniform install story, our own plugin is credible**, and the follow-up section sets out its realistic shape and recurring cost. The research does not recommend it over using stock OBS on Windows and macOS; it recommends the user choose knowing the cost.

**Risks that decide how much to trust this** (see "What would change this"): neither Linux alpha path has been observed working end to end; libfunnel has no tagged release and an explicitly unstable API; `obs-pwvideo` is a single-maintainer, best-effort project. Whatever is chosen, output should sit behind a narrow interface, and the first design task should be a spike that renders a half-transparent test frame through each candidate into a real OBS.

## Candidates

### 1. obs-vkcapture (Vulkan layer, OpenGL injection, or direct client)

- **How it works.** A Vulkan layer or GL hook in the target process copies the presented frame into a dmabuf-backed image and sends the dmabuf fds to the OBS plugin over an abstract unix socket (`SCM_RIGHTS`). OBS imports it with `gs_texture_create_from_dmabuf`. The OBS side is a "Game Capture" source.
- **Does alpha survive.** Probably yes, with caveats. The source has an `allow_transparency` setting, default off, that switches the OBS draw effect from opaque to default. The Vulkan export path keeps the swapchain's alpha format (`ARGB8888` / `ABGR8888`) and copies without touching alpha. The GL path requests an alpha channel and never forces opacity. Caveats: (a) alpha is only as good as the presented swapchain image, so the app must render alpha into it; (b) the plugin's draw effect assumes OBS's default blending, so the premultiplied/straight question needs a test; (c) user reports on Veadotube are mixed, including one saying transparency stayed a checkerboard under vkcapture. That is one anecdote, from an unknown setup, and could be the app rather than the plugin.
- **X11 / Wayland.** Both. It works at the Vulkan/GL layer, not through the display server. On X11 OBS must run with EGL (`OBS_USE_EGL=1` in the docs consulted); on NVIDIA, driver 515.43.04 or newer and `nvidia-drm.modeset=1`. A reported NVIDIA failure (`GetMemoryFdKHR` initialisation) exists in the issue tracker, so NVIDIA is the platform to test first.
- **Plugin.** Yes, but third-party and already widely packaged: distro packages, and Flatpak extensions for both the OBS plugin and the Vulkan layer.
- **Zero-copy.** Yes on the main path. A fallback path maps host memory and uploads each frame; it is a degraded mode the plugin requests when dmabuf import fails.
- **Maturity.** Active. Latest release v1.5.6 on 2026-05-28; the previous one on 2026-02-22. GPL-2.0. The Vulkan layer exists since 2021 (GamingOnLinux coverage dated 2021-03).
- **Beginner setup.** Install plugin; launch the player via `obs-gamecapture` or `OBS_VKCAPTURE=1`; in OBS add Game Capture and tick Allow Transparency. A launcher script or desktop entry from our side removes the wrapper step.
- **Variant: speak the protocol directly.** The wire protocol is small: fixed-size binary structs, an abstract socket name, dmabuf fds as ancillary data. A player could implement the client side itself, with no layer and no window. Costs: the protocol carries no version field, and the structs live in a header we would have to track. Viable but fragile; unconfirmed that the maintainer considers it stable.
- **Security.** Capture is opt-in per process (only the process with the layer or client sends frames). It is not window or desktop capture.

### 2. PipeWire dmabuf stream (libfunnel model) + obs-pwvideo

- **How it works.** The player creates a PipeWire video-source node whose buffers are dmabufs it renders into (libfunnel wraps this for EGL, Vulkan and raw GBM). `obs-pwvideo` is an OBS source that connects to arbitrary PipeWire video nodes directly, not through a portal, and imports the dmabufs.
- **Does alpha survive.** Probably yes; the strongest documented contract of any Linux candidate. libfunnel documents RGBA8888 as premultiplied alpha and says "Alpha is always premultiplied". The pwvideo format table maps `BGRA`/`RGBA` to alpha-bearing OBS formats, and its dmabuf import passes the mapped format. Built-in OBS passes `GS_BGRX` at the same call; the fork differs there. Both blend as premultiplied (`ONE, INVSRCALPHA`). Veadotube's own OBS documentation presents its PipeWire Video route as transparent. I could not confirm from the sources what Veadotube renders, so this is corroboration, not proof. Whether the OBS-side texture is truly sampled with alpha through the dmabuf path is unconfirmed by test: I read `formats.c` and `pipewire.c` but not the dmabuf format mapping header.
- **X11 / Wayland.** Both; PipeWire is the transport. EGL integration is documented as working with X11, Wayland, headless and raw DRM. The GLX path is supported only with a caveat. No compositor-specific behaviour found, because no compositor is in the path. NVIDIA proprietary requires `nvidia-drm.modeset=1` and, for OpenGL, a stricter sync mode (libfunnel docs).
- **Plugin.** Yes, `obs-pwvideo`: third-party, not in OBS upstream (I found no upstream effort for a generic non-portal PipeWire source). Packaged: Flathub extension (about 1,400 downloads a month), Fedora (review approved 2026-01-21, stable 2026-01-31 to 2026-02-01), AUR. Not confirmed for Debian/Ubuntu.
- **Zero-copy.** Yes, dmabuf end to end, with explicit sync described as preliminary.
- **Maturity.** Weakest of the candidates. Both projects are by one author and describe themselves as "maintained on a best-effort, personal basis"; the pwvideo maintenance note was added 2026-10-14, after the last functional release (0.2.4, 2026-04-08), and says the author no longer plans to invest in use cases outside their own. libfunnel has an empty releases list and says the API is unstable. libfunnel is MIT; pwvideo is GPL-2.0. I found no application confirmed to use libfunnel in production; Veadotube has its own PipeWire stream and I did not establish how it is built.
- **Beginner setup.** Install one OBS plugin; add a "PipeWire Video" source and pick the player's stream from a list. No launch wrapper. PipeWire is required, and Veadotube's docs say older distributions (Linux Mint 21 is the example) are too old.
- **Security.** Not capture. The stream is the player's own render target.

### 3. OBS built-in PipeWire Screen/Window Capture (xdg-desktop-portal ScreenCast)

- **Depends on the screencast portal and window/desktop capture. Excluded by the project constraint.** No variant avoids it: the source is portal-gated by design.
- Alpha is also doubtful: the built-in dmabuf import declares `GS_BGRX`, and the portal path delivers whatever the compositor chooses to emit. GNOME's mutter has an open request (gitlab issue 2153, title "Support RGBA capture") for the capture API to include window alpha. I could not retrieve its body or status.

### 4. OBS built-in PipeWire camera source (portal camera access)

- Not screen capture, so not excluded by the constraint: it asks for camera access, not screen access. The player would publish a PipeWire video-source node and OBS would list it as a camera.
- Alpha unconfirmed and doubtful: it reuses the same built-in PipeWire code (dmabuf declared `GS_BGRX`), and the source is still marked beta in logs seen. It requires the portal permission flow and gives no advantage over obs-pwvideo, which reads the same kind of node without a portal.
- Not recommended; no test evidence.

### 5. A custom OBS source plugin we write (shared memory or dmabuf)

- Works by construction, and is what the two plugins above already are. Costs are those the brief named: a second distributable, per-OBS-version builds, a second install step, plus owning the Flatpak extension path. See the follow-up section for the cross-platform shape and recurring cost.
- Shared-memory (CPU) variant needs a GPU readback of 1080p RGBA every frame: unacceptable against the frame budget in `docs/requirements/platform.md`.
- OBS discussion 7998 (2023) records a community suggestion of exactly this, a sender plus a plugin using `gs_texture_create_from_dmabuf`, and a statement that OBS itself has no generic route.

### 6. V4L2 loopback virtual camera

- OBS's Linux virtual camera is the other direction (OBS to apps). A player-to-OBS use would need a loopback device the player writes to and OBS reads as a V4L2 capture source. That needs a kernel module installed with root, a per-frame GPU readback (no zero-copy), and webcam-style pixel formats; I found no evidence that OBS's V4L2 source or the loopback path carries alpha. An OBS discussion from 2021-04-06 notes ARGB is absent from the virtual camera format options and has no replies.
- Not recommended.

### 7. NDI (DistroAV) and Beam

- NDI carries alpha when the format is BGRA, but user reports say only the default latency mode keeps it, and some report regressions. It is a network path with CPU-side frames, requires the proprietary NDI runtime (licence terms not checked), and also needs an OBS plugin. Beam (xObsBeam) supports alpha in lossless modes but lists Linux as occasionally tested.
- Neither is zero-copy. Not recommended, but NDI is the only option here that works across machines and across GPUs, which matters if two-PC setups become a requirement.

### 8. Spout/Syphon-equivalent efforts on Linux

- Spout2's OBS plugins are Windows-only; a Linux request (obs-spout2-plugin issue 83) contains an argument that dmabuf and PipeWire are the right substrate. libfunnel plus obs-pwvideo is the nearest thing to a Spout equivalent that exists. The libfunnel author is also working on a Spout2-to-PipeWire bridge for Wine, per a wine-devel thread (not read in full). I found no OBS-upstream initiative.
- Experimental dmabuf projects (obs-kmsgrab, dma-buf-texture-sharing) are screen capture or proof of concept and not candidates.

## Comparison (Linux candidates)

| | vkcapture | PipeWire + obs-pwvideo | Custom plugin | Built-in PipeWire capture | V4L2 loopback |
|---|---|---|---|---|---|
| Alpha, by source reading | Likely, with opt-in switch | Likely, documented premultiplied | By construction | Doubtful | Unknown |
| Alpha verified in practice | No (mixed anecdotes) | No (docs claim only) | n/a | No | No |
| X11 | Yes | Yes | Yes | Yes (portal) | Yes |
| Wayland | Yes | Yes | Yes | Yes (portal) | Yes |
| Needs window capture or portal | No | No | No | **Yes: excluded** | No |
| Needs OBS plugin | Third-party | Third-party | **Ours** | No | No |
| Zero-copy | Yes | Yes | Yes if dmabuf | Yes | **No** |
| Needs a window to exist | Layer path yes; direct client no | No | No | Yes | No |
| Maintenance risk | Lowest of these | Highest: one author, best-effort | Ours, permanently | OBS | Kernel module |
| Beginner setup | Plugin + launch wrapper | Plugin | Plugin | None | Root + kernel module |

## Follow-up: Windows, macOS and a plugin of our own

Added 2026-11-26. Question: is "install our OBS plugin" a Linux-only wart, or the uniform story? Current OBS at the time of writing is 33.0 (released 2026-10-23, official release notes).

### Windows: stock OBS and Spout

- **Spout is not in OBS.** The OBS source tree has no Spout plugin directory (the Windows capture code is `win-capture`, `win-dshow`, and so on; I read the `plugins/` listing on master, 2026-11-26). Spout support comes from the third-party `obs-spout2-plugin` (Off-World Live, GPL-2.0, Windows 64-bit only), installed via an `.exe` installer or zip. I found no evidence of an upstreaming effort and no OBS release note mentioning Spout through 33.0, so there is **no version in which it changed**: it has always been a plugin. The premise in `performing.md` that "Windows has Spout" is true of Spout the protocol, but receiving it in OBS needs a plugin. That is a contradiction with the requirement's "without a plugin" for the Spout route.
- **The no-plugin alpha route on Windows is Game Capture.** OBS's knowledge base lists Game Capture as available on Windows only, with an "Allow Transparency" option ("for games and 3D programs with transparency, sets the source to not show an opaque black background"). It hooks the target's DirectX or OpenGL swapchain, so the player would need to be capturable as a game, and the hook has an anti-cheat compatibility option on by default. This is the closest Windows analogue of obs-vkcapture, and it is built in. Veadotube's OBS docs also point Windows-style users to Game Capture with Allow Transparency for transparent backgrounds.
- **Does alpha survive in Windows Game Capture?** The documented option and the VTuber ecosystem's reliance on it say yes. **I did not read the hook source** (`graphics-hook`) and I did not test. There is no equivalent of the Linux `GS_BGRX` defect that I found, but I have not looked for one, so it is unconfirmed rather than ruled out.
- **Does alpha survive through the Spout plugin?** Unconfirmed. The plugin README says nothing about alpha or transparency in the text I could retrieve; its raw README returned 404 on one fetch.
- **Plugin cadence, as evidence for the cost.** `obs-spout2-plugin` releases track OBS: rebuilds for OBS 27, 28, 29, 31, 32.1.2 and 33.0 (releases dated 2021-08-17 to 2026-10-29). Version 1.13.0 (2026-10-26) added code signing of the DLLs and installer "to address load failures under Application Control policies". Version 1.15.0 (2026-10-29) says it does not work with OBS 32 or earlier. Also: sender and OBS must be on the same GPU (error 80070057 otherwise), and OBS must use Direct3D 11 for the capture source.

### macOS: stock OBS and Syphon

- **Syphon Client is built into OBS.** The OBS tree has a `mac-syphon` plugin directory; the source is added as "Syphon Client". It imports an `IOSurface` with `gs_texture_create_from_iosurface`. No plugin install for the user.
- **Alpha: yes, opt-in.** The source has an `allow_transparency` setting; with it off, the opaque effect is used, with it on the default effect. I found no premultiplied-alpha handling in the file, so which alpha convention a sender should use is unconfirmed. No built-in `GS_BGRX`-style defect found, but the IOSurface pixel format is not specified in OBS's code either; see the next point.
- **Recent fragility.** OBS 32.0 and 32.0.1 broke Syphon sources that sent an IOSurface with pixel format 0 (they showed black or empty; 31.1.2 worked). An OBS admin confirmed it on 2025-10-04 and pointed to PR 12688, which treats pixel format 0 as BGRA32 and changes the opaque effect. The thread names no fixed release; whether 33.0 contains it is **unconfirmed**. Lesson for us: the built-in path is not maintenance-free from our side either.
- **Support tier.** `platform.md` already treats macOS as best-effort with no test hardware, so any macOS alpha claim beyond "built-in source exists and takes IOSurface" is unverifiable by us.

### What a plugin of our own would look like

- **One codebase is realistic for the plugin; the transport is per platform.** libobs exposes a texture-import call per platform: `gs_texture_open_shared` and `gs_texture_open_nt_shared` on Windows (shared D3D handle), `gs_texture_create_from_iosurface` on macOS, `gs_texture_create_from_dmabuf` on Linux (confirmed in `libobs/graphics/graphics.h`, master, 2026-11-26). So a single source plugin would have a thin per-platform import layer and a shared source, properties and render path. The **sender side** (the player exporting the texture) is a separate per-platform piece, as it is for Spout, Syphon and libfunnel; one plugin does not remove that work. I am not choosing a language or graphics API.
- **A one-codebase plugin has precedent in the template, not in a comparable plugin.** `obs-plugintemplate` (official) builds for Windows, macOS and Ubuntu from one repository with GitHub Actions and, on tagging, produces installer packages as release artifacts; its README covers macOS code signing and notarisation. The two references I was asked to look at are each single-platform: obs-vkcapture is Linux-only, and the Spout plugin is Windows-only. I did not find a cross-platform texture-sharing plugin to point to, so "one plugin, one transport per platform" is plausible from the API but has no existing example I could study.
- **What OBS commits plugins to.**
  - OBS will no longer load plugins built for a newer OBS release (stated in the 32.0 release notes). So a plugin built against 33 does not load in 32, and each user's OBS version dictates which build of ours they need. The Spout plugin shows this in practice: its current releases require OBS 33.
  - OBS publishes no ABI-stability promise that I found. The OBS plugin guide does not discuss ABI; it says a plugin may stop working for being unavailable on the platform, built for the wrong architecture, or "not updated for your OBS Studio version". Breaking changes are listed per release: OBS 28 required plugins that link Qt to be rebuilt against Qt 6 (forum and OBS discussion sources), and 33.0 lists a breaking libnsgif change for plugins using `gs_image_file`, the monitoring-type deprecation, and canvas API changes, with no blanket "rebuild all" statement. Releases come a few times a year, and the Spout plugin's own history shows a rebuild or retarget roughly per OBS major version. **I could not establish a precise rate of ABI breaks**; the best evidence is the plugin release table above and the unofficial claim in a forum answer that each release usually requires a recompile.
  - Platform packaging obligations, from the evidence: Windows code signing and an installer (the Spout plugin added signing in 2026 after Application Control load failures); macOS signing and notarisation (template README); Linux native and Flatpak extension packaging, where the Flatpak extension point gained versioning in 33.0.
- **Distribution and discovery today.**
  - Install locations per the OBS plugin guide: Windows `C:\ProgramData\obs-studio\plugins` with a required `bin\64bit` and `data` layout (the older Program Files location is Legacy); macOS `~/Library/Application Support/obs-studio/plugins` as `.plugin` bundles; Linux `~/.config/obs-studio/plugins`, or Flatpak extensions.
  - OBS 33.0 introduces a new third-party plugin layout on Windows and keeps the old locations loading only until 34.0. Plugin authors have to re-package for it; the Spout plugin did so within days of 33.0.
  - **In-app discovery does not exist yet.** The 32.0 plugin manager lists installed plugins and enables or disables them; "Browse" and "Update" were reported greyed out (press coverage; OBS's own release notes say only "added a basic plugin manager"). So a beginner downloads and runs an installer or copies files by hand, per platform. This claim is secondary-sourced and should be checked against current OBS before relying on it.

### Is a plugin a Linux-only wart or the uniform story?

| | Stock OBS, no plugin, alpha path | Our-plugin install needed? | Evidence of alpha |
|---|---|---|---|
| Windows | Game Capture + Allow Transparency (hook into the player's swapchain). Spout needs a third-party plugin | Not for Game Capture; yes for Spout | Documented option; hook source and a test not done |
| macOS | Syphon Client + Allow Transparency, built in | No | Documented option and source setting; premultiplied convention unconfirmed; 32.0.x regression noted |
| Linux | None | Yes: third-party today, or ours | Source reading only on both candidate plugins |

**Answer: a Linux-only wart today.** On Windows and macOS OBS already ships a transparent-capable path that needs no plugin. The cases where it becomes uniform are by our choice: using our own plugin on Windows and macOS instead of Game Capture and Syphon Client would give one install instruction, one source type, and one behaviour to document, and would free us from the `GS_BGRX`-style defects and regressions (the macOS 32.0 Syphon breakage) in OBS's built-in sources. Its price is permanent: signed and notarised builds per platform, a rebuild or retarget per OBS major, install steps for a beginner audience that currently has none on Windows (via Game Capture) and macOS, and no in-app install flow to lean on. Note too that Game Capture on Windows requires the player to be hookable and its anti-cheat compatibility hook is on by default, so "hook into the player" is not without caveats there either.

My reading, flagged as judgement not evidence: uniformity alone does not justify our own plugin on Windows and macOS, because it adds an install step where there is none. It does help the Linux case: the choice there is between depending on a single-maintainer third-party plugin and owning one, and the Linux plugin would be the same code and release machinery we could later extend to Windows and macOS. The decision therefore sits with the user as: (a) plugin on Linux only, third-party first, or (b) own plugin on Linux first, built so it can extend to other platforms, or (c) own plugin on every platform from the start.

## Evidence

Primary sources read directly (all accessed 2026-11-26 unless stated):

- obs-vkcapture repository, `src/vkcapture.c`, `src/vklayer.c`, `src/glinject.c`, `src/capture.c`, `src/capture.h`, raw files from the master branch: https://github.com/nowrep/obs-vkcapture . Basis for: `allow_transparency` setting, DRM format tables, ARGB export, protocol structs and abstract socket.
- obs-vkcapture releases via GitHub API: https://api.github.com/repos/nowrep/obs-vkcapture/releases . Release dates (v1.5.6 on 2026-05-28).
- OBS Studio `plugins/linux-pipewire/pipewire.c`, `formats.c`, `camera-portal.c` (master): https://github.com/obsproject/obs-studio/tree/master/plugins/linux-pipewire . Basis for: `GS_BGRX` passed at dmabuf import, premultiplied-blend assumption, camera source portal-only.
- OBS Studio `plugins/` directory listing (master): https://github.com/obsproject/obs-studio/tree/master/plugins . Basis for: `mac-syphon` present, no Spout plugin.
- OBS Studio `plugins/mac-syphon/syphon.m` (master): https://raw.githubusercontent.com/obsproject/obs-studio/master/plugins/mac-syphon/syphon.m . Basis for: IOSurface import, `allow_transparency`, effect selection.
- OBS Studio `libobs/graphics/graphics.h` (master): https://raw.githubusercontent.com/obsproject/obs-studio/master/libobs/graphics/graphics.h . Basis for: per-platform texture import calls.
- OBS Studio 33.0 release notes (2026-10-23): https://obsproject.com/blog/obs-studio-33-0-release-notes . Basis for: plugin loading overhaul, API changes, no Spout/Syphon changes listed.
- OBS Studio 32.0 release notes: https://obsproject.com/blog/obs-studio-32-0-release-notes . Basis for: basic plugin manager, no loading of plugins built for a newer OBS.
- OBS Knowledge Base, Game Capture source: https://obsproject.com/kb/game-capture-source . Basis for: Allow Transparency, Windows-only availability.
- OBS Knowledge Base, plugins guide: https://obsproject.com/kb/plugins-guide . Basis for: install locations, no in-app manager mention, no ABI discussion.
- obs-plugintemplate: https://github.com/obsproject/obs-plugintemplate . Basis for: three-platform CI and packaging, notarisation notes.
- obs-spout2-plugin repository and releases (API): https://github.com/Off-World-Live/obs-spout2-plugin , https://api.github.com/repos/Off-World-Live/obs-spout2-plugin/releases . Basis for: Windows-only, install layout, rebuild cadence, signing, OBS 33 requirement.
- OBS forum, Syphon broken in 32.0 (thread dated 2025-09-29 to 2025-10-04): https://obsproject.com/forum/threads/syphon-client-broken-in-32-0.190817/ . Basis for: 32.0.x regression and PR 12688.
- obs-pwvideo repository and source: https://github.com/hoshinolina/obs-pwvideo ; commit dates via https://api.github.com/repos/hoshinolina/obs-pwvideo/commits . Basis for: format mapping, dmabuf import call, maintenance statement, last commits 2026-10-14 and 2026-04-08.
- libfunnel documentation: https://libfunnel.readthedocs.io/en/latest/overview.html and https://libfunnel.readthedocs.io/en/latest/colorformats.html ; repository https://github.com/hoshinolina/libfunnel . Basis for: premultiplied contract, EGL/Vulkan/headless, OBS double-buffered advice, instability and no releases.
- Veadotube OBS usage docs: https://veado.tube/docs/usage/obs/ . Basis for: PipeWire Video plugin (obs-pwvideo) needed on OBS, vkcapture needed for Game Capture on Linux, Allow Transparency guidance.
- Fedora packaging review, dates of pwvideo availability: https://bugzilla.redhat.com/show_bug.cgi?id=2430668 .
- Flathub listing: https://flathub.org/en/apps/com.obsproject.Studio.Plugin.OBSPWVideo (version 0.2.4, monthly downloads as shown at fetch time).
- OBS discussion on PipeWire and dmabuf: https://github.com/obsproject/obs-studio/discussions/7998 (comments dated 2023 to 2024).
- OBS discussion on ARGB virtual camera: https://github.com/obsproject/obs-studio/discussions/4476 (2021-04-06).

Secondary or anecdotal, treated as such:

- Veadotube users on itch.io, vkcapture transparency reports: https://itch.io/post/5943584 , https://itch.io/post/9893718 .
- Linux vtubing guides recommending chroma key and Window Capture (Xcomposite): https://vtubing-on-linux.github.io/linux-vtubing-guide/docs/streaming/obs.html . A search summary attributed to another guide that VNyan/VSeeFace users enable Allow Transparency via `obs-gamecapture`; I could not open that page (404), so it is unconfirmed.
- Mutter RGBA capture request: https://gitlab.gnome.org/GNOME/mutter/-/issues/2153 (page not retrievable; title and gist from search results only).
- NDI/Beam alpha behaviour: OBS forum DistroAV thread and xObsBeam repository, via search results only.
- OBS 32.0 plugin manager scope (list, enable/disable; Browse and Update greyed out): press coverage, e.g. https://www.omgubuntu.co.uk/2025/09/obs-studio-32-0-release-brings-new-plugin-manager-nvidia-rtx-effects , via search results only.
- Plugin ABI churn: OBS discussion on the Qt 6 move https://github.com/obsproject/obs-studio/discussions/6481 , and an unofficial forum comment that new releases usually require recompiles.

## What would change this

- **A failed alpha spike on either Linux path.** If obs-pwvideo's dmabuf path samples the texture as opaque, the primary recommendation reverses to vkcapture or a patched fork, and the case for our own plugin strengthens. If vkcapture's effect mishandles premultiplied alpha, the player must render straight alpha for that backend only.
- **Contradiction to resolve.** A search-engine summary of Veadotube's docs said the PipeWire route's transparency was not stated; a direct fetch of the same page returned the phrase "perfectly transparent without any weird borders" and a "Transparency" instruction tied to Game Capture, with the surrounding sentences truncated, so I could not tell which method that phrase belongs to. Read the page by hand before leaning on it.
- **Contradiction in the plugins' alpha handling.** Built-in OBS passes `GS_BGRX` at its Linux dmabuf import; pwvideo passes a looked-up format. Unconfirmed whether the lookup yields an alpha format for dmabuf in practice, because I did not read its `formats.h`.
- **Windows alpha through Game Capture and through the Spout plugin** is documented-by-option only. Read `graphics-hook` and the Spout plugin source, or test, before asserting it in a requirement.
- **macOS premultiplied convention** for Syphon Client is unknown, and whether the 32.0.x Syphon fix shipped in 33.0 is unconfirmed.
- **Maintenance of the Linux primary path.** obs-pwvideo and libfunnel are best-effort and personal. Re-check release activity before design lock, and again before each release. Neither was checked against OBS 33.0 (released 2026-10-23); compatibility with the new plugin layout and rebuild requirement is unconfirmed for obs-vkcapture and obs-pwvideo. If either is abandoned, the own-plugin option becomes the answer on Linux.
- **OBS upstream.** If OBS adopts a generic PipeWire source, fixes the `GS_BGRX` dmabuf declaration, or ships in-app plugin browse and install, the cost of a plugin of our own drops. Nothing found suggests this is planned; re-check on each OBS major release, especially 34.0, when the legacy Windows plugin locations stop loading.
- **ABI break frequency** is not established; if it matters to the decision, read the libobs changelog across several releases and count what actually broke third-party source plugins.
- **Not examined:** Flatpak/Snap OBS sandbox interplay for vkcapture's abstract socket; Debian and Ubuntu packaging of obs-pwvideo; GPU-vendor behaviour of cross-process dmabuf under nested compositors or Gamescope; PipeWire version floor on older LTS distributions; Wayland frame-callback throttling for a vkcapture layer fed by a hidden or occluded window (a risk by inference, not observed); NDI runtime licensing; Windows Game Capture behaviour for a non-game process and for anti-cheat interplay.
- **Stale after:** roughly 2027-03, or OBS 34.0, whichever is first.

## Rejected options

- **xdg-desktop-portal ScreenCast / OBS Screen Capture (PipeWire) / Window Capture (Xcomposite).** Excluded by the security constraint: they capture a window or the desktop. No variant removes the capability. Also drops or leaves alpha doubtful.
- **OBS built-in PipeWire camera source.** Avoids screen capture but still goes through a portal, is beta, shares the built-in `GS_BGRX` dmabuf declaration, and offers nothing obs-pwvideo does not.
- **V4L2 loopback.** Needs root and a kernel module, forces GPU readback, no evidence of alpha.
- **Shared-memory plugin.** Readback every frame breaks the frame budget.
- **NDI and Beam.** Network and CPU-side; alpha behaviour conditional; extra runtime or plugin. Kept only as a note for a possible two-PC requirement.
- **Chroma key in OBS.** Already rejected by the requirement.
- **Spout as the Windows no-plugin route.** Rejected as a route to "no plugin": OBS has no built-in Spout receiver. Spout remains usable with the third-party plugin.
- **Writing our own OBS plugin now, on research grounds alone.** Not rejected on technical grounds; the choice depends on the user's answer to the uniform-install question above and on the alpha spike.
