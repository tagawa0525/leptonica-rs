/* Verify fhmtauto golden: use same thinning SELs as Rust test.
 * Rust uses make_thin_sels(Set4cc1) → sel_4_1,sel_4_2,sel_4_3 (3 SELs)
 *           make_thin_sels(Set8cc1) → sel_8_2,sel_8_3,sel_8_5,sel_8_6 (4 SELs)
 * C: selaMakeThinSets(1,0) → 3 SELs, selaMakeThinSets(5,0) → 4 SELs
 */
#include "allheaders.h"
#include <stdio.h>

static void compare_1bpp(const char *c_path, const char *r_path, const char *desc) {
    PIX *c_pix = pixRead(c_path);
    PIX *r_pix = pixRead(r_path);
    if (!c_pix || !r_pix) {
        printf("%-30s CANNOT READ (c=%s r=%s)\n", desc,
               c_pix ? "OK" : "MISSING", r_pix ? "OK" : "MISSING");
        if (c_pix) pixDestroy(&c_pix);
        if (r_pix) pixDestroy(&r_pix);
        return;
    }
    PIX *xorp = pixXor(NULL, c_pix, r_pix);
    l_int32 count;
    pixCountPixels(xorp, &count, NULL);
    int w = pixGetWidth(c_pix), h = pixGetHeight(c_pix);
    if (count == 0)
        printf("%-30s %dx%d  IDENTICAL (0 pixel diff)\n", desc, w, h);
    else
        printf("%-30s %dx%d  DIFFER (%d pixels)\n", desc, w, h, count);
    pixDestroy(&xorp);
    pixDestroy(&c_pix);
    pixDestroy(&r_pix);
}

int main() {
    PIX *pixs = pixRead("/home/tagawa/github/leptonica-rs/tests/data/images/feyn-fract.tif");
    if (!pixs) { fprintf(stderr, "Cannot read\n"); return 1; }
    printf("Input: %dx%dx%d\n\n", pixGetWidth(pixs), pixGetHeight(pixs), pixGetDepth(pixs));

    /* Set4cc1 = selaMakeThinSets(1, 0): sel_4_1, sel_4_2, sel_4_3 */
    SELA *set4 = selaMakeThinSets(1, 0);
    int n4 = selaGetCount(set4);
    printf("Set4cc1: %d SELs\n", n4);
    
    /* Set8cc1 = selaMakeThinSets(5, 0): sel_8_2, sel_8_3, sel_8_5, sel_8_6 */
    SELA *set8 = selaMakeThinSets(5, 0);
    int n8 = selaGetCount(set8);
    printf("Set8cc1: %d SELs\n", n8);

    /* Rust golden index mapping:
     * fhmtauto_hmt_golden.02.tif → Set4cc1[0] = sel_4_1
     * fhmtauto_hmt_golden.04.tif → Set4cc1[1] = sel_4_2
     * fhmtauto_hmt_golden.06.tif → Set4cc1[2] = sel_4_3
     * fhmtauto_hmt_golden.08.tif → Set8cc1[0] = sel_8_2
     * fhmtauto_hmt_golden.10.tif → Set8cc1[1] = sel_8_3
     * fhmtauto_hmt_golden.12.tif → Set8cc1[2] = sel_8_5
     * fhmtauto_hmt_golden.14.tif → Set8cc1[3] = sel_8_6
     */
    const char *golden_dir = "/home/tagawa/github/leptonica-rs/tests/golden";
    char r_path[512], c_path[256];
    int golden_idx = 2;  /* starts at 02 (even indices) */
    
    printf("\n=== Set4cc1 (sel_4_1, sel_4_2, sel_4_3) ===\n");
    for (int i = 0; i < n4; i++) {
        SEL *sel = selaGetSel(set4, i);
        PIX *result = pixHMT(NULL, pixs, sel);
        snprintf(c_path, sizeof(c_path), "/tmp/c_fhmtauto_%02d.tif", golden_idx);
        pixWriteTiff(c_path, result, IFF_TIFF_G4, "w");
        snprintf(r_path, sizeof(r_path), "%s/fhmtauto_hmt_golden.%02d.tif", golden_dir, golden_idx);
        char desc[64];
        snprintf(desc, sizeof(desc), "Set4cc1[%d] golden.%02d", i, golden_idx);
        compare_1bpp(c_path, r_path, desc);
        pixDestroy(&result);
        golden_idx += 2;
    }
    
    printf("\n=== Set8cc1 (sel_8_2, sel_8_3, sel_8_5, sel_8_6) ===\n");
    for (int i = 0; i < n8; i++) {
        SEL *sel = selaGetSel(set8, i);
        PIX *result = pixHMT(NULL, pixs, sel);
        snprintf(c_path, sizeof(c_path), "/tmp/c_fhmtauto_%02d.tif", golden_idx);
        pixWriteTiff(c_path, result, IFF_TIFF_G4, "w");
        snprintf(r_path, sizeof(r_path), "%s/fhmtauto_hmt_golden.%02d.tif", golden_dir, golden_idx);
        char desc[64];
        snprintf(desc, sizeof(desc), "Set8cc1[%d] golden.%02d", i, golden_idx);
        compare_1bpp(c_path, r_path, desc);
        pixDestroy(&result);
        golden_idx += 2;
    }
    
    /* Identity: 1x1 HIT sel */
    printf("\n=== Identity (1x1 brick) ===\n");
    SEL *sel_id = selCreateBrick(1, 1, 0, 0, SEL_HIT);
    PIX *id_result = pixHMT(NULL, pixs, sel_id);
    pixWriteTiff("/tmp/c_fhmtauto_id.tif", id_result, IFF_TIFF_G4, "w");
    snprintf(r_path, sizeof(r_path), "%s/fhmtauto_id_golden.01.tif", golden_dir);
    compare_1bpp("/tmp/c_fhmtauto_id.tif", r_path, "Identity 1x1 golden.01");
    pixDestroy(&id_result);
    selDestroy(&sel_id);
    
    selaDestroy(&set4);
    selaDestroy(&set8);
    pixDestroy(&pixs);
    return 0;
}
