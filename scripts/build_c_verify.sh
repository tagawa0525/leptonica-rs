#!/usr/bin/env bash
# Compile the C-side verification helpers (scripts/verify_*.c) into
# /tmp/lept_verify/. These programs reproduce intermediate C outputs for
# assertion-only regression tests (binmorph1/3, fhmtauto, graymorph2) so
# that scripts/gen_c_manifest.sh can populate tests/golden_manifest_c.tsv
# with hashes that prog/*_reg never emits.
#
# Idempotent: skips compilation when /tmp/lept_verify/.build_complete exists.
# Pass --force to rebuild from scratch.
#
# See docs/porting/c-compat-coverage.md (Phase 1.5) and
# docs/plans/901_c-hash-compat.md.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LEPT_DIR="$ROOT/reference/leptonica"
INCLUDE_DIR="$LEPT_DIR/src"
LIB_DIR="$LEPT_DIR/build/src"
SCRIPTS_DIR="$ROOT/scripts"
BIN_DIR="/tmp/lept_verify"
MARKER="$BIN_DIR/.build_complete"

VERIFY_PROGS=(verify_binmorph verify_fhmtauto verify_graymorph2)

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

if [[ ! -d "$LIB_DIR" ]]; then
    echo "libleptonica not built at $LIB_DIR." >&2
    echo "Run scripts/build_c_leptonica.sh first." >&2
    exit 1
fi

# Reuse the link line cmake produced for one of the prog/*_reg binaries to
# discover the full absolute paths to libgif / libjpeg / libtiff / etc. in
# the nix store. This avoids depending on `nix develop` and matches whatever
# image libs the existing reg binaries already link against. We strip
# libleptonica.a itself (we add our own -lleptonica below) and -lm (added
# explicitly at the end).
LINK_TXT="$LEPT_DIR/build/prog/CMakeFiles/binarize_reg.dir/link.txt"
if [[ ! -f "$LINK_TXT" ]]; then
    echo "cmake link recipe not found at $LINK_TXT." >&2
    echo "Run scripts/build_c_leptonica.sh first (BUILD_PROG=ON)." >&2
    exit 1
fi

mapfile -t EXTRA_LIBS < <(
    tr ' ' '\n' < "$LINK_TXT" \
        | grep -E '\.(so[\.0-9]*|a)$' \
        | grep -v 'libleptonica\.'
)

mkdir -p "$BIN_DIR"

if [[ "$force" -eq 1 ]]; then
    rm -f "$MARKER" "$BIN_DIR"/verify_*
fi

# Rebuild when any verify_*.c source is newer than the marker, so editing
# a helper without --force does not silently keep stale binaries.
needs_rebuild=0
if [[ "$force" -eq 1 || ! -f "$MARKER" ]]; then
    needs_rebuild=1
else
    for prog in "${VERIFY_PROGS[@]}"; do
        src="$SCRIPTS_DIR/${prog}.c"
        if [[ -f "$src" && "$src" -nt "$MARKER" ]]; then
            needs_rebuild=1
            break
        fi
    done
fi

if [[ "$needs_rebuild" -eq 0 ]]; then
    echo "Verify programs already built (marker: $MARKER, use --force to rebuild)."
    exit 0
fi

# Compile each verify program against libleptonica.a produced by
# scripts/build_c_leptonica.sh (cmake builds a static archive by default
# with BUILD_SHARED_LIBS=OFF). We invoke cc directly rather than going
# through `nix develop` (which would need a flake.nix at the repo root)
# because the C toolchain is expected to be on PATH in the same environment
# that already built libleptonica.
for prog in "${VERIFY_PROGS[@]}"; do
    src="$SCRIPTS_DIR/${prog}.c"
    bin="$BIN_DIR/${prog}"
    if [[ ! -f "$src" ]]; then
        echo "warning: $src not found, skipping" >&2
        continue
    fi
    echo "==> compiling $prog"
    # $LIB_DIR (= build/src) also holds the cmake-generated endianness.h
    # that the leptonica headers `#include "endianness.h"`. Pass it as a
    # second -I path so the include resolves outside of the cmake build tree.
    cc -o "$bin" "$src" \
        -I"$INCLUDE_DIR" -I"$LIB_DIR" \
        -L"$LIB_DIR" -lleptonica \
        "${EXTRA_LIBS[@]}" \
        -lm
done

touch "$MARKER"

echo "Built $(ls "$BIN_DIR"/verify_* 2>/dev/null | wc -l) verify programs in $BIN_DIR"
