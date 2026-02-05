# Adaptive Mapping Implementation Plan

## Status

- [x] Planning
- [x] Implementation
- [x] Testing
- [ ] Review

## Overview

適応マッピング（局所的なコントラスト調整）を `leptonica-filter` クレートに実装する。
C版の `adaptmap.c` を参考に、不均一な照明の補正や背景正規化を実装する。

## Reference

- C source: `reference/leptonica/src/adaptmap.c`
- Key functions:
  - `pixBackgroundNorm()` - 背景正規化（タイルベース）
  - `pixBackgroundNormSimple()` - デフォルトパラメータでの簡易版
  - `pixContrastNorm()` - 局所的コントラスト正規化
  - `pixGetBackgroundGrayMap()` - 背景マップ生成
  - `pixGetInvBackgroundMap()` - 逆背景マップ生成
  - `pixApplyInvBackgroundGrayMap()` - マップ適用

## Algorithm

### Background Normalization

背景正規化は3つのステップで行われる:

1. **背景マップ生成**: タイルベースで背景値を推定
   - 前景ピクセル（閾値以下）を除外
   - 各タイルで残りのピクセルの平均値を計算
   - タイル内のピクセル数がmincount未満の場合は隣接タイルから補間

2. **逆背景マップ生成**: マップを反転して乗算係数を計算
   - 各ピクセルの乗算係数 = (bgval * 256) / map_value
   - スムージング（オプション）

3. **マップ適用**: 各ピクセルに係数を適用
   - 出力値 = (入力値 * 係数) / 256
   - クランプ処理（0-255）

### Contrast Normalization

タイルベースの局所コントラスト正規化:

1. **Min/Max取得**: 各タイルの最小・最大値を計算
2. **低コントラスト除去**: min-max差がmindiff未満のタイルを除外
3. **穴埋め**: 除外されたタイルの値を隣接から補間
4. **スムージング**: ブロック畳み込みで滑らかに
5. **線形TRC適用**: 各タイルでmin→0、max→255にマッピング

## API Design

```rust
/// Default parameters for background normalization
pub const DEFAULT_TILE_WIDTH: u32 = 10;
pub const DEFAULT_TILE_HEIGHT: u32 = 15;
pub const DEFAULT_FG_THRESHOLD: u32 = 60;
pub const DEFAULT_MIN_COUNT: u32 = 40;
pub const DEFAULT_BG_VAL: u32 = 200;
pub const DEFAULT_SMOOTH_X: u32 = 2;
pub const DEFAULT_SMOOTH_Y: u32 = 1;

/// Background normalization options
pub struct BackgroundNormOptions {
    pub tile_width: u32,
    pub tile_height: u32,
    pub fg_threshold: u32,
    pub min_count: u32,
    pub bg_val: u32,
    pub smooth_x: u32,
    pub smooth_y: u32,
}

/// Normalize background using adaptive mapping (simplified API)
pub fn background_norm_simple(pix: &Pix) -> FilterResult<Pix>;

/// Normalize background with full parameter control
pub fn background_norm(pix: &Pix, options: &BackgroundNormOptions) -> FilterResult<Pix>;

/// Internal: Get background map for grayscale image
fn get_background_gray_map(
    pix: &Pix,
    tile_width: u32,
    tile_height: u32,
    fg_threshold: u32,
    min_count: u32,
) -> FilterResult<Pix>;

/// Internal: Generate inverted background map
fn get_inv_background_map(
    pixm: &Pix,
    bg_val: u32,
    smooth_x: u32,
    smooth_y: u32,
) -> FilterResult<Pix>;

/// Internal: Apply inverted background map
fn apply_inv_background_gray_map(
    pix: &Pix,
    pixm: &Pix,
    tile_width: u32,
    tile_height: u32,
) -> FilterResult<Pix>;

/// Contrast normalization options
pub struct ContrastNormOptions {
    pub tile_width: u32,
    pub tile_height: u32,
    pub min_diff: u32,
    pub smooth_x: u32,
    pub smooth_y: u32,
}

/// Local contrast normalization
pub fn contrast_norm(pix: &Pix, options: &ContrastNormOptions) -> FilterResult<Pix>;

/// Simplified contrast normalization with default parameters
pub fn contrast_norm_simple(pix: &Pix) -> FilterResult<Pix>;
```

## Implementation Plan

### Phase 1: Core infrastructure

1. 新規ファイル `crates/leptonica-filter/src/adaptmap.rs` を作成
2. オプション構造体とデフォルト定数を定義
3. `lib.rs` にモジュールを追加

### Phase 2: Background map generation

1. `get_background_gray_map()` を実装
   - タイル単位で背景値を計算
   - 前景マスク生成（閾値二値化）
   - 穴埋め処理

2. `get_background_rgb_map()` を実装
   - カラー画像用（R, G, B各チャンネル）

### Phase 3: Inverted background map

1. `get_inv_background_map()` を実装
   - スムージング（ブロック畳み込み）
   - 逆マップ計算

### Phase 4: Map application

1. `apply_inv_background_gray_map()` を実装
   - タイル単位で係数を適用
   - 境界処理

2. `apply_inv_background_rgb_map()` を実装
   - カラー画像用

### Phase 5: Top-level API

1. `background_norm_simple()` を実装
2. `background_norm()` を実装
3. デフォルトパラメータの設定

### Phase 6: Contrast normalization

1. `min_max_tiles()` - タイルごとのmin/max取得
2. `set_low_contrast()` - 低コントラストタイルの除去
3. `linear_trc_tiled()` - タイル単位の線形TRC適用
4. `contrast_norm()` を実装

### Phase 7: Testing

1. 単体テスト
   - パラメータ検証テスト
   - 背景正規化テスト（グレースケール）
   - 背景正規化テスト（カラー）
   - コントラスト正規化テスト

## Boundary Handling

タイル境界では、C版と同様に隣接タイルからの補間を行う。
画像端の不完全なタイルは、隣接値のレプリケーションで補完する。

## Performance Considerations

- タイル処理により計算量を削減
- マップのスムージングはオプション
- 16bpp中間マップを使用してダイナミックレンジを維持

## Files to Modify

1. `crates/leptonica-filter/src/adaptmap.rs` - 新規作成
2. `crates/leptonica-filter/src/lib.rs` - モジュール追加

## Questions

なし

## Notes

- C版には多くの派生関数があるが、最も重要な`pixBackgroundNorm`と`pixContrastNorm`に集中する
- モルフォロジーベースの背景推定（`pixBackgroundNormMorph`）は別途実装を検討
- マスク（pixim）サポートは初期実装では省略し、将来の拡張とする
