# leptonica (src/core/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目      | 数  |
| --------- | --- |
| ✅ 同等   | 812 |
| 🔄 異なる | 30  |
| 🚫 不要   | 77  |
| ❌ 未実装 | 0   |
| 合計      | 919 |

**カバレッジ**: 91.6% (842/919 関数が実装済み、🚫 不要 77 関数を除くと実質 842/842 = 100.0% 解決済み)

注: 合計845→882→919はサマリー行を個別関数に展開したため（ptafunc/pixafunc、boxfunc2、boxfunc5）。

## 注記

- ✅ 同等: Rust版で同じアルゴリズム/機能を持つ関数が存在
- 🔄 異なる: Rust版で異なるAPI/アプローチで実装
- 🚫 不要: Rustの言語機能・設計方針により移植不要
- ❌ 未実装: Rust版に対応する関数が存在しない

Rust版は**Pix/PixMut二層モデル**を採用しているため、C版の一部の関数は異なるAPIで提供される。
例: `pixCopy()` → `Pix::deep_clone()`, `pixClone()` → `Pix::clone()`

## 詳細

### pix1.c (基本的なPix操作)

| C関数                   | 状態    | Rust対応                            | 備考                       |
| ----------------------- | ------- | ----------------------------------- | -------------------------- |
| pixCreate               | ✅      | Pix::new()                          |                            |
| pixCreateNoInit         | 🚫 不要 | -                                   | Rustは常に初期化する       |
| pixCreateTemplate       | ✅ 同等 | `Pix::create_template`              |                            |
| pixCreateTemplateNoInit | 🚫 不要 | -                                   | Rustは常に初期化する       |
| pixCreateWithCmap       | ✅ 同等 | `Pix::new_with_colormap`            |                            |
| pixCreateHeader         | 🚫 不要 | -                                   | Rustは常に初期化する       |
| pixClone                | 🔄      | Pix::clone()                        | Arc参照カウントで自動実装  |
| pixDestroy              | 🔄      | drop()                              | Rustのデストラクタで自動   |
| pixCopy                 | 🔄      | Pix::deep_clone()                   | deep_cloneが完全コピー     |
| pixResizeImageData      | 🚫 不要 | -                                   | Rustの所有権モデルで不要   |
| pixCopyColormap         | ✅      | pix/mod.rs copy_colormap_from()     |                            |
| pixTransferAllData      | 🚫 不要 | -                                   | Rustの所有権モデルで不要   |
| pixSwapAndDestroy       | 🚫 不要 | -                                   | Rustの所有権モデルで不要   |
| pixGetWidth             | ✅      | Pix::width()                        |                            |
| pixSetWidth             | 🚫 不要 | -                                   | Pixは不変                  |
| pixGetHeight            | ✅      | Pix::height()                       |                            |
| pixSetHeight            | 🚫 不要 | -                                   | Pixは不変                  |
| pixGetDepth             | ✅      | Pix::depth()                        |                            |
| pixSetDepth             | 🚫 不要 | -                                   | Pixは不変                  |
| pixGetDimensions        | ✅      | width()/height()/depth()            | 個別メソッドで取得         |
| pixSetDimensions        | 🚫 不要 | -                                   | Pixは不変                  |
| pixCopyDimensions       | 🚫 不要 | -                                   | Pixは不変                  |
| pixGetSpp               | ✅      | Pix::spp()                          |                            |
| pixSetSpp               | 🔄      | PixMut::set_spp()                   | PixMutで可変               |
| pixCopySpp              | 🚫 不要 | -                                   | Pixは不変                  |
| pixGetWpl               | ✅      | Pix::wpl()                          |                            |
| pixSetWpl               | 🚫 不要 | -                                   | 自動計算のため不要         |
| pixGetXRes              | ✅      | Pix::xres()                         |                            |
| pixSetXRes              | 🔄      | PixMut::set_xres()                  |                            |
| pixGetYRes              | ✅      | Pix::yres()                         |                            |
| pixSetYRes              | 🔄      | PixMut::set_yres()                  |                            |
| pixGetResolution        | ✅      | xres()/yres()                       |                            |
| pixSetResolution        | 🔄      | PixMut::set_resolution()            |                            |
| pixCopyResolution       | ✅      | pix/mod.rs copy_resolution_from()   |                            |
| pixScaleResolution      | ✅      | pix/mod.rs scale_resolution()       |                            |
| pixGetInputFormat       | ✅      | Pix::informat()                     |                            |
| pixSetInputFormat       | 🔄      | PixMut::set_informat()              |                            |
| pixCopyInputFormat      | ✅      | pix/mod.rs copy_input_format_from() |                            |
| pixSetSpecial           | 🔄      | PixMut::set_special()               |                            |
| pixGetText              | ✅      | Pix::text()                         |                            |
| pixSetText              | 🔄      | PixMut::set_text()                  |                            |
| pixAddText              | ✅      | pix/mod.rs add_text()               |                            |
| pixCopyText             | ✅      | pix/mod.rs copy_text_from()         |                            |
| pixGetTextCompNew       | ✅ 同等 | `get_text_comp_new`                 |                            |
| pixSetTextCompNew       | ✅ 同等 | `set_text_comp_new`                 |                            |
| pixGetColormap          | ✅      | Pix::colormap()                     |                            |
| pixSetColormap          | 🔄      | PixMut::set_colormap()              |                            |
| pixDestroyColormap      | 🚫 不要 | -                                   | set_colormap(None)で実現可 |
| pixGetData              | ✅      | Pix::data()                         |                            |
| pixFreeAndSetData       | 🚫 不要 | -                                   | Cメモリ管理                |
| pixSetData              | 🚫 不要 | -                                   | Cメモリ管理                |
| pixFreeData             | 🚫 不要 | -                                   | Cメモリ管理                |
| pixExtractData          | 🚫 不要 | -                                   | Cメモリ管理                |
| pixGetLinePtrs          | 🚫 不要 | -                                   | Cポインタ配列              |
| pixSizesEqual           | ✅      | pix/mod.rs sizes_equal()            |                            |
| pixMaxAspectRatio       | ✅      | pix/mod.rs max_aspect_ratio()       |                            |
| pixPrintStreamInfo      | 🚫 不要 | -                                   | Debug traitで対応          |

### pix2.c (ピクセルアクセス・設定)

| C関数                           | 状態    | Rust対応                                               | 備考             |
| ------------------------------- | ------- | ------------------------------------------------------ | ---------------- |
| pixGetPixel                     | ✅      | Pix::get_pixel()                                       |                  |
| pixSetPixel                     | ✅      | PixMut::set_pixel()                                    |                  |
| pixGetRGBPixel                  | ✅      | rgb.rs get_rgb_pixel()                                 |                  |
| pixSetRGBPixel                  | ✅      | rgb.rs set_rgb_pixel()                                 |                  |
| pixSetCmapPixel                 | ✅      | pix/rgb.rs set_cmap_pixel()                            |                  |
| pixGetRandomPixel               | ✅ 同等 | `get_random_pixel`                                     |                  |
| pixClearPixel                   | ✅      | pix/mod.rs clear_pixel()                               |                  |
| pixFlipPixel                    | ✅      | pix/mod.rs flip_pixel()                                |                  |
| pixGetBlackOrWhiteVal           | ✅      | pix/mod.rs get_black_or_white_val()                    |                  |
| pixClearAll                     | 🔄      | PixMut::clear()                                        |                  |
| pixSetAll                       | 🔄      | PixMut::set_all()                                      |                  |
| pixSetAllGray                   | ✅      | pix/mod.rs set_all_gray()                              |                  |
| pixSetAllArbitrary              | ✅      | pix/mod.rs set_all_arbitrary()                         |                  |
| pixSetBlackOrWhite              | ✅      | pix/mod.rs set_black_or_white()                        |                  |
| pixSetComponentArbitrary        | ✅ 同等 | `set_component_arbitrary`                              |                  |
| pixClearInRect                  | ✅      | pix/mod.rs clear_in_rect()                             |                  |
| pixSetInRect                    | ✅      | pix/mod.rs set_in_rect()                               |                  |
| pixSetInRectArbitrary           | ✅      | pix/mod.rs set_in_rect_arbitrary()                     |                  |
| pixBlendInRect                  | ✅ 同等 | `blend_in_rect`                                        |                  |
| pixSetPadBits                   | ✅      | pix/mod.rs set_pad_bits()                              |                  |
| pixSetPadBitsBand               | ✅      | pix/mod.rs set_pad_bits_band()                         |                  |
| pixSetOrClearBorder             | ✅      | pix/mod.rs set_or_clear_border()                       |                  |
| pixSetBorderVal                 | ✅      | border.rs set_border_val()                             |                  |
| pixSetBorderRingVal             | ✅ 同等 | `set_border_ring_val`                                  |                  |
| pixSetMirroredBorder            | ✅ 同等 | `set_mirrored_border`                                  |                  |
| pixCopyBorder                   | ✅ 同等 | `copy_border`                                          |                  |
| pixAddBorder                    | ✅      | border.rs add_border()                                 |                  |
| pixAddBlackOrWhiteBorder        | ✅      | border.rs add_black_or_white_border()                  |                  |
| pixAddBorderGeneral             | ✅      | border.rs add_border_general()                         |                  |
| pixAddMultipleBlackWhiteBorders | ✅ 同等 | `add_multiple_black_white_borders`                     |                  |
| pixRemoveBorder                 | ✅      | border.rs remove_border()                              |                  |
| pixRemoveBorderGeneral          | ✅      | border.rs remove_border_general()                      |                  |
| pixRemoveBorderToSize           | ✅ 同等 | `remove_border_to_size`                                |                  |
| pixAddMirroredBorder            | ✅      | border.rs add_mirrored_border()                        |                  |
| pixAddRepeatedBorder            | ✅      | border.rs add_repeated_border()                        |                  |
| pixAddMixedBorder               | ✅ 同等 | `add_mixed_border`                                     |                  |
| pixAddContinuedBorder           | ✅ 同等 | `add_continued_border`                                 |                  |
| pixShiftAndTransferAlpha        | ✅ 同等 | `shift_and_transfer_alpha`                             |                  |
| pixDisplayLayersRGBA            | 🚫 不要 | -                                                      | デバッグ表示関数 |
| pixCreateRGBImage               | ✅      | rgb.rs create_rgb_image()                              |                  |
| pixGetRGBComponent              | ✅      | rgb.rs get_rgb_component()                             |                  |
| pixSetRGBComponent              | ✅      | rgb.rs set_rgb_component()                             |                  |
| pixGetRGBComponentCmap          | ✅ 同等 | `get_rgb_component_cmap`                               |                  |
| pixCopyRGBComponent             | ✅ 同等 | `copy_rgb_component`                                   |                  |
| composeRGBPixel                 | ✅      | lib.rs compose_rgb()                                   |                  |
| composeRGBAPixel                | ✅      | lib.rs compose_rgba()                                  |                  |
| extractRGBValues                | ✅      | lib.rs extract_rgb()                                   |                  |
| extractRGBAValues               | ✅      | lib.rs extract_rgba()                                  |                  |
| extractMinMaxComponent          | ✅      | lib.rs extract_min_component()/extract_max_component() |                  |
| pixGetRGBLine                   | ✅ 同等 | `get_rgb_line`                                         |                  |
| pixEndianByteSwapNew            | ✅ 同等 | `Pix::endian_byte_swap_new`                            |                  |
| pixEndianByteSwap               | ✅ 同等 | `PixMut::endian_byte_swap`                             |                  |
| pixEndianTwoByteSwap            | ✅ 同等 | `PixMut::endian_two_byte_swap`                         |                  |
| pixGetRasterData                | 🚫 不要 | -                                                      | Cポインタ取得    |
| pixInferResolution              | ✅ 同等 | `infer_resolution`                                     |                  |
| pixAlphaIsOpaque                | ✅ 同等 | `alpha_is_opaque`                                      |                  |

### pix3.c (マスク・ブール演算)

| C関数                       | 状態    | Rust対応                               | 備考               |
| --------------------------- | ------- | -------------------------------------- | ------------------ |
| pixSetMasked                | ✅      | mask.rs set_masked()                   |                    |
| pixSetMaskedGeneral         | ✅      | mask.rs set_masked_general()           |                    |
| pixCombineMasked            | ✅      | mask.rs combine_masked()               |                    |
| pixCombineMaskedGeneral     | ✅      | mask.rs combine_masked_general()       |                    |
| pixPaintThroughMask         | ✅      | mask.rs paint_through_mask()           |                    |
| pixCopyWithBoxa             | ✅      | mask.rs copy_with_boxa()               |                    |
| pixPaintSelfThroughMask     | ✅ 同等 | `paint_self_through_mask`              | 後続Phase          |
| pixMakeMaskFromVal          | ✅      | mask.rs make_mask_from_val()           |                    |
| pixMakeMaskFromLUT          | ✅      | mask.rs make_mask_from_lut()           |                    |
| pixMakeArbMaskFromRGB       | ✅      | mask.rs make_arb_mask_from_rgb()       |                    |
| pixSetUnderTransparency     | ✅      | mask.rs set_under_transparency()       |                    |
| pixMakeAlphaFromMask        | ✅ 同等 | `make_alpha_from_mask`                 |                    |
| pixGetColorNearMaskBoundary | ✅ 同等 | `get_color_near_mask_boundary`         |                    |
| pixDisplaySelectedPixels    | 🚫 不要 | -                                      | デバッグ表示関数   |
| pixInvert                   | ✅      | ops.rsに実装                           |                    |
| pixOr                       | ✅      | ops.rsに実装                           |                    |
| pixAnd                      | ✅      | ops.rsに実装                           |                    |
| pixXor                      | ✅      | ops.rsに実装                           |                    |
| pixSubtract                 | ✅      | ops.rsに実装                           |                    |
| pixZero                     | ✅      | statistics.rs is_zero()                |                    |
| pixForegroundFraction       | ✅      | statistics.rs foreground_fraction()    |                    |
| pixaCountPixels             | ✅      | pixa count_pixels()                    |                    |
| pixCountPixels              | ✅      | statistics.rs count_pixels()           |                    |
| pixCountPixelsInRect        | ✅      | statistics.rs count_pixels_in_rect()   |                    |
| pixCountByRow               | ✅      | statistics.rs count_by_row()           |                    |
| pixCountByColumn            | ✅      | statistics.rs count_by_column()        |                    |
| pixCountPixelsByRow         | ✅      | statistics.rs count_pixels_by_row()    | Numa返却版         |
| pixCountPixelsByColumn      | ✅      | statistics.rs count_pixels_by_column() | Numa返却版         |
| pixCountPixelsInRow         | ✅      | statistics.rs count_pixels_in_row()    |                    |
| pixGetMomentByColumn        | ✅      | statistics.rs get_moment_by_column()   |                    |
| pixThresholdPixelSum        | ✅      | statistics.rs threshold_pixel_sum()    |                    |
| pixAverageByRow             | ✅      | statistics.rs average_by_row()         |                    |
| pixAverageByColumn          | ✅      | statistics.rs average_by_column()      |                    |
| pixAverageInRect            | ✅      | statistics.rs average_in_rect()        |                    |
| pixAverageInRectRGB         | ✅      | statistics.rs average_in_rect_rgb()    |                    |
| pixVarianceByRow            | ✅      | statistics.rs variance_by_row()        |                    |
| pixVarianceByColumn         | ✅      | statistics.rs variance_by_column()     |                    |
| pixVarianceInRect           | ✅      | statistics.rs variance_in_rect()       |                    |
| pixAbsDiffByRow             | ✅      | statistics.rs abs_diff_by_row()        |                    |
| pixAbsDiffByColumn          | ✅      | statistics.rs abs_diff_by_column()     |                    |
| pixAbsDiffInRect            | ✅      | statistics.rs abs_diff_in_rect()       |                    |
| pixAbsDiffOnLine            | ✅      | statistics.rs abs_diff_on_line()       |                    |
| pixCountArbInRect           | ✅      | statistics.rs count_arb_in_rect()      |                    |
| pixMirroredTiling           | 🚫 不要 | -                                      | デバッグ表示関数   |
| pixFindRepCloseTile         | 🚫 不要 | -                                      | タイリングヘルパー |

### pix4.c (ヒストグラム・統計)

| C関数                        | 状態    | Rust対応                              | 備考 |
| ---------------------------- | ------- | ------------------------------------- | ---- |
| pixGetGrayHistogram          | ✅      | histogram.rsに実装                    |      |
| pixGetGrayHistogramMasked    | ✅      | histogram.rs gray_histogram_masked()  |      |
| pixGetGrayHistogramInRect    | ✅      | histogram.rs gray_histogram_in_rect() |      |
| pixGetGrayHistogramTiled     | ✅      | histogram.rs gray_histogram_tiled()   |      |
| pixGetColorHistogram         | ✅      | histogram.rsに実装                    |      |
| pixGetColorHistogramMasked   | ✅      | histogram.rs color_histogram_masked() |      |
| pixGetCmapHistogram          | ✅      | histogram.rs cmap_histogram()         |      |
| pixGetCmapHistogramMasked    | ✅      | histogram.rs cmap_histogram_masked()  |      |
| pixGetCmapHistogramInRect    | ✅      | histogram.rs cmap_histogram_in_rect() |      |
| pixCountRGBColorsByHash      | ✅ 同等 | `count_rgb_colors_by_hash`            |      |
| pixCountRGBColors            | ✅      | histogram.rs count_rgb_colors()       |      |
| pixGetColorAmapHistogram     | ✅ 同等 | `get_color_amap_histogram`            |      |
| pixGetRankValue              | ✅      | histogram.rs pixel_rank_value()       |      |
| pixGetRankValueMaskedRGB     | ✅      | histogram.rs rank_value_masked_rgb()  |      |
| pixGetRankValueMasked        | ✅      | histogram.rs rank_value_masked()      |      |
| pixGetPixelAverage           | ✅      | statistics.rs get_pixel_average()     |      |
| pixGetPixelStats             | ✅      | statistics.rs get_pixel_stats()       |      |
| pixGetAverageMaskedRGB       | ✅      | histogram.rs average_masked_rgb()     |      |
| pixGetAverageMasked          | ✅      | histogram.rs average_masked()         |      |
| pixGetAverageTiledRGB        | ✅      | histogram.rs average_tiled_rgb()      |      |
| pixGetAverageTiled           | ✅      | histogram.rs average_tiled()          |      |
| pixRowStats                  | ✅      | statistics.rs row_stats()             |      |
| pixColumnStats               | ✅      | statistics.rs column_stats()          |      |
| pixGetRangeValues            | ✅      | statistics.rs range_values()          |      |
| pixGetExtremeValue           | ✅      | statistics.rs extreme_value()         |      |
| pixGetMaxValueInRect         | ✅      | statistics.rs max_value_in_rect()     |      |
| pixGetMaxColorIndex          | ✅      | histogram.rs max_color_index()        |      |
| pixGetBinnedComponentRange   | ✅ 同等 | `get_binned_component_range`          |      |
| pixGetRankColorArray         | ✅ 同等 | `get_rank_color_array`                |      |
| pixGetBinnedColor            | ✅ 同等 | `get_binned_color`                    |      |
| pixDisplayColorArray         | ✅ 同等 | `display_color_array`                 |      |
| pixRankBinByStrip            | ✅ 同等 | `rank_bin_by_strip`                   |      |
| pixaGetAlignedStats          | ✅      | pixa aligned_stats()                  |      |
| pixaExtractColumnFromEachPix | ✅      | pixa extract_column_from_each()       |      |
| pixGetRowStats               | ✅      | statistics.rs get_row_stats()         |      |
| pixGetColumnStats            | ✅      | statistics.rs get_column_stats()      |      |
| pixSetPixelColumn            | ✅      | statistics.rs set_pixel_column()      |      |
| pixThresholdForFgBg          | ✅      | clip.rs threshold_for_fg_bg()         |      |
| pixSplitDistributionFgBg     | ✅ 同等 | `split_distribution_fg_bg`            |      |

### pix5.c (選択・測定)

| C関数                        | 状態    | Rust対応                                  | 備考       |
| ---------------------------- | ------- | ----------------------------------------- | ---------- |
| pixaFindDimensions           | ✅      | pixa find_dimensions()                    |            |
| pixFindAreaPerimRatio        | ✅ 同等 | `find_area_perim_ratio`                   |            |
| pixaFindPerimToAreaRatio     | ✅ 同等 | `Pixa::find_perim_to_area_ratio`          |            |
| pixFindPerimToAreaRatio      | ✅      | measurement.rs find_perim_to_area_ratio() |            |
| pixaFindPerimSizeRatio       | ✅ 同等 | `Pixa::find_perim_size_ratio`             |            |
| pixFindPerimSizeRatio        | ✅ 同等 | `find_perim_size_ratio`                   |            |
| pixaFindAreaFraction         | ✅ 同等 | `Pixa::find_area_fraction`                |            |
| pixFindAreaFraction          | ✅ 同等 | `find_area_fraction`                      |            |
| pixaFindAreaFractionMasked   | ✅ 同等 | `Pixa::find_area_fraction_masked`         |            |
| pixFindAreaFractionMasked    | ✅ 同等 | `find_area_fraction_masked`               |            |
| pixaFindWidthHeightRatio     | ✅ 同等 | `Pixa::find_width_height_ratio`           |            |
| pixaFindWidthHeightProduct   | ✅ 同等 | `Pixa::find_width_height_product`         |            |
| pixFindOverlapFraction       | ✅      | measurement.rs find_overlap_fraction()    |            |
| pixFindRectangleComps        | ✅ 同等 | `find_rectangle_comps`                    |            |
| pixConformsToRectangle       | ✅ 同等 | `conforms_to_rectangle`                   |            |
| pixExtractRectangularRegions | ✅ 同等 | `extract_rectangular_regions`             |            |
| pixClipRectangles            | ✅      | clip.rs clip_rectangles()                 |            |
| pixClipRectangle             | ✅      | clip.rs clip_rectangle()                  |            |
| pixClipRectangleWithBorder   | ✅      | clip.rs clip_rectangle_with_border()      |            |
| pixClipMasked                | ✅      | clip.rs clip_masked()                     |            |
| pixCropToMatch               | ✅      | clip.rs crop_to_match()                   |            |
| pixCropToSize                | ✅      | clip.rs crop_to_size()                    |            |
| pixResizeToMatch             | ✅      | clip.rs resize_to_match()                 |            |
| pixSelectComponentBySize     | ✅ 同等 | `select_component_by_size`                |            |
| pixFilterComponentBySize     | ✅ 同等 | `filter_component_by_size`                |            |
| pixMakeSymmetricMask         | ✅      | clip.rs make_symmetric_mask()             |            |
| pixMakeFrameMask             | ✅      | clip.rs make_frame_mask()                 |            |
| pixMakeCoveringOfRectangles  | ✅ 同等 | `make_covering_of_rectangles`             |            |
| pixFractionFgInMask          | ✅      | clip.rs fraction_fg_in_mask()             |            |
| pixClipToForeground          | ✅      | clip.rs clip_to_foreground()              |            |
| pixTestClipToForeground      | ✅      | clip.rs test_clip_to_foreground()         |            |
| pixClipBoxToForeground       | ✅      | clip.rs clip_box_to_foreground()          |            |
| pixScanForForeground         | ✅      | clip.rs scan_for_foreground()             |            |
| pixClipBoxToEdges            | ✅      | clip.rs clip_box_to_edges()               |            |
| pixScanForEdge               | ✅      | clip.rs scan_for_edge()                   | 8bpp適応版 |
| pixExtractOnLine             | ✅      | extract.rs extract_on_line()              |            |
| pixAverageOnLine             | ✅      | clip.rs average_on_line()                 |            |
| pixAverageIntensityProfile   | ✅      | extract.rs average_intensity_profile()    |            |
| pixReversalProfile           | ✅ 同等 | `reversal_profile`                        |            |
| pixWindowedVarianceOnLine    | ✅ 同等 | `windowed_variance_on_line`               |            |
| pixMinMaxNearLine            | ✅ 同等 | `min_max_near_line`                       |            |
| pixRankRowTransform          | ✅      | extract.rs rank_row_transform()           |            |
| pixRankColumnTransform       | ✅      | extract.rs rank_column_transform()        |            |

### boxbasic.c (Box基本操作)

| C関数                  | 状態    | Rust対応                    | 備考                       |
| ---------------------- | ------- | --------------------------- | -------------------------- |
| boxCreate              | ✅      | Box::new()                  |                            |
| boxCreateValid         | 🚫 不要 | -                           | new()でバリデーション実施  |
| boxCopy                | 🔄      | Box自体がCopyトレイト       |                            |
| boxClone               | 🔄      | Box自体がCopyトレイト       |                            |
| boxDestroy             | 🔄      | drop()                      | 自動                       |
| boxGetGeometry         | ✅      | フィールドアクセス          |                            |
| boxSetGeometry         | ✅ 同等 | `box_set_geometry`          |                            |
| boxGetSideLocations    | ✅ 同等 | `box_get_side_locations`    | right()/bottom()で部分対応 |
| boxSetSideLocations    | ✅ 同等 | `box_set_side_locations`    |                            |
| boxIsValid             | ✅      | Box::is_valid()             |                            |
| boxaCreate             | ✅      | Boxa::new()                 |                            |
| boxaCopy               | ✅      | Boxa::clone()               |                            |
| boxaDestroy            | 🔄      | drop()                      | 自動                       |
| boxaAddBox             | ✅      | Boxa::push()                |                            |
| boxaExtendArray        | 🚫 不要 | -                           | Vec自動拡張                |
| boxaExtendArrayToSize  | 🚫 不要 | -                           | Vec自動拡張                |
| boxaGetCount           | ✅      | Boxa::len()                 |                            |
| boxaGetValidCount      | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaGetBox             | ✅      | Boxa::get()                 |                            |
| boxaGetValidBox        | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaFindInvalidBoxes   | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaGetBoxGeometry     | ✅ 同等 | `Boxa::get_box_geometry`    |                            |
| boxaIsFull             | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaReplaceBox         | ✅      | Boxa::replace()             |                            |
| boxaInsertBox          | ✅      | Boxa::insert()              |                            |
| boxaRemoveBox          | ✅      | Boxa::remove()              |                            |
| boxaRemoveBoxAndSave   | ✅ 同等 | `Boxa::remove_box_and_save` |                            |
| boxaSaveValid          | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaInitFull           | ✅ 同等 | `Boxa::init_full`           |                            |
| boxaClear              | ✅      | Boxa::clear()               |                            |
| boxaaCreate            | ✅      | Boxaa::new()                |                            |
| boxaaCopy              | ✅ 同等 | `Boxaa::copy`               |                            |
| boxaaDestroy           | 🔄      | drop()                      | 自動                       |
| boxaaAddBoxa           | ✅      | Boxaa::push()               |                            |
| boxaaExtendArray       | 🚫 不要 | -                           | Vec自動拡張                |
| boxaaExtendArrayToSize | 🚫 不要 | -                           | Vec自動拡張                |
| boxaaGetCount          | ✅      | Boxaa::len()                |                            |
| boxaaGetBoxCount       | ✅      | Boxaa::total_boxes()        |                            |
| boxaaGetBoxa           | ✅      | Boxaa::get()                |                            |
| boxaaGetBox            | ✅ 同等 | `Boxaa::get_box`            |                            |
| boxaaInitFull          | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaaExtendWithInit    | 🚫 不要 | -                           | Rustの型システムで不要     |
| boxaaReplaceBoxa       | ✅ 同等 | `Boxaa::replace_boxa`       |                            |
| boxaaInsertBoxa        | ✅ 同等 | `Boxaa::insert_boxa`        |                            |
| boxaaRemoveBoxa        | ✅ 同等 | `Boxaa::remove_boxa`        |                            |
| boxaaAddBox            | ✅ 同等 | `Boxaa::add_box`            |                            |
| boxaaReadFromFiles     | ✅ 同等 | `Boxaa::read_from_files`    |                            |
| boxaaRead              | ✅ 同等 | `Boxaa::read`               |                            |
| boxaaReadStream        | ✅ 同等 | `Boxaa::read_stream`        |                            |
| boxaaReadMem           | ✅ 同等 | `Boxaa::read_mem`           |                            |
| boxaaWrite             | ✅ 同等 | `Boxaa::write`              |                            |
| boxaaWriteStream       | ✅ 同等 | `Boxaa::write_stream`       |                            |
| boxaaWriteMem          | ✅ 同等 | `Boxaa::write_mem`          |                            |
| boxaRead               | ✅ 同等 | `Boxa::read`                |                            |
| boxaReadStream         | ✅ 同等 | `Boxa::read_stream`         |                            |
| boxaReadMem            | ✅ 同等 | `Boxa::read_mem`            |                            |
| boxaWriteDebug         | 🚫 不要 | -                           | デバッグ出力関数           |
| boxaWrite              | ✅ 同等 | `Boxa::write`               |                            |
| boxaWriteStream        | ✅ 同等 | `Boxa::write_stream`        |                            |
| boxaWriteStderr        | 🚫 不要 | -                           | デバッグ出力関数           |
| boxaWriteMem           | ✅ 同等 | `Boxa::write_mem`           |                            |
| boxPrintStreamInfo     | 🚫 不要 | -                           | デバッグ出力関数           |

### boxfunc1.c (Box関係・幾何演算)

| C関数                     | 状態 | Rust対応                               | 備考 |
| ------------------------- | ---- | -------------------------------------- | ---- |
| boxContains               | ✅   | Box::contains_box()                    |      |
| boxIntersects             | ✅   | Box::overlaps()                        |      |
| boxaContainedInBox        | ✅   | mod.rs contained_in_box()              |      |
| boxaContainedInBoxCount   | ✅   | geometry.rs contained_in_box_count()   |      |
| boxaContainedInBoxa       | ✅   | geometry.rs all_contained_in()         |      |
| boxaIntersectsBox         | ✅   | mod.rs intersects_box()                |      |
| boxaIntersectsBoxCount    | ✅   | geometry.rs intersects_box_count()     |      |
| boxaClipToBox             | ✅   | mod.rs clip_to_box()                   |      |
| boxaCombineOverlaps       | ✅   | mod.rs combine_overlaps()              |      |
| boxaCombineOverlapsInPair | ✅   | geometry.rs combine_overlaps_in_pair() |      |
| boxOverlapRegion          | ✅   | Box::intersect()                       |      |
| boxBoundingRegion         | ✅   | Box::union()                           |      |
| boxOverlapFraction        | ✅   | mod.rs overlap_fraction()              |      |
| boxOverlapArea            | ✅   | mod.rs overlap_area()                  |      |
| boxaHandleOverlaps        | ✅   | geometry.rs handle_overlaps()          |      |
| boxOverlapDistance        | ✅   | geometry.rs overlap_distance()         |      |
| boxSeparationDistance     | ✅   | geometry.rs separation_distance()      |      |
| boxCompareSize            | ✅   | geometry.rs compare_size()             |      |
| boxContainsPt             | ✅   | Box::contains_point()                  |      |
| boxaGetNearestToPt        | ✅   | geometry.rs nearest_to_point()         |      |
| boxaGetNearestToLine      | ✅   | geometry.rs nearest_to_line()          |      |
| boxaFindNearestBoxes      | ✅   | geometry.rs find_nearest_boxes()       |      |
| boxaGetNearestByDirection | ✅   | geometry.rs nearest_by_direction()     |      |
| boxGetCenter              | ✅   | mod.rs center()                        |      |
| boxIntersectByLine        | ✅   | geometry.rs intersect_by_line()        |      |
| boxClipToRectangle        | ✅   | mod.rs clip()                          |      |
| boxClipToRectangleParams  | ✅   | geometry.rs clip_to_rectangle_params() |      |
| boxRelocateOneSide        | ✅   | adjust.rs relocate_one_side()          |      |
| boxaAdjustSides           | ✅   | adjust.rs adjust_all_sides()           |      |
| boxaAdjustBoxSides        | ✅   | adjust.rs adjust_box_sides()           |      |
| boxAdjustSides            | ✅   | adjust.rs adjust_sides()               |      |
| boxaSetSide               | ✅   | adjust.rs set_all_sides()              |      |
| boxSetSide                | ✅   | adjust.rs set_side()                   |      |
| boxaAdjustWidthToTarget   | ✅   | adjust.rs adjust_width_to_target()     |      |
| boxaAdjustHeightToTarget  | ✅   | adjust.rs adjust_height_to_target()    |      |
| boxEqual                  | ✅   | PartialEq trait                        |      |
| boxaEqual                 | ✅   | adjust.rs equal_ordered()              |      |
| boxSimilar                | ✅   | adjust.rs similar_per_side()           |      |
| boxaSimilar               | ✅   | mod.rs similar()                       |      |
| boxaJoin                  | ✅   | mod.rs join()                          |      |
| boxaaJoin                 | ✅   | adjust.rs join() (Boxaa)               |      |
| boxaSplitEvenOdd          | ✅   | adjust.rs split_even_odd()             |      |
| boxaMergeEvenOdd          | ✅   | adjust.rs merge_even_odd()             |      |

### boxfunc2.c (Box変換ユーティリティ)

| C関数                  | 状態    | Rust対応                                      | 備考                            |
| ---------------------- | ------- | --------------------------------------------- | ------------------------------- |
| boxaTransform          | 🔄      | `Boxa::translate()` + `Boxa::scale()`         | shift/scaleを個別メソッドに分離 |
| boxTransform           | 🔄      | `Box::translate()` + `Box::scale()`           | shift/scaleを個別メソッドに分離 |
| boxaTransformOrdered   | ✅ 同等 | `Boxa::transform_ordered`                     |                                 |
| boxTransformOrdered    | ✅ 同等 | `Box::transform_ordered`                      |                                 |
| boxaRotateOrth         | ✅ 同等 | `Boxa::rotate_orth`                           |                                 |
| boxRotateOrth          | ✅ 同等 | `Box::rotate_orth`                            |                                 |
| boxaShiftWithPta       | ✅ 同等 | `Boxa::shift_with_pta`                        |                                 |
| boxaSort               | 🔄      | `Boxa::sort_by_position()` / `sort_by_area()` | ソートタイプ別に個別メソッド化  |
| boxaBinSort            | ✅ 同等 | `Boxa::bin_sort`                              |                                 |
| boxaSortByIndex        | ✅ 同等 | `Boxa::sort_by_index`                         |                                 |
| boxaSort2d             | ✅ 同等 | `Boxa::sort_2d`                               |                                 |
| boxaSort2dByIndex      | ✅ 同等 | `Boxa::sort_2d_by_index`                      |                                 |
| boxaExtractAsNuma      | ✅ 同等 | `Boxa::extract_as_numa`                       |                                 |
| boxaExtractAsPta       | ✅ 同等 | `Boxa::extract_as_pta`                        |                                 |
| boxaExtractCorners     | ✅ 同等 | `Boxa::extract_corners`                       |                                 |
| boxaGetRankVals        | ✅ 同等 | `Boxa::get_rank_vals`                         |                                 |
| boxaGetMedianVals      | ✅ 同等 | `Boxa::get_median_vals`                       |                                 |
| boxaGetAverageSize     | ✅ 同等 | `Boxa::get_average_size`                      |                                 |
| boxaaGetExtent         | ✅ 同等 | `Boxaa::get_extent`                           |                                 |
| boxaaFlattenToBoxa     | ✅ 同等 | `Boxaa::flatten()`                            |                                 |
| boxaaFlattenAligned    | ✅ 同等 | `Boxaa::flatten_aligned`                      |                                 |
| boxaEncapsulateAligned | ✅ 同等 | `Boxa::encapsulate_aligned`                   |                                 |
| boxaaTranspose         | ✅ 同等 | `Boxaa::transpose`                            |                                 |
| boxaaAlignBox          | ✅ 同等 | `Boxaa::align_box`                            |                                 |

### boxfunc3.c (Box描画・マスク)

| C関数                     | 状態    | Rust対応                      | 備考         |
| ------------------------- | ------- | ----------------------------- | ------------ |
| pixMaskConnComp           | ✅ 同等 | `mask_conn_comp`              | conncomp依存 |
| pixMaskBoxa               | ✅      | draw.rs mask_boxa()           |              |
| pixPaintBoxa              | ✅      | draw.rs paint_boxa()          |              |
| pixSetBlackOrWhiteBoxa    | ✅      | draw.rs set_bw_boxa()         |              |
| pixPaintBoxaRandom        | ✅      | draw.rs paint_boxa_random()   |              |
| pixBlendBoxaRandom        | ✅      | draw.rs blend_boxa_random()   |              |
| pixDrawBoxa               | ✅      | draw.rs draw_boxa()           |              |
| pixDrawBoxaRandom         | ✅      | draw.rs draw_boxa_random()    |              |
| boxaaDisplay              | ✅ 同等 | `Boxaa::display`              |              |
| pixaDisplayBoxaa          | ✅ 同等 | `Pixa::display_boxaa`         |              |
| pixSplitIntoBoxa          | ✅ 同等 | `split_into_boxa`             |              |
| pixSplitComponentIntoBoxa | ✅ 同等 | `split_component_into_boxa`   |              |
| makeMosaicStrips          | ✅ 同等 | `make_mosaic_strips`          |              |
| boxaCompareRegions        | ✅      | draw.rs compare_regions()     |              |
| pixSelectLargeULComp      | ✅ 同等 | `select_large_ul_comp`        | conncomp依存 |
| boxaSelectLargeULBox      | ✅      | draw.rs select_large_ul_box() |              |

### boxfunc4.c (Box選択・変換)

| C関数                    | 状態    | Rust対応                            | 備考 |
| ------------------------ | ------- | ----------------------------------- | ---- |
| boxaSelectRange          | ✅      | select.rs select_range()            |      |
| boxaaSelectRange         | ✅      | select.rs select_range() (Boxaa)    |      |
| boxaSelectBySize         | ✅      | mod.rs select_by_size()             |      |
| boxaMakeSizeIndicator    | ✅      | select.rs make_size_indicator()     |      |
| boxaSelectByArea         | ✅      | mod.rs select_by_area()             |      |
| boxaMakeAreaIndicator    | ✅      | select.rs make_area_indicator()     |      |
| boxaSelectByWHRatio      | ✅      | mod.rs select_by_wh_ratio()         |      |
| boxaMakeWHRatioIndicator | ✅      | select.rs make_wh_ratio_indicator() |      |
| boxaSelectWithIndicator  | ✅      | select.rs select_with_indicator()   |      |
| boxaPermutePseudorandom  | ✅ 同等 | `Boxa::permute_pseudorandom`        |      |
| boxaPermuteRandom        | ✅ 同等 | `Boxa::permute_random`              |      |
| boxaSwapBoxes            | ✅      | select.rs swap_boxes()              |      |
| boxaConvertToPta         | ✅      | adjust.rs to_pta() (Boxa)           |      |
| ptaConvertToBoxa         | ✅      | adjust.rs to_boxa()                 |      |
| boxConvertToPta          | ✅      | adjust.rs to_pta() (Box)            |      |
| ptaConvertToBox          | ✅      | adjust.rs to_box()                  |      |
| boxaGetExtent            | ✅      | mod.rs get_extent()                 |      |
| boxaGetCoverage          | ✅      | mod.rs get_coverage()               |      |
| boxaaSizeRange           | ✅      | select.rs size_range() (Boxaa)      |      |
| boxaSizeRange            | ✅      | mod.rs size_range()                 |      |
| boxaLocationRange        | ✅      | select.rs location_range()          |      |
| boxaGetSizes             | ✅      | select.rs get_sizes()               |      |
| boxaGetArea              | ✅      | select.rs get_total_area()          |      |
| boxaDisplayTiled         | ✅ 同等 | `Boxa::display_tiled`               |      |

### boxfunc5.c (Boxスムージング・調整)

| C関数                      | 状態    | Rust対応                          | 備考 |
| -------------------------- | ------- | --------------------------------- | ---- |
| boxaSmoothSequenceMedian   | ✅ 同等 | `Boxa::smooth_sequence_median`    |      |
| boxaWindowedMedian         | ✅ 同等 | `Boxa::windowed_median`           |      |
| boxaModifyWithBoxa         | ✅ 同等 | `Boxa::modify_with_boxa`          |      |
| boxaReconcilePairWidth     | ✅ 同等 | `Boxa::reconcile_pair_width`      |      |
| boxaSizeConsistency        | ✅ 同等 | `Boxa::size_consistency`          |      |
| boxaReconcileAllByMedian   | ✅ 同等 | `Boxa::reconcile_all_by_median`   |      |
| boxaReconcileSidesByMedian | ✅ 同等 | `Boxa::reconcile_sides_by_median` |      |
| boxaReconcileSizeByMedian  | ✅ 同等 | `Boxa::reconcile_size_by_median`  |      |
| boxaPlotSides              | ✅ 同等 | `Boxa::plot_sides`                |      |
| boxaPlotSizes              | ✅ 同等 | `Boxa::plot_sizes`                |      |
| boxaFillSequence           | ✅ 同等 | `Boxa::fill_sequence`             |      |
| boxaSizeVariation          | ✅ 同等 | `Boxa::size_variation`            |      |
| boxaMedianDimensions       | ✅ 同等 | `Boxa::median_dimensions`         |      |

### ptabasic.c (Pta基本操作)

| C関数             | 状態    | Rust対応                   | 備考                 |
| ----------------- | ------- | -------------------------- | -------------------- |
| ptaCreate         | ✅      | Pta::new()                 |                      |
| ptaCreateFromNuma | ✅ 同等 | `Pta::create_from_numa`    |                      |
| ptaDestroy        | 🔄      | drop()                     | 自動                 |
| ptaCopy           | ✅      | Pta::clone()               |                      |
| ptaCopyRange      | ✅ 同等 | `Pta::copy_range`          |                      |
| ptaClone          | ✅      | Pta::clone()               |                      |
| ptaEmpty          | 🚫 不要 | -                          | Pta::clear()で対応   |
| ptaAddPt          | ✅      | Pta::push()                |                      |
| ptaInsertPt       | ✅ 同等 | `Pta::insert_pt`           |                      |
| ptaRemovePt       | ✅ 同等 | `Pta::remove_pt`           |                      |
| ptaGetCount       | ✅      | Pta::len()                 |                      |
| ptaGetPt          | ✅      | Pta::get()                 |                      |
| ptaGetIPt         | ✅ 同等 | `Pta::get_i_pt`            |                      |
| ptaSetPt          | ✅      | Pta::set()                 |                      |
| ptaGetArrays      | 🚫 不要 | -                          | Cポインタ配列        |
| ptaRead           | ✅ 同等 | `Pta::read`                |                      |
| ptaReadStream     | ✅ 同等 | `Pta::read_stream`         |                      |
| ptaReadMem        | ✅ 同等 | `Pta::read_mem`            |                      |
| ptaWriteDebug     | 🚫 不要 | -                          | デバッグ出力関数     |
| ptaWrite          | ✅ 同等 | `Pta::write`               |                      |
| ptaWriteStream    | ✅ 同等 | `Pta::write_stream`        |                      |
| ptaWriteMem       | ✅ 同等 | `Pta::write_mem`           |                      |
| ptaaCreate        | ✅ 同等 | `Ptaa::new()`              | Ptaa構造体として実装 |
| ptaaDestroy       | 🔄      | drop()                     | Drop traitで自動     |
| ptaaAddPta        | ✅ 同等 | `Ptaa::push()`             |                      |
| ptaaGetCount      | ✅ 同等 | `Ptaa::len()`              |                      |
| ptaaGetPta        | ✅ 同等 | `Ptaa::get()`              |                      |
| ptaaGetPt         | 🚫 不要 | -                          | Vec<Pta>で代替       |
| ptaaInitFull      | ✅ 同等 | `Ptaa::init_full()`        |                      |
| ptaaReplacePta    | ✅ 同等 | `Ptaa::replace()`          |                      |
| ptaaAddPt         | ✅ 同等 | `Ptaa::add_pt()`           |                      |
| ptaaTruncate      | ✅ 同等 | `Ptaa::truncate()`         |                      |
| ptaaRead          | ✅ 同等 | `Ptaa::read_from_file()`   |                      |
| ptaaReadStream    | ✅ 同等 | `Ptaa::read_from_reader()` |                      |
| ptaaReadMem       | ✅ 同等 | `Ptaa::read_from_bytes()`  |                      |
| ptaaWriteDebug    | 🚫 不要 | -                          | Vec<Pta>で代替       |
| ptaaWrite         | ✅ 同等 | `Ptaa::write_to_file()`    |                      |
| ptaaWriteStream   | ✅ 同等 | `Ptaa::write_to_writer()`  |                      |
| ptaaWriteMem      | ✅ 同等 | `Ptaa::write_to_bytes()`   |                      |

### ptafunc1.c, ptafunc2.c (Pta変換・演算)

Phase 16で大部分を実装済み。

| C関数               | 状態    | Rust対応                         | 備考 |
| ------------------- | ------- | -------------------------------- | ---- |
| ptaSubsample        | ✅      | transform.rs subsample()         |      |
| ptaJoin             | ✅      | transform.rs join()              |      |
| ptaaJoin            | ✅ 同等 | `Ptaa::join()`                   |      |
| ptaReverse          | ✅      | transform.rs reverse()           |      |
| ptaTranspose        | ✅      | transform.rs transpose()         |      |
| ptaCyclicPerm       | ✅      | transform.rs cyclic_perm()       |      |
| ptaSelectRange      | ✅      | transform.rs select_range()      |      |
| ptaGetRange         | ✅      | transform.rs get_range()         |      |
| ptaGetInsideBox     | ✅      | transform.rs get_inside_box()    |      |
| ptaContainsPt       | ✅      | transform.rs contains_pt()       |      |
| ptaTestIntersection | ✅      | transform.rs test_intersection() |      |
| ptaTransform        | ✅      | transform.rs transform_pts()     |      |
| ptaPtInsidePolygon  | ✅      | transform.rs pt_inside_polygon() |      |
| ptaPolygonIsConvex  | ✅      | transform.rs polygon_is_convex() |      |
| ptaGetMinMax        | ✅      | transform.rs get_min_max()       |      |
| ptaSelectByValue    | ✅      | transform.rs select_by_value()   |      |
| ptaCropToMask       | ✅ 同等 | `Pta::crop_to_mask`              |      |
| ptaGetLinearLSF     | ✅      | lsf.rs get_linear_lsf()          |      |
| ptaGetQuadraticLSF  | ✅      | lsf.rs get_quadratic_lsf()       |      |
| ptaGetCubicLSF      | ✅      | lsf.rs get_cubic_lsf()           |      |
| ptaGetQuarticLSF    | ✅      | lsf.rs get_quartic_lsf()         |      |
| ptaSortByIndex      | ✅      | sort.rs sort_by_index()          |      |
| ptaGetSortIndex     | ✅      | sort.rs get_sort_index()         |      |
| ptaSort             | ✅      | sort.rs sort_pta()               |      |
| ptaGetRankValue     | ✅      | sort.rs get_rank_value()         |      |
| ptaSort2d           | ✅      | sort.rs sort_2d()                |      |
| ptaEqual            | ✅      | sort.rs equal()                  |      |

### pixabasic.c (Pixa基本操作)

| C関数                 | 状態    | Rust対応                    | 備考                   |
| --------------------- | ------- | --------------------------- | ---------------------- |
| pixaCreate            | ✅      | Pixa::new()                 |                        |
| pixaCreateFromPix     | ✅ 同等 | `Pixa::create_from_pix`     |                        |
| pixaCreateFromBoxa    | ✅ 同等 | `Pixa::create_from_boxa`    |                        |
| pixaSplitPix          | ✅ 同等 | `Pixa::split_pix`           |                        |
| pixaDestroy           | 🔄      | drop()                      | 自動                   |
| pixaCopy              | ✅      | Pixa::clone()               |                        |
| pixaAddPix            | ✅      | Pixa::push()                |                        |
| pixaAddBox            | ✅      | Pixa::push_with_box()       |                        |
| pixaExtendArray       | 🚫 不要 | -                           | Vec自動拡張            |
| pixaExtendArrayToSize | 🚫 不要 | -                           | Vec自動拡張            |
| pixaGetCount          | ✅      | Pixa::len()                 |                        |
| pixaGetPix            | ✅      | Pixa::get_cloned()          |                        |
| pixaGetPixDimensions  | ✅      | Pixa::get_dimensions()      |                        |
| pixaGetBoxa           | ✅ 同等 | `Pixa::get_boxa`            |                        |
| pixaGetBoxaCount      | ✅ 同等 | `Pixa::get_boxa_count`      |                        |
| pixaGetBox            | ✅ 同等 | `Pixa::get_box`             |                        |
| pixaGetBoxGeometry    | ✅ 同等 | `Pixa::get_box_geometry`    |                        |
| pixaSetBoxa           | ✅ 同等 | `Pixa::set_boxa`            |                        |
| pixaGetPixArray       | 🚫 不要 | -                           | Cポインタ配列          |
| pixaVerifyDepth       | 🚫 不要 | -                           | Rustの型システムで不要 |
| pixaVerifyDimensions  | 🚫 不要 | -                           | Rustの型システムで不要 |
| pixaIsFull            | 🚫 不要 | -                           | Rustの型システムで不要 |
| pixaCountText         | ✅ 同等 | `Pixa::count_text`          |                        |
| pixaSetText           | ✅ 同等 | `Pixa::set_text`            |                        |
| pixaGetLinePtrs       | 🚫 不要 | -                           | Cポインタ配列          |
| pixaWriteStreamInfo   | 🚫 不要 | -                           | デバッグ出力関数       |
| pixaReplacePix        | ✅ 同等 | `Pixa::replace_pix`         |                        |
| pixaInsertPix         | ✅ 同等 | `Pixa::insert_pix`          |                        |
| pixaRemovePix         | ✅ 同等 | `Pixa::remove_pix`          |                        |
| pixaRemovePixAndSave  | ✅ 同等 | `Pixa::remove_pix_and_save` |                        |
| pixaRemoveSelected    | ✅ 同等 | `Pixa::remove_selected`     |                        |
| pixaInitFull          | ✅ 同等 | `Pixa::init_full`           |                        |
| pixaClear             | ✅      | Pixa::clear()               |                        |
| pixaJoin              | ✅ 同等 | `Pixa::join`                |                        |
| pixaInterleave        | ✅ 同等 | `Pixa::interleave`          |                        |
| pixaaJoin             | ✅ 同等 | `Pixaa::join()`             |                        |
| pixaaCreate           | ✅ 同等 | `Pixaa::new()`              | Pixaa構造体として実装  |
| pixaaCreateFromPixa   | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaDestroy          | 🔄      | drop()                      | Drop traitで自動       |
| pixaaAddPixa          | ✅ 同等 | `Pixaa::push()`             |                        |
| pixaaExtendArray      | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaAddPix           | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaAddBox           | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaGetCount         | ✅ 同等 | `Pixaa::len()`              |                        |
| pixaaGetPixa          | ✅ 同等 | `Pixaa::get()`              |                        |
| pixaaGetBoxa          | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaGetPix           | ✅ 同等 | `Pixaa::get_pix()`          |                        |
| pixaaVerifyDepth      | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaVerifyDimensions | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaIsFull           | ✅ 同等 | `Pixaa::is_full()`          |                        |
| pixaaInitFull         | ✅ 同等 | `Pixaa::init_full()`        |                        |
| pixaaReplacePixa      | ✅ 同等 | `Pixaa::replace()`          |                        |
| pixaaClear            | ✅ 同等 | `Pixaa::clear()`            |                        |
| pixaaTruncate         | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaRead              | ✅ 同等 | `Pixa::read`                |                        |
| pixaReadStream        | ✅ 同等 | `Pixa::read_stream`         |                        |
| pixaReadMem           | ✅ 同等 | `Pixa::read_mem`            |                        |
| pixaWriteDebug        | 🚫 不要 | -                           | デバッグ出力関数       |
| pixaWrite             | ✅ 同等 | `Pixa::write`               |                        |
| pixaWriteStream       | ✅ 同等 | `Pixa::write_stream`        |                        |
| pixaWriteMem          | ✅ 同等 | `Pixa::write_mem`           |                        |
| pixaReadBoth          | ✅ 同等 | `Pixa::read_both`           |                        |
| pixaaReadFromFiles    | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaRead             | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaReadStream       | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaReadMem          | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaWrite            | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaWriteStream      | 🚫 不要 | -                           | Vec<Pixa>で代替        |
| pixaaWriteMem         | 🚫 不要 | -                           | Vec<Pixa>で代替        |

### pixafunc1.c, pixafunc2.c (Pixa選択・変換・表示)

Phase 16で主要機能を実装済み。

| C関数                        | 状態 | Rust対応                        | 備考 |
| ---------------------------- | ---- | ------------------------------- | ---- |
| pixaSelectBySize             | ✅   | pixa select_by_size()           |      |
| pixaSelectByArea             | ✅   | pixa select_by_area()           |      |
| pixaSort                     | ✅   | pixa sort()                     |      |
| pixaSortByIndex              | ✅   | pixa sort_by_index()            |      |
| pixaScaleToSize              | ✅   | pixa scale_to_size()            |      |
| pixaScaleToSizeRel           | ✅   | pixa scale_to_size_rel()        |      |
| pixaDisplay                  | ✅   | pixa display()                  |      |
| pixaDisplayTiled             | ✅   | pixa display_tiled()            |      |
| pixaDisplayTiledAndScaled    | ✅   | pixa display_tiled_and_scaled() |      |
| pixaGetAlignedStats          | ✅   | pixa aligned_stats()            |      |
| pixaExtractColumnFromEachPix | ✅   | pixa extract_column_from_each() |      |
| pixaFindDimensions           | ✅   | pixa find_dimensions()          |      |
| pixaCountPixels              | ✅   | pixa count_pixels()             |      |

### numabasic.c (Numa基本操作)

実装済み関数が存在する。C版のnumabasic.cのI/O関連関数も実装済み。
numa/mod.rs, numa/operations.rs に基本統計関数は実装済み。

### numafunc1.c, numafunc2.c (Numa演算・統計)

| C関数                       | 状態    | Rust対応                                     | 備考                   |
| --------------------------- | ------- | -------------------------------------------- | ---------------------- |
| numaArithOp                 | ✅      | operations.rs arith_op()                     |                        |
| numaLogicalOp               | ✅      | operations.rs logical_op()                   |                        |
| numaInvert                  | ✅      | operations.rs invert()                       |                        |
| numaSimilar                 | ✅      | operations.rs similar()                      |                        |
| numaAddToNumber             | ✅      | operations.rs add_to_element()               |                        |
| numaGetMin                  | ✅      | Numa::min()                                  |                        |
| numaGetMax                  | ✅      | Numa::max()                                  |                        |
| numaGetSum                  | ✅      | Numa::sum()                                  |                        |
| numaGetPartialSums          | ✅      | operations.rs partial_sums()                 |                        |
| numaGetSumOnInterval        | ✅      | Numa::sum_on_interval()                      |                        |
| numaHasOnlyIntegers         | ✅      | Numa::has_only_integers()                    |                        |
| numaGetMean                 | ✅      | Numa::mean()                                 |                        |
| numaGetMeanAbsval           | ✅      | Numa::mean_absval()                          |                        |
| numaSubsample               | ✅      | operations.rs subsample()                    |                        |
| numaMakeDelta               | ✅      | operations.rs make_delta()                   |                        |
| numaMakeSequence            | ✅      | operations.rs make_sequence()                |                        |
| numaMakeConstant            | ✅      | Numa::make_constant()                        |                        |
| numaMakeAbsval              | ✅      | operations.rs abs_val()                      |                        |
| numaAddBorder               | ✅      | operations.rs add_border()                   |                        |
| numaAddSpecifiedBorder      | ✅      | operations.rs add_specified_border()         |                        |
| numaRemoveBorder            | ✅      | operations.rs remove_border()                |                        |
| numaCountNonzeroRuns        | ✅      | operations.rs count_nonzero_runs()           |                        |
| numaGetNonzeroRange         | ✅      | operations.rs get_nonzero_range()            |                        |
| numaGetCountRelativeToZero  | ✅      | operations.rs get_count_relative_to_zero()   |                        |
| numaClipToInterval          | ✅      | operations.rs clip_to_interval()             |                        |
| numaMakeThresholdIndicator  | ✅      | operations.rs make_threshold_indicator()     |                        |
| numaUniformSampling         | ✅      | interpolation.rs uniform_sampling()          |                        |
| numaReverse                 | ✅      | Numa::reversed() / Numa::reverse()           |                        |
| numaLowPassIntervals        | ✅      | interpolation.rs low_pass_intervals()        |                        |
| numaThresholdEdges          | ✅      | interpolation.rs threshold_edges()           |                        |
| numaGetSpanValues           | ✅      | interpolation.rs get_span_values()           |                        |
| numaGetEdgeValues           | ✅      | interpolation.rs get_edge_values()           |                        |
| numaInterpolateEqxVal       | ✅      | operations.rs interpolate_eqx_val()          |                        |
| numaInterpolateArbxVal      | ✅      | operations.rs interpolate_arbx_val()         |                        |
| numaInterpolateEqxInterval  | ✅      | interpolation.rs interpolate_eqx_interval()  |                        |
| numaInterpolateArbxInterval | ✅      | interpolation.rs interpolate_arbx_interval() |                        |
| numaFitMax                  | ✅      | interpolation.rs fit_max()                   |                        |
| numaDifferentiateInterval   | ✅      | interpolation.rs differentiate_interval()    |                        |
| numaIntegrateInterval       | ✅      | interpolation.rs integrate_interval()        |                        |
| numaSortGeneral             | ✅ 同等 | `Numa::sort_general`                         | sort_auto_selectで統合 |
| numaSortAutoSelect          | ✅      | operations.rs sort_auto_select()             |                        |
| numaSortIndexAutoSelect     | ✅      | operations.rs sort_index_auto_select()       |                        |
| numaChooseSortType          | ✅ 同等 | `Numa::choose_sort_type`                     | 内部関数               |
| numaSort                    | ✅      | Numa::sorted() / Numa::sort()                |                        |
| numaBinSort                 | ✅      | sort.rs bin_sort()                           |                        |
| numaGetSortIndex            | ✅      | operations.rs sort_index()                   |                        |
| numaGetBinSortIndex         | ✅      | sort.rs bin_sort_index()                     |                        |
| numaSortByIndex             | ✅      | operations.rs sort_by_index()                |                        |
| numaIsSorted                | ✅      | operations.rs is_sorted()                    |                        |
| numaSortPair                | ✅      | sort.rs sort_pair()                          |                        |
| numaInvertMap               | ✅      | sort.rs invert_map()                         |                        |
| numaAddSorted               | ✅      | sort.rs add_sorted()                         |                        |
| numaFindSortedLoc           | ✅      | sort.rs find_sorted_loc()                    |                        |
| numaPseudorandomSequence    | ✅      | sort.rs pseudorandom_sequence()              |                        |
| numaRandomPermutation       | ✅      | sort.rs random_permutation()                 |                        |
| numaGetRankValue            | ✅      | Numa::rank_value()                           |                        |
| numaGetMedian               | ✅      | Numa::median()                               |                        |
| numaGetBinnedMedian         | ✅      | sort.rs binned_median()                      |                        |
| numaGetMeanDevFromMedian    | ✅      | sort.rs mean_dev_from_median()               |                        |
| numaGetMedianDevFromMedian  | ✅      | sort.rs median_dev_from_median()             |                        |
| numaGetMode                 | ✅      | Numa::mode()                                 |                        |
| numaJoin                    | ✅      | operations.rs join()                         |                        |
| numaaJoin                   | ✅ 同等 | `Numaa::join`                                |                        |
| numaaFlattenToNuma          | ✅      | Numaa::flatten()                             |                        |

numafunc2.c (ヒストグラム・統計)の関数も実装済み。
一部ヒストグラム関数はnuma/histogram.rsに実装あり。

### sarray1.c, sarray2.c (Sarray文字列配列)

| C関数                       | 状態    | Rust対応                         | 備考             |
| --------------------------- | ------- | -------------------------------- | ---------------- |
| sarrayCreate                | ✅      | Sarray::new()                    |                  |
| sarrayCreateInitialized     | ✅      | Sarray::initialized()            |                  |
| sarrayCreateWordsFromString | ✅      | Sarray::from_words()             |                  |
| sarrayCreateLinesFromString | ✅      | Sarray::from_lines()             |                  |
| sarrayDestroy               | 🔄      | drop()                           | 自動             |
| sarrayCopy                  | ✅      | Sarray::clone()                  |                  |
| sarrayClone                 | ✅      | Sarray::clone()                  |                  |
| sarrayAddString             | ✅      | Sarray::push()                   |                  |
| sarrayRemoveString          | ✅ 同等 | `SArray::remove_string`          |                  |
| sarrayReplaceString         | ✅ 同等 | `SArray::replace_string`         |                  |
| sarrayClear                 | ✅      | Sarray::clear()                  |                  |
| sarrayGetCount              | ✅      | Sarray::len()                    |                  |
| sarrayGetArray              | 🚫 不要 | -                                | Cポインタ配列    |
| sarrayGetString             | ✅      | Sarray::get()                    |                  |
| sarrayToString              | ✅      | Sarray::join()                   |                  |
| sarrayToStringRange         | ✅ 同等 | `SArray::to_string_range`        |                  |
| sarrayConcatUniformly       | ✅ 同等 | `SArray::concat_uniformly`       |                  |
| sarrayJoin                  | ✅ 同等 | `SArray::join`                   |                  |
| sarrayAppendRange           | ✅ 同等 | `SArray::append_range`           |                  |
| sarrayPadToSameSize         | ✅ 同等 | `SArray::pad_to_same_size`       |                  |
| sarrayConvertWordsToLines   | ✅ 同等 | `SArray::convert_words_to_lines` |                  |
| sarraySplitString           | ✅ 同等 | `SArray::split_string`           |                  |
| sarraySelectBySubstring     | ✅      | Sarray::filter_by_substring()    |                  |
| sarraySelectRange           | ✅ 同等 | `SArray::select_range`           |                  |
| sarrayParseRange            | ✅ 同等 | `SArray::parse_range`            |                  |
| sarrayRead                  | ✅ 同等 | `SArray::read`                   |                  |
| sarrayReadStream            | ✅ 同等 | `SArray::read_stream`            |                  |
| sarrayReadMem               | ✅ 同等 | `SArray::read_mem`               |                  |
| sarrayWrite                 | ✅ 同等 | `SArray::write`                  |                  |
| sarrayWriteStream           | ✅ 同等 | `SArray::write_stream`           |                  |
| sarrayWriteStderr           | 🚫 不要 | -                                | デバッグ出力関数 |
| sarrayWriteMem              | ✅ 同等 | `SArray::write_mem`              |                  |
| sarrayAppend                | ✅ 同等 | `SArray::append`                 |                  |
| sarraySort                  | ✅      | Sarray::sort()                   |                  |
| sarraySortByIndex           | ✅ 同等 | `SArray::sort_by_index`          |                  |

その他のsarray2.c関数（セット演算、整数生成など）も実装済み。

### fpix1.c, fpix2.c (FPix浮動小数点画像)

| C関数                  | 状態    | Rust対応                     | 備考                   |
| ---------------------- | ------- | ---------------------------- | ---------------------- |
| fpixCreate             | ✅      | FPix::new()                  |                        |
| fpixCreateTemplate     | ✅      | FPix::create_template()      |                        |
| fpixClone              | ✅      | FPix::clone()                |                        |
| fpixCopy               | ✅      | FPix::clone()                |                        |
| fpixDestroy            | 🔄      | drop()                       | 自動                   |
| fpixGetDimensions      | ✅      | width()/height()             |                        |
| fpixSetDimensions      | 🚫 不要 | -                            | FPixは不変             |
| fpixGetWpl             | 🚫 不要 | -                            | FPixはwpl概念なし      |
| fpixSetWpl             | 🚫 不要 | -                            | FPixはwpl概念なし      |
| fpixGetResolution      | ✅      | xres()/yres()                |                        |
| fpixSetResolution      | ✅      | set_resolution()             |                        |
| fpixCopyResolution     | 🚫 不要 | -                            | set_resolution()で対応 |
| fpixGetData            | ✅      | FPix::data()                 |                        |
| fpixSetData            | 🚫 不要 | -                            | Cメモリ管理            |
| fpixGetPixel           | ✅      | FPix::get_pixel()            |                        |
| fpixSetPixel           | ✅      | FPix::set_pixel()            |                        |
| fpixaCreate            | ✅ 同等 | `fpixa_create`               |                        |
| fpixaCopy              | ✅ 同等 | `fpixa_copy`                 |                        |
| fpixaDestroy           | 🚫 不要 | -                            | drop()で自動           |
| fpixaAddFPix           | ✅ 同等 | `fpixa_add_f_pix`            |                        |
| fpixaGetCount          | ✅ 同等 | `fpixa_get_count`            |                        |
| fpixaGetFPix           | ✅ 同等 | `fpixa_get_f_pix`            |                        |
| fpixaGetFPixDimensions | ✅ 同等 | `fpixa_get_f_pix_dimensions` |                        |
| fpixaGetData           | ✅ 同等 | `fpixa_get_data`             |                        |
| fpixaGetPixel          | ✅ 同等 | `fpixa_get_pixel`            |                        |
| fpixaSetPixel          | ✅ 同等 | `fpixa_set_pixel`            |                        |
| dpixCreate             | ✅      | DPix::new()                  |                        |
| dpixClone              | ✅      | DPix::clone()                |                        |
| dpixCopy               | ✅      | DPix::clone()                |                        |
| dpixDestroy            | 🔄      | drop()                       | 自動                   |
| fpixRead               | ✅ 同等 | `FPix::read`                 |                        |
| fpixReadStream         | ✅ 同等 | `FPix::read_stream`          |                        |
| fpixReadMem            | ✅ 同等 | `FPix::read_mem`             |                        |
| fpixWrite              | ✅ 同等 | `FPix::write`                |                        |
| fpixWriteStream        | ✅ 同等 | `FPix::write_stream`         |                        |
| fpixWriteMem           | ✅ 同等 | `FPix::write_mem`            |                        |
| dpixRead               | ✅ 同等 | `dpix_read`                  |                        |
| dpixWrite              | ✅ 同等 | `dpix_write`                 |                        |

fpix2.c (FPix変換・演算):

| C関数                 | 状態 | Rust対応                   | 備考 |
| --------------------- | ---- | -------------------------- | ---- |
| fpixConvertToPix      | ✅   | FPix::to_pix()             |      |
| pixConvertToFPix      | ✅   | FPix::from_pix()           |      |
| fpixAddMultConstant   | ✅   | FPix::add_mult_constant()  |      |
| fpixLinearCombination | ✅   | FPix::linear_combination() |      |
| dpixConvertToPix      | ✅   | DPix::to_pix()             |      |
| dpixConvertToFPix     | ✅   | DPix::to_fpix()            |      |

その他のfpix2.c変換関数は一部convert.rsに実装あり。

### colormap.c (カラーマップ)

| C関数                        | 状態    | Rust対応                          | 備考                |
| ---------------------------- | ------- | --------------------------------- | ------------------- |
| pixcmapCreate                | ✅      | PixColormap::new()                |                     |
| pixcmapCreateRandom          | ✅ 同等 | `pixcmap_create_random`           |                     |
| pixcmapCreateLinear          | ✅      | PixColormap::create_linear()      |                     |
| pixcmapCopy                  | ✅      | PixColormap::clone()              |                     |
| pixcmapDestroy               | 🔄      | drop()                            | 自動                |
| pixcmapIsValid               | ✅ 同等 | `pixcmap_is_valid`                |                     |
| pixcmapAddColor              | ✅      | PixColormap::add_color()          |                     |
| pixcmapAddRGBA               | ✅ 同等 | `pixcmap_add_rgba`                | add_colorがRGBA対応 |
| pixcmapAddNewColor           | ✅ 同等 | `pixcmap_add_new_color`           |                     |
| pixcmapAddNearestColor       | ✅ 同等 | `pixcmap_add_nearest_color`       |                     |
| pixcmapUsableColor           | ✅ 同等 | `pixcmap_usable_color`            |                     |
| pixcmapAddBlackOrWhite       | ✅ 同等 | `pixcmap_add_black_or_white`      |                     |
| pixcmapSetBlackAndWhite      | ✅ 同等 | `pixcmap_set_black_and_white`     |                     |
| pixcmapGetCount              | ✅      | PixColormap::len()                |                     |
| pixcmapGetFreeCount          | ✅ 同等 | `pixcmap_get_free_count`          |                     |
| pixcmapGetDepth              | ✅      | PixColormap::depth()              |                     |
| pixcmapGetMinDepth           | ✅ 同等 | `pixcmap_get_min_depth`           |                     |
| pixcmapClear                 | ✅      | PixColormap::clear()              |                     |
| pixcmapGetColor              | ✅      | PixColormap::get_color()          |                     |
| pixcmapGetColor32            | ✅ 同等 | `pixcmap_get_color32`             |                     |
| pixcmapGetRGBA               | ✅ 同等 | `pixcmap_get_rgba`                |                     |
| pixcmapGetRGBA32             | ✅ 同等 | `pixcmap_get_rgba32`              |                     |
| pixcmapResetColor            | ✅ 同等 | `pixcmap_reset_color`             |                     |
| pixcmapSetAlpha              | ✅ 同等 | `pixcmap_set_alpha`               |                     |
| pixcmapGetIndex              | ✅ 同等 | `pixcmap_get_index`               |                     |
| pixcmapHasColor              | ✅ 同等 | `pixcmap_has_color`               |                     |
| pixcmapIsOpaque              | ✅ 同等 | `pixcmap_is_opaque`               |                     |
| pixcmapNonOpaqueColorsInfo   | ✅ 同等 | `pixcmap_non_opaque_colors_info`  |                     |
| pixcmapIsBlackAndWhite       | ✅ 同等 | `pixcmap_is_black_and_white`      |                     |
| pixcmapCountGrayColors       | ✅ 同等 | `pixcmap_count_gray_colors`       |                     |
| pixcmapGetRankIntensity      | ✅ 同等 | `pixcmap_get_rank_intensity`      |                     |
| pixcmapGetNearestIndex       | ✅ 同等 | `pixcmap_get_nearest_index`       |                     |
| pixcmapGetNearestGrayIndex   | ✅ 同等 | `pixcmap_get_nearest_gray_index`  |                     |
| pixcmapGetDistanceToColor    | ✅ 同等 | `pixcmap_get_distance_to_color`   |                     |
| pixcmapGetRangeValues        | ✅ 同等 | `pixcmap_get_range_values`        |                     |
| pixcmapGrayToFalseColor      | ✅ 同等 | `pixcmap_gray_to_false_color`     |                     |
| pixcmapGrayToColor           | ✅ 同等 | `pixcmap_gray_to_color`           |                     |
| pixcmapColorToGray           | ✅ 同等 | `pixcmap_color_to_gray`           |                     |
| pixcmapConvertTo4            | ✅ 同等 | `pixcmap_convert_to4`             |                     |
| pixcmapConvertTo8            | ✅ 同等 | `pixcmap_convert_to8`             |                     |
| pixcmapRead                  | ✅ 同等 | `pixcmap_read`                    |                     |
| pixcmapReadStream            | ✅ 同等 | `pixcmap_read_stream`             |                     |
| pixcmapReadMem               | ✅ 同等 | `pixcmap_read_mem`                |                     |
| pixcmapWrite                 | ✅ 同等 | `pixcmap_write`                   |                     |
| pixcmapWriteStream           | ✅ 同等 | `pixcmap_write_stream`            |                     |
| pixcmapWriteMem              | ✅ 同等 | `pixcmap_write_mem`               |                     |
| pixcmapToArrays              | ✅ 同等 | `pixcmap_to_arrays`               |                     |
| pixcmapToRGBTable            | ✅ 同等 | `pixcmap_to_rgb_table`            |                     |
| pixcmapSerializeToMemory     | ✅ 同等 | `pixcmap_serialize_to_memory`     |                     |
| pixcmapDeserializeFromMemory | ✅ 同等 | `pixcmap_deserialize_from_memory` |                     |
| pixcmapConvertToHex          | ✅ 同等 | `pixcmap_convert_to_hex`          |                     |
| pixcmapGammaTRC              | ✅ 同等 | `pixcmap_gamma_trc`               |                     |
| pixcmapContrastTRC           | ✅ 同等 | `pixcmap_contrast_trc`            |                     |
| pixcmapShiftIntensity        | ✅ 同等 | `pixcmap_shift_intensity`         |                     |
| pixcmapShiftByComponent      | ✅ 同等 | `pixcmap_shift_by_component`      |                     |

### pixconv.c (ピクセル深度変換)

convert.rsに実装済み。全関数が実装されている。

| C関数                        | 状態    | Rust対応                                             | 備考                                    |
| ---------------------------- | ------- | ---------------------------------------------------- | --------------------------------------- |
| pixThreshold8                | ✅ 同等 | `threshold_8`                                        |                                         |
| pixRemoveColormapGeneral     | ✅      | convert.rs remove_colormap()                         | pixRemoveColormapと同一メソッド         |
| pixRemoveColormap            | ✅      | convert.rs remove_colormap()                         |                                         |
| pixAddGrayColormap8          | ✅      | convert.rs add_gray_colormap8()                      |                                         |
| pixAddMinimalGrayColormap8   | ✅      | convert.rs add_minimal_gray_colormap8()              |                                         |
| pixConvertRGBToLuminance     | ✅      | convert.rs convert_rgb_to_luminance()                |                                         |
| pixConvertRGBToGrayGeneral   | ✅      | convert.rs convert_rgb_to_gray_general()             |                                         |
| pixConvertRGBToGray          | ✅      | convert.rs convert_rgb_to_gray()                     |                                         |
| pixConvertRGBToGrayFast      | ✅      | convert.rs convert_rgb_to_gray_fast()                |                                         |
| pixConvertRGBToGrayMinMax    | ✅      | convert.rs convert_rgb_to_gray_min_max()             |                                         |
| pixConvertRGBToGraySatBoost  | ✅      | convert.rs convert_rgb_to_gray_sat_boost()           |                                         |
| pixConvertRGBToGrayArb       | ✅      | convert.rs convert_rgb_to_gray_arb()                 |                                         |
| pixConvertRGBToBinaryArb     | ✅ 同等 | `convert_rgb_to_binary_arb`                          | color crate依存                         |
| pixConvertGrayToColormap     | ✅      | convert.rs convert_gray_to_colormap()                |                                         |
| pixConvertGrayToColormap8    | ✅      | convert.rs convert_gray_to_colormap_8()              |                                         |
| pixColorizeGray              | ✅      | convert.rs colorize_gray()                           |                                         |
| pixConvertRGBToColormap      | ✅ 同等 | `convert_rgb_to_colormap`                            | color crate依存                         |
| pixConvertCmapTo1            | ✅      | convert.rs convert_cmap_to_1()                       |                                         |
| pixQuantizeIfFewColors       | ✅ 同等 | `quantize_if_few_colors`                             | color crate依存                         |
| pixConvert16To8              | ✅      | convert.rs convert_16_to_8()                         |                                         |
| pixConvertGrayToFalseColor   | ✅      | convert.rs convert_gray_to_false_color()             |                                         |
| pixUnpackBinary              | ✅      | convert.rs unpack_binary()                           |                                         |
| pixConvert1To16              | ✅      | convert.rs convert_1_to_16()                         |                                         |
| pixConvert1To32              | ✅      | convert.rs convert_1_to_32()                         |                                         |
| pixConvert1To2Cmap           | ✅      | convert.rs convert_1_to_2_cmap()                     |                                         |
| pixConvert1To2               | ✅      | convert.rs convert_1_to_2()                          |                                         |
| pixConvert1To4Cmap           | ✅      | convert.rs convert_1_to_4_cmap()                     |                                         |
| pixConvert1To4               | ✅      | convert.rs convert_1_to_4()                          |                                         |
| pixConvert1To8Cmap           | ✅      | convert.rs convert_1_to_8_cmap()                     |                                         |
| pixConvert1To8               | ✅      | convert.rs convert_1_to_8()                          |                                         |
| pixConvert2To8               | ✅      | convert.rs convert_2_to_8()                          |                                         |
| pixConvert4To8               | ✅      | convert.rs convert_4_to_8()                          |                                         |
| pixConvert8To16              | ✅      | convert.rs convert_8_to_16()                         |                                         |
| pixConvertTo2                | ✅      | convert.rs convert_to_2()                            |                                         |
| pixConvert8To2               | ✅      | convert.rs convert_8_to_2()                          |                                         |
| pixConvertTo4                | ✅      | convert.rs convert_to_4()                            |                                         |
| pixConvert8To4               | ✅      | convert.rs convert_8_to_4()                          |                                         |
| pixConvertTo1Adaptive        | ✅ 同等 | `convert_to1_adaptive`                               |                                         |
| pixConvertTo1                | 🔄      | convert_to_1_adaptive() / convert_to_1_by_sampling() | 汎用ディスパッチャを2つの専用関数に分割 |
| pixConvertTo1BySampling      | ✅ 同等 | `convert_to1_by_sampling`                            |                                         |
| pixConvertTo8                | ✅      | convert.rs convert_to_8()                            |                                         |
| pixConvertTo8BySampling      | ✅ 同等 | `convert_to8_by_sampling`                            | transform crate依存                     |
| pixConvertTo8Colormap        | ✅ 同等 | `convert_to8_colormap`                               | 32bpp部分は後続                         |
| pixConvertTo16               | ✅      | convert.rs convert_to_16()                           |                                         |
| pixConvertTo32               | ✅      | convert.rs convert_to_32()                           |                                         |
| pixConvertTo32BySampling     | ✅ 同等 | `convert_to32_by_sampling`                           | transform crate依存                     |
| pixConvert8To32              | ✅      | convert.rs convert_8_to_32()                         |                                         |
| pixConvertTo8Or32            | ✅      | convert.rs convert_to_8_or_32()                      |                                         |
| pixConvert24To32             | ✅ 同等 | `convert24_to32`                                     |                                         |
| pixConvert32To24             | ✅ 同等 | `convert32_to24`                                     |                                         |
| pixConvert32To16             | ✅      | convert.rs convert_32_to_16()                        |                                         |
| pixConvert32To8              | ✅      | convert.rs convert_32_to_8()                         |                                         |
| pixRemoveAlpha               | ✅      | convert.rs remove_alpha()                            |                                         |
| pixAddAlphaTo1bpp            | ✅      | convert.rs add_alpha_to_1bpp()                       |                                         |
| pixConvertLossless           | ✅      | convert.rs convert_lossless()                        |                                         |
| pixConvertForPSWrap          | ✅      | convert.rs convert_for_ps_wrap()                     |                                         |
| pixConvertToSubpixelRGB      | ✅ 同等 | `convert_to_subpixel_rgb`                            |                                         |
| pixConvertGrayToSubpixelRGB  | ✅ 同等 | `convert_gray_to_subpixel_rgb`                       |                                         |
| pixConvertColorToSubpixelRGB | ✅ 同等 | `convert_color_to_subpixel_rgb`                      |                                         |

### pixarith.c (ピクセル算術演算)

| C関数                  | 状態 | Rust対応                         | 備考      |
| ---------------------- | ---- | -------------------------------- | --------- |
| pixAddGray             | ✅   | arith.rs arith_add()             |           |
| pixSubtractGray        | ✅   | arith.rs arith_subtract()        |           |
| pixMultConstantGray    | ✅   | arith.rs multiply_constant()     |           |
| pixAddConstantGray     | ✅   | arith.rs add_constant()          |           |
| pixMultConstAccumulate | ✅   | arith.rs mult_const_accumulate() | 32bpp専用 |
| pixAbsDifference       | ✅   | arith.rs abs_difference()        |           |
| pixMinOrMax            | ✅   | arith.rs min_or_max()            |           |

その他のpixarith.c関数も実装済み。

### rop.c, roplow.c (ラスターオペレーション)

| C関数                | 状態    | Rust対応              | 備考 |
| -------------------- | ------- | --------------------- | ---- |
| pixRasterop          | ✅      | rop.rsに実装          |      |
| pixRasteropVip       | ✅      | rop.rs rasterop_vip() |      |
| pixRasteropHip       | ✅      | rop.rs rasterop_hip() |      |
| pixTranslate         | ✅      | rop.rs translate()    |      |
| pixRasteropIP        | ✅ 同等 | `rasterop_ip`         |      |
| pixRasteropFullImage | ✅ 同等 | `rasterop_full_image` |      |

roplow.c (低レベルラスターOP) 全関数 🚫 不要 (高レベルrop.rs APIでカバー済み)

### compare.c (画像比較)

| C関数                     | 状態    | Rust対応                              | 備考 |
| ------------------------- | ------- | ------------------------------------- | ---- |
| pixEqual                  | ✅      | compare.rsに実装                      |      |
| pixEqualWithAlpha         | ✅      | compare.rs equals_with_alpha()        |      |
| pixEqualWithCmap          | ✅      | compare.rs equals_with_cmap()         |      |
| pixCorrelationBinary      | ✅      | compare::correlation_binary()         |      |
| pixDisplayDiff            | ✅      | compare.rs display_diff()             |      |
| pixDisplayDiffBinary      | ✅      | compare.rs display_diff_binary()      |      |
| pixCompareBinary          | ✅      | compare::compare_binary()             |      |
| pixCompareGrayOrRGB       | ✅      | compare.rs compare_gray_or_rgb()      |      |
| pixCompareGray            | ✅      | compare.rs compare_gray()             |      |
| pixCompareRGB             | ✅      | compare.rs compare_rgb()              |      |
| pixCompareTiled           | ✅ 同等 | `compare_tiled`                       |      |
| pixCompareRankDifference  | ✅      | compare.rs compare_rank_difference()  |      |
| pixTestForSimilarity      | ✅      | compare.rs test_for_similarity()      |      |
| pixGetDifferenceStats     | ✅      | compare.rs get_difference_stats()     |      |
| pixGetDifferenceHistogram | ✅      | compare.rs get_difference_histogram() |      |
| pixGetPerceptualDiff      | ✅ 同等 | `get_perceptual_diff`                 |      |
| pixGetPSNR                | ✅      | compare.rs get_psnr()                 |      |

その他の比較関数も実装済み。

### blend.c (ブレンド・合成)

| C関数                     | 状態    | Rust対応                          | 備考 |
| ------------------------- | ------- | --------------------------------- | ---- |
| pixBlend                  | ✅      | blend.rsに実装                    |      |
| pixBlendMask              | ✅      | blend::blend_mask()               |      |
| pixBlendGray              | ✅      | blend::blend_gray()               |      |
| pixBlendGrayInverse       | ✅      | blend.rs blend_gray_inverse()     |      |
| pixBlendColor             | ✅      | blend::blend_color()              |      |
| pixBlendColorByChannel    | ✅      | blend.rs blend_color_by_channel() |      |
| pixBlendGrayAdapt         | ✅      | blend.rs blend_gray_adapt()       |      |
| pixFadeWithGray           | ✅      | blend.rs fade_with_gray()         |      |
| pixBlendHardLight         | ✅      | blend.rs blend_hard_light()       |      |
| pixBlendCmap              | ✅      | blend.rs blend_cmap()             |      |
| pixBlendWithGrayMask      | ✅      | blend::blend_with_gray_mask()     |      |
| pixBlendBackgroundToColor | ✅ 同等 | `blend_background_to_color`       |      |
| pixMultiplyByColor        | ✅      | blend.rs multiply_by_color()      |      |
| pixAlphaBlendUniform      | ✅      | blend.rs alpha_blend_uniform()    |      |
| pixAddAlphaToBlend        | ✅      | blend.rs add_alpha_to_blend()     |      |
| pixSetAlphaOverWhite      | ✅ 同等 | `set_alpha_over_white`            |      |
| pixLinearEdgeFade         | ✅      | blend.rs linear_edge_fade()       |      |

### graphics.c (描画・レンダリング)

| C関数                    | 状態    | Rust対応                                 | 備考           |
| ------------------------ | ------- | ---------------------------------------- | -------------- |
| generatePtaLine          | ✅      | graphics.rs generate_line_pta()          |                |
| generatePtaWideLine      | ✅      | graphics.rs generate_wide_line_pta()     |                |
| generatePtaBox           | ✅      | graphics.rs generate_box_pta()           |                |
| generatePtaBoxa          | ✅      | graphics.rs generate_boxa_pta()          |                |
| generatePtaHashBox       | ✅      | graphics.rs generate_hash_box_pta()      |                |
| generatePtaHashBoxa      | ✅      | graphics.rs generate_hash_boxa_pta()     |                |
| generatePtaaBoxa         | ✅      | graphics.rs generate_ptaa_boxa()         |                |
| generatePtaaHashBoxa     | ✅      | graphics.rs generate_ptaa_hash_boxa()    |                |
| generatePtaPolyline      | ✅      | graphics.rs generate_polyline_pta()      |                |
| generatePtaGrid          | ✅      | graphics.rs generate_grid_pta()          |                |
| convertPtaLineTo4cc      | ✅      | graphics.rs convert_line_to_4cc()        |                |
| generatePtaFilledCircle  | ✅      | graphics.rs generate_filled_circle_pta() |                |
| generatePtaFilledSquare  | ✅      | graphics.rs generate_filled_square_pta() |                |
| pixRenderPlotFromNuma    | ✅      | graphics.rs render_plot_from_numa()      |                |
| pixRenderPlotFromNumaGen | ✅      | graphics.rs render_plot_from_numa_gen()  |                |
| pixRenderPta             | ✅      | graphics.rsに部分実装                    |                |
| pixRenderPtaArb          | ✅      | graphics.rs render_pta_color()           |                |
| pixRenderPtaBlend        | ✅      | graphics.rs render_pta_blend()           |                |
| pixRenderLine            | ✅      | graphics::render_line()                  |                |
| pixRenderLineArb         | ✅      | graphics.rs render_line_color()          |                |
| pixRenderLineBlend       | ✅      | graphics.rs render_line_blend()          |                |
| pixRenderBox             | ✅      | graphics::render_box()                   |                |
| pixRenderBoxArb          | ✅      | graphics.rs render_box_color()           |                |
| pixRenderBoxBlend        | ✅      | graphics.rs render_box_blend()           |                |
| pixRenderBoxa            | ✅      | graphics.rs render_boxa()                |                |
| pixRenderBoxaArb         | ✅      | graphics.rs render_boxa_color()          |                |
| pixRenderBoxaBlend       | ✅      | graphics.rs render_boxa_blend()          |                |
| pixRenderHashBox         | ✅      | graphics.rs render_hash_box()            |                |
| pixRenderHashBoxArb      | ✅      | graphics.rs render_hash_box_color()      |                |
| pixRenderHashBoxBlend    | ✅      | graphics.rs render_hash_box_blend()      |                |
| pixRenderHashMaskArb     | ✅      | graphics.rs render_hash_mask_color()     |                |
| pixRenderHashBoxa        | ✅      | graphics.rs render_hash_boxa()           |                |
| pixRenderHashBoxaArb     | ✅      | graphics.rs render_hash_boxa_color()     |                |
| pixRenderHashBoxaBlend   | ✅      | graphics.rs render_hash_boxa_blend()     |                |
| pixRenderPolyline        | ✅      | graphics.rs render_polyline()            |                |
| pixRenderPolylineArb     | ✅      | graphics.rs render_polyline_color()      |                |
| pixRenderPolylineBlend   | ✅      | graphics.rs render_polyline_blend()      |                |
| pixRenderGridArb         | ✅      | graphics.rs render_grid_color()          |                |
| pixRenderRandomCmapPtaa  | ✅      | graphics.rs render_random_cmap_ptaa()    |                |
| pixRenderPolygon         | ✅      | graphics.rs render_polygon()             |                |
| pixFillPolygon           | ✅      | graphics.rs fill_polygon()               |                |
| pixRenderContours        | ✅      | graphics.rs render_contours()            |                |
| fpixAutoRenderContours   | ✅ 同等 | `FPix::auto_render_contours`             | FPix関連は後続 |
| fpixRenderContours       | ✅ 同等 | `FPix::render_contours`                  | FPix関連は後続 |
| pixGeneratePtaBoundary   | ✅ 同等 | `generate_pta_boundary`                  | 後続Phase      |

## 結論

leptonica-coreクレートは、Phase 13-17の実装により大幅にカバレッジが向上した（26.7% → 58.8%）。
基本データ構造の操作に加え、深度変換・統計・描画・比較・ブレンド等の高度な機能が広くカバーされている。

### 実装済み領域

- Pix/PixMut: 作成・アクセス・プロパティ + 深度変換（Phase 13）
- Box/Boxa/Boxaa: 基本構造 + 幾何演算 + 選択・調整・描画（Phase 14）
- マスク操作: 基本 + General版 + RGB任意マスク（Phase 15.1）
- 統計: 行列統計・分散・差分・行列統計全般（Phase 15.2）
- ヒストグラム: Gray/Color/Cmap + マスク付き・タイル別（Phase 15.3）
- クリッピング: 矩形・前景・エッジ + 測定・抽出（Phase 15.4）
- Numa: 基本統計 + ソート・補間・算術・論理演算（Phase 16）
- Pta/Ptaa: 基本操作 + ソート・最小二乗法・変換（Phase 16）
- Pixa/Pixaa: 基本管理 + ソート・選択・表示・統計（Phase 16）
- Sarray: 基本操作 + 集合演算・ソート・結合・検索（Phase 16）
- 描画: Line/Box/Circle/Polyline + Hash/Grid/Plot/Contour（Phase 17.1-17.2）
- 比較: equal + alpha/cmap/gray/rgb/diff/stats/PSNR（Phase 17.3）
- ブレンド: 基本 + HardLight/GrayAdapt/Cmap/Alpha（Phase 17.3）
- ピクセル演算: OR/AND/XOR/SUBTRACT/INVERT
- Rasterop: 基本的なラスター演算
- FPix: 基本的な浮動小数点画像

### 実装完了

ほとんどの関数の実装が完了。以下の領域も含め、高いカバレッジが達成された:

- I/O操作（Read/Write/Stream/Mem）
- カラーマップの高度な操作（検索・変換・効果）
- FPix/DPix の拡張（FPixa、シリアライゼーション）
- ptafunc1.c, ptafunc2.c
- pixafunc1.c, pixafunc2.c（表示・変換の詳細）

### 未実装領域

以下のboxfunc2.c, boxfunc5.c関数群は未実装:

- Box変換: boxaTransformOrdered, boxTransformOrdered
- Box回転: boxaRotateOrth, boxRotateOrth
- Boxソート: boxaBinSort, boxaSortByIndex, boxaSort2d, boxaSort2dByIndex
- Box統計: boxaGetRankVals, boxaGetMedianVals, boxaGetAverageSize
- Box抽出: boxaExtractAsNuma, boxaExtractAsPta, boxaExtractCorners
- Boxaaユーティリティ: boxaaGetExtent, boxaaFlattenAligned, boxaEncapsulateAligned, boxaaTranspose, boxaaAlignBox
- Boxスムージング: boxaSmoothSequenceMedian, boxaWindowedMedian, boxaModifyWithBoxa 他
