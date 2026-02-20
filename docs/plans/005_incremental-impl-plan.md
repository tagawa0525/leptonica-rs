# leptonica-rs 未実装関数の段階的実装計画

Status: IN_PROGRESS (Phase 1-4 大部分実装済み、残り13関数)

## Context

C版leptonicaの1,863関数に対してRust版の実装を段階的に進める計画。
依存関係・インパクト・実現可能性の順で優先度を決定し、クレートごとにサブエージェント、ファイルごとにサブサブエージェントで並列管理する。

Phase 1-4 の大部分は 1000_core-full-porting.md (Phase 10-17) および個別の計画書で実装済み。
本計画書に残る未実装関数は約20個。

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

#### 1.1 RGB→Gray変換 — ✅ 全6関数実装済み

実装場所: `crates/leptonica-core/src/pix/convert.rs:336-555`

#### 1.2 RGBコンポーネント操作 — ✅ 5/5実装済み

実装場所: `crates/leptonica-core/src/pix/rgb.rs`

| 関数 | 状態 |
|------|------|
| `pixGetRGBComponent` | ✅ `get_rgb_component` (rgb.rs:37) |
| `pixSetRGBComponent` | ✅ `set_rgb_component` (rgb.rs:443) |
| `pixCreateRGBImage` | ✅ `create_rgb_image` (rgb.rs:71) |
| `pixGetRGBPixel` | ✅ `get_rgb_pixel` (rgb.rs:340) |
| `pixSetRGBPixel` | ✅ `set_rgb_pixel` (rgb.rs:465) |

#### 1.3 カラーマップ除去 — ✅ 4/4実装済み

実装場所: `crates/leptonica-core/src/pix/convert.rs:560-778`

| 関数 | 状態 |
|------|------|
| `pixRemoveColormap` | ✅ `remove_colormap` (convert.rs:560) |
| `pixRemoveColormapGeneral` | ✅ 不要（`remove_colormap` + `RemoveColormapTarget` で同等機能をカバー） |
| `pixAddGrayColormap8` | ✅ `add_gray_colormap_8` (convert.rs:700) |
| `pixAddMinimalGrayColormap8` | ✅ `add_minimal_gray_colormap_8` (convert.rs:725) |

#### 1.4 深度変換（トップレベル） — ⚠️ 8/10実装済み、残り2関数（外部依存）

実装場所: `crates/leptonica-core/src/pix/convert.rs`

| 関数 | 状態 |
|------|------|
| `pixConvertTo1` | ❌ 未実装（二値化/閾値処理が必要） |
| `pixConvertTo1Adaptive` | ❌ 未実装（適応的閾値が必要） |
| `pixConvertTo16` | ✅ `convert_to_16` (convert.rs:779) |
| `pixConvert8To32` | ✅ `convert_8_to_32` (convert.rs:810) |
| `pixConvert8To16` | ✅ `convert_8_to_16` (convert.rs:854) |
| `pixConvert16To8` | ✅ `convert_16_to_8` (convert.rs:897) |
| `pixConvert32To8` | ✅ `convert_32_to_8` (convert.rs:1276) |
| `pixConvertTo8Or32` | ✅ `convert_to_8_or_32` (convert.rs:956) |
| `pixConvertLossless` | ✅ `convert_lossless` (convert.rs:984) |
| `pixRemoveAlpha` | ✅ `remove_alpha` (convert.rs:1041) |

#### 1.5 バイナリ展開 — ✅ 9/9実装済み

実装場所: `crates/leptonica-core/src/pix/convert.rs:1084-1202`

| 関数 | 状態 |
|------|------|
| `pixUnpackBinary` | ✅ `unpack_binary` (convert.rs:1084) |
| `pixConvert1To2` | ✅ (convert.rs:1155) |
| `pixConvert1To4` | ✅ (convert.rs:1167) |
| `pixConvert1To8` | ✅ (convert.rs:1179) |
| `pixConvert1To16` | ✅ (convert.rs:1190) |
| `pixConvert1To32` | ✅ (convert.rs:1201) |
| `pixConvert1To2Cmap` | ✅ `convert_1_to_2_cmap` (convert.rs:1210) |
| `pixConvert1To4Cmap` | ✅ `convert_1_to_4_cmap` (convert.rs:1231) |
| `pixConvert1To8Cmap` | ✅ `convert_1_to_8_cmap` (convert.rs:1252) |

#### 1.6 ボーダー操作 — ✅ 8/8実装済み

実装場所: `crates/leptonica-core/src/pix/border.rs`

| 関数 | 状態 |
|------|------|
| `pixAddBorder` | ✅ `add_border` (border.rs:45) |
| `pixAddBorderGeneral` | ✅ `add_border_general` (border.rs:86) |
| `pixRemoveBorder` | ✅ `remove_border` (border.rs:156) |
| `pixRemoveBorderGeneral` | ✅ `remove_border_general` (border.rs:196) |
| `pixAddBlackOrWhiteBorder` | ✅ `add_black_or_white_border` (border.rs:378) |
| `pixSetBorderVal` | ✅ `set_border_val` (border.rs:389) |
| `pixAddMirroredBorder` | ✅ `add_mirrored_border` (border.rs:251) |
| `pixAddRepeatedBorder` | ✅ `add_repeated_border` (border.rs:318) |

#### 1.7 マスク操作 — ✅ 全5関数実装済み

実装場所: `crates/leptonica-core/src/pix/mask.rs:26-207`

#### 1.8 ピクセルカウント・行列統計 — ✅ 全6関数実装済み

実装場所: `crates/leptonica-core/src/pix/statistics.rs:189-346`

---

### Phase 2: leptonica-filter enhance.c — ⚠️ 19/21実装済み、残り2関数

Phase 1のcore関数（特にpixconv, RGB操作, マスク）に依存。

#### 2.1 TRC基盤 — ✅ 全5関数実装済み

実装場所: `crates/leptonica-filter/src/enhance.rs:37-274`

#### 2.2 ガンマ・コントラスト・均等化 — ✅ 全6関数実装済み

実装場所: `crates/leptonica-filter/src/enhance.rs:345-457`

#### 2.3 HSV修正 — ✅ 全4関数実装済み

実装場所: `crates/leptonica-filter/src/enhance.rs:499-645`

#### 2.4 カラーシフト・行列変換 — ⚠️ 4/6実装済み、残り2関数

実装場所: `crates/leptonica-filter/src/enhance.rs:696-860`

| 関数 | 状態 |
|------|------|
| `pixColorShiftRGB` | ✅ `color_shift_rgb` (enhance.rs:696) |
| `pixMosaicColorShiftRGB` | ❌ 未実装（タイル別カラーシフト） |
| `pixDarkenGray` | ✅ `darken_gray` (enhance.rs:760) |
| `pixMultConstantColor` | ✅ `mult_constant_color` (enhance.rs:817) |
| `pixMultMatrixColor` | ✅ `mult_matrix_color` (enhance.rs:860) |
| `pixHalfEdgeByBandpass` | ❌ 未実装（バンドパスエッジ検出） |

---

### Phase 3: leptonica-filter convolve.c — ⚠️ 19/24実装済み、残り5関数

#### 3.1 ブロック畳み込み — ⚠️ 4/6実装済み、残り2関数

実装場所: `crates/leptonica-filter/src/block_conv.rs`

| 関数 | 状態 |
|------|------|
| `pixBlockconvAccum` | ✅ `blockconv_accum` (block_conv.rs:36) |
| `pixBlockconvGray` | ✅ `blockconv_gray` (block_conv.rs:91) |
| `pixBlockconv` | ✅ `blockconv` (block_conv.rs:179) |
| `pixBlockconvGrayUnnormalized` | ✅ `blockconv_gray_unnormalized` (block_conv.rs:215) |
| `pixBlockconvTiled` | ❌ 未実装（タイル化大画像対応） |
| `pixBlockconvGrayTile` | ❌ 未実装（Grayタイル化版） |

#### 3.2 ウィンドウ統計 — ✅ 全5関数実装済み

実装場所: `crates/leptonica-filter/src/windowed.rs:122-326`

#### 3.3 分離可能畳み込み — ✅ 全2関数実装済み

実装場所: `crates/leptonica-filter/src/convolve.rs:158-195`

#### 3.4 アンシャープマスク拡張 — ⚠️ 3/5実装済み、残り2関数

実装場所: `crates/leptonica-filter/src/edge.rs`

| 関数 | 状態 |
|------|------|
| `pixUnsharpMasking` | ✅ `unsharp_mask` (edge.rs:143) |
| `pixUnsharpMaskingFast` | ✅ `unsharp_masking_fast` (edge.rs:183) |
| `pixUnsharpMaskingGrayFast` | ✅ `unsharp_masking_gray_fast` (edge.rs:224) |
| `pixUnsharpMaskingGray1D` | ❌ 未実装 |
| `pixUnsharpMaskingGray2D` | ❌ 未実装 |

#### 3.5 その他 — ⚠️ 4/5実装済み、残り1関数

実装場所: `crates/leptonica-filter/src/convolve.rs`

| 関数 | 状態 |
|------|------|
| `pixCensusTransform` | ✅ `census_transform` (convolve.rs:252) |
| `pixAddGaussianNoise` | ✅ `add_gaussian_noise` (convolve.rs:300) |
| `pixConvolveWithBias` | ❌ 未実装（バイアス付き畳み込み） |
| `pixBlockrank` | ✅ `blockrank` (convolve.rs:522) |
| `pixBlocksum` | ✅ `blocksum` (convolve.rs:422) |

---

### Phase 4: leptonica-core 統計・Numa拡張 — ✅ 全関数実装済み

詳細計画: `docs/plans/400_core-numa-stats.md` (IMPLEMENTED)

#### 4.1 ヒストグラム拡張 — ✅ 全7関数実装済み

実装場所: `crates/leptonica-core/src/pix/histogram.rs`, `statistics.rs`

#### 4.2 Numa高度操作 — ✅ 全8関数実装済み

実装場所: `crates/leptonica-core/src/numa/operations.rs`

---

### Phase 5以降: 後続クレート

Phase 1-4完了後に計画策定:

- **Phase 5**: leptonica-filter adaptmap.c拡張（背景マップ、モルフォロジーベース正規化）
- **Phase 6**: leptonica-filter bilateral.c（高速バイラテラル）
- **Phase 7**: leptonica-core pix5.c（クリップ、測定）、boxfunc拡張
- **Phase 8**: leptonica-color 拡張（画像レベル変換、色分析、量子化拡張）
- **Phase 9+**: leptonica-region, leptonica-morph, leptonica-recog 拡張

## 対象関数数サマリー

| Phase | 総関数数 | 実装済み | 未実装 | 進捗率 |
|-------|---------|---------|--------|--------|
| 1 (core基盤) | 53 | 51 | 2 | 96% |
| 2 (filter enhance) | 21 | 19 | 2 | 90% |
| 3 (filter convolve) | 24 | 19 | 5 | 79% |
| 4 (core統計/Numa) | 15 | 15 | 0 | 100% |
| **合計 (Phase 1-4)** | **113** | **104** | **9** | **92%** |

### 残り未実装関数一覧

**Phase 1 (2個、外部依存)**:
- `pixConvertTo1` (1.4) — filter crateの二値化/閾値処理が必要
- `pixConvertTo1Adaptive` (1.4) — filter crateの適応的閾値が必要

**Phase 2 (2個)**:
- `pixMosaicColorShiftRGB` (2.4) — BMF/Pixa display/scaling依存
- `pixHalfEdgeByBandpass` (2.4)

**Phase 3 (5個)**:
- `pixBlockconvTiled`, `pixBlockconvGrayTile` (3.1)
- `pixUnsharpMaskingGray1D`, `pixUnsharpMaskingGray2D` (3.4)
- `pixConvolveWithBias` (3.5)

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
