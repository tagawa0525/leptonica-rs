//! Measurement functions for binary (1bpp) images
//!
//! Functions for computing area, perimeter, and overlap ratios.
//! Corresponds to functions in C Leptonica's `pix5.c`.

use super::{Pix, PixelDepth};
use crate::core::box_::{Boxa, SizeRelation};
use crate::core::error::{Error, Result};
use crate::core::pixa::Pixa;

impl Pix {
    /// Compute the fraction of foreground pixels in a 1bpp image.
    ///
    /// Returns `(fg count) / (w * h)`. Returns 0.0 if the image is empty.
    ///
    /// C equivalent: `pixFindAreaFraction()` in `pix5.c`
    pub fn find_area_fraction(&self) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        let total = (w as u64) * (h as u64);
        if total == 0 {
            return Ok(0.0);
        }
        let mut count = 0u64;
        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) != 0 {
                    count += 1;
                }
            }
        }
        Ok(count as f32 / total as f32)
    }

    /// Compute the ratio of boundary pixels to all foreground pixels.
    ///
    /// A boundary pixel is a foreground pixel that has at least one background
    /// 8-neighbor. This is equivalent to `nbound / nfg`.
    /// Returns 0.0 if there are no foreground pixels.
    ///
    /// C equivalent: `pixFindPerimToAreaRatio()` in `pix5.c`
    pub fn find_perim_to_area_ratio(&self) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let mut nfg = 0u64;
        let mut nboundary = 0u64;
        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    nfg += 1;
                    // Check all 8 neighbors; boundary if any is bg or out-of-bounds
                    let is_interior = [
                        (x - 1, y - 1),
                        (x, y - 1),
                        (x + 1, y - 1),
                        (x - 1, y),
                        (x + 1, y),
                        (x - 1, y + 1),
                        (x, y + 1),
                        (x + 1, y + 1),
                    ]
                    .iter()
                    .all(|&(nx, ny)| {
                        nx >= 0
                            && ny >= 0
                            && nx < w
                            && ny < h
                            && self.get_pixel_unchecked(nx as u32, ny as u32) != 0
                    });
                    if !is_interior {
                        nboundary += 1;
                    }
                }
            }
        }
        if nfg == 0 {
            return Ok(0.0);
        }
        Ok((nboundary as f64 / nfg as f64) as f32)
    }

    /// Compute the Jaccard overlap fraction between two 1bpp images.
    ///
    /// Places `other` at offset `(x2, y2)` relative to `self`, computes
    /// `intersection / union` of their foreground pixels.
    /// Returns `(ratio, n_overlap)`.
    ///
    /// C equivalent: `pixFindOverlapFraction()` in `pix5.c`
    pub fn find_overlap_fraction(&self, other: &Pix, x2: i32, y2: i32) -> Result<(f32, u32)> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if other.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(other.depth().bits()));
        }
        let w1 = self.width() as i32;
        let h1 = self.height() as i32;
        let w2 = other.width() as i32;
        let h2 = other.height() as i32;

        let mut nintersect = 0u32;
        let mut nunion = 0u32;

        // Count fg pixels in self
        for y in 0..h1 {
            for x in 0..w1 {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    nunion += 1;
                }
            }
        }
        // For each fg pixel in other, check overlap with self
        for oy in 0..h2 {
            for ox in 0..w2 {
                if other.get_pixel_unchecked(ox as u32, oy as u32) != 0 {
                    let sx = x2 + ox;
                    let sy = y2 + oy;
                    if sx >= 0 && sy >= 0 && sx < w1 && sy < h1 {
                        if self.get_pixel_unchecked(sx as u32, sy as u32) != 0 {
                            nintersect += 1;
                        } else {
                            nunion += 1; // in other but not in self
                        }
                    } else {
                        nunion += 1; // outside self, add to union
                    }
                }
            }
        }

        if nunion == 0 {
            return Ok((0.0, 0));
        }
        Ok((nintersect as f32 / nunion as f32, nintersect))
    }

    /// Find ratio of area to perimeter for a 1bpp connected component.
    ///
    /// Counts foreground pixels (area) and boundary pixels (perimeter).
    /// Returns area / perimeter.
    ///
    /// C equivalent: `pixFindAreaPerimRatio()` in `pix5.c`
    pub fn find_area_perim_ratio(&self) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let mut nfg = 0u64;
        let mut nboundary = 0u64;

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    nfg += 1;
                    let is_interior = [
                        (x - 1, y - 1),
                        (x, y - 1),
                        (x + 1, y - 1),
                        (x - 1, y),
                        (x + 1, y),
                        (x - 1, y + 1),
                        (x, y + 1),
                        (x + 1, y + 1),
                    ]
                    .iter()
                    .all(|&(nx, ny)| {
                        nx >= 0
                            && ny >= 0
                            && nx < w
                            && ny < h
                            && self.get_pixel_unchecked(nx as u32, ny as u32) != 0
                    });
                    if !is_interior {
                        nboundary += 1;
                    }
                }
            }
        }
        if nboundary == 0 {
            return Ok(0.0);
        }
        Ok(nfg as f32 / nboundary as f32)
    }

    /// Find ratio of perimeter to sqrt(area) for a 1bpp connected component.
    ///
    /// C equivalent: `pixFindPerimSizeRatio()` in `pix5.c`
    pub fn find_perim_size_ratio(&self) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let mut nfg = 0u64;
        let mut nboundary = 0u64;

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    nfg += 1;
                    let is_interior = [
                        (x - 1, y - 1),
                        (x, y - 1),
                        (x + 1, y - 1),
                        (x - 1, y),
                        (x + 1, y),
                        (x - 1, y + 1),
                        (x, y + 1),
                        (x + 1, y + 1),
                    ]
                    .iter()
                    .all(|&(nx, ny)| {
                        nx >= 0
                            && ny >= 0
                            && nx < w
                            && ny < h
                            && self.get_pixel_unchecked(nx as u32, ny as u32) != 0
                    });
                    if !is_interior {
                        nboundary += 1;
                    }
                }
            }
        }
        if nfg == 0 {
            return Ok(0.0);
        }
        Ok(nboundary as f32 / (nfg as f32).sqrt())
    }

    /// Find fraction of 1-pixels in `self` that are under `mask`.
    ///
    /// Both must be 1bpp. Counts pixels in self AND mask, divides by
    /// count in self.
    ///
    /// C equivalent: `pixFindAreaFractionMasked()` in `pix5.c`
    pub fn find_area_fraction_masked(&self, mask: &Pix) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }

        let w = self.width().min(mask.width());
        let h = self.height().min(mask.height());
        let mut nself = 0u64;
        let mut nboth = 0u64;

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) != 0 {
                    nself += 1;
                    if mask.get_pixel_unchecked(x, y) != 0 {
                        nboth += 1;
                    }
                }
            }
        }
        // Also count self pixels outside the mask overlap region
        for y in 0..self.height() {
            for x in 0..self.width() {
                if (x >= w || y >= h) && self.get_pixel_unchecked(x, y) != 0 {
                    nself += 1;
                }
            }
        }
        if nself == 0 {
            return Ok(0.0);
        }
        Ok(nboth as f32 / nself as f32)
    }

    /// Check if a 1bpp connected component is roughly rectangular.
    ///
    /// Returns true if the fraction of foreground pixels exceeds `min_fract`.
    ///
    /// C equivalent: `pixConformsToRectangle()` in `pix5.c`
    pub fn conforms_to_rectangle(&self, min_fract: f32) -> Result<bool> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let frac = self.find_area_fraction()?;
        Ok(frac >= min_fract)
    }

    /// Find connected components that are approximately rectangular.
    ///
    /// For 1bpp. Finds CCs, tests each for rectangularity using
    /// `conforms_to_rectangle`, returns bounding boxes of those passing.
    ///
    /// C equivalent: `pixFindRectangleComps()` in `pix5.c`
    pub fn find_rectangle_comps(&self, min_fract: f32) -> Result<Boxa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let (cc_boxa, cc_pixa) =
            crate::region::conncomp_pixa(self, crate::region::ConnectivityType::EightWay)
                .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        let mut result = Boxa::new();
        for i in 0..cc_pixa.len() {
            let comp = &cc_pixa[i];
            if comp.conforms_to_rectangle(min_fract)?
                && let Some(b) = cc_boxa.get(i)
            {
                result.push(*b);
            }
        }
        Ok(result)
    }

    /// Extract all near-rectangular connected components as a Pixa.
    ///
    /// For 1bpp. Finds CCs, returns those that conform to a rectangle
    /// (area fraction >= `min_fract` of bounding box).
    ///
    /// C equivalent: `pixExtractRectangularRegions()` in `pix5.c`
    pub fn extract_rectangular_regions(&self, min_fract: f32) -> Result<Pixa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let (cc_boxa, cc_pixa) =
            crate::region::conncomp_pixa(self, crate::region::ConnectivityType::EightWay)
                .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        let mut result = Pixa::new();
        for i in 0..cc_pixa.len() {
            let comp = &cc_pixa[i];
            if comp.conforms_to_rectangle(min_fract)? {
                let b = cc_boxa.get(i).copied().unwrap_or_default();
                result.push_with_box(comp.deep_clone(), b);
            }
        }
        Ok(result)
    }

    /// Select connected components by size criterion.
    ///
    /// Returns (mask, boxa) where only components matching the size criterion
    /// are included.
    ///
    /// * `width` - target width threshold
    /// * `height` - target height threshold
    /// * `connectivity` - 4 or 8
    /// * `relation` - size comparison relation
    ///
    /// C equivalent: `pixSelectComponentBySize()` in `pix5.c`
    pub fn select_component_by_size(
        &self,
        width: u32,
        height: u32,
        connectivity: u32,
        relation: SizeRelation,
    ) -> Result<(Pix, Boxa)> {
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
        let (cc_boxa, cc_pixa) = crate::region::conncomp_pixa(self, conn)
            .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        let w = self.width();
        let h = self.height();
        let pixd = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut pixd_mut = pixd.try_into_mut().unwrap_or_else(|p| p.to_mut());
        let mut result_boxa = Boxa::new();

        for i in 0..cc_pixa.len() {
            let b = cc_boxa.get(i).copied().unwrap_or_default();
            let bw = b.w as u32;
            let bh = b.h as u32;
            let matches = match relation {
                SizeRelation::LessThan => bw < width && bh < height,
                SizeRelation::LessThanOrEqual => bw <= width && bh <= height,
                SizeRelation::GreaterThan => bw > width || bh > height,
                SizeRelation::GreaterThanOrEqual => bw >= width || bh >= height,
            };
            if matches {
                // Copy component pixels into output at original position
                let comp = &cc_pixa[i];
                let ox = b.x.max(0) as u32;
                let oy = b.y.max(0) as u32;
                let cw = comp.width();
                let ch = comp.height();
                for cy in 0..ch {
                    for cx in 0..cw {
                        if comp.get_pixel_unchecked(cx, cy) != 0 {
                            let dx = ox + cx;
                            let dy = oy + cy;
                            if dx < w && dy < h {
                                pixd_mut.set_pixel_unchecked(dx, dy, 1);
                            }
                        }
                    }
                }
                result_boxa.push(b);
            }
        }

        Ok((pixd_mut.into(), result_boxa))
    }

    /// Filter connected components, keeping only those matching size criteria.
    ///
    /// Returns a new 1bpp image with only matching components.
    ///
    /// C equivalent: `pixFilterComponentBySize()` in `pix5.c`
    pub fn filter_component_by_size(
        &self,
        width: u32,
        height: u32,
        connectivity: u32,
        relation: SizeRelation,
    ) -> Result<Pix> {
        let (pix, _boxa) = self.select_component_by_size(width, height, connectivity, relation)?;
        Ok(pix)
    }

    /// Create a set of non-overlapping rectangles that cover all foreground.
    ///
    /// For 1bpp. Iteratively expands bounding boxes of connected components
    /// until convergence, grouping nearby foreground into covering rectangles.
    ///
    /// * `distance` - expansion distance per iteration for merging nearby components.
    ///
    /// C equivalent: `pixMakeCoveringOfRectangles()` in `pix5.c`
    pub fn make_covering_of_rectangles(&self, distance: u32) -> Result<Boxa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();

        // Start with CC bounding boxes
        let (initial_boxa, _pixa) =
            crate::region::conncomp_pixa(self, crate::region::ConnectivityType::EightWay)
                .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        if initial_boxa.is_empty() {
            return Ok(Boxa::new());
        }

        let dist = distance as i32;
        let max_iters = 20;
        let mut current_boxa = initial_boxa;

        for _ in 0..max_iters {
            // Paint expanded bounding boxes into a mask
            let canvas = Pix::new(w, h, PixelDepth::Bit1)?;
            let mut canvas_mut = canvas.try_into_mut().unwrap_or_else(|p| p.to_mut());
            for b in current_boxa.boxes() {
                let x0 = (b.x - dist).max(0) as u32;
                let y0 = (b.y - dist).max(0) as u32;
                let x1 = ((b.x + b.w + dist) as u32).min(w);
                let y1 = ((b.y + b.h + dist) as u32).min(h);
                for y in y0..y1 {
                    for x in x0..x1 {
                        canvas_mut.set_pixel_unchecked(x, y, 1);
                    }
                }
            }
            let canvas_pix: Pix = canvas_mut.into();

            // Extract new CCs from the expanded mask
            let (new_boxa, _) = crate::region::conncomp_pixa(
                &canvas_pix,
                crate::region::ConnectivityType::EightWay,
            )
            .map_err(|e| Error::InvalidParameter(e.to_string()))?;

            // Check convergence
            if new_boxa.len() == current_boxa.len() {
                return Ok(new_boxa);
            }
            current_boxa = new_boxa;
        }

        Ok(current_boxa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Pix::find_area_fraction --

    #[test]
    fn test_find_area_fraction_half() {
        // 4x4 image with exactly half the pixels ON
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for x in 0..4u32 {
                for y in 0..2u32 {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(pm)
        };
        let frac = pix.find_area_fraction().unwrap();
        assert!((frac - 0.5).abs() < 1e-6, "expected 0.5, got {frac}");
    }

    #[test]
    fn test_find_area_fraction_all_off() {
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let frac = pix.find_area_fraction().unwrap();
        assert_eq!(frac, 0.0);
    }

    // -- Pix::find_perim_to_area_ratio --

    #[test]
    fn test_find_perim_to_area_ratio_solid_block() {
        // 5x5 solid block: interior pixels are (1..=3)x(1..=3) = 9 pixels
        // boundary pixels = 25 - 9 = 16
        // ratio = 16/25 = 0.64
        let pix = {
            let base = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..5u32 {
                for x in 0..5u32 {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(pm)
        };
        let ratio = pix.find_perim_to_area_ratio().unwrap();
        // boundary = pixels touching bg; for a solid block the boundaries are
        // the outer ring (16 pixels), ratio = 16/25
        assert!(
            (ratio - 16.0_f32 / 25.0).abs() < 1e-6,
            "expected 16/25 = 0.64, got {ratio}"
        );
    }

    #[test]
    fn test_find_perim_to_area_ratio_no_fg() {
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let ratio = pix.find_perim_to_area_ratio().unwrap();
        assert_eq!(ratio, 0.0);
    }

    // -- Pix::find_overlap_fraction --

    #[test]
    fn test_find_overlap_fraction_full_overlap() {
        // Two identical 4x4 all-ON images overlapping at (0,0)
        let make_all_on = || {
            let base = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..4u32 {
                for x in 0..4u32 {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(pm)
        };
        let a = make_all_on();
        let b = make_all_on();
        let (ratio, noverlap) = a.find_overlap_fraction(&b, 0, 0).unwrap();
        assert!((ratio - 1.0).abs() < 1e-6);
        assert_eq!(noverlap, 16);
    }

    #[test]
    fn test_find_overlap_fraction_no_overlap() {
        // Two 4x4 images placed far apart (no overlap)
        let make_all_on = || {
            let base = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..4u32 {
                for x in 0..4u32 {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(pm)
        };
        let a = make_all_on();
        let b = make_all_on();
        // Place b completely outside a (offset beyond a's size)
        let (ratio, noverlap) = a.find_overlap_fraction(&b, 10, 10).unwrap();
        assert_eq!(ratio, 0.0);
        assert_eq!(noverlap, 0);
    }
}
