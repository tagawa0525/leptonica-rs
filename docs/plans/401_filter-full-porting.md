# leptonica-filter 全未実装関数の移植計画

Status: PLANNED

## Context

leptonica-filter crateはconvolution、bilateral filter、rank filter、adaptive mapping、
enhance等のコア機能を高いカバレッジで実装済みだが、以下の機能が欠落している:

1. **FPix畳み込み** - 浮動小数点カーネルによるFPix畳み込みが未実装
2. **Tiled block畳み込み** - 大画像向けタイル分割畳み込みが未実装
3. **Adaptmap拡張** - 前景マップ、閾値スプレッド正規化、フレキシブル正規化等が未実装
4. **Block bilateral** - ブロックベースの正確bilateral filterが未実装
5. **Edge拡張** - two-sided edge filter等が未実装

### 現状の実装状況

| モジュール | 実装済み / C版 | カバレッジ |
|-----------|-------------|---------|
| convolve.rs | 19/23 | 83% |
| adaptmap.rs | 22/27 | 81% |
| bilateral.rs | 5/6 | 83% |
| enhance.rs | 19/26 | 73% |
| edge.rs | 1/6 | 17% |
| rank.rs | 4/5 | 80% |
| block_conv.rs | 4 | 完了 |
| windowed.rs | 6 | 完了 |
| kernel.rs | 11 | 完了 |

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `l_setConvolveSampling` | グローバル状態設定（オプション構造体で対応済み） |
| `gaussDistribSampling` | 統計ユーティリティ（randクレートで対応可） |
| `numaGammaTRC`, `numaContrastTRC`, `numaEqualizeTRC` | NUMA返却版（Rustでは配列を直接返す） |
| `pixUnsharpMaskingGray1D`, `pixUnsharpMaskingGray2D` | 内部実装の分離（Rustでは統合実装） |
| `pixMosaicColorShiftRGB` | モザイク効果（ニッチな用途） |
| `pixHalfEdgeByBandpass` | バンドパスエッジ検出（ニッチ） |
| `pixGetLastOffPixelInRun`, `pixGetLastOnPixelInRun` | ピクセルラン走査ユーティリティ（region crateの領域） |
| `pixRankFilterWithScaling` | scaling付きrank filter（scale + rank_filterの組み合わせで代替可） |
| `pixTwoSidedEdgeFilter` | two-sided edge（ニッチで利用頻度低） |
| `pixMeasureEdgeSmoothness`, `pixGetEdgeProfile` | エッジプロファイル（ニッチ） |

---

## 実行順序

Phase 1 → 2 → 3 → 4 の順に直列で実行する。

```
Phase 1 (FPix畳み込み) ← FPixへの直接畳み込みサポート
  → Phase 2 (Tiled block畳み込み) ← 大画像への対応
    → Phase 3 (Adaptmap拡張) ← 正規化機能拡充
      → Phase 4 (Block bilateral + 追加)
```

---

## Phase 1: FPix畳み込み（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/convolve.c` L1600-1850

### 実装内容

- `fpix_convolve(fpix: &FPix, kernel: &Kernel) -> FilterResult<FPix>` - 浮動小数点画像にカーネル畳み込み
- `fpix_convolve_sep(fpix: &FPix, kx: &Kernel, ky: &Kernel) -> FilterResult<FPix>` - 分離可能カーネルによる2パス畳み込み
- `convolve_with_bias(pix: &Pix, kernel1: &Kernel, kernel2: &Kernel, bias: u32) -> FilterResult<Pix>` - バイアス付き畳み込み（結果が非負になるよう保証）

### 動作

`fpix_convolve`: FPixの各ピクセルに対してカーネルを適用し、
浮動小数点精度で結果を計算する。整数丸め誤差を避けたい場合に使用。

`convolve_with_bias`: 2つのカーネル（正と負のコンポーネント）を別々に適用し、
バイアスを加えて非負の結果を保証する。エッジ検出等で使用。

### 修正ファイル

- `crates/leptonica-filter/src/convolve.rs`: 上記3関数追加

### テスト

- fpix_convolve: ボックスカーネルでの平均化結果検証
- fpix_convolve_sep: 分離カーネルと非分離カーネルの結果一致
- convolve_with_bias: バイアス付きの結果が全て非負であること

---

## Phase 2: Tiled block畳み込み（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/convolve.c` L280-400

### 実装内容

- `blockconv_tiled(pix: &Pix, wc: u32, hc: u32, tile_w: u32, tile_h: u32) -> FilterResult<Pix>` - タイル分割block畳み込み
- `blockconv_gray_tile(pix: &Pix, tile: &Pix, wc: u32, hc: u32) -> FilterResult<Pix>` - 個別タイルのblock畳み込み

### 動作

大画像を小さなタイルに分割し、各タイルの境界にオーバーラップ領域を
設けて個別に畳み込みを実行する。メモリ使用量を制限しつつ、
タイル境界でのアーティファクトを防止する。

### 修正ファイル

- `crates/leptonica-filter/src/convolve.rs`: 上記2関数追加

### テスト

- タイル版と非タイル版の結果一致確認
- 大画像（5000x5000等）でのメモリ効率検証
- タイル境界でのシームレス性確認

---

## Phase 3: Adaptmap拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/adaptmap.c` L1600-2100

### 実装内容

- `get_foreground_gray_map(pix: &Pix, options: &ForegroundMapOptions) -> FilterResult<Pix>` - 前景マップ計算
- `threshold_spread_norm(pix: &Pix, filter_type: EdgeFilterType, edge_thresh: u8, smooth_x: u32, smooth_y: u32, thresh_norm: f32) -> FilterResult<Pix>` - 適応的閾値スプレッド正規化
- `background_norm_flex(pix: &Pix, options: &FlexNormOptions) -> FilterResult<Pix>` - フレキシブル背景正規化
- `smooth_connected_regions(pix: &Pix, connectivity: u8, factor: u32) -> FilterResult<Pix>` - 連結領域スムージング
- `background_norm_to_1_min_max(pix: &Pix, contrast: u32) -> FilterResult<Pix>` - MinMax背景正規化

```rust
pub struct ForegroundMapOptions {
    pub reduction: u32,      // 縮小係数（default: 2）
    pub smooth_size: u32,    // スムージングサイズ
    pub thresh: u8,          // 前景/背景閾値
}

pub struct FlexNormOptions {
    pub tile_width: u32,
    pub tile_height: u32,
    pub smooth_x: u32,
    pub smooth_y: u32,
    pub delta: u32,
}
```

### 修正ファイル

- `crates/leptonica-filter/src/adaptmap.rs`: 上記関数・構造体追加

### テスト

- get_foreground_gray_map: テキスト画像での前景マップ生成
- threshold_spread_norm: 不均一照明画像の正規化
- smooth_connected_regions: 接続領域のスムージング効果確認
- テスト画像: 不均一照明のある文書画像

---

## Phase 4: Block bilateral + 追加（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/bilateral.c` L380-500, `adaptmap.c` L2200-2350

### 実装内容

- `block_bilateral_exact(pix: &Pix, spatial_stdev: f32, range_stdev: f32, block_size: u32, ncomps: u32) -> FilterResult<Pix>` - ブロックベースの正確bilateral filter
- `global_norm_no_sat_rgb(pix: &Pix, rval: i32, gval: i32, bval: i32, factor: u32, rank: f32) -> FilterResult<Pix>` - 飽和なしグローバルRGB正規化
- `unsharp_masking(pix: &Pix, half_width: u32, fract: f32) -> FilterResult<Pix>` - 標準unsharp masking（Fast版ではない精密版）
- `unsharp_masking_gray(pix: &Pix, half_width: u32, fract: f32) -> FilterResult<Pix>` - グレースケール精密unsharp masking

### 動作

`block_bilateral_exact`: 画像をブロックに分割し、各ブロック内で
正確なbilateral filterを適用する。完全なbilateral_exactよりも
高速だが、近似bilateral filterよりも高品質。

### 修正ファイル

- `crates/leptonica-filter/src/bilateral.rs`: `block_bilateral_exact` 追加
- `crates/leptonica-filter/src/adaptmap.rs`: `global_norm_no_sat_rgb` 追加
- `crates/leptonica-filter/src/enhance.rs`: `unsharp_masking`, `unsharp_masking_gray` 追加

### テスト

- block_bilateral_exact: 通常bilateral_exactとの品質比較
- global_norm_no_sat_rgb: 正規化後の飽和ピクセル確認
- unsharp_masking: Fast版との結果比較（精密版は品質高い）

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | FPix畳み込み | 1 | 3 |
| 2 | Tiled block畳み込み | 1 | 2 |
| 3 | Adaptmap拡張 | 1 | 5 |
| 4 | Block bilateral + 追加 | 1 | 4 |
| **合計** | | **4** | **14** |

C版の約127関数（filter scope内89関数）のうち:
- 既存実装でカバー済み: ~70関数
- 本計画で追加: ~14関数
- スコープ除外: ~12関数（NUMA版、グローバル状態、ニッチ機能）

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
└── feat/filter-fpix-convolve    ← Phase 1
└── feat/filter-tiled-conv       ← Phase 2
└── feat/filter-adaptmap-ext     ← Phase 3
└── feat/filter-bilateral-ext    ← Phase 4
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-filter
cargo clippy -p leptonica-filter -- -D warnings
cargo test -p leptonica-filter
cargo test --workspace  # PR前に全ワークスペーステスト
```
