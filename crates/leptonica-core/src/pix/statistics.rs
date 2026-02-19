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

/// Type of extreme value to find.
///
/// C equivalent: `L_SELECT_MIN` / `L_SELECT_MAX`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtremeType {
    /// Find the minimum value.
    Min,
    /// Find the maximum value.
    Max,
}

/// Result of an extreme value query.
///
/// For 8bpp grayscale, returns `Gray(u32)`.
/// For 32bpp RGB, returns per-channel values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtremeResult {
    /// Grayscale extreme value.
    Gray(u32),
    /// Per-channel RGB extreme values.
    Rgb { r: u32, g: u32, b: u32 },
}

/// Result of a maximum value search in a rectangular region.
///
/// C equivalent: output of `pixGetMaxValueInRect()`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxValueResult {
    /// Maximum pixel value found.
    pub max_val: u32,
    /// X coordinate of the maximum value pixel.
    pub x: u32,
    /// Y coordinate of the maximum value pixel.
    pub y: u32,
}

/// Clip a `Box` to the image rectangle `(0, 0, w, h)`.
///
/// Returns `(xstart, ystart, xend, yend, bw, bh)` of the clipped region.
/// Returns `None` if the clipped box has zero area.
pub(crate) fn clip_box_to_rect(
    bx: Option<&Box>,
    w: i32,
    h: i32,
) -> Option<(i32, i32, i32, i32, i32, i32)> {
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
        let (xstart, ystart, xend, yend, _, _) = match clip_box_to_rect(region, w, h) {
            Some(vals) => vals,
            None => return Ok(0),
        };

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
        let (xstart, ystart, xend, yend, _, bh) = match clip_box_to_rect(region, w, h) {
            Some(vals) => vals,
            None => return Ok(Numa::new()),
        };

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
        let (xstart, ystart, xend, yend, bw, _) = match clip_box_to_rect(region, w, h) {
            Some(vals) => vals,
            None => return Ok(Numa::new()),
        };

        let mut counts = vec![0.0f32; bw as usize];
        for y in ystart..yend {
            for x in xstart..xend {
                if self.get_pixel_unchecked(x as u32, y as u32) != 0 {
                    counts[(x - xstart) as usize] += 1.0;
                }
            }
        }
        Ok(Numa::from_vec(counts))
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

    /// Get extreme (min or max) pixel value across the image.
    ///
    /// For 8bpp grayscale images, returns a single gray value.
    /// For 32bpp RGB images, returns per-channel values.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = all pixels).
    /// * `extreme_type` - Whether to find the minimum or maximum.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetExtremeValue()` in `pix4.c`
    pub fn extreme_value(&self, factor: u32, extreme_type: ExtremeType) -> Result<ExtremeResult> {
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }

        let w = self.width();
        let h = self.height();

        if depth == PixelDepth::Bit8 {
            let mut ext: u32 = match extreme_type {
                ExtremeType::Min => u32::MAX,
                ExtremeType::Max => 0,
            };

            let mut y = 0u32;
            while y < h {
                let line = self.row_data(y);
                let mut x = 0u32;
                while x < w {
                    // Extract byte from packed u32 word
                    let word_idx = (x >> 2) as usize;
                    let byte_idx = 3 - (x & 3);
                    let val = (line[word_idx] >> (byte_idx * 8)) & 0xFF;
                    match extreme_type {
                        ExtremeType::Min => {
                            if val < ext {
                                ext = val;
                            }
                        }
                        ExtremeType::Max => {
                            if val > ext {
                                ext = val;
                            }
                        }
                    }
                    x += factor;
                }
                y += factor;
            }
            Ok(ExtremeResult::Gray(ext))
        } else {
            // 32bpp RGB
            let (mut ext_r, mut ext_g, mut ext_b): (u32, u32, u32) = match extreme_type {
                ExtremeType::Min => (u32::MAX, u32::MAX, u32::MAX),
                ExtremeType::Max => (0, 0, 0),
            };

            let mut y = 0u32;
            while y < h {
                let line = self.row_data(y);
                let mut x = 0u32;
                while x < w {
                    let pixel = line[x as usize];
                    let r = crate::color::red(pixel) as u32;
                    let g = crate::color::green(pixel) as u32;
                    let b = crate::color::blue(pixel) as u32;
                    match extreme_type {
                        ExtremeType::Min => {
                            if r < ext_r {
                                ext_r = r;
                            }
                            if g < ext_g {
                                ext_g = g;
                            }
                            if b < ext_b {
                                ext_b = b;
                            }
                        }
                        ExtremeType::Max => {
                            if r > ext_r {
                                ext_r = r;
                            }
                            if g > ext_g {
                                ext_g = g;
                            }
                            if b > ext_b {
                                ext_b = b;
                            }
                        }
                    }
                    x += factor;
                }
                y += factor;
            }
            Ok(ExtremeResult::Rgb {
                r: ext_r,
                g: ext_g,
                b: ext_b,
            })
        }
    }

    /// Find the maximum pixel value and its location within a rectangular region.
    ///
    /// Works with 8, 16, and 32bpp grayscale images (pixel values are treated
    /// as numbers, not RGB components). If the max value is 0, returns the
    /// center of the rectangle.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular region. If `None`, uses entire image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetMaxValueInRect()` in `pix4.c`
    pub fn max_value_in_rect(&self, region: Option<&Box>) -> Result<MaxValueResult> {
        let depth = self.depth();
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit16 && depth != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if self.colormap().is_some() {
            return Err(Error::InvalidParameter(
                "colormapped images not supported".to_string(),
            ));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend) = match region {
            Some(b) => {
                let xe = (b.x + b.w - 1).min(w - 1);
                let ye = (b.y + b.h - 1).min(h - 1);
                (b.x.max(0), b.y.max(0), xe, ye)
            }
            None => (0, 0, w - 1, h - 1),
        };

        let mut max_val: u32 = 0;
        let mut x_max: i32 = 0;
        let mut y_max: i32 = 0;

        for iy in ystart..=yend {
            let line = self.row_data(iy as u32);
            for ix in xstart..=xend {
                let val = match depth {
                    PixelDepth::Bit8 => {
                        let word_idx = (ix as u32 >> 2) as usize;
                        let byte_idx = 3 - (ix as u32 & 3);
                        (line[word_idx] >> (byte_idx * 8)) & 0xFF
                    }
                    PixelDepth::Bit16 => {
                        let word_idx = (ix as u32 >> 1) as usize;
                        let half_idx = 1 - (ix as u32 & 1);
                        (line[word_idx] >> (half_idx * 16)) & 0xFFFF
                    }
                    PixelDepth::Bit32 => line[ix as usize],
                    _ => unreachable!(),
                };
                if val > max_val {
                    max_val = val;
                    x_max = ix;
                    y_max = iy;
                }
            }
        }

        // If all zero, return center of rectangle (C behavior)
        if max_val == 0 {
            x_max = (xstart + xend) / 2;
            y_max = (ystart + yend) / 2;
        }

        Ok(MaxValueResult {
            max_val,
            x: x_max as u32,
            y: y_max as u32,
        })
    }

    /// Get the min and max values of a specific color component.
    ///
    /// For 8bpp grayscale, the `color` argument is ignored.
    /// For 32bpp RGB, only the specified channel's range is returned.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = all pixels).
    /// * `color` - Which RGB component to query (ignored for 8bpp).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRangeValues()` in `pix4.c`
    pub fn range_values(&self, factor: u32, color: super::rgb::RgbComponent) -> Result<(u32, u32)> {
        use super::rgb::RgbComponent;

        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if depth == PixelDepth::Bit32 && color == RgbComponent::Alpha {
            return Err(Error::InvalidParameter(
                "alpha channel not supported for range_values".to_string(),
            ));
        }

        let w = self.width();
        let h = self.height();

        if depth == PixelDepth::Bit8 {
            // Single pass for both min and max
            let mut min_val: u32 = u32::MAX;
            let mut max_val: u32 = 0;

            let mut y = 0u32;
            while y < h {
                let line = self.row_data(y);
                let mut x = 0u32;
                while x < w {
                    let word_idx = (x >> 2) as usize;
                    let byte_idx = 3 - (x & 3);
                    let val = (line[word_idx] >> (byte_idx * 8)) & 0xFF;
                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }
                    x += factor;
                }
                y += factor;
            }
            Ok((min_val, max_val))
        } else {
            // 32bpp: single pass, extract the requested channel
            let mut min_val: u32 = u32::MAX;
            let mut max_val: u32 = 0;

            let shift = match color {
                RgbComponent::Red => crate::color::RED_SHIFT,
                RgbComponent::Green => crate::color::GREEN_SHIFT,
                RgbComponent::Blue => crate::color::BLUE_SHIFT,
                RgbComponent::Alpha => unreachable!(),
            };

            let mut y = 0u32;
            while y < h {
                let line = self.row_data(y);
                let mut x = 0u32;
                while x < w {
                    let val = (line[x as usize] >> shift) & 0xFF;
                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }
                    x += factor;
                }
                y += factor;
            }
            Ok((min_val, max_val))
        }
    }

    /// Get the rank value from the image's histogram.
    ///
    /// For 8bpp: computes gray histogram, then finds the pixel value at
    /// the given rank fraction. For 32bpp: computes per-channel histograms,
    /// finds each channel's rank value, and composes back to an RGB pixel.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = all pixels).
    /// * `rank` - Fraction between 0.0 (darkest) and 1.0 (brightest).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRankValue()` in `pix4.c`
    pub fn pixel_rank_value(&self, factor: u32, rank: f32) -> Result<u32> {
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }
        if !(0.0..=1.0).contains(&rank) {
            return Err(Error::InvalidParameter(format!(
                "rank {rank} not in [0.0, 1.0]"
            )));
        }

        let depth = self.depth();

        if depth == PixelDepth::Bit8 {
            let hist = self.gray_histogram(factor)?;
            let val = hist
                .histogram_val_from_rank(rank)
                .ok_or_else(|| Error::InvalidParameter("empty histogram".to_string()))?;
            Ok(val.round() as u32)
        } else if depth == PixelDepth::Bit32 {
            // Build per-channel histograms in one pass
            let w = self.width();
            let h = self.height();
            let mut r_hist = vec![0.0f32; 256];
            let mut g_hist = vec![0.0f32; 256];
            let mut b_hist = vec![0.0f32; 256];

            let mut y = 0u32;
            while y < h {
                let line = self.row_data(y);
                let mut x = 0u32;
                while x < w {
                    let pixel = line[x as usize];
                    let r = crate::color::red(pixel) as usize;
                    let g = crate::color::green(pixel) as usize;
                    let b = crate::color::blue(pixel) as usize;
                    r_hist[r] += 1.0;
                    g_hist[g] += 1.0;
                    b_hist[b] += 1.0;
                    x += factor;
                }
                y += factor;
            }

            let mut r_numa = Numa::from_vec(r_hist);
            let mut g_numa = Numa::from_vec(g_hist);
            let mut b_numa = Numa::from_vec(b_hist);
            r_numa.set_parameters(0.0, 1.0);
            g_numa.set_parameters(0.0, 1.0);
            b_numa.set_parameters(0.0, 1.0);

            let r_val = r_numa.histogram_val_from_rank(rank).unwrap_or(0.0);
            let g_val = g_numa.histogram_val_from_rank(rank).unwrap_or(0.0);
            let b_val = b_numa.histogram_val_from_rank(rank).unwrap_or(0.0);

            Ok(crate::color::compose_rgb(
                r_val.round() as u8,
                g_val.round() as u8,
                b_val.round() as u8,
            ))
        } else {
            Err(Error::UnsupportedDepth(depth.bits()))
        }
    }
}

/// Direction for computing adjacent pixel differences.
///
/// C equivalent: `L_HORIZONTAL_LINE` / `L_VERTICAL_LINE`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffDirection {
    /// Compute differences along rows (horizontal).
    ///
    /// C equivalent: `L_HORIZONTAL_LINE` (value 0)
    Horizontal,
    /// Compute differences along columns (vertical).
    ///
    /// C equivalent: `L_VERTICAL_LINE` (value 2)
    Vertical,
}

/// Type of pixel statistic to compute.
///
/// C equivalent: `L_MEAN_ABSVAL`, `L_ROOT_MEAN_SQUARE`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelStatType {
    /// Mean of absolute values.
    MeanAbsVal,
    /// Root mean square.
    RootMeanSquare,
    /// Standard deviation from mean.
    StandardDeviation,
    /// Variance.
    Variance,
}

/// Per-row or per-column statistics output.
///
/// Each field is optional; only requested statistics are computed.
#[derive(Debug, Default)]
pub struct RowColumnStats {
    /// Mean value per row/column.
    pub mean: Option<Numa>,
    /// Median value per row/column.
    pub median: Option<Numa>,
    /// Mode (most frequent value) per row/column.
    pub mode: Option<Numa>,
    /// Count of mode occurrences per row/column.
    pub mode_count: Option<Numa>,
    /// Variance per row/column.
    pub variance: Option<Numa>,
    /// RMS deviation per row/column.
    pub rootvar: Option<Numa>,
}

/// Bitmask specifying which statistics to compute in row/column stats.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatsRequest {
    pub mean: bool,
    pub median: bool,
    pub mode: bool,
    pub mode_count: bool,
    pub variance: bool,
    pub rootvar: bool,
}

impl StatsRequest {
    /// Request all statistics.
    pub fn all() -> Self {
        Self {
            mean: true,
            median: true,
            mode: true,
            mode_count: true,
            variance: true,
            rootvar: true,
        }
    }
}

impl Pix {
    /// Average absolute differences between adjacent pixels per row.
    ///
    /// Returns a Numa with one entry per row, each being the average
    /// of `|pixel[x+1] - pixel[x]|` for that row.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular sub-region. If `None`, the entire
    ///   image is used. The box is clipped to the image bounds.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * the clipped region has zero area
    /// * the region width is less than 2
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAbsDiffByRow()` in `pix3.c`
    pub fn abs_diff_by_row(&self, region: Option<&Box>) -> Result<Numa> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, bw, _bh) = clip_box_to_rect(region, w, h)
            .ok_or(Error::InvalidParameter("region has zero area".into()))?;
        if bw < 2 {
            return Err(Error::InvalidParameter("region width must be >= 2".into()));
        }
        let norm = 1.0 / (bw - 1) as f32;
        let mut numa = Numa::new();
        for y in ystart..yend {
            let mut sum = 0u32;
            let mut prev = self.get_pixel_unchecked(xstart as u32, y as u32);
            for x in (xstart + 1)..xend {
                let val = self.get_pixel_unchecked(x as u32, y as u32);
                sum += (val as i32 - prev as i32).unsigned_abs();
                prev = val;
            }
            numa.push(sum as f32 * norm);
        }
        Ok(numa)
    }

    /// Average absolute differences between adjacent pixels per column.
    ///
    /// Returns a Numa with one entry per column, each being the average
    /// of `|pixel[y+1] - pixel[y]|` for that column.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular sub-region. If `None`, the entire
    ///   image is used. The box is clipped to the image bounds.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * the clipped region has zero area
    /// * the region height is less than 2
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAbsDiffByColumn()` in `pix3.c`
    pub fn abs_diff_by_column(&self, region: Option<&Box>) -> Result<Numa> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, _bw, bh) = clip_box_to_rect(region, w, h)
            .ok_or(Error::InvalidParameter("region has zero area".into()))?;
        if bh < 2 {
            return Err(Error::InvalidParameter("region height must be >= 2".into()));
        }
        let norm = 1.0 / (bh - 1) as f32;
        let mut numa = Numa::new();
        for x in xstart..xend {
            let mut sum = 0u32;
            let mut prev = self.get_pixel_unchecked(x as u32, ystart as u32);
            for y in (ystart + 1)..yend {
                let val = self.get_pixel_unchecked(x as u32, y as u32);
                sum += (val as i32 - prev as i32).unsigned_abs();
                prev = val;
            }
            numa.push(sum as f32 * norm);
        }
        Ok(numa)
    }

    /// Average absolute difference between adjacent pixels in a region.
    ///
    /// Returns a single float value representing the average absolute
    /// difference in the specified direction.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular sub-region. If `None`, the entire
    ///   image is used.
    /// * `direction` - Direction of differencing:
    ///   - `Horizontal`: `|pixel[x+1] - pixel[x]|` within each row
    ///   - `Vertical`: `|pixel[y+1] - pixel[y]|` within each column
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * the clipped region has zero area
    /// * the region is too small in the specified direction (< 2 pixels)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAbsDiffInRect()` in `pix3.c`
    pub fn abs_diff_in_rect(&self, region: Option<&Box>, direction: DiffDirection) -> Result<f32> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, bw, bh) = clip_box_to_rect(region, w, h)
            .ok_or(Error::InvalidParameter("region has zero area".into()))?;

        match direction {
            DiffDirection::Horizontal => {
                if bw < 2 {
                    return Err(Error::InvalidParameter("region width must be >= 2".into()));
                }
                let mut total_sum = 0u64;
                for y in ystart..yend {
                    let mut prev = self.get_pixel_unchecked(xstart as u32, y as u32);
                    for x in (xstart + 1)..xend {
                        let val = self.get_pixel_unchecked(x as u32, y as u32);
                        total_sum += (val as i32 - prev as i32).unsigned_abs() as u64;
                        prev = val;
                    }
                }
                Ok(total_sum as f32 / (bh as f32 * (bw - 1) as f32))
            }
            DiffDirection::Vertical => {
                if bh < 2 {
                    return Err(Error::InvalidParameter("region height must be >= 2".into()));
                }
                let mut total_sum = 0u64;
                for x in xstart..xend {
                    let mut prev = self.get_pixel_unchecked(x as u32, ystart as u32);
                    for y in (ystart + 1)..yend {
                        let val = self.get_pixel_unchecked(x as u32, y as u32);
                        total_sum += (val as i32 - prev as i32).unsigned_abs() as u64;
                        prev = val;
                    }
                }
                Ok(total_sum as f32 / (bw as f32 * (bh - 1) as f32))
            }
        }
    }

    /// Compute multiple statistics per row.
    ///
    /// For each row in the region, computes the requested statistics
    /// (mean, median, mode, mode_count, variance, rootvar) using
    /// per-row histograms.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular sub-region. If `None`, the entire
    ///   image is used.
    /// * `request` - Specifies which statistics to compute. Use
    ///   `StatsRequest::all()` for all, or set individual fields to `true`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * the clipped region has zero area
    ///
    /// # See also
    ///
    /// C Leptonica: `pixRowStats()` in `pix4.c`
    pub fn row_stats(
        &self,
        region: Option<&Box>,
        request: &StatsRequest,
    ) -> Result<RowColumnStats> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, bw, _bh) = clip_box_to_rect(region, w, h)
            .ok_or(Error::InvalidParameter("region has zero area".into()))?;

        let n = (yend - ystart) as usize;
        let need_mean_var = request.mean || request.variance || request.rootvar;
        let need_hist = request.median || request.mode || request.mode_count;

        let mut mean_arr = if request.mean {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut var_arr = if request.variance {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut rootvar_arr = if request.rootvar {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut median_arr = if request.median {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut mode_arr = if request.mode {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut modecount_arr = if request.mode_count {
            Some(Numa::with_capacity(n))
        } else {
            None
        };

        let norm = 1.0 / bw as f64;

        for y in ystart..yend {
            // Pass 1: mean, variance, rootvar
            if need_mean_var {
                let mut sum1 = 0.0f64;
                let mut sum2 = 0.0f64;
                for x in xstart..xend {
                    let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                    sum1 += val;
                    sum2 += val * val;
                }
                let mean = sum1 * norm;
                let variance = sum2 * norm - mean * mean;
                if let Some(ref mut a) = mean_arr {
                    a.push(mean as f32);
                }
                if let Some(ref mut a) = var_arr {
                    a.push(variance as f32);
                }
                if let Some(ref mut a) = rootvar_arr {
                    a.push((variance.max(0.0)).sqrt() as f32);
                }
            }

            // Pass 2: median, mode, mode_count (via histogram)
            if need_hist {
                let mut hist = [0u32; 256];
                for x in xstart..xend {
                    let val = self.get_pixel_unchecked(x as u32, y as u32) as usize;
                    hist[val] += 1;
                }

                if request.median {
                    let target = (bw as u32).div_ceil(2);
                    let mut cumsum = 0u32;
                    let mut median_val = 0u32;
                    for (i, &count) in hist.iter().enumerate() {
                        cumsum += count;
                        if cumsum >= target {
                            median_val = i as u32;
                            break;
                        }
                    }
                    if let Some(ref mut a) = median_arr {
                        a.push(median_val as f32);
                    }
                }

                if request.mode || request.mode_count {
                    let mut max_count = 0u32;
                    let mut mode_val = 0u32;
                    for (i, &count) in hist.iter().enumerate() {
                        if count > max_count {
                            max_count = count;
                            mode_val = i as u32;
                        }
                    }
                    if let Some(ref mut a) = mode_arr {
                        a.push(mode_val as f32);
                    }
                    if let Some(ref mut a) = modecount_arr {
                        a.push(max_count as f32);
                    }
                }
            }
        }

        Ok(RowColumnStats {
            mean: mean_arr,
            median: median_arr,
            mode: mode_arr,
            mode_count: modecount_arr,
            variance: var_arr,
            rootvar: rootvar_arr,
        })
    }

    /// Compute multiple statistics per column.
    ///
    /// For each column in the region, computes the requested statistics.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular sub-region. If `None`, the entire
    ///   image is used.
    /// * `request` - Specifies which statistics to compute. Use
    ///   `StatsRequest::all()` for all, or set individual fields to `true`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * the clipped region has zero area
    ///
    /// # See also
    ///
    /// C Leptonica: `pixColumnStats()` in `pix4.c`
    pub fn column_stats(
        &self,
        region: Option<&Box>,
        request: &StatsRequest,
    ) -> Result<RowColumnStats> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let (xstart, ystart, xend, yend, _bw, bh) = clip_box_to_rect(region, w, h)
            .ok_or(Error::InvalidParameter("region has zero area".into()))?;

        let n = (xend - xstart) as usize;
        let need_mean_var = request.mean || request.variance || request.rootvar;
        let need_hist = request.median || request.mode || request.mode_count;

        let mut mean_arr = if request.mean {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut var_arr = if request.variance {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut rootvar_arr = if request.rootvar {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut median_arr = if request.median {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut mode_arr = if request.mode {
            Some(Numa::with_capacity(n))
        } else {
            None
        };
        let mut modecount_arr = if request.mode_count {
            Some(Numa::with_capacity(n))
        } else {
            None
        };

        let norm = 1.0 / bh as f64;

        for x in xstart..xend {
            if need_mean_var {
                let mut sum1 = 0.0f64;
                let mut sum2 = 0.0f64;
                for y in ystart..yend {
                    let val = self.get_pixel_unchecked(x as u32, y as u32) as f64;
                    sum1 += val;
                    sum2 += val * val;
                }
                let mean = sum1 * norm;
                let variance = sum2 * norm - mean * mean;
                if let Some(ref mut a) = mean_arr {
                    a.push(mean as f32);
                }
                if let Some(ref mut a) = var_arr {
                    a.push(variance as f32);
                }
                if let Some(ref mut a) = rootvar_arr {
                    a.push((variance.max(0.0)).sqrt() as f32);
                }
            }

            if need_hist {
                let mut hist = [0u32; 256];
                for y in ystart..yend {
                    let val = self.get_pixel_unchecked(x as u32, y as u32) as usize;
                    hist[val] += 1;
                }

                if request.median {
                    let target = (bh as u32).div_ceil(2);
                    let mut cumsum = 0u32;
                    let mut median_val = 0u32;
                    for (i, &count) in hist.iter().enumerate() {
                        cumsum += count;
                        if cumsum >= target {
                            median_val = i as u32;
                            break;
                        }
                    }
                    if let Some(ref mut a) = median_arr {
                        a.push(median_val as f32);
                    }
                }

                if request.mode || request.mode_count {
                    let mut max_count = 0u32;
                    let mut mode_val = 0u32;
                    for (i, &count) in hist.iter().enumerate() {
                        if count > max_count {
                            max_count = count;
                            mode_val = i as u32;
                        }
                    }
                    if let Some(ref mut a) = mode_arr {
                        a.push(mode_val as f32);
                    }
                    if let Some(ref mut a) = modecount_arr {
                        a.push(max_count as f32);
                    }
                }
            }
        }

        Ok(RowColumnStats {
            mean: mean_arr,
            median: median_arr,
            mode: mode_arr,
            mode_count: modecount_arr,
            variance: var_arr,
            rootvar: rootvar_arr,
        })
    }

    /// Average pixel value with optional mask.
    ///
    /// For 8bpp, returns a grayscale average. For 32bpp RGB, returns
    /// the average of each channel recomposed into an RGB pixel value.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask. Only pixels where mask is non-zero
    ///   are included. If `None`, all pixels are sampled.
    /// * `mask_x`, `mask_y` - Offset of the mask relative to the image.
    /// * `factor` - Subsampling factor (1 = every pixel, 2 = every other, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 or 32 bpp
    /// * `factor` is 0
    /// * no pixels are sampled (e.g. mask excludes all pixels)
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetPixelAverage()` in `pix4.c`
    pub fn get_pixel_average(
        &self,
        mask: Option<&Pix>,
        mask_x: u32,
        mask_y: u32,
        factor: u32,
    ) -> Result<u32> {
        let depth = self.depth();
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        let w = self.width();
        let h = self.height();

        if depth == PixelDepth::Bit8 {
            let mut sum = 0u64;
            let mut count = 0u64;
            let mut y = 0u32;
            while y < h {
                let mut x = 0u32;
                while x < w {
                    if let Some(m) = mask {
                        let mx = x as i32 - mask_x as i32;
                        let my = y as i32 - mask_y as i32;
                        if mx < 0 || my < 0 || mx >= m.width() as i32 || my >= m.height() as i32 {
                            x += factor;
                            continue;
                        }
                        if m.get_pixel_unchecked(mx as u32, my as u32) == 0 {
                            x += factor;
                            continue;
                        }
                    }
                    sum += self.get_pixel_unchecked(x, y) as u64;
                    count += 1;
                    x += factor;
                }
                y += factor;
            }
            if count == 0 {
                return Err(Error::InvalidParameter("no pixels sampled".into()));
            }
            Ok(((sum as f64 / count as f64) + 0.5) as u32)
        } else {
            // 32bpp: average each channel independently
            let mut sum_r = 0u64;
            let mut sum_g = 0u64;
            let mut sum_b = 0u64;
            let mut count = 0u64;
            let mut y = 0u32;
            while y < h {
                let mut x = 0u32;
                while x < w {
                    if let Some(m) = mask {
                        let mx = x as i32 - mask_x as i32;
                        let my = y as i32 - mask_y as i32;
                        if mx < 0 || my < 0 || mx >= m.width() as i32 || my >= m.height() as i32 {
                            x += factor;
                            continue;
                        }
                        if m.get_pixel_unchecked(mx as u32, my as u32) == 0 {
                            x += factor;
                            continue;
                        }
                    }
                    let pixel = self.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = crate::color::extract_rgba(pixel);
                    sum_r += r as u64;
                    sum_g += g as u64;
                    sum_b += b as u64;
                    count += 1;
                    x += factor;
                }
                y += factor;
            }
            if count == 0 {
                return Err(Error::InvalidParameter("no pixels sampled".into()));
            }
            let avg_r = ((sum_r as f64 / count as f64) + 0.5) as u8;
            let avg_g = ((sum_g as f64 / count as f64) + 0.5) as u8;
            let avg_b = ((sum_b as f64 / count as f64) + 0.5) as u8;
            Ok(crate::color::compose_rgb(avg_r, avg_g, avg_b))
        }
    }

    /// Compute a pixel statistic over the entire image.
    ///
    /// For 8bpp images, computes the statistic directly. For 32bpp RGB,
    /// computes per-channel and recomposes as an RGB pixel value.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel). Must be >= 1.
    /// * `stat_type` - The statistic to compute ([`PixelStatType`]).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 or 32 bpp
    /// * `factor` is 0
    /// * no pixels are sampled
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetPixelStats()` in `pix4.c`
    pub fn get_pixel_stats(&self, factor: u32, stat_type: PixelStatType) -> Result<u32> {
        let depth = self.depth();
        if depth != PixelDepth::Bit8 && depth != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        if depth == PixelDepth::Bit8 {
            self.get_pixel_stats_gray(factor, stat_type)
        } else {
            // 32bpp: compute per channel, recompose
            let r_pix = self.get_rgb_component(super::RgbComponent::Red)?;
            let g_pix = self.get_rgb_component(super::RgbComponent::Green)?;
            let b_pix = self.get_rgb_component(super::RgbComponent::Blue)?;
            let r = r_pix.get_pixel_stats_gray(factor, stat_type)?;
            let g = g_pix.get_pixel_stats_gray(factor, stat_type)?;
            let b = b_pix.get_pixel_stats_gray(factor, stat_type)?;
            Ok(crate::color::compose_rgb(r as u8, g as u8, b as u8))
        }
    }

    /// Internal: compute a single statistic for an 8bpp image.
    fn get_pixel_stats_gray(&self, factor: u32, stat_type: PixelStatType) -> Result<u32> {
        let w = self.width();
        let h = self.height();
        let mut sum1 = 0.0f64;
        let mut sum2 = 0.0f64;
        let mut count = 0u64;

        let mut y = 0u32;
        while y < h {
            let mut x = 0u32;
            while x < w {
                let val = self.get_pixel_unchecked(x, y) as f64;
                sum1 += val;
                sum2 += val * val;
                count += 1;
                x += factor;
            }
            y += factor;
        }
        if count == 0 {
            return Err(Error::InvalidParameter("no pixels sampled".into()));
        }
        let mean = sum1 / count as f64;
        let mean_sq = sum2 / count as f64;
        let variance = (mean_sq - mean * mean).max(0.0);

        let result = match stat_type {
            PixelStatType::MeanAbsVal => mean,
            PixelStatType::RootMeanSquare => mean_sq.sqrt(),
            PixelStatType::StandardDeviation => variance.sqrt(),
            PixelStatType::Variance => variance,
        };
        Ok((result + 0.5) as u32)
    }

    /// Count ON pixels in a specific row of a 1 bpp image.
    ///
    /// C equivalent: `pixCountPixelsInRow()` in `pix3.c`
    pub fn count_pixels_in_row(&self, row: u32) -> Result<u64> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if row >= self.height() {
            return Err(Error::IndexOutOfBounds {
                index: row as usize,
                len: self.height() as usize,
            });
        }
        let w = self.width();
        let mut count: u64 = 0;
        for x in 0..w {
            if self.get_pixel_unchecked(x, row) != 0 {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Compute the moment of fg pixels by column.
    ///
    /// For each column, sums `row_index^order` for every ON pixel.
    /// Order must be 1 or 2.
    ///
    /// C equivalent: `pixGetMomentByColumn()` in `pix3.c`
    pub fn get_moment_by_column(&self, order: u32) -> Result<Numa> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if !(1..=2).contains(&order) {
            return Err(Error::InvalidParameter("order must be 1 or 2".into()));
        }
        let w = self.width();
        let h = self.height();
        let mut moments = Numa::with_capacity(w as usize);
        for x in 0..w {
            let mut sum: f64 = 0.0;
            for y in 0..h {
                if self.get_pixel_unchecked(x, y) != 0 {
                    let row = y as f64;
                    sum += if order == 1 { row } else { row * row };
                }
            }
            moments.push(sum as f32);
        }
        Ok(moments)
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
    fn test_count_by_row_with_region() {
        let pix = Pix::new(10, 4, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(5, 0, 1);
        pm.set_pixel_unchecked(4, 2, 1);
        let pix: Pix = pm.into();

        let region = Box::new(0, 0, 5, 3).unwrap();
        let counts = pix.count_by_row(Some(&region)).unwrap();
        assert_eq!(counts.len(), 3);
        assert_eq!(counts.get(0).unwrap(), 1.0); // only x=0 in [0..5)
        assert_eq!(counts.get(2).unwrap(), 1.0); // x=4 in [0..5)
    }

    #[test]
    fn test_count_by_column_with_region() {
        let pix = Pix::new(5, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(0, 5, 1);
        pm.set_pixel_unchecked(4, 3, 1);
        let pix: Pix = pm.into();

        let region = Box::new(0, 0, 3, 10).unwrap();
        let counts = pix.count_by_column(Some(&region)).unwrap();
        assert_eq!(counts.len(), 3);
        assert_eq!(counts.get(0).unwrap(), 2.0); // col 0 has 2 pixels in region
    }

    #[test]
    fn test_count_by_row_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.count_by_row(None).is_err());
    }

    #[test]
    fn test_count_by_column_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.count_by_column(None).is_err());
    }

    #[test]
    fn test_threshold_pixel_sum_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.threshold_pixel_sum(0).is_err());
    }

    // --- extreme_value tests ---

    #[test]

    fn test_extreme_value_8bpp_min() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        // Set some pixels to non-zero values
        pm.set_pixel(5, 5, 100).unwrap();
        pm.set_pixel(3, 3, 50).unwrap();
        let pix: Pix = pm.into();

        let result = pix.extreme_value(1, ExtremeType::Min).unwrap();
        assert_eq!(result, ExtremeResult::Gray(0)); // Most pixels are 0
    }

    #[test]

    fn test_extreme_value_8bpp_max() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(5, 5, 200).unwrap();
        pm.set_pixel(3, 3, 50).unwrap();
        let pix: Pix = pm.into();

        let result = pix.extreme_value(1, ExtremeType::Max).unwrap();
        assert_eq!(result, ExtremeResult::Gray(200));
    }

    #[test]

    fn test_extreme_value_32bpp_min() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        // All pixels start as (0,0,0,255). Set one to non-zero RGB.
        pm.set_rgb(5, 5, 100, 50, 200).unwrap();
        let pix: Pix = pm.into();

        let result = pix.extreme_value(1, ExtremeType::Min).unwrap();
        assert_eq!(result, ExtremeResult::Rgb { r: 0, g: 0, b: 0 });
    }

    #[test]

    fn test_extreme_value_32bpp_max() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        pm.set_rgb(5, 5, 100, 50, 200).unwrap();
        pm.set_rgb(7, 7, 255, 128, 64).unwrap();
        let pix: Pix = pm.into();

        let result = pix.extreme_value(1, ExtremeType::Max).unwrap();
        assert_eq!(
            result,
            ExtremeResult::Rgb {
                r: 255,
                g: 128,
                b: 200
            }
        );
    }

    #[test]

    fn test_extreme_value_with_factor() {
        // 100x100 all zeros, set (99,99) to 255
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(99, 99, 255).unwrap();
        let pix: Pix = pm.into();

        // Factor 2 samples every other pixel; (99,99) is odd coordinate
        // and should still be hit since 99 = 0 + 49*2 + 1, wait:
        // iteration: 0, 2, 4, ..., 98. x=99 would not be sampled.
        let result = pix.extreme_value(2, ExtremeType::Max).unwrap();
        // (99,99) is NOT sampled (98 is last), so max should be 0
        assert_eq!(result, ExtremeResult::Gray(0));
    }

    #[test]

    fn test_extreme_value_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.extreme_value(1, ExtremeType::Max).is_err());
    }

    #[test]

    fn test_extreme_value_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.extreme_value(0, ExtremeType::Max).is_err());
    }

    // --- max_value_in_rect tests ---

    #[test]

    fn test_max_value_in_rect_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(10, 10, 200).unwrap();
        pm.set_pixel(5, 5, 100).unwrap();
        let pix: Pix = pm.into();

        let result = pix.max_value_in_rect(None).unwrap();
        assert_eq!(result.max_val, 200);
        assert_eq!(result.x, 10);
        assert_eq!(result.y, 10);
    }

    #[test]

    fn test_max_value_in_rect_with_region() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(15, 15, 200).unwrap(); // Outside region
        pm.set_pixel(3, 3, 100).unwrap(); // Inside region
        let pix: Pix = pm.into();

        let region = crate::Box::new(0, 0, 10, 10).unwrap();
        let result = pix.max_value_in_rect(Some(&region)).unwrap();
        assert_eq!(result.max_val, 100);
        assert_eq!(result.x, 3);
        assert_eq!(result.y, 3);
    }

    #[test]

    fn test_max_value_in_rect_all_zero() {
        // When all zero, should return center of rectangle
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = pix.max_value_in_rect(None).unwrap();
        assert_eq!(result.max_val, 0);
        // Center of (0..19, 0..19) = (9, 9) per C: (xstart+xend)/2
        assert_eq!(result.x, 9);
        assert_eq!(result.y, 9);
    }

    #[test]

    fn test_max_value_in_rect_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        // Set pixel as a raw 32-bit value (treated as number, not RGB)
        pm.set_pixel(3, 3, 0x00FF0000).unwrap();
        let pix: Pix = pm.into();

        let result = pix.max_value_in_rect(None).unwrap();
        assert_eq!(result.max_val, 0x00FF0000);
        assert_eq!(result.x, 3);
        assert_eq!(result.y, 3);
    }

    #[test]

    fn test_max_value_in_rect_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.max_value_in_rect(None).is_err());
    }

    // --- range_values tests ---

    #[test]

    fn test_range_values_8bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(0, 0, 10).unwrap();
        pm.set_pixel(5, 5, 200).unwrap();
        let pix: Pix = pm.into();

        // For 8bpp, color argument is ignored
        let (min, max) = pix
            .range_values(1, super::super::rgb::RgbComponent::Red)
            .unwrap();
        assert_eq!(min, 0); // Most pixels are 0
        assert_eq!(max, 200);
    }

    #[test]

    fn test_range_values_32bpp_red() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        pm.set_rgb(3, 3, 100, 50, 200).unwrap();
        pm.set_rgb(7, 7, 255, 128, 64).unwrap();
        let pix: Pix = pm.into();

        let (min, max) = pix
            .range_values(1, super::super::rgb::RgbComponent::Red)
            .unwrap();
        assert_eq!(min, 0);
        assert_eq!(max, 255);
    }

    #[test]

    fn test_range_values_32bpp_blue() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        pm.set_rgb(3, 3, 100, 50, 200).unwrap();
        pm.set_rgb(7, 7, 255, 128, 64).unwrap();
        let pix: Pix = pm.into();

        let (min, max) = pix
            .range_values(1, super::super::rgb::RgbComponent::Blue)
            .unwrap();
        assert_eq!(min, 0);
        assert_eq!(max, 200);
    }

    #[test]

    fn test_range_values_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(
            pix.range_values(1, super::super::rgb::RgbComponent::Red)
                .is_err()
        );
    }

    // --- pixel_rank_value tests ---

    #[test]

    fn test_pixel_rank_value_8bpp() {
        // Create 10x10, all value 0 except one pixel at value 200
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        pm.set_pixel(5, 5, 200).unwrap();
        let pix: Pix = pm.into();

        // Rank 0.0 -> smallest value = 0
        let val = pix.pixel_rank_value(1, 0.0).unwrap();
        assert_eq!(val, 0);

        // Rank 1.0 -> largest value ~ 200 (histogram interpolation may add ±1)
        let val = pix.pixel_rank_value(1, 1.0).unwrap();
        assert!((val as i32 - 200).abs() <= 1, "expected ~200, got {val}");
    }

    #[test]

    fn test_pixel_rank_value_8bpp_uniform() {
        // All pixels are value 128
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                pm.set_pixel(x, y, 128).unwrap();
            }
        }
        let pix: Pix = pm.into();

        // All ranks should return ~128 (histogram interpolation may add ±1)
        let val = pix.pixel_rank_value(1, 0.5).unwrap();
        assert!((val as i32 - 128).abs() <= 1, "expected ~128, got {val}");
    }

    #[test]

    fn test_pixel_rank_value_32bpp() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        pm.set_rgb(0, 0, 100, 50, 200).unwrap();
        pm.set_rgb(1, 0, 100, 50, 200).unwrap();
        pm.set_rgb(0, 1, 100, 50, 200).unwrap();
        pm.set_rgb(1, 1, 100, 50, 200).unwrap();
        let pix: Pix = pm.into();

        // All pixels same color; histogram interpolation may cause ±1 per channel
        let val = pix.pixel_rank_value(1, 0.5).unwrap();
        let r = crate::color::red(val) as i32;
        let g = crate::color::green(val) as i32;
        let b = crate::color::blue(val) as i32;
        assert!((r - 100).abs() <= 1, "expected r~100, got {r}");
        assert!((g - 50).abs() <= 1, "expected g~50, got {g}");
        assert!((b - 200).abs() <= 1, "expected b~200, got {b}");
    }

    #[test]

    fn test_pixel_rank_value_unsupported_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.pixel_rank_value(1, 0.5).is_err());
    }

    #[test]

    fn test_pixel_rank_value_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.pixel_rank_value(0, 0.5).is_err());
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

    // -- Pix::count_pixels_in_row --

    #[test]
    fn test_count_pixels_in_row() {
        let pix = Pix::new(10, 5, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Row 2: set 3 pixels ON
        pm.set_pixel_unchecked(1, 2, 1);
        pm.set_pixel_unchecked(5, 2, 1);
        pm.set_pixel_unchecked(9, 2, 1);
        // Row 0: set 1 pixel ON
        pm.set_pixel_unchecked(0, 0, 1);
        let pix: Pix = pm.into();

        assert_eq!(pix.count_pixels_in_row(0).unwrap(), 1);
        assert_eq!(pix.count_pixels_in_row(2).unwrap(), 3);
        assert_eq!(pix.count_pixels_in_row(4).unwrap(), 0);
    }

    #[test]
    fn test_count_pixels_in_row_out_of_bounds() {
        let pix = Pix::new(10, 5, PixelDepth::Bit1).unwrap();
        assert!(pix.count_pixels_in_row(5).is_err());
    }

    #[test]
    fn test_count_pixels_in_row_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.count_pixels_in_row(0).is_err());
    }

    // -- Pix::get_moment_by_column --

    #[test]
    fn test_get_moment_by_column_first() {
        // 5 wide, 4 tall image
        let pix = Pix::new(5, 4, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Column 0: pixel at row 1 and row 3
        pm.set_pixel_unchecked(0, 1, 1);
        pm.set_pixel_unchecked(0, 3, 1);
        // Column 2: pixel at row 2
        pm.set_pixel_unchecked(2, 2, 1);
        let pix: Pix = pm.into();

        let moments = pix.get_moment_by_column(1).unwrap();
        assert_eq!(moments.len(), 5);
        // Column 0: moment = 1 + 3 = 4
        assert_eq!(moments.get_i32(0), Some(4));
        // Column 1: no pixels → 0
        assert_eq!(moments.get_i32(1), Some(0));
        // Column 2: moment = 2
        assert_eq!(moments.get_i32(2), Some(2));
    }

    #[test]
    fn test_get_moment_by_column_second() {
        let pix = Pix::new(3, 4, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Column 0: pixel at row 2
        pm.set_pixel_unchecked(0, 2, 1);
        let pix: Pix = pm.into();

        let moments = pix.get_moment_by_column(2).unwrap();
        // Column 0: moment = 2*2 = 4
        assert_eq!(moments.get_i32(0), Some(4));
    }

    #[test]
    fn test_get_moment_by_column_invalid_order() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.get_moment_by_column(0).is_err());
        assert!(pix.get_moment_by_column(3).is_err());
    }

    #[test]
    fn test_get_moment_by_column_not_1bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.get_moment_by_column(1).is_err());
    }
}
