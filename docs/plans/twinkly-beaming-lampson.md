# leptonica-core 全関数移植計画 (Phase 10-17)

Status: IN_PROGRESS (Phase 13.1 完了)

## Context

leptonica-core クレートの実装率は 26.7%（226/845 関数）でワークスペース内最低。
845関数のうち619が未実装だが、以下を除外して実質的な移植対象を絞る:

- **roplow.c (~50関数)**: 低レベルビット操作。rop.rs の高レベルAPIでカバー済み → **スキップ**
- **メモリ管理関数**: Create/Destroy/Clone/Copy → Rust の new/Drop/Clone で **実装済み扱い**
- **N/A関数**: pixSetWidth等の不変型setters, ExtendArray等のVec自動拡張 → **スキップ**

これらを除いた実質的な移植対象は **約450関数**。

### 設計上の決定事項

1. **シリアライゼーション**: leptonica-io ではなく leptonica-core に配置（データ構造と密結合のため）
2. **Read/Write トレイト**: C版の `FILE*` / `Mem` / `Stream` の3パターンを `std::io::Read/Write` で統一
3. **Ptaa/Pixaa/FPixa**: 必要に応じて新規データ構造を追加

---

## 実行順序

Phase 10 → 11 → 12 → 13 → 14 → 15 → 16 → 17 の順に**直列で**実行する。
1つのPRがマージされるまで次のPRの実装を開始しない。

```
Phase 10 (Serialization)
  → Phase 11 (Pix utilities)
    → Phase 12 (Colormap)
      → Phase 13 (Depth conversion)
        → Phase 14 (Box operations)
          → Phase 15 (Pix mask/stats/clip)
            → Phase 16 (Numa/Pta/Pixa)
              → Phase 17 (Graphics/Compare/Blend)
```

---

## 共通ワークフロー

### TDD

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### PRワークフロー（厳守）

1. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
2. `/gh-pr-create` でPR作成
3. **GitHub Copilotレビューを必ず待つ（3〜10分かかる）**
   - `/gh-actions-check` でレビューの到着を確認する
   - **レビュー到着前にマージしない**。これは最も頻発する違反であり、絶対に守ること
4. `/gh-pr-review` でレビューコメントを確認し、各指摘に対して以下を判断:
   - **妥当な指摘**: コード修正を行い、修正内容をPRコメントに返信
   - **不要・誤検知**: 理由を添えてPRコメントに返信し、修正しない旨を明示
   - いずれの場合も**PRコメントへの返信は必須**（無視しない）
5. 修正コミット後、CIパスを確認
6. **再レビューは来ない**。修正反映後、`/gh-actions-check` でレビューワークフローが走っていないことを確認したらマージしてよい
7. `/gh-pr-merge --merge` でマージ
8. ブランチ削除

**禁止事項（再掲）**:
- レビュー到着前のマージ（最重要）
- レビュー指摘を確認せずにマージ
- PRコメントに返信せずにマージ
- 「変更が少ないから」「自明だから」でレビュー確認を省略

### ブランチ戦略

```
main
└── feat/core-<feature>           ← PR対象ブランチ
    ├── feat/core-<feature>-<sub>    ← 作業単位
    └── feat/core-<feature>-<sub2>
```

---

## Phase 10: シリアライゼーション基盤（~90関数, 5 PR）

全データ構造の Read/Write を統一パターンで実装。

### 共通パターン

```rust
// C版の3パターンを統一
// xxxRead(filename) + xxxReadStream(fp) + xxxReadMem(data, size)
// → read_from_file(path) + read_from_reader(reader: impl Read)

pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> { ... }
pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
    let file = File::open(path)?;
    Self::read_from_reader(&mut BufReader::new(file))
}
pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
    Self::read_from_reader(&mut Cursor::new(data))
}

// 同様に write_to_writer / write_to_file / write_to_bytes
```

### 10.1 Pix シリアライゼーション (`feat/core-pix-serial`)

対象: pixRead/pixReadStream/pixReadMem/pixWrite/pixWriteStream/pixWriteMem + デバッグ出力

修正ファイル: `crates/leptonica-core/src/pix/serial.rs`（新規）

**注**: 画像フォーマット(PNG/JPEG等)のI/Oは leptonica-io の責務。ここでは leptonica 独自バイナリフォーマットの読み書きのみ。

### 10.2 Box/Boxa/Boxaa シリアライゼーション (`feat/core-box-serial`)

対象: boxaRead/boxaReadStream/boxaReadMem/boxaWrite/boxaWriteStream/boxaWriteMem + boxaaRead/Write系 + boxPrintStreamInfo + boxaWriteDebug/boxaWriteStderr

修正ファイル: `crates/leptonica-core/src/box_/serial.rs`（新規）

### 10.3 Numa/Numaa シリアライゼーション (`feat/core-numa-serial`)

対象: numaRead/numaReadStream/numaReadMem/numaWrite/numaWriteStream/numaWriteMem + numaaRead/Write系

修正ファイル: `crates/leptonica-core/src/numa/serial.rs`（新規）

### 10.4 Pta/Ptaa/Pixa/Pixaa シリアライゼーション (`feat/core-collection-serial`)

対象: ptaRead/Write系 + ptaaRead/Write系 + pixaRead/Write系 + pixaaRead/Write系

修正ファイル:
- `crates/leptonica-core/src/pta/serial.rs`（新規）
- `crates/leptonica-core/src/pixa/serial.rs`（新規）

### 10.5 FPix/DPix/Sarray/Colormap シリアライゼーション (`feat/core-misc-serial`)

対象: fpixRead/Write系 + dpixRead/Write系 + sarrayRead/Write系 + pixcmapRead/Write系 + pixcmapSerialize/Deserialize

修正ファイル:
- `crates/leptonica-core/src/fpix/serial.rs`（新規）
- `crates/leptonica-core/src/sarray/serial.rs`（新規）
- `crates/leptonica-core/src/colormap/serial.rs`（新規）

---

## Phase 11: Pix ユーティリティ（~45関数, 3 PR）

### 11.1 Pix 生成・テンプレート (`feat/core-pix-create`)

対象（pix1.c）:
- pixCreateTemplate, pixCreateTemplateNoInit, pixCreateWithCmap
- pixCopyColormap, pixSizesEqual, pixMaxAspectRatio
- pixCopyResolution, pixScaleResolution, pixCopyInputFormat
- pixAddText, pixCopyText, pixPrintStreamInfo

修正ファイル: `crates/leptonica-core/src/pix/mod.rs`, `access.rs`

### 11.2 ピクセル設定・ボーダー拡張 (`feat/core-pix-setters`)

対象（pix2.c）:
- pixSetAllGray, pixSetAllArbitrary, pixSetBlackOrWhite, pixSetComponentArbitrary
- pixClearInRect, pixSetInRect, pixSetInRectArbitrary, pixBlendInRect
- pixSetPadBits, pixSetPadBitsBand
- pixSetOrClearBorder, pixSetBorderRingVal, pixSetMirroredBorder, pixCopyBorder
- pixAddMultipleBlackWhiteBorders, pixRemoveBorderToSize
- pixAddMixedBorder, pixAddContinuedBorder
- pixGetBlackOrWhiteVal, pixClearPixel, pixFlipPixel, pixGetRandomPixel

修正ファイル: `crates/leptonica-core/src/pix/access.rs`, `border.rs`

### 11.3 RGB成分・アルファ操作 (`feat/core-pix-rgb`)

対象（pix2.c）:
- pixGetRGBComponentCmap, pixCopyRGBComponent, pixGetRGBLine
- composeRGBPixel, composeRGBAPixel, extractRGBValues, extractRGBAValues
- extractMinMaxComponent
- pixShiftAndTransferAlpha, pixDisplayLayersRGBA
- pixAlphaIsOpaque, pixInferResolution
- pixEndianByteSwapNew, pixEndianByteSwap, pixEndianTwoByteSwap
- pixGetRasterData, pixSetCmapPixel

修正ファイル: `crates/leptonica-core/src/pix/rgb.rs`, `access.rs`

---

## Phase 12: カラーマップ操作（~35関数, 2 PR）

### 12.1 カラーマップ検索・情報 (`feat/core-cmap-query`)

対象（colormap.c）:
- pixcmapCreateRandom, pixcmapIsValid
- pixcmapAddRGBA, pixcmapAddNewColor, pixcmapAddNearestColor
- pixcmapUsableColor, pixcmapAddBlackOrWhite, pixcmapSetBlackAndWhite
- pixcmapGetFreeCount, pixcmapGetMinDepth
- pixcmapGetColor32, pixcmapGetRGBA, pixcmapGetRGBA32
- pixcmapResetColor, pixcmapSetAlpha, pixcmapGetIndex
- pixcmapHasColor, pixcmapIsOpaque, pixcmapNonOpaqueColorsInfo
- pixcmapIsBlackAndWhite, pixcmapCountGrayColors
- pixcmapGetRankIntensity, pixcmapGetNearestIndex, pixcmapGetNearestGrayIndex
- pixcmapGetDistanceToColor, pixcmapGetRangeValues

修正ファイル: `crates/leptonica-core/src/colormap/query.rs`（新規）

### 12.2 カラーマップ変換・効果 (`feat/core-cmap-convert`)

対象（colormap.c）:
- pixcmapGrayToFalseColor, pixcmapGrayToColor, pixcmapColorToGray
- pixcmapConvertTo4, pixcmapConvertTo8
- pixcmapToArrays, pixcmapToRGBTable, pixcmapConvertToHex
- pixcmapGammaTRC, pixcmapContrastTRC
- pixcmapShiftIntensity, pixcmapShiftByComponent

修正ファイル: `crates/leptonica-core/src/colormap/convert.rs`（新規）

---

## Phase 13: 深度変換（~25関数, 2 PR）

### 13.1 低ビット深度変換 (`feat/core-conv-low`) ✅ 完了 (PR #93)

対象（pixconv.c）:
- ✅ pixConvert2To8 - 実装済み
- ✅ pixConvert4To8 - 実装済み
- ✅ pixConvertTo2 - 実装済み
- ✅ pixConvert8To2 - 実装済み
- ✅ pixConvertTo4 - 実装済み
- ✅ pixConvert8To4 - 実装済み
- ✅ pixConvertGrayToColormap - 実装済み
- ✅ pixConvertGrayToColormap8 - 実装済み
- ⏭️ pixThreshold8 - 後続（filter crateの`pixBackgroundNormSimple`依存）
- ⏭️ pixConvertTo8BySampling - 後続（transform crateの`pixScaleBySampling`依存）
- ⏭️ pixConvertTo8Colormap - 32bpp部分は後続
- ⏭️ pixConvertTo1Adaptive - 後続（filter crateの`pixBackgroundNormSimple`依存）
- ⏭️ pixConvertTo1BySampling - 後続（transform crateの`pixScaleBySampling`依存）

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`

実装内容: 8関数 + 35テスト

### 13.2 高ビット・特殊変換 (`feat/core-conv-high`)

対象（pixconv.c）:
- pixConvertTo32, pixConvertTo32BySampling
- pixConvert24To32, pixConvert32To24, pixConvert32To16
- pixAddAlphaTo1bpp
- pixColorizeGray, pixConvertGrayToFalseColor
- pixConvertRGBToGrayArb, pixConvertRGBToBinaryArb
- pixConvertRGBToColormap, pixConvertCmapTo1
- pixQuantizeIfFewColors, pixConvertForPSWrap
- pixConvertToSubpixelRGB, pixConvertGrayToSubpixelRGB, pixConvertColorToSubpixelRGB

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`

---

## Phase 14: Box 操作（~80関数, 4 PR）

### 14.1 Box 幾何・関係演算 (`feat/core-box-geometry`)

対象（boxfunc1.c）:
- boxaContainedInBox, boxaContainedInBoxCount, boxaContainedInBoxa
- boxaIntersectsBox, boxaIntersectsBoxCount
- boxaClipToBox, boxaCombineOverlaps, boxaCombineOverlapsInPair
- boxOverlapFraction, boxOverlapArea, boxaHandleOverlaps
- boxOverlapDistance, boxSeparationDistance, boxCompareSize
- boxaGetNearestToPt, boxaGetNearestToLine
- boxaFindNearestBoxes, boxaGetNearestByDirection
- boxGetCenter, boxIntersectByLine
- boxClipToRectangle, boxClipToRectangleParams

修正ファイル: `crates/leptonica-core/src/box_/geometry.rs`（新規）

### 14.2 Box 調整・変換 (`feat/core-box-adjust`)

対象（boxfunc1.c, boxfunc4.c）:
- boxRelocateOneSide, boxaAdjustSides, boxaAdjustBoxSides, boxAdjustSides
- boxaSetSide, boxSetSide, boxaAdjustWidthToTarget, boxaAdjustHeightToTarget
- boxEqual, boxaEqual, boxSimilar, boxaSimilar
- boxaJoin, boxaaJoin, boxaSplitEvenOdd, boxaMergeEvenOdd
- boxaConvertToPta, ptaConvertToBoxa, boxConvertToPta, ptaConvertToBox

修正ファイル: `crates/leptonica-core/src/box_/adjust.rs`（新規）

### 14.3 Box 選択・統計 (`feat/core-box-select`)

対象（boxfunc4.c, boxfunc5.c）:
- boxaSelectRange, boxaaSelectRange
- boxaSelectBySize, boxaMakeSizeIndicator
- boxaSelectByArea, boxaMakeAreaIndicator
- boxaSelectByWHRatio, boxaMakeWHRatioIndicator
- boxaSelectWithIndicator
- boxaPermutePseudorandom, boxaPermuteRandom, boxaSwapBoxes
- boxaGetExtent, boxaGetCoverage
- boxaaSizeRange, boxaSizeRange, boxaLocationRange
- boxaGetSizes, boxaGetArea
- boxfunc5.c スムージング関数群

修正ファイル: `crates/leptonica-core/src/box_/select.rs`（新規）

### 14.4 Box 描画・マスク (`feat/core-box-draw`)

対象（boxfunc3.c）:
- pixMaskConnComp, pixMaskBoxa, pixPaintBoxa, pixSetBlackOrWhiteBoxa
- pixPaintBoxaRandom, pixBlendBoxaRandom
- pixDrawBoxa, pixDrawBoxaRandom
- boxaaDisplay, pixaDisplayBoxaa
- pixSplitIntoBoxa, pixSplitComponentIntoBoxa, makeMosaicStrips
- boxaCompareRegions, pixSelectLargeULComp, boxaSelectLargeULBox
- boxaDisplayTiled

修正ファイル: `crates/leptonica-core/src/box_/draw.rs`（新規）

---

## Phase 15: Pix マスク・統計・クリッピング（~75関数, 4 PR）

### 15.1 マスク拡張 (`feat/core-pix-mask-ext`)

対象（pix3.c）:
- pixSetMaskedGeneral, pixCombineMaskedGeneral
- pixCopyWithBoxa, pixPaintSelfThroughMask
- pixMakeArbMaskFromRGB, pixSetUnderTransparency
- pixMakeAlphaFromMask, pixGetColorNearMaskBoundary
- pixDisplaySelectedPixels
- pixaCountPixels, pixCountPixels
- pixCountPixelsByRow, pixCountPixelsByColumn, pixCountPixelsInRow
- pixGetMomentByColumn

修正ファイル: `crates/leptonica-core/src/pix/mask.rs`

### 15.2 行列統計・差分 (`feat/core-pix-rowcol-stats`)

対象（pix3.c, pix4.c）:
- pixAverageByRow, pixAverageByColumn, pixAverageInRect, pixAverageInRectRGB
- pixVarianceByRow, pixVarianceByColumn, pixVarianceInRect
- pixAbsDiffByRow, pixAbsDiffByColumn, pixAbsDiffInRect, pixAbsDiffOnLine
- pixCountArbInRect
- pixRowStats, pixColumnStats, pixGetRowStats, pixGetColumnStats
- pixSetPixelColumn
- pixMirroredTiling, pixFindRepCloseTile

修正ファイル: `crates/leptonica-core/src/pix/statistics.rs`

### 15.3 ヒストグラム拡張 (`feat/core-pix-hist-ext`)

対象（pix4.c）:
- pixGetGrayHistogramTiled
- pixGetCmapHistogram, pixGetCmapHistogramMasked, pixGetCmapHistogramInRect
- pixCountRGBColorsByHash, pixCountRGBColors, pixGetColorAmapHistogram
- pixGetRankValueMaskedRGB, pixGetRankValueMasked
- pixGetPixelAverage, pixGetPixelStats
- pixGetAverageMaskedRGB, pixGetAverageMasked
- pixGetAverageTiledRGB, pixGetAverageTiled
- pixGetMaxColorIndex, pixGetBinnedComponentRange, pixGetRankColorArray
- pixGetBinnedColor, pixDisplayColorArray, pixRankBinByStrip
- pixaGetAlignedStats, pixaExtractColumnFromEachPix
- pixSplitDistributionFgBg

修正ファイル: `crates/leptonica-core/src/pix/histogram.rs`

### 15.4 クリッピング・測定 (`feat/core-pix-clip-ext`)

対象（pix5.c）:
- pixClipRectangle, pixClipRectangleWithBorder, pixClipRectangles
- pixCropToMatch, pixCropToSize, pixResizeToMatch
- pixClipToForeground, pixTestClipToForeground, pixClipBoxToForeground
- pixScanForForeground
- pixMakeFrameMask, pixMakeCoveringOfRectangles, pixFractionFgInMask
- pixExtractOnLine, pixAverageOnLine
- pixAverageIntensityProfile, pixReversalProfile, pixWindowedVarianceOnLine
- pixMinMaxNearLine, pixRankRowTransform, pixRankColumnTransform
- pixSelectComponentBySize, pixFilterComponentBySize
- pixaFindDimensions, pixFindAreaPerimRatio, etc. (pix5.c measurement群)

修正ファイル: `crates/leptonica-core/src/pix/clip.rs`, `extract.rs`

---

## Phase 16: Numa/Pta/Pixa 拡張（~100関数, 5 PR）

### 16.1 Numa 算術・変換 (`feat/core-numa-arith`)

対象（numafunc1.c）:
- numaArithOp, numaLogicalOp, numaInvert, numaSimilar, numaAddToNumber
- numaGetPartialSums, numaMakeDelta, numaMakeSequence, numaMakeAbsval
- numaAddBorder, numaAddSpecifiedBorder, numaRemoveBorder
- numaCountNonzeroRuns, numaSubsample
- numaJoin, numaaJoin

修正ファイル: `crates/leptonica-core/src/numa/operations.rs`

### 16.2 Numa ソート・補間 (`feat/core-numa-sort`)

対象（numafunc1.c）:
- numaSortGeneral, numaSortAutoSelect, numaSortIndexAutoSelect
- numaChooseSortType, numaBinSort, numaGetSortIndex, numaGetBinSortIndex
- numaSortByIndex, numaIsSorted, numaSortPair
- numaInvertMap, numaAddSorted, numaFindSortedLoc
- numaPseudorandomSequence, numaRandomPermutation
- numaGetBinnedMedian, numaGetMeanDevFromMedian, numaGetMedianDevFromMedian
- numaInterpolateEqxVal, numaInterpolateArbxVal
- numaInterpolateEqxInterval, numaInterpolateArbxInterval
- numaFitMax, numaDifferentiateInterval, numaIntegrateInterval
- numaGetNonzeroRange, numaGetCountRelativeToZero
- numaClipToInterval, numaMakeThresholdIndicator
- numaUniformSampling, numaLowPassIntervals, numaThresholdEdges
- numaGetSpanValues, numaGetEdgeValues

修正ファイル: `crates/leptonica-core/src/numa/sort.rs`（新規）, `interpolation.rs`（新規）

### 16.3 Pta/Ptaa 基本・変換 (`feat/core-pta-ext`)

対象（ptabasic.c, ptafunc1.c, ptafunc2.c）:
- ptaCreateFromNuma, ptaCopyRange, ptaEmpty, ptaInsertPt, ptaRemovePt
- ptaGetIPt, ptaGetArrays
- Ptaa型の全実装（ptaaCreate, ptaaDestroy, ptaaAddPta, ptaaGetCount, ptaaGetPta, ptaaGetPt, ptaaInitFull, ptaaReplacePta, ptaaAddPt, ptaaTruncate）
- ptafunc1: ポイント配列変換、回転、スケール、幾何演算
- ptafunc2: 最小二乗法、ソート、統計

修正ファイル:
- `crates/leptonica-core/src/pta/mod.rs`（拡張）
- `crates/leptonica-core/src/pta/transform.rs`（新規）

### 16.4 Pixa 基本拡張 (`feat/core-pixa-basic`)

対象（pixabasic.c）:
- pixaCreateFromPix, pixaCreateFromBoxa, pixaSplitPix
- pixaGetBoxa, pixaGetBoxaCount, pixaGetBox, pixaGetBoxGeometry, pixaSetBoxa
- pixaGetPixArray, pixaVerifyDepth, pixaVerifyDimensions, pixaIsFull
- pixaCountText, pixaSetText, pixaGetLinePtrs, pixaWriteStreamInfo
- pixaReplacePix, pixaInsertPix, pixaRemovePix, pixaRemovePixAndSave, pixaRemoveSelected
- pixaInitFull, pixaJoin, pixaInterleave
- Pixaa型の全実装

修正ファイル: `crates/leptonica-core/src/pixa/mod.rs`（拡張）

### 16.5 Pixa/Sarray 高度操作 (`feat/core-pixa-advanced`)

対象（pixafunc1.c, pixafunc2.c, sarray2.c）:
- pixaSelectBySize, pixaSelectByArea, pixaSort, pixaSortByIndex
- pixaScaleToSize, pixaScaleToSizeRel
- pixaDisplay, pixaDisplayTiled, pixaDisplayTiledAndScaled
- Sarray 残り: sarrayRemoveString, sarrayReplaceString, sarrayGetArray
- sarrayToStringRange, sarrayConcatUniformly, sarrayJoin, sarrayAppendRange
- sarrayPadToSameSize, sarrayConvertWordsToLines, sarraySplitString
- sarraySelectRange, sarrayParseRange, sarraySortByIndex, sarrayAppend

修正ファイル:
- `crates/leptonica-core/src/pixa/display.rs`（新規）
- `crates/leptonica-core/src/sarray/operations.rs`（新規）

---

## Phase 17: Graphics/Compare/Blend（~55関数, 3 PR）

### 17.1 PTA生成関数 (`feat/core-graphics-pta`)

対象（graphics.c）:
- generatePtaLine, generatePtaWideLine
- generatePtaBox, generatePtaBoxa, generatePtaHashBox, generatePtaHashBoxa
- generatePtaaBoxa, generatePtaaHashBoxa
- generatePtaPolyline, generatePtaGrid
- convertPtaLineTo4cc
- generatePtaFilledCircle, generatePtaFilledSquare
- pixGeneratePtaBoundary

修正ファイル: `crates/leptonica-core/src/pix/graphics.rs`

### 17.2 レンダリング拡張 (`feat/core-graphics-render`)

対象（graphics.c）:
- pixRenderPtaArb, pixRenderPtaBlend
- pixRenderLineArb, pixRenderLineBlend
- pixRenderBoxArb, pixRenderBoxBlend
- pixRenderBoxa, pixRenderBoxaArb, pixRenderBoxaBlend
- pixRenderHashBox, pixRenderHashBoxArb, pixRenderHashBoxBlend
- pixRenderHashMaskArb
- pixRenderHashBoxa, pixRenderHashBoxaArb, pixRenderHashBoxaBlend
- pixRenderPolyline, pixRenderPolylineArb, pixRenderPolylineBlend
- pixRenderGridArb, pixRenderRandomCmapPtaa
- pixRenderPolygon, pixFillPolygon
- pixRenderContours, fpixAutoRenderContours, fpixRenderContours
- pixRenderPlotFromNuma, pixRenderPlotFromNumaGen

修正ファイル: `crates/leptonica-core/src/pix/graphics.rs`

### 17.3 Compare/Blend 拡張 (`feat/core-compare-blend`)

対象（compare.c, blend.c）:
- pixEqualWithAlpha, pixEqualWithCmap
- pixDisplayDiff, pixDisplayDiffBinary
- pixCompareGrayOrRGB, pixCompareGray, pixCompareRGB
- pixCompareTiled, pixCompareRankDifference
- pixTestForSimilarity, pixGetDifferenceStats, pixGetDifferenceHistogram
- pixGetPerceptualDiff, pixGetPSNR
- pixBlendGrayInverse, pixBlendColorByChannel, pixBlendGrayAdapt
- pixFadeWithGray, pixBlendHardLight, pixBlendCmap
- pixBlendBackgroundToColor, pixMultiplyByColor
- pixAlphaBlendUniform, pixAddAlphaToBlend, pixSetAlphaOverWhite
- pixLinearEdgeFade

修正ファイル: `crates/leptonica-core/src/pix/compare.rs`, `blend.rs`

---

## FPix/DPix 拡張（Phase 16 に含む）

以下の関数は各Phaseの適切な場所で実装:
- FPixa型: Phase 16.4 (Pixa基本拡張と同時)
- fpixSetDimensions, fpixCopyResolution: Phase 11.1
- fpixConvolveSep, fpixConvolve: leptonica-filter の責務（スコープ外）

修正ファイル: `crates/leptonica-core/src/fpix/mod.rs`

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 10 | シリアライゼーション基盤 | 5 | ~90 |
| 11 | Pix ユーティリティ | 3 | ~45 |
| 12 | カラーマップ操作 | 2 | ~35 |
| 13 | 深度変換 | 2 | ~25 |
| 14 | Box 操作 | 4 | ~80 |
| 15 | Pix マスク・統計・クリッピング | 4 | ~75 |
| 16 | Numa/Pta/Pixa 拡張 | 5 | ~100 |
| 17 | Graphics/Compare/Blend | 3 | ~55 |
| **合計** | | **28** | **~505** |

完了後の推定カバレッジ: (226+505) / 845 ≈ **86.5%**
（残りはroplow.c スキップ分 + Rust設計上N/Aの関数）

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-core
cargo clippy -p leptonica-core -- -D warnings
cargo test -p leptonica-core
cargo test --workspace  # PR前に全ワークスペーステスト
```

シリアライゼーション（Phase 10）はラウンドトリップテストを重点的に:
```rust
// write → read → compare のパターン
let original = Boxa::from(vec![Box::new(10, 20, 30, 40)]);
let mut buf = Vec::new();
original.write_to_writer(&mut buf)?;
let restored = Boxa::read_from_bytes(&buf)?;
assert_eq!(original, restored);
```
