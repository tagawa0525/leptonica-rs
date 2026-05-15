# Core: Pta noisy linear/quadratic LSF (plan 032 残: 111b)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ E (111b)

## 対象 C 関数 (2)

`ptafunc1.c` の robust LSF 系 2 関数。

- `ptaNoisyLinearLSF(pta, factor, ...)` — 1 回目 linear LSF →
  各点誤差の中央値で外れ値判定 → 残った点で再 LSF
- `ptaNoisyQuadraticLSF(pta, factor, ...)` — 同じ flow を quadratic
  fit で実行

## API 設計

```rust
pub struct NoisyLinearLsf {
    pub a: f32,             // y = a*x + b
    pub b: f32,
    pub median_error: f32,  // 初回 fit に対する |y - yf| の中央値
    pub inliers: Pta,       // 外れ値除去後の Pta
    pub fit: Option<Numa>,  // want_fit=true で再 fit の per-point 値
}

pub struct NoisyQuadraticLsf {
    pub a: f32, pub b: f32, pub c: f32,  // y = a*x^2 + b*x + c
    pub median_error: f32,
    pub inliers: Pta,
    pub fit: Option<Numa>,
}

impl Pta {
    pub fn noisy_linear_lsf(&self, factor: f32, want_fit: bool) -> Result<NoisyLinearLsf>;
    pub fn noisy_quadratic_lsf(&self, factor: f32, want_fit: bool) -> Result<NoisyQuadraticLsf>;
}
```

- `factor > 0.0` 必須 (typical ~3)
- linear は `n >= 3`、quadratic は `n >= 4` を要求
- 外れ値判定は `|error| <= factor * median_error` (`<=` で
  median_error == 0 のとき全点保持)

## 完了条件

- [x] cargo test/clippy/fmt 通過 (7 件パス)
- [x] core.md 2 件 ❌ → ✅
- [x] plan 032 で 138 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
