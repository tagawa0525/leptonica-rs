# Core: fpix2.c の FPix 拡張 7 関数 (plan 032 カテゴリ D の一部)

Status: PLANNED
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ D

## 対象 C 関数 (7)

軽量・独立性の高い FPix 拡張 7 関数。affine/projective + Pta 系の
残り 5 関数 (`fpixAffine` / `fpixAffinePta` / `fpixProjective` /
`fpixProjectivePta` / `pixComponentFunction`) は別 plan 110b で扱う。

### Min/Max

- `fpixGetMin(fpix) -> (val, x, y)` — C は output pointer 経由。

  既存 `FPix::min()` が同等機能なのでエイリアス追加または
  ドキュメント更新で対応する判断もある

- `fpixGetMax(fpix) -> (val, x, y)` — 同上

### Threshold/Rasterop/Scale

- `fpixThresholdToPix(fpix, thresh) -> Pix(1bpp)` — `val <= thresh` で前景を立てる 1 bpp 出力
- `fpixRasterop(dest, dx, dy, dw, dh, src, sx, sy)` — クリップ後にsrc の矩形を dest にコピー (常に Src オペレーション)
- `fpixScaleByInteger(fpix, factor) -> FPix` — integer factor でbilinear 補間しながら拡大 (出力サイズは `factor*(ws-1)+1` x ...)

### Border/Sampling

- `fpixRemoveBorder(fpix, left, right, top, bot) -> FPix` — 境界を削った FPix を返す
- `linearInterpolatePixelFloat(data, w, h, x, y, inval) -> f32` — 16 段の固定小数点で bilinear 補間する内部ヘルパー (FPix 直接ではなくraw data 入力)

## API 設計

```rust
// in src/core/fpix/mod.rs (or extension module)
impl FPix {
    /// C: `fpixGetMin` (既存の `min` のエイリアス)
    pub fn get_min(&self) -> Option<(f32, u32, u32)>;

    /// C: `fpixGetMax` (既存の `max` のエイリアス)
    pub fn get_max(&self) -> Option<(f32, u32, u32)>;

    /// C: `fpixThresholdToPix`
    pub fn threshold_to_pix(&self, thresh: f32) -> Result<Pix>;

    /// C: `fpixRasterop` (dest is &mut self)
    pub fn rasterop(
        &mut self,
        dx: i32, dy: i32, dw: i32, dh: i32,
        src: &FPix, sx: i32, sy: i32,
    ) -> Result<()>;

    /// C: `fpixScaleByInteger` (factor>=1)
    pub fn scale_by_integer(&self, factor: u32) -> Result<FPix>;

    /// C: `fpixRemoveBorder`
    pub fn remove_border(&self, left: i32, right: i32, top: i32, bot: i32) -> Result<FPix>;
}

/// C: `linearInterpolatePixelFloat`
pub fn linear_interpolate_pixel_float(
    data: &[f32], w: i32, h: i32, x: f32, y: f32, inval: f32,
) -> f32;
```

`linearInterpolatePixelFloat` は raw `&[f32]` を取るので
free fn として `src/core/fpix/interpolate.rs` 等に置く。
内部用だが affine/projective からも参照されるため `pub` 公開。

## 依存

- 既存 `FPix::min` / `FPix::max` (エイリアスの裏付け)
- 既存 `Pix::new` (1bpp 出力用)
- 既存 `FPix::new` / `FPix::set_all` / `FPix::data` / `FPix::data_mut`

## テスト方針

- get_min / get_max: 単純な値配置 / 全 0 / 端 (0,0) / 単峰
- threshold_to_pix: 全 <= thresh / 全 > thresh / 混在
- rasterop: 単純コピー / 負オフセット / 範囲外クリップ
- scale_by_integer: factor=1 (恒等) / factor=2 (bilinear 補間)
- remove_border: 全 0 ボーダー (恒等) / 部分削除 / 過剰削除 (Err)
- linear_interpolate_pixel_float: 整数位置 / 中間位置 / 範囲外 (inval)

## 完了条件

- [ ] cargo test/clippy/fmt 通過
- [ ] core.md 7 件 ❌ -> ✅
- [ ] plan 032 で 110 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ
