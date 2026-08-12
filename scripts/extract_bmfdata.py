#!/usr/bin/env python3
"""Extract the embedded bitmap-font TIFFs from C leptonica's bmfdata.h.

C leptonica embeds its 9 bitmap-font sheets (fontsize 4-20, even) as
base64-encoded CCITT G4 TIFF strings (fontdata_N) in src/bmfdata.h.
This script decodes them into src/core/fonts/chars-N.tif so the Rust
`Bmf` can `include_bytes!` the exact same font data (plan 902 PR 12).

The output files are committed; rerun only when reference/leptonica's
bmfdata.h changes. The decoded bytes are verified to be TIFF (II*\\0).

Usage: python3 scripts/extract_bmfdata.py
"""

import base64
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "reference" / "leptonica" / "src" / "bmfdata.h"
OUT_DIR = ROOT / "src" / "core" / "fonts"


def main() -> int:
    if not SRC.exists():
        print(f"bmfdata.h not found at {SRC}", file=sys.stderr)
        print("Clone C leptonica into reference/leptonica first.", file=sys.stderr)
        return 1

    text = SRC.read_text()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    sizes = []
    pattern = r'static const char\s+fontdata_(\d+)\[\]\s*=\s*((?:\s*"[^"]*")+);'
    for m in re.finditer(pattern, text):
        size = int(m.group(1))
        b64 = "".join(re.findall(r'"([^"]*)"', m.group(2)))
        data = base64.b64decode(b64)
        if data[:4] != b"II*\x00":
            print(f"fontdata_{size}: not a little-endian TIFF", file=sys.stderr)
            return 1
        out = OUT_DIR / f"chars-{size}.tif"
        out.write_bytes(data)
        sizes.append(size)
        print(f"fontdata_{size}: {len(data)} bytes -> {out.relative_to(ROOT)}")

    expected = [4, 6, 8, 10, 12, 14, 16, 18, 20]
    if sorted(sizes) != expected:
        print(f"expected sizes {expected}, got {sorted(sizes)}", file=sys.stderr)
        return 1
    print(f"OK: {len(sizes)} font sheets extracted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
