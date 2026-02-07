//! Pixel statistics functions
//!
//! Functions for computing averages, variances, and other statistics
//! of pixel values in images or rectangular sub-regions.
//! Corresponds to functions in C Leptonica's `pix3.c`.

use super::{Pix, PixelDepth};
use crate::Numa;
use crate::box_::Box;
use crate::error::{Error, Result};

/// Type of pixel value interpretation for average calculations.
///
/// C equivalent: `L_WHITE_IS_MAX` / `L_BLACK_IS_MAX`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelMaxType {
    /// White pixels have maximum value (0xff for 8 bpp, 0xffff for 16 bpp);
    /// black pixels are 0. This is the normal interpretation.
    ///
    /// C equivalent: `L_WHITE_IS_MAX` (value 1)
    WhiteIsMax,

    /// Black pixels get the maximum value; white pixels get 0.
    /// The output is inverted: `avg = max_val - raw_avg`.
    ///
    /// C equivalent: `L_BLACK_IS_MAX` (value 2)
    BlackIsMax,
}

/// Clip a `Box` to the image rectangle `(0, 0, w, h)`.
///
/// Returns `(xstart, ystart, xend, yend, bw, bh)` of the clipped region.
/// Returns `None` if the clipped box has zero area.
fn clip_box_to_rect(bx: Option<&Box>, w: i32, h: i32) -> Option<(i32, i32, i32, i32, i32, i32)> {
    let (xstart, ystart, xend, yend) = match bx {
        Some(b) => {
            let xstart = b.x.max(0);
            let ystart = b.y.max(0);
            let xend = (b.x + b.w).min(w);
            let yend = (b.y + b.h).min(h);
            (xstart, ystart, xend, yend)
        }
        None => (0, 0, w, h),
    };
    let bw = xend - xstart;
    let bh = yend - ystart;
    if bw <= 0 || bh <= 0 {
        None
    } else {
        Some((xstart, ystart, xend, yend, bw, bh))
    }
}

impl Pix {
    /// Compute the average pixel value for each row in a rectangular region.
    ///
    /// Returns a `Numa` of length equal to the number of rows in the region.
    /// Each value is the average pixel value across that row.
    ///
    /// If `pixel_type` is [`PixelMaxType::BlackIsMax`], the result is inverted:
    /// `avg = max_val - raw_avg` (where max_val is 255 for 8 bpp, 65535 for 16 bpp).
    ///
    /// C equivalent: `pixAverageByRow(pix, box, type)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    /// * `pixel_type` - How to interpret pixel values.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8 or 16 bpp, or has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    /// use leptonica_core::pix::statistics::PixelMaxType;
    ///
    /// let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
    /// let na = pix.average_by_row(None, PixelMaxType::WhiteIsMax).unwrap();
    /// assert_eq!(na.len(), 5);
    /// ```
    pub fn average_by_row(&self, region: Option<&Box>, pixel_type: PixelMaxType) -> Result<Numa> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "average_by_row does not support colormapped images".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, bw, _bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(Numa::new()),
        };

        let norm = 1.0 / bw as f64;
        let max_val = d.max_value() as f64;
        let mut na = Numa::with_capacity((yend - ystart) as usize);

        for y in ystart..yend {
            let mut sum = 0.0f64;
            for x in xstart..xend {
                let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                sum += val;
            }
            if pixel_type == PixelMaxType::BlackIsMax {
                sum = bw as f64 * max_val - sum;
            }
            na.push((norm * sum) as f32);
        }

        Ok(na)
    }

    /// Compute the average pixel value for each column in a rectangular region.
    ///
    /// Returns a `Numa` of length equal to the number of columns in the region.
    /// Each value is the average pixel value down that column.
    ///
    /// If `pixel_type` is [`PixelMaxType::BlackIsMax`], the result is inverted:
    /// `avg = max_val - raw_avg`.
    ///
    /// C equivalent: `pixAverageByColumn(pix, box, type)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    /// * `pixel_type` - How to interpret pixel values.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8 or 16 bpp, or has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    /// use leptonica_core::pix::statistics::PixelMaxType;
    ///
    /// let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
    /// let na = pix.average_by_column(None, PixelMaxType::WhiteIsMax).unwrap();
    /// assert_eq!(na.len(), 10);
    /// ```
    pub fn average_by_column(
        &self,
        region: Option<&Box>,
        pixel_type: PixelMaxType,
    ) -> Result<Numa> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "average_by_column does not support colormapped images".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, _bw, bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(Numa::new()),
        };

        let norm = 1.0 / bh as f32;
        let max_val = d.max_value() as f32;
        let mut na = Numa::with_capacity((xend - xstart) as usize);

        for x in xstart..xend {
            let mut sum = 0.0f32;
            for y in ystart..yend {
                let val = self.get_pixel_unchecked(x as u32, y as u32) as f32;
                sum += val;
            }
            if pixel_type == PixelMaxType::BlackIsMax {
                sum = bh as f32 * max_val - sum;
            }
            na.push(norm * sum);
        }

        Ok(na)
    }

    /// Compute the average pixel value in a rectangular region.
    ///
    /// This function computes the average with optional range filtering and
    /// subsampling. Only pixels with values in `[minval, maxval]` are included
    /// in the average.
    ///
    /// C equivalent: `pixAverageInRect(pixs, NULL, box, minval, maxval, subsamp, &ave)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    /// * `minval` - Minimum pixel value to include.
    /// * `maxval` - Maximum pixel value to include.
    /// * `subsamp` - Subsampling factor (>= 1); 1 means every pixel.
    ///
    /// # Returns
    ///
    /// `Ok(Some(average))` if pixels were found in range, `Ok(None)` if all
    /// pixels were filtered out, `Err` on invalid input.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 1, 2, 4, or 8 bpp, or if
    /// it has a colormap, or if `subsamp` < 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let ave = pix.average_in_rect(None, 0, 255, 1).unwrap();
    /// assert_eq!(ave, Some(0.0)); // All pixels are 0
    /// ```
    pub fn average_in_rect(
        &self,
        region: Option<&Box>,
        minval: u32,
        maxval: u32,
        subsamp: u32,
    ) -> Result<Option<f32>> {
        let d = self.depth();
        match d {
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 => {}
            _ => return Err(Error::UnsupportedDepth(d.bits())),
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "average_in_rect does not support colormapped images".to_string(),
            ));
        }
        if subsamp < 1 {
            return Err(Error::InvalidParameter("subsamp must be >= 1".to_string()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, _bw, _bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(None),
        };

        let step = subsamp as i32;
        let mut sum = 0.0f64;
        let mut count = 0u64;

        let mut y = ystart;
        while y < yend {
            let mut x = xstart;
            while x < xend {
                let val = self.get_pixel_unchecked(x as u32, y as u32);
                if val >= minval && val <= maxval {
                    sum += val as f64;
                    count += 1;
                }
                x += step;
            }
            y += step;
        }

        if count == 0 {
            Ok(None)
        } else {
            Ok(Some((sum / count as f64) as f32))
        }
    }

    /// Compute the root-variance (standard deviation) of pixel values in a
    /// rectangular region.
    ///
    /// This is the square root of the variance: `sqrt(E[X^2] - E[X]^2)`.
    ///
    /// C equivalent: `pixVarianceInRect(pix, box, &rootvar)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Returns
    ///
    /// The root-variance (standard deviation) of pixel values.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 1, 2, 4, or 8 bpp, or
    /// has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let rootvar = pix.variance_in_rect(None).unwrap();
    /// assert_eq!(rootvar, 0.0); // All pixels are 0, so no variance
    /// ```
    pub fn variance_in_rect(&self, region: Option<&Box>) -> Result<f32> {
        let d = self.depth();
        match d {
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 => {}
            _ => return Err(Error::UnsupportedDepth(d.bits())),
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "variance_in_rect does not support colormapped images".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, bw, bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(0.0),
        };

        let mut sum1 = 0.0f64;
        let mut sum2 = 0.0f64;

        for y in ystart..yend {
            for x in xstart..xend {
                let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                sum1 += val;
                sum2 += val * val;
            }
        }

        let norm = 1.0 / (bw as f64 * bh as f64);
        let ave = norm * sum1;
        let var = norm * sum2 - ave * ave;
        Ok(var.max(0.0).sqrt() as f32)
    }

    /// Compute the RMS deviation (standard deviation) for each row in a region.
    ///
    /// Returns a `Numa` where each value is the root-variance (standard deviation)
    /// of pixel values in that row, within the specified rectangular region.
    ///
    /// C equivalent: `pixVarianceByRow(pix, box)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8 or 16 bpp, or has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
    /// let na = pix.variance_by_row(None).unwrap();
    /// assert_eq!(na.len(), 5);
    /// ```
    pub fn variance_by_row(&self, region: Option<&Box>) -> Result<Numa> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "variance_by_row does not support colormapped images".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, bw, _bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(Numa::new()),
        };

        let norm = 1.0 / bw as f64;
        let mut na = Numa::with_capacity((yend - ystart) as usize);

        for y in ystart..yend {
            let mut sum1 = 0.0f64;
            let mut sum2 = 0.0f64;
            for x in xstart..xend {
                let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                sum1 += val;
                sum2 += val * val;
            }
            let ave = norm * sum1;
            let var = norm * sum2 - ave * ave;
            na.push(var.max(0.0).sqrt() as f32);
        }

        Ok(na)
    }

    /// Compute the RMS deviation (standard deviation) for each column in a region.
    ///
    /// Returns a `Numa` where each value is the root-variance (standard deviation)
    /// of pixel values in that column, within the specified rectangular region.
    ///
    /// C equivalent: `pixVarianceByColumn(pix, box)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 8 or 16 bpp, or has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
    /// let na = pix.variance_by_column(None).unwrap();
    /// assert_eq!(na.len(), 10);
    /// ```
    pub fn variance_by_column(&self, region: Option<&Box>) -> Result<Numa> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if self.has_colormap() {
            return Err(Error::NotSupported(
                "variance_by_column does not support colormapped images".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, _bw, bh) = match clip_box_to_rect(region, w, h) {
            Some(v) => v,
            None => return Ok(Numa::new()),
        };

        let norm = 1.0 / bh as f64;
        let mut na = Numa::with_capacity((xend - xstart) as usize);

        for x in xstart..xend {
            let mut sum1 = 0.0f64;
            let mut sum2 = 0.0f64;
            for y in ystart..yend {
                let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                sum1 += val;
                sum2 += val * val;
            }
            let ave = norm * sum1;
            let var = norm * sum2 - ave * ave;
            na.push(var.max(0.0).sqrt() as f32);
        }

        Ok(na)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average_by_row_all_zero() {
        let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
        let na = pix.average_by_row(None, PixelMaxType::WhiteIsMax).unwrap();
        assert_eq!(na.len(), 5);
        for i in 0..5 {
            assert_eq!(na.get(i).unwrap(), 0.0);
        }
    }

    #[test]
    fn test_average_by_column_all_zero() {
        let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
        let na = pix
            .average_by_column(None, PixelMaxType::WhiteIsMax)
            .unwrap();
        assert_eq!(na.len(), 10);
    }

    #[test]
    fn test_average_by_row_black_is_max() {
        let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
        let na = pix.average_by_row(None, PixelMaxType::BlackIsMax).unwrap();
        assert_eq!(na.len(), 5);
        // All pixels are 0, black_is_max inverts: 255 - 0 = 255
        for i in 0..5 {
            assert_eq!(na.get(i).unwrap(), 255.0);
        }
    }

    #[test]
    fn test_average_in_rect_basic() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..10u32 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pix_mut.into();

        let ave = pix.average_in_rect(None, 0, 255, 1).unwrap();
        assert_eq!(ave, Some(100.0));
    }

    #[test]
    fn test_average_in_rect_range_filter() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let ave = pix.average_in_rect(None, 1, 255, 1).unwrap();
        // All pixels are 0, which is below minval=1
        assert_eq!(ave, None);
    }

    #[test]
    fn test_variance_in_rect_constant() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..10u32 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, 42);
            }
        }
        let pix: Pix = pix_mut.into();

        let rootvar = pix.variance_in_rect(None).unwrap();
        assert!(rootvar < 0.001);
    }

    #[test]
    fn test_variance_by_row_constant() {
        let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..5u32 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }
        let pix: Pix = pix_mut.into();

        let na = pix.variance_by_row(None).unwrap();
        assert_eq!(na.len(), 5);
        for i in 0..5 {
            assert!(na.get(i).unwrap() < 0.001);
        }
    }

    #[test]
    fn test_variance_by_column_constant() {
        let pix = Pix::new(10, 5, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..5u32 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, 128);
            }
        }
        let pix: Pix = pix_mut.into();

        let na = pix.variance_by_column(None).unwrap();
        assert_eq!(na.len(), 10);
        for i in 0..10 {
            assert!(na.get(i).unwrap() < 0.001);
        }
    }

    #[test]
    fn test_wrong_depth_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.average_by_row(None, PixelMaxType::WhiteIsMax).is_err());
        assert!(
            pix.average_by_column(None, PixelMaxType::WhiteIsMax)
                .is_err()
        );
        assert!(pix.average_in_rect(None, 0, 255, 1).is_err());
        assert!(pix.variance_in_rect(None).is_err());
        assert!(pix.variance_by_row(None).is_err());
        assert!(pix.variance_by_column(None).is_err());
    }
}
