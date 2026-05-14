# Core: Numa Earth-Mover Distance + discretize (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (3)

`numafunc2.c` の Numa 残課題 (109b 13 件) のうち、相互依存が少なく
他のロードマップ項目 (compare/photo-histo) で多用される 3 関数を
切り出して移植する。

- `numaEarthMoverDistance(na1, na2) -> f32` — 同サイズの 2 Numa の
  1 次元 Earth-Mover Distance を返す
- `numaDiscretizeSortedInBins(na, nbins) -> Numa` — 既ソートの Numa を
  nbins 等数バケットで離散化し、各バケットの平均値を返す
- `numaDiscretizeHistoInBins(na, nbins) -> (Numa, Numa)` —
  ヒストグラム Numa を rank-bin で離散化、`(bin 平均値, 累積 rank)`
  を返す

## API 設計

```rust
impl Numa {
    /// C: `numaEarthMoverDistance`
    pub fn earth_mover_distance(&self, other: &Numa) -> Result<f32>;

    /// C: `numaDiscretizeSortedInBins` (input は既ソート)
    pub fn discretize_sorted_in_bins(&self, nbins: u32) -> Result<Numa>;

    /// C: `numaDiscretizeHistoInBins`
    /// 戻り値: (bin 平均値, optional 累積 rank)
    pub fn discretize_histo_in_bins(
        &self,
        nbins: u32,
        want_rank: bool,
    ) -> Result<(Numa, Option<Numa>)>;
}
```

## 依存

- 既存 `Numa::get_f32 / get_i32 / push / len / sum / from_i32_slice`
- 既存 `numa_uniform_bin_sizes(ntotal, nbins)` (plan 109)

## テスト方針

- `earth_mover_distance`:
  - 同じ Numa 同士は 0.0
  - シフトした Numa で正値、シフト距離に比例
  - 異なる総量で正規化される (`sum1 / sum2` 倍に揃える)
  - 長さ不一致で Err
- `discretize_sorted_in_bins`:
  - 1..=10 の昇順 Numa を 2 bins に → [3.0, 8.0] (平均値)
  - nbins < 2 で Err、空 Numa で Err
- `discretize_histo_in_bins`:
  - 全 16 値が equal count のヒストグラム → 平均値が等間隔
  - want_rank=true で累積 norm histogram を併せて返す

## 完了条件

- [x] cargo test/clippy/fmt 通過 (10 件パス)
- [x] core.md 3 件 ❌ → ✅
- [x] plan 032 で 130 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- EMD の `na3` は `na2` を `sum1/sum2` でスケールしたコピー。
  total = Σ|na1[i] - cumulative_na3[i]| / sum1
- 既ソート/ヒストグラム両方とも `numa_uniform_bin_sizes` で
  各 bin の要素数を決め、累積平均を計算する
