/* Verify Rust get_inv_background_map matches C pixGetInvBackgroundMap.
 *
 * Input: pre-aligned C bg gray map produced by verify_bg_gray_map.c
 *   /tmp/c_bg_gray_map_dreyfus.png
 * Output: /tmp/c_inv_bg_map_dreyfus.png (16 bpp PNG)
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_inv_bg_map.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_inv_bg_map'
 */
#include "allheaders.h"
#include <stdio.h>

static int dump(const char *map_path, l_int32 bgval, l_int32 smoothx,
                l_int32 smoothy, const char *out_path, const char *desc) {
    PIX *map = pixRead(map_path);
    if (!map) {
        fprintf(stderr, "%-30s CANNOT READ %s\n", desc, map_path);
        return 1;
    }
    PIX *inv = pixGetInvBackgroundMap(map, bgval, smoothx, smoothy);
    if (!inv) {
        fprintf(stderr, "%-30s pixGetInvBackgroundMap failed\n", desc);
        pixDestroy(&map);
        return 1;
    }
    int rc = 0;
    if (pixWrite(out_path, inv, IFF_PNG) != 0) {
        fprintf(stderr, "%-30s pixWrite failed for %s\n", desc, out_path);
        rc = 1;
    } else {
        printf("%-30s wrote %dx%dx%d to %s\n", desc,
               pixGetWidth(inv), pixGetHeight(inv), pixGetDepth(inv), out_path);
    }
    pixDestroy(&map);
    pixDestroy(&inv);
    return rc;
}

int main(void) {
    setLeptDebugOK(1);
    /* Default Rust BackgroundNormOptions: bg_val=200, smooth_x=2, smooth_y=2. */
    return dump("/tmp/c_bg_gray_map_dreyfus.png", 200, 2, 2,
                "/tmp/c_inv_bg_map_dreyfus.png", "inv_bg_map dreyfus8");
}
