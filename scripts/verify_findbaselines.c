/* Verify Rust find_baselines() matches C pixFindBaselines() / Gen.
 *
 * Mirrors tests/recog/baseline_reg.rs test_3/11/13/15.
 * Writes per-image y-coordinate lists to /tmp/c_baseline_<name>.txt.
 * The values from these files are then transcribed into
 * tests/recog/baseline_c_parity.rs as `const C_*: &[i32]` arrays — the
 * Rust tests assert against those embedded constants rather than reading
 * the .txt files at test time, so the C build is only needed when
 * refreshing the constants.
 *
 * Build (from repo root):
 *   nix-shell -p libpng libjpeg libtiff libwebp giflib zlib openjpeg \
 *             pkg-config gcc \
 *     --run 'gcc -I reference/leptonica/src -I reference/leptonica/build/src \
 *            scripts/verify_findbaselines.c \
 *            reference/leptonica/build/src/libleptonica.a \
 *            $(pkg-config --libs libpng libjpeg libtiff-4 libwebp libopenjp2 zlib) \
 *            -lwebpmux -lwebpdemux -lgif -lm \
 *            -o /tmp/verify_findbaselines'
 */
#include "allheaders.h"
#include <stdio.h>

static int dump_baselines(const char *img_path, l_int32 minw,
                          const char *out_path, const char *desc) {
    PIX *pix = pixRead(img_path);
    if (!pix) {
        fprintf(stderr, "%-30s CANNOT READ %s\n", desc, img_path);
        return 1;
    }
    NUMA *na = pixFindBaselinesGen(pix, minw, NULL, NULL);
    if (!na) {
        fprintf(stderr, "%-30s pixFindBaselinesGen returned NULL\n", desc);
        pixDestroy(&pix);
        return 1;
    }
    FILE *fp = fopen(out_path, "w");
    if (!fp) {
        fprintf(stderr, "cannot open %s\n", out_path);
        numaDestroy(&na);
        pixDestroy(&pix);
        return 1;
    }
    l_int32 n = numaGetCount(na);
    fprintf(fp, "# %s: pixFindBaselinesGen minw=%d count=%d\n", desc, minw, n);
    for (l_int32 i = 0; i < n; ++i) {
        l_int32 y = 0;
        numaGetIValue(na, i, &y);
        fprintf(fp, "%d\n", y);
    }
    fclose(fp);
    printf("%-30s wrote %d baselines to %s\n", desc, n, out_path);
    numaDestroy(&na);
    pixDestroy(&pix);
    return 0;
}

int main(void) {
    setLeptDebugOK(1);
    int rc = 0;
    /* test_3: keystone.png - C pixFindBaselines uses default minw=80. */
    rc |= dump_baselines("reference/leptonica/prog/keystone.png", 80,
                         "/tmp/c_baseline_keystone.txt",
                         "find_baselines keystone");
    /* test_11: baseline1.png - default minw=80. */
    rc |= dump_baselines("reference/leptonica/prog/baseline1.png", 80,
                         "/tmp/c_baseline_short_textblock.txt",
                         "find_baselines short_textblock");
    /* test_13: baseline2.tif - minw=30. */
    rc |= dump_baselines("reference/leptonica/prog/baseline2.tif", 30,
                         "/tmp/c_baseline_short_lines.txt",
                         "find_baselines short_lines");
    /* test_15: baseline3.tif - minw=30. */
    rc |= dump_baselines("reference/leptonica/prog/baseline3.tif", 30,
                         "/tmp/c_baseline_more_short_lines.txt",
                         "find_baselines more_short_lines");
    return rc;
}
