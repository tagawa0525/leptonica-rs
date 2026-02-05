//! Warping transformations for images
//!
//! This module provides image warping operations including:
//! - Random harmonic (sinusoidal) warping for CAPTCHA generation
//! - Stereoscopic warping for 3D anaglyph effects
//! - Horizontal stretching (linear and quadratic)
//! - Quadratic vertical shear
//! - Stereo image pair composition
//!
//! # Random Harmonic Warp
//!
//! The random harmonic warp applies multiple sinusoidal distortions to create
//! unpredictable but smooth warping effects. This is commonly used for CAPTCHA
//! generation to make text recognition difficult for automated systems.
//!
//! The transformation is defined by:
//! ```text
//! x' = x + sum(xmag * rand * sin(anglex) * sin(angley))
//! y' = y + sum(ymag * rand * sin(angley) * sin(anglex))
//! ```
//!
//! # Stereoscopic Warp
//!
//! Creates a red-cyan anaglyph by shifting the red channel horizontally
//! to create a 3D stereoscopic effect when viewed with anaglyph glasses.
//!
//! # Example
//!
//! ```no_run
//! use leptonica_transform::warper::{
//!     random_harmonic_warp, stretch_horizontal, WarpDirection, WarpType, WarpOperation, WarpFill,
//! };
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
//!
//! // Apply random harmonic warp (useful for CAPTCHA)
//! let warped = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 3, 3, 42, 255).unwrap();
//!
//! // Apply horizontal stretch
//! let stretched = stretch_horizontal(
//!     &pix,
//!     WarpDirection::ToLeft,
//!     WarpType::Quadratic,
//!     10,
//!     WarpOperation::Interpolated,
//!     WarpFill::White,
//! ).unwrap();
//! ```

use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixMut, PixelDepth, color};
use std::f64::consts::PI;

// ============================================================================
// Constants
// ============================================================================

const TWO_PI: f64 = 2.0 * PI;

/// Default weights for stereo pair composition
const DEFAULT_RED_WEIGHT: f32 = 0.0;
const DEFAULT_GREEN_WEIGHT: f32 = 0.7;
const DEFAULT_BLUE_WEIGHT: f32 = 0.3;

// ============================================================================
// Type Definitions
// ============================================================================

/// Direction of warp transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarpDirection {
    /// Warp toward the left edge (L_WARP_TO_LEFT)
    ///
    /// For horizontal stretch: right edge is unchanged, left edge moves
    /// For vertical shear: right edge is unchanged, left edge pixels shift
    #[default]
    ToLeft,
    /// Warp toward the right edge (L_WARP_TO_RIGHT)
    ///
    /// For horizontal stretch: left edge is unchanged, right edge moves
    /// For vertical shear: left edge is unchanged, right edge pixels shift
    ToRight,
}

/// Type of warp function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarpType {
    /// Linear warp (L_LINEAR_WARP)
    ///
    /// Displacement varies linearly with distance from the fixed edge
    #[default]
    Linear,
    /// Quadratic warp (L_QUADRATIC_WARP)
    ///
    /// Displacement varies quadratically with distance from the fixed edge
    Quadratic,
}

/// Operation type for warp transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarpOperation {
    /// Sampled (nearest-neighbor) (L_SAMPLED)
    ///
    /// Fastest but lowest quality
    #[default]
    Sampled,
    /// Interpolated (linear) (L_INTERPOLATED)
    ///
    /// Higher quality but slower
    Interpolated,
}

/// Background fill color for warp transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarpFill {
    /// Fill with white pixels (L_BRING_IN_WHITE)
    #[default]
    White,
    /// Fill with black pixels (L_BRING_IN_BLACK)
    Black,
}

impl WarpFill {
    /// Get the fill value for a specific pixel depth
    pub fn to_value(self, depth: PixelDepth) -> u32 {
        match self {
            WarpFill::White => match depth {
                PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
                PixelDepth::Bit2 => 3,
                PixelDepth::Bit4 => 15,
                PixelDepth::Bit8 => 255,
                PixelDepth::Bit16 => 65535,
                PixelDepth::Bit32 => 0xFFFFFF00,
            },
            WarpFill::Black => match depth {
                PixelDepth::Bit1 => 1, // 1 = black for binary
                PixelDepth::Bit32 => 0x00000000,
                _ => 0,
            },
        }
    }
}

/// Parameters for stereoscopic warp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StereoscopicParams {
    /// Horizontal separation in pixels of red and cyan at edges
    /// Gives rise to quadratic curvature out of the image plane.
    /// Positive values curve away from viewer.
    pub zbend: i32,

    /// Uniform pixel translation at top that pushes the plane
    /// away from viewer (positive) or toward viewer (negative)
    pub zshift_top: i32,

    /// Uniform pixel translation at bottom that pushes the plane
    /// away from viewer (positive) or toward viewer (negative)
    pub zshift_bottom: i32,

    /// Vertical displacement at edges at top
    /// y = ybend_top * (2x/w - 1)^2
    pub ybend_top: i32,

    /// Vertical displacement at edges at bottom
    pub ybend_bottom: i32,

    /// True if red filter is on the left eye
    pub red_left: bool,
}

impl Default for StereoscopicParams {
    fn default() -> Self {
        Self {
            zbend: 20,
            zshift_top: 15,
            zshift_bottom: -15,
            ybend_top: 30,
            ybend_bottom: 0,
            red_left: true,
        }
    }
}

// ============================================================================
// Random Harmonic Warp
// ============================================================================

/// Apply random sinusoidal warping to an 8bpp grayscale image
///
/// This is equivalent to Leptonica's `pixRandomHarmonicWarp`.
///
/// The warping is computed as a sum of sinusoidal terms with random
/// amplitudes and phases. This creates smooth, unpredictable distortions
/// suitable for CAPTCHA generation.
///
/// # Arguments
/// * `pix` - Input image (8bpp, no colormap)
/// * `xmag` - Maximum magnitude of x distortion in pixels
/// * `ymag` - Maximum magnitude of y distortion in pixels
/// * `xfreq` - Maximum frequency of x distortion
/// * `yfreq` - Maximum frequency of y distortion
/// * `nx` - Number of x harmonic terms
/// * `ny` - Number of y harmonic terms
/// * `seed` - Random number generator seed for reproducibility
/// * `gray_val` - Fill value for pixels from outside (0-255)
///
/// # Returns
/// Warped 8bpp image
///
/// # Errors
/// Returns `TransformError::UnsupportedDepth` if input is not 8bpp.
///
/// # Example
/// ```no_run
/// use leptonica_transform::warper::random_harmonic_warp;
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let warped = random_harmonic_warp(&pix, 4.0, 6.0, 0.10, 0.13, 3, 3, 42, 255).unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn random_harmonic_warp(
    pix: &Pix,
    xmag: f32,
    ymag: f32,
    xfreq: f32,
    yfreq: f32,
    nx: u32,
    ny: u32,
    seed: u32,
    gray_val: u8,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth != PixelDepth::Bit8 || pix.colormap().is_some() {
        return Err(TransformError::UnsupportedDepth(
            "random_harmonic_warp requires 8bpp image without colormap".to_string(),
        ));
    }

    // Generate random number array
    let randa = generate_random_array(5 * (nx + ny) as usize, seed);

    // Create output image
    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let wi = w as i32;
    let hi = h as i32;

    // For each destination pixel, compute source location and interpolate
    for j in 0..hi {
        for i in 0..wi {
            let (x, y) = apply_warp_transform(
                xmag,
                ymag,
                xfreq,
                yfreq,
                &randa,
                nx as usize,
                ny as usize,
                i,
                j,
            );

            let val = linear_interpolate_gray(pix, w, h, x, y, gray_val);
            unsafe { out_mut.set_pixel_unchecked(i as u32, j as u32, val as u32) };
        }
    }

    Ok(out_mut.into())
}

/// Generate an array of random numbers in range [0.5, 1.0]
fn generate_random_array(size: usize, seed: u32) -> Vec<f64> {
    let mut rng = SimpleRng::new(seed);
    (0..size).map(|_| 0.5 * (1.0 + rng.next_f64())).collect()
}

/// Simple linear congruential generator for reproducible randomness
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: seed as u64 }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

/// Apply the harmonic warp transform at a point
#[allow(clippy::too_many_arguments)]
fn apply_warp_transform(
    xmag: f32,
    ymag: f32,
    xfreq: f32,
    yfreq: f32,
    randa: &[f64],
    nx: usize,
    ny: usize,
    xp: i32,
    yp: i32,
) -> (f32, f32) {
    let xmag = xmag as f64;
    let ymag = ymag as f64;
    let xfreq = xfreq as f64;
    let yfreq = yfreq as f64;
    let xp_f = xp as f64;
    let yp_f = yp as f64;

    // Compute x displacement
    let mut x = xp_f;
    for i in 0..nx {
        let anglex = xfreq * randa[3 * i + 1] * xp_f + TWO_PI * randa[3 * i + 2];
        let angley = yfreq * randa[3 * i + 3] * yp_f + TWO_PI * randa[3 * i + 4];
        x += xmag * randa[3 * i] * anglex.sin() * angley.sin();
    }

    // Compute y displacement
    let mut y = yp_f;
    for i in nx..(nx + ny) {
        let angley = yfreq * randa[3 * i + 1] * yp_f + TWO_PI * randa[3 * i + 2];
        let anglex = xfreq * randa[3 * i + 3] * xp_f + TWO_PI * randa[3 * i + 4];
        y += ymag * randa[3 * i] * angley.sin() * anglex.sin();
    }

    (x as f32, y as f32)
}

/// Linear interpolation for grayscale images
fn linear_interpolate_gray(pix: &Pix, w: u32, h: u32, x: f32, y: f32, fill_val: u8) -> u8 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;

    let wi = w as i32;
    let hi = h as i32;

    // Bounds check
    if xi < 0 || xi >= wi - 1 || yi < 0 || yi >= hi - 1 {
        return fill_val;
    }

    // Get four neighboring pixels
    let v00 = unsafe { pix.get_pixel_unchecked(xi as u32, yi as u32) } as f32;
    let v10 = unsafe { pix.get_pixel_unchecked((xi + 1) as u32, yi as u32) } as f32;
    let v01 = unsafe { pix.get_pixel_unchecked(xi as u32, (yi + 1) as u32) } as f32;
    let v11 = unsafe { pix.get_pixel_unchecked((xi + 1) as u32, (yi + 1) as u32) } as f32;

    // Bilinear interpolation
    let val = (1.0 - xf) * (1.0 - yf) * v00
        + xf * (1.0 - yf) * v10
        + (1.0 - xf) * yf * v01
        + xf * yf * v11;

    (val + 0.5).clamp(0.0, 255.0) as u8
}

// ============================================================================
// Horizontal Stretch
// ============================================================================

/// Apply linear or quadratic horizontal stretching
///
/// This is equivalent to Leptonica's `pixStretchHorizontal`.
///
/// # Arguments
/// * `pix` - Input image (1, 8, or 32 bpp)
/// * `direction` - Which edge is fixed (ToLeft = right fixed, ToRight = left fixed)
/// * `warp_type` - Linear or quadratic displacement function
/// * `hmax` - Maximum horizontal displacement at the moving edge
/// * `operation` - Sampled or interpolated
/// * `fill` - Background fill color
///
/// # Returns
/// Stretched image
///
/// # Notes
/// - If `hmax > 0`, pixels move in the positive x direction
/// - If `direction == ToLeft`, right edge pixels don't move; left edge moves by hmax
/// - If `direction == ToRight`, left edge pixels don't move; right edge moves by hmax
pub fn stretch_horizontal(
    pix: &Pix,
    direction: WarpDirection,
    warp_type: WarpType,
    hmax: i32,
    operation: WarpOperation,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let d = pix.depth();

    if d != PixelDepth::Bit1 && d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "stretch_horizontal requires 1, 8, or 32 bpp".to_string(),
        ));
    }

    // For 1bpp, use sampling regardless
    if d == PixelDepth::Bit1 || operation == WarpOperation::Sampled {
        stretch_horizontal_sampled(pix, direction, warp_type, hmax, fill)
    } else {
        stretch_horizontal_li(pix, direction, warp_type, hmax, fill)
    }
}

/// Horizontal stretch using nearest-neighbor sampling
///
/// This is equivalent to Leptonica's `pixStretchHorizontalSampled`.
pub fn stretch_horizontal_sampled(
    pix: &Pix,
    direction: WarpDirection,
    warp_type: WarpType,
    hmax: i32,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth != PixelDepth::Bit1 && depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "stretch_horizontal_sampled requires 1, 8, or 32 bpp".to_string(),
        ));
    }

    let fill_value = fill.to_value(depth);

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;
    let hi = h as i32;
    let wm = wi - 1;

    for jd in 0..wi {
        // Compute source x coordinate for destination jd
        let j = match (direction, warp_type) {
            (WarpDirection::ToLeft, WarpType::Linear) => jd - (hmax * (wm - jd)) / wm,
            (WarpDirection::ToLeft, WarpType::Quadratic) => {
                jd - (hmax * (wm - jd) * (wm - jd)) / (wm * wm)
            }
            (WarpDirection::ToRight, WarpType::Linear) => jd - (hmax * jd) / wm,
            (WarpDirection::ToRight, WarpType::Quadratic) => jd - (hmax * jd * jd) / (wm * wm),
        };

        if j < 0 || j >= wi {
            continue;
        }

        // Copy column from source to destination
        for i in 0..hi {
            let val = unsafe { pix.get_pixel_unchecked(j as u32, i as u32) };
            unsafe { out_mut.set_pixel_unchecked(jd as u32, i as u32, val) };
        }
    }

    Ok(out_mut.into())
}

/// Horizontal stretch using linear interpolation
///
/// This is equivalent to Leptonica's `pixStretchHorizontalLI`.
pub fn stretch_horizontal_li(
    pix: &Pix,
    direction: WarpDirection,
    warp_type: WarpType,
    hmax: i32,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "stretch_horizontal_li requires 8 or 32 bpp".to_string(),
        ));
    }

    let fill_value = fill.to_value(depth);

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;
    let hi = h as i32;
    let wm = wi - 1;

    for jd in 0..wi {
        // Compute source x coordinate (scaled by 64 for sub-pixel precision)
        let j = match (direction, warp_type) {
            (WarpDirection::ToLeft, WarpType::Linear) => 64 * jd - 64 * (hmax * (wm - jd)) / wm,
            (WarpDirection::ToLeft, WarpType::Quadratic) => {
                64 * jd - 64 * (hmax * (wm - jd) * (wm - jd)) / (wm * wm)
            }
            (WarpDirection::ToRight, WarpType::Linear) => 64 * jd - 64 * (hmax * jd) / wm,
            (WarpDirection::ToRight, WarpType::Quadratic) => {
                64 * jd - 64 * (hmax * jd * jd) / (wm * wm)
            }
        };

        let jp = j >> 6; // Integer part
        let jf = j & 0x3f; // Fractional part (0-63)

        if jp < 0 || jp > wm {
            continue;
        }

        match depth {
            PixelDepth::Bit8 => {
                if jp < wm {
                    for i in 0..hi {
                        let v0 = unsafe { pix.get_pixel_unchecked(jp as u32, i as u32) } as i32;
                        let v1 =
                            unsafe { pix.get_pixel_unchecked((jp + 1) as u32, i as u32) } as i32;
                        let val = ((63 - jf) * v0 + jf * v1 + 31) / 63;
                        unsafe { out_mut.set_pixel_unchecked(jd as u32, i as u32, val as u32) };
                    }
                } else {
                    for i in 0..hi {
                        let val = unsafe { pix.get_pixel_unchecked(jp as u32, i as u32) };
                        unsafe { out_mut.set_pixel_unchecked(jd as u32, i as u32, val) };
                    }
                }
            }
            PixelDepth::Bit32 => {
                if jp < wm {
                    for i in 0..hi {
                        let word0 = unsafe { pix.get_pixel_unchecked(jp as u32, i as u32) };
                        let word1 = unsafe { pix.get_pixel_unchecked((jp + 1) as u32, i as u32) };

                        let (r0, g0, b0, a0) = color::extract_rgba(word0);
                        let (r1, g1, b1, a1) = color::extract_rgba(word1);

                        let r = interp_channel(r0, r1, jf);
                        let g = interp_channel(g0, g1, jf);
                        let b = interp_channel(b0, b1, jf);
                        let a = interp_channel(a0, a1, jf);

                        let pixel = color::compose_rgba(r, g, b, a);
                        unsafe { out_mut.set_pixel_unchecked(jd as u32, i as u32, pixel) };
                    }
                } else {
                    for i in 0..hi {
                        let val = unsafe { pix.get_pixel_unchecked(jp as u32, i as u32) };
                        unsafe { out_mut.set_pixel_unchecked(jd as u32, i as u32, val) };
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Quadratic Vertical Shear
// ============================================================================

/// Apply quadratic vertical shear
///
/// This is equivalent to Leptonica's `pixQuadraticVShear`.
///
/// # Arguments
/// * `pix` - Input image (1, 8, or 32 bpp)
/// * `direction` - Which edge is fixed (ToLeft = right fixed, ToRight = left fixed)
/// * `vmax_top` - Maximum vertical displacement at edge at top
/// * `vmax_bottom` - Maximum vertical displacement at edge at bottom
/// * `operation` - Sampled or interpolated
/// * `fill` - Background fill color
///
/// # Returns
/// Sheared image
///
/// # Notes
/// - Positive vmax values cause downward shift at edges
/// - The shear varies quadratically from the center to edges
pub fn quadratic_v_shear(
    pix: &Pix,
    direction: WarpDirection,
    vmax_top: i32,
    vmax_bottom: i32,
    operation: WarpOperation,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let d = pix.depth();

    if d != PixelDepth::Bit1 && d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "quadratic_v_shear requires 1, 8, or 32 bpp".to_string(),
        ));
    }

    // Zero shear returns copy
    if vmax_top == 0 && vmax_bottom == 0 {
        return Ok(pix.deep_clone());
    }

    // For 1bpp, use sampling regardless
    if d == PixelDepth::Bit1 || operation == WarpOperation::Sampled {
        quadratic_v_shear_sampled(pix, direction, vmax_top, vmax_bottom, fill)
    } else {
        quadratic_v_shear_li(pix, direction, vmax_top, vmax_bottom, fill)
    }
}

/// Quadratic vertical shear using nearest-neighbor sampling
///
/// This is equivalent to Leptonica's `pixQuadraticVShearSampled`.
pub fn quadratic_v_shear_sampled(
    pix: &Pix,
    direction: WarpDirection,
    vmax_top: i32,
    vmax_bottom: i32,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth != PixelDepth::Bit1 && depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "quadratic_v_shear_sampled requires 1, 8, or 32 bpp".to_string(),
        ));
    }

    if vmax_top == 0 && vmax_bottom == 0 {
        return Ok(pix.deep_clone());
    }

    let fill_value = fill.to_value(depth);

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;
    let hi = h as i32;
    let wm = wi - 1;
    let hm = hi - 1;
    let denom1 = 1.0 / (hi as f32);
    let denom2 = 1.0 / ((wm * wm) as f32);

    for j in 0..wi {
        let (delrowt, delrowb) = match direction {
            WarpDirection::ToLeft => {
                let t = (vmax_top * (wm - j) * (wm - j)) as f32 * denom2;
                let b = (vmax_bottom * (wm - j) * (wm - j)) as f32 * denom2;
                (t, b)
            }
            WarpDirection::ToRight => {
                let t = (vmax_top * j * j) as f32 * denom2;
                let b = (vmax_bottom * j * j) as f32 * denom2;
                (t, b)
            }
        };

        for id in 0..hi {
            let dely = (delrowt * (hm - id) as f32 + delrowb * id as f32) * denom1;
            let i = id - (dely + 0.5) as i32;

            if i < 0 || i > hm {
                continue;
            }

            let val = unsafe { pix.get_pixel_unchecked(j as u32, i as u32) };
            unsafe { out_mut.set_pixel_unchecked(j as u32, id as u32, val) };
        }
    }

    Ok(out_mut.into())
}

/// Quadratic vertical shear using linear interpolation
///
/// This is equivalent to Leptonica's `pixQuadraticVShearLI`.
pub fn quadratic_v_shear_li(
    pix: &Pix,
    direction: WarpDirection,
    vmax_top: i32,
    vmax_bottom: i32,
    fill: WarpFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "quadratic_v_shear_li requires 8 or 32 bpp".to_string(),
        ));
    }

    if vmax_top == 0 && vmax_bottom == 0 {
        return Ok(pix.deep_clone());
    }

    // Remove colormap if present
    let src_pix = if let Some(cmap) = pix.colormap() {
        remove_colormap(pix, cmap)?
    } else {
        pix.deep_clone()
    };

    let src_depth = src_pix.depth();
    let fill_value = fill.to_value(src_depth);

    let out_pix = Pix::new(w, h, src_depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;
    let hi = h as i32;
    let wm = wi - 1;
    let hm = hi - 1;
    let denom1 = 1.0 / (hi as f32);
    let denom2 = 1.0 / ((wm * wm) as f32);

    for j in 0..wi {
        let (delrowt, delrowb) = match direction {
            WarpDirection::ToLeft => {
                let t = (vmax_top * (wm - j) * (wm - j)) as f32 * denom2;
                let b = (vmax_bottom * (wm - j) * (wm - j)) as f32 * denom2;
                (t, b)
            }
            WarpDirection::ToRight => {
                let t = (vmax_top * j * j) as f32 * denom2;
                let b = (vmax_bottom * j * j) as f32 * denom2;
                (t, b)
            }
        };

        match src_depth {
            PixelDepth::Bit8 => {
                for id in 0..hi {
                    let dely = (delrowt * (hm - id) as f32 + delrowb * id as f32) * denom1;
                    let i = 64 * id - (64.0 * dely) as i32;
                    let yp = i >> 6;
                    let yf = i & 63;

                    if yp < 0 || yp > hm {
                        continue;
                    }

                    let val = if yp < hm {
                        let v0 = unsafe { src_pix.get_pixel_unchecked(j as u32, yp as u32) } as i32;
                        let v1 = unsafe { src_pix.get_pixel_unchecked(j as u32, (yp + 1) as u32) }
                            as i32;
                        ((63 - yf) * v0 + yf * v1 + 31) / 63
                    } else {
                        (unsafe { src_pix.get_pixel_unchecked(j as u32, yp as u32) }) as i32
                    };

                    unsafe { out_mut.set_pixel_unchecked(j as u32, id as u32, val as u32) };
                }
            }
            PixelDepth::Bit32 => {
                for id in 0..hi {
                    let dely = (delrowt * (hm - id) as f32 + delrowb * id as f32) * denom1;
                    let i = 64 * id - (64.0 * dely) as i32;
                    let yp = i >> 6;
                    let yf = i & 63;

                    if yp < 0 || yp > hm {
                        continue;
                    }

                    let pixel = if yp < hm {
                        let word0 = unsafe { src_pix.get_pixel_unchecked(j as u32, yp as u32) };
                        let word1 =
                            unsafe { src_pix.get_pixel_unchecked(j as u32, (yp + 1) as u32) };

                        let (r0, g0, b0, a0) = color::extract_rgba(word0);
                        let (r1, g1, b1, a1) = color::extract_rgba(word1);

                        let r = interp_channel(r0, r1, yf);
                        let g = interp_channel(g0, g1, yf);
                        let b = interp_channel(b0, b1, yf);
                        let a = interp_channel(a0, a1, yf);

                        color::compose_rgba(r, g, b, a)
                    } else {
                        unsafe { src_pix.get_pixel_unchecked(j as u32, yp as u32) }
                    };

                    unsafe { out_mut.set_pixel_unchecked(j as u32, id as u32, pixel) };
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Stereoscopic Warp
// ============================================================================

/// Apply stereoscopic (3D anaglyph) warping
///
/// This is equivalent to Leptonica's `pixWarpStereoscopic`.
///
/// Creates a red-cyan anaglyph by:
/// 1. Optionally applying quadratic vertical shear (in-plane bending)
/// 2. Splitting into RGB channels
/// 3. Horizontally stretching the red channel
/// 4. Applying horizontal shear/translation to the red channel
/// 5. Recombining into a 32bpp output
///
/// # Arguments
/// * `pix` - Input image (any depth, colormap ok)
/// * `params` - Stereoscopic parameters
///
/// # Returns
/// 32bpp RGB image with stereoscopic effect
///
/// # Example
/// ```no_run
/// use leptonica_transform::warper::{warp_stereoscopic, StereoscopicParams};
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let stereo = warp_stereoscopic(&pix, StereoscopicParams::default()).unwrap();
/// ```
pub fn warp_stereoscopic(pix: &Pix, params: StereoscopicParams) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // Convert to 32bpp
    let pix32 = convert_to_32bpp(pix)?;

    // Apply vertical shear if needed
    let pix_sheared = if params.ybend_top != 0 || params.ybend_bottom != 0 {
        apply_quad_v_shear_for_stereo(&pix32, params.ybend_top, params.ybend_bottom)?
    } else {
        pix32
    };

    // Split into RGB channels
    let (mut pixr, pixg, pixb) = split_rgb(&pix_sheared)?;

    // Determine sign based on red_left
    let (zbend, zshift_top, zshift_bottom) = if params.red_left {
        (-params.zbend, -params.zshift_top, -params.zshift_bottom)
    } else {
        (params.zbend, params.zshift_top, params.zshift_bottom)
    };

    // Apply horizontal stretch to red channel (quadratic)
    if zbend != 0 {
        let half_w = w / 2;

        // Process left half
        let left = extract_region(&pixr, 0, 0, half_w, h)?;
        let left_stretched = stretch_horizontal_li(
            &left,
            WarpDirection::ToLeft,
            WarpType::Quadratic,
            zbend,
            WarpFill::White,
        )?;

        // Process right half
        let right = extract_region(&pixr, half_w, 0, w - half_w, h)?;
        let right_stretched = stretch_horizontal_li(
            &right,
            WarpDirection::ToRight,
            WarpType::Quadratic,
            zbend,
            WarpFill::White,
        )?;

        // Combine halves
        pixr = combine_halves(&left_stretched, &right_stretched, w, h)?;
    }

    // Apply horizontal shear/translation
    if zshift_top != 0 || zshift_bottom != 0 {
        if zshift_top == zshift_bottom {
            // Pure translation
            pixr = translate_horizontal(&pixr, zshift_top)?;
        } else {
            // Shear + translation
            let angle = (zshift_bottom - zshift_top) as f32 / h.max(1) as f32;
            let zshift = (zshift_top + zshift_bottom) / 2;
            pixr = translate_horizontal(&pixr, zshift)?;
            pixr = h_shear_li(&pixr, (h / 2) as i32, angle, WarpFill::White)?;
        }
    }

    // Recombine into RGB
    combine_rgb(&pixr, &pixg, &pixb)
}

/// Create stereo anaglyph from a pair of images
///
/// This is equivalent to Leptonica's `pixStereoFromPair`.
///
/// # Arguments
/// * `pix1` - Left eye image (32bpp RGB)
/// * `pix2` - Right eye image (32bpp RGB)
/// * `rwt` - Red channel weight from pix1 (typically 0.0)
/// * `gwt` - Green channel weight from pix1 for red output (typically 0.7)
/// * `bwt` - Blue channel weight from pix1 for red output (typically 0.3)
///
/// # Returns
/// 32bpp stereo anaglyph image
///
/// # Notes
/// - Output red channel = rwt*R1 + gwt*G1 + bwt*B1
/// - Output green channel = G2
/// - Output blue channel = B2
/// - Weights should sum to 1.0 (will be normalized if not)
pub fn stereo_from_pair(
    pix1: &Pix,
    pix2: &Pix,
    rwt: f32,
    gwt: f32,
    bwt: f32,
) -> TransformResult<Pix> {
    if pix1.depth() != PixelDepth::Bit32 || pix2.depth() != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "stereo_from_pair requires 32bpp images".to_string(),
        ));
    }

    let w = pix1.width();
    let h = pix1.height();

    // Normalize weights
    let (rwt, gwt, bwt) = if rwt == 0.0 && gwt == 0.0 && bwt == 0.0 {
        (
            DEFAULT_RED_WEIGHT,
            DEFAULT_GREEN_WEIGHT,
            DEFAULT_BLUE_WEIGHT,
        )
    } else {
        let sum = rwt + gwt + bwt;
        if (sum - 1.0).abs() > 0.0001 {
            (rwt / sum, gwt / sum, bwt / sum)
        } else {
            (rwt, gwt, bwt)
        }
    };

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for j in 0..h {
        for i in 0..w {
            let word1 = unsafe { pix1.get_pixel_unchecked(i, j) };
            let word2 = unsafe { pix2.get_pixel_unchecked(i, j) };

            let (r1, g1, b1, _) = color::extract_rgba(word1);
            let (_, g2, b2, _) = color::extract_rgba(word2);

            // Compute weighted red from pix1
            let rval = (rwt * r1 as f32 + gwt * g1 as f32 + bwt * b1 as f32 + 0.5) as u8;

            let pixel = color::compose_rgba(rval, g2, b2, 255);
            unsafe { out_mut.set_pixel_unchecked(i, j, pixel) };
        }
    }

    Ok(out_mut.into())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Fill an image with a constant value
fn fill_image(pix: &mut PixMut, value: u32) {
    let w = pix.width();
    let h = pix.height();
    for y in 0..h {
        for x in 0..w {
            unsafe { pix.set_pixel_unchecked(x, y, value) };
        }
    }
}

/// Linear interpolation helper for a single channel
#[inline]
fn interp_channel(v0: u8, v1: u8, f: i32) -> u8 {
    (((63 - f) * v0 as i32 + f * v1 as i32 + 31) / 63).clamp(0, 255) as u8
}

/// Remove colormap from an image
fn remove_colormap(pix: &Pix, cmap: &leptonica_core::PixColormap) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    // Determine if colormap is grayscale or color
    let is_gray = cmap
        .colors()
        .iter()
        .all(|c| c.red == c.green && c.green == c.blue);

    if is_gray {
        let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut out_mut = out_pix.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let idx = unsafe { pix.get_pixel_unchecked(x, y) } as usize;
                let gray = if idx < cmap.len() {
                    cmap.colors()[idx].red
                } else {
                    0
                };
                unsafe { out_mut.set_pixel_unchecked(x, y, gray as u32) };
            }
        }

        Ok(out_mut.into())
    } else {
        let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut out_mut = out_pix.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let idx = unsafe { pix.get_pixel_unchecked(x, y) } as usize;
                let pixel = if idx < cmap.len() {
                    let c = &cmap.colors()[idx];
                    color::compose_rgba(c.red, c.green, c.blue, 255)
                } else {
                    0
                };
                unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }

        Ok(out_mut.into())
    }
}

/// Convert any image to 32bpp
fn convert_to_32bpp(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    if depth == PixelDepth::Bit32 {
        return Ok(pix.deep_clone());
    }

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        // Use colormap
        for y in 0..h {
            for x in 0..w {
                let idx = unsafe { pix.get_pixel_unchecked(x, y) } as usize;
                let pixel = if idx < cmap.len() {
                    let c = &cmap.colors()[idx];
                    color::compose_rgba(c.red, c.green, c.blue, 255)
                } else {
                    0
                };
                unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }
    } else {
        // Grayscale to RGB
        for y in 0..h {
            for x in 0..w {
                let val = unsafe { pix.get_pixel_unchecked(x, y) };
                let gray = match depth {
                    PixelDepth::Bit1 => {
                        if val == 0 {
                            255u8
                        } else {
                            0u8
                        }
                    }
                    PixelDepth::Bit2 => (val * 85) as u8,
                    PixelDepth::Bit4 => (val * 17) as u8,
                    PixelDepth::Bit8 => val as u8,
                    PixelDepth::Bit16 => (val >> 8) as u8,
                    PixelDepth::Bit32 => unreachable!(),
                };
                let pixel = color::compose_rgba(gray, gray, gray, 255);
                unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Split 32bpp image into R, G, B channels (each 8bpp)
fn split_rgb(pix: &Pix) -> TransformResult<(Pix, Pix, Pix)> {
    let w = pix.width();
    let h = pix.height();

    let pix_r = Pix::new(w, h, PixelDepth::Bit8)?;
    let pix_g = Pix::new(w, h, PixelDepth::Bit8)?;
    let pix_b = Pix::new(w, h, PixelDepth::Bit8)?;

    let mut r_mut = pix_r.try_into_mut().unwrap();
    let mut g_mut = pix_g.try_into_mut().unwrap();
    let mut b_mut = pix_b.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            let (r, g, b, _) = color::extract_rgba(pixel);
            unsafe {
                r_mut.set_pixel_unchecked(x, y, r as u32);
                g_mut.set_pixel_unchecked(x, y, g as u32);
                b_mut.set_pixel_unchecked(x, y, b as u32);
            }
        }
    }

    Ok((r_mut.into(), g_mut.into(), b_mut.into()))
}

/// Combine R, G, B channels into 32bpp image
fn combine_rgb(pix_r: &Pix, pix_g: &Pix, pix_b: &Pix) -> TransformResult<Pix> {
    let w = pix_r.width();
    let h = pix_r.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit32)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let r = unsafe { pix_r.get_pixel_unchecked(x, y) } as u8;
            let g = unsafe { pix_g.get_pixel_unchecked(x, y) } as u8;
            let b = unsafe { pix_b.get_pixel_unchecked(x, y) } as u8;
            let pixel = color::compose_rgba(r, g, b, 255);
            unsafe { out_mut.set_pixel_unchecked(x, y, pixel) };
        }
    }

    Ok(out_mut.into())
}

/// Extract a rectangular region from an image
fn extract_region(pix: &Pix, x: u32, y: u32, w: u32, h: u32) -> TransformResult<Pix> {
    let out_pix = Pix::new(w, h, pix.depth())?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for dy in 0..h {
        for dx in 0..w {
            let val = unsafe { pix.get_pixel_unchecked(x + dx, y + dy) };
            unsafe { out_mut.set_pixel_unchecked(dx, dy, val) };
        }
    }

    Ok(out_mut.into())
}

/// Combine left and right halves into a single image
fn combine_halves(left: &Pix, right: &Pix, w: u32, h: u32) -> TransformResult<Pix> {
    let out_pix = Pix::new(w, h, left.depth())?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let half_w = w / 2;

    // Copy left half
    for y in 0..h {
        for x in 0..half_w {
            let val = unsafe { left.get_pixel_unchecked(x, y) };
            unsafe { out_mut.set_pixel_unchecked(x, y, val) };
        }
    }

    // Copy right half
    for y in 0..h {
        for x in 0..(w - half_w) {
            let val = unsafe { right.get_pixel_unchecked(x, y) };
            unsafe { out_mut.set_pixel_unchecked(half_w + x, y, val) };
        }
    }

    Ok(out_mut.into())
}

/// Translate image horizontally
fn translate_horizontal(pix: &Pix, dx: i32) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    let fill_value = WarpFill::White.to_value(depth);
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;

    for y in 0..h {
        for x in 0..wi {
            let src_x = x - dx;
            if src_x >= 0 && src_x < wi {
                let val = unsafe { pix.get_pixel_unchecked(src_x as u32, y) };
                unsafe { out_mut.set_pixel_unchecked(x as u32, y, val) };
            }
        }
    }

    Ok(out_mut.into())
}

/// Apply quadratic vertical shear for stereoscopic effect
fn apply_quad_v_shear_for_stereo(
    pix: &Pix,
    ybend_top: i32,
    ybend_bottom: i32,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let half_w = w / 2;

    // Process left half
    let left = extract_region(pix, 0, 0, half_w, h)?;
    let left_sheared = quadratic_v_shear(
        &left,
        WarpDirection::ToLeft,
        ybend_top,
        ybend_bottom,
        WarpOperation::Interpolated,
        WarpFill::White,
    )?;

    // Process right half
    let right = extract_region(pix, half_w, 0, w - half_w, h)?;
    let right_sheared = quadratic_v_shear(
        &right,
        WarpDirection::ToRight,
        ybend_top,
        ybend_bottom,
        WarpOperation::Interpolated,
        WarpFill::White,
    )?;

    // Combine halves
    combine_halves(&left_sheared, &right_sheared, w, h)
}

/// Horizontal shear with linear interpolation (for stereo)
fn h_shear_li(pix: &Pix, yloc: i32, angle: f32, fill: WarpFill) -> TransformResult<Pix> {
    use crate::shear::ShearFill;
    use crate::shear::h_shear_li as shear_h_shear_li;

    let shear_fill = match fill {
        WarpFill::White => ShearFill::White,
        WarpFill::Black => ShearFill::Black,
    };

    shear_h_shear_li(pix, yloc, angle, shear_fill)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // WarpFill tests
    // ========================================================================

    #[test]
    fn test_warp_fill_values() {
        assert_eq!(WarpFill::White.to_value(PixelDepth::Bit1), 0);
        assert_eq!(WarpFill::Black.to_value(PixelDepth::Bit1), 1);
        assert_eq!(WarpFill::White.to_value(PixelDepth::Bit8), 255);
        assert_eq!(WarpFill::Black.to_value(PixelDepth::Bit8), 0);
        assert_eq!(WarpFill::White.to_value(PixelDepth::Bit32), 0xFFFFFF00);
        assert_eq!(WarpFill::Black.to_value(PixelDepth::Bit32), 0);
    }

    // ========================================================================
    // SimpleRng tests
    // ========================================================================

    #[test]
    fn test_simple_rng_reproducible() {
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(42);

        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_simple_rng_different_seeds() {
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(43);

        // Should produce different values
        assert_ne!(rng1.next(), rng2.next());
    }

    // ========================================================================
    // Random harmonic warp tests
    // ========================================================================

    #[test]
    fn test_random_harmonic_warp_basic() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let result = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 3, 3, 42, 255);
        assert!(result.is_ok());
        let warped = result.unwrap();
        assert_eq!(warped.width(), 50);
        assert_eq!(warped.height(), 50);
        assert_eq!(warped.depth(), PixelDepth::Bit8);
    }

    #[test]
    fn test_random_harmonic_warp_reproducible() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();

        let warped1 = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 2, 2, 123, 255).unwrap();
        let warped2 = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 2, 2, 123, 255).unwrap();

        // Same seed should produce same result
        for y in 0..20 {
            for x in 0..20 {
                assert_eq!(unsafe { warped1.get_pixel_unchecked(x, y) }, unsafe {
                    warped2.get_pixel_unchecked(x, y)
                });
            }
        }
    }

    #[test]
    fn test_random_harmonic_warp_different_seeds() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Create some content
        for y in 0..20 {
            for x in 0..20 {
                unsafe { pix_mut.set_pixel_unchecked(x, y, (x + y) * 10 % 256) };
            }
        }
        let pix: Pix = pix_mut.into();

        let warped1 = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 2, 2, 123, 255).unwrap();
        let warped2 = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 2, 2, 456, 255).unwrap();

        // Different seeds should produce different results
        let mut all_same = true;
        for y in 0..20 {
            for x in 0..20 {
                if unsafe { warped1.get_pixel_unchecked(x, y) }
                    != unsafe { warped2.get_pixel_unchecked(x, y) }
                {
                    all_same = false;
                    break;
                }
            }
        }
        assert!(!all_same);
    }

    #[test]
    fn test_random_harmonic_warp_unsupported_depth() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let result = random_harmonic_warp(&pix, 4.0, 6.0, 0.1, 0.13, 2, 2, 42, 255);
        assert!(matches!(result, Err(TransformError::UnsupportedDepth(_))));
    }

    // ========================================================================
    // Horizontal stretch tests
    // ========================================================================

    #[test]
    fn test_stretch_horizontal_sampled_zero() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(10, 10, 100) };
        let pix: Pix = pix_mut.into();

        let result = stretch_horizontal_sampled(
            &pix,
            WarpDirection::ToLeft,
            WarpType::Linear,
            0,
            WarpFill::White,
        )
        .unwrap();

        // Zero stretch should preserve pixels
        assert_eq!(unsafe { result.get_pixel_unchecked(10, 10) }, 100);
    }

    #[test]
    fn test_stretch_horizontal_linear() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let result = stretch_horizontal(
            &pix,
            WarpDirection::ToLeft,
            WarpType::Linear,
            5,
            WarpOperation::Sampled,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_stretch_horizontal_quadratic() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let result = stretch_horizontal(
            &pix,
            WarpDirection::ToRight,
            WarpType::Quadratic,
            5,
            WarpOperation::Sampled,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_stretch_horizontal_li_8bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let result = stretch_horizontal_li(
            &pix,
            WarpDirection::ToLeft,
            WarpType::Quadratic,
            5,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_stretch_horizontal_li_32bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let result = stretch_horizontal_li(
            &pix,
            WarpDirection::ToRight,
            WarpType::Linear,
            5,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_stretch_horizontal_1bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap();
        let result = stretch_horizontal(
            &pix,
            WarpDirection::ToLeft,
            WarpType::Linear,
            5,
            WarpOperation::Sampled,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    // ========================================================================
    // Quadratic vertical shear tests
    // ========================================================================

    #[test]
    fn test_quadratic_v_shear_zero() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(10, 10, 100) };
        let pix: Pix = pix_mut.into();

        let result = quadratic_v_shear(
            &pix,
            WarpDirection::ToLeft,
            0,
            0,
            WarpOperation::Sampled,
            WarpFill::White,
        )
        .unwrap();

        // Zero shear should preserve pixels
        assert_eq!(unsafe { result.get_pixel_unchecked(10, 10) }, 100);
    }

    #[test]
    fn test_quadratic_v_shear_sampled() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let result = quadratic_v_shear_sampled(&pix, WarpDirection::ToLeft, 5, -5, WarpFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quadratic_v_shear_li_8bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let result = quadratic_v_shear_li(&pix, WarpDirection::ToRight, 10, -10, WarpFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quadratic_v_shear_li_32bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let result = quadratic_v_shear_li(&pix, WarpDirection::ToLeft, 5, 5, WarpFill::Black);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quadratic_v_shear_1bpp() {
        let pix = Pix::new(30, 30, PixelDepth::Bit1).unwrap();
        let result = quadratic_v_shear(
            &pix,
            WarpDirection::ToRight,
            5,
            -5,
            WarpOperation::Sampled,
            WarpFill::White,
        );
        assert!(result.is_ok());
    }

    // ========================================================================
    // Stereoscopic tests
    // ========================================================================

    #[test]
    fn test_warp_stereoscopic_basic() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let result = warp_stereoscopic(&pix, StereoscopicParams::default());
        assert!(result.is_ok());
        let stereo = result.unwrap();
        assert_eq!(stereo.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_warp_stereoscopic_32bpp_input() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let result = warp_stereoscopic(&pix, StereoscopicParams::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_warp_stereoscopic_no_bend() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let params = StereoscopicParams {
            zbend: 0,
            zshift_top: 0,
            zshift_bottom: 0,
            ybend_top: 0,
            ybend_bottom: 0,
            red_left: true,
        };
        let result = warp_stereoscopic(&pix, params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stereo_from_pair_basic() {
        let pix1 = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let pix2 = Pix::new(30, 30, PixelDepth::Bit32).unwrap();

        let result = stereo_from_pair(&pix1, &pix2, 0.0, 0.7, 0.3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_stereo_from_pair_default_weights() {
        let pix1 = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let pix2 = Pix::new(30, 30, PixelDepth::Bit32).unwrap();

        // Zero weights should use defaults
        let result = stereo_from_pair(&pix1, &pix2, 0.0, 0.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stereo_from_pair_wrong_depth() {
        let pix1 = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(30, 30, PixelDepth::Bit32).unwrap();

        let result = stereo_from_pair(&pix1, &pix2, 0.0, 0.7, 0.3);
        assert!(matches!(result, Err(TransformError::UnsupportedDepth(_))));
    }

    // ========================================================================
    // Helper function tests
    // ========================================================================

    #[test]
    fn test_convert_to_32bpp_from_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(5, 5, 128) };
        let pix: Pix = pix_mut.into();

        let result = convert_to_32bpp(&pix).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit32);

        let pixel = unsafe { result.get_pixel_unchecked(5, 5) };
        let (r, g, b, _) = color::extract_rgba(pixel);
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }

    #[test]
    fn test_split_and_combine_rgb() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        let red_pixel = color::compose_rgba(255, 0, 0, 255);
        unsafe { pix_mut.set_pixel_unchecked(5, 5, red_pixel) };
        let pix: Pix = pix_mut.into();

        let (r, g, b) = split_rgb(&pix).unwrap();
        assert_eq!(unsafe { r.get_pixel_unchecked(5, 5) }, 255);
        assert_eq!(unsafe { g.get_pixel_unchecked(5, 5) }, 0);
        assert_eq!(unsafe { b.get_pixel_unchecked(5, 5) }, 0);

        let combined = combine_rgb(&r, &g, &b).unwrap();
        let pixel = unsafe { combined.get_pixel_unchecked(5, 5) };
        let (rc, gc, bc, _) = color::extract_rgba(pixel);
        assert_eq!(rc, 255);
        assert_eq!(gc, 0);
        assert_eq!(bc, 0);
    }

    #[test]
    fn test_translate_horizontal() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        unsafe { pix_mut.set_pixel_unchecked(5, 5, 100) };
        let pix: Pix = pix_mut.into();

        let translated = translate_horizontal(&pix, 3).unwrap();
        // Pixel should have moved from (5,5) to (8,5)
        assert_eq!(unsafe { translated.get_pixel_unchecked(8, 5) }, 100);
        // Position (0,5) should be fill value since src_x = 0 - 3 = -3 is out of bounds
        assert_eq!(unsafe { translated.get_pixel_unchecked(0, 5) }, 255); // White fill
    }

    // ========================================================================
    // StereoscopicParams tests
    // ========================================================================

    #[test]
    fn test_stereoscopic_params_default() {
        let params = StereoscopicParams::default();
        assert_eq!(params.zbend, 20);
        assert_eq!(params.zshift_top, 15);
        assert_eq!(params.zshift_bottom, -15);
        assert_eq!(params.ybend_top, 30);
        assert_eq!(params.ybend_bottom, 0);
        assert!(params.red_left);
    }
}
