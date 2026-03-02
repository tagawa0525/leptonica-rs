/* Generate and compare binmorph1/binmorph3 golden outputs from C leptonica.
 * binmorph1: dilate/erode/open/close_brick(21, 15) on feyn-fract.tif
 * binmorph3: separable dilate(11,1)+(1,7), direct dilate(11,7), dilate(21,1)
 */
#include "allheaders.h"
#include <stdio.h>

static void compare_tiff(const char *c_path, const char *r_path, const char *desc) {
    PIX *c_pix = pixRead(c_path);
    PIX *r_pix = pixRead(r_path);
    if (!c_pix || !r_pix) {
        printf("%-30s CANNOT READ (c=%p r=%p)\n", desc, (void*)c_pix, (void*)r_pix);
        if (c_pix) pixDestroy(&c_pix);
        if (r_pix) pixDestroy(&r_pix);
        return;
    }
    
    l_int32 same;
    pixEqual(c_pix, r_pix, &same);
    
    int w = pixGetWidth(c_pix), h = pixGetHeight(c_pix), d = pixGetDepth(c_pix);
    int rw = pixGetWidth(r_pix), rh = pixGetHeight(r_pix), rd = pixGetDepth(r_pix);
    
    if (same) {
        printf("%-30s %dx%dx%d  IDENTICAL\n", desc, w, h, d);
    } else {
        /* Count pixel differences */
        PIX *xor = pixXor(NULL, c_pix, r_pix);
        l_int32 count;
        pixCountPixels(xor, &count, NULL);
        printf("%-30s C=%dx%dx%d R=%dx%dx%d  DIFFER (%d pixels)\n",
               desc, w, h, d, rw, rh, rd, count);
        pixDestroy(&xor);
    }
    
    pixDestroy(&c_pix);
    pixDestroy(&r_pix);
}

int main() {
    PIX *pixs = pixRead("feyn-fract.tif");
    if (!pixs) { fprintf(stderr, "Cannot read feyn-fract.tif\n"); return 1; }
    printf("Input: feyn-fract.tif %dx%dx%d\n\n", 
           pixGetWidth(pixs), pixGetHeight(pixs), pixGetDepth(pixs));

    /* binmorph1: dilate/erode/open/close with 21x15 brick */
    printf("=== binmorph1: dilate/erode/open/close_brick(21, 15) ===\n");
    PIX *dil = pixDilateBrick(NULL, pixs, 21, 15);
    PIX *ero = pixErodeBrick(NULL, pixs, 21, 15);
    PIX *opn = pixOpenBrick(NULL, pixs, 21, 15);
    PIX *cls = pixCloseBrick(NULL, pixs, 21, 15);
    
    pixWriteTiff("/tmp/c_binmorph1_dilate.tif", dil, IFF_TIFF_G4, "w");
    pixWriteTiff("/tmp/c_binmorph1_erode.tif",  ero, IFF_TIFF_G4, "w");
    pixWriteTiff("/tmp/c_binmorph1_open.tif",   opn, IFF_TIFF_G4, "w");
    pixWriteTiff("/tmp/c_binmorph1_close.tif",  cls, IFF_TIFF_G4, "w");
    
    const char *rust_dir = "tests/golden/";
    char r_path[256];
    
    snprintf(r_path, sizeof(r_path), "%sbinmorph1_golden.09.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph1_dilate.tif", r_path, "binmorph1 dilate(21,15)");
    snprintf(r_path, sizeof(r_path), "%sbinmorph1_golden.10.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph1_erode.tif", r_path, "binmorph1 erode(21,15)");
    snprintf(r_path, sizeof(r_path), "%sbinmorph1_golden.11.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph1_open.tif", r_path, "binmorph1 open(21,15)");
    snprintf(r_path, sizeof(r_path), "%sbinmorph1_golden.12.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph1_close.tif", r_path, "binmorph1 close(21,15)");
    
    pixDestroy(&dil); pixDestroy(&ero); pixDestroy(&opn); pixDestroy(&cls);

    /* binmorph3: separable and direct dilation */
    printf("\n=== binmorph3: separable and direct dilation ===\n");
    PIX *dil_sep_h = pixDilateBrick(NULL, pixs, 11, 1);   /* horizontal first */
    PIX *dil_sep   = pixDilateBrick(NULL, dil_sep_h, 1, 7); /* then vertical */
    PIX *dil_dir   = pixDilateBrick(NULL, pixs, 11, 7);   /* direct 11x7 */
    PIX *dil_h21   = pixDilateBrick(NULL, pixs, 21, 1);   /* horizontal 21x1 */
    
    pixWriteTiff("/tmp/c_binmorph3_sep.tif",   dil_sep, IFF_TIFF_G4, "w");
    pixWriteTiff("/tmp/c_binmorph3_dir.tif",   dil_dir, IFF_TIFF_G4, "w");
    pixWriteTiff("/tmp/c_binmorph3_h21.tif",   dil_h21, IFF_TIFF_G4, "w");
    
    snprintf(r_path, sizeof(r_path), "%sbinmorph3_golden.14.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph3_sep.tif", r_path, "binmorph3 sep dilate(11,1)+(1,7)");
    snprintf(r_path, sizeof(r_path), "%sbinmorph3_golden.15.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph3_dir.tif", r_path, "binmorph3 dir dilate(11,7)");
    snprintf(r_path, sizeof(r_path), "%sbinmorph3_golden.16.tif", rust_dir);
    compare_tiff("/tmp/c_binmorph3_h21.tif", r_path, "binmorph3 dilate(21,1)");
    
    /* Also verify separability in C: sep == dir */
    l_int32 sep_eq;
    pixEqual(dil_sep, dil_dir, &sep_eq);
    printf("\n  C separability: sep(11,1)+(1,7) == dir(11,7)? %s\n",
           sep_eq ? "YES (identical)" : "NO (different!)");
    
    pixDestroy(&dil_sep_h); pixDestroy(&dil_sep);
    pixDestroy(&dil_dir); pixDestroy(&dil_h21);
    pixDestroy(&pixs);
    
    return 0;
}
