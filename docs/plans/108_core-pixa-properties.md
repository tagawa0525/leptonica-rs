# Core: pixafunc1.c / pixafunc2.c の Pixa プロパティ 8 関数 (plan 032 カテゴリ A-3 の一部)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3

## 対象 C 関数 (8)

軽量・独立性の高い Pixa の検査・比較・初期化 8 関数。残り 16 関数
(`pixaMakeSizeIndicator`, `pixaSort2dByIndex`, `pixaBinSort`,
`pixaClipToPix`, `pixaRenderComponent`, `pixaConstrainedSelect`,
`pixaSplitIntoFiles`, `pixaSelectToPdf`, `pixaMakeFromTiledPixa`,
`pixaMakeFromTiledPix`, `pixaaFlattenToPixa`, `pixaaScaleToSizeVar`,
`pixaaSelectRange`, `pixaaSizeRange`) は別 plan 108b で扱う。

### Property checks

- `pixaAnyColormaps(pixa, &hascmap)` — どれか 1 つでも colormap を持つか
- `pixaHasColor(pixa, &hascolor)` — colormap が色を使う or 32bpp を含むか
- `pixaGetDepthInfo(pixa, &maxdepth, &same)` — 最大 depth と全て同じ depth か

### Property derivation

- `pixaGetRenderingDepth(pixa, &depth)` — レンダリングに必要な depth (1/8/32)
- `pixaSizeRange(pixa, &minw, &minh, &maxw, &maxh)` — Pix の寸法範囲

### Equality

- `pixaEqual(pixa1, pixa2, maxdist, &naindex, &same)` — 境界 box + 内容比較 (順序非依存版を簡略化)

### Initialization

- `pixaSetFullSizeBoxa(pixa)` — 各 Pix の (0,0,w,h) で Boxa を作り直す

### Pix-level

- `pixGetTileCount(pix, &n)` — `pix.text()` が `"n = <N>"` 形式ならパースして count を返す (簡易テキストヘッダ機能)

## API 設計

```rust
impl Pixa {
    /// C: `pixaAnyColormaps`
    pub fn any_colormaps(&self) -> bool;

    /// C: `pixaHasColor`
    pub fn has_color(&self) -> bool;

    /// C: `pixaGetDepthInfo` → (maxdepth, all_same)
    pub fn get_depth_info(&self) -> Result<(u32, bool)>;

    /// C: `pixaGetRenderingDepth` → 1/8/32
    pub fn get_rendering_depth(&self) -> Result<u32>;

    /// C: `pixaSizeRange` → (minw, minh, maxw, maxh)
    pub fn size_range(&self) -> Option<(u32, u32, u32, u32)>;

    /// C: `pixaSetFullSizeBoxa` (mut, 各 Pix の 0,0,w,h を持つ Boxa を構築)
    pub fn set_full_size_boxa(&mut self);

    /// C: `pixaEqual` (順序版のみ。maxdist は将来予約。
    ///   不揃いの box reordering は plan 108b で対応)
    pub fn equal_to_ordered(&self, other: &Pixa, max_dist: u32) -> bool;
}

impl Pix {
    /// C: `pixGetTileCount` (text が "n = N" 形式なら N を返す)
    pub fn get_tile_count(&self) -> u32;
}
```

## 依存

- 既存: `Pixa::pix_slice`, `Pixa::boxa`, `Pixa::boxa_mut`
- 既存: `Pix::depth`, `Pix::width`, `Pix::height`, `Pix::colormap`, `Pix::text`, `Pix::equals`
- 既存: `PixColormap::has_color`
- 既存: `Boxa::equal_ordered`

## テスト方針

- any_colormaps / has_color: cmap あり/なし、32bpp 混在
- get_depth_info: 単一/混在 depth、空 Pixa で Err
- get_rendering_depth: 1bpp -> 1、8bpp -> 8、cmap 色 -> 32
- size_range: 単一/異種サイズ、空で None
- set_full_size_boxa: 各 Box の (x,y,w,h) が (0,0,pw,ph)
- equal_to_ordered: 同一 / 寸法違い / 順序違い
- get_tile_count: `"n = 5"` -> 5、無 text -> 0、不正 format -> 0

## 完了条件

- [x] cargo test/clippy/fmt 通過 (24 件パス)
- [x] core.md 8 件 ❌ -> ✅
- [x] plan 032 で 108 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- すべて `Pixa::pix_slice` / `Pixa::boxa` から read-only に走査
- `has_color`: 32bpp は即 true、cmap は `PixColormap::has_color()` で判定
- `get_depth_info`: 空 Pixa は Err、初期値は先頭 Pix の depth
- `get_rendering_depth`: has_color() で 32 早期 return、それ以外はmax_depth が 1 なら 1、それ以外は 8
- `size_range`: 単一スキャンで min/max を同時計算
- `set_full_size_boxa`: 各 Pix の幅・高さで Boxa を再構築
- `equal_to_ordered`: C `pixaEqual` の順序版のみ実装。 unordered (boxaEqual の reorder Numa を使う) は plan 108b で対応
- `Pix::get_tile_count`: text が `"n = N"` 形式の時のみ N をパース、それ以外はすべて 0 を返す (C の inexact-text 契約)
