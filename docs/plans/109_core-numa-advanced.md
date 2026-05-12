# Core: numafunc2.c の Numa 高度関数 5 関数 (plan 032 カテゴリ B の一部)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B

## 対象 C 関数 (5)

軽量・独立性の高い Numa 解析・生成系 5 関数。
残り 18 関数 (histogram stats、EMD、Haar、discretize 系)
は plan 109b で扱う。

`numaHistogramGetRankFromVal` / `numaHistogramGetValFromRank` は
既存の `Numa::histogram_rank_from_val` / `Numa::histogram_val_from_rank`
(numa::histogram) で実装済み。対応エントリのみ更新。

### 反転・交差・ピーク

- `numaCountReversals(nas, minreversal) -> (nr, rd)` — 値の反転回数と密度
- `numaCrossingsByThreshold(nax, nay, thresh) -> Numa` — 閾値を跨ぐ x 位置
- `numaFindPeaks(nas, nmax, fract1, fract2) -> Numa` — 上位 nmax のピーク

### ヒストグラム rank/val

- `numaHistogramGetRankFromVal(na, rval) -> rank` — 値からランク
- `numaHistogramGetValFromRank(na, rank) -> val` — ランクから値

### 範囲生成

- `numaGetUniformBinSizes(ntotal, nbins) -> Numa` — 等分割サイズ
- `genConstrainedNumaInRange(first, last, nmax, use_pairs) -> Numa` — 範囲内の制約付き値列生成

## API 設計

```rust
impl Numa {
    /// C: `numaCountReversals` → (nr, rd)
    pub fn count_reversals(&self, min_reversal: f32) -> Result<(u32, f32)>;

    /// C: `numaFindPeaks` (nmax, fract1, fract2)
    pub fn find_peaks(&self, nmax: u32, fract1: f32, fract2: f32) -> Numa;

    /// C: `numaHistogramGetRankFromVal`
    pub fn histogram_rank_from_val(&self, rval: f32) -> f32;

    /// C: `numaHistogramGetValFromRank`
    pub fn histogram_val_from_rank(&self, rank: f32) -> f32;
}

/// C: `numaCrossingsByThreshold` (nax may be None for default x = startx + i*delx)
pub fn numa_crossings_by_threshold(
    nay: &Numa,
    nax: Option<&Numa>,
    thresh: f32,
) -> Result<Numa>;

/// C: `numaGetUniformBinSizes`
pub fn numa_uniform_bin_sizes(ntotal: i32, nbins: i32) -> Result<Numa>;

/// C: `genConstrainedNumaInRange`
pub fn gen_constrained_numa_in_range(
    first: i32, last: i32, nmax: i32, use_pairs: bool,
) -> Result<Numa>;
```

## 依存

- 既存 `Numa::push`, `Numa::get`, `Numa::parameters`, `Numa::sum`, `Numa::max`, `Numa::with_capacity`, `Numa::from_vec`

## テスト方針

- count_reversals: バイナリ (0/1) 配列 / 連続値 / min_reversal で reject
- crossings_by_threshold: 単純な上昇/下降 / 0 跨ぎ / nax 指定
- find_peaks: 単峰 / 双峰 / 平坦
- histogram_rank_from_val / val_from_rank: 既知 histogram でラウンドトリップ
- uniform_bin_sizes: 等分 / ntotal < nbins / 端数あり
- gen_constrained_numa_in_range: 単純 / use_pairs / nmax 制約

## 完了条件

- [x] cargo test/clippy/fmt 通過 (17 件パス)
- [x] core.md 7 件 ❌ -> ✅ (本 plan 5 + 既存 Numa::histogram_**from** 対応の numaHistogramGetRank/ValFromVal/Rank)
- [x] plan 032 で 109 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `count_reversals`: 全要素が 0/1 ならバイナリ高速パス、それ以外は `find_extrema` を delta=min_reversal で呼んで反転数を取得
- `find_peaks`: clone() で破壊的更新可能な作業用 Numa を作り、最大値を nmax 回検出。各ピークは (lloc, max_loc, rloc, peak_fract) の 4 要素で記録
- `numa_crossings_by_threshold`: 隣接サンプルが thresh を跨ぐ場合に 線形補間で x を求める。nax 指定時は長さ一致を検証
- `numa_uniform_bin_sizes`: ntotal < nbins は前から 1 ずつ詰める、それ以外は等分割 (端数を最後の bin で吸収)
- `gen_constrained_numa_in_range`: nmax/range/use_pairs から nsets を導出して delta を計算、(use_pairs 時) 隣接ペア追加
