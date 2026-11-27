# The test frame, and what each route must emit

**Derived 2026-11-26, before any route was tested.** That ordering is the point: premultiplied
and straight alpha differ by a factor that is easy to rationalise after the fact, so the expected
numbers are fixed here first and every route is scored against them unchanged.

Part of S0 ([../../docs/wip/s0-transparency-spike.md](../../docs/wip/s0-transparency-spike.md)).
Throwaway scaffolding — S1 keeps whatever survives.

## The frame

Six vertical bands, each 128 px wide, in a 768 × 256 surface. One probe per band, taken at the
band's centre: `x = 64 + 128 i`, `y = 128`.

**768 is chosen, not incidental.** At 4 bytes per pixel the row stride is 3072, an exact multiple
of the 256-byte alignment GPU buffer readback requires — so the dump copies rows with no padding
to strip. Padding handled wrongly shifts every pixel after the first row and reads as a colour
error, which is a fault that would look exactly like the one this spike is hunting.

Every value below is **8-bit, and exactly representable**. The alphas are 0, 64, 128, 192 and 255
rather than percentages, so no value lands on a rounding boundary and no result can be explained
away as an off-by-one.

| Probe | x | Intent | A | If the pipeline carries **premultiplied** | If it carries **straight** |
|---|---|---|---|---|---|
| P0 | 64 | Opaque white | 255 | `255,255,255,255` | `255,255,255,255` |
| P1 | 192 | White, half alpha | 128 | `128,128,128,128` | `255,255,255,128` |
| P2 | 320 | Red, quarter alpha | 64 | `64,0,0,64` | `255,0,0,64` |
| P3 | 448 | Green, three-quarter alpha | 192 | `0,192,0,192` | `0,255,0,192` |
| P4 | 576 | Fully transparent | 0 | `0,0,0,0` | RGB undefined, `A=0` |
| P5 | 704 | Opaque black | 255 | `0,0,0,255` | `0,0,0,255` |

## How to read a result

**P1 is the instrument.** Everything else is a control or a cross-check. White at half alpha is
where the two interpretations are furthest apart and where no channel is zero, so a route cannot
coincidentally agree with both.

Three independent reads, so that one wrong one is caught:

- **RGB greater than A anywhere proves straight.** Premultiplied data cannot contain it — the
  stored colour is already scaled by alpha, so each channel is bounded above by it. A route
  emitting `255,255,255,128` has handed over straight data whatever it claims.
- **P0 and P5 must match under both columns.** They are opaque, so they carry no information
  about premultiplication. If either is wrong, the fault is upstream of the alpha question —
  colour space, format, or the sender itself — and the rest of the table means nothing until it
  is fixed.
- **P4 with non-zero RGB** means straight data, or that something has written into fully
  transparent pixels. Either way it is not premultiplied.

## The trap this table is built to avoid

**The surface must not be an sRGB format.** With `Bgra8UnormSrgb` the 8-bit values actually
stored are the transfer-encoded ones, not the values written, and every number above is wrong by
an amount that looks plausible — about `188` where this table says `128`. The sender therefore
requests a linear `Bgra8Unorm` surface and fails loudly rather than silently accepting an sRGB
one.

This is recorded here rather than in the code because it is the first thing to re-check if a
route's numbers are close to the table but not equal.

## What the sender reports

`--dump <path.png>` renders offscreen with no window and writes the frame, then prints each
probe as `Pn x,y → R,G,B,A` with its verdict against this table. Running that before any route is
wired up is what makes a later failure attributable: if the dump is already wrong, no OBS route
is at fault yet.

## What the compositor must show (task 2)

The dump above measures what the sender *produces*. This measures what a real compositor does
with it — `stack.md`'s half of the spike, and independent of OBS.

The test window is transparent and undecorated, over an **opaque magenta backdrop**,
`255,0,255`, which the sender also owns so the background is known rather than whatever happened
to be on the desktop. Magenta appears nowhere in the test frame, so any leak is obvious on sight.

**The backdrop's channels are 0 or 255, and that is deliberate.** Compositing is
`result = src + dst × (1 − a)`, and with `dst` at 0 or 255 every expected value below is an exact
integer — `255 × k/255 = k`. A backdrop of, say, 200 would put rounding into the one measurement
that is supposed to have none.

| Probe | A | Source (premultiplied) | Over magenta, if alpha works | If alpha is ignored |
|---|---|---|---|---|
| P0 | 255 | `255,255,255` | `255,255,255` | `255,255,255` |
| P1 | 128 | `128,128,128` | `255,128,255` | `128,128,128` |
| P2 | 64 | `64,0,0` | `255,0,191` | `64,0,0` |
| P3 | 192 | `0,192,0` | `63,192,63` | `0,192,0` |
| P4 | 0 | `0,0,0` | `255,0,255` — the backdrop, exactly | `0,0,0` — black |
| P5 | 255 | `0,0,0` | `0,0,0` | `0,0,0` |

**P4 is the instrument here**, as P1 was for the dump. A fully transparent pixel must show the
backdrop. If it comes back black, the alpha channel never reached the compositor and the window
is simply opaque — which is the failure `stack.md` warns about when it quotes winit calling
transparency "more of a hint for the system".

P1, P2 and P3 then separate *alpha works* from *alpha blends correctly*: a compositor could honour
transparency and still blend wrongly, and those three would catch it where P4 alone would not.

**One caveat this cannot settle from the numbers alone.** The table assumes the compositor blends
the 8-bit encoded values directly, which is what compositors conventionally do. One blending in
linear light would give different figures that are not wrong, merely different — so a near miss
on P1–P3 with P4 exact should be read as a colour-space question, not as a transparency failure.

## The band this frame is missing, to add before the next measurement

**Deliberately not added now.** The measurements already recorded were taken against the frame
above, and changing its geometry would break the correspondence between the committed instrument
and the committed results. This is the specification for whoever measures next — a second
machine, a second GPU generation, or macOS.

**The blind spot.** Every band above is full-intensity white, red or green. In premultiplied form
that makes the stored colour *equal to the alpha* at every probe; in straight form every channel
is 0 or 255, where the sRGB transfer function is the identity. The opaque and fully-transparent
bands discriminate nothing by design. So **no probe has a colour that is neither 0, nor 255, nor
equal to its own alpha** — and a behaviour with no alpha term at all fits one platform's
premultiplied run as well as the real one does. Only the second run on that platform excludes it.

**One band closes it.** Grey at **colour 153, alpha 85**:

| | Stored |
|---|---|
| Premultiplied | `51,51,51,85` |
| Straight | `153,153,153,85` |

Both forms put the colour strictly between 0 and 255, neither equals its alpha, and the two
differ from each other — so the band discriminates on its own. `153 × 85 = 13005 = 255 × 51`, so
the premultiplied store is exact and inherits the no-rounding property the other bands have.

Adding it means a seven-band frame. Keep the row stride 256-byte aligned: **896 × 256**, seven
bands of 128, probes at `x = 64 + 128 i`, since `896 × 4 = 3584 = 256 × 14`.
