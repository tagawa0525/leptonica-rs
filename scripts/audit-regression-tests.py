#!/usr/bin/env python3
"""Audit regression test coverage: compare C and Rust check counts.

Scans C reference tests (reference/leptonica/prog/*_reg.c) and Rust tests
(tests/*/**_reg.rs) to count verification calls, then outputs a CSV sorted
by divergence score (highest first).

Outputs:
  - docs/porting/regression-test-audit.csv

Usage:
  python3 scripts/audit-regression-tests.py
"""

from __future__ import annotations

import csv
import re
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
C_PROG_DIR = REPO_ROOT / "reference" / "leptonica" / "prog"
RUST_TEST_DIR = REPO_ROOT / "tests"

# alltests_reg.c typically lists 149 tests (including optional jp2kio, webpanimio,
# webpio which may not build if the corresponding libraries are absent). The C
# benchmark script normalises this down to 145 by stripping those three when they
# are present, but this audit script intentionally includes all of them so that
# every entry in alltests_reg.c has a corresponding row in the CSV.
ALLTESTS_REG = C_PROG_DIR / "alltests_reg.c"

# C-side verification function patterns
C_CHECK_PATTERNS = {
    "regTestWritePixAndCheck": re.compile(r"\bregTestWritePixAndCheck\s*\("),
    "regTestCompareValues": re.compile(r"\bregTestCompareValues\s*\("),
    "regTestComparePix": re.compile(r"\bregTestComparePix\s*\("),
    "regTestCheckFile": re.compile(r"\bregTestCheckFile\s*\("),
}

# Rust-side verification function patterns
RUST_CHECK_PATTERNS = {
    "write_pix_and_check": re.compile(r"\bwrite_pix_and_check\s*\("),
    "compare_pix": re.compile(r"\bcompare_pix\s*\("),
    "compare_values": re.compile(r"\bcompare_values\s*\("),
    "compare_strings": re.compile(r"\bcompare_strings\s*\("),
    "write_data_and_check": re.compile(r"\bwrite_data_and_check\s*\("),
}

RUST_TEST_FN_RE = re.compile(r"#\[test\]")
RUST_IGNORE_RE = re.compile(r"#\[ignore")
RUST_LOAD_IMAGE_RE = re.compile(r"\bload_test_image\s*\(")
RUST_REGPARAMS_RE = re.compile(r"\bRegParams::new\s*\(")


@dataclass
class CTestStats:
    name: str
    file: Path
    write_pix_and_check: int = 0
    compare_values: int = 0
    compare_pix: int = 0
    check_file: int = 0

    @property
    def total_checks(self) -> int:
        return (
            self.write_pix_and_check
            + self.compare_values
            + self.compare_pix
            + self.check_file
        )


@dataclass
class RustTestStats:
    name: str
    files: list[Path] = field(default_factory=list)
    test_fn_count: int = 0
    ignore_count: int = 0
    write_pix_and_check: int = 0
    compare_pix: int = 0
    compare_values: int = 0
    compare_strings: int = 0
    write_data_and_check: int = 0
    has_load_test_image: bool = False
    has_regparams: bool = False

    @property
    def total_checks(self) -> int:
        return (
            self.write_pix_and_check
            + self.compare_pix
            + self.compare_values
            + self.compare_strings
            + self.write_data_and_check
        )


def get_alltests_names() -> list[str]:
    """Parse alltests_reg.c to get the canonical test list."""
    if not ALLTESTS_REG.exists():
        raise FileNotFoundError(
            f"Missing required file: {ALLTESTS_REG}\n"
            "The C leptonica source is not available.\n"
            "Please clone it manually:\n"
            "    git clone https://github.com/DanBloomberg/leptonica.git reference/leptonica"
        )
    content = ALLTESTS_REG.read_text(encoding="utf-8")
    names = re.findall(r'"([a-z0-9_]+)_reg"', content)
    return names


def scan_c_test(test_name: str) -> CTestStats | None:
    """Scan a C regression test file for verification calls."""
    c_file = C_PROG_DIR / f"{test_name}_reg.c"
    if not c_file.exists():
        return None

    content = c_file.read_text(encoding="utf-8")
    stats = CTestStats(name=test_name, file=c_file)
    stats.write_pix_and_check = len(
        C_CHECK_PATTERNS["regTestWritePixAndCheck"].findall(content)
    )
    stats.compare_values = len(
        C_CHECK_PATTERNS["regTestCompareValues"].findall(content)
    )
    stats.compare_pix = len(C_CHECK_PATTERNS["regTestComparePix"].findall(content))
    stats.check_file = len(C_CHECK_PATTERNS["regTestCheckFile"].findall(content))
    return stats


def find_rust_test_files(test_name: str) -> list[Path]:
    """Find all Rust test files matching a C test name."""
    files = []
    for module_dir in RUST_TEST_DIR.iterdir():
        if not module_dir.is_dir() or not (module_dir / "main.rs").exists():
            continue
        # Exact match: {test_name}_reg.rs
        exact = module_dir / f"{test_name}_reg.rs"
        if exact.exists():
            files.append(exact)
    return files


def scan_rust_test(test_name: str) -> RustTestStats:
    """Scan Rust test files for verification calls."""
    stats = RustTestStats(name=test_name)
    files = find_rust_test_files(test_name)
    stats.files = files

    for f in files:
        content = f.read_text(encoding="utf-8")
        stats.test_fn_count += len(RUST_TEST_FN_RE.findall(content))
        stats.ignore_count += len(RUST_IGNORE_RE.findall(content))
        stats.write_pix_and_check += len(
            RUST_CHECK_PATTERNS["write_pix_and_check"].findall(content)
        )
        stats.compare_pix += len(RUST_CHECK_PATTERNS["compare_pix"].findall(content))
        stats.compare_values += len(
            RUST_CHECK_PATTERNS["compare_values"].findall(content)
        )
        stats.compare_strings += len(
            RUST_CHECK_PATTERNS["compare_strings"].findall(content)
        )
        stats.write_data_and_check += len(
            RUST_CHECK_PATTERNS["write_data_and_check"].findall(content)
        )
        if RUST_LOAD_IMAGE_RE.search(content):
            stats.has_load_test_image = True
        if RUST_REGPARAMS_RE.search(content):
            stats.has_regparams = True

    return stats


def divergence_score(c: CTestStats | None, r: RustTestStats) -> float:
    """Calculate divergence score. Higher = more divergent from C version.

    The score is composed of:
    - missing_ratio: fraction of total C checks absent from Rust (0-1)
    - pixel_gap: fraction of C pixel/file checks (writePixAndCheck + checkFile)
      not covered by Rust pixel checks (write_pix_and_check + compare_pix).
      Weighted 0.5 because these represent verification *quality*, not just
      quantity – a test that replaces all pixel checks with value-only checks
      appears equivalent by count but is actually much weaker.
    - no_image_penalty: 0.2 if Rust test never loads a real image
    - no_regparams_penalty: 0.1 if Rust test never instantiates RegParams
    - all_ignored_penalty: 0.3 if every Rust #[test] fn is #[ignore]
    - no_file_penalty: 1.0 if no Rust file exists at all
    """
    if c is None:
        return 0.0  # No C test to compare against

    c_total = c.total_checks

    if c_total == 0:
        return 0.0

    r_total = r.total_checks

    # Base: ratio of missing checks (quantity gap)
    missing_ratio = max(0, c_total - r_total) / c_total

    # Quality gap: pixel/file checks in C not covered by pixel checks in Rust
    c_pixel = c.write_pix_and_check + c.check_file
    r_pixel = r.write_pix_and_check + r.compare_pix
    if c_pixel > 0:
        pixel_gap = max(0, c_pixel - r_pixel) / c_pixel * 0.5
    else:
        pixel_gap = 0.0

    # Penalty for no load_test_image when C test exists
    no_image_penalty = 0.2 if not r.has_load_test_image else 0.0

    # Penalty for no RegParams
    no_regparams_penalty = 0.1 if not r.has_regparams else 0.0

    # Penalty for all tests ignored
    all_ignored_penalty = (
        0.3 if r.test_fn_count > 0 and r.ignore_count == r.test_fn_count else 0.0
    )

    # Penalty for no Rust file at all
    no_file_penalty = 1.0 if not r.files else 0.0

    score = (
        missing_ratio
        + pixel_gap
        + no_image_penalty
        + no_regparams_penalty
        + all_ignored_penalty
        + no_file_penalty
    )
    return round(min(score, 2.0), 3)


def rust_module_for(test_name: str) -> str:
    """Determine which Rust test module contains this test."""
    for module_dir in RUST_TEST_DIR.iterdir():
        if not module_dir.is_dir() or not (module_dir / "main.rs").exists():
            continue
        if (module_dir / f"{test_name}_reg.rs").exists():
            return module_dir.name
    return ""


def main() -> None:
    test_names = get_alltests_names()
    print(f"Found {len(test_names)} tests in alltests_reg.c")

    rows: list[dict] = []
    for test_name in test_names:
        c_stats = scan_c_test(test_name)
        r_stats = scan_rust_test(test_name)
        score = divergence_score(c_stats, r_stats)
        module = rust_module_for(test_name)

        rows.append(
            {
                "test": test_name,
                "module": module,
                "c_write_pix_and_check": c_stats.write_pix_and_check if c_stats else 0,
                "c_compare_values": c_stats.compare_values if c_stats else 0,
                "c_compare_pix": c_stats.compare_pix if c_stats else 0,
                "c_check_file": c_stats.check_file if c_stats else 0,
                "c_total": c_stats.total_checks if c_stats else 0,
                "rust_test_fns": r_stats.test_fn_count,
                "rust_ignored": r_stats.ignore_count,
                "rust_write_pix_and_check": r_stats.write_pix_and_check,
                "rust_compare_pix": r_stats.compare_pix,
                "rust_compare_values": r_stats.compare_values,
                "rust_compare_strings": r_stats.compare_strings,
                "rust_write_data_and_check": r_stats.write_data_and_check,
                "rust_total": r_stats.total_checks,
                "rust_load_image": "Y" if r_stats.has_load_test_image else "N",
                "rust_regparams": "Y" if r_stats.has_regparams else "N",
                "divergence": score,
            }
        )

    # Sort by divergence descending
    rows.sort(key=lambda r: (-r["divergence"], r["test"]))

    # Write CSV
    out_dir = REPO_ROOT / "docs" / "porting"
    out_dir.mkdir(parents=True, exist_ok=True)
    csv_path = out_dir / "regression-test-audit.csv"

    if not rows:
        raise RuntimeError(
            "No tests found. Check that alltests_reg.c contains '\"*_reg\"' entries."
        )

    fieldnames = list(rows[0].keys())
    with open(csv_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    print(f"Wrote {csv_path} ({len(rows)} tests)")

    # Summary
    high = sum(1 for r in rows if r["divergence"] >= 1.0)
    medium = sum(1 for r in rows if 0.3 <= r["divergence"] < 1.0)
    low = sum(1 for r in rows if 0.0 < r["divergence"] < 0.3)
    zero = sum(1 for r in rows if r["divergence"] == 0.0)

    print("\nDivergence summary:")
    print(f"  High (>= 1.0):   {high}")
    print(f"  Medium (0.3-1.0): {medium}")
    print(f"  Low (0.0-0.3):   {low}")
    print(f"  Zero:            {zero}")

    # Top 20
    print("\nTop 20 most divergent:")
    print(f"{'test':<25} {'module':<10} {'C':>5} {'Rust':>5} {'div':>7}")
    print("-" * 55)
    for r in rows[:20]:
        print(
            f"{r['test']:<25} {r['module']:<10} {r['c_total']:>5} {r['rust_total']:>5} {r['divergence']:>7.3f}"
        )


if __name__ == "__main__":
    main()
