# Core: Pixa convert-to-8-cmap + tiled extraction (plan 032 残)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3 (108b)

## 対象 C 関数 (3)

`pixafunc2.c` の Pixa 残課題のうち、既存 Pix/Pixa 操作の薄い
ラッパーで実装可能な 3 関数を移植する。

- `pixaConvertTo8Colormap(pixas, dither) -> Pixa` —
  各 Pix を 8bpp + colormap に変換、box を引き継ぐ
- `pixaMakeFromTiledPix(pixs, w, h, start, num, boxa) -> Pixa` —
  単一 Pix を nx×ny グリッドに分割 (boxa 指定時は box ベース)
- `pixaMakeFromTiledPixa(pixas, w, h, nsamp) -> Pixa` —
  Pixa の各 inner Pix をタイル分割し、`Pixa::join` で連結

## API 設計

```rust
impl Pixa {
    /// C: `pixaConvertTo8Colormap`
    pub fn convert_to_8_colormap(&self, dither: bool) -> Result<Pixa>;

    /// C: `pixaMakeFromTiledPixa` (各 Pix を w×h のグリッドに分割し
    /// nsamp 個ずつ取り出して全 inner を連結)
    pub fn make_tiled_pixa(&self, w: u32, h: u32, nsamp: u32) -> Result<Pixa>;
}

impl Pix {
    /// C: `pixaMakeFromTiledPix` (start から num 個のタイルを抽出。
    /// `boxa = Some(...)` なら box 配列で位置指定、`None` なら w×h 均等)
    pub fn make_tiled_pixa(
        &self,
        w: u32,
        h: u32,
        start: u32,
        num: u32,
        boxa: Option<&Boxa>,
    ) -> Result<Pixa>;
}
```

## 依存

- 既存 `Pix::convert_to_8_colormap`
- 既存 `Pix::clip_rectangle`
- 既存 `Pixa::create_from_boxa`
- 既存 `Pix::get_tile_count`
- 既存 `Pixa::join`

## テスト方針

- convert_to_8_colormap:
  - 32bpp → 8bpp + cmap、box が保持される
- Pix::make_tiled_pixa:
  - 30x20 の Pix を 10x10 タイル化 → 6 タイル (start=0, num=0=全件)
  - start/num で部分抽出
  - boxa 指定時は create_from_boxa 動作
  - tile_count text (`n = 5`) で n=5 のみ取得
- Pixa::make_tiled_pixa:
  - 2 個の inner Pixa から make_tiled_pixa(w, h, 3) で 2*3=6 件

## 完了条件

- [x] cargo test/clippy/fmt 通過 (8 件パス)
- [x] core.md 3 件 ❌ → ✅
- [x] plan 032 で 125 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `Pix::make_tiled_pixa`: C版が `pixaCreateFromBoxa(start, num)` で
  範囲指定するが、Rust の `create_from_boxa` には start/num が無いため、
  boxa 指定時は手動で範囲を切り出す形にする
- `Pixa::make_tiled_pixa`: C版は固定 10 個に対し、Rust では
  `self.len()` 分回す (より汎用)。nsamp は per-inner 件数の上限
