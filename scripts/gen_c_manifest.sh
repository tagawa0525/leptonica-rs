#!/usr/bin/env bash
# Run C-version regression tests in 'generate' mode and update
# tests/golden_manifest_c.tsv via examples/gen_c_manifest.
#
# By default runs every prog/*_reg binary except those in SKIP_REGS;
# pass test names (without _reg suffix) to limit the run, e.g.:
#
#     scripts/gen_c_manifest.sh             # run everything
#     scripts/gen_c_manifest.sh edge io     # run edge_reg and io_reg only
#
# Honours LEPT_REGOUT_DIR (defaults to /tmp/lept/regout).
# See docs/plans/901_c-hash-compat.md.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LEPT_DIR="$ROOT/reference/leptonica"
BIN_DIR="$LEPT_DIR/build/bin"
PROG_DIR="$LEPT_DIR/prog"
C_OUT_DIR="${LEPT_REGOUT_DIR:-/tmp/lept/regout}"
MANIFEST_OUT="$ROOT/tests/golden_manifest_c.tsv"

# Regression binaries that should never be invoked with the bare 'generate'
# argument: either they are wrappers that re-invoke the others, or their
# argv[1] is interpreted as something other than the regtest mode keyword.
# Skipping them here keeps `failures` reserved for genuine regressions.
SKIP_REGS=(
    alltests_reg     # wrapper: system()s every other *_reg (alltests_reg.c:252)
    binmorph2_reg    # rejects 'generate' (different CLI surface)
    dwamorph2_reg    # rejects 'generate'
    fmorphauto_reg   # treats argv[1] as a file path, not the mode keyword
    morphseq_reg     # rejects 'generate'
)

skip_match() {
    local name="$1"
    local s
    for s in "${SKIP_REGS[@]}"; do
        [[ "$name" == "$s" ]] && return 0
    done
    return 1
}

if [[ ! -d "$BIN_DIR" ]] || ! compgen -G "$BIN_DIR/*_reg" > /dev/null; then
    echo "C leptonica not built at $BIN_DIR." >&2
    echo "Run scripts/build_c_leptonica.sh first." >&2
    exit 1
fi

if [[ ! -d "$PROG_DIR" ]]; then
    echo "prog/ directory not found at $PROG_DIR." >&2
    exit 1
fi

# Build the binary list. Empty positional args ⇒ run everything (full run);
# in that case we wipe the regout dir so the manifest reflects only outputs
# from this invocation. For a scoped run, keep the directory but warn the
# operator that pre-existing files will leak into the manifest.
binaries=()
if [[ "$#" -eq 0 ]]; then
    echo "==> cleaning $C_OUT_DIR (full run)"
    rm -rf "$C_OUT_DIR"
    mkdir -p "$C_OUT_DIR"
    while IFS= read -r -d '' bin; do
        if skip_match "$(basename "$bin")"; then
            continue
        fi
        binaries+=("$bin")
    done < <(find "$BIN_DIR" -maxdepth 1 -type f -name '*_reg' -print0 | sort -z)
else
    mkdir -p "$C_OUT_DIR"
    if compgen -G "$C_OUT_DIR/*" > /dev/null; then
        echo "warning: $C_OUT_DIR already contains files." >&2
        echo "         Outputs from earlier runs will be hashed into the manifest" >&2
        echo "         alongside this scoped run. Run with no args (or rm -rf the" >&2
        echo "         dir) for a clean baseline." >&2
    fi
    for name in "$@"; do
        candidate="$BIN_DIR/${name}_reg"
        if [[ ! -x "$candidate" ]]; then
            echo "warning: $candidate not found, skipping" >&2
            continue
        fi
        if skip_match "${name}_reg"; then
            echo "warning: ${name}_reg is in SKIP_REGS, skipping" >&2
            continue
        fi
        binaries+=("$candidate")
    done
fi

if [[ "${#binaries[@]}" -eq 0 ]]; then
    echo "No regression binaries selected." >&2
    exit 1
fi

cd "$PROG_DIR"

failures=()
for bin in "${binaries[@]}"; do
    name="$(basename "$bin")"
    echo "==> $name generate"
    if ! "$bin" generate > /tmp/lept_run.log 2>&1; then
        echo "    FAILED ($name) — last 10 lines:"
        tail -10 /tmp/lept_run.log | sed 's/^/    /'
        failures+=("$name")
    fi
done

echo
echo "Ran ${#binaries[@]} regression binaries (failures: ${#failures[@]})"
for f in "${failures[@]}"; do
    echo "  - $f"
done

# Refuse to overwrite the manifest if any C binary failed: the resulting
# /tmp/lept/regout would be missing the corresponding outputs and a stale
# baseline could be committed silently.
if [[ "${#failures[@]}" -gt 0 ]]; then
    echo
    echo "Aborting: ${#failures[@]} regression binary(ies) failed; manifest not regenerated." >&2
    echo "         Fix the failure or extend SKIP_REGS if 'generate' is unsupported." >&2
    exit 1
fi

# Phase 1.5: run scripts/verify_*.c programs to obtain C outputs for the
# assertion-only regressions (binmorph1/3, fhmtauto, graymorph2) that
# prog/*_reg never emits. Only happens on a full run (no positional args)
# because scoped runs by definition target a subset of prog/*_reg.
VERIFY_BIN_DIR="/tmp/lept_verify"
VERIFY_MAP="$ROOT/scripts/c_verify_outputs.tsv"

if [[ "$#" -eq 0 ]]; then
    echo
    echo "==> building verify_*.c (idempotent)"
    bash "$ROOT/scripts/build_c_verify.sh"

    echo "==> running verify_*.c programs (cwd: $PROG_DIR)"
    # Clean any stale /tmp/c_*.tif / .jpg from previous runs so the copy
    # step below only sees outputs from this invocation.
    rm -f /tmp/c_binmorph1_*.tif /tmp/c_binmorph3_*.tif \
          /tmp/c_fhmtauto_*.tif /tmp/c_graymorph2_*.jpg
    verify_failed=()
    for prog in verify_binmorph verify_fhmtauto verify_graymorph2; do
        bin="$VERIFY_BIN_DIR/$prog"
        if [[ ! -x "$bin" ]]; then
            verify_failed+=("$prog (not built)")
            continue
        fi
        if ! "$bin" > /tmp/lept_run.log 2>&1; then
            echo "    FAILED ($prog) — last 5 lines:"
            tail -5 /tmp/lept_run.log | sed 's/^/    /'
            verify_failed+=("$prog")
        fi
    done

    if [[ "${#verify_failed[@]}" -gt 0 ]]; then
        echo
        echo "Aborting: ${#verify_failed[@]} verify program(s) failed; manifest not regenerated." >&2
        for f in "${verify_failed[@]}"; do
            echo "  - $f" >&2
        done
        exit 1
    fi

    echo "==> copying verify outputs to $C_OUT_DIR with golden_map names"
    verify_copied=0
    verify_missing=()
    while IFS=$'\t' read -r src dst; do
        # Skip comment / blank lines in c_verify_outputs.tsv
        [[ -z "$src" || "$src" =~ ^# ]] && continue
        if [[ -f "$src" ]]; then
            cp "$src" "$C_OUT_DIR/$dst"
            verify_copied=$((verify_copied + 1))
        else
            verify_missing+=("$src → $dst")
        fi
    done < "$VERIFY_MAP"
    echo "    copied $verify_copied verify outputs"

    if [[ "${#verify_missing[@]}" -gt 0 ]]; then
        echo
        echo "Aborting: ${#verify_missing[@]} verify output(s) missing from /tmp/." >&2
        for m in "${verify_missing[@]}"; do
            echo "  - $m" >&2
        done
        exit 1
    fi
fi

cd "$ROOT"
echo
echo "==> hashing $C_OUT_DIR → $MANIFEST_OUT"
cargo run --release --quiet --example gen_c_manifest --features all-formats -- \
    --c-dir "$C_OUT_DIR" \
    --out "$MANIFEST_OUT"
