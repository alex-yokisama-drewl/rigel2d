#!/usr/bin/env python3
"""Probe the two PNGs returned from the Windows leg of S0.

Run this on the machine doing the analysis, not on Windows. It takes the images produced by the
steps in WINDOWS-RUN.md and scores them against REFERENCE.md.

  python3 probe-windows.py --screen screen.png --obs obs-source.png

Neither image needs window coordinates. The screen capture is located by finding the sender's
magenta backdrop, whose colour appears nowhere else in the test frame, and the scale is derived
from that rectangle's measured size — so Windows display scaling is handled rather than assumed
away.
"""

import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from compositor_test_png import read_png  # noqa: E402

WIDTH, HEIGHT, MARGIN = 768, 256, 40
BACKDROP = (255, 0, 255)
PROBES = [("P0", 64), ("P1", 192), ("P2", 320), ("P3", 448), ("P4", 576), ("P5", 704)]
PROBE_Y = 128

# REFERENCE.md, "What the compositor must show".
EXPECT_COMPOSITED = {
    "P0": (255, 255, 255), "P1": (255, 128, 255), "P2": (255, 0, 191),
    "P3": (63, 192, 63), "P4": (255, 0, 255), "P5": (0, 0, 0),
}
EXPECT_IF_ALPHA_IGNORED = {
    "P0": (255, 255, 255), "P1": (128, 128, 128), "P2": (64, 0, 0),
    "P3": (0, 192, 0), "P4": (0, 0, 0), "P5": (0, 0, 0),
}

# REFERENCE.md, "The frame" — what OBS should emit for the source itself.
EXPECT_PREMULTIPLIED = {
    "P0": (255, 255, 255, 255), "P1": (128, 128, 128, 128), "P2": (64, 0, 0, 64),
    "P3": (0, 192, 0, 192), "P4": (0, 0, 0, 0), "P5": (0, 0, 0, 255),
}
EXPECT_STRAIGHT = {
    "P0": (255, 255, 255, 255), "P1": (255, 255, 255, 128), "P2": (255, 0, 0, 64),
    "P3": (0, 255, 0, 192), "P4": None, "P5": (0, 0, 0, 255),
}


def px(rgba, w, x, y, n=4):
    o = (y * w + x) * 4
    return tuple(rgba[o : o + n])


def find_backdrop(rgba, w, h):
    """Bounding box of the magenta backdrop, scanned without any image library."""
    xs, ys = [], []
    step = max(1, min(w, h) // 400)
    for y in range(0, h, step):
        for x in range(0, w, step):
            if px(rgba, w, x, y, 3) == BACKDROP:
                xs.append(x)
                ys.append(y)
    if len(xs) < 20:
        return None
    return min(xs), min(ys), max(xs), max(ys)


def probe_screen(path):
    print(f"\n=== Screen capture: does a transparent window composite on Windows? ===\n{path}")
    rgba, w, h = read_png(path)
    box = find_backdrop(rgba, w, h)
    if not box:
        print("Could not find the magenta backdrop. Was the sender running, and did the")
        print("capture include its window? Without the backdrop nothing here is measurable.")
        return False
    x0, y0, x1, y1 = box
    bw, bh = x1 - x0 + 1, y1 - y0 + 1
    scale = bw / (WIDTH + 2 * MARGIN)
    print(f"backdrop {bw}x{bh} at {x0},{y0} -> display scale {scale:.3f}")
    if abs(bh / (HEIGHT + 2 * MARGIN) - scale) > 0.05:
        print("WARNING: backdrop aspect is wrong; it may be clipped. Results are suspect.")

    tx, ty = x0 + MARGIN * scale, y0 + MARGIN * scale
    print("\nprobe   got              composited       alpha-ignored    verdict")
    ok = True
    for name, dx in PROBES:
        x, y = int(round(tx + dx * scale)), int(round(ty + PROBE_Y * scale))
        got = px(rgba, w, x, y, 3)
        want, ign = EXPECT_COMPOSITED[name], EXPECT_IF_ALPHA_IGNORED[name]
        v = "ok" if got == want else ("ALPHA IGNORED" if got == ign else "UNEXPECTED")
        ok &= v == "ok"
        print(f"{name}      {str(got):<16} {str(want):<16} {str(ign):<16} {v}")
    print()
    if ok:
        print("PASS - a transparent undecorated window composites correctly on Windows.")
        print("stack.md's point 1 is settled for this platform.")
    else:
        print("NOT a clean pass. If P4 reads black, the window is opaque and winit's")
        print("transparency is a hint the system ignored - which reopens the stack choice.")
        print("A near miss on P1-P3 with P4 exact is a colour-space question instead.")
    return ok


def probe_obs(path):
    print(f"\n=== OBS source render: does Game Capture carry alpha? ===\n{path}")
    rgba, w, h = read_png(path)
    print(f"image {w}x{h}")
    if (w, h) != (WIDTH, HEIGHT):
        print(f"WARNING: not the sender's {WIDTH}x{HEIGHT}. OBS may have captured a different")
        print("window or scaled it; probe positions below are therefore suspect.")
    print("\nprobe   got                  premultiplied        straight             verdict")
    premul = straight = True
    over = False
    for name, x in PROBES:
        if x >= w or PROBE_Y >= h:
            print(f"{name}      out of bounds")
            premul = straight = False
            continue
        got = px(rgba, w, x, PROBE_Y)
        if max(got[:3]) > got[3]:
            over = True
        ep, es = EXPECT_PREMULTIPLIED[name], EXPECT_STRAIGHT[name]
        okp, oks = ep is None or got == ep, es is None or got == es
        premul &= okp
        straight &= oks
        v = "premul" if okp and not oks else ("straight" if oks and not okp else ("both" if okp else "NEITHER"))
        print(f"{name}      {str(got):<20} {str(ep):<20} {str(es):<20} {v}")
    print(f"\nRGB > A anywhere: {over}")
    if premul and not straight:
        print("RESULT: Game Capture delivered PREMULTIPLIED alpha, matching the sender exactly.")
    elif straight and not premul:
        print("RESULT: Game Capture delivered STRAIGHT alpha - the path un-premultiplied it.")
    elif premul and straight:
        print("RESULT: ambiguous; every probe agrees with both columns. Frame likely blank.")
    else:
        print("RESULT: matches NEITHER column. Compare against the Linux vkcapture run, where")
        print("colour came back sRGB-encoded while alpha stayed exact - the same shape here")
        print("would mean the hazard is OBS-wide rather than particular to one Linux route.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--screen", help="full-screen capture with the sender's windows visible")
    ap.add_argument("--obs", help="OBS 'Screenshot (Source)' output of the Game Capture source")
    a = ap.parse_args()
    if not a.screen and not a.obs:
        ap.error("give --screen, --obs, or both")
    if a.screen:
        probe_screen(a.screen)
    if a.obs:
        probe_obs(a.obs)


if __name__ == "__main__":
    main()
