//! Projective transformations for images
//!
//! This module provides projective (homography) transformation operations including:
//! - Projective transformation coefficient calculation (from 4 point correspondences)
//! - Sampled projective transformation (nearest-neighbor, like pixProjectiveSampled)
//! - Interpolated projective transformation (like pixProjective)
//! - Coordinate transformation
//!
//! # Projective Transform
//!
//! A projective transform maps 4 points to 4 other points using the equations:
//! ```text
//! x' = (a*x + b*y + c) / (g*x + h*y + 1)
//! y' = (d*x + e*y + f) / (g*x + h*y + 1)
//! ```
//!
//! The 8 coefficients are computed by solving a system of 8 linear equations
//! from the 4 point correspondences.
//!
//! Unlike affine transforms (which preserve parallel lines) and bilinear transforms
//! (which can introduce curvature), projective transforms preserve straight lines
//! but not necessarily parallel lines. They are useful for correcting keystoning
//! and other perspective distortions.
//!
//! # Example
//!
//! ```no_run
//! use leptonica_transform::projective::{ProjectiveCoeffs, projective_sampled};
//! use leptonica_transform::affine::{AffineFill, Point};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
//!
//! // Define 4 source points and 4 destination points
//! let src_pts = [
//!     Point::new(0.0, 0.0),
//!     Point::new(99.0, 0.0),
//!     Point::new(0.0, 99.0),
//!     Point::new(99.0, 99.0),
//! ];
//! let dst_pts = [
//!     Point::new(10.0, 10.0),
//!     Point::new(89.0, 10.0),
//!     Point::new(10.0, 89.0),
//!     Point::new(89.0, 89.0),
//! ];
//!
//! // Compute the transformation coefficients (dst -> src for backward mapping)
//! let coeffs = ProjectiveCoeffs::from_four_points(dst_pts, src_pts).unwrap();
//!
//! // Apply the transformation
//! let transformed = projective_sampled(&pix, &coeffs, AffineFill::White).unwrap();
//! ```

use crate::affine::{AffineFill, Point};
use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixelDepth, color};

// ============================================================================
// Projective Coefficients
// ============================================================================

/// Projective transformation coefficients
///
/// Stores 8 coefficients [a, b, c, d, e, f, g, h] representing:
/// ```text
/// x' = (a*x + b*y + c) / (g*x + h*y + 1)
/// y' = (d*x + e*y + f) / (g*x + h*y + 1)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectiveCoeffs {
    /// Coefficients [a, b, c, d, e, f, g, h]
    coeffs: [f32; 8],
}

impl Default for ProjectiveCoeffs {
    fn default() -> Self {
        Self::identity()
    }
}

impl ProjectiveCoeffs {
    /// Create an identity transform (no transformation)
    ///
    /// This corresponds to:
    /// - x' = x (a=1, b=0, c=0)
    /// - y' = y (d=0, e=1, f=0)
    /// - denominator = 1 (g=0, h=0)
    pub fn identity() -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Create from raw coefficients [a, b, c, d, e, f, g, h]
    pub fn from_coeffs(coeffs: [f32; 8]) -> Self {
        Self { coeffs }
    }

    /// Get the coefficients [a, b, c, d, e, f, g, h]
    pub fn coeffs(&self) -> &[f32; 8] {
        &self.coeffs
    }

    /// Create projective coefficients from 4 point correspondences
    ///
    /// Given 4 source points and 4 destination points, computes the projective
    /// transformation that maps src_pts to dst_pts.
    ///
    /// No 3 of the 4 points should be collinear.
    ///
    /// # Arguments
    /// * `src_pts` - Source points [p1, p2, p3, p4]
    /// * `dst_pts` - Destination points [p1', p2', p3', p4']
    ///
    /// # Returns
    /// The projective coefficients that transform src_pts to dst_pts
    ///
    /// # Errors
    /// Returns `TransformError::SingularMatrix` if the system is not solvable
    ///
    /// # Note
    /// For image transformation, you typically want the inverse mapping (dst -> src).
    /// Call `from_four_points(dst_pts, src_pts)` to get coefficients for backward mapping.
    pub fn from_four_points(src_pts: [Point; 4], dst_pts: [Point; 4]) -> TransformResult<Self> {
        // We solve the system of 8 equations:
        // For each point i:
        //   x'_i = (a*x_i + b*y_i + c) / (g*x_i + h*y_i + 1)
        //   y'_i = (d*x_i + e*y_i + f) / (g*x_i + h*y_i + 1)
        //
        // Rearranging to linear form:
        //   x'_i * (g*x_i + h*y_i + 1) = a*x_i + b*y_i + c
        //   y'_i * (g*x_i + h*y_i + 1) = d*x_i + e*y_i + f
        //
        // Which gives:
        //   a*x_i + b*y_i + c - g*x_i*x'_i - h*y_i*x'_i = x'_i
        //   d*x_i + e*y_i + f - g*x_i*y'_i - h*y_i*y'_i = y'_i

        let x1 = src_pts[0].x;
        let y1 = src_pts[0].y;
        let x2 = src_pts[1].x;
        let y2 = src_pts[1].y;
        let x3 = src_pts[2].x;
        let y3 = src_pts[2].y;
        let x4 = src_pts[3].x;
        let y4 = src_pts[3].y;

        let x1p = dst_pts[0].x;
        let y1p = dst_pts[0].y;
        let x2p = dst_pts[1].x;
        let y2p = dst_pts[1].y;
        let x3p = dst_pts[2].x;
        let y3p = dst_pts[2].y;
        let x4p = dst_pts[3].x;
        let y4p = dst_pts[3].y;

        // Build 8x8 matrix A
        // Row format for x' equation: [x, y, 1, 0, 0, 0, -x*x', -y*x']
        // Row format for y' equation: [0, 0, 0, x, y, 1, -x*y', -y*y']
        let mut a = [
            [x1, y1, 1.0, 0.0, 0.0, 0.0, -x1 * x1p, -y1 * x1p],
            [0.0, 0.0, 0.0, x1, y1, 1.0, -x1 * y1p, -y1 * y1p],
            [x2, y2, 1.0, 0.0, 0.0, 0.0, -x2 * x2p, -y2 * x2p],
            [0.0, 0.0, 0.0, x2, y2, 1.0, -x2 * y2p, -y2 * y2p],
            [x3, y3, 1.0, 0.0, 0.0, 0.0, -x3 * x3p, -y3 * x3p],
            [0.0, 0.0, 0.0, x3, y3, 1.0, -x3 * y3p, -y3 * y3p],
            [x4, y4, 1.0, 0.0, 0.0, 0.0, -x4 * x4p, -y4 * x4p],
            [0.0, 0.0, 0.0, x4, y4, 1.0, -x4 * y4p, -y4 * y4p],
        ];

        // Right-hand side vector
        let mut b = [x1p, y1p, x2p, y2p, x3p, y3p, x4p, y4p];

        // Solve using Gauss-Jordan elimination
        gauss_jordan_8x8(&mut a, &mut b)?;

        // b now contains the solution [a, b, c, d, e, f, g, h]
        Ok(Self { coeffs: b })
    }

    /// Transform a point using this projective transform (sampled, integer result)
    ///
    /// Returns the nearest integer coordinates after transformation.
    /// Returns None if the denominator is zero (point at infinity).
    pub fn transform_point_sampled(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let xf = x as f32;
        let yf = y as f32;
        let [a, b, c, d, e, f, g, h] = self.coeffs;

        let denom = g * xf + h * yf + 1.0;
        if denom.abs() < 1e-10 {
            return None;
        }

        let factor = 1.0 / denom;
        let xp = (factor * (a * xf + b * yf + c) + 0.5).floor() as i32;
        let yp = (factor * (d * xf + e * yf + f) + 0.5).floor() as i32;
        Some((xp, yp))
    }

    /// Transform a point returning floating point coordinates
    ///
    /// Returns None if the denominator is zero (point at infinity).
    pub fn transform_point_float(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let [a, b, c, d, e, f, g, h] = self.coeffs;

        let denom = g * x + h * y + 1.0;
        if denom.abs() < 1e-10 {
            return None;
        }

        let factor = 1.0 / denom;
        let xp = factor * (a * x + b * y + c);
        let yp = factor * (d * x + e * y + f);
        Some((xp, yp))
    }

    /// Transform a Point struct
    ///
    /// Returns None if the denominator is zero (point at infinity).
    pub fn transform_point(&self, pt: Point) -> Option<Point> {
        self.transform_point_float(pt.x, pt.y)
            .map(|(x, y)| Point::new(x, y))
    }
}

// ============================================================================
// Gauss-Jordan Elimination for 8x8 System
// ============================================================================

/// Solve a system of 8 linear equations using Gauss-Jordan elimination
///
/// Solves Ax = b in-place. After completion:
/// - a is transformed to the identity matrix
/// - b contains the solution x
fn gauss_jordan_8x8(a: &mut [[f32; 8]; 8], b: &mut [f32; 8]) -> TransformResult<()> {
    const N: usize = 8;
    let mut index_c = [0usize; N];
    let mut index_r = [0usize; N];
    let mut ipiv = [0i32; N];

    for i in 0..N {
        let mut max_val = 0.0f32;
        let mut irow = 0;
        let mut icol = 0;

        // Find pivot
        for j in 0..N {
            if ipiv[j] != 1 {
                for k in 0..N {
                    if ipiv[k] == 0 {
                        let abs_val = a[j][k].abs();
                        if abs_val >= max_val {
                            max_val = abs_val;
                            irow = j;
                            icol = k;
                        }
                    } else if ipiv[k] > 1 {
                        return Err(TransformError::SingularMatrix);
                    }
                }
            }
        }
        ipiv[icol] += 1;

        // Swap rows if needed
        if irow != icol {
            a.swap(irow, icol);
            b.swap(irow, icol);
        }

        index_r[i] = irow;
        index_c[i] = icol;

        if a[icol][icol] == 0.0 {
            return Err(TransformError::SingularMatrix);
        }

        let pivinv = 1.0 / a[icol][icol];
        a[icol][icol] = 1.0;
        for item in a[icol].iter_mut().take(N) {
            *item *= pivinv;
        }
        b[icol] *= pivinv;

        // Reduce other rows
        for row in 0..N {
            if row != icol {
                let val = a[row][icol];
                a[row][icol] = 0.0;
                #[allow(clippy::needless_range_loop)]
                for col in 0..N {
                    a[row][col] -= a[icol][col] * val;
                }
                b[row] -= b[icol] * val;
            }
        }
    }

    // Unscramble columns
    for col in (0..N).rev() {
        if index_r[col] != index_c[col] {
            for row_arr in a.iter_mut().take(N) {
                row_arr.swap(index_r[col], index_c[col]);
            }
        }
    }

    Ok(())
}

// ============================================================================
// Sampled Projective Transformation
// ============================================================================

/// Apply a projective transformation using nearest-neighbor sampling
///
/// This is equivalent to Leptonica's `pixProjectiveSampled`.
/// Works with all pixel depths. Fastest but lowest quality.
///
/// # Arguments
/// * `pix` - Input image
/// * `coeffs` - Projective transformation coefficients (inverse: dest -> src)
/// * `fill` - Background fill color
///
/// # Note
/// The coefficients should map destination pixels to source pixels.
/// Use `from_four_points(dst_pts, src_pts)` to get the correct coefficients.
pub fn projective_sampled(
    pix: &Pix,
    coeffs: &ProjectiveCoeffs,
    fill: AffineFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    // Create output image with same dimensions
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

    // For each destination pixel, find the source pixel
    for j in 0..h {
        for i in 0..w {
            if let Some((sx, sy)) = coeffs.transform_point_sampled(i as i32, j as i32)
                && sx >= 0
                && sx < wi
                && sy >= 0
                && sy < hi
            {
                let val = pix.get_pixel_unchecked(sx as u32, sy as u32);
                out_mut.set_pixel_unchecked(i, j, val);
            }
            // Pixels outside source or at infinity keep the fill value
        }
    }

    Ok(out_mut.into())
}

/// Apply a projective transformation using 4 point correspondences (sampled)
///
/// This is equivalent to Leptonica's `pixProjectiveSampledPta`.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 points in source coordinate space
/// * `dst_pts` - 4 corresponding points in destination coordinate space
/// * `fill` - Background fill color
pub fn projective_sampled_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    // Compute the transform from dst -> src (inverse)
    // This maps destination points to source points
    let inv_coeffs = ProjectiveCoeffs::from_four_points(dst_pts, src_pts)?;
    projective_sampled(pix, &inv_coeffs, fill)
}

// ============================================================================
// Interpolated Projective Transformation
// ============================================================================

/// Apply a projective transformation with bilinear interpolation
///
/// This is equivalent to Leptonica's `pixProjective`.
/// Works best with 8bpp grayscale and 32bpp color images.
/// Falls back to sampling for 1bpp and other depths.
///
/// # Arguments
/// * `pix` - Input image
/// * `coeffs` - Projective transformation coefficients (inverse: dest -> src)
/// * `fill` - Background fill color
pub fn projective(pix: &Pix, coeffs: &ProjectiveCoeffs, fill: AffineFill) -> TransformResult<Pix> {
    let depth = pix.depth();

    // For 1bpp, use sampling (interpolation doesn't make sense)
    if depth == PixelDepth::Bit1 {
        return projective_sampled(pix, coeffs, fill);
    }

    match depth {
        PixelDepth::Bit8 if pix.colormap().is_none() => projective_gray(pix, coeffs, fill),
        PixelDepth::Bit32 => projective_color(pix, coeffs, fill),
        _ => {
            // For other depths (2bpp, 4bpp, 8bpp with colormap, 16bpp),
            // fall back to sampling
            projective_sampled(pix, coeffs, fill)
        }
    }
}

/// Apply a projective transformation using 4 point correspondences (interpolated)
///
/// This is equivalent to Leptonica's `pixProjectivePta`.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 4 points in source coordinate space
/// * `dst_pts` - 4 corresponding points in destination coordinate space
/// * `fill` - Background fill color
pub fn projective_pta(
    pix: &Pix,
    src_pts: [Point; 4],
    dst_pts: [Point; 4],
    fill: AffineFill,
) -> TransformResult<Pix> {
    let depth = pix.depth();

    // For 1bpp, use sampling
    if depth == PixelDepth::Bit1 {
        return projective_sampled_pta(pix, src_pts, dst_pts, fill);
    }

    // Compute the transform from dst -> src (inverse)
    let inv_coeffs = ProjectiveCoeffs::from_four_points(dst_pts, src_pts)?;

    match depth {
        PixelDepth::Bit8 if pix.colormap().is_none() => projective_gray(pix, &inv_coeffs, fill),
        PixelDepth::Bit32 => projective_color(pix, &inv_coeffs, fill),
        _ => projective_sampled(pix, &inv_coeffs, fill),
    }
}

/// Projective transform for 8bpp grayscale with bilinear interpolation
fn projective_gray(pix: &Pix, coeffs: &ProjectiveCoeffs, fill: AffineFill) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth) as u8;

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Fill with background
    fill_image(&mut out_mut, fill_value as u32);

    let wi = w as i32;
    let hi = h as i32;
    let wm2 = wi - 2;
    let hm2 = hi - 2;

    let [a, b, c, d, e, f, g, h_coeff] = *coeffs.coeffs();

    for j in 0..h {
        let jf = j as f32;
        for i in 0..w {
            let if_ = i as f32;

            // Compute denominator
            let denom = g * if_ + h_coeff * jf + 1.0;
            if denom.abs() < 1e-10 {
                continue; // Point at infinity, keep fill
            }

            let factor = 1.0 / denom;

            // Compute sub-pixel position (scaled by 16)
            let xp_float = factor * (a * if_ + b * jf + c);
            let yp_float = factor * (d * if_ + e * jf + f);

            let xpm = (16.0 * xp_float) as i32;
            let ypm = (16.0 * yp_float) as i32;

            // Integer and fractional parts
            let xp = xpm >> 4;
            let yp = ypm >> 4;
            let xf = xpm & 0x0f;
            let yf = ypm & 0x0f;

            // Bounds check
            if xp < 0 || yp < 0 || xp > wm2 || yp > hm2 {
                // Keep fill value (already set)
                continue;
            }

            // Get four neighboring pixels
            let v00 = pix.get_pixel_unchecked(xp as u32, yp as u32) as i32;
            let v10 = pix.get_pixel_unchecked((xp + 1) as u32, yp as u32) as i32;
            let v01 = pix.get_pixel_unchecked(xp as u32, (yp + 1) as u32) as i32;
            let v11 = pix.get_pixel_unchecked((xp + 1) as u32, (yp + 1) as u32) as i32;

            // Area-weighted interpolation
            let val = ((16 - xf) * (16 - yf) * v00
                + xf * (16 - yf) * v10
                + (16 - xf) * yf * v01
                + xf * yf * v11
                + 128)
                / 256;

            out_mut.set_pixel_unchecked(i, j, val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Projective transform for 32bpp color with bilinear interpolation
fn projective_color(
    pix: &Pix,
    coeffs: &ProjectiveCoeffs,
    fill: AffineFill,
) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();
    let fill_value = fill.to_value(depth);

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();
    out_mut.set_spp(pix.spp());

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    let wi = w as i32;
    let hi = h as i32;
    let wm2 = wi - 2;
    let hm2 = hi - 2;

    let [a, b, c, d, e, f, g, h_coeff] = *coeffs.coeffs();

    for j in 0..h {
        let jf = j as f32;
        for i in 0..w {
            let if_ = i as f32;

            // Compute denominator
            let denom = g * if_ + h_coeff * jf + 1.0;
            if denom.abs() < 1e-10 {
                continue; // Point at infinity, keep fill
            }

            let factor = 1.0 / denom;

            // Compute sub-pixel position (scaled by 16)
            let xp_float = factor * (a * if_ + b * jf + c);
            let yp_float = factor * (d * if_ + e * jf + f);

            let xpm = (16.0 * xp_float) as i32;
            let ypm = (16.0 * yp_float) as i32;

            // Integer and fractional parts
            let xp = xpm >> 4;
            let yp = ypm >> 4;
            let xf = xpm & 0x0f;
            let yf = ypm & 0x0f;

            // Bounds check
            if xp < 0 || yp < 0 || xp > wm2 || yp > hm2 {
                // Keep fill value (already set)
                continue;
            }

            // Get four neighboring pixels
            let p00 = pix.get_pixel_unchecked(xp as u32, yp as u32);
            let p10 = pix.get_pixel_unchecked((xp + 1) as u32, yp as u32);
            let p01 = pix.get_pixel_unchecked(xp as u32, (yp + 1) as u32);
            let p11 = pix.get_pixel_unchecked((xp + 1) as u32, (yp + 1) as u32);

            // Extract RGBA components
            let (r00, g00, b00, a00) = color::extract_rgba(p00);
            let (r10, g10, b10, a10) = color::extract_rgba(p10);
            let (r01, g01, b01, a01) = color::extract_rgba(p01);
            let (r11, g11, b11, a11) = color::extract_rgba(p11);

            // Area-weighted interpolation for each channel
            let r = area_interp(r00, r10, r01, r11, xf, yf);
            let gv = area_interp(g00, g10, g01, g11, xf, yf);
            let bv = area_interp(b00, b10, b01, b11, xf, yf);
            let av = area_interp(a00, a10, a01, a11, xf, yf);

            let pixel = color::compose_rgba(r, gv, bv, av);
            out_mut.set_pixel_unchecked(i, j, pixel);
        }
    }

    Ok(out_mut.into())
}

/// Area interpolation helper for a single channel
#[inline]
fn area_interp(v00: u8, v10: u8, v01: u8, v11: u8, xf: i32, yf: i32) -> u8 {
    let val = ((16 - xf) * (16 - yf) * v00 as i32
        + xf * (16 - yf) * v10 as i32
        + (16 - xf) * yf * v01 as i32
        + xf * yf * v11 as i32
        + 128)
        / 256;
    val.clamp(0, 255) as u8
}

// ============================================================================
// WithAlpha Projective Transformation
// ============================================================================

/// Apply a projective transformation preserving the alpha channel
///
/// This is equivalent to Leptonica's `pixProjectivePtaWithAlpha`.
///
/// The function transforms the RGB and alpha channels independently using
/// the same geometric mapping. This allows precise blending control—pixels
/// outside the transformed boundary become fully transparent.
///
/// # Arguments
/// * `pix` - Input 32bpp image (with or without alpha)
/// * `src_pts` - 4 source points
/// * `dst_pts` - 4 destination points
/// * `alpha_mask` - Optional 8bpp grayscale image for alpha. If `None`, uses `opacity`
/// * `opacity` - Opacity fraction (0.0 = transparent, 1.0 = opaque). Used when `alpha_mask` is `None`
/// * `border` - Number of border pixels for edge feathering
///
/// # Returns
/// A 32bpp RGBA image with spp=4
pub fn projective_pta_with_alpha(
    _pix: &Pix,
    _src_pts: [Point; 4],
    _dst_pts: [Point; 4],
    _alpha_mask: Option<&Pix>,
    _opacity: f32,
    _border: u32,
) -> TransformResult<Pix> {
    todo!("projective_pta_with_alpha not yet implemented")
}

/// Fill an image with a constant value
fn fill_image(pix: &mut leptonica_core::PixMut, value: u32) {
    let w = pix.width();
    let h = pix.height();
    for y in 0..h {
        for x in 0..w {
            pix.set_pixel_unchecked(x, y, value);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ProjectiveCoeffs basic tests
    // ========================================================================

    #[test]
    fn test_identity_coeffs() {
        let c = ProjectiveCoeffs::identity();
        let (x, y) = c.transform_point_float(10.0, 20.0).unwrap();
        assert!((x - 10.0).abs() < 1e-5);
        assert!((y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_from_coeffs() {
        // Translation by (5, 3): x' = (x + 5) / 1, y' = (y + 3) / 1
        let coeffs = [1.0, 0.0, 5.0, 0.0, 1.0, 3.0, 0.0, 0.0];
        let c = ProjectiveCoeffs::from_coeffs(coeffs);
        let (x, y) = c.transform_point_float(0.0, 0.0).unwrap();
        assert!((x - 5.0).abs() < 1e-5);
        assert!((y - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_transform_point_sampled() {
        let c = ProjectiveCoeffs::identity();
        let (x, y) = c.transform_point_sampled(10, 20).unwrap();
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }

    #[test]
    fn test_transform_point_at_infinity() {
        // Create coefficients where denominator can be zero
        // g=1, h=0 means denom = x + 1, which is 0 when x = -1
        let coeffs = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let c = ProjectiveCoeffs::from_coeffs(coeffs);
        // At x = -1, denominator = 0
        let result = c.transform_point_sampled(-1, 0);
        assert!(result.is_none());
    }

    // ========================================================================
    // Four-point correspondence tests
    // ========================================================================

    #[test]
    fn test_from_four_points_identity() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
        ];

        let c = ProjectiveCoeffs::from_four_points(pts, pts).unwrap();

        // Should be very close to identity
        for pt in &pts {
            let transformed = c.transform_point(*pt).unwrap();
            assert!(
                (transformed.x - pt.x).abs() < 1e-4,
                "x mismatch: {} vs {}",
                transformed.x,
                pt.x
            );
            assert!(
                (transformed.y - pt.y).abs() < 1e-4,
                "y mismatch: {} vs {}",
                transformed.y,
                pt.y
            );
        }
    }

    #[test]
    fn test_from_four_points_translation() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
        ];
        let dst = [
            Point::new(5.0, 3.0),
            Point::new(15.0, 3.0),
            Point::new(5.0, 13.0),
            Point::new(15.0, 13.0),
        ];

        let c = ProjectiveCoeffs::from_four_points(src, dst).unwrap();

        // Check that source points map to destination points
        for (s, d) in src.iter().zip(dst.iter()) {
            let transformed = c.transform_point(*s).unwrap();
            assert!((transformed.x - d.x).abs() < 1e-4);
            assert!((transformed.y - d.y).abs() < 1e-4);
        }
    }

    #[test]
    fn test_from_four_points_scale() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
        ];
        let dst = [
            Point::new(0.0, 0.0),
            Point::new(20.0, 0.0),
            Point::new(0.0, 30.0),
            Point::new(20.0, 30.0),
        ];

        let c = ProjectiveCoeffs::from_four_points(src, dst).unwrap();

        // Check corner points
        for (s, d) in src.iter().zip(dst.iter()) {
            let transformed = c.transform_point(*s).unwrap();
            assert!((transformed.x - d.x).abs() < 1e-4);
            assert!((transformed.y - d.y).abs() < 1e-4);
        }
    }

    #[test]
    fn test_from_four_points_keystone() {
        // Simulate keystone distortion (perspective-like)
        let src = [
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(0.0, 100.0),
            Point::new(100.0, 100.0),
        ];
        let dst = [
            Point::new(10.0, 0.0),    // Top-left moved right
            Point::new(90.0, 0.0),    // Top-right moved left
            Point::new(0.0, 100.0),   // Bottom-left unchanged
            Point::new(100.0, 100.0), // Bottom-right unchanged
        ];

        let c = ProjectiveCoeffs::from_four_points(src, dst).unwrap();

        // Check that all 4 points map correctly
        for (s, d) in src.iter().zip(dst.iter()) {
            let transformed = c.transform_point(*s).unwrap();
            assert!(
                (transformed.x - d.x).abs() < 1e-3,
                "x: {} vs {}",
                transformed.x,
                d.x
            );
            assert!(
                (transformed.y - d.y).abs() < 1e-3,
                "y: {} vs {}",
                transformed.y,
                d.y
            );
        }
    }

    // ========================================================================
    // Image transformation tests
    // ========================================================================

    #[test]
    fn test_projective_sampled_identity() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                pix_mut.set_pixel_unchecked(x, y, (x + y * 20) % 256);
            }
        }
        let pix: Pix = pix_mut.into();

        let c = ProjectiveCoeffs::identity();
        let result = projective_sampled(&pix, &c, AffineFill::White).unwrap();

        // Check that all pixels are preserved
        for y in 0..20 {
            for x in 0..20 {
                let orig = pix.get_pixel_unchecked(x, y);
                let trans = result.get_pixel_unchecked(x, y);
                assert_eq!(orig, trans, "Mismatch at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_projective_sampled_translation() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set a marker pixel at (10, 10)
        pix_mut.set_pixel_unchecked(10, 10, 200);
        let pix: Pix = pix_mut.into();

        // Translation by (5, 3) using inverse transform
        // For forward transform: x_dst = x_src + 5, y_dst = y_src + 3
        // For inverse (dst -> src): x_src = x_dst - 5, y_src = y_dst - 3
        // Coeffs: x' = x - 5, y' = y - 3
        // a=1, b=0, c=-5, d=0, e=1, f=-3, g=0, h=0
        let inv_coeffs = ProjectiveCoeffs::from_coeffs([1.0, 0.0, -5.0, 0.0, 1.0, -3.0, 0.0, 0.0]);

        let result = projective_sampled(&pix, &inv_coeffs, AffineFill::Black).unwrap();

        // Marker should now be at (15, 13)
        assert_eq!(result.get_pixel_unchecked(15, 13), 200);
    }

    #[test]
    fn test_projective_sampled_pta() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();

        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
            Point::new(49.0, 49.0),
        ];
        let dst = [
            Point::new(5.0, 5.0),
            Point::new(44.0, 5.0),
            Point::new(5.0, 44.0),
            Point::new(44.0, 44.0),
        ];

        let result = projective_sampled_pta(&pix, src, dst, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_projective_interpolated_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let c = ProjectiveCoeffs::identity();
        let result = projective(&pix, &c, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_projective_interpolated_32bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let c = ProjectiveCoeffs::identity();
        let result = projective(&pix, &c, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_projective_1bpp_falls_back_to_sampling() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let c = ProjectiveCoeffs::identity();
        let result = projective(&pix, &c, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_projective_pta() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();

        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
            Point::new(49.0, 49.0),
        ];
        let dst = [
            Point::new(5.0, 5.0),
            Point::new(44.0, 5.0),
            Point::new(5.0, 44.0),
            Point::new(44.0, 44.0),
        ];

        let result = projective_pta(&pix, src, dst, AffineFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Colormap preservation test
    // ========================================================================

    #[test]
    fn test_projective_preserves_colormap() {
        use leptonica_core::PixColormap;

        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a simple colormap
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // Red
        cmap.add_rgb(0, 255, 0).unwrap(); // Green
        let _ = pix_mut.set_colormap(Some(cmap));

        let pix: Pix = pix_mut.into();

        let c = ProjectiveCoeffs::identity();
        let result = projective_sampled(&pix, &c, AffineFill::White).unwrap();

        assert!(result.colormap().is_some());
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_projective_out_of_bounds_fill() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Fill with value 100
        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pix_mut.into();

        // Large translation that moves most pixels out of bounds
        let inv_coeffs =
            ProjectiveCoeffs::from_coeffs([1.0, 0.0, -100.0, 0.0, 1.0, -100.0, 0.0, 0.0]);
        let result = projective_sampled(&pix, &inv_coeffs, AffineFill::Black).unwrap();

        // All pixels should be black (fill value)
        for y in 0..10 {
            for x in 0..10 {
                let val = result.get_pixel_unchecked(x, y);
                assert_eq!(val, 0, "Expected black fill at ({}, {})", x, y);
            }
        }
    }

    // ========================================================================
    // WithAlpha tests
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_projective_pta_with_alpha_basic() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        for y in 0..50u32 {
            for x in 0..50u32 {
                let pixel = color::compose_rgba((x * 5) as u8, (y * 5) as u8, 128, 255);
                pm.set_pixel_unchecked(x, y, pixel);
            }
        }
        let pix: Pix = pm.into();

        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
            Point::new(49.0, 49.0),
        ];
        let dst = [
            Point::new(2.0, 2.0),
            Point::new(47.0, 2.0),
            Point::new(2.0, 47.0),
            Point::new(47.0, 47.0),
        ];

        let result = projective_pta_with_alpha(&pix, src, dst, None, 1.0, 10).unwrap();

        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.spp(), 4);
        assert_eq!(result.width(), 70);
        assert_eq!(result.height(), 70);

        // Border pixels should have alpha = 0
        assert_eq!(color::alpha(result.get_pixel_unchecked(0, 0)), 0);

        // Interior pixels should have non-zero alpha
        let center_pixel = result.get_pixel_unchecked(35, 35);
        assert!(color::alpha(center_pixel) > 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_projective_pta_with_alpha_invalid_depth() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(19.0, 0.0),
            Point::new(0.0, 19.0),
            Point::new(19.0, 19.0),
        ];
        let result = projective_pta_with_alpha(&pix, pts, pts, None, 1.0, 5);
        assert!(result.is_err());
    }

    // ========================================================================
    // Gauss-Jordan tests
    // ========================================================================

    #[test]
    fn test_gauss_jordan_identity() {
        // Test solving Ax = b where A is identity
        let mut a = [
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        ];
        let mut b = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let result = gauss_jordan_8x8(&mut a, &mut b);
        assert!(result.is_ok());

        // Solution should be unchanged
        assert!((b[0] - 1.0).abs() < 1e-5);
        assert!((b[7] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_gauss_jordan_singular() {
        // Test with a singular matrix (all zeros in first row)
        let mut a = [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        ];
        let mut b = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let result = gauss_jordan_8x8(&mut a, &mut b);
        assert!(matches!(result, Err(TransformError::SingularMatrix)));
    }
}
