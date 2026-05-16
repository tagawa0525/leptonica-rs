# Core: pix_compare_gray_by_histo (plan 032 残: 117)

Status: IMPLEMENTED
作成日: 2026-05-15
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ F (117)

## 対象 C 関数 (1) + 内部 helper 2

- `pixCompareGrayByHisto(pix1, pix2, box1, box2, minratio, maxgray, factor, n)` — 2 枚のグレー画像の類似度を histogram-based に算出
- 内部: `findHistoGridDimensions` (grid 決定), `pixCompareTilesByHisto` (per-tile histo + EMD)

## API 設計

```rust
pub fn pix_compare_gray_by_histo(
    pix1: &Pix, pix2: &Pix,
    box1: Option<&Box>, box2: Option<&Box>,
    minratio: f32, maxgray: u32, factor: u32, n: i32,
) -> Result<f32>;
```

手順 (C と同じ):

1. 寸法比 < minratio → 0.0
2. box1/box2 で initial crop (optional)
3. 8 bpp 変換 + `pix_crop_aligned_to_centroid` で揃え
4. nx × ny 分割 (`find_histo_grid_dimensions`)
5. 各 tile で gray_histogram → maxgray より上を 0 → windowed_mean(5) → max を 255 に正規化 → EMD → score = max(0, 1 - 8 * dist / 255)
6. 全 tile の minimum を返す

## 依存

- 既存 `Pixa::split_pix`, `Pix::gray_histogram`, `Numa::windowed_mean`, `Numa::transform`, `Numa::earth_mover_distance`, `pix_crop_aligned_to_centroid`, `Pix::convert_to_8`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (6 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 143 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
