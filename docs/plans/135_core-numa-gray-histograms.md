# Core: gray histogram 系 2 関数 (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-14
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (2)

- `grayHistogramsToEMD(naa1, naa2, &nad)` — 2 Numaa 内の対応する 256-bin ヒストグラム間 EMD を計算、`/255` で [0.0, 1.0] に正規化
- `grayInterHistogramStats(naa, wc, ...)` — 各 inner histogram を smooth + normalize (sum=10000) し、256 bin 位置ごとに `simple_stats` で aggregate

## API 設計

```rust
impl Numa {
    /// C: `grayHistogramsToEMD`
    pub fn gray_histograms_to_emd(
        naa1: &Numaa,
        naa2: &Numaa,
    ) -> Result<Numa>;

    /// C: `grayInterHistogramStats`
    pub fn gray_inter_histogram_stats(
        naa: &Numaa,
        wc: usize,
        want_mean: bool,
        want_mean_square: bool,
        want_variance: bool,
        want_rms: bool,
    ) -> Result<InterHistogramStats>;
}

pub struct InterHistogramStats {
    pub mean: Option<Numa>,
    pub mean_square: Option<Numa>,
    pub variance: Option<Numa>,
    pub rms: Option<Numa>,
}
```

## 依存

- 既存 `Numa::earth_mover_distance` (plan 130)
- 既存 `Numa::windowed_mean` / `normalize_histogram` / `simple_stats`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (7 件パス)
- [x] core.md 2 件 ❌ → ✅
- [x] plan 032 で 135 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
