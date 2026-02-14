//! Pixel extraction functions
//!
//! Functions for extracting pixel values along lines and other geometric paths.
//! Corresponds to functions in C Leptonica's `pix5.c`.

use super::{Pix, PixelDepth};
use crate::Numa;
use crate::error::{Error, Result};

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
                } else {
                    if x1 > x2 { 1 } else { -1 }
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
}
