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
    /// Count the foreground (non-zero) pixels in the image.
    ///
    /// For 1-bit images, this counts ON pixels (value 1) using an optimized
    /// word-level popcount algorithm matching C Leptonica's `pixCountPixels()`.
    ///
    /// For other depths (2, 4, 8, 16, 32 bpp), this counts all pixels whose
    /// value is non-zero.
    ///
    /// # Returns
    ///
    /// The number of non-zero pixels.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
    /// assert_eq!(pix.count_pixels(), 0);
    ///
    /// let mut pix_mut = pix.to_mut();
    /// pix_mut.set_pixel(0, 0, 1).unwrap();
    /// pix_mut.set_pixel(10, 10, 1).unwrap();
    /// let pix2: Pix = pix_mut.into();
    /// assert_eq!(pix2.count_pixels(), 2);
    /// ```
    pub fn count_pixels(&self) -> u64 {
        match self.depth() {
            PixelDepth::Bit1 => self.count_pixels_binary(),
            _ => self.count_pixels_general(),
        }
    }

    /// Optimized pixel count for 1-bit images using popcount.
    ///
    /// Matches C Leptonica's `pixCountPixels()` algorithm.
    fn count_pixels_binary(&self) -> u64 {
        let width = self.width();
        let height = self.height();
        let wpl = self.wpl();

        let bits_used = width % 32;
        let end_mask = if bits_used == 0 {
            0xFFFFFFFF
        } else {
            !((1u32 << (32 - bits_used)) - 1)
        };
        let full_words = (width / 32) as usize;

        let mut count: u64 = 0;

        for y in 0..height {
            let line = self.row_data(y);

            for word in line.iter().take(full_words) {
                count += word.count_ones() as u64;
            }

            if bits_used != 0 && (full_words as u32) < wpl {
                count += (line[full_words] & end_mask).count_ones() as u64;
            }
        }

        count
    }

    /// General pixel count for non-binary images.
    fn count_pixels_general(&self) -> u64 {
        let width = self.width();
        let height = self.height();
        let mut count: u64 = 0;

        for y in 0..height {
            for x in 0..width {
                if self.get_pixel(x, y).unwrap_or(0) != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    /// Count ON pixels in a rectangular region of a 1 bpp image.
    ///
    /// C equivalent: `pixCountPixelsInRect()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1 bpp.
    pub fn count_pixels_in_rect(&self, region: Option<&Box>) -> Result<u64> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, _, _) =
            clip_box_to_rect(region, w, h).ok_or_else(|| {
                Error::InvalidParameter("region has zero intersection with image".into())
            })?;

        let mut count = 0u64;
        for y in ystart..yend {
            for x in xstart..xend {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Count ON pixels per row in a 1 bpp image.
    ///
    /// C equivalent: `pixCountByRow()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1 bpp.
    pub fn count_by_row(&self, region: Option<&Box>) -> Result<Numa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, _, bh) =
            clip_box_to_rect(region, w, h).ok_or_else(|| {
                Error::InvalidParameter("region has zero intersection with image".into())
            })?;

        let mut na = Numa::with_capacity(bh as usize);
        for y in ystart..yend {
            let mut row_count = 0.0f32;
            for x in xstart..xend {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    row_count += 1.0;
                }
            }
            na.push(row_count);
        }
        Ok(na)
    }

    /// Count ON pixels per column in a 1 bpp image.
    ///
    /// C equivalent: `pixCountByColumn()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1 bpp.
    pub fn count_by_column(&self, region: Option<&Box>) -> Result<Numa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, bw, _) =
            clip_box_to_rect(region, w, h).ok_or_else(|| {
                Error::InvalidParameter("region has zero intersection with image".into())
            })?;

        let mut counts = vec![0.0f32; bw as usize];
        for y in ystart..yend {
            for x in xstart..xend {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    counts[(x - xstart) as usize] += 1.0;
                }
            }
        }
        let mut na = Numa::with_capacity(bw as usize);
        for &c in &counts {
            na.push(c);
        }
        Ok(na)
    }

    /// Check if all pixels in the image are zero.
    ///
    /// C equivalent: `pixZero()` in `pix3.c`
    pub fn is_zero(&self) -> bool {
        let w = self.width();
        let h = self.height();
        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Compute the fraction of ON pixels in a 1 bpp image.
    ///
    /// C equivalent: `pixForegroundFraction()` in `pix3.c`
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1 bpp.
    pub fn foreground_fraction(&self) -> Result<f32> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let total = (self.width() as u64) * (self.height() as u64);
        if total == 0 {
            return Ok(0.0);
        }
        let count = self.count_pixels();
        Ok(count as f32 / total as f32)
    }

    /// Check if the ON pixel count exceeds a threshold.
    ///
    /// May exit early once the threshold is exceeded.
    ///
    /// C equivalent: `pixThresholdPixelSum()` in `pix3.c`
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1 bpp.
    pub fn threshold_pixel_sum(&self, thresh: u64) -> Result<bool> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width();
        let h = self.height();
        let mut count = 0u64;
        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) != 0 {
                    count += 1;
                    if count > thresh {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

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
    /// Computes the mean of all pixel values within the given region
    /// (or the entire image when `region` is `None`).
    ///
    /// C equivalent: `pixAverageInRect(pixs, NULL, box, 0, maxval, 1, &ave)`
    ///
    /// # Arguments
    ///
    /// * `region` - Optional clipping box. If `None`, uses the entire image.
    ///
    /// # Returns
    ///
    /// The average pixel value as `f32`.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 1, 2, 4, or 8 bpp, or if
    /// it has a colormap.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    /// let ave = pix.average_in_rect(None).unwrap();
    /// assert_eq!(ave, 0.0); // All pixels are 0
    /// ```
    pub fn average_in_rect(&self, region: Option<&Box>) -> Result<f32> {
        let max = self.depth().max_value();
        self.average_in_rect_filtered(region, 0, max, 1)
            .map(|opt| opt.unwrap_or(0.0))
    }

    /// Compute the average pixel value in a rectangular region with filtering.
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
    /// let ave = pix.average_in_rect_filtered(None, 0, 255, 1).unwrap();
    /// assert_eq!(ave, Some(0.0)); // All pixels are 0
    /// ```
    pub fn average_in_rect_filtered(
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

        let ave = pix.average_in_rect_filtered(None, 0, 255, 1).unwrap();
        assert_eq!(ave, Some(100.0));
    }

    #[test]
    fn test_average_in_rect_range_filter() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let ave = pix.average_in_rect_filtered(None, 1, 255, 1).unwrap();
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
    fn test_count_pixels_in_rect() {
        let pix = Pix::new(20, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Set some pixels
        pm.set_pixel_unchecked(5, 5, 1);
        pm.set_pixel_unchecked(6, 5, 1);
        pm.set_pixel_unchecked(15, 8, 1);
        let pix: Pix = pm.into();

        // Full image count
        assert_eq!(pix.count_pixels_in_rect(None).unwrap(), 3);

        // Region containing first two pixels
        let region = Box::new(0, 0, 10, 10).unwrap();
        assert_eq!(pix.count_pixels_in_rect(Some(&region)).unwrap(), 2);

        // Region containing only third pixel
        let region2 = Box::new(10, 0, 10, 10).unwrap();
        assert_eq!(pix.count_pixels_in_rect(Some(&region2)).unwrap(), 1);

        // Empty region
        let region3 = Box::new(0, 0, 1, 1).unwrap();
        assert_eq!(pix.count_pixels_in_rect(Some(&region3)).unwrap(), 0);
    }

    #[test]
    fn test_count_pixels_in_rect_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.count_pixels_in_rect(None).is_err());
    }

    #[test]
    fn test_count_by_row() {
        let pix = Pix::new(10, 4, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Row 0: 3 pixels ON
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(5, 0, 1);
        pm.set_pixel_unchecked(9, 0, 1);
        // Row 1: 0 pixels
        // Row 2: 1 pixel
        pm.set_pixel_unchecked(4, 2, 1);
        // Row 3: all 10 pixels
        for x in 0..10 {
            pm.set_pixel_unchecked(x, 3, 1);
        }
        let pix: Pix = pm.into();

        let counts = pix.count_by_row(None).unwrap();
        assert_eq!(counts.len(), 4);
        assert_eq!(counts.get(0).unwrap(), 3.0);
        assert_eq!(counts.get(1).unwrap(), 0.0);
        assert_eq!(counts.get(2).unwrap(), 1.0);
        assert_eq!(counts.get(3).unwrap(), 10.0);
    }

    #[test]
    fn test_count_by_column() {
        let pix = Pix::new(5, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Col 0: 2 pixels
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(0, 5, 1);
        // Col 1: 0 pixels
        // Col 4: all 10 pixels
        for y in 0..10 {
            pm.set_pixel_unchecked(4, y, 1);
        }
        let pix: Pix = pm.into();

        let counts = pix.count_by_column(None).unwrap();
        assert_eq!(counts.len(), 5);
        assert_eq!(counts.get(0).unwrap(), 2.0);
        assert_eq!(counts.get(1).unwrap(), 0.0);
        assert_eq!(counts.get(4).unwrap(), 10.0);
    }

    #[test]
    fn test_is_zero() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        assert!(pix.is_zero());

        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(50, 50, 1);
        let pix: Pix = pm.into();
        assert!(!pix.is_zero());
    }

    #[test]
    fn test_foreground_fraction() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Set 25 out of 100 pixels
        for y in 0..5 {
            for x in 0..5 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pm.into();

        let frac = pix.foreground_fraction().unwrap();
        assert!((frac - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_foreground_fraction_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.foreground_fraction().is_err());
    }

    #[test]
    fn test_threshold_pixel_sum() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pm.into();
        // 25 ON pixels
        assert!(pix.threshold_pixel_sum(20).unwrap()); // 25 > 20
        assert!(pix.threshold_pixel_sum(24).unwrap()); // 25 > 24
        assert!(!pix.threshold_pixel_sum(25).unwrap()); // 25 not > 25
        assert!(!pix.threshold_pixel_sum(100).unwrap());
    }

    #[test]
    fn test_wrong_depth_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.average_by_row(None, PixelMaxType::WhiteIsMax).is_err());
        assert!(
            pix.average_by_column(None, PixelMaxType::WhiteIsMax)
                .is_err()
        );
        assert!(pix.average_in_rect_filtered(None, 0, 255, 1).is_err());
        assert!(pix.variance_in_rect(None).is_err());
        assert!(pix.variance_by_row(None).is_err());
        assert!(pix.variance_by_column(None).is_err());
    }
}
