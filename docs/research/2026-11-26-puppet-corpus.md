# How large is a real 2D vtuber puppet?

**Measured:** 2026-11-26 · **Feeds:** the reference and stress puppet figures in
[platform.md](../requirements/platform.md) · **Method:** direct measurement of freely and
openly downloadable rigs in two formats. Companion to
[2026-11-26-inochi-file-measurements.md](2026-11-26-inochi-file-measurements.md), which
measured the project owner's own two rigs; those are carried here for comparison and were not
re-measured.

## Verdict

**The reference puppet is roughly the smallest thing in the corpus, and the stress puppet is
roughly its middle.** On every structural dimension — nodes, parts, vertices, triangles,
parameters, bindings, masks — the pair is shifted about one step down: what `platform.md` calls
the reference is near the corpus minimum, and what it calls stress is near the corpus median.
Only the texture figures run the other way, and those are the ones the corpus is least able to
judge.

The cause is visible in the measurements. The two rigs the current figures were calibrated
against are the two smallest rigs available anywhere in this corpus on parts, vertices,
triangles and bindings. They were a sample of two from a beginner, and they were a low sample.

## The corpus

| Model | Format | Nodes | Drawn parts | Vertices | Triangles | Parameters | Bindings | Keypoints | Mask refs | Physics | Animations | Textures | Mpx | Resident |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Wanko | moc3 v1 | 83 | 34 | 397 | 447 | 25 | 85 | 126 | 0 | 4 | 12 | 1 | 1.05 | 4 MB |
| Mark | moc3 v3 | 75 | 30 | 1,703 | 2,526 | 21 | 61 | 62 | 6 | 1 | 6 | 1 | 4.19 | 16 MB |
| Kei (vowels) | moc3 v5 | 128 | 60 | 1,653 | 2,266 | 31 | 139 | 166 | 11 | 6 | 4 | 1 | 4.19 | 16 MB |
| Kei (basic) | moc3 v5 | 129 | 60 | 1,653 | 2,266 | 27 | 148 | 154 | 11 | 6 | 4 | 1 | 4.19 | 16 MB |
| Koharu | moc3 v1 | 189 | 91 | 1,984 | 2,605 | 52 | 220 | 151 | 0 | — | 3 | 1 | 4.19 | 16 MB |
| Hiyori | moc3 v3 | 262 | 134 | 2,822 | 3,426 | 70 | 334 | 212 | 8 | 11 | 10 | 2 | 8.39 | 32 MB |
| Haru | moc3 v1 | 200 | 84 | 3,248 | 4,696 | 42 | 221 | 126 | 10 | 4 | 27 | 2 | 8.39 | 32 MB |
| Rice | moc3 v3 | 346 | 178 | 4,119 | 5,313 | 96 | 291 | 571 | 19 | 9 | 4 | 2 | 8.39 | 32 MB |
| Natori | moc3 v1 | 388 | 176 | 5,227 | 6,952 | 96 | 406 | 287 | 10 | 11 | 8 | 1 | 4.19 | 16 MB |
| Mao | moc3 v5 | 472 | 262 | 6,020 | 7,943 | 132 | 497 | 934 | 65 | 16 | 8 | 1 | 4.19 | 16 MB |
| Ren | moc3 v6 | 399 | 198 | 7,843 | 11,041 | 73 | 465 | 422 | 16 | 16 | 3 | 1 | 4.19 | 16 MB |
| Aka | inx | 81 | 76 | 3,427 | 5,353 | 34 | 259 | 283 | 22 | 19 | 0 | 76 | 22.86 | 87 MB |
| Midori | inx | 106 | 81 | 6,038 | 9,800 | 35 | 416 | 293 | 24 | 20 | 0 | 81 | 20.52 | 78 MB |
| *yokisama* | *inp* | *35* | *20* | *771* | *1,158* | *15* | *72* | — | — | *5* | *2* | *1* | *67.1* | *256 MB* |
| *blob* | *inp* | *43* | *29* | *1,377* | *2,138* | *82* | *166* | — | *5* | — | *14* | *1* | *16.8* | *64 MB* |

The two italic rows are the owner's rigs, from the earlier measurement, for comparison only.

Sorted by vertices, since that is the dimension `platform.md` scales the other geometry figures
against. **Nodes** is the whole tree; **drawn parts** is the strictly comparable figure, because
the two formats build a tree very differently (see *Where the comparison is loose* below).

### Where each model and each number came from

| Models | Source | Licence | What the numbers came from |
|---|---|---|---|
| Haru, Hiyori, Mao, Mark, Natori, Ren, Rice, Wanko | `Live2D/CubismWebSamples`, `Samples/Resources/` | Live2D Free Material License Agreement; the repository is public and needs no account | `.moc3` count table; `.physics3.json`; `.motion3.json` files; PNG headers |
| Koharu | `Live2D/CubismUnityComponents`, `Assets/Live2D/Cubism/Samples/Models/` | as above | as above, no `.physics3.json` present |
| Kei (basic), Kei (vowels) | `Live2D/CubismWebMotionSyncComponents`, `Samples/Resources/` | as above | as above |
| Aka, Midori | `Inochi2D/example-models` (git-LFS, via the `media.githubusercontent.com` endpoint) | CC BY 4.0. Design and rigging by seagetch | the `.inx` JSON directly; TGA headers in the `TEX_SECT` payload |

Live2D's `.moc3` is a closed binary, but its layout is reverse-engineered and stable: a 64-byte
header, a section-offset table at byte 64 whose first slot points at a count table, and 23
count fields in a fixed order. The implementation this was read against is
`MahouTechnologies/moc3-rs` (`moc3-rs/src/data.rs`). **The parse is cross-checked**: the
parameter count read out of every `.moc3` matches the parameter list in that model's
`.cdi3.json` exactly, across moc3 versions 1 through 6, and the part counts match or differ by
the one root entry `.cdi3.json` adds.

Per-dimension mapping from the `.moc3` count table: *drawn parts* is `art_meshes`; *nodes* is
`parts + deformers + art_meshes`; *vertices* is `uvs / 2` and *triangles* `vertex_indices / 3`;
*keypoints* is `keys`; *mask refs* is `art_mesh_masks`. *Bindings* is the number of
(driven object, parameter) pairs, summed by reading each object's keyform binding and counting
the parameters in it. *Physics* is `PhysicsSettings` entries in `.physics3.json` — the pendulum
chains — and *animations* is `.motion3.json` files.

## How `platform.md` compares, dimension by dimension

| Dimension | Reference | Where it sits | Stress | Where it sits |
|---|---|---|---|---|
| Nodes | 40 | **below the corpus minimum** | 160 | below the median |
| Parts | 25 | **below the corpus minimum** | 100 | just above the median |
| Vertices | 1,200 | second-lowest | 4,800 | above the median, below the maximum — **about right** |
| Triangles | 2,000 | second-lowest | 8,000 | below the maximum — slightly low |
| Parameters | 30 | below the median | 120 | just below the maximum — **about right** |
| Bindings | 160 | below the median | 640 | **above the maximum** |
| Composites and masks | 5 | below the median | 20 | far below the maximum |
| Physics nodes | 6 | below the median | 24 | **above the maximum** — about right |
| Animations | 8 | **above the median** | 32 | **above the maximum** — generous |
| Source artwork | 27 Mpx | **above everything measured** | 108 Mpx | far above everything measured |
| GPU-resident texture bytes | 110 MB | **above everything measured** | 440 MB | far above everything measured |

Corpus minimum, median and maximum per dimension are read off the table above; it is sorted so
the geometry columns can be read directly, and the others are short enough to scan.

**The structural half is too small, consistently.** A reference puppet is supposed to be what a
user typically has, and this one is smaller than every rig a user could download. A stress
puppet is supposed to be the thing that must not fall over, and Mao and Ren already exceed it
on parts and on vertices — Ren on triangles too, Mao on parameters.
Setting the reference at the corpus median and the stress at or
above its maximum would move parts from 25/100 to roughly 85/260, vertices from 1,200/4,800 to
roughly 3,200/8,000, triangles from 2,000/8,000 to roughly 4,700/11,000, parameters from 30/120
to roughly 42/132, bindings from 160/640 to roughly 260/500, and mask-and-composite from 5/20
to roughly 11/65.

**Animations are the one structural dimension already generous**, and the `.motion3.json`
figures are a weak signal — a Cubism model ships whatever motions its demo needs, not what a
rigger authored.

**The texture figures are above the whole corpus, and the corpus cannot settle them.** Every
Cubism model here ships one or two 2048×2048 atlases, which is a distribution decision, not
evidence about authored artwork: the atlas size is an export setting, the SDK repositories ship
only the `.2048` variant, and the layered source lives in the `.cmo3` that is not in these
repositories. The inochi2d project files are the honest datapoint, because an `.inx` carries
each part's artwork unpacked — and those land at 20–23 Mpx and 78–87 MB resident, below the
current 27 Mpx / 110 MB reference but within reach of it. **The reference texture figures are
plausible and the stress figures are unsupported**; nothing measured here comes close to
108 Mpx.

There is one positive confirmation. Mao, the densest rig in the corpus by drawn parts, fits all
of them into a single 2048×2048 sheet. That is a commercial pipeline cropping each part to its
content before packing, and it is the same win the earlier measurement identified as available
and unclaimed in inochi2d's exporter.

## Where the comparison is loose

- **Node counts are not comparable across the two formats.** Cubism rigs almost everything
  through a hierarchy of warp and rotation deformers, so its tree is dominated by nodes that
  draw nothing; Aka has no mesh groups at all and Midori has a handful, with the same work done
  by per-part deformation bindings. *Drawn parts* is the column to compare on. The node row in
  `platform.md` should say which convention it counts, or the figure cannot be checked.
- **Bindings are counted more coarsely in Cubism.** One Live2D keyform set carries a part's
  deformation, opacity and draw order together, where inochi2d counts those as separate
  bindings against the same parameter. The Cubism binding column therefore undercounts relative
  to the inochi2d one, and the gap between the current reference figure and reality is if
  anything wider than the table shows.
- **Keypoints are not the same unit.** The inochi2d figure is the product of each parameter's
  axis lengths summed over parameters; the Cubism figure is stored key values. They are listed
  for scale, not for comparison.

## What could not be measured, and why

| Wanted | Status |
|---|---|
| Composites, in Cubism | Not a thing Cubism has. Its offscreen work is the clipping-mask buffer, so only the mask side of that row has a Cubism figure |
| Source artwork in pixels, in Cubism | Only the packed atlas ships. The layered source is in the `.cmo3`, which is not in the public repositories |
| Offscreen passes and shaded pixels per frame | Not derivable from a file. These are properties of a renderer running a puppet, and belong to the counter work `platform.md` already specifies |
| Per-part texture footprint, in Cubism | Would need the UV and atlas-region data decoded, not just the count table |
| Live2D's wider official sample catalogue — Zundamon, Miara, Chitose, Tsumiki Harugasa, Nito, Hatsune Miku, Epsilon, Shizuku, Hibiki, Izumi, Unity-chan, Tororo & Hijiki, Gantzert & Felixander, the simple model, Haru receptionist | Freely distributed, but behind a click-through consent to the Free Material License Agreement and the Sample Data Terms of Use, with no plain download URL. Not downloaded. These are the cheapest way to roughly double the corpus if the licence is read and accepted by a person |
| Commercially sold vtuber puppets — Booth, Nizima, VTube Studio Workshop | Account or payment gated without exception. Not downloaded |
| Free community models on itch.io and similar | Free to download but individually licensed, with terms that restrict modification and redistribution and that would need reading one at a time. Not pursued |
| `Inochi2D/inochi2d` `examples/ada-static.inx` | Downloaded and opened: a single textured quad, not a rig. Excluded rather than counted |

VTube Studio ships no sample models of its own; its documentation points users at the same
Live2D samples measured here. That is worth knowing — it means the SDK samples are not merely
developer fixtures, they are what a beginner vtuber actually loads first.

## How representative is this?

**Honestly: partially, and the gaps are known.**

- **The upper end is probably real.** Mao, Ren, Natori and Rice are Live2D's own flagship
  samples, rigged in-house to show the editor off. A commissioned vtuber rig is unlikely to be
  far above them structurally, and may well be below.
- **The upper end may also be inflated in a specific way.** A sample built to demonstrate
  features will use more deformers, more parameters and more mask work than a rig built to do a
  job. The Cubism samples skew polished, exactly as anticipated.
- **The lower end skews old rather than simple.** Wanko, Mark, Haru, Natori and Koharu are
  Cubism 3-era (moc3 v1 and v3); the heavy ones are v5 and v6. Some of the spread in this
  corpus is a decade of the format growing, not a range of rigger skill.
- **The inochi2d sample is two models by one rigger**, which is the same weakness as the
  measurement it is correcting, with a more experienced rigger. It is useful as a second format
  and a second skill level, not as a distribution.
- **Nothing here is a model someone paid for**, which is the population the project actually
  cares about and the one that is systematically unobtainable for free.
- **The texture figures should not be trusted in either direction** for the reasons above.

**What would change the conclusion:** accepting the Live2D sample licence by hand and measuring
the rest of that catalogue would roughly double the corpus and add the modern Cubism 5 models at
both ends. Obtaining one commissioned vtuber rig — the project owner may be able to ask a rigger
directly — would be worth more than all of it, because it is the only way to find out whether
the Live2D samples are the ceiling or the floor of commercial work.
