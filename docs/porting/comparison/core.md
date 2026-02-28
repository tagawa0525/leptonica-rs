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

#### その他

| C関数             | 状態 | Rust対応                 | 備考                      |
| ----------------- | ---- | ------------------------ | ------------------------- |
| pixCreate         | ✅   | Pix::new()               |                           |
| pixCreateTemplate | ✅   | Pix::create_template     |                           |
| pixCreateWithCmap | ✅   | Pix::new_with_colormap   |                           |
| pixClone          | 🔄   | Pix::clone()             | Arc参照カウントで自動実装 |
| pixDestroy        | 🔄   | drop()                   | Rustのデストラクタで自動  |
| pixCopy           | 🔄   | Pix::deep_clone()        | deep_cloneが完全コピー    |
| pixGetWidth       | ✅   | Pix::width()             |                           |
| pixGetHeight      | ✅   | Pix::height()            |                           |
| pixGetDepth       | ✅   | Pix::depth()             |                           |
| pixGetDimensions  | ✅   | width()/height()/depth() | 個別メソッドで取得        |
| pixGetSpp         | ✅   | Pix::spp()               |                           |
| pixSetSpp         | 🔄   | PixMut::set_spp()        | PixMutで可変              |
| pixGetWpl         | ✅   | Pix::wpl()               |                           |
| pixGetXRes        | ✅   | Pix::xres()              |                           |
| pixSetXRes        | 🔄   | PixMut::set_xres()       |                           |
| pixGetYRes        | ✅   | Pix::yres()              |                           |
| pixSetYRes        | 🔄   | PixMut::set_yres()       |                           |
| pixGetResolution  | ✅   | xres()/yres()            |                           |
| pixSetResolution  | 🔄   | PixMut::set_resolution() |                           |
| pixGetInputFormat | ✅   | Pix::informat()          |                           |
| pixSetInputFormat | 🔄   | PixMut::set_informat()   |                           |
| pixSetSpecial     | 🔄   | PixMut::set_special()    |                           |
| pixGetText        | ✅   | Pix::text()              |                           |
| pixSetText        | 🔄   | PixMut::set_text()       |                           |
| pixGetTextCompNew | ✅   | get_text_comp_new        |                           |
| pixSetTextCompNew | ✅   | set_text_comp_new        |                           |
| pixGetColormap    | ✅   | Pix::colormap()          |                           |
| pixSetColormap    | 🔄   | PixMut::set_colormap()   |                           |
| pixGetData        | ✅   | Pix::data()              |                           |

#### 対応なし

| C関数                   | 状態 | Rust対応 | 備考                       |
| ----------------------- | ---- | -------- | -------------------------- |
| pixCreateNoInit         | 🚫   | -        | Rustは常に初期化する       |
| pixCreateTemplateNoInit | 🚫   | -        | Rustは常に初期化する       |
| pixCreateHeader         | 🚫   | -        | Rustは常に初期化する       |
| pixResizeImageData      | 🚫   | -        | Rustの所有権モデルで不要   |
| pixTransferAllData      | 🚫   | -        | Rustの所有権モデルで不要   |
| pixSwapAndDestroy       | 🚫   | -        | Rustの所有権モデルで不要   |
| pixSetWidth             | 🚫   | -        | Pixは不変                  |
| pixSetHeight            | 🚫   | -        | Pixは不変                  |
| pixSetDepth             | 🚫   | -        | Pixは不変                  |
| pixSetDimensions        | 🚫   | -        | Pixは不変                  |
| pixCopyDimensions       | 🚫   | -        | Pixは不変                  |
| pixCopySpp              | 🚫   | -        | Pixは不変                  |
| pixSetWpl               | 🚫   | -        | 自動計算のため不要         |
| pixDestroyColormap      | 🚫   | -        | set_colormap(None)で実現可 |
| pixFreeAndSetData       | 🚫   | -        | Cメモリ管理                |
| pixSetData              | 🚫   | -        | Cメモリ管理                |
| pixFreeData             | 🚫   | -        | Cメモリ管理                |
| pixExtractData          | 🚫   | -        | Cメモリ管理                |
| pixGetLinePtrs          | 🚫   | -        | Cポインタ配列              |
| pixPrintStreamInfo      | 🚫   | -        | Debug traitで対応          |

#### pix/mod.rs

| C関数              | 状態 | Rust対応                 | 備考 |
| ------------------ | ---- | ------------------------ | ---- |
| pixCopyColormap    | ✅   | copy_colormap_from()     |      |
| pixCopyResolution  | ✅   | copy_resolution_from()   |      |
| pixScaleResolution | ✅   | scale_resolution()       |      |
| pixCopyInputFormat | ✅   | copy_input_format_from() |      |
| pixAddText         | ✅   | add_text()               |      |
| pixCopyText        | ✅   | copy_text_from()         |      |
| pixSizesEqual      | ✅   | sizes_equal()            |      |
| pixMaxAspectRatio  | ✅   | max_aspect_ratio()       |      |

### pix2.c (ピクセルアクセス・設定)

#### その他

| C関数                           | 状態 | Rust対応                         | 備考 |
| ------------------------------- | ---- | -------------------------------- | ---- |
| pixGetPixel                     | ✅   | Pix::get_pixel()                 |      |
| pixSetPixel                     | ✅   | PixMut::set_pixel()              |      |
| pixGetRandomPixel               | ✅   | get_random_pixel                 |      |
| pixClearAll                     | 🔄   | PixMut::clear()                  |      |
| pixSetAll                       | 🔄   | PixMut::set_all()                |      |
| pixSetComponentArbitrary        | ✅   | set_component_arbitrary          |      |
| pixBlendInRect                  | ✅   | blend_in_rect                    |      |
| pixSetBorderRingVal             | ✅   | set_border_ring_val              |      |
| pixSetMirroredBorder            | ✅   | set_mirrored_border              |      |
| pixCopyBorder                   | ✅   | copy_border                      |      |
| pixAddMultipleBlackWhiteBorders | ✅   | add_multiple_black_white_borders |      |
| pixRemoveBorderToSize           | ✅   | remove_border_to_size            |      |
| pixAddMixedBorder               | ✅   | add_mixed_border                 |      |
| pixAddContinuedBorder           | ✅   | add_continued_border             |      |
| pixShiftAndTransferAlpha        | ✅   | shift_and_transfer_alpha         |      |
| pixGetRGBComponentCmap          | ✅   | get_rgb_component_cmap           |      |
| pixCopyRGBComponent             | ✅   | copy_rgb_component               |      |
| pixGetRGBLine                   | ✅   | get_rgb_line                     |      |
| pixEndianByteSwapNew            | ✅   | Pix::endian_byte_swap_new        |      |
| pixEndianByteSwap               | ✅   | PixMut::endian_byte_swap         |      |
| pixEndianTwoByteSwap            | ✅   | PixMut::endian_two_byte_swap     |      |
| pixInferResolution              | ✅   | infer_resolution                 |      |
| pixAlphaIsOpaque                | ✅   | alpha_is_opaque                  |      |

#### rgb.rs

| C関数              | 状態 | Rust対応            | 備考 |
| ------------------ | ---- | ------------------- | ---- |
| pixGetRGBPixel     | ✅   | get_rgb_pixel()     |      |
| pixSetRGBPixel     | ✅   | set_rgb_pixel()     |      |
| pixCreateRGBImage  | ✅   | create_rgb_image()  |      |
| pixGetRGBComponent | ✅   | get_rgb_component() |      |
| pixSetRGBComponent | ✅   | set_rgb_component() |      |

#### pix/rgb.rs

| C関数           | 状態 | Rust対応         | 備考 |
| --------------- | ---- | ---------------- | ---- |
| pixSetCmapPixel | ✅   | set_cmap_pixel() |      |

#### pix/mod.rs

| C関数                 | 状態 | Rust対応                 | 備考 |
| --------------------- | ---- | ------------------------ | ---- |
| pixClearPixel         | ✅   | clear_pixel()            |      |
| pixFlipPixel          | ✅   | flip_pixel()             |      |
| pixGetBlackOrWhiteVal | ✅   | get_black_or_white_val() |      |
| pixSetAllGray         | ✅   | set_all_gray()           |      |
| pixSetAllArbitrary    | ✅   | set_all_arbitrary()      |      |
| pixSetBlackOrWhite    | ✅   | set_black_or_white()     |      |
| pixClearInRect        | ✅   | clear_in_rect()          |      |
| pixSetInRect          | ✅   | set_in_rect()            |      |
| pixSetInRectArbitrary | ✅   | set_in_rect_arbitrary()  |      |
| pixSetPadBits         | ✅   | set_pad_bits()           |      |
| pixSetPadBitsBand     | ✅   | set_pad_bits_band()      |      |
| pixSetOrClearBorder   | ✅   | set_or_clear_border()    |      |

#### border.rs

| C関数                    | 状態 | Rust対応                    | 備考 |
| ------------------------ | ---- | --------------------------- | ---- |
| pixSetBorderVal          | ✅   | set_border_val()            |      |
| pixAddBorder             | ✅   | add_border()                |      |
| pixAddBlackOrWhiteBorder | ✅   | add_black_or_white_border() |      |
| pixAddBorderGeneral      | ✅   | add_border_general()        |      |
| pixRemoveBorder          | ✅   | remove_border()             |      |
| pixRemoveBorderGeneral   | ✅   | remove_border_general()     |      |
| pixAddMirroredBorder     | ✅   | add_mirrored_border()       |      |
| pixAddRepeatedBorder     | ✅   | add_repeated_border()       |      |

#### 対応なし

| C関数                | 状態 | Rust対応 | 備考             |
| -------------------- | ---- | -------- | ---------------- |
| pixDisplayLayersRGBA | 🚫   | -        | デバッグ表示関数 |
| pixGetRasterData     | 🚫   | -        | Cポインタ取得    |

#### core/pixel.rs

| C関数                  | 状態 | Rust対応                                        | 備考 |
| ---------------------- | ---- | ----------------------------------------------- | ---- |
| composeRGBPixel        | ✅   | compose_rgb()                                   |      |
| composeRGBAPixel       | ✅   | compose_rgba()                                  |      |
| extractRGBValues       | ✅   | extract_rgb()                                   |      |
| extractRGBAValues      | ✅   | extract_rgba()                                  |      |
| extractMinMaxComponent | ✅   | extract_min_component()/extract_max_component() |      |

### pix3.c (マスク・ブール演算)

#### mask.rs

| C関数                   | 状態 | Rust対応                 | 備考 |
| ----------------------- | ---- | ------------------------ | ---- |
| pixSetMasked            | ✅   | set_masked()             |      |
| pixSetMaskedGeneral     | ✅   | set_masked_general()     |      |
| pixCombineMasked        | ✅   | combine_masked()         |      |
| pixCombineMaskedGeneral | ✅   | combine_masked_general() |      |
| pixPaintThroughMask     | ✅   | paint_through_mask()     |      |
| pixCopyWithBoxa         | ✅   | copy_with_boxa()         |      |
| pixMakeMaskFromVal      | ✅   | make_mask_from_val()     |      |
| pixMakeMaskFromLUT      | ✅   | make_mask_from_lut()     |      |
| pixMakeArbMaskFromRGB   | ✅   | make_arb_mask_from_rgb() |      |
| pixSetUnderTransparency | ✅   | set_under_transparency() |      |

#### その他

| C関数                       | 状態 | Rust対応                     | 備考      |
| --------------------------- | ---- | ---------------------------- | --------- |
| pixPaintSelfThroughMask     | ✅   | paint_self_through_mask      | 後続Phase |
| pixMakeAlphaFromMask        | ✅   | make_alpha_from_mask         |           |
| pixGetColorNearMaskBoundary | ✅   | get_color_near_mask_boundary |           |
| pixInvert                   | ✅   | ops.rsに実装                 |           |
| pixOr                       | ✅   | ops.rsに実装                 |           |
| pixAnd                      | ✅   | ops.rsに実装                 |           |
| pixXor                      | ✅   | ops.rsに実装                 |           |
| pixSubtract                 | ✅   | ops.rsに実装                 |           |
| pixaCountPixels             | ✅   | pixa count_pixels()          |           |

#### 対応なし

| C関数                    | 状態 | Rust対応 | 備考               |
| ------------------------ | ---- | -------- | ------------------ |
| pixDisplaySelectedPixels | 🚫   | -        | デバッグ表示関数   |
| pixMirroredTiling        | 🚫   | -        | デバッグ表示関数   |
| pixFindRepCloseTile      | 🚫   | -        | タイリングヘルパー |

#### statistics.rs

| C関数                  | 状態 | Rust対応                 | 備考       |
| ---------------------- | ---- | ------------------------ | ---------- |
| pixZero                | ✅   | is_zero()                |            |
| pixForegroundFraction  | ✅   | foreground_fraction()    |            |
| pixCountPixels         | ✅   | count_pixels()           |            |
| pixCountPixelsInRect   | ✅   | count_pixels_in_rect()   |            |
| pixCountByRow          | ✅   | count_by_row()           |            |
| pixCountByColumn       | ✅   | count_by_column()        |            |
| pixCountPixelsByRow    | ✅   | count_pixels_by_row()    | Numa返却版 |
| pixCountPixelsByColumn | ✅   | count_pixels_by_column() | Numa返却版 |
| pixCountPixelsInRow    | ✅   | count_pixels_in_row()    |            |
| pixGetMomentByColumn   | ✅   | get_moment_by_column()   |            |
| pixThresholdPixelSum   | ✅   | threshold_pixel_sum()    |            |
| pixAverageByRow        | ✅   | average_by_row()         |            |
| pixAverageByColumn     | ✅   | average_by_column()      |            |
| pixAverageInRect       | ✅   | average_in_rect()        |            |
| pixAverageInRectRGB    | ✅   | average_in_rect_rgb()    |            |
| pixVarianceByRow       | ✅   | variance_by_row()        |            |
| pixVarianceByColumn    | ✅   | variance_by_column()     |            |
| pixVarianceInRect      | ✅   | variance_in_rect()       |            |
| pixAbsDiffByRow        | ✅   | abs_diff_by_row()        |            |
| pixAbsDiffByColumn     | ✅   | abs_diff_by_column()     |            |
| pixAbsDiffInRect       | ✅   | abs_diff_in_rect()       |            |
| pixAbsDiffOnLine       | ✅   | abs_diff_on_line()       |            |
| pixCountArbInRect      | ✅   | count_arb_in_rect()      |            |

### pix4.c (ヒストグラム・統計)

#### その他

| C関数                        | 状態 | Rust対応                        | 備考 |
| ---------------------------- | ---- | ------------------------------- | ---- |
| pixGetGrayHistogram          | ✅   | histogram.rsに実装              |      |
| pixGetColorHistogram         | ✅   | histogram.rsに実装              |      |
| pixCountRGBColorsByHash      | ✅   | count_rgb_colors_by_hash        |      |
| pixGetColorAmapHistogram     | ✅   | get_color_amap_histogram        |      |
| pixGetBinnedComponentRange   | ✅   | get_binned_component_range      |      |
| pixGetRankColorArray         | ✅   | get_rank_color_array            |      |
| pixGetBinnedColor            | ✅   | get_binned_color                |      |
| pixDisplayColorArray         | ✅   | display_color_array             |      |
| pixRankBinByStrip            | ✅   | rank_bin_by_strip               |      |
| pixaGetAlignedStats          | ✅   | pixa aligned_stats()            |      |
| pixaExtractColumnFromEachPix | ✅   | pixa extract_column_from_each() |      |
| pixSplitDistributionFgBg     | ✅   | split_distribution_fg_bg        |      |

#### histogram.rs

| C関数                      | 状態 | Rust対応                 | 備考 |
| -------------------------- | ---- | ------------------------ | ---- |
| pixGetGrayHistogramMasked  | ✅   | gray_histogram_masked()  |      |
| pixGetGrayHistogramInRect  | ✅   | gray_histogram_in_rect() |      |
| pixGetGrayHistogramTiled   | ✅   | gray_histogram_tiled()   |      |
| pixGetColorHistogramMasked | ✅   | color_histogram_masked() |      |
| pixGetCmapHistogram        | ✅   | cmap_histogram()         |      |
| pixGetCmapHistogramMasked  | ✅   | cmap_histogram_masked()  |      |
| pixGetCmapHistogramInRect  | ✅   | cmap_histogram_in_rect() |      |
| pixCountRGBColors          | ✅   | count_rgb_colors()       |      |
| pixGetRankValue            | ✅   | pixel_rank_value()       |      |
| pixGetRankValueMaskedRGB   | ✅   | rank_value_masked_rgb()  |      |
| pixGetRankValueMasked      | ✅   | rank_value_masked()      |      |
| pixGetAverageMaskedRGB     | ✅   | average_masked_rgb()     |      |
| pixGetAverageMasked        | ✅   | average_masked()         |      |
| pixGetAverageTiledRGB      | ✅   | average_tiled_rgb()      |      |
| pixGetAverageTiled         | ✅   | average_tiled()          |      |
| pixGetMaxColorIndex        | ✅   | max_color_index()        |      |

#### statistics.rs

| C関数                | 状態 | Rust対応            | 備考 |
| -------------------- | ---- | ------------------- | ---- |
| pixGetPixelAverage   | ✅   | get_pixel_average() |      |
| pixGetPixelStats     | ✅   | get_pixel_stats()   |      |
| pixRowStats          | ✅   | row_stats()         |      |
| pixColumnStats       | ✅   | column_stats()      |      |
| pixGetRangeValues    | ✅   | range_values()      |      |
| pixGetExtremeValue   | ✅   | extreme_value()     |      |
| pixGetMaxValueInRect | ✅   | max_value_in_rect() |      |
| pixGetRowStats       | ✅   | get_row_stats()     |      |
| pixGetColumnStats    | ✅   | get_column_stats()  |      |
| pixSetPixelColumn    | ✅   | set_pixel_column()  |      |

#### clip.rs

| C関数               | 状態 | Rust対応              | 備考 |
| ------------------- | ---- | --------------------- | ---- |
| pixThresholdForFgBg | ✅   | threshold_for_fg_bg() |      |

### pix5.c (選択・測定)

#### その他

| C関数                        | 状態 | Rust対応                        | 備考 |
| ---------------------------- | ---- | ------------------------------- | ---- |
| pixaFindDimensions           | ✅   | pixa find_dimensions()          |      |
| pixFindAreaPerimRatio        | ✅   | find_area_perim_ratio           |      |
| pixaFindPerimToAreaRatio     | ✅   | Pixa::find_perim_to_area_ratio  |      |
| pixaFindPerimSizeRatio       | ✅   | Pixa::find_perim_size_ratio     |      |
| pixFindPerimSizeRatio        | ✅   | find_perim_size_ratio           |      |
| pixaFindAreaFraction         | ✅   | Pixa::find_area_fraction        |      |
| pixFindAreaFraction          | ✅   | find_area_fraction              |      |
| pixaFindAreaFractionMasked   | ✅   | Pixa::find_area_fraction_masked |      |
| pixFindAreaFractionMasked    | ✅   | find_area_fraction_masked       |      |
| pixaFindWidthHeightRatio     | ✅   | Pixa::find_width_height_ratio   |      |
| pixaFindWidthHeightProduct   | ✅   | Pixa::find_width_height_product |      |
| pixFindRectangleComps        | ✅   | find_rectangle_comps            |      |
| pixConformsToRectangle       | ✅   | conforms_to_rectangle           |      |
| pixExtractRectangularRegions | ✅   | extract_rectangular_regions     |      |
| pixSelectComponentBySize     | ✅   | select_component_by_size        |      |
| pixFilterComponentBySize     | ✅   | filter_component_by_size        |      |
| pixMakeCoveringOfRectangles  | ✅   | make_covering_of_rectangles     |      |
| pixReversalProfile           | ✅   | reversal_profile                |      |
| pixWindowedVarianceOnLine    | ✅   | windowed_variance_on_line       |      |
| pixMinMaxNearLine            | ✅   | min_max_near_line               |      |

#### measurement.rs

| C関数                   | 状態 | Rust対応                   | 備考 |
| ----------------------- | ---- | -------------------------- | ---- |
| pixFindPerimToAreaRatio | ✅   | find_perim_to_area_ratio() |      |
| pixFindOverlapFraction  | ✅   | find_overlap_fraction()    |      |

#### clip.rs

| C関数                      | 状態 | Rust対応                     | 備考       |
| -------------------------- | ---- | ---------------------------- | ---------- |
| pixClipRectangles          | ✅   | clip_rectangles()            |            |
| pixClipRectangle           | ✅   | clip_rectangle()             |            |
| pixClipRectangleWithBorder | ✅   | clip_rectangle_with_border() |            |
| pixClipMasked              | ✅   | clip_masked()                |            |
| pixCropToMatch             | ✅   | crop_to_match()              |            |
| pixCropToSize              | ✅   | crop_to_size()               |            |
| pixResizeToMatch           | ✅   | resize_to_match()            |            |
| pixMakeSymmetricMask       | ✅   | make_symmetric_mask()        |            |
| pixMakeFrameMask           | ✅   | make_frame_mask()            |            |
| pixFractionFgInMask        | ✅   | fraction_fg_in_mask()        |            |
| pixClipToForeground        | ✅   | clip_to_foreground()         |            |
| pixTestClipToForeground    | ✅   | test_clip_to_foreground()    |            |
| pixClipBoxToForeground     | ✅   | clip_box_to_foreground()     |            |
| pixScanForForeground       | ✅   | scan_for_foreground()        |            |
| pixClipBoxToEdges          | ✅   | clip_box_to_edges()          |            |
| pixScanForEdge             | ✅   | scan_for_edge()              | 8bpp適応版 |
| pixAverageOnLine           | ✅   | average_on_line()            |            |

#### extract.rs

| C関数                      | 状態 | Rust対応                    | 備考 |
| -------------------------- | ---- | --------------------------- | ---- |
| pixExtractOnLine           | ✅   | extract_on_line()           |      |
| pixAverageIntensityProfile | ✅   | average_intensity_profile() |      |
| pixRankRowTransform        | ✅   | rank_row_transform()        |      |
| pixRankColumnTransform     | ✅   | rank_column_transform()     |      |

### boxbasic.c (Box基本操作)

| C関数                  | 状態 | Rust対応                  | 備考                       |
| ---------------------- | ---- | ------------------------- | -------------------------- |
| boxCreate              | ✅   | Box::new()                |                            |
| boxCreateValid         | 🚫   | -                         | new()でバリデーション実施  |
| boxCopy                | 🔄   | Box自体がCopyトレイト     |                            |
| boxClone               | 🔄   | Box自体がCopyトレイト     |                            |
| boxDestroy             | 🔄   | drop()                    | 自動                       |
| boxGetGeometry         | ✅   | フィールドアクセス        |                            |
| boxSetGeometry         | ✅   | box_set_geometry          |                            |
| boxGetSideLocations    | ✅   | box_get_side_locations    | right()/bottom()で部分対応 |
| boxSetSideLocations    | ✅   | box_set_side_locations    |                            |
| boxIsValid             | ✅   | Box::is_valid()           |                            |
| boxaCreate             | ✅   | Boxa::new()               |                            |
| boxaCopy               | ✅   | Boxa::clone()             |                            |
| boxaDestroy            | 🔄   | drop()                    | 自動                       |
| boxaAddBox             | ✅   | Boxa::push()              |                            |
| boxaExtendArray        | 🚫   | -                         | Vec自動拡張                |
| boxaExtendArrayToSize  | 🚫   | -                         | Vec自動拡張                |
| boxaGetCount           | ✅   | Boxa::len()               |                            |
| boxaGetValidCount      | 🚫   | -                         | Rustの型システムで不要     |
| boxaGetBox             | ✅   | Boxa::get()               |                            |
| boxaGetValidBox        | 🚫   | -                         | Rustの型システムで不要     |
| boxaFindInvalidBoxes   | 🚫   | -                         | Rustの型システムで不要     |
| boxaGetBoxGeometry     | ✅   | Boxa::get_box_geometry    |                            |
| boxaIsFull             | 🚫   | -                         | Rustの型システムで不要     |
| boxaReplaceBox         | ✅   | Boxa::replace()           |                            |
| boxaInsertBox          | ✅   | Boxa::insert()            |                            |
| boxaRemoveBox          | ✅   | Boxa::remove()            |                            |
| boxaRemoveBoxAndSave   | ✅   | Boxa::remove_box_and_save |                            |
| boxaSaveValid          | 🚫   | -                         | Rustの型システムで不要     |
| boxaInitFull           | ✅   | Boxa::init_full           |                            |
| boxaClear              | ✅   | Boxa::clear()             |                            |
| boxaaCreate            | ✅   | Boxaa::new()              |                            |
| boxaaCopy              | ✅   | Boxaa::copy               |                            |
| boxaaDestroy           | 🔄   | drop()                    | 自動                       |
| boxaaAddBoxa           | ✅   | Boxaa::push()             |                            |
| boxaaExtendArray       | 🚫   | -                         | Vec自動拡張                |
| boxaaExtendArrayToSize | 🚫   | -                         | Vec自動拡張                |
| boxaaGetCount          | ✅   | Boxaa::len()              |                            |
| boxaaGetBoxCount       | ✅   | Boxaa::total_boxes()      |                            |
| boxaaGetBoxa           | ✅   | Boxaa::get()              |                            |
| boxaaGetBox            | ✅   | Boxaa::get_box            |                            |
| boxaaInitFull          | 🚫   | -                         | Rustの型システムで不要     |
| boxaaExtendWithInit    | 🚫   | -                         | Rustの型システムで不要     |
| boxaaReplaceBoxa       | ✅   | Boxaa::replace_boxa       |                            |
| boxaaInsertBoxa        | ✅   | Boxaa::insert_boxa        |                            |
| boxaaRemoveBoxa        | ✅   | Boxaa::remove_boxa        |                            |
| boxaaAddBox            | ✅   | Boxaa::add_box            |                            |
| boxaaReadFromFiles     | ✅   | Boxaa::read_from_files    |                            |
| boxaaRead              | ✅   | Boxaa::read_from_file     |                            |
| boxaaReadStream        | ✅   | Boxaa::read_from_reader   |                            |
| boxaaReadMem           | ✅   | Boxaa::read_from_bytes    |                            |
| boxaaWrite             | ✅   | Boxaa::write_to_file      |                            |
| boxaaWriteStream       | ✅   | Boxaa::write_to_writer    |                            |
| boxaaWriteMem          | ✅   | Boxaa::write_to_bytes     |                            |
| boxaRead               | ✅   | Boxa::read_from_file      |                            |
| boxaReadStream         | ✅   | Boxa::read_from_reader    |                            |
| boxaReadMem            | ✅   | Boxa::read_from_bytes     |                            |
| boxaWriteDebug         | 🚫   | -                         | デバッグ出力関数           |
| boxaWrite              | ✅   | Boxa::write_to_file       |                            |
| boxaWriteStream        | ✅   | Boxa::write_to_writer     |                            |
| boxaWriteStderr        | 🚫   | -                         | デバッグ出力関数           |
| boxaWriteMem           | ✅   | Boxa::write_to_bytes      |                            |
| boxPrintStreamInfo     | 🚫   | -                         | デバッグ出力関数           |

### boxfunc1.c (Box関係・幾何演算)

#### その他

| C関数             | 状態 | Rust対応              | 備考 |
| ----------------- | ---- | --------------------- | ---- |
| boxContains       | ✅   | Box::contains_box()   |      |
| boxIntersects     | ✅   | Box::overlaps()       |      |
| boxOverlapRegion  | ✅   | Box::intersect()      |      |
| boxBoundingRegion | ✅   | Box::union()          |      |
| boxContainsPt     | ✅   | Box::contains_point() |      |
| boxEqual          | ✅   | PartialEq trait       |      |

#### mod.rs

| C関数               | 状態 | Rust対応           | 備考 |
| ------------------- | ---- | ------------------ | ---- |
| boxaContainedInBox  | ✅   | contained_in_box() |      |
| boxaIntersectsBox   | ✅   | intersects_box()   |      |
| boxaClipToBox       | ✅   | clip_to_box()      |      |
| boxaCombineOverlaps | ✅   | combine_overlaps() |      |
| boxOverlapFraction  | ✅   | overlap_fraction() |      |
| boxOverlapArea      | ✅   | overlap_area()     |      |
| boxGetCenter        | ✅   | center()           |      |
| boxClipToRectangle  | ✅   | clip()             |      |
| boxaSimilar         | ✅   | similar()          |      |
| boxaJoin            | ✅   | join()             |      |

#### geometry.rs

| C関数                     | 状態 | Rust対応                   | 備考 |
| ------------------------- | ---- | -------------------------- | ---- |
| boxaContainedInBoxCount   | ✅   | contained_in_box_count()   |      |
| boxaContainedInBoxa       | ✅   | all_contained_in()         |      |
| boxaIntersectsBoxCount    | ✅   | intersects_box_count()     |      |
| boxaCombineOverlapsInPair | ✅   | combine_overlaps_in_pair() |      |
| boxaHandleOverlaps        | ✅   | handle_overlaps()          |      |
| boxOverlapDistance        | ✅   | overlap_distance()         |      |
| boxSeparationDistance     | ✅   | separation_distance()      |      |
| boxCompareSize            | ✅   | compare_size()             |      |
| boxaGetNearestToPt        | ✅   | nearest_to_point()         |      |
| boxaGetNearestToLine      | ✅   | nearest_to_line()          |      |
| boxaFindNearestBoxes      | ✅   | find_nearest_boxes()       |      |
| boxaGetNearestByDirection | ✅   | nearest_by_direction()     |      |
| boxIntersectByLine        | ✅   | intersect_by_line()        |      |
| boxClipToRectangleParams  | ✅   | clip_to_rectangle_params() |      |

#### adjust.rs

| C関数                    | 状態 | Rust対応                  | 備考 |
| ------------------------ | ---- | ------------------------- | ---- |
| boxRelocateOneSide       | ✅   | relocate_one_side()       |      |
| boxaAdjustSides          | ✅   | adjust_all_sides()        |      |
| boxaAdjustBoxSides       | ✅   | adjust_box_sides()        |      |
| boxAdjustSides           | ✅   | adjust_sides()            |      |
| boxaSetSide              | ✅   | set_all_sides()           |      |
| boxSetSide               | ✅   | set_side()                |      |
| boxaAdjustWidthToTarget  | ✅   | adjust_width_to_target()  |      |
| boxaAdjustHeightToTarget | ✅   | adjust_height_to_target() |      |
| boxaEqual                | ✅   | equal_ordered()           |      |
| boxSimilar               | ✅   | similar_per_side()        |      |
| boxaaJoin                | ✅   | join() (Boxaa)            |      |
| boxaSplitEvenOdd         | ✅   | split_even_odd()          |      |
| boxaMergeEvenOdd         | ✅   | merge_even_odd()          |      |

### boxfunc2.c (Box変換ユーティリティ)

| C関数                  | 状態 | Rust対応                                  | 備考                            |
| ---------------------- | ---- | ----------------------------------------- | ------------------------------- |
| boxaTransform          | 🔄   | Boxa::translate() + Boxa::scale()         | shift/scaleを個別メソッドに分離 |
| boxTransform           | 🔄   | Box::translate() + Box::scale()           | shift/scaleを個別メソッドに分離 |
| boxaTransformOrdered   | ✅   | Boxa::transform_ordered                   |                                 |
| boxTransformOrdered    | ✅   | Box::transform_ordered                    |                                 |
| boxaRotateOrth         | ✅   | Boxa::rotate_orth                         |                                 |
| boxRotateOrth          | ✅   | Box::rotate_orth                          |                                 |
| boxaShiftWithPta       | ✅   | Boxa::shift_with_pta                      |                                 |
| boxaSort               | 🔄   | Boxa::sort_by_position() / sort_by_area() | ソートタイプ別に個別メソッド化  |
| boxaBinSort            | ✅   | Boxa::bin_sort                            |                                 |
| boxaSortByIndex        | ✅   | Boxa::sort_by_index                       |                                 |
| boxaSort2d             | ✅   | Boxa::sort_2d                             |                                 |
| boxaSort2dByIndex      | ✅   | Boxa::sort_2d_by_index                    |                                 |
| boxaExtractAsNuma      | ✅   | Boxa::extract_as_numa                     |                                 |
| boxaExtractAsPta       | ✅   | Boxa::extract_as_pta                      |                                 |
| boxaExtractCorners     | ✅   | Boxa::extract_corners                     |                                 |
| boxaGetRankVals        | ✅   | Boxa::get_rank_vals                       |                                 |
| boxaGetMedianVals      | ✅   | Boxa::get_median_vals                     |                                 |
| boxaGetAverageSize     | ✅   | Boxa::get_average_size                    |                                 |
| boxaaGetExtent         | ✅   | Boxaa::get_extent                         |                                 |
| boxaaFlattenToBoxa     | ✅   | Boxaa::flatten()                          |                                 |
| boxaaFlattenAligned    | ✅   | Boxaa::flatten_aligned                    |                                 |
| boxaEncapsulateAligned | ✅   | Boxa::encapsulate_aligned                 |                                 |
| boxaaTranspose         | ✅   | Boxaa::transpose                          |                                 |
| boxaaAlignBox          | ✅   | Boxaa::align_box                          |                                 |

### boxfunc3.c (Box描画・マスク)

#### その他

| C関数                     | 状態 | Rust対応                  | 備考         |
| ------------------------- | ---- | ------------------------- | ------------ |
| pixMaskConnComp           | ✅   | mask_conn_comp            | conncomp依存 |
| boxaaDisplay              | ✅   | Boxaa::display            |              |
| pixaDisplayBoxaa          | ✅   | Pixa::display_boxaa       |              |
| pixSplitIntoBoxa          | ✅   | split_into_boxa           |              |
| pixSplitComponentIntoBoxa | ✅   | split_component_into_boxa |              |
| makeMosaicStrips          | ✅   | make_mosaic_strips        |              |
| pixSelectLargeULComp      | ✅   | select_large_ul_comp      | conncomp依存 |

#### draw.rs

| C関数                  | 状態 | Rust対応              | 備考 |
| ---------------------- | ---- | --------------------- | ---- |
| pixMaskBoxa            | ✅   | mask_boxa()           |      |
| pixPaintBoxa           | ✅   | paint_boxa()          |      |
| pixSetBlackOrWhiteBoxa | ✅   | set_bw_boxa()         |      |
| pixPaintBoxaRandom     | ✅   | paint_boxa_random()   |      |
| pixBlendBoxaRandom     | ✅   | blend_boxa_random()   |      |
| pixDrawBoxa            | ✅   | draw_boxa()           |      |
| pixDrawBoxaRandom      | ✅   | draw_boxa_random()    |      |
| boxaCompareRegions     | ✅   | compare_regions()     |      |
| boxaSelectLargeULBox   | ✅   | select_large_ul_box() |      |

### boxfunc4.c (Box選択・変換)

#### select.rs

| C関数                    | 状態 | Rust対応                  | 備考 |
| ------------------------ | ---- | ------------------------- | ---- |
| boxaSelectRange          | ✅   | select_range()            |      |
| boxaaSelectRange         | ✅   | select_range() (Boxaa)    |      |
| boxaMakeSizeIndicator    | ✅   | make_size_indicator()     |      |
| boxaMakeAreaIndicator    | ✅   | make_area_indicator()     |      |
| boxaMakeWHRatioIndicator | ✅   | make_wh_ratio_indicator() |      |
| boxaSelectWithIndicator  | ✅   | select_with_indicator()   |      |
| boxaSwapBoxes            | ✅   | swap_boxes()              |      |
| boxaaSizeRange           | ✅   | size_range() (Boxaa)      |      |
| boxaLocationRange        | ✅   | location_range()          |      |
| boxaGetSizes             | ✅   | get_sizes()               |      |
| boxaGetArea              | ✅   | get_total_area()          |      |

#### mod.rs

| C関数               | 状態 | Rust対応             | 備考 |
| ------------------- | ---- | -------------------- | ---- |
| boxaSelectBySize    | ✅   | select_by_size()     |      |
| boxaSelectByArea    | ✅   | select_by_area()     |      |
| boxaSelectByWHRatio | ✅   | select_by_wh_ratio() |      |
| boxaGetExtent       | ✅   | get_extent()         |      |
| boxaGetCoverage     | ✅   | get_coverage()       |      |
| boxaSizeRange       | ✅   | size_range()         |      |

#### その他

| C関数                   | 状態 | Rust対応                   | 備考 |
| ----------------------- | ---- | -------------------------- | ---- |
| boxaPermutePseudorandom | ✅   | Boxa::permute_pseudorandom |      |
| boxaPermuteRandom       | ✅   | Boxa::permute_random       |      |
| boxaDisplayTiled        | ✅   | Boxa::display_tiled        |      |

#### adjust.rs

| C関数            | 状態 | Rust対応        | 備考 |
| ---------------- | ---- | --------------- | ---- |
| boxaConvertToPta | ✅   | to_pta() (Boxa) |      |
| ptaConvertToBoxa | ✅   | to_boxa()       |      |
| boxConvertToPta  | ✅   | to_pta() (Box)  |      |
| ptaConvertToBox  | ✅   | to_box()        |      |

### boxfunc5.c (Boxスムージング・調整)

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

| C関数             | 状態 | Rust対応                 | 備考                 |
| ----------------- | ---- | ------------------------ | -------------------- |
| ptaCreate         | ✅   | Pta::new()               |                      |
| ptaCreateFromNuma | ✅   | Pta::create_from_numa    |                      |
| ptaDestroy        | 🔄   | drop()                   | 自動                 |
| ptaCopy           | ✅   | Pta::clone()             |                      |
| ptaCopyRange      | ✅   | Pta::copy_range          |                      |
| ptaClone          | ✅   | Pta::clone()             |                      |
| ptaEmpty          | 🚫   | -                        | Pta::clear()で対応   |
| ptaAddPt          | ✅   | Pta::push()              |                      |
| ptaInsertPt       | ✅   | Pta::insert              |                      |
| ptaRemovePt       | ✅   | Pta::remove_pt           |                      |
| ptaGetCount       | ✅   | Pta::len()               |                      |
| ptaGetPt          | ✅   | Pta::get()               |                      |
| ptaGetIPt         | ✅   | Pta::get_i_pt            |                      |
| ptaSetPt          | ✅   | Pta::set()               |                      |
| ptaGetArrays      | 🚫   | -                        | Cポインタ配列        |
| ptaRead           | ✅   | Pta::read_from_file      |                      |
| ptaReadStream     | ✅   | Pta::read_from_reader    |                      |
| ptaReadMem        | ✅   | Pta::read_from_bytes     |                      |
| ptaWriteDebug     | 🚫   | -                        | デバッグ出力関数     |
| ptaWrite          | ✅   | Pta::write_to_file       |                      |
| ptaWriteStream    | ✅   | Pta::write_to_writer     |                      |
| ptaWriteMem       | ✅   | Pta::write_to_bytes      |                      |
| ptaaCreate        | ✅   | Ptaa::new()              | Ptaa構造体として実装 |
| ptaaDestroy       | 🔄   | drop()                   | Drop traitで自動     |
| ptaaAddPta        | ✅   | Ptaa::push()             |                      |
| ptaaGetCount      | ✅   | Ptaa::len()              |                      |
| ptaaGetPta        | ✅   | Ptaa::get()              |                      |
| ptaaGetPt         | 🚫   | -                        | Vec<Pta>で代替       |
| ptaaInitFull      | ✅   | Ptaa::init_full()        |                      |
| ptaaReplacePta    | ✅   | Ptaa::replace()          |                      |
| ptaaAddPt         | ✅   | Ptaa::add_pt()           |                      |
| ptaaTruncate      | ✅   | Ptaa::truncate()         |                      |
| ptaaRead          | ✅   | Ptaa::read_from_file()   |                      |
| ptaaReadStream    | ✅   | Ptaa::read_from_reader() |                      |
| ptaaReadMem       | ✅   | Ptaa::read_from_bytes()  |                      |
| ptaaWriteDebug    | 🚫   | -                        | Vec<Pta>で代替       |
| ptaaWrite         | ✅   | Ptaa::write_to_file()    |                      |
| ptaaWriteStream   | ✅   | Ptaa::write_to_writer()  |                      |
| ptaaWriteMem      | ✅   | Ptaa::write_to_bytes()   |                      |

### ptafunc1.c, ptafunc2.c (Pta変換・演算)

Phase 16で大部分を実装済み。

#### transform.rs

| C関数               | 状態 | Rust対応            | 備考 |
| ------------------- | ---- | ------------------- | ---- |
| ptaSubsample        | ✅   | subsample()         |      |
| ptaJoin             | ✅   | join()              |      |
| ptaReverse          | ✅   | reverse()           |      |
| ptaTranspose        | ✅   | transpose()         |      |
| ptaCyclicPerm       | ✅   | cyclic_perm()       |      |
| ptaSelectRange      | ✅   | select_range()      |      |
| ptaGetRange         | ✅   | get_range()         |      |
| ptaGetInsideBox     | ✅   | get_inside_box()    |      |
| ptaContainsPt       | ✅   | contains_pt()       |      |
| ptaTestIntersection | ✅   | test_intersection() |      |
| ptaTransform        | ✅   | transform_pts()     |      |
| ptaPtInsidePolygon  | ✅   | pt_inside_polygon() |      |
| ptaPolygonIsConvex  | ✅   | polygon_is_convex() |      |
| ptaGetMinMax        | ✅   | get_min_max()       |      |
| ptaSelectByValue    | ✅   | select_by_value()   |      |

#### その他

| C関数         | 状態 | Rust対応          | 備考 |
| ------------- | ---- | ----------------- | ---- |
| ptaaJoin      | ✅   | Ptaa::join()      |      |
| ptaCropToMask | ✅   | Pta::crop_to_mask |      |

#### lsf.rs

| C関数              | 状態 | Rust対応            | 備考 |
| ------------------ | ---- | ------------------- | ---- |
| ptaGetLinearLSF    | ✅   | get_linear_lsf()    |      |
| ptaGetQuadraticLSF | ✅   | get_quadratic_lsf() |      |
| ptaGetCubicLSF     | ✅   | get_cubic_lsf()     |      |
| ptaGetQuarticLSF   | ✅   | get_quartic_lsf()   |      |

#### sort.rs

| C関数           | 状態 | Rust対応         | 備考 |
| --------------- | ---- | ---------------- | ---- |
| ptaSortByIndex  | ✅   | sort_by_index()  |      |
| ptaGetSortIndex | ✅   | get_sort_index() |      |
| ptaSort         | ✅   | sort_pta()       |      |
| ptaGetRankValue | ✅   | get_rank_value() |      |
| ptaSort2d       | ✅   | sort_2d()        |      |
| ptaEqual        | ✅   | equal()          |      |

### pixabasic.c (Pixa基本操作)

| C関数                 | 状態 | Rust対応                  | 備考                   |
| --------------------- | ---- | ------------------------- | ---------------------- |
| pixaCreate            | ✅   | Pixa::new()               |                        |
| pixaCreateFromPix     | ✅   | Pixa::create_from_pix     |                        |
| pixaCreateFromBoxa    | ✅   | Pixa::create_from_boxa    |                        |
| pixaSplitPix          | ✅   | Pixa::split_pix           |                        |
| pixaDestroy           | 🔄   | drop()                    | 自動                   |
| pixaCopy              | ✅   | Pixa::clone()             |                        |
| pixaAddPix            | ✅   | Pixa::push()              |                        |
| pixaAddBox            | ✅   | Pixa::push_with_box()     |                        |
| pixaExtendArray       | 🚫   | -                         | Vec自動拡張            |
| pixaExtendArrayToSize | 🚫   | -                         | Vec自動拡張            |
| pixaGetCount          | ✅   | Pixa::len()               |                        |
| pixaGetPix            | ✅   | Pixa::get_cloned()        |                        |
| pixaGetPixDimensions  | ✅   | Pixa::get_dimensions()    |                        |
| pixaGetBoxa           | ✅   | Pixa::get_boxa            |                        |
| pixaGetBoxaCount      | ✅   | Pixa::get_boxa_count      |                        |
| pixaGetBox            | ✅   | Pixa::get_box             |                        |
| pixaGetBoxGeometry    | ✅   | Pixa::get_box_geometry    |                        |
| pixaSetBoxa           | ✅   | Pixa::set_boxa            |                        |
| pixaGetPixArray       | 🚫   | -                         | Cポインタ配列          |
| pixaVerifyDepth       | 🚫   | -                         | Rustの型システムで不要 |
| pixaVerifyDimensions  | 🚫   | -                         | Rustの型システムで不要 |
| pixaIsFull            | 🚫   | -                         | Rustの型システムで不要 |
| pixaCountText         | ✅   | Pixa::count_text          |                        |
| pixaSetText           | ✅   | Pixa::set_text            |                        |
| pixaGetLinePtrs       | 🚫   | -                         | Cポインタ配列          |
| pixaWriteStreamInfo   | 🚫   | -                         | デバッグ出力関数       |
| pixaReplacePix        | ✅   | Pixa::replace_pix         |                        |
| pixaInsertPix         | ✅   | Pixa::insert_pix          |                        |
| pixaRemovePix         | ✅   | Pixa::remove_pix          |                        |
| pixaRemovePixAndSave  | ✅   | Pixa::remove_pix_and_save |                        |
| pixaRemoveSelected    | ✅   | Pixa::remove_selected     |                        |
| pixaInitFull          | ✅   | Pixa::init_full           |                        |
| pixaClear             | ✅   | Pixa::clear()             |                        |
| pixaJoin              | ✅   | Pixa::join                |                        |
| pixaInterleave        | ✅   | Pixa::interleave          |                        |
| pixaaJoin             | ✅   | Pixaa::join()             |                        |
| pixaaCreate           | ✅   | Pixaa::new()              | Pixaa構造体として実装  |
| pixaaCreateFromPixa   | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaDestroy          | 🔄   | drop()                    | Drop traitで自動       |
| pixaaAddPixa          | ✅   | Pixaa::push()             |                        |
| pixaaExtendArray      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaAddPix           | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaAddBox           | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaGetCount         | ✅   | Pixaa::len()              |                        |
| pixaaGetPixa          | ✅   | Pixaa::get()              |                        |
| pixaaGetBoxa          | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaGetPix           | ✅   | Pixaa::get_pix()          |                        |
| pixaaVerifyDepth      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaVerifyDimensions | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaIsFull           | ✅   | Pixaa::is_full()          |                        |
| pixaaInitFull         | ✅   | Pixaa::init_full()        |                        |
| pixaaReplacePixa      | ✅   | Pixaa::replace()          |                        |
| pixaaClear            | ✅   | Pixaa::clear()            |                        |
| pixaaTruncate         | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaRead              | ✅   | Pixa::read_from_file      |                        |
| pixaReadStream        | ✅   | Pixa::read_from_reader    |                        |
| pixaReadMem           | ✅   | Pixa::read_from_bytes     |                        |
| pixaWriteDebug        | 🚫   | -                         | デバッグ出力関数       |
| pixaWrite             | ✅   | Pixa::write_to_file       |                        |
| pixaWriteStream       | ✅   | Pixa::write_to_writer     |                        |
| pixaWriteMem          | ✅   | Pixa::write_to_bytes      |                        |
| pixaReadBoth          | ✅   | Pixa::read_both           |                        |
| pixaaReadFromFiles    | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaRead             | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaReadStream       | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaReadMem          | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWrite            | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWriteStream      | 🚫   | -                         | Vec<Pixa>で代替        |
| pixaaWriteMem         | 🚫   | -                         | Vec<Pixa>で代替        |

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

#### operations.rs

| C関数                      | 状態 | Rust対応                     | 備考 |
| -------------------------- | ---- | ---------------------------- | ---- |
| numaArithOp                | ✅   | arith_op()                   |      |
| numaLogicalOp              | ✅   | logical_op()                 |      |
| numaInvert                 | ✅   | invert()                     |      |
| numaSimilar                | ✅   | similar()                    |      |
| numaAddToNumber            | ✅   | add_to_element()             |      |
| numaGetPartialSums         | ✅   | partial_sums()               |      |
| numaSubsample              | ✅   | subsample()                  |      |
| numaMakeDelta              | ✅   | make_delta()                 |      |
| numaMakeSequence           | ✅   | make_sequence()              |      |
| numaMakeAbsval             | ✅   | abs_val()                    |      |
| numaAddBorder              | ✅   | add_border()                 |      |
| numaAddSpecifiedBorder     | ✅   | add_specified_border()       |      |
| numaRemoveBorder           | ✅   | remove_border()              |      |
| numaCountNonzeroRuns       | ✅   | count_nonzero_runs()         |      |
| numaGetNonzeroRange        | ✅   | get_nonzero_range()          |      |
| numaGetCountRelativeToZero | ✅   | get_count_relative_to_zero() |      |
| numaClipToInterval         | ✅   | clip_to_interval()           |      |
| numaMakeThresholdIndicator | ✅   | make_threshold_indicator()   |      |
| numaInterpolateEqxVal      | ✅   | interpolate_eqx_val()        |      |
| numaInterpolateArbxVal     | ✅   | interpolate_arbx_val()       |      |
| numaSortAutoSelect         | ✅   | sort_auto_select()           |      |
| numaSortIndexAutoSelect    | ✅   | sort_index_auto_select()     |      |
| numaGetSortIndex           | ✅   | sort_index()                 |      |
| numaSortByIndex            | ✅   | sort_by_index()              |      |
| numaIsSorted               | ✅   | is_sorted()                  |      |
| numaJoin                   | ✅   | join()                       |      |

#### その他

| C関数                | 状態 | Rust対応                           | 備考                   |
| -------------------- | ---- | ---------------------------------- | ---------------------- |
| numaGetMin           | ✅   | Numa::min()                        |                        |
| numaGetMax           | ✅   | Numa::max()                        |                        |
| numaGetSum           | ✅   | Numa::sum()                        |                        |
| numaGetSumOnInterval | ✅   | Numa::sum_on_interval()            |                        |
| numaHasOnlyIntegers  | ✅   | Numa::has_only_integers()          |                        |
| numaGetMean          | ✅   | Numa::mean()                       |                        |
| numaGetMeanAbsval    | ✅   | Numa::mean_absval()                |                        |
| numaMakeConstant     | ✅   | Numa::make_constant()              |                        |
| numaReverse          | ✅   | Numa::reversed() / Numa::reverse() |                        |
| numaSortGeneral      | ✅   | Numa::sort_general                 | sort_auto_selectで統合 |
| numaChooseSortType   | ✅   | Numa::choose_sort_type             | 内部関数               |
| numaSort             | ✅   | Numa::sorted() / Numa::sort()      |                        |
| numaGetRankValue     | ✅   | Numa::rank_value()                 |                        |
| numaGetMedian        | ✅   | Numa::median()                     |                        |
| numaGetMode          | ✅   | Numa::mode()                       |                        |
| numaaJoin            | ✅   | Numaa::join                        |                        |
| numaaFlattenToNuma   | ✅   | Numaa::flatten()                   |                        |

#### interpolation.rs

| C関数                       | 状態 | Rust対応                    | 備考 |
| --------------------------- | ---- | --------------------------- | ---- |
| numaUniformSampling         | ✅   | uniform_sampling()          |      |
| numaLowPassIntervals        | ✅   | low_pass_intervals()        |      |
| numaThresholdEdges          | ✅   | threshold_edges()           |      |
| numaGetSpanValues           | ✅   | get_span_values()           |      |
| numaGetEdgeValues           | ✅   | get_edge_values()           |      |
| numaInterpolateEqxInterval  | ✅   | interpolate_eqx_interval()  |      |
| numaInterpolateArbxInterval | ✅   | interpolate_arbx_interval() |      |
| numaFitMax                  | ✅   | fit_max()                   |      |
| numaDifferentiateInterval   | ✅   | differentiate_interval()    |      |
| numaIntegrateInterval       | ✅   | integrate_interval()        |      |

#### sort.rs

| C関数                      | 状態 | Rust対応                 | 備考 |
| -------------------------- | ---- | ------------------------ | ---- |
| numaBinSort                | ✅   | bin_sort()               |      |
| numaGetBinSortIndex        | ✅   | bin_sort_index()         |      |
| numaSortPair               | ✅   | sort_pair()              |      |
| numaInvertMap              | ✅   | invert_map()             |      |
| numaAddSorted              | ✅   | add_sorted()             |      |
| numaFindSortedLoc          | ✅   | find_sorted_loc()        |      |
| numaPseudorandomSequence   | ✅   | pseudorandom_sequence()  |      |
| numaRandomPermutation      | ✅   | random_permutation()     |      |
| numaGetBinnedMedian        | ✅   | binned_median()          |      |
| numaGetMeanDevFromMedian   | ✅   | mean_dev_from_median()   |      |
| numaGetMedianDevFromMedian | ✅   | median_dev_from_median() |      |

numafunc2.c (ヒストグラム・統計)の関数も実装済み。
一部ヒストグラム関数はnuma/histogram.rsに実装あり。

### sarray1.c, sarray2.c (Sarray文字列配列)

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
| sarrayRead                  | ✅   | Sarray::read_from_file         |                  |
| sarrayReadStream            | ✅   | Sarray::read_from_reader       |                  |
| sarrayReadMem               | ✅   | Sarray::read_from_bytes        |                  |
| sarrayWrite                 | ✅   | Sarray::write_to_file          |                  |
| sarrayWriteStream           | ✅   | Sarray::write_to_writer        |                  |
| sarrayWriteStderr           | 🚫   | -                              | デバッグ出力関数 |
| sarrayWriteMem              | ✅   | Sarray::write_to_bytes         |                  |
| sarrayAppend                | ✅   | Sarray::append                 |                  |
| sarraySort                  | ✅   | Sarray::sort()                 |                  |
| sarraySortByIndex           | ✅   | Sarray::sort_by_index          |                  |

その他のsarray2.c関数（セット演算、整数生成など）も実装済み。

### fpix1.c, fpix2.c (FPix浮動小数点画像)

| C関数                  | 状態 | Rust対応                | 備考                   |
| ---------------------- | ---- | ----------------------- | ---------------------- |
| fpixCreate             | ✅   | FPix::new()             |                        |
| fpixCreateTemplate     | ✅   | FPix::create_template() |                        |
| fpixClone              | ✅   | FPix::clone()           |                        |
| fpixCopy               | ✅   | FPix::clone()           |                        |
| fpixDestroy            | 🔄   | drop()                  | 自動                   |
| fpixGetDimensions      | ✅   | width()/height()        |                        |
| fpixSetDimensions      | 🚫   | -                       | FPixは不変             |
| fpixGetWpl             | 🚫   | -                       | FPixはwpl概念なし      |
| fpixSetWpl             | 🚫   | -                       | FPixはwpl概念なし      |
| fpixGetResolution      | ✅   | xres()/yres()           |                        |
| fpixSetResolution      | ✅   | set_resolution()        |                        |
| fpixCopyResolution     | 🚫   | -                       | set_resolution()で対応 |
| fpixGetData            | ✅   | FPix::data()            |                        |
| fpixSetData            | 🚫   | -                       | Cメモリ管理            |
| fpixGetPixel           | ✅   | FPix::get_pixel()       |                        |
| fpixSetPixel           | ✅   | FPix::set_pixel()       |                        |
| fpixaCreate            | ✅   | FPixa::new()            |                        |
| fpixaCopy              | ✅   | FPixa::clone()          |                        |
| fpixaDestroy           | 🚫   | -                       | drop()で自動           |
| fpixaAddFPix           | ✅   | FPixa::push()           |                        |
| fpixaGetCount          | ✅   | FPixa::len()            |                        |
| fpixaGetFPix           | ✅   | FPixa::get()            |                        |
| fpixaGetFPixDimensions | ✅   | FPixa::get_dimensions() |                        |
| fpixaGetData           | ✅   | FPixa::get_data()       |                        |
| fpixaGetPixel          | ✅   | FPixa::get_pixel()      |                        |
| fpixaSetPixel          | ✅   | FPixa::set_pixel()      |                        |
| dpixCreate             | ✅   | DPix::new()             |                        |
| dpixClone              | ✅   | DPix::clone()           |                        |
| dpixCopy               | ✅   | DPix::clone()           |                        |
| dpixDestroy            | 🔄   | drop()                  | 自動                   |
| fpixRead               | ✅   | FPix::read_from_file    |                        |
| fpixReadStream         | ✅   | FPix::read_from_reader  |                        |
| fpixReadMem            | ✅   | FPix::read_from_bytes   |                        |
| fpixWrite              | ✅   | FPix::write_to_file     |                        |
| fpixWriteStream        | ✅   | FPix::write_to_writer   |                        |
| fpixWriteMem           | ✅   | FPix::write_to_bytes    |                        |
| dpixRead               | ✅   | dpix_read               |                        |
| dpixWrite              | ✅   | dpix_write              |                        |

fpix2.c (FPix変換・演算):

| C関数                 | 状態 | Rust対応                                    | 備考          |
| --------------------- | ---- | ------------------------------------------- | ------------- |
| fpixConvertToPix      | ✅   | FPix::to_pix()                              |               |
| pixConvertToFPix      | ✅   | FPix::from_pix()                            |               |
| fpixAddMultConstant   | 🔄   | FPix::add_constant() + FPix::mul_constant() | 2段階呼び出し |
| fpixLinearCombination | ✅   | FPix::linear_combination()                  |               |
| dpixConvertToPix      | ✅   | DPix::to_pix()                              |               |
| dpixConvertToFPix     | ✅   | DPix::to_fpix()                             |               |

その他のfpix2.c変換関数は一部convert.rsに実装あり。

### colormap.c (カラーマップ)

| C関数                        | 状態 | Rust対応                             | 備考                |
| ---------------------------- | ---- | ------------------------------------ | ------------------- |
| pixcmapCreate                | ✅   | PixColormap::new()                   |                     |
| pixcmapCreateRandom          | ✅   | PixColormap::create_random           |                     |
| pixcmapCreateLinear          | ✅   | PixColormap::create_linear()         |                     |
| pixcmapCopy                  | ✅   | PixColormap::clone()                 |                     |
| pixcmapDestroy               | 🔄   | drop()                               | 自動                |
| pixcmapIsValid               | ✅   | PixColormap::is_valid                |                     |
| pixcmapAddColor              | ✅   | PixColormap::add_color()             |                     |
| pixcmapAddRGBA               | ✅   | PixColormap::add_rgba                | add_colorがRGBA対応 |
| pixcmapAddNewColor           | ✅   | PixColormap::add_new_color           |                     |
| pixcmapAddNearestColor       | ✅   | PixColormap::add_nearest_color       |                     |
| pixcmapUsableColor           | ✅   | PixColormap::is_usable_color         |                     |
| pixcmapAddBlackOrWhite       | ✅   | PixColormap::add_black_or_white      |                     |
| pixcmapSetBlackAndWhite      | ✅   | PixColormap::set_black_and_white     |                     |
| pixcmapGetCount              | ✅   | PixColormap::len()                   |                     |
| pixcmapGetFreeCount          | ✅   | PixColormap::free_count              |                     |
| pixcmapGetDepth              | ✅   | PixColormap::depth()                 |                     |
| pixcmapGetMinDepth           | ✅   | PixColormap::min_depth               |                     |
| pixcmapClear                 | ✅   | PixColormap::clear()                 |                     |
| pixcmapGetColor              | ✅   | PixColormap::get_rgb()               |                     |
| pixcmapGetColor32            | ✅   | PixColormap::get_color32             |                     |
| pixcmapGetRGBA               | ✅   | PixColormap::get_rgba                |                     |
| pixcmapGetRGBA32             | ✅   | PixColormap::get_rgba32              |                     |
| pixcmapResetColor            | ✅   | PixColormap::reset_color             |                     |
| pixcmapSetAlpha              | ✅   | PixColormap::set_alpha               |                     |
| pixcmapGetIndex              | ✅   | PixColormap::get_index               |                     |
| pixcmapHasColor              | ✅   | PixColormap::has_color               |                     |
| pixcmapIsOpaque              | ✅   | PixColormap::is_opaque               |                     |
| pixcmapNonOpaqueColorsInfo   | ✅   | PixColormap::non_opaque_info         |                     |
| pixcmapIsBlackAndWhite       | ✅   | PixColormap::is_black_and_white      |                     |
| pixcmapCountGrayColors       | ✅   | PixColormap::count_gray_colors       |                     |
| pixcmapGetRankIntensity      | ✅   | PixColormap::get_rank_intensity      |                     |
| pixcmapGetNearestIndex       | ✅   | PixColormap::find_nearest            |                     |
| pixcmapGetNearestGrayIndex   | ✅   | PixColormap::find_nearest_gray       |                     |
| pixcmapGetDistanceToColor    | ✅   | PixColormap::distance_to_color       |                     |
| pixcmapGetRangeValues        | ✅   | PixColormap::get_range_values        |                     |
| pixcmapGrayToFalseColor      | ✅   | PixColormap::gray_to_false_color     |                     |
| pixcmapGrayToColor           | ✅   | PixColormap::gray_to_color           |                     |
| pixcmapColorToGray           | ✅   | PixColormap::color_to_gray           |                     |
| pixcmapConvertTo4            | ✅   | PixColormap::convert_to4             |                     |
| pixcmapConvertTo8            | ✅   | PixColormap::convert_to8             |                     |
| pixcmapRead                  | ✅   | PixColormap::read_from_file          |                     |
| pixcmapReadStream            | ✅   | PixColormap::read_from_reader        |                     |
| pixcmapReadMem               | ✅   | PixColormap::read_from_bytes         |                     |
| pixcmapWrite                 | ✅   | PixColormap::write_to_file           |                     |
| pixcmapWriteStream           | ✅   | PixColormap::write_to_writer         |                     |
| pixcmapWriteMem              | ✅   | PixColormap::write_to_bytes          |                     |
| pixcmapToArrays              | ✅   | PixColormap::to_arrays               |                     |
| pixcmapToRGBTable            | ✅   | PixColormap::to_rgb_table            |                     |
| pixcmapSerializeToMemory     | ✅   | PixColormap::serialize_to_memory     |                     |
| pixcmapDeserializeFromMemory | ✅   | PixColormap::deserialize_from_memory |                     |
| pixcmapConvertToHex          | ✅   | PixColormap::convert_to_hex          |                     |
| pixcmapGammaTRC              | ✅   | PixColormap::gamma_trc               |                     |
| pixcmapContrastTRC           | ✅   | PixColormap::contrast_trc            |                     |
| pixcmapShiftIntensity        | ✅   | PixColormap::shift_intensity         |                     |
| pixcmapShiftByComponent      | ✅   | PixColormap::shift_by_component      |                     |

### pixconv.c (ピクセル深度変換)

convert.rsに実装済み。全関数が実装されている。

#### その他

| C関数                        | 状態 | Rust対応                                             | 備考                                    |
| ---------------------------- | ---- | ---------------------------------------------------- | --------------------------------------- |
| pixThreshold8                | ✅   | threshold_8                                          |                                         |
| pixConvertRGBToBinaryArb     | ✅   | convert_rgb_to_binary_arb                            | color crate依存                         |
| pixConvertRGBToColormap      | ✅   | convert_rgb_to_colormap                              | color crate依存                         |
| pixQuantizeIfFewColors       | ✅   | quantize_if_few_colors                               | color crate依存                         |
| pixConvertTo1Adaptive        | ✅   | convert_to1_adaptive                                 |                                         |
| pixConvertTo1                | 🔄   | convert_to_1_adaptive() / convert_to_1_by_sampling() | 汎用ディスパッチャを2つの専用関数に分割 |
| pixConvertTo1BySampling      | ✅   | convert_to1_by_sampling                              |                                         |
| pixConvertTo8BySampling      | ✅   | convert_to8_by_sampling                              | transform crate依存                     |
| pixConvertTo8Colormap        | ✅   | convert_to8_colormap                                 | 32bpp部分は後続                         |
| pixConvertTo32BySampling     | ✅   | convert_to32_by_sampling                             | transform crate依存                     |
| pixConvert24To32             | ✅   | convert24_to32                                       |                                         |
| pixConvert32To24             | ✅   | convert32_to24                                       |                                         |
| pixConvertToSubpixelRGB      | ✅   | convert_to_subpixel_rgb                              |                                         |
| pixConvertGrayToSubpixelRGB  | ✅   | convert_gray_to_subpixel_rgb                         |                                         |
| pixConvertColorToSubpixelRGB | ✅   | convert_color_to_subpixel_rgb                        |                                         |

#### convert.rs

| C関数                       | 状態 | Rust対応                        | 備考                            |
| --------------------------- | ---- | ------------------------------- | ------------------------------- |
| pixRemoveColormapGeneral    | ✅   | remove_colormap()               | pixRemoveColormapと同一メソッド |
| pixRemoveColormap           | ✅   | remove_colormap()               |                                 |
| pixAddGrayColormap8         | ✅   | add_gray_colormap8()            |                                 |
| pixAddMinimalGrayColormap8  | ✅   | add_minimal_gray_colormap8()    |                                 |
| pixConvertRGBToLuminance    | ✅   | convert_rgb_to_luminance()      |                                 |
| pixConvertRGBToGrayGeneral  | ✅   | convert_rgb_to_gray_general()   |                                 |
| pixConvertRGBToGray         | ✅   | convert_rgb_to_gray()           |                                 |
| pixConvertRGBToGrayFast     | ✅   | convert_rgb_to_gray_fast()      |                                 |
| pixConvertRGBToGrayMinMax   | ✅   | convert_rgb_to_gray_min_max()   |                                 |
| pixConvertRGBToGraySatBoost | ✅   | convert_rgb_to_gray_sat_boost() |                                 |
| pixConvertRGBToGrayArb      | ✅   | convert_rgb_to_gray_arb()       |                                 |
| pixConvertGrayToColormap    | ✅   | convert_gray_to_colormap()      |                                 |
| pixConvertGrayToColormap8   | ✅   | convert_gray_to_colormap_8()    |                                 |
| pixColorizeGray             | ✅   | colorize_gray()                 |                                 |
| pixConvertCmapTo1           | ✅   | convert_cmap_to_1()             |                                 |
| pixConvert16To8             | ✅   | convert_16_to_8()               |                                 |
| pixConvertGrayToFalseColor  | ✅   | convert_gray_to_false_color()   |                                 |
| pixUnpackBinary             | ✅   | unpack_binary()                 |                                 |
| pixConvert1To16             | ✅   | convert_1_to_16()               |                                 |
| pixConvert1To32             | ✅   | convert_1_to_32()               |                                 |
| pixConvert1To2Cmap          | ✅   | convert_1_to_2_cmap()           |                                 |
| pixConvert1To2              | ✅   | convert_1_to_2()                |                                 |
| pixConvert1To4Cmap          | ✅   | convert_1_to_4_cmap()           |                                 |
| pixConvert1To4              | ✅   | convert_1_to_4()                |                                 |
| pixConvert1To8Cmap          | ✅   | convert_1_to_8_cmap()           |                                 |
| pixConvert1To8              | ✅   | convert_1_to_8()                |                                 |
| pixConvert2To8              | ✅   | convert_2_to_8()                |                                 |
| pixConvert4To8              | ✅   | convert_4_to_8()                |                                 |
| pixConvert8To16             | ✅   | convert_8_to_16()               |                                 |
| pixConvertTo2               | ✅   | convert_to_2()                  |                                 |
| pixConvert8To2              | ✅   | convert_8_to_2()                |                                 |
| pixConvertTo4               | ✅   | convert_to_4()                  |                                 |
| pixConvert8To4              | ✅   | convert_8_to_4()                |                                 |
| pixConvertTo8               | ✅   | convert_to_8()                  |                                 |
| pixConvertTo16              | ✅   | convert_to_16()                 |                                 |
| pixConvertTo32              | ✅   | convert_to_32()                 |                                 |
| pixConvert8To32             | ✅   | convert_8_to_32()               |                                 |
| pixConvertTo8Or32           | ✅   | convert_to_8_or_32()            |                                 |
| pixConvert32To16            | ✅   | convert_32_to_16()              |                                 |
| pixConvert32To8             | ✅   | convert_32_to_8()               |                                 |
| pixRemoveAlpha              | ✅   | remove_alpha()                  |                                 |
| pixAddAlphaTo1bpp           | ✅   | add_alpha_to_1bpp()             |                                 |
| pixConvertLossless          | ✅   | convert_lossless()              |                                 |
| pixConvertForPSWrap         | ✅   | convert_for_ps_wrap()           |                                 |

### pixarith.c (ピクセル算術演算)

#### arith.rs

| C関数                  | 状態 | Rust対応                | 備考      |
| ---------------------- | ---- | ----------------------- | --------- |
| pixAddGray             | ✅   | arith_add()             |           |
| pixSubtractGray        | ✅   | arith_subtract()        |           |
| pixMultConstantGray    | ✅   | multiply_constant()     |           |
| pixAddConstantGray     | ✅   | add_constant()          |           |
| pixMultConstAccumulate | ✅   | mult_const_accumulate() | 32bpp専用 |
| pixAbsDifference       | ✅   | abs_difference()        |           |
| pixMinOrMax            | ✅   | min_or_max()            |           |

その他のpixarith.c関数も実装済み。

### rop.c, roplow.c (ラスターオペレーション)

#### その他

| C関数                | 状態 | Rust対応            | 備考 |
| -------------------- | ---- | ------------------- | ---- |
| pixRasterop          | ✅   | rop.rsに実装        |      |
| pixRasteropIP        | ✅   | rasterop_ip         |      |
| pixRasteropFullImage | ✅   | rasterop_full_image |      |

#### rop.rs

| C関数          | 状態 | Rust対応       | 備考 |
| -------------- | ---- | -------------- | ---- |
| pixRasteropVip | ✅   | rasterop_vip() |      |
| pixRasteropHip | ✅   | rasterop_hip() |      |
| pixTranslate   | ✅   | translate()    |      |

roplow.c (低レベルラスターOP) 全関数 🚫 不要 (高レベルrop.rs APIでカバー済み)

### compare.c (画像比較)

#### その他

| C関数                | 状態 | Rust対応                      | 備考 |
| -------------------- | ---- | ----------------------------- | ---- |
| pixEqual             | ✅   | compare.rsに実装              |      |
| pixCorrelationBinary | ✅   | compare::correlation_binary() |      |
| pixCompareBinary     | ✅   | compare::compare_binary()     |      |
| pixCompareTiled      | ✅   | compare_tiled                 |      |
| pixGetPerceptualDiff | ✅   | get_perceptual_diff           |      |

#### compare.rs

| C関数                     | 状態 | Rust対応                   | 備考 |
| ------------------------- | ---- | -------------------------- | ---- |
| pixEqualWithAlpha         | ✅   | equals_with_alpha()        |      |
| pixEqualWithCmap          | ✅   | equals_with_cmap()         |      |
| pixDisplayDiff            | ✅   | display_diff()             |      |
| pixDisplayDiffBinary      | ✅   | display_diff_binary()      |      |
| pixCompareGrayOrRGB       | ✅   | compare_gray_or_rgb()      |      |
| pixCompareGray            | ✅   | compare_gray()             |      |
| pixCompareRGB             | ✅   | compare_rgb()              |      |
| pixCompareRankDifference  | ✅   | compare_rank_difference()  |      |
| pixTestForSimilarity      | ✅   | test_for_similarity()      |      |
| pixGetDifferenceStats     | ✅   | get_difference_stats()     |      |
| pixGetDifferenceHistogram | ✅   | get_difference_histogram() |      |
| pixGetPSNR                | ✅   | get_psnr()                 |      |

その他の比較関数も実装済み。

### blend.c (ブレンド・合成)

#### その他

| C関数                     | 状態 | Rust対応                      | 備考 |
| ------------------------- | ---- | ----------------------------- | ---- |
| pixBlend                  | ✅   | blend.rsに実装                |      |
| pixBlendMask              | ✅   | blend::blend_mask()           |      |
| pixBlendGray              | ✅   | blend::blend_gray()           |      |
| pixBlendColor             | ✅   | blend::blend_color()          |      |
| pixBlendWithGrayMask      | ✅   | blend::blend_with_gray_mask() |      |
| pixBlendBackgroundToColor | ✅   | blend_background_to_color     |      |
| pixSetAlphaOverWhite      | ✅   | set_alpha_over_white          |      |

#### blend.rs

| C関数                  | 状態 | Rust対応                 | 備考 |
| ---------------------- | ---- | ------------------------ | ---- |
| pixBlendGrayInverse    | ✅   | blend_gray_inverse()     |      |
| pixBlendColorByChannel | ✅   | blend_color_by_channel() |      |
| pixBlendGrayAdapt      | ✅   | blend_gray_adapt()       |      |
| pixFadeWithGray        | ✅   | fade_with_gray()         |      |
| pixBlendHardLight      | ✅   | blend_hard_light()       |      |
| pixBlendCmap           | ✅   | blend_cmap()             |      |
| pixMultiplyByColor     | ✅   | multiply_by_color()      |      |
| pixAlphaBlendUniform   | ✅   | alpha_blend_uniform()    |      |
| pixAddAlphaToBlend     | ✅   | add_alpha_to_blend()     |      |
| pixLinearEdgeFade      | ✅   | linear_edge_fade()       |      |

### graphics.c (描画・レンダリング)

#### graphics.rs

| C関数                    | 状態 | Rust対応                     | 備考 |
| ------------------------ | ---- | ---------------------------- | ---- |
| generatePtaLine          | ✅   | generate_line_pta()          |      |
| generatePtaWideLine      | ✅   | generate_wide_line_pta()     |      |
| generatePtaBox           | ✅   | generate_box_pta()           |      |
| generatePtaBoxa          | ✅   | generate_boxa_pta()          |      |
| generatePtaHashBox       | ✅   | generate_hash_box_pta()      |      |
| generatePtaHashBoxa      | ✅   | generate_hash_boxa_pta()     |      |
| generatePtaaBoxa         | ✅   | generate_ptaa_boxa()         |      |
| generatePtaaHashBoxa     | ✅   | generate_ptaa_hash_boxa()    |      |
| generatePtaPolyline      | ✅   | generate_polyline_pta()      |      |
| generatePtaGrid          | ✅   | generate_grid_pta()          |      |
| convertPtaLineTo4cc      | ✅   | convert_line_to_4cc()        |      |
| generatePtaFilledCircle  | ✅   | generate_filled_circle_pta() |      |
| generatePtaFilledSquare  | ✅   | generate_filled_square_pta() |      |
| pixRenderPlotFromNuma    | ✅   | render_plot_from_numa()      |      |
| pixRenderPlotFromNumaGen | ✅   | render_plot_from_numa_gen()  |      |
| pixRenderPtaArb          | ✅   | render_pta_color()           |      |
| pixRenderPtaBlend        | ✅   | render_pta_blend()           |      |
| pixRenderLineArb         | ✅   | render_line_color()          |      |
| pixRenderLineBlend       | ✅   | render_line_blend()          |      |
| pixRenderBoxArb          | ✅   | render_box_color()           |      |
| pixRenderBoxBlend        | ✅   | render_box_blend()           |      |
| pixRenderBoxa            | ✅   | render_boxa()                |      |
| pixRenderBoxaArb         | ✅   | render_boxa_color()          |      |
| pixRenderBoxaBlend       | ✅   | render_boxa_blend()          |      |
| pixRenderHashBox         | ✅   | render_hash_box()            |      |
| pixRenderHashBoxArb      | ✅   | render_hash_box_color()      |      |
| pixRenderHashBoxBlend    | ✅   | render_hash_box_blend()      |      |
| pixRenderHashMaskArb     | ✅   | render_hash_mask_color()     |      |
| pixRenderHashBoxa        | ✅   | render_hash_boxa()           |      |
| pixRenderHashBoxaArb     | ✅   | render_hash_boxa_color()     |      |
| pixRenderHashBoxaBlend   | ✅   | render_hash_boxa_blend()     |      |
| pixRenderPolyline        | ✅   | render_polyline()            |      |
| pixRenderPolylineArb     | ✅   | render_polyline_color()      |      |
| pixRenderPolylineBlend   | ✅   | render_polyline_blend()      |      |
| pixRenderGridArb         | ✅   | render_grid_color()          |      |
| pixRenderRandomCmapPtaa  | ✅   | render_random_cmap_ptaa()    |      |
| pixRenderPolygon         | ✅   | render_polygon()             |      |
| pixFillPolygon           | ✅   | fill_polygon()               |      |
| pixRenderContours        | ✅   | render_contours()            |      |

#### その他

| C関数                  | 状態 | Rust対応                   | 備考           |
| ---------------------- | ---- | -------------------------- | -------------- |
| pixRenderPta           | ✅   | graphics.rsに部分実装      |                |
| pixRenderLine          | ✅   | graphics::render_line()    |                |
| pixRenderBox           | ✅   | graphics::render_box()     |                |
| fpixAutoRenderContours | ✅   | FPix::auto_render_contours | FPix関連は後続 |
| fpixRenderContours     | ✅   | FPix::render_contours      | FPix関連は後続 |
| pixGeneratePtaBoundary | ✅   | generate_pta_boundary      | 後続Phase      |

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

comparison テーブル上で未実装（❌）として残る項目はない。
boxfunc2.c / boxfunc5.c の該当関数群は実装済み。
