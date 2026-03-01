#!/usr/bin/env python3
"""Diagnose 1bpp PNG polarity differences between C and Rust golden files.

Pure Python implementation (no PIL/numpy). Handles PNG filter types 0-4.
Compares C and Rust golden PNG files to determine if differences are
purely a PNG I/O polarity convention issue or indicate an algorithm bug.

Usage:
    python3 scripts/diagnose_1bpp_polarity.py
"""

import struct
import zlib
from pathlib import Path

C_DIR = Path("/tmp/lept/regout")
RUST_DIR = Path("tests/golden")

# Mapping: (c_file, rust_file, description)
PAIRS = [
    # ccthin1: C checks 6-9 ↔ Rust cthin1_thin 1-4
    ("ccthin1.06.png", "cthin1_thin_golden.01.png", "thin FG 4-cc"),
    ("ccthin1.07.png", "cthin1_thin_golden.02.png", "thin BG 4-cc"),
    ("ccthin1.08.png", "cthin1_thin_golden.03.png", "thin FG 8-cc"),
    ("ccthin1.09.png", "cthin1_thin_golden.04.png", "thin BG 8-cc"),
    # ccthin2: C checks 0-10 ↔ Rust cthin2_set 1-11
    ("ccthin2.00.png", "cthin2_set_golden.01.png", "set FG Set4cc1"),
    ("ccthin2.01.png", "cthin2_set_golden.02.png", "set FG Set4cc2"),
    ("ccthin2.02.png", "cthin2_set_golden.03.png", "set FG Set4cc3"),
    ("ccthin2.03.png", "cthin2_set_golden.04.png", "set FG Set48"),
    ("ccthin2.04.png", "cthin2_set_golden.05.png", "set FG Set8cc1"),
    ("ccthin2.05.png", "cthin2_set_golden.06.png", "set FG Set8cc2"),
    ("ccthin2.06.png", "cthin2_set_golden.07.png", "set FG Set8cc3"),
    ("ccthin2.07.png", "cthin2_set_golden.08.png", "set FG Set8cc4"),
    ("ccthin2.08.png", "cthin2_set_golden.09.png", "set FG Set8cc5"),
    ("ccthin2.09.png", "cthin2_set_golden.10.png", "set BG Thick4"),
    ("ccthin2.10.png", "cthin2_set_golden.11.png", "set BG Thick8"),
]


def paeth_predictor(a, b, c):
    """PNG Paeth predictor."""
    p = a + b - c
    pa = abs(p - a)
    pb = abs(p - b)
    pc = abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    elif pb <= pc:
        return b
    else:
        return c


def unfilter_row(filter_type, row_bytes, prev_row, bpp=1):
    """Reverse PNG row filtering. bpp = bytes per pixel (1 for 1-bit packed)."""
    out = bytearray(len(row_bytes))
    for i in range(len(row_bytes)):
        x = row_bytes[i]
        a = out[i - bpp] if i >= bpp else 0
        b = prev_row[i] if prev_row is not None else 0
        c = (prev_row[i - bpp] if i >= bpp else 0) if prev_row is not None else 0

        if filter_type == 0:  # None
            out[i] = x
        elif filter_type == 1:  # Sub
            out[i] = (x + a) & 0xFF
        elif filter_type == 2:  # Up
            out[i] = (x + b) & 0xFF
        elif filter_type == 3:  # Average
            out[i] = (x + (a + b) // 2) & 0xFF
        elif filter_type == 4:  # Paeth
            out[i] = (x + paeth_predictor(a, b, c)) & 0xFF
        else:
            raise ValueError(f"Unknown filter type: {filter_type}")
    return bytes(out)


def read_png_1bpp(path):
    """Read a 1bpp PNG and return (width, height, pixels_list).

    Returns pixel values as stored in the PNG (0 or 1), after filter reversal.
    """
    with open(path, "rb") as f:
        sig = f.read(8)
        assert sig == b"\x89PNG\r\n\x1a\n", f"Not a PNG: {path}"

        width = height = bit_depth = color_type = 0
        idat_chunks = []

        while True:
            raw_len = f.read(4)
            if len(raw_len) < 4:
                break
            chunk_len = struct.unpack(">I", raw_len)[0]
            chunk_type = f.read(4)
            chunk_data = f.read(chunk_len)
            _crc = f.read(4)

            if chunk_type == b"IHDR":
                width, height, bit_depth, color_type = struct.unpack(
                    ">IIBB", chunk_data[:10]
                )
            elif chunk_type == b"IDAT":
                idat_chunks.append(chunk_data)
            elif chunk_type == b"IEND":
                break

    assert bit_depth == 1, f"Expected 1bpp, got {bit_depth}bpp: {path}"

    raw = zlib.decompress(b"".join(idat_chunks))
    bytes_per_row = (width + 7) // 8

    pixels = []
    prev_row = None
    offset = 0

    for _y in range(height):
        filter_type = raw[offset]
        offset += 1
        filtered_row = raw[offset : offset + bytes_per_row]
        offset += bytes_per_row

        row_data = unfilter_row(filter_type, filtered_row, prev_row)
        prev_row = row_data

        for x in range(width):
            byte_idx = x // 8
            bit_idx = 7 - (x % 8)
            val = (row_data[byte_idx] >> bit_idx) & 1
            pixels.append(val)

    return width, height, pixels


def compare_pair(c_path, r_path, desc):
    """Compare a C and Rust golden file pair."""
    w_c, h_c, px_c = read_png_1bpp(c_path)
    w_r, h_r, px_r = read_png_1bpp(r_path)

    result = {"desc": desc}

    if (w_c, h_c) != (w_r, h_r):
        result["error"] = f"Dim mismatch: C={w_c}x{h_c} Rust={w_r}x{h_r}"
        return result

    total = w_c * h_c
    result["width"] = w_c
    result["height"] = h_c
    result["total"] = total

    # Count 1-bits in PNG (PNG: 0=black, 1=white)
    c_ones = sum(px_c)
    r_ones = sum(px_r)
    result["c_ones"] = c_ones  # white pixels in C PNG
    result["r_ones"] = r_ones  # white pixels in Rust PNG

    # Direct comparison
    direct_diff = sum(1 for a, b in zip(px_c, px_r) if a != b)
    result["direct_diff"] = direct_diff
    result["direct_pct"] = direct_diff / total * 100

    # Inverted comparison: invert C bits, then compare with Rust
    # If C inverts on write (Pix fg=1 → PNG 0), inverting C PNG should give
    # the same internal Pix values. If Rust doesn't invert (Pix fg=1 → PNG 1),
    # then inverted-C PNG should match Rust PNG if algorithms produce same result.
    inv_diff = sum(1 for a, b in zip(px_c, px_r) if (1 - a) != b)
    result["inv_diff"] = inv_diff
    result["inv_pct"] = inv_diff / total * 100

    return result


def main():
    print("=" * 105)
    print("1bpp PNG Polarity Diagnosis: C vs Rust Golden Files")
    print("=" * 105)
    print()
    print("Test: Invert C PNG bits → compare with Rust PNG.")
    print("If inv_diff == 0, the only difference is PNG polarity convention.")
    print("If inv_diff > 0, there is an actual algorithm difference.")
    print()
    hdr = (
        f"{'Description':<20} {'Dim':>10} {'Total':>8} "
        f"{'C 1-px':>8} {'R 1-px':>8} "
        f"{'Direct':>8} {'DirPct':>7} "
        f"{'InvDiff':>8} {'InvPct':>7} {'Verdict'}"
    )
    print(hdr)
    print("-" * 105)

    all_results = []
    for c_name, r_name, desc in PAIRS:
        c_path = C_DIR / c_name
        r_path = RUST_DIR / r_name

        if not c_path.exists():
            print(f"{desc:<20} C missing: {c_path}")
            continue
        if not r_path.exists():
            print(f"{desc:<20} Rust missing: {r_path}")
            continue

        r = compare_pair(c_path, r_path, desc)
        all_results.append(r)

        if "error" in r:
            print(f"{desc:<20} {r['error']}")
            continue

        dim = f"{r['width']}x{r['height']}"

        if r["inv_diff"] == 0:
            verdict = "INVERSION ONLY"
        elif r["inv_diff"] < r["direct_diff"]:
            verdict = f"INV+BUG({r['inv_diff']}px)"
        else:
            verdict = f"ALG BUG({r['direct_diff']}px)"

        print(
            f"{desc:<20} {dim:>10} {r['total']:>8} "
            f"{r['c_ones']:>8} {r['r_ones']:>8} "
            f"{r['direct_diff']:>8} {r['direct_pct']:>6.2f}% "
            f"{r['inv_diff']:>8} {r['inv_pct']:>6.2f}% {verdict}"
        )

    print()
    print("=" * 105)
    inv_only = sum(1 for r in all_results if "error" not in r and r["inv_diff"] == 0)
    algo_bug = sum(1 for r in all_results if "error" not in r and r["inv_diff"] > 0)
    print(f"Pure inversion (no alg diff):   {inv_only}")
    print(f"Algorithm differences detected:  {algo_bug}")

    if algo_bug > 0:
        print("\n*** ALGORITHM BUGS DETECTED ***")
        for r in all_results:
            if "error" not in r and r["inv_diff"] > 0:
                print(
                    f"  {r['desc']}: {r['inv_diff']}/{r['total']} px "
                    f"({r['inv_pct']:.4f}%) differ after polarity correction"
                )


if __name__ == "__main__":
    main()
