# C vs Rust Feature Comparison

Survey date: 2026-02-23 (Phase 10-1 blend regression tests complete, all 10 phases reflected)

## Overview

| Item                             | C version (reference/leptonica) | Rust version (leptonica-rs) |
| -------------------------------- | ------------------------------- | --------------------------- |
| Source files                     | **182** (.c)                    | **151** (.rs)               |
| Lines of code                    | **~240,000**                    | **~119,500**                |
| Implementation rate (line-based) | 100%                            | **~50%**                    |

## Function-level Comparison Summary

All public functions from the C version were extracted and classified into 3 categories by implementation status in the Rust version.
See files under `docs/porting/comparison/` for details (currently only available in Japanese).

| Module                                                              | ✅ Equivalent | 🔄 Different | ❌ Unimplemented | Total     | Coverage  |
| ------------------------------------------------------------------- | ------------- | ------------ | ---------------- | --------- | --------- |
| [leptonica (src/core/)](../../porting/comparison/core.md)           | 521           | 24           | 337              | 882       | 61.8%     |
| [leptonica (src/io/)](../../porting/comparison/io.md)               | 68            | 17           | 61               | 146       | 58.2%     |
| [leptonica (src/transform/)](../../porting/comparison/transform.md) | 82            | 9            | 61               | 152       | 59.9%     |
| [leptonica (src/morph/)](../../porting/comparison/morph.md)         | 82            | 16           | 22               | 120       | 81.7%     |
| [leptonica (src/filter/)](../../porting/comparison/filter.md)       | 82            | 0            | 17               | 99        | 82.8%     |
| [leptonica (src/color/)](../../porting/comparison/color.md)         | 52            | 16           | 58               | 126       | 54.0%     |
| [leptonica (src/region/)](../../porting/comparison/region.md)       | 40            | 8            | 47               | 95        | 50.5%     |
| [leptonica (src/recog/)](../../porting/comparison/recog.md)         | 83            | 16           | 45               | 144       | 68.8%     |
| [Other](../../porting/comparison/misc.md)                           | 12            | 0            | 104              | 116       | 10.3%     |
| **Total**                                                           | **1,022**     | **106**      | **752**          | **1,880** | **60.0%** |

### Classification Criteria

- **✅ Equivalent**: Same algorithm and functionality exists in Rust version as in C version
- **🔄 Different**: Equivalent functionality exists, but API design or algorithm differs
- **❌ Unimplemented**: No corresponding functionality exists in Rust version

### Major Design Differences

| Aspect            | C version                         | Rust version                                            |
| ----------------- | --------------------------------- | ------------------------------------------------------- |
| Memory management | Reference counting (manual)       | `Arc<PixData>` / ownership (Pix/PixMut two-layer model) |
| Error handling    | NULL return / error codes         | `Result<T, Error>` / `thiserror`                        |
| API unification   | Separate functions for Gray/Color | Unified API with auto depth detection                   |
| Collection types  | Boxa/Pixa/Numa/Sarray             | `Vec<T>` + dedicated types                              |
| I/O streams       | `FILE*` pointer                   | `Read`/`Write` traits                                   |
| Data structures   | heap/list/stack/queue             | Standard library (`BinaryHeap`/`Vec`/`VecDeque`)        |
| unsafe            | Used throughout                   | Prohibited in principle, minimized                      |

## Feature Category Comparison

### 1. Core Data Structures

| Feature                     | C version                     | Rust version             | Notes                                                 |
| --------------------------- | ----------------------------- | ------------------------ | ----------------------------------------------------- |
| Pix (image container)       | ✅ pix1-5.c                   | ✅ leptonica (src/core/) | Basic ops implemented, some depth conversions not yet |
| Box (rectangular region)    | ✅ boxbasic.c, boxfunc1-5.c   | ✅ leptonica (src/core/) | Basic ops and geometric calculations implemented      |
| Pta (point array)           | ✅ ptabasic.c, ptafunc1-2.c   | ✅ leptonica (src/core/) | Basic ops implemented                                 |
| Colormap                    | ✅ colormap.c                 | ✅ leptonica (src/core/) | Basic ops implemented                                 |
| Pixa (Pix array)            | ✅ pixabasic.c, pixafunc1-2.c | ✅ pixa/mod.rs           | Basic ops implemented                                 |
| Numa (numeric array)        | ✅ numabasic.c, numafunc1-2.c | ✅ numa/mod.rs           | Basic ops implemented                                 |
| Sarray (string array)       | ✅ sarray1-2.c                | ✅ sarray/mod.rs         | String array/set operations                           |
| FPix (floating point image) | ✅ fpix1-2.c                  | ✅ fpix/mod.rs           | Pix conversion/arithmetic                             |
| Pixel arithmetic            | ✅ pixarith.c                 | ✅ arith.rs              | Add/sub/mul/div/const ops                             |
| Logical ops                 | ✅ rop.c, roplow.c            | ✅ rop.rs                | AND/OR/XOR/NOT etc.                                   |
| Comparison                  | ✅ compare.c                  | ✅ compare.rs            | Diff/RMS/correlation                                  |
| Blend                       | ✅ blend.c                    | ✅ blend.rs              | Alpha/mask/multiply etc.                              |
| Graphics                    | ✅ graphics.c                 | ✅ graphics.rs           | Line/rect/circle/contour drawing                      |

### 2. Image I/O

| Format                | C version                 | Rust version | Notes                                               |
| --------------------- | ------------------------- | ------------ | --------------------------------------------------- |
| BMP                   | ✅ bmpio.c                | ✅ bmp.rs    | Enabled by default                                  |
| PNG                   | ✅ pngio.c                | ✅ png.rs    | feature gate (`png-format`, default enabled)        |
| JPEG                  | ✅ jpegio.c               | ✅ jpeg.rs   | feature gate (`jpeg`, default enabled), read/write  |
| PNM (PBM/PGM/PPM/PAM) | ✅ pnmio.c                | ✅ pnm.rs    | Default enabled, ASCII/Binary/PAM support           |
| TIFF                  | ✅ tiffio.c               | ✅ tiff.rs   | feature gate (`tiff-format`), multi-page support    |
| GIF                   | ✅ gifio.c                | ✅ gif.rs    | feature gate (`gif-format`)                         |
| WebP                  | ✅ webpio.c, webpanimio.c | ✅ webp.rs   | feature gate (`webp-format`)                        |
| JP2K (JPEG2000)       | ✅ jp2kio.c               | ✅ jp2k.rs   | feature gate (`jp2k-format`), read only             |
| SPIX                  | ✅ spixio.c               | ✅ spix.rs   | Leptonica native serialization format               |
| PDF                   | ✅ pdfio1-2.c, pdfapp.c   | ✅ pdf.rs    | feature gate (`pdf-format`), Flate/DCT compression  |
| PostScript            | ✅ psio1-2.c              | ✅ ps/       | feature gate (`ps-format`), Level 1/2/3, multi-page |
| Format detection      | ✅ readfile.c             | ✅ format.rs | Fully implemented                                   |
| Header reading        | ✅ readfile.c             | ✅ header.rs | All formats supported                               |

### 3. Geometric Transforms

| Feature                    | C version                    | Rust version     | Notes                                              |
| -------------------------- | ---------------------------- | ---------------- | -------------------------------------------------- |
| Rotation (orthogonal)      | ✅ rotateorth.c              | ✅ rotate.rs     | 90°/180°/270°                                      |
| Rotation (arbitrary angle) | ✅ rotate.c, rotateam.c      | ✅ rotate.rs     | Area mapping/sampling/shear                        |
| Rotation (shear)           | ✅ rotateshear.c             | ✅ rotate.rs     | 2-shear/3-shear support                            |
| Scaling                    | ✅ scale1-2.c                | ✅ scale.rs      | 3 algorithms (1bpp specialization not implemented) |
| Affine transform           | ✅ affine.c, affinecompose.c | ✅ affine.rs     | Sampling/interpolation support                     |
| Bilinear transform         | ✅ bilinear.c                | ✅ bilinear.rs   | 4-point correspondence/interpolation               |
| Projective transform       | ✅ projective.c              | ✅ projective.rs | 4-point homography                                 |
| Shear transform            | ✅ shear.c                   | ✅ shear.rs      | Horizontal/vertical/linear interp support          |
| Flip (horizontal/vertical) | ✅ rotateorth.c              | ✅ rotate.rs     | Fully implemented                                  |

### 4. Morphology

| Feature                   | C version                  | Rust version    | Notes                                                  |
| ------------------------- | -------------------------- | --------------- | ------------------------------------------------------ |
| Binary erosion/dilation   | ✅ morph.c                 | ✅ binary.rs    | Fully implemented                                      |
| Binary open/close         | ✅ morph.c                 | ✅ binary.rs    | Fully implemented                                      |
| Hit-miss transform        | ✅ morph.c                 | ✅ binary.rs    | Fully implemented                                      |
| Morphological gradient    | ✅ morph.c                 | ✅ binary.rs    | Fully implemented                                      |
| Top-hat/Bottom-hat        | ✅ morph.c                 | ✅ binary.rs    | Fully implemented                                      |
| Grayscale morphology      | ✅ graymorph.c             | ✅ grayscale.rs | Dilate/erode/open/close                                |
| Color morphology          | ✅ colormorph.c            | ✅ color.rs     | Independent processing per RGB channel                 |
| DWA (fast morphology)     | ✅ morphdwa.c, dwacomb.2.c | ✅ dwa.rs       | Brick fast operations                                  |
| Structuring element (SEL) | ✅ sel1-2.c, selgen.c      | ✅ sel.rs       | Basic implementation (auto-generation not implemented) |
| Sequence operations       | ✅ morphseq.c              | ✅ sequence.rs  | String-format sequences                                |
| Thinning                  | ✅ ccthin.c                | ✅ thin.rs      | Connectivity-preserving thinning                       |

### 5. Filtering

| Feature              | C version      | Rust version    | Notes                                                   |
| -------------------- | -------------- | --------------- | ------------------------------------------------------- |
| Convolution          | ✅ convolve.c  | ✅ convolve.rs  | Basic/block/separable/windowed stats                    |
| Box filter           | ✅ convolve.c  | ✅ convolve.rs  | Includes block convolution optimization                 |
| Gaussian filter      | ✅ convolve.c  | ✅ convolve.rs  | Basic implementation                                    |
| Sobel edge detection | ✅ edge.c      | ✅ edge.rs      | Fully implemented                                       |
| Laplacian            | ✅ edge.c      | ✅ edge.rs      | Fully implemented                                       |
| Sharpening           | ✅ enhance.c   | ✅ edge.rs      | Basic implementation                                    |
| Unsharp mask         | ✅ enhance.c   | ✅ edge.rs      | Basic and fast variants                                 |
| Bilateral filter     | ✅ bilateral.c | ✅ bilateral.rs | Edge-preserving smoothing (fast approx not implemented) |
| Adaptive mapping     | ✅ adaptmap.c  | ✅ adaptmap.rs  | Background/contrast normalization                       |
| Rank filter          | ✅ rank.c      | ✅ rank.rs      | Median/min/max                                          |

### 6. Color Processing

| Feature                  | C version          | Rust version     | Notes                           |
| ------------------------ | ------------------ | ---------------- | ------------------------------- |
| Colorspace conversion    | ✅ colorspace.c    | ✅ colorspace.rs | RGB↔HSV/LAB/XYZ/YUV (per-pixel) |
| Color quantization       | ✅ colorquant1-2.c | ✅ quantize.rs   | Median cut, Octree (simplified) |
| Color segmentation       | ✅ colorseg.c      | ✅ segment.rs    | 4-stage algorithm               |
| Color content extraction | ✅ colorcontent.c  | ✅ analysis.rs   | Color stats, color count        |
| Color fill               | ✅ colorfill.c     | ✅ colorfill.rs  | Seed-based region detection     |
| Colorization             | ✅ coloring.c      | ✅ coloring.rs   | Gray colorization/color shift   |

### 7. Binarization

| Feature               | C version      | Rust version    | Notes                  |
| --------------------- | -------------- | --------------- | ---------------------- |
| Simple threshold      | ✅ binarize.c  | ✅ threshold.rs | Fully implemented      |
| Otsu binarization     | ✅ binarize.c  | ✅ threshold.rs | Fully implemented      |
| Sauvola binarization  | ✅ binarize.c  | ✅ threshold.rs | Fully implemented      |
| Adaptive binarization | ✅ binarize.c  | ✅ threshold.rs | Mean/Gaussian          |
| Dithering             | ✅ grayquant.c | ✅ threshold.rs | Floyd-Steinberg, Bayer |

### 8. Region Processing

| Feature                      | C version      | Rust version    | Notes                                             |
| ---------------------------- | -------------- | --------------- | ------------------------------------------------- |
| Connected components         | ✅ conncomp.c  | 🔄 conncomp.rs  | Union-Find approach (C returns BOXA/PIXA)         |
| Connected component labeling | ✅ pixlabel.c  | ✅ label.rs     | Basic implementation                              |
| Border tracing               | ✅ ccbord.c    | 🔄 ccbord.rs    | Simplified Border struct (C uses CCBORDA)         |
| Seed fill                    | ✅ seedfill.c  | 🔄 seedfill.rs  | Queue-based BFS (C uses stack-based)              |
| Watershed transform          | ✅ watershed.c | 🔄 watershed.rs | Priority queue approach                           |
| Quadtree                     | ✅ quadtree.c  | ✅ quadtree.rs  | Integral image/hierarchical stats (high coverage) |
| Maze                         | ✅ maze.c      | ✅ maze.rs      | Generation/BFS solver                             |

### 9. Document Processing & Recognition

| Feature                   | C version                     | Rust version   | Notes                                                       |
| ------------------------- | ----------------------------- | -------------- | ----------------------------------------------------------- |
| Page segmentation         | ✅ pageseg.c                  | ✅ pageseg.rs  | Halftone/text detection                                     |
| Skew detection/correction | ✅ skew.c                     | ✅ skew.rs     | Differential squared sum scoring                            |
| Dewarping                 | ✅ dewarp1-4.c                | ✅ dewarp/     | Single page (Dewarpa multi-page management not implemented) |
| Baseline detection        | ✅ baseline.c                 | ✅ baseline.rs | Horizontal projection method                                |
| Character recognition     | ✅ recogbasic.c, recogident.c | ✅ recog/      | Template matching, DID                                      |
| JBIG2 classification      | ✅ jbclass.c                  | ✅ jbclass/    | RankHaus, correlation-based classification                  |
| Barcode                   | ✅ bardecode.c, readbarcode.c | ✅ barcode/    | EAN/UPC/Code39 etc.                                         |
| Warper                    | ✅ warper.c                   | ✅ warper.rs   | Harmonic distortion/stereo (91% implemented)                |

## Rust Module Implementation Status

| Module                     | Lines        | Function Coverage       | Key Features                                                                    |
| -------------------------- | ------------ | ----------------------- | ------------------------------------------------------------------------------- |
| leptonica (src/core/)      | ~47,100      | 545/882 (61.8%)         | Pix, Box, Pta, Colormap, arithmetic, compare, blend, graphics, stats, histogram |
| leptonica (src/io/)        | ~7,900       | 85/146 (58.2%)          | BMP/PNG/JPEG/PNM/TIFF/GIF/WebP/JP2K/PDF/PS/SPIX + header reading                |
| leptonica (src/transform/) | ~11,200      | 91/152 (59.9%)          | Rotate, scale, affine, projective, shear                                        |
| leptonica (src/morph/)     | ~9,400       | 98/120 (81.7%)          | Binary/grayscale/color morphology, DWA, thinning                                |
| leptonica (src/filter/)    | ~9,800       | 82/99 (82.8%)           | Convolution, edge detection, bilateral, rank, adaptive mapping                  |
| leptonica (src/color/)     | ~7,400       | 68/126 (54.0%)          | Colorspace conversion, quantization, segmentation, binarization, color analysis |
| leptonica (src/region/)    | ~10,600      | 48/95 (50.5%)           | Connected components, seed fill, watershed, quadtree, maze                      |
| leptonica (src/recog/)     | ~16,000      | 99/144 (68.8%)          | Skew correction, dewarping, character recognition, barcode                      |
| Other                      | -            | 12/116 (10.3%)          | Warper, encoding                                                                |
| **Total**                  | **~119,500** | **1,128/1,880 (60.0%)** |                                                                                 |

## Major Unimplemented Areas

### leptonica (src/core/) (remaining unimplemented: ~337 functions)

Significantly improved in Phases 13–17 (26.7% → 61.8%). Main unimplemented areas:

- **I/O helper functions**: Pix/Boxa/Pixa/Numa Read/Write/Serialize (planned for Phase 10)
- **Advanced colormap ops**: Search/conversion/effects (planned for Phase 12)
- **roplow.c**: Low-level bit operations (covered by high-level API in Rust rop.rs, skip candidate)
- **boxfunc2.c/5.c**: Box conversion utilities, smoothing

### leptonica (src/filter/) (coverage: 82.8%)

- **Fast bilateral approximation** (pixBilateral)
- **adaptmap.c detailed features**: Morphology-based background normalization, map utilities
- **Tiled convolution**: pixBlockconvTiled etc.

### Other (coverage: 10.3%)

- **Compressed image container** (pixcomp.c): Pixcomp/Pixacomp
- **Tiling** (pixtiling.c): Large image split processing
- **Advanced labeling** (pixlabel.c): Distance functions, local extrema
- **Data structures**: Replaceable with Rust standard library (heap→BinaryHeap etc.)

## C Version Feature Categories (182 files)

```text
Core structures:  pix1-5, boxbasic, boxfunc1-5, ptabasic, ptafunc1-2,
                  pixabasic, pixafunc1-2, numabasic, numafunc1-2, sarray1-2
I/O:              bmpio, pngio, jpegio, pnmio, tiffio, gifio, webpio, jp2kio,
                  pdfio1-2, psio1-2, readfile, writefile, spixio
Geometric:        rotate, rotateam, rotateorth, rotateshear, scale1-2,
                  affine, affinecompose, bilinear, projective, shear
Morphology:       morph, morphapp, morphdwa, morphseq, graymorph, colormorph,
                  sel1-2, selgen, ccthin
Filter:           convolve, edge, enhance, bilateral, adaptmap, rank
Color:            colorspace, colorquant1-2, colorseg, colorcontent,
                  colorfill, coloring, colormap
Binarization:     binarize, grayquant
Region:           conncomp, ccbord, seedfill, watershed, pixlabel, quadtree
Document:         pageseg, skew, dewarp1-4, baseline
Recognition:      recogbasic, recogdid, recogident, recogtrain
JBIG2:            jbclass
Other:            compare, blend, pixarith, rop, bardecode, graphics, maze, warper
```

## Detailed Comparison Documents

Function-level comparison per module (list of all public functions and their implementation status).
Note: these per-module files are currently only available in Japanese.

- [leptonica (src/core/)](../../porting/comparison/core.md) — 882 functions
- [leptonica (src/io/)](../../porting/comparison/io.md) — 146 functions
- [leptonica (src/transform/)](../../porting/comparison/transform.md) — 152 functions
- [leptonica (src/morph/)](../../porting/comparison/morph.md) — 120 functions
- [leptonica (src/filter/)](../../porting/comparison/filter.md) — 99 functions
- [leptonica (src/color/)](../../porting/comparison/color.md) — 126 functions
- [leptonica (src/region/)](../../porting/comparison/region.md) — 95 functions
- [leptonica (src/recog/)](../../porting/comparison/recog.md) — 144 functions
- [Other](../../porting/comparison/misc.md) — 116 functions

## References

- C source: `reference/leptonica/src/`
- Rust source: `src/`
