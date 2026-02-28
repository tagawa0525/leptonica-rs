# C vs Rust Regression Test Comparison

> 🇯🇵 [日本語版](../../porting/test-comparison.md)

Survey date: 2026-03-01 (all 159 C regression tests ported — 100% coverage)

## Overview

C version's `prog/*_reg.c` and Rust version's `tests/*_reg.rs` correspondence.

| Item                      | C version (reference/leptonica) | Rust version (leptonica-rs) |
| ------------------------- | ------------------------------- | --------------------------- |
| Total tests               | **305** (.c)                    | **205 files**               |
| Regression tests          | **160** (*_reg.c)               | **159** (*_reg.rs)          |
| Individual test functions | Many                            | **3,270**                   |
| Test runner               | alltests_reg.c                  | `cargo test`                |

※ `alltests_reg.c` is excluded from the C count as it is a test runner (159 files are the target).
※ C test classification is based on the actual crate placement of the corresponding Rust test files.

## Full Test Correspondence Table

Legend:

- ✅ Rust regression test with same name as C version exists
- ❌ Not ported

### leptonica (src/core/) (Pix, Box, Numa, FPix, Pta, Pixa, etc.)

※ encoding→io, expand/multitype/smallpix→transform, rectangle→region reclassified. overlap←moved from region.

| C test     | Rust equivalent   | Status |
| ---------- | ----------------- | ------ |
| boxa1      | boxa1_reg.rs      | ✅     |
| boxa2      | boxa2_reg.rs      | ✅     |
| boxa3      | boxa3_reg.rs      | ✅     |
| boxa4      | boxa4_reg.rs      | ✅     |
| bytea      | bytea_reg.rs      | ✅     |
| compare    | compare_reg.rs    | ✅     |
| conversion | conversion_reg.rs | ✅     |
| dna        | dna_reg.rs        | ✅     |
| equal      | equal_reg.rs      | ✅     |
| extrema    | extrema_reg.rs    | ✅     |
| fpix1      | fpix1_reg.rs      | ✅     |
| fpix2      | fpix2_reg.rs      | ✅     |
| hash       | hash_reg.rs       | ✅     |
| heap       | heap_reg.rs       | ✅     |
| insert     | insert_reg.rs     | ✅     |
| logicops   | logicops_reg.rs   | ✅     |
| lowaccess  | lowaccess_reg.rs  | ✅     |
| numa1      | numa1_reg.rs      | ✅     |
| numa2      | numa2_reg.rs      | ✅     |
| numa3      | numa3_reg.rs      | ✅     |
| overlap    | overlap_reg.rs    | ✅     |
| pixa1      | pixa1_reg.rs      | ✅     |
| pixa2      | pixa2_reg.rs      | ✅     |
| pixalloc   | pixalloc_reg.rs   | ✅     |
| pixcomp    | pixcomp_reg.rs    | ✅     |
| pixmem     | pixmem_reg.rs     | ✅     |
| pixserial  | pixserial_reg.rs  | ✅     |
| pta        | pta_reg.rs        | ✅     |
| ptra1      | ptra1_reg.rs      | ✅     |
| ptra2      | ptra2_reg.rs      | ✅     |
| rasterop   | rasterop_reg.rs   | ✅     |
| rasteropip | rasteropip_reg.rs | ✅     |
| string     | string_reg.rs     | ✅     |

Rust-only: boxfunc, numa_sort_interp, pix_arith_rop, pix_clip_advanced, pix_clip_advanced_ext, pix_histogram_advanced, pix_stats_advanced, pixafunc

✅ 33 / ❌ 0 (out of 33 C tests)

### leptonica (src/io/) (Image I/O)

※ encoding←moved from core.

| C test     | Rust equivalent   | Status |
| ---------- | ----------------- | ------ |
| encoding   | encoding_reg.rs   | ✅     |
| files      | files_reg.rs      | ✅     |
| gifio      | gifio_reg.rs      | ✅     |
| ioformats  | ioformats_reg.rs  | ✅     |
| iomisc     | iomisc_reg.rs     | ✅     |
| jp2kio     | jp2kio_reg.rs     | ✅     |
| jpegio     | jpegio_reg.rs     | ✅     |
| mtiff      | mtiff_reg.rs      | ✅     |
| pdfio1     | pdfio1_reg.rs     | ✅     |
| pdfio2     | pdfio2_reg.rs     | ✅     |
| pdfseg     | pdfseg_reg.rs     | ✅     |
| pixtile    | pixtile_reg.rs    | ✅     |
| pngio      | pngio_reg.rs      | ✅     |
| pnmio      | pnmio_reg.rs      | ✅     |
| psio       | psio_reg.rs       | ✅     |
| psioseg    | psioseg_reg.rs    | ✅     |
| webpanimio | webpanimio_reg.rs | ✅     |
| webpio     | webpio_reg.rs     | ✅     |
| writetext  | writetext_reg.rs  | ✅     |

Rust-only: spixio

✅ 19 / ❌ 0 (out of 19 C tests)

### leptonica (src/morph/) (Morphological operations)

| C test     | Rust equivalent   | Status |
| ---------- | ----------------- | ------ |
| binmorph1  | binmorph1_reg.rs  | ✅     |
| binmorph2  | binmorph2_reg.rs  | ✅     |
| binmorph3  | binmorph3_reg.rs  | ✅     |
| binmorph4  | binmorph4_reg.rs  | ✅     |
| binmorph5  | binmorph5_reg.rs  | ✅     |
| binmorph6  | binmorph6_reg.rs  | ✅     |
| ccthin1    | ccthin1_reg.rs    | ✅     |
| ccthin2    | ccthin2_reg.rs    | ✅     |
| colormorph | colormorph_reg.rs | ✅     |
| dwamorph1  | dwamorph1_reg.rs  | ✅     |
| dwamorph2  | dwamorph2_reg.rs  | ✅     |
| fhmtauto   | fhmtauto_reg.rs   | ✅     |
| fmorphauto | fmorphauto_reg.rs | ✅     |
| graymorph1 | graymorph1_reg.rs | ✅     |
| graymorph2 | graymorph2_reg.rs | ✅     |
| morphseq   | morphseq_reg.rs   | ✅     |
| selio      | selio_reg.rs      | ✅     |

Rust-only: sel_morphapp

✅ 17 / ❌ 0 (out of 17 C tests)

### leptonica (src/transform/) (Geometric transforms)

※ expand/multitype/smallpix←moved from core.

| C test       | Rust equivalent     | Status |
| ------------ | ------------------- | ------ |
| affine       | affine_reg.rs       | ✅     |
| alphaxform   | alphaxform_reg.rs   | ✅     |
| bilinear     | bilinear_reg.rs     | ✅     |
| checkerboard | checkerboard_reg.rs | ✅     |
| circle       | circle_reg.rs       | ✅     |
| crop         | crop_reg.rs         | ✅     |
| expand       | expand_reg.rs       | ✅     |
| multitype    | multitype_reg.rs    | ✅     |
| projection   | projection_reg.rs   | ✅     |
| projective   | projective_reg.rs   | ✅     |
| rotate1      | rotate1_reg.rs      | ✅     |
| rotate2      | rotate2_reg.rs      | ✅     |
| rotateorth   | rotateorth_reg.rs   | ✅     |
| scale        | scale_reg.rs        | ✅     |
| shear1       | shear1_reg.rs       | ✅     |
| shear2       | shear2_reg.rs       | ✅     |
| smallpix     | smallpix_reg.rs     | ✅     |
| subpixel     | subpixel_reg.rs     | ✅     |
| translate    | translate_reg.rs    | ✅     |
| warper       | warper_reg.rs       | ✅     |
| xformbox     | xformbox_reg.rs     | ✅     |

✅ 21 / ❌ 0 (out of 21 C tests)

### leptonica (src/filter/) (Filtering)

※ lowsat←moved from color.

| C test     | Rust equivalent   | Status |
| ---------- | ----------------- | ------ |
| adaptmap   | adaptmap_reg.rs   | ✅     |
| adaptnorm  | adaptnorm_reg.rs  | ✅     |
| bilateral1 | bilateral1_reg.rs | ✅     |
| bilateral2 | bilateral2_reg.rs | ✅     |
| compfilter | compfilter_reg.rs | ✅     |
| convolve   | convolve_reg.rs   | ✅     |
| edge       | edge_reg.rs       | ✅     |
| enhance    | enhance_reg.rs    | ✅     |
| kernel     | kernel_reg.rs     | ✅     |
| locminmax  | locminmax_reg.rs  | ✅     |
| lowsat     | lowsat_reg.rs     | ✅     |
| rank       | rank_reg.rs       | ✅     |
| rankbin    | rankbin_reg.rs    | ✅     |
| rankhisto  | rankhisto_reg.rs  | ✅     |

Rust-only: adaptmap_advanced, adaptmap_bg, adaptmap_morph, bilateral_fast, extend_replication

✅ 14 / ❌ 0 (out of 14 C tests)

### leptonica (src/color/) (Color processing, binarization, blend)

※ blend1–5 are implemented in leptonica (src/color/) in Rust. lowsat→reclassified to filter.

| C test       | Rust equivalent     | Status |
| ------------ | ------------------- | ------ |
| alphaops     | alphaops_reg.rs     | ✅     |
| binarize     | binarize_reg.rs     | ✅     |
| blackwhite   | blackwhite_reg.rs   | ✅     |
| blend1       | blend1_reg.rs       | ✅     |
| blend2       | blend2_reg.rs       | ✅     |
| blend3       | blend3_reg.rs       | ✅     |
| blend4       | blend4_reg.rs       | ✅     |
| blend5       | blend5_reg.rs       | ✅     |
| cmapquant    | cmapquant_reg.rs    | ✅     |
| colorcontent | colorcontent_reg.rs | ✅     |
| colorfill    | colorfill_reg.rs    | ✅     |
| coloring     | coloring_reg.rs     | ✅     |
| colorize     | colorize_reg.rs     | ✅     |
| colormask    | colormask_reg.rs    | ✅     |
| colorquant   | colorquant_reg.rs   | ✅     |
| colorseg     | colorseg_reg.rs     | ✅     |
| colorspace   | colorspace_reg.rs   | ✅     |
| dither       | dither_reg.rs       | ✅     |
| falsecolor   | falsecolor_reg.rs   | ✅     |
| grayquant    | grayquant_reg.rs    | ✅     |
| hardlight    | hardlight_reg.rs    | ✅     |
| paint        | paint_reg.rs        | ✅     |
| paintmask    | paintmask_reg.rs    | ✅     |
| threshnorm   | threshnorm_reg.rs   | ✅     |

Rust-only: binarize_advanced, color_magnitude, colorcontent_advanced, colorspace_hsv, quantize_ext

✅ 24 / ❌ 0 (out of 24 C tests)

### leptonica (src/region/) (Region analysis)

※ overlap→moved to core. rectangle←moved from core.

| C test      | Rust equivalent    | Status |
| ----------- | ------------------ | ------ |
| ccbord      | ccbord_reg.rs      | ✅     |
| conncomp    | conncomp_reg.rs    | ✅     |
| distance    | distance_reg.rs    | ✅     |
| grayfill    | grayfill_reg.rs    | ✅     |
| label       | label_reg.rs       | ✅     |
| maze        | maze_reg.rs        | ✅     |
| quadtree    | quadtree_reg.rs    | ✅     |
| rectangle   | rectangle_reg.rs   | ✅     |
| seedspread  | seedspread_reg.rs  | ✅     |
| smoothedge  | smoothedge_reg.rs  | ✅     |
| speckle     | speckle_reg.rs     | ✅     |
| splitcomp   | splitcomp_reg.rs   | ✅     |
| texturefill | texturefill_reg.rs | ✅     |
| watershed   | watershed_reg.rs   | ✅     |

Rust-only: conncomp_ext, seedfill_ext

✅ 14 / ❌ 0 (out of 14 C tests)

### leptonica (src/recog/) (Recognition, page analysis)

| C test       | Rust equivalent     | Status |
| ------------ | ------------------- | ------ |
| baseline     | baseline_reg.rs     | ✅     |
| dewarp       | dewarp_reg.rs       | ✅     |
| findcorners  | findcorners_reg.rs  | ✅     |
| findpattern1 | findpattern1_reg.rs | ✅     |
| findpattern2 | findpattern2_reg.rs | ✅     |
| flipdetect   | flipdetect_reg.rs   | ✅     |
| genfonts     | genfonts_reg.rs     | ✅     |
| italic       | italic_reg.rs       | ✅     |
| jbclass      | jbclass_reg.rs      | ✅     |
| lineremoval  | lineremoval_reg.rs  | ✅     |
| nearline     | nearline_reg.rs     | ✅     |
| newspaper    | newspaper_reg.rs    | ✅     |
| pageseg      | pageseg_reg.rs      | ✅     |
| partition    | partition_reg.rs    | ✅     |
| pixadisp     | pixadisp_reg.rs     | ✅     |
| skew         | skew_reg.rs         | ✅     |
| wordboxes    | wordboxes_reg.rs    | ✅     |

✅ 17 / ❌ 0 (out of 17 C tests)

## Summary

### Coverage by Module

| Module                     | C       | ✅      | ❌    | Rust-only | Coverage   |
| -------------------------- | ------- | ------- | ----- | --------- | ---------- |
| leptonica (src/core/)      | 33      | 33      | 0     | 8         | 100.0%     |
| leptonica (src/io/)        | 19      | 19      | 0     | 1         | 100.0%     |
| leptonica (src/morph/)     | 17      | 17      | 0     | 1         | 100.0%     |
| leptonica (src/transform/) | 21      | 21      | 0     | 0         | 100.0%     |
| leptonica (src/filter/)    | 14      | 14      | 0     | 5         | 100.0%     |
| leptonica (src/color/)     | 24      | 24      | 0     | 5         | 100.0%     |
| leptonica (src/region/)    | 14      | 14      | 0     | 2         | 100.0%     |
| leptonica (src/recog/)     | 17      | 17      | 0     | 0         | 100.0%     |
| **Total**                  | **159** | **159** | **0** | **22**    | **100.0%** |

All 159 C regression tests have Rust counterparts — no unported tests remain.

## Rust Test Status

### Structure (Rust version)

- `#[cfg(test)]` modules within each `src/*.rs` (unit tests)
- Integration tests under `tests/` (205 files, corresponding to C's `*_reg.c`)
- Test data: `tests/data/images/` (real images)
- Test output: `tests/regout/` (.gitignore target, generated with REGTEST_MODE=generate)

## Quality Comparison

| Aspect                | C version                        | Rust version                      |
| --------------------- | -------------------------------- | --------------------------------- |
| **Regression tests**  | Golden file comparison           | ✅ RegParams + golden files       |
| **Visual tests**      | Image output / visual inspection | REGTEST_MODE=display support      |
| **I/O tests**         | All formats covered              | ✅ All formats supported          |
| **Integration tests** | alltests_reg.c                   | 205 files (full regression tests) |
| **Test data**         | Extensive (images, PDFs, etc.)   | Real images in tests/data/images/ |
| **Coverage**          | 159 areas                        | 8 modules, 3,270 test functions   |

## References

- C source: `reference/leptonica/prog/*_reg.c`
- Rust regression tests: `tests/*_reg.rs`
- Regression test mode: `REGTEST_MODE={generate,compare,display}`
- Golden files: `tests/golden/` (committed)
- Test output: `tests/regout/` (.gitignore target)
