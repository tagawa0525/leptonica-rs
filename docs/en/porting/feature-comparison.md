# C vs Rust Feature Comparison

> 🇯🇵 [日本語版](../../porting/feature-comparison.md)

Survey date: 2026-05-10 (verify-comparison-counts results reflected)

## Overview

| Item                             | C version (reference/leptonica) | Rust version (leptonica-rs) |
| -------------------------------- | ------------------------------- | --------------------------- |
| Source files                     | **182** (.c)                    | **180** (.rs)               |
| Lines of code                    | **~249,000**                    | **~147,000**                |
| Implementation rate (line-based) | 100%                            | **~59%**                    |

## Function-level Comparison Summary

All public functions from the C version were extracted and classified into 4 categories by implementation status in the Rust version.
See files under `docs/porting/comparison/` for details (currently only available in Japanese).

| Module                                                              | ✅ Equivalent | 🔄 Different | ❌ Unimplemented | 🚫 Not needed | Total     | Coverage  | Effective Coverage |
| ------------------------------------------------------------------- | ------------- | ------------ | ---------------- | ------------- | --------- | --------- | ------------------ |
| [leptonica (src/core/)](../../porting/comparison/core.md)           | 855           | 93           | 121              | 146           | 1,215     | 78.0%     | 88.7%              |
| [leptonica (src/io/)](../../porting/comparison/io.md)               | 139           | 19           | 0                | 50            | 208       | 76.0%     | 100.0%             |
| [leptonica (src/transform/)](../../porting/comparison/transform.md) | 109           | 19           | 0                | 14            | 142       | 90.1%     | 100.0%             |
| [leptonica (src/morph/)](../../porting/comparison/morph.md)         | 116           | 22           | 0                | 33            | 171       | 80.7%     | 100.0%             |
| [leptonica (src/filter/)](../../porting/comparison/filter.md)       | 117           | 0            | 1                | 13            | 131       | 89.3%     | 99.2%              |
| [leptonica (src/color/)](../../porting/comparison/color.md)         | 102           | 20           | 0                | 17            | 139       | 87.8%     | 100.0%             |
| [leptonica (src/region/)](../../porting/comparison/region.md)       | 65            | 8            | 0                | 22            | 95        | 76.8%     | 100.0%             |
| [leptonica (src/recog/)](../../porting/comparison/recog.md)         | 134           | 45           | 10               | 18            | 207       | 86.5%     | 94.7%              |
| [Other](../../porting/comparison/misc.md)                           | 145           | 5            | 5                | 371           | 526       | 28.5%     | 96.8%              |
| **Total**                                                           | **1,782**     | **231**      | **137**          | **684**       | **2,834** | **71.1%** | **93.6%**          |

### Classification Criteria

- **✅ Equivalent**: Same algorithm and functionality exists in Rust version as in C version
- **🔄 Different**: Equivalent functionality exists, but API design or algorithm differs
- **❌ Unimplemented**: No corresponding functionality exists in Rust version (implementation target)
- **🚫 Not needed**: Not required in the Rust version (covered by Rust standard library, C-specific design, debug-only, low-level internal functions, etc.)

**Effective Coverage** = (✅ + 🔄) / (Total − 🚫) — practical coverage excluding unnecessary functions

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

| Feature                     | C version                     | Rust version             | Notes                                                        |
| --------------------------- | ----------------------------- | ------------------------ | ------------------------------------------------------------ |
| Pix (image container)       | ✅ pix1-5.c                   | ✅ leptonica (src/core/) | Basic ops + depth conversions (1/2/4/8/16/32bpp) implemented |
| Box (rectangular region)    | ✅ boxbasic.c, boxfunc1-5.c   | ✅ leptonica (src/core/) | Basic ops and geometric calculations implemented             |
| Pta (point array)           | ✅ ptabasic.c, ptafunc1-2.c   | ✅ leptonica (src/core/) | Basic ops implemented                                        |
| Colormap                    | ✅ colormap.c                 | ✅ leptonica (src/core/) | Basic ops implemented                                        |
| Pixa (Pix array)            | ✅ pixabasic.c, pixafunc1-2.c | ✅ pixa/mod.rs           | Basic ops implemented                                        |
| Numa (numeric array)        | ✅ numabasic.c, numafunc1-2.c | ✅ numa/mod.rs           | Basic ops implemented                                        |
| Sarray (string array)       | ✅ sarray1-2.c                | ✅ sarray/mod.rs         | String array/set operations                                  |
| FPix (floating point image) | ✅ fpix1-2.c                  | ✅ fpix/mod.rs           | Pix conversion/arithmetic                                    |
| Pixel arithmetic            | ✅ pixarith.c                 | ✅ arith.rs              | Add/sub/mul/div/const ops                                    |
| Logical ops                 | ✅ rop.c, roplow.c            | ✅ rop.rs                | AND/OR/XOR/NOT etc.                                          |
| Comparison                  | ✅ compare.c                  | ✅ compare.rs            | Diff/RMS/correlation                                         |
| Blend                       | ✅ blend.c                    | ✅ blend.rs              | Alpha/mask/multiply etc.                                     |
| Graphics                    | ✅ graphics.c                 | ✅ graphics.rs           | Line/rect/circle/contour drawing                             |

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

| Feature                    | C version                    | Rust version     | Notes                                                                                                        |
| -------------------------- | ---------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------ |
| Rotation (orthogonal)      | ✅ rotateorth.c              | ✅ rotate.rs     | 90/180/270 degrees                                                                                           |
| Rotation (arbitrary angle) | ✅ rotate.c, rotateam.c      | ✅ rotate.rs     | Area mapping/sampling/shear                                                                                  |
| Rotation (shear)           | ✅ rotateshear.c             | ✅ rotate.rs     | 2-shear/3-shear support                                                                                      |
| Scaling                    | ✅ scale1-2.c                | ✅ scale.rs      | 3 algorithms + 1bpp specialization (`scale_binary`/`scale_to_gray`/`expand_binary_*`/`reduce_rank_binary_*`) |
| Affine transform           | ✅ affine.c, affinecompose.c | ✅ affine.rs     | Sampling/interpolation support                                                                               |
| Bilinear transform         | ✅ bilinear.c                | ✅ bilinear.rs   | 4-point correspondence/interpolation                                                                         |
| Projective transform       | ✅ projective.c              | ✅ projective.rs | 4-point homography                                                                                           |
| Shear transform            | ✅ shear.c                   | ✅ shear.rs      | Horizontal/vertical/linear interp support                                                                    |
| Flip (horizontal/vertical) | ✅ rotateorth.c              | ✅ rotate.rs     | Fully implemented                                                                                            |

### 4. Morphology

| Feature                   | C version                  | Rust version         | Notes                                                                          |
| ------------------------- | -------------------------- | -------------------- | ------------------------------------------------------------------------------ |
| Binary erosion/dilation   | ✅ morph.c                 | ✅ binary.rs         | Fully implemented                                                              |
| Binary open/close         | ✅ morph.c                 | ✅ binary.rs         | Fully implemented                                                              |
| Hit-miss transform        | ✅ morph.c                 | ✅ binary.rs         | Fully implemented                                                              |
| Morphological gradient    | ✅ morph.c                 | ✅ binary.rs         | Fully implemented                                                              |
| Top-hat/Bottom-hat        | ✅ morph.c                 | ✅ binary.rs         | Fully implemented                                                              |
| Grayscale morphology      | ✅ graymorph.c             | ✅ grayscale.rs      | Dilate/erode/open/close                                                        |
| Color morphology          | ✅ colormorph.c            | ✅ color.rs          | Independent processing per RGB channel                                         |
| DWA (fast morphology)     | ✅ morphdwa.c, dwacomb.2.c | ✅ dwa.rs            | Brick fast operations                                                          |
| Structuring element (SEL) | ✅ sel1-2.c, selgen.c      | ✅ sel.rs, selgen.rs | Basic + auto-generation (`selgen`) + Sela persistence I/O (`Sela::read/write`) |
| Sequence operations       | ✅ morphseq.c              | ✅ sequence.rs       | String-format sequences                                                        |
| Thinning                  | ✅ ccthin.c                | ✅ thin.rs           | Connectivity-preserving thinning                                               |

### 5. Filtering

| Feature              | C version      | Rust version    | Notes                                                          |
| -------------------- | -------------- | --------------- | -------------------------------------------------------------- |
| Convolution          | ✅ convolve.c  | ✅ convolve.rs  | Basic/block/separable/windowed stats                           |
| Box filter           | ✅ convolve.c  | ✅ convolve.rs  | Includes block convolution optimization                        |
| Gaussian filter      | ✅ convolve.c  | ✅ convolve.rs  | Basic implementation                                           |
| Sobel edge detection | ✅ edge.c      | ✅ edge.rs      | Fully implemented                                              |
| Laplacian            | ✅ edge.c      | ✅ edge.rs      | Fully implemented                                              |
| Sharpening           | ✅ enhance.c   | ✅ edge.rs      | Basic implementation                                           |
| Unsharp mask         | ✅ enhance.c   | ✅ edge.rs      | Basic and fast variants                                        |
| Bilateral filter     | ✅ bilateral.c | ✅ bilateral.rs | Edge-preserving smoothing + fast separable approximation (PBC) |
| Adaptive mapping     | ✅ adaptmap.c  | ✅ adaptmap.rs  | Background/contrast normalization                              |
| Rank filter          | ✅ rank.c      | ✅ rank.rs      | Median/min/max                                                 |

### 6. Color Processing

| Feature                  | C version          | Rust version     | Notes                               |
| ------------------------ | ------------------ | ---------------- | ----------------------------------- |
| Colorspace conversion    | ✅ colorspace.c    | ✅ colorspace.rs | RGB <-> HSV/LAB/XYZ/YUV (per-pixel) |
| Color quantization       | ✅ colorquant1-2.c | ✅ quantize.rs   | Median cut, Octree (simplified)     |
| Color segmentation       | ✅ colorseg.c      | ✅ segment.rs    | 4-stage algorithm                   |
| Color content extraction | ✅ colorcontent.c  | ✅ analysis.rs   | Color stats, color count            |
| Color fill               | ✅ colorfill.c     | ✅ colorfill.rs  | Seed-based region detection         |
| Colorization             | ✅ coloring.c      | ✅ coloring.rs   | Gray colorization/color shift       |

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

| Feature                   | C version                     | Rust version   | Notes                                         |
| ------------------------- | ----------------------------- | -------------- | --------------------------------------------- |
| Page segmentation         | ✅ pageseg.c                  | ✅ pageseg.rs  | Halftone/text detection                       |
| Skew detection/correction | ✅ skew.c                     | ✅ skew.rs     | Differential squared sum scoring              |
| Dewarping                 | ✅ dewarp1-4.c                | ✅ dewarp/     | Single page + Dewarpa (multi-page management) |
| Baseline detection        | ✅ baseline.c                 | ✅ baseline.rs | Horizontal projection method                  |
| Character recognition     | ✅ recogbasic.c, recogident.c | ✅ recog/      | Template matching, DID                        |
| JBIG2 classification      | ✅ jbclass.c                  | ✅ jbclass/    | RankHaus, correlation-based classification    |
| Barcode                   | ✅ bardecode.c, readbarcode.c | ✅ barcode/    | EAN/UPC/Code39 etc.                           |
| Warper                    | ✅ warper.c                   | ✅ warper.rs   | Harmonic distortion/stereo (91% implemented)  |

## Rust Module Implementation Status

| Module                     | Lines        | Function Coverage       | Effective Coverage      | Key Features                                                                                       |
| -------------------------- | ------------ | ----------------------- | ----------------------- | -------------------------------------------------------------------------------------------------- |
| leptonica (src/core/)      | ~47,100      | 948/1,215 (78.0%)       | 948/1,069 (88.7%)       | Pix, Box, Pta, Ptaa, Pixaa, Colormap, arithmetic, compare, blend, graphics, stats, histogram       |
| leptonica (src/io/)        | ~7,900       | 158/208 (76.0%)         | 158/158 (100.0%)        | BMP/PNG/JPEG/PNM/TIFF/GIF/WebP/JP2K/PDF/PS/SPIX + header reading                                   |
| leptonica (src/transform/) | ~11,200      | 128/142 (90.1%)         | 128/128 (100.0%)        | Rotate, scale, affine, projective, shear                                                           |
| leptonica (src/morph/)     | ~9,400       | 138/171 (80.7%)         | 138/138 (100.0%)        | Binary/grayscale/color morphology, DWA, thinning                                                   |
| leptonica (src/filter/)    | ~9,800       | 117/131 (89.3%)         | 117/118 (99.2%)         | Convolution, edge detection, bilateral, rank, adaptive mapping, run-length                         |
| leptonica (src/color/)     | ~7,400       | 122/139 (87.8%)         | 122/122 (100.0%)        | Colorspace conversion, quantization, segmentation, binarization, color analysis, colormap painting |
| leptonica (src/region/)    | ~10,600      | 73/95 (76.8%)           | 73/73 (100.0%)          | Connected components, seed fill, watershed, quadtree, maze                                         |
| leptonica (src/recog/)     | ~16,000      | 179/207 (86.5%)         | 179/189 (94.7%)         | Skew correction, dewarping, character recognition, barcode                                         |
| Other                      | -            | 150/526 (28.5%)         | 150/155 (96.8%)         | Warper, encoding, debug/timing helpers, etc.                                                       |
| **Total**                  | **~147,000** | **2,013/2,834 (71.1%)** | **2,013/2,150 (93.6%)** |                                                                                                    |

## Unimplemented Function Status

Of the 2,150 functions remaining after excluding 684 classified as 🚫 Not needed,
2,013 are implemented (✅ 1,782 + 🔄 231). The remaining 137 (❌) are unimplemented.
Effective coverage is 93.6%.

These figures reflect the `gap-fill audit 2026-05-10`, which enumerated all 2,743 public
C functions in `allheaders.h` and individually reviewed each entry by comparing C function
names with Rust implementation locations. Numa/Pixa and other core data structures are
reclassified as 🔄 (idiomatic Rust API: `push`/`len`/`clone`/etc.), which significantly
increased the 🔄 count. See the "追加検証エントリ" section at the bottom of each
`comparison/*.md` for per-function classifications.

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

- [leptonica (src/core/)](../../porting/comparison/core.md) — 919 functions (🚫 77 not needed)
- [leptonica (src/io/)](../../porting/comparison/io.md) — 202 functions (🚫 45 not needed)
- [leptonica (src/transform/)](../../porting/comparison/transform.md) — 142 functions (🚫 14 not needed)
- [leptonica (src/morph/)](../../porting/comparison/morph.md) — 163 functions (🚫 25 not needed)
- [leptonica (src/filter/)](../../porting/comparison/filter.md) — 120 functions (🚫 13 not needed)
- [leptonica (src/color/)](../../porting/comparison/color.md) — 139 functions (🚫 17 not needed)
- [leptonica (src/region/)](../../porting/comparison/region.md) — 95 functions (🚫 22 not needed)
- [leptonica (src/recog/)](../../porting/comparison/recog.md) — 182 functions (🚫 18 not needed)
- [Other](../../porting/comparison/misc.md) — 324 functions (🚫 181 not needed)

## References

- C source: `reference/leptonica/src/`
- Rust source: module directories under `src/`
- C→Rust file mapping: [`c-file-mapping.md`](../../porting/c-file-mapping.md)
