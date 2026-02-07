# Histogram Implementation Plan

## Overview

ヒストグラム機能の実装。画像のピクセル値分布を分析するためのNumaベースの機能を提供。

## Reference

- C Source: `reference/leptonica/src/pix4.c`
  (pixGetGrayHistogram, pixGetColorHistogram)
- C Source: `reference/leptonica/src/numafunc2.c`
  (numaGetHistogramStats, numaHistogramGetRankFromVal,
   numaHistogramGetValFromRank)
- Target: `crates/leptonica-core/src/numa/`

## Implementation Plan

### Phase 1: Numa Histogram Statistics Extension

`crates/leptonica-core/src/numa/histogram.rs` に以下を実装:

#### 1.1 HistogramStats 構造体

```rust
/// Statistics computed from a histogram
pub struct HistogramStats {
    pub mean: f32,
    pub median: f32,
    pub mode: f32,
    pub variance: f32,
}
```

#### 1.2 numaGetHistogramStats() 相当

```rust
impl Numa {
    /// Get statistical measures from a histogram
    ///
    /// Arguments:
    /// - startx: x value of first bin (bin 0)
    /// - deltax: x increment between bins
    ///
    /// Returns HistogramStats with mean, median, mode, variance
    pub fn histogram_stats(&self, startx: f32, deltax: f32) -> Option<HistogramStats>;

    /// Get histogram stats on a specific interval [ifirst, ilast]
    pub fn histogram_stats_on_interval(
        &self,
        startx: f32,
        deltax: f32,
        ifirst: usize,
        ilast: Option<usize>,  // None means to the end
    ) -> Option<HistogramStats>;
}
```

#### 1.3 numaHistogramGetRankFromVal() 相当

```rust
impl Numa {
    /// Get rank (cumulative fraction) for a given value
    ///
    /// For a histogram y(x), computes the integral of y from startx to rval,
    /// normalized by total area.
    ///
    /// Returns rank in [0.0, 1.0]
    pub fn histogram_rank_from_val(&self, rval: f32) -> Option<f32>;
}
```

#### 1.4 numaHistogramGetValFromRank() 相当

```rust
impl Numa {
    /// Get the value corresponding to a given rank (cumulative fraction)
    ///
    /// Returns the x value such that the cumulative distribution
    /// from startx to that value equals the given rank.
    pub fn histogram_val_from_rank(&self, rank: f32) -> Option<f32>;
}
```

### Phase 2: Pix Histogram Functions

`crates/leptonica-core/src/pix/histogram.rs` に以下を実装:

#### 2.1 pixGetGrayHistogram() 相当

```rust
impl Pix {
    /// Get grayscale histogram
    ///
    /// Arguments:
    /// - factor: subsampling factor (1 = all pixels)
    ///
    /// Returns Numa of size 2^depth with pixel counts
    pub fn gray_histogram(&self, factor: u32) -> Result<Numa>;
}
```

対応するbit depth:

- 1-bit: 2 bins
- 2-bit: 4 bins
- 4-bit: 16 bins
- 8-bit: 256 bins
- 16-bit: 65536 bins

カラーマップがある場合は8-bitグレースケールに変換。

#### 2.2 pixGetColorHistogram() 相当

```rust
/// RGB channel histograms
pub struct ColorHistogram {
    pub red: Numa,
    pub green: Numa,
    pub blue: Numa,
}

impl Pix {
    /// Get color histogram (R, G, B channels separately)
    ///
    /// Only valid for 32-bit RGB images or colormapped images.
    /// Each channel returns a 256-bin histogram.
    pub fn color_histogram(&self, factor: u32) -> Result<ColorHistogram>;
}
```

## File Structure

```text
crates/leptonica-core/src/numa/
  mod.rs           # 既存 + histogram module公開
  histogram.rs     # NEW: Numa histogram statistics

crates/leptonica-core/src/pix/
  mod.rs           # histogram module公開追加
  histogram.rs     # NEW: Pix histogram functions
```

## Implementation Details

### Histogram Generation Algorithm (gray)

```text
1. Check depth is 1, 2, 4, 8, or 16
2. If colormap exists, convert to 8-bit grayscale conceptually
3. Create Numa with size = 2^depth, all zeros
4. Iterate over pixels with subsampling factor:
   for y in (0..height).step_by(factor)
     for x in (0..width).step_by(factor)
       val = get_pixel(x, y)
       histogram[val] += 1.0
5. Set Numa parameters: startx=0, deltax=1
6. Return histogram
```

### Histogram Stats Algorithm

```text
mean = sum(x[i] * count[i]) / sum(count[i])
variance = sum(x[i]^2 * count[i]) / sum(count[i]) - mean^2
median = x value where cumulative count reaches 50%
mode = x value with maximum count
```

### Rank/Val Conversion

- `rank_from_val(v)`: 0.0 to v までの累積分布
- `val_from_rank(r)`: 累積分布が r となる v の値

## Testing

1. Unit tests for histogram stats (known distributions)
2. Synthetic image histogram tests
3. Edge cases: empty image, single value, uniform distribution
4. Rank/val conversion roundtrip tests

## Dependencies

- 既存の Numa, Pix
- pix/access.rs の get_pixel 機能

## Questions

なし

## Status

- [x] Phase 1: Numa histogram statistics
- [x] Phase 2: Pix histogram functions
- [x] Tests
- [x] Documentation
