#!/usr/bin/env python3
"""Fix common markdownlint errors in markdown files.

Usage:
    # Fix specific files
    python3 scripts/fix-markdown-lint.py docs/plans/001_*.md docs/porting/*.md

    # Fix all docs
    find docs/ -name '*.md' | xargs python3 scripts/fix-markdown-lint.py

    # Dry-run (show what would change)
    python3 scripts/fix-markdown-lint.py --dry-run docs/**/*.md

Fixes:
    MD004  Unordered list style (normalize to -)
    MD012  No multiple blank lines
    MD022  Blank lines around headings
    MD031  Blank lines around fenced code blocks
    MD032  Blank lines around lists
    MD040  Fenced code blocks should have a language specified
    MD058  Blank lines around tables
    MD060  Table column alignment (CJK full-width aware)
"""

import re
import sys
import unicodedata
from pathlib import Path


def display_width(s: str) -> int:
    """Calculate display width accounting for CJK wide characters."""
    w = 0
    for c in s:
        eaw = unicodedata.east_asian_width(c)
        if eaw in ("F", "W"):
            w += 2
        else:
            w += 1
    return w


def is_table_row(line: str) -> bool:
    stripped = line.strip()
    return stripped.startswith("|") and stripped.endswith("|") and "|" in stripped[1:-1]


def is_separator_row(line: str) -> bool:
    if not is_table_row(line):
        return False
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    return all(re.match(r"^:?-+:?$", c) for c in cells if c)


def is_heading(line: str) -> bool:
    return bool(re.match(r"^#{1,6}\s", line))


def is_list_item(line: str) -> bool:
    return bool(re.match(r"^\s*[-*+]\s", line) or re.match(r"^\s*\d+[.)]\s", line))


def is_fenced_code(line: str) -> bool:
    return line.strip().startswith("```")


def format_table(table_lines: list[str]) -> list[str]:
    """Format a markdown table with aligned columns (CJK-aware)."""
    rows = []
    for line in table_lines:
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        rows.append(cells)

    if len(rows) < 2:
        return table_lines

    ncols = len(rows[0])

    # Detect alignment from separator row
    alignments = []
    for cell in rows[1]:
        stripped = cell.strip()
        if stripped.startswith(":") and stripped.endswith(":"):
            alignments.append("center")
        elif stripped.endswith(":"):
            alignments.append("right")
        else:
            alignments.append("left")
    while len(alignments) < ncols:
        alignments.append("left")

    # Calculate column widths (skip separator row)
    col_widths = [0] * ncols
    for ri, row in enumerate(rows):
        if ri == 1:
            continue
        for i, cell in enumerate(row):
            if i < ncols:
                w = display_width(cell)
                if w > col_widths[i]:
                    col_widths[i] = w

    # Build formatted rows
    result = []
    for ri, row in enumerate(rows):
        parts = []
        for i in range(ncols):
            cell = row[i] if i < len(row) else ""
            cw = col_widths[i]

            if ri == 1:  # separator
                if alignments[i] == "center":
                    parts.append(" :" + "-" * (cw - 2) + ": ")
                elif alignments[i] == "right":
                    parts.append(" " + "-" * (cw - 1) + ": ")
                else:
                    parts.append(" " + "-" * cw + " ")
                continue

            dw = display_width(cell)
            padding = cw - dw

            if alignments[i] == "right":
                parts.append(" " * (padding + 1) + cell + " ")
            elif alignments[i] == "center":
                lpad = padding // 2
                rpad = padding - lpad
                parts.append(" " * (lpad + 1) + cell + " " * (rpad + 1))
            else:
                parts.append(" " + cell + " " * (padding + 1))

        result.append("|" + "|".join(parts) + "|")

    return result


def guess_language(content_lines: list[str]) -> str:
    """Guess the language of a fenced code block from its content."""
    sample = "\n".join(content_lines)
    if re.search(
        r"(cargo |fn |let |use |pub |mod |impl |struct |enum |trait |#\[)", sample
    ):
        return "rust"
    if re.search(r"(git |gh |npm |pip |mkdir |cd |ls |rm |cp |mv )", sample):
        return "bash"
    return "text"


def fix_markdown(content: str) -> str:
    lines = content.split("\n")

    # Pass 1: Format tables (MD060) and fix list markers (MD004)
    output: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]

        # MD060: Format tables
        if is_table_row(line) and i + 1 < len(lines) and is_separator_row(lines[i + 1]):
            table = []
            j = i
            while j < len(lines) and is_table_row(lines[j]):
                table.append(lines[j])
                j += 1
            output.extend(format_table(table))
            i = j
            continue

        # MD004: Normalize list markers to -
        if re.match(r"^(\s*)[*+]\s", line):
            line = re.sub(r"^(\s*)[*+](\s)", r"\1-\2", line)

        output.append(line)
        i += 1

    # Pass 2: Fix fenced code block languages (MD040)
    lines = output
    output = []
    in_code = False
    for i, line in enumerate(lines):
        if line.strip().startswith("```"):
            if not in_code:
                if line.strip() == "```":
                    # Find content to guess language
                    content_lines = []
                    for j in range(i + 1, len(lines)):
                        if lines[j].strip().startswith("```"):
                            break
                        content_lines.append(lines[j])
                    lang = guess_language(content_lines)
                    output.append(f"```{lang}")
                else:
                    output.append(line)
                in_code = True
            else:
                output.append(line)
                in_code = False
        else:
            output.append(line)

    # Pass 3: Remove multiple blank lines (MD012)
    lines = output
    output = []
    for line in lines:
        if line.strip() == "" and output and output[-1].strip() == "":
            continue
        output.append(line)

    # Pass 4: Ensure blank lines around headings/lists/tables/fences (MD022/31/32/58)
    lines = output
    output = []
    in_code = False

    for i, line in enumerate(lines):
        prev = output[-1] if output else ""
        prev_blank = prev.strip() == ""
        at_start = len(output) <= 1

        if is_fenced_code(line):
            if not in_code:
                if output and not prev_blank and not at_start:
                    output.append("")
                in_code = True
            else:
                in_code = False
                output.append(line)
                if i + 1 < len(lines) and lines[i + 1].strip() != "":
                    output.append("")
                continue

        if not in_code:
            if is_heading(line) and output and not prev_blank and not at_start:
                output.append("")
            if (
                is_list_item(line)
                and output
                and not prev_blank
                and not at_start
                and not is_list_item(prev)
            ):
                output.append("")
            if (
                is_table_row(line)
                and output
                and not prev_blank
                and not at_start
                and not is_table_row(prev)
            ):
                output.append("")

        output.append(line)

        if not in_code and i + 1 < len(lines):
            next_line = lines[i + 1]
            next_blank = next_line.strip() == ""
            if is_heading(line) and not next_blank:
                output.append("")
            if is_list_item(line) and not next_blank and not is_list_item(next_line):
                output.append("")
            if is_table_row(line) and not next_blank and not is_table_row(next_line):
                output.append("")

    # Pass 5: Final cleanup of double blank lines
    lines = output
    output = []
    for line in lines:
        if line.strip() == "" and output and output[-1].strip() == "":
            continue
        output.append(line)

    return "\n".join(output)


def main():
    dry_run = "--dry-run" in sys.argv
    files = [a for a in sys.argv[1:] if not a.startswith("--")]

    if not files:
        print(__doc__)
        sys.exit(1)

    fixed_count = 0
    for filepath in files:
        path = Path(filepath)
        if not path.exists():
            print(f"Skip: {filepath} (not found)")
            continue

        content = path.read_text()
        fixed = fix_markdown(content)

        if content != fixed:
            if dry_run:
                print(f"Would fix: {filepath}")
            else:
                path.write_text(fixed)
                print(f"Fixed: {filepath}")
            fixed_count += 1
        else:
            print(f"OK: {filepath}")

    if dry_run and fixed_count:
        print(
            f"\n{fixed_count} file(s) would be modified. Run without --dry-run to apply."
        )


if __name__ == "__main__":
    main()
