# Core: numafunc2.c の Numa ヒストグラム拡張 5 関数 (plan 032 カテゴリ B の続き)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B

## 対象 C 関数 (5)

plan 109 (5 関数) に続く分。3 件は既存 API でカバー済みなので
対応エントリのみ更新、2 件は新規実装。

### 既存 API でカバー済み (doc only)

- `numaGetHistogramStats` -> 既存 `Numa::histogram_stats(startx, deltax)`
- `numaGetHistogramStatsOnInterval` -> 既存 `Numa::histogram_stats_on_interval`
- `numaGetStatsUsingHistogram` -> 既存 `Numa::stats_using_histogram`

### 新規実装

- `numaRebinHistogram(nas, newsize) -> Numa` — bin を `newsize` ごとに合算して粗いヒストグラムに rebin
- `numaMakeRankFromHistogram(startx, deltax, nasy, npts) -> (nax, nay)` — 正規化 → 累積 → 等間隔補間で rank 曲線を生成

## API 設計

```rust
impl Numa {
    /// C: `numaRebinHistogram`
    pub fn rebin_histogram(&self, new_size: usize) -> Result<Numa>;
}

/// C: `numaMakeRankFromHistogram` (npts 個に等間隔補間)
pub fn make_rank_from_histogram(
    startx: f32, deltax: f32, nasy: &Numa, npts: usize,
) -> Result<(Numa, Numa)>;
```

## 依存

- 既存 `Numa::normalize_histogram`
- 既存 `Numa::interpolate_eqx_interval`
- 既存 `Numa::parameters` / `set_parameters` / `get` / `push`

## テスト方針

- rebin_histogram: 単純な等寸法 / 端数あり / new_size <= 1 で Err
- make_rank_from_histogram: 単峰 histogram からの単調増加 rank /

  最終値 = 1.0 / npts 制御

## 完了条件

- [x] cargo test/clippy/fmt 通過 (7 件パス)
- [x] core.md 5 件 ❌ -> ✅
- [x] plan 032 で 119 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `numa_rebin_histogram`: ns/new_size を div_ceil で算出、 各出力 bin に new_size 個ずつ累積。trailing partial group は そのまま部分集計。出力の `deltax = src.deltax * new_size`
- `make_rank_from_histogram`: `normalize_histogram` で正規化、 cumulative sum を Numa に格納し `interpolate_eqx_interval` (Linear) で `npts` 個に再サンプル。nax は等間隔で 生成 (`startx + i * dx`)。npts < 3 / deltax <= 0 / empty で Err
- 既存 API でカバー済みの 3 関数 (Get/On_Interval/UsingHistogram) は対応エントリのみ更新
