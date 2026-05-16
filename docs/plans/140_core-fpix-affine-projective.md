# Core: fpix_affine + fpix_projective (plan 032 残: 110b)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ D (110b)

## 対象 C 関数 (2)

`fpix2.c` の FPix 残課題のうち、係数ベースの affine / projective
変換を移植する。

- `fpixAffine(fpixs, vc[6], inval)` — 6 係数で逆方向に変換し、各 dst 画素に bilinear interpolation で値を写す
- `fpixProjective(fpixs, vc[8], inval)` — 8 係数の射影変換 (`denom = vc[6]*j + vc[7]*i + 1` で割る)

Pta-based wrapper (`fpixAffinePta` / `fpixProjectivePta`) は
`fpix_add_slope_border` 依存のため別 plan で扱う。

## API 設計

```rust
pub fn fpix_affine(
    fpixs: &FPix,
    vc: &[f32; 6],
    inval: f32,
) -> Result<FPix>;

pub fn fpix_projective(
    fpixs: &FPix,
    vc: &[f32; 8],
    inval: f32,
) -> Result<FPix>;
```

## 依存

- 既存 `linear_interpolate_pixel_float` (plan 110)
- 既存 `FPix::create_template` / `set_all` / `data` / `data_mut`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (6 件パス)
- [x] core.md 2 件 ❌ → ✅
- [x] plan 032 で 140 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
