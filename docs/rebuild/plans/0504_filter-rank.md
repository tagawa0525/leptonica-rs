# Rank Filter Implementation Plan

## Status

- [x] Planning
- [x] Implementation
- [x] Testing
- [ ] Review

## Overview

ランクフィルタ（順序統計フィルタ）を `leptonica-filter` クレートに実装する。
C版の `rank.c` を参考に、8bppグレースケールと32bppカラー画像に対応した
効率的なランクフィルタを実装する。

## Reference

- C source: `reference/leptonica/src/rank.c`
- Key functions:
  - `pixRankFilter()` - メインディスパッチ関数
  - `pixRankFilterGray()` - 8bppグレースケール用
  - `pixRankFilterRGB()` - 32bppカラー用（チャンネル独立処理）
  - `pixMedianFilter()` - rank=0.5の特殊ケース

## Algorithm

### Two-histogram approach

C版では効率化のため、2つのヒストグラムを使用している:

1. **Coarse histogram (16 bins)**: 粒度16のヒストグラム（値を16で割った位置）
2. **Fine histogram (256 bins)**: 通常のヒストグラム

ランク値を見つける手順:

1. Coarse histogramを走査して該当ビンを特定
2. Fine histogramの対応する16個のビンを走査して正確な値を特定

これにより、平均的に16+16=32回の走査で済む（256回の代わりに）。

### Incremental update

フィルタを1ピクセル移動するたびに:

- 行優先走査（wf >= hf の場合）: 左端の列を削除、右端の列を追加
- 列優先走査（hf > wf の場合）: 上端の行を削除、下端の行を追加

## API Design

```rust
// Main dispatch function
pub fn rank_filter(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix>;

// Grayscale-specific implementation
pub fn rank_filter_gray(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix>;

// Color-specific implementation
pub fn rank_filter_color(pix: &Pix, width: u32, height: u32, rank: f32) -> FilterResult<Pix>;

// Convenience functions
pub fn median_filter(pix: &Pix, width: u32, height: u32)
    -> FilterResult<Pix>;
pub fn min_filter(pix: &Pix, width: u32, height: u32)
    -> FilterResult<Pix>;  // rank=0.0
pub fn max_filter(pix: &Pix, width: u32, height: u32)
    -> FilterResult<Pix>;  // rank=1.0
```

## Implementation Plan

### Phase 1: Core infrastructure

1. 新規ファイル `crates/leptonica-filter/src/rank.rs` を作成
2. エラーハンドリング追加（必要に応じて `error.rs` を更新）
3. `lib.rs` にモジュールを追加

### Phase 2: Grayscale rank filter

1. `RankHistogram` 構造体を実装
   - coarse histogram (16 bins)
   - fine histogram (256 bins)
   - add/remove pixel operations
   - get_rank_value() method

2. `rank_filter_gray()` を実装
   - 境界処理（mirror padding）
   - 行優先/列優先の走査切り替え
   - incremental update

### Phase 3: Color rank filter

1. `rank_filter_color()` を実装
   - 各チャンネル（R, G, B）を独立に処理
   - `rank_filter_gray()` を再利用

### Phase 4: Convenience functions

1. `median_filter()` - rank=0.5
2. `min_filter()` - rank=0.0
3. `max_filter()` - rank=1.0

### Phase 5: Testing

1. 単体テスト
   - パラメータ検証テスト
   - 1x1フィルタ（no-op）テスト
   - 基本的なランクフィルタテスト
   - メディアン/最小/最大フィルタテスト

## Boundary Handling

C版と同様にミラーパディングを使用する。
ただし、Rust実装ではclamp方式（境界値を複製）を採用してシンプル化する
（bilateral.rsやconvolve.rsと同じ方式）。

## Performance Considerations

- 2-histogram アプローチで O(wf*hf) を O(16) に削減
- Incremental update で各ピクセルの処理を効率化
- 行優先/列優先の選択でキャッシュ効率を向上

## Files to Modify

1. `crates/leptonica-filter/src/rank.rs` - 新規作成
2. `crates/leptonica-filter/src/lib.rs` - モジュール追加
3. `crates/leptonica-filter/src/error.rs` - 必要に応じてエラー追加

## Questions

なし

## Notes

- C版では rank=0.0 と rank=1.0 の場合に grayscale erosion/dilation に
  ディスパッチしているが、本実装では一貫性のためランクフィルタとして処理する
- 境界処理は既存のフィルタ実装（bilateral, convolve）に合わせてclamp方式を採用
