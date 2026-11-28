# How many tracks does an animation drive, and how many keyframes does a track hold?

**Measured:** 2026-11-28 · **Feeds:** the two missing animation rows in the workload table in
[platform.md](../requirements/platform.md), which that document names as the gap that leaves
animation cost unbounded · **Method:** parsed every `.motion3.json` in the Cubism half of the
corpus recorded in [2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md). Completes that
measurement, which counted animation *files* and never opened them.

## Verdict

**Two rows, with the table's own convention applied — reference at the corpus median, stress
above the largest rig measured:**

| | Reference | Stress | |
|---|---|---|---|
| Tracks per animation | 54 | 200 | Every animated curve, whatever it drives. Reference is the median of each rig's own median; see *Which median* |
| Keyframes per track | 3 | 100 | **Heavily skewed — the median is not the shape.** A generator using a flat 3 produces about half the keyframe data a real animation carries. See *The distribution is the finding* |
| Keyframes per animation | 284 | 1000 | Added on review, because the two rows above must not be multiplied — see the first caution below. This is the row a load limit binds |

**Two cautions that matter more than the figures.**

*The two per-track rows do not multiply, which is why there is a third row.* Reference tracks times reference keyframes gives far less than
the corpus median of total keyframes per animation (both in *Corpus aggregates* below), because
most tracks are near the floor while a few carry an order of magnitude more. The generator should
draw keyframes per track from the distribution in *The distribution is the finding*, and the
load limit should bound total keyframes per animation directly rather than deriving it from the
product.

*Per-frame animation cost is the track row, not the keyframe row.* Playing an animation costs
one curve evaluation per playing track; the stress figure sits well below the binding count the
same table already budgets per frame. Keyframes are storage and the cost of finding the active
segment, not per-frame work, which is why the two rows carry different kinds of limit.

## Independently re-measured

The encoding below is the part that could corrupt a load limit silently, so the whole
measurement was re-run from scratch against the 78 `.motion3.json` files of
`CubismWebSamples`, by a second reader working from the raw files rather than from this
document. It reproduced the derived curve, segment and point counts with **no disagreement
against `Meta` on any file**, and reproduced the keyframe median, mean and maximum, the
share of tracks at or below three keyframes, and the per-rig track medians.

The one thing it did not reproduce was the reference figure, and that is what found the Kei
double-count recorded under *Which median*.

## The segment encoding

A curve's `Segments` array is flat and self-describing. It opens with an initial point — two
numbers, time and value — and is then a sequence of blocks, each opening with a segment-type
identifier:

| Identifier | Segment | Numbers after the identifier | Points the block contributes |
|---|---|---|---|
| 0 | Linear | 2 | 1 — the endpoint |
| 1 | Bézier | 6 | 3 — two control points, then the endpoint |
| 2 | Stepped | 2 | 1 — the endpoint |
| 3 | Inverse stepped | 2 | 1 — the endpoint |

Identifiers 0, 1 and 2 occur in this corpus; 3 does not. `Meta.AreBeziersRestricted` varies
across the corpus and **does not change the encoding** — a restricted Bézier still stores all
three of its points.

**A keyframe is a point the curve passes through: the initial point plus one endpoint per
segment.** Bézier control points are tangent handles, not keyframes, so

```
keyframes per track = segment count + 1
```

and the point count that `Meta` reports is *not* a keyframe count — it over-counts every Bézier
by two. This is the factor the brief warned about: taking `TotalPointCount / CurveCount` as
keyframes per track would overstate the reference row by roughly half again.

### Cross-check against `Meta`

Every file was parsed without consulting `Meta`, and the derived curve count, segment count and
point count were then compared against `Meta.CurveCount`, `Meta.TotalSegmentCount` and
`Meta.TotalPointCount`. **All three agree exactly on every file in the corpus** — no
discrepancy to explain, and the walk consumes each `Segments` array to its exact end, which
independently confirms the block lengths above. `Meta` is trustworthy here; it simply does not
carry the figure wanted.

## The distribution

### Per rig

| Rig | Animations | Tracks per animation: min | median | max | Curves targeting a parameter | …part opacity | …the model | Keyframes per track: median | max | Keyframes in all its animations |
|---|---|---|---|---|---|---|---|---|---|---|
| Haru | 27 | 63 | 63 | 63 | 1134 | 540 | 27 | 3 | 21 | 8422 |
| Hiyori | 10 | 31 | 33 | 35 | 308 | 20 | 0 | 5 | 20 | 1930 |
| Kei (basic) | 4 | 6 | 6 | 6 | 24 | 0 | 0 | 2 | 26 | 224 |
| Kei (vowels) | 4 | 9 | 9 | 9 | 36 | 0 | 0 | 2 | 26 | 248 |
| Mao | 8 | 136 | 136 | 136 | 1056 | 32 | 0 | 2 | 35 | 3889 |
| Mark | 6 | 37 | 37 | 38 | 114 | 97 | 12 | 4 | 33 | 1493 |
| Natori | 8 | 96 | 99 | 101 | 724 | 64 | 0 | 3 | 11 | 2729 |
| Ren | 3 | 68 | 68 | 68 | 204 | 0 | 0 | 4 | 67 | 1565 |
| Rice | 4 | 54 | 54 | 54 | 216 | 0 | 0 | 2.5 | 18 | 1100 |
| Wanko | 12 | 22 | 22 | 22 | 264 | 0 | 0 | 3 | 19 | 1276 |

Rig names and sources follow [2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md).
Koharu is absent because it lives in `CubismUnityComponents`, which was outside this pass; Aka
and Midori are absent for the reason under *Where the inochi2d half went*.

**Tracks per animation is a property of the rig, not of the animation.** Read the min, median
and max columns together: most rigs ship animations that all drive the same curve set, because
a Cubism motion is exported against the model's parameter list rather than authored per clip.
Hiyori, Mark and Natori vary slightly; the rest do not vary at all.

### Corpus aggregates

| Quantity | Population | Min | Median | Mean | Max |
|---|---|---|---|---|---|
| Tracks per animation | every animation | 6 | 63 | 56.7 | 136 |
| Tracks per animation | one datapoint per rig, its own median | 6 | 45.5 | — | 136 |
| Keyframes per track | every track | 2 | 3 | 4.70 | 67 |
| Keyframes per track | one datapoint per rig, its own median | 2 | 3 | — | 5 |
| Keyframes per animation | every animation | 56 | 247 | — | 762 |

Percentiles of keyframes per track, over every track: p75 = 6, p90 = 9, p95 = 12, p99 = 21.

### Which median

The two rows for tracks per animation disagree, and the gap is Haru: it ships more animations
than any other rig, all with an identical curve count, and that count is what sets the
per-animation median. Weighting by animation lets one rig's demo content decide the figure.

**The reference row takes the per-rig median**, because every other row in the workload table is
one datapoint per rig and because the per-animation figure is an artefact of how many clips a
sample happens to ship. A reader who prefers the per-animation median has it in the table above.

**Corrected on review from 45.5 to 54: Kei is one rig, not two.** *Kei (basic)* and *Kei
(vowels)* are the same character, and the per-rig table above shows them holding the same four
recordings — the durations match exactly, 14.833 / 17.867 / 17.733 / 14.4 — exported against two
different parameter configurations. Counting them as two of ten datapoints let one motion-sync
demo, at 6 and 9 tracks the two smallest figures in the corpus, pull the median down by eight.
Counted once, the population is nine rigs and the median is 54.

The alternative considered and rejected was dropping Kei from this row altogether, on the
grounds that baked lip-sync clips over a handful of mouth parameters are not authored character
animation and that Live drives those parameters from a tracker rather than from a stored
animation. That reasoning is sound but the remedy is too strong: the double-count is a
measurement error and fixing it is not a judgement about which rigs count, whereas exclusion
is. Dropping Kei would give 58.5. **The spread between the three candidate figures — 45.5, 54,
58.5 — is itself the finding**: this reference row rests on nine datapoints and is the softest
in the table after the texture rows.

### The distribution is the finding

Keyframes per track, over every track in the corpus:

| Keyframes in the track | Tracks | Share |
|---|---|---|
| 2 | 1519 | 31.2% |
| 3 | 1459 | 29.9% |
| 4–5 | 520 | 10.7% |
| 6–9 | 900 | 18.5% |
| 10–19 | 417 | 8.6% |
| 20–39 | 56 | 1.1% |
| 40 and above | 1 | 0.0% |

A two-keyframe track is a curve that holds one value and moves to another, or holds one value
for the whole clip; the table shows how much of a real animation is exactly that. The long tail
is where the work is, and it is body motion rather than anything exotic: the longest track in the
corpus is Ren's `ParamNoise`, and the next are Mao's `ParamStrengthenLight` and Mark's
`ParamBodyAngleY`, `ParamArmR`, `ParamArmL` and `ParamAngleY`.

**Consequences for the generator and the limit.** Emitting every track with the reference
keyframe count produces an animation far below the corpus median of total keyframes, so the
generated workload would be unrepresentative in the direction that matters — too little data,
evaluated too cheaply. Draw from the distribution above instead, and state the load limit
against total keyframes per animation, which this corpus bounds directly.

### What a curve targets, and whether it is a track

A Cubism curve carries a `Target`, and only one of the three values it takes is a parameter in
this project's sense ([vocabulary.md](../vocabulary.md)):

| `Target` | What it drives | Is it a track here? |
|---|---|---|
| `Parameter` | One of the model's parameters | Yes, directly |
| `PartOpacity` | The opacity of one Cubism part | Not as the term is defined — we would reach a part's opacity through a parameter and a binding |
| `Model` | A model-wide setting such as eye-blink or lip-sync enablement | No. It has no referent in our format |

**The rows above count every curve regardless of target**, because the rows feed a cost budget
and all three are per-frame curve evaluations whatever they are wired to. The per-rig table
breaks the three out so a reader who wants the strict reading can take the parameter column
instead; doing so would lower the reference by roughly a sixth and leave the maximum, Mao, almost
unchanged.

### Per animation

Every animation measured, so the medians and maxima above can be re-derived rather than trusted.

|---|---|---|---|---|---|---|
| Haru | haru_g_idle | 10.00 | 63 | 359 | 3 | 18 |
| Haru | haru_g_m01 | 2.90 | 63 | 206 | 3 | 9 |
| Haru | haru_g_m02 | 2.03 | 63 | 209 | 3 | 7 |
| Haru | haru_g_m03 | 4.63 | 63 | 284 | 3 | 15 |
| Haru | haru_g_m04 | 5.30 | 63 | 391 | 5 | 19 |
| Haru | haru_g_m05 | 2.03 | 63 | 242 | 3 | 9 |
| Haru | haru_g_m06 | 4.53 | 63 | 256 | 3 | 12 |
| Haru | haru_g_m07 | 3.93 | 63 | 250 | 3 | 10 |
| Haru | haru_g_m08 | 4.60 | 63 | 262 | 3 | 12 |
| Haru | haru_g_m09 | 4.03 | 63 | 432 | 8 | 10 |
| Haru | haru_g_m10 | 5.53 | 63 | 577 | 9 | 21 |
| Haru | haru_g_m11 | 3.43 | 63 | 244 | 3 | 10 |
| Haru | haru_g_m12 | 4.93 | 63 | 391 | 6 | 10 |
| Haru | haru_g_m13 | 2.53 | 63 | 198 | 3 | 9 |
| Haru | haru_g_m14 | 3.03 | 63 | 347 | 6 | 12 |
| Haru | haru_g_m15 | 5.33 | 63 | 230 | 3 | 10 |
| Haru | haru_g_m16 | 4.00 | 63 | 310 | 5 | 8 |
| Haru | haru_g_m17 | 4.50 | 63 | 270 | 4 | 8 |
| Haru | haru_g_m18 | 3.20 | 63 | 228 | 3 | 7 |
| Haru | haru_g_m19 | 8.00 | 63 | 194 | 3 | 11 |
| Haru | haru_g_m20 | 6.03 | 63 | 390 | 6 | 17 |
| Haru | haru_g_m21 | 5.00 | 63 | 415 | 6 | 16 |
| Haru | haru_g_m22 | 5.03 | 63 | 471 | 8 | 13 |
| Haru | haru_g_m23 | 4.00 | 63 | 282 | 5 | 8 |
| Haru | haru_g_m24 | 3.40 | 63 | 301 | 5 | 8 |
| Haru | haru_g_m25 | 4.03 | 63 | 294 | 3 | 12 |
| Haru | haru_g_m26 | 4.97 | 63 | 389 | 6 | 17 |
| Hiyori | Hiyori_m01 | 4.70 | 31 | 166 | 2 | 13 |
| Hiyori | Hiyori_m02 | 5.93 | 33 | 223 | 6 | 15 |
| Hiyori | Hiyori_m03 | 4.20 | 33 | 202 | 3 | 13 |
| Hiyori | Hiyori_m04 | 4.43 | 31 | 137 | 2 | 11 |
| Hiyori | Hiyori_m05 | 8.57 | 32 | 250 | 7 | 19 |
| Hiyori | Hiyori_m06 | 5.37 | 35 | 297 | 9 | 20 |
| Hiyori | Hiyori_m07 | 1.90 | 32 | 153 | 5 | 11 |
| Hiyori | Hiyori_m08 | 2.10 | 35 | 168 | 5 | 12 |
| Hiyori | Hiyori_m09 | 1.60 | 33 | 183 | 5 | 10 |
| Hiyori | Hiyori_m10 | 4.17 | 33 | 151 | 4 | 11 |
| Kei_basic | 01_kei_en | 14.83 | 6 | 52 | 2 | 22 |
| Kei_basic | 01_kei_jp | 17.87 | 6 | 60 | 2 | 26 |
| Kei_basic | 01_kei_ko | 17.73 | 6 | 60 | 2 | 26 |
| Kei_basic | 01_kei_zh | 14.40 | 6 | 52 | 2 | 22 |
| Kei_vowels | 01_kei_en | 14.83 | 9 | 58 | 2 | 22 |
| Kei_vowels | 01_kei_jp | 17.87 | 9 | 66 | 2 | 26 |
| Kei_vowels | 01_kei_ko | 17.73 | 9 | 66 | 2 | 26 |
| Kei_vowels | 01_kei_zh | 14.40 | 9 | 58 | 2 | 22 |
| Mao | mtn_01 | 5.57 | 136 | 349 | 2 | 10 |
| Mao | mtn_02 | 3.47 | 136 | 389 | 2 | 16 |
| Mao | mtn_03 | 4.40 | 136 | 371 | 2 | 12 |
| Mao | mtn_04 | 4.20 | 136 | 436 | 2 | 18 |
| Mao | sample_01 | 5.57 | 136 | 368 | 2 | 10 |
| Mao | special_01 | 7.80 | 136 | 549 | 2 | 18 |
| Mao | special_02 | 9.37 | 136 | 672 | 2 | 29 |
| Mao | special_03 | 9.23 | 136 | 755 | 2 | 35 |
| Mark | mark_m01 | 10.40 | 37 | 442 | 17 | 33 |
| Mark | mark_m02 | 4.80 | 37 | 297 | 9 | 33 |
| Mark | mark_m03 | 1.97 | 37 | 146 | 4 | 8 |
| Mark | mark_m04 | 4.83 | 37 | 261 | 9 | 14 |
| Mark | mark_m05 | 4.00 | 38 | 211 | 6 | 11 |
| Mark | mark_m06 | 1.43 | 37 | 136 | 4 | 8 |
| Natori | mtn_00 | 7.97 | 96 | 201 | 2 | 5 |
| Natori | mtn_01 | 5.00 | 97 | 325 | 3 | 8 |
| Natori | mtn_02 | 5.50 | 96 | 358 | 3 | 10 |
| Natori | mtn_03 | 5.00 | 101 | 419 | 3 | 11 |
| Natori | mtn_04 | 3.33 | 101 | 379 | 3 | 10 |
| Natori | mtn_05 | 5.00 | 99 | 382 | 3 | 11 |
| Natori | mtn_06 | 3.33 | 99 | 345 | 3 | 9 |
| Natori | mtn_07 | 4.00 | 99 | 320 | 3 | 9 |
| Ren | mtn_01 | 5.85 | 68 | 243 | 2 | 11 |
| Ren | mtn_02 | 7.60 | 68 | 560 | 6.5 | 25 |
| Ren | mtn_03 | 8.52 | 68 | 762 | 9.5 | 67 |
| Rice | idle | 4.00 | 54 | 149 | 2 | 8 |
| Rice | mtn_01 | 6.00 | 54 | 376 | 7 | 18 |
| Rice | mtn_02 | 5.00 | 54 | 218 | 2 | 12 |
| Rice | mtn_03 | 8.00 | 54 | 357 | 6 | 18 |
| Wanko | idle_01 | 5.97 | 22 | 107 | 3 | 13 |
| Wanko | idle_02 | 6.93 | 22 | 124 | 3.5 | 19 |
| Wanko | idle_03 | 6.70 | 22 | 119 | 3 | 19 |
| Wanko | idle_04 | 10.37 | 22 | 98 | 3 | 12 |
| Wanko | shake_01 | 2.50 | 22 | 114 | 3 | 12 |
| Wanko | shake_02 | 2.53 | 22 | 131 | 6 | 13 |
| Wanko | touch_01 | 2.00 | 22 | 92 | 3 | 9 |
| Wanko | touch_02 | 2.00 | 22 | 107 | 4 | 9 |
| Wanko | touch_03 | 2.00 | 22 | 99 | 3.5 | 9 |
| Wanko | touch_04 | 2.00 | 22 | 95 | 3 | 9 |
| Wanko | touch_05 | 1.50 | 22 | 80 | 3 | 6 |
| Wanko | touch_06 | 1.87 | 22 | 110 | 5 | 10 |

## Where the inochi2d half went

**Neither inochi2d example model carries animation data**, so the corpus for these two rows is
Cubism only. Checked rather than assumed: the `.inx` container opens with the magic `TRNSRTS\0`,
a big-endian `uint32` payload length and then the puppet as JSON, so a range request for the
first few megabytes is enough to read the top-level keys. Aka has no `animations` key at all;
Midori has one and its value is `null`. This confirms the zero that
[2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md) recorded in its `Animations` column.

## What this does not establish

- **Tracks per animation is measured on a format that exports, not authors.** A Cubism motion
  is baked against the model's full parameter list, which is why most rigs show no variation at
  all across their clips. A puppet format where a rigger writes a track deliberately would
  plausibly show *fewer* tracks and more variance. The reference row may therefore be high and
  the shape of the distribution is not transferable.
- **The corpus is demo content, not streaming content.** These clips are the gestures a sample
  viewer needs to show the rig off. What a streamer actually plays on stream — a wave, a
  reaction, an idle loop — is not sampled here at all, and nothing in the corpus says how many
  animations play at once, which is the figure a frame budget most wants.
- **`PartOpacity` and `Model` curves are counted as tracks for cost, and are not tracks for
  vocabulary.** The figures above are therefore an upper reading of the row; the strict reading
  is lower and is recoverable from the per-rig table.
- **Koharu is missing**, so this corpus is narrower than the one that produced the other rows.
  It sits in `CubismUnityComponents`, whose tree was not walked in this pass.
- **Nothing here is a rig someone paid for.** Same gap the corpus measurement named, same
  consequence: the upper end may be a sample author's ceiling rather than a commissioned
  rigger's.
- **The keyframe distribution may be a Cubism editor artefact.** Two- and three-keyframe tracks
  dominating is consistent with an editor that writes a key at the start and end of every clip
  for every curve it exports, whether or not that curve moves. Nothing was done to distinguish a
  genuinely static track from an authored two-key move, and if most of the floor is the former,
  the real authored keyframe count is lower still and the reference row is generous rather than
  tight.
- **Nothing was rate-limited and nothing was unreachable.** Every `.motion3.json` in the two
  named repositories' `Samples/Resources/` trees parsed cleanly.

## Reproducing it

Both repositories are public and need no account. Downloaded data was kept in its own
directory, the script in another, and Python run isolated.

```sh
DL=$(mktemp -d); BIN=$(mktemp -d); mkdir "$DL/motions"

for repo in CubismWebSamples CubismWebMotionSyncComponents; do
  curl -sS "https://api.github.com/repos/Live2D/$repo/git/trees/HEAD?recursive=1" \
    > "$DL/tree-$repo.json"
  grep -o '"path":"[^"]*motion3\.json"' "$DL/tree-$repo.json" \
    | sed 's/"path":"//; s/"$//' \
    | while read -r p; do
        out=$(echo "$p" | sed 's|Samples/Resources/||; s|/motions/|__|')
        curl -sS -o "$DL/motions/$out" \
          "https://raw.githubusercontent.com/Live2D/$repo/HEAD/$p"
      done
done

python3 -I "$BIN/measure.py" "$DL/motions"
```

`measure.py`, which never consults `Meta` to derive its figures and then asserts against it:

```python
import json, os, sys, statistics

# Cubism segment-type identifier -> (numbers following the identifier, points contributed)
SEG = {0: (2, 1),   # Linear:          endpoint only
       1: (6, 3),   # Bezier:          two control points, then the endpoint
       2: (2, 1),   # Stepped:         endpoint only
       3: (2, 1)}   # InverseStepped:  endpoint only

def parse(path):
    with open(path, encoding='utf-8-sig') as f:
        d = json.load(f)
    curves = []
    for c in d['Curves']:
        s = c['Segments']
        i, segments, points = 2, 0, 1      # the array opens with one point: time, value
        while i < len(s):
            n, p = SEG[s[i]]               # KeyError here means an unhandled segment type
            i += 1 + n
            segments += 1
            points += p
        assert i == len(s), path           # the walk must land exactly on the end
        curves.append((c['Target'], segments + 1, points))   # keyframes = segments + 1
    m = d['Meta']
    assert len(curves) == m['CurveCount'], path
    assert sum(k - 1 for _, k, _ in curves) == m['TotalSegmentCount'], path
    assert sum(p for _, _, p in curves) == m['TotalPointCount'], path
    return curves

def main(dirpath):
    per_animation, per_track = [], []
    for fn in sorted(os.listdir(dirpath)):
        if not fn.endswith('.motion3.json'):
            continue
        curves = parse(os.path.join(dirpath, fn))
        per_animation.append(len(curves))
        per_track += [k for _, k, _ in curves]
    for label, xs in (('tracks per animation', per_animation),
                      ('keyframes per track', per_track)):
        print('%-22s n=%-5d min=%-4d median=%-6g mean=%-6.2f max=%d'
              % (label, len(xs), min(xs), statistics.median(xs),
                 statistics.mean(xs), max(xs)))

main(sys.argv[1])
```

The inochi2d check, which reads only the JSON payload at the head of each `.inx`:

```sh
curl -sS -r 0-4194303 -o Aka.inx.head \
  "https://media.githubusercontent.com/media/Inochi2D/example-models/HEAD/Aka.inx"

python3 -I -c '
import json, struct, sys
b = open(sys.argv[1], "rb").read()
assert b[:8] == b"TRNSRTS\0"
n = struct.unpack(">I", b[8:12])[0]
d = json.loads(b[12:12 + n].decode("utf-8"))
print(sorted(d.keys()), repr(d.get("animations")))
' Aka.inx.head
```
