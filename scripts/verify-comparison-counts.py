#!/usr/bin/env python3
"""Verify that comparison doc summary counts match actual table row counts.

Usage:
    python3 scripts/verify-comparison-counts.py
"""

import re
import sys
from pathlib import Path

COMPARISON_DIR = Path("docs/porting/comparison")
MODULES = [
    "core",
    "io",
    "transform",
    "morph",
    "filter",
    "color",
    "region",
    "recog",
    "misc",
]


def count_table_statuses(path: Path) -> dict[str, int]:
    """Count status markers in table data rows."""
    counts: dict[str, int] = {"✅": 0, "🔄": 0, "🚫": 0, "❌": 0}
    skip_patterns = re.compile(r"^\|\s*(C関数|項目|ステータス|合計|---)")
    for line in path.read_text().splitlines():
        if not line.startswith("|"):
            continue
        if skip_patterns.match(line):
            continue
        cols = [c.strip() for c in line.split("|")]
        if len(cols) < 3:
            continue
        status_col = cols[2]
        for emoji in counts:
            if emoji in status_col:
                counts[emoji] += 1
                break
    return counts


def get_reported_summary(path: Path) -> dict[str, int]:
    """Extract self-reported summary values from the file."""
    text = path.read_text()
    result: dict[str, int] = {}
    patterns = {
        "✅": r"\|\s*✅\s*同等\s*\|\s*(\d+)",
        "🔄": r"\|\s*🔄\s*異なる\s*\|\s*(\d+)",
        "🚫": r"\|\s*🚫\s*不要\s*\|\s*(\d+)",
        "❌": r"\|\s*❌\s*未実装\s*\|\s*(\d+)",
        "合計": r"\|\s*合計\s*\|\s*(\d+)",
    }
    for key, pat in patterns.items():
        m = re.search(pat, text)
        result[key] = int(m.group(1)) if m else -1
    return result


def main() -> int:
    errors = 0
    totals_actual = {"✅": 0, "🔄": 0, "🚫": 0, "❌": 0}
    totals_reported = {"✅": 0, "🔄": 0, "🚫": 0, "❌": 0}

    print(
        f"{'file':<18} {'✅':>5} {'🔄':>5} {'🚫':>5} {'❌':>5} {'total':>6}  reported"
    )
    print("-" * 75)

    for mod in MODULES:
        path = COMPARISON_DIR / f"{mod}.md"
        if not path.exists():
            print(f"{mod}.md: NOT FOUND")
            errors += 1
            continue

        actual = count_table_statuses(path)
        reported = get_reported_summary(path)
        actual_total = sum(actual.values())
        reported_total = reported.get("合計", -1)

        for k in totals_actual:
            totals_actual[k] += actual[k]
            totals_reported[k] += reported.get(k, 0)

        mismatches = []
        for k in ["✅", "🔄", "🚫", "❌"]:
            if actual[k] != reported.get(k, -1):
                mismatches.append(f"{k}:{actual[k]}≠{reported[k]}")
        if actual_total != reported_total:
            mismatches.append(f"total:{actual_total}≠{reported_total}")

        status = " ".join(mismatches) if mismatches else "OK"
        print(
            f"{mod + '.md':<18} {actual['✅']:>5} {actual['🔄']:>5} "
            f"{actual['🚫']:>5} {actual['❌']:>5} {actual_total:>6}  {status}"
        )
        if mismatches:
            errors += 1

    print("-" * 75)
    grand = sum(totals_actual.values())
    print(
        f"{'TOTAL':<18} {totals_actual['✅']:>5} {totals_actual['🔄']:>5} "
        f"{totals_actual['🚫']:>5} {totals_actual['❌']:>5} {grand:>6}"
    )

    if errors:
        print(f"\n❌ {errors} file(s) have mismatches")
        return 1
    print("\n✅ All counts match")
    return 0


if __name__ == "__main__":
    sys.exit(main())
