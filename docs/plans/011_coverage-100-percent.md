# 未実装関数の実装（カバレッジ100%達成）

**Status: PLANNED**

## 概要

feature-comparison.md によると、❌未実装は95関数（5モジュール）。
実カバレッジ94.8%を100%に引き上げる。

## C版対応ファイル

| Phase | モジュール | C版ファイル | ❌数 |
|-------|-----------|-----------|------|
| 1 | io | pngio.c, jpegio.c | 2 |
| 2 | filter | kernel.c, adaptmap.c | 11 |
| 3 | recog | classapp.c | 3 |
| 4 | core | boxfunc2.c, boxfunc5.c | 33 |
| 5 | misc | strokes.c, runlength.c, partition.c, partify.c, bmf.c, textops.c, gplot.c, binreduce.c, checkerboard.c, convertfiles.c, finditalic.c | 46 |
| **合計** | | | **95** |

## アプローチ

- モジュールごとにブランチを切り、順次実装（並列なし）
- TDDワークフロー: RED → GREEN → REFACTOR
- PRはGitHub上のレビュー完了後にマージ
- gplot.cはgnuplot非依存でRust向け再設計（SVG直接生成 or plottersクレート使用）
- bmf/textopsも含めて全て実装

## Phase 1: io（2関数）

**ブランチ**: `feat/io-coverage`

### pngio.c
- `fgetPngColormapInfo` → PNG colormapのcolor_type/bit_depth/ncolors/nentries取得

### jpegio.c
- `fgetJpegComment` → JPEGコメント(COM)マーカー読み取り

## Phase 2: filter（11関数）

**ブランチ**: `feat/filter-coverage`

### kernel.c（10関数）
- `kernelGetMinMax` — カーネルのmin/max値
- `kernelInvert` — カーネル反転（max - val）
- `kernelRead`/`kernelReadStream` — ファイル/ストリームからカーネル読み込み
- `kernelWrite`/`kernelWriteStream` — ファイル/ストリームへカーネル書き込み
- `kernelCreateFromString` — 文字列パースでカーネル生成
- `kernelCreateFromFile` — ファイルからカーネル生成
- `kernelCreateFromPix` — Pixからカーネル生成
- `kernelDisplayInPix` — Pix内にカーネル可視化

### adaptmap.c（1関数）
- `pixGetForegroundGrayMap` — グレー前景マップ取得

## Phase 3: recog（3関数）

**ブランチ**: `feat/recog-coverage`

### classapp.c（3関数）
- `pixFindWordAndCharacterBoxes` — 単語/文字ボックス検出
- `boxaExtractSortedPattern` — パターンに基づくBoxa抽出
- `numaaCompareImagesByBoxes` — ボックスベースの画像比較

## Phase 4: core（33関数）

**ブランチ**: `feat/core-coverage`

### boxfunc2.c（20関数）
- 変換: `boxaTransformOrdered`, `boxTransformOrdered`, `boxaRotateOrth`, `boxRotateOrth`, `boxaShiftWithPta`
- ソート: `boxaBinSort`, `boxaSortByIndex`, `boxaSort2d`, `boxaSort2dByIndex`
- 抽出: `boxaExtractAsNuma`, `boxaExtractAsPta`, `boxaExtractCorners`
- 統計: `boxaGetRankVals`, `boxaGetMedianVals`, `boxaGetAverageSize`
- Boxaa: `boxaaGetExtent`, `boxaaFlattenAligned`, `boxaEncapsulateAligned`, `boxaaTranspose`, `boxaaAlignBox`

### boxfunc5.c（13関数）
- スムージング: `boxaSmoothSequenceMedian`, `boxaWindowedMedian`
- 調整: `boxaModifyWithBoxa`, `boxaReconcilePairWidth`, `boxaSizeConsistency`
- メディアン: `boxaReconcileAllByMedian`, `boxaReconcileSidesByMedian`, `boxaReconcileSizeByMedian`
- プロット: `boxaPlotSides`, `boxaPlotSizes` (gplot依存 → Phase 5後に実装)
- シーケンス: `boxaFillSequence`, `boxaSizeVariation`, `boxaMedianDimensions`

## Phase 5: misc（46関数）

**ブランチ**: `feat/misc-coverage`

### strokes.c（7関数）
- `pixFindStrokeLength` — 線長検出
- `pixFindStrokeWidth` — 線幅検出
- `pixaFindStrokeWidth` — 複数線幅検出
- `pixaModifyStrokeWidth` — 線幅変更
- `pixModifyStrokeWidth` — 単一線幅変更
- `pixaSetStrokeWidth` — 線幅設定
- `pixSetStrokeWidth` — 単一線幅設定

### runlength.c（3関数）
- `pixRunlengthTransform` — ランレングス変換
- `runlengthMembershipOnLine` — ライン上のランレングスメンバーシップ
- `makeMSBitLocTab` — MSBビット位置テーブル生成

### partition.c（2関数）
- `boxaGetWhiteblocks` — ホワイトブロック検出
- `boxaPruneSortedOnOverlap` — オーバーラップに基づくプルーニング

### partify.c（2関数）
- `partifyFiles` — ファイル分割
- `partifyPixac` — Pixac分割

### bmf.c（5関数）
- `bmfCreate` — ビットマップフォント作成
- `bmfGetPix` — フォント画像取得
- `bmfGetWidth` — フォント幅取得
- `bmfGetBaseline` — ベースライン取得
- `pixaGetFont` — フォント取得

### textops.c（9関数）— bmf依存
- `pixAddTextlines` — テキストライン追加
- `pixSetTextblock` — テキストブロック設定
- `pixSetTextline` — テキストライン設定
- `pixaAddTextNumber` — テキスト番号付き追加
- `pixaAddTextlines` — テキストライン群追加
- `pixaAddPixWithText` — テキスト付きPix追加
- `bmfGetLineStrings` — ライン文字列取得
- `bmfGetWordWidths` — 単語幅取得
- `bmfGetStringWidth` — 文字列幅取得

### gplot.c（13関数）— Rust向け再設計
gnuplot非依存。SVG直接生成またはplottersクレートを使用。
- `gplotCreate` — グラフ作成
- `gplotAddPlot` — プロット追加
- `gplotSetScaling` — スケーリング設定
- `gplotMakeOutputPix` — 出力Pix生成
- `gplotMakeOutput` — 出力ファイル生成
- `gplotGenCommandFile` — コマンドファイル生成
- `gplotGenDataFiles` — データファイル生成
- `gplotSimple1/2/N` — 単純グラフ
- `gplotSimplePix1/2/N` — Pix単純グラフ

### その他（4関数）
- `pixReduceBinary2` (binreduce.c) — 2倍縮小
- `pixFindCheckerboardCorners` (checkerboard.c) — チェッカーボードコーナー検出
- `convertFilesTo1bpp` (convertfiles.c) — ファイルを1bppに変換
- `pixItalicWords` (finditalic.c) — イタリック体単語検出

## 各Phase共通ワークフロー

1. ブランチ作成（`/git-branch`）
2. RED: テスト作成（`#[ignore = "not yet implemented"]`付き）→ コミット
3. GREEN: 実装（`#[ignore]`除去）→ コミット
4. `cargo test --all-features` + `cargo clippy` + `cargo fmt` 確認
5. push → PR作成
6. CI + Copilotレビュー待ち（`/gh-actions-check`）
7. レビューコメント対応（独立`fix(<scope>):`コミット）
8. 全チェック通過後マージ（`/gh-pr-merge --merge`）
9. `docs/porting/`の実装状況を更新

## 依存関係

- Phase 1-3: 独立（依存なし）
- Phase 4: boxaPlotSides/boxaPlotSizes は gplot依存 → Phase 5のgplot実装後
- Phase 5: bmf → textops（bmf先に実装）
