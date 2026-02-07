# FPix Implementation Plan

## Overview

FPix (Floating-Point Image) is a specialized image container that stores
pixel values as `f32` floating-point numbers. This is useful for intermediate
computations in image processing where integer precision is insufficient.

## Reference

- C source: `reference/leptonica/src/fpix1.c`, `fpix2.c`
- Rust patterns: `Pix` (Arc-based immutable + PixMut), `Numa` (simple Vec wrapper)

## Design Decisions

### 1. Architecture

FPix will follow a simpler pattern than Pix:

- Use `Vec<f32>` for data storage (no Arc, as FPix is typically intermediate data)
- Provide both immutable and mutable access
- Clone creates a deep copy (no reference counting for simplicity)

### 2. Structure

```rust
pub struct FPix {
    width: u32,
    height: u32,
    data: Vec<f32>,
    xres: i32,       // X resolution (ppi)
    yres: i32,       // Y resolution (ppi)
}
```

Note: Unlike C Leptonica's `wpl` field, we store data contiguously without
row padding. The width serves directly as the "words per line" equivalent.

### 3. Key Methods

#### Creation/Copy

- `FPix::new(width, height)` - Create with zeros
- `FPix::new_with_value(width, height, value)` - Create with initial value
- `FPix::from_pix(pix)` - Convert from Pix (8-bit grayscale only initially)
- `clone()` - Deep copy

#### Accessors

- `width()`, `height()`, `dimensions()` - Size getters
- `xres()`, `yres()`, `resolution()` - Resolution getters/setters
- `get_pixel(x, y)` - Get pixel value at (x, y)
- `set_pixel(x, y, val)` - Set pixel value at (x, y)
- `data()`, `data_mut()` - Raw data access

#### Conversion

- `to_pix(depth, neg_handling)` - Convert to Pix
- `FPix::from_pix(pix)` - Convert from Pix

#### Arithmetic Operations

- `add(&self, other: &FPix)` - Element-wise addition
- `sub(&self, other: &FPix)` - Element-wise subtraction
- `mul(&self, other: &FPix)` - Element-wise multiplication
- `div(&self, other: &FPix)` - Element-wise division
- `add_constant(&mut self, val)` - Add constant to all pixels
- `mul_constant(&mut self, val)` - Multiply all pixels by constant

#### Statistics

- `min()` - Find minimum value and location
- `max()` - Find maximum value and location
- `mean()` - Calculate mean value

## Implementation Steps

### Phase 1: Core Structure

1. Create `crates/leptonica-core/src/fpix/mod.rs`
2. Implement `FPix` struct with basic fields
3. Implement creation methods (`new`, `new_with_value`)
4. Implement accessors (`width`, `height`, `get_pixel`, `set_pixel`)

### Phase 2: Conversion

1. Implement `FPix::from_pix()` for 8-bit grayscale
2. Implement `to_pix()` with depth selection and negative value handling
3. Add error handling for unsupported depths

### Phase 3: Arithmetic

1. Implement element-wise operations (`add`, `sub`, `mul`, `div`)
2. Implement constant operations (`add_constant`, `mul_constant`)
3. Use `std::ops` traits for operator overloading where appropriate

### Phase 4: Statistics

1. Implement `min()` with location
2. Implement `max()` with location
3. Implement `mean()`

### Phase 5: Integration

1. Add `pub mod fpix;` to lib.rs
2. Export public types
3. Add tests

## Error Handling

Use `crate::error::Error` for:

- `InvalidDimension` - Zero width/height
- `IncompatibleSizes` - Size mismatch in arithmetic
- `UnsupportedDepth` - Unsupported Pix depth for conversion
- `IndexOutOfBounds` - Invalid pixel coordinates

## Negative Value Handling (for to_pix)

```rust
pub enum NegativeHandling {
    ClipToZero,   // Set negative values to 0
    TakeAbsValue, // Use absolute value
}
```

## Test Plan

1. Creation tests
   - `new()` creates zeroed image
   - `new_with_value()` creates with correct value
   - Invalid dimensions return error

2. Pixel access tests
   - `get_pixel`/`set_pixel` work correctly
   - Out-of-bounds access returns error

3. Conversion tests
   - Round-trip: Pix -> FPix -> Pix preserves values
   - Negative handling works correctly

4. Arithmetic tests
   - Element-wise operations produce correct results
   - Constant operations work correctly
   - Size mismatch returns error

5. Statistics tests
   - `min()` returns correct value and location
   - `max()` returns correct value and location
   - `mean()` returns correct average

## Questions

None at this time. The design follows established patterns from Pix and Numa.

## Files to Create/Modify

- **Create**: `crates/leptonica-core/src/fpix/mod.rs`
- **Modify**: `crates/leptonica-core/src/lib.rs` (add fpix module and exports)
- **Modify**: `crates/leptonica-core/src/error.rs` (if new error variants needed)
