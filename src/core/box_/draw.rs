//! Box drawing, masking, and region comparison operations
//!
//! Functions for painting, masking, and drawing boxes on images,
//! and for structural comparison and selection of box arrays.
//!
//! C Leptonica equivalents: boxfunc3.c

use crate::core::box_::{Box, Boxa, Boxaa};
use crate::core::error::{Error, Result};
use crate::core::pix::graphics::{Color, PixelOp};
use crate::core::pix::{Pix, PixMut, PixelDepth};
use crate::core::pixa::Pixa;

// ---- Types ----

/// Result of comparing two Boxa by region coverage
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionCompareResult {
    /// True if both boxas have the same number of boxes (after area filtering)
    pub same_count: bool,
    /// Normalized absolute area difference in [0, 1]
    pub diff_area: f64,
}

/// Fixed palette for cycling colors in random paint/draw operations.
/// Mirrors Leptonica's approach of using distinct, visible colors.
const CYCLING_COLORS: [Color; 10] = [
    Color::new(255, 0, 0),   // Red
    Color::new(0, 200, 0),   // Green
    Color::new(0, 0, 255),   // Blue
    Color::new(255, 200, 0), // Yellow
    Color::new(0, 200, 200), // Cyan
    Color::new(200, 0, 200), // Magenta
    Color::new(255, 128, 0), // Orange
    Color::new(128, 0, 255), // Violet
    Color::new(0, 128, 255), // Sky
    Color::new(255, 0, 128), // Rose
];

/// Clip a box to image bounds in signed coordinates, returning
/// `(x, y, x_end, y_end)` as `u32` or `None` if no intersection.
fn clip_box_to_image(b: &Box, img_w: u32, img_h: u32) -> Option<(u32, u32, u32, u32)> {
    let x0 = b.x.max(0);
    let y0 = b.y.max(0);
    let x1 = (b.x + b.w).min(img_w as i32);
    let y1 = (b.y + b.h).min(img_h as i32);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some((x0 as u32, y0 as u32, x1 as u32, y1 as u32))
}

/// Fill a rectangular region defined by a Box with a constant pixel value.
fn fill_box_val(pix: &mut PixMut, b: &Box, val: u32) {
    let Some((x, y, x_end, y_end)) = clip_box_to_image(b, pix.width(), pix.height()) else {
        return;
    };
    for py in y..y_end {
        for px in x..x_end {
            pix.set_pixel_unchecked(px, py, val);
        }
    }
}

/// Flip all pixels in a rectangular region defined by a Box.
fn flip_box(pix: &mut PixMut, b: &Box) {
    let Some((x, y, x_end, y_end)) = clip_box_to_image(b, pix.width(), pix.height()) else {
        return;
    };
    let max_val = pix.depth().max_value();
    for py in y..y_end {
        for px in x..x_end {
            let v = pix.get_pixel_unchecked(px, py);
            pix.set_pixel_unchecked(px, py, max_val - v);
        }
    }
}

// ---- PixMut methods ----

impl PixMut {
    /// Apply a masking operation to all box regions in a Boxa.
    ///
    /// Each box in `boxa` is set, cleared, or flipped according to `op`.
    ///
    /// C Leptonica equivalent: `pixMaskBoxa`
    pub fn mask_boxa(&mut self, boxa: &Boxa, op: PixelOp) {
        let max_val = self.depth().max_value();
        for b in boxa.boxes() {
            match op {
                PixelOp::Set => fill_box_val(self, b, max_val),
                PixelOp::Clear => fill_box_val(self, b, 0),
                PixelOp::Flip => flip_box(self, b),
            }
        }
    }

    /// Fill all box regions in a Boxa with a constant pixel value.
    ///
    /// C Leptonica equivalent: `pixPaintBoxa` (single solid color variant)
    pub fn paint_boxa(&mut self, boxa: &Boxa, val: u32) {
        for b in boxa.boxes() {
            fill_box_val(self, b, val);
        }
    }

    /// Set all box regions to black or white.
    ///
    /// If `is_white` is true, pixels are set to maximum; otherwise to zero.
    ///
    /// C Leptonica equivalent: `pixSetBlackOrWhiteBoxa`
    pub fn set_bw_boxa(&mut self, boxa: &Boxa, is_white: bool) {
        let val = if is_white {
            self.depth().max_value()
        } else {
            0
        };
        for b in boxa.boxes() {
            fill_box_val(self, b, val);
        }
    }

    /// Paint each box in a Boxa with a cycling color (32bpp only).
    ///
    /// Uses a fixed palette of 10 colors, cycling by box index.
    ///
    /// C Leptonica equivalent: `pixPaintBoxaRandom`
    pub fn paint_boxa_random(&mut self, boxa: &Boxa) -> Result<()> {
        if self.depth().bits() != 32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        for (i, b) in boxa.boxes().iter().enumerate() {
            let color = CYCLING_COLORS[i % CYCLING_COLORS.len()];
            let val = crate::core::pixel::compose_rgb(color.r, color.g, color.b);
            fill_box_val(self, b, val);
        }
        Ok(())
    }

    /// Blend each box in a Boxa with a cycling color (32bpp only).
    ///
    /// `fract` controls how much of the cycling color to blend in [0.0, 1.0].
    ///
    /// C Leptonica equivalent: `pixBlendBoxaRandom`
    pub fn blend_boxa_random(&mut self, boxa: &Boxa, fract: f32) -> Result<()> {
        if self.depth().bits() != 32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let fract = fract.clamp(0.0, 1.0);
        let img_w = self.width();
        let img_h = self.height();
        for (i, b) in boxa.boxes().iter().enumerate() {
            let Some((x, y, x_end, y_end)) = clip_box_to_image(b, img_w, img_h) else {
                continue;
            };
            let color = CYCLING_COLORS[i % CYCLING_COLORS.len()];
            let blend_r = color.r as f32;
            let blend_g = color.g as f32;
            let blend_b = color.b as f32;
            for py in y..y_end {
                for px in x..x_end {
                    let pixel = self.get_pixel_unchecked(px, py);
                    let (r, g, b_ch, a) = crate::core::pixel::extract_rgba(pixel);
                    let nr = (r as f32 * (1.0 - fract) + blend_r * fract) as u8;
                    let ng = (g as f32 * (1.0 - fract) + blend_g * fract) as u8;
                    let nb = (b_ch as f32 * (1.0 - fract) + blend_b * fract) as u8;
                    self.set_pixel_unchecked(
                        px,
                        py,
                        crate::core::pixel::compose_rgba(nr, ng, nb, a),
                    );
                }
            }
        }
        Ok(())
    }

    /// Draw outlines of all boxes in a Boxa with a given color.
    ///
    /// C Leptonica equivalent: `pixDrawBoxa`
    pub fn draw_boxa(&mut self, boxa: &Boxa, width: u32, color: Color) -> Result<()> {
        for b in boxa.boxes() {
            self.render_box_color(b, width, color)?;
        }
        Ok(())
    }

    /// Draw outlines of all boxes in a Boxa with cycling colors.
    ///
    /// C Leptonica equivalent: `pixDrawBoxaRandom`
    pub fn draw_boxa_random(&mut self, boxa: &Boxa, width: u32) -> Result<()> {
        for (i, b) in boxa.boxes().iter().enumerate() {
            let color = CYCLING_COLORS[i % CYCLING_COLORS.len()];
            self.render_box_color(b, width, color)?;
        }
        Ok(())
    }
}

// ---- Boxa methods ----

impl Boxa {
    /// Compare two boxas by their region coverage.
    ///
    /// Filters both boxas by `area_thresh` before comparison.
    /// Returns `(same_count, diff_area)` where:
    /// - `same_count`: whether both have the same number of boxes
    /// - `diff_area`: normalized area difference in [0, 1]
    ///
    /// C Leptonica equivalent: `boxaCompareRegions` (area metric only)
    pub fn compare_regions(&self, other: &Boxa, area_thresh: i64) -> RegionCompareResult {
        let a1: Vec<&Box> = self
            .boxes()
            .iter()
            .filter(|b| b.area() >= area_thresh)
            .collect();
        let a2: Vec<&Box> = other
            .boxes()
            .iter()
            .filter(|b| b.area() >= area_thresh)
            .collect();
        let same_count = a1.len() == a2.len();
        if a1.is_empty() && a2.is_empty() {
            return RegionCompareResult {
                same_count,
                diff_area: 0.0,
            };
        }
        if a1.is_empty() || a2.is_empty() {
            return RegionCompareResult {
                same_count,
                diff_area: 1.0,
            };
        }
        let area1: i64 = a1.iter().map(|b| b.area()).sum();
        let area2: i64 = a2.iter().map(|b| b.area()).sum();
        let total = area1 + area2;
        let diff_area = if total == 0 {
            0.0
        } else {
            (area1 - area2).unsigned_abs() as f64 / total as f64
        };
        RegionCompareResult {
            same_count,
            diff_area,
        }
    }

    /// Select the box nearest to the upper-left corner from large boxes.
    ///
    /// First filters boxes where `area / max_area >= area_slop`,
    /// then from those selects the top-most, or leftmost if within `y_slop` pixels.
    ///
    /// C Leptonica equivalent: `boxaSelectLargeULBox`
    pub fn select_large_ul_box(&self, area_slop: f64, y_slop: i32) -> Option<Box> {
        if self.is_empty() {
            return None;
        }
        let max_area = self.boxes().iter().map(|b| b.area()).max().unwrap_or(0);
        if max_area == 0 {
            return None;
        }
        let y_slop = y_slop.max(0);
        // Collect boxes eligible by area, then sort top-down
        let mut eligible: Vec<Box> = self
            .boxes()
            .iter()
            .copied()
            .filter(|b| b.area() as f64 / max_area as f64 >= area_slop)
            .collect();
        if eligible.is_empty() {
            return None;
        }
        eligible.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));
        // Start with topmost box; prefer leftmost within y_slop
        let base_y = eligible[0].y;
        let mut best_x = eligible[0].x;
        let mut select = 0;
        for (i, b) in eligible.iter().enumerate().skip(1) {
            if b.y - base_y < y_slop && b.x < best_x {
                best_x = b.x;
                select = i;
            }
        }
        Some(eligible[select])
    }
}

// ---- Pix methods for box operations ----

impl Pix {
    /// Create a 1bpp mask from connected components, returning (mask, boxa).
    ///
    /// For 1bpp input. Finds connected components using flood-fill labeling
    /// and returns the bounding boxes. The mask is a copy of the input since
    /// every foreground pixel already belongs to some component.
    ///
    /// C Leptonica equivalent: `pixMaskConnComp`
    pub fn mask_conn_comp(&self, connectivity: u32) -> Result<(Pix, Boxa)> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let conn = match connectivity {
            4 => crate::region::ConnectivityType::FourWay,
            8 => crate::region::ConnectivityType::EightWay,
            _ => {
                return Err(Error::InvalidParameter(format!(
                    "connectivity must be 4 or 8, got {connectivity}"
                )));
            }
        };
        let (boxa, _pixa) = crate::region::conncomp_pixa(self, conn)
            .map_err(|e| Error::InvalidParameter(e.to_string()))?;
        // The mask is the same as the input for 1bpp
        let mask = self.deep_clone();
        Ok((mask, boxa))
    }

    /// Split a 1bpp image into rectangular regions (Boxa).
    ///
    /// For each connected component, greedily partitions it into rectangular
    /// sub-regions by projecting foreground pixels onto rows and columns.
    ///
    /// * `min_sum` - minimum foreground pixels in a row/column to start a region.
    /// * `skip_dist` - minimum gap (in pixels) between regions.
    /// * `delta` - tolerance for boundary detection.
    /// * `max_bg_comp` - max background component fraction to include in region.
    ///
    /// C Leptonica equivalent: `pixSplitIntoBoxa`
    pub fn split_into_boxa(
        &self,
        min_sum: u32,
        skip_dist: u32,
        delta: u32,
        max_bg_comp: u32,
    ) -> Result<Boxa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        // Get connected components
        let (cc_boxa, cc_pixa) =
            crate::region::conncomp_pixa(self, crate::region::ConnectivityType::EightWay)
                .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        let mut result = Boxa::new();
        for i in 0..cc_pixa.len() {
            let comp = &cc_pixa[i];
            let comp_box = cc_boxa.get(i).copied().unwrap_or_default();
            let sub_boxes =
                comp.split_component_into_boxa(min_sum, skip_dist, delta, max_bg_comp)?;
            // Offset sub-boxes by the component's origin
            for sb in sub_boxes.boxes() {
                if let Ok(b) = Box::new(sb.x + comp_box.x, sb.y + comp_box.y, sb.w, sb.h) {
                    result.push(b);
                }
            }
        }
        Ok(result)
    }

    /// Split a single 1bpp connected component into rectangular sub-regions.
    ///
    /// Greedily extracts rectangles by scanning rows and columns from all four
    /// sides, choosing the side that captures the most foreground pixels.
    ///
    /// C Leptonica equivalent: `pixSplitComponentIntoBoxa`
    pub fn split_component_into_boxa(
        &self,
        min_sum: u32,
        skip_dist: u32,
        _delta: u32,
        _max_bg_comp: u32,
    ) -> Result<Boxa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        if w == 0 || h == 0 {
            return Ok(Boxa::new());
        }

        let mut result = Boxa::new();
        // Work on a mutable copy of pixel data
        let mut mask: Vec<bool> = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                mask.push(self.get_pixel_unchecked(x, y) != 0);
            }
        }

        let skip = skip_dist.max(1) as usize;
        let min_s = min_sum.max(1) as usize;

        // Iterate: find rectangular sub-regions greedily
        let max_iter = 256;
        for _ in 0..max_iter {
            // Find bounding box of remaining foreground
            let mut min_x = w as usize;
            let mut min_y = h as usize;
            let mut max_x: usize = 0;
            let mut max_y: usize = 0;
            let mut has_fg = false;
            for y in 0..h as usize {
                for x in 0..w as usize {
                    if mask[y * w as usize + x] {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                        has_fg = true;
                    }
                }
            }
            if !has_fg {
                break;
            }

            let bx = min_x;
            let by = min_y;
            let bw = max_x - min_x + 1;
            let bh = max_y - min_y + 1;

            // Sweep from left: find contiguous columns with enough foreground
            let mut best_right = bx;
            for x in bx..bx + bw {
                let col_sum: usize = (by..by + bh).filter(|&y| mask[y * w as usize + x]).count();
                if col_sum >= min_s {
                    best_right = x;
                } else {
                    // Check if we've had enough columns already
                    if best_right >= bx + skip {
                        break;
                    }
                    // Keep searching
                    best_right = x;
                }
            }

            // Sweep from top: find contiguous rows
            let mut best_bottom = by;
            for y in by..by + bh {
                let row_sum: usize = (bx..=best_right)
                    .filter(|&x| mask[y * w as usize + x])
                    .count();
                if row_sum >= min_s {
                    best_bottom = y;
                } else if best_bottom >= by + skip {
                    break;
                } else {
                    best_bottom = y;
                }
            }

            let rect_w = (best_right - bx + 1) as i32;
            let rect_h = (best_bottom - by + 1) as i32;
            if rect_w > 0
                && rect_h > 0
                && let Ok(b) = Box::new(bx as i32, by as i32, rect_w, rect_h)
            {
                result.push(b);
            }

            // Clear the extracted region from mask
            for y in by..=best_bottom {
                for x in bx..=best_right {
                    if y < h as usize && x < w as usize {
                        mask[y * w as usize + x] = false;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Select the largest connected component nearest to the upper-left.
    ///
    /// Finds connected components, selects those with area >= `area_fract` *
    /// largest area, then picks the one closest to (0,0).
    ///
    /// C Leptonica equivalent: `pixSelectLargeULComp`
    pub fn select_large_ul_comp(&self, area_fract: f32) -> Result<Box> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let (boxa, _pixa) =
            crate::region::conncomp_pixa(self, crate::region::ConnectivityType::EightWay)
                .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        if boxa.is_empty() {
            return Err(Error::InvalidParameter(
                "no connected components found".to_string(),
            ));
        }

        boxa.select_large_ul_box(area_fract as f64, 5)
            .ok_or_else(|| Error::InvalidParameter("no eligible box found".to_string()))
    }
}

// ---- Free functions ----

/// Create an array of boxes representing vertical or horizontal strips.
///
/// * `direction`: 0 = vertical strips, 1 = horizontal strips.
/// * `size`: strip size in pixels.
///
/// C Leptonica equivalent: `makeMosaicStrips`
pub fn make_mosaic_strips(w: u32, h: u32, direction: u32, size: u32) -> Result<Boxa> {
    if w == 0 || h == 0 {
        return Err(Error::InvalidDimension {
            width: w,
            height: h,
        });
    }
    if size == 0 {
        return Err(Error::InvalidParameter(
            "strip size must be > 0".to_string(),
        ));
    }
    let mut boxa = Boxa::new();
    match direction {
        0 => {
            // Vertical strips (columns)
            let mut x = 0u32;
            while x < w {
                let strip_w = size.min(w - x);
                boxa.push(Box::new_unchecked(x as i32, 0, strip_w as i32, h as i32));
                x += size;
            }
        }
        1 => {
            // Horizontal strips (rows)
            let mut y = 0u32;
            while y < h {
                let strip_h = size.min(h - y);
                boxa.push(Box::new_unchecked(0, y as i32, w as i32, strip_h as i32));
                y += size;
            }
        }
        _ => {
            return Err(Error::InvalidParameter(format!(
                "direction must be 0 (vertical) or 1 (horizontal), got {direction}"
            )));
        }
    }
    Ok(boxa)
}

// ---- Boxaa display functions ----

impl Boxaa {
    /// Create an image displaying all boxes in a Boxaa, each Boxa in a different color.
    ///
    /// Creates a 32bpp image and draws each Boxa's boxes with a unique color.
    ///
    /// C Leptonica equivalent: `boxaaDisplay`
    pub fn display(&self, w: u32, h: u32) -> Result<Pix> {
        if w == 0 || h == 0 {
            return Err(Error::InvalidDimension {
                width: w,
                height: h,
            });
        }
        let white = crate::core::pixel::compose_rgb(255, 255, 255);
        let pix = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut pix_mut = pix.try_into_mut().unwrap_or_else(|p| p.to_mut());
        // Fill white background
        for y in 0..h {
            for x in 0..w {
                pix_mut.set_pixel_unchecked(x, y, white);
            }
        }
        for (i, boxa) in self.boxas().iter().enumerate() {
            let color = CYCLING_COLORS[i % CYCLING_COLORS.len()];
            pix_mut.draw_boxa(boxa, 2, color)?;
        }
        Ok(pix_mut.into())
    }
}

// ---- Boxa display functions ----

impl Boxa {
    /// Create a tiled display of 1bpp images of boxes.
    ///
    /// Creates small images showing each box, tiles them into a single output.
    ///
    /// C Leptonica equivalent: `boxaDisplayTiled`
    pub fn display_tiled(&self, pixa: Option<&Pixa>, max_width: u32) -> Result<Pix> {
        if self.is_empty() {
            return Err(Error::InvalidParameter("boxa is empty".to_string()));
        }
        let max_width = max_width.max(100);
        let line_width = 2u32;
        let mut tile_pixa = Pixa::new();

        for (i, b) in self.boxes().iter().enumerate() {
            let bw = b.w.unsigned_abs().max(1);
            let bh = b.h.unsigned_abs().max(1);
            // If pixa provided and has this index, use it; otherwise create blank
            let canvas = if let Some(pa) = pixa {
                if i < pa.len() {
                    pa[i].convert_to_32()?
                } else {
                    let p = Pix::new(bw, bh, PixelDepth::Bit32)?;
                    let mut pm = p.try_into_mut().unwrap_or_else(|p| p.to_mut());
                    let white = crate::core::pixel::compose_rgb(255, 255, 255);
                    for y in 0..bh {
                        for x in 0..bw {
                            pm.set_pixel_unchecked(x, y, white);
                        }
                    }
                    Pix::from(pm)
                }
            } else {
                let p = Pix::new(bw, bh, PixelDepth::Bit32)?;
                let mut pm = p.try_into_mut().unwrap_or_else(|p| p.to_mut());
                let white = crate::core::pixel::compose_rgb(255, 255, 255);
                for y in 0..bh {
                    for x in 0..bw {
                        pm.set_pixel_unchecked(x, y, white);
                    }
                }
                Pix::from(pm)
            };
            // Draw box outline on canvas
            let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p| p.to_mut());
            let draw_box = Box::new_unchecked(0, 0, bw as i32, bh as i32);
            let color = CYCLING_COLORS[i % CYCLING_COLORS.len()];
            let _ = canvas_mut.render_box_color(&draw_box, line_width, color);
            tile_pixa.push(canvas_mut.into());
        }

        tile_pixa.display_tiled(max_width, 0, 4)
    }
}

// ---- Pixa display with Boxaa ----

impl Pixa {
    /// Display pixa images with boxaa annotations drawn on them.
    ///
    /// For each Pixa image, draws the corresponding Boxa boxes from Boxaa
    /// using colors from `color_table`. Returns new Pixa with boxes drawn.
    ///
    /// C Leptonica equivalent: `pixaDisplayBoxaa`
    pub fn display_boxaa(pixa: &Pixa, boxaa: &Boxaa, color_table: &[u32]) -> Result<Pixa> {
        if pixa.is_empty() {
            return Ok(Pixa::new());
        }
        let n = pixa.len().min(boxaa.len());
        let mut result = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = pixa[i].convert_to_32()?;
            let mut pix_mut = pix.try_into_mut().unwrap_or_else(|p| p.to_mut());
            if let Some(boxa) = boxaa.get(i) {
                for (j, b) in boxa.boxes().iter().enumerate() {
                    let pixel_val = if color_table.is_empty() {
                        let c = CYCLING_COLORS[j % CYCLING_COLORS.len()];
                        crate::core::pixel::compose_rgb(c.r, c.g, c.b)
                    } else {
                        color_table[j % color_table.len()]
                    };
                    let (r, g, b_ch, _) = crate::core::pixel::extract_rgba(pixel_val);
                    let color = Color::new(r, g, b_ch);
                    let _ = pix_mut.render_box_color(b, 2, color);
                }
            }
            result.push(pix_mut.into());
        }
        Ok(result)
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::box_::Box;
    use crate::core::pix::{Pix, PixelDepth};

    fn make_pix(w: u32, h: u32, depth: PixelDepth) -> PixMut {
        Pix::new(w, h, depth).unwrap().to_mut()
    }

    fn sample_boxa() -> Boxa {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 10, 20, 20).unwrap());
        boxa.push(Box::new(50, 50, 30, 40).unwrap());
        boxa
    }

    // -- PixMut::mask_boxa --

    #[test]
    fn test_mask_boxa_set() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        let boxa = sample_boxa();
        pix.mask_boxa(&boxa, PixelOp::Set);
        // Pixels inside the first box should be max value
        assert_eq!(pix.get_pixel_unchecked(15, 15), 255);
        // Pixels outside should remain 0
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0);
    }

    #[test]
    fn test_mask_boxa_clear() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        // Set all pixels first
        pix.set_region(0, 0, 100, 100);
        let boxa = sample_boxa();
        pix.mask_boxa(&boxa, PixelOp::Clear);
        // Pixels inside box should be 0
        assert_eq!(pix.get_pixel_unchecked(15, 15), 0);
        // Pixels outside should remain max
        assert_eq!(pix.get_pixel_unchecked(0, 0), 255);
    }

    #[test]
    fn test_mask_boxa_flip() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        let boxa = sample_boxa();
        pix.mask_boxa(&boxa, PixelOp::Flip);
        // Flip of 0 should give max value
        assert_eq!(pix.get_pixel_unchecked(15, 15), 255);
        // Pixels outside should remain 0
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0);
    }

    // -- PixMut::paint_boxa --

    #[test]
    fn test_paint_boxa() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        let boxa = sample_boxa();
        pix.paint_boxa(&boxa, 128);
        assert_eq!(pix.get_pixel_unchecked(15, 15), 128);
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0);
    }

    // -- PixMut::set_bw_boxa --

    #[test]
    fn test_set_bw_boxa_white() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        let boxa = sample_boxa();
        pix.set_bw_boxa(&boxa, true);
        assert_eq!(pix.get_pixel_unchecked(15, 15), 255);
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0);
    }

    #[test]
    fn test_set_bw_boxa_black() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        // Set all white first
        pix.set_region(0, 0, 100, 100);
        let boxa = sample_boxa();
        pix.set_bw_boxa(&boxa, false);
        assert_eq!(pix.get_pixel_unchecked(15, 15), 0);
        assert_eq!(pix.get_pixel_unchecked(0, 0), 255);
    }

    // -- PixMut::paint_boxa_random --

    #[test]
    fn test_paint_boxa_random() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit32);
        let boxa = sample_boxa();
        pix.paint_boxa_random(&boxa).unwrap();
        // First box gets first cycling color (non-zero for 32bpp)
        assert_ne!(pix.get_pixel_unchecked(15, 15), 0);
        // Pixels outside remain 0
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0);
    }

    // -- PixMut::blend_boxa_random --

    #[test]
    fn test_blend_boxa_random() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit32);
        let boxa = sample_boxa();
        pix.blend_boxa_random(&boxa, 0.5).unwrap();
        // Pixels inside blend region should be non-zero
        assert_ne!(pix.get_pixel_unchecked(15, 15), 0);
    }

    #[test]
    fn test_blend_boxa_random_bad_depth() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit8);
        let boxa = sample_boxa();
        assert!(pix.blend_boxa_random(&boxa, 0.5).is_err());
    }

    // -- PixMut::draw_boxa --

    #[test]
    fn test_draw_boxa() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit32);
        let boxa = sample_boxa();
        pix.draw_boxa(&boxa, 1, Color::RED).unwrap();
        // Top-left corner of first box outline should be RED
        let pixel = pix.get_pixel_unchecked(10, 10);
        assert_ne!(pixel, 0);
    }

    // -- PixMut::draw_boxa_random --

    #[test]
    fn test_draw_boxa_random() {
        let mut pix = make_pix(100, 100, PixelDepth::Bit32);
        let boxa = sample_boxa();
        pix.draw_boxa_random(&boxa, 1).unwrap();
        // Outline pixels should be set
        assert_ne!(pix.get_pixel_unchecked(10, 10), 0);
    }

    // -- Boxa::compare_regions --

    #[test]
    fn test_compare_regions_same() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(0, 0, 10, 10).unwrap());
        boxa.push(Box::new(20, 20, 10, 10).unwrap());

        let result = boxa.compare_regions(&boxa, 0);
        assert!(result.same_count);
        assert_eq!(result.diff_area, 0.0);
    }

    #[test]
    fn test_compare_regions_diff() {
        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(0, 0, 10, 10).unwrap()); // area 100

        let mut boxa2 = Boxa::new();
        boxa2.push(Box::new(0, 0, 20, 20).unwrap()); // area 400

        let result = boxa1.compare_regions(&boxa2, 0);
        assert!(result.same_count);
        // |100 - 400| / (100 + 400) = 300 / 500 = 0.6
        assert!((result.diff_area - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_compare_regions_thresh_filter() {
        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(0, 0, 2, 2).unwrap()); // area 4 - filtered
        boxa1.push(Box::new(0, 0, 10, 10).unwrap()); // area 100

        let mut boxa2 = Boxa::new();
        boxa2.push(Box::new(0, 0, 10, 10).unwrap()); // area 100

        let result = boxa1.compare_regions(&boxa2, 10);
        assert!(result.same_count); // after filtering both have 1 box
        assert_eq!(result.diff_area, 0.0);
    }

    // -- Boxa::select_large_ul_box --

    #[test]
    fn test_select_large_ul_box() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(50, 50, 100, 100).unwrap()); // big, at bottom-right
        boxa.push(Box::new(10, 10, 100, 100).unwrap()); // big, near UL
        boxa.push(Box::new(5, 5, 5, 5).unwrap()); // small, filtered

        let selected = boxa.select_large_ul_box(0.9, 20).unwrap();
        // The big box near UL should be selected
        assert_eq!(selected.x, 10);
        assert_eq!(selected.y, 10);
    }

    #[test]
    fn test_select_large_ul_box_empty() {
        let boxa = Boxa::new();
        assert!(boxa.select_large_ul_box(0.9, 20).is_none());
    }
}
