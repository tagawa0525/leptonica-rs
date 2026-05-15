# Core: compare_tiles_by_histo (plan 032 残: 117)

Status: IMPLEMENTED
作成日: 2026-05-15
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ F (117)

## 対象 C 関数 (1)

- `compareTilesByHisto(naa1, naa2, minratio, w1, h1, w2, h2, &score, pixadebug)`
  — 2 Numaa の per-tile 256-bin ヒストグラム間 EMD を計算し、
  最も低いタイルの類似度スコアを返す

## API 設計

```rust
pub fn compare_tiles_by_histo(
    naa1: &Numaa,
    naa2: &Numaa,
    minratio: f32,
    w1: i32, h1: i32,
    w2: i32, h2: i32,
) -> Result<f32>;
```

- 寸法比 (`min(w1,w2)/max(w1,w2)` / `min(h1,h2)/max(h1,h2)`) が
  `minratio` 未満なら 0.0
- Numaa 件数が異なれば 0.0
- 各 tile: bin 255 (white) を 0 にしてから EMD、
  `score = max(0, 1 - 10 * dist / 255)`
- 戻り値は全 tile の minimum score

## 依存

- 既存 `Numa::earth_mover_distance` (plan 130)
- 既存 `Numa::set`, `Numaa::get`

## 完了条件

- [x] cargo test/clippy/fmt 通過 (8 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 142 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- pixadebug (gplot PDF 出力) は debug-only で実装しない
