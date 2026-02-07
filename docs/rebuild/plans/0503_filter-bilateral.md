# Bilateral Filter Implementation Plan

## Status: IMPLEMENTED

## Summary

バイラテラルフィルタ（エッジ保存平滑化フィルタ）をleptonica-filterクレートに実装する。

## Reference

- C version: `reference/leptonica/src/bilateral.c`
- Existing patterns: `crates/leptonica-filter/src/convolve.rs`, `src/edge.rs`

## Background

バイラテラルフィルタは、エッジを保存しながらノイズを除去する非線形フィルタ。
2つのガウシアンカーネルを組み合わせる:

1. **空間カーネル** (spatial): ピクセル間の距離に基づく重み
2. **レンジカーネル** (range): ピクセル値の差に基づく重み

## Implementation Scope

### Phase 1: Exact Implementation (pixBilateralExact相当)

より直感的で理解しやすい厳密実装から開始:

- `bilateral_gray_exact()`: 8bppグレースケール用
- `bilateral_exact()`: 8bpp/32bpp自動ディスパッチ
- `bilateral_block_exact()`: 標準偏差パラメータ版

### Phase 2: Range Kernel Helper

- `make_range_kernel()`: レンジカーネル生成

### Future (not in scope)

- 高速近似実装（pixBilateral相当） - 複雑なため別PR

## API Design

```rust
// bilateral.rs module

/// Parameters for bilateral filtering
pub struct BilateralParams {
    /// Spatial standard deviation (in pixels)
    pub spatial_stdev: f32,
    /// Range standard deviation (grayscale difference weighting)
    pub range_stdev: f32,
}

/// Apply bilateral filter using exact (slow) method
///
/// # Arguments
/// * `pix` - 8bpp grayscale or 32bpp color image
/// * `spatial_stdev` - Standard deviation for spatial Gaussian (> 0.0)
/// * `range_stdev` - Standard deviation for range Gaussian (> 0.0)
pub fn bilateral_exact(pix: &Pix, spatial_stdev: f32, range_stdev: f32) -> FilterResult<Pix>;

/// Apply bilateral filter to grayscale image
pub fn bilateral_gray_exact(
    pix: &Pix,
    spatial_kernel: &Kernel,
    range_kernel: Option<&[f32; 256]>,
) -> FilterResult<Pix>;

/// Create a range kernel (256-element Gaussian based on intensity difference)
pub fn make_range_kernel(range_stdev: f32) -> FilterResult<[f32; 256]>;
```

## Implementation Details

### Algorithm (Exact Method)

```text
For each pixel (x, y):
    center_val = image[x, y]
    sum = 0.0
    weight_sum = 0.0

    For each neighbor (nx, ny) in spatial kernel:
        neighbor_val = image[nx, ny]
        spatial_weight = spatial_kernel[nx-x, ny-y]
        range_weight = range_kernel[|center_val - neighbor_val|]
        weight = spatial_weight * range_weight

        sum += neighbor_val * weight
        weight_sum += weight

    result[x, y] = sum / weight_sum
```

### Modules Structure

```text
src/
├── lib.rs           # Add: pub mod bilateral; + re-exports
├── bilateral.rs     # NEW: Bilateral filter implementation
├── convolve.rs      # Existing
├── edge.rs          # Existing
├── error.rs         # Existing
└── kernel.rs        # Existing (may add gaussian_1d helper)
```

### Error Handling

- `spatial_stdev <= 0.0` -> InvalidParameters
- `range_stdev <= 0.0` -> InvalidParameters
- Unsupported depth -> UnsupportedDepth
- Image too small for kernel -> return copy with warning (match C behavior)

### 32bpp Color Support

- R, G, B channels processed independently
- Alpha channel preserved (or averaged similarly)

## Tasks

1. [ ] Create `src/bilateral.rs` with module structure
2. [ ] Implement `make_range_kernel()`
3. [ ] Implement `bilateral_gray_exact()` for 8bpp
4. [ ] Implement `bilateral_exact()` for 8bpp/32bpp dispatch
5. [ ] Add unit tests
6. [ ] Update `lib.rs` with module and re-exports
7. [ ] Run `cargo fmt && cargo clippy`
8. [ ] Run full test suite

## Test Plan

1. Identity test: With range_stdev = infinity, should approximate Gaussian blur
2. Edge preservation: Sharp edges should remain sharp with low range_stdev
3. Small image handling: Images smaller than kernel return copy
4. Color image: Each channel processed correctly
5. Parameter validation: Invalid params return errors

## Questions

(None at this time)

## Estimates

- Implementation: ~200 lines of code
- Time: ~2 hours
