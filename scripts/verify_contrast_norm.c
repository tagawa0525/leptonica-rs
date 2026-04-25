/* Verify Rust contrast_norm matches C pixContrastNorm.
 *
 * Default Rust `ContrastNormOptions`: tile 20x20
 * (`DEFAULT_CONTRAST_TILE_SIZE`), min_diff=50, smooth_x=2, smooth_y=2.
 * Note this is independent of `BackgroundNormOptions` defaults — C's
 * `pixContrastNorm` doesn't define module-level constants, so the
 * Rust-side defaults follow leptonica's documented recommendation
 * "sx and sy ... typically at least 20".
 *
 * Output: /tmp/c_contrast_norm_dreyfus.png
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_contrast_norm.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_contrast_norm'
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

    PIX *pixs = NULL, *gray = NULL, *normed = NULL;

    pixs = pixRead("tests/data/images/dreyfus8.png");
    if (!pixs) {
        fprintf(stderr, "could not read dreyfus8.png\n");
        rc = 1;
        goto cleanup;
    }
    gray = pixGetColormap(pixs)
        ? pixRemoveColormap(pixs, REMOVE_CMAP_TO_GRAYSCALE)
        : pixClone(pixs);
    if (!gray || pixGetDepth(gray) != 8) {
        fprintf(stderr, "expected 8 bpp grayscale\n");
        rc = 1;
        goto cleanup;
    }

    /* Mirrors Rust `ContrastNormOptions::default()`. The first two args
     * are tile dimensions (20x20, per C leptonica's "sx and sy ...
     * typically at least 20" recommendation), then mindiff=50, then the
     * smoothing half-widths smoothx=2, smoothy=2. Passing NULL for
     * `pixd` makes pixContrastNorm allocate a new output image. */
    normed = pixContrastNorm(NULL, gray, 20, 20, 50, 2, 2);
    if (!normed) {
        fprintf(stderr, "pixContrastNorm failed\n");
        rc = 1;
        goto cleanup;
    }
    rc |= write_pix("/tmp/c_contrast_norm_dreyfus.png", normed,
                    "contrast_norm dreyfus8");

cleanup:
    pixDestroy(&pixs);
    pixDestroy(&gray);
    pixDestroy(&normed);
    return rc;
}
