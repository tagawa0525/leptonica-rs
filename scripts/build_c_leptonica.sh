#!/usr/bin/env bash
# Build C leptonica with regression test programs (BUILD_PROG=ON).
#
# Idempotent: skips rebuild when build/bin already contains *_reg binaries.
# Pass --force to wipe build/ and rebuild from scratch.
#
# See docs/plans/901_c-hash-compat.md.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LEPT_DIR="$ROOT/reference/leptonica"

if [[ ! -d "$LEPT_DIR" ]]; then
    echo "reference/leptonica not found." >&2
    echo "Clone with: git clone https://github.com/DanBloomberg/leptonica $LEPT_DIR" >&2
    exit 1
fi

force=0
for arg in "$@"; do
    case "$arg" in
        --force) force=1 ;;
        -h|--help)
            echo "Usage: $0 [--force]"
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 1
            ;;
    esac
done

BUILD_DIR="$LEPT_DIR/build"

if [[ "$force" -eq 1 ]]; then
    rm -rf "$BUILD_DIR"
fi

if [[ "$force" -eq 0 && -d "$BUILD_DIR/bin" ]] \
    && compgen -G "$BUILD_DIR/bin/*_reg" > /dev/null; then
    echo "C leptonica already built at $BUILD_DIR (use --force to rebuild)."
    exit 0
fi

cd "$LEPT_DIR"
nix develop --command bash -c "
    set -e
    mkdir -p build
    cd build
    cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_PROG=ON
    cmake --build . -j\$(nproc)
"

echo "Built $(ls "$BUILD_DIR/bin"/*_reg 2>/dev/null | wc -l) regression binaries."
