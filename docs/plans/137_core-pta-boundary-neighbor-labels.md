# Core: Pta boundary / neighbor / labeled-pixels 3 関数 (plan 032 残: 111b)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ E (111b)

## 対象 C 関数 (3)

`ptafunc1.c` の Pta/Ptaa 残課題 (111b) のうち、依存が薄く独立性の
高い 3 関数を切り出して移植。

- `ptaGetBoundaryPixels(pixs, type) -> Pta` — erode/dilate + XOR で
  fg/bg 境界ピクセル座標を抽出
- `ptaGetNeighborPixLocs(pixs, x, y, conn) -> Pta` — `(x, y)` の
  4/8-連結近傍座標を列挙 (画像外は除く)
- `ptaaIndexLabeledPixels(pixs) -> (Ptaa, max_label)` — 32 bpp の
  ラベル付き画像をラベルごとに Pta バケットへ振り分ける

## API 設計

```rust
pub enum BoundaryType { Foreground, Background }

pub fn pta_get_boundary_pixels(pixs: &Pix, btype: BoundaryType) -> Result<Pta>;
pub fn pta_get_neighbor_pix_locs(pixs: &Pix, x: i32, y: i32, conn: u32) -> Result<Pta>;
pub fn ptaa_index_labeled_pixels(pixs: &Pix) -> Result<(Ptaa, u32)>;
```

## 依存

- 既存 `morph::sequence::morph_sequence` ("e3.3" / "d3.3")
- 既存 `Pix::xor`
- 既存 `pta_get_pixels_from_pix` (plan 111)
- 既存 `Ptaa::add_pt` / `Pta::push`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (12 件パス)
- [x] core.md 3 件 ❌ → ✅
- [x] plan 032 で 137 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `ptaa_index_labeled_pixels` は `Ptaa::with_capacity` だけだと
  `add_pt` が IndexOutOfBounds になるため、`(maxval + 1)` 個の
  空 Pta を事前に push して長さを揃える
