#!/usr/bin/env python3
"""Verify that comparison doc summary counts match actual table row counts.

Usage:
    python3 scripts/verify-comparison-counts.py
    python3 scripts/verify-comparison-counts.py core io
    python3 scripts/verify-comparison-counts.py --comparison-dir docs/porting/comparison
"""

import argparse
import re
import sys
from pathlib import Path

STATUS_EMOJIS = ("✅", "🔄", "🚫", "❌")
DEFAULT_COMPARISON_DIR = Path("docs/porting/comparison")
DEFAULT_MODULE_ORDER = [
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify comparison markdown summary counts against table rows."
    )
    parser.add_argument(
        "targets",
        nargs="*",
        help=(
            "Module names or markdown file paths. "
            "If omitted, all markdown files in --comparison-dir matching --glob are checked."
        ),
    )
    parser.add_argument(
        "--comparison-dir",
        type=Path,
        default=DEFAULT_COMPARISON_DIR,
        help="Base directory for module-name targets (default: docs/porting/comparison).",
    )
    parser.add_argument(
        "--glob",
        default="*.md",
        help="Glob pattern used when no explicit targets are provided (default: *.md).",
    )
    return parser.parse_args()


def module_sort_key(path: Path) -> tuple[int, str]:
    stem = path.stem
    if stem in DEFAULT_MODULE_ORDER:
        return DEFAULT_MODULE_ORDER.index(stem), stem
    return len(DEFAULT_MODULE_ORDER), stem


def resolve_targets(args: argparse.Namespace) -> list[Path]:
    if not args.targets:
        return sorted(args.comparison_dir.glob(args.glob), key=module_sort_key)

    resolved: list[Path] = []
    for target in args.targets:
        target_path = Path(target)
        if target_path.suffix.lower() == ".md":
            if target_path.exists():
                resolved.append(target_path)
            else:
                resolved.append(args.comparison_dir / target_path.name)
            continue
        resolved.append(args.comparison_dir / f"{target}.md")
    return resolved


def display_name(path: Path, comparison_dir: Path) -> str:
    try:
        return str(path.relative_to(comparison_dir))
    except ValueError:
        return str(path)


def count_table_statuses(path: Path) -> dict[str, int]:
    """Count status markers in table data rows."""
    counts: dict[str, int] = {emoji: 0 for emoji in STATUS_EMOJIS}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip("|").split("|")]
        if len(cols) < 3:
            continue
        status_col = cols[1]
        for emoji in STATUS_EMOJIS:
            if emoji in status_col:
                counts[emoji] += 1
                break
    return counts


def get_reported_summary(path: Path) -> dict[str, int]:
    """Extract self-reported summary values from the file."""
    result: dict[str, int] = {emoji: -1 for emoji in STATUS_EMOJIS}
    result["合計"] = -1

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip("|").split("|")]
        if len(cols) < 2:
            continue
        label = cols[0]
        number = re.search(r"\d+", cols[1])
        if not number:
            continue
        value = int(number.group(0))
        matched = re.match(r"^\s*([✅🔄🚫❌])", label)
        if matched:
            result[matched.group(1)] = value
        elif re.search(r"(合計|total)", label, flags=re.IGNORECASE):
            result["合計"] = value
    return result


def main() -> int:
    args = parse_args()
    targets = resolve_targets(args)
    if not targets:
        print(
            f"❌ No markdown files found (dir={args.comparison_dir}, glob={args.glob})"
        )
        return 1

    errors = 0
    totals_actual = {emoji: 0 for emoji in STATUS_EMOJIS}
    names = [display_name(path, args.comparison_dir) for path in targets]
    file_col_width = max(18, max(len(name) for name in names))

    print(
        f"{'file':<{file_col_width}} {'✅':>5} {'🔄':>5} {'🚫':>5} {'❌':>5} {'total':>6}  reported"
    )
    print("-" * (file_col_width + 57))

    for path, name in zip(targets, names):
        if not path.exists():
            print(
                f"{name:<{file_col_width}} {'-':>5} {'-':>5} {'-':>5} {'-':>5} {'-':>6}  NOT FOUND"
            )
            errors += 1
            continue

        actual = count_table_statuses(path)
        reported = get_reported_summary(path)
        actual_total = sum(actual.values())
        reported_total = reported.get("合計", -1)

        for k in STATUS_EMOJIS:
            totals_actual[k] += actual[k]

        mismatches = []
        for k in STATUS_EMOJIS:
            if actual[k] != reported.get(k, -1):
                mismatches.append(f"{k}:{actual[k]}≠{reported[k]}")
        if actual_total != reported_total:
            mismatches.append(f"total:{actual_total}≠{reported_total}")

        status = " ".join(mismatches) if mismatches else "OK"
        print(
            f"{name:<{file_col_width}} {actual['✅']:>5} {actual['🔄']:>5} "
            f"{actual['🚫']:>5} {actual['❌']:>5} {actual_total:>6}  {status}"
        )
        if mismatches:
            errors += 1

    print("-" * (file_col_width + 57))
    grand = sum(totals_actual.values())
    print(
        f"{'TOTAL':<{file_col_width}} {totals_actual['✅']:>5} {totals_actual['🔄']:>5} "
        f"{totals_actual['🚫']:>5} {totals_actual['❌']:>5} {grand:>6}"
    )

    if errors:
        print(f"\n❌ {errors} file(s) have mismatches")
        return 1
    print("\n✅ All counts match")
    return 0


if __name__ == "__main__":
    sys.exit(main())
