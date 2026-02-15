# leptonica-core: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 82 |
| 🔄 異なる | 24 |
| ❌ 未実装 | 742 |
| 合計 | 848 |

**カバレッジ**: 12.5% (106/848 関数が何らかの形で実装済み)

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
| pixGetRGBPixel | ❌ | - | RGBコンポーネント分離は未実装 |
| pixSetRGBPixel | ❌ | - | |
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
| pixSetBorderVal | ❌ | - | |
| pixSetBorderRingVal | ❌ | - | |
| pixSetMirroredBorder | ❌ | - | |
| pixCopyBorder | ❌ | - | |
| pixAddBorder | ❌ | - | border.rsに部分実装あり |
| pixAddBlackOrWhiteBorder | ❌ | - | |
| pixAddBorderGeneral | ❌ | - | |
| pixAddMultipleBlackWhiteBorders | ❌ | - | |
| pixRemoveBorder | ❌ | - | |
| pixRemoveBorderGeneral | ❌ | - | |
| pixRemoveBorderToSize | ❌ | - | |
| pixAddMirroredBorder | ❌ | - | |
| pixAddRepeatedBorder | ❌ | - | |
| pixAddMixedBorder | ❌ | - | |
| pixAddContinuedBorder | ❌ | - | |
| pixShiftAndTransferAlpha | ❌ | - | |
| pixDisplayLayersRGBA | ❌ | - | |
| pixCreateRGBImage | ❌ | - | |
| pixGetRGBComponent | ❌ | - | |
| pixSetRGBComponent | ❌ | - | |
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
| pixSetMasked | ❌ | - | |
| pixSetMaskedGeneral | ❌ | - | |
| pixCombineMasked | ❌ | - | |
| pixCombineMaskedGeneral | ❌ | - | |
| pixPaintThroughMask | ❌ | - | |
| pixCopyWithBoxa | ❌ | - | |
| pixPaintSelfThroughMask | ❌ | - | |
| pixMakeMaskFromVal | ❌ | - | |
| pixMakeMaskFromLUT | ❌ | - | |
| pixMakeArbMaskFromRGB | ❌ | - | |
| pixSetUnderTransparency | ❌ | - | |
| pixMakeAlphaFromMask | ❌ | - | |
| pixGetColorNearMaskBoundary | ❌ | - | |
| pixDisplaySelectedPixels | ❌ | - | |
| pixInvert | ✅ | ops.rsに実装 | |
| pixOr | ✅ | ops.rsに実装 | |
| pixAnd | ✅ | ops.rsに実装 | |
| pixXor | ✅ | ops.rsに実装 | |
| pixSubtract | ✅ | ops.rsに実装 | |
| pixZero | ❌ | - | |
| pixForegroundFraction | ❌ | - | |
| pixaCountPixels | ❌ | - | |
| pixCountPixels | ❌ | - | statistics.rsに関連実装あり |
| pixCountPixelsInRect | ❌ | - | |
| pixCountByRow | ❌ | - | |
| pixCountByColumn | ❌ | - | |
| pixCountPixelsByRow | ❌ | - | |
| pixCountPixelsByColumn | ❌ | - | |
| pixCountPixelsInRow | ❌ | - | |
| pixGetMomentByColumn | ❌ | - | |
| pixThresholdPixelSum | ❌ | - | |
| pixAverageByRow | ❌ | - | |
| pixAverageByColumn | ❌ | - | |
| pixAverageInRect | ❌ | - | |
| pixAverageInRectRGB | ❌ | - | |
| pixVarianceByRow | ❌ | - | |
| pixVarianceByColumn | ❌ | - | |
| pixVarianceInRect | ❌ | - | |
| pixAbsDiffByRow | ❌ | - | |
| pixAbsDiffByColumn | ❌ | - | |
| pixAbsDiffInRect | ❌ | - | |
| pixAbsDiffOnLine | ❌ | - | |
| pixCountArbInRect | ❌ | - | |
| pixMirroredTiling | ❌ | - | |
| pixFindRepCloseTile | ❌ | - | |

### pix4.c (ヒストグラム・統計)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixGetGrayHistogram | ✅ | histogram.rsに実装 | |
| pixGetGrayHistogramMasked | ❌ | - | |
| pixGetGrayHistogramInRect | ❌ | - | |
| pixGetGrayHistogramTiled | ❌ | - | |
| pixGetColorHistogram | ✅ | histogram.rsに実装 | |
| pixGetColorHistogramMasked | ❌ | - | |
| pixGetCmapHistogram | ❌ | - | |
| pixGetCmapHistogramMasked | ❌ | - | |
| pixGetCmapHistogramInRect | ❌ | - | |
| pixCountRGBColorsByHash | ❌ | - | |
| pixCountRGBColors | ❌ | - | |
| pixGetColorAmapHistogram | ❌ | - | |
| pixGetRankValue | ❌ | - | |
| pixGetRankValueMaskedRGB | ❌ | - | |
| pixGetRankValueMasked | ❌ | - | |
| pixGetPixelAverage | ❌ | - | |
| pixGetPixelStats | ❌ | - | |
| pixGetAverageMaskedRGB | ❌ | - | |
| pixGetAverageMasked | ❌ | - | |
| pixGetAverageTiledRGB | ❌ | - | |
| pixGetAverageTiled | ❌ | - | |
| pixRowStats | ❌ | - | |
| pixColumnStats | ❌ | - | |
| pixGetRangeValues | ❌ | - | |
| pixGetExtremeValue | ❌ | - | |
| pixGetMaxValueInRect | ❌ | - | |
| pixGetMaxColorIndex | ❌ | - | |
| pixGetBinnedComponentRange | ❌ | - | |
| pixGetRankColorArray | ❌ | - | |
| pixGetBinnedColor | ❌ | - | |
| pixDisplayColorArray | ❌ | - | |
| pixRankBinByStrip | ❌ | - | |
| pixaGetAlignedStats | ❌ | - | |
| pixaExtractColumnFromEachPix | ❌ | - | |
| pixGetRowStats | ❌ | - | |
| pixGetColumnStats | ❌ | - | |
| pixSetPixelColumn | ❌ | - | |
| pixThresholdForFgBg | ❌ | - | |
| pixSplitDistributionFgBg | ❌ | - | |

### pix5.c (選択・測定)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaFindDimensions | ❌ | - | |
| pixFindAreaPerimRatio | ❌ | - | |
| pixaFindPerimToAreaRatio | ❌ | - | |
| pixFindPerimToAreaRatio | ❌ | - | |
| pixaFindPerimSizeRatio | ❌ | - | |
| pixFindPerimSizeRatio | ❌ | - | |
| pixaFindAreaFraction | ❌ | - | |
| pixFindAreaFraction | ❌ | - | |
| pixaFindAreaFractionMasked | ❌ | - | |
| pixFindAreaFractionMasked | ❌ | - | |
| pixaFindWidthHeightRatio | ❌ | - | |
| pixaFindWidthHeightProduct | ❌ | - | |
| pixFindOverlapFraction | ❌ | - | |
| pixFindRectangleComps | ❌ | - | |
| pixConformsToRectangle | ❌ | - | |
| pixExtractRectangularRegions | ❌ | - | |
| pixClipRectangles | ❌ | - | clip.rsに関連実装あり |
| pixClipRectangle | ❌ | - | |
| pixClipRectangleWithBorder | ❌ | - | |
| pixClipMasked | ❌ | - | |
| pixCropToMatch | ❌ | - | |
| pixCropToSize | ❌ | - | |
| pixResizeToMatch | ❌ | - | |
| pixSelectComponentBySize | ❌ | - | |
| pixFilterComponentBySize | ❌ | - | |
| pixMakeSymmetricMask | ❌ | - | |
| pixMakeFrameMask | ❌ | - | |
| pixMakeCoveringOfRectangles | ❌ | - | |
| pixFractionFgInMask | ❌ | - | |
| pixClipToForeground | ❌ | - | |
| pixTestClipToForeground | ❌ | - | |
| pixClipBoxToForeground | ❌ | - | |
| pixScanForForeground | ❌ | - | |
| pixClipBoxToEdges | ❌ | - | |
| pixScanForEdge | ❌ | - | |
| pixExtractOnLine | ❌ | - | extract.rsに関連実装あり |
| pixAverageOnLine | ❌ | - | |
| pixAverageIntensityProfile | ❌ | - | |
| pixReversalProfile | ❌ | - | |
| pixWindowedVarianceOnLine | ❌ | - | |
| pixMinMaxNearLine | ❌ | - | |
| pixRankRowTransform | ❌ | - | |
| pixRankColumnTransform | ❌ | - | |

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
| boxaContainedInBox | ❌ | - | |
| boxaContainedInBoxCount | ❌ | - | |
| boxaContainedInBoxa | ❌ | - | |
| boxaIntersectsBox | ❌ | - | |
| boxaIntersectsBoxCount | ❌ | - | |
| boxaClipToBox | ❌ | - | |
| boxaCombineOverlaps | ❌ | - | |
| boxaCombineOverlapsInPair | ❌ | - | |
| boxOverlapRegion | ✅ | Box::intersect() | |
| boxBoundingRegion | ✅ | Box::union() | |
| boxOverlapFraction | ❌ | - | |
| boxOverlapArea | ❌ | - | |
| boxaHandleOverlaps | ❌ | - | |
| boxOverlapDistance | ❌ | - | |
| boxSeparationDistance | ❌ | - | |
| boxCompareSize | ❌ | - | |
| boxContainsPt | ✅ | Box::contains_point() | |
| boxaGetNearestToPt | ❌ | - | |
| boxaGetNearestToLine | ❌ | - | |
| boxaFindNearestBoxes | ❌ | - | |
| boxaGetNearestByDirection | ❌ | - | |
| boxGetCenter | ❌ | - | center_x()/center_y()で対応 |
| boxIntersectByLine | ❌ | - | |
| boxClipToRectangle | ❌ | - | Box::clip()で類似 |
| boxClipToRectangleParams | ❌ | - | |
| boxRelocateOneSide | ❌ | - | |
| boxaAdjustSides | ❌ | - | |
| boxaAdjustBoxSides | ❌ | - | |
| boxAdjustSides | ❌ | - | |
| boxaSetSide | ❌ | - | |
| boxSetSide | ❌ | - | |
| boxaAdjustWidthToTarget | ❌ | - | |
| boxaAdjustHeightToTarget | ❌ | - | |
| boxEqual | ❌ | - | PartialEqで対応可 |
| boxaEqual | ❌ | - | |
| boxSimilar | ❌ | - | |
| boxaSimilar | ❌ | - | |
| boxaJoin | ❌ | - | |
| boxaaJoin | ❌ | - | |
| boxaSplitEvenOdd | ❌ | - | |
| boxaMergeEvenOdd | ❌ | - | |

### boxfunc2.c (未実装)
全関数 ❌ 未実装

### boxfunc3.c (Box描画・マスク)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixMaskConnComp | ❌ | - | |
| pixMaskBoxa | ❌ | - | |
| pixPaintBoxa | ❌ | - | |
| pixSetBlackOrWhiteBoxa | ❌ | - | |
| pixPaintBoxaRandom | ❌ | - | |
| pixBlendBoxaRandom | ❌ | - | |
| pixDrawBoxa | ❌ | - | graphics.rsに関連実装あり |
| pixDrawBoxaRandom | ❌ | - | |
| boxaaDisplay | ❌ | - | |
| pixaDisplayBoxaa | ❌ | - | |
| pixSplitIntoBoxa | ❌ | - | |
| pixSplitComponentIntoBoxa | ❌ | - | |
| makeMosaicStrips | ❌ | - | |
| boxaCompareRegions | ❌ | - | |
| pixSelectLargeULComp | ❌ | - | |
| boxaSelectLargeULBox | ❌ | - | |

### boxfunc4.c (Box選択・変換)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| boxaSelectRange | ❌ | - | |
| boxaaSelectRange | ❌ | - | |
| boxaSelectBySize | ❌ | - | |
| boxaMakeSizeIndicator | ❌ | - | |
| boxaSelectByArea | ❌ | - | |
| boxaMakeAreaIndicator | ❌ | - | |
| boxaSelectByWHRatio | ❌ | - | |
| boxaMakeWHRatioIndicator | ❌ | - | |
| boxaSelectWithIndicator | ❌ | - | |
| boxaPermutePseudorandom | ❌ | - | |
| boxaPermuteRandom | ❌ | - | |
| boxaSwapBoxes | ❌ | - | |
| boxaConvertToPta | ❌ | - | |
| ptaConvertToBoxa | ❌ | - | |
| boxConvertToPta | ❌ | - | |
| ptaConvertToBox | ❌ | - | |
| boxaGetExtent | ❌ | - | Boxa::bounding_box()で類似 |
| boxaGetCoverage | ❌ | - | |
| boxaaSizeRange | ❌ | - | |
| boxaSizeRange | ❌ | - | |
| boxaLocationRange | ❌ | - | |
| boxaGetSizes | ❌ | - | |
| boxaGetArea | ❌ | - | |
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
全関数 ❌ 未実装 (ポイント配列の変換、幾何演算、最小二乗法など)

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
ほぼすべて ❌ 未実装 (選択、ソート、スケール、表示など)

### numabasic.c (Numa基本操作)

実装済み関数が存在するが、C版のnumabasic.cはI/O関連なので未実装。
numa/mod.rs, numa/operations.rs に基本統計関数は実装済み。

### numafunc1.c, numafunc2.c (Numa演算・統計)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| numaArithOp | ❌ | - | |
| numaLogicalOp | ❌ | - | |
| numaInvert | ❌ | - | |
| numaSimilar | ❌ | - | |
| numaAddToNumber | ❌ | - | |
| numaGetMin | ✅ | Numa::min() | |
| numaGetMax | ✅ | Numa::max() | |
| numaGetSum | ✅ | Numa::sum() | |
| numaGetPartialSums | ❌ | - | |
| numaGetSumOnInterval | ✅ | Numa::sum_on_interval() | |
| numaHasOnlyIntegers | ✅ | Numa::has_only_integers() | |
| numaGetMean | ✅ | Numa::mean() | |
| numaGetMeanAbsval | ✅ | Numa::mean_absval() | |
| numaSubsample | ❌ | - | |
| numaMakeDelta | ❌ | - | |
| numaMakeSequence | ❌ | - | |
| numaMakeConstant | ❌ | - | |
| numaMakeAbsval | ❌ | - | |
| numaAddBorder | ❌ | - | |
| numaAddSpecifiedBorder | ❌ | - | |
| numaRemoveBorder | ❌ | - | |
| numaCountNonzeroRuns | ❌ | - | |
| numaGetNonzeroRange | ❌ | - | |
| numaGetCountRelativeToZero | ❌ | - | |
| numaClipToInterval | ❌ | - | |
| numaMakeThresholdIndicator | ❌ | - | |
| numaUniformSampling | ❌ | - | |
| numaReverse | ❌ | - | |
| numaLowPassIntervals | ❌ | - | |
| numaThresholdEdges | ❌ | - | |
| numaGetSpanValues | ❌ | - | |
| numaGetEdgeValues | ❌ | - | |
| numaInterpolateEqxVal | ❌ | - | |
| numaInterpolateArbxVal | ❌ | - | |
| numaInterpolateEqxInterval | ❌ | - | |
| numaInterpolateArbxInterval | ❌ | - | |
| numaFitMax | ❌ | - | |
| numaDifferentiateInterval | ❌ | - | |
| numaIntegrateInterval | ❌ | - | |
| numaSortGeneral | ❌ | - | |
| numaSortAutoSelect | ❌ | - | |
| numaSortIndexAutoSelect | ❌ | - | |
| numaChooseSortType | ❌ | - | |
| numaSort | ❌ | - | |
| numaBinSort | ❌ | - | |
| numaGetSortIndex | ❌ | - | |
| numaGetBinSortIndex | ❌ | - | |
| numaSortByIndex | ❌ | - | |
| numaIsSorted | ❌ | - | |
| numaSortPair | ❌ | - | |
| numaInvertMap | ❌ | - | |
| numaAddSorted | ❌ | - | |
| numaFindSortedLoc | ❌ | - | |
| numaPseudorandomSequence | ❌ | - | |
| numaRandomPermutation | ❌ | - | |
| numaGetRankValue | ❌ | - | |
| numaGetMedian | ❌ | - | |
| numaGetBinnedMedian | ❌ | - | |
| numaGetMeanDevFromMedian | ❌ | - | |
| numaGetMedianDevFromMedian | ❌ | - | |
| numaGetMode | ❌ | - | |
| numaJoin | ❌ | - | |
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
| fpixCreateTemplate | ❌ | - | |
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
| dpixCreate | ❌ | - | DPix未実装 |
| dpixClone | ❌ | - | |
| dpixCopy | ❌ | - | |
| dpixDestroy | ❌ | - | |
| fpixRead | ❌ | - | I/O未実装 |
| fpixReadStream | ❌ | - | |
| fpixReadMem | ❌ | - | |
| fpixWrite | ❌ | - | |
| fpixWriteStream | ❌ | - | |
| fpixWriteMem | ❌ | - | |
| dpixRead | ❌ | - | |
| dpixWrite | ❌ | - | |

fpix2.c (FPix変換・演算)の関数も多くが未実装。
一部変換関数はconvert.rsに実装あり。

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
| pixRemoveColormapGeneral | ❌ | - | |
| pixRemoveColormap | ❌ | - | |
| pixAddGrayColormap8 | ❌ | - | |
| pixAddMinimalGrayColormap8 | ❌ | - | |
| pixConvertRGBToLuminance | ❌ | - | |
| pixConvertRGBToGrayGeneral | ❌ | - | |
| pixConvertRGBToGray | ❌ | - | |
| pixConvertRGBToGrayFast | ❌ | - | |
| pixConvertRGBToGrayMinMax | ❌ | - | |
| pixConvertRGBToGraySatBoost | ❌ | - | |
| pixConvertRGBToGrayArb | ❌ | - | |
| pixConvertRGBToBinaryArb | ❌ | - | |
| pixConvertGrayToColormap | ❌ | - | |
| pixConvertGrayToColormap8 | ❌ | - | |
| pixColorizeGray | ❌ | - | |
| pixConvertRGBToColormap | ❌ | - | |
| pixConvertCmapTo1 | ❌ | - | |
| pixQuantizeIfFewColors | ❌ | - | |
| pixConvert16To8 | ❌ | - | |
| pixConvertGrayToFalseColor | ❌ | - | |
| pixUnpackBinary | ❌ | - | |
| pixConvert1To16 | ❌ | - | |
| pixConvert1To32 | ❌ | - | |
| pixConvert1To2Cmap | ❌ | - | |
| pixConvert1To2 | ❌ | - | |
| pixConvert1To4Cmap | ❌ | - | |
| pixConvert1To4 | ❌ | - | |
| pixConvert1To8Cmap | ❌ | - | |
| pixConvert1To8 | ❌ | - | |
| pixConvert2To8 | ❌ | - | |
| pixConvert4To8 | ❌ | - | |
| pixConvert8To16 | ❌ | - | |
| pixConvertTo2 | ❌ | - | |
| pixConvert8To2 | ❌ | - | |
| pixConvertTo4 | ❌ | - | |
| pixConvert8To4 | ❌ | - | |
| pixConvertTo1Adaptive | ❌ | - | |
| pixConvertTo1 | ❌ | - | |
| pixConvertTo1BySampling | ❌ | - | |
| pixConvertTo8 | ❌ | - | |
| pixConvertTo8BySampling | ❌ | - | |
| pixConvertTo8Colormap | ❌ | - | |
| pixConvertTo16 | ❌ | - | |
| pixConvertTo32 | ❌ | - | |
| pixConvertTo32BySampling | ❌ | - | |
| pixConvert8To32 | ❌ | - | |
| pixConvertTo8Or32 | ❌ | - | |
| pixConvert24To32 | ❌ | - | |
| pixConvert32To24 | ❌ | - | |
| pixConvert32To16 | ❌ | - | |
| pixConvert32To8 | ❌ | - | |
| pixRemoveAlpha | ❌ | - | |
| pixAddAlphaTo1bpp | ❌ | - | |
| pixConvertLossless | ❌ | - | |
| pixConvertForPSWrap | ❌ | - | |
| pixConvertToSubpixelRGB | ❌ | - | |
| pixConvertGrayToSubpixelRGB | ❌ | - | |
| pixConvertColorToSubpixelRGB | ❌ | - | |

### pixarith.c (ピクセル算術演算)

arith.rsに実装あり。

全関数 ❌ 未実装

### rop.c, roplow.c (ラスターオペレーション)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixRasterop | ✅ | rop.rsに実装 | |
| pixRasteropVip | ❌ | - | |
| pixRasteropHip | ❌ | - | |
| pixTranslate | ❌ | - | |
| pixRasteropIP | ❌ | - | |
| pixRasteropFullImage | ❌ | - | |

roplow.c (低レベルラスターOP) 全関数 ❌ 未実装

### compare.c (画像比較)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixEqual | ✅ | compare.rsに実装 | |
| pixEqualWithAlpha | ❌ | - | |
| pixEqualWithCmap | ❌ | - | |
| pixCorrelationBinary | ✅ | compare::correlation_binary() | |
| pixDisplayDiff | ❌ | - | |
| pixDisplayDiffBinary | ❌ | - | |
| pixCompareBinary | ✅ | compare::compare_binary() | |
| pixCompareGrayOrRGB | ❌ | - | |
| pixCompareGray | ❌ | - | |
| pixCompareRGB | ❌ | - | |
| pixCompareTiled | ❌ | - | |
| pixCompareRankDifference | ❌ | - | |
| pixTestForSimilarity | ❌ | - | |
| pixGetDifferenceStats | ❌ | - | |
| pixGetDifferenceHistogram | ❌ | - | |
| pixGetPerceptualDiff | ❌ | - | |
| pixGetPSNR | ❌ | - | |

その他の比較関数も未実装。

### blend.c (ブレンド・合成)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixBlend | ✅ | blend.rsに実装 | |
| pixBlendMask | ✅ | blend::blend_mask() | |
| pixBlendGray | ✅ | blend::blend_gray() | |
| pixBlendGrayInverse | ❌ | - | |
| pixBlendColor | ✅ | blend::blend_color() | |
| pixBlendColorByChannel | ❌ | - | |
| pixBlendGrayAdapt | ❌ | - | |
| pixFadeWithGray | ❌ | - | |
| pixBlendHardLight | ❌ | - | |
| pixBlendCmap | ❌ | - | |
| pixBlendWithGrayMask | ✅ | blend::blend_with_gray_mask() | |
| pixBlendBackgroundToColor | ❌ | - | |
| pixMultiplyByColor | ❌ | - | |
| pixAlphaBlendUniform | ❌ | - | |
| pixAddAlphaToBlend | ❌ | - | |
| pixSetAlphaOverWhite | ❌ | - | |
| pixLinearEdgeFade | ❌ | - | |

### graphics.c (描画・レンダリング)

| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| generatePtaLine | ❌ | - | |
| generatePtaWideLine | ❌ | - | |
| generatePtaBox | ❌ | - | |
| generatePtaBoxa | ❌ | - | |
| generatePtaHashBox | ❌ | - | |
| generatePtaHashBoxa | ❌ | - | |
| generatePtaaBoxa | ❌ | - | |
| generatePtaaHashBoxa | ❌ | - | |
| generatePtaPolyline | ❌ | - | |
| generatePtaGrid | ❌ | - | |
| convertPtaLineTo4cc | ❌ | - | |
| generatePtaFilledCircle | ❌ | - | |
| generatePtaFilledSquare | ❌ | - | |
| pixRenderPlotFromNuma | ❌ | - | |
| pixRenderPlotFromNumaGen | ❌ | - | |
| pixRenderPta | ✅ | graphics.rsに部分実装 | |
| pixRenderPtaArb | ❌ | - | |
| pixRenderPtaBlend | ❌ | - | |
| pixRenderLine | ✅ | graphics::render_line() | |
| pixRenderLineArb | ❌ | - | |
| pixRenderLineBlend | ❌ | - | |
| pixRenderBox | ✅ | graphics::render_box() | |
| pixRenderBoxArb | ❌ | - | |
| pixRenderBoxBlend | ❌ | - | |
| pixRenderBoxa | ❌ | - | |
| pixRenderBoxaArb | ❌ | - | |
| pixRenderBoxaBlend | ❌ | - | |
| pixRenderHashBox | ❌ | - | |
| pixRenderHashBoxArb | ❌ | - | |
| pixRenderHashBoxBlend | ❌ | - | |
| pixRenderHashMaskArb | ❌ | - | |
| pixRenderHashBoxa | ❌ | - | |
| pixRenderHashBoxaArb | ❌ | - | |
| pixRenderHashBoxaBlend | ❌ | - | |
| pixRenderPolyline | ❌ | - | |
| pixRenderPolylineArb | ❌ | - | |
| pixRenderPolylineBlend | ❌ | - | |
| pixRenderGridArb | ❌ | - | |
| pixRenderRandomCmapPtaa | ❌ | - | |
| pixRenderPolygon | ❌ | - | |
| pixFillPolygon | ❌ | - | |
| pixRenderContours | ❌ | - | |
| fpixAutoRenderContours | ❌ | - | |
| fpixRenderContours | ❌ | - | |
| pixGeneratePtaBoundary | ❌ | - | |

## 結論

leptonica-coreクレートは、基本的なデータ構造（Pix, Box, Numa, Pta, Pixa, FPix, Colormap, Sarray）の
作成・破棄・基本アクセサは実装済みだが、高度な操作（変換、統計、描画、I/O）の大部分が未実装。

### 実装済み領域
- Pix/PixMut: 基本的な作成・アクセス・プロパティ取得
- Box/Boxa/Boxaa: 基本構造と幾何演算（交差・結合・包含判定）
- Numa/Numaa: 基本統計（min/max/sum/mean）
- Pta: 基本的なポイント配列操作
- Pixa: 基本的なPix配列管理
- Sarray: 基本的な文字列配列操作
- FPix: 基本的な浮動小数点画像
- PixColormap: 基本的なカラーマップ操作
- ピクセル演算: OR/AND/XOR/SUBTRACT/INVERT
- Rasterop: 基本的なラスター演算
- 比較: equal, correlation_binary
- ブレンド: 基本的なブレンド操作
- 描画: Line, Box の基本描画

### 未実装領域
- I/O操作全般（Read/Write/Stream/Mem）
- 深度変換（pixconv.c）のほとんど
- ヒストグラム処理の高度な機能
- 統計処理の高度な機能
- マスク操作
- ボーダー処理の詳細
- RGB成分操作
- Pta/Ptaa の変換・演算
- Pixa/Pixaa の選択・ソート・表示
- Numa の高度な演算・ソート・補間
- FPix/DPix の変換・演算
- Box の高度な調整・スムージング
- 描画の高度な機能（ハッシュ、ポリゴン、輪郭）
- アルファチャンネル操作

今後の実装優先度は、具体的なユースケースに応じて決定すべき。
