# Core: Pixa::convert_to_nup (plan 032 残)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-2 (107b)

## 対象 C 関数 (1)

107b 残課題は `pixaConvertToNUpPixa` のみ。テキスト注釈なし
(C 版 `fontsize == 0` 経路) で移植する。

- `pixaConvertToNUpPixa(pixas, sa, nx, ny, tw, spacing, border, fontsize) -> Pixa` — Pixa を `nx × ny` のグリッドに並べた N-up Pixa を返す(1 ページ = 1 Pix で、計 ceil(n / (nx*ny)) Pix)

## API 設計

```rust
impl Pixa {
    /// C: `pixaConvertToNUpPixa` の fontsize = 0 経路。
    /// テキスト注釈は実装しない。
    pub fn convert_to_nup(
        &self,
        nx: u32,
        ny: u32,
        tile_width: u32,
        spacing: u32,
        border: u32,
    ) -> Result<Pixa>;
}
```

## 依存

- 既存 `Pixa::display_tiled_and_scaled`
- 既存 `Pixa::get_rendering_depth`
- 既存 `Pixa::scale_to_size`

## テスト方針

- 4 個の Pix を 2×2 グリッドで 1 ページ生成 → output Pixa の Pix 数 1
- 5 個の Pix を 2×2 グリッドで 2 ページ生成 → output Pixa の Pix 数 2 (5 = 4 + 1; 最後のページは部分埋め)
- `nx = 0` または `ny = 0` で Err
- `tile_width < 20` で Err (C 版仕様)
- 空 Pixa で空 Pixa を返す

## 完了条件

- [x] cargo test/clippy/fmt 通過 (5 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 127 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- 各ページ内の Pix を `scale_to_size(tile_width, 0)` で width 揃え
- inner `display_tiled_and_scaled(outdepth, tile_width + 2*border, nx, 0, spacing, border)` で 1 ページの合成画像を生成
- outdepth は `get_rendering_depth()` から決定
- C 版上限 `nx, ny <= 50` を踏襲
- BMF/text overlay (fontsize > 0 経路) は実装しない
