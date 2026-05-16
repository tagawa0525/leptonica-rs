#!/usr/bin/env bash
# Run C-version regression tests in 'generate' mode and update
# tests/golden_manifest_c.tsv via examples/gen_c_manifest.
#
# By default runs all *_reg binaries; pass test names (without _reg suffix)
# to limit the run, e.g.:
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

if [[ ! -d "$BIN_DIR" ]] || ! compgen -G "$BIN_DIR/*_reg" > /dev/null; then
    echo "C leptonica not built at $BIN_DIR." >&2
    echo "Run scripts/build_c_leptonica.sh first." >&2
    exit 1
fi

if [[ ! -d "$PROG_DIR" ]]; then
    echo "prog/ directory not found at $PROG_DIR." >&2
    exit 1
fi

mkdir -p "$C_OUT_DIR"

# Build the binary list. Empty positional args ⇒ run everything.
binaries=()
if [[ "$#" -eq 0 ]]; then
    while IFS= read -r -d '' bin; do
        binaries+=("$bin")
    done < <(find "$BIN_DIR" -maxdepth 1 -type f -name '*_reg' -print0 | sort -z)
else
    for name in "$@"; do
        candidate="$BIN_DIR/${name}_reg"
        if [[ ! -x "$candidate" ]]; then
            echo "warning: $candidate not found, skipping" >&2
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

cd "$ROOT"
echo
echo "==> hashing $C_OUT_DIR → $MANIFEST_OUT"
cargo run --release --quiet --example gen_c_manifest --features all-formats -- \
    --c-dir "$C_OUT_DIR" \
    --out "$MANIFEST_OUT"
