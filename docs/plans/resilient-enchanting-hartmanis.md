# transform/morph/filter/region/recog 全未実装関数の移植計画

## Context

IO全移植計画（102）が完了し、IO crateのカバレッジが~58%に到達した。
次のステップとして、残る5つの計算処理crateの未実装部分を移植する。

現状カバレッジ:
- leptonica-transform: 33.6% (51/152)
- leptonica-morph: 38.3% (46/120)
- leptonica-filter: 50.5% (50/99)
- leptonica-region: 36.8% (35/95)
- leptonica-recog: 26.2% (62/237)

目標: デバッグ/可視化関数・低レベルC固有ヘルパーを除外した上で、残りの実用的な関数を全て実装する。

---

## 成果物: 5つの計画書

102_io-full-porting.md と同じ形式で、以下の5つの計画書を作成しコミットする:

1. `300_transform-full-porting.md`
2. `301_morph-full-porting.md`
3. `401_filter-full-porting.md`
4. `500_region-full-porting.md`
5. `700_recog-full-porting.md`

各計画書のコミット後、計画書に記載された順序で実装PRを作成する。

---

## 共通スコープ除外

全crateに共通して以下を除外:

| 除外対象 | 理由 |
|----------|------|
| `*Display*`, `*Show*`, `*Debug*` 関数 | デバッグ/可視化専用 |
| `*Low` 接尾辞の関数 | C実装固有の低レベルヘルパー（Rust版は内部実装で対応済み） |
| `make*Tab*` 系関数 | Cルックアップテーブル生成（Rustでは不要） |
| グローバル状態設定関数 (`reset*`, `l_set*`) | Rustではオプション構造体で対応 |

---

## 計画書1: 300_transform-full-porting.md

### 実装対象（スコープ除外後: ~45関数）

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Alpha変換サポート | 1 | 3 (affine/bilinear/projective WithAlpha) |
| 2 | PTA/BOXA変換ユーティリティ | 1 | 8 (translate/scale/rotate/affineTransform × PTA+BOXA) |
| 3 | Scale拡張 - 基本 | 1 | ~8 (pixScaleLI, pixScaleGeneral, ToSize系, BySampling系) |
| 4 | Scale拡張 - 1bpp→8bpp変換 | 1 | ~10 (pixScaleToGray系、pixExpandReplicate) |
| 5 | Scale拡張 - 特殊 | 1 | ~8 (2x/4xLI, MinMax, Rank, Mipmap) |
| 6 | Rotation拡張 | 1 | ~5 (Corner系、Center系、IP系、pixRotateWithAlpha) |
| 7 | Flip検出 | 1 | ~3 (pixOrientDetect, pixOrientCorrect, pixMirrorDetect) |

### スコープ除外

| 除外対象 | 理由 |
|----------|------|
| `pixScaleRGBToGrayFast`, `pixScaleRGBToBinaryFast` 等 | 変換はcore側の深度変換 + scale の組み合わせで対応可能 |
| `pixRotateAMColorFast` | 精度が低い近似実装、通常のAreaMapで十分 |
| `l_productMat*` | 汎用行列乗算ユーティリティ（nalgebra等を使えば良い） |
| `pixAffineSequential` | 行列合成で対応可能 |

### 修正ファイル

- `crates/leptonica-transform/src/affine.rs`: WithAlpha
- `crates/leptonica-transform/src/bilinear.rs`: WithAlpha
- `crates/leptonica-transform/src/projective.rs`: WithAlpha
- `crates/leptonica-transform/src/scale.rs`: 大幅拡張
- `crates/leptonica-transform/src/rotate.rs`: Corner/Center/IP拡張
- `crates/leptonica-transform/src/flipdetect.rs` (新規)
- `crates/leptonica-core/src/pta.rs`: PTA変換メソッド追加
- `crates/leptonica-core/src/boxa.rs`: BOXA変換メソッド追加

---

## 計画書2: 301_morph-full-porting.md

### 実装対象（スコープ除外後: ~35関数）

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Safe closing + Generalized ops | 1 | 5 (closeSafe系, openGeneralized, closeGeneralized) |
| 2 | Morphological applications | 1 | ~8 (masked sequence, by-component, union/intersection, HDome, FastTophat) |
| 3 | SEL管理拡張 | 1 | ~8 (Sel I/O, createFromString, createFromPix, rotateOrth, findMaxTranslation) |
| 4 | SEL生成 | 1 | ~5 (generateSelBoundary, generateSelWithRuns, generateSelRandom) |
| 5 | DWA拡張 + シーケンス拡張 | 1 | ~6 (Extended DWA, DWA sequence, color sequence) |
| 6 | Sela配列管理 | 1 | ~3 (selaCreate, findSelByName, getSel) |

### スコープ除外

| 除外対象 | 理由 |
|----------|------|
| `fmorphautogen*`, `fmorphgen*` | DWAコード生成はRustでは不要（手書き実装済み） |
| `selDisplayInPix`, `selaDisplayInPix` | 可視化専用 |
| `resetMorphBoundaryCondition`, `getMorphBorderPixelColor` | グローバル状態（Rustではオプション構造体で対応） |
| `pixaThinConnected` | Pixa操作はアプリケーション層でループ |
| `pixDisplayHitMissSel` | デバッグ可視化 |
| `pixRemoveMatchedPattern`, `pixDisplayMatchedPattern` | パターン可視化 |

### 修正ファイル

- `crates/leptonica-morph/src/binary.rs`: safe closing, generalized ops
- `crates/leptonica-morph/src/morphapp.rs` (新規): application関数群
- `crates/leptonica-morph/src/sel.rs`: I/O、生成、操作拡張
- `crates/leptonica-morph/src/dwa.rs`: Extended DWA
- `crates/leptonica-morph/src/sequence.rs`: DWA/colorシーケンス

---

## 計画書3: 401_filter-full-porting.md

### 実装対象（スコープ除外後: ~15関数）

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | FPix畳み込み | 1 | 3 (fpixConvolve, fpixConvolveSep, pixConvolveWithBias) |
| 2 | Tiled block畳み込み | 1 | 2 (pixBlockconvTiled, pixBlockconvGrayTile) |
| 3 | Adaptmap拡張 | 1 | ~5 (foreground map, threshold spread, flex norm, smoothConnected) |
| 4 | Block bilateral + 追加 | 1 | ~3 (pixBlockBilateralExact, pixGlobalNormNoSatRGB) |

### スコープ除外

| 除外対象 | 理由 |
|----------|------|
| `l_setConvolveSampling` | グローバル状態設定（オプション構造体で対応済み） |
| `gaussDistribSampling` | 統計ユーティリティ（randクレートで対応可） |

### 修正ファイル

- `crates/leptonica-filter/src/convolve.rs`: FPix畳み込み、bias、tiled
- `crates/leptonica-filter/src/adaptmap.rs`: foreground map、flex norm等
- `crates/leptonica-filter/src/bilateral.rs`: block bilateral

---

## 計画書4: 500_region-full-porting.md

### 実装対象（スコープ除外後: ~35関数）

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Seedfill拡張 | 1 | ~8 (border comp, hole filling variants, simple gray seedfill) |
| 2 | Local extrema | 1 | 3 (pixLocalExtrema, pixQualifyLocalMinima, pixSelectedLocalExtrema) |
| 3 | ConnComp拡張 | 1 | ~5 (pixCountConnComp, nextOnPixelInRaster, seedfillBB系) |
| 4 | Label拡張 | 1 | ~4 (connCompTransform, connCompAreaTransform, incrInit/Add, locToColor) |
| 5 | CCBord拡張 | 1 | ~6 (step chains, single path, I/O, SVG export) |
| 6 | Watershed拡張 | 1 | ~3 (basin tracking, render fill/colors) |
| 7 | Gray maze | 1 | 1 (pixSearchGrayMaze) |

### スコープ除外

| 除外対象 | 理由 |
|----------|------|
| `ccbaDisplayImage1/2`, `ccbaDisplayBorder/SPBorder` | 可視化専用 |
| `pageseg.c` 全体 | leptonica-recogのpageseg.rsで既に基本実装あり |
| `classapp.c` 全体 | leptonica-recogのjbclass.rsで既にカバー |

### 修正ファイル

- `crates/leptonica-region/src/seedfill.rs`: border comp、hole filling、simple variants、local extrema
- `crates/leptonica-region/src/conncomp.rs`: count、nextPixel、seedfillBB
- `crates/leptonica-region/src/label.rs`: transform、incremental
- `crates/leptonica-region/src/ccbord.rs`: chain code生成、I/O
- `crates/leptonica-region/src/watershed.rs`: basin、render
- `crates/leptonica-region/src/maze.rs`: gray maze

---

## 計画書5: 700_recog-full-porting.md

### 実装対象（スコープ除外後: ~55関数）

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Recog シリアライゼーション | 1 | ~6 (read/write for Recog) |
| 2 | Recog query/inspection | 1 | ~8 (getCount, getClassIndex, getClassString, setParams等) |
| 3 | Bootstrap digit recognizer | 1 | ~4 (makeBootDigitRecog, trainFromBoot, digitPad) |
| 4 | 高度な識別・フィルタリング | 1 | ~5 (preSplittingFilter, splittingFilter, correlationBestShift) |
| 5 | Dewarp シリアライゼーション | 1 | ~4 (dewarpRead/Write) |
| 6 | Dewarpa コンテナ管理 | 1 | ~8 (create, insert, get, set系) |
| 7 | Dewarpa モデル管理 | 1 | ~6 (testForValidModel, setValidModels, insertRefModels等) |
| 8 | Dewarp2 高度なモデル構築 | 1 | ~5 (buildPageModel, LSF, endpoint処理) |
| 9 | Dewarp3/4 拡張適用 | 1 | ~5 (boxaApplyDisparity, dewarpa適用, singlePageInit/Run) |
| 10 | JbClass シリアライゼーション + 拡張 | 1 | ~6 (jbDataRead/Write, word detection) |
| 11 | Skew拡張 | 1 | ~5 (sweepAndSearch variants, deskew variants) |
| 12 | Baseline拡張 | 1 | ~3 (localSkew, deskewLocal) |
| 13 | Barcode拡張 | 1 | ~5 (width extraction, peak detection, mask generation) |

### スコープ除外

| 除外対象 | 理由 |
|----------|------|
| `recogShowAverageTemplates`, `recogShowContent`, `recogShowMatch` 等 | デバッグ可視化 |
| `dewarpShowResults`, `dewarpDebug`, `dewarpaShowArrays` | デバッグ可視化 |
| `pixDisplayOutliers`, `recogDisplayOutlier` | デバッグ可視化 |
| `recogShowPath` | デバッグ可視化 |
| `showExtractNumbers`, `l_showIndicatorSplitValues` | デバッグ可視化 |
| `jbDataRender` | 可視化 |
| `pixRenderHorizEndPoints`, `pixRenderMidYs` | 可視化 |

### 修正ファイル

- `crates/leptonica-recog/src/recog/types.rs`: 構造体拡張
- `crates/leptonica-recog/src/recog/io.rs` (新規): シリアライゼーション
- `crates/leptonica-recog/src/recog/ident.rs`: フィルタリング拡張
- `crates/leptonica-recog/src/recog/train.rs`: bootstrap、digit pad
- `crates/leptonica-recog/src/dewarp/`: 各サブモジュール拡張
- `crates/leptonica-recog/src/dewarp/io.rs` (新規): シリアライゼーション
- `crates/leptonica-recog/src/dewarp/dewarpa.rs` (新規): コンテナ管理
- `crates/leptonica-recog/src/jbclass/io.rs` (新規): シリアライゼーション
- `crates/leptonica-recog/src/skew.rs`: sweep/search拡張
- `crates/leptonica-recog/src/baseline.rs`: local skew
- `crates/leptonica-recog/src/barcode/`: width/peak拡張

---

## 実行順序

5つの計画書は依存関係に基づいて以下の順序で実装する:

```
1. 300_transform (依存なし、他crateの基盤)
2. 301_morph (依存なし、他crateの基盤)
3. 401_filter (transform/morphに軽く依存)
4. 500_region (coreに依存)
5. 700_recog (transform/morph/filter/regionに依存)
```

1と2は並行実装可能（worktree分離時）。3以降は直列。

---

## 全体サマリー

| 計画書 | crate | Phase数 | PR数 | 推定関数数 |
|--------|-------|---------|------|-----------|
| 300 | transform | 7 | 7 | ~45 |
| 301 | morph | 6 | 6 | ~35 |
| 401 | filter | 4 | 4 | ~15 |
| 500 | region | 7 | 7 | ~30 |
| 700 | recog | 13 | 13 | ~55 |
| **合計** | | **37** | **37** | **~180** |

## ワークフロー

IO計画(102)と同じワークフローを踏襲する。

### 直列実行の厳守

**同一worktree内では1つのPRが完全にマージされるまで、次のPhaseの実装を開始してはならない。**

- Phase N のPRがマージされていない状態で Phase N+1 のコードを書き始めない
- 「レビュー待ちの間に次を進める」は禁止（worktreeを分ける場合のみ例外）
- 計画書内のPhase順序は依存関係を反映しており、順番を入れ替えない

### Copilotレビュー待ちの厳守

**PRを作成したら、Copilotレビューが到着するまで必ず待機する。レビュー到着前にマージしない。**

1. PR作成後、`/gh-actions-check` でCIとCopilotレビューの状態を確認する
2. Copilotレビューが到着していなければ、到着するまで待つ
3. Copilotレビュー指摘があれば全て確認・対応してからマージする
4. CIが失敗していれば原因を調査・修正し、再度CIパスを確認する
5. 上記全てが完了して初めて `/gh-pr-merge --merge` を実行する

**以下は明確な違反であり、絶対に行わない:**
- 「Copilotレビューが来ないので先にマージする」
- 「変更が小さいのでレビュー不要」
- 「前のPRと似ているのでレビューを省略する」
- レビュー指摘を未確認のままマージする

### PRワークフロー（各Phaseで繰り返す）

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]` 付き）
2. **GREEN**: 実装コミット（`#[ignore]` 除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット
4. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
5. `/gh-pr-create` でPR作成
6. `/gh-actions-check` でCopilotレビュー到着を確認（**到着するまで待つ**）
7. `/gh-pr-review` でレビューコメント対応
8. 対応コミット後、再度CIパスを確認
9. CIパス確認後 `/gh-pr-merge --merge` でマージ
10. ブランチ削除
11. **マージ完了後に** 次のPhaseの実装を開始

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p <crate名>
cargo clippy -p <crate名> -- -D warnings
cargo test -p <crate名>
cargo test --workspace  # PR前に全ワークスペーステスト
```
