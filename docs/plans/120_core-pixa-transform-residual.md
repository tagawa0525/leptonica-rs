# Core: Pixa transform 残り 4 関数 (plan 032 カテゴリ A-2 の続き)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-2

## 対象 C 関数 (4)

plan 107 (Pixa transform 7 件) の残り 7 件のうち、軽量で
依存先が既に揃っているもの 4 件を移植する。残り 3 件 (任意角
`pixaRotate`, `pixaConvertTo8Colormap`, `pixaConvertToNUpPixa`)
は別 plan で扱う。

### Border / Clip

- `pixaAddBorderGeneral(pixad, pixas, left, right, top, bot, val)` — 各 Pix にボーダーを追加、Boxa を補正
- `pixaClipToForeground(pixas)` — 各 Pix を FG にクリップし、 Pixa と Boxa の両方を返す

### Convert

- `pixaConvertToGivenDepth(pixas, depth)` — `depth ∈ {8, 32}` で 全 Pix を統一深度に
- `pixaConvertToSameDepth(pixas)` — cmap 除去 + 最大深度で揃える

## API 設計

```rust
impl Pixa {
    /// C: `pixaAddBorderGeneral`
    /// `val` は出力深度のピクセル表現 (1bpp は 0/1、8bpp は 0..=255、
    ///  32bpp は packed RGBA)
    pub fn add_border_general(
        &self,
        left: u32, right: u32, top: u32, bot: u32,
        val: u32,
    ) -> Result<Pixa>;

    /// C: `pixaClipToForeground` → (Pixa, Boxa)
    pub fn clip_to_foreground_all(&self) -> Result<(Pixa, Boxa)>;

    /// C: `pixaConvertToGivenDepth` (8 or 32 のみ)
    pub fn convert_to_given_depth(&self, depth: u32) -> Result<Pixa>;

    /// C: `pixaConvertToSameDepth`
    pub fn convert_to_same_depth(&self) -> Result<Pixa>;
}
```

## 依存

- 既存 `Pix::add_border_general`
- 既存 `Pix::clip_to_foreground`
- 既存 `Pix::convert_to_8` / `Pix::convert_to_32`
- 既存 `Pixa::get_rendering_depth`, `Pixa::any_colormaps`, `Pixa::get_depth_info` (plan 108 で追加済)
- 既存 `Box::adjust_sides` (or manual coordinate shift)

## テスト方針

- add_border_general: 寸法 / Box が左右上下シフト
- clip_to_foreground_all: 単一 Pix の FG クリップ / 空 FG / 非 1bpp
- convert_to_given_depth: 1bpp -> 8bpp / 8bpp -> 32bpp / 不正 depth で Err
- convert_to_same_depth: 混在 (1bpp + 8bpp) -> 8bpp / cmap 除去

## 完了条件

- [x] cargo test/clippy/fmt 通過 (11 件パス)
- [x] core.md 4 件 ❌ -> ✅
- [x] plan 032 で 120 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `add_border_general`: 各 Pix で `Pix::add_border_general` を呼び、 Box は `(b.x - left, b.y - top)` にシフト
- `clip_to_foreground_all`: 各 Pix で `Pix::clip_to_foreground` を呼ぶ。 FG が無い場合は元 Pix のディープクローン + 画像全体 Box を返す
- `convert_to_given_depth`: depth 8/32 限定。それ以外は Err。 既存 `convert_to_8`/`convert_to_32` に委譲
- `convert_to_same_depth`: rendering depth で cmap を除去 → 最大深度 (≤16 で 8、それ以外で 32) に揃える 2 段階処理
