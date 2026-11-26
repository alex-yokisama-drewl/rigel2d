# The Windows leg of S0 — what to run, and what to send back

Needs nothing installed on Windows. No Python, no Rust, no build tools. Two files come back and
the measuring happens on the Linux side.

This answers three things at once, which is why it is worth one session rather than three:

1. **Does a transparent, undecorated window work on Windows at all?** `stack.md` records winit's
   own documentation calling transparency *"more of a hint for the system"*. If it is a hint
   Windows ignores, the stack choice reopens — so this gates everything downstream, whichever
   route the output eventually takes.
2. **Does OBS's built-in Game Capture carry alpha?** It has never been observed doing so.
3. **Does Game Capture hooking our swapchain conflict with *the player requests no
   screen-capture capability*?** `performing.md` leaves that open and asks the spike to answer
   it rather than assume.

---

## Run it

**1.** Unzip anywhere and open the folder.

**2.** Run `alpha-sender.exe --window` (a `run.bat` is included; double-click it).

Two windows appear: a **magenta rectangle** with a strip of **six colour bands** inside it. The
magenta is a backdrop the program draws itself, so the colour behind the bands is known rather
than whatever happened to be on the desktop. Leave both where they are — do not move or resize
them.

If the bands look washed out or the magenta is missing, take the screenshot anyway. That is a
result, not a failure, and the numbers will say which.

**3.** Take a **full-screen screenshot** with both windows visible, and save it as a **PNG**.

`PrtScn`, or `Win+Shift+S` then save. Any tool is fine as long as it is a PNG and not a JPEG —
JPEG compression would alter the very pixel values being measured. Call it `screen.png`.

**4.** In OBS, add a **Game Capture** source:

- Sources panel → **+** → **Game Capture**
- Mode: **Capture specific window**
- Window: **`alpha-sender test`** — this one specifically, *not* `alpha-sender backdrop`
- Tick **Allow Transparency**
- OK

**5.** Right-click that source → **Screenshot (Source)**.

OBS writes a PNG carrying the alpha channel. It goes to the folder in
*Settings → Output → Recording → Recording Path*. Call it `obs-source.png`.

**6.** Send back `screen.png`, `obs-source.png`, and the OBS version from *Help → About*.

---

## Things that would quietly spoil the result

| | |
|---|---|
| Saving either image as **JPEG** | It re-encodes the pixel values that are the measurement |
| Selecting **`alpha-sender backdrop`** in step 4 | Captures the wrong window. Detected on this side — the frame would be 848×336 instead of 768×256 — so it is recoverable, not fatal |
| **Screenshot (Output)** instead of **Screenshot (Source)** | Output is the whole composited scene over the canvas background; Source is the frame as OBS received it, which is what the premultiplied-versus-straight question is about |
| Moving the windows between step 2 and step 3 | Harmless. The backdrop is located by its colour, and display scaling is derived from its measured size |

## What happens next

`probe-windows.py` scores both images against `REFERENCE.md`, whose expected values were written
down before any of this was built. Nothing is judged by eye: the probe reads named pixels and
compares integers.
