//! Color space conversion
//!
//! Provides conversion between various color spaces:
//! - RGB ↔ HSV (Hue, Saturation, Value)
//! - RGB ↔ LAB (CIE L*a*b*)
//! - RGB ↔ XYZ (CIE XYZ)
//! - RGB ↔ YUV
//! - RGB → Grayscale

use crate::{ColorError, ColorResult};
use leptonica_core::{Pix, PixelDepth, color};

/// HSV color representation
///
/// - `h`: Hue in range [0.0, 1.0] (where 1.0 wraps to 0.0)
/// - `s`: Saturation in range [0.0, 1.0]
/// - `v`: Value in range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    /// Create a new HSV color
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }
}

/// CIE L*a*b* color representation
///
/// - `l`: Lightness in range [0.0, 100.0]
/// - `a`: Green-Red component, typically [-128, 127]
/// - `b`: Blue-Yellow component, typically [-128, 127]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Lab {
    /// Create a new LAB color
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }
}

/// CIE XYZ color representation (D65 illuminant)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz {
    /// Create a new XYZ color
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// YUV color representation (BT.601)
///
/// - `y`: Luma component
/// - `u`: Blue-difference chroma
/// - `v`: Red-difference chroma
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yuv {
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

impl Yuv {
    /// Create a new YUV color
    pub fn new(y: f32, u: f32, v: f32) -> Self {
        Self { y, u, v }
    }
}

/// Color channel selector for extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
    /// Hue from HSV
    Hue,
    /// Saturation from HSV
    Saturation,
    /// Value from HSV
    Value,
}

// =============================================================================
// RGB ↔ Grayscale
// =============================================================================

/// Convert RGB to grayscale using ITU-R BT.601 coefficients
///
/// Formula: gray = 0.299*R + 0.587*G + 0.114*B
#[inline]
pub fn rgb_to_gray(r: u8, g: u8, b: u8) -> u8 {
    // Use integer math for speed: (77*R + 150*G + 29*B) / 256
    let gray = (77 * r as u32 + 150 * g as u32 + 29 * b as u32) >> 8;
    gray as u8
}

// =============================================================================
// RGB ↔ HSV
// =============================================================================

/// Convert RGB values to HSV
///
/// Returns HSV with all components in range [0.0, 1.0]
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;

    if delta < f32::EPSILON {
        // Gray - no chroma
        return Hsv::new(0.0, 0.0, v);
    }

    let s = delta / max;

    let h = if (r - max).abs() < f32::EPSILON {
        // Red is max
        (g - b) / delta
    } else if (g - max).abs() < f32::EPSILON {
        // Green is max
        2.0 + (b - r) / delta
    } else {
        // Blue is max
        4.0 + (r - g) / delta
    };

    // Convert to [0, 1] range
    let h = h / 6.0;
    let h = if h < 0.0 { h + 1.0 } else { h };

    Hsv::new(h, s, v)
}

/// Convert HSV values to RGB
///
/// Input HSV should have all components in range [0.0, 1.0]
pub fn hsv_to_rgb(hsv: Hsv) -> (u8, u8, u8) {
    let Hsv { h, s, v } = hsv;

    if s < f32::EPSILON {
        // Gray
        let gray = (v * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let h = h * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

// =============================================================================
// RGB ↔ XYZ (D65 illuminant, sRGB)
// =============================================================================

/// Convert RGB to CIE XYZ (D65 illuminant, sRGB color space)
pub fn rgb_to_xyz(r: u8, g: u8, b: u8) -> Xyz {
    // Normalize to [0, 1] and apply sRGB gamma correction
    let r = srgb_to_linear(r as f32 / 255.0);
    let g = srgb_to_linear(g as f32 / 255.0);
    let b = srgb_to_linear(b as f32 / 255.0);

    // sRGB to XYZ matrix (D65)
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

    Xyz::new(x * 100.0, y * 100.0, z * 100.0)
}

/// Convert CIE XYZ to RGB (D65 illuminant, sRGB color space)
pub fn xyz_to_rgb(xyz: Xyz) -> (u8, u8, u8) {
    let x = xyz.x / 100.0;
    let y = xyz.y / 100.0;
    let z = xyz.z / 100.0;

    // XYZ to sRGB matrix (D65)
    let r = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    // Apply sRGB gamma and convert to u8
    (
        (linear_to_srgb(r) * 255.0).round().clamp(0.0, 255.0) as u8,
        (linear_to_srgb(g) * 255.0).round().clamp(0.0, 255.0) as u8,
        (linear_to_srgb(b) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// sRGB to linear RGB conversion
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear RGB to sRGB conversion
#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// =============================================================================
// XYZ ↔ LAB
// =============================================================================

// D65 reference white point
const REF_X: f32 = 95.047;
const REF_Y: f32 = 100.0;
const REF_Z: f32 = 108.883;

/// Convert CIE XYZ to CIE L*a*b*
pub fn xyz_to_lab(xyz: Xyz) -> Lab {
    let x = lab_forward(xyz.x / REF_X);
    let y = lab_forward(xyz.y / REF_Y);
    let z = lab_forward(xyz.z / REF_Z);

    let l = 116.0 * y - 16.0;
    let a = 500.0 * (x - y);
    let b = 200.0 * (y - z);

    Lab::new(l, a, b)
}

/// Convert CIE L*a*b* to CIE XYZ
pub fn lab_to_xyz(lab: Lab) -> Xyz {
    let y = (lab.l + 16.0) / 116.0;
    let x = lab.a / 500.0 + y;
    let z = y - lab.b / 200.0;

    Xyz::new(
        lab_reverse(x) * REF_X,
        lab_reverse(y) * REF_Y,
        lab_reverse(z) * REF_Z,
    )
}

/// Forward function for LAB conversion
#[inline]
fn lab_forward(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;
    const DELTA_CUBE: f32 = DELTA * DELTA * DELTA;

    if t > DELTA_CUBE {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

/// Reverse function for LAB conversion
#[inline]
fn lab_reverse(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;

    if t > DELTA {
        t * t * t
    } else {
        3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
    }
}

// =============================================================================
// RGB ↔ LAB (convenience functions via XYZ)
// =============================================================================

/// Convert RGB to CIE L*a*b*
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> Lab {
    xyz_to_lab(rgb_to_xyz(r, g, b))
}

/// Convert CIE L*a*b* to RGB
pub fn lab_to_rgb(lab: Lab) -> (u8, u8, u8) {
    xyz_to_rgb(lab_to_xyz(lab))
}

// =============================================================================
// RGB ↔ YUV (BT.601)
// =============================================================================

/// Convert RGB to YUV (BT.601)
pub fn rgb_to_yuv(r: u8, g: u8, b: u8) -> Yuv {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;

    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
    let v = 0.615 * r - 0.51499 * g - 0.10001 * b;

    Yuv::new(y, u, v)
}

/// Convert YUV to RGB (BT.601)
pub fn yuv_to_rgb(yuv: Yuv) -> (u8, u8, u8) {
    let Yuv { y, u, v } = yuv;

    let r = y + 1.13983 * v;
    let g = y - 0.39465 * u - 0.58060 * v;
    let b = y + 2.03211 * u;

    (
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
    )
}

// =============================================================================
// Image-level operations
// =============================================================================

/// Convert a color image to 8-bit grayscale
///
/// Supports 32-bit RGB/RGBA input.
pub fn pix_convert_to_gray(pix: &Pix) -> ColorResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit8 => {
            // Already grayscale, just clone
            Ok(pix.clone())
        }
        PixelDepth::Bit32 => {
            let w = pix.width();
            let h = pix.height();
            let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
            let mut out_mut = out_pix.try_into_mut().unwrap();

            for y in 0..h {
                for x in 0..w {
                    let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
                    let (r, g, b) = color::extract_rgb(pixel);
                    let gray = rgb_to_gray(r, g, b);
                    unsafe { out_mut.set_pixel_unchecked(x, y, gray as u32) };
                }
            }

            Ok(out_mut.into())
        }
        _ => Err(ColorError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Extract a single color channel from a 32-bit image
///
/// Returns an 8-bit grayscale image containing only the selected channel.
pub fn pix_extract_channel(pix: &Pix, channel: ColorChannel) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, a) = color::extract_rgba(pixel);

            let value = match channel {
                ColorChannel::Red => r,
                ColorChannel::Green => g,
                ColorChannel::Blue => b,
                ColorChannel::Alpha => a,
                ColorChannel::Hue => {
                    let hsv = rgb_to_hsv(r, g, b);
                    (hsv.h * 255.0).round() as u8
                }
                ColorChannel::Saturation => {
                    let hsv = rgb_to_hsv(r, g, b);
                    (hsv.s * 255.0).round() as u8
                }
                ColorChannel::Value => {
                    let hsv = rgb_to_hsv(r, g, b);
                    (hsv.v * 255.0).round() as u8
                }
            };

            unsafe { out_mut.set_pixel_unchecked(x, y, value as u32) };
        }
    }

    Ok(out_mut.into())
}

/// Convert RGB image to HSV representation
///
/// The resulting image stores H, S, V in the R, G, B channels respectively,
/// scaled to [0, 255].
pub fn pix_convert_rgb_to_hsv(pix: &Pix) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, a) = color::extract_rgba(pixel);
            let hsv = rgb_to_hsv(r, g, b);

            let h_val = (hsv.h * 255.0).round() as u8;
            let s_val = (hsv.s * 255.0).round() as u8;
            let v_val = (hsv.v * 255.0).round() as u8;

            let result = color::compose_rgba(h_val, s_val, v_val, a);
            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
        }
    }

    Ok(out_mut.into())
}

/// Convert HSV image back to RGB
///
/// Expects an image where H, S, V are stored in R, G, B channels,
/// scaled to [0, 255].
pub fn pix_convert_hsv_to_rgb(pix: &Pix) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (h_val, s_val, v_val, a) = color::extract_rgba(pixel);

            let hsv = Hsv::new(
                h_val as f32 / 255.0,
                s_val as f32 / 255.0,
                v_val as f32 / 255.0,
            );
            let (r, g, b) = hsv_to_rgb(hsv);

            let result = color::compose_rgba(r, g, b, a);
            unsafe { out_mut.set_pixel_unchecked(x, y, result) };
        }
    }

    Ok(out_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_gray() {
        // White -> white
        assert_eq!(rgb_to_gray(255, 255, 255), 255);
        // Black -> black
        assert_eq!(rgb_to_gray(0, 0, 0), 0);
        // Middle gray
        let gray = rgb_to_gray(128, 128, 128);
        assert!(gray >= 127 && gray <= 129);
    }

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let test_colors = [
            (255, 0, 0),     // Red
            (0, 255, 0),     // Green
            (0, 0, 255),     // Blue
            (255, 255, 0),   // Yellow
            (0, 255, 255),   // Cyan
            (255, 0, 255),   // Magenta
            (128, 128, 128), // Gray
            (0, 0, 0),       // Black
            (255, 255, 255), // White
        ];

        for (r, g, b) in test_colors {
            let hsv = rgb_to_hsv(r, g, b);
            let (r2, g2, b2) = hsv_to_rgb(hsv);

            // Allow small rounding errors
            assert!(
                (r as i32 - r2 as i32).abs() <= 1,
                "Red mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (g as i32 - g2 as i32).abs() <= 1,
                "Green mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (b as i32 - b2 as i32).abs() <= 1,
                "Blue mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
        }
    }

    #[test]
    fn test_hsv_values() {
        // Red: H=0, S=1, V=1
        let hsv = rgb_to_hsv(255, 0, 0);
        assert!((hsv.h - 0.0).abs() < 0.01);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);

        // Green: H=1/3, S=1, V=1
        let hsv = rgb_to_hsv(0, 255, 0);
        assert!((hsv.h - 1.0 / 3.0).abs() < 0.01);

        // Blue: H=2/3, S=1, V=1
        let hsv = rgb_to_hsv(0, 0, 255);
        assert!((hsv.h - 2.0 / 3.0).abs() < 0.01);

        // Gray: S=0
        let hsv = rgb_to_hsv(128, 128, 128);
        assert!(hsv.s < 0.01);
    }

    #[test]
    fn test_rgb_xyz_roundtrip() {
        let test_colors = [
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 128, 128),
            (0, 0, 0),
            (255, 255, 255),
        ];

        for (r, g, b) in test_colors {
            let xyz = rgb_to_xyz(r, g, b);
            let (r2, g2, b2) = xyz_to_rgb(xyz);

            assert!(
                (r as i32 - r2 as i32).abs() <= 2,
                "Red mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (g as i32 - g2 as i32).abs() <= 2,
                "Green mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (b as i32 - b2 as i32).abs() <= 2,
                "Blue mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
        }
    }

    #[test]
    fn test_rgb_lab_roundtrip() {
        let test_colors = [
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 128, 128),
            (0, 0, 0),
            (255, 255, 255),
        ];

        for (r, g, b) in test_colors {
            let lab = rgb_to_lab(r, g, b);
            let (r2, g2, b2) = lab_to_rgb(lab);

            // LAB conversion can have larger rounding errors
            assert!(
                (r as i32 - r2 as i32).abs() <= 3,
                "Red mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (g as i32 - g2 as i32).abs() <= 3,
                "Green mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (b as i32 - b2 as i32).abs() <= 3,
                "Blue mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
        }
    }

    #[test]
    fn test_rgb_yuv_roundtrip() {
        let test_colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 128, 128)];

        for (r, g, b) in test_colors {
            let yuv = rgb_to_yuv(r, g, b);
            let (r2, g2, b2) = yuv_to_rgb(yuv);

            assert!(
                (r as i32 - r2 as i32).abs() <= 2,
                "Red mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (g as i32 - g2 as i32).abs() <= 2,
                "Green mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
            assert!(
                (b as i32 - b2 as i32).abs() <= 2,
                "Blue mismatch for ({}, {}, {})",
                r,
                g,
                b
            );
        }
    }

    #[test]
    fn test_pix_convert_to_gray() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                let pixel = color::compose_rgb(100, 150, 200);
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        let gray_pix = pix_convert_to_gray(&pix_mut.into()).unwrap();
        assert_eq!(gray_pix.depth(), PixelDepth::Bit8);

        let expected_gray = rgb_to_gray(100, 150, 200);
        let actual = unsafe { gray_pix.get_pixel_unchecked(5, 5) } as u8;
        assert_eq!(actual, expected_gray);
    }

    #[test]
    fn test_pix_extract_channel() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let pixel = color::compose_rgba(100, 150, 200, 255);
        for y in 0..5 {
            for x in 0..5 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        let pix = pix_mut.into();

        let red_channel = pix_extract_channel(&pix, ColorChannel::Red).unwrap();
        assert_eq!(unsafe { red_channel.get_pixel_unchecked(0, 0) } as u8, 100);

        let green_channel = pix_extract_channel(&pix, ColorChannel::Green).unwrap();
        assert_eq!(
            unsafe { green_channel.get_pixel_unchecked(0, 0) } as u8,
            150
        );

        let blue_channel = pix_extract_channel(&pix, ColorChannel::Blue).unwrap();
        assert_eq!(unsafe { blue_channel.get_pixel_unchecked(0, 0) } as u8, 200);
    }

    #[test]
    fn test_pix_rgb_hsv_roundtrip() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let pixel = color::compose_rgb(200, 100, 50);
        for y in 0..5 {
            for x in 0..5 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        let pix = pix_mut.into();
        let hsv_pix = pix_convert_rgb_to_hsv(&pix).unwrap();
        let rgb_pix = pix_convert_hsv_to_rgb(&hsv_pix).unwrap();

        let original = unsafe { pix.get_pixel_unchecked(0, 0) };
        let converted = unsafe { rgb_pix.get_pixel_unchecked(0, 0) };

        let (r1, g1, b1) = color::extract_rgb(original);
        let (r2, g2, b2) = color::extract_rgb(converted);

        assert!((r1 as i32 - r2 as i32).abs() <= 2);
        assert!((g1 as i32 - g2 as i32).abs() <= 2);
        assert!((b1 as i32 - b2 as i32).abs() <= 2);
    }
}
