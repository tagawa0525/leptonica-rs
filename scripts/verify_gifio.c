/*
 * verify_gifio.c — Compare C gifio golden output with Rust golden files.
 *
 * Build (from reference/leptonica/build):
 *   See link command in prog/CMakeFiles/gifio_reg.dir/link.txt
 *
 * Run: ./verify_gifio
 */
#include "allheaders.h"
#include <stdio.h>

#define PROG_DIR "/home/tagawa/github/leptonica-rs/reference/leptonica/prog"

static const char *FILES[] = {
    PROG_DIR "/feyn.tif",          /* 0: 1bpp */
    PROG_DIR "/weasel2.4g.png",    /* 1: 2bpp */
    PROG_DIR "/weasel4.16c.png",   /* 2: 4bpp */
    PROG_DIR "/dreyfus8.png",      /* 3: 8bpp grayscale */
    PROG_DIR "/weasel8.240c.png",  /* 4: 8bpp colormap */
    PROG_DIR "/test8.jpg",         /* 5: 8bpp from JPEG */
    PROG_DIR "/test16.tif",        /* 6: 16bpp */
    PROG_DIR "/marge.jpg",         /* 7: 32bpp */
};
#define N_FILES 8

/* Rust Part 1 golden indices (write_pix_and_check index) */
static const int RUST_INDICES[] = {2, 4, 6, 8, 10, 12, 14, 16};

int main(void)
{
    int i, same;
    l_int32 xor_count;
    char buf_a[512], buf_b[512], rust_path[512];
    PIX *pixs, *pix1, *pix2, *pixr, *pixd;

    lept_mkdir("lept/gif");

    printf("=== gifio: C vs Rust golden comparison ===\n\n");

    for (i = 0; i < N_FILES; i++) {
        printf("[%d] %s\n", i, FILES[i]);

        /* Reproduce C test_gif(): double roundtrip */
        pixs = pixRead(FILES[i]);
        if (!pixs) {
            printf("  SKIP: cannot load %s\n\n", FILES[i]);
            continue;
        }

        snprintf(buf_a, sizeof(buf_a), "/tmp/lept/gif/gifio-a.%d.gif", i + 1);
        pixWrite(buf_a, pixs, IFF_GIF);
        pix1 = pixRead(buf_a);

        snprintf(buf_b, sizeof(buf_b), "/tmp/lept/gif/gifio-b.%d.gif", i + 1);
        pixWrite(buf_b, pix1, IFF_GIF);
        pix2 = pixRead(buf_b);

        /* Write C golden as GIF */
        snprintf(buf_a, sizeof(buf_a), "/tmp/c_gifio_%d.gif", i);
        pixWrite(buf_a, pix2, IFF_GIF);

        /* Load Rust golden */
        snprintf(rust_path, sizeof(rust_path),
                 "/home/tagawa/github/leptonica-rs/tests/golden/gifio_golden.%02d.gif",
                 RUST_INDICES[i]);

        pixr = pixRead(rust_path);
        if (!pixr) {
            printf("  SKIP: cannot read Rust golden %s\n\n", rust_path);
            pixDestroy(&pixs);
            pixDestroy(&pix1);
            pixDestroy(&pix2);
            continue;
        }

        /* Compare */
        pixEqual(pix2, pixr, &same);
        if (same) {
            printf("  C golden vs Rust golden: IDENTICAL (pixEqual=1)\n");
        } else {
            /* XOR comparison for pixel diff count */
            if (pixGetDepth(pix2) == pixGetDepth(pixr)) {
                pixd = pixXor(NULL, pix2, pixr);
                pixCountPixels(pixd, &xor_count, NULL);
                printf("  C golden vs Rust golden: DIFFER (pixEqual=0, XOR pixels=%d)\n",
                       xor_count);
                printf("  C: %dx%d d=%d  Rust: %dx%d d=%d\n",
                       pixGetWidth(pix2), pixGetHeight(pix2), pixGetDepth(pix2),
                       pixGetWidth(pixr), pixGetHeight(pixr), pixGetDepth(pixr));
                pixDestroy(&pixd);
            } else {
                printf("  C golden vs Rust golden: DIFFER (depth mismatch: C=%d, Rust=%d)\n",
                       pixGetDepth(pix2), pixGetDepth(pixr));
                printf("  C: %dx%d  Rust: %dx%d\n",
                       pixGetWidth(pix2), pixGetHeight(pix2),
                       pixGetWidth(pixr), pixGetHeight(pixr));
            }
        }

        printf("\n");
        pixDestroy(&pixs);
        pixDestroy(&pix1);
        pixDestroy(&pix2);
        pixDestroy(&pixr);
    }

    return 0;
}
