# Grayscale Morphology Implementation Plan

## Overview

Implement grayscale morphological operations for 8-bpp images in the
`leptonica-morph` crate. This corresponds to `graymorph.c` in the
original Leptonica C library.

## Reference

- C source: `reference/leptonica/src/graymorph.c`
- Algorithm: van Herk / Gil-Werman (vHGW) algorithm
- Existing pattern: `crates/leptonica-morph/src/binary.rs`

## Scope

### Functions to Implement

| C Function      | Rust Function  | Description                      |
| --------------- | -------------- | -------------------------------- |
| `pixDilateGray` | `dilate_gray`  | Grayscale dilation with brick SE |
| `pixErodeGray`  | `erode_gray`   | Grayscale erosion with brick SE  |
| `pixOpenGray`   | `open_gray`    | Grayscale opening                |
| `pixCloseGray`  | `close_gray`   | Grayscale closing                |

### Implementation Approach

Two approaches are available:

1. **Naive approach**: Simple min/max over SE neighborhood
2. **vHGW approach**: van Herk / Gil-Werman algorithm

For initial implementation, use the **naive approach** for simplicity
and correctness. The vHGW optimization can be added later.

## Design

### File Structure

```text
crates/leptonica-morph/src/
  lib.rs          # Add pub mod grayscale; and re-exports
  grayscale.rs    # NEW: Grayscale morphology implementation
```

### API Design

```rust
// crates/leptonica-morph/src/grayscale.rs

/// Dilate a grayscale image with a brick structuring element
///
/// Dilation computes the maximum pixel value in the SE neighborhood.
/// Border pixels are handled by treating out-of-bounds pixels as 0.
pub fn dilate_gray(pix: &Pix, hsize: u32, vsize: u32)
  -> MorphResult<Pix>;

/// Erode a grayscale image with a brick structuring element
///
/// Erosion computes the minimum pixel value in the SE neighborhood.
/// Border pixels are handled by treating out-of-bounds pixels as 255.
pub fn erode_gray(pix: &Pix, hsize: u32, vsize: u32)
  -> MorphResult<Pix>;

/// Open a grayscale image (erosion followed by dilation)
pub fn open_gray(pix: &Pix, hsize: u32, vsize: u32)
  -> MorphResult<Pix>;

/// Close a grayscale image (dilation followed by erosion)
pub fn close_gray(pix: &Pix, hsize: u32, vsize: u32)
  -> MorphResult<Pix>;
```

### Parameters

- `pix`: 8-bpp grayscale image
- `hsize`: Horizontal size of brick SE (auto-incremented if even)
- `vsize`: Vertical size of brick SE (auto-incremented if even)

## Implementation Details

### Grayscale Dilation Algorithm

```rust
fn dilate_gray(pix: &Pix, hsize: u32, vsize: u32)
  -> MorphResult<Pix> {
    check_grayscale(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize);

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    let w = pix.width();
    let h = pix.height();
    let half_h = (hsize / 2) as i32;
    let half_v = (vsize / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut max_val: u8 = 0;

            for dy in -half_v..=half_v {
                for dx in -half_h..=half_h {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;

                    if sx >= 0 && sx < w as i32
                        && sy >= 0 && sy < h as i32 {
                        let val = pix.get_pixel(sx as u32, sy as u32)
                          as u8;
                        max_val = max_val.max(val);
                    }
                }
            }

            out_mut.set_pixel(x, y, max_val as u32);
        }
    }

    Ok(out_mut.into())
}
```

### Grayscale Erosion Algorithm

Similar to dilation but:

- Computes minimum instead of maximum
- Out-of-bounds pixels treated as 255 (maximum value)

### Separability Optimization

For brick SEs, operations can be separated into horizontal and
vertical passes:

- `dilate_gray(3, 5)` = `dilate_gray_h(3)` then `dilate_gray_v(5)`

This is optional for initial implementation but provides 2x speedup.

## Test Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dilate_gray_identity() {
        // hsize=1, vsize=1 should return identical image
    }

    #[test]
    fn test_dilate_gray_expands_bright() {
        // Single bright pixel should expand
    }

    #[test]
    fn test_erode_gray_shrinks_bright() {
        // Bright region should shrink
    }

    #[test]
    fn test_open_removes_small_bright() {
        // Small bright features should be removed
    }

    #[test]
    fn test_close_fills_small_dark() {
        // Small dark features should be filled
    }

    #[test]
    fn test_grayscale_only() {
        // Non-8bpp images should return error
    }

    #[test]
    fn test_even_size_incremented() {
        // Even sizes should be auto-incremented to odd
    }
}
```

## Status

All implementation complete and tested.

- [x] Create feature branch
- [x] Create `grayscale.rs` with check functions
- [x] Implement `dilate_gray`
- [x] Implement `erode_gray`
- [x] Implement `open_gray` and `close_gray`
- [x] Add unit tests
- [x] Update `lib.rs` with exports
- [x] Run `cargo fmt && cargo clippy`
- [x] Run tests
- [x] Commit changes

## Additional Features

The implementation also includes:

- `gradient_gray`: Morphological gradient (dilation - erosion)
- `top_hat_gray`: Top-hat transform (original - opening)
- `bottom_hat_gray`: Bottom-hat transform (closing - original)

## Future Enhancements

- van Herk / Gil-Werman optimization for O(1) complexity
- 3x3 special case optimization (pixErodeGray3, pixDilateGray3)
- Support for non-brick SEs
