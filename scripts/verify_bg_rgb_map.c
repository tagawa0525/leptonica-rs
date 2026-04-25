/* Verify Rust get_background_rgb_map matches C pixGetBackgroundRGBMap.
 *
 * C builds a single foreground mask from a grayscale conversion of the
 * RGB input (`pixConvertRGBToGrayFast` + threshold + dilate), then
 * accumulates R/G/B sums in one tile pass against that shared mask.
 * Rust currently builds a per-channel fg mask, which is structurally
 * different. This helper emits the C reference outputs so PR4 can
 * verify the Rust rewrite.
 *
 * Outputs:
 *   /tmp/c_bg_rgb_map_r_church.png   /tmp/c_bg_rgb_map_g_church.png
 *   /tmp/c_bg_rgb_map_b_church.png
 *   /tmp/c_bg_norm_church.png        (full pixBackgroundNorm)
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_bg_rgb_map.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_bg_rgb_map'
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

    PIX *pixs = NULL, *pixmr = NULL, *pixmg = NULL, *pixmb = NULL;
    PIX *normed = NULL;

    pixs = pixRead("tests/data/images/church.png");
    if (!pixs) {
        fprintf(stderr, "could not read church.png\n");
        rc = 1;
        goto cleanup;
    }
    /* pixGetBackgroundRGBMap requires 32 bpp RGB. church.png loads as
     * 32 bpp RGBA via leptonica; treat alpha as ignored. */
    if (pixGetDepth(pixs) != 32) {
        fprintf(stderr, "expected 32 bpp RGB, got %d bpp\n",
                pixGetDepth(pixs));
        rc = 1;
        goto cleanup;
    }

    /* Default Rust BackgroundNormOptions: 10x15 tiles, fg=60, mincount=40. */
    if (pixGetBackgroundRGBMap(pixs, NULL, NULL, 10, 15, 60, 40,
                               &pixmr, &pixmg, &pixmb) != 0) {
        fprintf(stderr, "pixGetBackgroundRGBMap failed\n");
        rc = 1;
        goto cleanup;
    }
    rc |= write_pix("/tmp/c_bg_rgb_map_r_church.png", pixmr,
                    "bg_rgb_map_r church");
    rc |= write_pix("/tmp/c_bg_rgb_map_g_church.png", pixmg,
                    "bg_rgb_map_g church");
    rc |= write_pix("/tmp/c_bg_rgb_map_b_church.png", pixmb,
                    "bg_rgb_map_b church");

    /* Full pixBackgroundNorm with RGB defaults (smooth_x=2, smooth_y=1). */
    normed = pixBackgroundNorm(pixs, NULL, NULL, 10, 15, 60, 40, 200, 2, 1);
    if (!normed) {
        fprintf(stderr, "pixBackgroundNorm failed\n");
        rc = 1;
        goto cleanup;
    }
    rc |= write_pix("/tmp/c_bg_norm_church.png", normed,
                    "bg_norm church (full)");

cleanup:
    pixDestroy(&pixs);
    pixDestroy(&pixmr);
    pixDestroy(&pixmg);
    pixDestroy(&pixmb);
    pixDestroy(&normed);
    return rc;
}
