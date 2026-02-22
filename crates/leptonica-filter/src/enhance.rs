//! Image enhancement operations
//!
//! Tone reproduction curve (TRC) mapping, gamma correction, contrast
//! enhancement, histogram equalization, HSV modification, and color
//! shifting.
//!
//! # See also
//!
//! C Leptonica: `enhance.c`

use crate::{FilterError, FilterResult, Kernel};
use leptonica_core::pix::RgbComponent;
use leptonica_core::{Pix, PixMut, PixelDepth, color};

/// Scale factor for contrast enhancement, matching C Leptonica.
const ENHANCE_SCALE_FACTOR: f64 = 5.0;

/// A 256-entry lookup table for tone reproduction curve mapping.
///
/// Maps input pixel values [0..255] to output pixel values [0..255].
pub type TrcLut = [u8; 256];

/// Generate a gamma TRC (tone reproduction curve) lookup table.
///
/// The mapping uses a power function: `output = 255 * ((input - minval) / (maxval - minval)) ^ (1/gamma)`
///
/// # Arguments
///
/// * `gamma` - Gamma correction factor; must be > 0.0.
///   Values > 1.0 lighten the image; values < 1.0 darken it.
/// * `minval` - Input value that maps to 0 output. Can be negative.
/// * `maxval` - Input value that maps to 255 output. Can exceed 255.
///
/// # See also
///
/// C Leptonica: `numaGammaTRC()` in `enhance.c`
pub fn gamma_trc(gamma: f32, minval: i32, maxval: i32) -> FilterResult<TrcLut> {
    if minval >= maxval {
        return Err(FilterError::InvalidParameters(
            "minval must be less than maxval".into(),
        ));
    }
    if gamma <= 0.0 {
        return Err(FilterError::InvalidParameters("gamma must be > 0.0".into()));
    }

    let inv_gamma = 1.0_f32 / gamma;
    let range = (maxval - minval) as f32;
    let mut lut = [0u8; 256];

    for i in 0..256i32 {
        let val = if i < minval {
            0
        } else if i > maxval {
            255
        } else {
            let x = (i - minval) as f32 / range;
            let mapped = 255.0 * x.powf(inv_gamma) + 0.5;
            (mapped as i32).clamp(0, 255)
        };
        lut[i as usize] = val as u8;
    }

    Ok(lut)
}

/// Generate a contrast enhancement TRC lookup table.
///
/// Uses an atan-based mapping with maximum slope at value 127.
/// Pixels below 127 are darkened and pixels above 127 are lightened.
///
/// # Arguments
///
/// * `factor` - Contrast enhancement factor. 0.0 is no enhancement;
///   useful range is (0.0, 1.0) but larger values are allowed.
///
/// # See also
///
/// C Leptonica: `numaContrastTRC()` in `enhance.c`
pub fn contrast_trc(factor: f32) -> FilterResult<TrcLut> {
    if factor < 0.0 {
        return Err(FilterError::InvalidParameters(
            "factor must be >= 0.0".into(),
        ));
    }

    let mut lut = [0u8; 256];

    if factor == 0.0 {
        // Identity mapping
        for (i, entry) in lut.iter_mut().enumerate() {
            *entry = i as u8;
        }
        return Ok(lut);
    }

    let scale = ENHANCE_SCALE_FACTOR;
    let factor_d = factor as f64;
    let ymax = (1.0 * factor_d * scale).atan();
    let ymin = (-127.0 * factor_d * scale / 128.0).atan();
    let dely = ymax - ymin;

    for (i, entry) in lut.iter_mut().enumerate() {
        let x = i as f64;
        let val = (255.0 / dely) * (-ymin + (factor_d * scale * (x - 127.0) / 128.0).atan()) + 0.5;
        *entry = (val as i32).clamp(0, 255) as u8;
    }

    Ok(lut)
}

/// Generate a histogram equalization TRC lookup table.
///
/// Computes a mapping that equalizes the histogram of an 8 bpp image.
///
/// # Arguments
///
/// * `pix` - Input 8 bpp grayscale image (no colormap)
/// * `fract` - Fraction of equalization movement. 0.0 = no change, 1.0 = full equalization.
/// * `factor` - Subsampling factor for histogram computation; >= 1.
///
/// # See also
///
/// C Leptonica: `numaEqualizeTRC()` in `enhance.c`
pub fn equalize_trc(pix: &Pix, fract: f32, factor: u32) -> FilterResult<TrcLut> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(0.0..=1.0).contains(&fract) {
        return Err(FilterError::InvalidParameters(
            "fract must be in [0.0, 1.0]".into(),
        ));
    }
    if factor < 1 {
        return Err(FilterError::InvalidParameters("factor must be >= 1".into()));
    }

    let hist = pix.gray_histogram(factor)?;
    let sum: f32 = hist.sum().unwrap_or(0.0);

    // Handle empty histogram case (sum == 0.0) by returning identity mapping
    if sum == 0.0 || !sum.is_normal() {
        let mut lut = [0u8; 256];
        for (i, entry) in lut.iter_mut().enumerate() {
            *entry = i as u8;
        }
        return Ok(lut);
    }

    let partial = hist.partial_sums();

    let mut lut = [0u8; 256];
    for (iin, entry) in lut.iter_mut().enumerate() {
        let cumul = partial.get(iin).unwrap_or(0.0);
        let itarg = (255.0 * cumul / sum + 0.5) as i32;
        let iout = iin as i32 + (fract * (itarg - iin as i32) as f32) as i32;
        *entry = iout.clamp(0, 255) as u8;
    }

    Ok(lut)
}

/// Apply a TRC lookup table to an image in-place.
///
/// For 8 bpp images, each pixel value is remapped through the LUT.
/// For 32 bpp images, R, G, and B are each remapped independently
/// (alpha is not preserved).
///
/// # Arguments
///
/// * `pix` - Mutable 8 or 32 bpp image (not colormapped)
/// * `mask` - Optional 1 bpp mask; if provided, only pixels under
///   foreground mask pixels are modified.
/// * `lut` - 256-entry lookup table
///
/// # See also
///
/// C Leptonica: `pixTRCMap()` in `enhance.c`
pub fn trc_map(pix: &mut PixMut, mask: Option<&Pix>, lut: &TrcLut) -> FilterResult<()> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    if let Some(m) = mask
        && m.depth() != PixelDepth::Bit1
    {
        return Err(FilterError::UnsupportedDepth {
            expected: "1 bpp mask",
            actual: m.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    match (d, mask) {
        (PixelDepth::Bit8, None) => {
            for y in 0..h {
                for x in 0..w {
                    let val = pix.get_pixel_unchecked(x, y) as u8;
                    pix.set_pixel_unchecked(x, y, lut[val as usize] as u32);
                }
            }
        }
        (PixelDepth::Bit32, None) => {
            for y in 0..h {
                for x in 0..w {
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = color::extract_rgba(pixel);
                    let nr = lut[r as usize];
                    let ng = lut[g as usize];
                    let nb = lut[b as usize];
                    pix.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
                }
            }
        }
        (PixelDepth::Bit8, Some(m)) => {
            let mw = m.width();
            let mh = m.height();
            for y in 0..h.min(mh) {
                for x in 0..w.min(mw) {
                    if m.get_pixel_unchecked(x, y) == 0 {
                        continue;
                    }
                    let val = pix.get_pixel_unchecked(x, y) as u8;
                    pix.set_pixel_unchecked(x, y, lut[val as usize] as u32);
                }
            }
        }
        (PixelDepth::Bit32, Some(m)) => {
            let mw = m.width();
            let mh = m.height();
            for y in 0..h.min(mh) {
                for x in 0..w.min(mw) {
                    if m.get_pixel_unchecked(x, y) == 0 {
                        continue;
                    }
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = color::extract_rgba(pixel);
                    let nr = lut[r as usize];
                    let ng = lut[g as usize];
                    let nb = lut[b as usize];
                    pix.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Apply separate R, G, B TRC lookup tables to a 32 bpp image in-place.
///
/// Each color channel is remapped through its own LUT independently.
///
/// # Arguments
///
/// * `pix` - Mutable 32 bpp RGB image (not colormapped)
/// * `mask` - Optional 1 bpp mask
/// * `lut_r` - Red channel lookup table
/// * `lut_g` - Green channel lookup table
/// * `lut_b` - Blue channel lookup table
///
/// # See also
///
/// C Leptonica: `pixTRCMapGeneral()` in `enhance.c`
pub fn trc_map_general(
    pix: &mut PixMut,
    mask: Option<&Pix>,
    lut_r: &TrcLut,
    lut_g: &TrcLut,
    lut_b: &TrcLut,
) -> FilterResult<()> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if let Some(m) = mask
        && m.depth() != PixelDepth::Bit1
    {
        return Err(FilterError::UnsupportedDepth {
            expected: "1 bpp mask",
            actual: m.depth().bits(),
        });
    }

    let w = pix.width();
    let h = pix.height();

    match mask {
        None => {
            for y in 0..h {
                for x in 0..w {
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = color::extract_rgba(pixel);
                    let nr = lut_r[r as usize];
                    let ng = lut_g[g as usize];
                    let nb = lut_b[b as usize];
                    pix.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
                }
            }
        }
        Some(m) => {
            let mw = m.width();
            let mh = m.height();
            for y in 0..h.min(mh) {
                for x in 0..w.min(mw) {
                    if m.get_pixel_unchecked(x, y) == 0 {
                        continue;
                    }
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = color::extract_rgba(pixel);
                    let nr = lut_r[r as usize];
                    let ng = lut_g[g as usize];
                    let nb = lut_b[b as usize];
                    pix.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
                }
            }
        }
    }

    Ok(())
}

// =========================================================================
//  High-level gamma / contrast / equalization wrappers
// =========================================================================

/// Apply gamma TRC to an 8 or 32 bpp image.
///
/// Returns a new image with gamma correction applied.
///
/// # See also
///
/// C Leptonica: `pixGammaTRC()` in `enhance.c`
pub fn gamma_trc_pix(pix: &Pix, gamma: f32, minval: i32, maxval: i32) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    let lut = gamma_trc(gamma, minval, maxval)?;
    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    trc_map(&mut pm, None, &lut)?;
    Ok(pm.into())
}

/// Apply gamma TRC with an optional 1 bpp mask.
///
/// Only pixels under the foreground of `mask` are modified.
/// If `mask` is `None`, the entire image is modified.
///
/// # See also
///
/// C Leptonica: `pixGammaTRCMasked()` in `enhance.c`
pub fn gamma_trc_masked(
    pix: &Pix,
    mask: Option<&Pix>,
    gamma: f32,
    minval: i32,
    maxval: i32,
) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    let lut = gamma_trc(gamma, minval, maxval)?;
    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    trc_map(&mut pm, mask, &lut)?;
    Ok(pm.into())
}

/// Apply gamma TRC to a 32 bpp image, preserving the alpha channel.
///
/// # See also
///
/// C Leptonica: `pixGammaTRCWithAlpha()` in `enhance.c`
pub fn gamma_trc_with_alpha(pix: &Pix, gamma: f32, minval: i32, maxval: i32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    let alpha = pix.get_rgb_component(RgbComponent::Alpha)?;
    let lut = gamma_trc(gamma, minval, maxval)?;
    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    trc_map(&mut pm, None, &lut)?;
    pm.set_rgb_component(&alpha, RgbComponent::Alpha)?;
    Ok(pm.into())
}

/// Apply contrast enhancement TRC to an 8 or 32 bpp image.
///
/// # See also
///
/// C Leptonica: `pixContrastTRC()` in `enhance.c`
pub fn contrast_trc_pix(pix: &Pix, factor: f32) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    let lut = contrast_trc(factor)?;
    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    trc_map(&mut pm, None, &lut)?;
    Ok(pm.into())
}

/// Apply contrast enhancement TRC with an optional 1 bpp mask.
///
/// # See also
///
/// C Leptonica: `pixContrastTRCMasked()` in `enhance.c`
pub fn contrast_trc_masked(pix: &Pix, mask: Option<&Pix>, factor: f32) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }
    let lut = contrast_trc(factor)?;
    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    trc_map(&mut pm, mask, &lut)?;
    Ok(pm.into())
}

/// Apply histogram equalization to an 8 or 32 bpp image.
///
/// For 32 bpp images, each R/G/B channel is equalized independently.
///
/// # See also
///
/// C Leptonica: `pixEqualizeTRC()` in `enhance.c`
pub fn equalize_trc_pix(pix: &Pix, fract: f32, factor: u32) -> FilterResult<Pix> {
    let d = pix.depth();
    if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8 or 32 bpp",
            actual: d.bits(),
        });
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();

    if d == PixelDepth::Bit8 {
        let lut = equalize_trc(pix, fract, factor)?;
        trc_map(&mut pm, None, &lut)?;
    } else {
        // 32bpp: equalize each channel separately
        let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
        let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
        let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;
        let lut_r = equalize_trc(&pix_r, fract, factor)?;
        let lut_g = equalize_trc(&pix_g, fract, factor)?;
        let lut_b = equalize_trc(&pix_b, fract, factor)?;
        trc_map_general(&mut pm, None, &lut_r, &lut_g, &lut_b)?;
    }

    Ok(pm.into())
}

// =========================================================================
//  HSV modification
// =========================================================================

/// Modify the hue of a 32 bpp RGB image.
///
/// `fract` is in [-1.0, 1.0] and represents a fractional rotation of the
/// hue wheel. 1.0 (or -1.0) is a full 360-degree rotation (no visible
/// change). 0.0 also produces no change.
///
/// # See also
///
/// C Leptonica: `pixModifyHue()` in `enhance.c`
pub fn modify_hue(pix: &Pix, fract: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if fract.abs() > 1.0 {
        return Err(FilterError::InvalidParameters(
            "fract must be in [-1.0, 1.0]".into(),
        ));
    }

    let delhue = (240.0 * fract) as i32;
    if delhue == 0 || delhue == 240 || delhue == -240 {
        return Ok(pix.deep_clone());
    }
    let delhue = if delhue < 0 { delhue + 240 } else { delhue };

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let mut hsv = color::rgb_to_hsv(r, g, b);
            hsv.h = (hsv.h + delhue) % 240;
            let (nr, ng, nb) = color::hsv_to_rgb(hsv);
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

/// Modify the saturation of a 32 bpp RGB image.
///
/// `fract` is in [-1.0, 1.0]:
/// - Positive: moves saturation toward 255 (fully saturated)
/// - Negative: moves saturation toward 0 (desaturated/gray)
/// - 0.0: no change
///
/// # See also
///
/// C Leptonica: `pixModifySaturation()` in `enhance.c`
pub fn modify_saturation(pix: &Pix, fract: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if fract.abs() > 1.0 {
        return Err(FilterError::InvalidParameters(
            "fract must be in [-1.0, 1.0]".into(),
        ));
    }
    if fract == 0.0 {
        return Ok(pix.deep_clone());
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let mut hsv = color::rgb_to_hsv(r, g, b);
            if fract < 0.0 {
                hsv.s = (hsv.s as f32 * (1.0 + fract)) as i32;
            } else {
                hsv.s = (hsv.s as f32 + fract * (255.0 - hsv.s as f32)) as i32;
            }
            hsv.s = hsv.s.clamp(0, 255);
            let (nr, ng, nb) = color::hsv_to_rgb(hsv);
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

/// Modify the brightness (V in HSV) of a 32 bpp RGB image.
///
/// `fract` is in [-1.0, 1.0]:
/// - Positive: moves brightness toward 255
/// - Negative: moves brightness toward 0
/// - 0.0: no change
///
/// # See also
///
/// C Leptonica: `pixModifyBrightness()` in `enhance.c`
pub fn modify_brightness(pix: &Pix, fract: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if fract.abs() > 1.0 {
        return Err(FilterError::InvalidParameters(
            "fract must be in [-1.0, 1.0]".into(),
        ));
    }
    if fract == 0.0 {
        return Ok(pix.deep_clone());
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let mut hsv = color::rgb_to_hsv(r, g, b);
            if fract > 0.0 {
                hsv.v = (hsv.v as f32 + fract * (255.0 - hsv.v as f32)) as i32;
            } else {
                hsv.v = (hsv.v as f32 * (1.0 + fract)) as i32;
            }
            hsv.v = hsv.v.clamp(0, 255);
            let (nr, ng, nb) = color::hsv_to_rgb(hsv);
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

/// Measure the average saturation of a 32 bpp RGB image.
///
/// Returns the mean saturation value (in [0..255]) computed over a
/// subsampled grid of pixels.
///
/// # See also
///
/// C Leptonica: `pixMeasureSaturation()` in `enhance.c`
pub fn measure_saturation(pix: &Pix, factor: u32) -> FilterResult<f32> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if factor < 1 {
        return Err(FilterError::InvalidParameters("factor must be >= 1".into()));
    }

    let w = pix.width();
    let h = pix.height();
    let mut sum: i64 = 0;
    let mut count: i64 = 0;

    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let hsv = color::rgb_to_hsv(r, g, b);
            sum += hsv.s as i64;
            count += 1;
            x += factor;
        }
        y += factor;
    }

    if count > 0 {
        Ok(sum as f32 / count as f32)
    } else {
        Ok(0.0)
    }
}

// =========================================================================
//  Color shift and matrix operations
// =========================================================================

/// Shift each RGB channel of a 32 bpp image by a fractional amount.
///
/// For each channel, `fract` in [-1.0, 1.0]:
/// - Positive: pushes values toward 255 (`out = val + (255 - val) * fract`)
/// - Negative: pushes values toward 0 (`out = val * (1.0 + fract)`)
/// - 0.0: no change
///
/// # See also
///
/// C Leptonica: `pixColorShiftRGB()` in `enhance.c`
pub fn color_shift_rgb(pix: &Pix, rfract: f32, gfract: f32, bfract: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    for (name, fract) in [("rfract", rfract), ("gfract", gfract), ("bfract", bfract)] {
        if !(-1.0..=1.0).contains(&fract) {
            return Err(FilterError::InvalidParameters(format!(
                "{name} must be in [-1.0, 1.0]"
            )));
        }
    }
    if rfract == 0.0 && gfract == 0.0 && bfract == 0.0 {
        return Ok(pix.deep_clone());
    }

    // Build LUTs for each channel
    let build_lut = |fract: f32| -> [u8; 256] {
        let mut lut = [0u8; 256];
        for (i, entry) in lut.iter_mut().enumerate() {
            let val = i as f32;
            let out = if fract >= 0.0 {
                val + (255.0 - val) * fract
            } else {
                val * (1.0 + fract)
            };
            *entry = out.round().clamp(0.0, 255.0) as u8;
        }
        lut
    };
    let rlut = build_lut(rfract);
    let glut = build_lut(gfract);
    let blut = build_lut(bfract);

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            pm.set_pixel_unchecked(
                x,
                y,
                color::compose_rgb(rlut[r as usize], glut[g as usize], blut[b as usize]),
            );
        }
    }

    Ok(pm.into())
}

/// Darken low-saturation (gray) pixels in a 32 bpp RGB image.
///
/// Pixels where `max(r,g,b) < thresh` AND `max - min < satlimit` are
/// darkened by multiplying each channel by `(max - min) / satlimit`.
///
/// # See also
///
/// C Leptonica: `pixDarkenGray()` in `enhance.c`
pub fn darken_gray(pix: &Pix, thresh: u32, satlimit: u32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if thresh > 255 {
        return Err(FilterError::InvalidParameters(
            "thresh must be in [0, 255]".into(),
        ));
    }
    if satlimit < 1 {
        return Err(FilterError::InvalidParameters(
            "satlimit must be >= 1".into(),
        ));
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let ri = r as i32;
            let gi = g as i32;
            let bi = b as i32;
            let min = ri.min(gi).min(bi);
            let max = ri.max(gi).max(bi);
            let sat = max - min;

            if max >= thresh as i32 || sat >= satlimit as i32 {
                continue;
            }

            let ratio = sat as f32 / satlimit as f32;
            let nr = (ri as f32 * ratio) as u8;
            let ng = (gi as f32 * ratio) as u8;
            let nb = (bi as f32 * ratio) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

/// Multiply each channel by a constant factor (clipped to [0, 255]).
///
/// Factors must be >= 0.0 (can be > 1.0 for amplification).
/// Supports 32 bpp RGB images.
///
/// # See also
///
/// C Leptonica: `pixMultConstantColor()` in `enhance.c`
pub fn mult_constant_color(pix: &Pix, rfact: f32, gfact: f32, bfact: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    for (name, fact) in [("rfact", rfact), ("gfact", gfact), ("bfact", bfact)] {
        if fact < 0.0 {
            return Err(FilterError::InvalidParameters(format!(
                "{name} must be >= 0.0"
            )));
        }
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let nr = (r as f32 * rfact).round().min(255.0) as u8;
            let ng = (g as f32 * gfact).round().min(255.0) as u8;
            let nb = (b as f32 * bfact).round().min(255.0) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

/// Apply a 3×3 color matrix transformation to each pixel.
///
/// The kernel must be exactly 3×3. Each output channel is the dot product
/// of the corresponding kernel row with the input (R, G, B) vector,
/// clipped to [0, 255].
///
/// # See also
///
/// C Leptonica: `pixMultMatrixColor()` in `enhance.c`
pub fn mult_matrix_color(pix: &Pix, kel: &Kernel) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(FilterError::UnsupportedDepth {
            expected: "32 bpp",
            actual: pix.depth().bits(),
        });
    }
    if kel.width() != 3 || kel.height() != 3 {
        return Err(FilterError::InvalidParameters("kernel must be 3x3".into()));
    }

    // Extract 9 kernel elements (row-major: v[row][col])
    let mut v = [0.0f32; 9];
    for row in 0..3u32 {
        for col in 0..3u32 {
            v[(row * 3 + col) as usize] = kel.get(col, row).unwrap_or(0.0);
        }
    }

    let cloned = pix.deep_clone();
    let mut pm = cloned.try_into_mut().unwrap();
    let w = pm.width();
    let h = pm.height();

    for y in 0..h {
        for x in 0..w {
            let pixel = pm.get_pixel_unchecked(x, y);
            let (r, g, b) = color::extract_rgb(pixel);
            let rf = r as f32;
            let gf = g as f32;
            let bf = b as f32;

            let nr = (v[0] * rf + v[1] * gf + v[2] * bf)
                .round()
                .clamp(0.0, 255.0) as u8;
            let ng = (v[3] * rf + v[4] * gf + v[5] * bf)
                .round()
                .clamp(0.0, 255.0) as u8;
            let nb = (v[6] * rf + v[7] * gf + v[8] * bf)
                .round()
                .clamp(0.0, 255.0) as u8;
            pm.set_pixel_unchecked(x, y, color::compose_rgb(nr, ng, nb));
        }
    }

    Ok(pm.into())
}

// ============================================================================
// Phase 4: unsharp masking (precise, Gaussian-based)
// ============================================================================

/// Apply unsharp masking to an 8bpp grayscale image using box convolution.
///
/// For `halfwidth` <= 2, delegates to the fast block-convolution version.
/// For larger kernels, applies box convolution for the blur and then computes:
/// `output = pix + fract * (pix - blur)`
///
/// C版: `pixUnsharpMaskingGray()` in `enhance.c`
///
/// # Arguments
/// * `pix` - 8bpp grayscale image (no colormap)
/// * `halfwidth` - Half-width of the smoothing kernel; must be >= 1
/// * `fract` - Fraction of the high-pass signal to add back; must be > 0.0
pub fn unsharp_masking_gray(pix: &Pix, halfwidth: u32, fract: f32) -> FilterResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(FilterError::UnsupportedDepth {
            expected: "8bpp",
            actual: pix.depth().bits(),
        });
    }
    if pix.colormap().is_some() {
        return Err(FilterError::InvalidParameters(
            "colormapped images not supported".into(),
        ));
    }
    if fract <= 0.0 || halfwidth == 0 {
        return Ok(pix.deep_clone());
    }
    if halfwidth <= 2 {
        use crate::edge::unsharp_masking_gray_fast;
        return unsharp_masking_gray_fast(pix, halfwidth, fract);
    }

    let blurred = crate::block_conv::blockconv_gray(pix, None, halfwidth, halfwidth)?;
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let src = pix.get_pixel_unchecked(x, y) as f32;
            let blur = blurred.get_pixel_unchecked(x, y) as f32;
            let result = (src + fract * (src - blur) + 0.5) as i32;
            out_mut.set_pixel_unchecked(x, y, result.clamp(0, 255) as u32);
        }
    }

    Ok(out_mut.into())
}

/// Apply unsharp masking to an 8bpp grayscale or 32bpp color image.
///
/// For 1bpp input, returns an error. For `halfwidth` <= 2, delegates to the
/// fast version. For other depths, converts to 8 or 32 bpp first.
///
/// C版: `pixUnsharpMasking()` in `enhance.c`
///
/// # Arguments
/// * `pix` - Any depth except 1bpp; with or without colormap
/// * `halfwidth` - Half-width of the smoothing kernel; must be >= 1
/// * `fract` - Fraction of the high-pass signal to add back; must be > 0.0
pub fn unsharp_masking(pix: &Pix, halfwidth: u32, fract: f32) -> FilterResult<Pix> {
    if pix.depth() == PixelDepth::Bit1 {
        return Err(FilterError::UnsupportedDepth {
            expected: "not 1bpp",
            actual: 1,
        });
    }
    if fract <= 0.0 || halfwidth == 0 {
        return Ok(pix.deep_clone());
    }
    if halfwidth <= 2 {
        use crate::edge::unsharp_masking_fast;
        return unsharp_masking_fast(pix, halfwidth, fract);
    }

    match pix.depth() {
        PixelDepth::Bit8 => unsharp_masking_gray(pix, halfwidth, fract),
        PixelDepth::Bit32 => {
            use leptonica_core::pix::RgbComponent;
            let pix_r = pix.get_rgb_component(RgbComponent::Red)?;
            let pix_g = pix.get_rgb_component(RgbComponent::Green)?;
            let pix_b = pix.get_rgb_component(RgbComponent::Blue)?;
            let res_r = unsharp_masking_gray(&pix_r, halfwidth, fract)?;
            let res_g = unsharp_masking_gray(&pix_g, halfwidth, fract)?;
            let res_b = unsharp_masking_gray(&pix_b, halfwidth, fract)?;
            let mut result = Pix::create_rgb_image(&res_r, &res_g, &res_b)?;
            // Preserve alpha channel for 32bpp RGBA images
            if pix.spp() == 4 {
                let pix_a = pix.get_rgb_component(RgbComponent::Alpha)?;
                let mut result_mut = result.try_into_mut().unwrap();
                result_mut.set_rgb_component(&pix_a, RgbComponent::Alpha)?;
                result = result_mut.into();
            }
            Ok(result)
        }
        _ => {
            let converted = pix.convert_to_8_or_32()?;
            unsharp_masking(&converted, halfwidth, fract)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::Pix;

    // ========== gamma_trc tests ==========

    #[test]
    fn test_gamma_trc_identity() {
        // gamma=1.0, minval=0, maxval=255 should be identity
        let lut = gamma_trc(1.0, 0, 255).unwrap();
        for (i, &val) in lut.iter().enumerate() {
            assert_eq!(val, i as u8, "identity mismatch at {}", i);
        }
    }

    #[test]
    fn test_gamma_trc_lighten() {
        // gamma > 1.0 lightens: midtones should increase
        let lut = gamma_trc(2.0, 0, 255).unwrap();
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
        // Midpoint should be lighter (higher) than 128
        assert!(lut[128] > 128, "expected > 128, got {}", lut[128]);
    }

    #[test]
    fn test_gamma_trc_darken() {
        // gamma < 1.0 darkens: midtones should decrease
        let lut = gamma_trc(0.5, 0, 255).unwrap();
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
        assert!(lut[128] < 128, "expected < 128, got {}", lut[128]);
    }

    #[test]
    fn test_gamma_trc_custom_range() {
        // minval=50, maxval=200: values below 50 map to 0, above 200 to 255
        let lut = gamma_trc(1.0, 50, 200).unwrap();
        assert_eq!(lut[0], 0);
        assert_eq!(lut[49], 0);
        assert_eq!(lut[200], 255);
        assert_eq!(lut[255], 255);
        // Value at 125 (midpoint of 50..200) should be ~128
        assert!((lut[125] as i32 - 128).abs() <= 2, "got {}", lut[125]);
    }

    #[test]
    fn test_gamma_trc_invalid_params() {
        assert!(gamma_trc(1.0, 200, 100).is_err()); // minval >= maxval
        assert!(gamma_trc(0.0, 0, 255).is_err()); // gamma <= 0
        assert!(gamma_trc(-1.0, 0, 255).is_err()); // gamma negative
    }

    // ========== contrast_trc tests ==========

    #[test]
    fn test_contrast_trc_zero_factor() {
        // factor=0 should be identity
        let lut = contrast_trc(0.0).unwrap();
        for (i, &val) in lut.iter().enumerate() {
            assert_eq!(val, i as u8, "identity mismatch at {}", i);
        }
    }

    #[test]
    fn test_contrast_trc_enhancement() {
        // Positive factor: dark pixels get darker, light pixels get lighter
        let lut = contrast_trc(0.5).unwrap();
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
        // Midpoint stays ~128
        assert!((lut[127] as i32 - 127).abs() <= 2, "got {}", lut[127]);
        // Dark pixel (64) should be darker
        assert!(lut[64] < 64, "expected < 64, got {}", lut[64]);
        // Light pixel (192) should be lighter
        assert!(lut[192] > 192, "expected > 192, got {}", lut[192]);
    }

    #[test]
    fn test_contrast_trc_monotonic() {
        let lut = contrast_trc(0.8).unwrap();
        for i in 1..256 {
            assert!(lut[i] >= lut[i - 1], "not monotonic at {}", i);
        }
    }

    #[test]
    fn test_contrast_trc_invalid_factor() {
        assert!(contrast_trc(-0.5).is_err());
    }

    // ========== equalize_trc tests ==========

    #[test]
    fn test_equalize_trc_uniform() {
        // Uniform image (all pixels = 0): full equalization maps the sole
        // populated bin to 255 via the cumulative distribution.
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let lut = equalize_trc(&pix, 1.0, 1).unwrap();
        assert_eq!(lut[0], 255);
    }

    #[test]
    fn test_equalize_trc_fract_zero() {
        // fract=0: identity mapping
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let lut = equalize_trc(&pix, 0.0, 1).unwrap();
        for (i, &val) in lut.iter().enumerate() {
            assert_eq!(val, i as u8, "identity mismatch at {}", i);
        }
    }

    #[test]
    fn test_equalize_trc_invalid_params() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(equalize_trc(&pix, -0.1, 1).is_err());
        assert!(equalize_trc(&pix, 1.5, 1).is_err());
        assert!(equalize_trc(&pix, 0.5, 0).is_err());

        let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(equalize_trc(&pix32, 0.5, 1).is_err());
    }

    // ========== trc_map tests ==========

    #[test]
    fn test_trc_map_8bpp_identity() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);

        let lut: TrcLut = core::array::from_fn(|i| i as u8);
        trc_map(&mut pm, None, &lut).unwrap();

        assert_eq!(pm.get_pixel_unchecked(0, 0), 0);
        assert_eq!(pm.get_pixel_unchecked(1, 0), 128);
        assert_eq!(pm.get_pixel_unchecked(2, 0), 255);
    }

    #[test]
    fn test_trc_map_8bpp_invert() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);

        let lut: TrcLut = core::array::from_fn(|i| (255 - i) as u8);
        trc_map(&mut pm, None, &lut).unwrap();

        assert_eq!(pm.get_pixel_unchecked(0, 0), 255);
        assert_eq!(pm.get_pixel_unchecked(1, 0), 127);
        assert_eq!(pm.get_pixel_unchecked(2, 0), 0);
    }

    #[test]
    fn test_trc_map_32bpp() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));

        // Double all channels (clamped at 255)
        let lut: TrcLut = core::array::from_fn(|i| (i * 2).min(255) as u8);
        trc_map(&mut pm, None, &lut).unwrap();

        let (r, g, b) = color::extract_rgb(pm.get_pixel_unchecked(0, 0));
        assert_eq!(r, 200);
        assert_eq!(g, 255); // 150*2=300, clamped to 255
        assert_eq!(b, 255); // 200*2=400, clamped to 255
    }

    #[test]
    fn test_trc_map_with_mask() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 100);
        pm.set_pixel_unchecked(1, 0, 100);
        pm.set_pixel_unchecked(2, 0, 100);

        let mask = Pix::new(3, 1, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1); // ON: apply
        mm.set_pixel_unchecked(1, 0, 0); // OFF: skip
        mm.set_pixel_unchecked(2, 0, 1); // ON: apply
        let mask: Pix = mm.into();

        let lut: TrcLut = core::array::from_fn(|i| (255 - i) as u8); // invert
        trc_map(&mut pm, Some(&mask), &lut).unwrap();

        assert_eq!(pm.get_pixel_unchecked(0, 0), 155); // inverted
        assert_eq!(pm.get_pixel_unchecked(1, 0), 100); // unchanged
        assert_eq!(pm.get_pixel_unchecked(2, 0), 155); // inverted
    }

    #[test]
    fn test_trc_map_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let lut: TrcLut = core::array::from_fn(|i| i as u8);
        assert!(trc_map(&mut pm, None, &lut).is_err());
    }

    // ========== trc_map_general tests ==========

    #[test]
    fn test_trc_map_general_separate_channels() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 100, 100));

        // R: invert, G: identity, B: zero
        let lut_r: TrcLut = core::array::from_fn(|i| (255 - i) as u8);
        let lut_g: TrcLut = core::array::from_fn(|i| i as u8);
        let lut_b: TrcLut = [0u8; 256];

        trc_map_general(&mut pm, None, &lut_r, &lut_g, &lut_b).unwrap();

        let (r, g, b) = color::extract_rgb(pm.get_pixel_unchecked(0, 0));
        assert_eq!(r, 155); // 255 - 100
        assert_eq!(g, 100); // identity
        assert_eq!(b, 0); // zeroed
    }

    #[test]
    fn test_trc_map_general_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let lut: TrcLut = core::array::from_fn(|i| i as u8);
        assert!(trc_map_general(&mut pm, None, &lut, &lut, &lut).is_err());
    }

    // ========== gamma_trc_pix tests ==========

    #[test]
    fn test_gamma_trc_pix_8bpp() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 255);
        let pix: Pix = pm.into();

        let result = gamma_trc_pix(&pix, 2.0, 0, 255).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        assert_eq!(result.get_pixel_unchecked(0, 0), 0);
        assert_eq!(result.get_pixel_unchecked(2, 0), 255);
        assert!(result.get_pixel_unchecked(1, 0) > 128);
    }

    #[test]
    fn test_gamma_trc_pix_32bpp() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 100, 100));
        let pix: Pix = pm.into();

        let result = gamma_trc_pix(&pix, 2.0, 0, 255).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert!(r > 100);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn test_gamma_trc_pix_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(gamma_trc_pix(&pix, 1.0, 0, 255).is_err());
    }

    // ========== gamma_trc_masked tests ==========

    #[test]
    fn test_gamma_trc_masked_partial() {
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 128);
        pm.set_pixel_unchecked(1, 0, 128);
        let pix: Pix = pm.into();

        let mask = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(1, 0, 0);
        let mask: Pix = mm.into();

        let result = gamma_trc_masked(&pix, Some(&mask), 2.0, 0, 255).unwrap();
        assert!(result.get_pixel_unchecked(0, 0) > 128);
        assert_eq!(result.get_pixel_unchecked(1, 0), 128);
    }

    #[test]
    fn test_gamma_trc_masked_no_mask() {
        let pix = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 128);
        let pix: Pix = pm.into();

        let result = gamma_trc_masked(&pix, None, 2.0, 0, 255).unwrap();
        assert!(result.get_pixel_unchecked(0, 0) > 128);
    }

    // ========== gamma_trc_with_alpha tests ==========

    #[test]
    fn test_gamma_trc_with_alpha_preserves_alpha() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        pm.set_pixel_unchecked(0, 0, color::compose_rgba(100, 100, 100, 128));
        let pix: Pix = pm.into();

        let result = gamma_trc_with_alpha(&pix, 2.0, 0, 255).unwrap();
        let (r, _, _, a) = color::extract_rgba(result.get_pixel_unchecked(0, 0));
        assert!(r > 100);
        assert_eq!(a, 128);
    }

    #[test]
    fn test_gamma_trc_with_alpha_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(gamma_trc_with_alpha(&pix, 1.0, 0, 255).is_err());
    }

    // ========== contrast_trc_pix tests ==========

    #[test]
    fn test_contrast_trc_pix_enhancement() {
        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 64);
        pm.set_pixel_unchecked(1, 0, 128);
        pm.set_pixel_unchecked(2, 0, 192);
        let pix: Pix = pm.into();

        let result = contrast_trc_pix(&pix, 0.5).unwrap();
        assert!(result.get_pixel_unchecked(0, 0) < 64);
        assert!(result.get_pixel_unchecked(2, 0) > 192);
    }

    #[test]
    fn test_contrast_trc_pix_zero_factor() {
        let pix = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 100);
        let pix: Pix = pm.into();

        let result = contrast_trc_pix(&pix, 0.0).unwrap();
        assert_eq!(result.get_pixel_unchecked(0, 0), 100);
    }

    // ========== contrast_trc_masked tests ==========

    #[test]
    fn test_contrast_trc_masked() {
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 64);
        pm.set_pixel_unchecked(1, 0, 64);
        let pix: Pix = pm.into();

        let mask = Pix::new(2, 1, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(1, 0, 0);
        let mask: Pix = mm.into();

        let result = contrast_trc_masked(&pix, Some(&mask), 0.5).unwrap();
        assert!(result.get_pixel_unchecked(0, 0) < 64);
        assert_eq!(result.get_pixel_unchecked(1, 0), 64);
    }

    // ========== equalize_trc_pix tests ==========

    #[test]
    fn test_equalize_trc_pix_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..10 {
            for x in 0..5 {
                pm.set_pixel_unchecked(x, y, 50);
            }
            for x in 5..10 {
                pm.set_pixel_unchecked(x, y, 200);
            }
        }
        let pix: Pix = pm.into();

        let result = equalize_trc_pix(&pix, 0.5, 1).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit8);
        let v0 = result.get_pixel_unchecked(0, 0);
        let v1 = result.get_pixel_unchecked(5, 0);
        assert!(v1 > v0);
    }

    #[test]
    fn test_equalize_trc_pix_zero_fract() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 100);
        let pix: Pix = pm.into();

        let result = equalize_trc_pix(&pix, 0.0, 1).unwrap();
        assert_eq!(result.get_pixel_unchecked(0, 0), 100);
    }

    // ========== modify_hue tests ==========

    #[test]
    fn test_modify_hue_shift() {
        // Pure red pixel: shift hue by 1/3 → should move toward green
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(255, 0, 0));
        let pix: Pix = pm.into();

        let result = modify_hue(&pix, 1.0 / 3.0).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        // After hue shift, red should decrease, green should increase
        assert!(g > r, "expected green > red, got r={r} g={g} b={b}");
    }

    #[test]
    fn test_modify_hue_zero() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));
        let pix: Pix = pm.into();

        // fract=0: no change
        let result = modify_hue(&pix, 0.0).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    #[test]
    fn test_modify_hue_invalid() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(modify_hue(&pix, 1.5).is_err());

        let pix8 = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        assert!(modify_hue(&pix8, 0.5).is_err());
    }

    // ========== modify_saturation tests ==========

    #[test]
    fn test_modify_saturation_increase() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 100, 100));
        let pix: Pix = pm.into();

        let result = modify_saturation(&pix, 0.5).unwrap();
        let (r, _, _) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        // Increasing saturation should push the dominant channel higher
        assert!(r >= 200, "expected r >= 200, got {r}");
    }

    #[test]
    fn test_modify_saturation_zero() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));
        let pix: Pix = pm.into();

        let result = modify_saturation(&pix, 0.0).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    #[test]
    fn test_modify_saturation_desaturate() {
        // fract=-1.0 should fully desaturate (s=0, so r=g=b=v)
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 100, 50));
        let pix: Pix = pm.into();

        let result = modify_saturation(&pix, -1.0).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        // Fully desaturated: all channels equal to V (=max=200)
        assert_eq!(r, g);
        assert_eq!(g, b);
        assert_eq!(r, 200);
    }

    // ========== modify_brightness tests ==========

    #[test]
    fn test_modify_brightness_increase() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 50, 25));
        let pix: Pix = pm.into();

        let result = modify_brightness(&pix, 0.5).unwrap();
        let (r, _, _) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert!(r > 100, "expected brighter, got r={r}");
    }

    #[test]
    fn test_modify_brightness_decrease() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 150, 100));
        let pix: Pix = pm.into();

        let result = modify_brightness(&pix, -0.5).unwrap();
        let (r, _, _) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert!(r < 200, "expected darker, got r={r}");
    }

    #[test]
    fn test_modify_brightness_zero() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));
        let pix: Pix = pm.into();

        let result = modify_brightness(&pix, 0.0).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    // ========== measure_saturation tests ==========

    #[test]
    fn test_measure_saturation_gray() {
        // Gray image: saturation should be 0
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(128, 128, 128));
            }
        }
        let pix: Pix = pm.into();

        let sat = measure_saturation(&pix, 1).unwrap();
        assert_eq!(sat, 0.0);
    }

    #[test]
    fn test_measure_saturation_colored() {
        // Fully saturated red: saturation = 255
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(255, 0, 0));
            }
        }
        let pix: Pix = pm.into();

        let sat = measure_saturation(&pix, 1).unwrap();
        assert_eq!(sat, 255.0);
    }

    #[test]
    fn test_measure_saturation_invalid() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(measure_saturation(&pix, 1).is_err());

        let pix32 = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(measure_saturation(&pix32, 0).is_err());
    }

    // ========== color_shift_rgb tests ==========

    #[test]
    fn test_color_shift_rgb_positive() {
        // Shift red channel up: (100,100,100) with rfract=0.5
        // new_r = 100 + (255-100)*0.5 = 100 + 77.5 = 177
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 100, 100));
        let pix: Pix = pm.into();

        let result = color_shift_rgb(&pix, 0.5, 0.0, 0.0).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert!(r > 150, "expected r > 150, got r={r}");
        assert_eq!(g, 100);
        assert_eq!(b, 100);
    }

    #[test]
    fn test_color_shift_rgb_negative() {
        // Shift blue channel down: (200,200,200) with bfract=-0.5
        // new_b = 200 * (1.0 + (-0.5)) = 200 * 0.5 = 100
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 200, 200));
        let pix: Pix = pm.into();

        let result = color_shift_rgb(&pix, 0.0, 0.0, -0.5).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 200);
        assert_eq!(g, 200);
        assert_eq!(b, 100);
    }

    #[test]
    fn test_color_shift_rgb_zero() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));
        let pix: Pix = pm.into();

        let result = color_shift_rgb(&pix, 0.0, 0.0, 0.0).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    #[test]
    fn test_color_shift_rgb_invalid() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(color_shift_rgb(&pix, 1.5, 0.0, 0.0).is_err());

        let pix8 = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        assert!(color_shift_rgb(&pix8, 0.5, 0.0, 0.0).is_err());
    }

    // ========== darken_gray tests ==========

    #[test]
    fn test_darken_gray_low_saturation() {
        // Gray pixel (128,128,128): sat=0 < satlimit, max=128 < thresh=200
        // ratio = sat/satlimit = 0/10 = 0 → darkened to (0,0,0)
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(128, 128, 128));
        let pix: Pix = pm.into();

        let result = darken_gray(&pix, 200, 10).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_darken_gray_saturated_unchanged() {
        // Saturated pixel: sat=200 >= satlimit=10 → unchanged
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(255, 55, 55));
        let pix: Pix = pm.into();

        let result = darken_gray(&pix, 200, 10).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    #[test]
    fn test_darken_gray_bright_unchanged() {
        // Bright pixel: max=250 >= thresh=200 → unchanged
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(250, 248, 248));
        let pix: Pix = pm.into();

        let result = darken_gray(&pix, 200, 10).unwrap();
        assert_eq!(
            result.get_pixel_unchecked(0, 0),
            pix.get_pixel_unchecked(0, 0)
        );
    }

    #[test]
    fn test_darken_gray_invalid() {
        let pix8 = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        assert!(darken_gray(&pix8, 200, 10).is_err());
    }

    // ========== mult_constant_color tests ==========

    #[test]
    fn test_mult_constant_color_basic() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 200, 50));
        let pix: Pix = pm.into();

        let result = mult_constant_color(&pix, 0.5, 1.0, 2.0).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 50);
        assert_eq!(g, 200);
        assert_eq!(b, 100);
    }

    #[test]
    fn test_mult_constant_color_clipping() {
        // 200 * 2.0 = 400 → clipped to 255
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(200, 200, 200));
        let pix: Pix = pm.into();

        let result = mult_constant_color(&pix, 2.0, 2.0, 2.0).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_mult_constant_color_invalid() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(mult_constant_color(&pix, -0.5, 1.0, 1.0).is_err());
    }

    // ========== mult_matrix_color tests ==========

    #[test]
    fn test_mult_matrix_color_identity() {
        use crate::Kernel;

        // Identity matrix → no change
        let kel = Kernel::from_slice(3, 3, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 150, 200));
        let pix: Pix = pm.into();

        let result = mult_matrix_color(&pix, &kel).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 100);
        assert_eq!(g, 150);
        assert_eq!(b, 200);
    }

    #[test]
    fn test_mult_matrix_color_swap_rg() {
        use crate::Kernel;

        // Swap R and G channels
        let kel = Kernel::from_slice(3, 3, &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 200, 50));
        let pix: Pix = pm.into();

        let result = mult_matrix_color(&pix, &kel).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 200);
        assert_eq!(g, 100);
        assert_eq!(b, 50);
    }

    #[test]
    fn test_mult_matrix_color_clipping() {
        use crate::Kernel;

        // All channels sum: each output = r + g + b, clipped
        let kel = Kernel::from_slice(3, 3, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]).unwrap();
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, color::compose_rgb(100, 100, 100));
        let pix: Pix = pm.into();

        let result = mult_matrix_color(&pix, &kel).unwrap();
        let (r, g, b) = color::extract_rgb(result.get_pixel_unchecked(0, 0));
        assert_eq!(r, 255); // 100+100+100=300 → 255
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_mult_matrix_color_invalid_size() {
        use crate::Kernel;

        // 2x2 kernel should fail
        let kel = Kernel::from_slice(2, 2, &[1.0, 0.0, 0.0, 1.0]).unwrap();
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        assert!(mult_matrix_color(&pix, &kel).is_err());
    }

    // -------------------------------------------------------------------------
    // Phase 4: unsharp_masking / unsharp_masking_gray tests
    // -------------------------------------------------------------------------

    fn create_8bpp_gradient() -> Pix {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                pm.set_pixel_unchecked(x, y, (x * 8).min(255));
            }
        }
        pm.into()
    }

    #[test]
    fn test_unsharp_masking_gray_basic() {
        let pix = create_8bpp_gradient();
        let result = unsharp_masking_gray(&pix, 3, 0.5).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_unsharp_masking_gray_no_sharpening() {
        let pix = create_8bpp_gradient();
        // fract <= 0 should return a clone
        let result = unsharp_masking_gray(&pix, 3, 0.0).unwrap();
        for y in 0..pix.height() {
            for x in 0..pix.width() {
                assert_eq!(
                    result.get_pixel_unchecked(x, y),
                    pix.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]
    fn test_unsharp_masking_color() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                let v = (x * 8).min(255) as u8;
                pm.set_pixel_unchecked(x, y, leptonica_core::color::compose_rgb(v, v, v));
            }
        }
        let pix: Pix = pm.into();
        let result = unsharp_masking(&pix, 3, 0.5).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
        assert_eq!(result.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_unsharp_masking_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(unsharp_masking(&pix, 3, 0.5).is_err());
    }
}
