# Core: Numa::split_distribution (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (1)

- `numaSplitDistribution` — Otsu-style 分割。ヒストグラムを下/上の
  2 区画に分けるベスト分割点と、各区画の平均・件数、optional に
  per-bin score 配列を返す。

## API 設計

```rust
pub struct SplitDistribution {
    pub split_index: i32,  // best_split + 1, capped at 255
    pub ave1: f32,
    pub ave2: f32,
    pub num1: f32,
    pub num2: f32,
    pub score: Option<Numa>,
}

impl Numa {
    pub fn split_distribution(
        &self,
        score_fract: f32,
        want_score: bool,
    ) -> Result<SplitDistribution>;
}
```

- Score is normalized by `4 / (n - 1)^2` per C
- 候補スコアの最大値から (1 - score_fract) の連続範囲内で最小値を持つ
  bin を選ぶ
- `split_index = (best_split + 1).min(255)` — `pixThresholdToBinary`
  と整合させるための C 仕様

## 完了条件

- [x] cargo test/clippy/fmt 通過 (6 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 133 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
