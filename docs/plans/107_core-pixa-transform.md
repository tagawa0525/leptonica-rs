# Core: pixafunc1.c / pixafunc2.c の Pixa 変換 7 関数 (plan 032 カテゴリ A-2 の一部)

Status: IMPLEMENTED
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-2

## 対象 C 関数 (7)

軽量・独立性の高い Pixa の geometry/convert 系 7 関数。残り 7 関数
(任意角 `pixaRotate` / `pixaConvertTo8Colormap` / `pixaConvertToNUpPixa` /
`pixaConvertToGivenDepth` / `pixaConvertToSameDepth` /
`pixaAddBorderGeneral` / `pixaClipToForeground`)
は別 plan 107b で扱う。

### Geometry

- `pixaScale(pixas, scalex, scaley)` — 各 Pix を scale、Boxa も同係数で変換
- `pixaScaleBySampling(pixas, scalex, scaley)` — sampling 版 scale
- `pixaRotateOrth(pixas, quads)` — 直交回転 (0/90/180/270 deg)(任意角 `pixaRotate` は plan 107b で対応)
- `pixaTranslate(pixas, hshift, vshift, incolor)` — 平行移動

### Convert

- `pixaConvertTo1(pixas, thresh)` — 全ての Pix を 1 bpp に
- `pixaConvertTo8(pixas, cmap_flag)` — 全ての Pix を 8 bpp に
- `pixaConvertTo32(pixas)` — 全ての Pix を 32 bpp に

## API 設計

```rust
impl Pixa {
    /// C: `pixaScale`
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Result<Pixa>;

    /// C: `pixaScaleBySampling`
    pub fn scale_by_sampling(&self, scale_x: f32, scale_y: f32) -> Result<Pixa>;

    /// C: `pixaRotateOrth` (quads = 0/1/2/3 → 0/90/180/270 deg)
    pub fn rotate_orth(&self, quads: u32) -> Result<Pixa>;

    /// C: `pixaTranslate`
    pub fn translate(&self, hshift: i32, vshift: i32, incolor: InColor) -> Result<Pixa>;

    /// C: `pixaConvertTo1` (threshold-based)
    pub fn convert_to_1(&self, thresh: u32) -> Result<Pixa>;

    /// C: `pixaConvertTo8` (cmap_flag: add gray colormap)
    pub fn convert_to_8(&self, cmap_flag: bool) -> Result<Pixa>;

    /// C: `pixaConvertTo32`
    pub fn convert_to_32(&self) -> Result<Pixa>;
}
```

`pixaRotate` (任意角回転) は依存する `pix_rotate` の API が
RotateOptions を取るため、シグネチャ整合に時間がかかる。
別 plan 107b で扱う。

## 依存

- 既存: `transform::scale`, `transform::scale_by_sampling`, `transform::rotate_orth`, `transform::translate`
- 既存: `Pix::convert_to_8`, `Pix::convert_to_32`, `Pix::convert_to_1_by_sampling`
- 既存: `Pix::add_border_general`, `Pix::clip_to_foreground`
- 既存: `Pixa::with_capacity`, `Pixa::push_with_box`
- 既存: `Boxa::transform`, `Box::scale_by`

## テスト方針

- 各メソッドにつき:
  - 基本動作 (出力サイズ・Box が期待通り)
  - 空 Pixa で空 Pixa を返す
  - 不正パラメータ (scale<=0, quads>3, thresh out-of-range) で Err

## 完了条件

- [x] cargo test/clippy/fmt 通過 (12 件パス)
- [x] core.md 7 件 ❌ -> ✅
- [x] plan 032 で 107 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- すべて `Pixa::with_capacity` + `push_with_box` で構築
- `scale` / `scale_by_sampling`: `transform::scale` / `scale_by_sampling`

  を各 Pix に適用。Box は `scale_box` ヘルパー (C `boxaTransform(0,0,sx,sy)` 相当)

- `rotate_orth`: `quads == 0` でクローン、それ以外は `transform::rotate_orth`
  - `Box::rotate_orth` で Box を変換
- `translate`: `Pix::translate(hshift, vshift, incolor)` を呼び、

  Box は `(x+hshift, y+vshift)` にシフト

- `convert_to_1/8/32`: 各 Pix の対応メソッドを呼ぶ。cmap_flag は

  現状 8bpp ⇒ gray cmap を生成しないシンプルな実装。
  完全な cmap 対応は plan 107b で扱う
