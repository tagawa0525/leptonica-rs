# Core: compare.c の補助系 5 関数 (plan 032 カテゴリ F の一部)

Status: IMPLEMENTED
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ F

## 対象 C 関数 (5)

軽量・独立性の高い 5 関数。残りの photo-histo 比較チェーン
(`pixGenPhotoHistos` / `pixComparePhotoRegionsByHisto` /
`compareTilesByHisto` / `pixCompareGrayByHisto` /
`pixDecideIfPhotoImage` / `pixaComparePhotoRegionsByHisto`) は
別の plan 117 で扱う。

### Colormap

- `cmapEqual(cmap1, cmap2, ncomps) -> bool` — 同じサイズで全 entry の
  RGB(A) が一致するかチェック。サイズが違えば即 false (C は INFO ログ)
- `pixUsesCmapColor(pixs) -> bool` — colormap に色 entry があり、
  かつ画像で実際に使用されているか

### Centroid

- `pixCentroid8(pixs, factor) -> (cx, cy)` — 8 bpp の輝度を invert して
  幾何重心を計算 (subsampling factor 対応)
- `pixCropAlignedToCentroid(pix1, pix2, factor) -> (Box, Box)` — 2 枚の
  Pix の重心を揃えるクロップ範囲を計算
- `pixPadToCenterCentroid(pixs, factor) -> Pix` — 8 bpp に変換した上で
  重心がキャンバス中央に来るようにパディング

## API 設計

```rust
// in src/core/colormap/mod.rs
impl Colormap {
    /// C: `cmapEqual` (ncomps=3/4)
    pub fn equal_to(&self, other: &Colormap, include_alpha: bool) -> bool;
}

// in src/core/pix/compare.rs
impl Pix {
    /// C: `pixUsesCmapColor`
    pub fn uses_cmap_color(&self) -> bool;

    /// C: `pixCentroid8` (factor>=1, 8bpp required)
    pub fn centroid8(&self, factor: u32) -> Result<(f32, f32)>;

    /// C: `pixPadToCenterCentroid` (factor>=1)
    pub fn pad_to_center_centroid(&self, factor: u32) -> Result<Pix>;
}

/// C: `pixCropAlignedToCentroid`
pub fn pix_crop_aligned_to_centroid(
    pix1: &Pix,
    pix2: &Pix,
    factor: u32,
) -> Result<(Box, Box)>;
```

返り値はオプションパラメータを `Result<(...)>` でまとめる。C で
`*pcolor` のような output pointer を返していたものは bool/タプル化。

## 依存

- 既存 `Pix::convert_to_8`, `Pix::invert`, `Pix::gray_histogram`
- 既存 `Pix::rop_region_inplace` (C `pixRasterop` 相当)
- 既存 `Colormap::has_color`, `Colormap::get_rgba`, `Colormap::len`
- 既存 `Box::new`

## テスト方針

- 既存テストデータ (1bpp/8bpp/32bpp) で:
  - cmapEqual: 同一 cmap / サイズ違い / RGB 違い / alpha 違い
  - uses_cmap_color: cmap なし / モノクロ cmap / 色 cmap (使用) /
    色 cmap (未使用)
  - centroid8: 一様 (中央) / 単一ピクセル / 全白 (= ws/2, hs/2)
  - pad_to_center_centroid: 既に中央 (サイズ不変) / 端寄り (パディング)
  - pix_crop_aligned_to_centroid: 同サイズ / 異サイズ

## 完了条件

- [x] cargo test/clippy/fmt 通過 (17 件パス)
- [x] core.md 5 件 ❌ -> ✅
- [x] plan 032 で 112 を IMPLEMENTED に
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `Colormap::equal_to(other, include_alpha)` は単純な entry 比較
- `Pix::uses_cmap_color` は cmap entry index を直接スキャンする実装。
  C の `pixGetGrayHistogram` 経由ではなく、Rust の
  `gray_histogram_colormapped` が gray 値で集計するため、ここでは
  cmap entry index ベースで色 entry の利用を判定する
- `Pix::centroid8` は invert() + 重み付き重心計算。factor 引数は
  C 版が無視しているのを尊重しつつ、API シグネチャは保持
- `Pix::pad_to_center_centroid` は convert_to_8 -> centroid8 ->
  set_all_gray(255) -> rop_region_inplace(Src) パイプライン
- `pix_crop_aligned_to_centroid` は 2 枚の centroid8 結果から
  対応 Box を計算 (C 版とビット同一の算術)
