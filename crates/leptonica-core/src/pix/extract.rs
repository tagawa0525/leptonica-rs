//! Pixel extraction functions
//!
//! Functions for extracting pixel values along lines and other geometric paths.
//! Corresponds to functions in C Leptonica's `pix5.c`.

use super::{Pix, PixelDepth};
use crate::Numa;
use crate::error::{Error, Result};

/// Direction for profile scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileDirection {
    /// Scan along horizontal lines (rows)
    Horizontal,
    /// Scan along vertical lines (columns)
    Vertical,
}

impl Pix {
    /// Extract pixel values along a line from `(x1, y1)` to `(x2, y2)`.
    ///
    /// Uses Bresenham-like line drawing to determine which pixels to sample.
    /// For horizontal or near-horizontal lines, points are extracted left to right.
    /// For vertical or near-vertical lines, points are extracted top to bottom.
    ///
    /// The `factor` parameter controls subsampling: a factor of 1 extracts
    /// every point, 2 extracts every other point, etc.
    ///
    /// C equivalent: `pixExtractOnLine(pixs, x1, y1, x2, y2, factor)`
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` - One endpoint of the line
    /// * `x2`, `y2` - Other endpoint of the line
    /// * `factor` - Sampling factor (>= 1); 1 means every pixel
    ///
    /// # Returns
    ///
    /// A `Numa` containing the extracted pixel values.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 1 or 8 bpp, or if the image
    /// has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// // Extract along a horizontal line at y=50
    /// let na = pix.extract_on_line(0, 50, 99, 50, 1).unwrap();
    /// assert_eq!(na.len(), 100);
    /// ```
    pub fn extract_on_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, factor: i32) -> Result<Numa> {
        let d = self.depth();
        if d != PixelDepth::Bit1 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "extract_on_line does not support colormapped images".to_string(),
            ));
        }

        if factor < 1 {
            return Err(Error::InvalidParameter(format!(
                "factor must be >= 1, got {}",
                factor
            )));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;

        // Clip endpoints to image bounds
        let x1 = x1.clamp(0, w - 1);
        let x2 = x2.clamp(0, w - 1);
        let y1 = y1.clamp(0, h - 1);
        let y2 = y2.clamp(0, h - 1);

        // Single point case
        if x1 == x2 && y1 == y2 {
            let val = self.get_pixel(x1 as u32, y1 as u32).unwrap_or(0);
            let mut na = Numa::with_capacity(1);
            na.push(val as f32);
            return Ok(na);
        }

        let mut na = Numa::new();

        if y1 == y2 {
            // Horizontal line: extract left to right
            let xmin = x1.min(x2);
            let xmax = x1.max(x2);
            let mut i = xmin;
            while i <= xmax {
                let val = self.get_pixel(i as u32, y1 as u32).unwrap_or(0);
                na.push(val as f32);
                i += factor;
            }
        } else if x1 == x2 {
            // Vertical line: extract top to bottom
            let ymin = y1.min(y2);
            let ymax = y1.max(y2);
            let mut i = ymin;
            while i <= ymax {
                let val = self.get_pixel(x1 as u32, i as u32).unwrap_or(0);
                na.push(val as f32);
                i += factor;
            }
        } else {
            // Oblique line
            let slope = (y2 - y1) as f64 / (x2 - x1) as f64;

            if slope.abs() < 1.0 {
                // Quasi-horizontal: step along x
                let xmin = x1.min(x2);
                let xmax = x1.max(x2);
                let (ymin, _ymax) = if xmin == x1 { (y1, y2) } else { (y2, y1) };
                // Generate Bresenham-like points
                let npts = (xmax - xmin) + 1;
                let sign = if xmin == x1 {
                    if x2 > x1 { 1 } else { -1 }
                } else if x1 > x2 {
                    1
                } else {
                    -1
                };
                // slope for the sorted direction
                let sorted_slope = (if xmin == x1 { y2 - y1 } else { y1 - y2 }) as f32
                    / (xmax - xmin).abs() as f32;
                let _ = sign; // sign not needed; we go from xmin to xmax

                let mut i = 0;
                while i < npts {
                    let x = xmin + i;
                    let y = (ymin as f32 + i as f32 * sorted_slope + 0.5) as i32;
                    let val = self.get_pixel(x as u32, y as u32).unwrap_or(0);
                    na.push(val as f32);
                    i += factor;
                }
            } else {
                // Quasi-vertical: step along y
                let ymin = y1.min(y2);
                let ymax = y1.max(y2);
                let (xmin, _xmax) = if ymin == y1 { (x1, x2) } else { (x2, x1) };
                let npts = (ymax - ymin) + 1;
                let sorted_slope = (if ymin == y1 { x2 - x1 } else { x1 - x2 }) as f32
                    / (ymax - ymin).abs() as f32;

                let mut i = 0;
                while i < npts {
                    let x = (xmin as f32 + i as f32 * sorted_slope + 0.5) as i32;
                    let y = ymin + i;
                    let val = self.get_pixel(x as u32, y as u32).unwrap_or(0);
                    na.push(val as f32);
                    i += factor;
                }
            }
        }

        Ok(na)
    }

    /// Sort each row of an 8bpp image from minimum to maximum value.
    ///
    /// Uses a 256-bin histogram for each row (O(n) time).
    /// The input must be 8bpp without a colormap.
    ///
    /// C equivalent: `pixRankRowTransform()` in `pix5.c`
    pub fn rank_row_transform(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if self.has_colormap() {
            return Err(Error::InvalidParameter(
                "rank_row_transform: image must not have a colormap".into(),
            ));
        }
        let w = self.width();
        let h = self.height();
        let pixd_base = Pix::new(w, h, PixelDepth::Bit8)
            .map_err(|e| Error::InvalidParameter(format!("cannot create pixd: {e}")))?;
        let mut pixd = pixd_base.try_into_mut().unwrap();
        pixd.set_resolution(self.xres(), self.yres());
        for i in 0..h {
            let mut histo = [0u32; 256];
            for j in 0..w {
                histo[self.get_pixel_unchecked(j, i) as usize] += 1;
            }
            let mut j = 0u32;
            for (m, &count) in histo.iter().enumerate() {
                for _ in 0..count {
                    pixd.set_pixel_unchecked(j, i, m as u32);
                    j += 1;
                }
            }
        }
        Ok(pixd.into())
    }

    /// Sort each column of an 8bpp image from minimum to maximum value.
    ///
    /// Uses a 256-bin histogram for each column (O(n) time).
    /// The input must be 8bpp without a colormap.
    ///
    /// C equivalent: `pixRankColumnTransform()` in `pix5.c`
    pub fn rank_column_transform(&self) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if self.has_colormap() {
            return Err(Error::InvalidParameter(
                "rank_column_transform: image must not have a colormap".into(),
            ));
        }
        let w = self.width();
        let h = self.height();
        let pixd_base = Pix::new(w, h, PixelDepth::Bit8)
            .map_err(|e| Error::InvalidParameter(format!("cannot create pixd: {e}")))?;
        let mut pixd = pixd_base.try_into_mut().unwrap();
        pixd.set_resolution(self.xres(), self.yres());
        for j in 0..w {
            let mut histo = [0u32; 256];
            for i in 0..h {
                histo[self.get_pixel_unchecked(j, i) as usize] += 1;
            }
            let mut i = 0u32;
            for (m, &count) in histo.iter().enumerate() {
                for _ in 0..count {
                    pixd.set_pixel_unchecked(j, i, m as u32);
                    i += 1;
                }
            }
        }
        Ok(pixd.into())
    }

    /// Compute the average pixel intensity profile along rows or columns.
    ///
    /// For `dir = Horizontal`: scans rows `first..=last` (step `factor2`),
    /// averaging a fraction `fract` of each row (centred) with subsampling `factor1`.
    /// For `dir = Vertical`: analogous along columns.
    ///
    /// The returned `Numa` has `delta = factor2`.
    ///
    /// C equivalent: `pixAverageIntensityProfile()` in `pix5.c`
    pub fn average_intensity_profile(
        &self,
        fract: f32,
        dir: ProfileDirection,
        first: u32,
        last: u32,
        factor1: u32,
        factor2: u32,
    ) -> Result<Numa> {
        if fract < 0.0 || fract > 1.0 {
            return Err(Error::InvalidParameter(
                "fract must be in [0.0, 1.0]".into(),
            ));
        }
        if last < first {
            return Err(Error::InvalidParameter("last must be >= first".into()));
        }
        let f1 = factor1.max(1);
        let f2 = factor2.max(1);
        let w = self.width();
        let h = self.height();
        let mut nad = Numa::new();
        nad.set_parameters(0.0, f2 as f32);
        match dir {
            ProfileDirection::Horizontal => {
                let start = (0.5 * (1.0 - fract) * w as f32) as i32;
                let end = w as i32 - start - 1;
                let last_clamped = last.min(h - 1);
                let mut i = first;
                while i <= last_clamped {
                    let ave = self.average_on_line(start, i as i32, end, i as i32, f1 as i32)?;
                    nad.push(ave);
                    i += f2;
                }
            }
            ProfileDirection::Vertical => {
                let start = (0.5 * (1.0 - fract) * h as f32) as i32;
                let end = h as i32 - start - 1;
                let last_clamped = last.min(w - 1);
                let mut j = first;
                while j <= last_clamped {
                    let ave = self.average_on_line(j as i32, start, j as i32, end, f1 as i32)?;
                    nad.push(ave);
                    j += f2;
                }
            }
        }
        Ok(nad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_horizontal_line() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for x in 0..100u32 {
            pix_mut.set_pixel_unchecked(x, 50, x);
        }
        let pix: Pix = pix_mut.into();

        let na = pix.extract_on_line(0, 50, 99, 50, 1).unwrap();
        assert_eq!(na.len(), 100);
        assert_eq!(na.get(0).unwrap(), 0.0);
        assert_eq!(na.get(50).unwrap(), 50.0);
        assert_eq!(na.get(99).unwrap(), 99.0);
    }

    #[test]
    fn test_extract_vertical_line() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..100u32 {
            pix_mut.set_pixel_unchecked(50, y, y);
        }
        let pix: Pix = pix_mut.into();

        let na = pix.extract_on_line(50, 0, 50, 99, 1).unwrap();
        assert_eq!(na.len(), 100);
        assert_eq!(na.get(0).unwrap(), 0.0);
        assert_eq!(na.get(99).unwrap(), 99.0);
    }

    #[test]
    fn test_extract_single_point() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(5, 5, 42);
        let pix: Pix = pix_mut.into();

        let na = pix.extract_on_line(5, 5, 5, 5, 1).unwrap();
        assert_eq!(na.len(), 1);
        assert_eq!(na.get(0).unwrap(), 42.0);
    }

    #[test]
    fn test_extract_with_subsampling() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let na = pix.extract_on_line(0, 50, 99, 50, 2).unwrap();
        assert_eq!(na.len(), 50); // 0, 2, 4, ..., 98 = 50 points
    }

    #[test]
    fn test_extract_wrong_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.extract_on_line(0, 0, 9, 0, 1).is_err());
    }

    // -- Pix::rank_row_transform --

    #[test]
    fn test_rank_row_transform_basic() {
        // 1×4 image: pixels [3,1,4,2] → sorted [1,2,3,4]
        let pix = {
            let base = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(0, 0, 3);
            pm.set_pixel_unchecked(1, 0, 1);
            pm.set_pixel_unchecked(2, 0, 4);
            pm.set_pixel_unchecked(3, 0, 2);
            Pix::from(pm)
        };
        let result = pix.rank_row_transform().unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(1, 0), Some(2));
        assert_eq!(result.get_pixel(2, 0), Some(3));
        assert_eq!(result.get_pixel(3, 0), Some(4));
    }

    // -- Pix::rank_column_transform --

    #[test]
    fn test_rank_column_transform_basic() {
        // 4×1 image (single column of 4 rows): pixels [3,1,4,2] → sorted [1,2,3,4]
        let pix = {
            let base = Pix::new(1, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            pm.set_pixel_unchecked(0, 0, 3);
            pm.set_pixel_unchecked(0, 1, 1);
            pm.set_pixel_unchecked(0, 2, 4);
            pm.set_pixel_unchecked(0, 3, 2);
            Pix::from(pm)
        };
        let result = pix.rank_column_transform().unwrap();
        assert_eq!(result.get_pixel(0, 0), Some(1));
        assert_eq!(result.get_pixel(0, 1), Some(2));
        assert_eq!(result.get_pixel(0, 2), Some(3));
        assert_eq!(result.get_pixel(0, 3), Some(4));
    }

    // -- Pix::average_intensity_profile --

    #[test]
    fn test_average_intensity_profile_horizontal() {
        use crate::pix::extract::ProfileDirection;
        // Uniform 8bpp image - profile should be constant
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..10u32 {
                for x in 0..10u32 {
                    pm.set_pixel_unchecked(x, y, 100);
                }
            }
            Pix::from(pm)
        };
        let profile = pix
            .average_intensity_profile(1.0, ProfileDirection::Horizontal, 0, 9, 1, 1)
            .unwrap();
        assert_eq!(profile.len(), 10);
        for i in 0..10 {
            let v = profile.get(i).unwrap();
            assert!((v - 100.0).abs() < 1.0, "row {i}: expected ~100, got {v}");
        }
    }

    #[test]
    fn test_average_intensity_profile_vertical() {
        use crate::pix::extract::ProfileDirection;
        // Uniform 8bpp image - vertical profile should also be constant
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..10u32 {
                for x in 0..10u32 {
                    pm.set_pixel_unchecked(x, y, 80);
                }
            }
            Pix::from(pm)
        };
        let profile = pix
            .average_intensity_profile(1.0, ProfileDirection::Vertical, 0, 9, 1, 1)
            .unwrap();
        assert_eq!(profile.len(), 10);
        for j in 0..10 {
            let v = profile.get(j).unwrap();
            assert!((v - 80.0).abs() < 1.0, "col {j}: expected ~80, got {v}");
        }
    }

    #[test]
    fn test_average_intensity_profile_partial_fract() {
        use crate::pix::extract::ProfileDirection;
        // 10×10 image: left half = 0, right half = 200
        // fract = 0.5 centres on the middle 5 columns (2..=6 for w=10: start=2, end=7)
        // The middle 5 columns include both halves so average is 100.
        let pix = {
            let base = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            for y in 0..10u32 {
                for x in 0..10u32 {
                    pm.set_pixel_unchecked(x, y, if x < 5 { 0 } else { 200 });
                }
            }
            Pix::from(pm)
        };
        let profile = pix
            .average_intensity_profile(1.0, ProfileDirection::Horizontal, 0, 9, 1, 1)
            .unwrap();
        // With fract=1.0 the full row is used, so average over left(0)+right(200) = 100
        for i in 0..10 {
            let v = profile.get(i).unwrap();
            assert!((v - 100.0).abs() < 1.0, "row {i}: expected ~100, got {v}");
        }
    }
}
