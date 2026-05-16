# Core: Numa Haar sum + best params (plan 032 残: 109b 完全解消)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (2)

109b 最後の 2 関数。これで 109b カテゴリ完全解消。

- `numaEvalHaarSum(nas, width, shift, relweight)` — Haar 風 comb (+1 と -relweight が交互、width 間隔、shift 位相) との畳み込みを `2 * width / n` で正規化して返す
- `numaEvalBestHaarParameters(nas, relweight, nwidth, nshift, minwidth, maxwidth)` — width/shift を sweep して最大スコアの(width, shift, score) を返す

## API 設計

```rust
impl Numa {
    /// C: `numaEvalHaarSum`
    pub fn eval_haar_sum(
        &self,
        width: f32,
        shift: f32,
        relweight: f32,
    ) -> Result<f32>;

    /// C: `numaEvalBestHaarParameters`
    pub fn eval_best_haar_parameters(
        &self,
        relweight: f32,
        nwidth: u32,
        nshift: u32,
        minwidth: f32,
        maxwidth: f32,
    ) -> Result<(f32, f32, f32)>; // (best_width, best_shift, best_score)
}
```

- `width` は > 0 finite、`shift` は >= 0 finite、`n >= 2 * width`
- best params 用途: `nwidth >= 2`, `nshift >= 1`, `0 < minwidth <= maxwidth`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (8 件パス)
- [x] core.md 2 件 ❌ → ✅
- [x] plan 032 で 136 を新規 IMPLEMENTED 行として追加、**109b 完全解消**
- [ ] PR + Copilot レビュー対応 + マージ
