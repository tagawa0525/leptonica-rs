//! Measurement functions for binary (1bpp) images
//!
//! Functions for computing area, perimeter, and overlap ratios.
//! Corresponds to functions in C Leptonica's `pix5.c`.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};

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
        let mut nfg = 0u32;
        let mut nboundary = 0u32;
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
        Ok(nboundary as f32 / nfg as f32)
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
        assert!(ratio > 0.0 && ratio <= 1.0);
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
