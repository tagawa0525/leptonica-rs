//! Box drawing, masking, and region comparison operations
//!
//! Functions for painting, masking, and drawing boxes on images,
//! and for structural comparison and selection of box arrays.
//!
//! C Leptonica equivalents: boxfunc3.c

use crate::box_::{Box, Boxa};
use crate::error::{Error, Result};
use crate::pix::PixMut;
use crate::pix::graphics::{Color, PixelOp};

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
            let val = crate::color::compose_rgb(color.r, color.g, color.b);
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
                    let (r, g, b_ch, a) = crate::color::extract_rgba(pixel);
                    let nr = (r as f32 * (1.0 - fract) + blend_r * fract) as u8;
                    let ng = (g as f32 * (1.0 - fract) + blend_g * fract) as u8;
                    let nb = (b_ch as f32 * (1.0 - fract) + blend_b * fract) as u8;
                    self.set_pixel_unchecked(px, py, crate::color::compose_rgba(nr, ng, nb, a));
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

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_::Box;
    use crate::pix::{Pix, PixelDepth};

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
