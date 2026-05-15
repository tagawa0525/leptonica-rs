# leptonica (src/core/): C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目      | 数   |
| --------- | ---- |
| ✅ 同等   | 961  |
| 🔄 異なる | 94   |
| 🚫 不要   | 147  |
| ❌ 未実装 | 13   |
| 合計      | 1215 |

**カバレッジ**: 86.9% (1055/1215 関数が実装済み、🚫 不要 147 関数を除くと実質 1055/1068 = 98.8% 実装)

## 注記

- ✅ 同等: Rust版で同じアルゴリズム/機能を持つ関数が存在
- 🔄 異なる: Rust版で異なるAPI/アプローチで実装
- 🚫 不要: Rustの言語機能・設計方針により移植不要
- ❌ 未実装: Rust版に対応する関数が存在しない

Rust版は**Pix/PixMut二層モデル**を採用しているため、C版の一部の関数は異なるAPIで提供される。
例: `pixCopy()` → `Pix::deep_clone()`, `pixClone()` → `Pix::clone()`

## 詳細

### pix1.c (基本的なPix操作)

#### core/pix/mod.rs (pix1.c)

| C関数                   | 状態 | Rust対応                         | 備考                       |
| ----------------------- | ---- | -------------------------------- | -------------------------- |
| pixCreate               | ✅   | Pix::new()                       |                            |
| pixCreateTemplate       | ✅   | Pix::create_template             |                            |
| pixCreateWithCmap       | ✅   | Pix::new_with_colormap           |                            |
| pixClone                | 🔄   | Pix::clone()                     | Arc参照カウントで自動実装  |
| pixCopy                 | 🔄   | Pix::deep_clone()                | deep_cloneが完全コピー     |
| pixGetWidth             | ✅   | Pix::width()                     |                            |
| pixGetHeight            | ✅   | Pix::height()                    |                            |
| pixGetDepth             | ✅   | Pix::depth()                     |                            |
| pixGetSpp               | ✅   | Pix::spp()                       |                            |
| pixGetWpl               | ✅   | Pix::wpl()                       |                            |
| pixGetXRes              | ✅   | Pix::xres()                      |                            |
| pixGetYRes              | ✅   | Pix::yres()                      |                            |
| pixGetInputFormat       | ✅   | Pix::informat()                  |                            |
| pixGetText              | ✅   | Pix::text()                      |                            |
| pixGetColormap          | ✅   | Pix::colormap()                  |                            |
| pixGetData              | ✅   | Pix::data()                      |                            |
| pixDestroy              | 🔄   | drop()                           | Rustのデストラクタで自動   |
| pixGetTextCompNew       | ✅   | Pix::get_text_comp_new           |                            |
| pixSetTextCompNew       | ✅   | PixMut::set_text_comp_new        |                            |
| pixSetSpp               | 🔄   | PixMut::set_spp()                | PixMutで可変               |
| pixSetXRes              | 🔄   | PixMut::set_xres()               |                            |
| pixSetYRes              | 🔄   | PixMut::set_yres()               |                            |
| pixSetResolution        | 🔄   | PixMut::set_resolution()         |                            |
| pixSetInputFormat       | 🔄   | PixMut::set_informat()           |                            |
| pixSetSpecial           | 🔄   | PixMut::set_special()            |                            |
| pixSetText              | 🔄   | PixMut::set_text()               |                            |
| pixSetColormap          | 🔄   | PixMut::set_colormap()           |                            |
| pixGetDimensions        | ✅   | width()/height()/depth()         | 個別メソッドで取得         |
| pixGetResolution        | ✅   | xres()/yres()                    |                            |
| pixCreateNoInit         | 🚫   | -                                | Rustは常に初期化する       |
| pixCreateTemplateNoInit | 🚫   | -                                | Rustは常に初期化する       |
| pixCreateHeader         | 🚫   | -                                | Rustは常に初期化する       |
| pixResizeImageData      | 🚫   | -                                | Rustの所有権モデルで不要   |
| pixTransferAllData      | 🚫   | -                                | Rustの所有権モデルで不要   |
| pixSwapAndDestroy       | 🚫   | -                                | Rustの所有権モデルで不要   |
| pixSetWidth             | 🚫   | -                                | Pixは不変                  |
| pixSetHeight            | 🚫   | -                                | Pixは不変                  |
| pixSetDepth             | 🚫   | -                                | Pixは不変                  |
| pixSetDimensions        | 🚫   | -                                | Pixは不変                  |
| pixCopyDimensions       | 🚫   | -                                | Pixは不変                  |
| pixCopySpp              | 🚫   | -                                | Pixは不変                  |
| pixSetWpl               | 🚫   | -                                | 自動計算のため不要         |
| pixDestroyColormap      | 🚫   | -                                | set_colormap(None)で実現可 |
| pixFreeAndSetData       | 🚫   | -                                | Cメモリ管理                |
| pixSetData              | 🚫   | -                                | Cメモリ管理                |
| pixFreeData             | 🚫   | -                                | Cメモリ管理                |
| pixExtractData          | 🚫   | -                                | Cメモリ管理                |
| pixGetLinePtrs          | 🚫   | -                                | Cポインタ配列              |
| pixPrintStreamInfo      | 🚫   | -                                | Debug traitで対応          |
| pixCopyColormap         | ✅   | PixMut::copy_colormap_from()     |                            |
| pixCopyResolution       | ✅   | PixMut::copy_resolution_from()   |                            |
| pixScaleResolution      | ✅   | PixMut::scale_resolution()       |                            |
| pixCopyInputFormat      | ✅   | PixMut::copy_input_format_from() |                            |
| pixAddText              | ✅   | PixMut::add_text()               |                            |
| pixCopyText             | ✅   | PixMut::copy_text_from()         |                            |
| pixSizesEqual           | ✅   | Pix::sizes_equal()               |                            |
| pixMaxAspectRatio       | ✅   | Pix::max_aspect_ratio()          |                            |

### pix2.c (ピクセルアクセス・設定)

#### core/pix/access.rs (pix2.c)

| C関数       | 状態 | Rust対応            | 備考 |
| ----------- | ---- | ------------------- | ---- |
| pixGetPixel | ✅   | Pix::get_pixel()    |      |
| pixSetPixel | ✅   | PixMut::set_pixel() |      |

#### core/pix/rgb.rs (pix2.c)

| C関数                  | 状態 | Rust対応                     | 備考 |
| ---------------------- | ---- | ---------------------------- | ---- |
| pixEndianByteSwapNew   | ✅   | Pix::endian_byte_swap_new    |      |
| pixEndianByteSwap      | ✅   | PixMut::endian_byte_swap     |      |
| pixEndianTwoByteSwap   | ✅   | PixMut::endian_two_byte_swap |      |
| pixGetRGBComponentCmap | ✅   | Pix::get_rgb_component_cmap  |      |
| pixCopyRGBComponent    | ✅   | PixMut::copy_rgb_component   |      |
| pixGetRGBLine          | ✅   | Pix::get_rgb_line            |      |
| pixInferResolution     | ✅   | Pix::infer_resolution        |      |
| pixAlphaIsOpaque       | ✅   | Pix::alpha_is_opaque         |      |
| pixGetRGBPixel         | ✅   | Pix::get_rgb_pixel()         |      |
| pixSetRGBPixel         | ✅   | PixMut::set_rgb_pixel()      |      |
| pixCreateRGBImage      | ✅   | Pix::create_rgb_image()      |      |
| pixGetRGBComponent     | ✅   | Pix::get_rgb_component()     |      |
| pixSetRGBComponent     | ✅   | PixMut::set_rgb_component()  |      |
| pixSetCmapPixel        | ✅   | PixMut::set_cmap_pixel()     |      |

#### core/pix/mod.rs (pix2.c)

| C関数                    | 状態 | Rust対応                         | 備考 |
| ------------------------ | ---- | -------------------------------- | ---- |
| pixGetRandomPixel        | ✅   | Pix::get_random_pixel            |      |
| pixSetComponentArbitrary | ✅   | PixMut::set_component_arbitrary  |      |
| pixClearAll              | 🔄   | PixMut::clear()                  |      |
| pixSetAll                | 🔄   | PixMut::set_all()                |      |
| pixClearPixel            | ✅   | PixMut::clear_pixel()            |      |
| pixFlipPixel             | ✅   | PixMut::flip_pixel()             |      |
| pixGetBlackOrWhiteVal    | ✅   | PixMut::get_black_or_white_val() |      |
| pixSetAllGray            | ✅   | PixMut::set_all_gray()           |      |
| pixSetAllArbitrary       | ✅   | PixMut::set_all_arbitrary()      |      |
| pixSetBlackOrWhite       | ✅   | PixMut::set_black_or_white()     |      |
| pixClearInRect           | ✅   | PixMut::clear_in_rect()          |      |
| pixSetInRect             | ✅   | PixMut::set_in_rect()            |      |
| pixSetInRectArbitrary    | ✅   | PixMut::set_in_rect_arbitrary()  |      |
| pixSetPadBits            | ✅   | PixMut::set_pad_bits()           |      |
| pixSetPadBitsBand        | ✅   | PixMut::set_pad_bits_band()      |      |
| pixSetOrClearBorder      | ✅   | PixMut::set_or_clear_border()    |      |

#### core/pix/blend.rs (pix2.c)

| C関数          | 状態 | Rust対応           | 備考 |
| -------------- | ---- | ------------------ | ---- |
| pixBlendInRect | ✅   | Pix::blend_in_rect |      |

#### core/pix/border.rs (pix2.c)

| C関数                           | 状態 | Rust対応                              | 備考             |
| ------------------------------- | ---- | ------------------------------------- | ---------------- |
| pixSetBorderRingVal             | ✅   | PixMut::set_border_ring_val           |                  |
| pixSetMirroredBorder            | ✅   | PixMut::set_mirrored_border           |                  |
| pixCopyBorder                   | ✅   | Pix::copy_border                      |                  |
| pixAddMultipleBlackWhiteBorders | ✅   | Pix::add_multiple_black_white_borders |                  |
| pixRemoveBorderToSize           | ✅   | Pix::remove_border_to_size            |                  |
| pixAddMixedBorder               | ✅   | Pix::add_mixed_border                 |                  |
| pixAddContinuedBorder           | ✅   | Pix::add_continued_border             |                  |
| pixShiftAndTransferAlpha        | ✅   | Pix::shift_and_transfer_alpha         |                  |
| pixSetBorderVal                 | ✅   | PixMut::set_border_val()              |                  |
| pixAddBlackOrWhiteBorder        | ✅   | Pix::add_black_or_white_border()      |                  |
| pixAddBorderGeneral             | ✅   | Pix::add_border_general()             |                  |
| pixRemoveBorderGeneral          | ✅   | Pix::remove_border_general()          |                  |
| pixAddRepeatedBorder            | ✅   | Pix::add_repeated_border()            |                  |
| pixAddBorder                    | ✅   | Pix::add_border()                     |                  |
| pixRemoveBorder                 | ✅   | Pix::remove_border()                  |                  |
| pixAddMirroredBorder            | ✅   | Pix::add_mirrored_border()            |                  |
| pixDisplayLayersRGBA            | 🚫   | -                                     | デバッグ表示関数 |
| pixGetRasterData                | 🚫   | -                                     | Cポインタ取得    |

#### core/pixel.rs (pix2.c)

| C関数                  | 状態 | Rust対応                                        | 備考 |
| ---------------------- | ---- | ----------------------------------------------- | ---- |
| composeRGBPixel        | ✅   | compose_rgb()                                   |      |
| composeRGBAPixel       | ✅   | compose_rgba()                                  |      |
| extractRGBValues       | ✅   | extract_rgb()                                   |      |
| extractRGBAValues      | ✅   | extract_rgba()                                  |      |
| extractMinMaxComponent | ✅   | extract_min_component()/extract_max_component() |      |

### pix3.c (マスク・ブール演算)

#### core/pix/mask.rs (pix3.c)

| C関数                       | 状態 | Rust対応                          | 備考      |
| --------------------------- | ---- | --------------------------------- | --------- |
| pixSetMasked                | ✅   | PixMut::set_masked()              |           |
| pixSetMaskedGeneral         | ✅   | PixMut::set_masked_general()      |           |
| pixCombineMasked            | ✅   | PixMut::combine_masked()          |           |
| pixCombineMaskedGeneral     | ✅   | PixMut::combine_masked_general()  |           |
| pixPaintThroughMask         | ✅   | PixMut::paint_through_mask()      |           |
| pixCopyWithBoxa             | ✅   | Pix::copy_with_boxa()             |           |
| pixMakeMaskFromVal          | ✅   | Pix::make_mask_from_val()         |           |
| pixMakeMaskFromLUT          | ✅   | Pix::make_mask_from_lut()         |           |
| pixMakeArbMaskFromRGB       | ✅   | Pix::make_arb_mask_from_rgb()     |           |
| pixSetUnderTransparency     | ✅   | Pix::set_under_transparency()     |           |
| pixPaintSelfThroughMask     | ✅   | Pix::paint_self_through_mask      | 後続Phase |
| pixMakeAlphaFromMask        | ✅   | Pix::make_alpha_from_mask         |           |
| pixGetColorNearMaskBoundary | ✅   | Pix::get_color_near_mask_boundary |           |

#### core/pix/rop.rs (pix3.c)

| C関数                    | 状態 | Rust対応      | 備考               |
| ------------------------ | ---- | ------------- | ------------------ |
| pixInvert                | ✅   | Pix::invert() |                    |
| pixOr                    | ✅   | Pix::or()     |                    |
| pixAnd                   | ✅   | Pix::and()    |                    |
| pixXor                   | ✅   | Pix::xor()    |                    |
| pixDisplaySelectedPixels | 🚫   | -             | デバッグ表示関数   |
| pixMirroredTiling        | 🚫   | -             | デバッグ表示関数   |
| pixFindRepCloseTile      | 🚫   | -             | タイリングヘルパー |

#### core/pix/compare.rs (pix3.c)

| C関数       | 状態 | Rust対応        | 備考 |
| ----------- | ---- | --------------- | ---- |
| pixSubtract | ✅   | Pix::subtract() |      |

#### core/pix/statistics.rs (pix3.c)

| C関数                  | 状態 | Rust対応                    | 備考       |
| ---------------------- | ---- | --------------------------- | ---------- |
| pixZero                | ✅   | Pix::is_zero()              |            |
| pixForegroundFraction  | ✅   | Pix::foreground_fraction()  |            |
| pixCountPixels         | ✅   | Pix::count_pixels()         |            |
| pixCountPixelsInRect   | ✅   | Pix::count_pixels_in_rect() |            |
| pixCountByRow          | ✅   | Pix::count_by_row()         |            |
| pixCountByColumn       | ✅   | Pix::count_by_column()      |            |
| pixCountPixelsByRow    | ✅   | Pix::count_by_row()         | Numa返却版 |
| pixCountPixelsByColumn | ✅   | Pix::count_by_column()      | Numa返却版 |
| pixCountPixelsInRow    | ✅   | Pix::count_pixels_in_row()  |            |
| pixGetMomentByColumn   | ✅   | Pix::get_moment_by_column() |            |
| pixThresholdPixelSum   | ✅   | Pix::threshold_pixel_sum()  |            |
| pixAverageByRow        | ✅   | Pix::average_by_row()       |            |
| pixAverageByColumn     | ✅   | Pix::average_by_column()    |            |
| pixAverageInRect       | ✅   | Pix::average_in_rect()      |            |
| pixAverageInRectRGB    | ✅   | Pix::average_in_rect_rgb()  |            |
| pixVarianceByRow       | ✅   | Pix::variance_by_row()      |            |
| pixVarianceByColumn    | ✅   | Pix::variance_by_column()   |            |
| pixVarianceInRect      | ✅   | Pix::variance_in_rect()     |            |
| pixAbsDiffByRow        | ✅   | Pix::abs_diff_by_row()      |            |
| pixAbsDiffByColumn     | ✅   | Pix::abs_diff_by_column()   |            |
| pixAbsDiffInRect       | ✅   | Pix::abs_diff_in_rect()     |            |
| pixAbsDiffOnLine       | ✅   | Pix::abs_diff_on_line()     |            |
| pixCountArbInRect      | ✅   | Pix::count_arb_in_rect()    |            |

#### core/pixa/mod.rs (pix3.c)

| C関数           | 状態 | Rust対応             | 備考 |
| --------------- | ---- | -------------------- | ---- |
| pixaCountPixels | 🔄   | Pixa::count_pixels() |      |

### pix4.c (ヒストグラム・統計)

#### core/pix/histogram.rs (pix4.c)

| C関数                      | 状態 | Rust対応                        | 備考 |
| -------------------------- | ---- | ------------------------------- | ---- |
| pixGetGrayHistogram        | ✅   | Pix::gray_histogram()           |      |
| pixGetColorHistogram       | ✅   | Pix::color_histogram()          |      |
| pixCountRGBColorsByHash    | ✅   | Pix::count_rgb_colors_by_hash   |      |
| pixGetColorAmapHistogram   | ✅   | Pix::color_amap_histogram()     |      |
| pixGetBinnedComponentRange | ✅   | Pix::get_binned_component_range |      |
| pixGetRankColorArray       | ✅   | Pix::get_rank_color_array       |      |
| pixGetBinnedColor          | ✅   | Pix::get_binned_color           |      |
| pixDisplayColorArray       | ✅   | Pix::display_color_array        |      |
| pixRankBinByStrip          | ✅   | Pix::rank_bin_by_strip          |      |
| pixSplitDistributionFgBg   | ✅   | Pix::split_distribution_fg_bg   |      |
| pixGetGrayHistogramMasked  | ✅   | Pix::gray_histogram_masked()    |      |
| pixGetGrayHistogramInRect  | ✅   | Pix::gray_histogram_in_rect()   |      |
| pixGetGrayHistogramTiled   | ✅   | Pix::gray_histogram_tiled()     |      |
| pixGetColorHistogramMasked | ✅   | Pix::color_histogram_masked()   |      |
| pixGetCmapHistogram        | ✅   | Pix::cmap_histogram()           |      |
| pixGetCmapHistogramMasked  | ✅   | Pix::cmap_histogram_masked()    |      |
| pixGetCmapHistogramInRect  | ✅   | Pix::cmap_histogram_in_rect()   |      |
| pixCountRGBColors          | ✅   | Pix::count_rgb_colors()         |      |
| pixGetRankValueMaskedRGB   | ✅   | Pix::rank_value_masked_rgb()    |      |
| pixGetRankValueMasked      | ✅   | Pix::rank_value_masked()        |      |
| pixGetAverageMaskedRGB     | ✅   | Pix::average_masked_rgb()       |      |
| pixGetAverageMasked        | ✅   | Pix::average_masked()           |      |
| pixGetAverageTiledRGB      | ✅   | Pix::average_tiled_rgb()        |      |
| pixGetAverageTiled         | ✅   | Pix::average_tiled()            |      |
| pixGetMaxColorIndex        | ✅   | Pix::max_color_index()          |      |

#### core/pixa/mod.rs (pix4.c)

| C関数                        | 状態 | Rust対応                        | 備考 |
| ---------------------------- | ---- | ------------------------------- | ---- |
| pixaGetAlignedStats          | ✅   | pixa aligned_stats()            |      |
| pixaExtractColumnFromEachPix | ✅   | pixa extract_column_from_each() |      |

#### core/pix/statistics.rs (pix4.c)

| C関数                | 状態 | Rust対応                 | 備考 |
| -------------------- | ---- | ------------------------ | ---- |
| pixGetRankValue      | ✅   | Pix::pixel_rank_value()  |      |
| pixGetPixelAverage   | ✅   | Pix::get_pixel_average() |      |
| pixGetPixelStats     | ✅   | Pix::get_pixel_stats()   |      |
| pixRowStats          | ✅   | Pix::row_stats()         |      |
| pixColumnStats       | ✅   | Pix::column_stats()      |      |
| pixGetRangeValues    | ✅   | Pix::range_values()      |      |
| pixGetExtremeValue   | ✅   | Pix::extreme_value()     |      |
| pixGetMaxValueInRect | ✅   | Pix::max_value_in_rect() |      |
| pixGetRowStats       | ✅   | Pix::get_row_stats()     |      |
| pixGetColumnStats    | ✅   | Pix::get_column_stats()  |      |

#### core/pix/access.rs (pix4.c)

| C関数             | 状態 | Rust対応                   | 備考 |
| ----------------- | ---- | -------------------------- | ---- |
| pixSetPixelColumn | ✅   | PixMut::set_pixel_column() |      |

#### core/pix/clip.rs (pix4.c)

| C関数               | 状態 | Rust対応                   | 備考 |
| ------------------- | ---- | -------------------------- | ---- |
| pixThresholdForFgBg | ✅   | Pix::threshold_for_fg_bg() |      |

### pix5.c (選択・測定)

#### core/pixa/mod.rs (pix5.c)

| C関数                      | 状態 | Rust対応                        | 備考 |
| -------------------------- | ---- | ------------------------------- | ---- |
| pixaFindDimensions         | ✅   | pixa find_dimensions()          |      |
| pixaFindPerimSizeRatio     | ✅   | Pixa::find_perim_size_ratio     |      |
| pixaFindAreaFractionMasked | ✅   | Pixa::find_area_fraction_masked |      |
| pixaFindWidthHeightRatio   | ✅   | Pixa::find_width_height_ratio   |      |
| pixaFindWidthHeightProduct | ✅   | Pixa::find_width_height_product |      |

#### core/pix/measurement.rs (pix5.c)

| C関数                        | 状態 | Rust対応                         | 備考 |
| ---------------------------- | ---- | -------------------------------- | ---- |
| pixFindAreaPerimRatio        | ✅   | Pix::find_area_perim_ratio       |      |
| pixFindPerimSizeRatio        | ✅   | Pix::find_perim_size_ratio       |      |
| pixFindAreaFraction          | ✅   | Pix::find_area_fraction          |      |
| pixFindAreaFractionMasked    | ✅   | Pix::find_area_fraction_masked   |      |
| pixFindRectangleComps        | ✅   | Pix::find_rectangle_comps        |      |
| pixConformsToRectangle       | ✅   | Pix::conforms_to_rectangle       |      |
| pixExtractRectangularRegions | ✅   | Pix::extract_rectangular_regions |      |
| pixSelectComponentBySize     | ✅   | Pix::select_component_by_size    |      |
| pixFilterComponentBySize     | ✅   | Pix::filter_component_by_size    |      |
| pixMakeCoveringOfRectangles  | ✅   | Pix::make_covering_of_rectangles |      |
| pixaFindPerimToAreaRatio     | ✅   | Pixa::find_perim_to_area_ratio   |      |
| pixaFindAreaFraction         | ✅   | Pixa::find_area_fraction         |      |
| pixFindPerimToAreaRatio      | ✅   | Pix::find_perim_to_area_ratio()  |      |
| pixFindOverlapFraction       | ✅   | Pix::find_overlap_fraction()     |      |

#### core/pix/extract.rs (pix5.c)

| C関数                      | 状態 | Rust対応                         | 備考 |
| -------------------------- | ---- | -------------------------------- | ---- |
| pixReversalProfile         | ✅   | Pix::reversal_profile            |      |
| pixWindowedVarianceOnLine  | ✅   | Pix::windowed_variance_on_line   |      |
| pixMinMaxNearLine          | ✅   | Pix::min_max_near_line           |      |
| pixExtractOnLine           | ✅   | Pix::extract_on_line()           |      |
| pixAverageIntensityProfile | ✅   | Pix::average_intensity_profile() |      |
| pixRankRowTransform        | ✅   | Pix::rank_row_transform()        |      |
| pixRankColumnTransform     | ✅   | Pix::rank_column_transform()     |      |

#### core/pix/clip.rs (pix5.c)

| C関数                      | 状態 | Rust対応                          | 備考       |
| -------------------------- | ---- | --------------------------------- | ---------- |
| pixClipRectangles          | ✅   | Pix::clip_rectangles()            |            |
| pixClipRectangle           | ✅   | Pix::clip_rectangle()             |            |
| pixClipRectangleWithBorder | ✅   | Pix::clip_rectangle_with_border() |            |
| pixClipMasked              | ✅   | Pix::clip_masked()                |            |
| pixCropToMatch             | ✅   | Pix::crop_to_match()              |            |
| pixCropToSize              | ✅   | Pix::crop_to_size()               |            |
| pixResizeToMatch           | ✅   | Pix::resize_to_match()            |            |
| pixMakeSymmetricMask       | ✅   | Pix::make_symmetric_mask()        |            |
| pixMakeFrameMask           | ✅   | Pix::make_frame_mask()            |            |
| pixFractionFgInMask        | ✅   | Pix::fraction_fg_in_mask()        |            |
| pixClipToForeground        | ✅   | Pix::clip_to_foreground()         |            |
| pixTestClipToForeground    | ✅   | Pix::test_clip_to_foreground()    |            |
| pixClipBoxToForeground     | ✅   | Pix::clip_box_to_foreground()     |            |
| pixScanForForeground       | ✅   | Pix::scan_for_foreground()        |            |
| pixClipBoxToEdges          | ✅   | Pix::clip_box_to_edges()          |            |
| pixScanForEdge             | ✅   | Pix::scan_for_edge()              | 8bpp適応版 |
| pixAverageOnLine           | ✅   | Pix::average_on_line()            |            |

### boxbasic.c (Box基本操作)

#### core/box_/mod.rs (boxbasic.c)

| C関数                  | 状態 | Rust対応                   | 備考                      |
| ---------------------- | ---- | -------------------------- | ------------------------- |
| boxCreate              | ✅   | Box::new()                 |                           |
| boxIsValid             | ✅   | Box::is_valid()            |                           |
| boxCreateValid         | 🚫   | -                          | new()でバリデーション実施 |
| boxCopy                | 🔄   | Box自体がCopyトレイト      |                           |
| boxClone               | 🔄   | Box自体がCopyトレイト      |                           |
| boxDestroy             | 🔄   | drop()                     | 自動                      |
| boxGetGeometry         | ✅   | フィールドアクセス         |                           |
| boxSetGeometry         | ✅   | Box::set_geometry()        |                           |
| boxGetSideLocations    | ✅   | Box::side_locations()      |                           |
| boxSetSideLocations    | ✅   | Box::from_side_locations() |                           |
| boxaDestroy            | 🔄   | drop()                     | 自動                      |
| boxaExtendArray        | 🚫   | -                          | Vec自動拡張               |
| boxaExtendArrayToSize  | 🚫   | -                          | Vec自動拡張               |
| boxaGetValidCount      | 🚫   | -                          | Rustの型システムで不要    |
| boxaGetValidBox        | 🚫   | -                          | Rustの型システムで不要    |
| boxaFindInvalidBoxes   | 🚫   | -                          | Rustの型システムで不要    |
| boxaIsFull             | 🚫   | -                          | Rustの型システムで不要    |
| boxaSaveValid          | 🚫   | -                          | Rustの型システムで不要    |
| boxaaDestroy           | 🔄   | drop()                     | 自動                      |
| boxaaExtendArray       | 🚫   | -                          | Vec自動拡張               |
| boxaaExtendArrayToSize | 🚫   | -                          | Vec自動拡張               |
| boxaaInitFull          | 🚫   | -                          | Rustの型システムで不要    |
| boxaaExtendWithInit    | 🚫   | -                          | Rustの型システムで不要    |
| boxaWriteDebug         | 🚫   | -                          | デバッグ出力関数          |
| boxaWriteStderr        | 🚫   | -                          | デバッグ出力関数          |
| boxPrintStreamInfo     | 🚫   | -                          | デバッグ出力関数          |
| boxaCreate             | ✅   | Boxa::new()                |                           |
| boxaCopy               | ✅   | Boxa::clone()              |                           |
| boxaAddBox             | ✅   | Boxa::push()               |                           |
| boxaGetCount           | ✅   | Boxa::len()                |                           |
| boxaGetBox             | ✅   | Boxa::get()                |                           |
| boxaGetBoxGeometry     | ✅   | Boxa::get_box_geometry     |                           |
| boxaReplaceBox         | ✅   | Boxa::replace()            |                           |
| boxaInsertBox          | ✅   | Boxa::insert()             |                           |
| boxaRemoveBox          | ✅   | Boxa::remove()             |                           |
| boxaRemoveBoxAndSave   | ✅   | Boxa::remove_and_save()    |                           |
| boxaClear              | ✅   | Boxa::clear()              |                           |
| boxaaCreate            | ✅   | Boxaa::new()               |                           |
| boxaaCopy              | ✅   | Boxaa::clone()             |                           |
| boxaaAddBoxa           | ✅   | Boxaa::push()              |                           |
| boxaaGetCount          | ✅   | Boxaa::len()               |                           |
| boxaaGetBoxCount       | ✅   | Boxaa::total_boxes()       |                           |
| boxaaGetBoxa           | ✅   | Boxaa::get()               |                           |
| boxaaGetBox            | ✅   | Boxaa::get_box             |                           |
| boxaaReplaceBoxa       | ✅   | Boxaa::replace()           |                           |
| boxaaInsertBoxa        | ✅   | Boxaa::insert()            |                           |
| boxaaRemoveBoxa        | ✅   | Boxaa::remove()            |                           |
| boxaaAddBox            | ✅   | Boxaa::add_box             |                           |
| boxaInitFull           | ✅   | Boxa::init_full()          |                           |

#### core/box_/serial.rs (boxbasic.c)

| C関数              | 状態 | Rust対応                | 備考 |
| ------------------ | ---- | ----------------------- | ---- |
| boxaRead           | ✅   | Boxa::read_from_file    |      |
| boxaReadStream     | ✅   | Boxa::read_from_reader  |      |
| boxaReadMem        | ✅   | Boxa::read_from_bytes   |      |
| boxaWrite          | ✅   | Boxa::write_to_file     |      |
| boxaWriteStream    | ✅   | Boxa::write_to_writer   |      |
| boxaWriteMem       | ✅   | Boxa::write_to_bytes    |      |
| boxaaReadFromFiles | ✅   | Boxaa::read_from_files  |      |
| boxaaRead          | ✅   | Boxaa::read_from_file   |      |
| boxaaReadStream    | ✅   | Boxaa::read_from_reader |      |
| boxaaReadMem       | ✅   | Boxaa::read_from_bytes  |      |
| boxaaWrite         | ✅   | Boxaa::write_to_file    |      |
| boxaaWriteStream   | ✅   | Boxaa::write_to_writer  |      |
| boxaaWriteMem      | ✅   | Boxaa::write_to_bytes   |      |

### boxfunc1.c (Box関係・幾何演算)

#### core/box_/mod.rs (boxfunc1.c)

| C関数               | 状態 | Rust対応                 | 備考 |
| ------------------- | ---- | ------------------------ | ---- |
| boxContains         | ✅   | Box::contains_box()      |      |
| boxIntersects       | ✅   | Box::overlaps()          |      |
| boxOverlapRegion    | ✅   | Box::intersect()         |      |
| boxBoundingRegion   | ✅   | Box::union()             |      |
| boxContainsPt       | ✅   | Box::contains_point()    |      |
| boxEqual            | ✅   | PartialEq trait          |      |
| boxaContainedInBox  | ✅   | Boxa::contained_in_box() |      |
| boxaIntersectsBox   | ✅   | Boxa::intersects_box()   |      |
| boxaClipToBox       | ✅   | Boxa::clip_to_box()      |      |
| boxaCombineOverlaps | ✅   | Boxa::combine_overlaps() |      |
| boxOverlapFraction  | ✅   | Box::overlap_fraction()  |      |
| boxOverlapArea      | ✅   | Box::overlap_area()      |      |
| boxClipToRectangle  | ✅   | Box::clip()              |      |
| boxaSimilar         | 🔄   | Boxa::similar()          |      |
| boxaJoin            | 🔄   | Boxa::join()             |      |

#### transform/rotate.rs (boxfunc1.c)

| C関数        | 状態 | Rust対応                | 備考 |
| ------------ | ---- | ----------------------- | ---- |
| boxGetCenter | ✅   | RotateOptions::center() |      |

#### core/box_/geometry.rs (boxfunc1.c)

| C関数                     | 状態 | Rust対応                         | 備考 |
| ------------------------- | ---- | -------------------------------- | ---- |
| boxaContainedInBoxCount   | ✅   | Boxa::contained_in_box_count()   |      |
| boxaContainedInBoxa       | ✅   | Boxa::all_contained_in()         |      |
| boxaIntersectsBoxCount    | ✅   | Boxa::intersects_box_count()     |      |
| boxaCombineOverlapsInPair | ✅   | Boxa::combine_overlaps_in_pair() |      |
| boxaHandleOverlaps        | ✅   | Boxa::handle_overlaps()          |      |
| boxOverlapDistance        | ✅   | Box::overlap_distance()          |      |
| boxSeparationDistance     | ✅   | Box::separation_distance()       |      |
| boxCompareSize            | ✅   | Box::compare_size()              |      |
| boxaGetNearestToPt        | ✅   | Boxa::nearest_to_point()         |      |
| boxaGetNearestToLine      | ✅   | Boxa::nearest_to_line()          |      |
| boxaFindNearestBoxes      | ✅   | Boxa::find_nearest_boxes()       |      |
| boxaGetNearestByDirection | ✅   | Boxa::nearest_by_direction()     |      |
| boxIntersectByLine        | ✅   | Box::intersect_by_line()         |      |
| boxClipToRectangleParams  | ✅   | Box::clip_to_rectangle_params()  |      |

#### core/box_/adjust.rs (boxfunc1.c)

| C関数                    | 状態 | Rust対応                        | 備考 |
| ------------------------ | ---- | ------------------------------- | ---- |
| boxRelocateOneSide       | ✅   | Box::relocate_one_side()        |      |
| boxaAdjustSides          | ✅   | Boxa::adjust_all_sides()        |      |
| boxaAdjustBoxSides       | ✅   | Boxa::adjust_box_sides()        |      |
| boxAdjustSides           | ✅   | Box::adjust_sides()             |      |
| boxaSetSide              | ✅   | Boxa::set_all_sides()           |      |
| boxSetSide               | ✅   | Box::set_side()                 |      |
| boxaAdjustWidthToTarget  | ✅   | Boxa::adjust_width_to_target()  |      |
| boxaAdjustHeightToTarget | ✅   | Boxa::adjust_height_to_target() |      |
| boxaEqual                | ✅   | Boxa::equal_ordered()           |      |
| boxSimilar               | ✅   | Box::similar_per_side()         |      |
| boxaaJoin                | ✅   | join() (Boxaa)                  |      |
| boxaSplitEvenOdd         | ✅   | Boxa::split_even_odd()          |      |
| boxaMergeEvenOdd         | ✅   | Boxa::merge_even_odd()          |      |

### boxfunc2.c (Box変換ユーティリティ)

#### core/box_/mod.rs (boxfunc2.c)

| C関数               | 状態 | Rust対応                                  | 備考                            |
| ------------------- | ---- | ----------------------------------------- | ------------------------------- |
| boxaTransform       | 🔄   | Boxa::translate() + Boxa::scale()         | shift/scaleを個別メソッドに分離 |
| boxaSort            | 🔄   | Boxa::sort_by_position() / sort_by_area() | ソートタイプ別に個別メソッド化  |
| boxTransform        | 🔄   | Box::translate() + Box::scale()           | shift/scaleを個別メソッドに分離 |
| boxaaGetExtent      | ✅   | Boxaa::get_extent                         |                                 |
| boxaaFlattenToBoxa  | ✅   | Boxaa::flatten()                          |                                 |
| boxaaFlattenAligned | ✅   | Boxaa::flatten_aligned                    |                                 |
| boxaaTranspose      | ✅   | Boxaa::transpose                          |                                 |
| boxaaAlignBox       | ✅   | Boxaa::align_box                          |                                 |

#### core/box_/transform.rs (boxfunc2.c)

| C関数                | 状態 | Rust対応                | 備考 |
| -------------------- | ---- | ----------------------- | ---- |
| boxaTransformOrdered | ✅   | Boxa::transform_ordered |      |
| boxaRotateOrth       | ✅   | Boxa::rotate_orth       |      |
| boxaShiftWithPta     | ✅   | Boxa::shift_with_pta    |      |
| boxTransformOrdered  | ✅   | Box::transform_ordered  |      |
| boxRotateOrth        | ✅   | Box::rotate_orth        |      |

#### core/box_/sort.rs (boxfunc2.c)

| C関数                  | 状態 | Rust対応                  | 備考 |
| ---------------------- | ---- | ------------------------- | ---- |
| boxaBinSort            | ✅   | Boxa::bin_sort            |      |
| boxaSortByIndex        | ✅   | Boxa::sort_by_index       |      |
| boxaSort2d             | ✅   | Boxa::sort_2d             |      |
| boxaSort2dByIndex      | ✅   | Boxa::sort_2d_by_index    |      |
| boxaEncapsulateAligned | ✅   | Boxa::encapsulate_aligned |      |

#### core/box_/extract.rs (boxfunc2.c)

| C関数              | 状態 | Rust対応               | 備考 |
| ------------------ | ---- | ---------------------- | ---- |
| boxaExtractAsNuma  | ✅   | Boxa::extract_as_numa  |      |
| boxaExtractAsPta   | ✅   | Boxa::extract_as_pta   |      |
| boxaExtractCorners | ✅   | Boxa::extract_corners  |      |
| boxaGetRankVals    | ✅   | Boxa::get_rank_vals    |      |
| boxaGetMedianVals  | ✅   | Boxa::get_median_vals  |      |
| boxaGetAverageSize | ✅   | Boxa::get_average_size |      |

### boxfunc3.c (Box描画・マスク)

#### core/box_/draw.rs (boxfunc3.c)

| C関数                     | 状態 | Rust対応                       | 備考         |
| ------------------------- | ---- | ------------------------------ | ------------ |
| pixMaskConnComp           | ✅   | Pix::mask_conn_comp            | conncomp依存 |
| boxaaDisplay              | ✅   | Boxaa::display                 |              |
| pixSplitIntoBoxa          | ✅   | Pix::split_into_boxa           |              |
| pixSplitComponentIntoBoxa | ✅   | Pix::split_component_into_boxa |              |
| makeMosaicStrips          | ✅   | make_mosaic_strips             |              |
| pixSelectLargeULComp      | ✅   | Pix::select_large_ul_comp      | conncomp依存 |
| pixaDisplayBoxaa          | ✅   | Pixa::display_boxaa            |              |
| pixMaskBoxa               | ✅   | PixMut::mask_boxa()            |              |
| pixPaintBoxa              | ✅   | PixMut::paint_boxa()           |              |
| pixSetBlackOrWhiteBoxa    | ✅   | PixMut::set_bw_boxa()          |              |
| pixPaintBoxaRandom        | ✅   | PixMut::paint_boxa_random()    |              |
| pixBlendBoxaRandom        | ✅   | PixMut::blend_boxa_random()    |              |
| pixDrawBoxa               | ✅   | PixMut::draw_boxa()            |              |
| pixDrawBoxaRandom         | ✅   | PixMut::draw_boxa_random()     |              |
| boxaCompareRegions        | ✅   | Boxa::compare_regions()        |              |
| boxaSelectLargeULBox      | ✅   | Boxa::select_large_ul_box()    |              |

### boxfunc4.c (Box選択・変換)

#### core/box_/select.rs (boxfunc4.c)

| C関数                    | 状態 | Rust対応                        | 備考 |
| ------------------------ | ---- | ------------------------------- | ---- |
| boxaSelectRange          | ✅   | Boxa::select_range()            |      |
| boxaaSelectRange         | ✅   | select_range() (Boxaa)          |      |
| boxaMakeSizeIndicator    | ✅   | Boxa::make_size_indicator()     |      |
| boxaMakeAreaIndicator    | ✅   | Boxa::make_area_indicator()     |      |
| boxaMakeWHRatioIndicator | ✅   | Boxa::make_wh_ratio_indicator() |      |
| boxaSelectWithIndicator  | ✅   | Boxa::select_with_indicator()   |      |
| boxaSwapBoxes            | ✅   | Boxa::swap_boxes()              |      |
| boxaaSizeRange           | ✅   | size_range() (Boxaa)            |      |
| boxaLocationRange        | ✅   | Boxa::location_range()          |      |
| boxaGetSizes             | ✅   | Boxa::get_sizes()               |      |
| boxaGetArea              | ✅   | Boxa::get_total_area()          |      |

#### core/box_/mod.rs (boxfunc4.c)

| C関数                   | 状態 | Rust対応                   | 備考 |
| ----------------------- | ---- | -------------------------- | ---- |
| boxaSelectBySize        | ✅   | Boxa::select_by_size()     |      |
| boxaSelectByArea        | ✅   | Boxa::select_by_area()     |      |
| boxaSelectByWHRatio     | ✅   | Boxa::select_by_wh_ratio() |      |
| boxaGetExtent           | ✅   | Boxa::get_extent()         |      |
| boxaGetCoverage         | ✅   | Boxa::get_coverage()       |      |
| boxaSizeRange           | ✅   | Boxa::size_range()         |      |
| boxaPermutePseudorandom | ✅   | Boxa::permute_pseudorandom |      |
| boxaPermuteRandom       | ✅   | Boxa::permute_random       |      |

#### core/box_/draw.rs (boxfunc4.c)

| C関数            | 状態 | Rust対応            | 備考 |
| ---------------- | ---- | ------------------- | ---- |
| boxaDisplayTiled | ✅   | Boxa::display_tiled |      |

#### core/box_/adjust.rs (boxfunc4.c)

| C関数            | 状態 | Rust対応        | 備考 |
| ---------------- | ---- | --------------- | ---- |
| boxaConvertToPta | ✅   | to_pta() (Boxa) |      |
| ptaConvertToBoxa | ✅   | Pta::to_boxa()  |      |
| boxConvertToPta  | ✅   | to_pta() (Box)  |      |
| ptaConvertToBox  | ✅   | Pta::to_box()   |      |

### boxfunc5.c (Boxスムージング・調整)

#### core/box_/smooth.rs (boxfunc5.c)

| C関数                      | 状態 | Rust対応                        | 備考 |
| -------------------------- | ---- | ------------------------------- | ---- |
| boxaSmoothSequenceMedian   | ✅   | Boxa::smooth_sequence_median    |      |
| boxaWindowedMedian         | ✅   | Boxa::windowed_median           |      |
| boxaModifyWithBoxa         | ✅   | Boxa::modify_with_boxa          |      |
| boxaReconcilePairWidth     | ✅   | Boxa::reconcile_pair_width      |      |
| boxaSizeConsistency        | ✅   | Boxa::size_consistency          |      |
| boxaReconcileAllByMedian   | ✅   | Boxa::reconcile_all_by_median   |      |
| boxaReconcileSidesByMedian | ✅   | Boxa::reconcile_sides_by_median |      |
| boxaReconcileSizeByMedian  | ✅   | Boxa::reconcile_size_by_median  |      |
| boxaPlotSides              | ✅   | Boxa::plot_sides                |      |
| boxaPlotSizes              | ✅   | Boxa::plot_sizes                |      |
| boxaFillSequence           | ✅   | Boxa::fill_sequence             |      |
| boxaSizeVariation          | ✅   | Boxa::size_variation            |      |
| boxaMedianDimensions       | ✅   | Boxa::median_dimensions         |      |

### ptabasic.c (Pta基本操作)

#### core/pta/mod.rs (ptabasic.c)

| C関数             | 状態 | Rust対応              | 備考                 |
| ----------------- | ---- | --------------------- | -------------------- |
| ptaCreate         | ✅   | Pta::new()            |                      |
| ptaCreateFromNuma | ✅   | Pta::create_from_numa |                      |
| ptaCopy           | ✅   | Pta::clone()          |                      |
| ptaCopyRange      | ✅   | Pta::copy_range       |                      |
| ptaClone          | ✅   | Pta::clone()          |                      |
| ptaAddPt          | ✅   | Pta::push()           |                      |
| ptaInsertPt       | ✅   | Pta::insert           |                      |
| ptaRemovePt       | ✅   | Pta::remove_pt        |                      |
| ptaGetCount       | ✅   | Pta::len()            |                      |
| ptaGetPt          | ✅   | Pta::get()            |                      |
| ptaGetIPt         | ✅   | Pta::get_i_pt         |                      |
| ptaSetPt          | ✅   | Pta::set()            |                      |
| ptaDestroy        | 🔄   | drop()                | 自動                 |
| ptaEmpty          | 🚫   | -                     | Pta::clear()で対応   |
| ptaGetArrays      | 🚫   | -                     | Cポインタ配列        |
| ptaWriteDebug     | 🚫   | -                     | デバッグ出力関数     |
| ptaaDestroy       | 🔄   | drop()                | Drop traitで自動     |
| ptaaGetPt         | 🚫   | -                     | Vec<Pta>で代替       |
| ptaaWriteDebug    | 🚫   | -                     | Vec<Pta>で代替       |
| ptaaCreate        | ✅   | Ptaa::new()           | Ptaa構造体として実装 |
| ptaaAddPta        | ✅   | Ptaa::push()          |                      |
| ptaaGetCount      | ✅   | Ptaa::len()           |                      |
| ptaaGetPta        | ✅   | Ptaa::get()           |                      |
| ptaaInitFull      | ✅   | Ptaa::init_full()     |                      |
| ptaaReplacePta    | ✅   | Ptaa::replace()       |                      |
| ptaaAddPt         | ✅   | Ptaa::add_pt()        |                      |
| ptaaTruncate      | ✅   | Ptaa::truncate()      |                      |

#### core/pta/serial.rs (ptabasic.c)

| C関数           | 状態 | Rust対応                 | 備考 |
| --------------- | ---- | ------------------------ | ---- |
| ptaRead         | ✅   | Pta::read_from_file      |      |
| ptaReadStream   | ✅   | Pta::read_from_reader    |      |
| ptaReadMem      | ✅   | Pta::read_from_bytes     |      |
| ptaWrite        | ✅   | Pta::write_to_file       |      |
| ptaWriteStream  | ✅   | Pta::write_to_writer     |      |
| ptaWriteMem     | ✅   | Pta::write_to_bytes      |      |
| ptaaRead        | ✅   | Ptaa::read_from_file()   |      |
| ptaaReadStream  | ✅   | Ptaa::read_from_reader() |      |
| ptaaReadMem     | ✅   | Ptaa::read_from_bytes()  |      |
| ptaaWrite       | ✅   | Ptaa::write_to_file()    |      |
| ptaaWriteStream | ✅   | Ptaa::write_to_writer()  |      |
| ptaaWriteMem    | ✅   | Ptaa::write_to_bytes()   |      |

### ptafunc1.c, ptafunc2.c (Pta変換・演算)

Phase 16で大部分を実装済み。

#### core/mod.rs (ptafunc1.c, ptafunc2.c)

| C関数 | 状態 | Rust対応 | 備考 |
| ----- | ---- | -------- | ---- |

#### core/pta/transform.rs (ptafunc1.c, ptafunc2.c)

| C関数               | 状態 | Rust対応                 | 備考 |
| ------------------- | ---- | ------------------------ | ---- |
| ptaTranspose        | ✅   | Pta::transpose()         |      |
| ptaCyclicPerm       | ✅   | Pta::cyclic_perm()       |      |
| ptaGetRange         | ✅   | Pta::get_range()         |      |
| ptaGetInsideBox     | ✅   | Pta::get_inside_box()    |      |
| ptaContainsPt       | ✅   | Pta::contains_pt()       |      |
| ptaTestIntersection | ✅   | Pta::test_intersection() |      |
| ptaTransform        | ✅   | Pta::transform_pts()     |      |
| ptaPtInsidePolygon  | ✅   | Pta::pt_inside_polygon() |      |
| ptaPolygonIsConvex  | ✅   | Pta::polygon_is_convex() |      |
| ptaGetMinMax        | ✅   | Pta::get_min_max()       |      |
| ptaSelectByValue    | ✅   | Pta::select_by_value()   |      |
| ptaSelectRange      | ✅   | Pta::select_range()      |      |
| ptaaJoin            | ✅   | Ptaa::join()             |      |
| ptaSubsample        | 🔄   | Pta::subsample()         |      |
| ptaReverse          | 🔄   | Pta::reverse()           |      |
| ptaJoin             | 🔄   | Pta::join()              |      |

#### core/pta/mod.rs (ptafunc1.c, ptafunc2.c)

| C関数         | 状態 | Rust対応          | 備考 |
| ------------- | ---- | ----------------- | ---- |
| ptaCropToMask | ✅   | Pta::crop_to_mask |      |

#### core/pta/lsf.rs (ptafunc1.c, ptafunc2.c)

| C関数              | 状態 | Rust対応                 | 備考 |
| ------------------ | ---- | ------------------------ | ---- |
| ptaGetLinearLSF    | ✅   | Pta::get_linear_lsf()    |      |
| ptaGetQuadraticLSF | ✅   | Pta::get_quadratic_lsf() |      |
| ptaGetCubicLSF     | ✅   | Pta::get_cubic_lsf()     |      |
| ptaGetQuarticLSF   | ✅   | Pta::get_quartic_lsf()   |      |

#### core/pta/sort.rs (ptafunc1.c, ptafunc2.c)

| C関数           | 状態 | Rust対応              | 備考 |
| --------------- | ---- | --------------------- | ---- |
| ptaGetSortIndex | ✅   | Pta::get_sort_index() |      |
| ptaSort         | ✅   | Pta::sort_pta()       |      |
| ptaGetRankValue | ✅   | Pta::get_rank_value() |      |
| ptaEqual        | ✅   | Pta::equal()          |      |
| ptaSortByIndex  | 🔄   | Pta::sort_by_index()  |      |
| ptaSort2d       | 🔄   | Pta::sort_2d()        |      |

### pixabasic.c (Pixa基本操作)

#### core/pixa/mod.rs (pixabasic.c)

| C関数                 | 状態 | Rust対応                  | 備考                   |
| --------------------- | ---- | ------------------------- | ---------------------- |
| pixaCreate            | ✅   | Pixa::new()               |                        |
| pixaCreateFromPix     | ✅   | Pixa::create_from_pix     |                        |
| pixaCreateFromBoxa    | ✅   | Pixa::create_from_boxa    |                        |
| pixaSplitPix          | ✅   | Pixa::split_pix           |                        |
| pixaCopy              | ✅   | Pixa::clone()             |                        |
| pixaAddPix            | ✅   | Pixa::push()              |                        |
| pixaAddBox            | ✅   | Pixa::push_with_box()     |                        |
| pixaGetCount          | ✅   | Pixa::len()               |                        |
| pixaGetPix            | ✅   | Pixa::get_cloned()        |                        |
| pixaGetPixDimensions  | ✅   | Pixa::get_dimensions()    |                        |
| pixaGetBoxaCount      | ✅   | Pixa::boxa_count()        |                        |
| pixaGetBox            | ✅   | Pixa::get_box             |                        |
| pixaGetBoxGeometry    | ✅   | Pixa::get_box_geometry    |                        |
| pixaSetBoxa           | ✅   | Pixa::set_boxa            |                        |
| pixaCountText         | ✅   | Pixa::count_text          |                        |
| pixaSetText           | ✅   | Pixa::set_text            |                        |
| pixaInsertPix         | 🔄   | Pixa::insert()            | box引数は別操作に分離  |
| pixaRemovePix         | ✅   | Pixa::remove_pix          |                        |
| pixaRemovePixAndSave  | ✅   | Pixa::remove_pix_and_save |                        |
| pixaRemoveSelected    | ✅   | Pixa::remove_selected     |                        |
| pixaInitFull          | ✅   | Pixa::init_full           |                        |
| pixaClear             | ✅   | Pixa::clear()             |                        |
| pixaJoin              | ✅   | Pixa::join                |                        |
| pixaInterleave        | ✅   | Pixa::interleave          |                        |
| pixaReadBoth          | ✅   | Pixa::read_both           |                        |
| pixaDestroy           | 🔄   | drop()                    | 自動                   |
| pixaExtendArray       | 🚫   | -                         | Vec自動拡張            |
| pixaExtendArrayToSize | 🚫   | -                         | Vec自動拡張            |
| pixaGetPixArray       | 🚫   | -                         | Cポインタ配列          |
| pixaVerifyDepth       | 🚫   | -                         | Rustの型システムで不要 |
| pixaVerifyDimensions  | 🚫   | -                         | Rustの型システムで不要 |
| pixaIsFull            | 🚫   | -                         | Rustの型システムで不要 |
| pixaGetLinePtrs       | 🚫   | -                         | Cポインタ配列          |
| pixaWriteStreamInfo   | 🚫   | -                         | デバッグ出力関数       |
| pixaaCreateFromPixa   | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaDestroy          | 🔄   | drop()                    | Drop traitで自動       |
| pixaaExtendArray      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaAddPix           | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaAddBox           | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaGetBoxa          | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaVerifyDepth      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaVerifyDimensions | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaTruncate         | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaWriteDebug        | 🚫   | -                         | デバッグ出力関数       |
| pixaaReadFromFiles    | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaRead             | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaReadStream       | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaReadMem          | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWrite            | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWriteStream      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWriteMem         | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaJoin             | ✅   | Pixaa::join()             |                        |
| pixaaCreate           | ✅   | Pixaa::new()              | Pixaa構造体として実装  |
| pixaaAddPixa          | ✅   | Pixaa::push()             |                        |
| pixaaGetCount         | ✅   | Pixaa::len()              |                        |
| pixaaGetPixa          | ✅   | Pixaa::get()              |                        |
| pixaaGetPix           | ✅   | Pixaa::get_pix()          |                        |
| pixaaIsFull           | ✅   | Pixaa::is_full()          |                        |
| pixaaInitFull         | ✅   | Pixaa::init_full()        |                        |
| pixaaReplacePixa      | ✅   | Pixaa::replace()          |                        |
| pixaaClear            | ✅   | Pixaa::clear()            |                        |

#### core/pixcomp.rs (pixabasic.c)

| C関数          | 状態 | Rust対応          | 備考 |
| -------------- | ---- | ----------------- | ---- |
| pixaGetBoxa    | ✅   | Pixa::get_boxa    |      |
| pixaReplacePix | ✅   | Pixa::replace_pix |      |

#### core/pixa/serial.rs (pixabasic.c)

| C関数           | 状態 | Rust対応               | 備考 |
| --------------- | ---- | ---------------------- | ---- |
| pixaRead        | ✅   | Pixa::read_from_file   |      |
| pixaReadStream  | ✅   | Pixa::read_from_reader |      |
| pixaReadMem     | ✅   | Pixa::read_from_bytes  |      |
| pixaWrite       | ✅   | Pixa::write_to_file    |      |
| pixaWriteStream | ✅   | Pixa::write_to_writer  |      |
| pixaWriteMem    | ✅   | Pixa::write_to_bytes   |      |

### pixafunc1.c, pixafunc2.c (Pixa選択・変換・表示)

Phase 16で主要機能を実装済み。

#### core/mod.rs (pixafunc1.c, pixafunc2.c)

| C関数    | 状態 | Rust対応    | 備考 |
| -------- | ---- | ----------- | ---- |
| pixaSort | ✅   | pixa sort() |      |

#### core/pixa/mod.rs (pixafunc1.c, pixafunc2.c)

| C関数                        | 状態 | Rust対応                        | 備考 |
| ---------------------------- | ---- | ------------------------------- | ---- |
| pixaScaleToSize              | ✅   | pixa scale_to_size()            |      |
| pixaScaleToSizeRel           | ✅   | pixa scale_to_size_rel()        |      |
| pixaDisplayTiledAndScaled    | ✅   | pixa display_tiled_and_scaled() |      |
| pixaGetAlignedStats          | ✅   | pixa aligned_stats()            |      |
| pixaExtractColumnFromEachPix | ✅   | pixa extract_column_from_each() |      |
| pixaFindDimensions           | ✅   | pixa find_dimensions()          |      |
| pixaDisplay                  | ✅   | pixa display()                  |      |
| pixaDisplayTiled             | ✅   | pixa display_tiled()            |      |
| pixaCountPixels              | ✅   | pixa count_pixels()             |      |
| pixaSelectBySize             | 🔄   | Pixa::select_by_size()          |      |
| pixaSelectByArea             | 🔄   | Pixa::select_by_area()          |      |
| pixaSortByIndex              | 🔄   | Pixa::sort_by_index()           |      |

### numabasic.c (Numa基本操作)

実装済み関数が存在する。C版のnumabasic.cのI/O関連関数も実装済み。
numa/mod.rs, numa/operations.rs に基本統計関数は実装済み。

### numafunc1.c, numafunc2.c (Numa演算・統計)

#### core/numa/operations.rs (numafunc1.c, numafunc2.c)

| C関数                      | 状態 | Rust対応                           | 備考                   |
| -------------------------- | ---- | ---------------------------------- | ---------------------- |
| numaArithOp                | ✅   | Numa::arith_op()                   |                        |
| numaLogicalOp              | ✅   | Numa::logical_op()                 |                        |
| numaInvert                 | ✅   | Numa::invert()                     |                        |
| numaSimilar                | ✅   | Numa::similar()                    |                        |
| numaAddToNumber            | ✅   | Numa::add_to_element()             |                        |
| numaGetPartialSums         | ✅   | Numa::partial_sums()               |                        |
| numaSubsample              | ✅   | Numa::subsample()                  |                        |
| numaMakeDelta              | ✅   | Numa::make_delta()                 |                        |
| numaMakeSequence           | ✅   | Numa::make_sequence()              |                        |
| numaMakeAbsval             | ✅   | Numa::abs_val()                    |                        |
| numaAddBorder              | ✅   | Numa::add_border()                 |                        |
| numaAddSpecifiedBorder     | ✅   | Numa::add_specified_border()       |                        |
| numaRemoveBorder           | ✅   | Numa::remove_border()              |                        |
| numaCountNonzeroRuns       | ✅   | Numa::count_nonzero_runs()         |                        |
| numaGetNonzeroRange        | ✅   | Numa::get_nonzero_range()          |                        |
| numaGetCountRelativeToZero | ✅   | Numa::get_count_relative_to_zero() |                        |
| numaClipToInterval         | ✅   | Numa::clip_to_interval()           |                        |
| numaMakeThresholdIndicator | ✅   | Numa::make_threshold_indicator()   |                        |
| numaInterpolateEqxVal      | ✅   | Numa::interpolate_eqx_val()        |                        |
| numaInterpolateArbxVal     | ✅   | Numa::interpolate_arbx_val()       |                        |
| numaSortAutoSelect         | ✅   | Numa::sort_auto_select()           |                        |
| numaSortIndexAutoSelect    | ✅   | Numa::sort_index_auto_select()     |                        |
| numaGetSortIndex           | ✅   | Numa::sort_index()                 |                        |
| numaIsSorted               | ✅   | Numa::is_sorted()                  |                        |
| numaSortByIndex            | ✅   | Numa::sort_by_index()              |                        |
| numaJoin                   | ✅   | Numa::join()                       |                        |
| numaMakeConstant           | ✅   | Numa::make_constant()              |                        |
| numaReverse                | ✅   | Numa::reversed() / Numa::reverse() |                        |
| numaSortGeneral            | ✅   | Numa::sort_general                 | sort_auto_selectで統合 |
| numaChooseSortType         | ✅   | Numa::choose_sort_type             | 内部関数               |
| numaSort                   | ✅   | Numa::sorted() / Numa::sort()      |                        |
| numaGetRankValue           | ✅   | Numa::rank_value()                 |                        |
| numaGetMedian              | ✅   | Numa::median()                     |                        |
| numaGetMode                | ✅   | Numa::mode()                       |                        |
| numaaJoin                  | ✅   | Numaa::join                        |                        |

#### core/numa/mod.rs (numafunc1.c, numafunc2.c)

| C関数                | 状態 | Rust対応                  | 備考 |
| -------------------- | ---- | ------------------------- | ---- |
| numaGetMin           | ✅   | Numa::min()               |      |
| numaGetMax           | ✅   | Numa::max()               |      |
| numaGetSum           | ✅   | Numa::sum()               |      |
| numaGetSumOnInterval | ✅   | Numa::sum_on_interval()   |      |
| numaHasOnlyIntegers  | ✅   | Numa::has_only_integers() |      |
| numaGetMean          | ✅   | Numa::mean()              |      |
| numaGetMeanAbsval    | ✅   | Numa::mean_absval()       |      |
| numaaFlattenToNuma   | ✅   | Numaa::flatten()          |      |

#### core/numa/interpolation.rs (numafunc1.c, numafunc2.c)

| C関数                       | 状態 | Rust対応                          | 備考 |
| --------------------------- | ---- | --------------------------------- | ---- |
| numaUniformSampling         | ✅   | Numa::uniform_sampling()          |      |
| numaLowPassIntervals        | ✅   | Numa::low_pass_intervals()        |      |
| numaThresholdEdges          | ✅   | Numa::threshold_edges()           |      |
| numaGetSpanValues           | ✅   | Numa::get_span_values()           |      |
| numaGetEdgeValues           | ✅   | Numa::get_edge_values()           |      |
| numaInterpolateEqxInterval  | ✅   | Numa::interpolate_eqx_interval()  |      |
| numaInterpolateArbxInterval | ✅   | Numa::interpolate_arbx_interval() |      |
| numaFitMax                  | ✅   | Numa::fit_max()                   |      |
| numaDifferentiateInterval   | ✅   | Numa::differentiate_interval()    |      |
| numaIntegrateInterval       | ✅   | Numa::integrate_interval()        |      |

#### core/numa/sort.rs (numafunc1.c, numafunc2.c)

| C関数                      | 状態 | Rust対応                       | 備考 |
| -------------------------- | ---- | ------------------------------ | ---- |
| numaGetBinSortIndex        | ✅   | Numa::bin_sort_index()         |      |
| numaSortPair               | ✅   | Numa::sort_pair()              |      |
| numaInvertMap              | ✅   | Numa::invert_map()             |      |
| numaAddSorted              | ✅   | Numa::add_sorted()             |      |
| numaFindSortedLoc          | ✅   | Numa::find_sorted_loc()        |      |
| numaPseudorandomSequence   | ✅   | Numa::pseudorandom_sequence()  |      |
| numaRandomPermutation      | ✅   | Numa::random_permutation()     |      |
| numaGetBinnedMedian        | ✅   | Numa::binned_median()          |      |
| numaGetMeanDevFromMedian   | ✅   | Numa::mean_dev_from_median()   |      |
| numaGetMedianDevFromMedian | ✅   | Numa::median_dev_from_median() |      |
| numaBinSort                | 🔄   | Numa::bin_sort()               |      |

numafunc2.c (ヒストグラム・統計)の関数も実装済み。
一部ヒストグラム関数はnuma/histogram.rsに実装あり。

### sarray1.c, sarray2.c (Sarray文字列配列)

#### core/sarray/mod.rs (sarray1.c, sarray2.c)

| C関数                       | 状態 | Rust対応                       | 備考             |
| --------------------------- | ---- | ------------------------------ | ---------------- |
| sarrayCreate                | ✅   | Sarray::new()                  |                  |
| sarrayCreateInitialized     | ✅   | Sarray::initialized()          |                  |
| sarrayCreateWordsFromString | ✅   | Sarray::from_words()           |                  |
| sarrayCreateLinesFromString | ✅   | Sarray::from_lines()           |                  |
| sarrayDestroy               | 🔄   | drop()                         | 自動             |
| sarrayCopy                  | ✅   | Sarray::clone()                |                  |
| sarrayClone                 | ✅   | Sarray::clone()                |                  |
| sarrayAddString             | ✅   | Sarray::push()                 |                  |
| sarrayRemoveString          | ✅   | Sarray::remove                 |                  |
| sarrayReplaceString         | ✅   | Sarray::replace                |                  |
| sarrayClear                 | ✅   | Sarray::clear()                |                  |
| sarrayGetCount              | ✅   | Sarray::len()                  |                  |
| sarrayGetArray              | 🚫   | -                              | Cポインタ配列    |
| sarrayGetString             | ✅   | Sarray::get()                  |                  |
| sarrayToString              | ✅   | Sarray::join()                 |                  |
| sarrayToStringRange         | ✅   | Sarray::join_range             |                  |
| sarrayConcatUniformly       | ✅   | Sarray::concat_uniformly       |                  |
| sarrayJoin                  | ✅   | Sarray::append                 |                  |
| sarrayAppendRange           | ✅   | Sarray::append_range           |                  |
| sarrayPadToSameSize         | ✅   | Sarray::pad_to_same_size       |                  |
| sarrayConvertWordsToLines   | ✅   | Sarray::convert_words_to_lines |                  |
| sarraySplitString           | ✅   | Sarray::split_string           |                  |
| sarraySelectBySubstring     | ✅   | Sarray::filter_by_substring()  |                  |
| sarraySelectRange           | ✅   | Sarray::select_range           |                  |
| sarrayParseRange            | ✅   | Sarray::parse_range            |                  |
| sarrayWriteStderr           | 🚫   | -                              | デバッグ出力関数 |
| sarrayAppend                | ✅   | Sarray::append                 |                  |
| sarraySort                  | ✅   | Sarray::sort()                 |                  |
| sarraySortByIndex           | ✅   | Sarray::sort_by_index          |                  |

#### core/sarray/serial.rs (sarray1.c, sarray2.c)

| C関数             | 状態 | Rust対応                 | 備考 |
| ----------------- | ---- | ------------------------ | ---- |
| sarrayRead        | ✅   | Sarray::read_from_file   |      |
| sarrayReadStream  | ✅   | Sarray::read_from_reader |      |
| sarrayReadMem     | ✅   | Sarray::read_from_bytes  |      |
| sarrayWrite       | ✅   | Sarray::write_to_file    |      |
| sarrayWriteStream | ✅   | Sarray::write_to_writer  |      |
| sarrayWriteMem    | ✅   | Sarray::write_to_bytes   |      |

その他のsarray2.c関数（セット演算、整数生成など）も実装済み。

### fpix1.c, fpix2.c (FPix浮動小数点画像)

#### core/fpix/mod.rs (fpix1.c, fpix2.c)

| C関数                  | 状態 | Rust対応                                              | 備考                   |
| ---------------------- | ---- | ----------------------------------------------------- | ---------------------- |
| fpixCreate             | ✅   | FPix::new()                                           |                        |
| fpixCreateTemplate     | ✅   | FPix::create_template()                               |                        |
| fpixClone              | ✅   | FPix::clone()                                         |                        |
| fpixCopy               | ✅   | FPix::clone()                                         |                        |
| fpixDestroy            | 🔄   | drop()                                                | 自動                   |
| fpixGetDimensions      | ✅   | width()/height()                                      |                        |
| fpixSetDimensions      | 🚫   | -                                                     | FPixは不変             |
| fpixGetWpl             | 🚫   | -                                                     | FPixはwpl概念なし      |
| fpixSetWpl             | 🚫   | -                                                     | FPixはwpl概念なし      |
| fpixGetResolution      | ✅   | xres()/yres()                                         |                        |
| fpixSetResolution      | ✅   | FPix::set_resolution()                                |                        |
| fpixCopyResolution     | 🚫   | -                                                     | set_resolution()で対応 |
| fpixGetData            | ✅   | FPix::data()                                          |                        |
| fpixSetData            | 🚫   | -                                                     | Cメモリ管理            |
| fpixGetPixel           | ✅   | FPix::get_pixel()                                     |                        |
| fpixSetPixel           | ✅   | FPix::set_pixel()                                     |                        |
| fpixaDestroy           | 🚫   | -                                                     | drop()で自動           |
| dpixCreate             | ✅   | DPix::new()                                           |                        |
| dpixClone              | ✅   | DPix::clone()                                         |                        |
| dpixCopy               | ✅   | DPix::clone()                                         |                        |
| dpixDestroy            | 🔄   | drop()                                                | 自動                   |
| dpixRead               | ✅   | DPix::read_from_file/read_from_reader/read_from_bytes |                        |
| dpixWrite              | ✅   | DPix::write_to_file/write_to_writer/write_to_bytes    |                        |
| fpixaCreate            | ✅   | FPixa::new()                                          |                        |
| fpixaCopy              | ✅   | FPixa::clone()                                        |                        |
| fpixaAddFPix           | ✅   | FPixa::push()                                         |                        |
| fpixaGetCount          | ✅   | FPixa::len()                                          |                        |
| fpixaGetFPix           | ✅   | FPixa::get()                                          |                        |
| fpixaGetFPixDimensions | ✅   | FPixa::get_dimensions()                               |                        |
| fpixaGetData           | ✅   | FPixa::get_data()                                     |                        |
| fpixaGetPixel          | ✅   | FPixa::get_pixel()                                    |                        |
| fpixaSetPixel          | ✅   | FPixa::set_pixel()                                    |                        |
| fpixConvertToPix       | ✅   | FPix::to_pix()                                        |                        |
| pixConvertToFPix       | ✅   | FPix::from_pix()                                      |                        |
| fpixAddMultConstant    | 🔄   | FPix::add_constant() + FPix::mul_constant()           | 2段階呼び出し          |
| fpixLinearCombination  | ✅   | FPix::linear_combination()                            |                        |
| dpixConvertToPix       | ✅   | DPix::to_pix()                                        |                        |
| dpixConvertToFPix      | ✅   | DPix::to_fpix()                                       |                        |

#### core/fpix/serial.rs (fpix1.c, fpix2.c)

| C関数           | 状態 | Rust対応               | 備考 |
| --------------- | ---- | ---------------------- | ---- |
| fpixRead        | ✅   | FPix::read_from_file   |      |
| fpixReadStream  | ✅   | FPix::read_from_reader |      |
| fpixReadMem     | ✅   | FPix::read_from_bytes  |      |
| fpixWrite       | ✅   | FPix::write_to_file    |      |
| fpixWriteStream | ✅   | FPix::write_to_writer  |      |
| fpixWriteMem    | ✅   | FPix::write_to_bytes   |      |

その他のfpix2.c変換関数は一部convert.rsに実装あり。

### colormap.c (カラーマップ)

#### core/colormap/mod.rs (colormap.c)

| C関数                  | 状態 | Rust対応                        | 備考                |
| ---------------------- | ---- | ------------------------------- | ------------------- |
| pixcmapCreate          | ✅   | PixColormap::new()              |                     |
| pixcmapCreateLinear    | ✅   | PixColormap::create_linear()    |                     |
| pixcmapAddColor        | ✅   | PixColormap::add_color()        |                     |
| pixcmapAddRGBA         | ✅   | PixColormap::add_rgba           | add_colorがRGBA対応 |
| pixcmapGetCount        | ✅   | PixColormap::len()              |                     |
| pixcmapGetDepth        | ✅   | PixColormap::depth()            |                     |
| pixcmapGetColor        | ✅   | PixColormap::get_rgb()          |                     |
| pixcmapGetRGBA         | ✅   | PixColormap::get_rgba           |                     |
| pixcmapHasColor        | ✅   | PixColormap::has_color          |                     |
| pixcmapIsOpaque        | ✅   | PixColormap::is_opaque          |                     |
| pixcmapIsBlackAndWhite | ✅   | PixColormap::is_black_and_white |                     |
| pixcmapGetNearestIndex | ✅   | PixColormap::find_nearest       |                     |

#### core/colormap/query.rs (colormap.c)

| C関数                      | 状態 | Rust対応                         | 備考 |
| -------------------------- | ---- | -------------------------------- | ---- |
| pixcmapCreateRandom        | ✅   | PixColormap::create_random       |      |
| pixcmapCopy                | ✅   | PixColormap::clone()             |      |
| pixcmapDestroy             | 🔄   | drop()                           | 自動 |
| pixcmapIsValid             | ✅   | PixColormap::is_valid            |      |
| pixcmapAddNewColor         | ✅   | PixColormap::add_new_color       |      |
| pixcmapAddNearestColor     | ✅   | PixColormap::add_nearest_color   |      |
| pixcmapUsableColor         | ✅   | PixColormap::is_usable_color     |      |
| pixcmapAddBlackOrWhite     | ✅   | PixColormap::add_black_or_white  |      |
| pixcmapSetBlackAndWhite    | ✅   | PixColormap::set_black_and_white |      |
| pixcmapGetFreeCount        | ✅   | PixColormap::free_count          |      |
| pixcmapGetMinDepth         | ✅   | PixColormap::min_depth           |      |
| pixcmapClear               | ✅   | PixColormap::clear()             |      |
| pixcmapGetColor32          | ✅   | PixColormap::get_color32         |      |
| pixcmapGetRGBA32           | ✅   | PixColormap::get_rgba32          |      |
| pixcmapResetColor          | ✅   | PixColormap::reset_color         |      |
| pixcmapSetAlpha            | ✅   | PixColormap::set_alpha           |      |
| pixcmapGetIndex            | ✅   | PixColormap::get_index           |      |
| pixcmapNonOpaqueColorsInfo | ✅   | PixColormap::non_opaque_info     |      |
| pixcmapCountGrayColors     | ✅   | PixColormap::count_gray_colors   |      |
| pixcmapGetRankIntensity    | ✅   | PixColormap::get_rank_intensity  |      |
| pixcmapGetNearestGrayIndex | ✅   | PixColormap::find_nearest_gray   |      |
| pixcmapGetDistanceToColor  | ✅   | PixColormap::distance_to_color   |      |
| pixcmapGetRangeValues      | ✅   | PixColormap::get_range_values    |      |
| pixcmapConvertTo4          | ✅   | PixColormap::convert_to_4()      |      |
| pixcmapConvertTo8          | ✅   | PixColormap::convert_to_8()      |      |

#### core/colormap/convert.rs (colormap.c)

| C関数                        | 状態 | Rust対応                             | 備考 |
| ---------------------------- | ---- | ------------------------------------ | ---- |
| pixcmapGrayToFalseColor      | ✅   | PixColormap::gray_to_false_color     |      |
| pixcmapGrayToColor           | ✅   | PixColormap::gray_to_color           |      |
| pixcmapColorToGray           | ✅   | PixColormap::color_to_gray           |      |
| pixcmapToArrays              | ✅   | PixColormap::to_arrays               |      |
| pixcmapToRGBTable            | ✅   | PixColormap::to_rgb_table            |      |
| pixcmapSerializeToMemory     | ✅   | PixColormap::serialize_to_memory     |      |
| pixcmapDeserializeFromMemory | ✅   | PixColormap::deserialize_from_memory |      |
| pixcmapConvertToHex          | ✅   | PixColormap::convert_to_hex          |      |
| pixcmapGammaTRC              | ✅   | PixColormap::gamma_trc               |      |
| pixcmapContrastTRC           | ✅   | PixColormap::contrast_trc            |      |
| pixcmapShiftIntensity        | ✅   | PixColormap::shift_intensity         |      |
| pixcmapShiftByComponent      | ✅   | PixColormap::shift_by_component      |      |

#### core/colormap/serial.rs (colormap.c)

| C関数              | 状態 | Rust対応                      | 備考 |
| ------------------ | ---- | ----------------------------- | ---- |
| pixcmapRead        | ✅   | PixColormap::read_from_file   |      |
| pixcmapReadStream  | ✅   | PixColormap::read_from_reader |      |
| pixcmapReadMem     | ✅   | PixColormap::read_from_bytes  |      |
| pixcmapWrite       | ✅   | PixColormap::write_to_file    |      |
| pixcmapWriteStream | ✅   | PixColormap::write_to_writer  |      |
| pixcmapWriteMem    | ✅   | PixColormap::write_to_bytes   |      |

### pixconv.c (ピクセル深度変換)

convert.rsに実装済み。全関数が実装されている。

#### core/pix/convert.rs (pixconv.c)

| C関数                        | 状態 | Rust対応                                             | 備考                                    |
| ---------------------------- | ---- | ---------------------------------------------------- | --------------------------------------- |
| pixThreshold8                | ✅   | Pix::threshold_8                                     |                                         |
| pixConvertRGBToBinaryArb     | ✅   | Pix::convert_rgb_to_binary_arb                       | color crate依存                         |
| pixConvertRGBToColormap      | ✅   | Pix::convert_rgb_to_colormap                         | color crate依存                         |
| pixQuantizeIfFewColors       | ✅   | Pix::quantize_if_few_colors                          | color crate依存                         |
| pixConvertTo1Adaptive        | ✅   | Pix::convert_to_1_adaptive()                         |                                         |
| pixConvertTo1                | 🔄   | convert_to_1_adaptive() / convert_to_1_by_sampling() | 汎用ディスパッチャを2つの専用関数に分割 |
| pixConvertTo1BySampling      | ✅   | Pix::convert_to_1_by_sampling()                      |                                         |
| pixConvertTo8BySampling      | ✅   | Pix::convert_to_8_by_sampling()                      |                                         |
| pixConvertTo8Colormap        | ✅   | Pix::convert_to_8_colormap()                         |                                         |
| pixConvertTo32BySampling     | ✅   | Pix::convert_to_32_by_sampling()                     |                                         |
| pixConvert24To32             | ✅   | Pix::convert_24_to_32()                              |                                         |
| pixConvert32To24             | ✅   | Pix::convert_32_to_24()                              |                                         |
| pixConvertToSubpixelRGB      | ✅   | Pix::convert_to_subpixel_rgb                         |                                         |
| pixConvertGrayToSubpixelRGB  | ✅   | Pix::convert_gray_to_subpixel_rgb                    |                                         |
| pixConvertColorToSubpixelRGB | ✅   | Pix::convert_color_to_subpixel_rgb                   |                                         |
| pixRemoveColormapGeneral     | ✅   | Pix::remove_colormap()                               | pixRemoveColormapと同一メソッド         |
| pixRemoveColormap            | ✅   | Pix::remove_colormap()                               |                                         |
| pixAddGrayColormap8          | ✅   | Pix::add_gray_colormap_8()                           |                                         |
| pixAddMinimalGrayColormap8   | ✅   | Pix::add_minimal_gray_colormap_8()                   |                                         |
| pixConvertRGBToLuminance     | ✅   | Pix::convert_rgb_to_luminance()                      |                                         |
| pixConvertRGBToGrayGeneral   | ✅   | Pix::convert_rgb_to_gray_general()                   |                                         |
| pixConvertRGBToGray          | ✅   | Pix::convert_rgb_to_gray()                           |                                         |
| pixConvertRGBToGrayFast      | ✅   | Pix::convert_rgb_to_gray_fast()                      |                                         |
| pixConvertRGBToGrayMinMax    | ✅   | Pix::convert_rgb_to_gray_min_max()                   |                                         |
| pixConvertRGBToGraySatBoost  | ✅   | Pix::convert_rgb_to_gray_sat_boost()                 |                                         |
| pixConvertRGBToGrayArb       | ✅   | Pix::convert_rgb_to_gray_arb()                       |                                         |
| pixConvertGrayToColormap     | ✅   | Pix::convert_gray_to_colormap()                      |                                         |
| pixConvertGrayToColormap8    | ✅   | Pix::convert_gray_to_colormap_8()                    |                                         |
| pixColorizeGray              | ✅   | Pix::colorize_gray()                                 |                                         |
| pixConvertCmapTo1            | ✅   | Pix::convert_cmap_to_1()                             |                                         |
| pixConvert16To8              | ✅   | Pix::convert_16_to_8()                               |                                         |
| pixConvertGrayToFalseColor   | ✅   | Pix::convert_gray_to_false_color()                   |                                         |
| pixUnpackBinary              | ✅   | Pix::unpack_binary()                                 |                                         |
| pixConvert1To16              | ✅   | Pix::convert_1_to_16()                               |                                         |
| pixConvert1To32              | ✅   | Pix::convert_1_to_32()                               |                                         |
| pixConvert1To2Cmap           | ✅   | Pix::convert_1_to_2_cmap()                           |                                         |
| pixConvert1To2               | ✅   | Pix::convert_1_to_2()                                |                                         |
| pixConvert1To4Cmap           | ✅   | Pix::convert_1_to_4_cmap()                           |                                         |
| pixConvert1To4               | ✅   | Pix::convert_1_to_4()                                |                                         |
| pixConvert1To8Cmap           | ✅   | Pix::convert_1_to_8_cmap()                           |                                         |
| pixConvert1To8               | ✅   | Pix::convert_1_to_8()                                |                                         |
| pixConvert2To8               | ✅   | Pix::convert_2_to_8()                                |                                         |
| pixConvert4To8               | ✅   | Pix::convert_4_to_8()                                |                                         |
| pixConvert8To16              | ✅   | Pix::convert_8_to_16()                               |                                         |
| pixConvertTo2                | ✅   | Pix::convert_to_2()                                  |                                         |
| pixConvert8To2               | ✅   | Pix::convert_8_to_2()                                |                                         |
| pixConvert8To4               | ✅   | Pix::convert_8_to_4()                                |                                         |
| pixConvertTo16               | ✅   | Pix::convert_to_16()                                 |                                         |
| pixConvertTo32               | ✅   | Pix::convert_to_32()                                 |                                         |
| pixConvert8To32              | ✅   | Pix::convert_8_to_32()                               |                                         |
| pixConvertTo8Or32            | ✅   | Pix::convert_to_8_or_32()                            |                                         |
| pixConvert32To16             | ✅   | Pix::convert_32_to_16()                              |                                         |
| pixConvert32To8              | ✅   | Pix::convert_32_to_8()                               |                                         |
| pixRemoveAlpha               | ✅   | Pix::remove_alpha()                                  |                                         |
| pixAddAlphaTo1bpp            | ✅   | Pix::add_alpha_to_1bpp()                             |                                         |
| pixConvertLossless           | ✅   | Pix::convert_lossless()                              |                                         |
| pixConvertForPSWrap          | ✅   | Pix::convert_for_ps_wrap()                           |                                         |
| pixConvertTo4                | ✅   | Pix::convert_to_4()                                  |                                         |
| pixConvertTo8                | ✅   | Pix::convert_to_8()                                  |                                         |

### pixarith.c (ピクセル算術演算)

#### core/pix/arith.rs (pixarith.c)

| C関数                  | 状態 | Rust対応                            | 備考      |
| ---------------------- | ---- | ----------------------------------- | --------- |
| pixAddGray             | ✅   | Pix::arith_add()                    |           |
| pixSubtractGray        | ✅   | Pix::arith_subtract()               |           |
| pixMultConstantGray    | ✅   | Pix::multiply_constant()            |           |
| pixAddConstantGray     | ✅   | Pix::add_constant()                 |           |
| pixMultConstAccumulate | ✅   | PixMut::mult_const_accumulate()     | 32bpp専用 |
| pixAbsDifference       | ✅   | abs_difference()                    |           |
| pixMinOrMax            | 🔄   | Pix::arith_min() / Pix::arith_max() |           |

その他のpixarith.c関数も実装済み。

### rop.c, roplow.c (ラスターオペレーション)

#### core/pix/rop.rs (rop.c, roplow.c)

| C関数                | 状態 | Rust対応                 | 備考 |
| -------------------- | ---- | ------------------------ | ---- |
| pixRasterop          | ✅   | Pix::rasterop_full_image |      |
| pixRasteropIP        | ✅   | Pix::rasterop_ip         |      |
| pixRasteropFullImage | ✅   | Pix::rasterop_full_image |      |
| pixRasteropVip       | ✅   | PixMut::rasterop_vip()   |      |
| pixRasteropHip       | ✅   | PixMut::rasterop_hip()   |      |
| pixTranslate         | ✅   | Pix::translate()         |      |

roplow.c (低レベルラスターOP) 全関数 🚫 不要 (高レベルrop.rs APIでカバー済み)

### compare.c (画像比較)

#### core/pix/compare.rs (compare.c)

| C関数                     | 状態 | Rust対応                        | 備考        |
| ------------------------- | ---- | ------------------------------- | ----------- |
| pixEqual                  | ✅   | Pix::equals()                   |             |
| pixCorrelationBinary      | ✅   | compare::correlation_binary()   |             |
| pixCompareBinary          | 🔄   | Pix::compare()                  | 統合比較API |
| pixCompareTiled           | ✅   | Pix::compare_tiled              |             |
| pixGetPerceptualDiff      | ✅   | Pix::get_perceptual_diff        |             |
| pixEqualWithAlpha         | ✅   | Pix::equals_with_alpha()        |             |
| pixEqualWithCmap          | ✅   | Pix::equals_with_cmap()         |             |
| pixDisplayDiff            | ✅   | Pix::display_diff()             |             |
| pixDisplayDiffBinary      | ✅   | Pix::display_diff_binary()      |             |
| pixCompareGrayOrRGB       | ✅   | Pix::compare_gray_or_rgb()      |             |
| pixCompareGray            | ✅   | Pix::compare_gray()             |             |
| pixCompareRGB             | ✅   | Pix::compare_rgb()              |             |
| pixCompareRankDifference  | ✅   | Pix::compare_rank_difference()  |             |
| pixTestForSimilarity      | ✅   | Pix::test_for_similarity()      |             |
| pixGetDifferenceStats     | ✅   | Pix::get_difference_stats()     |             |
| pixGetDifferenceHistogram | ✅   | Pix::get_difference_histogram() |             |
| pixGetPSNR                | ✅   | Pix::get_psnr()                 |             |

その他の比較関数も実装済み。

### blend.c (ブレンド・合成)

#### core/pix/blend.rs (blend.c)

| C関数                     | 状態 | Rust対応                       | 備考 |
| ------------------------- | ---- | ------------------------------ | ---- |
| pixBlend                  | ✅   | Pix::blend()                   |      |
| pixBlendMask              | ✅   | blend::blend_mask()            |      |
| pixBlendGray              | ✅   | blend::blend_gray()            |      |
| pixBlendColor             | ✅   | blend::blend_color()           |      |
| pixBlendWithGrayMask      | ✅   | blend::blend_with_gray_mask()  |      |
| pixBlendBackgroundToColor | ✅   | Pix::blend_background_to_color |      |
| pixSetAlphaOverWhite      | ✅   | Pix::set_alpha_over_white      |      |
| pixBlendGrayInverse       | ✅   | Pix::blend_gray_inverse()      |      |
| pixBlendColorByChannel    | ✅   | Pix::blend_color_by_channel()  |      |
| pixBlendGrayAdapt         | ✅   | Pix::blend_gray_adapt()        |      |
| pixFadeWithGray           | ✅   | Pix::fade_with_gray()          |      |
| pixBlendHardLight         | ✅   | Pix::blend_hard_light()        |      |
| pixBlendCmap              | ✅   | PixMut::blend_cmap()           |      |
| pixMultiplyByColor        | ✅   | Pix::multiply_by_color()       |      |
| pixAlphaBlendUniform      | ✅   | Pix::alpha_blend_uniform()     |      |
| pixAddAlphaToBlend        | ✅   | Pix::add_alpha_to_blend()      |      |
| pixLinearEdgeFade         | ✅   | PixMut::linear_edge_fade()     |      |

### graphics.c (描画・レンダリング)

#### core/pix/graphics.rs (graphics.c)

| C関数                    | 状態 | Rust対応                            | 備考      |
| ------------------------ | ---- | ----------------------------------- | --------- |
| generatePtaLine          | ✅   | generate_line_pta()                 |           |
| generatePtaWideLine      | ✅   | generate_wide_line_pta()            |           |
| generatePtaBox           | ✅   | generate_box_pta()                  |           |
| generatePtaBoxa          | ✅   | generate_boxa_pta()                 |           |
| generatePtaHashBox       | ✅   | generate_hash_box_pta()             |           |
| generatePtaHashBoxa      | ✅   | generate_hash_boxa_pta()            |           |
| generatePtaaBoxa         | ✅   | generate_ptaa_boxa()                |           |
| generatePtaaHashBoxa     | ✅   | generate_ptaa_hash_boxa()           |           |
| generatePtaPolyline      | ✅   | generate_polyline_pta()             |           |
| generatePtaGrid          | ✅   | generate_grid_pta()                 |           |
| convertPtaLineTo4cc      | ✅   | convert_line_to_4cc()               |           |
| generatePtaFilledCircle  | ✅   | generate_filled_circle_pta()        |           |
| generatePtaFilledSquare  | ✅   | generate_filled_square_pta()        |           |
| pixRenderPlotFromNuma    | ✅   | PixMut::render_plot_from_numa()     |           |
| pixRenderPlotFromNumaGen | ✅   | PixMut::render_plot_from_numa_gen() |           |
| pixRenderPtaArb          | ✅   | PixMut::render_pta_color()          |           |
| pixRenderPtaBlend        | ✅   | PixMut::render_pta_blend()          |           |
| pixRenderLineArb         | ✅   | PixMut::render_line_color()         |           |
| pixRenderLineBlend       | ✅   | PixMut::render_line_blend()         |           |
| pixRenderBoxArb          | ✅   | PixMut::render_box_color()          |           |
| pixRenderBoxBlend        | ✅   | PixMut::render_box_blend()          |           |
| pixRenderBoxa            | ✅   | PixMut::render_boxa()               |           |
| pixRenderBoxaArb         | ✅   | PixMut::render_boxa_color()         |           |
| pixRenderBoxaBlend       | ✅   | PixMut::render_boxa_blend()         |           |
| pixRenderHashBox         | ✅   | PixMut::render_hash_box()           |           |
| pixRenderHashBoxArb      | ✅   | PixMut::render_hash_box_color()     |           |
| pixRenderHashBoxBlend    | ✅   | PixMut::render_hash_box_blend()     |           |
| pixRenderHashMaskArb     | ✅   | PixMut::render_hash_mask_color()    |           |
| pixRenderHashBoxa        | ✅   | PixMut::render_hash_boxa()          |           |
| pixRenderHashBoxaArb     | ✅   | PixMut::render_hash_boxa_color()    |           |
| pixRenderHashBoxaBlend   | ✅   | PixMut::render_hash_boxa_blend()    |           |
| pixRenderPolyline        | ✅   | PixMut::render_polyline()           |           |
| pixRenderPolylineArb     | ✅   | PixMut::render_polyline_color()     |           |
| pixRenderPolylineBlend   | ✅   | PixMut::render_polyline_blend()     |           |
| pixRenderGridArb         | ✅   | PixMut::render_grid_color()         |           |
| pixRenderRandomCmapPtaa  | ✅   | Pix::render_random_cmap_ptaa()      |           |
| pixRenderPolygon         | ✅   | render_polygon()                    |           |
| pixFillPolygon           | ✅   | fill_polygon()                      |           |
| pixRenderContours        | ✅   | Pix::render_contours()              |           |
| pixRenderPta             | ✅   | graphics.rsに部分実装               |           |
| pixRenderLine            | ✅   | graphics::render_line()             |           |
| pixRenderBox             | ✅   | graphics::render_box()              |           |
| pixGeneratePtaBoundary   | ✅   | Pix::generate_pta_boundary          | 後続Phase |

#### core/fpix/mod.rs (graphics.c)

| C関数                  | 状態 | Rust対応                   | 備考           |
| ---------------------- | ---- | -------------------------- | -------------- |
| fpixAutoRenderContours | ✅   | FPix::auto_render_contours | FPix関連は後続 |
| fpixRenderContours     | ✅   | FPix::render_contours      | FPix関連は後続 |

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

gap-fill audit (2026-05-10) で C 公開関数を全数突き合わせた結果、本ファイル
末尾の「追加検証エントリ」で **128 件**の ❌ エントリを検出。主な領域:

- pixafunc1.c の Pixa selection / transform 拡張 (約 39 件)
- numafunc2.c の高度な Numa ヒストグラム/分布関数 (23 件)
- pixafunc2.c の Pixa display/conversion (11 件)
- ptafunc1.c の Pta + graphics ヘルパー (15 件)
- compare.c の photo region 比較 (11 件)
- fpix2.c の FPix 幾何変換 (12 件)

実装ロードマップは `docs/plans/032_gap-fill-roadmap-v2.md` を参照。

## 追加検証エントリ (gap-fill audit 2026-05-10)

以下は当初 `verify-comparison-counts` では捕捉されていなかった C 公開関数の追加分類。
当初のヒューリスティック検索結果を、C 関数名と Rust 実装の場所・シグネチャで個別レビュー
して再分類した結果である。

- ✅ 同等: Rust 側に同名・同モジュールの実装を確認
- 🔄 異なる: Rust 側で異なる API/モジュール配置で実装 (Vec idiomatic 等)
- 🚫 不要: Rust 標準ライブラリ等で代替
- ❌ 未実装: 当該機能が Rust 側に存在しない

**追加分類サマリー**: ✅ 52 / 🔄 47 / 🚫 69 / ❌ 128 (合計 296)

### arrayaccess.c (追加分)

| C関数              | 状態 | Rust対応                                   | 備考              |
| ------------------ | ---- | ------------------------------------------ | ----------------- |
| l_clearDataBit     | ✅   | `clear_data_bit` (core/pix/access.rs)      | name+module match |
| l_clearDataDibit   | ✅   | `clear_data_dibit` (core/pix/access.rs)    | plan 113          |
| l_clearDataQbit    | ✅   | `clear_data_qbit` (core/pix/access.rs)     | plan 113          |
| l_getDataBit       | ✅   | `get_data_bit` (core/pix/access.rs)        | name+module match |
| l_getDataByte      | ✅   | `get_data_byte` (core/pix/access.rs)       | name+module match |
| l_getDataDibit     | ✅   | `get_data_dibit` (core/pix/access.rs)      | name+module match |
| l_getDataFourBytes | ✅   | `get_data_four_bytes` (core/pix/access.rs) | plan 113          |
| l_getDataQbit      | ✅   | `get_data_qbit` (core/pix/access.rs)       | name+module match |
| l_getDataTwoBytes  | ✅   | `get_data_two_bytes` (core/pix/access.rs)  | name+module match |
| l_setDataBit       | ✅   | `set_data_bit` (core/pix/access.rs)        | name+module match |
| l_setDataBitVal    | ✅   | `set_data_bit_val` (core/pix/access.rs)    | name+module match |
| l_setDataByte      | ✅   | `set_data_byte` (core/pix/access.rs)       | name+module match |
| l_setDataDibit     | ✅   | `set_data_dibit` (core/pix/access.rs)      | name+module match |
| l_setDataFourBytes | ✅   | `set_data_four_bytes` (core/pix/access.rs) | plan 113          |
| l_setDataQbit      | ✅   | `set_data_qbit` (core/pix/access.rs)       | name+module match |
| l_setDataTwoBytes  | ✅   | `set_data_two_bytes` (core/pix/access.rs)  | name+module match |

### compare.c (追加分)

| C関数                          | 状態 | Rust対応                                             | 備考                          |
| ------------------------------ | ---- | ---------------------------------------------------- | ----------------------------- |
| cmapEqual                      | ✅   | `PixColormap::equal_to` (core/colormap/mod.rs)       | plan 112                      |
| compareTilesByHisto            | ❌   | -                                                    | no Rust impl in expected dirs |
| pixBestCorrelation             | ✅   | `best_correlation` (core/pix/compare.rs)             | name+module match             |
| pixCentroid8                   | ✅   | `Pix::centroid8` (core/pix/compare.rs)               | plan 112                      |
| pixCompareGrayByHisto          | ❌   | -                                                    | no Rust impl in expected dirs |
| pixComparePhotoRegionsByHisto  | ❌   | -                                                    | no Rust impl in expected dirs |
| pixCompareWithTranslation      | ✅   | `compare_with_translation` (core/pix/compare.rs)     | name+module match             |
| pixCropAlignedToCentroid       | ✅   | `pix_crop_aligned_to_centroid` (core/pix/compare.rs) | plan 112                      |
| pixDecideIfPhotoImage          | ❌   | -                                                    | no Rust impl in expected dirs |
| pixGenPhotoHistos              | ❌   | -                                                    | no Rust impl in expected dirs |
| pixPadToCenterCentroid         | ✅   | `Pix::pad_to_center_centroid` (core/pix/compare.rs)  | plan 112                      |
| pixUsesCmapColor               | ✅   | `Pix::uses_cmap_color` (core/pix/compare.rs)         | plan 112                      |
| pixaComparePhotoRegionsByHisto | ❌   | -                                                    | no Rust impl in expected dirs |

### fpix1.c (追加分)

| C関数              | 状態 | Rust対応 | 備考                                           |
| ------------------ | ---- | -------- | ---------------------------------------------- |
| dpixCopyResolution | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixCreateTemplate | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixEndianByteSwap | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixGetData        | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixGetDimensions  | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixGetPixel       | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixGetResolution  | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixGetWpl         | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixReadMem        | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixReadStream     | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixSetData        | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixSetDimensions  | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixSetPixel       | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixSetResolution  | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixSetWpl         | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixWriteMem       | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| dpixWriteStream    | 🚫   | -        | DPix (倍精度画像) は Rust 未提供 - FPix で代替 |
| fpixEndianByteSwap | 🚫   | -        | Rust 標準 (i32::swap_bytes 等) で代替          |
| fpixPrintStream    | 🚫   | -        | デバッグ出力                                   |

### fpix2.c (追加分)

| C関数                       | 状態 | Rust対応                                                 | 備考                                                      |
| --------------------------- | ---- | -------------------------------------------------------- | --------------------------------------------------------- |
| dpixAddMultConstant         | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| dpixGetMax                  | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| dpixGetMin                  | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| dpixLinearCombination       | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| dpixScaleByInteger          | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| dpixSetAllArbitrary         | 🚫   | -                                                        | DPix (倍精度画像) は Rust 未提供 - FPix で代替            |
| fpixAddBorder               | ✅   | `add_border` (core/fpix/transform.rs)                    | name+module match                                         |
| fpixAddContinuedBorder      | ✅   | `add_continued_border` (core/fpix/transform.rs)          | name+module match                                         |
| fpixAddMirroredBorder       | ✅   | `add_mirrored_border` (core/fpix/transform.rs)           | name+module match                                         |
| fpixAddSlopeBorder          | 🚫   | -                                                        | DPix 未実装方針                                           |
| fpixAffine                  | ❌   | -                                                        | no Rust impl in expected dirs                             |
| fpixAffinePta               | ❌   | -                                                        | no Rust impl in expected dirs                             |
| fpixConvertToDPix           | 🚫   | -                                                        | DPix 未実装方針                                           |
| fpixDisplayMaxDynamicRange  | 🚫   | -                                                        | DPix 未実装方針                                           |
| fpixFlipLR                  | ✅   | `flip_lr` (core/fpix/transform.rs)                       | name+module match                                         |
| fpixFlipTB                  | ✅   | `flip_tb` (core/fpix/transform.rs)                       | name+module match                                         |
| fpixGetMax                  | ✅   | `FPix::get_max` (core/fpix/extended.rs)                  | plan 110                                                  |
| fpixGetMin                  | ✅   | `FPix::get_min` (core/fpix/extended.rs)                  | plan 110                                                  |
| fpixProjective              | ❌   | -                                                        | no Rust impl in expected dirs                             |
| fpixProjectivePta           | ❌   | -                                                        | no Rust impl in expected dirs                             |
| fpixRasterop                | ✅   | `FPix::rasterop` (core/fpix/extended.rs)                 | plan 110                                                  |
| fpixRemoveBorder            | ✅   | `FPix::remove_border` (core/fpix/extended.rs)            | plan 110                                                  |
| fpixRotate180               | ✅   | `rotate_180` (core/fpix/transform.rs)                    | name+module match                                         |
| fpixRotate90                | ✅   | `rotate_90` (core/fpix/transform.rs)                     | name+module match                                         |
| fpixRotateOrth              | ✅   | `rotate_orth` (core/fpix/transform.rs)                   | name+module match                                         |
| fpixScaleByInteger          | ✅   | `FPix::scale_by_integer` (core/fpix/extended.rs)         | plan 110                                                  |
| fpixSetAllArbitrary         | 🔄   | `(idiomatic)`                                            | FPix::set_all_arbitrary に類似機能あり (Pix 用は実装済み) |
| fpixThresholdToPix          | ✅   | `FPix::threshold_to_pix` (core/fpix/extended.rs)         | plan 110                                                  |
| linearInterpolatePixelFloat | ✅   | `linear_interpolate_pixel_float` (core/fpix/extended.rs) | plan 110                                                  |
| pixComponentFunction        | ❌   | -                                                        | no Rust impl in expected dirs                             |
| pixConvertToDPix            | 🚫   | -                                                        | DPix 未実装方針                                           |

### graphics.c (追加分)

| C関数                  | 状態 | Rust対応                                             | 備考              |
| ---------------------- | ---- | ---------------------------------------------------- | ----------------- |
| generatePtaLineFromPt  | ✅   | `generate_pta_line_from_pt` (core/pix/graphics.rs)   | plan 115          |
| locatePtRadially       | ✅   | `locate_pt_radially` (core/pix/graphics.rs)          | plan 115          |
| makePlotPtaFromNuma    | ✅   | `make_plot_pta_from_numa` (core/pix/graphics.rs)     | plan 115          |
| makePlotPtaFromNumaGen | ✅   | `make_plot_pta_from_numa_gen` (core/pix/graphics.rs) | name+module match |

### numabasic.c (追加分)

| C関数                | 状態 | Rust対応                                 | 備考                                                                    |
| -------------------- | ---- | ---------------------------------------- | ----------------------------------------------------------------------- |
| numaAddNumber        | 🔄   | `push` (core/numa/mod.rs)                | C: numaAddNumber -> Rust: Numa::push() (idiomatic)                      |
| numaClone            | 🔄   | `(idiomatic)`                            | Rust の Clone trait で代替                                              |
| numaConvertToSarray  | ✅   | `convert_to_sarray` (core/numa/mod.rs)   | plan 116                                                                |
| numaCopy             | 🔄   | `(idiomatic)`                            | Rust の Clone trait で代替                                              |
| numaCopyParameters   | ✅   | `copy_parameters` (core/numa/mod.rs)     | plan 116                                                                |
| numaCreate           | 🔄   | `new` (core/numa/mod.rs)                 | C: numaCreate -> Rust: Numa::new()/with_capacity()                      |
| numaCreateFromFArray | 🔄   | `from_slice` (core/numa/mod.rs)          | C: numaCreateFromFArray -> Rust: Numa::from_slice()/from_vec()          |
| numaCreateFromIArray | 🔄   | `from_i32_slice` (core/numa/mod.rs)      | C: numaCreateFromIArray -> Rust: Numa::from_i32_slice()                 |
| numaCreateFromString | ✅   | `create_from_string` (core/numa/mod.rs)  | plan 116                                                                |
| numaDestroy          | 🚫   | -                                        | Drop trait で自動破棄                                                   |
| numaEmpty            | 🔄   | `clear` (core/numa/mod.rs)               | Numa::clear()/is_empty()                                                |
| numaGetCount         | 🔄   | `len` (core/numa/mod.rs)                 | Numa::len()                                                             |
| numaGetFArray        | 🔄   | `as_slice` (core/numa/mod.rs)            | Numa::as_slice()/into_vec()                                             |
| numaGetFValue        | 🔄   | `get` (core/numa/mod.rs)                 | Numa::get()                                                             |
| numaGetIArray        | 🔄   | `(idiomatic)`                            | `iter().map(\                                                           |
| numaGetIValue        | 🔄   | `get_i32` (core/numa/mod.rs)             | Numa::get_i32()                                                         |
| numaGetParameters    | 🔄   | `parameters` (core/numa/mod.rs)          | Numa::parameters()                                                      |
| numaInsertNumber     | 🔄   | `insert` (core/numa/mod.rs)              | Numa::insert()                                                          |
| numaRead             | 🔄   | `read_from_file` (core/numa/serial.rs)   | Numa::read_from_file()                                                  |
| numaReadMem          | 🔄   | `read_from_bytes` (core/numa/serial.rs)  | Numa::read_from_bytes()                                                 |
| numaReadStream       | 🔄   | `read_from_reader` (core/numa/serial.rs) | Numa::read_from_reader()                                                |
| numaRemoveNumber     | 🔄   | `remove` (core/numa/mod.rs)              | Numa::remove()                                                          |
| numaReplaceNumber    | 🔄   | `replace` (core/numa/mod.rs)             | Numa::replace()                                                         |
| numaSetCount         | 🔄   | `(idiomatic)`                            | Vec::truncate/resize で代替                                             |
| numaSetParameters    | ✅   | `set_parameters` (core/numa/mod.rs)      | name+module match                                                       |
| numaSetValue         | 🔄   | `set` (core/numa/mod.rs)                 | Numa::set()                                                             |
| numaShiftValue       | 🔄   | `shift` (core/numa/mod.rs)               | Numa::shift()                                                           |
| numaWrite            | 🔄   | `write_to_file` (core/numa/serial.rs)    | Numa::write_to_file()                                                   |
| numaWriteDebug       | 🚫   | -                                        | デバッグ出力                                                            |
| numaWriteMem         | 🔄   | `write_to_bytes` (core/numa/serial.rs)   | Numa::write_to_bytes()                                                  |
| numaWriteStderr      | 🚫   | -                                        | デバッグ出力                                                            |
| numaWriteStream      | 🔄   | `write_to_writer` (core/numa/serial.rs)  | Numa::write_to_writer()                                                 |
| numaaAddNuma         | 🔄   | `(idiomatic)`                            | Numaa::push() で代替                                                    |
| numaaAddNumber       | 🔄   | `(idiomatic)`                            | Numaa の特定 Numa に値追加 -> 個別に Numaa::get_mut + Numa::push で代替 |
| numaaCreate          | 🔄   | `new` (core/numa/mod.rs)                 | Numaa::new()                                                            |
| numaaCreateFull      | ✅   | `create_full` (core/numa/mod.rs)         | plan 116                                                                |
| numaaDestroy         | 🚫   | -                                        | Drop で自動破棄                                                         |
| numaaGetCount        | 🔄   | `len` (core/numa/mod.rs)                 | Numaa::len()                                                            |
| numaaGetNuma         | 🔄   | `(idiomatic)`                            | Numaa::get() で代替                                                     |
| numaaGetNumaCount    | 🔄   | `(idiomatic)`                            | Numaa::len() で代替                                                     |
| numaaGetNumberCount  | 🔄   | `total_count` (core/numa/mod.rs)         | Numaa::total_count() で代替                                             |
| numaaGetPtrArray     | 🚫   | -                                        | C ポインタ配列 - Rust では as_slice() で代替                            |
| numaaGetValue        | 🔄   | `get_value` (core/numa/mod.rs)           | Numaa::get_value()                                                      |
| numaaRead            | 🔄   | `read_from_file` (core/numa/serial.rs)   | Numaa::read_from_file()                                                 |
| numaaReadMem         | 🔄   | `read_from_bytes` (core/numa/serial.rs)  | Numaa::read_from_bytes()                                                |
| numaaReadStream      | 🔄   | `read_from_reader` (core/numa/serial.rs) | Numaa::read_from_reader()                                               |
| numaaReplaceNuma     | 🔄   | `(idiomatic)`                            | Numaa::replace() 系で代替                                               |
| numaaTruncate        | 🔄   | `(idiomatic)`                            | Numaa::truncate() で代替                                                |
| numaaWrite           | 🔄   | `write_to_file` (core/numa/serial.rs)    | Numaa::write_to_file()                                                  |
| numaaWriteMem        | 🔄   | `write_to_bytes` (core/numa/serial.rs)   | Numaa::write_to_bytes()                                                 |
| numaaWriteStream     | 🔄   | `write_to_writer` (core/numa/serial.rs)  | Numaa::write_to_writer()                                                |

### numafunc2.c (追加分)

| C関数                           | 状態 | Rust対応                                                     | 備考              |
| ------------------------------- | ---- | ------------------------------------------------------------ | ----------------- |
| genConstrainedNumaInRange       | ✅   | `gen_constrained_numa_in_range` (core/numa/advanced.rs)      | plan 109          |
| grayHistogramsToEMD             | ✅   | `Numa::gray_histograms_to_emd` (core/numa/advanced.rs)       | plan 135          |
| grayInterHistogramStats         | ✅   | `Numa::gray_inter_histogram_stats` (core/numa/advanced.rs)   | plan 135          |
| numaClose                       | ✅   | `close` (core/numa/operations.rs)                            | name+module match |
| numaConvertToInt                | 🔄   | `(idiomatic)`                                                | `iter().map(\     |
| numaCountReversals              | ✅   | `Numa::count_reversals` (core/numa/advanced.rs)              | plan 109          |
| numaCrossingsByPeaks            | ✅   | `Numa::crossings_by_peaks` (core/numa/advanced.rs)           | plan 134          |
| numaCrossingsByThreshold        | ✅   | `numa_crossings_by_threshold` (core/numa/advanced.rs)        | plan 109          |
| numaDilate                      | ✅   | `dilate` (core/numa/operations.rs)                           | name+module match |
| numaDiscretizeHistoInBins       | ✅   | `Numa::discretize_histo_in_bins` (core/numa/advanced.rs)     | plan 130          |
| numaDiscretizeSortedInBins      | ✅   | `Numa::discretize_sorted_in_bins` (core/numa/advanced.rs)    | plan 130          |
| numaEarthMoverDistance          | ✅   | `Numa::earth_mover_distance` (core/numa/advanced.rs)         | plan 130          |
| numaErode                       | ✅   | `erode` (core/numa/operations.rs)                            | name+module match |
| numaEvalBestHaarParameters      | ✅   | `Numa::eval_best_haar_parameters` (core/numa/advanced.rs)    | plan 136          |
| numaEvalHaarSum                 | ✅   | `Numa::eval_haar_sum` (core/numa/advanced.rs)                | plan 136          |
| numaFindExtrema                 | ✅   | `find_extrema` (core/numa/operations.rs)                     | name+module match |
| numaFindPeaks                   | ✅   | `Numa::find_peaks` (core/numa/advanced.rs)                   | plan 109          |
| numaGetHistogramStats           | ✅   | `Numa::histogram_stats` (core/numa/histogram.rs)             | plan 119          |
| numaGetHistogramStatsOnInterval | ✅   | `Numa::histogram_stats_on_interval` (core/numa/histogram.rs) | plan 119          |
| numaGetRankBinValues            | ✅   | `Numa::get_rank_bin_values` (core/numa/advanced.rs)          | plan 131          |
| numaGetStatsUsingHistogram      | ✅   | `Numa::stats_using_histogram` (core/numa/operations.rs)      | plan 119          |
| numaGetUniformBinSizes          | ✅   | `numa_uniform_bin_sizes` (core/numa/advanced.rs)             | plan 109          |
| numaHistogramGetRankFromVal     | ✅   | `Numa::histogram_rank_from_val` (core/numa/histogram.rs)     | name+module match |
| numaHistogramGetValFromRank     | ✅   | `Numa::histogram_val_from_rank` (core/numa/histogram.rs)     | name+module match |
| numaMakeHistogram               | ✅   | `make_histogram` (core/numa/operations.rs)                   | name+module match |
| numaMakeHistogramAuto           | ✅   | `Numa::make_histogram_auto` (core/numa/advanced.rs)          | plan 132          |
| numaMakeHistogramClipped        | ✅   | `make_histogram_clipped` (core/numa/operations.rs)           | name+module match |
| numaMakeRankFromHistogram       | ✅   | `make_rank_from_histogram` (core/numa/advanced.rs)           | plan 119          |
| numaNormalizeHistogram          | ✅   | `normalize_histogram` (core/numa/histogram.rs)               | name+module match |
| numaOpen                        | ✅   | `open` (core/numa/operations.rs)                             | name+module match |
| numaRebinHistogram              | ✅   | `numa_rebin_histogram` (core/numa/advanced.rs)               | plan 119          |
| numaSelectCrossingThreshold     | ✅   | `select_crossing_threshold` (recog/barcode/signal.rs)        | name+module match |
| numaSimpleStats                 | ✅   | `simple_stats` (core/numa/operations.rs)                     | name+module match |
| numaSplitDistribution           | ✅   | `Numa::split_distribution` (core/numa/advanced.rs)           | plan 133          |
| numaTransform                   | ✅   | `transform` (core/numa/operations.rs)                        | name+module match |
| numaWindowedMean                | ✅   | `windowed_mean` (core/numa/operations.rs)                    | name+module match |
| numaWindowedMeanSquare          | ✅   | `windowed_mean_square` (core/numa/operations.rs)             | name+module match |
| numaWindowedMedian              | ✅   | `windowed_median` (core/numa/operations.rs)                  | name+module match |
| numaWindowedStats               | ✅   | `windowed_stats` (core/numa/operations.rs)                   | name+module match |
| numaWindowedVariance            | ✅   | `windowed_variance` (filter/windowed.rs)                     | name+module match |

### pix1.c (追加分)

| C関数               | 状態 | Rust対応 | 備考                       |
| ------------------- | ---- | -------- | -------------------------- |
| setPixMemoryManager | 🚫   | -        | Rust の GlobalAlloc で代替 |

### pix2.c (追加分)

| C関数                    | 状態 | Rust対応                                     | 備考                                      |
| ------------------------ | ---- | -------------------------------------------- | ----------------------------------------- |
| l_setAlphaMaskBorder     | 🚫   | -                                            | デフォルト値設定ヘルパー                  |
| lineEndianByteSwap       | 🚫   | -                                            | Rust 標準 (i32::swap_bytes 等) で代替     |
| pixCleanupByteProcessing | 🚫   | -                                            | 内部ヘルパー (バイト単位処理用)           |
| pixEndianTwoByteSwapNew  | ✅   | `endian_two_byte_swap_new` (core/pix/rgb.rs) | name+module match                         |
| pixSetupByteProcessing   | 🚫   | -                                            | 内部ヘルパー (バイト単位処理用)           |
| setLineDataVal           | 🚫   | -                                            | 内部ヘルパー                              |
| setPixelLow              | 🚫   | -                                            | 内部ヘルパー (set_pixel_unchecked で代替) |

### pix3.c (追加分)

| C関数                 | 状態 | Rust対応 | 備考                                   |
| --------------------- | ---- | -------- | -------------------------------------- |
| makePixelCentroidTab8 | 🚫   | -        | Rust では遅延構築/インライン展開で代替 |
| makePixelSumTab8      | 🚫   | -        | Rust では遅延構築/インライン展開で代替 |

### pix4.c (追加分)

| C関数                | 状態 | Rust対応 | 備考                      |
| -------------------- | ---- | -------- | ------------------------- |
| amapGetCountForColor | 🚫   | -        | L_Amap は BTreeMap で代替 |

### pixafunc1.c (追加分)

| C関数                        | 状態 | Rust対応                                                    | 備考                       |
| ---------------------------- | ---- | ----------------------------------------------------------- | -------------------------- |
| pixAddWithIndicator          | ✅   | `pix_add_with_indicator` (core/pixa/select.rs)              | plan 106                   |
| pixRemoveWithIndicator       | ✅   | `pix_remove_with_indicator` (core/pixa/select.rs)           | plan 106                   |
| pixSelectByArea              | ✅   | `select_by_area` (core/pixa/mod.rs)                         | name+module match          |
| pixSelectByAreaFraction      | ✅   | `pix_select_by_area_fraction` (core/pixa/select.rs)         | plan 106                   |
| pixSelectByPerimSizeRatio    | ✅   | `pix_select_by_perim_size_ratio` (core/pixa/select.rs)      | plan 106                   |
| pixSelectByPerimToAreaRatio  | ✅   | `pix_select_by_perim_to_area_ratio` (core/pixa/select.rs)   | plan 106                   |
| pixSelectBySize              | ✅   | `select_by_size` (core/pixa/mod.rs)                         | name+module match          |
| pixSelectByWidthHeightRatio  | ✅   | `pix_select_by_width_height_ratio` (core/pixa/select.rs)    | plan 106                   |
| pixaAddBorderGeneral         | ✅   | `Pixa::add_border_general` (core/pixa/transform.rs)         | plan 120                   |
| pixaAnyColormaps             | ✅   | `Pixa::any_colormaps` (core/pixa/properties.rs)             | plan 108                   |
| pixaBinSort                  | ✅   | `Pixa::bin_sort` (core/pixa/transform.rs)                   | plan 124                   |
| pixaClipToForeground         | ✅   | `Pixa::clip_to_foreground_all` (core/pixa/transform.rs)     | plan 120                   |
| pixaClipToPix                | ✅   | `Pixa::clip_to_pix` (core/pixa/transform.rs)                | plan 123                   |
| pixaConvertToGivenDepth      | ✅   | `Pixa::convert_to_given_depth` (core/pixa/transform.rs)     | plan 120                   |
| pixaConvertToSameDepth       | ✅   | `Pixa::convert_to_same_depth` (core/pixa/transform.rs)      | plan 120                   |
| pixaEqual                    | ✅   | `Pixa::equal_to_ordered` (core/pixa/properties.rs)          | plan 108 (ordered variant) |
| pixaGetDepthInfo             | ✅   | `Pixa::get_depth_info` (core/pixa/properties.rs)            | plan 108                   |
| pixaGetRenderingDepth        | ✅   | `Pixa::get_rendering_depth` (core/pixa/properties.rs)       | plan 108                   |
| pixaHasColor                 | ✅   | `Pixa::has_color` (core/pixa/properties.rs)                 | plan 108                   |
| pixaMakeSizeIndicator        | ✅   | `Pixa::make_size_indicator` (core/pixa/properties.rs)       | plan 121                   |
| pixaRenderComponent          | ✅   | `Pixa::render_component` (core/pixa/transform.rs)           | plan 123                   |
| pixaRotate                   | ✅   | `Pixa::rotate` (core/pixa/transform.rs)                     | plan 123                   |
| pixaRotateOrth               | ✅   | `Pixa::rotate_orth` (core/pixa/transform.rs)                | plan 107                   |
| pixaScale                    | ✅   | `Pixa::scale` (core/pixa/transform.rs)                      | plan 107                   |
| pixaScaleBySampling          | ✅   | `Pixa::scale_by_sampling` (core/pixa/transform.rs)          | plan 107                   |
| pixaSelectByAreaFraction     | ✅   | `Pixa::select_by_area_fraction` (core/pixa/select.rs)       | plan 106                   |
| pixaSelectByNumConnComp      | ✅   | `Pixa::select_by_num_conn_comp` (core/pixa/select.rs)       | plan 106                   |
| pixaSelectByPerimSizeRatio   | ✅   | `Pixa::select_by_perim_size_ratio` (core/pixa/select.rs)    | plan 106                   |
| pixaSelectByPerimToAreaRatio | ✅   | `Pixa::select_by_perim_to_area_ratio` (core/pixa/select.rs) | plan 106                   |
| pixaSelectByWidthHeightRatio | ✅   | `Pixa::select_by_width_height_ratio` (core/pixa/select.rs)  | plan 106                   |
| pixaSelectRange              | ✅   | `Pixa::select_range` (core/pixa/select.rs)                  | plan 106                   |
| pixaSelectWithIndicator      | ✅   | `Pixa::select_with_indicator` (core/pixa/select.rs)         | plan 106                   |
| pixaSelectWithString         | ✅   | `Pixa::select_with_string` (core/pixa/select.rs)            | plan 106                   |
| pixaSetFullSizeBoxa          | ✅   | `Pixa::set_full_size_boxa` (core/pixa/properties.rs)        | plan 108                   |
| pixaSizeRange                | ✅   | `Pixa::size_range` (core/pixa/properties.rs)                | plan 108                   |
| pixaSort2dByIndex            | ✅   | `Pixa::sort_2d_by_index` (core/pixa/properties.rs)          | plan 121                   |
| pixaTranslate                | ✅   | `Pixa::translate` (core/pixa/transform.rs)                  | plan 107                   |
| pixaaFlattenToPixa           | ✅   | `Pixaa::flatten_to_pixa` (core/pixa/properties.rs)          | plan 122                   |
| pixaaScaleToSize             | ✅   | `scale_to_size` (core/pixa/mod.rs)                          | name+module match          |
| pixaaScaleToSizeVar          | ✅   | `Pixaa::scale_to_size_var` (core/pixa/transform.rs)         | plan 124                   |
| pixaaSelectRange             | ✅   | `Pixaa::select_range` (core/pixa/properties.rs)             | plan 122                   |
| pixaaSizeRange               | ✅   | `Pixaa::size_range` (core/pixa/properties.rs)               | plan 122                   |

### pixafunc2.c (追加分)

| C関数                         | 状態 | Rust対応                                               | 備考                                    |
| ----------------------------- | ---- | ------------------------------------------------------ | --------------------------------------- |
| convertToNUpFiles             | 🚫   | -                                                      | PDF/PS生成のN-up処理                    |
| convertToNUpPixa              | 🚫   | -                                                      | PDF/PS生成のN-up処理                    |
| pixGetTileCount               | ✅   | `Pix::get_tile_count` (core/pixa/properties.rs)        | plan 108                                |
| pixaCompareInPdf              | 🚫   | -                                                      | デバッグ用 PDF 比較 (Rust では別途実装) |
| pixaConstrainedSelect         | ✅   | `Pixa::constrained_select` (core/pixa/properties.rs)   | plan 121                                |
| pixaConvertTo1                | ✅   | `Pixa::convert_to_1` (core/pixa/transform.rs)          | plan 107                                |
| pixaConvertTo32               | ✅   | `Pixa::convert_to_32` (core/pixa/transform.rs)         | plan 107                                |
| pixaConvertTo8                | ✅   | `Pixa::convert_to_8` (core/pixa/transform.rs)          | plan 107                                |
| pixaConvertTo8Colormap        | ✅   | `Pixa::convert_to_8_colormap` (core/pixa/transform.rs) | plan 125                                |
| pixaConvertToNUpPixa          | ✅   | `Pixa::convert_to_nup` (core/pixa/transform.rs)        | plan 127 (fontsize == 0 のみ)           |
| pixaDisplayLinearly           | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayMultiTiled         | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayOnLattice          | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayPairTiledInColumns | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayRandomCmap         | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayTiledByIndex       | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayTiledInColumns     | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayTiledInRows        | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayTiledWithText      | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaDisplayUnsplit            | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaMakeFromTiledPix          | ✅   | `Pix::make_tiled_pixa` (core/pixa/transform.rs)        | plan 125                                |
| pixaMakeFromTiledPixa         | ✅   | `Pixa::make_tiled_pixa` (core/pixa/transform.rs)       | plan 125                                |
| pixaSelectToPdf               | ✅   | `Pixa::select_to_pdf` (core/pixa/transform.rs)         | plan 126 (fontsize <= 0 のみ)           |
| pixaSplitIntoFiles            | 🚫   | -                                                      | デバッグ用 file-system 書き出し helper  |
| pixaaDisplay                  | ✅   | `display` (core/pixa/mod.rs)                           | name+module match                       |
| pixaaDisplayByPixa            | 🚫   | -                                                      | 画像表示用 (Rust では別途実装)          |
| pixaaDisplayTiledAndScaled    | ✅   | `display_tiled_and_scaled` (core/pixa/mod.rs)          | name+module match                       |

### pixarith.c (追加分)

| C関数                       | 状態 | Rust対応                                    | 備考                                                  |
| --------------------------- | ---- | ------------------------------------------- | ----------------------------------------------------- |
| getLogBase2                 | 🚫   | -                                           | Rust の u32::ilog2() で代替                           |
| linearScaleRGBVal           | ✅   | `linear_scale_rgb_val` (core/pix/arith.rs)  | plan 114                                              |
| logScaleRGBVal              | ✅   | `log_scale_rgb_val` (core/pix/arith.rs)     | plan 114                                              |
| makeLogBase2Tab             | 🚫   | -                                           | Rust では遅延構築で代替                               |
| pixAccumulate               | 🔄   | `(idiomatic)`                               | Pixacc::add 等で代替 (core/pixacc.rs)                 |
| pixAddRGB                   | ✅   | `add_rgb` (core/pix/arith.rs)               | plan 114                                              |
| pixFinalAccumulate          | 🔄   | `(idiomatic)`                               | Pixacc::final_accumulate で代替 (core/pixacc.rs)      |
| pixFinalAccumulateThreshold | 🔄   | `(idiomatic)`                               | Pixacc::final_accumulate_threshold で代替             |
| pixInitAccumulate           | 🔄   | `(idiomatic)`                               | Pixacc::new() で代替 (core/pixacc.rs)                 |
| pixMaxDynamicRange          | 🔄   | `(idiomatic)`                               | pix_max_dynamic_range が core/pix/arith.rs で利用可能 |
| pixMaxDynamicRangeRGB       | ✅   | `max_dynamic_range_rgb` (core/pix/arith.rs) | plan 114                                              |
| pixMultiplyGray             | ✅   | `multiply_gray` (core/pix/arith.rs)         | name+module match                                     |
| pixThresholdToValue         | ✅   | `threshold_to_value` (core/pix/arith.rs)    | plan 114                                              |

### pixconv.c (追加分)

| C関数                | 状態 | Rust対応 | 備考                            |
| -------------------- | ---- | -------- | ------------------------------- |
| l_setNeutralBoostVal | 🚫   | -        | 内部設定値変更 (グローバル状態) |

### ptafunc1.c (追加分)

| C関数                  | 状態 | Rust対応                                           | 備考                          |
| ---------------------- | ---- | -------------------------------------------------- | ----------------------------- |
| applyCubicFit          | ✅   | `apply_cubic_fit` (core/pta/lsf.rs)                | name+module match             |
| applyLinearFit         | ✅   | `apply_linear_fit` (core/pta/lsf.rs)               | name+module match             |
| applyQuadraticFit      | ✅   | `apply_quadratic_fit` (core/pta/lsf.rs)            | name+module match             |
| applyQuarticFit        | ✅   | `apply_quartic_fit` (core/pta/lsf.rs)              | name+module match             |
| l_angleBetweenVectors  | ✅   | `angle_between_vectors` (core/pta/transform.rs)    | name+module match             |
| numaConvertToPta1      | ✅   | `Pta::create_from_numa` (core/pta/mod.rs)          | name+module match             |
| numaConvertToPta2      | ✅   | `Pta::create_from_numa` (core/pta/mod.rs)          | name+module match             |
| pixDisplayPta          | 🚫   | -                                                  | GUI/X11 表示は Rust 未提供    |
| pixDisplayPtaPattern   | 🚫   | -                                                  | GUI/X11 表示は Rust 未提供    |
| pixDisplayPtaa         | 🚫   | -                                                  | GUI/X11 表示は Rust 未提供    |
| pixDisplayPtaaPattern  | 🚫   | -                                                  | GUI/X11 表示は Rust 未提供    |
| pixFindCornerPixels    | ✅   | `Pix::find_corner_pixels` (core/pta/graphics.rs)   | plan 111                      |
| pixGenerateFromPta     | ✅   | `pix_generate_from_pta` (core/pta/graphics.rs)     | plan 111                      |
| pixPlotAlongPta        | ❌   | -                                                  | no Rust impl in expected dirs |
| ptaConvertToNuma       | ✅   | `Pta::to_numa_pair` (core/pta/graphics.rs)         | plan 111                      |
| ptaGetBoundaryPixels   | ✅   | `pta_get_boundary_pixels` (core/pta/graphics.rs)   | plan 137                      |
| ptaGetBoundingRegion   | ✅   | `Pta::bounding_region` (core/pta/graphics.rs)      | plan 111                      |
| ptaGetNeighborPixLocs  | ✅   | `pta_get_neighbor_pix_locs` (core/pta/graphics.rs) | plan 137                      |
| ptaGetPixelsFromPix    | ✅   | `pta_get_pixels_from_pix` (core/pta/graphics.rs)   | plan 111                      |
| ptaNoisyLinearLSF      | ✅   | `Pta::noisy_linear_lsf` (core/pta/lsf.rs)          | plan 138                      |
| ptaNoisyQuadraticLSF   | ✅   | `Pta::noisy_quadratic_lsf` (core/pta/lsf.rs)       | plan 138                      |
| ptaReplicatePattern    | ✅   | `Pta::replicate_pattern` (core/pta/graphics.rs)    | plan 111                      |
| ptaaGetBoundaryPixels  | ❌   | -                                                  | no Rust impl in expected dirs |
| ptaaIndexLabeledPixels | ✅   | `ptaa_index_labeled_pixels` (core/pta/graphics.rs) | plan 137                      |

### sarray1.c (追加分)

| C関数                            | 状態 | Rust対応 | 備考                                    |
| -------------------------------- | ---- | -------- | --------------------------------------- |
| convertSortedToNumberedPathnames | 🚫   | -        | PDF/PS生成のN-up処理                    |
| getFilenamesInDirectory          | 🚫   | -        | Rust の std::fs::read_dir で代替        |
| getNumberedPathnamesInDirectory  | 🚫   | -        | Rust の std::fs::read_dir + sort で代替 |
| getSortedPathnamesInDirectory    | 🚫   | -        | Rust の std::fs::read_dir + sort で代替 |
