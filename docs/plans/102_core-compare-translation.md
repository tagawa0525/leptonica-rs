# core/compare: 並進付き比較関数の移植

Status: PLANNED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 A)

## Context

C 版 `compare.c` の以下 2 関数が Rust 版に存在しない。

- `pixCompareWithTranslation()` (compare.c:3420, 109 行)
- `pixBestCorrelation()` (compare.c:3572, 79 行)

両者は粗→細の段階的シフト探索によって 2 枚の 2 値画像の最適並進量と相関スコアを求める。
祖先用途は OCR の文字テンプレート位置合わせ、レイアウト比較等。

C 版アルゴリズム概要:

1. 2 値化（必要なら）して 4 段階に縮小（rank reduce 2x を 3 回）
2. 最低解像度（8x 縮小）で重心差から初期推定 (etransx, etransy) を取り、maxshift=6 で `pixBestCorrelation` を呼ぶ
3. 解像度を上げる毎に推定値を 2 倍し、maxshift=2 で再探索
4. `pixBestCorrelation` は `(etransx ± maxshift) × (etransy ± maxshift)` の各シフトで `pixCorrelationScoreShifted` を呼び最大スコアを返す

## 既存 Rust 依存（確認済み）

| 必要関数                     | Rust 実装                                             |
| ---------------------------- | ----------------------------------------------------- |
| `pixCorrelationScoreShifted` | `src/recog/correlscore.rs::correlation_score_shifted` |
| `pixCentroid`                | `src/morph/morphapp.rs::pix_centroid`                 |
| `pixCountPixels`             | `src/core/pix/statistics.rs::Pix::count_pixels`       |
| `pixReduceRankBinary2`       | `src/transform/binreduce.rs::reduce_rank_binary_2`    |
| `pixConvertTo1`              | core 既存                                             |

## 配置先・API 設計

- ファイル: `src/core/pix/compare.rs` に追記（`PixCompareError` を拡張）
- 公開関数:

```rust
pub struct TranslationMatch {
    pub delx: i32,
    pub dely: i32,
    pub score: f32,
}

/// pixCompareWithTranslation 相当
pub fn compare_with_translation(
    pix1: &Pix,
    pix2: &Pix,
    thresh: i32,
) -> Result<TranslationMatch>;

/// pixBestCorrelation 相当（1bpp 入力前提）
pub fn best_correlation(
    pix1: &Pix,
    pix2: &Pix,
    area1: u32,
    area2: u32,
    etransx: i32,
    etransy: i32,
    maxshift: i32,
) -> Result<TranslationMatch>;
```

debugflag (PDF 出力) は移植しない。デバッグ目的のため呼び出し側で必要なら別関数化。

## TDD ステップ

1. **RED コミット** (`test(core): pixCompareWithTranslation/pixBestCorrelation の RED テスト`)
   - `tests/core/compare_reg.rs` の `#[ignore = "...pixBestCorrelation not available"]` と `#[ignore = "...pixCompareWithTranslation not available"]` を解除し、Rust API を呼ぶ実体に置換（実装はまだなのでコンパイル不可になるか、stub で `unimplemented!()` を返す）
   - golden hash は当面 `display` モードで生成し、テストは `Result::Ok` のみ確認するレベルに留める

2. **GREEN コミット** (`feat(core): compare_with_translation / best_correlation を実装`)
   - 上記アルゴリズムを Rust に移植
   - `compare_with_translation` は内部で `best_correlation` を呼ぶ
   - 単体テストは `tests/core/compare_reg.rs` 既存 `#[ignore]` を unignore する形

3. **REFACTOR**（必要なら）
   - 縮小カスケード生成を `Pixa` ヘルパに切り出すなど。最小限に留める

## テスト戦略

- C 版 regression `prog/comparetest.c` の対応データを `tests/data/images/` から取り、既知の (delx, dely) で人工的にずらした画像対を用意
- 既存 `#[ignore]` の `pixBestCorrelation_score_match` と `pixCompareWithTranslation_finds_translation` を有効化
- スコアは float なので `compare_values` で許容差比較

## ブランチ・PR

- ブランチ: `feat/core-compare-translation`
- PR タイトル: `feat(core): port pixCompareWithTranslation / pixBestCorrelation`
- 1PR、RED → GREEN の 2 コミット

## ステータス

- [ ] RED コミット
- [ ] GREEN コミット
- [ ] manifest 更新
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
