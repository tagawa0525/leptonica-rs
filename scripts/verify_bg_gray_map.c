/* Verify Rust get_background_gray_map matches C pixGetBackgroundGrayMap.
 *
 * Mirrors the way Rust get_background_gray_map_inner is exercised in
 * tests/filter/adaptmap_reg.rs (default BackgroundNormOptions:
 *   tile_width=10, tile_height=15, fg_threshold=60, min_count=40).
 *
 * Outputs:
 *   /tmp/c_bg_gray_map_dreyfus.png — C reference output for dreyfus8.png
 *   /tmp/c_bg_gray_map_lucasta.png — C reference output for lucasta.150.jpg
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_bg_gray_map.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_bg_gray_map'
 */
#include "allheaders.h"
#include <stdio.h>

static int dump(const char *path, l_int32 sx, l_int32 sy, l_int32 thresh,
                l_int32 mincount, const char *out_path, const char *desc) {
    PIX *pixs = pixRead(path);
    if (!pixs) {
        fprintf(stderr, "%-30s CANNOT READ %s\n", desc, path);
        return 1;
    }
    PIX *gray = pixGetColormap(pixs)
        ? pixRemoveColormap(pixs, REMOVE_CMAP_TO_GRAYSCALE)
        : pixClone(pixs);
    if (!gray || pixGetDepth(gray) != 8) {
        fprintf(stderr, "%-30s expected 8 bpp grayscale\n", desc);
        pixDestroy(&pixs); pixDestroy(&gray);
        return 1;
    }

    PIX *pixd = NULL;
    if (pixGetBackgroundGrayMap(gray, NULL, sx, sy, thresh, mincount, &pixd) != 0
        || !pixd) {
        fprintf(stderr, "%-30s pixGetBackgroundGrayMap failed\n", desc);
        pixDestroy(&pixs); pixDestroy(&gray);
        return 1;
    }
    pixWrite(out_path, pixd, IFF_PNG);
    printf("%-30s wrote %dx%dx%d to %s\n", desc,
           pixGetWidth(pixd), pixGetHeight(pixd), pixGetDepth(pixd), out_path);
    pixDestroy(&pixs); pixDestroy(&gray); pixDestroy(&pixd);
    return 0;
}

int main(void) {
    setLeptDebugOK(1);
    int rc = 0;
    /* Default Rust BackgroundNormOptions: 10x15 tiles, fg_thresh=60, min_count=40. */
    rc |= dump("tests/data/images/dreyfus8.png",     10, 15, 60, 40,
               "/tmp/c_bg_gray_map_dreyfus.png", "bg_gray_map dreyfus8");
    rc |= dump("tests/data/images/lucasta.150.jpg",  10, 15, 60, 40,
               "/tmp/c_bg_gray_map_lucasta.png", "bg_gray_map lucasta.150");
    return rc;
}
