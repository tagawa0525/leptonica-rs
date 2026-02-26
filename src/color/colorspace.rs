//! Color space conversion
//!
//! Provides conversion between various color spaces:
//! - RGB ↔ HSV (Hue, Saturation, Value)
//! - RGB ↔ LAB (CIE L*a*b*)
//! - RGB ↔ XYZ (CIE XYZ)
//! - RGB ↔ YUV
//! - RGB → Grayscale

use crate::color::{ColorError, ColorResult};
use crate::core::{FPix, Pix, PixColormap, PixelDepth, pixel};

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
    let z = 0.0193339 * r + 0.119_192 * g + 0.9503041 * b;

    Xyz::new(x * 100.0, y * 100.0, z * 100.0)
}

/// Convert CIE XYZ to RGB (D65 illuminant, sRGB color space)
pub fn xyz_to_rgb(xyz: Xyz) -> (u8, u8, u8) {
    let x = xyz.x / 100.0;
    let y = xyz.y / 100.0;
    let z = xyz.z / 100.0;

    // XYZ to sRGB matrix (D65)
    let r = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g = -0.969_266 * x + 1.8760108 * y + 0.0415560 * z;
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
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b) = pixel::extract_rgb(pixel);
                    let gray = rgb_to_gray(r, g, b);
                    out_mut.set_pixel_unchecked(x, y, gray as u32);
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
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b, a) = pixel::extract_rgba(pixel);

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

            out_mut.set_pixel_unchecked(x, y, value as u32);
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
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b, a) = pixel::extract_rgba(pixel);
            let hsv = rgb_to_hsv(r, g, b);

            let h_val = (hsv.h * 255.0).round() as u8;
            let s_val = (hsv.s * 255.0).round() as u8;
            let v_val = (hsv.v * 255.0).round() as u8;

            let result = pixel::compose_rgba(h_val, s_val, v_val, a);
            out_mut.set_pixel_unchecked(x, y, result);
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
            let pixel = pix.get_pixel_unchecked(x, y);
            let (h_val, s_val, v_val, a) = pixel::extract_rgba(pixel);

            let hsv = Hsv::new(
                h_val as f32 / 255.0,
                s_val as f32 / 255.0,
                v_val as f32 / 255.0,
            );
            let (r, g, b) = hsv_to_rgb(hsv);

            let result = pixel::compose_rgba(r, g, b, a);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Region selection mode for HSV range masks.
///
/// # See also
///
/// C Leptonica: `L_INCLUDE_REGION`, `L_EXCLUDE_REGION`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionFlag {
    /// Include only pixels within the specified range
    Include,
    /// Exclude pixels within the specified range (select everything else)
    Exclude,
}

// =============================================================================
// HSV range masks
// =============================================================================

/// Create a 1bpp mask over pixels within a hue-saturation range.
///
/// Hue uses Leptonica convention: `[0..239]` (wrap-around at 240).
/// Saturation range: `[0..255]`.
/// Both ranges are specified as center ± half-width.
///
/// # See also
///
/// C Leptonica: `pixMakeRangeMaskHS()` in `colorspace.c`
pub fn make_range_mask_hs(
    pix: &Pix,
    huecenter: i32,
    huehw: i32,
    satcenter: i32,
    sathw: i32,
    region_flag: RegionFlag,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let hlut = make_hue_lut(huecenter, huehw);
    let slut = make_linear_lut(satcenter, sathw, 256);

    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    // Pre-fill based on region flag
    let include = region_flag == RegionFlag::Include;
    if !include {
        for y in 0..h {
            for x in 0..w {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            let hval = hsv.h as usize;
            let sval = hsv.s as usize;

            if hval < 240 && hlut[hval] && slut[sval] {
                if include {
                    mask_mut.set_pixel_unchecked(x, y, 1);
                } else {
                    mask_mut.set_pixel_unchecked(x, y, 0);
                }
            }
        }
    }

    Ok(mask_mut.into())
}

/// Create a 1bpp mask over pixels within a hue-value range.
///
/// Hue uses Leptonica convention: `[0..239]` (wrap-around at 240).
/// Value (max intensity) range: `[0..255]`.
///
/// # See also
///
/// C Leptonica: `pixMakeRangeMaskHV()` in `colorspace.c`
pub fn make_range_mask_hv(
    pix: &Pix,
    huecenter: i32,
    huehw: i32,
    valcenter: i32,
    valhw: i32,
    region_flag: RegionFlag,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let hlut = make_hue_lut(huecenter, huehw);
    let vlut = make_linear_lut(valcenter, valhw, 256);

    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    let include = region_flag == RegionFlag::Include;
    if !include {
        for y in 0..h {
            for x in 0..w {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            let hval = hsv.h as usize;
            let vval = hsv.v as usize;

            if hval < 240 && hlut[hval] && vlut[vval] {
                if include {
                    mask_mut.set_pixel_unchecked(x, y, 1);
                } else {
                    mask_mut.set_pixel_unchecked(x, y, 0);
                }
            }
        }
    }

    Ok(mask_mut.into())
}

/// Create a 1bpp mask over pixels within a saturation-value range.
///
/// Saturation range: `[0..255]`. Value (max intensity) range: `[0..255]`.
/// Neither component has wrap-around.
///
/// # See also
///
/// C Leptonica: `pixMakeRangeMaskSV()` in `colorspace.c`
pub fn make_range_mask_sv(
    pix: &Pix,
    satcenter: i32,
    sathw: i32,
    valcenter: i32,
    valhw: i32,
    region_flag: RegionFlag,
) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let slut = make_linear_lut(satcenter, sathw, 256);
    let vlut = make_linear_lut(valcenter, valhw, 256);

    let w = pix.width();
    let h = pix.height();
    let mask = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut mask_mut = mask.try_into_mut().unwrap();

    let include = region_flag == RegionFlag::Include;
    if !include {
        for y in 0..h {
            for x in 0..w {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            let sval = hsv.s as usize;
            let vval = hsv.v as usize;

            if slut[sval] && vlut[vval] {
                if include {
                    mask_mut.set_pixel_unchecked(x, y, 1);
                } else {
                    mask_mut.set_pixel_unchecked(x, y, 0);
                }
            }
        }
    }

    Ok(mask_mut.into())
}

/// Build a hue lookup table with wrap-around at 240.
fn make_hue_lut(center: i32, hw: i32) -> Vec<bool> {
    let mut lut = vec![false; 240];
    let start = ((center - hw) % 240 + 240) % 240;
    let end = ((center + hw) % 240 + 240) % 240;
    if start <= end {
        for i in start..=end {
            lut[i as usize] = true;
        }
    } else {
        // Wrap-around
        for i in start..240 {
            lut[i as usize] = true;
        }
        for i in 0..=end {
            lut[i as usize] = true;
        }
    }
    lut
}

/// Build a linear (non-wrapping) lookup table for saturation or value.
fn make_linear_lut(center: i32, hw: i32, size: usize) -> Vec<bool> {
    let mut lut = vec![false; size];
    let start = 0.max(center - hw) as usize;
    let end = ((size as i32 - 1).min(center + hw)) as usize;
    for item in lut.iter_mut().take(end + 1).skip(start) {
        *item = true;
    }
    lut
}

// =============================================================================
// 2D HSV histograms
// =============================================================================

/// Create a 2D hue-saturation histogram from an RGB image.
///
/// Returns a 32bpp image of size 256 (sat) × 240 (hue).
/// Each pixel value is the count of input pixels at that (hue, saturation).
/// Hue is on the vertical axis, saturation on the horizontal.
///
/// # See also
///
/// C Leptonica: `pixMakeHistoHS()` in `colorspace.c`
pub fn make_histo_hs(pix: &Pix, factor: i32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let factor = factor.max(1) as u32;
    let w = pix.width();
    let h = pix.height();

    // 256 wide (saturation) × 240 tall (hue), 32bpp counts
    let histo_pix = Pix::new(256, 240, PixelDepth::Bit32)?;
    let mut histo_mut = histo_pix.try_into_mut().unwrap();

    for y in (0..h).step_by(factor as usize) {
        for x in (0..w).step_by(factor as usize) {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            // rgb_to_hsv guarantees: h∈[0,239], s∈[0,255]
            let count = histo_mut.get_pixel_unchecked(hsv.s as u32, hsv.h as u32);
            histo_mut.set_pixel_unchecked(hsv.s as u32, hsv.h as u32, count.saturating_add(1));
        }
    }

    Ok(histo_mut.into())
}

/// Create a 2D hue-value histogram from an RGB image.
///
/// Returns a 32bpp image of size 256 (val) × 240 (hue).
/// Each pixel value is the count of input pixels at that (hue, value).
/// Hue is on the vertical axis, value on the horizontal.
///
/// # See also
///
/// C Leptonica: `pixMakeHistoHV()` in `colorspace.c`
pub fn make_histo_hv(pix: &Pix, factor: i32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let factor = factor.max(1) as u32;
    let w = pix.width();
    let h = pix.height();

    let histo_pix = Pix::new(256, 240, PixelDepth::Bit32)?;
    let mut histo_mut = histo_pix.try_into_mut().unwrap();

    for y in (0..h).step_by(factor as usize) {
        for x in (0..w).step_by(factor as usize) {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            // rgb_to_hsv guarantees: h∈[0,239], v∈[0,255]
            let count = histo_mut.get_pixel_unchecked(hsv.v as u32, hsv.h as u32);
            histo_mut.set_pixel_unchecked(hsv.v as u32, hsv.h as u32, count.saturating_add(1));
        }
    }

    Ok(histo_mut.into())
}

/// Create a 2D saturation-value histogram from an RGB image.
///
/// Returns a 32bpp image of size 256 (val) × 256 (sat).
/// Each pixel value is the count of input pixels at that (sat, value).
/// Saturation is on the vertical axis, value on the horizontal.
///
/// # See also
///
/// C Leptonica: `pixMakeHistoSV()` in `colorspace.c`
pub fn make_histo_sv(pix: &Pix, factor: i32) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let factor = factor.max(1) as u32;
    let w = pix.width();
    let h = pix.height();

    let histo_pix = Pix::new(256, 256, PixelDepth::Bit32)?;
    let mut histo_mut = histo_pix.try_into_mut().unwrap();

    for y in (0..h).step_by(factor as usize) {
        for x in (0..w).step_by(factor as usize) {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let hsv = pixel::rgb_to_hsv(r, g, b);
            // rgb_to_hsv guarantees: s∈[0,255], v∈[0,255]
            let count = histo_mut.get_pixel_unchecked(hsv.v as u32, hsv.s as u32);
            histo_mut.set_pixel_unchecked(hsv.v as u32, hsv.s as u32, count.saturating_add(1));
        }
    }

    Ok(histo_mut.into())
}

// =============================================================================
// Image-level RGB ↔ YUV (Leptonica video-range encoding)
// =============================================================================

/// Convert RGB pixel to Leptonica video-range YUV.
///
/// Returns (Y, U, V) where Y∈[16,235], U∈[16,240], V∈[16,240].
#[inline]
fn convert_rgb_to_yuv_leptonica(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let norm = 1.0 / 256.0;
    let y = (16.0 + norm * (65.738 * r + 129.057 * g + 25.064 * b) + 0.5) as i32;
    let u = (128.0 + norm * (-37.945 * r - 74.494 * g + 112.439 * b) + 0.5) as i32;
    let v = (128.0 + norm * (112.439 * r - 94.154 * g - 18.285 * b) + 0.5) as i32;
    (y as u8, u as u8, v as u8)
}

/// Convert Leptonica video-range YUV pixel to RGB.
///
/// Expects Y∈[16,235], U∈[16,240], V∈[16,240].
#[inline]
fn convert_yuv_to_rgb_leptonica(y: u8, u: u8, v: u8) -> (u8, u8, u8) {
    let norm = 1.0 / 256.0;
    let ym = y as f32 - 16.0;
    let um = u as f32 - 128.0;
    let vm = v as f32 - 128.0;
    let r = (norm * (298.082 * ym + 408.583 * vm) + 0.5) as i32;
    let g = (norm * (298.082 * ym - 100.291 * um - 208.120 * vm) + 0.5) as i32;
    let b = (norm * (298.082 * ym + 516.411 * um) + 0.5) as i32;
    (
        r.clamp(0, 255) as u8,
        g.clamp(0, 255) as u8,
        b.clamp(0, 255) as u8,
    )
}

/// Convert a 32bpp RGB image to YUV color space.
///
/// Uses Leptonica video-range BT.601 encoding:
/// Y `[16..235]`, U `[16..240]`, V `[16..240]`.
/// Y, U, V are stored in the R, G, B bytes of the output pixel.
///
/// # See also
///
/// C Leptonica: `pixConvertRGBToYUV()` in `colorspace.c`
pub fn pix_convert_rgb_to_yuv(pix: &Pix) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(pixel);
            let (yv, uv, vv) = convert_rgb_to_yuv_leptonica(r, g, b);
            // Store as (Y << 24) | (U << 16) | (V << 8), matching C
            let result = ((yv as u32) << 24) | ((uv as u32) << 16) | ((vv as u32) << 8);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Convert a 32bpp YUV image back to RGB color space.
///
/// Expects Y, U, V stored in the R, G, B bytes using Leptonica
/// video-range BT.601 encoding.
///
/// # See also
///
/// C Leptonica: `pixConvertYUVToRGB()` in `colorspace.c`
pub fn pix_convert_yuv_to_rgb(pix: &Pix) -> ColorResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let yv = (pixel >> 24) & 0xff;
            let uv = (pixel >> 16) & 0xff;
            let vv = (pixel >> 8) & 0xff;
            let (r, g, b) = convert_yuv_to_rgb_leptonica(yv as u8, uv as u8, vv as u8);
            let result = pixel::compose_rgb(r, g, b);
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Colormap-level conversions
// =============================================================================

/// Convert colormap entries from RGB to HSV in-place.
///
/// Each (r,g,b) entry is replaced with (h,s,v) using the Leptonica integer
/// HSV convention: h∈\[0,239\], s∈\[0,255\], v∈\[0,255\].
///
/// # See also
///
/// C Leptonica: `pixcmapConvertRGBToHSV()` in `colorspace.c`
pub fn pix_colormap_convert_rgb_to_hsv(cmap: &mut PixColormap) -> ColorResult<()> {
    for i in 0..cmap.len() {
        let (r, g, b) = cmap.get_rgb(i).unwrap();
        let hsv = pixel::rgb_to_hsv(r, g, b);
        let quad = cmap.get_mut(i).unwrap();
        quad.red = hsv.h as u8;
        quad.green = hsv.s as u8;
        quad.blue = hsv.v as u8;
    }
    Ok(())
}

/// Convert colormap entries from HSV back to RGB in-place.
///
/// Each (h,s,v) entry is replaced with (r,g,b) using the Leptonica integer
/// HSV convention.
///
/// # See also
///
/// C Leptonica: `pixcmapConvertHSVToRGB()` in `colorspace.c`
pub fn pix_colormap_convert_hsv_to_rgb(cmap: &mut PixColormap) -> ColorResult<()> {
    for i in 0..cmap.len() {
        let (h, s, v) = cmap.get_rgb(i).unwrap();
        let hsv = pixel::Hsv {
            h: h as i32,
            s: s as i32,
            v: v as i32,
        };
        let (r, g, b) = pixel::hsv_to_rgb(hsv);
        let quad = cmap.get_mut(i).unwrap();
        quad.red = r;
        quad.green = g;
        quad.blue = b;
    }
    Ok(())
}

/// Convert colormap entries from RGB to YUV in-place.
///
/// Uses Leptonica video-range BT.601 encoding.
///
/// # See also
///
/// C Leptonica: `pixcmapConvertRGBToYUV()` in `colorspace.c`
pub fn pix_colormap_convert_rgb_to_yuv(cmap: &mut PixColormap) -> ColorResult<()> {
    for i in 0..cmap.len() {
        let (r, g, b) = cmap.get_rgb(i).unwrap();
        let (yv, uv, vv) = convert_rgb_to_yuv_leptonica(r, g, b);
        let quad = cmap.get_mut(i).unwrap();
        quad.red = yv;
        quad.green = uv;
        quad.blue = vv;
    }
    Ok(())
}

/// Convert colormap entries from YUV back to RGB in-place.
///
/// Uses Leptonica video-range BT.601 encoding.
///
/// # See also
///
/// C Leptonica: `pixcmapConvertYUVToRGB()` in `colorspace.c`
pub fn pix_colormap_convert_yuv_to_rgb(cmap: &mut PixColormap) -> ColorResult<()> {
    for i in 0..cmap.len() {
        let (yv, uv, vv) = cmap.get_rgb(i).unwrap();
        let (r, g, b) = convert_yuv_to_rgb_leptonica(yv, uv, vv);
        let quad = cmap.get_mut(i).unwrap();
        quad.red = r;
        quad.green = g;
        quad.blue = b;
    }
    Ok(())
}

// =============================================================================
// HSV histogram peak finding
// =============================================================================

/// Type of HSV histogram for peak finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsvHistoType {
    /// Hue-Saturation histogram (240 × 256)
    HS,
    /// Hue-Value histogram (240 × 256)
    HV,
    /// Saturation-Value histogram (256 × 256)
    SV,
}

/// Result of HSV histogram peak finding: `(peak_locations, peak_areas)`
pub type HsvPeaksResult = (Vec<(i32, i32)>, Vec<u32>);

/// Find peaks in an HSV 2D histogram.
///
/// Takes a 32bpp histogram image (from `make_histo_hs` / `make_histo_hv` /
/// `make_histo_sv`), finds peaks using a sliding window approach.
///
/// Returns `(points, areas)` – locations and integrated areas of peaks.
///
/// # Arguments
///
/// * `pixs` – 32bpp histogram image
/// * `histo_type` – which kind of HSV histogram
/// * `width` – half-width of the sliding window
/// * `height` – half-height of the sliding window
/// * `npeaks` – maximum number of peaks to find
/// * `erase_factor` – ratio of erase window to sliding window (≥ 1.0)
///
/// # See also
///
/// C Leptonica: `pixFindHistoPeaksHSV()` in `colorspace.c`
pub fn find_histo_peaks_hsv(
    pixs: &Pix,
    histo_type: HsvHistoType,
    width: i32,
    height: i32,
    npeaks: i32,
    erase_factor: f32,
) -> ColorResult<HsvPeaksResult> {
    if pixs.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pixs.depth().bits(),
        });
    }
    if width < 1 || height < 1 || npeaks < 1 {
        return Err(ColorError::InvalidParameters(
            "width, height and npeaks must be >= 1".into(),
        ));
    }

    let pw = pixs.width() as i32;
    let ph = pixs.height() as i32;
    let erase_factor = erase_factor.max(1.0);

    // Build a local accumulated-sum buffer (sliding window sums)
    let mut sums = vec![0u64; (pw * ph) as usize];
    for yy in 0..ph {
        for xx in 0..pw {
            let y0 = (yy - height).max(0);
            let y1 = (yy + height).min(ph - 1);
            let x0 = (xx - width).max(0);
            let x1 = (xx + width).min(pw - 1);
            let mut total = 0u64;
            for sy in y0..=y1 {
                for sx in x0..=x1 {
                    total += pixs.get_pixel_unchecked(sx as u32, sy as u32) as u64;
                }
            }
            sums[(yy * pw + xx) as usize] = total;
        }
    }

    let ew = (width as f32 * erase_factor) as i32;
    let eh = (height as f32 * erase_factor) as i32;

    // For HS / HV histograms the hue axis (vertical) wraps at 240
    let hue_wrap = matches!(histo_type, HsvHistoType::HS | HsvHistoType::HV);

    let mut points = Vec::new();
    let mut areas = Vec::new();

    for _ in 0..npeaks {
        // Find the maximum in the sums buffer
        let mut best_val = 0u64;
        let mut best_idx = 0usize;
        for (idx, &v) in sums.iter().enumerate() {
            if v > best_val {
                best_val = v;
                best_idx = idx;
            }
        }
        if best_val == 0 {
            break;
        }

        let peak_x = (best_idx as i32) % pw;
        let peak_y = (best_idx as i32) / pw;
        points.push((peak_x, peak_y));
        areas.push(best_val as u32);

        // Erase a window around the peak in sums
        let ey0 = peak_y - eh;
        let ey1 = peak_y + eh;
        let ex0 = (peak_x - ew).max(0);
        let ex1 = (peak_x + ew).min(pw - 1);

        for ey in ey0..=ey1 {
            let ay = if hue_wrap {
                ((ey % ph) + ph) % ph
            } else {
                ey.clamp(0, ph - 1)
            };
            for ex in ex0..=ex1 {
                sums[(ay * pw + ex) as usize] = 0;
            }
        }
    }

    Ok((points, areas))
}

// =============================================================================
// Image-level RGB ↔ XYZ (floating-point FPix channels)
// =============================================================================

/// Convert a 32bpp RGB image to CIE XYZ using three `FPix` channels.
///
/// Returns `(fpix_x, fpix_y, fpix_z)`.
///
/// # See also
///
/// C Leptonica: `pixConvertRGBToXYZ()` in `colorspace.c`
pub fn pix_convert_rgb_to_xyz(pix: &Pix) -> ColorResult<(FPix, FPix, FPix)> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let mut fx = FPix::new(w, h)?;
    let mut fy = FPix::new(w, h)?;
    let mut fz = FPix::new(w, h)?;

    for y in 0..h {
        for x in 0..w {
            let p = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(p);
            let xyz = rgb_to_xyz(r, g, b);
            fx.set_pixel_unchecked(x, y, xyz.x);
            fy.set_pixel_unchecked(x, y, xyz.y);
            fz.set_pixel_unchecked(x, y, xyz.z);
        }
    }

    Ok((fx, fy, fz))
}

/// Convert CIE XYZ `FPix` channels back to a 32bpp RGB `Pix`.
///
/// # See also
///
/// C Leptonica: `fpixaConvertXYZToRGB()` in `colorspace.c`
pub fn fpixa_convert_xyz_to_rgb(fx: &FPix, fy: &FPix, fz: &FPix) -> ColorResult<Pix> {
    let w = fx.width();
    let h = fx.height();
    if fy.width() != w || fy.height() != h || fz.width() != w || fz.height() != h {
        return Err(ColorError::InvalidParameters(
            "FPix dimensions must match".into(),
        ));
    }

    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let xyz = Xyz::new(
                fx.get_pixel_unchecked(x, y),
                fy.get_pixel_unchecked(x, y),
                fz.get_pixel_unchecked(x, y),
            );
            let (r, g, b) = xyz_to_rgb(xyz);
            out_mut.set_pixel_unchecked(x, y, pixel::compose_rgb(r, g, b));
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// Image-level RGB ↔ LAB (floating-point FPix channels)
// =============================================================================

/// Convert a 32bpp RGB image to CIE L\*a\*b\* using three `FPix` channels.
///
/// Returns `(fpix_l, fpix_a, fpix_b)`.
///
/// # See also
///
/// C Leptonica: `pixConvertRGBToLAB()` in `colorspace.c`
pub fn pix_convert_rgb_to_lab(pix: &Pix) -> ColorResult<(FPix, FPix, FPix)> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(ColorError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();
    let mut fl = FPix::new(w, h)?;
    let mut fa = FPix::new(w, h)?;
    let mut fb = FPix::new(w, h)?;

    for y in 0..h {
        for x in 0..w {
            let p = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = pixel::extract_rgb(p);
            let lab = rgb_to_lab(r, g, b);
            fl.set_pixel_unchecked(x, y, lab.l);
            fa.set_pixel_unchecked(x, y, lab.a);
            fb.set_pixel_unchecked(x, y, lab.b);
        }
    }

    Ok((fl, fa, fb))
}

/// Convert CIE L\*a\*b\* `FPix` channels back to a 32bpp RGB `Pix`.
///
/// # See also
///
/// C Leptonica: `fpixaConvertLABToRGB()` in `colorspace.c`
pub fn fpixa_convert_lab_to_rgb(fl: &FPix, fa: &FPix, fb: &FPix) -> ColorResult<Pix> {
    let w = fl.width();
    let h = fl.height();
    if fa.width() != w || fa.height() != h || fb.width() != w || fb.height() != h {
        return Err(ColorError::InvalidParameters(
            "FPix dimensions must match".into(),
        ));
    }

    let out = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let lab = Lab::new(
                fl.get_pixel_unchecked(x, y),
                fa.get_pixel_unchecked(x, y),
                fb.get_pixel_unchecked(x, y),
            );
            let (r, g, b) = lab_to_rgb(lab);
            out_mut.set_pixel_unchecked(x, y, pixel::compose_rgb(r, g, b));
        }
    }

    Ok(out_mut.into())
}

// =============================================================================
// FPix-level XYZ ↔ LAB
// =============================================================================

/// Convert XYZ `FPix` channels to LAB `FPix` channels.
///
/// # See also
///
/// C Leptonica: `fpixaConvertXYZToLAB()` in `colorspace.c`
pub fn fpixa_convert_xyz_to_lab(
    fx: &FPix,
    fy: &FPix,
    fz: &FPix,
) -> ColorResult<(FPix, FPix, FPix)> {
    let w = fx.width();
    let h = fx.height();
    if fy.width() != w || fy.height() != h || fz.width() != w || fz.height() != h {
        return Err(ColorError::InvalidParameters(
            "FPix dimensions must match".into(),
        ));
    }

    let mut fl = FPix::new(w, h)?;
    let mut fa = FPix::new(w, h)?;
    let mut fb = FPix::new(w, h)?;

    for y in 0..h {
        for x in 0..w {
            let xyz = Xyz::new(
                fx.get_pixel_unchecked(x, y),
                fy.get_pixel_unchecked(x, y),
                fz.get_pixel_unchecked(x, y),
            );
            let lab = xyz_to_lab(xyz);
            fl.set_pixel_unchecked(x, y, lab.l);
            fa.set_pixel_unchecked(x, y, lab.a);
            fb.set_pixel_unchecked(x, y, lab.b);
        }
    }

    Ok((fl, fa, fb))
}

/// Convert LAB `FPix` channels to XYZ `FPix` channels.
///
/// # See also
///
/// C Leptonica: `fpixaConvertLABToXYZ()` in `colorspace.c`
pub fn fpixa_convert_lab_to_xyz(
    fl: &FPix,
    fa: &FPix,
    fb: &FPix,
) -> ColorResult<(FPix, FPix, FPix)> {
    let w = fl.width();
    let h = fl.height();
    if fa.width() != w || fa.height() != h || fb.width() != w || fb.height() != h {
        return Err(ColorError::InvalidParameters(
            "FPix dimensions must match".into(),
        ));
    }

    let mut fx = FPix::new(w, h)?;
    let mut fy = FPix::new(w, h)?;
    let mut fz = FPix::new(w, h)?;

    for y in 0..h {
        for x in 0..w {
            let lab = Lab::new(
                fl.get_pixel_unchecked(x, y),
                fa.get_pixel_unchecked(x, y),
                fb.get_pixel_unchecked(x, y),
            );
            let xyz = lab_to_xyz(lab);
            fx.set_pixel_unchecked(x, y, xyz.x);
            fy.set_pixel_unchecked(x, y, xyz.y);
            fz.set_pixel_unchecked(x, y, xyz.z);
        }
    }

    Ok((fx, fy, fz))
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
        assert!((127..=129).contains(&gray));
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
                let pixel = pixel::compose_rgb(100, 150, 200);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let gray_pix = pix_convert_to_gray(&pix_mut.into()).unwrap();
        assert_eq!(gray_pix.depth(), PixelDepth::Bit8);

        let expected_gray = rgb_to_gray(100, 150, 200);
        let actual = gray_pix.get_pixel_unchecked(5, 5) as u8;
        assert_eq!(actual, expected_gray);
    }

    #[test]
    fn test_pix_extract_channel() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let pixel = pixel::compose_rgba(100, 150, 200, 255);
        for y in 0..5 {
            for x in 0..5 {
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let pix = pix_mut.into();

        let red_channel = pix_extract_channel(&pix, ColorChannel::Red).unwrap();
        assert_eq!(red_channel.get_pixel_unchecked(0, 0) as u8, 100);

        let green_channel = pix_extract_channel(&pix, ColorChannel::Green).unwrap();
        assert_eq!(green_channel.get_pixel_unchecked(0, 0) as u8, 150);

        let blue_channel = pix_extract_channel(&pix, ColorChannel::Blue).unwrap();
        assert_eq!(blue_channel.get_pixel_unchecked(0, 0) as u8, 200);
    }

    #[test]
    fn test_pix_rgb_hsv_roundtrip() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let pixel = pixel::compose_rgb(200, 100, 50);
        for y in 0..5 {
            for x in 0..5 {
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        let pix = pix_mut.into();
        let hsv_pix = pix_convert_rgb_to_hsv(&pix).unwrap();
        let rgb_pix = pix_convert_hsv_to_rgb(&hsv_pix).unwrap();

        let original = pix.get_pixel_unchecked(0, 0);
        let converted = rgb_pix.get_pixel_unchecked(0, 0);

        let (r1, g1, b1) = pixel::extract_rgb(original);
        let (r2, g2, b2) = pixel::extract_rgb(converted);

        assert!((r1 as i32 - r2 as i32).abs() <= 2);
        assert!((g1 as i32 - g2 as i32).abs() <= 2);
        assert!((b1 as i32 - b2 as i32).abs() <= 2);
    }
}
