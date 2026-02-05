# Image Comparison Implementation Plan

## Status: IMPLEMENTED

## Summary

画像比較機能をleptonica-coreクレートに実装する。
C版Leptonicaのcompare.cに相当する機能を、Rust idiomaticなAPIで提供する。

## Reference

- C version: `reference/leptonica/src/compare.c`
- Existing patterns: `crates/leptonica-core/src/pix/ops.rs`, `crates/leptonica-filter/src/bilateral.rs`

## Background

画像比較は画像処理の基本操作であり、以下の用途がある:

- テスト時の期待結果との比較
- 重複画像の検出
- 変更検出（ビフォー・アフター）
- 画像品質評価（PSNR、RMSなど）

## Implementation Scope

### Core Functions (Phase 1)

1. **ピクセル単位の差分**
   - `subtract()`: pix1 - pix2（負の値は0にクリップ）
   - `abs_diff()`: |pix1 - pix2|

2. **画像等価性**
   - `equals()`: 2つの画像が同一ピクセル値を持つか判定
   - `equals_with_alpha()`: アルファチャンネルも含めて比較

3. **相関比較（バイナリ画像用）**
   - `correlation_binary()`: バイナリ画像間の相関係数

4. **統計的比較**
   - `rms_diff()`: RMS（Root Mean Square）差分
   - `mean_abs_diff()`: 平均絶対差分

### Phase 2 (Future)

- `psnr()`: PSNR（Peak Signal-to-Noise Ratio）
- `compare_by_histogram()`: ヒストグラムベースの比較
- `perceptual_diff()`: 知覚的差分

## API Design

```rust
// crates/leptonica-core/src/pix/compare.rs

use crate::{Pix, PixelDepth, Result};

/// Result of comparing two images
#[derive(Debug, Clone)]
pub struct CompareResult {
    /// Whether the images are identical
    pub equal: bool,
    /// Root mean square difference (0.0 for identical images)
    pub rms_diff: f64,
    /// Mean absolute difference
    pub mean_abs_diff: f64,
    /// Maximum difference found
    pub max_diff: u32,
    /// Number of pixels that differ
    pub diff_count: u64,
}

/// Compare type for pixel operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareType {
    /// Subtract: pix1 - pix2, clipped to 0
    Subtract,
    /// Absolute difference: |pix1 - pix2|
    AbsDiff,
}

impl Pix {
    /// Check if two images have identical pixel values
    ///
    /// Ignores alpha channel for 32-bit images unless compare_alpha is true.
    pub fn equals(&self, other: &Pix) -> bool;

    /// Check if two images are equal, optionally comparing alpha
    pub fn equals_with_alpha(&self, other: &Pix, compare_alpha: bool) -> bool;

    /// Compute difference image: self - other or |self - other|
    ///
    /// For grayscale images, returns grayscale result.
    /// For color images, computes per-channel difference.
    pub fn diff(&self, other: &Pix, compare_type: CompareType) -> Result<Pix>;

    /// Compute RMS (root mean square) difference between two images
    pub fn rms_diff(&self, other: &Pix) -> Result<f64>;

    /// Compute mean absolute difference between two images
    pub fn mean_abs_diff(&self, other: &Pix) -> Result<f64>;

    /// Compute full comparison statistics
    pub fn compare(&self, other: &Pix) -> Result<CompareResult>;
}

/// Binary image correlation (for 1-bit images)
///
/// Returns correlation value between 0.0 (no overlap) and 1.0 (identical)
/// Formula: (|A AND B|)^2 / (|A| * |B|)
pub fn correlation_binary(pix1: &Pix, pix2: &Pix) -> Result<f64>;
```

## Implementation Details

### Module Structure

```text
crates/leptonica-core/src/pix/
├── mod.rs       # Add: mod compare; pub use compare::*;
├── access.rs    # Existing
├── convert.rs   # Existing
├── ops.rs       # Existing (may move some to compare.rs)
└── compare.rs   # NEW: Image comparison functions
```

### Algorithm: equals()

```text
1. Check dimensions match (width, height, depth)
2. If different dimensions, return false
3. For binary images:
   - XOR the two images word-by-word
   - If any word is non-zero, return false
4. For grayscale/color:
   - Compare pixel by pixel
   - For 32bpp, mask out alpha unless compare_alpha=true
5. Return true if all pixels match
```

### Algorithm: diff()

```text
For each pixel (x, y):
    val1 = pix1.get_pixel(x, y)
    val2 = pix2.get_pixel(x, y)

    if CompareType::Subtract:
        result = max(0, val1 - val2)
    else if CompareType::AbsDiff:
        result = abs(val1 - val2)

    output.set_pixel(x, y, result)
```

### Algorithm: rms_diff()

```text
sum_sq = 0.0
count = 0

For each pixel (x, y):
    diff = |pix1[x,y] - pix2[x,y]|
    sum_sq += diff * diff
    count += 1

rms = sqrt(sum_sq / count)
```

### Algorithm: correlation_binary()

```text
count1 = count_foreground_pixels(pix1)
count2 = count_foreground_pixels(pix2)

if count1 == 0 || count2 == 0:
    return 0.0

pix_and = pix1 AND pix2
count_and = count_foreground_pixels(pix_and)

correlation = (count_and * count_and) / (count1 * count2)
```

### Error Handling

- Dimension mismatch: Return error with details
- Unsupported depth combination: Return error
- Zero-size images: Return error

### Performance Considerations

1. **Word-level operations**: For 1-bit images, use u32 XOR instead of pixel-by-pixel
2. **Early termination**: For `equals()`, stop at first difference
3. **SIMD potential**: Consider SIMD for large images (future optimization)

## Tasks

1. [x] Create implementation plan
2. [x] Create `src/pix/compare.rs` with module structure
3. [x] Implement `equals()` and `equals_with_alpha()`
4. [x] Implement `diff()` for Subtract and AbsDiff
5. [x] Implement `rms_diff()` and `mean_abs_diff()`
6. [x] Implement `correlation_binary()`
7. [x] Implement `compare()` (full statistics)
8. [x] Add unit tests for all functions
9. [x] Update `mod.rs` with module and re-exports
10. [x] Run `cargo fmt && cargo clippy`
11. [x] Run full test suite

## Test Plan

1. **Identity tests**:
   - Same image equals itself
   - Same image has 0 RMS diff

2. **Different images**:
   - Known pixel differences produce expected diff values
   - Different dimensions return appropriate error

3. **Edge cases**:
   - Empty images (0x0 or 1x1)
   - Maximum difference (0 vs 255)
   - Binary images with various overlaps

4. **Color images**:
   - Per-channel comparison works correctly
   - Alpha handling (with and without)

5. **Performance**:
   - Large images complete in reasonable time

## Questions

(None at this time)

## Estimates

- Implementation: ~300 lines of code
- Time: ~3 hours
