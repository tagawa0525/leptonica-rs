#!/usr/bin/env python3
"""Run C/Rust regression benchmarks and summarize per-test timing statistics.

This script targets the 145 tests executed by C Leptonica's `alltests_reg`
in the current environment, then maps Rust test timings to the same test IDs.

Outputs (default: target/benchmark-regression-145):
  - c_raw.json
  - rust_raw.json
  - summary.csv
  - summary.md
  - c/logs/run_XX.log
  - rust/logs/run_XX.log
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

RUST_TEST_BINARIES = [
    "core",
    "io",
    "morph",
    "transform",
    "filter",
    "color",
    "region",
    "recog",
]


@dataclass
class CommandResult:
    returncode: int
    output: str
    elapsed: float


def run_command(
    cmd: list[str],
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
) -> CommandResult:
    start = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    elapsed = time.perf_counter() - start
    result = CommandResult(proc.returncode, proc.stdout, elapsed)
    if check and proc.returncode != 0:
        raise RuntimeError(
            f"Command failed ({proc.returncode}): {' '.join(cmd)}\n{proc.stdout}"
        )
    return result


def find_first_in_nix_store(filename: str) -> Path | None:
    cmd = [
        "bash",
        "-lc",
        f"find /nix/store -type f -name '{filename}' 2>/dev/null | head -n1",
    ]
    result = run_command(cmd, check=False)
    found = result.output.strip()
    return Path(found) if found else None


def parse_pkg_include(cflags: str) -> str | None:
    for token in cflags.split():
        if token.startswith("-I"):
            return token[2:]
    return None


def find_library_file(libdir: str, patterns: Iterable[str]) -> str | None:
    root = Path(libdir)
    if not root.exists():
        return None
    for pattern in patterns:
        candidates = sorted(root.glob(pattern))
        if candidates:
            return str(candidates[0])
    return None


def extend_pkg_config_path(env: dict[str, str], pc_file: Path) -> None:
    pc_dir = str(pc_file.parent)
    existing = env.get("PKG_CONFIG_PATH", "")
    parts = [p for p in existing.split(":") if p]
    if pc_dir not in parts:
        parts.insert(0, pc_dir)
    env["PKG_CONFIG_PATH"] = ":".join(parts)


def pkg_exists(env: dict[str, str], package: str) -> bool:
    return (
        run_command(
            ["pkg-config", "--exists", package], env=env, check=False
        ).returncode
        == 0
    )


def pkg_get(env: dict[str, str], package: str, *args: str) -> str:
    result = run_command(["pkg-config", *args, package], env=env)
    return result.output.strip()


def configure_and_build_c(leptonica_dir: Path, build_dir: Path) -> None:
    env = os.environ.copy()

    # If pkg-config can't find optional image libs, discover *.pc in /nix/store.
    package_to_pc = {
        "libpng": "libpng.pc",
        "libjpeg": "libjpeg.pc",
        "libtiff-4": "libtiff-4.pc",
        "libwebp": "libwebp.pc",
        "libopenjp2": "libopenjp2.pc",
    }
    for pkg, pc_name in package_to_pc.items():
        if not pkg_exists(env, pkg):
            pc_path = find_first_in_nix_store(pc_name)
            if pc_path:
                extend_pkg_config_path(env, pc_path)

    required_pkgs = ["zlib", "libpng", "libjpeg", "libtiff-4"]
    missing = [pkg for pkg in required_pkgs if not pkg_exists(env, pkg)]
    if missing:
        raise RuntimeError(
            f"Missing required pkg-config packages: {', '.join(missing)}. "
            f"PKG_CONFIG_PATH={env.get('PKG_CONFIG_PATH', '')}"
        )

    zlib_inc = pkg_get(env, "zlib", "--variable=includedir")
    zlib_libdir = pkg_get(env, "zlib", "--variable=libdir")
    zlib_lib = find_library_file(zlib_libdir, ["libz.so*", "libz.a"])
    if not zlib_lib:
        raise RuntimeError(f"Unable to locate zlib library under {zlib_libdir}")

    png_inc = parse_pkg_include(pkg_get(env, "libpng", "--cflags"))
    png_libdir = pkg_get(env, "libpng", "--variable=libdir")
    png_lib = find_library_file(png_libdir, ["libpng*.so*", "libpng*.a"])
    if not png_inc or not png_lib:
        raise RuntimeError("Unable to resolve libpng include/library path")

    jpeg_inc = parse_pkg_include(pkg_get(env, "libjpeg", "--cflags"))
    jpeg_libdir = pkg_get(env, "libjpeg", "--variable=libdir")
    jpeg_lib = find_library_file(jpeg_libdir, ["libjpeg.so*", "libjpeg.a"])
    if not jpeg_inc or not jpeg_lib:
        raise RuntimeError("Unable to resolve libjpeg include/library path")

    tiff_inc = parse_pkg_include(pkg_get(env, "libtiff-4", "--cflags"))
    tiff_libdir = pkg_get(env, "libtiff-4", "--variable=libdir")
    tiff_lib = find_library_file(tiff_libdir, ["libtiff.so*", "libtiff.a"])
    if not tiff_inc or not tiff_lib:
        raise RuntimeError("Unable to resolve libtiff include/library path")

    gif_hdr = find_first_in_nix_store("gif_lib.h")
    gif_lib = find_first_in_nix_store("libgif.so")
    if not gif_lib:
        gif_lib = find_first_in_nix_store("libgif.a")

    webp_cfg = find_first_in_nix_store("WebPConfig.cmake")
    openjpeg_cfg = find_first_in_nix_store("OpenJPEGConfig.cmake")

    cmake_args = [
        "cmake",
        "-S",
        str(leptonica_dir),
        "-B",
        str(build_dir),
        "-DBUILD_PROG=ON",
        "-DCMAKE_BUILD_TYPE=Release",
        f"-DZLIB_INCLUDE_DIR={zlib_inc}",
        f"-DZLIB_LIBRARY={zlib_lib}",
        f"-DPNG_PNG_INCLUDE_DIR={png_inc}",
        f"-DPNG_LIBRARY={png_lib}",
        f"-DJPEG_INCLUDE_DIR={jpeg_inc}",
        f"-DJPEG_LIBRARY={jpeg_lib}",
        f"-DTIFF_INCLUDE_DIR={tiff_inc}",
        f"-DTIFF_LIBRARY={tiff_lib}",
    ]
    if gif_hdr and gif_lib:
        cmake_args.append(f"-DGIF_INCLUDE_DIR={gif_hdr.parent}")
        cmake_args.append(f"-DGIF_LIBRARY={gif_lib}")
    if webp_cfg:
        cmake_args.append(f"-DWebP_DIR={webp_cfg.parent}")
    if openjpeg_cfg:
        cmake_args.append(f"-DOpenJPEG_DIR={openjpeg_cfg.parent}")

    run_command(cmake_args, env=env)
    run_command(
        ["cmake", "--build", str(build_dir), "-j", str(os.cpu_count() or 1)],
        env=env,
    )

    build_bin = build_dir / "bin"
    build_bin.mkdir(parents=True, exist_ok=True)
    # Required for bytea_reg/string_reg and test images.
    src_prog = leptonica_dir / "prog"
    if shutil.which("rsync"):
        run_command(["rsync", "-a", f"{src_prog}/", f"{build_bin}/"])
    else:
        for item in src_prog.iterdir():
            target = build_bin / item.name
            if item.is_dir():
                shutil.copytree(item, target, dirs_exist_ok=True)
            else:
                shutil.copy2(item, target)


def parse_c_log(output: str) -> tuple[list[str], dict[str, float | None], list[str]]:
    section_re = re.compile(r"/+\s+([a-z0-9]+_reg)\s+/+")
    time_re = re.compile(r"^Time:\s*([0-9]+(?:\.[0-9]+)?)\s+sec\s*$")
    fail_re = re.compile(r"^Failed to complete\s+([a-z0-9]+_reg)\s*$")

    tests: list[str] = []
    final_times: dict[str, float | None] = {}
    failed: list[str] = []

    current: str | None = None
    current_times: list[float] = []
    for raw in output.splitlines():
        line = raw.strip()
        sec = section_re.search(line)
        if sec:
            if current is not None:
                final_times[current] = current_times[-1] if current_times else None
            current = sec.group(1)
            if current not in tests:
                tests.append(current)
            current_times = []
            continue
        if current is not None:
            tm = time_re.match(line)
            if tm:
                current_times.append(float(tm.group(1)))
        fm = fail_re.match(line)
        if fm:
            failed.append(fm.group(1))

    if current is not None:
        final_times[current] = current_times[-1] if current_times else None
    return tests, final_times, failed


def normalize_c_tests(tests: list[str]) -> list[str]:
    """Normalize to canonical 145 tests if optional JP2K/WebP tests are included."""
    optional = {"jp2kio_reg", "webpanimio_reg", "webpio_reg"}
    if len(tests) == 145:
        return tests
    filtered = [t for t in tests if t not in optional]
    if len(filtered) == 145:
        return filtered
    return tests


def run_c_benchmark(repo_root: Path, out_dir: Path, runs: int, rebuild: bool) -> dict:
    leptonica_dir = repo_root / "reference" / "leptonica"
    build_dir = leptonica_dir / "build-bench-145"
    build_bin = build_dir / "bin"
    logs_dir = out_dir / "c" / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    if rebuild and build_dir.exists():
        shutil.rmtree(build_dir)
    configure_and_build_c(leptonica_dir, build_dir)

    alltests = build_bin / "alltests_reg"
    if not alltests.exists():
        raise RuntimeError(f"Missing C test runner: {alltests}")

    tests: list[str] = []
    times_by_test: dict[str, list[float | None]] = {}
    run_failures: list[list[str]] = []
    suite_seconds: list[float] = []
    fallback_counts: dict[str, int] = {}

    for run_idx in range(1, runs + 1):
        regout = Path("/tmp/lept/regout")
        if regout.exists():
            shutil.rmtree(regout)
        regout.mkdir(parents=True, exist_ok=True)

        result = run_command([str(alltests), "display"], cwd=build_bin)
        log_path = logs_dir / f"run_{run_idx:02d}.log"
        log_path.write_text(result.output, encoding="utf-8")
        suite_seconds.append(result.elapsed)

        parsed_tests, parsed_times, failed = parse_c_log(result.output)
        run_failures.append(failed)
        if not tests:
            tests = normalize_c_tests(parsed_tests)
            times_by_test = {t: [] for t in tests}
            fallback_counts = {t: 0 for t in tests}
        for t in tests:
            v = parsed_times.get(t)
            if v is None:
                single = run_command(
                    [str(build_bin / t), "display"], cwd=build_bin, check=False
                )
                single_log = logs_dir / f"run_{run_idx:02d}_missing_{t}.log"
                single_log.write_text(single.output, encoding="utf-8")
                v = single.elapsed
                fallback_counts[t] = fallback_counts.get(t, 0) + 1
            times_by_test[t].append(v)

        print(
            f"[C] run {run_idx:02d}/{runs}: "
            f"suite={result.elapsed:.3f}s, failed={len(failed)}"
        )

    data = {
        "runs": runs,
        "tests": tests,
        "times_by_test": times_by_test,
        "suite_seconds": suite_seconds,
        "run_failures": run_failures,
        "fallback_counts": fallback_counts,
        "build_dir": str(build_dir),
    }
    (out_dir / "c_raw.json").write_text(
        json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"[C] wrote {out_dir / 'c_raw.json'}")
    return data


def parse_rust_report_time_log(output: str) -> dict[str, float]:
    # Example:
    # test boxa1_reg::boxa1_reg ... ok <0.000s>
    line_re = re.compile(
        r"^test\s+(\S+)\s+\.\.\.\s+(ok|FAILED|ignored)(?:,\s+|\s+)<([0-9]*\.?[0-9]+)s>$"
    )
    times: dict[str, float] = {}
    for raw in output.splitlines():
        m = line_re.match(raw.strip())
        if not m:
            continue
        name, status, sec = m.groups()
        if status == "ok":
            times[name] = float(sec)
    return times


def build_rust_mapping(
    c_tests: list[str], rust_test_times: dict[str, float]
) -> dict[str, list[str]]:
    all_names = sorted(rust_test_times.keys())
    mapping: dict[str, list[str]] = {}
    for c_test in c_tests:
        pat = re.compile(rf"(?:^|::){re.escape(c_test)}(?:$|::)")
        mapping[c_test] = [name for name in all_names if pat.search(name)]
    return mapping


def run_rust_single_filter(
    repo_root: Path,
    env: dict[str, str],
    c_test: str,
    logs_dir: Path,
    run_idx: int,
) -> float | None:
    base = [
        "cargo",
        "+nightly",
        "test",
        "--release",
        "--all-features",
        *sum([["--test", t] for t in RUST_TEST_BINARIES], []),
        c_test,
        "--",
    ]
    attempts = [
        ("ignored", ["--ignored", "--test-threads=1"]),
        ("normal", ["--test-threads=1"]),
    ]
    for label, tail in attempts:
        cmd = [*base, *tail]
        result = run_command(cmd, cwd=repo_root, env=env, check=False)
        log_path = logs_dir / f"run_{run_idx:02d}_missing_{c_test}_{label}.log"
        log_path.write_text(result.output, encoding="utf-8")
        running_counts = [
            int(x) for x in re.findall(r"running\s+(\d+)\s+test(?:s)?", result.output)
        ]
        if any(count > 0 for count in running_counts):
            # Even on non-zero exit (e.g., OOM), record elapsed.
            return result.elapsed
    return None


def run_rust_benchmark(
    repo_root: Path, out_dir: Path, runs: int, c_tests: list[str]
) -> dict:
    logs_dir = out_dir / "rust" / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["REGTEST_MODE"] = "display"

    base_cmd = [
        "cargo",
        "+nightly",
        "test",
        "--release",
        "--all-features",
        *sum([["--test", t] for t in RUST_TEST_BINARIES], []),
    ]
    run_command([*base_cmd, "--no-run"], cwd=repo_root, env=env)

    times_by_test: dict[str, list[float | None]] = {t: [] for t in c_tests}
    suite_seconds: list[float] = []
    mapping: dict[str, list[str]] | None = None
    missing_mapping: list[str] = []
    fallback_counts: dict[str, int] = {}

    for run_idx in range(1, runs + 1):
        regout = repo_root / "tests" / "regout"
        if regout.exists():
            shutil.rmtree(regout)
        regout.mkdir(parents=True, exist_ok=True)

        cmd = [
            *base_cmd,
            "--",
            "--report-time",
            "-Z",
            "unstable-options",
            "--test-threads=1",
        ]
        result = run_command(cmd, cwd=repo_root, env=env)
        log_path = logs_dir / f"run_{run_idx:02d}.log"
        log_path.write_text(result.output, encoding="utf-8")
        suite_seconds.append(result.elapsed)

        per_test_times = parse_rust_report_time_log(result.output)
        if mapping is None:
            mapping = build_rust_mapping(c_tests, per_test_times)
            missing_mapping = [k for k, v in mapping.items() if not v]
            fallback_counts = {k: 0 for k in missing_mapping}

        for c_test in c_tests:
            rust_tests = mapping.get(c_test, [])
            if not rust_tests:
                fallback_elapsed = run_rust_single_filter(
                    repo_root, env, c_test, logs_dir, run_idx
                )
                if fallback_elapsed is not None:
                    fallback_counts[c_test] = fallback_counts.get(c_test, 0) + 1
                times_by_test[c_test].append(fallback_elapsed)
                continue
            total = sum(per_test_times.get(name, 0.0) for name in rust_tests)
            times_by_test[c_test].append(total)

        print(
            f"[Rust] run {run_idx:02d}/{runs}: "
            f"suite={result.elapsed:.3f}s, mapped_missing={len(missing_mapping)}"
        )

    data = {
        "runs": runs,
        "tests": c_tests,
        "times_by_test": times_by_test,
        "suite_seconds": suite_seconds,
        "mapping": mapping or {},
        "mapping_missing": missing_mapping,
        "fallback_counts": fallback_counts,
    }
    (out_dir / "rust_raw.json").write_text(
        json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"[Rust] wrote {out_dir / 'rust_raw.json'}")
    return data


def calc_stats(samples: list[float | None]) -> dict[str, float | int | None]:
    valid = [x for x in samples if x is not None]
    if not valid:
        return {
            "valid_count": 0,
            "min_10": None,
            "max_10": None,
            "trimmed_mean_6": None,
            "trimmed_variance_6": None,
        }

    sorted_vals = sorted(valid)
    trimmed = sorted_vals[2:-2] if len(sorted_vals) >= 5 else []
    if trimmed:
        mean = statistics.fmean(trimmed)
        var = statistics.fmean([(x - mean) ** 2 for x in trimmed])
    else:
        mean = None
        var = None

    return {
        "valid_count": len(valid),
        "min_10": min(valid),
        "max_10": max(valid),
        "trimmed_mean_6": mean,
        "trimmed_variance_6": var,
    }


def fmt_num(v: float | int | None) -> str:
    if v is None:
        return "N/A"
    if isinstance(v, int):
        return str(v)
    return f"{v:.6f}"


def write_summary(
    out_dir: Path, runs: int, c_data: dict, rust_data: dict, markdown_path: Path | None
) -> None:
    tests = c_data["tests"]
    rows: list[dict[str, str]] = []
    for test in tests:
        c_stats = calc_stats(c_data["times_by_test"].get(test, []))
        r_stats = calc_stats(rust_data["times_by_test"].get(test, []))
        rows.append(
            {
                "test": test,
                "c_valid": fmt_num(c_stats["valid_count"]),
                "c_min_10": fmt_num(c_stats["min_10"]),
                "c_max_10": fmt_num(c_stats["max_10"]),
                "c_trimmed_mean_6": fmt_num(c_stats["trimmed_mean_6"]),
                "c_trimmed_variance_6": fmt_num(c_stats["trimmed_variance_6"]),
                "rust_valid": fmt_num(r_stats["valid_count"]),
                "rust_min_10": fmt_num(r_stats["min_10"]),
                "rust_max_10": fmt_num(r_stats["max_10"]),
                "rust_trimmed_mean_6": fmt_num(r_stats["trimmed_mean_6"]),
                "rust_trimmed_variance_6": fmt_num(r_stats["trimmed_variance_6"]),
            }
        )

    csv_path = out_dir / "summary.csv"
    with csv_path.open("w", encoding="utf-8", newline="") as f:
        fieldnames = list(rows[0].keys()) if rows else []
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    md_path = markdown_path if markdown_path else out_dir / "summary.md"
    md_path.parent.mkdir(parents=True, exist_ok=True)
    with md_path.open("w", encoding="utf-8") as f:
        f.write("# C/Rust Regression Benchmark (145 tests)\n\n")
        f.write(f"- runs per side: {runs}\n")
        f.write(
            "- rule: remove 2 smallest + 2 largest, then compute mean/variance on remaining 6\n"
        )
        f.write("- variance is population variance over the trimmed 6 samples\n\n")
        f.write(
            "| test | C mean(6) | C var(6) | C min(10) | C max(10) | "
            "Rust mean(6) | Rust var(6) | Rust min(10) | Rust max(10) |\n"
        )
        f.write("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n")
        for row in rows:
            f.write(
                "| "
                f"{row['test']} | {row['c_trimmed_mean_6']} | {row['c_trimmed_variance_6']} | "
                f"{row['c_min_10']} | {row['c_max_10']} | "
                f"{row['rust_trimmed_mean_6']} | {row['rust_trimmed_variance_6']} | "
                f"{row['rust_min_10']} | {row['rust_max_10']} |\n"
            )

    print(f"[Compare] wrote {csv_path}")
    print(f"[Compare] wrote {md_path}")


def load_json(path: Path) -> dict:
    if not path.exists():
        raise RuntimeError(f"Missing input file: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=["all", "c", "rust", "compare"],
        help="Run C benchmarks, Rust benchmarks, compare, or all in order.",
    )
    parser.add_argument("--runs", type=int, default=10, help="Number of runs per side.")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("target/benchmark-regression-145"),
        help="Output directory for logs/raw/summary files.",
    )
    parser.add_argument(
        "--markdown-path",
        type=Path,
        default=Path("docs/porting/c-rust-regression-benchmark-145.md"),
        help="Output markdown report path.",
    )
    parser.add_argument(
        "--rebuild-c",
        action="store_true",
        help="Delete C build dir and rebuild from scratch.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    out_dir = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.mode in {"all", "c"}:
        c_data = run_c_benchmark(repo_root, out_dir, args.runs, rebuild=args.rebuild_c)
    else:
        c_data = load_json(out_dir / "c_raw.json")

    if args.mode in {"all", "rust"}:
        c_tests = c_data["tests"]
        rust_data = run_rust_benchmark(repo_root, out_dir, args.runs, c_tests)
    else:
        rust_data = load_json(out_dir / "rust_raw.json")

    if args.mode in {"all", "compare"}:
        write_summary(out_dir, args.runs, c_data, rust_data, args.markdown_path)

    return 0


if __name__ == "__main__":
    sys.exit(main())
