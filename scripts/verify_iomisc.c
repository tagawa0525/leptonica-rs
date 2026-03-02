/*
 * verify_iomisc.c — Compare C iomisc golden output with Rust golden files.
 *
 * C checkpoints that have regTestWritePixAndCheck:
 *   6:  alpha channel from books_logo.png
 *   7:  alpha blend over white
 *   9:  alpha channel from regenerated logo2.png
 *  10:  alpha blend over cyan
 *  13:  pixRemoveColormap(weasel4.11c.png)
 *  14:  pixConvertRGBToColormap(pix1)
 *  15:  pixRemoveColormap(weasel4.5g.png)
 *  16:  pixConvertGrayToColormap(pix1)
 *
 * We verify checkpoints 13-16 (colormap operations) against Rust golden.
 */
#include "allheaders.h"
#include <stdio.h>

#define PROG_DIR "/home/tagawa/github/leptonica-rs/reference/leptonica/prog"
#define RUST_GOLDEN "/home/tagawa/github/leptonica-rs/tests/golden"

static void compare_pix(const char *label, PIX *c_pix, const char *rust_path)
{
    PIX *pixr;
    l_int32 same;

    pixr = pixRead(rust_path);
    if (!pixr) {
        printf("  %s: SKIP (cannot read %s)\n", label, rust_path);
        return;
    }

    pixEqual(c_pix, pixr, &same);
    if (same) {
        printf("  %s: IDENTICAL\n", label);
    } else {
        printf("  %s: DIFFER (C=%dx%d d=%d, Rust=%dx%d d=%d)\n",
               label,
               pixGetWidth(c_pix), pixGetHeight(c_pix), pixGetDepth(c_pix),
               pixGetWidth(pixr), pixGetHeight(pixr), pixGetDepth(pixr));
        /* If same depth, check pixel diff */
        if (pixGetDepth(c_pix) == pixGetDepth(pixr) && pixGetDepth(c_pix) == 1) {
            PIX *pixd = pixXor(NULL, c_pix, pixr);
            l_int32 cnt;
            pixCountPixels(pixd, &cnt, NULL);
            printf("    XOR pixel diff: %d\n", cnt);
            pixDestroy(&pixd);
        }
    }
    pixDestroy(&pixr);
}

int main(void)
{
    PIX *pixs, *pix1, *pix2;

    printf("=== iomisc: C vs Rust golden comparison ===\n\n");

    /* --- Checkpoint 13: pixRemoveColormap(weasel4.11c.png) --- */
    printf("[13] pixRemoveColormap(weasel4.11c.png, REMOVE_CMAP_BASED_ON_SRC)\n");
    pixs = pixRead(PROG_DIR "/weasel4.11c.png");
    if (!pixs) { printf("  SKIP: cannot load\n\n"); return 1; }

    pix1 = pixRemoveColormap(pixs, REMOVE_CMAP_BASED_ON_SRC);
    compare_pix("C[13] vs Rust regen_rgb_cmap.04",
                pix1, RUST_GOLDEN "/iomisc_regen_rgb_cmap_golden.04.png");

    /* --- Checkpoint 14: pixConvertRGBToColormap --- */
    printf("[14] pixConvertRGBToColormap(pix1, 1)\n");
    pix2 = pixConvertRGBToColormap(pix1, 1);
    compare_pix("C[14] vs Rust regen_rgb_cmap.06",
                pix2, RUST_GOLDEN "/iomisc_regen_rgb_cmap_golden.06.png");
    pixDestroy(&pixs);
    pixDestroy(&pix1);
    pixDestroy(&pix2);

    /* --- Checkpoint 15: pixRemoveColormap(weasel4.5g.png) --- */
    printf("[15] pixRemoveColormap(weasel4.5g.png, REMOVE_CMAP_BASED_ON_SRC)\n");
    pixs = pixRead(PROG_DIR "/weasel4.5g.png");
    if (!pixs) { printf("  SKIP: cannot load\n\n"); return 1; }

    pix1 = pixRemoveColormap(pixs, REMOVE_CMAP_BASED_ON_SRC);
    compare_pix("C[15] vs Rust regen_gray_cmap.03",
                pix1, RUST_GOLDEN "/iomisc_regen_gray_cmap_golden.03.png");

    /* --- Checkpoint 16: pixConvertGrayToColormap --- */
    printf("[16] pixConvertGrayToColormap(pix1)\n");
    pix2 = pixConvertGrayToColormap(pix1);
    compare_pix("C[16] vs Rust regen_gray_cmap.05",
                pix2, RUST_GOLDEN "/iomisc_regen_gray_cmap_golden.05.png");
    pixDestroy(&pixs);
    pixDestroy(&pix1);
    pixDestroy(&pix2);

    printf("\nDone.\n");
    return 0;
}
