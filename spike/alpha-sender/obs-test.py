#!/usr/bin/env python3
"""Tasks 3, 4 and 6 of S0: what does OBS actually emit for our frame?

Drives OBS through its bundled obs-websocket: builds a scene and a capture source, asks OBS to
save that source's own render as a PNG (which carries alpha), and probes the result against the
dump table in REFERENCE.md.

Measuring the *source's* render rather than the program output is deliberate: it is the frame as
OBS received it, before any scene background is composited behind it, which is what the
premultiplied-versus-straight question is actually about.

Needs `websocket-client`. Pass the interpreter that has it as OBS_TEST_PYTHON, or run this with
that interpreter directly.

  python3 obs-test.py --kind vkcapture-source --sender-args "--window"
"""

import argparse
import base64
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
SENDER = os.path.join(HERE, "target", "release", "alpha-sender")
WS_CONFIG = os.path.expanduser(
    "~/.config/obs-studio/plugin_config/obs-websocket/config.json"
)

SCENE = "rigel-s0-spike"
INPUT = "rigel-s0-capture"

# REFERENCE.md, "The frame". Probe offsets within the captured frame.
PROBES = [("P0", 64), ("P1", 192), ("P2", 320), ("P3", 448), ("P4", 576), ("P5", 704)]
PROBE_Y = 128

EXPECT_PREMULTIPLIED = {
    "P0": (255, 255, 255, 255),
    "P1": (128, 128, 128, 128),
    "P2": (64, 0, 0, 64),
    "P3": (0, 192, 0, 192),
    "P4": (0, 0, 0, 0),
    "P5": (0, 0, 0, 255),
}
EXPECT_STRAIGHT = {
    "P0": (255, 255, 255, 255),
    "P1": (255, 255, 255, 128),
    "P2": (255, 0, 0, 64),
    "P3": (0, 255, 0, 192),
    "P4": None,  # RGB undefined at zero alpha
    "P5": (0, 0, 0, 255),
}

sys.path.insert(0, HERE)
from compositor_test_png import read_png  # noqa: E402


def launch(unit, cmd, env_overrides=None):
    """Start a process outside any snap mount namespace, as a transient user service.

    This session may itself run inside a confined snap (an editor, for instance). A snap gives
    its children an older runtime and a private /tmp, and OBS built against the host glibc dies
    on the mismatch: `symbol lookup error ... /snap/core20/.../libpthread.so.0`.

    `systemd-run --user --scope` does NOT help — a scope stays in the caller's namespace, and
    only looks like it works for a command such as `--version` that never loads the heavy
    libraries. A transient *service*, with no --scope, is started by the user's systemd manager
    instead, outside the confinement. That is the distinction, and it costs an afternoon to
    rediscover.
    """
    stop(unit)
    argv = [
        "systemd-run", "--user", "--quiet", "--collect", f"--unit={unit}",
        f"--setenv=DISPLAY={os.environ.get('DISPLAY', ':0')}",
        f"--setenv=XAUTHORITY={os.environ.get('XAUTHORITY', os.path.expanduser('~/.Xauthority'))}",
    ]
    for k, v in (env_overrides or {}).items():
        argv.append(f"--setenv={k}={v}")
    subprocess.run(argv + list(cmd), check=True)
    return unit


def stop(unit):
    """Shut a unit down cleanly, then make sure it is actually gone.

    OBS treats SIGTERM as an unclean exit and offers Safe Mode on next launch — which disables
    third-party plugins, so the capture plugin silently vanishes and the next run reports the
    input kind as simply unavailable. SIGINT is the clean quit. `systemctl stop` with --collect
    has also been seen to retire the unit while leaving the process behind, so this verifies
    rather than assumes.
    """
    subprocess.run(
        ["systemctl", "--user", "kill", "--signal=SIGINT", unit],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    for _ in range(20):
        active = subprocess.run(
            ["systemctl", "--user", "is-active", unit],
            capture_output=True, text=True,
        ).stdout.strip()
        if active != "active":
            break
        time.sleep(0.5)
    subprocess.run(
        ["systemctl", "--user", "stop", unit],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    # Verify: a retired unit does not guarantee a dead process.
    leftover = subprocess.run(
        ["pgrep", "-f", f"--multi --disable-missing-files-check"],
        capture_output=True, text=True,
    ).stdout.split()
    for pid in leftover:
        subprocess.run(["kill", "-INT", pid], stderr=subprocess.DEVNULL)


class Obs:
    """Minimal obs-websocket v5 client: connect, identify, call requests."""

    def __init__(self, port, password):
        import websocket

        self.ws = websocket.create_connection(f"ws://127.0.0.1:{port}", timeout=20)
        hello = json.loads(self.ws.recv())["d"]
        ident = {"rpcVersion": hello["rpcVersion"]}
        if "authentication" in hello:
            a = hello["authentication"]
            secret = base64.b64encode(
                hashlib.sha256((password + a["salt"]).encode()).digest()
            ).decode()
            ident["authentication"] = base64.b64encode(
                hashlib.sha256((secret + a["challenge"]).encode()).digest()
            ).decode()
        self.ws.send(json.dumps({"op": 1, "d": ident}))
        got = json.loads(self.ws.recv())
        if got.get("op") != 2:
            raise RuntimeError(f"obs-websocket refused identify: {got}")

    def call(self, request_type, data=None, tolerate=False):
        rid = str(uuid.uuid4())
        self.ws.send(
            json.dumps(
                {
                    "op": 6,
                    "d": {
                        "requestType": request_type,
                        "requestId": rid,
                        "requestData": data or {},
                    },
                }
            )
        )
        while True:
            msg = json.loads(self.ws.recv())
            if msg.get("op") == 7 and msg["d"]["requestId"] == rid:
                status = msg["d"]["requestStatus"]
                if not status["result"]:
                    if tolerate:
                        return None
                    raise RuntimeError(
                        f"{request_type} failed: {status.get('comment', status)}"
                    )
                return msg["d"].get("responseData") or {}


def enable_websocket():
    """Turn the bundled server on, always recording the prior state to a file.

    The backup is written unconditionally, even when the server is already enabled. An earlier
    version only wrote one when it had to flip the setting, which meant that a run finding the
    server already on recorded nothing and restored nothing — so a setting enabled once by hand
    stayed enabled for good. A restore path that only runs in some cases is not a restore path.
    """
    cfg = json.load(open(WS_CONFIG))
    if not os.path.exists(WS_CONFIG + ".rigel-backup"):
        shutil.copy(WS_CONFIG, WS_CONFIG + ".rigel-backup")
    if not cfg.get("server_enabled"):
        cfg["server_enabled"] = True
        json.dump(cfg, open(WS_CONFIG, "w"), indent=2)
        print("enabled obs-websocket for this run")
    return cfg["server_port"], cfg["server_password"]


def restore_websocket():
    """Put the config back exactly as it was found, from the backup on disk."""
    backup = WS_CONFIG + ".rigel-backup"
    if os.path.exists(backup):
        shutil.copy(backup, WS_CONFIG)
        os.remove(backup)
        print("restored obs-websocket config")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kind", default="vkcapture-source", help="OBS input kind to create")
    ap.add_argument(
        "--settings", default="{}", help="JSON input settings, e.g. allow_transparency"
    )
    ap.add_argument("--sender-args", default="--window")
    ap.add_argument("--sender-env", default="", help="k=v,k=v applied to the sender")
    ap.add_argument("--keep-obs", action="store_true", help="do not kill OBS at the end")
    args = ap.parse_args()

    if not os.path.exists(SENDER):
        sys.exit(f"build first: {SENDER} missing")

    port, password = enable_websocket()
    obs_proc = sender = None
    out = os.path.join(tempfile.mkdtemp(), "obs-source.png")

    try:
        overrides = {}
        for pair in filter(None, args.sender_env.split(",")):
            k, v = pair.split("=", 1)
            overrides[k] = v
        sender = launch("rigel-s0-sender", [SENDER] + args.sender_args.split(), overrides)
        time.sleep(3)

        obs_proc = launch(
            "rigel-s0-obs", ["/usr/bin/obs", "--multi", "--disable-missing-files-check"]
        )

        obs = None
        deadline = time.time() + 60
        while time.time() < deadline and obs is None:
            try:
                obs = Obs(port, password)
            except Exception:
                time.sleep(2)
        if obs is None:
            sys.exit("could not reach obs-websocket; is OBS running?")

        ver = obs.call("GetVersion")
        print(f"OBS {ver['obsVersion']}, websocket {ver['obsWebSocketVersion']}")
        kinds = obs.call("GetInputKindList")["inputKinds"]
        if args.kind not in kinds:
            sys.exit(
                f"input kind {args.kind!r} not available.\n"
                f"OBS reports: {sorted(kinds)}\n"
                "The capture plugin is not loaded."
            )

        obs.call("RemoveScene", {"sceneName": SCENE}, tolerate=True)
        obs.call("CreateScene", {"sceneName": SCENE})
        obs.call("SetCurrentProgramScene", {"sceneName": SCENE})
        obs.call(
            "CreateInput",
            {
                "sceneName": SCENE,
                "inputName": INPUT,
                "inputKind": args.kind,
                "inputSettings": json.loads(args.settings),
            },
        )
        print(f"created {args.kind} with {args.settings}")

        # Let the capture negotiate and deliver frames before measuring.
        time.sleep(6)

        obs.call(
            "SaveSourceScreenshot",
            {
                "sourceName": INPUT,
                "imageFormat": "png",
                "imageFilePath": out,
            },
        )
        print(f"saved source render to {out}")

        rgba, w, h = read_png(out)
        print(f"captured {w}x{h}")
        if (w, h) != (768, 256):
            print(
                "WARNING: not the sender's 768x256. OBS may be capturing a different "
                "window, or scaling. Probe positions below are therefore suspect."
            )
        report(rgba, w, h)

        obs.call("RemoveScene", {"sceneName": SCENE}, tolerate=True)

    finally:
        if sender:
            stop(sender)
        if obs_proc and not args.keep_obs:
            stop(obs_proc)
        restore_websocket()


def report(rgba, w, h):
    def at(x, y):
        o = (y * w + x) * 4
        return tuple(rgba[o : o + 4])

    print("\nprobe   got                  premultiplied        straight             verdict")
    premul = straight = True
    rgb_over_a = False
    for name, x in PROBES:
        if x >= w or PROBE_Y >= h:
            print(f"{name}      out of bounds")
            premul = straight = False
            continue
        got = at(x, PROBE_Y)
        if max(got[:3]) > got[3]:
            rgb_over_a = True
        ep, es = EXPECT_PREMULTIPLIED[name], EXPECT_STRAIGHT[name]
        okp = ep is None or got == ep
        oks = es is None or got == es
        premul &= okp
        straight &= oks
        verdict = "premul" if okp and not oks else ("straight" if oks and not okp else ("both" if okp else "NEITHER"))
        print(f"{name}      {str(got):<20} {str(ep):<20} {str(es):<20} {verdict}")

    print(f"\nRGB > A anywhere: {rgb_over_a}")
    if premul and not straight:
        print("RESULT: OBS received PREMULTIPLIED alpha, matching the sender exactly.")
    elif straight and not premul:
        print("RESULT: OBS received STRAIGHT alpha. The route converted, or un-premultiplied.")
    elif premul and straight:
        print("RESULT: ambiguous — every probe agrees with both columns. Frame is likely blank.")
    else:
        print("RESULT: matches NEITHER column. Alpha is being mangled, not merely converted.")
        print("Compare the saved PNG against the sender's own --dump output.")


if __name__ == "__main__":
    main()
