# C→Rust ファイル対応表

C版ソースファイル・回帰テストファイルとRust版の対応。
関数レベルの詳細は [comparison/](comparison/) を参照。

## ソースファイル（182件）

対応あり 151件、🚫不要 31件、❌未実装 0件。

## 補足: C 1:1 対応外のRust補助ファイル

以下はCファイルとの1:1対応表には載せていないが、実装上重要なRust側ファイル:

- `src/lib.rs`
- `src/{color,core,filter,io,morph,recog,region,transform}/error.rs`
- `src/core/pixel.rs`, `src/core/pix/serial.rs`
- `src/recog/barcode/formats/{codabar,code2of5,code39,code93,codei2of5,ean13,upca}.rs`
- `src/recog/barcode/types.rs`
- `src/recog/recog/io.rs`, `src/recog/recog/query.rs`

※ `src/io/ps/ascii85.rs` は `psio2.c` の対応先として本表に記載済み。

| C                  | Rust                                                                                                            |
| ------------------ | --------------------------------------------------------------------------------------------------------------- |
| `adaptmap.c`       | src/filter/adaptmap.rs                                                                                          |
| `affine.c`         | src/transform/affine.rs                                                                                         |
| `affinecompose.c`  | src/transform/affine.rs, src/core/pta/transform.rs, src/core/box_/transform.rs                                  |
| `arrayaccess.c`    | src/core/pix/mod.rs                                                                                             |
| `bardecode.c`      | src/recog/barcode/decode.rs                                                                                     |
| `baseline.c`       | src/recog/baseline.rs                                                                                           |
| `bbuffer.c`        | 🚫                                                                                                              |
| `bilateral.c`      | src/filter/bilateral.rs                                                                                         |
| `bilinear.c`       | src/transform/bilinear.rs                                                                                       |
| `binarize.c`       | src/color/threshold.rs                                                                                          |
| `binexpand.c`      | src/transform/binexpand.rs                                                                                      |
| `binreduce.c`      | src/morph/binreduce.rs                                                                                          |
| `blend.c`          | src/core/pix/blend.rs                                                                                           |
| `bmf.c`            | src/core/bmf.rs                                                                                                 |
| `bmpio.c`          | src/io/bmp.rs                                                                                                   |
| `bmpiostub.c`      | src/io/bmp.rs                                                                                                   |
| `bootnumgen1.c`    | src/recog/recog/bootstrap.rs                                                                                    |
| `bootnumgen2.c`    | src/recog/recog/bootstrap.rs                                                                                    |
| `bootnumgen3.c`    | src/recog/recog/bootstrap.rs                                                                                    |
| `bootnumgen4.c`    | src/recog/recog/bootstrap.rs                                                                                    |
| `boxbasic.c`       | src/core/box_/mod.rs, src/core/box_/serial.rs                                                                   |
| `boxfunc1.c`       | src/core/box_/geometry.rs, src/core/box_/adjust.rs                                                              |
| `boxfunc2.c`       | src/core/box_/extract.rs, src/core/box_/transform.rs, src/core/box_/sort.rs                                     |
| `boxfunc3.c`       | src/core/box_/draw.rs                                                                                           |
| `boxfunc4.c`       | src/core/box_/select.rs, src/core/box_/adjust.rs                                                                |
| `boxfunc5.c`       | src/core/box_/smooth.rs                                                                                         |
| `bytearray.c`      | 🚫                                                                                                              |
| `ccbord.c`         | src/region/ccbord.rs                                                                                            |
| `ccthin.c`         | src/morph/thin.rs                                                                                               |
| `checkerboard.c`   | src/region/checkerboard.rs                                                                                      |
| `classapp.c`       | src/recog/classapp.rs                                                                                           |
| `colorcontent.c`   | src/color/analysis.rs                                                                                           |
| `colorfill.c`      | src/color/colorfill.rs                                                                                          |
| `coloring.c`       | src/color/coloring.rs                                                                                           |
| `colormap.c`       | src/core/colormap/mod.rs, src/core/colormap/convert.rs, src/core/colormap/query.rs, src/core/colormap/serial.rs |
| `colormorph.c`     | src/morph/color.rs                                                                                              |
| `colorquant1.c`    | src/color/quantize.rs                                                                                           |
| `colorquant2.c`    | src/color/quantize.rs                                                                                           |
| `colorseg.c`       | src/color/segment.rs                                                                                            |
| `colorspace.c`     | src/color/colorspace.rs                                                                                         |
| `compare.c`        | src/core/pix/compare.rs                                                                                         |
| `conncomp.c`       | src/region/conncomp.rs                                                                                          |
| `convertfiles.c`   | src/io/convertfiles.rs                                                                                          |
| `convolve.c`       | src/filter/convolve.rs, src/filter/block_conv.rs, src/filter/windowed.rs                                        |
| `correlscore.c`    | src/recog/correlscore.rs                                                                                        |
| `dewarp1.c`        | src/recog/dewarp/types.rs, src/recog/dewarp/dewarpa.rs, src/recog/dewarp/io.rs                                  |
| `dewarp2.c`        | src/recog/dewarp/model.rs, src/recog/dewarp/textline.rs                                                         |
| `dewarp3.c`        | src/recog/dewarp/apply.rs                                                                                       |
| `dewarp4.c`        | src/recog/dewarp/single_page.rs                                                                                 |
| `dnabasic.c`       | 🚫                                                                                                              |
| `dnafunc1.c`       | 🚫                                                                                                              |
| `dnahash.c`        | 🚫                                                                                                              |
| `dwacomb.2.c`      | src/morph/dwa.rs                                                                                                |
| `dwacomblow.2.c`   | src/morph/dwa.rs                                                                                                |
| `edge.c`           | src/filter/edge.rs                                                                                              |
| `encoding.c`       | src/core/encoding.rs                                                                                            |
| `enhance.c`        | src/filter/enhance.rs                                                                                           |
| `fhmtauto.c`       | 🚫                                                                                                              |
| `fhmtgen.1.c`      | 🚫                                                                                                              |
| `fhmtgenlow.1.c`   | 🚫                                                                                                              |
| `finditalic.c`     | src/recog/finditalic.rs                                                                                         |
| `flipdetect.c`     | src/recog/flipdetect.rs                                                                                         |
| `fmorphauto.c`     | 🚫                                                                                                              |
| `fmorphgen.1.c`    | 🚫                                                                                                              |
| `fmorphgenlow.1.c` | 🚫                                                                                                              |
| `fpix1.c`          | src/core/fpix/mod.rs, src/core/fpix/serial.rs                                                                   |
| `fpix2.c`          | src/core/fpix/mod.rs                                                                                            |
| `gifio.c`          | src/io/gif.rs                                                                                                   |
| `gifiostub.c`      | src/io/gif.rs                                                                                                   |
| `gplot.c`          | src/core/gplot.rs                                                                                               |
| `graphics.c`       | src/core/pix/graphics.rs                                                                                        |
| `graymorph.c`      | src/morph/grayscale.rs                                                                                          |
| `grayquant.c`      | src/color/threshold.rs                                                                                          |
| `hashmap.c`        | 🚫                                                                                                              |
| `heap.c`           | 🚫                                                                                                              |
| `jbclass.c`        | src/recog/jbclass/mod.rs, src/recog/jbclass/types.rs, src/recog/jbclass/classify.rs, src/recog/jbclass/io.rs    |
| `jp2kheader.c`     | src/io/jp2k.rs                                                                                                  |
| `jp2kheaderstub.c` | src/io/jp2k.rs                                                                                                  |
| `jp2kio.c`         | src/io/jp2k.rs                                                                                                  |
| `jp2kiostub.c`     | src/io/jp2k.rs                                                                                                  |
| `jpegio.c`         | src/io/jpeg.rs                                                                                                  |
| `jpegiostub.c`     | src/io/jpeg.rs                                                                                                  |
| `kernel.c`         | src/filter/kernel.rs                                                                                            |
| `leptwin.c`        | 🚫                                                                                                              |
| `libversions.c`    | 🚫                                                                                                              |
| `list.c`           | 🚫                                                                                                              |
| `map.c`            | 🚫                                                                                                              |
| `maze.c`           | src/region/maze.rs                                                                                              |
| `morph.c`          | src/morph/binary.rs                                                                                             |
| `morphapp.c`       | src/morph/morphapp.rs                                                                                           |
| `morphdwa.c`       | src/morph/dwa.rs                                                                                                |
| `morphseq.c`       | src/morph/sequence.rs                                                                                           |
| `numabasic.c`      | src/core/numa/mod.rs, src/core/numa/serial.rs                                                                   |
| `numafunc1.c`      | src/core/numa/interpolation.rs, src/core/numa/sort.rs, src/core/numa/operations.rs                              |
| `numafunc2.c`      | src/core/numa/operations.rs, src/core/numa/histogram.rs                                                         |
| `pageseg.c`        | src/recog/pageseg.rs                                                                                            |
| `paintcmap.c`      | src/color/paintcmap.rs                                                                                          |
| `parseprotos.c`    | 🚫                                                                                                              |
| `partify.c`        | src/io/partify.rs                                                                                               |
| `partition.c`      | src/region/partition.rs                                                                                         |
| `pdfapp.c`         | src/io/pdf.rs                                                                                                   |
| `pdfappstub.c`     | src/io/pdf.rs                                                                                                   |
| `pdfio1.c`         | src/io/pdf.rs                                                                                                   |
| `pdfio1stub.c`     | src/io/pdf.rs                                                                                                   |
| `pdfio2.c`         | src/io/pdf.rs                                                                                                   |
| `pdfio2stub.c`     | src/io/pdf.rs                                                                                                   |
| `pix1.c`           | src/core/pix/mod.rs                                                                                             |
| `pix2.c`           | src/core/pix/access.rs, src/core/pix/rgb.rs, src/core/pix/clip.rs, src/core/pix/border.rs                       |
| `pix3.c`           | src/core/pix/mask.rs, src/core/pix/statistics.rs, src/core/pix/rop.rs, src/core/pix/arith.rs                    |
| `pix4.c`           | src/core/pix/histogram.rs, src/core/pix/statistics.rs, src/core/pix/access.rs                                   |
| `pix5.c`           | src/core/pix/clip.rs, src/core/pix/extract.rs, src/core/pix/measurement.rs                                      |
| `pixabasic.c`      | src/core/pixa/mod.rs, src/core/pixa/serial.rs                                                                   |
| `pixacc.c`         | src/core/pixacc.rs                                                                                              |
| `pixafunc1.c`      | src/core/pixa/mod.rs, src/region/select.rs                                                                      |
| `pixafunc2.c`      | src/core/pixa/mod.rs                                                                                            |
| `pixalloc.c`       | 🚫                                                                                                              |
| `pixarith.c`       | src/core/pix/arith.rs                                                                                           |
| `pixcomp.c`        | src/core/pixcomp.rs                                                                                             |
| `pixconv.c`        | src/core/pix/convert.rs                                                                                         |
| `pixlabel.c`       | src/region/label.rs, src/region/conncomp.rs                                                                     |
| `pixtiling.c`      | src/core/pixtiling.rs                                                                                           |
| `pngio.c`          | src/io/png.rs                                                                                                   |
| `pngiostub.c`      | src/io/png.rs                                                                                                   |
| `pnmio.c`          | src/io/pnm.rs                                                                                                   |
| `pnmiostub.c`      | src/io/pnm.rs                                                                                                   |
| `projective.c`     | src/transform/projective.rs                                                                                     |
| `psio1.c`          | src/io/ps/mod.rs                                                                                                |
| `psio1stub.c`      | src/io/ps/mod.rs                                                                                                |
| `psio2.c`          | src/io/ps/mod.rs, src/io/ps/ascii85.rs                                                                          |
| `psio2stub.c`      | src/io/ps/mod.rs                                                                                                |
| `ptabasic.c`       | src/core/pta/mod.rs, src/core/pta/serial.rs                                                                     |
| `ptafunc1.c`       | src/core/pta/lsf.rs, src/core/pta/transform.rs                                                                  |
| `ptafunc2.c`       | src/core/pta/sort.rs                                                                                            |
| `ptra.c`           | 🚫                                                                                                              |
| `quadtree.c`       | src/region/quadtree.rs                                                                                          |
| `queue.c`          | 🚫                                                                                                              |
| `rank.c`           | src/filter/rank.rs                                                                                              |
| `rbtree.c`         | 🚫                                                                                                              |
| `readbarcode.c`    | src/recog/barcode/detect.rs, src/recog/barcode/signal.rs                                                        |
| `readfile.c`       | src/io/mod.rs, src/io/header.rs, src/io/format.rs                                                               |
| `recogbasic.c`     | src/recog/recog/mod.rs, src/recog/recog/types.rs                                                                |
| `recogdid.c`       | src/recog/recog/did.rs                                                                                          |
| `recogident.c`     | src/recog/recog/ident.rs                                                                                        |
| `recogtrain.c`     | src/recog/recog/train.rs                                                                                        |
| `regutils.c`       | 🚫                                                                                                              |
| `renderpdf.c`      | 🚫                                                                                                              |
| `rop.c`            | src/core/pix/rop.rs                                                                                             |
| `roplow.c`         | src/core/pix/rop.rs                                                                                             |
| `rotate.c`         | src/transform/rotate.rs                                                                                         |
| `rotateam.c`       | src/transform/rotate.rs                                                                                         |
| `rotateorth.c`     | src/transform/rotate.rs                                                                                         |
| `rotateshear.c`    | src/transform/rotate.rs                                                                                         |
| `runlength.c`      | src/filter/runlength.rs                                                                                         |
| `sarray1.c`        | src/core/sarray/mod.rs, src/core/sarray/serial.rs                                                               |
| `sarray2.c`        | src/core/sarray/mod.rs                                                                                          |
| `scale1.c`         | src/transform/scale.rs                                                                                          |
| `scale2.c`         | src/transform/scale.rs                                                                                          |
| `seedfill.c`       | src/region/seedfill.rs                                                                                          |
| `sel1.c`           | src/morph/sel.rs                                                                                                |
| `sel2.c`           | src/morph/thin_sels.rs                                                                                          |
| `selgen.c`         | src/morph/selgen.rs                                                                                             |
| `shear.c`          | src/transform/shear.rs                                                                                          |
| `skew.c`           | src/recog/skew.rs                                                                                               |
| `spixio.c`         | src/io/spix.rs                                                                                                  |
| `stack.c`          | 🚫                                                                                                              |
| `stringcode.c`     | 🚫                                                                                                              |
| `strokes.c`        | src/recog/strokes.rs                                                                                            |
| `sudoku.c`         | 🚫                                                                                                              |
| `textops.c`        | src/core/bmf.rs                                                                                                 |
| `tiffio.c`         | src/io/tiff.rs                                                                                                  |
| `tiffiostub.c`     | src/io/tiff.rs                                                                                                  |
| `utils1.c`         | 🚫                                                                                                              |
| `utils2.c`         | 🚫                                                                                                              |
| `warper.c`         | src/transform/warper.rs                                                                                         |
| `watershed.c`      | src/region/watershed.rs                                                                                         |
| `webpanimio.c`     | src/io/webp.rs                                                                                                  |
| `webpanimiostub.c` | src/io/webp.rs                                                                                                  |
| `webpio.c`         | src/io/webp.rs                                                                                                  |
| `webpiostub.c`     | src/io/webp.rs                                                                                                  |
| `writefile.c`      | src/io/mod.rs, src/io/header.rs                                                                                 |
| `zlibmem.c`        | 🚫                                                                                                              |
| `zlibmemstub.c`    | 🚫                                                                                                              |

## 回帰テスト（160件）

対応あり 140件、🚫不要 12件、❌未実装 8件。

| C                    | Rust                                |
| -------------------- | ----------------------------------- |
| `adaptmap_reg.c`     | tests/filter/adaptmap_reg.rs        |
| `adaptnorm_reg.c`    | tests/filter/adaptnorm_reg.rs       |
| `affine_reg.c`       | tests/transform/affine_reg.rs       |
| `alltests_reg.c`     | 🚫                                  |
| `alphaops_reg.c`     | tests/color/alphaops_reg.rs         |
| `alphaxform_reg.c`   | tests/transform/alphaxform_reg.rs   |
| `baseline_reg.c`     | tests/recog/baseline_reg.rs         |
| `bilateral1_reg.c`   | tests/filter/bilateral1_reg.rs      |
| `bilateral2_reg.c`   | tests/filter/bilateral2_reg.rs      |
| `bilinear_reg.c`     | tests/transform/bilinear_reg.rs     |
| `binarize_reg.c`     | tests/color/binarize_reg.rs         |
| `binmorph1_reg.c`    | tests/morph/binmorph1_reg.rs        |
| `binmorph2_reg.c`    | tests/morph/binmorph2_reg.rs        |
| `binmorph3_reg.c`    | tests/morph/binmorph3_reg.rs        |
| `binmorph4_reg.c`    | tests/morph/binmorph4_reg.rs        |
| `binmorph5_reg.c`    | tests/morph/binmorph5_reg.rs        |
| `binmorph6_reg.c`    | tests/morph/binmorph6_reg.rs        |
| `blackwhite_reg.c`   | tests/color/blackwhite_reg.rs       |
| `blend1_reg.c`       | tests/color/blend1_reg.rs           |
| `blend2_reg.c`       | tests/color/blend2_reg.rs           |
| `blend3_reg.c`       | tests/color/blend3_reg.rs           |
| `blend4_reg.c`       | tests/color/blend4_reg.rs           |
| `blend5_reg.c`       | tests/color/blend5_reg.rs           |
| `boxa1_reg.c`        | tests/core/boxa1_reg.rs             |
| `boxa2_reg.c`        | tests/core/boxa2_reg.rs             |
| `boxa3_reg.c`        | tests/core/boxa3_reg.rs             |
| `boxa4_reg.c`        | tests/core/boxa4_reg.rs             |
| `bytea_reg.c`        | 🚫                                  |
| `ccbord_reg.c`       | tests/region/ccbord_reg.rs          |
| `ccthin1_reg.c`      | tests/morph/ccthin1_reg.rs          |
| `ccthin2_reg.c`      | tests/morph/ccthin2_reg.rs          |
| `checkerboard_reg.c` | tests/transform/checkerboard_reg.rs |
| `circle_reg.c`       | tests/transform/circle_reg.rs       |
| `cmapquant_reg.c`    | tests/color/cmapquant_reg.rs        |
| `colorcontent_reg.c` | tests/color/colorcontent_reg.rs     |
| `colorfill_reg.c`    | tests/color/colorfill_reg.rs        |
| `coloring_reg.c`     | tests/color/coloring_reg.rs         |
| `colorize_reg.c`     | tests/color/colorize_reg.rs         |
| `colormask_reg.c`    | tests/color/colormask_reg.rs        |
| `colormorph_reg.c`   | tests/morph/colormorph_reg.rs       |
| `colorquant_reg.c`   | tests/color/colorquant_reg.rs       |
| `colorseg_reg.c`     | tests/color/colorseg_reg.rs         |
| `colorspace_reg.c`   | tests/color/colorspace_reg.rs       |
| `compare_reg.c`      | tests/core/compare_reg.rs           |
| `compfilter_reg.c`   | tests/filter/compfilter_reg.rs      |
| `conncomp_reg.c`     | tests/region/conncomp_reg.rs        |
| `conversion_reg.c`   | tests/core/conversion_reg.rs        |
| `convolve_reg.c`     | tests/filter/convolve_reg.rs        |
| `crop_reg.c`         | tests/transform/crop_reg.rs         |
| `dewarp_reg.c`       | tests/recog/dewarp_reg.rs           |
| `distance_reg.c`     | tests/region/distance_reg.rs        |
| `dither_reg.c`       | tests/color/dither_reg.rs           |
| `dna_reg.c`          | 🚫                                  |
| `dwamorph1_reg.c`    | tests/morph/dwamorph1_reg.rs        |
| `dwamorph2_reg.c`    | tests/morph/dwamorph2_reg.rs        |
| `edge_reg.c`         | tests/filter/edge_reg.rs            |
| `encoding_reg.c`     | tests/io/encoding_reg.rs            |
| `enhance_reg.c`      | tests/filter/enhance_reg.rs         |
| `equal_reg.c`        | tests/core/equal_reg.rs             |
| `expand_reg.c`       | tests/transform/expand_reg.rs       |
| `extrema_reg.c`      | tests/core/extrema_reg.rs           |
| `falsecolor_reg.c`   | ❌                                  |
| `fhmtauto_reg.c`     | tests/morph/fhmtauto_reg.rs         |
| `files_reg.c`        | 🚫                                  |
| `findcorners_reg.c`  | ❌                                  |
| `findpattern1_reg.c` | tests/recog/findpattern1_reg.rs     |
| `findpattern2_reg.c` | tests/recog/findpattern2_reg.rs     |
| `flipdetect_reg.c`   | tests/recog/flipdetect_reg.rs       |
| `fmorphauto_reg.c`   | tests/morph/fmorphauto_reg.rs       |
| `fpix1_reg.c`        | tests/core/fpix1_reg.rs             |
| `fpix2_reg.c`        | tests/core/fpix2_reg.rs             |
| `genfonts_reg.c`     | 🚫                                  |
| `gifio_reg.c`        | tests/io/gifio_reg.rs               |
| `grayfill_reg.c`     | tests/region/grayfill_reg.rs        |
| `graymorph1_reg.c`   | tests/morph/graymorph1_reg.rs       |
| `graymorph2_reg.c`   | tests/morph/graymorph2_reg.rs       |
| `grayquant_reg.c`    | tests/color/grayquant_reg.rs        |
| `hardlight_reg.c`    | tests/color/hardlight_reg.rs        |
| `hash_reg.c`         | 🚫                                  |
| `heap_reg.c`         | 🚫                                  |
| `insert_reg.c`       | tests/core/insert_reg.rs            |
| `ioformats_reg.c`    | tests/io/ioformats_reg.rs           |
| `iomisc_reg.c`       | tests/io/iomisc_reg.rs              |
| `italic_reg.c`       | tests/recog/italic_reg.rs           |
| `jbclass_reg.c`      | tests/recog/jbclass_reg.rs          |
| `jp2kio_reg.c`       | tests/io/jp2kio_reg.rs              |
| `jpegio_reg.c`       | tests/io/jpegio_reg.rs              |
| `kernel_reg.c`       | tests/filter/kernel_reg.rs          |
| `label_reg.c`        | tests/region/label_reg.rs           |
| `lineremoval_reg.c`  | tests/recog/lineremoval_reg.rs      |
| `locminmax_reg.c`    | tests/filter/locminmax_reg.rs       |
| `logicops_reg.c`     | tests/core/logicops_reg.rs          |
| `lowaccess_reg.c`    | tests/core/lowaccess_reg.rs         |
| `lowsat_reg.c`       | tests/filter/lowsat_reg.rs          |
| `maze_reg.c`         | tests/region/maze_reg.rs            |
| `morphseq_reg.c`     | tests/morph/morphseq_reg.rs         |
| `mtiff_reg.c`        | tests/io/mtiff_reg.rs               |
| `multitype_reg.c`    | tests/transform/multitype_reg.rs    |
| `nearline_reg.c`     | ❌                                  |
| `newspaper_reg.c`    | tests/recog/newspaper_reg.rs        |
| `numa1_reg.c`        | tests/core/numa1_reg.rs             |
| `numa2_reg.c`        | tests/core/numa2_reg.rs             |
| `numa3_reg.c`        | tests/core/numa3_reg.rs             |
| `overlap_reg.c`      | tests/core/overlap_reg.rs           |
| `pageseg_reg.c`      | tests/recog/pageseg_reg.rs          |
| `paint_reg.c`        | tests/color/paint_reg.rs            |
| `paintmask_reg.c`    | tests/color/paintmask_reg.rs        |
| `partition_reg.c`    | tests/recog/partition_reg.rs        |
| `pdfio1_reg.c`       | tests/io/pdfio1_reg.rs              |
| `pdfio2_reg.c`       | tests/io/pdfio2_reg.rs              |
| `pdfseg_reg.c`       | tests/io/pdfseg_reg.rs              |
| `pixa1_reg.c`        | tests/core/pixa1_reg.rs             |
| `pixa2_reg.c`        | tests/core/pixa2_reg.rs             |
| `pixadisp_reg.c`     | tests/recog/pixadisp_reg.rs         |
| `pixalloc_reg.c`     | 🚫                                  |
| `pixcomp_reg.c`      | tests/core/pixcomp_reg.rs           |
| `pixmem_reg.c`       | 🚫                                  |
| `pixserial_reg.c`    | tests/core/pixserial_reg.rs         |
| `pixtile_reg.c`      | tests/io/pixtile_reg.rs             |
| `pngio_reg.c`        | tests/io/pngio_reg.rs               |
| `pnmio_reg.c`        | tests/io/pnmio_reg.rs               |
| `projection_reg.c`   | tests/transform/projection_reg.rs   |
| `projective_reg.c`   | tests/transform/projective_reg.rs   |
| `psio_reg.c`         | tests/io/psio_reg.rs                |
| `psioseg_reg.c`      | tests/io/psioseg_reg.rs             |
| `pta_reg.c`          | tests/core/pta_reg.rs               |
| `ptra1_reg.c`        | 🚫                                  |
| `ptra2_reg.c`        | 🚫                                  |
| `quadtree_reg.c`     | tests/region/quadtree_reg.rs        |
| `rank_reg.c`         | tests/filter/rank_reg.rs            |
| `rankbin_reg.c`      | tests/filter/rankbin_reg.rs         |
| `rankhisto_reg.c`    | tests/filter/rankhisto_reg.rs       |
| `rasterop_reg.c`     | tests/core/rasterop_reg.rs          |
| `rasteropip_reg.c`   | tests/core/rasteropip_reg.rs        |
| `rectangle_reg.c`    | tests/region/rectangle_reg.rs       |
| `rotate1_reg.c`      | tests/transform/rotate1_reg.rs      |
| `rotate2_reg.c`      | tests/transform/rotate2_reg.rs      |
| `rotateorth_reg.c`   | tests/transform/rotateorth_reg.rs   |
| `scale_reg.c`        | tests/transform/scale_reg.rs        |
| `seedspread_reg.c`   | tests/region/seedspread_reg.rs      |
| `selio_reg.c`        | tests/morph/selio_reg.rs            |
| `shear1_reg.c`       | tests/transform/shear1_reg.rs       |
| `shear2_reg.c`       | tests/transform/shear2_reg.rs       |
| `skew_reg.c`         | tests/recog/skew_reg.rs             |
| `smallpix_reg.c`     | tests/transform/smallpix_reg.rs     |
| `smoothedge_reg.c`   | ❌                                  |
| `speckle_reg.c`      | tests/region/speckle_reg.rs         |
| `splitcomp_reg.c`    | ❌                                  |
| `string_reg.c`       | 🚫                                  |
| `subpixel_reg.c`     | tests/transform/subpixel_reg.rs     |
| `texturefill_reg.c`  | ❌                                  |
| `threshnorm_reg.c`   | tests/color/threshnorm_reg.rs       |
| `translate_reg.c`    | tests/transform/translate_reg.rs    |
| `warper_reg.c`       | tests/transform/warper_reg.rs       |
| `watershed_reg.c`    | tests/region/watershed_reg.rs       |
| `webpanimio_reg.c`   | ❌                                  |
| `webpio_reg.c`       | tests/io/webpio_reg.rs              |
| `wordboxes_reg.c`    | tests/recog/wordboxes_reg.rs        |
| `writetext_reg.c`    | ❌                                  |
| `xformbox_reg.c`     | tests/transform/xformbox_reg.rs     |
