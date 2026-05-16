# Core: Numa::crossings_by_peaks (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (1)

- `numaCrossingsByPeaks(nax, nay, delta)` — 隣接する peaks/troughs の中点を閾値として、各セグメントの threshold 交差点を計算する

## API 設計

```rust
impl Numa {
    pub fn crossings_by_peaks(&self, nax: Option<&Numa>, delta: f32) -> Result<Numa>;
}
```

- self は y 値、nax (optional) は x 座標
- 既存 `Numa::find_extrema(delta)` で peak/trough 位置を求め、末尾に `n-1` を追加して最終遷移を捕捉
- 各セグメント内で (prev_peak_val + cur_peak_val) / 2 を閾値とし、線形補間で交差点を 1 つ抽出

## 完了条件

- [x] cargo test/clippy/fmt 通過 (4 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 134 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
