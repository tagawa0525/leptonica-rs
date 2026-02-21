//! Rotation and flip operations
//!
//! This module provides:
//! - Orthogonal rotations (90/180/270 degrees)
//! - Arbitrary angle rotations (with multiple algorithms)
//! - Horizontal and vertical flips
//!
//! # Rotation Methods
//!
//! - **Sampling**: Fastest, uses nearest-neighbor interpolation. Best for speed.
//! - **AreaMap**: Highest quality, uses area-weighted averaging. Best for quality.
//! - **Shear**: Good for 1bpp images, uses 2 or 3 shear operations.
//! - **Bilinear**: Good balance of speed and quality.

use crate::shear::{ShearFill, h_shear_ip, v_shear_ip};
use crate::{TransformError, TransformResult};
use leptonica_core::{Pix, PixMut, PixelDepth, color};

// ============================================================================
// Constants from Leptonica
// ============================================================================

/// Minimum angle to actually rotate (below this, just clone)
const MIN_ANGLE_TO_ROTATE: f32 = 0.001; // radians, ~0.06 degrees
/// Maximum angle for 2-shear rotation
const MAX_TWO_SHEAR_ANGLE: f32 = 0.06; // radians, ~3 degrees
/// Maximum angle for 3-shear rotation (warning threshold)
#[allow(dead_code)]
const MAX_THREE_SHEAR_ANGLE: f32 = 0.35; // radians, ~20 degrees
/// Maximum angle for shear rotation
const MAX_SHEAR_ANGLE: f32 = 0.50; // radians, ~29 degrees
/// Angle threshold for switching 1bpp from shear to sampling
const MAX_1BPP_SHEAR_ANGLE: f32 = 0.06; // radians, ~3 degrees

// ============================================================================
// Rotation method enumeration
// ============================================================================

/// Rotation algorithm to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotateMethod {
    /// Sampling (nearest-neighbor) - fastest, lowest quality
    Sampling,
    /// Area mapping - highest quality, uses 16x16 sub-pixel grid
    AreaMap,
    /// Shear-based rotation - good for 1bpp images
    Shear,
    /// Bilinear interpolation - good balance of speed and quality
    Bilinear,
    /// Automatic selection based on depth and angle
    #[default]
    Auto,
}

/// Background fill color for rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotateFill {
    /// Fill with white pixels
    #[default]
    White,
    /// Fill with black pixels
    Black,
    /// Fill with a specific color value (interpretation depends on depth)
    Color(u32),
}

impl RotateFill {
    /// Get the fill value for a specific pixel depth
    pub fn to_value(self, depth: PixelDepth) -> u32 {
        match self {
            RotateFill::White => match depth {
                PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
                PixelDepth::Bit2 => 3,
                PixelDepth::Bit4 => 15,
                PixelDepth::Bit8 => 255,
                PixelDepth::Bit16 => 65535,
                PixelDepth::Bit32 => 0xFFFFFF00,
            },
            RotateFill::Black => match depth {
                PixelDepth::Bit1 => 1, // 1 = black for binary
                PixelDepth::Bit32 => 0x00000000,
                _ => 0,
            },
            RotateFill::Color(val) => val,
        }
    }
}

/// Options for rotation operations
#[derive(Debug, Clone)]
pub struct RotateOptions {
    /// Rotation algorithm to use
    pub method: RotateMethod,
    /// Background fill color
    pub fill: RotateFill,
    /// Custom rotation center X (None = image center)
    pub center_x: Option<f32>,
    /// Custom rotation center Y (None = image center)
    pub center_y: Option<f32>,
    /// Expand output to fit all rotated pixels
    pub expand: bool,
}

impl Default for RotateOptions {
    fn default() -> Self {
        Self {
            method: RotateMethod::Auto,
            fill: RotateFill::White,
            center_x: None,
            center_y: None,
            expand: true,
        }
    }
}

impl RotateOptions {
    /// Create options with a specific method
    pub fn with_method(method: RotateMethod) -> Self {
        Self {
            method,
            ..Default::default()
        }
    }

    /// Create options with a specific fill color
    pub fn with_fill(fill: RotateFill) -> Self {
        Self {
            fill,
            ..Default::default()
        }
    }

    /// Set the rotation center
    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center_x = Some(x);
        self.center_y = Some(y);
        self
    }

    /// Set whether to expand output dimensions
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }
}

/// Rotate an image by 90-degree increments
///
/// # Arguments
/// * `pix` - Input image
/// * `quads` - Number of 90-degree clockwise rotations (0-3)
///
/// # Returns
/// The rotated image
pub fn rotate_orth(pix: &Pix, quads: u32) -> TransformResult<Pix> {
    match quads % 4 {
        0 => Ok(pix.deep_clone()),
        1 => rotate_90(pix, true),
        2 => rotate_180(pix),
        3 => rotate_90(pix, false),
        _ => unreachable!(),
    }
}

/// Rotate an image 90 degrees
///
/// # Arguments
/// * `pix` - Input image
/// * `clockwise` - If true, rotate clockwise; otherwise counterclockwise
pub fn rotate_90(pix: &Pix, clockwise: bool) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    // Output dimensions are swapped
    let out_pix = Pix::new(h, w, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    rotate_90_impl(pix, &mut out_mut, clockwise, w, h);

    Ok(out_mut.into())
}

/// Internal implementation of 90 degree rotation
fn rotate_90_impl(src: &Pix, dst: &mut PixMut, clockwise: bool, w: u32, h: u32) {
    for y in 0..h {
        for x in 0..w {
            let val = src.get_pixel_unchecked(x, y);
            let (nx, ny) = if clockwise {
                (h - 1 - y, x)
            } else {
                (y, w - 1 - x)
            };
            dst.set_pixel_unchecked(nx, ny, val);
        }
    }
}

/// Rotate an image 180 degrees
pub fn rotate_180(pix: &Pix) -> TransformResult<Pix> {
    // 180 rotation = horizontal flip + vertical flip
    let flipped_h = flip_lr(pix)?;
    flip_tb(&flipped_h)
}

/// Flip an image left-right (horizontal mirror)
pub fn flip_lr(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let nx = w - 1 - x;
            out_mut.set_pixel_unchecked(nx, y, val);
        }
    }

    Ok(out_mut.into())
}

/// Flip an image top-bottom (vertical mirror)
pub fn flip_tb(pix: &Pix) -> TransformResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let depth = pix.depth();

    let out_pix = Pix::new(w, h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let ny = h - 1 - y;
            out_mut.set_pixel_unchecked(x, ny, val);
        }
    }

    Ok(out_mut.into())
}

/// Rotate an image in-place by 180 degrees
pub fn rotate_180_in_place(pix: &mut PixMut) -> TransformResult<()> {
    let w = pix.width();
    let h = pix.height();

    // Swap pixels from opposite corners
    let total_pixels = (w as u64) * (h as u64);
    let half = total_pixels / 2;

    for i in 0..half {
        let x1 = (i % (w as u64)) as u32;
        let y1 = (i / (w as u64)) as u32;
        let x2 = w - 1 - x1;
        let y2 = h - 1 - y1;

        let val1 = pix.get_pixel_unchecked(x1, y1);
        let val2 = pix.get_pixel_unchecked(x2, y2);

        pix.set_pixel_unchecked(x1, y1, val2);
        pix.set_pixel_unchecked(x2, y2, val1);
    }

    Ok(())
}

// ============================================================================
// Arbitrary angle rotation
// ============================================================================

/// Rotate an image by an arbitrary angle in degrees
///
/// Uses bilinear interpolation for smooth results. The output image size
/// is adjusted to contain the entire rotated image.
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in degrees (positive = counterclockwise)
///
/// # Returns
/// The rotated image with white background
///
/// # Example
/// ```no_run
/// use leptonica_transform::rotate_by_angle;
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let rotated = rotate_by_angle(&pix, 45.0).unwrap();
/// ```
pub fn rotate_by_angle(pix: &Pix, angle: f32) -> TransformResult<Pix> {
    let fill_value = get_default_fill_value(pix.depth());
    rotate_by_angle_with_options(pix, angle, fill_value)
}

/// Rotate an image by an arbitrary angle in radians
///
/// Uses bilinear interpolation for smooth results.
///
/// # Arguments
/// * `pix` - Input image
/// * `radians` - Rotation angle in radians (positive = counterclockwise)
pub fn rotate_by_radians(pix: &Pix, radians: f32) -> TransformResult<Pix> {
    let angle = radians.to_degrees();
    rotate_by_angle(pix, angle)
}

/// Rotate an image by an arbitrary angle with custom fill value
///
/// Uses bilinear interpolation for 8-bit and 32-bit images,
/// nearest neighbor for 1-bit images.
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in degrees (positive = counterclockwise)
/// * `fill_value` - Value to fill pixels outside the original image
pub fn rotate_by_angle_with_options(
    pix: &Pix,
    angle: f32,
    fill_value: u32,
) -> TransformResult<Pix> {
    // Normalize angle to [0, 360)
    let angle = angle % 360.0;
    let angle = if angle < 0.0 { angle + 360.0 } else { angle };

    // Handle special cases for orthogonal rotations
    if (angle - 0.0).abs() < 0.001 || (angle - 360.0).abs() < 0.001 {
        return Ok(pix.deep_clone());
    }
    if (angle - 90.0).abs() < 0.001 {
        return rotate_90(pix, false); // counterclockwise
    }
    if (angle - 180.0).abs() < 0.001 {
        return rotate_180(pix);
    }
    if (angle - 270.0).abs() < 0.001 {
        return rotate_90(pix, true); // clockwise = 270 CCW
    }

    // General rotation
    let radians = angle.to_radians();
    let cos_a = radians.cos();
    let sin_a = radians.sin();

    let w = pix.width() as f32;
    let h = pix.height() as f32;

    // Calculate new dimensions to fit the rotated image
    let (new_w, new_h) = calculate_rotated_bounds(w, h, cos_a, sin_a);
    let new_w = new_w as u32;
    let new_h = new_h as u32;

    let out_pix = Pix::new(new_w, new_h, pix.depth())?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    // Center of original and new images
    let cx_src = w / 2.0;
    let cy_src = h / 2.0;
    let cx_dst = new_w as f32 / 2.0;
    let cy_dst = new_h as f32 / 2.0;

    // Apply rotation with appropriate interpolation
    match pix.depth() {
        PixelDepth::Bit1 => {
            rotate_nearest_neighbor(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
        PixelDepth::Bit8 | PixelDepth::Bit16 | PixelDepth::Bit32 => {
            rotate_bilinear(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
        _ => {
            // For 2-bit and 4-bit, use nearest neighbor
            rotate_nearest_neighbor(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
    }

    Ok(out_mut.into())
}

/// Get default fill value (white) for a given pixel depth
fn get_default_fill_value(depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => 0, // 0 = white for binary (foreground is black)
        PixelDepth::Bit2 => 3,
        PixelDepth::Bit4 => 15,
        PixelDepth::Bit8 => 255,
        PixelDepth::Bit16 => 65535,
        PixelDepth::Bit32 => 0xFFFFFFFF,
    }
}

/// Calculate the bounding box dimensions after rotation
fn calculate_rotated_bounds(w: f32, h: f32, cos_a: f32, sin_a: f32) -> (f32, f32) {
    // Corners of original image (relative to center)
    let corners = [
        (-w / 2.0, -h / 2.0),
        (w / 2.0, -h / 2.0),
        (w / 2.0, h / 2.0),
        (-w / 2.0, h / 2.0),
    ];

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for (x, y) in corners {
        let rx = x * cos_a - y * sin_a;
        let ry = x * sin_a + y * cos_a;
        min_x = min_x.min(rx);
        max_x = max_x.max(rx);
        min_y = min_y.min(ry);
        max_y = max_y.max(ry);
    }

    ((max_x - min_x).ceil(), (max_y - min_y).ceil())
}

/// Fill an image with a constant value
fn fill_image(pix: &mut PixMut, value: u32) {
    let w = pix.width();
    let h = pix.height();
    for y in 0..h {
        for x in 0..w {
            pix.set_pixel_unchecked(x, y, value);
        }
    }
}

/// Nearest neighbor rotation (for 1-bit images)
#[allow(clippy::too_many_arguments)]
fn rotate_nearest_neighbor(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            // Transform destination coordinates to source
            let x_rel = dx as f32 - cx_dst;
            let y_rel = dy as f32 - cy_dst;

            // Inverse rotation
            let sx = x_rel * cos_a + y_rel * sin_a + cx_src;
            let sy = -x_rel * sin_a + y_rel * cos_a + cy_src;

            // Nearest neighbor
            let sx_i = sx.round() as i32;
            let sy_i = sy.round() as i32;

            if sx_i >= 0 && sx_i < src_w && sy_i >= 0 && sy_i < src_h {
                let val = src.get_pixel_unchecked(sx_i as u32, sy_i as u32);
                dst.set_pixel_unchecked(dx, dy, val);
            }
        }
    }
}

/// Bilinear interpolation rotation (for 8-bit and 32-bit images)
#[allow(clippy::too_many_arguments)]
fn rotate_bilinear(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();
    let depth = src.depth();

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            // Transform destination coordinates to source
            let x_rel = dx as f32 - cx_dst;
            let y_rel = dy as f32 - cy_dst;

            // Inverse rotation
            let sx = x_rel * cos_a + y_rel * sin_a + cx_src;
            let sy = -x_rel * sin_a + y_rel * cos_a + cy_src;

            // Bilinear interpolation
            let x0 = sx.floor() as i32;
            let y0 = sy.floor() as i32;
            let x1 = x0 + 1;
            let y1 = y0 + 1;

            if x0 >= 0 && x1 < src_w && y0 >= 0 && y1 < src_h {
                let fx = sx - x0 as f32;
                let fy = sy - y0 as f32;

                let val = interpolate_pixel(
                    src, depth, x0 as u32, y0 as u32, x1 as u32, y1 as u32, fx, fy,
                );
                dst.set_pixel_unchecked(dx, dy, val);
            } else if x0 >= -1 && x1 <= src_w && y0 >= -1 && y1 <= src_h {
                // Edge case: partially outside, use available pixels
                let val = interpolate_edge_pixel(src, depth, x0, y0, x1, y1, sx, sy, src_w, src_h);
                dst.set_pixel_unchecked(dx, dy, val);
            }
        }
    }
}

/// Bilinear interpolation of a single pixel
#[allow(clippy::too_many_arguments)]
fn interpolate_pixel(
    src: &Pix,
    depth: PixelDepth,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    fx: f32,
    fy: f32,
) -> u32 {
    let p00 = src.get_pixel_unchecked(x0, y0);
    let p10 = src.get_pixel_unchecked(x1, y0);
    let p01 = src.get_pixel_unchecked(x0, y1);
    let p11 = src.get_pixel_unchecked(x1, y1);

    match depth {
        PixelDepth::Bit32 => {
            // Interpolate each channel separately
            let r = interpolate_channel(
                (p00 >> 24) & 0xFF,
                (p10 >> 24) & 0xFF,
                (p01 >> 24) & 0xFF,
                (p11 >> 24) & 0xFF,
                fx,
                fy,
            );
            let g = interpolate_channel(
                (p00 >> 16) & 0xFF,
                (p10 >> 16) & 0xFF,
                (p01 >> 16) & 0xFF,
                (p11 >> 16) & 0xFF,
                fx,
                fy,
            );
            let b = interpolate_channel(
                (p00 >> 8) & 0xFF,
                (p10 >> 8) & 0xFF,
                (p01 >> 8) & 0xFF,
                (p11 >> 8) & 0xFF,
                fx,
                fy,
            );
            let a = interpolate_channel(p00 & 0xFF, p10 & 0xFF, p01 & 0xFF, p11 & 0xFF, fx, fy);
            (r << 24) | (g << 16) | (b << 8) | a
        }
        _ => {
            // Single channel interpolation
            interpolate_channel(p00, p10, p01, p11, fx, fy)
        }
    }
}

/// Interpolate a single channel value
fn interpolate_channel(p00: u32, p10: u32, p01: u32, p11: u32, fx: f32, fy: f32) -> u32 {
    let top = p00 as f32 * (1.0 - fx) + p10 as f32 * fx;
    let bottom = p01 as f32 * (1.0 - fx) + p11 as f32 * fx;
    let result = top * (1.0 - fy) + bottom * fy;
    result.round() as u32
}

/// Handle edge pixels with partial interpolation
#[allow(clippy::too_many_arguments)]
fn interpolate_edge_pixel(
    src: &Pix,
    depth: PixelDepth,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    sx: f32,
    sy: f32,
    src_w: i32,
    src_h: i32,
) -> u32 {
    // Get available pixels, using nearest valid pixel for out-of-bounds
    let clamp_x = |x: i32| x.clamp(0, src_w - 1) as u32;
    let clamp_y = |y: i32| y.clamp(0, src_h - 1) as u32;

    let p00 = src.get_pixel_unchecked(clamp_x(x0), clamp_y(y0));
    let p10 = src.get_pixel_unchecked(clamp_x(x1), clamp_y(y0));
    let p01 = src.get_pixel_unchecked(clamp_x(x0), clamp_y(y1));
    let p11 = src.get_pixel_unchecked(clamp_x(x1), clamp_y(y1));

    let fx = sx - x0 as f32;
    let fy = sy - y0 as f32;

    match depth {
        PixelDepth::Bit32 => {
            let r = interpolate_channel(
                (p00 >> 24) & 0xFF,
                (p10 >> 24) & 0xFF,
                (p01 >> 24) & 0xFF,
                (p11 >> 24) & 0xFF,
                fx,
                fy,
            );
            let g = interpolate_channel(
                (p00 >> 16) & 0xFF,
                (p10 >> 16) & 0xFF,
                (p01 >> 16) & 0xFF,
                (p11 >> 16) & 0xFF,
                fx,
                fy,
            );
            let b = interpolate_channel(
                (p00 >> 8) & 0xFF,
                (p10 >> 8) & 0xFF,
                (p01 >> 8) & 0xFF,
                (p11 >> 8) & 0xFF,
                fx,
                fy,
            );
            let a = interpolate_channel(p00 & 0xFF, p10 & 0xFF, p01 & 0xFF, p11 & 0xFF, fx, fy);
            (r << 24) | (g << 16) | (b << 8) | a
        }
        _ => interpolate_channel(p00, p10, p01, p11, fx, fy),
    }
}

// ============================================================================
// New rotation API with multiple algorithms
// ============================================================================

/// Rotate an image by an arbitrary angle using specified options
///
/// This is the most flexible rotation function, supporting multiple algorithms
/// and custom rotation centers.
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in radians (positive = clockwise, like Leptonica)
/// * `options` - Rotation options including method, fill color, and center
///
/// # Example
/// ```no_run
/// use leptonica_transform::{rotate, RotateOptions, RotateMethod, RotateFill};
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let options = RotateOptions::with_method(RotateMethod::AreaMap);
/// let rotated = rotate(&pix, 0.5, &options).unwrap();
/// ```
pub fn rotate(pix: &Pix, angle: f32, options: &RotateOptions) -> TransformResult<Pix> {
    // For very small angles, just clone
    if angle.abs() < MIN_ANGLE_TO_ROTATE {
        return Ok(pix.deep_clone());
    }

    let depth = pix.depth();
    let fill_value = options.fill.to_value(depth);

    // Select method (auto-select based on depth and angle if needed)
    let method = select_rotate_method(options.method, depth, angle);

    // Calculate dimensions and centers
    let w = pix.width() as f32;
    let h = pix.height() as f32;
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let (out_w, out_h) = if options.expand {
        let (nw, nh) = calculate_rotated_bounds(w, h, cos_a, sin_a);
        (nw as u32, nh as u32)
    } else {
        (pix.width(), pix.height())
    };

    let cx_src = options.center_x.unwrap_or(w / 2.0);
    let cy_src = options.center_y.unwrap_or(h / 2.0);
    let cx_dst = if options.expand {
        out_w as f32 / 2.0
    } else {
        cx_src
    };
    let cy_dst = if options.expand {
        out_h as f32 / 2.0
    } else {
        cy_src
    };

    // Create output image
    let out_pix = Pix::new(out_w, out_h, depth)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    // Copy colormap if present
    if let Some(cmap) = pix.colormap() {
        let _ = out_mut.set_colormap(Some(cmap.clone()));
    }

    // Fill with background
    fill_image(&mut out_mut, fill_value);

    // Perform rotation with selected method
    match method {
        RotateMethod::Sampling => {
            rotate_by_sampling_impl(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
                fill_value,
            );
        }
        RotateMethod::AreaMap => {
            rotate_area_map_impl(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
                fill_value,
            );
        }
        RotateMethod::Shear => {
            rotate_shear_impl(
                pix,
                &mut out_mut,
                angle,
                cx_src as i32,
                cy_src as i32,
                fill_value,
            );
        }
        RotateMethod::Bilinear => {
            rotate_bilinear(
                pix,
                &mut out_mut,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
            );
        }
        RotateMethod::Auto => unreachable!(),
    }

    Ok(out_mut.into())
}

/// Rotate an image using a specific method (simple API)
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `method` - Rotation algorithm to use
pub fn rotate_with_method(pix: &Pix, angle: f32, method: RotateMethod) -> TransformResult<Pix> {
    let options = RotateOptions {
        method,
        ..Default::default()
    };
    rotate(pix, angle, &options)
}

/// Rotate an image about a specified center point
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `center_x` - X coordinate of rotation center
/// * `center_y` - Y coordinate of rotation center
/// * `fill` - Background fill color
pub fn rotate_about_center(
    pix: &Pix,
    angle: f32,
    center_x: f32,
    center_y: f32,
    fill: RotateFill,
) -> TransformResult<Pix> {
    let options = RotateOptions {
        fill,
        center_x: Some(center_x),
        center_y: Some(center_y),
        expand: false, // Don't expand when using custom center
        ..Default::default()
    };
    rotate(pix, angle, &options)
}

/// Select the best rotation method based on depth and angle
fn select_rotate_method(method: RotateMethod, depth: PixelDepth, angle: f32) -> RotateMethod {
    match method {
        RotateMethod::Auto => {
            let abs_angle = angle.abs();

            // For 1bpp, use shear for small angles, sampling for large
            if depth == PixelDepth::Bit1 {
                if abs_angle > MAX_1BPP_SHEAR_ANGLE {
                    RotateMethod::Sampling
                } else {
                    RotateMethod::Shear
                }
            }
            // For other depths, use area mapping for best quality
            // unless angle is very large
            else if abs_angle > MAX_SHEAR_ANGLE {
                RotateMethod::Sampling
            } else {
                match depth {
                    PixelDepth::Bit8 | PixelDepth::Bit32 => RotateMethod::AreaMap,
                    _ => RotateMethod::Sampling,
                }
            }
        }
        // Validate requested method is appropriate
        RotateMethod::AreaMap => {
            if depth == PixelDepth::Bit1
                || depth == PixelDepth::Bit2
                || depth == PixelDepth::Bit4
                || depth == PixelDepth::Bit16
            {
                // Fall back to sampling for unsupported depths
                RotateMethod::Sampling
            } else {
                RotateMethod::AreaMap
            }
        }
        RotateMethod::Shear => {
            if angle.abs() > MAX_SHEAR_ANGLE {
                // Angle too large for shear
                RotateMethod::Sampling
            } else {
                RotateMethod::Shear
            }
        }
        other => other,
    }
}

// ============================================================================
// Sampling rotation implementation (pixRotateBySampling equivalent)
// ============================================================================

/// Rotation by sampling (nearest neighbor) - like pixRotateBySampling
#[allow(clippy::too_many_arguments)]
fn rotate_by_sampling_impl(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
    _fill_value: u32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();
    let wm1 = src_w - 1;
    let hm1 = src_h - 1;

    for i in 0..dst_h {
        let ydif = cy_dst - i as f32;
        for j in 0..dst_w {
            let xdif = cx_dst - j as f32;

            // Inverse rotation (Leptonica convention: clockwise positive)
            let x = (cx_src + (-xdif * cos_a - ydif * sin_a)).round() as i32;
            let y = (cy_src + (-ydif * cos_a + xdif * sin_a)).round() as i32;

            if x >= 0 && x <= wm1 && y >= 0 && y <= hm1 {
                let val = src.get_pixel_unchecked(x as u32, y as u32);
                dst.set_pixel_unchecked(j, i, val);
            }
            // Pixels outside bounds keep the fill value set earlier
        }
    }
}

// ============================================================================
// Area mapping rotation implementation (pixRotateAM equivalent)
// ============================================================================

/// Rotation by area mapping - like pixRotateAMGray/pixRotateAMColor
///
/// Uses 16x16 sub-pixel grid for high-quality interpolation
#[allow(clippy::too_many_arguments)]
fn rotate_area_map_impl(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
    fill_value: u32,
) {
    let depth = src.depth();
    match depth {
        PixelDepth::Bit8 => {
            rotate_area_map_gray(
                src,
                dst,
                cos_a,
                sin_a,
                cx_src,
                cy_src,
                cx_dst,
                cy_dst,
                fill_value as u8,
            );
        }
        PixelDepth::Bit32 => {
            rotate_area_map_color(
                src, dst, cos_a, sin_a, cx_src, cy_src, cx_dst, cy_dst, fill_value,
            );
        }
        _ => {
            // Fall back to sampling for other depths
            rotate_by_sampling_impl(
                src, dst, cos_a, sin_a, cx_src, cy_src, cx_dst, cy_dst, fill_value,
            );
        }
    }
}

/// Area mapping rotation for 8bpp grayscale
#[allow(clippy::too_many_arguments)]
fn rotate_area_map_gray(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
    grayval: u8,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();
    let wm2 = src_w - 2;
    let hm2 = src_h - 2;

    // Scale sin/cos by 16 for sub-pixel precision
    let sina = 16.0 * sin_a;
    let cosa = 16.0 * cos_a;

    for i in 0..dst_h {
        let ydif = cy_dst - i as f32;
        for j in 0..dst_w {
            let xdif = cx_dst - j as f32;

            // Sub-pixel position (scaled by 16)
            let xpm = (-xdif * cosa - ydif * sina) as i32;
            let ypm = (-ydif * cosa + xdif * sina) as i32;

            // Integer and fractional parts
            let xp = (cx_src as i32) + (xpm >> 4);
            let yp = (cy_src as i32) + (ypm >> 4);
            let xf = xpm & 0x0f;
            let yf = ypm & 0x0f;

            // Check bounds
            if xp < 0 || yp < 0 || xp > wm2 || yp > hm2 {
                dst.set_pixel_unchecked(j, i, grayval as u32);
                continue;
            }

            // Get four neighboring pixels
            let v00 = src.get_pixel_unchecked(xp as u32, yp as u32);
            let v10 = src.get_pixel_unchecked((xp + 1) as u32, yp as u32);
            let v01 = src.get_pixel_unchecked(xp as u32, (yp + 1) as u32);
            let v11 = src.get_pixel_unchecked((xp + 1) as u32, (yp + 1) as u32);

            // Area-weighted interpolation
            let val = ((16 - xf) * (16 - yf) * v00 as i32
                + xf * (16 - yf) * v10 as i32
                + (16 - xf) * yf * v01 as i32
                + xf * yf * v11 as i32
                + 128)
                / 256;

            dst.set_pixel_unchecked(j, i, val as u32);
        }
    }
}

/// Area mapping rotation for 32bpp color
#[allow(clippy::too_many_arguments)]
fn rotate_area_map_color(
    src: &Pix,
    dst: &mut PixMut,
    cos_a: f32,
    sin_a: f32,
    cx_src: f32,
    cy_src: f32,
    cx_dst: f32,
    cy_dst: f32,
    colorval: u32,
) {
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_w = dst.width();
    let dst_h = dst.height();
    let wm2 = src_w - 2;
    let hm2 = src_h - 2;

    // Scale sin/cos by 16 for sub-pixel precision
    let sina = 16.0 * sin_a;
    let cosa = 16.0 * cos_a;

    for i in 0..dst_h {
        let ydif = cy_dst - i as f32;
        for j in 0..dst_w {
            let xdif = cx_dst - j as f32;

            // Sub-pixel position (scaled by 16)
            let xpm = (-xdif * cosa - ydif * sina) as i32;
            let ypm = (-ydif * cosa + xdif * sina) as i32;

            // Integer and fractional parts
            let xp = (cx_src as i32) + (xpm >> 4);
            let yp = (cy_src as i32) + (ypm >> 4);
            let xf = xpm & 0x0f;
            let yf = ypm & 0x0f;

            // Check bounds
            if xp < 0 || yp < 0 || xp > wm2 || yp > hm2 {
                dst.set_pixel_unchecked(j, i, colorval);
                continue;
            }

            // Get four neighboring pixels
            let word00 = src.get_pixel_unchecked(xp as u32, yp as u32);
            let word10 = src.get_pixel_unchecked((xp + 1) as u32, yp as u32);
            let word01 = src.get_pixel_unchecked(xp as u32, (yp + 1) as u32);
            let word11 = src.get_pixel_unchecked((xp + 1) as u32, (yp + 1) as u32);

            // Extract and interpolate each channel (RGBA format)
            let (r00, g00, b00, a00) = color::extract_rgba(word00);
            let (r10, g10, b10, a10) = color::extract_rgba(word10);
            let (r01, g01, b01, a01) = color::extract_rgba(word01);
            let (r11, g11, b11, a11) = color::extract_rgba(word11);

            let rval = area_interp(r00, r10, r01, r11, xf, yf);
            let gval = area_interp(g00, g10, g01, g11, xf, yf);
            let bval = area_interp(b00, b10, b01, b11, xf, yf);
            let aval = area_interp(a00, a10, a01, a11, xf, yf);

            let pixel = color::compose_rgba(rval, gval, bval, aval);
            dst.set_pixel_unchecked(j, i, pixel);
        }
    }
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
// Shear-based rotation implementation (pixRotateShear equivalent)
// ============================================================================

/// Rotation by shear - like pixRotateShear
///
/// Uses 2-shear for small angles (< ~3 degrees) or 3-shear for larger angles
#[allow(clippy::too_many_arguments)]
fn rotate_shear_impl(
    src: &Pix,
    dst: &mut PixMut,
    angle: f32,
    xcen: i32,
    ycen: i32,
    fill_value: u32,
) {
    let abs_angle = angle.abs();

    if abs_angle <= MAX_TWO_SHEAR_ANGLE {
        rotate_2_shear(src, dst, angle, xcen, ycen, fill_value);
    } else {
        rotate_3_shear(src, dst, angle, xcen, ycen, fill_value);
    }
}

/// 2-shear rotation (for small angles)
///
/// x' = x + tan(angle) * (y - ycen)
/// y' = y + tan(angle) * (x - xcen)
fn rotate_2_shear(src: &Pix, dst: &mut PixMut, angle: f32, xcen: i32, ycen: i32, fill_value: u32) {
    let w = src.width() as i32;
    let h = src.height() as i32;
    let tan_a = angle.tan();

    // First pass: horizontal shear into temporary
    let temp_pix = Pix::new(src.width(), src.height(), src.depth()).unwrap();
    let mut temp = temp_pix.try_into_mut().unwrap();
    fill_image(&mut temp, fill_value);

    for y in 0..h {
        let shift = ((y - ycen) as f32 * tan_a).round() as i32;
        for x in 0..w {
            let new_x = x + shift;
            if new_x >= 0 && new_x < w {
                let val = src.get_pixel_unchecked(x as u32, y as u32);
                temp.set_pixel_unchecked(new_x as u32, y as u32, val);
            }
        }
    }

    // Second pass: vertical shear from temp to dst
    let temp_ref: Pix = temp.into();
    for x in 0..w {
        let shift = ((x - xcen) as f32 * tan_a).round() as i32;
        for y in 0..h {
            let new_y = y + shift;
            if new_y >= 0 && new_y < h {
                let val = temp_ref.get_pixel_unchecked(x as u32, y as u32);
                dst.set_pixel_unchecked(x as u32, new_y as u32, val);
            }
        }
    }
}

// ============================================================================
// Phase 6: Corner area-map rotations
// ============================================================================

/// Rotate an image by area-map about the upper-left corner (dispatcher)
///
/// Dispatches to `rotate_am_color_corner` for 32bpp, `rotate_am_gray_corner`
/// for 8bpp, and clones for other depths.
///
/// # Arguments
/// * `pix` - Input image (8bpp or 32bpp)
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_am_corner(pix: &Pix, angle: f32, fill: RotateFill) -> TransformResult<Pix> {
    let _ = (pix, angle, fill);
    unimplemented!()
}

/// Rotate a 32bpp color image by area-map about the upper-left corner
///
/// # Arguments
/// * `pix` - 32bpp input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_am_color_corner(pix: &Pix, angle: f32, fill: RotateFill) -> TransformResult<Pix> {
    let _ = (pix, angle, fill);
    unimplemented!()
}

/// Rotate an 8bpp grayscale image by area-map about the upper-left corner
///
/// # Arguments
/// * `pix` - 8bpp input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_am_gray_corner(pix: &Pix, angle: f32, fill: RotateFill) -> TransformResult<Pix> {
    let _ = (pix, angle, fill);
    unimplemented!()
}

// ============================================================================
// Phase 6: Public shear rotation API
// ============================================================================

/// Rotate an image by shear about an arbitrary center point
///
/// Uses 2-shear for small angles (|angle| <= ~0.06 rad) and 3-shear otherwise.
/// The output is the same size as the input.
///
/// # Arguments
/// * `pix` - Input image
/// * `cx` - X coordinate of rotation center
/// * `cy` - Y coordinate of rotation center
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_shear(
    pix: &Pix,
    cx: i32,
    cy: i32,
    angle: f32,
    fill: ShearFill,
) -> TransformResult<Pix> {
    let _ = (pix, cx, cy, angle, fill);
    unimplemented!()
}

/// Rotate an image in-place by 3-shear about an arbitrary center point
///
/// Uses the H-V-H 3-shear sequence (Paeth's algorithm).
/// The image must not have a colormap.
///
/// # Arguments
/// * `pix` - Image to rotate in place (must not have colormap)
/// * `cx` - X coordinate of rotation center
/// * `cy` - Y coordinate of rotation center
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_shear_ip(
    pix: &mut PixMut,
    cx: i32,
    cy: i32,
    angle: f32,
    fill: ShearFill,
) -> TransformResult<()> {
    let _ = (pix, cx, cy, angle, fill);
    unimplemented!()
}

/// Rotate an image by shear about the image center
///
/// Equivalent to `rotate_shear` with center = (w/2, h/2).
///
/// # Arguments
/// * `pix` - Input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_shear_center(pix: &Pix, angle: f32, fill: ShearFill) -> TransformResult<Pix> {
    let _ = (pix, angle, fill);
    unimplemented!()
}

/// Rotate an image in-place by shear about the image center
///
/// Equivalent to `rotate_shear_ip` with center = (w/2, h/2).
/// The image must not have a colormap.
///
/// # Arguments
/// * `pix` - Image to rotate in place (must not have colormap)
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `fill` - Background fill color
pub fn rotate_shear_center_ip(
    pix: &mut PixMut,
    angle: f32,
    fill: ShearFill,
) -> TransformResult<()> {
    let _ = (pix, angle, fill);
    unimplemented!()
}

// ============================================================================
// Phase 6: Alpha-channel rotation
// ============================================================================

/// Rotate a 32bpp image with alpha channel handling
///
/// Rotates the RGB channels with area-map interpolation, then separately
/// rotates the alpha channel and combines the results.
///
/// # Arguments
/// * `pix` - 32bpp input image
/// * `angle` - Rotation angle in radians (positive = clockwise)
/// * `alpha_pix` - Optional 8bpp alpha channel; if None, a uniform alpha
///   derived from `fract` is used
/// * `fract` - Alpha fill fraction in [0.0, 1.0] when `alpha_pix` is None
///   (0.0 = fully transparent, 1.0 = fully opaque)
pub fn rotate_with_alpha(
    pix: &Pix,
    angle: f32,
    alpha_pix: Option<&Pix>,
    fract: f32,
) -> TransformResult<Pix> {
    let _ = (pix, angle, alpha_pix, fract);
    unimplemented!()
}

/// 3-shear rotation (Paeth's algorithm)
///
/// y' = y + tan(angle/2) * (x - xcen)  [first V-shear]
/// x' = x + sin(angle) * (y - ycen)    [H-shear]
/// y' = y + tan(angle/2) * (x - xcen)  [second V-shear]
fn rotate_3_shear(src: &Pix, dst: &mut PixMut, angle: f32, xcen: i32, ycen: i32, fill_value: u32) {
    let w = src.width() as i32;
    let h = src.height() as i32;
    let half_tan = (angle / 2.0).tan();
    let hangle = (angle.sin()).atan(); // atan(sin(angle))

    // Create two temporary images
    let temp1_pix = Pix::new(src.width(), src.height(), src.depth()).unwrap();
    let mut temp1 = temp1_pix.try_into_mut().unwrap();
    fill_image(&mut temp1, fill_value);

    let temp2_pix = Pix::new(src.width(), src.height(), src.depth()).unwrap();
    let mut temp2 = temp2_pix.try_into_mut().unwrap();
    fill_image(&mut temp2, fill_value);

    // First V-shear
    for x in 0..w {
        let shift = ((x - xcen) as f32 * half_tan).round() as i32;
        for y in 0..h {
            let new_y = y + shift;
            if new_y >= 0 && new_y < h {
                let val = src.get_pixel_unchecked(x as u32, y as u32);
                temp1.set_pixel_unchecked(x as u32, new_y as u32, val);
            }
        }
    }

    // H-shear
    let temp1_ref: Pix = temp1.into();
    for y in 0..h {
        let shift = ((y - ycen) as f32 * hangle).round() as i32;
        for x in 0..w {
            let new_x = x + shift;
            if new_x >= 0 && new_x < w {
                let val = temp1_ref.get_pixel_unchecked(x as u32, y as u32);
                temp2.set_pixel_unchecked(new_x as u32, y as u32, val);
            }
        }
    }

    // Second V-shear
    let temp2_ref: Pix = temp2.into();
    for x in 0..w {
        let shift = ((x - xcen) as f32 * half_tan).round() as i32;
        for y in 0..h {
            let new_y = y + shift;
            if new_y >= 0 && new_y < h {
                let val = temp2_ref.get_pixel_unchecked(x as u32, y as u32);
                dst.set_pixel_unchecked(x as u32, new_y as u32, val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{PixelDepth, color};

    #[test]
    fn test_rotate_90_clockwise() {
        // 2x3 grayscale image
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set pattern:
        // [1, 2]
        // [3, 4]
        // [5, 6]
        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(0, 1, 3);
        pix_mut.set_pixel_unchecked(1, 1, 4);
        pix_mut.set_pixel_unchecked(0, 2, 5);
        pix_mut.set_pixel_unchecked(1, 2, 6);

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, true).unwrap();

        // After 90 CW rotation: 3x2
        // [5, 3, 1]
        // [6, 4, 2]
        assert_eq!((rotated.width(), rotated.height()), (3, 2));
        assert_eq!(rotated.get_pixel_unchecked(0, 0), 5);
        assert_eq!(rotated.get_pixel_unchecked(1, 0), 3);
        assert_eq!(rotated.get_pixel_unchecked(2, 0), 1);
        assert_eq!(rotated.get_pixel_unchecked(0, 1), 6);
        assert_eq!(rotated.get_pixel_unchecked(1, 1), 4);
        assert_eq!(rotated.get_pixel_unchecked(2, 1), 2);
    }

    #[test]
    fn test_rotate_90_counterclockwise() {
        // 2x3 grayscale image
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(0, 1, 3);
        pix_mut.set_pixel_unchecked(1, 1, 4);
        pix_mut.set_pixel_unchecked(0, 2, 5);
        pix_mut.set_pixel_unchecked(1, 2, 6);

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, false).unwrap();

        // After 90 CCW rotation: 3x2
        // [2, 4, 6]
        // [1, 3, 5]
        assert_eq!((rotated.width(), rotated.height()), (3, 2));
        assert_eq!(rotated.get_pixel_unchecked(0, 0), 2);
        assert_eq!(rotated.get_pixel_unchecked(1, 0), 4);
        assert_eq!(rotated.get_pixel_unchecked(2, 0), 6);
        assert_eq!(rotated.get_pixel_unchecked(0, 1), 1);
        assert_eq!(rotated.get_pixel_unchecked(1, 1), 3);
        assert_eq!(rotated.get_pixel_unchecked(2, 1), 5);
    }

    #[test]
    fn test_rotate_180() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2]
        // [3, 4]
        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(0, 1, 3);
        pix_mut.set_pixel_unchecked(1, 1, 4);

        let pix: Pix = pix_mut.into();
        let rotated = rotate_180(&pix).unwrap();

        // After 180 rotation:
        // [4, 3]
        // [2, 1]
        assert_eq!(rotated.get_pixel_unchecked(0, 0), 4);
        assert_eq!(rotated.get_pixel_unchecked(1, 0), 3);
        assert_eq!(rotated.get_pixel_unchecked(0, 1), 2);
        assert_eq!(rotated.get_pixel_unchecked(1, 1), 1);
    }

    #[test]
    fn test_flip_lr() {
        let pix = Pix::new(3, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2, 3]
        // [4, 5, 6]
        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(2, 0, 3);
        pix_mut.set_pixel_unchecked(0, 1, 4);
        pix_mut.set_pixel_unchecked(1, 1, 5);
        pix_mut.set_pixel_unchecked(2, 1, 6);

        let pix: Pix = pix_mut.into();
        let flipped = flip_lr(&pix).unwrap();

        // After LR flip:
        // [3, 2, 1]
        // [6, 5, 4]
        assert_eq!(flipped.get_pixel_unchecked(0, 0), 3);
        assert_eq!(flipped.get_pixel_unchecked(1, 0), 2);
        assert_eq!(flipped.get_pixel_unchecked(2, 0), 1);
        assert_eq!(flipped.get_pixel_unchecked(0, 1), 6);
        assert_eq!(flipped.get_pixel_unchecked(1, 1), 5);
        assert_eq!(flipped.get_pixel_unchecked(2, 1), 4);
    }

    #[test]
    fn test_flip_tb() {
        let pix = Pix::new(2, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // [1, 2]
        // [3, 4]
        // [5, 6]
        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(0, 1, 3);
        pix_mut.set_pixel_unchecked(1, 1, 4);
        pix_mut.set_pixel_unchecked(0, 2, 5);
        pix_mut.set_pixel_unchecked(1, 2, 6);

        let pix: Pix = pix_mut.into();
        let flipped = flip_tb(&pix).unwrap();

        // After TB flip:
        // [5, 6]
        // [3, 4]
        // [1, 2]
        assert_eq!(flipped.get_pixel_unchecked(0, 0), 5);
        assert_eq!(flipped.get_pixel_unchecked(1, 0), 6);
        assert_eq!(flipped.get_pixel_unchecked(0, 1), 3);
        assert_eq!(flipped.get_pixel_unchecked(1, 1), 4);
        assert_eq!(flipped.get_pixel_unchecked(0, 2), 1);
        assert_eq!(flipped.get_pixel_unchecked(1, 2), 2);
    }

    #[test]
    fn test_rotate_orth_all_quadrants() {
        let pix = Pix::new(2, 2, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_pixel_unchecked(0, 0, 1);
        pix_mut.set_pixel_unchecked(1, 0, 2);
        pix_mut.set_pixel_unchecked(0, 1, 3);
        pix_mut.set_pixel_unchecked(1, 1, 4);

        let pix: Pix = pix_mut.into();

        // 0 quads = no rotation
        let r0 = rotate_orth(&pix, 0).unwrap();
        assert_eq!(r0.get_pixel_unchecked(0, 0), 1);

        // 4 quads = back to original (mod 4)
        let r4 = rotate_orth(&pix, 4).unwrap();
        assert_eq!(r4.get_pixel_unchecked(0, 0), 1);
    }

    #[test]
    fn test_rotate_32bpp() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        let red = color::compose_rgb(255, 0, 0);
        let green = color::compose_rgb(0, 255, 0);
        let blue = color::compose_rgb(0, 0, 255);
        let white = color::compose_rgb(255, 255, 255);

        // [R, G]
        // [B, W]
        pix_mut.set_pixel_unchecked(0, 0, red);
        pix_mut.set_pixel_unchecked(1, 0, green);
        pix_mut.set_pixel_unchecked(0, 1, blue);
        pix_mut.set_pixel_unchecked(1, 1, white);

        let pix: Pix = pix_mut.into();
        let rotated = rotate_90(&pix, true).unwrap();

        // After 90 CW:
        // [B, R]
        // [W, G]
        assert_eq!(rotated.get_pixel_unchecked(0, 0), blue);
        assert_eq!(rotated.get_pixel_unchecked(1, 0), red);
        assert_eq!(rotated.get_pixel_unchecked(0, 1), white);
        assert_eq!(rotated.get_pixel_unchecked(1, 1), green);
    }

    // ========================================================================
    // Arbitrary angle rotation tests
    // ========================================================================

    #[test]
    fn test_rotate_by_angle_zero() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 0.0).unwrap();
        assert_eq!(rotated.width(), 10);
        assert_eq!(rotated.height(), 10);
    }

    #[test]
    fn test_rotate_by_angle_90() {
        let pix = Pix::new(20, 10, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 90.0).unwrap();
        // 90 degree rotation swaps dimensions
        assert_eq!(rotated.width(), 10);
        assert_eq!(rotated.height(), 20);
    }

    #[test]
    fn test_rotate_by_angle_180() {
        let pix = Pix::new(20, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 100);
        let pix: Pix = pix_mut.into();

        let rotated = rotate_by_angle(&pix, 180.0).unwrap();
        assert_eq!(rotated.width(), 20);
        assert_eq!(rotated.height(), 10);
        // Top-left becomes bottom-right
        assert_eq!(rotated.get_pixel_unchecked(19, 9), 100);
    }

    #[test]
    fn test_rotate_by_angle_45() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let rotated = rotate_by_angle(&pix, 45.0).unwrap();

        // 45 degree rotation should expand dimensions by ~sqrt(2)
        let expected_size = (100.0 * 2.0_f32.sqrt()).ceil() as u32;
        assert!(rotated.width() >= expected_size - 2 && rotated.width() <= expected_size + 2);
        assert!(rotated.height() >= expected_size - 2 && rotated.height() <= expected_size + 2);
    }

    #[test]
    fn test_rotate_by_radians() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated_deg = rotate_by_angle(&pix, 45.0).unwrap();
        let rotated_rad = rotate_by_radians(&pix, std::f32::consts::FRAC_PI_4).unwrap();

        // Both should produce same dimensions
        assert_eq!(rotated_deg.width(), rotated_rad.width());
        assert_eq!(rotated_deg.height(), rotated_rad.height());
    }

    #[test]
    fn test_rotate_negative_angle() {
        let pix = Pix::new(100, 50, PixelDepth::Bit8).unwrap();
        let rotated_pos = rotate_by_angle(&pix, 30.0).unwrap();
        let rotated_neg = rotate_by_angle(&pix, -330.0).unwrap(); // -330 = +30

        assert_eq!(rotated_pos.width(), rotated_neg.width());
        assert_eq!(rotated_pos.height(), rotated_neg.height());
    }

    #[test]
    fn test_rotate_1bpp() {
        // 1-bit image uses nearest neighbor
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        // Set some pixels (1 = black in 1bpp)
        for i in 10..40 {
            pix_mut.set_pixel_unchecked(i, 25, 1);
        }
        let pix: Pix = pix_mut.into();

        let rotated = rotate_by_angle(&pix, 15.0).unwrap();
        assert!(rotated.width() > 50);
        assert!(rotated.height() > 50);
    }

    #[test]
    fn test_calculate_rotated_bounds() {
        // 45 degree rotation of a square
        let cos_45 = std::f32::consts::FRAC_PI_4.cos();
        let sin_45 = std::f32::consts::FRAC_PI_4.sin();
        let (w, h) = calculate_rotated_bounds(100.0, 100.0, cos_45, sin_45);

        // Diagonal of 100x100 square is ~141.4
        let expected = 100.0 * 2.0_f32.sqrt();
        assert!((w - expected).abs() < 2.0);
        assert!((h - expected).abs() < 2.0);
    }

    // ========================================================================
    // New API tests
    // ========================================================================

    #[test]
    fn test_rotate_with_options_sampling() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions::with_method(RotateMethod::Sampling);
        let rotated = rotate(&pix, 0.5, &options).unwrap();
        assert!(rotated.width() > 0);
        assert!(rotated.height() > 0);
    }

    #[test]
    fn test_rotate_with_options_area_map() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions::with_method(RotateMethod::AreaMap);
        let rotated = rotate(&pix, 0.2, &options).unwrap();
        assert!(rotated.width() > 0);
        assert!(rotated.height() > 0);
    }

    #[test]
    fn test_rotate_with_options_shear() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions::with_method(RotateMethod::Shear);
        let rotated = rotate(&pix, 0.05, &options).unwrap();
        assert!(rotated.width() > 0);
        assert!(rotated.height() > 0);
    }

    #[test]
    fn test_rotate_with_fill_black() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions::with_fill(RotateFill::Black);
        let rotated = rotate(&pix, 0.3, &options).unwrap();
        assert!(rotated.width() > 0);
    }

    #[test]
    fn test_rotate_with_custom_center() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let rotated = rotate_about_center(&pix, 0.2, 25.0, 25.0, RotateFill::White).unwrap();
        // Should have same dimensions when not expanding
        assert_eq!(rotated.width(), 100);
        assert_eq!(rotated.height(), 100);
    }

    #[test]
    fn test_rotate_method_auto_selection() {
        // 1bpp should use shear for small angles
        let method_1bpp = select_rotate_method(RotateMethod::Auto, PixelDepth::Bit1, 0.02);
        assert_eq!(method_1bpp, RotateMethod::Shear);

        // 1bpp should use sampling for large angles
        let method_1bpp_large = select_rotate_method(RotateMethod::Auto, PixelDepth::Bit1, 0.1);
        assert_eq!(method_1bpp_large, RotateMethod::Sampling);

        // 8bpp should use area mapping
        let method_8bpp = select_rotate_method(RotateMethod::Auto, PixelDepth::Bit8, 0.2);
        assert_eq!(method_8bpp, RotateMethod::AreaMap);

        // 32bpp should use area mapping
        let method_32bpp = select_rotate_method(RotateMethod::Auto, PixelDepth::Bit32, 0.2);
        assert_eq!(method_32bpp, RotateMethod::AreaMap);
    }

    #[test]
    fn test_rotate_32bpp_area_map() {
        let pix = Pix::new(30, 30, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a simple gradient
        for y in 0..30 {
            for x in 0..30 {
                let val = color::compose_rgb(x as u8 * 8, y as u8 * 8, 128);
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        let pix: Pix = pix_mut.into();
        let options = RotateOptions::with_method(RotateMethod::AreaMap);
        let rotated = rotate(&pix, 0.3, &options).unwrap();

        assert!(rotated.width() > 30);
        assert!(rotated.height() > 30);
    }

    #[test]
    fn test_rotate_very_small_angle() {
        // Angles below MIN_ANGLE_TO_ROTATE should just clone
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions::default();
        let rotated = rotate(&pix, 0.0001, &options).unwrap();

        // Should be same dimensions (cloned)
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
    }

    #[test]
    fn test_rotate_fill_to_value() {
        // Test RotateFill::to_value for different depths
        assert_eq!(RotateFill::White.to_value(PixelDepth::Bit1), 0);
        assert_eq!(RotateFill::Black.to_value(PixelDepth::Bit1), 1);
        assert_eq!(RotateFill::White.to_value(PixelDepth::Bit8), 255);
        assert_eq!(RotateFill::Black.to_value(PixelDepth::Bit8), 0);
        assert_eq!(RotateFill::Color(128).to_value(PixelDepth::Bit8), 128);
    }

    #[test]
    fn test_rotate_options_builder() {
        let options = RotateOptions::with_method(RotateMethod::Sampling)
            .center(25.0, 30.0)
            .expand(false);

        assert_eq!(options.method, RotateMethod::Sampling);
        assert_eq!(options.center_x, Some(25.0));
        assert_eq!(options.center_y, Some(30.0));
        assert!(!options.expand);
    }

    #[test]
    fn test_rotate_with_method_convenience() {
        let pix = Pix::new(40, 40, PixelDepth::Bit8).unwrap();
        let rotated = rotate_with_method(&pix, 0.25, RotateMethod::Bilinear).unwrap();
        assert!(rotated.width() > 0);
    }

    #[test]
    fn test_shear_rotation_2_shear() {
        // Small angle should use 2-shear
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions {
            method: RotateMethod::Shear,
            expand: false, // Don't expand output
            ..Default::default()
        };
        let rotated = rotate(&pix, 0.03, &options).unwrap(); // ~1.7 degrees
        assert_eq!(rotated.width(), 50); // No expansion
    }

    #[test]
    fn test_shear_rotation_3_shear() {
        // Larger angle should use 3-shear
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let options = RotateOptions {
            method: RotateMethod::Shear,
            expand: false,
            ..Default::default()
        };
        let rotated = rotate(&pix, 0.15, &options).unwrap(); // ~8.5 degrees
        assert_eq!(rotated.width(), 50);
    }

    // ========================================================================
    // Phase 6: Rotation拡張テスト
    // ========================================================================

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_am_gray_corner_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated = rotate_am_gray_corner(&pix, 0.2, RotateFill::White).unwrap();
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
        assert_eq!(rotated.depth(), PixelDepth::Bit8);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_am_gray_corner_small_angle_clones() {
        let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let rotated = rotate_am_gray_corner(&pix, 0.0, RotateFill::White).unwrap();
        assert_eq!(rotated.width(), 30);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_am_color_corner_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let rotated = rotate_am_color_corner(&pix, 0.2, RotateFill::White).unwrap();
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
        assert_eq!(rotated.depth(), PixelDepth::Bit32);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_am_corner_dispatches_gray() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated = rotate_am_corner(&pix, 0.2, RotateFill::White).unwrap();
        assert_eq!(rotated.depth(), PixelDepth::Bit8);
        assert_eq!(rotated.width(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_am_corner_dispatches_color() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let rotated = rotate_am_corner(&pix, 0.2, RotateFill::White).unwrap();
        assert_eq!(rotated.depth(), PixelDepth::Bit32);
        assert_eq!(rotated.width(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_shear_pub_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated = rotate_shear(&pix, 25, 25, 0.1, ShearFill::White).unwrap();
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_shear_center_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated = rotate_shear_center(&pix, 0.1, ShearFill::White).unwrap();
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_shear_ip_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        rotate_shear_ip(&mut pix_mut, 25, 25, 0.1, ShearFill::White).unwrap();
        assert_eq!(pix_mut.width(), 50);
        assert_eq!(pix_mut.height(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_shear_center_ip_smoke() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        rotate_shear_center_ip(&mut pix_mut, 0.1, ShearFill::White).unwrap();
        assert_eq!(pix_mut.width(), 50);
        assert_eq!(pix_mut.height(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_with_alpha_uniform() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let rotated = rotate_with_alpha(&pix, 0.2, None, 0.5).unwrap();
        assert_eq!(rotated.depth(), PixelDepth::Bit32);
        assert_eq!(rotated.width(), 50);
        assert_eq!(rotated.height(), 50);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_rotate_with_alpha_custom_alpha() {
        let pix = Pix::new(50, 50, PixelDepth::Bit32).unwrap();
        let alpha = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let rotated = rotate_with_alpha(&pix, 0.2, Some(&alpha), 1.0).unwrap();
        assert_eq!(rotated.depth(), PixelDepth::Bit32);
    }
}
