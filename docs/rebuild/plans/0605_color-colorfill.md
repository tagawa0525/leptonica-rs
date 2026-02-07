# Color Fill Implementation Plan

## Overview

Implement color fill functionality for RGB images based on Leptonica's `colorfill.c`.

## Analysis of C Implementation

The C version (`reference/leptonica/src/colorfill.c`) provides:

### Core Functions

1. **l_colorfillCreate() / l_colorfillDestroy()** - Struct management for
   tile-based color analysis
2. **pixColorContentByLocation()** - Analyzes color content using
   proximity-based region growing
3. **pixColorFill()** - Main color filling operation using 8-connected
   flood fill
4. **makeColorfillTestData()** - Test data generation with seed-based
   region creation

### Helper Functions (static)

- `colorelCreate()` - Create color element for queue
- `pixColorFillFromSeed()` - Flood fill from seed point
- `pixGetVisitedNeighbors()` - Get 8-connected neighbor visit status
- `findNextUnvisited()` - Find next unvisited pixel in raster order
- `colorsAreSimilarForFill()` - Color similarity check for fill propagation
- `pixelColorIsValid()` - Check if pixel has valid color (not too dark)
- `pixelIsOnColorBoundary()` - Check if pixel is on color boundary
- `evalColorfillData()` - Evaluate color components in filled regions

## Design Decisions

### Scope

Focus on the core color fill algorithm suitable for general use:

1. `color_fill()` - Main entry point for color region detection
2. `color_fill_from_seed()` - Single seed flood fill for similar colors
3. Helper types and functions

The tile-based `L_COLORFILL` structure is specific to color content
analysis and can be implemented later if needed.

### Relationship with Existing seedfill.rs

- `seedfill.rs` handles binary (1-bit) and grayscale (8-bit) images
- `colorfill.rs` handles RGB (32-bit) images with color similarity
- Different use cases: binary flood fill vs. color region detection

### API Design

```rust
// Main API
pub fn color_fill(pix: &Pix, options: &ColorFillOptions) -> ColorResult<Pix>
pub fn color_fill_from_seed(
    pix: &Pix,
    seed_x: u32,
    seed_y: u32,
    options: &ColorFillOptions,
) -> ColorResult<ColorFillResult>

// Options
pub struct ColorFillOptions {
    pub min_max: u32,       // Min max component for valid color (default: 70)
    pub max_diff: u32,      // Max color diff to be in same region (default: 40)
    pub min_area: u32,      // Min pixels for a region (default: 100)
    pub connectivity: Connectivity, // 4-way or 8-way (default: 8-way)
    pub smooth: u32,        // Low-pass kernel size (1,3,5; 1 = skip)
}

// Result types
pub struct ColorFillResult {
    pub mask: Pix,          // 1-bit mask of filled region
    pub pixel_count: u32,   // Number of pixels in region
    pub avg_color: (u8, u8, u8), // Average RGB color of region
}

// Connectivity type (reuse from region crate or define locally)
pub enum Connectivity {
    FourWay,
    EightWay,
}
```

## Implementation Steps

### Step 1: Add Module Structure

- Create `crates/leptonica-color/src/colorfill.rs`
- Add module to `lib.rs`
- Define types and error variants

### Step 2: Implement Color Similarity

```rust
fn colors_are_similar(val1: u32, val2: u32, max_diff: u32) -> bool
fn pixel_color_is_valid(val: u32, min_max: u32) -> bool
```

### Step 3: Implement Queue-based Flood Fill

- Use `VecDeque` for BFS queue (similar to C's L_QUEUE)
- Track visited pixels with 1-bit mask
- Collect filled pixel coordinates

### Step 4: Implement Main API

- `color_fill_from_seed()` - Single seed fill
- `color_fill()` - Full image fill finding all regions

### Step 5: Add Tests

- Unit tests for color similarity
- Integration tests for fill operations
- Edge cases: empty regions, boundary conditions

## File Structure

```text
crates/leptonica-color/src/
  colorfill.rs   # New file
  lib.rs         # Add module and re-exports
```

## Dependencies

- `leptonica-core`: Pix, PixMut, color utilities
- `std::collections::VecDeque` for queue

No new external dependencies required.

## Test Cases

1. **Basic Fill**: Single color region
2. **Similar Colors**: Region with gradient
3. **Multiple Regions**: Image with distinct color areas
4. **Boundary Conditions**: Edge and corner seeds
5. **Min Area Filter**: Reject small regions
6. **Invalid Colors**: Dark pixel handling

## Implementation Timeline

1. Types and structure (15 min)
2. Helper functions (20 min)
3. Seed fill implementation (30 min)
4. Full image fill (15 min)
5. Tests (20 min)

Total: ~2 hours

## Questions

None at this time. The scope is clear and the implementation approach is
well-defined based on the C reference.
