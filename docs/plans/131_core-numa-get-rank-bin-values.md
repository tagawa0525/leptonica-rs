# Core: Numa::get_rank_bin_values (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (1)

- `numaGetRankBinValues(na, nbins) -> Numa` — 任意 Numa から rank bin 平均値を求める。データ範囲に応じて shell-sort 経路または histogram 経路を選ぶ。

## 実装

plan 130 で追加した `discretize_sorted_in_bins` / `discretize_histo_in_bins`
を利用し、`numaChooseSortType` (既存 `Numa::choose_sort_type`)
に基づいて経路を分岐する。

```rust
impl Numa {
    pub fn get_rank_bin_values(&self, nbins: u32) -> Result<Numa>;
}
```

- shell-sort 経路: `sort + discretize_sorted_in_bins`
- histogram 経路: `make_histogram + discretize_histo_in_bins`
- 経路選択: `choose_sort_type(len, max_val)` の結果が `true` のときヒストグラム、`false` のときソート

## 完了条件

- [x] cargo test/clippy/fmt 通過 (3 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 131 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
