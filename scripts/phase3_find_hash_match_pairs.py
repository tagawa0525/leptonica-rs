#!/usr/bin/env python3
"""Find Rust/C golden-manifest entries that share an FNV-1a content hash
and could be added to `scripts/golden_map.tsv` as `Ok` mappings.

Reads:
  - tests/golden_manifest.tsv   (Rust output hashes)
  - tests/golden_manifest_c.tsv (C verify-program output hashes)
  - scripts/golden_map.tsv      (existing Rust → C key mappings; skipped)

Reports one-to-one hash matches grouped by Rust prefix. Because the
hash space is 64-bit FNV-1a, accidental collisions are extremely
unlikely (~10^-9 for 580 × 1879 cross-checks), so a hash match
strongly implies the two outputs are pixel-identical and therefore
genuinely paired.

This script does NOT mutate any file: it only prints candidates. To
add them, edit `scripts/golden_map.tsv` by hand (see the "Phase 3
pairings" comment block in that file for the format).
"""

from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path


def parse_manifest(path: Path) -> dict[str, list[tuple[str, int, str]]]:
    """Return a mapping `hash -> [(prefix, index, full_name)]`."""
    out: dict[str, list[tuple[str, int, str]]] = defaultdict(list)
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.rstrip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        name, h = parts[0], parts[1]
        try:
            prefix, idx_str, _ext = name.rsplit(".", 2)
            idx = int(idx_str.lstrip("0") or "0")
        except ValueError:
            continue
        out[h].append((prefix, idx, name))
    return out


def parse_existing_map(path: Path) -> set[tuple[str, int]]:
    """Return the set of (rust_prefix, rust_index) already mapped."""
    existing: set[tuple[str, int]] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 5:
            try:
                existing.add((fields[3], int(fields[4])))
            except ValueError:
                pass
    return existing


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    rust = parse_manifest(root / "tests" / "golden_manifest.tsv")
    c_manifest = parse_manifest(root / "tests" / "golden_manifest_c.tsv")
    existing = parse_existing_map(root / "scripts" / "golden_map.tsv")

    by_module: dict[str, list[tuple]] = defaultdict(list)
    total = 0
    for h, rust_list in rust.items():
        if h not in c_manifest:
            continue
        c_list = c_manifest[h]
        if len(rust_list) != 1 or len(c_list) != 1:
            continue
        rp, ri, _rn = rust_list[0]
        cp, ci, _cn = c_list[0]
        if (rp, ri) in existing:
            continue
        total += 1
        by_module[rp].append((rp, ri, cp, ci, h[:12]))

    print(f"Hash-match candidates not yet in golden_map.tsv: {total}\n")
    if total == 0:
        print(
            "Nothing to add. Every Rust/C hash-equal pair is already "
            "registered. To extend coverage further, you need to map "
            "Rust ↔ C entries whose hashes differ (semantic-only "
            "matches), which require manual semantic confirmation."
        )
        return 0

    print(f"{'Rust prefix':30} {'Rust idx':>9}  ↔  {'C prefix':20} {'C idx':>5}  hash")
    print("-" * 80)
    for prefix in sorted(by_module):
        for rp, ri, cp, ci, h in by_module[prefix]:
            print(f"{rp:30} {ri:>9}  ↔  {cp:20} {ci:>5}  {h}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
