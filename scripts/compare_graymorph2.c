/* Compare C and Rust graymorph2 outputs pixel-by-pixel.
 * Reports max pixel difference for each pair. */
#include "allheaders.h"
#include <stdio.h>

int main() {
    struct { const char *c_path; const char *r_path; const char *desc; } pairs[] = {
        {"/tmp/c_graymorph2_dilate_gray_3x1.jpg", "tests/golden/gmorph2_dilate_erode_golden.02.jpg", "dilate 3x1"},
        {"/tmp/c_graymorph2_erode_gray_3x1.jpg",  "tests/golden/gmorph2_dilate_erode_golden.04.jpg", "erode 3x1"},
        {"/tmp/c_graymorph2_dilate_gray_1x3.jpg", "tests/golden/gmorph2_dilate_erode_golden.06.jpg", "dilate 1x3"},
        {"/tmp/c_graymorph2_erode_gray_1x3.jpg",  "tests/golden/gmorph2_dilate_erode_golden.08.jpg", "erode 1x3"},
        {"/tmp/c_graymorph2_dilate_gray_3x3.jpg", "tests/golden/gmorph2_dilate_erode_golden.10.jpg", "dilate 3x3"},
        {"/tmp/c_graymorph2_erode_gray_3x3.jpg",  "tests/golden/gmorph2_dilate_erode_golden.12.jpg", "erode 3x3"},
        {"/tmp/c_graymorph2_open_gray_3x1.jpg",   "tests/golden/gmorph2_open_close_golden.02.jpg",   "open 3x1"},
        {"/tmp/c_graymorph2_close_gray_3x1.jpg",  "tests/golden/gmorph2_open_close_golden.04.jpg",   "close 3x1"},
        {"/tmp/c_graymorph2_open_gray_1x3.jpg",   "tests/golden/gmorph2_open_close_golden.06.jpg",   "open 1x3"},
        {"/tmp/c_graymorph2_close_gray_1x3.jpg",  "tests/golden/gmorph2_open_close_golden.08.jpg",   "close 1x3"},
        {"/tmp/c_graymorph2_open_gray_3x3.jpg",   "tests/golden/gmorph2_open_close_golden.10.jpg",   "open 3x3"},
        {"/tmp/c_graymorph2_close_gray_3x3.jpg",  "tests/golden/gmorph2_open_close_golden.12.jpg",   "close 3x3"},
    };

    int n = sizeof(pairs) / sizeof(pairs[0]);
    printf("%-15s %6s %6s %8s %10s %10s\n", "Operation", "Width", "Height", "MaxDiff", "DiffPixels", "TotalPix");
    printf("--------------------------------------------------------------\n");

    for (int i = 0; i < n; i++) {
        PIX *c_pix = pixRead(pairs[i].c_path);
        PIX *r_pix = pixRead(pairs[i].r_path);
        if (!c_pix || !r_pix) {
            printf("%-15s CANNOT READ\n", pairs[i].desc);
            if (c_pix) pixDestroy(&c_pix);
            if (r_pix) pixDestroy(&r_pix);
            continue;
        }

        int w = pixGetWidth(c_pix);
        int h = pixGetHeight(c_pix);
        int total = w * h;
        int max_diff = 0;
        int diff_count = 0;

        for (int y = 0; y < h; y++) {
            for (int x = 0; x < w; x++) {
                l_uint32 c_val, r_val;
                pixGetPixel(c_pix, x, y, &c_val);
                pixGetPixel(r_pix, x, y, &r_val);
                int d = abs((int)c_val - (int)r_val);
                if (d > 0) diff_count++;
                if (d > max_diff) max_diff = d;
            }
        }

        printf("%-15s %6d %6d %8d %10d %10d\n",
               pairs[i].desc, w, h, max_diff, diff_count, total);

        pixDestroy(&c_pix);
        pixDestroy(&r_pix);
    }

    return 0;
}
