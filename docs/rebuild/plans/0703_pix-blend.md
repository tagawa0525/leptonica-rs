# Image Blending Implementation Plan

## Status: IMPLEMENTED

## Summary

画像合成/ブレンド機能をleptonica-coreクレートに実装する。
C版Leptonicaのblend.cに相当する機能を、Rust idiomaticなAPIで提供する。

## Reference

- C version: `reference/leptonica/src/blend.c`
- Existing patterns: `crates/leptonica-core/src/pix/compare.rs`

## Background

画像ブレンディングは画像処理の重要な操作であり、以下の用途がある:

- 透かしの合成
- 画像のフェードイン/フェードアウト
- レイヤー合成
- 特殊効果（ハードライト、スクリーン、オーバーレイなど）

## Implementation Scope

### Core Functions (Phase 1)

1. **アルファブレンド（pixBlendColor相当）**
   - カラー画像同士のブレンド
   - 式: `result = (1 - fract) * base + fract * blend`

2. **グレースケールブレンド（pixBlendGray相当）**
   - グレースケール画像を使ったブレンド
   - L_BLEND_GRAY: 単純な線形補間
   - L_BLEND_GRAY_WITH_INVERSE: 逆数を使った補間

3. **マスクを使用した合成（pixBlendMask相当）**
   - 1bitマスクでブレンド領域を指定
   - L_BLEND_WITH_INVERSE: 反転ブレンド
   - L_BLEND_TO_WHITE: 白方向へのフェード
   - L_BLEND_TO_BLACK: 黒方向へのフェード

4. **乗算ブレンド**
   - 式: `result = (base * blend) / 255`
   - 黒は変化なし、白は元画像そのまま

5. **スクリーンブレンド**
   - 式: `result = 255 - ((255 - base) * (255 - blend)) / 255`
   - 白は変化なし、黒は元画像そのまま

6. **オーバーレイブレンド**
   - base < 128: `result = 2 * base * blend / 255`
   - base >= 128: `result = 255 - 2 * (255 - base) * (255 - blend) / 255`

7. **ハードライトブレンド（pixBlendHardLight相当）**
   - オーバーレイの反対（blendの値でスイッチ）

### Phase 2 (Future)

- `pixBlendWithGrayMask`: アルファマスクを使った合成
- `pixBlendBackgroundToColor`: 背景色への合成
- `pixFadeWithGray`: グレースケールでのフェード
- `pixAlphaBlendUniform`: 一様背景へのアルファブレンド

## API Design

```rust
// crates/leptonica-core/src/pix/blend.rs

use crate::{Pix, PixelDepth, Result};

/// Mask blend type for 1-bit mask blending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskBlendType {
    /// Blend with inverse: p -> (1-f)*p + f*(1-p)
    WithInverse,
    /// Blend to white: p -> p + f*(1-p)
    ToWhite,
    /// Blend to black: p -> (1-f)*p
    ToBlack,
}

/// Gray blend type for grayscale blending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrayBlendType {
    /// Standard gray blend: linear interpolation
    Gray,
    /// Gray with inverse: blend using inverse values
    GrayWithInverse,
}

/// Blend mode for standard blend operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal alpha blending
    Normal,
    /// Multiply: result = base * blend / 255
    Multiply,
    /// Screen: result = 255 - (255-base)*(255-blend)/255
    Screen,
    /// Overlay: combination of multiply and screen
    Overlay,
    /// Hard light: like overlay but uses blend to determine
    HardLight,
    /// Soft light: gentler version of hard light
    SoftLight,
}

impl Pix {
    /// Blend a color image onto this image.
    ///
    /// result = (1 - fract) * self + fract * blend
    ///
    /// # Arguments
    /// * `blend` - The image to blend onto self
    /// * `x`, `y` - Position of blend relative to self
    /// * `fract` - Blending fraction (0.0 = all self, 1.0 = all blend)
    ///
    /// # Returns
    /// New blended image
    pub fn blend_color(&self, blend: &Pix, x: i32, y: i32, fract: f32) -> Result<Pix>;

    /// Blend using a grayscale image as blender.
    pub fn blend_gray(
        &self,
        blend: &Pix,
        x: i32,
        y: i32,
        fract: f32,
        blend_type: GrayBlendType,
    ) -> Result<Pix>;

    /// Blend using a 1-bit mask.
    pub fn blend_mask(
        &self,
        mask: &Pix,
        x: i32,
        y: i32,
        fract: f32,
        blend_type: MaskBlendType,
    ) -> Result<Pix>;

    /// Apply a blend mode with another image.
    pub fn blend(&self, other: &Pix, mode: BlendMode, fract: f32) -> Result<Pix>;

    /// Multiply blend: darker colors darken the image.
    pub fn blend_multiply(&self, other: &Pix) -> Result<Pix>;

    /// Screen blend: lighter colors lighten the image.
    pub fn blend_screen(&self, other: &Pix) -> Result<Pix>;

    /// Overlay blend: combines multiply and screen.
    pub fn blend_overlay(&self, other: &Pix) -> Result<Pix>;

    /// Hard light blend.
    pub fn blend_hard_light(&self, other: &Pix, fract: f32) -> Result<Pix>;
}

/// Blend a gray mask onto an image with specified alpha.
///
/// Uses the mask values as alpha: 0 = transparent, 255 = opaque
pub fn blend_with_gray_mask(
    base: &Pix,
    overlay: &Pix,
    mask: &Pix,
    x: i32,
    y: i32,
) -> Result<Pix>;
```

## Implementation Details

### Module Structure

```text
crates/leptonica-core/src/pix/
├── mod.rs       # Add: mod blend; pub use blend::*;
├── access.rs    # Existing
├── compare.rs   # Existing
├── convert.rs   # Existing
├── ops.rs       # Existing
└── blend.rs     # NEW: Image blending functions
```

### Algorithm: blend_color()

```text
For each pixel (x, y) in overlap region:
    base_pixel = self.get_pixel(x, y)
    blend_pixel = blend.get_pixel(bx, by)

    For each channel c in {R, G, B}:
        result[c] = (1 - fract) * base[c] + fract * blend[c]
        result[c] = clamp(result[c], 0, 255)

    output.set_pixel(x, y, result)
```

### Algorithm: blend_mask() - WithInverse

```text
For each pixel (x, y):
    if mask[mx, my] == 1:  # foreground pixel
        p = self.get_pixel(x, y) / 255.0
        result = p + fract * (1 - 2*p)
        result = clamp(result * 255, 0, 255)
        output.set_pixel(x, y, result)
    else:
        output.set_pixel(x, y, self.get_pixel(x, y))
```

### Algorithm: blend_multiply()

```text
For each pixel (x, y):
    base = self.get_pixel(x, y)
    blend = other.get_pixel(x, y)

    For each channel c:
        result[c] = (base[c] * blend[c]) / 255

    output.set_pixel(x, y, result)
```

### Algorithm: blend_screen()

```text
For each pixel (x, y):
    base = self.get_pixel(x, y)
    blend = other.get_pixel(x, y)

    For each channel c:
        result[c] = 255 - ((255 - base[c]) * (255 - blend[c])) / 255

    output.set_pixel(x, y, result)
```

### Algorithm: blend_overlay()

```text
For each pixel (x, y):
    base = self.get_pixel(x, y)
    blend = other.get_pixel(x, y)

    For each channel c:
        if base[c] < 128:
            result[c] = 2 * base[c] * blend[c] / 255
        else:
            result[c] = 255 - 2 * (255 - base[c]) * (255 - blend[c]) / 255

    output.set_pixel(x, y, result)
```

### Algorithm: blend_hard_light()

```text
For each pixel (x, y):
    base = self.get_pixel(x, y)
    blend = other.get_pixel(x, y)

    For each channel c:
        if blend[c] < 128:
            # Apply fract to reduce effect
            adjusted_blend = 128 - fract * (128 - blend[c])
            result[c] = (base[c] * adjusted_blend) / 128
        else:
            adjusted_blend = 128 + fract * (blend[c] - 128)
            result[c] = 255 - ((255 - adjusted_blend) * (255 - base[c])) / 128

    output.set_pixel(x, y, result)
```

### Error Handling

- Unsupported depth: 1bpp images not supported for color blend
- Dimension issues: Clipping is done in inner loop, no error
- Invalid fraction: Clamp to [0.0, 1.0] with warning or use as-is for special effects

### Performance Considerations

1. **Clipping in inner loop**: Handle overlap calculation once, then iterate
2. **Avoid division**: Use multiplication by reciprocal where possible
3. **Integer arithmetic**: Use fixed-point for fract calculations
4. **Early exit**: Skip fully transparent regions

## Tasks

1. [x] Create implementation plan
2. [x] Create `src/pix/blend.rs` with module structure
3. [x] Implement enums (MaskBlendType, GrayBlendType, BlendMode)
4. [x] Implement helper functions (clipping, component operations)
5. [x] Implement `blend_color()`
6. [x] Implement `blend_gray()`
7. [x] Implement `blend_mask()`
8. [x] Implement `blend_multiply()`
9. [x] Implement `blend_screen()`
10. [x] Implement `blend_overlay()`
11. [x] Implement `blend_hard_light()`
12. [x] Implement generic `blend()` with BlendMode
13. [x] Add unit tests for all functions
14. [x] Update `mod.rs` with module and re-exports
15. [x] Run `cargo fmt && cargo clippy`
16. [x] Run full test suite

## Test Plan

1. **Identity tests**:
   - Blend with fract=0 returns original
   - Blend with fract=1 returns blend image

2. **Known values**:
   - Multiply black with any = black
   - Multiply white with any = any
   - Screen black with any = any
   - Screen white with any = white

3. **Edge cases**:
   - Blend outside image bounds (should clip)
   - Negative positions
   - Zero-size overlap

4. **Color images**:
   - RGB blending works per-channel
   - Alpha handling (if present)

5. **Mask blending**:
   - Only masked pixels change
   - WithInverse produces expected inversion

6. **Performance**:
   - Large images complete in reasonable time

## Questions

(None at this time)

## Estimates

- Implementation: ~500 lines of code
- Time: ~4 hours
