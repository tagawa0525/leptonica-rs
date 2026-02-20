# leptonica-core: C版 vs Rust版 関数レベル比較

調査日: 2026-02-21（Phase 13-17.3 実装を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 495 |
| 🔄 異なる | 24 |
| ❌ 未実装 | 363 |
| 合計 | 882 |

**カバレッジ**: 58.8% (519/882 関数が何らかの形で実装済み)

注: 合計845→882はptafunc/pixafuncのサマリー行を個別関数に展開したため。

## 注記

- ✅ 同等: Rust版で同じアルゴリズム/機能を持つ関数が存在
- 🔄 異なる: Rust版で異なるAPI/アプローチで実装
- ❌ 未実装: Rust版に対応する関数が存在しない

Rust版は**Pix/PixMut二層モデル**を採用しているため、C版の一部の関数は異なるAPIで提供される。
例: `pixCopy()` → `Pix::deep_clone()`, `pixClone()` → `Pix::clone()`

## 詳細

### pix1.c (基本的なPix操作)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixCreate | ✅ | Pix::new() | |
| pixCreateNoInit | ❌ | - | 初期化なし版は未実装 |
| pixCreateTemplate | ❌ | - | |
| pixCreateTemplateNoInit | ❌ | - | |
| pixCreateWithCmap | ❌ | - | |
| pixCreateHeader | ❌ | - | ヘッダのみ作成は未実装 |
| pixClone | 🔄 | Pix::clone() | Arc参照カウントで自動実装 |
| pixDestroy | 🔄 | drop() | Rustのデストラクタで自動 |
| pixCopy | 🔄 | Pix::deep_clone() | deep_cloneが完全コピー |
| pixResizeImageData | ❌ | - | |
| pixCopyColormap | ❌ | - | |
| pixTransferAllData | ❌ | - | |
| pixSwapAndDestroy | ❌ | - | |
| pixGetWidth | ✅ | Pix::width() | |
| pixSetWidth | ❌ | - | 不変なため設定不可 |
| pixGetHeight | ✅ | Pix::height() | |
| pixSetHeight | ❌ | - | 不変なため設定不可 |
| pixGetDepth | ✅ | Pix::depth() | |
| pixSetDepth | ❌ | - | 不変なため設定不可 |
| pixGetDimensions | ✅ | width()/height()/depth() | 個別メソッドで取得 |
| pixSetDimensions | ❌ | - | |
| pixCopyDimensions | ❌ | - | |
| pixGetSpp | ✅ | Pix::spp() | |
| pixSetSpp | 🔄 | PixMut::set_spp() | PixMutで可変 |
| pixCopySpp | ❌ | - | |
| pixGetWpl | ✅ | Pix::wpl() | |
| pixSetWpl | ❌ | - | 自動計算のため設定不可 |
| pixGetXRes | ✅ | Pix::xres() | |
| pixSetXRes | 🔄 | PixMut::set_xres() | |
| pixGetYRes | ✅ | Pix::yres() | |
| pixSetYRes | 🔄 | PixMut::set_yres() | |
| pixGetResolution | ✅ | xres()/yres() | |
| pixSetResolution | 🔄 | PixMut::set_resolution() | |
| pixCopyResolution | ❌ | - | |
| pixScaleResolution | ❌ | - | |
| pixGetInputFormat | ✅ | Pix::informat() | |
| pixSetInputFormat | 🔄 | PixMut::set_informat() | |
| pixCopyInputFormat | ❌ | - | |
| pixSetSpecial | 🔄 | PixMut::set_special() | |
| pixGetText | ✅ | Pix::text() | |
| pixSetText | 🔄 | PixMut::set_text() | |
| pixAddText | ❌ | - | |
| pixCopyText | ❌ | - | |
| pixGetTextCompNew | ❌ | - | |
| pixSetTextCompNew | ❌ | - | |
| pixGetColormap | ✅ | Pix::colormap() | |
| pixSetColormap | 🔄 | PixMut::set_colormap() | |
| pixDestroyColormap | ❌ | - | set_colormap(None)で実現可 |
| pixGetData | ✅ | Pix::data() | |
| pixFreeAndSetData | ❌ | - | |
| pixSetData | ❌ | - | |
| pixFreeData | ❌ | - | |
| pixExtractData | ❌ | - | |
| pixGetLinePtrs | ❌ | - | |
| pixSizesEqual | ❌ | - | |
| pixMaxAspectRatio | ❌ | - | |
| pixPrintStreamInfo | ❌ | - | Debug traitで部分的に対応 |

### pix2.c (ピクセルアクセス・設定)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGetPixel | ✅ | Pix::get_pixel() | |
| pixSetPixel | ✅ | PixMut::set_pixel() | |
| pixGetRGBPixel | ✅ | rgb.rs get_rgb_pixel() | |
| pixSetRGBPixel | ✅ | rgb.rs set_rgb_pixel() | |
| pixSetCmapPixel | ❌ | - | |
| pixGetRandomPixel | ❌ | - | |
| pixClearPixel | ❌ | - | set_pixel(x, y, 0)で可 |
| pixFlipPixel | ❌ | - | |
| pixGetBlackOrWhiteVal | ❌ | - | |
| pixClearAll | 🔄 | PixMut::clear() | |
| pixSetAll | 🔄 | PixMut::set_all() | |
| pixSetAllGray | ❌ | - | |
| pixSetAllArbitrary | ❌ | - | |
| pixSetBlackOrWhite | ❌ | - | |
| pixSetComponentArbitrary | ❌ | - | |
| pixClearInRect | ❌ | - | |
| pixSetInRect | ❌ | - | |
| pixSetInRectArbitrary | ❌ | - | |
| pixBlendInRect | ❌ | - | |
| pixSetPadBits | ❌ | - | |
| pixSetPadBitsBand | ❌ | - | |
| pixSetOrClearBorder | ❌ | - | |
| pixSetBorderVal | ✅ | border.rs set_border_val() | |
| pixSetBorderRingVal | ❌ | - | |
| pixSetMirroredBorder | ❌ | - | |
| pixCopyBorder | ❌ | - | |
| pixAddBorder | ✅ | border.rs add_border() | |
| pixAddBlackOrWhiteBorder | ✅ | border.rs add_black_or_white_border() | |
| pixAddBorderGeneral | ✅ | border.rs add_border_general() | |
| pixAddMultipleBlackWhiteBorders | ❌ | - | |
| pixRemoveBorder | ✅ | border.rs remove_border() | |
| pixRemoveBorderGeneral | ✅ | border.rs remove_border_general() | |
| pixRemoveBorderToSize | ❌ | - | |
| pixAddMirroredBorder | ✅ | border.rs add_mirrored_border() | |
| pixAddRepeatedBorder | ✅ | border.rs add_repeated_border() | |
| pixAddMixedBorder | ❌ | - | |
| pixAddContinuedBorder | ❌ | - | |
| pixShiftAndTransferAlpha | ❌ | - | |
| pixDisplayLayersRGBA | ❌ | - | |
| pixCreateRGBImage | ✅ | rgb.rs create_rgb_image() | |
| pixGetRGBComponent | ✅ | rgb.rs get_rgb_component() | |
| pixSetRGBComponent | ✅ | rgb.rs set_rgb_component() | |
| pixGetRGBComponentCmap | ❌ | - | |
| pixCopyRGBComponent | ❌ | - | |
| composeRGBPixel | ❌ | - | |
| composeRGBAPixel | ❌ | - | |
| extractRGBValues | ❌ | - | |
| extractRGBAValues | ❌ | - | |
| extractMinMaxComponent | ❌ | - | |
| pixGetRGBLine | ❌ | - | |
| pixEndianByteSwapNew | ❌ | - | |
| pixEndianByteSwap | ❌ | - | |
| pixEndianTwoByteSwap | ❌ | - | |
| pixGetRasterData | ❌ | - | |
| pixInferResolution | ❌ | - | |
| pixAlphaIsOpaque | ❌ | - | |

### pix3.c (マスク・ブール演算)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixSetMasked | ✅ | mask.rs set_masked() | |
| pixSetMaskedGeneral | ✅ | mask.rs set_masked_general() | |
| pixCombineMasked | ✅ | mask.rs combine_masked() | |
| pixCombineMaskedGeneral | ✅ | mask.rs combine_masked_general() | |
| pixPaintThroughMask | ✅ | mask.rs paint_through_mask() | |
| pixCopyWithBoxa | ✅ | mask.rs copy_with_boxa() | |
| pixPaintSelfThroughMask | ❌ | - | 後続Phase |
| pixMakeMaskFromVal | ✅ | mask.rs make_mask_from_val() | |
| pixMakeMaskFromLUT | ✅ | mask.rs make_mask_from_lut() | |
| pixMakeArbMaskFromRGB | ✅ | mask.rs make_arb_mask_from_rgb() | |
| pixSetUnderTransparency | ✅ | mask.rs set_under_transparency() | |
| pixMakeAlphaFromMask | ❌ | - | |
| pixGetColorNearMaskBoundary | ❌ | - | |
| pixDisplaySelectedPixels | ❌ | - | |
| pixInvert | ✅ | ops.rsに実装 | |
| pixOr | ✅ | ops.rsに実装 | |
| pixAnd | ✅ | ops.rsに実装 | |
| pixXor | ✅ | ops.rsに実装 | |
| pixSubtract | ✅ | ops.rsに実装 | |
| pixZero | ✅ | statistics.rs is_zero() | |
| pixForegroundFraction | ✅ | statistics.rs foreground_fraction() | |
| pixaCountPixels | ✅ | pixa count_pixels() | |
| pixCountPixels | ✅ | statistics.rs count_pixels() | |
| pixCountPixelsInRect | ✅ | statistics.rs count_pixels_in_rect() | |
| pixCountByRow | ✅ | statistics.rs count_by_row() | |
| pixCountByColumn | ✅ | statistics.rs count_by_column() | |
| pixCountPixelsByRow | ✅ | statistics.rs count_pixels_by_row() | Numa返却版 |
| pixCountPixelsByColumn | ✅ | statistics.rs count_pixels_by_column() | Numa返却版 |
| pixCountPixelsInRow | ✅ | statistics.rs count_pixels_in_row() | |
| pixGetMomentByColumn | ✅ | statistics.rs get_moment_by_column() | |
| pixThresholdPixelSum | ✅ | statistics.rs threshold_pixel_sum() | |
| pixAverageByRow | ✅ | statistics.rs average_by_row() | |
| pixAverageByColumn | ✅ | statistics.rs average_by_column() | |
| pixAverageInRect | ✅ | statistics.rs average_in_rect() | |
| pixAverageInRectRGB | ✅ | statistics.rs average_in_rect_rgb() | |
| pixVarianceByRow | ✅ | statistics.rs variance_by_row() | |
| pixVarianceByColumn | ✅ | statistics.rs variance_by_column() | |
| pixVarianceInRect | ✅ | statistics.rs variance_in_rect() | |
| pixAbsDiffByRow | ✅ | statistics.rs abs_diff_by_row() | |
| pixAbsDiffByColumn | ✅ | statistics.rs abs_diff_by_column() | |
| pixAbsDiffInRect | ✅ | statistics.rs abs_diff_in_rect() | |
| pixAbsDiffOnLine | ✅ | statistics.rs abs_diff_on_line() | |
| pixCountArbInRect | ✅ | statistics.rs count_arb_in_rect() | |
| pixMirroredTiling | ❌ | - | |
| pixFindRepCloseTile | ❌ | - | |

### pix4.c (ヒストグラム・統計)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGetGrayHistogram | ✅ | histogram.rsに実装 | |
| pixGetGrayHistogramMasked | ✅ | histogram.rs gray_histogram_masked() | |
| pixGetGrayHistogramInRect | ✅ | histogram.rs gray_histogram_in_rect() | |
| pixGetGrayHistogramTiled | ✅ | histogram.rs gray_histogram_tiled() | |
| pixGetColorHistogram | ✅ | histogram.rsに実装 | |
| pixGetColorHistogramMasked | ✅ | histogram.rs color_histogram_masked() | |
| pixGetCmapHistogram | ✅ | histogram.rs cmap_histogram() | |
| pixGetCmapHistogramMasked | ✅ | histogram.rs cmap_histogram_masked() | |
| pixGetCmapHistogramInRect | ✅ | histogram.rs cmap_histogram_in_rect() | |
| pixCountRGBColorsByHash | ❌ | - | |
| pixCountRGBColors | ✅ | histogram.rs count_rgb_colors() | |
| pixGetColorAmapHistogram | ❌ | - | |
| pixGetRankValue | ✅ | histogram.rs pixel_rank_value() | |
| pixGetRankValueMaskedRGB | ✅ | histogram.rs rank_value_masked_rgb() | |
| pixGetRankValueMasked | ✅ | histogram.rs rank_value_masked() | |
| pixGetPixelAverage | ✅ | statistics.rs get_pixel_average() | |
| pixGetPixelStats | ✅ | statistics.rs get_pixel_stats() | |
| pixGetAverageMaskedRGB | ✅ | histogram.rs average_masked_rgb() | |
| pixGetAverageMasked | ✅ | histogram.rs average_masked() | |
| pixGetAverageTiledRGB | ✅ | histogram.rs average_tiled_rgb() | |
| pixGetAverageTiled | ✅ | histogram.rs average_tiled() | |
| pixRowStats | ✅ | statistics.rs row_stats() | |
| pixColumnStats | ✅ | statistics.rs column_stats() | |
| pixGetRangeValues | ✅ | statistics.rs range_values() | |
| pixGetExtremeValue | ✅ | statistics.rs extreme_value() | |
| pixGetMaxValueInRect | ✅ | statistics.rs max_value_in_rect() | |
| pixGetMaxColorIndex | ✅ | histogram.rs max_color_index() | |
| pixGetBinnedComponentRange | ❌ | - | |
| pixGetRankColorArray | ❌ | - | |
| pixGetBinnedColor | ❌ | - | |
| pixDisplayColorArray | ❌ | - | |
| pixRankBinByStrip | ❌ | - | |
| pixaGetAlignedStats | ✅ | pixa aligned_stats() | |
| pixaExtractColumnFromEachPix | ✅ | pixa extract_column_from_each() | |
| pixGetRowStats | ✅ | statistics.rs get_row_stats() | |
| pixGetColumnStats | ✅ | statistics.rs get_column_stats() | |
| pixSetPixelColumn | ✅ | statistics.rs set_pixel_column() | |
| pixThresholdForFgBg | ✅ | clip.rs threshold_for_fg_bg() | |
| pixSplitDistributionFgBg | ❌ | - | |

### pix5.c (選択・測定)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaFindDimensions | ✅ | pixa find_dimensions() | |
| pixFindAreaPerimRatio | ❌ | - | |
| pixaFindPerimToAreaRatio | ❌ | - | |
| pixFindPerimToAreaRatio | ✅ | measurement.rs find_perim_to_area_ratio() | |
| pixaFindPerimSizeRatio | ❌ | - | |
| pixFindPerimSizeRatio | ❌ | - | |
| pixaFindAreaFraction | ❌ | - | |
| pixFindAreaFraction | ❌ | - | |
| pixaFindAreaFractionMasked | ❌ | - | |
| pixFindAreaFractionMasked | ❌ | - | |
| pixaFindWidthHeightRatio | ❌ | - | |
| pixaFindWidthHeightProduct | ❌ | - | |
| pixFindOverlapFraction | ✅ | measurement.rs find_overlap_fraction() | |
| pixFindRectangleComps | ❌ | - | |
| pixConformsToRectangle | ❌ | - | |
| pixExtractRectangularRegions | ❌ | - | |
| pixClipRectangles | ✅ | clip.rs clip_rectangles() | |
| pixClipRectangle | ✅ | clip.rs clip_rectangle() | |
| pixClipRectangleWithBorder | ✅ | clip.rs clip_rectangle_with_border() | |
| pixClipMasked | ✅ | clip.rs clip_masked() | |
| pixCropToMatch | ✅ | clip.rs crop_to_match() | |
| pixCropToSize | ✅ | clip.rs crop_to_size() | |
| pixResizeToMatch | ✅ | clip.rs resize_to_match() | |
| pixSelectComponentBySize | ❌ | - | |
| pixFilterComponentBySize | ❌ | - | |
| pixMakeSymmetricMask | ✅ | clip.rs make_symmetric_mask() | |
| pixMakeFrameMask | ✅ | clip.rs make_frame_mask() | |
| pixMakeCoveringOfRectangles | ❌ | - | |
| pixFractionFgInMask | ✅ | clip.rs fraction_fg_in_mask() | |
| pixClipToForeground | ✅ | clip.rs clip_to_foreground() | |
| pixTestClipToForeground | ✅ | clip.rs test_clip_to_foreground() | |
| pixClipBoxToForeground | ✅ | clip.rs clip_box_to_foreground() | |
| pixScanForForeground | ✅ | clip.rs scan_for_foreground() | |
| pixClipBoxToEdges | ✅ | clip.rs clip_box_to_edges() | |
| pixScanForEdge | ✅ | clip.rs scan_for_edge() | 8bpp適応版 |
| pixExtractOnLine | ✅ | extract.rs extract_on_line() | |
| pixAverageOnLine | ✅ | clip.rs average_on_line() | |
| pixAverageIntensityProfile | ✅ | extract.rs average_intensity_profile() | |
| pixReversalProfile | ❌ | - | |
| pixWindowedVarianceOnLine | ❌ | - | |
| pixMinMaxNearLine | ❌ | - | |
| pixRankRowTransform | ✅ | extract.rs rank_row_transform() | |
| pixRankColumnTransform | ✅ | extract.rs rank_column_transform() | |

### boxbasic.c (Box基本操作)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| boxCreate | ✅ | Box::new() | |
| boxCreateValid | ❌ | - | newでバリデーション実施 |
| boxCopy | 🔄 | Box自体がCopyトレイト | |
| boxClone | 🔄 | Box自体がCopyトレイト | |
| boxDestroy | 🔄 | drop() | 自動 |
| boxGetGeometry | ✅ | フィールドアクセス | |
| boxSetGeometry | ❌ | - | |
| boxGetSideLocations | ❌ | - | right()/bottom()で部分対応 |
| boxSetSideLocations | ❌ | - | |
| boxIsValid | ✅ | Box::is_valid() | |
| boxaCreate | ✅ | Boxa::new() | |
| boxaCopy | ✅ | Boxa::clone() | |
| boxaDestroy | 🔄 | drop() | 自動 |
| boxaAddBox | ✅ | Boxa::push() | |
| boxaExtendArray | ❌ | - | Vec自動拡張 |
| boxaExtendArrayToSize | ❌ | - | |
| boxaGetCount | ✅ | Boxa::len() | |
| boxaGetValidCount | ❌ | - | |
| boxaGetBox | ✅ | Boxa::get() | |
| boxaGetValidBox | ❌ | - | |
| boxaFindInvalidBoxes | ❌ | - | |
| boxaGetBoxGeometry | ❌ | - | |
| boxaIsFull | ❌ | - | |
| boxaReplaceBox | ✅ | Boxa::replace() | |
| boxaInsertBox | ✅ | Boxa::insert() | |
| boxaRemoveBox | ✅ | Boxa::remove() | |
| boxaRemoveBoxAndSave | ❌ | - | |
| boxaSaveValid | ❌ | - | |
| boxaInitFull | ❌ | - | |
| boxaClear | ✅ | Boxa::clear() | |
| boxaaCreate | ✅ | Boxaa::new() | |
| boxaaCopy | ❌ | - | |
| boxaaDestroy | 🔄 | drop() | 自動 |
| boxaaAddBoxa | ✅ | Boxaa::push() | |
| boxaaExtendArray | ❌ | - | |
| boxaaExtendArrayToSize | ❌ | - | |
| boxaaGetCount | ✅ | Boxaa::len() | |
| boxaaGetBoxCount | ✅ | Boxaa::total_boxes() | |
| boxaaGetBoxa | ✅ | Boxaa::get() | |
| boxaaGetBox | ❌ | - | |
| boxaaInitFull | ❌ | - | |
| boxaaExtendWithInit | ❌ | - | |
| boxaaReplaceBoxa | ❌ | - | |
| boxaaInsertBoxa | ❌ | - | |
| boxaaRemoveBoxa | ❌ | - | |
| boxaaAddBox | ❌ | - | |
| boxaaReadFromFiles | ❌ | - | I/O未実装 |
| boxaaRead | ❌ | - | |
| boxaaReadStream | ❌ | - | |
| boxaaReadMem | ❌ | - | |
| boxaaWrite | ❌ | - | |
| boxaaWriteStream | ❌ | - | |
| boxaaWriteMem | ❌ | - | |
| boxaRead | ❌ | - | |
| boxaReadStream | ❌ | - | |
| boxaReadMem | ❌ | - | |
| boxaWriteDebug | ❌ | - | |
| boxaWrite | ❌ | - | |
| boxaWriteStream | ❌ | - | |
| boxaWriteStderr | ❌ | - | |
| boxaWriteMem | ❌ | - | |
| boxPrintStreamInfo | ❌ | - | |

### boxfunc1.c (Box関係・幾何演算)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| boxContains | ✅ | Box::contains_box() | |
| boxIntersects | ✅ | Box::overlaps() | |
| boxaContainedInBox | ✅ | mod.rs contained_in_box() | |
| boxaContainedInBoxCount | ✅ | geometry.rs contained_in_box_count() | |
| boxaContainedInBoxa | ✅ | geometry.rs all_contained_in() | |
| boxaIntersectsBox | ✅ | mod.rs intersects_box() | |
| boxaIntersectsBoxCount | ✅ | geometry.rs intersects_box_count() | |
| boxaClipToBox | ✅ | mod.rs clip_to_box() | |
| boxaCombineOverlaps | ✅ | mod.rs combine_overlaps() | |
| boxaCombineOverlapsInPair | ✅ | geometry.rs combine_overlaps_in_pair() | |
| boxOverlapRegion | ✅ | Box::intersect() | |
| boxBoundingRegion | ✅ | Box::union() | |
| boxOverlapFraction | ✅ | mod.rs overlap_fraction() | |
| boxOverlapArea | ✅ | mod.rs overlap_area() | |
| boxaHandleOverlaps | ✅ | geometry.rs handle_overlaps() | |
| boxOverlapDistance | ✅ | geometry.rs overlap_distance() | |
| boxSeparationDistance | ✅ | geometry.rs separation_distance() | |
| boxCompareSize | ✅ | geometry.rs compare_size() | |
| boxContainsPt | ✅ | Box::contains_point() | |
| boxaGetNearestToPt | ✅ | geometry.rs nearest_to_point() | |
| boxaGetNearestToLine | ✅ | geometry.rs nearest_to_line() | |
| boxaFindNearestBoxes | ✅ | geometry.rs find_nearest_boxes() | |
| boxaGetNearestByDirection | ✅ | geometry.rs nearest_by_direction() | |
| boxGetCenter | ✅ | mod.rs center() | |
| boxIntersectByLine | ✅ | geometry.rs intersect_by_line() | |
| boxClipToRectangle | ✅ | mod.rs clip() | |
| boxClipToRectangleParams | ✅ | geometry.rs clip_to_rectangle_params() | |
| boxRelocateOneSide | ✅ | adjust.rs relocate_one_side() | |
| boxaAdjustSides | ✅ | adjust.rs adjust_all_sides() | |
| boxaAdjustBoxSides | ✅ | adjust.rs adjust_box_sides() | |
| boxAdjustSides | ✅ | adjust.rs adjust_sides() | |
| boxaSetSide | ✅ | adjust.rs set_all_sides() | |
| boxSetSide | ✅ | adjust.rs set_side() | |
| boxaAdjustWidthToTarget | ✅ | adjust.rs adjust_width_to_target() | |
| boxaAdjustHeightToTarget | ✅ | adjust.rs adjust_height_to_target() | |
| boxEqual | ✅ | PartialEq trait | |
| boxaEqual | ✅ | adjust.rs equal_ordered() | |
| boxSimilar | ✅ | adjust.rs similar_per_side() | |
| boxaSimilar | ✅ | mod.rs similar() | |
| boxaJoin | ✅ | mod.rs join() | |
| boxaaJoin | ✅ | adjust.rs join() (Boxaa) | |
| boxaSplitEvenOdd | ✅ | adjust.rs split_even_odd() | |
| boxaMergeEvenOdd | ✅ | adjust.rs merge_even_odd() | |

### boxfunc2.c (未実装)
全関数 ❌ 未実装

### boxfunc3.c (Box描画・マスク)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMaskConnComp | ❌ | - | conncomp依存 |
| pixMaskBoxa | ✅ | draw.rs mask_boxa() | |
| pixPaintBoxa | ✅ | draw.rs paint_boxa() | |
| pixSetBlackOrWhiteBoxa | ✅ | draw.rs set_bw_boxa() | |
| pixPaintBoxaRandom | ✅ | draw.rs paint_boxa_random() | |
| pixBlendBoxaRandom | ✅ | draw.rs blend_boxa_random() | |
| pixDrawBoxa | ✅ | draw.rs draw_boxa() | |
| pixDrawBoxaRandom | ✅ | draw.rs draw_boxa_random() | |
| boxaaDisplay | ❌ | - | |
| pixaDisplayBoxaa | ❌ | - | |
| pixSplitIntoBoxa | ❌ | - | |
| pixSplitComponentIntoBoxa | ❌ | - | |
| makeMosaicStrips | ❌ | - | |
| boxaCompareRegions | ✅ | draw.rs compare_regions() | |
| pixSelectLargeULComp | ❌ | - | conncomp依存 |
| boxaSelectLargeULBox | ✅ | draw.rs select_large_ul_box() | |

### boxfunc4.c (Box選択・変換)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| boxaSelectRange | ✅ | select.rs select_range() | |
| boxaaSelectRange | ✅ | select.rs select_range() (Boxaa) | |
| boxaSelectBySize | ✅ | mod.rs select_by_size() | |
| boxaMakeSizeIndicator | ✅ | select.rs make_size_indicator() | |
| boxaSelectByArea | ✅ | mod.rs select_by_area() | |
| boxaMakeAreaIndicator | ✅ | select.rs make_area_indicator() | |
| boxaSelectByWHRatio | ✅ | mod.rs select_by_wh_ratio() | |
| boxaMakeWHRatioIndicator | ✅ | select.rs make_wh_ratio_indicator() | |
| boxaSelectWithIndicator | ✅ | select.rs select_with_indicator() | |
| boxaPermutePseudorandom | ❌ | - | |
| boxaPermuteRandom | ❌ | - | |
| boxaSwapBoxes | ✅ | select.rs swap_boxes() | |
| boxaConvertToPta | ✅ | adjust.rs to_pta() (Boxa) | |
| ptaConvertToBoxa | ✅ | adjust.rs to_boxa() | |
| boxConvertToPta | ✅ | adjust.rs to_pta() (Box) | |
| ptaConvertToBox | ✅ | adjust.rs to_box() | |
| boxaGetExtent | ✅ | mod.rs get_extent() | |
| boxaGetCoverage | ✅ | mod.rs get_coverage() | |
| boxaaSizeRange | ✅ | select.rs size_range() (Boxaa) | |
| boxaSizeRange | ✅ | mod.rs size_range() | |
| boxaLocationRange | ✅ | select.rs location_range() | |
| boxaGetSizes | ✅ | select.rs get_sizes() | |
| boxaGetArea | ✅ | select.rs get_total_area() | |
| boxaDisplayTiled | ❌ | - | |

### boxfunc5.c (Boxスムージング・調整)
全関数 ❌ 未実装 (ボックス位置のメディアンスムージングなど)

### ptabasic.c (Pta基本操作)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| ptaCreate | ✅ | Pta::new() | |
| ptaCreateFromNuma | ❌ | - | |
| ptaDestroy | 🔄 | drop() | 自動 |
| ptaCopy | ✅ | Pta::clone() | |
| ptaCopyRange | ❌ | - | |
| ptaClone | ✅ | Pta::clone() | |
| ptaEmpty | ❌ | - | |
| ptaAddPt | ✅ | Pta::push() | |
| ptaInsertPt | ❌ | - | |
| ptaRemovePt | ❌ | - | |
| ptaGetCount | ✅ | Pta::len() | |
| ptaGetPt | ✅ | Pta::get() | |
| ptaGetIPt | ❌ | - | |
| ptaSetPt | ✅ | Pta::set() | |
| ptaGetArrays | ❌ | - | |
| ptaRead | ❌ | - | I/O未実装 |
| ptaReadStream | ❌ | - | |
| ptaReadMem | ❌ | - | |
| ptaWriteDebug | ❌ | - | |
| ptaWrite | ❌ | - | |
| ptaWriteStream | ❌ | - | |
| ptaWriteMem | ❌ | - | |
| ptaaCreate | ❌ | - | Ptaa未実装 |
| ptaaDestroy | ❌ | - | |
| ptaaAddPta | ❌ | - | |
| ptaaGetCount | ❌ | - | |
| ptaaGetPta | ❌ | - | |
| ptaaGetPt | ❌ | - | |
| ptaaInitFull | ❌ | - | |
| ptaaReplacePta | ❌ | - | |
| ptaaAddPt | ❌ | - | |
| ptaaTruncate | ❌ | - | |
| ptaaRead | ❌ | - | |
| ptaaReadStream | ❌ | - | |
| ptaaReadMem | ❌ | - | |
| ptaaWriteDebug | ❌ | - | |
| ptaaWrite | ❌ | - | |
| ptaaWriteStream | ❌ | - | |
| ptaaWriteMem | ❌ | - | |

### ptafunc1.c, ptafunc2.c (Pta変換・演算)

Phase 16で大部分を実装済み。

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| ptaSubsample | ✅ | transform.rs subsample() | |
| ptaJoin | ✅ | transform.rs join() | |
| ptaaJoin | ❌ | - | |
| ptaReverse | ✅ | transform.rs reverse() | |
| ptaTranspose | ✅ | transform.rs transpose() | |
| ptaCyclicPerm | ✅ | transform.rs cyclic_perm() | |
| ptaSelectRange | ✅ | transform.rs select_range() | |
| ptaGetRange | ✅ | transform.rs get_range() | |
| ptaGetInsideBox | ✅ | transform.rs get_inside_box() | |
| ptaContainsPt | ✅ | transform.rs contains_pt() | |
| ptaTestIntersection | ✅ | transform.rs test_intersection() | |
| ptaTransform | ✅ | transform.rs transform_pts() | |
| ptaPtInsidePolygon | ✅ | transform.rs pt_inside_polygon() | |
| ptaPolygonIsConvex | ✅ | transform.rs polygon_is_convex() | |
| ptaGetMinMax | ✅ | transform.rs get_min_max() | |
| ptaSelectByValue | ✅ | transform.rs select_by_value() | |
| ptaCropToMask | ❌ | - | |
| ptaGetLinearLSF | ✅ | lsf.rs get_linear_lsf() | |
| ptaGetQuadraticLSF | ✅ | lsf.rs get_quadratic_lsf() | |
| ptaGetCubicLSF | ✅ | lsf.rs get_cubic_lsf() | |
| ptaGetQuarticLSF | ✅ | lsf.rs get_quartic_lsf() | |
| ptaSortByIndex | ✅ | sort.rs sort_by_index() | |
| ptaGetSortIndex | ✅ | sort.rs get_sort_index() | |
| ptaSort | ✅ | sort.rs sort_pta() | |
| ptaGetRankValue | ✅ | sort.rs get_rank_value() | |
| ptaSort2d | ✅ | sort.rs sort_2d() | |
| ptaEqual | ✅ | sort.rs equal() | |

### pixabasic.c (Pixa基本操作)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaCreate | ✅ | Pixa::new() | |
| pixaCreateFromPix | ❌ | - | |
| pixaCreateFromBoxa | ❌ | - | |
| pixaSplitPix | ❌ | - | |
| pixaDestroy | 🔄 | drop() | 自動 |
| pixaCopy | ✅ | Pixa::clone() | |
| pixaAddPix | ✅ | Pixa::push() | |
| pixaAddBox | ✅ | Pixa::push_with_box() | |
| pixaExtendArray | ❌ | - | Vec自動拡張 |
| pixaExtendArrayToSize | ❌ | - | |
| pixaGetCount | ✅ | Pixa::len() | |
| pixaGetPix | ✅ | Pixa::get_cloned() | |
| pixaGetPixDimensions | ✅ | Pixa::get_dimensions() | |
| pixaGetBoxa | ❌ | - | |
| pixaGetBoxaCount | ❌ | - | |
| pixaGetBox | ❌ | - | |
| pixaGetBoxGeometry | ❌ | - | |
| pixaSetBoxa | ❌ | - | |
| pixaGetPixArray | ❌ | - | |
| pixaVerifyDepth | ❌ | - | |
| pixaVerifyDimensions | ❌ | - | |
| pixaIsFull | ❌ | - | |
| pixaCountText | ❌ | - | |
| pixaSetText | ❌ | - | |
| pixaGetLinePtrs | ❌ | - | |
| pixaWriteStreamInfo | ❌ | - | |
| pixaReplacePix | ❌ | - | |
| pixaInsertPix | ❌ | - | |
| pixaRemovePix | ❌ | - | |
| pixaRemovePixAndSave | ❌ | - | |
| pixaRemoveSelected | ❌ | - | |
| pixaInitFull | ❌ | - | |
| pixaClear | ✅ | Pixa::clear() | |
| pixaJoin | ❌ | - | |
| pixaInterleave | ❌ | - | |
| pixaaJoin | ❌ | - | |
| pixaaCreate | ❌ | - | Pixaa未実装 |
| pixaaCreateFromPixa | ❌ | - | |
| pixaaDestroy | ❌ | - | |
| pixaaAddPixa | ❌ | - | |
| pixaaExtendArray | ❌ | - | |
| pixaaAddPix | ❌ | - | |
| pixaaAddBox | ❌ | - | |
| pixaaGetCount | ❌ | - | |
| pixaaGetPixa | ❌ | - | |
| pixaaGetBoxa | ❌ | - | |
| pixaaGetPix | ❌ | - | |
| pixaaVerifyDepth | ❌ | - | |
| pixaaVerifyDimensions | ❌ | - | |
| pixaaIsFull | ❌ | - | |
| pixaaInitFull | ❌ | - | |
| pixaaReplacePixa | ❌ | - | |
| pixaaClear | ❌ | - | |
| pixaaTruncate | ❌ | - | |
| pixaRead | ❌ | - | I/O未実装 |
| pixaReadStream | ❌ | - | |
| pixaReadMem | ❌ | - | |
| pixaWriteDebug | ❌ | - | |
| pixaWrite | ❌ | - | |
| pixaWriteStream | ❌ | - | |
| pixaWriteMem | ❌ | - | |
| pixaReadBoth | ❌ | - | |
| pixaaReadFromFiles | ❌ | - | |
| pixaaRead | ❌ | - | |
| pixaaReadStream | ❌ | - | |
| pixaaReadMem | ❌ | - | |
| pixaaWrite | ❌ | - | |
| pixaaWriteStream | ❌ | - | |
| pixaaWriteMem | ❌ | - | |

### pixafunc1.c, pixafunc2.c (Pixa選択・変換・表示)

Phase 16で主要機能を実装済み。

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaSelectBySize | ✅ | pixa select_by_size() | |
| pixaSelectByArea | ✅ | pixa select_by_area() | |
| pixaSort | ✅ | pixa sort() | |
| pixaSortByIndex | ✅ | pixa sort_by_index() | |
| pixaScaleToSize | ✅ | pixa scale_to_size() | |
| pixaScaleToSizeRel | ✅ | pixa scale_to_size_rel() | |
| pixaDisplay | ✅ | pixa display() | |
| pixaDisplayTiled | ✅ | pixa display_tiled() | |
| pixaDisplayTiledAndScaled | ✅ | pixa display_tiled_and_scaled() | |
| pixaGetAlignedStats | ✅ | pixa aligned_stats() | |
| pixaExtractColumnFromEachPix | ✅ | pixa extract_column_from_each() | |
| pixaFindDimensions | ✅ | pixa find_dimensions() | |
| pixaCountPixels | ✅ | pixa count_pixels() | |

### numabasic.c (Numa基本操作)

実装済み関数が存在するが、C版のnumabasic.cはI/O関連なので未実装。
numa/mod.rs, numa/operations.rs に基本統計関数は実装済み。

### numafunc1.c, numafunc2.c (Numa演算・統計)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| numaArithOp | ✅ | operations.rs arith_op() | |
| numaLogicalOp | ✅ | operations.rs logical_op() | |
| numaInvert | ✅ | operations.rs invert() | |
| numaSimilar | ✅ | operations.rs similar() | |
| numaAddToNumber | ✅ | operations.rs add_to_element() | |
| numaGetMin | ✅ | Numa::min() | |
| numaGetMax | ✅ | Numa::max() | |
| numaGetSum | ✅ | Numa::sum() | |
| numaGetPartialSums | ✅ | operations.rs partial_sums() | |
| numaGetSumOnInterval | ✅ | Numa::sum_on_interval() | |
| numaHasOnlyIntegers | ✅ | Numa::has_only_integers() | |
| numaGetMean | ✅ | Numa::mean() | |
| numaGetMeanAbsval | ✅ | Numa::mean_absval() | |
| numaSubsample | ✅ | operations.rs subsample() | |
| numaMakeDelta | ✅ | operations.rs make_delta() | |
| numaMakeSequence | ✅ | operations.rs make_sequence() | |
| numaMakeConstant | ✅ | Numa::make_constant() | |
| numaMakeAbsval | ✅ | operations.rs abs_val() | |
| numaAddBorder | ✅ | operations.rs add_border() | |
| numaAddSpecifiedBorder | ✅ | operations.rs add_specified_border() | |
| numaRemoveBorder | ✅ | operations.rs remove_border() | |
| numaCountNonzeroRuns | ✅ | operations.rs count_nonzero_runs() | |
| numaGetNonzeroRange | ✅ | operations.rs get_nonzero_range() | |
| numaGetCountRelativeToZero | ✅ | operations.rs get_count_relative_to_zero() | |
| numaClipToInterval | ✅ | operations.rs clip_to_interval() | |
| numaMakeThresholdIndicator | ✅ | operations.rs make_threshold_indicator() | |
| numaUniformSampling | ✅ | interpolation.rs uniform_sampling() | |
| numaReverse | ✅ | Numa::reversed() / Numa::reverse() | |
| numaLowPassIntervals | ✅ | interpolation.rs low_pass_intervals() | |
| numaThresholdEdges | ✅ | interpolation.rs threshold_edges() | |
| numaGetSpanValues | ✅ | interpolation.rs get_span_values() | |
| numaGetEdgeValues | ✅ | interpolation.rs get_edge_values() | |
| numaInterpolateEqxVal | ✅ | operations.rs interpolate_eqx_val() | |
| numaInterpolateArbxVal | ✅ | operations.rs interpolate_arbx_val() | |
| numaInterpolateEqxInterval | ✅ | interpolation.rs interpolate_eqx_interval() | |
| numaInterpolateArbxInterval | ✅ | interpolation.rs interpolate_arbx_interval() | |
| numaFitMax | ✅ | interpolation.rs fit_max() | |
| numaDifferentiateInterval | ✅ | interpolation.rs differentiate_interval() | |
| numaIntegrateInterval | ✅ | interpolation.rs integrate_interval() | |
| numaSortGeneral | ❌ | - | sort_auto_selectで統合 |
| numaSortAutoSelect | ✅ | operations.rs sort_auto_select() | |
| numaSortIndexAutoSelect | ✅ | operations.rs sort_index_auto_select() | |
| numaChooseSortType | ❌ | - | 内部関数 |
| numaSort | ✅ | Numa::sorted() / Numa::sort() | |
| numaBinSort | ✅ | sort.rs bin_sort() | |
| numaGetSortIndex | ✅ | operations.rs sort_index() | |
| numaGetBinSortIndex | ✅ | sort.rs bin_sort_index() | |
| numaSortByIndex | ✅ | operations.rs sort_by_index() | |
| numaIsSorted | ✅ | operations.rs is_sorted() | |
| numaSortPair | ✅ | sort.rs sort_pair() | |
| numaInvertMap | ✅ | sort.rs invert_map() | |
| numaAddSorted | ✅ | sort.rs add_sorted() | |
| numaFindSortedLoc | ✅ | sort.rs find_sorted_loc() | |
| numaPseudorandomSequence | ✅ | sort.rs pseudorandom_sequence() | |
| numaRandomPermutation | ✅ | sort.rs random_permutation() | |
| numaGetRankValue | ✅ | Numa::rank_value() | |
| numaGetMedian | ✅ | Numa::median() | |
| numaGetBinnedMedian | ✅ | sort.rs binned_median() | |
| numaGetMeanDevFromMedian | ✅ | sort.rs mean_dev_from_median() | |
| numaGetMedianDevFromMedian | ✅ | sort.rs median_dev_from_median() | |
| numaGetMode | ✅ | Numa::mode() | |
| numaJoin | ✅ | operations.rs join() | |
| numaaJoin | ❌ | - | |
| numaaFlattenToNuma | ✅ | Numaa::flatten() | |

numafunc2.c (ヒストグラム・統計)の多くの関数も未実装。
一部ヒストグラム関数はnuma/histogram.rsに実装あり。

### sarray1.c, sarray2.c (Sarray文字列配列)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| sarrayCreate | ✅ | Sarray::new() | |
| sarrayCreateInitialized | ✅ | Sarray::initialized() | |
| sarrayCreateWordsFromString | ✅ | Sarray::from_words() | |
| sarrayCreateLinesFromString | ✅ | Sarray::from_lines() | |
| sarrayDestroy | 🔄 | drop() | 自動 |
| sarrayCopy | ✅ | Sarray::clone() | |
| sarrayClone | ✅ | Sarray::clone() | |
| sarrayAddString | ✅ | Sarray::push() | |
| sarrayRemoveString | ❌ | - | |
| sarrayReplaceString | ❌ | - | |
| sarrayClear | ✅ | Sarray::clear() | |
| sarrayGetCount | ✅ | Sarray::len() | |
| sarrayGetArray | ❌ | - | |
| sarrayGetString | ✅ | Sarray::get() | |
| sarrayToString | ✅ | Sarray::join() | |
| sarrayToStringRange | ❌ | - | |
| sarrayConcatUniformly | ❌ | - | |
| sarrayJoin | ❌ | - | |
| sarrayAppendRange | ❌ | - | |
| sarrayPadToSameSize | ❌ | - | |
| sarrayConvertWordsToLines | ❌ | - | |
| sarraySplitString | ❌ | - | |
| sarraySelectBySubstring | ✅ | Sarray::filter_by_substring() | |
| sarraySelectRange | ❌ | - | |
| sarrayParseRange | ❌ | - | |
| sarrayRead | ❌ | - | I/O未実装 |
| sarrayReadStream | ❌ | - | |
| sarrayReadMem | ❌ | - | |
| sarrayWrite | ❌ | - | |
| sarrayWriteStream | ❌ | - | |
| sarrayWriteStderr | ❌ | - | |
| sarrayWriteMem | ❌ | - | |
| sarrayAppend | ❌ | - | |
| sarraySort | ✅ | Sarray::sort() | |
| sarraySortByIndex | ❌ | - | |

その他のsarray2.c関数（セット演算、整数生成など）も一部未実装。

### fpix1.c, fpix2.c (FPix浮動小数点画像)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fpixCreate | ✅ | FPix::new() | |
| fpixCreateTemplate | ✅ | FPix::create_template() | |
| fpixClone | ✅ | FPix::clone() | |
| fpixCopy | ✅ | FPix::clone() | |
| fpixDestroy | 🔄 | drop() | 自動 |
| fpixGetDimensions | ✅ | width()/height() | |
| fpixSetDimensions | ❌ | - | |
| fpixGetWpl | ❌ | - | FPixは1要素1f32でwpl概念なし |
| fpixSetWpl | ❌ | - | |
| fpixGetResolution | ✅ | xres()/yres() | |
| fpixSetResolution | ✅ | set_resolution() | |
| fpixCopyResolution | ❌ | - | |
| fpixGetData | ✅ | FPix::data() | |
| fpixSetData | ❌ | - | |
| fpixGetPixel | ✅ | FPix::get_pixel() | |
| fpixSetPixel | ✅ | FPix::set_pixel() | |
| fpixaCreate | ❌ | - | Fpixa未実装 |
| fpixaCopy | ❌ | - | |
| fpixaDestroy | ❌ | - | |
| fpixaAddFPix | ❌ | - | |
| fpixaGetCount | ❌ | - | |
| fpixaGetFPix | ❌ | - | |
| fpixaGetFPixDimensions | ❌ | - | |
| fpixaGetData | ❌ | - | |
| fpixaGetPixel | ❌ | - | |
| fpixaSetPixel | ❌ | - | |
| dpixCreate | ✅ | DPix::new() | |
| dpixClone | ✅ | DPix::clone() | |
| dpixCopy | ✅ | DPix::clone() | |
| dpixDestroy | 🔄 | drop() | 自動 |
| fpixRead | ❌ | - | I/O未実装 |
| fpixReadStream | ❌ | - | |
| fpixReadMem | ❌ | - | |
| fpixWrite | ❌ | - | |
| fpixWriteStream | ❌ | - | |
| fpixWriteMem | ❌ | - | |
| dpixRead | ❌ | - | |
| dpixWrite | ❌ | - | |

fpix2.c (FPix変換・演算):

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| fpixConvertToPix | ✅ | FPix::to_pix() | |
| pixConvertToFPix | ✅ | FPix::from_pix() | |
| fpixAddMultConstant | ✅ | FPix::add_mult_constant() | |
| fpixLinearCombination | ✅ | FPix::linear_combination() | |
| dpixConvertToPix | ✅ | DPix::to_pix() | |
| dpixConvertToFPix | ✅ | DPix::to_fpix() | |

その他のfpix2.c変換関数は一部convert.rsに実装あり。

### colormap.c (カラーマップ)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixcmapCreate | ✅ | PixColormap::new() | |
| pixcmapCreateRandom | ❌ | - | |
| pixcmapCreateLinear | ✅ | PixColormap::create_linear() | |
| pixcmapCopy | ✅ | PixColormap::clone() | |
| pixcmapDestroy | 🔄 | drop() | 自動 |
| pixcmapIsValid | ❌ | - | |
| pixcmapAddColor | ✅ | PixColormap::add_color() | |
| pixcmapAddRGBA | ❌ | - | add_colorがRGBA対応 |
| pixcmapAddNewColor | ❌ | - | |
| pixcmapAddNearestColor | ❌ | - | |
| pixcmapUsableColor | ❌ | - | |
| pixcmapAddBlackOrWhite | ❌ | - | |
| pixcmapSetBlackAndWhite | ❌ | - | |
| pixcmapGetCount | ✅ | PixColormap::len() | |
| pixcmapGetFreeCount | ❌ | - | |
| pixcmapGetDepth | ✅ | PixColormap::depth() | |
| pixcmapGetMinDepth | ❌ | - | |
| pixcmapClear | ✅ | PixColormap::clear() | |
| pixcmapGetColor | ✅ | PixColormap::get_color() | |
| pixcmapGetColor32 | ❌ | - | |
| pixcmapGetRGBA | ❌ | - | |
| pixcmapGetRGBA32 | ❌ | - | |
| pixcmapResetColor | ❌ | - | |
| pixcmapSetAlpha | ❌ | - | |
| pixcmapGetIndex | ❌ | - | |
| pixcmapHasColor | ❌ | - | |
| pixcmapIsOpaque | ❌ | - | |
| pixcmapNonOpaqueColorsInfo | ❌ | - | |
| pixcmapIsBlackAndWhite | ❌ | - | |
| pixcmapCountGrayColors | ❌ | - | |
| pixcmapGetRankIntensity | ❌ | - | |
| pixcmapGetNearestIndex | ❌ | - | |
| pixcmapGetNearestGrayIndex | ❌ | - | |
| pixcmapGetDistanceToColor | ❌ | - | |
| pixcmapGetRangeValues | ❌ | - | |
| pixcmapGrayToFalseColor | ❌ | - | |
| pixcmapGrayToColor | ❌ | - | |
| pixcmapColorToGray | ❌ | - | |
| pixcmapConvertTo4 | ❌ | - | |
| pixcmapConvertTo8 | ❌ | - | |
| pixcmapRead | ❌ | - | I/O未実装 |
| pixcmapReadStream | ❌ | - | |
| pixcmapReadMem | ❌ | - | |
| pixcmapWrite | ❌ | - | |
| pixcmapWriteStream | ❌ | - | |
| pixcmapWriteMem | ❌ | - | |
| pixcmapToArrays | ❌ | - | |
| pixcmapToRGBTable | ❌ | - | |
| pixcmapSerializeToMemory | ❌ | - | |
| pixcmapDeserializeFromMemory | ❌ | - | |
| pixcmapConvertToHex | ❌ | - | |
| pixcmapGammaTRC | ❌ | - | |
| pixcmapContrastTRC | ❌ | - | |
| pixcmapShiftIntensity | ❌ | - | |
| pixcmapShiftByComponent | ❌ | - | |

### pixconv.c (ピクセル深度変換)

convert.rsに一部実装あり。多くの関数は未実装。

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixThreshold8 | ❌ | - | |
| pixRemoveColormapGeneral | ✅ | convert.rs remove_colormap_general() | |
| pixRemoveColormap | ✅ | convert.rs remove_colormap() | |
| pixAddGrayColormap8 | ✅ | convert.rs add_gray_colormap8() | |
| pixAddMinimalGrayColormap8 | ✅ | convert.rs add_minimal_gray_colormap8() | |
| pixConvertRGBToLuminance | ✅ | convert.rs convert_rgb_to_luminance() | |
| pixConvertRGBToGrayGeneral | ✅ | convert.rs convert_rgb_to_gray_general() | |
| pixConvertRGBToGray | ✅ | convert.rs convert_rgb_to_gray() | |
| pixConvertRGBToGrayFast | ✅ | convert.rs convert_rgb_to_gray_fast() | |
| pixConvertRGBToGrayMinMax | ✅ | convert.rs convert_rgb_to_gray_min_max() | |
| pixConvertRGBToGraySatBoost | ✅ | convert.rs convert_rgb_to_gray_sat_boost() | |
| pixConvertRGBToGrayArb | ✅ | convert.rs convert_rgb_to_gray_arb() | |
| pixConvertRGBToBinaryArb | ❌ | - | color crate依存 |
| pixConvertGrayToColormap | ✅ | convert.rs convert_gray_to_colormap() | |
| pixConvertGrayToColormap8 | ✅ | convert.rs convert_gray_to_colormap_8() | |
| pixColorizeGray | ✅ | convert.rs colorize_gray() | |
| pixConvertRGBToColormap | ❌ | - | color crate依存 |
| pixConvertCmapTo1 | ✅ | convert.rs convert_cmap_to_1() | |
| pixQuantizeIfFewColors | ❌ | - | color crate依存 |
| pixConvert16To8 | ✅ | convert.rs convert_16_to_8() | |
| pixConvertGrayToFalseColor | ✅ | convert.rs convert_gray_to_false_color() | |
| pixUnpackBinary | ✅ | convert.rs unpack_binary() | |
| pixConvert1To16 | ✅ | convert.rs convert_1_to_16() | |
| pixConvert1To32 | ✅ | convert.rs convert_1_to_32() | |
| pixConvert1To2Cmap | ✅ | convert.rs convert_1_to_2_cmap() | |
| pixConvert1To2 | ✅ | convert.rs convert_1_to_2() | |
| pixConvert1To4Cmap | ✅ | convert.rs convert_1_to_4_cmap() | |
| pixConvert1To4 | ✅ | convert.rs convert_1_to_4() | |
| pixConvert1To8Cmap | ✅ | convert.rs convert_1_to_8_cmap() | |
| pixConvert1To8 | ✅ | convert.rs convert_1_to_8() | |
| pixConvert2To8 | ✅ | convert.rs convert_2_to_8() | |
| pixConvert4To8 | ✅ | convert.rs convert_4_to_8() | |
| pixConvert8To16 | ✅ | convert.rs convert_8_to_16() | |
| pixConvertTo2 | ✅ | convert.rs convert_to_2() | |
| pixConvert8To2 | ✅ | convert.rs convert_8_to_2() | |
| pixConvertTo4 | ✅ | convert.rs convert_to_4() | |
| pixConvert8To4 | ✅ | convert.rs convert_8_to_4() | |
| pixConvertTo1Adaptive | ❌ | - | |
| pixConvertTo1 | ✅ | convert.rs convert_to_1() | |
| pixConvertTo1BySampling | ❌ | - | |
| pixConvertTo8 | ✅ | convert.rs convert_to_8() | |
| pixConvertTo8BySampling | ❌ | - | transform crate依存 |
| pixConvertTo8Colormap | ❌ | - | 32bpp部分は後続 |
| pixConvertTo16 | ✅ | convert.rs convert_to_16() | |
| pixConvertTo32 | ✅ | convert.rs convert_to_32() | |
| pixConvertTo32BySampling | ❌ | - | transform crate依存 |
| pixConvert8To32 | ✅ | convert.rs convert_8_to_32() | |
| pixConvertTo8Or32 | ✅ | convert.rs convert_to_8_or_32() | |
| pixConvert24To32 | ❌ | - | |
| pixConvert32To24 | ❌ | - | |
| pixConvert32To16 | ✅ | convert.rs convert_32_to_16() | |
| pixConvert32To8 | ✅ | convert.rs convert_32_to_8() | |
| pixRemoveAlpha | ✅ | convert.rs remove_alpha() | |
| pixAddAlphaTo1bpp | ✅ | convert.rs add_alpha_to_1bpp() | |
| pixConvertLossless | ✅ | convert.rs convert_lossless() | |
| pixConvertForPSWrap | ✅ | convert.rs convert_for_ps_wrap() | |
| pixConvertToSubpixelRGB | ❌ | - | |
| pixConvertGrayToSubpixelRGB | ❌ | - | |
| pixConvertColorToSubpixelRGB | ❌ | - | |

### pixarith.c (ピクセル算術演算)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixAddGray | ✅ | arith.rs add_gray() | |
| pixSubtractGray | ✅ | arith.rs subtract_gray() | |
| pixMultConstantGray | ✅ | arith.rs multiply_constant() | |
| pixAddConstantGray | ✅ | arith.rs add_constant() | |
| pixMultConstAccumulate | ✅ | arith.rs mult_const_accumulate() | 32bpp専用 |
| pixAbsDifference | ✅ | arith.rs abs_difference() | |
| pixMinOrMax | ✅ | arith.rs min_or_max() | |

その他のpixarith.c関数は未実装。

### rop.c, roplow.c (ラスターオペレーション)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRasterop | ✅ | rop.rsに実装 | |
| pixRasteropVip | ✅ | rop.rs rasterop_vip() | |
| pixRasteropHip | ✅ | rop.rs rasterop_hip() | |
| pixTranslate | ✅ | rop.rs translate() | |
| pixRasteropIP | ❌ | - | |
| pixRasteropFullImage | ❌ | - | |

roplow.c (低レベルラスターOP) 全関数 ❌ 未実装

### compare.c (画像比較)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixEqual | ✅ | compare.rsに実装 | |
| pixEqualWithAlpha | ✅ | compare.rs equals_with_alpha() | |
| pixEqualWithCmap | ✅ | compare.rs equals_with_cmap() | |
| pixCorrelationBinary | ✅ | compare::correlation_binary() | |
| pixDisplayDiff | ✅ | compare.rs display_diff() | |
| pixDisplayDiffBinary | ✅ | compare.rs display_diff_binary() | |
| pixCompareBinary | ✅ | compare::compare_binary() | |
| pixCompareGrayOrRGB | ✅ | compare.rs compare_gray_or_rgb() | |
| pixCompareGray | ✅ | compare.rs compare_gray() | |
| pixCompareRGB | ✅ | compare.rs compare_rgb() | |
| pixCompareTiled | ❌ | - | |
| pixCompareRankDifference | ✅ | compare.rs compare_rank_difference() | |
| pixTestForSimilarity | ✅ | compare.rs test_for_similarity() | |
| pixGetDifferenceStats | ✅ | compare.rs get_difference_stats() | |
| pixGetDifferenceHistogram | ✅ | compare.rs get_difference_histogram() | |
| pixGetPerceptualDiff | ❌ | - | |
| pixGetPSNR | ✅ | compare.rs get_psnr() | |

その他の比較関数も未実装。

### blend.c (ブレンド・合成)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBlend | ✅ | blend.rsに実装 | |
| pixBlendMask | ✅ | blend::blend_mask() | |
| pixBlendGray | ✅ | blend::blend_gray() | |
| pixBlendGrayInverse | ✅ | blend.rs blend_gray_inverse() | |
| pixBlendColor | ✅ | blend::blend_color() | |
| pixBlendColorByChannel | ✅ | blend.rs blend_color_by_channel() | |
| pixBlendGrayAdapt | ✅ | blend.rs blend_gray_adapt() | |
| pixFadeWithGray | ✅ | blend.rs fade_with_gray() | |
| pixBlendHardLight | ✅ | blend.rs blend_hard_light() | |
| pixBlendCmap | ✅ | blend.rs blend_cmap() | |
| pixBlendWithGrayMask | ✅ | blend::blend_with_gray_mask() | |
| pixBlendBackgroundToColor | ❌ | - | |
| pixMultiplyByColor | ✅ | blend.rs multiply_by_color() | |
| pixAlphaBlendUniform | ✅ | blend.rs alpha_blend_uniform() | |
| pixAddAlphaToBlend | ✅ | blend.rs add_alpha_to_blend() | |
| pixSetAlphaOverWhite | ❌ | - | |
| pixLinearEdgeFade | ✅ | blend.rs linear_edge_fade() | |

### graphics.c (描画・レンダリング)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| generatePtaLine | ✅ | graphics.rs generate_line_pta() | |
| generatePtaWideLine | ✅ | graphics.rs generate_wide_line_pta() | |
| generatePtaBox | ✅ | graphics.rs generate_box_pta() | |
| generatePtaBoxa | ✅ | graphics.rs generate_boxa_pta() | |
| generatePtaHashBox | ✅ | graphics.rs generate_hash_box_pta() | |
| generatePtaHashBoxa | ✅ | graphics.rs generate_hash_boxa_pta() | |
| generatePtaaBoxa | ✅ | graphics.rs generate_ptaa_boxa() | |
| generatePtaaHashBoxa | ✅ | graphics.rs generate_ptaa_hash_boxa() | |
| generatePtaPolyline | ✅ | graphics.rs generate_polyline_pta() | |
| generatePtaGrid | ✅ | graphics.rs generate_grid_pta() | |
| convertPtaLineTo4cc | ✅ | graphics.rs convert_line_to_4cc() | |
| generatePtaFilledCircle | ✅ | graphics.rs generate_filled_circle_pta() | |
| generatePtaFilledSquare | ✅ | graphics.rs generate_filled_square_pta() | |
| pixRenderPlotFromNuma | ✅ | graphics.rs render_plot_from_numa() | |
| pixRenderPlotFromNumaGen | ✅ | graphics.rs render_plot_from_numa_gen() | |
| pixRenderPta | ✅ | graphics.rsに部分実装 | |
| pixRenderPtaArb | ✅ | graphics.rs render_pta_color() | |
| pixRenderPtaBlend | ✅ | graphics.rs render_pta_blend() | |
| pixRenderLine | ✅ | graphics::render_line() | |
| pixRenderLineArb | ✅ | graphics.rs render_line_color() | |
| pixRenderLineBlend | ✅ | graphics.rs render_line_blend() | |
| pixRenderBox | ✅ | graphics::render_box() | |
| pixRenderBoxArb | ✅ | graphics.rs render_box_color() | |
| pixRenderBoxBlend | ✅ | graphics.rs render_box_blend() | |
| pixRenderBoxa | ✅ | graphics.rs render_boxa() | |
| pixRenderBoxaArb | ✅ | graphics.rs render_boxa_color() | |
| pixRenderBoxaBlend | ✅ | graphics.rs render_boxa_blend() | |
| pixRenderHashBox | ✅ | graphics.rs render_hash_box() | |
| pixRenderHashBoxArb | ✅ | graphics.rs render_hash_box_color() | |
| pixRenderHashBoxBlend | ✅ | graphics.rs render_hash_box_blend() | |
| pixRenderHashMaskArb | ✅ | graphics.rs render_hash_mask_color() | |
| pixRenderHashBoxa | ✅ | graphics.rs render_hash_boxa() | |
| pixRenderHashBoxaArb | ✅ | graphics.rs render_hash_boxa_color() | |
| pixRenderHashBoxaBlend | ✅ | graphics.rs render_hash_boxa_blend() | |
| pixRenderPolyline | ✅ | graphics.rs render_polyline() | |
| pixRenderPolylineArb | ✅ | graphics.rs render_polyline_color() | |
| pixRenderPolylineBlend | ✅ | graphics.rs render_polyline_blend() | |
| pixRenderGridArb | ✅ | graphics.rs render_grid_color() | |
| pixRenderRandomCmapPtaa | ✅ | graphics.rs render_random_cmap_ptaa() | |
| pixRenderPolygon | ✅ | graphics.rs render_polygon() | |
| pixFillPolygon | ✅ | graphics.rs fill_polygon() | |
| pixRenderContours | ✅ | graphics.rs render_contours() | |
| fpixAutoRenderContours | ❌ | - | FPix関連は後続 |
| fpixRenderContours | ❌ | - | FPix関連は後続 |
| pixGeneratePtaBoundary | ❌ | - | 後続Phase |

## 結論

leptonica-coreクレートは、Phase 13-17の実装により大幅にカバレッジが向上した（26.7% → 52.8%）。
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

### 未実装領域
- I/O操作全般（Read/Write/Stream/Mem）— Phase 10で計画
- カラーマップの高度な操作（検索・変換・効果）— Phase 12で計画
- FPix/DPix の拡張（FPixa、シリアライゼーション）
- roplow.c（低レベルビット操作）— Rust版rop.rsの高レベルAPIでカバー済み、スキップ
- boxfunc2.c, boxfunc5.c（Boxスムージング）
- ptafunc1.c, ptafunc2.c の一部
- pixafunc1.c, pixafunc2.c の一部（表示・変換の詳細）

残りは主にI/O・シリアライゼーション（Phase 10）とカラーマップ拡張（Phase 12）が中心。
