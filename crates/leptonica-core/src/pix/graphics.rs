//! Graphics rendering functions
//!
//! This module provides functions for drawing shapes on images:
//! - Lines (straight, with variable width)
//! - Boxes (rectangles, outlines)
//! - Polylines (connected line segments)
//! - Circles
//! - Contours (for grayscale images)

use super::{PixMut, PixelDepth};
use crate::box_::{Box, Boxa};
use crate::error::{Error, Result};
use crate::pta::{Pta, Ptaa};

/// Pixel operation for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelOp {
    /// Set pixels to maximum value (foreground)
    #[default]
    Set,
    /// Clear pixels to zero (background)
    Clear,
    /// Flip pixel values
    Flip,
}

/// RGB color for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Black color
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    /// White color
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    /// Red color
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    /// Green color
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    /// Blue color
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    /// Convert to grayscale value (0-255)
    pub fn to_gray(&self) -> u8 {
        ((self.r as u32 + self.g as u32 + self.b as u32) / 3) as u8
    }

    /// Compose as 32-bit RGBA pixel
    pub fn to_pixel32(&self) -> u32 {
        crate::color::compose_rgb(self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Output mode for contour rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContourOutput {
    /// Output 1bpp image with contour lines only
    #[default]
    Binary,
    /// Output same depth as input with contour lines overlaid
    Overlay,
}

// =============================================================================
// Point array generation helpers
// =============================================================================

/// Generate a point array for a line using Bresenham's integer algorithm.
///
/// Uses true integer Bresenham line drawing (no floating-point arithmetic).
/// The line connects `(x1, y1)` to `(x2, y2)` with 8-connectivity.
///
/// C equivalent: `generatePtaLine()` in `graphics.c`
pub fn generate_line_pta(x1: i32, y1: i32, x2: i32, y2: i32) -> Pta {
    // Handle degenerate case: single point
    if x1 == x2 && y1 == y2 {
        let mut pta = Pta::with_capacity(1);
        pta.push(x1 as f32, y1 as f32);
        return pta;
    }

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 > x1 { 1i32 } else { -1 };
    let sy = if y2 > y1 { 1i32 } else { -1 };

    let npts = dx.max(dy) + 1;
    let mut pta = Pta::with_capacity(npts as usize);

    let mut x = x1;
    let mut y = y1;

    if dx >= dy {
        // Step along x (more horizontal)
        let mut err = dx / 2;
        for _ in 0..npts {
            pta.push(x as f32, y as f32);
            err -= dy;
            if err < 0 {
                y += sy;
                err += dx;
            }
            x += sx;
        }
    } else {
        // Step along y (more vertical)
        let mut err = dy / 2;
        for _ in 0..npts {
            pta.push(x as f32, y as f32);
            err -= dx;
            if err < 0 {
                x += sx;
                err += dy;
            }
            y += sy;
        }
    }

    pta
}

/// Generate a point array for a line with specified width.
///
/// For width > 1, parallel lines are drawn on both sides.
pub fn generate_wide_line_pta(x1: i32, y1: i32, x2: i32, y2: i32, width: u32) -> Pta {
    let width = width.max(1);

    // Get the base line
    let base = generate_line_pta(x1, y1, x2, y2);

    if width == 1 {
        return base;
    }

    // Estimate capacity: base points * width
    let capacity = base.len() * width as usize;
    let mut result = Pta::with_capacity(capacity);

    // Copy base line
    for (x, y) in base.iter() {
        result.push(x, y);
    }

    // Determine if line is more horizontal or vertical
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let is_horizontal = dx > dy;

    // Add parallel lines
    for i in 1..width {
        let offset = (i + 1).div_ceil(2) as i32;
        let sign = if i % 2 == 1 { -1 } else { 1 };
        let actual_offset = offset * sign;

        let (x1a, y1a, x2a, y2a) = if is_horizontal {
            // Offset in y direction
            (x1, y1 + actual_offset, x2, y2 + actual_offset)
        } else {
            // Offset in x direction
            (x1 + actual_offset, y1, x2 + actual_offset, y2)
        };

        let parallel = generate_line_pta(x1a, y1a, x2a, y2a);
        for (x, y) in parallel.iter() {
            result.push(x, y);
        }
    }

    result
}

/// Generate a point array for a box outline.
pub fn generate_box_pta(b: &Box, width: u32) -> Pta {
    let width = width.max(1);
    let x = b.x;
    let y = b.y;
    let w = b.w;
    let h = b.h;

    if w == 0 || h == 0 {
        return Pta::new();
    }

    let half_w = (width / 2) as i32;

    // Four sides of the box
    let mut result = Pta::with_capacity((2 * (w + h) * width as i32) as usize);

    // Top edge
    let top = generate_wide_line_pta(x - half_w, y, x + w - 1 + half_w, y, width);
    for (px, py) in top.iter() {
        result.push(px, py);
    }

    // Bottom edge
    let bottom =
        generate_wide_line_pta(x - half_w, y + h - 1, x + w - 1 + half_w, y + h - 1, width);
    for (px, py) in bottom.iter() {
        result.push(px, py);
    }

    // Left edge (excluding corners already covered)
    let left = generate_wide_line_pta(x, y + 1 + half_w, x, y + h - 2 - half_w, width);
    for (px, py) in left.iter() {
        result.push(px, py);
    }

    // Right edge (excluding corners already covered)
    let right = generate_wide_line_pta(
        x + w - 1,
        y + 1 + half_w,
        x + w - 1,
        y + h - 2 - half_w,
        width,
    );
    for (px, py) in right.iter() {
        result.push(px, py);
    }

    result
}

/// Generate a point array for a polyline connecting vertices.
///
/// If `close` is true, the last vertex is connected back to the first.
pub fn generate_polyline_pta(vertices: &Pta, width: u32, close: bool) -> Pta {
    let n = vertices.len();
    if n < 2 {
        return Pta::new();
    }

    // Estimate capacity
    let capacity = n * 100 * width.max(1) as usize;
    let mut result = Pta::with_capacity(capacity);

    // Draw line segments between consecutive vertices
    for i in 0..(n - 1) {
        if let (Some((x1, y1)), Some((x2, y2))) = (vertices.get(i), vertices.get(i + 1)) {
            let segment = generate_wide_line_pta(x1 as i32, y1 as i32, x2 as i32, y2 as i32, width);
            for (x, y) in segment.iter() {
                result.push(x, y);
            }
        }
    }

    // Close the polyline if requested
    if close
        && n >= 2
        && let (Some((x1, y1)), Some((x2, y2))) = (vertices.get(n - 1), vertices.get(0))
    {
        let segment = generate_wide_line_pta(x1 as i32, y1 as i32, x2 as i32, y2 as i32, width);
        for (x, y) in segment.iter() {
            result.push(x, y);
        }
    }

    result
}

/// Generate a point array for a filled circle.
///
/// The circle has diameter = 2 * radius + 1 and is centered at (radius, radius).
pub fn generate_filled_circle_pta(radius: u32) -> Pta {
    if radius == 0 {
        let mut pta = Pta::with_capacity(1);
        pta.push(0.0, 0.0);
        return pta;
    }

    let diameter = 2 * radius + 1;
    let capacity = (diameter * diameter) as usize;
    let mut pta = Pta::with_capacity(capacity);

    let r = radius as i32;
    let threshold = (radius as f32 + 0.5).powi(2);

    for y in 0..=2 * r {
        for x in 0..=2 * r {
            let dx = x - r;
            let dy = y - r;
            let dist_sq = (dx * dx + dy * dy) as f32;
            if dist_sq <= threshold {
                pta.push(x as f32, y as f32);
            }
        }
    }

    pta
}

/// Generate a point array for a circle outline.
///
/// Uses Bresenham's circle algorithm with specified width.
pub fn generate_circle_outline_pta(cx: i32, cy: i32, radius: u32, width: u32) -> Pta {
    if radius == 0 {
        let mut pta = Pta::with_capacity(1);
        pta.push(cx as f32, cy as f32);
        return pta;
    }

    let width = width.max(1);
    let mut pta = Pta::with_capacity((8 * radius * width) as usize);

    let r_outer = radius as f32 + (width as f32 / 2.0);
    let r_inner = (radius as f32 - (width as f32 / 2.0)).max(0.0);

    let r_outer_sq = r_outer * r_outer;
    let r_inner_sq = r_inner * r_inner;

    let extent = (r_outer + 1.0) as i32;

    for dy in -extent..=extent {
        for dx in -extent..=extent {
            let dist_sq = (dx * dx + dy * dy) as f32;
            if dist_sq <= r_outer_sq && dist_sq >= r_inner_sq {
                pta.push((cx + dx) as f32, (cy + dy) as f32);
            }
        }
    }

    pta
}

// =============================================================================
// Phase 17.1: Additional PTA/PTAA generation functions
// =============================================================================

/// Orientation for hash-pattern line rendering.
///
/// C Leptonica constants: `L_HORIZONTAL_LINE`, `L_POS_SLOPE_LINE`,
/// `L_VERTICAL_LINE`, `L_NEG_SLOPE_LINE`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashOrientation {
    /// Horizontal lines (left to right)
    Horizontal,
    /// Diagonal lines with positive slope (+45°)
    PosSlope,
    /// Vertical lines (top to bottom)
    Vertical,
    /// Diagonal lines with negative slope (-45°)
    NegSlope,
}

/// Generate a Pta for the outlines of all boxes in `boxa`.
///
/// If `remove_dups` is true, duplicate points are removed.
///
/// C equivalent: `generatePtaBoxa()` in `graphics.c`
pub fn generate_boxa_pta(boxa: &Boxa, width: u32, remove_dups: bool) -> Pta {
    let width = width.max(1);
    let mut combined = Pta::new();
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            let pta = generate_box_pta(b, width);
            for pt in pta.iter() {
                combined.push(pt.0, pt.1);
            }
        }
    }
    if remove_dups {
        remove_duplicate_pts(combined)
    } else {
        combined
    }
}

/// Generate a Pta of hash lines for a single box.
///
/// * `spacing` – space between hash lines (must be > 1)
/// * `width` – line width in pixels
/// * `orient` – orientation of hash lines
/// * `outline` – whether to also draw the box outline
///
/// C equivalent: `generatePtaHashBox()` in `graphics.c`
pub fn generate_hash_box_pta(
    b: &Box,
    spacing: u32,
    width: u32,
    orient: HashOrientation,
    outline: bool,
) -> Result<Pta> {
    if spacing <= 1 {
        return Err(Error::InvalidParameter("spacing must be > 1".to_string()));
    }
    let bx = b.x;
    let by = b.y;
    let bw = b.w;
    let bh = b.h;
    if bw == 0 || bh == 0 {
        return Err(Error::InvalidParameter(
            "box has zero width or height".to_string(),
        ));
    }
    let width = width.max(1);

    let mut ptad = Pta::new();

    if outline {
        let outline_pta = generate_box_pta(b, width);
        for pt in outline_pta.iter() {
            ptad.push(pt.0, pt.1);
        }
    }

    match orient {
        HashOrientation::Horizontal => {
            let n = 1 + bh as usize / spacing as usize;
            for i in 0..n {
                let y = by
                    + if n > 1 {
                        (i * (bh as usize - 1) / (n - 1)) as i32
                    } else {
                        0
                    };
                let line = generate_wide_line_pta(bx, y, bx + bw - 1, y, width);
                for pt in line.iter() {
                    ptad.push(pt.0, pt.1);
                }
            }
        }
        HashOrientation::Vertical => {
            let n = 1 + bw as usize / spacing as usize;
            for i in 0..n {
                let x = bx
                    + if n > 1 {
                        (i * (bw as usize - 1) / (n - 1)) as i32
                    } else {
                        0
                    };
                let line = generate_wide_line_pta(x, by, x, by + bh - 1, width);
                for pt in line.iter() {
                    ptad.push(pt.0, pt.1);
                }
            }
        }
        HashOrientation::PosSlope => {
            let diag = bw as f32 + bh as f32;
            let n = (2.0 + diag / (1.4 * spacing as f32)) as usize;
            for i in 0..n {
                let x = (bx as f32 + (i as f32 + 0.5) * 1.4 * spacing as f32) as i32;
                if let Ok(isect) = b.intersect_by_line(x, by - 1, 1.0) {
                    if isect.count == 2 {
                        let (x1, y1) = isect.p1.unwrap();
                        let (x2, y2) = isect.p2.unwrap();
                        let line = generate_wide_line_pta(x1, y1, x2, y2, width);
                        for pt in line.iter() {
                            ptad.push(pt.0, pt.1);
                        }
                    }
                }
            }
        }
        HashOrientation::NegSlope => {
            let diag = bw as f32 + bh as f32;
            let n = (2.0 + diag / (1.4 * spacing as f32)) as usize;
            for i in 0..n {
                let x = (bx as f32 - bh as f32 + (i as f32 + 0.5) * 1.4 * spacing as f32) as i32;
                if let Ok(isect) = b.intersect_by_line(x, by - 1, -1.0) {
                    if isect.count == 2 {
                        let (x1, y1) = isect.p1.unwrap();
                        let (x2, y2) = isect.p2.unwrap();
                        let line = generate_wide_line_pta(x1, y1, x2, y2, width);
                        for pt in line.iter() {
                            ptad.push(pt.0, pt.1);
                        }
                    }
                }
            }
        }
    }

    Ok(ptad)
}

/// Generate a Pta of hash lines for all boxes in `boxa`.
///
/// C equivalent: `generatePtaHashBoxa()` in `graphics.c`
pub fn generate_hash_boxa_pta(
    boxa: &Boxa,
    spacing: u32,
    width: u32,
    orient: HashOrientation,
    outline: bool,
    remove_dups: bool,
) -> Result<Pta> {
    if spacing <= 1 {
        return Err(Error::InvalidParameter("spacing must be > 1".to_string()));
    }
    let mut combined = Pta::new();
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            let pta = generate_hash_box_pta(b, spacing, width, orient, outline)?;
            for pt in pta.iter() {
                combined.push(pt.0, pt.1);
            }
        }
    }
    if remove_dups {
        Ok(remove_duplicate_pts(combined))
    } else {
        Ok(combined)
    }
}

/// Generate a Ptaa where each Pta contains the 4 corner points of each box
/// in `boxa`.
///
/// C equivalent: `generatePtaaBoxa()` in `graphics.c`
pub fn generate_ptaa_boxa(boxa: &Boxa) -> Ptaa {
    let mut ptaa = Ptaa::with_capacity(boxa.len());
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            let x = b.x;
            let y = b.y;
            let w = b.w;
            let h = b.h;
            let mut pta = Pta::with_capacity(4);
            pta.push(x as f32, y as f32);
            pta.push((x + w - 1) as f32, y as f32);
            pta.push((x + w - 1) as f32, (y + h - 1) as f32);
            pta.push(x as f32, (y + h - 1) as f32);
            ptaa.push(pta);
        }
    }
    ptaa
}

/// Generate a Ptaa where each Pta is a hash-line pattern for a single box
/// in `boxa`.
///
/// C equivalent: `generatePtaaHashBoxa()` in `graphics.c`
pub fn generate_ptaa_hash_boxa(
    boxa: &Boxa,
    spacing: u32,
    width: u32,
    orient: HashOrientation,
    outline: bool,
) -> Result<Ptaa> {
    if spacing <= 1 {
        return Err(Error::InvalidParameter("spacing must be > 1".to_string()));
    }
    let mut ptaa = Ptaa::with_capacity(boxa.len());
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            let pta = generate_hash_box_pta(b, spacing, width, orient, outline)?;
            ptaa.push(pta);
        }
    }
    Ok(ptaa)
}

/// Generate a grid Pta: outlines of `nx` × `ny` rectangles covering a
/// `w` × `h` region, with line width `width`.
///
/// C equivalent: `generatePtaGrid()` in `graphics.c`
pub fn generate_grid_pta(w: u32, h: u32, nx: u32, ny: u32, width: u32) -> Result<Pta> {
    if nx == 0 || ny == 0 {
        return Err(Error::InvalidParameter("nx and ny must be > 0".to_string()));
    }
    if w < 2 * nx || h < 2 * ny {
        return Err(Error::InvalidParameter(
            "w or h too small for requested grid".to_string(),
        ));
    }
    let width = width.max(1);
    let bx = (w + nx - 1) / nx;
    let by = (h + ny - 1) / ny;

    let mut boxa = Boxa::new();
    for i in 0..ny {
        let y1 = by * i;
        let y2 = (y1 + by).min(h - 1);
        for j in 0..nx {
            let x1 = bx * j;
            let x2 = (x1 + bx).min(w - 1);
            let cell_w = (x2 - x1 + 1) as i32;
            let cell_h = (y2 - y1 + 1) as i32;
            if let Ok(b) = Box::new(x1 as i32, y1 as i32, cell_w, cell_h) {
                boxa.push(b);
            }
        }
    }

    Ok(generate_boxa_pta(&boxa, width, true))
}

/// Convert an 8-connected line Pta to 4-connected by inserting intermediate
/// points at diagonal steps.
///
/// C equivalent: `convertPtaLineTo4cc()` in `graphics.c`
pub fn convert_line_to_4cc(ptas: &Pta) -> Pta {
    let n = ptas.len();
    if n == 0 {
        return Pta::new();
    }
    let mut ptad = Pta::with_capacity(n + n / 2);
    let (mut xp, mut yp) = ptas.get(0).unwrap();
    ptad.push(xp, yp);
    for i in 1..n {
        let (x, y) = ptas.get(i).unwrap();
        if (x - xp).abs() > f32::EPSILON && (y - yp).abs() > f32::EPSILON {
            // Diagonal step: insert (x, yp) first to make it 4-connected
            ptad.push(x, yp);
        }
        ptad.push(x, y);
        xp = x;
        yp = y;
    }
    ptad
}

/// Generate a Pta of all points inside a `side` × `side` filled square.
///
/// The square occupies coordinates `(0, 0)` to `(side - 1, side - 1)`.
///
/// C equivalent: `generatePtaFilledSquare()` in `graphics.c`
pub fn generate_filled_square_pta(side: u32) -> Result<Pta> {
    if side == 0 {
        return Err(Error::InvalidParameter("side must be > 0".to_string()));
    }
    let mut pta = Pta::with_capacity((side * side) as usize);
    for y in 0..side {
        for x in 0..side {
            pta.push(x as f32, y as f32);
        }
    }
    Ok(pta)
}

/// Remove duplicate (x, y) pairs from a Pta, preserving order of first
/// occurrence.
fn remove_duplicate_pts(pta: Pta) -> Pta {
    use std::collections::HashSet;
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut out = Pta::with_capacity(pta.len());
    for (x, y) in pta.iter() {
        let key = (x.round() as i32, y.round() as i32);
        if seen.insert(key) {
            out.push(x, y);
        }
    }
    out
}

// =============================================================================
// PixMut rendering implementations
// =============================================================================

impl PixMut {
    /// Render a point array onto the image using the specified operation.
    ///
    /// Points outside the image bounds are clipped.
    pub fn render_pta(&mut self, pta: &Pta, op: PixelOp) -> Result<()> {
        if self.has_colormap() {
            return Err(Error::InvalidParameter(
                "render_pta does not support colormapped images".to_string(),
            ));
        }

        let w = self.width();
        let h = self.height();
        let depth = self.depth();

        let max_val = match depth {
            PixelDepth::Bit1 => 1,
            PixelDepth::Bit2 => 3,
            PixelDepth::Bit4 => 15,
            PixelDepth::Bit8 => 255,
            PixelDepth::Bit16 => 65535,
            PixelDepth::Bit32 => 0xFFFFFFFF,
        };

        for (x, y) in pta.iter() {
            let xi = x as i32;
            let yi = y as i32;

            // Clip to image bounds
            if xi < 0 || xi >= w as i32 || yi < 0 || yi >= h as i32 {
                continue;
            }

            let xu = xi as u32;
            let yu = yi as u32;

            match op {
                PixelOp::Set => {
                    self.set_pixel_unchecked(xu, yu, max_val);
                }
                PixelOp::Clear => {
                    self.set_pixel_unchecked(xu, yu, 0);
                }
                PixelOp::Flip => {
                    let current = self.get_pixel_unchecked(xu, yu);
                    self.set_pixel_unchecked(xu, yu, current ^ max_val);
                }
            }
        }

        Ok(())
    }

    /// Render a point array with a specific RGB color.
    ///
    /// For non-32bpp images, the color is converted to grayscale.
    pub fn render_pta_color(&mut self, pta: &Pta, color: Color) -> Result<()> {
        let w = self.width();
        let h = self.height();
        let depth = self.depth();

        // Calculate the pixel value based on depth
        let pixel_val = match depth {
            PixelDepth::Bit1 => 1u32,
            PixelDepth::Bit2 => (color.to_gray() >> 6) as u32,
            PixelDepth::Bit4 => (color.to_gray() >> 4) as u32,
            PixelDepth::Bit8 => color.to_gray() as u32,
            PixelDepth::Bit16 => {
                let g = color.to_gray() as u32;
                (g << 8) | g
            }
            PixelDepth::Bit32 => color.to_pixel32(),
        };

        for (x, y) in pta.iter() {
            let xi = x as i32;
            let yi = y as i32;

            if xi < 0 || xi >= w as i32 || yi < 0 || yi >= h as i32 {
                continue;
            }

            self.set_pixel_unchecked(xi as u32, yi as u32, pixel_val);
        }

        Ok(())
    }

    /// Render a point array with alpha blending (32bpp only).
    ///
    /// `fract` is the blend fraction: 1.0 = fully opaque, 0.0 = fully transparent.
    pub fn render_pta_blend(&mut self, pta: &Pta, color: Color, fract: f32) -> Result<()> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let fract = fract.clamp(0.0, 1.0);
        let w = self.width();
        let h = self.height();

        for (x, y) in pta.iter() {
            let xi = x as i32;
            let yi = y as i32;

            if xi < 0 || xi >= w as i32 || yi < 0 || yi >= h as i32 {
                continue;
            }

            let xu = xi as u32;
            let yu = yi as u32;

            let current = self.get_pixel_unchecked(xu, yu);
            let (r, g, b) = crate::color::extract_rgb(current);

            let new_r = ((1.0 - fract) * r as f32 + fract * color.r as f32) as u8;
            let new_g = ((1.0 - fract) * g as f32 + fract * color.g as f32) as u8;
            let new_b = ((1.0 - fract) * b as f32 + fract * color.b as f32) as u8;

            let new_pixel = crate::color::compose_rgb(new_r, new_g, new_b);
            self.set_pixel_unchecked(xu, yu, new_pixel);
        }

        Ok(())
    }

    /// Render a line from (x1, y1) to (x2, y2).
    pub fn render_line(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        width: u32,
        op: PixelOp,
    ) -> Result<()> {
        let pta = generate_wide_line_pta(x1, y1, x2, y2, width.max(1));
        self.render_pta(&pta, op)
    }

    /// Render a line with a specific color.
    pub fn render_line_color(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        width: u32,
        color: Color,
    ) -> Result<()> {
        let pta = generate_wide_line_pta(x1, y1, x2, y2, width.max(1));
        self.render_pta_color(&pta, color)
    }

    /// Render a line with alpha blending (32bpp only).
    #[allow(clippy::too_many_arguments)]
    pub fn render_line_blend(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        width: u32,
        color: Color,
        fract: f32,
    ) -> Result<()> {
        let pta = generate_wide_line_pta(x1, y1, x2, y2, width.max(1));
        self.render_pta_blend(&pta, color, fract)
    }

    /// Render a box (rectangle outline).
    pub fn render_box(&mut self, b: &Box, width: u32, op: PixelOp) -> Result<()> {
        let pta = generate_box_pta(b, width.max(1));
        self.render_pta(&pta, op)
    }

    /// Render a box with a specific color.
    pub fn render_box_color(&mut self, b: &Box, width: u32, color: Color) -> Result<()> {
        let pta = generate_box_pta(b, width.max(1));
        self.render_pta_color(&pta, color)
    }

    /// Render a box with alpha blending (32bpp only).
    pub fn render_box_blend(
        &mut self,
        b: &Box,
        width: u32,
        color: Color,
        fract: f32,
    ) -> Result<()> {
        let pta = generate_box_pta(b, width.max(1));
        self.render_pta_blend(&pta, color, fract)
    }

    /// Render a polyline connecting the vertices.
    ///
    /// If `close` is true, the last vertex is connected to the first.
    pub fn render_polyline(
        &mut self,
        vertices: &Pta,
        width: u32,
        close: bool,
        op: PixelOp,
    ) -> Result<()> {
        let pta = generate_polyline_pta(vertices, width.max(1), close);
        self.render_pta(&pta, op)
    }

    /// Render a polyline with a specific color.
    pub fn render_polyline_color(
        &mut self,
        vertices: &Pta,
        width: u32,
        close: bool,
        color: Color,
    ) -> Result<()> {
        let pta = generate_polyline_pta(vertices, width.max(1), close);
        self.render_pta_color(&pta, color)
    }

    /// Render a polyline with alpha blending (32bpp only).
    pub fn render_polyline_blend(
        &mut self,
        vertices: &Pta,
        width: u32,
        close: bool,
        color: Color,
        fract: f32,
    ) -> Result<()> {
        let pta = generate_polyline_pta(vertices, width.max(1), close);
        self.render_pta_blend(&pta, color, fract)
    }

    /// Render a circle outline centered at (cx, cy).
    pub fn render_circle(
        &mut self,
        cx: i32,
        cy: i32,
        radius: u32,
        width: u32,
        op: PixelOp,
    ) -> Result<()> {
        let pta = generate_circle_outline_pta(cx, cy, radius, width.max(1));
        self.render_pta(&pta, op)
    }

    /// Render a circle outline with a specific color.
    pub fn render_circle_color(
        &mut self,
        cx: i32,
        cy: i32,
        radius: u32,
        width: u32,
        color: Color,
    ) -> Result<()> {
        let pta = generate_circle_outline_pta(cx, cy, radius, width.max(1));
        self.render_pta_color(&pta, color)
    }

    /// Render a circle outline with alpha blending (32bpp only).
    pub fn render_circle_blend(
        &mut self,
        cx: i32,
        cy: i32,
        radius: u32,
        width: u32,
        color: Color,
        fract: f32,
    ) -> Result<()> {
        let pta = generate_circle_outline_pta(cx, cy, radius, width.max(1));
        self.render_pta_blend(&pta, color, fract)
    }

    /// Render a filled circle centered at (cx, cy).
    pub fn render_filled_circle(
        &mut self,
        cx: i32,
        cy: i32,
        radius: u32,
        op: PixelOp,
    ) -> Result<()> {
        let base_pta = generate_filled_circle_pta(radius);
        let r = radius as i32;

        // Translate the circle to the target position
        let mut pta = Pta::with_capacity(base_pta.len());
        for (x, y) in base_pta.iter() {
            pta.push(x + (cx - r) as f32, y + (cy - r) as f32);
        }

        self.render_pta(&pta, op)
    }

    /// Render a filled circle with a specific color.
    pub fn render_filled_circle_color(
        &mut self,
        cx: i32,
        cy: i32,
        radius: u32,
        color: Color,
    ) -> Result<()> {
        let base_pta = generate_filled_circle_pta(radius);
        let r = radius as i32;

        let mut pta = Pta::with_capacity(base_pta.len());
        for (x, y) in base_pta.iter() {
            pta.push(x + (cx - r) as f32, y + (cy - r) as f32);
        }

        self.render_pta_color(&pta, color)
    }

    // has_colormap() is now provided by the main PixMut impl in mod.rs
}

// =============================================================================
// Contour rendering (requires creating new Pix)
// =============================================================================

use super::Pix;

impl Pix {
    /// Render contour lines on a grayscale image.
    ///
    /// # Arguments
    /// * `start_val` - Value of the lowest contour (must be in [0, max_val])
    /// * `increment` - Increment between contours (must be > 0)
    /// * `output` - Output format (binary or overlay)
    ///
    /// # Returns
    /// A new Pix with contour lines rendered
    pub fn render_contours(
        &self,
        start_val: u32,
        increment: u32,
        output: ContourOutput,
    ) -> Result<Pix> {
        let depth = self.depth();

        // Only 8-bit and 16-bit grayscale supported
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }

        if self.has_colormap() {
            return Err(Error::InvalidParameter(
                "render_contours does not support colormapped images".to_string(),
            ));
        }

        let max_val = depth.max_value();
        if start_val > max_val {
            return Err(Error::InvalidParameter(format!(
                "start_val {} exceeds max value {} for {} bpp",
                start_val,
                max_val,
                depth.bits()
            )));
        }

        if increment == 0 {
            return Err(Error::InvalidParameter("increment must be > 0".to_string()));
        }

        let w = self.width();
        let h = self.height();

        // Create output image
        let out_depth = match output {
            ContourOutput::Binary => PixelDepth::Bit1,
            ContourOutput::Overlay => depth,
        };

        let mut pixd = if output == ContourOutput::Overlay {
            self.to_mut()
        } else {
            Pix::new(w, h, out_depth)?.to_mut()
        };

        // Render contours
        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel(x, y).unwrap_or(0);

                if val < start_val {
                    continue;
                }

                let test = (val - start_val) % increment;
                if test == 0 {
                    match output {
                        ContourOutput::Binary => {
                            // Set bit for contour line
                            let _ = pixd.set_pixel(x, y, 1);
                        }
                        ContourOutput::Overlay => {
                            // Set to black (0) for contour line
                            let _ = pixd.set_pixel(x, y, 0);
                        }
                    }
                }
            }
        }

        Ok(pixd.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_line_horizontal() {
        let pta = generate_line_pta(0, 0, 10, 0);
        assert_eq!(pta.len(), 11);

        // Check endpoints
        assert_eq!(pta.get(0), Some((0.0, 0.0)));
        assert_eq!(pta.get(10), Some((10.0, 0.0)));
    }

    #[test]
    fn test_generate_line_vertical() {
        let pta = generate_line_pta(0, 0, 0, 10);
        assert_eq!(pta.len(), 11);

        assert_eq!(pta.get(0), Some((0.0, 0.0)));
        assert_eq!(pta.get(10), Some((0.0, 10.0)));
    }

    #[test]
    fn test_generate_line_diagonal() {
        let pta = generate_line_pta(0, 0, 10, 10);
        assert_eq!(pta.len(), 11);

        assert_eq!(pta.get(0), Some((0.0, 0.0)));
        assert_eq!(pta.get(10), Some((10.0, 10.0)));
    }

    #[test]
    fn test_generate_line_single_point() {
        let pta = generate_line_pta(5, 5, 5, 5);
        assert_eq!(pta.len(), 1);
        assert_eq!(pta.get(0), Some((5.0, 5.0)));
    }

    #[test]
    fn test_generate_wide_line() {
        let pta = generate_wide_line_pta(0, 0, 10, 0, 3);
        // Width 3: base line + 2 parallel lines
        assert!(pta.len() >= 33); // At least 11 * 3
    }

    #[test]
    fn test_generate_box_pta() {
        let b = Box::new_unchecked(10, 10, 20, 20);
        let pta = generate_box_pta(&b, 1);
        // Box perimeter should have points
        assert!(!pta.is_empty());
    }

    #[test]
    fn test_generate_polyline_pta() {
        let mut vertices = Pta::new();
        vertices.push(0.0, 0.0);
        vertices.push(10.0, 0.0);
        vertices.push(10.0, 10.0);

        let pta = generate_polyline_pta(&vertices, 1, false);
        // Two line segments
        assert!(!pta.is_empty());

        let pta_closed = generate_polyline_pta(&vertices, 1, true);
        // Closed should have more points (third segment)
        assert!(pta_closed.len() > pta.len());
    }

    #[test]
    fn test_generate_filled_circle() {
        let pta = generate_filled_circle_pta(5);
        // Circle with radius 5 should have many points
        assert!(pta.len() > 50);

        // Check that center point exists
        let has_center = pta.iter().any(|(x, y)| x == 5.0 && y == 5.0);
        assert!(has_center);
    }

    #[test]
    fn test_render_line_8bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        pix_mut
            .render_line(10, 10, 90, 90, 1, PixelOp::Set)
            .unwrap();

        // Check that the line was drawn (some points should be set)
        let pix: Pix = pix_mut.into();
        let val = pix.get_pixel(50, 50);
        assert_eq!(val, Some(255)); // Diagonal line passes through center
    }

    #[test]
    fn test_render_box_8bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        let b = Box::new_unchecked(20, 20, 60, 60);
        pix_mut.render_box(&b, 1, PixelOp::Set).unwrap();

        let pix: Pix = pix_mut.into();

        // Check corners
        assert_eq!(pix.get_pixel(20, 20), Some(255));
        assert_eq!(pix.get_pixel(79, 20), Some(255));
        assert_eq!(pix.get_pixel(20, 79), Some(255));
        assert_eq!(pix.get_pixel(79, 79), Some(255));

        // Check center (should be empty)
        assert_eq!(pix.get_pixel(50, 50), Some(0));
    }

    #[test]
    fn test_render_circle_8bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        pix_mut.render_circle(50, 50, 30, 1, PixelOp::Set).unwrap();

        let pix: Pix = pix_mut.into();

        // Check points on the circle
        assert_eq!(pix.get_pixel(50, 20), Some(255)); // Top
        assert_eq!(pix.get_pixel(50, 80), Some(255)); // Bottom
        assert_eq!(pix.get_pixel(20, 50), Some(255)); // Left
        assert_eq!(pix.get_pixel(80, 50), Some(255)); // Right

        // Check center (should be empty for outline)
        assert_eq!(pix.get_pixel(50, 50), Some(0));
    }

    #[test]
    fn test_render_line_color_32bpp() {
        let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.to_mut();

        pix_mut
            .render_line_color(10, 50, 90, 50, 1, Color::RED)
            .unwrap();

        let pix: Pix = pix_mut.into();

        // Check that red was drawn
        let rgb = pix.get_rgb(50, 50);
        assert_eq!(rgb, Some((255, 0, 0)));
    }

    #[test]
    fn test_render_contours() {
        // Create a grayscale gradient
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        // Fill with horizontal gradient
        for y in 0..100 {
            for x in 0..100 {
                let val = ((x as f32 / 100.0) * 255.0) as u32;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        // Render contours every 50 levels
        let contours = pix.render_contours(0, 50, ContourOutput::Binary).unwrap();

        assert_eq!(contours.depth(), PixelDepth::Bit1);
        assert_eq!(contours.width(), 100);
        assert_eq!(contours.height(), 100);

        // There should be contour lines at values 0, 50, 100, 150, 200, 250
        // which correspond to x positions approximately at 0, 20, 39, 59, 78, 98
    }

    #[test]
    fn test_color_conversions() {
        let c = Color::new(100, 150, 200);
        assert_eq!(c.to_gray(), 150);

        let pixel = c.to_pixel32();
        let (r, g, b) = crate::color::extract_rgb(pixel);
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn test_pixel_op_flip() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        // Set initial value
        pix_mut.set_pixel(5, 5, 100).unwrap();

        // Flip
        let mut pta = Pta::new();
        pta.push(5.0, 5.0);
        pix_mut.render_pta(&pta, PixelOp::Flip).unwrap();

        // Value should be flipped: 100 ^ 255 = 155
        assert_eq!(pix_mut.get_pixel(5, 5), Some(155));
    }

    // -- Phase 17.1 new functions --

    #[test]
    fn test_generate_boxa_pta() {
        use crate::box_::Boxa;
        let mut boxa = Boxa::new();
        boxa.push(crate::box_::Box::new(0, 0, 10, 10).unwrap());
        boxa.push(crate::box_::Box::new(20, 20, 5, 5).unwrap());
        let pta = generate_boxa_pta(&boxa, 1, false);
        assert!(pta.len() > 0); // should contain outline points of both boxes
    }

    #[test]
    fn test_generate_hash_box_pta() {
        let b = crate::box_::Box::new(0, 0, 20, 20).unwrap();
        let pta = generate_hash_box_pta(&b, 4, 1, HashOrientation::Horizontal, false).unwrap();
        assert!(pta.len() > 0);
    }

    #[test]
    fn test_generate_hash_boxa_pta() {
        use crate::box_::Boxa;
        let mut boxa = Boxa::new();
        boxa.push(crate::box_::Box::new(0, 0, 20, 20).unwrap());
        let pta =
            generate_hash_boxa_pta(&boxa, 4, 1, HashOrientation::Vertical, false, false).unwrap();
        assert!(pta.len() > 0);
    }

    #[test]
    fn test_generate_ptaa_boxa() {
        use crate::box_::Boxa;
        let mut boxa = Boxa::new();
        boxa.push(crate::box_::Box::new(0, 0, 10, 10).unwrap());
        boxa.push(crate::box_::Box::new(5, 5, 8, 8).unwrap());
        let ptaa = generate_ptaa_boxa(&boxa);
        assert_eq!(ptaa.len(), 2);
        // Each pta should have 4 corner points
        assert_eq!(ptaa.get(0).unwrap().len(), 4);
    }

    #[test]
    fn test_generate_ptaa_hash_boxa() {
        use crate::box_::Boxa;
        let mut boxa = Boxa::new();
        boxa.push(crate::box_::Box::new(0, 0, 20, 20).unwrap());
        let ptaa = generate_ptaa_hash_boxa(&boxa, 4, 1, HashOrientation::PosSlope, false).unwrap();
        assert_eq!(ptaa.len(), 1);
    }

    #[test]
    fn test_generate_grid_pta() {
        let pta = generate_grid_pta(100, 80, 4, 3, 1).unwrap();
        assert!(pta.len() > 0);
    }

    #[test]
    fn test_convert_line_to_4cc() {
        // Create a diagonal line (8-connected)
        let mut pta = Pta::new();
        pta.push(0.0, 0.0);
        pta.push(1.0, 1.0);
        pta.push(2.0, 2.0);
        let pta4 = convert_line_to_4cc(&pta);
        // 2 diagonal steps → 2 extra intermediate points added
        assert_eq!(pta4.len(), 5); // 0,0 | 1,0 | 1,1 | 2,1 | 2,2
    }

    #[test]
    fn test_generate_filled_square_pta() {
        let pta = generate_filled_square_pta(3).unwrap();
        assert_eq!(pta.len(), 9); // 3×3 = 9 points
        // Check corners
        assert!(
            pta.iter()
                .any(|(x, y)| (x - 0.0).abs() < 0.01 && (y - 0.0).abs() < 0.01)
        );
        assert!(
            pta.iter()
                .any(|(x, y)| (x - 2.0).abs() < 0.01 && (y - 2.0).abs() < 0.01)
        );
    }
}
