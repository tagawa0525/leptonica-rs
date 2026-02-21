//! Affine transformations for images
//!
//! This module provides affine transformation operations including:
//! - Affine transformation matrix construction (from 3 point correspondences)
//! - Sampled affine transformation (nearest-neighbor, like pixAffineSampled)
//! - Interpolated affine transformation (bilinear, like pixAffine)
//! - Inverse affine transformation
//! - Transformation composition
//!
//! # Affine Matrix
//!
//! An affine transformation can be represented as:
//! ```text
//! | a  b  tx |
//! | c  d  ty |
//! | 0  0  1  |
//! ```
//!
//! The transformation equations are:
//! ```text
//! x' = a*x + b*y + tx
//! y' = c*x + d*y + ty
//! ```
//!
//! # Example
//!
//! ```no_run
//! use leptonica_transform::affine::{AffineMatrix, AffineFill, affine_sampled, Point};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
//!
//! // Create a rotation matrix
//! let matrix = AffineMatrix::rotation(50.0, 50.0, 0.5);
//!
//! // Apply the transformation
//! let transformed = affine_sampled(&pix, &matrix, AffineFill::White).unwrap();
//! ```

use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixelDepth, color};

// ============================================================================
// Type Definitions
// ============================================================================

/// A 2D point with floating-point coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Background fill color for affine transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AffineFill {
    /// Fill with white pixels
    #[default]
    White,
    /// Fill with black pixels
    Black,
    /// Fill with a specific color value (interpretation depends on depth)
    Color(u32),
}

impl AffineFill {
    /// Get the fill value for a specific pixel depth
    pub fn to_value(self, depth: PixelDepth) -> u32 {
        match self {
            AffineFill::White => match depth {
                PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
                PixelDepth::Bit2 => 3,
                PixelDepth::Bit4 => 15,
                PixelDepth::Bit8 => 255,
                PixelDepth::Bit16 => 65535,
                PixelDepth::Bit32 => 0xFFFFFF00,
            },
            AffineFill::Black => match depth {
                PixelDepth::Bit1 => 1, // 1 = black for binary
                PixelDepth::Bit32 => 0x00000000,
                _ => 0,
            },
            AffineFill::Color(val) => val,
        }
    }
}

/// 2D affine transformation matrix
///
/// The matrix is stored as 6 coefficients [a, b, tx, c, d, ty] representing:
/// ```text
/// | a  b  tx |
/// | c  d  ty |
/// | 0  0  1  |
/// ```
///
/// Transformation equations:
/// - x' = a*x + b*y + tx
/// - y' = c*x + d*y + ty
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineMatrix {
    /// Coefficients [a, b, tx, c, d, ty]
    coeffs: [f32; 6],
}

impl Default for AffineMatrix {
    fn default() -> Self {
        Self::identity()
    }
}

impl AffineMatrix {
    /// Create an identity matrix (no transformation)
    pub fn identity() -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        }
    }

    /// Create from raw coefficients [a, b, tx, c, d, ty]
    pub fn from_coeffs(coeffs: [f32; 6]) -> Self {
        Self { coeffs }
    }

    /// Get the coefficients [a, b, tx, c, d, ty]
    pub fn coeffs(&self) -> &[f32; 6] {
        &self.coeffs
    }

    /// Create a translation matrix
    ///
    /// Moves points by (tx, ty):
    /// - x' = x + tx
    /// - y' = y + ty
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            coeffs: [1.0, 0.0, tx, 0.0, 1.0, ty],
        }
    }

    /// Create a scaling matrix
    ///
    /// Scales about the origin by (sx, sy):
    /// - x' = sx * x
    /// - y' = sy * y
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            coeffs: [sx, 0.0, 0.0, 0.0, sy, 0.0],
        }
    }

    /// Create a rotation matrix
    ///
    /// Rotates about (center_x, center_y) by angle radians.
    /// Positive angle = clockwise rotation (Leptonica convention).
    ///
    /// # Arguments
    /// * `center_x` - X coordinate of rotation center
    /// * `center_y` - Y coordinate of rotation center
    /// * `angle` - Rotation angle in radians (positive = clockwise)
    pub fn rotation(center_x: f32, center_y: f32, angle: f32) -> Self {
        let cosa = angle.cos();
        let sina = angle.sin();

        // Rotation about (xc, yc):
        // x' = cosa*(x-xc) - sina*(y-yc) + xc
        // y' = sina*(x-xc) + cosa*(y-yc) + yc
        //
        // Expanded:
        // x' = cosa*x - sina*y + xc*(1-cosa) + yc*sina
        // y' = sina*x + cosa*y + yc*(1-cosa) - xc*sina

        Self {
            coeffs: [
                cosa,
                -sina,
                center_x * (1.0 - cosa) + center_y * sina,
                sina,
                cosa,
                center_y * (1.0 - cosa) - center_x * sina,
            ],
        }
    }

    /// Create an affine matrix from 3 point correspondences
    ///
    /// Given 3 source points and 3 destination points, computes the affine
    /// transformation that maps src_pts to dst_pts.
    ///
    /// The 3 points must not be collinear.
    ///
    /// # Arguments
    /// * `src_pts` - Source points [p1, p2, p3]
    /// * `dst_pts` - Destination points [p1', p2', p3']
    ///
    /// # Returns
    /// The affine matrix that transforms src_pts to dst_pts
    ///
    /// # Errors
    /// Returns `TransformError::SingularMatrix` if points are collinear
    pub fn from_three_points(src_pts: [Point; 3], dst_pts: [Point; 3]) -> TransformResult<Self> {
        // We solve the system of 6 equations:
        // x1' = a*x1 + b*y1 + tx
        // y1' = c*x1 + d*y1 + ty
        // x2' = a*x2 + b*y2 + tx
        // y2' = c*x2 + d*y2 + ty
        // x3' = a*x3 + b*y3 + tx
        // y3' = c*x3 + d*y3 + ty

        let x1 = src_pts[0].x;
        let y1 = src_pts[0].y;
        let x2 = src_pts[1].x;
        let y2 = src_pts[1].y;
        let x3 = src_pts[2].x;
        let y3 = src_pts[2].y;

        // Build 6x6 matrix A and right-hand side vector b
        let mut a = [
            [x1, y1, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, x1, y1, 1.0],
            [x2, y2, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, x2, y2, 1.0],
            [x3, y3, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, x3, y3, 1.0],
        ];

        let mut b = [
            dst_pts[0].x,
            dst_pts[0].y,
            dst_pts[1].x,
            dst_pts[1].y,
            dst_pts[2].x,
            dst_pts[2].y,
        ];

        // Solve using Gauss-Jordan elimination
        gauss_jordan(&mut a, &mut b, 6)?;

        // b now contains the solution [a, b, tx, c, d, ty]
        Ok(Self {
            coeffs: [b[0], b[1], b[2], b[3], b[4], b[5]],
        })
    }

    /// Compute the inverse transformation
    ///
    /// # Returns
    /// The inverse affine matrix M^-1 such that M * M^-1 = I
    ///
    /// # Errors
    /// Returns `TransformError::SingularMatrix` if the matrix is not invertible
    pub fn inverse(&self) -> TransformResult<Self> {
        let [a, b, tx, c, d, ty] = self.coeffs;

        // The 2x2 submatrix is:
        // | a  b |
        // | c  d |
        // Its inverse is 1/det * | d  -b |
        //                        | -c  a |

        let det = a * d - b * c;
        if det.abs() < 1e-10 {
            return Err(TransformError::SingularMatrix);
        }

        let inv_det = 1.0 / det;

        // Inverse of the 2x2 submatrix
        let a_inv = d * inv_det;
        let b_inv = -b * inv_det;
        let c_inv = -c * inv_det;
        let d_inv = a * inv_det;

        // The translation part: -A^-1 * t
        let tx_inv = -(a_inv * tx + b_inv * ty);
        let ty_inv = -(c_inv * tx + d_inv * ty);

        Ok(Self {
            coeffs: [a_inv, b_inv, tx_inv, c_inv, d_inv, ty_inv],
        })
    }

    /// Compose two transformations
    ///
    /// Returns a new matrix that applies `self` first, then `other`.
    /// Equivalent to matrix multiplication: other * self
    pub fn compose(&self, other: &Self) -> Self {
        let [a1, b1, tx1, c1, d1, ty1] = self.coeffs;
        let [a2, b2, tx2, c2, d2, ty2] = other.coeffs;

        // Matrix multiplication:
        // | a2 b2 tx2 |   | a1 b1 tx1 |
        // | c2 d2 ty2 | * | c1 d1 ty1 |
        // | 0  0  1   |   | 0  0  1   |

        Self {
            coeffs: [
                a2 * a1 + b2 * c1,
                a2 * b1 + b2 * d1,
                a2 * tx1 + b2 * ty1 + tx2,
                c2 * a1 + d2 * c1,
                c2 * b1 + d2 * d1,
                c2 * tx1 + d2 * ty1 + ty2,
            ],
        }
    }

    /// Transform a point using this matrix
    ///
    /// Returns the transformed point (x', y') where:
    /// - x' = a*x + b*y + tx
    /// - y' = c*x + d*y + ty
    pub fn transform_point(&self, pt: Point) -> Point {
        let [a, b, tx, c, d, ty] = self.coeffs;
        Point {
            x: a * pt.x + b * pt.y + tx,
            y: c * pt.x + d * pt.y + ty,
        }
    }

    /// Transform a point with integer rounding (for sampled transforms)
    ///
    /// Returns the nearest integer coordinates after transformation.
    pub fn transform_point_sampled(&self, x: i32, y: i32) -> (i32, i32) {
        let [a, b, tx, c, d, ty] = self.coeffs;
        let xf = x as f32;
        let yf = y as f32;
        let xp = (a * xf + b * yf + tx + 0.5).floor() as i32;
        let yp = (c * xf + d * yf + ty + 0.5).floor() as i32;
        (xp, yp)
    }

    /// Transform a point returning floating point coordinates
    pub fn transform_point_float(&self, x: f32, y: f32) -> (f32, f32) {
        let [a, b, tx, c, d, ty] = self.coeffs;
        let xp = a * x + b * y + tx;
        let yp = c * x + d * y + ty;
        (xp, yp)
    }
}

// ============================================================================
// Gauss-Jordan Elimination
// ============================================================================

/// Solve a system of linear equations using Gauss-Jordan elimination
///
/// Solves Ax = b in-place. After completion:
/// - a is transformed to the identity matrix
/// - b contains the solution x
///
/// # Arguments
/// * `a` - n x n matrix (modified in place to identity)
/// * `b` - n x 1 vector (modified in place to solution)
/// * `n` - dimension
fn gauss_jordan(a: &mut [[f32; 6]; 6], b: &mut [f32; 6], n: usize) -> TransformResult<()> {
    let mut index_c = [0usize; 6];
    let mut index_r = [0usize; 6];
    let mut ipiv = [0i32; 6];

    for i in 0..n {
        let mut max_val = 0.0f32;
        let mut irow = 0;
        let mut icol = 0;

        // Find pivot
        for j in 0..n {
            if ipiv[j] != 1 {
                for k in 0..n {
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
        for item in a[icol].iter_mut().take(n) {
            *item *= pivinv;
        }
        b[icol] *= pivinv;

        // Reduce other rows
        for row in 0..n {
            if row != icol {
                let val = a[row][icol];
                a[row][icol] = 0.0;
                // We need to access both a[row] and a[icol], so cannot use simple iterator
                #[allow(clippy::needless_range_loop)]
                for col in 0..n {
                    a[row][col] -= a[icol][col] * val;
                }
                b[row] -= b[icol] * val;
            }
        }
    }

    // Unscramble columns
    for col in (0..n).rev() {
        if index_r[col] != index_c[col] {
            for row_arr in a.iter_mut().take(n) {
                row_arr.swap(index_r[col], index_c[col]);
            }
        }
    }

    Ok(())
}

// ============================================================================
// Sampled Affine Transformation
// ============================================================================

/// Apply an affine transformation using nearest-neighbor sampling
///
/// This is equivalent to Leptonica's `pixAffineSampled`.
/// Works with all pixel depths. Fastest but lowest quality.
///
/// # Arguments
/// * `pix` - Input image
/// * `matrix` - Affine transformation matrix (forward transform)
/// * `fill` - Background fill color
///
/// # Note
/// The matrix should be the forward transform (src -> dst).
/// Internally, the inverse is computed to map destination pixels to source.
pub fn affine_sampled(pix: &Pix, matrix: &AffineMatrix, fill: AffineFill) -> TransformResult<Pix> {
    // We need the inverse matrix to map from destination to source
    let inv_matrix = matrix.inverse()?;
    affine_sampled_with_inverse(pix, &inv_matrix, fill)
}

/// Apply an affine transformation using the inverse matrix directly
fn affine_sampled_with_inverse(
    pix: &Pix,
    inv_matrix: &AffineMatrix,
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
            let (sx, sy) = inv_matrix.transform_point_sampled(i as i32, j as i32);

            if sx >= 0 && sx < wi && sy >= 0 && sy < hi {
                let val = pix.get_pixel_unchecked(sx as u32, sy as u32);
                out_mut.set_pixel_unchecked(i, j, val);
            }
            // Pixels outside source keep the fill value
        }
    }

    Ok(out_mut.into())
}

/// Apply an affine transformation using 3 point correspondences (sampled)
///
/// This is equivalent to Leptonica's `pixAffineSampledPta`.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 3 points in source coordinate space
/// * `dst_pts` - 3 corresponding points in destination coordinate space
/// * `fill` - Background fill color
pub fn affine_sampled_pta(
    pix: &Pix,
    src_pts: [Point; 3],
    dst_pts: [Point; 3],
    fill: AffineFill,
) -> TransformResult<Pix> {
    // Compute the transform from dst -> src (inverse)
    // This maps destination points to source points
    let inv_matrix = AffineMatrix::from_three_points(dst_pts, src_pts)?;
    affine_sampled_with_inverse(pix, &inv_matrix, fill)
}

// ============================================================================
// Interpolated Affine Transformation
// ============================================================================

/// Apply an affine transformation with bilinear interpolation
///
/// This is equivalent to Leptonica's `pixAffine`.
/// Works best with 8bpp grayscale and 32bpp color images.
/// Falls back to sampling for 1bpp and other depths.
///
/// # Arguments
/// * `pix` - Input image
/// * `matrix` - Affine transformation matrix (forward transform)
/// * `fill` - Background fill color
pub fn affine(pix: &Pix, matrix: &AffineMatrix, fill: AffineFill) -> TransformResult<Pix> {
    let depth = pix.depth();

    // For 1bpp, use sampling (interpolation doesn't make sense)
    if depth == PixelDepth::Bit1 {
        return affine_sampled(pix, matrix, fill);
    }

    // We need the inverse matrix to map from destination to source
    let inv_matrix = matrix.inverse()?;

    match depth {
        PixelDepth::Bit8 if pix.colormap().is_none() => affine_gray(pix, &inv_matrix, fill),
        PixelDepth::Bit32 => affine_color(pix, &inv_matrix, fill),
        _ => {
            // For other depths (2bpp, 4bpp, 8bpp with colormap, 16bpp),
            // fall back to sampling
            affine_sampled_with_inverse(pix, &inv_matrix, fill)
        }
    }
}

/// Apply an affine transformation using 3 point correspondences (interpolated)
///
/// This is equivalent to Leptonica's `pixAffinePta`.
///
/// # Arguments
/// * `pix` - Input image
/// * `src_pts` - 3 points in source coordinate space
/// * `dst_pts` - 3 corresponding points in destination coordinate space
/// * `fill` - Background fill color
pub fn affine_pta(
    pix: &Pix,
    src_pts: [Point; 3],
    dst_pts: [Point; 3],
    fill: AffineFill,
) -> TransformResult<Pix> {
    let depth = pix.depth();

    // For 1bpp, use sampling
    if depth == PixelDepth::Bit1 {
        return affine_sampled_pta(pix, src_pts, dst_pts, fill);
    }

    // Compute the transform from dst -> src (inverse)
    let inv_matrix = AffineMatrix::from_three_points(dst_pts, src_pts)?;

    match depth {
        PixelDepth::Bit8 if pix.colormap().is_none() => affine_gray(pix, &inv_matrix, fill),
        PixelDepth::Bit32 => affine_color(pix, &inv_matrix, fill),
        _ => affine_sampled_with_inverse(pix, &inv_matrix, fill),
    }
}

/// Affine transform for 8bpp grayscale with bilinear interpolation
fn affine_gray(pix: &Pix, inv_matrix: &AffineMatrix, fill: AffineFill) -> TransformResult<Pix> {
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

    let [a, b, tx, c, d, ty] = *inv_matrix.coeffs();

    // Scale coefficients by 16 for sub-pixel precision
    let a16 = 16.0 * a;
    let b16 = 16.0 * b;
    let c16 = 16.0 * c;
    let d16 = 16.0 * d;

    for j in 0..h {
        let jf = j as f32;
        for i in 0..w {
            let if_ = i as f32;

            // Compute sub-pixel position (scaled by 16)
            let xpm = (a16 * if_ + b16 * jf + 16.0 * tx) as i32;
            let ypm = (c16 * if_ + d16 * jf + 16.0 * ty) as i32;

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

/// Affine transform for 32bpp color with bilinear interpolation
fn affine_color(pix: &Pix, inv_matrix: &AffineMatrix, fill: AffineFill) -> TransformResult<Pix> {
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

    let [a, b, tx, c, d, ty] = *inv_matrix.coeffs();

    // Scale coefficients by 16 for sub-pixel precision
    let a16 = 16.0 * a;
    let b16 = 16.0 * b;
    let c16 = 16.0 * c;
    let d16 = 16.0 * d;

    for j in 0..h {
        let jf = j as f32;
        for i in 0..w {
            let if_ = i as f32;

            // Compute sub-pixel position (scaled by 16)
            let xpm = (a16 * if_ + b16 * jf + 16.0 * tx) as i32;
            let ypm = (c16 * if_ + d16 * jf + 16.0 * ty) as i32;

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
            let g = area_interp(g00, g10, g01, g11, xf, yf);
            let b = area_interp(b00, b10, b01, b11, xf, yf);
            let av = area_interp(a00, a10, a01, a11, xf, yf);

            let pixel = color::compose_rgba(r, g, b, av);
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
// WithAlpha Affine Transformation
// ============================================================================

/// Apply an affine transformation preserving the alpha channel
///
/// This is equivalent to Leptonica's `pixAffinePtaWithAlpha`.
///
/// The function transforms the RGB and alpha channels independently using
/// the same geometric mapping. This allows precise blending control—pixels
/// outside the transformed boundary become fully transparent.
///
/// # Arguments
/// * `pix` - Input 32bpp image (with or without alpha)
/// * `src_pts` - 3 source points
/// * `dst_pts` - 3 destination points
/// * `alpha_mask` - Optional 8bpp grayscale image for alpha. If `None`, uses `opacity`
/// * `opacity` - Opacity fraction (0.0 = transparent, 1.0 = opaque). Used when `alpha_mask` is `None`
/// * `border` - Number of border pixels to add around the image and alpha mask. Controls the
///   region used for edge feathering and increases the output image size (shifting the point
///   coordinate space by `border` pixels in both X and Y).
///
/// # Returns
/// A 32bpp RGBA image with spp=4
pub fn affine_pta_with_alpha(
    pix: &Pix,
    src_pts: [Point; 3],
    dst_pts: [Point; 3],
    alpha_mask: Option<&Pix>,
    opacity: f32,
    border: u32,
) -> TransformResult<Pix> {
    with_alpha_transform(
        pix,
        alpha_mask,
        opacity,
        border,
        &src_pts,
        &dst_pts,
        |img, src, dst| {
            let s: [Point; 3] = [src[0], src[1], src[2]];
            let d: [Point; 3] = [dst[0], dst[1], dst[2]];
            affine_pta(img, s, d, AffineFill::Black)
        },
    )
}

/// Common WithAlpha transform implementation
///
/// Shared by affine, bilinear, and projective WithAlpha transforms.
/// The `transform_fn` closure performs the actual geometric transform
/// on both color (32bpp) and grayscale (8bpp) images.
///
/// Algorithm (follows C Leptonica's `pixAffinePtaWithAlpha` pattern):
/// 1. Add border to source image
/// 2. Adjust point coordinates for border offset
/// 3. Transform RGB channels (color)
/// 4. Prepare alpha channel (from mask or uniform opacity)
/// 5. Apply edge feathering rings
/// 6. Add border to alpha and transform (grayscale)
/// 7. Set alpha channel in result
pub(crate) fn with_alpha_transform<F>(
    pix: &Pix,
    alpha_mask: Option<&Pix>,
    opacity: f32,
    border: u32,
    src_pts: &[Point],
    dst_pts: &[Point],
    transform_fn: F,
) -> TransformResult<Pix>
where
    F: Fn(&Pix, &[Point], &[Point]) -> TransformResult<Pix>,
{
    use leptonica_core::pix::RgbComponent;

    // Validate inputs
    if pix.depth() != PixelDepth::Bit32 {
        return Err(TransformError::UnsupportedDepth(
            "WithAlpha requires 32bpp input".into(),
        ));
    }

    let opacity = opacity.clamp(0.0, 1.0);

    if let Some(mask) = alpha_mask
        && mask.depth() != PixelDepth::Bit8
    {
        return Err(TransformError::InvalidParameters(
            "alpha_mask must be 8bpp".into(),
        ));
    }

    let ws = pix.width();
    let hs = pix.height();

    // Step 1: Add border to source image (black border)
    let pixb1 = pix.add_border(border, 0)?;

    // Step 2: Adjust point coordinates for border offset
    let border_f = border as f32;
    let adj_src: Vec<Point> = src_pts
        .iter()
        .map(|p| Point::new(p.x + border_f, p.y + border_f))
        .collect();
    let adj_dst: Vec<Point> = dst_pts
        .iter()
        .map(|p| Point::new(p.x + border_f, p.y + border_f))
        .collect();

    // Step 3: Transform RGB channels
    let pixd = transform_fn(&pixb1, &adj_src, &adj_dst)?;

    // Step 4: Prepare alpha channel (8bpp grayscale)
    let use_custom_mask = alpha_mask.is_some();
    let pix_alpha = if let Some(mask) = alpha_mask {
        // Resize alpha mask to match source dimensions using resize_to_match
        // (crops if larger, replicates last row/col if smaller)
        if mask.width() != ws || mask.height() != hs {
            mask.resize_to_match(None, ws, hs)?
        } else {
            mask.deep_clone()
        }
    } else {
        // Create uniform alpha from opacity
        let alpha_val = (255.0 * opacity) as u32;
        let alpha_pix = Pix::new(ws, hs, PixelDepth::Bit8)?;
        let mut am = alpha_pix.try_into_mut().unwrap();
        for y in 0..hs {
            for x in 0..ws {
                am.set_pixel_unchecked(x, y, alpha_val);
            }
        }
        am.into()
    };

    // Step 5: Apply edge feathering (only for images > 10x10, only when no custom mask)
    // Feathering is skipped for custom masks to avoid increasing alpha above the mask's values.
    let pix_alpha = if !use_custom_mask && ws > 10 && hs > 10 {
        let mut am = pix_alpha.try_into_mut().unwrap();
        // Ring 1 (outermost): fully transparent
        am.set_border_val(1, 1, 1, 1, 0)
            .map_err(TransformError::Core)?;

        // Ring 2: semi-transparent
        if ws > 12 && hs > 12 {
            let ring2_val = (255.0 * opacity * 0.5) as u32;
            for x in 1..(ws - 1) {
                am.set_pixel_unchecked(x, 1, ring2_val);
                am.set_pixel_unchecked(x, hs - 2, ring2_val);
            }
            for y in 1..(hs - 1) {
                am.set_pixel_unchecked(1, y, ring2_val);
                am.set_pixel_unchecked(ws - 2, y, ring2_val);
            }
        }
        am.into()
    } else {
        pix_alpha
    };

    // Step 6: Add border to alpha (black border = fully transparent)
    let pixb2 = pix_alpha.add_border(border, 0)?;

    // Step 7: Transform alpha channel (grayscale)
    let pixga = transform_fn(&pixb2, &adj_src, &adj_dst)?;

    // Step 8: Set alpha channel in result
    let mut pixd_mut = pixd.try_into_mut().unwrap();
    pixd_mut
        .set_rgb_component(&pixga, RgbComponent::Alpha)
        .map_err(TransformError::Core)?;

    Ok(pixd_mut.into())
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Translate an image by (tx, ty) pixels
///
/// This is a simple wrapper around affine_sampled with a translation matrix.
pub fn translate(pix: &Pix, tx: f32, ty: f32) -> TransformResult<Pix> {
    let matrix = AffineMatrix::translation(tx, ty);
    affine_sampled(pix, &matrix, AffineFill::White)
}

/// Scale an image using affine transformation
///
/// Unlike the dedicated scale functions, this uses affine transform
/// which may have different boundary behavior.
pub fn affine_scale(pix: &Pix, sx: f32, sy: f32) -> TransformResult<Pix> {
    if sx <= 0.0 || sy <= 0.0 {
        return Err(TransformError::InvalidParameters(
            "scale factors must be positive".to_string(),
        ));
    }
    let matrix = AffineMatrix::scale(sx, sy);
    affine(pix, &matrix, AffineFill::White)
}

/// Rotate an image using affine transformation
///
/// Rotates about the specified center point.
///
/// # Arguments
/// * `pix` - Input image
/// * `center_x` - X coordinate of rotation center
/// * `center_y` - Y coordinate of rotation center
/// * `angle` - Rotation angle in radians (positive = clockwise)
pub fn affine_rotate(pix: &Pix, center_x: f32, center_y: f32, angle: f32) -> TransformResult<Pix> {
    let matrix = AffineMatrix::rotation(center_x, center_y, angle);
    affine(pix, &matrix, AffineFill::White)
}

// ============================================================================
// PTA / BOXA Affine Transform
// ============================================================================

/// Returns a new Pta with all points transformed by an affine matrix.
///
/// Applies: x' = a*x + b*y + tx, y' = c*x + d*y + ty
///
/// Corresponds to C Leptonica's `ptaAffineTransform`.
pub fn pta_affine_transform(
    pta: &leptonica_core::Pta,
    matrix: &AffineMatrix,
) -> leptonica_core::Pta {
    let c = matrix.coeffs();
    // c = [a, b, tx, d, e, ty]
    // x' = a*x + b*y + tx
    // y' = d*x + e*y + ty
    pta.iter()
        .map(|(x, y)| (c[0] * x + c[1] * y + c[2], c[3] * x + c[4] * y + c[5]))
        .collect()
}

/// Returns a new Boxa with all boxes transformed by an affine matrix.
///
/// Each box's 4 corners are transformed by the affine matrix; the axis-aligned
/// bounding box of those corners becomes the new box.
///
/// Corresponds to C Leptonica's `boxaAffineTransform`.
pub fn boxa_affine_transform(
    boxa: &leptonica_core::Boxa,
    matrix: &AffineMatrix,
) -> leptonica_core::Boxa {
    use leptonica_core::Pta;

    // Convert each box to 4 corners, transform, then find bounding box
    let n = boxa.len();
    let mut flat_pta = Pta::with_capacity(n * 4);
    for b in boxa.iter() {
        let x0 = b.x as f32;
        let y0 = b.y as f32;
        let x1 = (b.x + b.w) as f32;
        let y1 = (b.y + b.h) as f32;
        flat_pta.push(x0, y0);
        flat_pta.push(x1, y0);
        flat_pta.push(x0, y1);
        flat_pta.push(x1, y1);
    }

    let transformed = pta_affine_transform(&flat_pta, matrix);

    // Group every 4 points into a bounding box
    let mut result = leptonica_core::Boxa::with_capacity(n);
    for i in 0..n {
        let base = i * 4;
        let xmin = (0..4)
            .map(|k| transformed.get(base + k).unwrap().0)
            .fold(f32::INFINITY, f32::min);
        let ymin = (0..4)
            .map(|k| transformed.get(base + k).unwrap().1)
            .fold(f32::INFINITY, f32::min);
        let xmax = (0..4)
            .map(|k| transformed.get(base + k).unwrap().0)
            .fold(f32::NEG_INFINITY, f32::max);
        let ymax = (0..4)
            .map(|k| transformed.get(base + k).unwrap().1)
            .fold(f32::NEG_INFINITY, f32::max);
        result.push(leptonica_core::Box::new_unchecked(
            xmin.round() as i32,
            ymin.round() as i32,
            (xmax - xmin).round() as i32,
            (ymax - ymin).round() as i32,
        ));
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Point tests
    // ========================================================================

    #[test]
    fn test_point_new() {
        let pt = Point::new(1.5, 2.5);
        assert_eq!(pt.x, 1.5);
        assert_eq!(pt.y, 2.5);
    }

    // ========================================================================
    // AffineMatrix basic tests
    // ========================================================================

    #[test]
    fn test_identity_matrix() {
        let m = AffineMatrix::identity();
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - pt.x).abs() < 1e-5);
        assert!((transformed.y - pt.y).abs() < 1e-5);
    }

    #[test]
    fn test_translation_matrix() {
        let m = AffineMatrix::translation(5.0, -3.0);
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - 15.0).abs() < 1e-5);
        assert!((transformed.y - 17.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_matrix() {
        let m = AffineMatrix::scale(2.0, 0.5);
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - 20.0).abs() < 1e-5);
        assert!((transformed.y - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotation_matrix_90_deg() {
        // 90 degrees = pi/2 radians
        // In image coordinates (Y down), positive angle = clockwise
        // The matrix formula x' = cos*x - sin*y, y' = sin*x + cos*y
        // gives mathematical counter-clockwise rotation.
        // For pi/2: cos=0, sin=1, so (1,0) -> (0,1)
        let m = AffineMatrix::rotation(0.0, 0.0, std::f32::consts::FRAC_PI_2);
        let pt = Point::new(1.0, 0.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - 0.0).abs() < 1e-5);
        assert!((transformed.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotation_about_center() {
        // Rotate 180 degrees about point (5, 5)
        let m = AffineMatrix::rotation(5.0, 5.0, std::f32::consts::PI);
        let pt = Point::new(10.0, 5.0);
        let transformed = m.transform_point(pt);
        // (10, 5) -> (0, 5) when rotated 180 degrees about (5, 5)
        assert!((transformed.x - 0.0).abs() < 1e-4);
        assert!((transformed.y - 5.0).abs() < 1e-4);
    }

    // ========================================================================
    // Inverse tests
    // ========================================================================

    #[test]
    fn test_inverse_identity() {
        let m = AffineMatrix::identity();
        let inv = m.inverse().unwrap();
        let pt = Point::new(10.0, 20.0);
        let transformed = inv.transform_point(pt);
        assert!((transformed.x - pt.x).abs() < 1e-5);
        assert!((transformed.y - pt.y).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_translation() {
        let m = AffineMatrix::translation(5.0, -3.0);
        let inv = m.inverse().unwrap();
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        let back = inv.transform_point(transformed);
        assert!((back.x - pt.x).abs() < 1e-5);
        assert!((back.y - pt.y).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_scale() {
        let m = AffineMatrix::scale(2.0, 3.0);
        let inv = m.inverse().unwrap();
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        let back = inv.transform_point(transformed);
        assert!((back.x - pt.x).abs() < 1e-5);
        assert!((back.y - pt.y).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_rotation() {
        let m = AffineMatrix::rotation(5.0, 5.0, 0.5);
        let inv = m.inverse().unwrap();
        let pt = Point::new(10.0, 20.0);
        let transformed = m.transform_point(pt);
        let back = inv.transform_point(transformed);
        assert!((back.x - pt.x).abs() < 1e-4);
        assert!((back.y - pt.y).abs() < 1e-4);
    }

    #[test]
    fn test_singular_matrix() {
        // A matrix with zero determinant
        let m = AffineMatrix::from_coeffs([1.0, 2.0, 0.0, 2.0, 4.0, 0.0]);
        let result = m.inverse();
        assert!(matches!(result, Err(TransformError::SingularMatrix)));
    }

    // ========================================================================
    // Composition tests
    // ========================================================================

    #[test]
    fn test_compose_identity() {
        let m = AffineMatrix::translation(5.0, 3.0);
        let id = AffineMatrix::identity();
        let composed = m.compose(&id);
        let pt = Point::new(10.0, 20.0);
        let t1 = m.transform_point(pt);
        let t2 = composed.transform_point(pt);
        assert!((t1.x - t2.x).abs() < 1e-5);
        assert!((t1.y - t2.y).abs() < 1e-5);
    }

    #[test]
    fn test_compose_translations() {
        let m1 = AffineMatrix::translation(5.0, 3.0);
        let m2 = AffineMatrix::translation(2.0, -1.0);
        let composed = m1.compose(&m2);
        // Should be equivalent to translation(7, 2)
        let pt = Point::new(0.0, 0.0);
        let transformed = composed.transform_point(pt);
        assert!((transformed.x - 7.0).abs() < 1e-5);
        assert!((transformed.y - 2.0).abs() < 1e-5);
    }

    // ========================================================================
    // Three-point correspondence tests
    // ========================================================================

    #[test]
    fn test_from_three_points_identity() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];
        let dst = src; // Same points = identity

        let m = AffineMatrix::from_three_points(src, dst).unwrap();
        let pt = Point::new(5.0, 7.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - pt.x).abs() < 1e-4);
        assert!((transformed.y - pt.y).abs() < 1e-4);
    }

    #[test]
    fn test_from_three_points_translation() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
        ];
        let dst = [
            Point::new(5.0, 3.0),
            Point::new(15.0, 3.0),
            Point::new(5.0, 13.0),
        ];

        let m = AffineMatrix::from_three_points(src, dst).unwrap();
        let pt = Point::new(0.0, 0.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - 5.0).abs() < 1e-4);
        assert!((transformed.y - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_from_three_points_scale() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
        ];
        let dst = [
            Point::new(0.0, 0.0),
            Point::new(20.0, 0.0),
            Point::new(0.0, 30.0),
        ];

        let m = AffineMatrix::from_three_points(src, dst).unwrap();
        let pt = Point::new(5.0, 5.0);
        let transformed = m.transform_point(pt);
        assert!((transformed.x - 10.0).abs() < 1e-4);
        assert!((transformed.y - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_collinear_points() {
        let src = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0), // Collinear!
        ];
        let dst = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];

        let result = AffineMatrix::from_three_points(src, dst);
        assert!(matches!(result, Err(TransformError::SingularMatrix)));
    }

    // ========================================================================
    // AffineFill tests
    // ========================================================================

    #[test]
    fn test_affine_fill_values() {
        assert_eq!(AffineFill::White.to_value(PixelDepth::Bit1), 0);
        assert_eq!(AffineFill::Black.to_value(PixelDepth::Bit1), 1);
        assert_eq!(AffineFill::White.to_value(PixelDepth::Bit8), 255);
        assert_eq!(AffineFill::Black.to_value(PixelDepth::Bit8), 0);
        assert_eq!(AffineFill::Color(128).to_value(PixelDepth::Bit8), 128);
    }

    // ========================================================================
    // Image transformation tests
    // ========================================================================

    #[test]
    fn test_affine_sampled_identity() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel_unchecked(x, y, x + y * 10);
            }
        }
        let pix: Pix = pix_mut.into();

        let m = AffineMatrix::identity();
        let result = affine_sampled(&pix, &m, AffineFill::White).unwrap();

        // Check that all pixels are preserved
        for y in 0..10 {
            for x in 0..10 {
                let orig = pix.get_pixel_unchecked(x, y);
                let trans = result.get_pixel_unchecked(x, y);
                assert_eq!(orig, trans);
            }
        }
    }

    #[test]
    fn test_affine_sampled_translation() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set a marker pixel at (5, 5)
        pix_mut.set_pixel_unchecked(5, 5, 100);
        let pix: Pix = pix_mut.into();

        // Translate by (3, 2)
        let m = AffineMatrix::translation(3.0, 2.0);
        let result = affine_sampled(&pix, &m, AffineFill::White).unwrap();

        // Marker should now be at (8, 7)
        assert_eq!(result.get_pixel_unchecked(8, 7), 100);
    }

    #[test]
    fn test_affine_interpolated_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let m = AffineMatrix::identity();
        let result = affine(&pix, &m, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_interpolated_32bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let m = AffineMatrix::identity();
        let result = affine(&pix, &m, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_1bpp_falls_back_to_sampling() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let m = AffineMatrix::identity();
        let result = affine(&pix, &m, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_function() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = translate(&pix, 5.0, 3.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_scale_function() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = affine_scale(&pix, 1.5, 1.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_scale_invalid() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = affine_scale(&pix, 0.0, 1.0);
        assert!(matches!(result, Err(TransformError::InvalidParameters(_))));
    }

    #[test]
    fn test_affine_rotate_function() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = affine_rotate(&pix, 10.0, 10.0, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_sampled_pta() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
        ];
        let dst = [
            Point::new(5.0, 5.0),
            Point::new(44.0, 5.0),
            Point::new(5.0, 44.0),
        ];
        let result = affine_sampled_pta(&pix, src, dst, AffineFill::White);
        assert!(result.is_ok());
    }

    #[test]
    fn test_affine_pta() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
        ];
        let dst = [
            Point::new(5.0, 5.0),
            Point::new(44.0, 5.0),
            Point::new(5.0, 44.0),
        ];
        let result = affine_pta(&pix, src, dst, AffineFill::White);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Color preservation tests
    // ========================================================================

    // ========================================================================
    // WithAlpha tests
    // ========================================================================

    #[test]
    fn test_affine_pta_with_alpha_basic() {
        // 32bpp RGBA image with known content
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

        // Identity-like transform (small translation)
        let src = [
            Point::new(0.0, 0.0),
            Point::new(49.0, 0.0),
            Point::new(0.0, 49.0),
        ];
        let dst = [
            Point::new(2.0, 2.0),
            Point::new(47.0, 2.0),
            Point::new(2.0, 47.0),
        ];

        let result = affine_pta_with_alpha(&pix, src, dst, None, 1.0, 10).unwrap();

        // Result should be 32bpp with spp=4
        assert_eq!(result.depth(), PixelDepth::Bit32);
        assert_eq!(result.spp(), 4);

        // Result should be larger than input due to border
        assert_eq!(result.width(), 70); // 50 + 2*10
        assert_eq!(result.height(), 70);

        // Border pixels should have alpha = 0 (transparent)
        assert_eq!(color::alpha(result.get_pixel_unchecked(0, 0)), 0);

        // Interior pixels should have non-zero alpha
        let center_pixel = result.get_pixel_unchecked(35, 35);
        assert!(color::alpha(center_pixel) > 0);
    }

    #[test]
    fn test_affine_pta_with_alpha_opacity() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(100, 150, 200));
            }
        }
        let pix: Pix = pm.into();

        let src = [
            Point::new(0.0, 0.0),
            Point::new(29.0, 0.0),
            Point::new(0.0, 29.0),
        ];
        let dst = src; // Identity

        // 50% opacity
        let result = affine_pta_with_alpha(&pix, src, dst, None, 0.5, 5).unwrap();
        assert_eq!(result.spp(), 4);

        // Interior pixel alpha should be approximately 127 (255 * 0.5)
        // Account for edge feathering by checking a pixel well inside
        let interior = result.get_pixel_unchecked(20, 20);
        let alpha = color::alpha(interior);
        assert!(
            (alpha as i32 - 127).abs() <= 2,
            "Expected alpha ~127, got {}",
            alpha
        );
    }

    #[test]
    fn test_affine_pta_with_alpha_custom_mask() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..30u32 {
                pm.set_pixel_unchecked(x, y, color::compose_rgb(100, 150, 200));
            }
        }
        let pix: Pix = pm.into();

        // Create custom alpha mask: left half opaque, right half transparent
        let mask = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        for y in 0..30u32 {
            for x in 0..15u32 {
                mm.set_pixel_unchecked(x, y, 255);
            }
            for x in 15..30u32 {
                mm.set_pixel_unchecked(x, y, 0);
            }
        }
        let mask: Pix = mm.into();

        let src = [
            Point::new(0.0, 0.0),
            Point::new(29.0, 0.0),
            Point::new(0.0, 29.0),
        ];
        let dst = src; // Identity

        let result = affine_pta_with_alpha(&pix, src, dst, Some(&mask), 1.0, 5).unwrap();
        assert_eq!(result.spp(), 4);

        // Output is 40x40 (30x30 + 5px border on each side).
        // Identity transform: source pixel (sx, sy) -> output pixel (sx+5, sy+5).
        // Mask: left half (sx < 15) = 255, right half (sx >= 15) = 0.
        // Sample away from the border at output row y=15 (source row 10).
        // Output (7, 15) -> source (2, 10): left half, alpha should be 255.
        // Output (27, 15) -> source (22, 10): right half, alpha should be 0.
        let opaque_pixel = result.get_pixel(7, 15).unwrap();
        let transparent_pixel = result.get_pixel(27, 15).unwrap();
        // Alpha is in the lowest byte (ALPHA_SHIFT=0, format 0xRRGGBBAA)
        let opaque_alpha = (opaque_pixel & 0xff) as u8;
        let transparent_alpha = (transparent_pixel & 0xff) as u8;
        assert!(
            opaque_alpha > 200,
            "expected high alpha in masked-opaque region, got {opaque_alpha}"
        );
        assert!(
            transparent_alpha < 50,
            "expected low alpha in masked-transparent region, got {transparent_alpha}"
        );
    }

    #[test]
    fn test_affine_pta_with_alpha_invalid_depth() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let src = [
            Point::new(0.0, 0.0),
            Point::new(19.0, 0.0),
            Point::new(0.0, 19.0),
        ];
        let result = affine_pta_with_alpha(&pix, src, src, None, 1.0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_affine_preserves_colormap() {
        use leptonica_core::PixColormap;

        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a simple colormap
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // Red
        cmap.add_rgb(0, 255, 0).unwrap(); // Green
        let _ = pix_mut.set_colormap(Some(cmap));

        let pix: Pix = pix_mut.into();

        let m = AffineMatrix::identity();
        let result = affine_sampled(&pix, &m, AffineFill::White).unwrap();

        assert!(result.colormap().is_some());
    }

    // -- pta_affine_transform --

    #[test]
    fn test_pta_affine_transform_identity() {
        use leptonica_core::Pta;
        let mut pta = Pta::new();
        pta.push(1.0, 2.0);
        pta.push(3.0, 4.0);

        let matrix = AffineMatrix::identity();
        let result = pta_affine_transform(&pta, &matrix);
        assert_eq!(result.len(), 2);
        assert!((result.get(0).unwrap().0 - 1.0).abs() < 1e-5);
        assert!((result.get(0).unwrap().1 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_pta_affine_transform_translation() {
        use leptonica_core::Pta;
        let mut pta = Pta::new();
        pta.push(1.0, 2.0);

        let matrix = AffineMatrix::translation(10.0, 20.0);
        let result = pta_affine_transform(&pta, &matrix);
        assert!((result.get(0).unwrap().0 - 11.0).abs() < 1e-5);
        assert!((result.get(0).unwrap().1 - 22.0).abs() < 1e-5);
    }

    // -- boxa_affine_transform --

    #[test]
    fn test_boxa_affine_transform_identity() {
        use leptonica_core::{Box as LBox, Boxa};
        let mut boxa = Boxa::new();
        boxa.push(LBox::new(10, 20, 30, 40).unwrap());

        let matrix = AffineMatrix::identity();
        let result = boxa_affine_transform(&boxa, &matrix);
        assert_eq!(result.len(), 1);
        let b = result.get(0).unwrap();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.w, 30);
        assert_eq!(b.h, 40);
    }

    #[test]
    fn test_boxa_affine_transform_translation() {
        use leptonica_core::{Box as LBox, Boxa};
        let mut boxa = Boxa::new();
        boxa.push(LBox::new(0, 0, 10, 10).unwrap());

        let matrix = AffineMatrix::translation(5.0, 3.0);
        let result = boxa_affine_transform(&boxa, &matrix);
        let b = result.get(0).unwrap();
        assert_eq!(b.x, 5);
        assert_eq!(b.y, 3);
        assert_eq!(b.w, 10);
        assert_eq!(b.h, 10);
    }
}
