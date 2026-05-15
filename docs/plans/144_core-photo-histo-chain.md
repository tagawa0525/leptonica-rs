# Core: photo-histo chain 4 関数 (plan 032 残: 117 完全解消)

Status: IMPLEMENTED
作成日: 2026-05-15
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ F (117)

## 対象 C 関数 (4)

117 の残り 4 関数を一気に移植。**117 完全解消**。

- `pixDecideIfPhotoImage(pix, factor, thresh, n, &naa, pixadebug)` —
  画像が photo か line-art/text かを histogram variance ratio で判定
- `pixGenPhotoHistos(pixs, box, factor, thresh, n, &naa, &w, &h, dbg)` —
  centroid 揃え + light-gray reset + photo 判定
- `pixComparePhotoRegionsByHisto(pix1, pix2, box1, box2, minratio,
  factor, n, &score, dbg)` — pix_gen_photo_histos × 2 + compare
- `pixaComparePhotoRegionsByHisto(pixa, minratio, textthresh, factor, n,
  simthresh, &nai, &scores, &ppixd, debug)` — Pixa 全要素の pairwise
  比較とクラスタリング

## API 設計

```rust
// 判定 (簡素化版: pixDecideIfText の事前チェックは省略)
pub fn pix_decide_if_photo_image(
    pix: &Pix, factor: u32, thresh: f32, n: i32,
) -> Result<(Option<Numaa>, bool)>;

// 単一画像のヒストグラム生成
pub fn pix_gen_photo_histos(
    pixs: &Pix, box: Option<&Box>, factor: u32, thresh: f32, n: i32,
) -> Result<(Option<Numaa>, u32, u32)>;

// 2 画像比較
pub fn pix_compare_photo_regions_by_histo(
    pix1: &Pix, pix2: &Pix, box1: Option<&Box>, box2: Option<&Box>,
    minratio: f32, factor: u32, n: i32,
) -> Result<f32>;

// N 画像 pairwise + クラスタリング
pub fn pixa_compare_photo_regions_by_histo(
    pixa: &Pixa, minratio: f32, factor: u32, n: i32, simthresh: f32,
) -> Result<(Vec<i32>, Vec<f32>)>;  // (class_indices, n×n score matrix)
```

## C 仕様との差分

- `pixDecideIfPhotoImage` は元コードで `pixDecideIfText` を呼ぶが、
  これは recog の pageseg.c の未実装関数 (`pixDecideIfText`) に依存。
  本実装ではこのチェックを省略し、純粋に histogram variance ratio
  のみで photo 判定を行う。テキスト画像を事前に弾きたい呼び出し側は
  自前で実装する必要がある
- `pixadebug` (gplot/PDF 出力) はすべて省略
- `pixaComparePhotoRegionsByHisto` の `ppixd` (比較画像の rendered
  composite) は省略 — クラスタ ID 配列から呼び出し側で構築可能

## 依存

- 既存 `Pixa::split_pix`, `Pix::gray_histogram`, `Pix::convert_to_8`,
  `Pix::remove_colormap`, `Pix::pad_to_center_centroid`
- 既存 `Numa::windowed_mean`, `transform`, `set`,
  `sum_on_interval`, `earth_mover_distance`,
  `gray_inter_histogram_stats`
- plan 142 `compare_tiles_by_histo`, plan 143 内部 helper
  `find_histo_grid_dimensions`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (13 件パス)
- [x] core.md 4 件 ❌ → ✅
- [x] plan 032 で 144 を新規 IMPLEMENTED 行として追加、**117 完全解消**
- [ ] PR + Copilot レビュー対応 + マージ
