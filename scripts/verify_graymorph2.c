/* Generate graymorph2 golden outputs from C leptonica for comparison.
 * Produces the same operations as Rust graymorph2_reg test.
 * Compile: cc -o /tmp/verify_graymorph2 /tmp/verify_graymorph2.c -I<lept_include> -L<lept_lib> -llept
 * Run from reference/leptonica/prog/ directory (needs test8.jpg)
 */
#include "allheaders.h"
#include <stdio.h>

int main() {
    PIX *pixs = pixRead("test8.jpg");
    if (!pixs) { fprintf(stderr, "Cannot read test8.jpg\n"); return 1; }

    /* Same operations as Rust graymorph2_reg */
    struct { const char *name; PIX *(*fn)(PIX*, l_int32, l_int32); int h; int v; } ops[] = {
        {"dilate_gray_3x1", pixDilateGray, 3, 1},
        {"erode_gray_3x1",  pixErodeGray,  3, 1},
        {"dilate_gray_1x3", pixDilateGray, 1, 3},
        {"erode_gray_1x3",  pixErodeGray,  1, 3},
        {"dilate_gray_3x3", pixDilateGray, 3, 3},
        {"erode_gray_3x3",  pixErodeGray,  3, 3},
        {"open_gray_3x1",   pixOpenGray,   3, 1},
        {"close_gray_3x1",  pixCloseGray,  3, 1},
        {"open_gray_1x3",   pixOpenGray,   1, 3},
        {"close_gray_1x3",  pixCloseGray,  1, 3},
        {"open_gray_3x3",   pixOpenGray,   3, 3},
        {"close_gray_3x3",  pixCloseGray,  3, 3},
    };

    int n = sizeof(ops) / sizeof(ops[0]);
    for (int i = 0; i < n; i++) {
        PIX *result = ops[i].fn(pixs, ops[i].h, ops[i].v);
        if (!result) {
            fprintf(stderr, "Failed: %s\n", ops[i].name);
            continue;
        }
        char path[256];
        snprintf(path, sizeof(path), "/tmp/c_graymorph2_%s.jpg", ops[i].name);
        pixWriteJpeg(path, result, 75, 0);
        printf("Wrote %s (%dx%d)\n", path, pixGetWidth(result), pixGetHeight(result));
        pixDestroy(&result);
    }

    pixDestroy(&pixs);
    return 0;
}
