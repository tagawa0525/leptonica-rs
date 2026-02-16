# comparison文書更新 + Phase 5-9 実装計画

Status: PLANNED

## Context

`docs/plans/humming-tickling-journal.md` の Phase 1-4 実装が完了したが、`docs/rebuild/comparison/*.md` の状態ステータスが実装前のまま残っている。個別ファイルのサマリーとdetail、`feature-comparison.md` の数値がそれぞれ不整合。
また、Phase 5以降の計画が未策定のため、C版リファレンスの調査結果に基づいて Phase 5-9 を計画する。

---

## Part 1: comparison文書の更新

### 1.1 filter.md の更新

**対象ファイル**: `docs/rebuild/comparison/filter.md`

#### convolve.c セクション（15関数: ❌ → ✅）

| C関数 | 変更 | Rust対応 |
|--------|------|----------|
| pixBlockconv | ❌→✅ | block_conv.rs blockconv() |
| pixBlockconvGray | ❌→✅ | block_conv.rs blockconv_gray() |
| pixBlockconvAccum | ❌→✅ | block_conv.rs blockconv_accum() |
| pixBlockconvGrayUnnormalized | ❌→✅ | block_conv.rs blockconv_gray_unnormalized() |
| pixWindowedStats | ❌→✅ | windowed.rs windowed_stats() |
| pixWindowedMean | ❌→✅ | windowed.rs windowed_mean() |
| pixWindowedMeanSquare | ❌→✅ | windowed.rs windowed_mean_square() |
| pixWindowedVariance | ❌→✅ | windowed.rs windowed_variance() |
| pixMeanSquareAccum | ❌→✅ | windowed.rs mean_square_accum() |
| pixBlockrank | ❌→✅ | convolve.rs blockrank() |
| pixBlocksum | ❌→✅ | convolve.rs blocksum() |
| pixCensusTransform | ❌→✅ | convolve.rs census_transform() |
| pixConvolveSep | ❌→✅ | convolve.rs convolve_sep() |
| pixConvolveRGBSep | ❌→✅ | convolve.rs convolve_rgb_sep() |
| pixAddGaussianNoise | ❌→✅ | convolve.rs add_gaussian_noise() |

#### enhance.c セクション（2関数: ❌ → ✅）

| C関数 | 変更 | Rust対応 |
|--------|------|----------|
| pixUnsharpMaskingFast | ❌→✅ | edge.rs unsharp_masking_fast() |
| pixUnsharpMaskingGrayFast | ❌→✅ | edge.rs unsharp_masking_gray_fast() |

#### サマリー再計算

detailテーブルの全行を数え直してサマリーを更新する。
現在のdetail: ✅ 33, ❌ 66 → 更新後: ✅ 50, ❌ 49

#### 実装状況分析セクションの更新

- 「主要な未実装機能 > 高優先度」からブロック畳み込み・分離可能畳み込み・ウィンドウ統計を削除
- 「今後の実装推奨順序」を更新（adaptmap.c拡張、高速bilateral、残りenhance関数）

### 1.2 core.md の更新

**対象ファイル**: `docs/rebuild/comparison/core.md`

detailテーブルは Phase 1 実装時に更新済み。サマリーのみ未更新。

- サマリー: `82, 24, 742` → detailを数え直して正確な値に更新
- `feature-comparison.md` の値（134, 24, 690）と整合させる

### 1.3 morph.md の更新

**対象ファイル**: `docs/rebuild/comparison/morph.md`

PR #51（3x3 grayscale fast path）の実装を反映。

| C関数 | 変更 | Rust対応 |
|--------|------|----------|
| pixErodeGray3 | ❌→🔄 | erode_gray() が 3x3 で fast path にディスパッチ |
| pixDilateGray3 | ❌→🔄 | dilate_gray() が 3x3 で fast path にディスパッチ |
| pixOpenGray3 | ❌→🔄 | open_gray() が 3x3 で fast path にディスパッチ |
| pixCloseGray3 | ❌→🔄 | close_gray() が 3x3 で fast path にディスパッチ |

サマリー: `34, 8, 78` → `34, 12, 74`

### 1.4 feature-comparison.md の更新

**対象ファイル**: `docs/rebuild/feature-comparison.md`

各個別ファイルの更新結果を反映:

| クレート | 変更前 | 変更後 |
|----------|--------|--------|
| filter | ✅ 30, ❌ 64, 31.9% | ✅ 50, ❌ 49, 50.5%（※） |
| morph | ✅ 34, 🔄 8, ❌ 78, 35.0% | ✅ 34, 🔄 12, ❌ 74, 38.3% |
| 合計行 | 再計算 | 再計算 |

※ 正確な値はdetailテーブルの全行カウントにより確定する

### コミット・PRワークフロー（Part 1）

Part 1 は1つのPRとして処理する。

**ブランチ**: `docs/comparison-update`

**コミット順序**（直列に実行、1コミット = 1つの論理的変更）:
1. `docs(filter): update comparison for Phase 2-3 implemented functions`
2. `docs(core): recalculate comparison summary to match detail tables`
3. `docs(morph): update comparison for 3x3 grayscale fast paths`
4. `docs: sync feature-comparison.md with updated individual files`

**PRワークフロー**:
1. 全コミット完了後、`/gh-pr-create` でPR作成
2. GitHub Copilotの自動レビューを**必ず待つ**（`/gh-actions-check` で到着確認）
3. `/gh-pr-review` でレビューコメントを確認し、指摘事項を全て修正
4. 修正コミットを積んだ後、再度CIが通ることを確認
5. `/gh-pr-merge` でマージ（`--merge`オプション）
6. マージ後のブランチを削除

---

## Part 2: Phase 5-9 実装計画

Phase 1-4 で約111関数を追加。Phase 5-9 で約195関数を追加し、合計カバレッジを約37%に引き上げる。

### PRワークフロー（厳守）

全Phaseに共通。`humming-tickling-journal.md` のワークフローを踏襲する。

**直列実行の原則**:
- 同一worktree内では1つのPRがマージされるまで次のPRの実装を開始しない
- worktreeを分ければ並行着手可（マージ前にrebaseしてgit graphを整える）

**各PRの手順**:
1. PR作成前に全テスト・リントを通す
   ```bash
   cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check
   ```
2. `/gh-pr-create` でPR作成
3. GitHub Copilotの自動レビューを**必ず待つ**（3〜10分かかる。`/gh-actions-check` で到着確認）
4. `/gh-pr-review` でレビューコメントを確認し、指摘事項を修正
5. 指摘対応完了後、再度CIが通ることを確認
6. `/gh-pr-merge` でマージ（`--merge`オプション）
7. マージ後のブランチは速やかに削除

**禁止事項**: レビュー到着前のマージ。レビュー指摘未対応のままのマージ。

### TDDワークフロー（各ファイルブランチ内）

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### ブランチ戦略

```
main
└── feat/<crate>-<feature>          ← クレートブランチ（PRターゲット）
    ├── feat/<crate>-<feature>-<sub>   ← ファイルブランチ（作業単位）
    ├── feat/<crate>-<feature>-<sub2>
    └── feat/<crate>-<feature>-<sub3>
```

- ファイルブランチ → クレートブランチへマージ（ローカル `git merge --no-ff`）
- クレートブランチ → main へPR（Copilotレビュー必須）

### 依存関係グラフ

```
Phase 1-4 (完了)
├──> Phase 5 (filter adaptmap + bilateral)
├──> Phase 6 (core stats/clip/Numa)  ← Phase 5と並行可
│         ├──> Phase 7 (color expansion)
│         └──> Phase 8 (morph/region)  ← Phase 7と並行可
└──> Phase 9 (core Box/Pixa/FPix)    ← Phase 7,8と並行可
```

### Phase 5: leptonica-filter adaptmap.c拡張 + 高速バイラテラル（27関数）

Phase 1-3完了後に着手可能。文書処理パイプラインに必要な背景正規化の完成。

#### 5.1 背景マップ関数 (`feat/filter-adaptmap-bg`)

対象C関数（`adaptmap.c`）:
| 関数 | 内容 |
|------|------|
| pixCleanBackgroundToWhite | 背景白化ラッパー |
| pixBackgroundNormMorph | モルフォロジーベース背景正規化 |
| pixBackgroundNormGrayArray | グレー背景マップ配列抽出 |
| pixBackgroundNormRGBArrays | RGB背景マップ配列抽出 |
| pixBackgroundNormGrayArrayMorph | モルフベースグレー背景マップ |
| pixBackgroundNormRGBArraysMorph | モルフベースRGB背景マップ |
| pixGetBackgroundGrayMap | グレー背景マップ取得 |
| pixGetBackgroundRGBMap | RGB背景マップ取得 |
| pixGetBackgroundGrayMapMorph | モルフベースグレー背景マップ取得 |
| pixGetBackgroundRGBMapMorph | モルフベースRGB背景マップ取得 |

修正ファイル: `crates/leptonica-filter/src/adaptmap.rs`（既存拡張）

#### 5.2 マップユーティリティ (`feat/filter-adaptmap-util`)

| 関数 | 内容 |
|------|------|
| pixFillMapHoles | 背景/前景マップの穴埋め |
| pixExtendByReplication | ボーダー複製による拡張 |
| pixSmoothConnectedRegions | 連結領域の平滑化 |
| pixGetForegroundGrayMap | 前景グレーマップ抽出 |
| pixGetInvBackgroundMap | 逆背景マップ計算 |
| pixApplyInvBackgroundGrayMap | グレー逆背景マップ適用 |
| pixApplyInvBackgroundRGBMap | RGB逆背景マップ適用 |
| pixApplyVariableGrayMap | 可変グレーマップ適用 |

修正ファイル: `crates/leptonica-filter/src/adaptmap.rs`

#### 5.3 高度正規化 (`feat/filter-adaptmap-advanced`)

| 関数 | 内容 |
|------|------|
| pixGlobalNormRGB | グローバルRGB正規化 |
| pixGlobalNormNoSatRGB | 彩度保持グローバルRGB正規化 |
| pixThresholdSpreadNorm | 閾値スプレッド正規化 |
| pixBackgroundNormFlex | フレキシブル背景正規化 |
| pixBackgroundNormTo1MinMax | 背景正規化→1bpp MinMax |
| pixConvertTo8MinMax | 8bpp MinMax変換 |

修正ファイル: `crates/leptonica-filter/src/adaptmap.rs`

#### 5.4 高速バイラテラルフィルタ (`feat/filter-bilateral-fast`)

| 関数 | 内容 |
|------|------|
| pixBilateral | 高速分離可能バイラテラル（auto dispatch） |
| pixBilateralGray | グレースケール高速バイラテラル |
| pixBlockBilateralExact | ブロックベース厳密バイラテラル |

修正ファイル: `crates/leptonica-filter/src/bilateral.rs`（既存拡張）

---

### Phase 6: leptonica-core pix4/pix5 + Numa拡張（52関数）

他クレートが依存する統計・クリッピング・ソート関数の充実。

#### 6.1 統計関数拡張 (`feat/core-stats-advanced`)

対象C関数（`pix3.c`, `pix4.c`）:
| 関数 | 内容 |
|------|------|
| pixAverageByRow | 行ごとの平均値 |
| pixAverageByColumn | 列ごとの平均値 |
| pixAverageInRect | 矩形内平均 |
| pixVarianceByRow | 行ごとの分散 |
| pixVarianceByColumn | 列ごとの分散 |
| pixVarianceInRect | 矩形内分散 |
| pixAbsDiffByRow | 行ごとの絶対差分 |
| pixAbsDiffByColumn | 列ごとの絶対差分 |
| pixAbsDiffInRect | 矩形内絶対差分 |
| pixRowStats | 行ごとの包括統計量 |
| pixColumnStats | 列ごとの包括統計量 |
| pixGetPixelAverage | 平均ピクセル値 |
| pixGetPixelStats | ピクセル統計量（mean/median/mode等） |

修正ファイル: `crates/leptonica-core/src/pix/statistics.rs`

#### 6.2 ヒストグラム拡張 (`feat/core-histogram-advanced`)

対象C関数（`pix4.c`）:
| 関数 | 内容 |
|------|------|
| pixGetGrayHistogramTiled | タイル別グレーヒストグラム |
| pixGetCmapHistogram | カラーマップヒストグラム |
| pixCountRGBColors | RGB色数カウント |
| pixGetAverageMaskedRGB | マスク内RGB平均 |
| pixGetAverageMasked | マスク内平均 |
| pixGetAverageTiledRGB | タイル別RGB平均 |
| pixGetAverageTiled | タイル別平均 |
| pixGetRankValueMaskedRGB | マスク内RGBランク値 |
| pixGetRankValueMasked | マスク内ランク値 |
| pixGetBinnedComponentRange | ビン化コンポーネント範囲 |
| pixGetRankColorArray | ランク順色配列 |
| pixThresholdForFgBg | 前景/背景分離閾値 |
| pixSplitDistributionFgBg | 前景/背景分布分割 |

修正ファイル: `crates/leptonica-core/src/pix/histogram.rs`

#### 6.3 クリッピング・測定 (`feat/core-pix-clip`)

対象C関数（`pix5.c`）:
| 関数 | 内容 |
|------|------|
| pixClipRectangle | 矩形クリッピング |
| pixClipRectangleWithBorder | ボーダー付きクリッピング |
| pixClipMasked | マスクによるクリッピング |
| pixCropToMatch | サイズ一致クロッピング |
| pixClipToForeground | 前景バウンディングボックスへクリップ |
| pixClipBoxToForeground | 前景へボックスクリップ |
| pixScanForForeground | 前景エッジスキャン |
| pixClipBoxToEdges | エッジへボックスクリップ |
| pixScanForEdge | エッジスキャン |
| pixMakeSymmetricMask | 対称マスク生成 |
| pixMakeFrameMask | フレームマスク生成 |
| pixFractionFgInMask | マスク内前景割合 |
| pixExtractOnLine | 線上ピクセル値抽出 |
| pixAverageOnLine | 線上平均値 |

修正ファイル: `crates/leptonica-core/src/pix/clip.rs`（新規）, `extract.rs`（既存拡張）

#### 6.4 Numaソート・補間 (`feat/core-numa-sort`)

対象C関数（`numafunc1.c`）:
| 関数 | 内容 |
|------|------|
| numaSortAutoSelect | 自動ソートアルゴリズム選択 |
| numaSortIndexAutoSelect | インデックスソート自動選択 |
| numaGetSortIndex | ソート順列インデックス取得 |
| numaSortByIndex | インデックス配列でソート |
| numaIsSorted | ソート済み判定 |
| numaInterpolateEqxVal | 等間隔補間 |
| numaInterpolateArbxVal | 任意間隔補間 |
| numaClipToInterval | 区間クリッピング |
| numaMakeThresholdIndicator | 閾値インジケータ生成 |
| numaGetNonzeroRange | 非ゼロ値範囲 |
| numaGetCountRelativeToZero | ゼロ基準カウント |
| numaSubsample | サブサンプリング |

修正ファイル: `crates/leptonica-core/src/numa/operations.rs`

---

### Phase 7: leptonica-color 拡張（37関数）

Phase 6（統計関数）に依存。文書処理・OCR前処理に必須の色分析と閾値処理。

#### 7.1 色内容分析 (`feat/color-content`)

対象C関数（`colorcontent.c`）:
| 関数 | 内容 |
|------|------|
| pixColorContent | 色内容計算（R-G, R-B, G-B差分） |
| pixColorMagnitude | 色マグニチュード計算 |
| pixColorFraction | 色付き vs グレーの割合 |
| pixMaskOverColorPixels | 色付きピクセルマスク生成 |
| pixMaskOverGrayPixels | グレーピクセルマスク生成 |
| pixMaskOverColorRange | 色範囲マスク生成 |
| pixFindColorRegions | 文書内の色領域検出 |
| pixNumSignificantGrayColors | 有意なグレー色数 |
| pixColorsForQuantization | 量子化用色数決定 |
| pixGetMostPopulatedColors | 最頻出色取得 |
| pixSimpleColorQuantize | 簡易色量子化 |
| pixGetRGBHistogram | RGBヒストグラム |

修正ファイル: `crates/leptonica-color/src/analysis.rs`（既存拡張）

#### 7.2 HSV範囲マスク・ヒストグラム (`feat/color-hsv-tools`)

対象C関数（`colorspace.c`）:
| 関数 | 内容 |
|------|------|
| pixMakeRangeMaskHS | H-S範囲マスク |
| pixMakeRangeMaskHV | H-V範囲マスク |
| pixMakeRangeMaskSV | S-V範囲マスク |
| pixMakeHistoHS | H-S 2Dヒストグラム |
| pixMakeHistoHV | H-V 2Dヒストグラム |
| pixMakeHistoSV | S-V 2Dヒストグラム |
| pixFindHistoPeaksHSV | HSVヒストグラムピーク検出 |
| pixConvertRGBToYUV (画像) | 画像レベルRGB→YUV変換 |
| pixConvertYUVToRGB (画像) | 画像レベルYUV→RGB変換 |

修正ファイル: `crates/leptonica-color/src/colorspace.rs`（既存拡張）

#### 7.3 高度二値化 (`feat/color-binarize-adv`)

対象C関数（`binarize.c`, `grayquant.c`）:
| 関数 | 内容 |
|------|------|
| pixOtsuAdaptiveThreshold | タイル別適応的Otsu |
| pixOtsuThreshOnBackgroundNorm | 背景正規化Otsu |
| pixSauvolaBinarizeTiled | タイル別Sauvola |
| pixSauvolaOnContrastNorm | コントラスト正規化Sauvola |
| pixThresholdByConnComp | 連結成分ベース閾値 |
| pixVarThresholdToBinary | 可変閾値二値化 |
| pixGenerateMaskByValue | 値別マスク生成 |
| pixGenerateMaskByBand | バンド別マスク生成 |
| pixThresholdTo2bpp | 2bpp閾値処理 |
| pixThresholdTo4bpp | 4bpp閾値処理 |

修正ファイル: `crates/leptonica-color/src/threshold.rs`（既存拡張）

#### 7.4 量子化拡張 (`feat/color-quant-ext`)

対象C関数（`colorquant1.c`, `colorquant2.c`）:
| 関数 | 内容 |
|------|------|
| pixOctreeQuantByPopulation | ポピュレーション基準Octree量子化 |
| pixOctreeQuantNumColors | N色Octree量子化 |
| pixMedianCutQuantMixed | グレー+カラー混合MedianCut |
| pixQuantFromCmap | 既存カラーマップからの量子化 |
| pixRemoveUnusedColors | 未使用カラーマップ色の削除 |
| pixFixedOctcubeQuant256 | 固定256色Octcube量子化 |

修正ファイル: `crates/leptonica-color/src/quantize.rs`（既存拡張）

---

### Phase 8: leptonica-morph Sel系 + leptonica-region seedfill拡張（35関数）

Phase 6に依存。Selデータ構造の完成と距離変換の実装。

#### 8.1 Sel/Selaデータ構造 (`feat/morph-sel-basic`)

対象C関数（`sel1.c`）:
| 関数 | 内容 |
|------|------|
| selCreate | 構造化要素生成 |
| selCreateBrick | 矩形SE生成 |
| selCreateComb | 複合SE対生成 |
| selGetElement | 要素取得 |
| selSetElement | 要素設定 |
| selGetParameters | SEパラメータ取得 |
| selSetOrigin | SE原点設定 |
| selFindMaxTranslations | 最大平行移動量 |
| selRotateOrth | 直交回転 |
| selCreateFromString | テキスト表現から生成 |
| selCreateFromPix | 画像からSE生成 |
| selDisplayInPix | SEを画像表示 |
| selGenerateSelBoundary | 境界からSE自動生成 |

修正ファイル: `crates/leptonica-morph/src/sel.rs`（新規 or 既存拡張）

#### 8.2 モルフォロジー応用 (`feat/morph-app`)

対象C関数（`morphapp.c`）:
| 関数 | 内容 |
|------|------|
| pixMorphGradient | モルフォロジー勾配（dilate-erode） |
| pixExtractBoundary | 境界抽出（1px内側/外側） |
| pixMorphSequenceMasked | マスク付きモルフォロジーシーケンス |
| pixMorphSequenceByComponent | コンポーネント別シーケンス |
| pixMorphSequenceByRegion | リージョン別シーケンス |

修正ファイル: `crates/leptonica-morph/src/binary.rs`, `sequence.rs`

#### 8.3 距離関数・局所極値 (`feat/region-seedfill-dist`)

対象C関数（`seedfill.c`）:
| 関数 | 内容 |
|------|------|
| pixDistanceFunction | Chamfer距離変換 |
| pixSeedspread | シード拡散（Voronoi類似） |
| pixLocalExtrema | 局所極値検出 |
| pixSelectedLocalExtrema | 制約付き局所極値選択 |
| pixFindEqualValues | 等値隣接ピクセル検出 |
| pixSelectMinInConnComp | 連結成分内最小値選択 |
| pixRemoveSeededComponents | シード付き成分除去 |
| pixSeedfillGrayInv | 逆グレースケールシードフィル |
| pixSeedfillBinaryRestricted | 制限付き二値シードフィル |
| pixFillClosedBorders | 閉境界充填 |

修正ファイル: `crates/leptonica-region/src/seedfill.rs`（既存拡張）

#### 8.4 連結成分拡張 (`feat/region-conncomp-ext`)

対象C関数（`conncomp.c`, `pixlabel.c`）:
| 関数 | 内容 |
|------|------|
| pixConnCompPixa | 連結成分をPixaとして取得 |
| pixSeedfillBB | シードフィルBB付き |
| pixSeedfill4BB | 4連結シードフィルBB |
| pixSeedfill8BB | 8連結シードフィルBB |
| pixConnCompIncrInit | インクリメンタルCC初期化 |
| pixConnCompIncrAdd | インクリメンタルCC追加 |
| pixGetSortedNeighborValues | ソート済み隣接値取得 |

修正ファイル: `crates/leptonica-region/src/conncomp.rs`, `label.rs`

---

### Phase 9: leptonica-core Box/Pixa/FPix + 演算拡張（44関数）

Phase 7,8と並行可能。インフラ完成による API カバレッジ向上。

#### 9.1 Box配列操作 (`feat/core-boxfunc`)

対象C関数（`boxfunc1.c`, `boxfunc4.c`）:
| 関数 | 内容 |
|------|------|
| boxaContainedInBox | 包含ボックスフィルタ |
| boxaIntersectsBox | 交差ボックスフィルタ |
| boxaClipToBox | 全ボックスのクリッピング |
| boxaCombineOverlaps | 重複ボックス結合 |
| boxOverlapFraction | 重複割合計算 |
| boxOverlapArea | 重複面積計算 |
| boxaSelectBySize | サイズ基準選択 |
| boxaSelectByArea | 面積基準選択 |
| boxaSelectByWHRatio | 縦横比基準選択 |
| boxaGetExtent | 全ボックスの外接矩形 |
| boxaGetCoverage | 面積カバレッジ計算 |
| boxaSizeRange | サイズ範囲取得 |
| boxEqual | ボックス等値判定 |
| boxaSimilar | Boxa類似判定 |
| boxaJoin | 2つのBoxaを結合 |

修正ファイル: `crates/leptonica-core/src/boxa/`（新規ファイル）

#### 9.2 Pixa操作 (`feat/core-pixa-ops`)

| 関数 | 内容 |
|------|------|
| pixaSelectBySize | サイズ基準Pix選択 |
| pixaSelectByArea | 面積基準Pix選択 |
| pixaSort | 基準別Pixaソート |
| pixaSortByIndex | インデックス配列ソート |
| pixaScaleToSize | 共通サイズスケーリング |
| pixaScaleToSizeRel | 相対サイズスケーリング |
| pixaDisplay | Pixa複合画像表示 |
| pixaDisplayTiled | タイル表示 |
| pixaDisplayTiledAndScaled | タイル+スケーリング表示 |

修正ファイル: `crates/leptonica-core/src/pixa/`（既存拡張）

#### 9.3 FPix/DPix操作 (`feat/core-fpix-ops`)

| 関数 | 内容 |
|------|------|
| fpixCreateTemplate | テンプレートからFPix生成 |
| fpixConvertToPix | FPix→Pix変換 |
| pixConvertToFPix | Pix→FPix変換 |
| fpixAddMultConstant | 定数加算/乗算 |
| fpixLinearCombination | 2つのFPixの線形結合 |
| fpixConvolveSep | FPix分離可能畳み込み |
| fpixConvolve | FPix畳み込み |
| dpixCreate | DPix生成 |
| dpixConvertToPix | DPix→Pix変換 |
| dpixConvertToFPix | DPix→FPix変換 |

修正ファイル: `crates/leptonica-core/src/fpix/`（既存拡張）

#### 9.4 ピクセル演算・ラスタオペ拡張 (`feat/core-pix-arith`)

| 関数 | 内容 |
|------|------|
| pixAddGray | グレースケール画像加算 |
| pixSubtractGray | グレースケール画像減算 |
| pixMultConstantGray | 定数乗算 |
| pixAddConstantGray | 定数加算 |
| pixMultConstAccumulate | 乗算累積 |
| pixAbsDifference | 絶対差分 |
| pixMinOrMax | ピクセル単位min/max |
| pixRasteropVip | 垂直インプレースラスタオペ |
| pixRasteropHip | 水平インプレースラスタオペ |
| pixTranslate | ラスタオペによる画像移動 |

修正ファイル: `crates/leptonica-core/src/pix/arith.rs`, `rop.rs`

---

## サマリー

| Phase | 対象 | ブランチ数 | 関数数 | 累計 |
|-------|------|-----------|--------|------|
| 1-4 (完了) | core基盤, filter enhance/convolve | 19 | 111 | 111 |
| 5 (filter adaptmap+bilateral) | leptonica-filter | 4 | 27 | 138 |
| 6 (core stats/clip/Numa) | leptonica-core | 4 | 52 | 190 |
| 7 (color expansion) | leptonica-color | 4 | 37 | 227 |
| 8 (morph Sel + region seedfill) | leptonica-morph, leptonica-region | 4 | 35 | 262 |
| 9 (core Box/Pixa/FPix) | leptonica-core | 4 | 44 | 306 |
| **合計** | | **39** | **306** | **(387+306)/1863 ≈ 37%** |

## 検証方法

### Part 1（comparison更新）

ドキュメント更新のため自動テストは不要。以下を手動確認:

1. 各comparison/*.mdのサマリー数値がdetailテーブルの行数と一致すること
2. feature-comparison.mdの数値が各個別ファイルの合計と一致すること
3. ❌→✅に変更した関数について、対応するRust実装が存在することを `cargo doc -p <crate>` で確認

### Part 2（Phase 5-9）

各Phaseの実装時に `humming-tickling-journal.md` と同じワークフローを適用:

```bash
cargo fmt --check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo test -p <crate>
cargo test --workspace  # PR前
```
