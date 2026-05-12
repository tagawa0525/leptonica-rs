# Gap-Fill 第2弾ロードマップ (151 関数)

Status: PLANNED
作成日: 2026-05-10

## Context

`docs/porting/comparison/` のフルオーディット (gap-fill audit 2026-05-10) で
`allheaders.h` の C 公開関数 2,743 件を全数集計した結果、**151 関数**が
真の未実装 (❌) として残った。本計画はこれら 151 関数を実装するための
カテゴリ別ロードマップである。

なお plan 031 で扱った 12 項目 (HOLD 2 / IMPLEMENTED 8 / FIXED 1 / PARTIAL 1)
はすべて完了済みで、本計画はその続編である。Rust 標準ライブラリで代替可能な
ファイル/パス/文字列ヘルパー、デバッグ/ロギング、低レベル内部関数等は既に
🚫 に再分類済み (684 件) で、本計画には含まれない。

## 全体像

| カテゴリ                                  | C ファイル                               | 件数 | 優先度 | 難易度 |
| ----------------------------------------- | ---------------------------------------- | ---- | ------ | ------ |
| A. Pixa 拡張 (selection/transform)        | pixafunc1.c, pixafunc2.c                 | 50   | 中     | M      |
| B. Numa 高度関数 (histogram/distribution) | numafunc2.c                              | 23   | 中     | M-L    |
| C. 文書解析 (page segmentation)           | pageseg.c                                | 10   | 高     | L      |
| D. FPix 拡張 (geometry transforms)        | fpix2.c                                  | 12   | 中     | M      |
| E. Pta + graphics (point arrays)          | ptafunc1.c                               | 15   | 低     | M      |
| F. 画像比較 (compare/histo)               | compare.c                                | 11   | 高     | L      |
| G. カーネル生成 (Gaussian/DoG)            | kernel.c                                 | 5    | 高     | S      |
| H. arrayaccess ビット演算拡張             | arrayaccess.c                            | 4    | 低     | S      |
| I. RGB スケーリング/演算                  | pixarith.c                               | 5    | 中     | S      |
| J. グラフィックスヘルパー                 | graphics.c                               | 3    | 低     | S      |
| K. Skew 補助                              | skew.c                                   | 2    | 低     | S      |
| L. Numa 基本 (parse/convert)              | numabasic.c                              | 5    | 低     | S      |
| M. その他単独関数                         | bmf, pdfapp, runlength, sarray2, textops | 6    | 低     | M      |

合計: **151 関数**

優先度の判断基準:

- **高**: 文書画像処理コア機能、Tesseract 連携で利用される
- **中**: 既存 Rust 機能の拡張、利用シーンが想定される
- **低**: 補助関数、ニッチ用途、現状の実カバレッジ 93% でカバー範囲外

難易度の判断基準:

- **S** (Small): 数十行、独立性高い、テストデータ要らない
- **M** (Medium): 100-300 行、既存型に依存、テスト画像が必要
- **L** (Large): 300+ 行、新型/新モジュール導入、複雑なアルゴリズム

## カテゴリ別詳細

### A. Pixa 拡張 (50 件)

#### A-1. Pixa 選択系 (pixafunc1.c, 12 件)

`pixSelectBy*` (Pix 単位) と `pixaSelectBy*` (Pixa 単位) のセット。
既存の `Pixa::select_by_size`, `select_by_area` の派生。

- pixSelectByAreaFraction / pixaSelectByAreaFraction
- pixSelectByPerimSizeRatio / pixaSelectByPerimSizeRatio
- pixSelectByPerimToAreaRatio / pixaSelectByPerimToAreaRatio
- pixSelectByWidthHeightRatio / pixaSelectByWidthHeightRatio
- pixaSelectByNumConnComp
- pixaSelectRange
- pixaSelectWithIndicator
- pixaSelectWithString
- pixAddWithIndicator
- pixRemoveWithIndicator

サブ計画候補: `docs/plans/106_core-pixa-selection.md`
依存: 既存 Pixa 型、Numa (indicator)

#### A-2. Pixa 変換系 (pixafunc1.c + pixafunc2.c, 14 件)

- pixaConvertToGivenDepth
- pixaConvertToSameDepth
- pixaConvertTo1
- pixaConvertTo32
- pixaConvertTo8
- pixaConvertTo8Colormap
- pixaConvertToNUpPixa
- pixaScale
- pixaScaleBySampling
- pixaRotate
- pixaRotateOrth
- pixaTranslate
- pixaAddBorderGeneral
- pixaClipToForeground

サブ計画候補: `docs/plans/107_core-pixa-transform.md`
依存: Pix の対応関数 (既存)

#### A-3. Pixa プロパティ/その他 (24 件)

- pixaAnyColormaps / pixaHasColor / pixaEqual
- pixaGetDepthInfo / pixaGetRenderingDepth / pixaSizeRange
- pixaMakeSizeIndicator / pixaSort2dByIndex
- pixaBinSort / pixaSetFullSizeBoxa
- pixaClipToPix / pixaRenderComponent
- pixaConstrainedSelect / pixaSplitIntoFiles
- pixaSelectToPdf (PDF 依存) / pixaMakeFromTiledPix(a)
- pixaaFlattenToPixa / pixaaScaleToSizeVar / pixaaSelectRange / pixaaSizeRange
- pixGetTileCount

サブ計画候補: `docs/plans/108_core-pixa-properties.md`
依存: Pixaa, Boxa

### B. Numa 高度関数 (23 件)

ヒストグラム解析・統計・分布処理の高度な機能群。

- numaCrossingsByThreshold / numaCrossingsByPeaks
- numaCountReversals / numaFindPeaks
- numaGetHistogramStats / numaGetHistogramStatsOnInterval
- numaGetStatsUsingHistogram / numaMakeHistogramAuto
- numaRebinHistogram / numaMakeRankFromHistogram
- numaHistogramGetRankFromVal / numaHistogramGetValFromRank
- numaDiscretizeHistoInBins / numaDiscretizeSortedInBins
- numaSplitDistribution / numaGetUniformBinSizes
- numaGetRankBinValues
- numaEarthMoverDistance / grayHistogramsToEMD
- numaEvalBestHaarParameters / numaEvalHaarSum
- grayInterHistogramStats
- genConstrainedNumaInRange

サブ計画候補: `docs/plans/109_core-numa-histogram-advanced.md`
依存: 既存 Numa 型, Pix ヒストグラム機能

### C. 文書解析 (10 件) — **高優先度**

ドキュメント画像処理の重要機能。

- pixCleanImage: 高レベル文書クリーンアップ
- pixCountTextColumns: テキスト列数推定
- pixDecideIfText: テキスト領域判定
- pixEstimateBackground: 背景推定
- pixExtractRawTextlines: 生テキスト行抽出
- pixFindThreshFgExtent: 前景閾値範囲検出
- pixGenHalftoneMask / pixGenTextblockMask / pixGenTextlineMask: マスク生成
- pixCropImage: 高レベル画像クロップ

サブ計画候補: `docs/plans/803_recog-document-analysis.md`
依存: 既存の pageseg, threshold, conncomp

### D. FPix 拡張 (12 件)

FPix の幾何変換と統計機能。

- fpixGetMax / fpixGetMin
- fpixThresholdToPix
- fpixRasterop
- fpixScaleByInteger
- fpixRemoveBorder
- fpixAffine / fpixAffinePta
- fpixProjective / fpixProjectivePta
- linearInterpolatePixelFloat
- pixComponentFunction

サブ計画候補: `docs/plans/110_core-fpix-extended.md`
依存: 既存 FPix, アフィン/射影変換 (Pix 用)

### E. Pta + graphics (15 件)

Pta から画像へのレンダリング、Pta-Numa 変換、最小二乗フィット。

- pixGenerateFromPta / pixPlotAlongPta / pixFindCornerPixels
- ptaGetPixelsFromPix / ptaGetBoundaryPixels / ptaGetBoundingRegion
- ptaGetNeighborPixLocs
- ptaConvertToNuma / numaConvertToPta1 / numaConvertToPta2
- ptaNoisyLinearLSF / ptaNoisyQuadraticLSF
- ptaReplicatePattern
- ptaaGetBoundaryPixels / ptaaIndexLabeledPixels

サブ計画候補: `docs/plans/111_core-pta-graphics.md`
依存: 既存 Pta, Pix 描画

### F. 画像比較 (11 件) — **高優先度**

画像比較・差分検出の重要機能。

- compareTilesByHisto: タイル単位ヒストグラム比較
- pixCompareGrayByHisto: グレー画像のヒストグラム比較
- pixComparePhotoRegionsByHisto / pixaComparePhotoRegionsByHisto
- pixGenPhotoHistos: 写真領域ヒストグラム生成
- pixDecideIfPhotoImage: 写真画像判定
- pixCentroid8: 8近傍重心
- pixCropAlignedToCentroid / pixPadToCenterCentroid
- pixUsesCmapColor: カラーマップ色使用判定
- cmapEqual: カラーマップ同等性

サブ計画候補: `docs/plans/112_core-compare-photo.md`
依存: 既存 Numa, Pix 統計, Colormap

### G. カーネル生成 (5 件) — **高優先度・最小**

- makeFlatKernel: 平坦カーネル
- makeGaussianKernel: ガウシアン
- makeGaussianKernelSep: 分離可能ガウシアン
- makeDoGKernel: DoG (Difference of Gaussians)
- parseStringForNumbers: 文字列から数値配列パース

サブ計画候補: `docs/plans/501_filter-kernel-generators.md`
依存: 既存 Kernel 型
**最小コスト・最大価値、最初に実装する候補**

### H. arrayaccess ビット演算 (4 件)

- l_clearDataDibit (2bit 単位クリア)
- l_clearDataQbit (4bit 単位クリア)
- l_getDataFourBytes (4byte 読み)
- l_setDataFourBytes (4byte 書き)

サブ計画候補: `docs/plans/113_core-arrayaccess-extended.md`
依存: 既存 access.rs
**実装は単純、テストも容易**

### I. RGB スケーリング/演算 (5 件)

- linearScaleRGBVal / logScaleRGBVal
- pixAddRGB
- pixMaxDynamicRangeRGB
- pixThresholdToValue

サブ計画候補: `docs/plans/114_core-rgb-arithmetic.md`
依存: 既存 RGB 演算

### J. グラフィックスヘルパー (3 件)

- generatePtaLineFromPt
- locatePtRadially
- makePlotPtaFromNuma

サブ計画候補: `docs/plans/115_core-graphics-helpers.md`
依存: 既存 Pta, graphics

### K. Skew 補助 (2 件)

- pixFindDifferentialSquareSum
- pixFindNormalizedSquareSum

サブ計画候補: `docs/plans/803_recog-skew-helpers.md` (再利用)
依存: 既存 skew.rs
**deskew のオプションとして公開検討**

### L. Numa 基本 (5 件)

- numaConvertToSarray
- numaCopyParameters
- numaCreateFromString
- numaaCreateFull
- numaaGetNumberCount

サブ計画候補: `docs/plans/116_core-numa-basic-extended.md`
依存: 既存 Numa, Numaa, Sarray

### M. 単独関数 (6 件)

- bmf.c: pixaSaveFont (BMF フォント保存)
- pdfapp.c: rotateorthFilesToPdf (回転 PDF 生成)
- runlength.c: pixStrokeWidthTransform (ストローク幅変換)
- sarray2.c: stringCompareLexical
- textops.c: pixAddSingleTextblock / splitStringToParagraphs

優先度低、必要時に個別実装。

## 推奨実装順序

1. **G. カーネル生成** (5 件) — 最小コスト、Tesseract 連携で利用見込み
2. **H. arrayaccess ビット演算** (4 件) — 単純、補完性高い
3. **F. 画像比較** (11 件) — 文書比較で需要、既存 compare.rs に追加
4. **C. 文書解析** (10 件) — 高価値、recog/ への追加
5. **A-1. Pixa 選択系** (12 件) — Pixa の Rust 化を強化
6. **K. Skew 補助** (2 件) — 既存 skew.rs に追加
7. **D. FPix 拡張** (12 件) — FPix の機能補完
8. **B. Numa 高度関数** (23 件) — 統計機能群
9. **A-2. Pixa 変換系** (14 件) — Pixa 変換機能拡充
10. **I. RGB スケーリング** (5 件)
11. **A-3. Pixa プロパティ** (24 件) — 残り
12. **E. Pta + graphics** (15 件)
13. **L. Numa 基本** (5 件)
14. **J. グラフィックスヘルパー** (3 件)
15. **M. 単独関数** (6 件)

## 完了基準

各サブ計画ごとに:

- [ ] RED コミット (テスト追加、`#[ignore = "not yet implemented"]` 付き)
- [ ] GREEN コミット (実装、`#[ignore]` 除去)
- [ ] cargo test/clippy/fmt 通過
- [ ] PR + Copilot レビュー対応
- [ ] マージ後 docs/porting/comparison/*.md の該当エントリを ❌ → ✅ に更新
- [ ] feature-comparison.md の数値再計算

## ステータス管理

| サブ計画 | カテゴリ                       | 件数 | Status      | PR  |
| -------- | ------------------------------ | ---- | ----------- | --- |
| 501      | G. カーネル生成                | 5    | IMPLEMENTED | TBD |
| 113      | H. arrayaccess                 | 4    | IMPLEMENTED | TBD |
| 112      | F. 画像比較 (補助 5)           | 5    | IMPLEMENTED | TBD |
| 117      | F. 画像比較 (残り 6)           | 6    | PLANNED     | -   |
| 803-K    | K. Skew 補助                   | 2    | IMPLEMENTED | TBD |
| 803      | C. 文書解析 (補助 4)           | 4    | IMPLEMENTED | TBD |
| 803b     | C. 文書解析 (残り 6)           | 6    | PLANNED     | -   |
| 106      | A-1. Pixa 選択                 | 14   | IMPLEMENTED | TBD |
| 110      | D. FPix 拡張 (補助 7)          | 7    | IMPLEMENTED | TBD |
| 110b     | D. FPix 拡張 (残り 5)          | 5    | PLANNED     | -   |
| 109      | B. Numa 高度 (補助 5)          | 5    | IMPLEMENTED | TBD |
| 109b     | B. Numa 高度 (残り 18)         | 18   | PLANNED     | -   |
| 107      | A-2. Pixa 変換 (補助 7)        | 7    | IMPLEMENTED | TBD |
| 107b     | A-2. Pixa 変換 (残り 7)        | 7    | PLANNED     | -   |
| 114      | I. RGB スケーリング            | 5    | IMPLEMENTED | TBD |
| 108      | A-3. Pixa プロパティ (補助 8)  | 8    | IMPLEMENTED | TBD |
| 108b     | A-3. Pixa プロパティ (残り 16) | 16   | PLANNED     | -   |
| 111      | E. Pta + graphics (補助 7)     | 7    | IMPLEMENTED | TBD |
| 111b     | E. Pta + graphics (残り 8)     | 8    | PLANNED     | -   |
| 116      | L. Numa 基本                   | 5    | IMPLEMENTED | TBD |
| 115      | J. graphics ヘルパー           | 3    | IMPLEMENTED | TBD |
| (個別)   | M. 単独                        | 6    | PLANNED     | -   |

合計 **151 関数 / 14 サブ計画**。

## 注意事項

- 優先度「低」のカテゴリは、実利用ニーズが発生してから実装する判断もあり
- C 版の API シグネチャをそのまま移植せず、Rust idiomatic な API に再設計することを推奨 (Pix→Pix の owned 戻り値、`Result` でエラー伝播、等)
- 各サブ計画の着手前に reference/leptonica/ で C 版実装を確認し、外部依存 (libc, libtiff 等) が無いことを確認する

## 関連ドキュメント

- 元の調査: `docs/porting/comparison/*.md` の「追加検証エントリ」セクション
- 旧 gap-fill (12 項目): `docs/plans/031_gap-fill-overall.md`
- ✅/🔄 サマリ: `docs/porting/feature-comparison.md`
