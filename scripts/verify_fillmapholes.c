/* Verify Rust fill_map_holes() matches C pixFillMapHoles().
 *
 * Mirrors tests/filter/adaptmap_reg.rs adaptmap_reg_fill_map_holes_{weasel,simple}.
 * Holes are encoded as zero pixels (Rust API convention) and filled with
 * L_FILL_BLACK so the algorithm input is identical on both sides.
 *
 * Prerequisites:
 *   1. Build C reference (from repo root):
 *      cd reference/leptonica && nix-shell -p libpng libjpeg libtiff libwebp \
 *        giflib zlib openjpeg pkg-config cmake clang \
 *        --run 'mkdir -p build && cd build && cmake .. -DCMAKE_BUILD_TYPE=Release \
 *               -DBUILD_PROG=ON && cmake --build . -j$(nproc)'
 *   2. Generate Rust regout artifacts that this script reads:
 *      cargo test --test filter adaptmap_reg_fill_map_holes
 *      (writes tests/regout/adaptmap_fill_holes_{weasel.04,simple.06}.png)
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_fillmapholes.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_fillmapholes'
 *
 * Run from the repository root:
 *   /tmp/verify_fillmapholes
 */
#include "allheaders.h"
#include <stdio.h>

static int compare_with_rust(const char *c_path, const char *r_path,
                             const char *desc) {
    PIX *c_pix = pixRead(c_path);
    PIX *r_pix = pixRead(r_path);
    if (!c_pix || !r_pix) {
        printf("%-30s CANNOT READ (c=%p r=%p)\n", desc,
               (void *)c_pix, (void *)r_pix);
        if (c_pix) pixDestroy(&c_pix);
        if (r_pix) pixDestroy(&r_pix);
        return 1;
    }

    l_int32 w = pixGetWidth(c_pix), h = pixGetHeight(c_pix);
    l_int32 d = pixGetDepth(c_pix);
    l_int32 rw = pixGetWidth(r_pix), rh = pixGetHeight(r_pix);
    l_int32 rd = pixGetDepth(r_pix);

    int rc = 0;
    if (w != rw || h != rh || d != rd) {
        printf("%-30s DIMENSION MISMATCH C=%dx%dx%d R=%dx%dx%d\n",
               desc, w, h, d, rw, rh, rd);
        rc = 1;
    } else {
        l_int32 same = 0;
        pixEqual(c_pix, r_pix, &same);
        if (same) {
            printf("%-30s %dx%dx%d  IDENTICAL\n", desc, w, h, d);
        } else {
            l_float32 fract = 0.0f, ave = 0.0f;
            pixGetDifferenceStats(c_pix, r_pix, 0, 1, &fract, &ave, 0);
            /* Also compute pixel-level differences manually for max delta. */
            l_int32 ndiff = 0, maxd = 0;
            for (l_int32 y = 0; y < h; ++y) {
                for (l_int32 x = 0; x < w; ++x) {
                    l_uint32 cv = 0, rv = 0;
                    pixGetPixel(c_pix, x, y, &cv);
                    pixGetPixel(r_pix, x, y, &rv);
                    l_int32 delta = (l_int32)cv - (l_int32)rv;
                    if (delta < 0) delta = -delta;
                    if (delta > 0) {
                        ++ndiff;
                        if (delta > maxd) maxd = delta;
                    }
                }
            }
            printf("%-30s %dx%dx%d  DIFFER  fract>=1=%.4f ave=%.3f "
                   "ndiff=%d maxd=%d\n",
                   desc, w, h, d, fract, ave, ndiff, maxd);
            rc = 1;
        }
    }

    pixDestroy(&c_pix);
    pixDestroy(&r_pix);
    return rc;
}

/* Reproduce the weasel test from adaptmap_reg.rs (zero-hole convention). */
static PIX *build_weasel_input(void) {
    PIX *pix = pixRead("reference/leptonica/prog/weasel8.png");
    if (!pix) {
        fprintf(stderr, "cannot load weasel8.png\n");
        return NULL;
    }
    pixGammaTRC(pix, pix, 1.0, 0, 200);

    l_int32 w = pixGetWidth(pix), h = pixGetHeight(pix);
    /* Match Rust hole pattern: set zero strips. */
    for (l_int32 y = 0; y < h; ++y) {
        for (l_int32 x = 0; x < 5 && x < w; ++x)  pixSetPixel(pix, x, y, 0);
        for (l_int32 x = 20; x < 22 && x < w; ++x) pixSetPixel(pix, x, y, 0);
        for (l_int32 x = 40; x < 43 && x < w; ++x) pixSetPixel(pix, x, y, 0);
    }
    for (l_int32 y = 0; y < 3 && y < h; ++y)
        for (l_int32 x = 0; x < w; ++x) pixSetPixel(pix, x, y, 0);
    for (l_int32 y = 15; y < 18 && y < h; ++y)
        for (l_int32 x = 0; x < w; ++x) pixSetPixel(pix, x, y, 0);
    for (l_int32 y = 35; y < 37 && y < h; ++y)
        for (l_int32 x = 0; x < w; ++x) pixSetPixel(pix, x, y, 0);

    return pix;
}

int main(void) {
    setLeptDebugOK(1);

    /* === weasel === */
    PIX *pix_w = build_weasel_input();
    if (!pix_w) return 1;
    /* Snapshot the input before fill so we can isolate gamma+hole vs algorithm. */
    pixWrite("/tmp/c_fillmapholes_weasel_input.png", pix_w, IFF_PNG);
    l_int32 w = pixGetWidth(pix_w), h = pixGetHeight(pix_w);
    if (pixFillMapHoles(pix_w, w, h, L_FILL_BLACK) != 0) {
        fprintf(stderr, "pixFillMapHoles failed (weasel)\n");
        pixDestroy(&pix_w);
        return 1;
    }
    pixWrite("/tmp/c_fillmapholes_weasel.png", pix_w, IFF_PNG);
    pixDestroy(&pix_w);

    /* === 3x3 simple === */
    PIX *pix_s = pixCreate(3, 3, 8);
    pixSetPixel(pix_s, 1, 0, 128);
    if (pixFillMapHoles(pix_s, 3, 3, L_FILL_BLACK) != 0) {
        fprintf(stderr, "pixFillMapHoles failed (simple)\n");
        pixDestroy(&pix_s);
        return 1;
    }
    pixWrite("/tmp/c_fillmapholes_simple.png", pix_s, IFF_PNG);
    pixDestroy(&pix_s);

    /* === Compare === */
    int rc = 0;
    rc |= compare_with_rust("/tmp/c_fillmapholes_weasel.png",
                            "tests/regout/adaptmap_fill_holes_weasel.04.png",
                            "fill_map_holes weasel");
    rc |= compare_with_rust("/tmp/c_fillmapholes_simple.png",
                            "tests/regout/adaptmap_fill_holes_simple.06.png",
                            "fill_map_holes simple 3x3");
    return rc;
}
