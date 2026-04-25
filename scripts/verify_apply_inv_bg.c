/* Verify Rust apply_inv_background_gray_map matches C
 * pixApplyInvBackgroundGrayMap, and the full pixBackgroundNorm pipeline.
 *
 * All intermediate steps run in-memory (16 bpp inv map cannot survive a
 * PNG roundtrip — leptonica downconverts on read — so we recompute the
 * bg+inv chain inside this binary).
 *
 * Outputs:
 *   /tmp/c_apply_inv_bg_dreyfus.png       (apply step in isolation, 8 bpp)
 *   /tmp/c_bg_norm_dreyfus.png            (full pixBackgroundNorm output, 8 bpp)
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_apply_inv_bg.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_apply_inv_bg'
 */
#include "allheaders.h"
#include <stdio.h>

static int write_pix(const char *path, PIX *pix, const char *desc) {
    if (pixWrite(path, pix, IFF_PNG) != 0) {
        fprintf(stderr, "%-30s pixWrite %s failed\n", desc, path);
        return 1;
    }
    printf("%-30s wrote %dx%dx%d to %s\n", desc,
           pixGetWidth(pix), pixGetHeight(pix), pixGetDepth(pix), path);
    return 0;
}

int main(void) {
    setLeptDebugOK(1);
    int rc = 0;

    PIX *pixs = pixRead("tests/data/images/dreyfus8.png");
    PIX *gray = pixGetColormap(pixs)
        ? pixRemoveColormap(pixs, REMOVE_CMAP_TO_GRAYSCALE)
        : pixClone(pixs);
    if (!gray) {
        fprintf(stderr, "could not load gray\n");
        return 1;
    }

    /* Apply step: bg_map (8 bpp) -> inv_map (16 bpp) -> apply.
     * Use the same smooth sizes as Rust BackgroundNormOptions::default(),
     * which mirrors C DefaultXSmoothSize=2, DefaultYSmoothSize=1. */
    PIX *bg_map = NULL;
    if (pixGetBackgroundGrayMap(gray, NULL, 10, 15, 60, 40, &bg_map) != 0) {
        fprintf(stderr, "pixGetBackgroundGrayMap failed\n");
        return 1;
    }
    PIX *inv = pixGetInvBackgroundMap(bg_map, 200, 2, 1);
    if (!inv) {
        fprintf(stderr, "pixGetInvBackgroundMap failed\n");
        return 1;
    }
    PIX *applied = pixApplyInvBackgroundGrayMap(gray, inv, 10, 15);
    if (!applied) {
        fprintf(stderr, "pixApplyInvBackgroundGrayMap failed\n");
        return 1;
    }
    rc |= write_pix("/tmp/c_apply_inv_bg_dreyfus.png", applied,
                    "apply_inv_bg dreyfus8");

    /* Full pixBackgroundNorm via the high-level API. */
    PIX *normed = pixBackgroundNorm(gray, NULL, NULL, 10, 15, 60, 40, 200, 2, 1);
    if (!normed) {
        fprintf(stderr, "pixBackgroundNorm failed\n");
        return 1;
    }
    rc |= write_pix("/tmp/c_bg_norm_dreyfus.png", normed,
                    "bg_norm dreyfus8 (full)");

    pixDestroy(&pixs);
    pixDestroy(&gray);
    pixDestroy(&bg_map);
    pixDestroy(&inv);
    pixDestroy(&applied);
    pixDestroy(&normed);
    return rc;
}
