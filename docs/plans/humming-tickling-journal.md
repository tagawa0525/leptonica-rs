# leptonica-rs 未実装関数の段階的実装計画

## Context

C版leptonicaの1,863関数に対してRust版は387関数（20.8%）を実装済み。残り1,476関数を段階的に実装する。
依存関係・インパクト・実現可能性の順で優先度を決定し、クレートごとにサブエージェント、ファイルごとにサブサブエージェントで並列管理する。

## エージェント構成

```
メインエージェント [Opus]（進捗管理）
├── サブエージェント [Opus]: leptonica-core（クレート進捗管理・指示出し）
│   ├── サブサブ [Sonnet]: pixconv-rgb2gray（実装）
│   ├── サブサブ [Sonnet]: pix-rgb（実装）
│   └── ...
├── サブエージェント [Opus]: leptonica-filter（クレート進捗管理・指示出し）
│   ├── サブサブ [Sonnet]: enhance-trc（実装）
│   └── ...
└── サブエージェント [Opus]: leptonica-color（クレート進捗管理・指示出し）
    └── ...
```

- サブサブエージェントはSonnetを使用（コスト・速度効率のため）
- パターンが確立済み＋C参照コードあり＋スコープが狭いため十分
- 複雑なアルゴリズム（積分画像、fast bilateral等）でSonnetが苦戦した場合、サブエージェントの判断でOpusに切り替え可

## ブランチ戦略

```
main
└── feat/core-pixconv          ← クレートブランチ（PRターゲット）
    ├── feat/core-pixconv-rgb2gray  ← ファイルブランチ（作業単位）
    ├── feat/core-pixconv-cmap
    └── feat/core-pixconv-depth
```

- ファイルブランチ → クレートブランチへマージ（ローカル `git merge --no-ff`）
- クレートブランチ → main へPR（GitHub Copilotレビュー必須）

## PRワークフロー（厳守）

1. PR作成前に全テスト・リントを通す（`cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --check`）
2. `/gh-pr-create` でPR作成
3. GitHub Copilotの自動レビューを**必ず待つ**（3〜10分かかる）
4. `/gh-pr-review` でレビューコメントを確認し、指摘事項を修正
5. 指摘対応完了後 `/gh-pr-merge` でマージ（`--merge`オプション）
6. マージ後のブランチは速やかに削除

**禁止事項**: レビューを待たずに勝手にマージすること。レビュー結果を確認せずにマージすること。

## ドキュメント更新

各クレートのサブエージェントは実装完了時に以下のドキュメントも更新する:
- `docs/rebuild/comparison/<crate>.md` — 実装した関数の状態を ❌→✅ に更新
- `docs/rebuild/feature-comparison.md` — カバレッジ数値を更新

## TDDワークフロー（各ファイルブランチ内）

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

各コミット前チェック:
```bash
cargo fmt --check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo test -p <crate>
```

## 実装順序

依存関係に基づく順序。Phase 1（core基盤）が後続全てのPhaseの前提条件。

### Phase 1: leptonica-core 基盤関数

他クレートが依存する変換・操作関数。全ての後続Phaseの前提条件。

#### 1.1 RGB→Gray変換 (`feat/core-pixconv-rgb2gray`)

対象C関数（`pixconv.c`）:
| 関数 | 内容 |
|------|------|
| `pixConvertRGBToLuminance` | 標準重み付き輝度変換 |
| `pixConvertRGBToGray` | カスタム重み付きRGB→Gray |
| `pixConvertRGBToGrayFast` | Greenチャンネル抽出（最速） |
| `pixConvertRGBToGrayMinMax` | RGB min/max チャンネル |
| `pixConvertRGBToGraySatBoost` | 彩度ブースト変換 |
| `pixConvertRGBToGrayGeneral` | 上記のディスパッチャー |

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`（既存拡張）

#### 1.2 RGBコンポーネント操作 (`feat/core-pix-rgb`)

対象C関数（`pix2.c`）:
| 関数 | 内容 |
|------|------|
| `pixGetRGBComponent` | R/G/B/Aチャンネルを8bppとして抽出 |
| `pixSetRGBComponent` | 8bppからチャンネル設定 |
| `pixCreateRGBImage` | 3枚の8bppから32bpp合成 |
| `pixGetRGBPixel` | 単一ピクセルのR,G,B取得 |
| `pixSetRGBPixel` | 単一ピクセルのR,G,B設定 |

新規ファイル: `crates/leptonica-core/src/pix/rgb.rs`

#### 1.3 カラーマップ除去 (`feat/core-pixconv-cmap`)

対象C関数（`pixconv.c`）:
| 関数 | 内容 |
|------|------|
| `pixRemoveColormap` | カラーマップ展開（自動深度選択） |
| `pixRemoveColormapGeneral` | ターゲット深度指定版 |
| `pixAddGrayColormap8` | 8bppにグレーカラーマップ追加 |
| `pixAddMinimalGrayColormap8` | 最小グレーカラーマップ追加 |

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`

#### 1.4 深度変換（トップレベル） (`feat/core-pixconv-depth`)

対象C関数（`pixconv.c`）:
| 関数 | 内容 |
|------|------|
| `pixConvertTo1` | 任意深度→1bpp（閾値指定） |
| `pixConvertTo1Adaptive` | 適応的閾値版 |
| `pixConvertTo16` | 任意深度→16bpp |
| `pixConvert8To32` | 8bpp→32bpp |
| `pixConvert8To16` | 8bpp→16bpp |
| `pixConvert16To8` | 16bpp→8bpp |
| `pixConvert32To8` | 32bpp→8bpp |
| `pixConvertTo8Or32` | 入力に基づき8or32選択 |
| `pixConvertLossless` | ロスレス深度拡大 |
| `pixRemoveAlpha` | アルファを白背景でブレンド |

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`

#### 1.5 バイナリ展開 (`feat/core-pixconv-unpack`)

対象C関数（`pixconv.c`）:
| 関数 | 内容 |
|------|------|
| `pixUnpackBinary` | 1bpp→2/4/8/16/32bpp |
| `pixConvert1To2/4/8` | 値マッピング付き展開 |
| `pixConvert1To2/4/8Cmap` | カラーマップ付き展開 |
| `pixConvert1To16/32` | 高深度への展開 |

修正ファイル: `crates/leptonica-core/src/pix/convert.rs`

#### 1.6 ボーダー操作 (`feat/core-pix-border`)

対象C関数（`pix2.c`）:
| 関数 | 内容 |
|------|------|
| `pixAddBorder` | 均一ボーダー追加 |
| `pixAddBorderGeneral` | 上下左右個別指定 |
| `pixRemoveBorder` | ボーダー除去 |
| `pixRemoveBorderGeneral` | 個別指定除去 |
| `pixAddBlackOrWhiteBorder` | 白黒ボーダー |
| `pixSetBorderVal` | ボーダー値設定 |
| `pixAddMirroredBorder` | ミラーボーダー（畳み込み用） |
| `pixAddRepeatedBorder` | リピートボーダー |

修正ファイル: `crates/leptonica-core/src/pix/border.rs`（既存拡張）

#### 1.7 マスク操作 (`feat/core-pix-mask`)

対象C関数（`pix3.c`）:
| 関数 | 内容 |
|------|------|
| `pixSetMasked` | マスクONの位置に値設定 |
| `pixCombineMasked` | マスクを通じて2画像合成 |
| `pixPaintThroughMask` | マスクを通じてペイント |
| `pixMakeMaskFromVal` | 特定値からマスク生成 |
| `pixMakeMaskFromLUT` | LUTからマスク生成 |

新規ファイル: `crates/leptonica-core/src/pix/mask.rs`

#### 1.8 ピクセルカウント・行列統計 (`feat/core-pix-counting`)

対象C関数（`pix3.c`）:
| 関数 | 内容 |
|------|------|
| `pixCountPixelsInRect` | 矩形領域内のONピクセル数 |
| `pixCountByRow` | 行ごとのONピクセル数 |
| `pixCountByColumn` | 列ごとのONピクセル数 |
| `pixZero` | 全ピクセルがゼロか判定 |
| `pixForegroundFraction` | ONピクセルの割合 |
| `pixThresholdPixelSum` | ピクセル数が閾値超か判定 |

修正ファイル: `crates/leptonica-core/src/pix/statistics.rs`（既存拡張）

---

### Phase 2: leptonica-filter enhance.c（画像強調）

Phase 1のcore関数（特にpixconv, RGB操作, マスク）に依存。

#### 2.1 TRC基盤 (`feat/filter-enhance-trc`)

| 関数 | 内容 |
|------|------|
| `numaGammaTRC` | ガンマLUT生成（256エントリ） |
| `numaContrastTRC` | コントラストLUT生成 |
| `numaEqualizeTRC` | ヒストグラム均等化LUT生成 |
| `pixTRCMap` | LUT適用（8bpp/32bpp） |
| `pixTRCMapGeneral` | R,G,B個別LUT適用 |

新規ファイル: `crates/leptonica-filter/src/enhance.rs`

#### 2.2 ガンマ・コントラスト・均等化 (`feat/filter-enhance-gamma`)

| 関数 | 内容 |
|------|------|
| `pixGammaTRC` | ガンマTRC適用 |
| `pixGammaTRCMasked` | マスク付きガンマ |
| `pixGammaTRCWithAlpha` | アルファ保持ガンマ |
| `pixContrastTRC` | コントラストTRC |
| `pixContrastTRCMasked` | マスク付きコントラスト |
| `pixEqualizeTRC` | ヒストグラム均等化 |

修正ファイル: `crates/leptonica-filter/src/enhance.rs`

#### 2.3 HSV修正 (`feat/filter-enhance-hsv`)

| 関数 | 内容 |
|------|------|
| `pixModifyHue` | 色相シフト |
| `pixModifySaturation` | 彩度スケーリング |
| `pixModifyBrightness` | 明度スケーリング |
| `pixMeasureSaturation` | 平均彩度測定 |

修正ファイル: `crates/leptonica-filter/src/enhance.rs`

#### 2.4 カラーシフト・行列変換 (`feat/filter-enhance-colorops`)

| 関数 | 内容 |
|------|------|
| `pixColorShiftRGB` | RGB定数加算 |
| `pixMosaicColorShiftRGB` | タイル別カラーシフト |
| `pixDarkenGray` | 低彩度ピクセル暗色化 |
| `pixMultConstantColor` | チャンネル別定数乗算 |
| `pixMultMatrixColor` | 3x3色行列変換 |
| `pixHalfEdgeByBandpass` | バンドパスエッジ検出 |

修正ファイル: `crates/leptonica-filter/src/enhance.rs`

---

### Phase 3: leptonica-filter convolve.c（高速畳み込み）

#### 3.1 ブロック畳み込み (`feat/filter-convolve-block`)

| 関数 | 内容 |
|------|------|
| `pixBlockconvAccum` | 積分画像（アキュムレータ）生成 |
| `pixBlockconvGray` | 積分画像によるGray畳み込み |
| `pixBlockconv` | Gray/Color自動ディスパッチ |
| `pixBlockconvGrayUnnormalized` | 正規化なし版 |
| `pixBlockconvTiled` | タイル化大画像対応 |
| `pixBlockconvGrayTile` | Grayタイル化版 |

新規ファイル: `crates/leptonica-filter/src/block_conv.rs`

#### 3.2 ウィンドウ統計 (`feat/filter-convolve-windowed`)

| 関数 | 内容 |
|------|------|
| `pixWindowedMean` | 局所平均（積分画像利用） |
| `pixWindowedMeanSquare` | 局所平均二乗 |
| `pixWindowedVariance` | 局所分散 |
| `pixWindowedStats` | 全統計量一括計算 |
| `pixMeanSquareAccum` | 平均二乗アキュムレータ |

新規ファイル: `crates/leptonica-filter/src/windowed.rs`

#### 3.3 分離可能畳み込み (`feat/filter-convolve-sep`)

| 関数 | 内容 |
|------|------|
| `pixConvolveSep` | 分離可能2D畳み込み |
| `pixConvolveRGBSep` | RGB分離可能畳み込み |

修正ファイル: `crates/leptonica-filter/src/convolve.rs`

#### 3.4 アンシャープマスク拡張 (`feat/filter-enhance-unsharp`)

Phase 3.1のブロック畳み込みに依存。

| 関数 | 内容 |
|------|------|
| `pixUnsharpMasking` | カラー対応ラッパー |
| `pixUnsharpMaskingFast` | ブロック畳み込み高速版 |
| `pixUnsharpMaskingGrayFast` | Gray高速版 |
| `pixUnsharpMaskingGray1D` | 1D版 |
| `pixUnsharpMaskingGray2D` | 2D版 |

修正ファイル: `crates/leptonica-filter/src/edge.rs`（既存拡張）

#### 3.5 その他 (`feat/filter-convolve-misc`)

| 関数 | 内容 |
|------|------|
| `pixCensusTransform` | ローカルバイナリパターン |
| `pixAddGaussianNoise` | ガウシアンノイズ追加 |
| `pixConvolveWithBias` | バイアス付き畳み込み |
| `pixBlockrank` | バイナリブロックランク |
| `pixBlocksum` | バイナリブロック和 |

修正ファイル: `crates/leptonica-filter/src/convolve.rs`

---

### Phase 4: leptonica-core 統計・Numa拡張

#### 4.1 ヒストグラム拡張 (`feat/core-stats-histogram`)

| 関数 | 内容 |
|------|------|
| `pixGetGrayHistogramMasked` | マスク内ヒストグラム |
| `pixGetGrayHistogramInRect` | 矩形内ヒストグラム |
| `pixGetColorHistogramMasked` | マスク内カラーヒストグラム |
| `pixGetRankValue` | ランク順値取得 |
| `pixGetExtremeValue` | 最小/最大値 |
| `pixGetMaxValueInRect` | 矩形内最大値 |
| `pixGetRangeValues` | 値範囲 |

修正ファイル: `crates/leptonica-core/src/pix/histogram.rs`, `statistics.rs`

#### 4.2 Numa高度操作 (`feat/core-numa-ops`)

| 関数 | 内容 |
|------|------|
| `numaSort` | ソート |
| `numaGetMedian` | 中央値 |
| `numaGetMode` | 最頻値 |
| `numaGetRankValue` | ランク値 |
| `numaReverse` | 反転 |
| `numaMakeSequence` | 数列生成 |
| `numaMakeConstant` | 定数配列生成 |
| `numaJoin` | 結合 |

修正ファイル: `crates/leptonica-core/src/numa/operations.rs`

---

### Phase 5以降: 後続クレート

Phase 1-4完了後に計画策定:

- **Phase 5**: leptonica-filter adaptmap.c拡張（背景マップ、モルフォロジーベース正規化）
- **Phase 6**: leptonica-filter bilateral.c（高速バイラテラル）
- **Phase 7**: leptonica-core pix5.c（クリップ、測定）、boxfunc拡張
- **Phase 8**: leptonica-color 拡張（画像レベル変換、色分析、量子化拡張）
- **Phase 9+**: leptonica-region, leptonica-morph, leptonica-recog 拡張

## 対象関数数サマリー

| Phase | ブランチ数 | 関数数 | 累計 |
|-------|-----------|--------|------|
| 1 (core基盤) | 8 | 53 | 53 |
| 2 (filter enhance) | 4 | 21 | 74 |
| 3 (filter convolve) | 5 | 24 | 98 |
| 4 (core統計/Numa) | 2 | 15 | 113 |
| **合計 (Phase 1-4)** | **19** | **113** | **113** |

## 検証方法

各コミットで以下を確認:
```bash
cargo fmt --check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo test -p <crate>
```

各PR作成後:
```bash
cargo test --workspace           # 全クレートテスト
cargo clippy --workspace -- -D warnings  # 全体lint
```

## 重要な設計パターン（既存踏襲）

- **Pix/PixMut二層モデル**: `Pix::new()` → `try_into_mut()` → 変更 → `.into()`
- **エラー型**: `thiserror`、`FilterResult<T>`パターン
- **ピクセルアクセス**: ループ内は`_unchecked`、境界は`clamp`
- **カラー操作**: `color::extract_rgba()` / `color::compose_rgba()`
- **モジュール分割**: 責務の明確さを優先（行数制限は設けない）
