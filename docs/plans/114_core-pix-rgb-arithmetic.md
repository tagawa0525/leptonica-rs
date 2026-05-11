# Core: pixarith.c の RGB スケーリング 5 関数 (plan 032 カテゴリ I)

Status: IMPLEMENTED
完了日: 2026-05-11
作成日: 2026-05-11
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ I

## 対象 C 関数

`reference/leptonica/src/pixarith.c`:

- `linearScaleRGBVal(sval, factor) -> dval` (1500行) — 単一 RGB ピクセル値の線形スケーリング (alpha 不変)
- `logScaleRGBVal(sval, tab, factor) -> dval` (1532行) — log2 スケーリング
- `pixAddRGB(pixs1, pixs2) -> pixd` (1043行) — 2 つの 32bpp/colormap 画像を成分別に飽和加算
- `pixMaxDynamicRangeRGB(pixs, type) -> pixd` (1411行) — RGB ダイナミックレンジ拡大 (linear / log)
- `pixThresholdToValue(pixd, pixs, threshval, setval) -> pixd` (530行) — 閾値以上 (setval>thresh) または以下 (setval<thresh) を setval にセット (8/16/32 bpp)

## API 設計

`src/core/pix/arith.rs` に追加 (Pix の impl ブロックに):

```rust
/// RGB スケーリング種別 (C の L_LINEAR_SCALE / L_LOG_SCALE)
pub enum RgbScaleType { Linear, Log }

/// 単一 RGB ピクセル値の線形スケーリング (alpha 不変)
/// C: linearScaleRGBVal
pub fn linear_scale_rgb_val(sval: u32, factor: f32) -> u32;

/// 単一 RGB ピクセル値の log2 スケーリング (alpha 不変)
/// C: logScaleRGBVal (log2 table は内部で構築)
pub fn log_scale_rgb_val(sval: u32, factor: f32) -> u32;

impl Pix {
    /// 2つの 32bpp RGB 画像の成分別飽和加算
    /// C: pixAddRGB
    pub fn add_rgb(&self, other: &Pix) -> Result<Pix>;

    /// 全ピクセル成分から最大値を求め、その値が 255 になるように
    /// 線形 / log2 スケール
    /// C: pixMaxDynamicRangeRGB
    pub fn max_dynamic_range_rgb(&self, scale_type: RgbScaleType) -> Result<Pix>;

    /// 8/16/32 bpp: threshval を境に setval を上または下方向に書き換える
    /// (setval > threshval なら setval 以下を setval に、setval < threshval
    /// なら setval 以上を setval に)
    /// C: pixThresholdToValue
    pub fn threshold_to_value(&self, threshval: u32, setval: u32) -> Result<Pix>;
}
```

## 設計上の注意

- C は `tab` 配列を外部で渡す API だが、Rust は内部で log2 計算を直接行うか

  キャッシュテーブルを内部実装する (公開 API には漏らさない)。

- `pixThresholdToValue` は colormap 経由は対象外 (C も 8/16/32 bpp で直接動作)。
- 8/16/32 bpp は `setval` の上限が異なる: 8bpp は 0xff、16bpp は 0xffff、32bpp は全 32bit。

## TDD 手順

`tests/core/pix_arith_rop_reg.rs` または `tests/core/pixacc_reg.rs` に
`#[ignore = "not yet implemented (plan 114)"]` 付きでテスト追加 →
GREEN で `#[ignore]` 除去。

## 完了条件

- [x] cargo test --all-features 全通過
- [x] cargo clippy --all-features --all-targets -- -D warnings 通過
- [x] cargo fmt --all -- --check 通過
- [x] PR + Copilot レビュー対応 + マージ
- [x] docs/porting/comparison/core.md で 5 件 ❌ → ✅
- [x] docs/plans/032 で 114 を IMPLEMENTED に
