# Coloring Implementation Plan

## Overview

Implement coloring functionality for grayscale and binary images based on Leptonica's
`coloring.c`. This module provides functions to colorize grayscale pixels, snap
colors to target values, and perform fractional color shifts.

## Analysis of C Implementation

The C version (`reference/leptonica/src/coloring.c`) provides:

### Core Functions

1. **pixColorGrayRegions()** - Colorize gray pixels in specified regions (boxes)
2. **pixColorGray()** - In-place colorization of gray pixels
3. **pixColorGrayMasked()** - Colorize gray pixels under a mask

4. **pixSnapColor()** - Snap colors within a diff to a target color
5. **pixSnapColorCmap()** - Snap colors in colormapped images

6. **pixLinearMapToTargetColor()** - Piecewise linear color mapping
7. **pixelLinearMapToTargetColor()** - Single pixel linear mapping

8. **pixShiftByComponent()** - Fractional shift RGB toward black/white
9. **pixelShiftByComponent()** - Single pixel fractional shift
10. **pixelFractionalShift()** - Fractional shift preserving hue
11. **pixMapWithInvariantHue()** - Map colors preserving hue

### Key Concepts

- **Paint Type**: `L_PAINT_LIGHT` (colorize light pixels) vs `L_PAINT_DARK`
  (colorize dark pixels)
- **Threshold**: Average pixel value threshold for colorization
- **Color Snapping**: Force pixels within a diff range to a target color
- **Fractional Shift**: Linear transformation shifting toward black (negative)
  or white (positive)
- **Hue Invariant**: Transform that changes saturation/brightness but
  preserves hue

## Design Decisions

### Scope

Focus on RGB image coloring operations:

1. `pix_color_gray()` - Colorize gray pixels in RGB images
2. `pix_color_gray_masked()` - Colorize with mask
3. `pix_snap_color()` - Snap colors to target
4. `pix_linear_map_to_target_color()` - Piecewise linear mapping
5. `pix_shift_by_component()` - Fractional shift toward black/white
6. `pix_map_with_invariant_hue()` - Hue-preserving transformation
7. Pixel-level helper functions

### Note on Colormap Support

Colormap-based coloring is deferred:

- `pixColorGrayCmap`, `pixColorGrayRegionsCmap` require colormap manipulation
- These can be added later when colormap infrastructure is more mature

### API Design

```rust
/// Paint type for gray colorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintType {
    /// Colorize light (non-black) pixels
    Light,
    /// Colorize dark (non-white) pixels
    Dark,
}

/// Options for color_gray operation
#[derive(Debug, Clone)]
pub struct ColorGrayOptions {
    pub paint_type: PaintType,
    /// Threshold: pixels with avg below (Light) or above (Dark) are skipped
    pub threshold: u8,
    /// Target color (r, g, b)
    pub target_color: (u8, u8, u8),
}

/// Colorize gray pixels in a 32-bit RGB image
///
/// For `PaintType::Light`: colorizes pixels with average > threshold
/// For `PaintType::Dark`: colorizes pixels with average < threshold
pub fn pix_color_gray(
    pix: &Pix,
    region: Option<&Box>,
    options: &ColorGrayOptions,
) -> ColorResult<Pix>

/// Colorize gray pixels under a mask
pub fn pix_color_gray_masked(
    pix: &Pix,
    mask: &Pix,
    options: &ColorGrayOptions,
) -> ColorResult<Pix>

/// Snap colors within diff to a target color
pub fn pix_snap_color(
    pix: &Pix,
    src_color: u32,  // 0xRRGGBB00 format
    dst_color: u32,  // 0xRRGGBB00 format
    diff: u8,
) -> ColorResult<Pix>

/// Piecewise linear mapping from source to target color
pub fn pix_linear_map_to_target_color(
    pix: &Pix,
    src_color: u32,  // 0xRRGGBB00 format
    dst_color: u32,  // 0xRRGGBB00 format
) -> ColorResult<Pix>

/// Fractional shift of RGB toward black or white
pub fn pix_shift_by_component(
    pix: &Pix,
    src_color: u32,  // 0xRRGGBB00 format
    dst_color: u32,  // 0xRRGGBB00 format
) -> ColorResult<Pix>

/// Map colors with invariant hue (changes saturation/brightness only)
pub fn pix_map_with_invariant_hue(
    pix: &Pix,
    src_color: u32,    // 0xRRGGBB00 format
    fract: f32,        // -1.0 (black) to 1.0 (white)
) -> ColorResult<Pix>

// Pixel-level helpers
pub fn pixel_linear_map_to_target_color(
    pixel: u32,
    src_map: u32,
    dst_map: u32,
) -> u32

pub fn pixel_shift_by_component(
    r: u8, g: u8, b: u8,
    src_color: u32,
    dst_color: u32,
) -> (u8, u8, u8)

pub fn pixel_fractional_shift(
    r: u8, g: u8, b: u8,
    fract: f32,
) -> ColorResult<(u8, u8, u8)>
```

## Implementation Steps

### Step 1: Add Module Structure

- Create `crates/leptonica-color/src/coloring.rs`
- Add module to `lib.rs`
- Define types (PaintType, ColorGrayOptions)

### Step 2: Implement Pixel-Level Functions

```rust
fn pixel_fractional_shift(
    r: u8, g: u8, b: u8, fract: f32
) -> ColorResult<(u8, u8, u8)>

fn pixel_shift_by_component(
    r: u8, g: u8, b: u8, src: u32, dst: u32
) -> (u8, u8, u8)

fn pixel_linear_map_to_target_color(
    pixel: u32, src_map: u32, dst_map: u32
) -> u32
```

### Step 3: Implement Core Image Functions

1. `pix_color_gray()` - Main colorization function
2. `pix_color_gray_masked()` - Masked colorization
3. `pix_snap_color()` - Color snapping
4. `pix_linear_map_to_target_color()` - Linear mapping
5. `pix_shift_by_component()` - Component shift
6. `pix_map_with_invariant_hue()` - Hue-invariant mapping

### Step 4: Add Tests

- Unit tests for pixel-level functions
- Integration tests for image operations
- Edge cases: boundaries, all white/black images

## File Structure

```text
crates/leptonica-color/src/
  coloring.rs   # New file
  lib.rs        # Add module and re-exports
```

## Dependencies

- `leptonica-core`: Pix, PixMut, Box, color utilities
- No new external dependencies

## Algorithm Details

### Color Gray Algorithm

For each pixel:

1. Calculate average: `avg = (r + g + b) / 3`
2. For `Light`: if `avg > threshold`, colorize
3. For `Dark`: if `avg < threshold`, colorize

Colorization formula:

- Light: `new_r = target_r * avg / 255`
- Dark: `new_r = target_r + (255 - target_r) * avg / 255`

### Linear Map Algorithm

For each component independently:

- If `pixel_val <= src_val`: `new_val = pixel_val * dst_val / src_val`
- If `pixel_val > src_val`:
  `new_val = dst_val + (255 - dst_val) * (pixel_val - src_val) / (255 - src_val)`

### Fractional Shift Algorithm

For each component:

- If `fract < 0`: `new_val = (1 + fract) * val` (toward black)
- If `fract >= 0`: `new_val = val + fract * (255 - val)` (toward white)

## Test Cases

1. **Color Gray Light**: White image with gray text -> colored text
2. **Color Gray Dark**: Gray image with white background -> colored background
3. **Color Gray Masked**: Selective colorization with mask
4. **Snap Color**: Replace near-white with pure white
5. **Linear Map**: Warm/cool color shift
6. **Shift By Component**: Darken/lighten image
7. **Invariant Hue**: Saturation boost without hue change
8. **Edge Cases**: Empty region, all same color

## Implementation Timeline

1. Types and module setup (10 min)
2. Pixel-level functions (20 min)
3. `pix_color_gray` and `pix_color_gray_masked` (25 min)
4. `pix_snap_color` (15 min)
5. `pix_linear_map_to_target_color` (20 min)
6. `pix_shift_by_component` (15 min)
7. `pix_map_with_invariant_hue` (10 min)
8. Tests (25 min)

Total: ~2.5 hours

## Questions

None at this time. The C implementation is well-documented and the algorithm
is straightforward.
