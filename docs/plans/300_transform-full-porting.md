# leptonica-transform 全未実装関数の移植計画

Status: COMPLETED

## Context

leptonica-transform crateは基本的なaffine/bilinear/projective変換、回転、シアー、ワーピングを
実装済みだが、C版leptonicaのtransform関数群（約130+関数）に対して以下の重要な機能が欠落している:

1. **Alpha付き変換** - affine/bilinear/projectiveのWithAlpha版が未実装
2. **Scale拡張** - LI(Linear Interpolation)、ToGray、Mipmap、MinMax/Rank等の高度なスケーリング
3. **Rotation拡張** - Corner/Center/IP系のエントリポイント、WithAlpha
4. **Flip検出** - ページ方向検出・ミラー検出が完全未実装
5. **PTA/BOXA変換** - 点群・矩形群への幾何変換適用が未実装

### 現状の実装状況

| モジュール | 実装済み関数数 | 状態 |
|-----------|-------------|------|
| scale.rs | 4 | 基本のみ（sampling, to_size） |
| affine.rs | 9 + AffineMatrix型 | コア実装済み、WithAlpha未対応 |
| bilinear.rs | 4 + BilinearCoeffs型 | コア実装済み、WithAlpha未対応 |
| projective.rs | 4 + ProjectiveCoeffs型 | コア実装済み、WithAlpha未対応 |
| rotate.rs | 15 + 型 | 基本回転・フリップ済み、Corner/IP未対応 |
| shear.rs | 10 | 完了 |
| warper.rs | 11 | 完了 |

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `pixScaleRGBToGrayFast`, `pixScaleRGBToBinaryFast`, `pixScaleGrayToBinaryFast` | 深度変換 + scale の組み合わせで対応可能 |
| `pixRotateAMColorFast`, `rotateAMColorFastLow` | 精度が低い近似実装、通常のAreaMapで十分 |
| `l_productMat*` | 汎用行列乗算ユーティリティ（nalgebra等を使えば良い） |
| `pixAffineSequential` | 行列合成（AffineMatrix::compose）で対応可能 |
| `pixDebugFlipDetect` | デバッグ可視化専用 |
| `*Low` 接尾辞の関数群 | C固有の低レベル実装（Rust版は内部実装で対応） |
| `makeReverseByteTab1/2/4` | Cルックアップテーブル生成（Rustでは不要） |
| `pixScaleRGBToGray2` | `pixScaleToGray2` + RGB→Gray変換で代替可 |

---

## 実行順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 の順に直列で実行する。

```
Phase 1 (Alpha変換) ← 他の変換と組み合わせで使用頻度高
  → Phase 2 (PTA/BOXA変換)
    → Phase 3 (Scale拡張 - 基本)
      → Phase 4 (Scale拡張 - 1bpp→8bpp)
        → Phase 5 (Scale拡張 - 特殊)
          → Phase 6 (Rotation拡張)
            → Phase 7 (Flip検出)
```

---

## Phase 1: Alpha付き変換（1 PR）

**Status: IMPLEMENTED** (PR: 既存実装、計画策定前に完了済み)

**C参照**: `reference/leptonica/src/affine.c` L780-870, `bilinear.c` L580-660, `projective.c` L580-660

### 実装内容

- `affine_pta_with_alpha(pix: &Pix, pta_src: &Pta, pta_dst: &Pta, fill: &Pix) -> TransformResult<Pix>`
- `bilinear_pta_with_alpha(pix: &Pix, pta_src: &Pta, pta_dst: &Pta, fill: &Pix) -> TransformResult<Pix>`
- `projective_pta_with_alpha(pix: &Pix, pta_src: &Pta, pta_dst: &Pta, fill: &Pix) -> TransformResult<Pix>`

### 動作

1. 入力画像のアルファチャンネルを分離
2. RGB部分に対して変換を適用
3. アルファチャンネルに対しても同じ変換を適用
4. 変換結果にアルファを再合成
5. アルファがない場合は通常の変換にフォールバック

### 修正ファイル

- `crates/leptonica-transform/src/affine.rs`: `affine_pta_with_alpha` 追加
- `crates/leptonica-transform/src/bilinear.rs`: `bilinear_pta_with_alpha` 追加
- `crates/leptonica-transform/src/projective.rs`: `projective_pta_with_alpha` 追加

### テスト

- 32bpp RGBA画像のaffine/bilinear/projective WithAlpha変換
- アルファなし画像でのフォールバック動作確認
- テスト画像: `test32.png` 等のRGBA画像

---

## Phase 2: PTA/BOXA変換ユーティリティ（1 PR）

**Status: IMPLEMENTED** (PR: #150)

**C参照**: `reference/leptonica/src/affine.c` (ptaXform系), `pta.c`, `boxa2.c`

### 実装内容

PTA（点群）への変換適用:
- `Pta::translate(dx, dy) -> Pta`
- `Pta::scale(sx, sy) -> Pta`
- `Pta::rotate(cx, cy, angle) -> Pta`
- `Pta::affine_transform(matrix: &AffineMatrix) -> Pta`

BOXA（矩形群）への変換適用:
- `Boxa::translate(dx, dy) -> Boxa`
- `Boxa::scale(sx, sy) -> Boxa`
- `Boxa::rotate(cx, cy, angle) -> Boxa`
- `Boxa::affine_transform(matrix: &AffineMatrix) -> Boxa`

### 修正ファイル

- `crates/leptonica-core/src/pta.rs`: 変換メソッド追加
- `crates/leptonica-core/src/boxa.rs`: 変換メソッド追加

### テスト

- PTA translate/scale/rotate の座標検証
- BOXA translate/scale/rotate の矩形座標検証
- affine_transform の行列変換検証
- 空のPTA/BOXAに対するエッジケース

---

## Phase 3: Scale拡張 - 基本（1 PR）

**Status: IMPLEMENTED** (PR: 既存実装、計画策定前に完了済み)

**C参照**: `reference/leptonica/src/scale1.c` L70-700

### 実装内容

- `scale_li(pix, sx, sy) -> TransformResult<Pix>` - Linear Interpolation スケーリング（8bpp/32bpp）
- `scale_color_li(pix, sx, sy) -> TransformResult<Pix>` - カラー専用LI
- `scale_gray_li(pix, sx, sy) -> TransformResult<Pix>` - グレースケール専用LI
- `scale_general(pix, sx, sy, sharpfract, sharpwidth) -> TransformResult<Pix>` - 汎用スケーリング（シャープニング付き）
- `scale_to_resolution(pix, xres, yres) -> TransformResult<Pix>` - 解像度指定スケーリング
- `scale_by_sampling_with_shift(pix, sx, sy, hshift, vshift) -> TransformResult<Pix>` - シフト付きサンプリング
- `scale_by_int_sampling(pix, factor) -> TransformResult<Pix>` - 整数倍サンプリング
- `scale_smooth(pix, sx, sy) -> TransformResult<Pix>` - スムーズスケーリング（低パスフィルタ + 縮小）

### 修正ファイル

- `crates/leptonica-transform/src/scale.rs`: 上記関数追加

### テスト

- LI拡大/縮小のラウンドトリップ精度検証
- scale_general のシャープニング効果確認
- scale_smooth と通常scaleの品質比較
- 各深度（1bpp, 8bpp, 32bpp）での動作確認

---

## Phase 4: Scale拡張 - 1bpp→8bpp変換（1 PR）

**Status: IMPLEMENTED** (PR: 既存実装、計画策定前に完了済み)

**C参照**: `reference/leptonica/src/scale2.c` L70-600

### 実装内容

- `scale_to_gray(pix, scale) -> TransformResult<Pix>` - 1bpp→8bpp 汎用縮小
- `scale_to_gray_2(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/2縮小
- `scale_to_gray_3(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/3縮小
- `scale_to_gray_4(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/4縮小
- `scale_to_gray_6(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/6縮小
- `scale_to_gray_8(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/8縮小
- `scale_to_gray_16(pix) -> TransformResult<Pix>` - 1bpp→8bpp 1/16縮小
- `scale_to_gray_fast(pix, scale) -> TransformResult<Pix>` - 高速版（サンプリングベース）
- `expand_replicate(pix, factor) -> TransformResult<Pix>` - ピクセル複製による拡大
- `scale_binary(pix, sx, sy) -> TransformResult<Pix>` - バイナリ画像専用スケーリング

### 動作原理

`scale_to_gray_N` は N×N ブロック内の白ピクセル数をカウントし、
0-255のグレー値にマッピングする。これにより1bpp文書画像を
アンチエイリアス付きグレースケールに変換できる。

### 修正ファイル

- `crates/leptonica-transform/src/scale.rs`: 上記関数追加

### テスト

- 1bpp画像の各scale_to_gray_N 変換結果の寸法・深度検証
- expand_replicate の各倍率テスト
- scale_binary のサイズ精度検証
- テスト画像: `feyn.tif` 等の1bpp文書画像

---

## Phase 5: Scale拡張 - 特殊（1 PR）

**Status: IMPLEMENTED** (PR: 既存実装、計画策定前に完了済み)

**C参照**: `reference/leptonica/src/scale1.c` L1800-2500, `scale2.c` L600-1200

### 実装内容

- `scale_color_2x_li(pix) -> TransformResult<Pix>` - カラー2倍拡大（LI）
- `scale_color_4x_li(pix) -> TransformResult<Pix>` - カラー4倍拡大（LI）
- `scale_gray_2x_li(pix) -> TransformResult<Pix>` - グレースケール2倍拡大（LI）
- `scale_gray_4x_li(pix) -> TransformResult<Pix>` - グレースケール4倍拡大（LI）
- `scale_gray_2x_li_thresh(pix, thresh) -> TransformResult<Pix>` - 2倍拡大+閾値→1bpp
- `scale_gray_4x_li_thresh(pix, thresh) -> TransformResult<Pix>` - 4倍拡大+閾値→1bpp
- `scale_gray_2x_li_dither(pix) -> TransformResult<Pix>` - 2倍拡大+ディザリング→1bpp
- `scale_gray_4x_li_dither(pix) -> TransformResult<Pix>` - 4倍拡大+ディザリング→1bpp
- `scale_gray_min_max(pix, mode) -> TransformResult<Pix>` - Min/Max 2x縮小
- `scale_gray_rank_cascade(pix, ranks...) -> TransformResult<Pix>` - Rank値カスケード縮小
- `scale_to_gray_mipmap(pix, scale) -> TransformResult<Pix>` - Mipmapベース縮小

### 修正ファイル

- `crates/leptonica-transform/src/scale.rs`: 上記関数追加

### テスト

- 2x/4x LI拡大の品質検証（隣接ピクセル補間の正確性）
- threshold/dither版の出力深度確認（1bpp）
- MinMax/Rank縮小の結果検証
- Mipmapスケーリングの精度テスト

---

## Phase 6: Rotation拡張（1 PR）

**Status: IMPLEMENTED** (PR: 既存実装、計画策定前に完了済み)

**C参照**: `reference/leptonica/src/rotateam.c`, `rotateshear.c`, `rotate.c`

### 実装内容

Area-map Corner系:
- `rotate_am_corner(pix, angle) -> TransformResult<Pix>` - 左上角中心のArea-map回転
- `rotate_am_color_corner(pix, angle, fill) -> TransformResult<Pix>` - カラー版
- `rotate_am_gray_corner(pix, angle, fill) -> TransformResult<Pix>` - グレースケール版

Shear回転の追加エントリポイント:
- `rotate_shear(pix, cx, cy, angle) -> TransformResult<Pix>` - 任意中心シアー回転
- `rotate_shear_ip(pix, cx, cy, angle) -> TransformResult<PixMut>` - in-place版
- `rotate_shear_center(pix, angle) -> TransformResult<Pix>` - 中心シアー回転
- `rotate_shear_center_ip(pix, angle) -> TransformResult<PixMut>` - in-place版

Alpha付き回転:
- `rotate_with_alpha(pix, angle, fill: &Pix) -> TransformResult<Pix>` - Alpha付き回転

### 修正ファイル

- `crates/leptonica-transform/src/rotate.rs`: 上記関数追加

### テスト

- Corner回転とCenter回転の結果差異検証
- in-place回転の正確性
- WithAlpha回転のアルファチャンネル保持確認
- 各回転メソッドでの0度/90度/180度の特殊ケース

---

## Phase 7: Flip検出（1 PR）

**Status: IMPLEMENTED** (crates/leptonica-recog/src/flipdetect.rs に実装済み、計画策定前に完了)

**C参照**: `reference/leptonica/src/flipdetect.c` 全1400行

### 実装内容

- `orient_detect(pix, up_conf, left_conf, min_count, debug) -> TransformResult<OrientDetectResult>` - ページ方向検出
- `orient_correct(pix, min_up_conf, min_ratio, up_conf, left_conf) -> TransformResult<Pix>` - 方向自動修正
- `mirror_detect(pix, conf) -> TransformResult<MirrorDetectResult>` - 左右反転検出

```rust
pub struct OrientDetectResult {
    pub up_confidence: f32,
    pub left_confidence: f32,
}

pub struct MirrorDetectResult {
    pub confidence: f32,
    pub is_mirrored: bool,
}
```

### 動作原理

テキスト画像の統計的特徴（アセンダー/ディセンダー比率、
文字のストローク方向）を解析してページの上下・左右を判定する。

### 修正ファイル

- `crates/leptonica-transform/src/flipdetect.rs`（新規）
- `crates/leptonica-transform/src/lib.rs`: `pub mod flipdetect` 追加

### テスト

- 正立テキスト画像の検出（高confidence）
- 180度回転画像の検出
- 左右反転画像のミラー検出
- 非テキスト画像での低confidence確認

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Alpha付き変換 | 1 | 3 |
| 2 | PTA/BOXA変換 | 1 | 8 |
| 3 | Scale拡張 - 基本 | 1 | ~8 |
| 4 | Scale拡張 - 1bpp→8bpp | 1 | ~10 |
| 5 | Scale拡張 - 特殊 | 1 | ~11 |
| 6 | Rotation拡張 | 1 | ~8 |
| 7 | Flip検出 | 1 | 3 |
| **合計** | | **7** | **~51** |

## 共通ワークフロー

### TDD

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### PRワークフロー

1. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
2. `/gh-pr-create` でPR作成
3. `/gh-actions-check` でCopilotレビュー到着を確認
4. `/gh-pr-review` でレビューコメント対応
5. CIパス確認後 `/gh-pr-merge --merge` でマージ
6. ブランチ削除

### ブランチ命名

```
main
└── feat/transform-alpha         ← Phase 1
└── feat/transform-pta-boxa      ← Phase 2
└── feat/transform-scale-basic   ← Phase 3
└── feat/transform-scale-togray  ← Phase 4
└── feat/transform-scale-special ← Phase 5
└── feat/transform-rotate-ext    ← Phase 6
└── feat/transform-flipdetect    ← Phase 7
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-transform
cargo clippy -p leptonica-transform -- -D warnings
cargo test -p leptonica-transform
cargo test --workspace  # PR前に全ワークスペーステスト
```
