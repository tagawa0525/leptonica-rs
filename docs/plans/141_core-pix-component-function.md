# Core: pix_component_function (plan 032 残: 110b)

Status: IMPLEMENTED
作成日: 2026-05-15
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ D (110b)

## 対象 C 関数 (1)

`fpix2.c` の `pixComponentFunction`。32 bpp RGB 画像から
チャネル線形結合の ratio を FPix で返す。

## API 設計

```rust
pub fn pix_component_function(
    pix: &Pix,
    rnum: f32, gnum: f32, bnum: f32,
    rdenom: f32, gdenom: f32, bdenom: f32,
) -> Result<FPix>;
```

ピクセルごとに:

- `f_num   = rnum   * r + gnum   * g + bnum   * b`
- `f_denom = rdenom * r + gdenom * g + bdenom * b`
- `out     = f_num / f_denom` (general case)
- 全 denom が 0: `out = f_num` (divide skip)
- denom のうち 1 つだけ 1.0 (他は 0.0): 256-entry 逆数テーブルで高速化
- `f_denom == 0` (general case): `out = 256 * f_num` (C と同じ sentinel)

## 完了条件

- [x] cargo test/clippy/fmt 通過 (6 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 141 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
