# Recog: skew.c の補助 2 関数 (plan 032 カテゴリ K)

Status: IMPLEMENTED
作成日: 2026-05-10
完了日: 2026-05-10
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ K

## 対象 C 関数

`reference/leptonica/src/skew.c` 1097-1228 行:

- `pixFindDifferentialSquareSum` — 行差分二乗和 (skew score の基礎)
- `pixFindNormalizedSquareSum` — 行/列正規化二乗和 (FG 分布の偏り指標)

## API 設計

`src/recog/skew.rs` に追加:

```rust
pub fn find_differential_square_sum(pix: &Pix) -> RecogResult<f32>;

/// Returns (hratio, vratio, fract). Some values are None when not requested
/// in C (here we always compute both ratios + fraction).
pub fn find_normalized_square_sum(pix: &Pix) -> RecogResult<(f32, f32, f32)>;
```

依存: 既存 `Pix::count_by_row(None)`, `Pix::rotate_orth_*`

## 完了条件

- [x] cargo test/clippy/fmt 通過
- [x] PR + Copilot レビュー対応 + マージ
- [x] docs/porting/comparison/recog.md で 2 件 ❌ → ✅
- [x] docs/plans/032 で K を IMPLEMENTED に
