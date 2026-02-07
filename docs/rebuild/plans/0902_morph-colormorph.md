# Color Morphology Implementation Plan

## Overview

Implement color morphological operations for 32-bpp RGB images in the
`leptonica-morph` crate. This corresponds to `colormorph.c` in the
original Leptonica C library.

## Reference

- C source: `reference/leptonica/src/colormorph.c`
- Algorithm: Apply grayscale morphological operations separately to each RGB channel
- Existing pattern: `crates/leptonica-morph/src/grayscale.rs`

## Scope

### Functions to Implement

| C Function      | Rust Function    | Description                       |
| --------------- | ---------------- | --------------------------------- |
| `pixColorMorph` | `dilate_color`   | Color dilation with brick SE      |
| `pixColorMorph` | `erode_color`    | Color erosion with brick SE       |
| `pixColorMorph` | `open_color`     | Color opening                     |
| `pixColorMorph` | `close_color`    | Color closing                     |

### Additional Functions

- `gradient_color`: Morphological gradient for color images
- `top_hat_color`: Top-hat transform for color images
- `bottom_hat_color`: Bottom-hat transform for color images

### Implementation Approach

The approach follows Leptonica's strategy:

1. Extract RGB channels as separate 8-bpp grayscale images
2. Apply grayscale morphology to each channel independently
3. Recombine the processed channels into a 32-bpp image

This leverages the existing `grayscale.rs` implementation.

## Design

### File Structure

```text
crates/leptonica-morph/src/
  lib.rs          # Add pub mod color; and re-exports
  color.rs        # NEW: Color morphology implementation
  grayscale.rs    # Existing grayscale morphology
  binary.rs       # Existing binary morphology
```

### API Design

```rust
// crates/leptonica-morph/src/color.rs

/// Dilate a color image with a brick structuring element
///
/// Applies grayscale dilation separately to R, G, B channels.
pub fn dilate_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Erode a color image with a brick structuring element
///
/// Applies grayscale erosion separately to R, G, B channels.
pub fn erode_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Open a color image (erosion followed by dilation)
pub fn open_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Close a color image (dilation followed by erosion)
pub fn close_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Morphological gradient for color images
pub fn gradient_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Top-hat transform for color images
pub fn top_hat_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// Bottom-hat transform for color images
pub fn bottom_hat_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;
```

### Parameters

- `pix`: 32-bpp color image
- `hsize`: Horizontal size of brick SE (auto-incremented if even)
- `vsize`: Vertical size of brick SE (auto-incremented if even)

## Implementation Details

### Helper Functions

```rust
/// Check that the image is 32-bpp color
fn check_color(pix: &Pix) -> MorphResult<()>;

/// Extract a single RGB channel as an 8-bpp grayscale image
fn extract_channel(pix: &Pix, channel: ColorChannel) -> MorphResult<Pix>;

/// Combine R, G, B channels into a 32-bpp color image
fn combine_rgb(r: &Pix, g: &Pix, b: &Pix) -> MorphResult<Pix>;
```

### Color Channel Enum

```rust
/// Color channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}
```

### Color Dilation Algorithm

```rust
pub fn dilate_color(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_color(pix)?;

    // Identity case
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // Extract channels
    let r = extract_channel(pix, ColorChannel::Red)?;
    let g = extract_channel(pix, ColorChannel::Green)?;
    let b = extract_channel(pix, ColorChannel::Blue)?;

    // Apply grayscale dilation to each channel
    let r_dilated = dilate_gray(&r, hsize, vsize)?;
    let g_dilated = dilate_gray(&g, hsize, vsize)?;
    let b_dilated = dilate_gray(&b, hsize, vsize)?;

    // Combine back into color image
    combine_rgb(&r_dilated, &g_dilated, &b_dilated)
}
```

### Other Operations

- `erode_color`: Same pattern using `erode_gray`
- `open_color`: `erode_color` then `dilate_color`
- `close_color`: `dilate_color` then `erode_color`
- `gradient_color`: `dilate_color - erode_color` (per channel)
- `top_hat_color`: `original - open_color` (per channel)
- `bottom_hat_color`: `close_color - original` (per channel)

## Test Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dilate_color_identity() {
        // hsize=1, vsize=1 should return identical image
    }

    #[test]
    fn test_dilate_color_expands_bright() {
        // Bright pixels should expand in all channels
    }

    #[test]
    fn test_erode_color_shrinks_bright() {
        // Bright region should shrink in all channels
    }

    #[test]
    fn test_open_close_color() {
        // Opening and closing preserve overall structure
    }

    #[test]
    fn test_color_only() {
        // Non-32bpp images should return error
    }

    #[test]
    fn test_channel_extraction() {
        // Verify correct channel extraction
    }

    #[test]
    fn test_channel_combination() {
        // Verify correct channel recombination
    }

    #[test]
    fn test_gradient_color() {
        // Gradient should highlight edges
    }

    #[test]
    fn test_top_hat_bottom_hat_color() {
        // Top-hat and bottom-hat transforms
    }
}
```

## Status

- [x] Create feature branch
- [x] Create plan file
- [x] Create `color.rs` with helper functions
- [x] Implement channel extraction/combination
- [x] Implement `dilate_color`
- [x] Implement `erode_color`
- [x] Implement `open_color` and `close_color`
- [x] Implement gradient, top-hat, bottom-hat
- [x] Add unit tests
- [x] Update `lib.rs` with exports
- [x] Run `cargo fmt && cargo clippy`
- [x] Run tests
- [x] Commit changes

## Questions

None at this time.

## Future Enhancements

- Alpha channel preservation option
- Support for non-brick SEs (would require different channel processing)
- SIMD optimization for channel extraction/combination
