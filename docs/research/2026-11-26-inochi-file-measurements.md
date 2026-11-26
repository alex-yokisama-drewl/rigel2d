# What is actually inside an inochi2d model file?

**Measured:** 2026-11-26 · **Feeds:** whether a puppet is one file or a project plus an export, and the reference
puppet figures, both in `docs/requirements/` · **Method:** direct measurement of two real rigs
supplied by the user — `yokisama` (in daily use, physics, modest parameters) and `blob` (older,
many parameters, pseudo-randomised animations). Both in `.inp` and `.inx` form.

## Verdict

**The rig is nearly free; the artwork is the file.** And inochi2d's atlas packing does not
shrink a model — it inflates it, by between two and a half and four times in pixel area, with
the whole of that landing in GPU memory.

## The container

`TRNSRTS\0`, a big-endian 32-bit JSON length, the JSON, then a `TEX_SECT` payload of
length-prefixed TGA images. Confirmed against the user's working extraction script.

## Rig data versus artwork

| | Total | Rig (JSON) | Artwork |
|---|---|---|---|
| `yokisama.inp` | 6.36 MB | 0.17 MB (2.6%) | 6.19 MB |
| `yokisama.inx` | 4.91 MB | 0.17 MB (3.4%) | 4.74 MB |
| `blob.inp` | 2.80 MB | 1.10 MB (39.3%) | 1.70 MB |
| `blob.inx` | 4.49 MB | 1.10 MB (24.4%) | 3.39 MB |

**The JSON is byte-identical between the `.inx` and the `.inp` of the same model**, bar one key:
the project file carries `groups` and the export does not. Same nodes, meshes, parameters,
bindings, animations, physics. Whatever the project-versus-export separation is buying, it is
not a smaller or simpler rig.

`blob`'s rig is large in relative terms only because its artwork is small; in absolute terms a
megabyte of JSON buys many parameters and fourteen animations.

## The atlas inflates the model

Texture payloads, as stored and as they land in GPU memory:

| | Images | Source pixels | Stored | **GPU-resident** |
|---|---|---|---|---|
| `yokisama.inx` | 13 | 26.9 Mpx | 4.73 MB | **102.7 MB** |
| `yokisama.inp` | 1 × 8192² | 67.1 Mpx | 6.18 MB | **256.0 MB** |
| `blob.inx` | 1 × 2048² | 4.2 Mpx | 3.38 MB | **16.0 MB** |
| `blob.inp` | 1 × 4096² | 16.8 Mpx | 1.68 MB | **64.0 MB** |

Exporting `yokisama` packs 26.9 Mpx of artwork into an 8192×8192 sheet — 67.1 Mpx, two and a
half times the pixels, **256 MB resident where 103 MB would do**. `blob` fares worse by ratio:
four times the pixels, four times the memory. The stored file barely grows because the waste is
empty space and empty space compresses, so the cost is invisible until the model is loaded.

On the reference machine in `docs/requirements/platform.md` — integrated graphics sharing 8 GB
of system memory — a quarter of a gigabyte for one avatar's textures is a serious number, and
roughly three fifths of it is nothing.

## The source textures are padded too

`yokisama.inx` holds eight images at exactly 920×780 and four at 1024×1077. Their stored sizes
run from 0.03 MB to 1.16 MB at identical dimensions, so most are largely transparent: parts
exported on a shared canvas rather than cropped to their own content. Each still costs its full
2.74 MB or 4.21 MB resident.

So two inefficiencies compound. Each part is padded to a canvas it does not fill, and the atlas
then pads the padded parts. **Cropping to content before packing is most of the available win**,
and neither half requires anything clever.

## What this settles, and what it does not

**Settles:** carrying editable artwork alongside a model is not what makes a model large. On
these rigs the entire rig is under a megabyte, and the two file kinds differ by one JSON key and
by whether the images are packed. The size argument for a separate export is not supported by
the evidence — if anything the export is the larger file.

**Does not settle:** whether a rigger wants layered source artwork travelling to a client. That
is a question about delivery and trust, not bytes, and no measurement answers it.

**Also does not settle:** whether these two rigs are representative. Both are the user's own. A
rig from someone who works at 4K with many separate pieces would shift the artwork figures,
though not the conclusion that the rig data is small.

## Reference puppet, recalibrated

> **Superseded for calibration purposes, 2026-11-26.** The conclusion drawn here — that the
> draft figures ran high on geometry — was wrong, and wrong because these two rigs are not
> representative. A thirteen-rig corpus
> ([2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md)) found both of them to be the
> *smallest* models measured on parts, vertices, triangles and bindings. The draft figures ran
> low, not high. The measurements below are accurate; the inference from them was not.

Measured:

| | `yokisama` | `blob` |
|---|---|---|
| Nodes | 35 | 43 |
| Parts | 20 | 29 |
| Vertices | 771 | 1,377 |
| Triangles | 1,158 | 2,138 |
| Parameters | 15 | 82 |
| Bindings | 72 | 166 |
| Physics nodes | 5 | — |
| Composites | — | 5 |
| Animations | 2 | 14 |
