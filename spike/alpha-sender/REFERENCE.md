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
