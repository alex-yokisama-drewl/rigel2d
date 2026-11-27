#!/usr/bin/env python3
"""Task 2 of S0: does a transparent, undecorated window composite correctly?

Runs the sender's windowed mode, screenshots the real screen, and probes the composited result
against the table in REFERENCE.md, "What the compositor must show".

Nothing here parses an image with judgement. It reads named pixels and compares integers.
"""

import os
import re
import subprocess
import sys
import tempfile
import time


HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from compositor_test_png import read_png  # noqa: E402
SENDER = os.path.join(HERE, "target", "release", "alpha-sender")

# REFERENCE.md: probes sit at x = 64 + 128 i, y = 128 within the test window.
PROBES = [("P0", 64), ("P1", 192), ("P2", 320), ("P3", 448), ("P4", 576), ("P5", 704)]
PROBE_Y = 128

# Over the opaque magenta backdrop, if alpha works.
EXPECT_COMPOSITED = {
    "P0": (255, 255, 255),
    "P1": (255, 128, 255),
    "P2": (255, 0, 191),
    "P3": (63, 192, 63),
    "P4": (255, 0, 255),
    "P5": (0, 0, 0),
}

# What the same probes would read if the compositor ignored alpha entirely.
EXPECT_IF_ALPHA_IGNORED = {
    "P0": (255, 255, 255),
    "P1": (128, 128, 128),
    "P2": (64, 0, 0),
    "P3": (0, 192, 0),
    "P4": (0, 0, 0),
    "P5": (0, 0, 0),
}

BACKDROP = (255, 0, 255)


def main():
    if not os.path.exists(SENDER):
        sys.exit(f"build first: cargo build --release ({SENDER} missing)")

    proc = subprocess.Popen(
        [SENDER, "--window"] + sys.argv[1:],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    geom, deadline = {}, time.time() + 20
    while time.time() < deadline:
        line = proc.stdout.readline()
        if not line:
            break
        line = line.strip()
        print("  sender:", line)
        m = re.match(r"(BACKDROP_AT|TEST_AT) (-?\d+) (-?\d+)", line)
        if m:
            geom[m.group(1)] = (int(m.group(2)), int(m.group(3)))
        if line == "READY":
            break

    if "TEST_AT" not in geom:
        proc.kill()
        sys.exit("sender never reported its geometry; stderr:\n" + proc.stderr.read())

    # Let the compositor settle before grabbing; a frame mid-map would read as a failure.
    time.sleep(2.0)

    shot = os.path.join(tempfile.mkdtemp(), "screen.png")
    subprocess.run(["scrot", "-o", shot], check=True)
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    rgba, w, h = read_png(shot)
    tx, ty = geom["TEST_AT"]
    bx, by = geom.get("BACKDROP_AT", (tx, ty))
    print(f"\nscreenshot {w}x{h}; backdrop at {bx},{by}; test window at {tx},{ty}")

    def at(x, y):
        o = (y * w + x) * 4
        return tuple(rgba[o : o + 3])

    # Confirm the backdrop is where the sender said before trusting anything inside it. A probe
    # taken over the wrong window would otherwise be reported as a compositing result.
    corner = at(bx + 10, by + 10)
    print(f"backdrop corner at {bx+10},{by+10}: {corner}  (want {BACKDROP})")
    if corner != BACKDROP:
        print("\nBACKDROP NOT FOUND where expected — another window is probably on top.")
        print("Every probe below is therefore meaningless. Not a transparency result.")
        sys.exit(2)

    print("\nprobe   screen x,y     got              composited       alpha-ignored    verdict")
    verdicts = []
    for name, dx in PROBES:
        x, y = tx + dx, ty + PROBE_Y
        got = at(x, y)
        want = EXPECT_COMPOSITED[name]
        ignored = EXPECT_IF_ALPHA_IGNORED[name]
        if got == want:
            v = "ok"
        elif got == ignored:
            v = "ALPHA IGNORED"
        else:
            v = "UNEXPECTED"
        verdicts.append(v)
        print(f"{name}      {x},{y}".ljust(22) + f"{got}".ljust(17)
              + f"{want}".ljust(17) + f"{ignored}".ljust(17) + v)

    print()
    if all(v == "ok" for v in verdicts):
        print("PASS — the compositor blends the window exactly as REFERENCE.md predicts.")
        print("Transparency works on this platform, and blending is in encoded 8-bit space.")
    elif verdicts[4] == "ALPHA IGNORED":
        print("FAIL — P4 shows black where the backdrop should be.")
        print("The alpha channel is not reaching the compositor; the window is opaque.")
        sys.exit(1)
    else:
        print("MIXED — see REFERENCE.md's caveat: a near miss on P1-P3 with P4 exact is a")
        print("colour-space question (blending in linear light), not a transparency failure.")
        sys.exit(1)


if __name__ == "__main__":
    main()
