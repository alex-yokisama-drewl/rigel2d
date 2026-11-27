"""Minimal PNG decoder, shared by the compositor and OBS harnesses.

Hand-rolled so the spike's scaffolding needs no image library: a dependency here would be one
more thing to install on the Windows leg, which the owner runs themselves.
"""

def read_png(path):
    """Decode a PNG to RGBA bytes without a third-party library."""
    import zlib

    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, idat, w = 8, b"", None
    while pos < len(data):
        ln = int.from_bytes(data[pos : pos + 4], "big")
        typ = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + ln]
        if typ == b"IHDR":
            w = int.from_bytes(body[0:4], "big")
            h = int.from_bytes(body[4:8], "big")
            depth, colour = body[8], body[9]
            assert depth == 8, f"unsupported bit depth {depth}"
            assert colour in (2, 6), f"unsupported colour type {colour}"
            channels = 3 if colour == 2 else 4
        elif typ == b"IDAT":
            idat += body
        elif typ == b"IEND":
            break
        pos += 12 + ln

    raw = zlib.decompress(idat)
    stride = w * channels
    out = bytearray()
    prev = bytearray(stride)
    p = 0
    for _ in range(h):
        f = raw[p]
        line = bytearray(raw[p + 1 : p + 1 + stride])
        p += 1 + stride
        for i in range(stride):
            a = line[i - channels] if i >= channels else 0
            b = prev[i]
            c = prev[i - channels] if i >= channels else 0
            if f == 1:
                line[i] = (line[i] + a) & 0xFF
            elif f == 2:
                line[i] = (line[i] + b) & 0xFF
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        prev = line
        if channels == 3:
            for i in range(0, stride, 3):
                out += line[i : i + 3] + b"\xff"
        else:
            out += line
    return bytes(out), w, h
