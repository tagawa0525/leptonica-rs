//! Histogram generation for Pix images
//!
//! Functions to compute pixel value distributions from images.

use super::statistics::{PixelStatType, clip_box_to_rect};
use super::{Pix, PixelDepth};
use crate::Box;
use crate::error::{Error, Result};
use crate::numa::{Numa, Numaa};
use crate::{PixColormap, color};

/// RGB channel histograms
///
/// Contains separate 256-bin histograms for red, green, and blue channels.
#[derive(Debug, Clone)]
pub struct ColorHistogram {
    /// Red channel histogram (256 bins)
    pub red: Numa,
    /// Green channel histogram (256 bins)
    pub green: Numa,
    /// Blue channel histogram (256 bins)
    pub blue: Numa,
}

impl Pix {
    /// Get the grayscale histogram of the image
    ///
    /// Counts the occurrence of each pixel value in the image.
    /// The histogram size depends on the image depth:
    /// - 1-bit: 2 bins (0 and 1)
    /// - 2-bit: 4 bins (0-3)
    /// - 4-bit: 16 bins (0-15)
    /// - 8-bit: 256 bins (0-255)
    /// - 16-bit: 65536 bins (0-65535)
    ///
    /// For colormapped images, the colormap is applied and the result
    /// is converted to grayscale before computing the histogram.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor. Use 1 to count all pixels,
    ///   2 to count every other pixel, etc.
    ///
    /// # Returns
    ///
    /// A `Numa` containing the histogram with parameters set to
    /// `startx=0, deltax=1`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The image depth is 32-bit (use `color_histogram` instead)
    /// - The factor is 0
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
    /// let hist = pix.gray_histogram(1).unwrap();
    /// assert_eq!(hist.len(), 256);
    /// ```
    pub fn gray_histogram(&self, factor: u32) -> Result<Numa> {
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();

        // 32-bit images should use color_histogram
        if depth == PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(32, 8));
        }

        // Handle colormapped images
        if let Some(cmap) = self.colormap() {
            return self.gray_histogram_colormapped(cmap, factor);
        }

        let size = 1usize << depth.bits();
        let mut histogram = vec![0.0f32; size];

        let width = self.width();
        let height = self.height();

        // Special case for 1-bit images: count 1-bits
        if depth == PixelDepth::Bit1 {
            let total_pixels = self.count_pixels_by_factor(factor);
            let ones = self.count_ones_by_factor(factor);
            histogram[0] = (total_pixels - ones) as f32;
            histogram[1] = ones as f32;
        } else {
            // General case: iterate over pixels
            let mut y = 0;
            while y < height {
                let line = self.row_data(y);
                let mut x = 0;
                while x < width {
                    let val = get_pixel_from_line(line, x, depth) as usize;
                    if val < size {
                        histogram[val] += 1.0;
                    }
                    x += factor;
                }
                y += factor;
            }
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Get the grayscale histogram of an image within a rectangular region.
    ///
    /// Returns a 256-bin histogram counting pixel values within the specified
    /// rectangular region. If no region is specified, delegates to `gray_histogram`.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional rectangular region. If `None`, uses the entire image.
    /// * `factor` - Subsampling factor (1 = all pixels).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetGrayHistogramInRect()` in `pix4.c`
    pub fn gray_histogram_in_rect(&self, region: Option<&Box>, factor: u32) -> Result<Numa> {
        if region.is_none() {
            return self.gray_histogram(factor);
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();
        if depth == PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(32, 8));
        }

        // Handle colormapped: convert to 8bpp gray equivalent
        if let Some(cmap) = self.colormap() {
            return self.gray_histogram_in_rect_colormapped(cmap, region.unwrap(), factor);
        }

        // Only 8bpp supported for rect histogram (matching C behavior)
        if depth != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;

        let (xstart, ystart, xend, yend, _, _) =
            clip_box_to_rect(region, w, h).ok_or_else(|| {
                Error::InvalidParameter("region has no overlap with image".to_string())
            })?;

        let mut histogram = vec![0.0f32; 256];

        let mut y = ystart;
        while y < yend {
            let line = self.row_data(y as u32);
            let mut x = xstart;
            while x < xend {
                let val = get_pixel_from_line(line, x as u32, depth) as usize;
                histogram[val] += 1.0;
                x += factor as i32;
            }
            y += factor as i32;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Internal: rect histogram for colormapped images
    fn gray_histogram_in_rect_colormapped(
        &self,
        cmap: &PixColormap,
        region: &Box,
        factor: u32,
    ) -> Result<Numa> {
        let w = self.width() as i32;
        let h = self.height() as i32;
        let depth = self.depth();

        let (xstart, ystart, xend, yend, _, _) =
            clip_box_to_rect(Some(region), w, h).ok_or_else(|| {
                Error::InvalidParameter("region has no overlap with image".to_string())
            })?;

        let mut histogram = vec![0.0f32; 256];

        let mut y = ystart;
        while y < yend {
            let line = self.row_data(y as u32);
            let mut x = xstart;
            while x < xend {
                let index = get_pixel_from_line(line, x as u32, depth) as usize;
                if let Some((r, g, b, _)) = cmap.get_rgba(index) {
                    let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as usize;
                    histogram[gray.min(255)] += 1.0;
                }
                x += factor as i32;
            }
            y += factor as i32;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Get the grayscale histogram of an image within a mask region.
    ///
    /// Computes a 256-bin histogram counting pixel values where the mask
    /// has ON (1) pixels. The mask is placed at offset `(x, y)` relative
    /// to the source image.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask image. If `None`, delegates to `gray_histogram`.
    /// * `x` - Horizontal offset of the mask origin on the source image.
    /// * `y` - Vertical offset of the mask origin on the source image.
    /// * `factor` - Subsampling factor (1 = all pixels).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetGrayHistogramMasked()` in `pix4.c`
    pub fn gray_histogram_masked(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
    ) -> Result<Numa> {
        if mask.is_none() {
            return self.gray_histogram(factor);
        }
        let mask = mask.unwrap();
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();
        if depth == PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(32, 8));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::InvalidParameter("mask must be 1 bpp".to_string()));
        }

        // Handle colormapped
        if let Some(cmap) = self.colormap() {
            return self.gray_histogram_masked_colormapped(cmap, mask, x, y, factor);
        }

        if depth != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let wm = mask.width() as i32;
        let hm = mask.height() as i32;

        let mut histogram = vec![0.0f32; 256];

        let mut i = 0i32;
        while i < hm {
            let sy = y + i;
            if sy >= 0 && sy < h {
                let lineg = self.row_data(sy as u32);
                let linem = mask.row_data(i as u32);
                let mut j = 0i32;
                while j < wm {
                    let sx = x + j;
                    if sx >= 0 && sx < w {
                        // Check if mask bit is set
                        let word_idx = (j as u32 >> 5) as usize;
                        let bit_idx = 31 - (j as u32 & 31);
                        if (linem[word_idx] >> bit_idx) & 1 != 0 {
                            let val = get_pixel_from_line(lineg, sx as u32, depth) as usize;
                            histogram[val] += 1.0;
                        }
                    }
                    j += factor as i32;
                }
            }
            i += factor as i32;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Internal: masked histogram for colormapped images
    fn gray_histogram_masked_colormapped(
        &self,
        cmap: &PixColormap,
        mask: &Pix,
        x: i32,
        y: i32,
        factor: u32,
    ) -> Result<Numa> {
        let w = self.width() as i32;
        let h = self.height() as i32;
        let wm = mask.width() as i32;
        let hm = mask.height() as i32;
        let depth = self.depth();

        let mut histogram = vec![0.0f32; 256];

        let mut i = 0i32;
        while i < hm {
            let sy = y + i;
            if sy >= 0 && sy < h {
                let lineg = self.row_data(sy as u32);
                let linem = mask.row_data(i as u32);
                let mut j = 0i32;
                while j < wm {
                    let sx = x + j;
                    if sx >= 0 && sx < w {
                        let word_idx = (j as u32 >> 5) as usize;
                        let bit_idx = 31 - (j as u32 & 31);
                        if (linem[word_idx] >> bit_idx) & 1 != 0 {
                            let index = get_pixel_from_line(lineg, sx as u32, depth) as usize;
                            if let Some((r, g, b, _)) = cmap.get_rgba(index) {
                                let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8)
                                    as usize;
                                histogram[gray.min(255)] += 1.0;
                            }
                        }
                    }
                    j += factor as i32;
                }
            }
            i += factor as i32;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Get RGB color histograms
    ///
    /// Computes separate 256-bin histograms for each color channel.
    /// Only valid for 32-bit RGB images or colormapped images.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor. Use 1 to count all pixels.
    ///
    /// # Returns
    ///
    /// A `ColorHistogram` containing separate histograms for R, G, B.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The image is not 32-bit RGB or colormapped
    /// - The factor is 0
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
    /// let hist = pix.color_histogram(1).unwrap();
    /// assert_eq!(hist.red.len(), 256);
    /// assert_eq!(hist.green.len(), 256);
    /// assert_eq!(hist.blue.len(), 256);
    /// ```
    pub fn color_histogram(&self, factor: u32) -> Result<ColorHistogram> {
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();

        // Handle colormapped images
        if let Some(cmap) = self.colormap() {
            return self.color_histogram_colormapped(cmap, factor);
        }

        // Must be 32-bit RGB
        if depth != PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(depth.bits(), 32));
        }

        let mut r_hist = vec![0.0f32; 256];
        let mut g_hist = vec![0.0f32; 256];
        let mut b_hist = vec![0.0f32; 256];

        let width = self.width();
        let height = self.height();

        let mut y = 0;
        while y < height {
            let line = self.row_data(y);
            let mut x = 0;
            while x < width {
                let pixel = line[x as usize];
                let r = color::red(pixel) as usize;
                let g = color::green(pixel) as usize;
                let b = color::blue(pixel) as usize;
                r_hist[r] += 1.0;
                g_hist[g] += 1.0;
                b_hist[b] += 1.0;
                x += factor;
            }
            y += factor;
        }

        let mut red = Numa::from_vec(r_hist);
        let mut green = Numa::from_vec(g_hist);
        let mut blue = Numa::from_vec(b_hist);

        red.set_parameters(0.0, 1.0);
        green.set_parameters(0.0, 1.0);
        blue.set_parameters(0.0, 1.0);

        Ok(ColorHistogram { red, green, blue })
    }

    /// Get RGB color histograms within a mask region.
    ///
    /// Computes separate 256-bin histograms for each color channel where
    /// the mask has ON (1) pixels. The mask is placed at offset `(x, y)`
    /// relative to the source image.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask image. If `None`, delegates to `color_histogram`.
    /// * `x` - Horizontal offset of the mask origin on the source image.
    /// * `y` - Vertical offset of the mask origin on the source image.
    /// * `factor` - Subsampling factor (1 = all pixels).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetColorHistogramMasked()` in `pix4.c`
    pub fn color_histogram_masked(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
    ) -> Result<ColorHistogram> {
        if mask.is_none() {
            return self.color_histogram(factor);
        }
        let mask = mask.unwrap();
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".to_string()));
        }

        let depth = self.depth();

        // Handle colormapped images
        if let Some(cmap) = self.colormap() {
            return self.color_histogram_masked_colormapped(cmap, mask, x, y, factor);
        }

        if depth != PixelDepth::Bit32 {
            return Err(Error::IncompatibleDepths(depth.bits(), 32));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::InvalidParameter("mask must be 1 bpp".to_string()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let wm = mask.width() as i32;
        let hm = mask.height() as i32;

        let mut r_hist = vec![0.0f32; 256];
        let mut g_hist = vec![0.0f32; 256];
        let mut b_hist = vec![0.0f32; 256];

        let mut i = 0i32;
        while i < hm {
            let sy = y + i;
            if sy >= 0 && sy < h {
                let lines = self.row_data(sy as u32);
                let linem = mask.row_data(i as u32);
                let mut j = 0i32;
                while j < wm {
                    let sx = x + j;
                    if sx >= 0 && sx < w {
                        let word_idx = (j as u32 >> 5) as usize;
                        let bit_idx = 31 - (j as u32 & 31);
                        if (linem[word_idx] >> bit_idx) & 1 != 0 {
                            let pixel = lines[sx as usize];
                            let r = color::red(pixel) as usize;
                            let g = color::green(pixel) as usize;
                            let b = color::blue(pixel) as usize;
                            r_hist[r] += 1.0;
                            g_hist[g] += 1.0;
                            b_hist[b] += 1.0;
                        }
                    }
                    j += factor as i32;
                }
            }
            i += factor as i32;
        }

        let mut red = Numa::from_vec(r_hist);
        let mut green = Numa::from_vec(g_hist);
        let mut blue = Numa::from_vec(b_hist);
        red.set_parameters(0.0, 1.0);
        green.set_parameters(0.0, 1.0);
        blue.set_parameters(0.0, 1.0);

        Ok(ColorHistogram { red, green, blue })
    }

    /// Internal: masked color histogram for colormapped images
    fn color_histogram_masked_colormapped(
        &self,
        cmap: &PixColormap,
        mask: &Pix,
        x: i32,
        y: i32,
        factor: u32,
    ) -> Result<ColorHistogram> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::InvalidParameter("mask must be 1 bpp".to_string()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let wm = mask.width() as i32;
        let hm = mask.height() as i32;
        let depth = self.depth();

        let mut r_hist = vec![0.0f32; 256];
        let mut g_hist = vec![0.0f32; 256];
        let mut b_hist = vec![0.0f32; 256];

        let mut i = 0i32;
        while i < hm {
            let sy = y + i;
            if sy >= 0 && sy < h {
                let lines = self.row_data(sy as u32);
                let linem = mask.row_data(i as u32);
                let mut j = 0i32;
                while j < wm {
                    let sx = x + j;
                    if sx >= 0 && sx < w {
                        let word_idx = (j as u32 >> 5) as usize;
                        let bit_idx = 31 - (j as u32 & 31);
                        if (linem[word_idx] >> bit_idx) & 1 != 0 {
                            let index = get_pixel_from_line(lines, sx as u32, depth) as usize;
                            if let Some((r, g, b, _)) = cmap.get_rgba(index) {
                                r_hist[r as usize] += 1.0;
                                g_hist[g as usize] += 1.0;
                                b_hist[b as usize] += 1.0;
                            }
                        }
                    }
                    j += factor as i32;
                }
            }
            i += factor as i32;
        }

        let mut red = Numa::from_vec(r_hist);
        let mut green = Numa::from_vec(g_hist);
        let mut blue = Numa::from_vec(b_hist);
        red.set_parameters(0.0, 1.0);
        green.set_parameters(0.0, 1.0);
        blue.set_parameters(0.0, 1.0);

        Ok(ColorHistogram { red, green, blue })
    }

    /// Compute grayscale histogram for colormapped image
    fn gray_histogram_colormapped(&self, cmap: &PixColormap, factor: u32) -> Result<Numa> {
        let mut histogram = vec![0.0f32; 256];

        let width = self.width();
        let height = self.height();
        let depth = self.depth();

        let mut y = 0;
        while y < height {
            let line = self.row_data(y);
            let mut x = 0;
            while x < width {
                let index = get_pixel_from_line(line, x, depth) as usize;
                if let Some((r, g, b, _)) = cmap.get_rgba(index) {
                    // Convert to grayscale using standard luminance weights
                    let gray = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as usize;
                    let gray = gray.min(255);
                    histogram[gray] += 1.0;
                }
                x += factor;
            }
            y += factor;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Compute color histogram for colormapped image
    fn color_histogram_colormapped(
        &self,
        cmap: &PixColormap,
        factor: u32,
    ) -> Result<ColorHistogram> {
        let mut r_hist = vec![0.0f32; 256];
        let mut g_hist = vec![0.0f32; 256];
        let mut b_hist = vec![0.0f32; 256];

        let width = self.width();
        let height = self.height();
        let depth = self.depth();

        let mut y = 0;
        while y < height {
            let line = self.row_data(y);
            let mut x = 0;
            while x < width {
                let index = get_pixel_from_line(line, x, depth) as usize;
                if let Some((r, g, b, _)) = cmap.get_rgba(index) {
                    r_hist[r as usize] += 1.0;
                    g_hist[g as usize] += 1.0;
                    b_hist[b as usize] += 1.0;
                }
                x += factor;
            }
            y += factor;
        }

        let mut red = Numa::from_vec(r_hist);
        let mut green = Numa::from_vec(g_hist);
        let mut blue = Numa::from_vec(b_hist);

        red.set_parameters(0.0, 1.0);
        green.set_parameters(0.0, 1.0);
        blue.set_parameters(0.0, 1.0);

        Ok(ColorHistogram { red, green, blue })
    }

    /// Count total pixels considering subsampling factor
    fn count_pixels_by_factor(&self, factor: u32) -> u32 {
        let w = self.width().div_ceil(factor);
        let h = self.height().div_ceil(factor);
        w * h
    }

    /// Count 1-bits with subsampling factor
    fn count_ones_by_factor(&self, factor: u32) -> u32 {
        if self.depth() != PixelDepth::Bit1 {
            return 0;
        }

        let width = self.width();
        let height = self.height();
        let mut count = 0u32;

        let mut y = 0;
        while y < height {
            let line = self.row_data(y);
            let mut x = 0;
            while x < width {
                let word_idx = (x >> 5) as usize;
                let bit_idx = 31 - (x & 31);
                if (line[word_idx] >> bit_idx) & 1 != 0 {
                    count += 1;
                }
                x += factor;
            }
            y += factor;
        }

        count
    }
}

// ============================================================================
// Advanced histogram and tile-based statistics (Phase 6.2)
// ============================================================================

impl Pix {
    /// Compute grayscale histograms for a tiled grid of the image.
    ///
    /// Divides the image into `nx * ny` tiles and returns a histogram
    /// for each tile.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel).
    /// * `nx` - Number of horizontal tiles.
    /// * `ny` - Number of vertical tiles.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 8 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetGrayHistogramTiled()` in `pix4.c`
    pub fn gray_histogram_tiled(&self, factor: u32, nx: u32, ny: u32) -> Result<Numaa> {
        let depth = self.depth();
        if depth == PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if factor == 0 || nx == 0 || ny == 0 {
            return Err(Error::InvalidParameter(
                "factor, nx, ny must be >= 1".into(),
            ));
        }

        let w = self.width();
        let h = self.height();
        if nx > w {
            return Err(Error::InvalidParameter(format!(
                "tile count nx ({nx}) must not exceed image width ({w})"
            )));
        }
        if ny > h {
            return Err(Error::InvalidParameter(format!(
                "tile count ny ({ny}) must not exceed image height ({h})"
            )));
        }
        let tw = w / nx;
        let th = h / ny;

        let mut result = Numaa::with_capacity((nx * ny) as usize);
        for iy in 0..ny {
            for ix in 0..nx {
                let x0 = ix * tw;
                let y0 = iy * th;
                let region = Box::new(x0 as i32, y0 as i32, tw as i32, th as i32)?;
                let hist = self.gray_histogram_in_rect(Some(&region), factor)?;
                result.push(hist);
            }
        }
        Ok(result)
    }

    /// Histogram of colormap indices.
    ///
    /// Returns a Numa counting occurrences of each colormap index.
    /// The image must have a colormap.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel).
    ///
    /// # Errors
    ///
    /// Returns an error if the image has no colormap.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetCmapHistogram()` in `pix4.c`
    pub fn cmap_histogram(&self, factor: u32) -> Result<Numa> {
        if self.colormap().is_none() {
            return Err(Error::InvalidParameter("image has no colormap".into()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        let depth = self.depth();
        let nbins = 1usize << depth.bits();
        let mut histogram = vec![0.0f32; nbins];

        let w = self.width();
        let h = self.height();
        let mut y = 0u32;
        while y < h {
            let mut x = 0u32;
            while x < w {
                let idx = self.get_pixel_unchecked(x, y) as usize;
                if idx < nbins {
                    histogram[idx] += 1.0;
                }
                x += factor;
            }
            y += factor;
        }

        let mut result = Numa::from_vec(histogram);
        result.set_parameters(0.0, 1.0);
        Ok(result)
    }

    /// Compute a histogram of colormap pixel indices over a masked region.
    ///
    /// The mask is 1bpp and specifies which pixels to include (ON pixels).
    /// `x`, `y` are the offset of the mask relative to the image UL corner.
    /// Returns a `Numa` of size `2^d` (d = image depth).
    ///
    /// C equivalent: `pixGetCmapHistogramMasked()` in `pix4.c`
    pub fn cmap_histogram_masked(&self, mask: &Pix, x: i32, y: i32, factor: u32) -> Result<Numa> {
        todo!("not yet implemented")
    }

    /// Compute a histogram of colormap pixel indices within a rectangular region.
    ///
    /// If `region` is `None`, the full image is used (same as `cmap_histogram`).
    /// Returns a `Numa` of size `2^d`.
    ///
    /// C equivalent: `pixGetCmapHistogramInRect()` in `pix4.c`
    pub fn cmap_histogram_in_rect(
        &self,
        region: Option<&crate::box_::Box>,
        factor: u32,
    ) -> Result<Numa> {
        todo!("not yet implemented")
    }

    /// Return the maximum colormap index value used in a colormapped image.
    ///
    /// Supported depths: 1, 2, 4, 8 bpp. The image must have a colormap.
    ///
    /// C equivalent: `pixGetMaxColorIndex()` in `pix4.c`
    pub fn max_color_index(&self) -> Result<u32> {
        todo!("not yet implemented")
    }

    /// Count unique RGB colors in a 32bpp image.
    ///
    /// # Arguments
    ///
    /// * `factor` - Subsampling factor (1 = every pixel).
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 32 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCountRGBColors()` in `pix4.c`
    pub fn count_rgb_colors(&self, factor: u32) -> Result<u32> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        let w = self.width();
        let h = self.height();
        let mut colors = std::collections::HashSet::new();
        let mut y = 0u32;
        while y < h {
            let mut x = 0u32;
            while x < w {
                let pixel = self.get_pixel_unchecked(x, y);
                // Mask out alpha channel for RGB comparison
                colors.insert(pixel & 0xFFFFFF00);
                x += factor;
            }
            y += factor;
        }
        Ok(colors.len() as u32)
    }

    /// Compute a pixel statistic over a masked region (8bpp).
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask. If `None`, all pixels are sampled.
    /// * `x`, `y` - Offset of the mask relative to the image.
    /// * `factor` - Subsampling factor (1 = every pixel).
    /// * `stat_type` - The statistic to compute.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * `factor` is 0
    /// * no pixels are sampled
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetAverageMasked()` in `pix4.c`
    pub fn average_masked(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
        stat_type: PixelStatType,
    ) -> Result<f32> {
        let depth = self.depth();
        if depth != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(depth.bits()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        let w = self.width() as i32;
        let h = self.height() as i32;
        let mut sum1 = 0.0f64;
        let mut sum2 = 0.0f64;
        let mut count = 0u64;

        if let Some(m) = mask {
            let wm = m.width() as i32;
            let hm = m.height() as i32;
            let mut iy = 0i32;
            while iy < hm {
                let sy = y + iy;
                if sy >= 0 && sy < h {
                    let mut ix = 0i32;
                    while ix < wm {
                        let sx = x + ix;
                        if sx >= 0 && sx < w && m.get_pixel_unchecked(ix as u32, iy as u32) != 0 {
                            let val = self.get_pixel_unchecked(sx as u32, sy as u32) as f64;
                            sum1 += val;
                            sum2 += val * val;
                            count += 1;
                        }
                        ix += factor as i32;
                    }
                }
                iy += factor as i32;
            }
        } else {
            let mut iy = 0u32;
            while iy < h as u32 {
                let mut ix = 0u32;
                while ix < w as u32 {
                    let val = self.get_pixel_unchecked(ix, iy) as f64;
                    sum1 += val;
                    sum2 += val * val;
                    count += 1;
                    ix += factor;
                }
                iy += factor;
            }
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
        Ok(result as f32)
    }

    /// Compute per-channel statistics over a masked region (32bpp RGB).
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask. If `None`, all pixels are sampled.
    /// * `x`, `y` - Offset of the mask relative to the image.
    /// * `factor` - Subsampling factor (1 = every pixel).
    /// * `stat_type` - The statistic to compute.
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 32 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetAverageMaskedRGB()` in `pix4.c`
    pub fn average_masked_rgb(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
        stat_type: PixelStatType,
    ) -> Result<(f32, f32, f32)> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let r_pix = self.get_rgb_component(super::RgbComponent::Red)?;
        let g_pix = self.get_rgb_component(super::RgbComponent::Green)?;
        let b_pix = self.get_rgb_component(super::RgbComponent::Blue)?;
        let r = r_pix.average_masked(mask, x, y, factor, stat_type)?;
        let g = g_pix.average_masked(mask, x, y, factor, stat_type)?;
        let b = b_pix.average_masked(mask, x, y, factor, stat_type)?;
        Ok((r, g, b))
    }

    /// Compute tile-based statistics for an 8bpp image.
    ///
    /// Returns an image where each pixel represents the statistic
    /// of the corresponding tile in the input.
    ///
    /// # Arguments
    ///
    /// * `sx`, `sy` - Tile dimensions in pixels.
    /// * `stat_type` - The statistic to compute (mean, RMS, or stdev).
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 8 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetAverageTiled()` in `pix4.c`
    pub fn average_tiled(&self, sx: u32, sy: u32, stat_type: PixelStatType) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if sx == 0 || sy == 0 {
            return Err(Error::InvalidParameter("tile size must be >= 1".into()));
        }

        let w = self.width();
        let h = self.height();
        let nx = w / sx;
        let ny = h / sy;
        if nx == 0 || ny == 0 {
            return Err(Error::InvalidParameter(
                "image too small for tile size".into(),
            ));
        }

        let out = Pix::new(nx, ny, PixelDepth::Bit8)?;
        let mut pm = out.try_into_mut().unwrap();

        for iy in 0..ny {
            for ix in 0..nx {
                let x0 = ix * sx;
                let y0 = iy * sy;
                let mut sum1 = 0.0f64;
                let mut sum2 = 0.0f64;
                let count = (sx * sy) as f64;

                for dy in 0..sy {
                    for dx in 0..sx {
                        let val = self.get_pixel_unchecked(x0 + dx, y0 + dy) as f64;
                        sum1 += val;
                        sum2 += val * val;
                    }
                }

                let mean = sum1 / count;
                let mean_sq = sum2 / count;
                let variance = (mean_sq - mean * mean).max(0.0);

                let result = match stat_type {
                    PixelStatType::MeanAbsVal => mean,
                    PixelStatType::RootMeanSquare => mean_sq.sqrt(),
                    PixelStatType::StandardDeviation => variance.sqrt(),
                    PixelStatType::Variance => variance,
                };
                pm.set_pixel_unchecked(ix, iy, (result + 0.5).min(255.0) as u32);
            }
        }
        Ok(pm.into())
    }

    /// Compute tile-based per-channel statistics for a 32bpp RGB image.
    ///
    /// Returns three 8bpp images (R, G, B) where each pixel
    /// represents the channel statistic of the corresponding tile.
    ///
    /// # Arguments
    ///
    /// * `sx`, `sy` - Tile dimensions in pixels.
    /// * `stat_type` - The statistic to compute (mean, RMS, or stdev).
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 32 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetAverageTiledRGB()` in `pix4.c`
    pub fn average_tiled_rgb(
        &self,
        sx: u32,
        sy: u32,
        stat_type: PixelStatType,
    ) -> Result<(Pix, Pix, Pix)> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let r_pix = self.get_rgb_component(super::RgbComponent::Red)?;
        let g_pix = self.get_rgb_component(super::RgbComponent::Green)?;
        let b_pix = self.get_rgb_component(super::RgbComponent::Blue)?;
        let pr = r_pix.average_tiled(sx, sy, stat_type)?;
        let pg = g_pix.average_tiled(sx, sy, stat_type)?;
        let pb = b_pix.average_tiled(sx, sy, stat_type)?;
        Ok((pr, pg, pb))
    }

    /// Compute the rank value from a masked 8bpp image histogram.
    ///
    /// Returns the pixel value at the given rank position (0.0 = min,
    /// 0.5 = median, 1.0 = max) and optionally the histogram used.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask.
    /// * `x`, `y` - Mask offset.
    /// * `factor` - Subsampling factor.
    /// * `rank` - Rank value in [0.0, 1.0].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * the image depth is not 8 bpp
    /// * `rank` is outside [0.0, 1.0]
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRankValueMasked()` in `pix4.c`
    pub fn rank_value_masked(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
        rank: f32,
    ) -> Result<(f32, Option<Numa>)> {
        if self.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if !(0.0..=1.0).contains(&rank) {
            return Err(Error::InvalidParameter("rank must be in [0.0, 1.0]".into()));
        }
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }

        let hist = self.gray_histogram_masked(mask, x, y, factor)?;
        let val = hist
            .histogram_val_from_rank(rank)
            .ok_or(Error::InvalidParameter(
                "could not compute rank value".into(),
            ))?;
        Ok((val, Some(hist)))
    }

    /// Compute per-channel rank values from a masked 32bpp RGB image.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional 1bpp mask.
    /// * `x`, `y` - Mask offset.
    /// * `factor` - Subsampling factor.
    /// * `rank` - Rank value in [0.0, 1.0].
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 32 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRankValueMaskedRGB()` in `pix4.c`
    pub fn rank_value_masked_rgb(
        &self,
        mask: Option<&Pix>,
        x: i32,
        y: i32,
        factor: u32,
        rank: f32,
    ) -> Result<(f32, f32, f32)> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let r_pix = self.get_rgb_component(super::RgbComponent::Red)?;
        let g_pix = self.get_rgb_component(super::RgbComponent::Green)?;
        let b_pix = self.get_rgb_component(super::RgbComponent::Blue)?;
        let (r, _) = r_pix.rank_value_masked(mask, x, y, factor, rank)?;
        let (g, _) = g_pix.rank_value_masked(mask, x, y, factor, rank)?;
        let (b, _) = b_pix.rank_value_masked(mask, x, y, factor, rank)?;
        Ok((r, g, b))
    }
}

/// Get pixel value from a line buffer
#[inline]
fn get_pixel_from_line(line: &[u32], x: u32, depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => {
            let word_idx = (x >> 5) as usize;
            let bit_idx = 31 - (x & 31);
            (line[word_idx] >> bit_idx) & 1
        }
        PixelDepth::Bit2 => {
            let word_idx = (x >> 4) as usize;
            let bit_idx = 2 * (15 - (x & 15));
            (line[word_idx] >> bit_idx) & 0x3
        }
        PixelDepth::Bit4 => {
            let word_idx = (x >> 3) as usize;
            let bit_idx = 4 * (7 - (x & 7));
            (line[word_idx] >> bit_idx) & 0xF
        }
        PixelDepth::Bit8 => {
            let word_idx = (x >> 2) as usize;
            let byte_idx = 3 - (x & 3);
            (line[word_idx] >> (byte_idx * 8)) & 0xFF
        }
        PixelDepth::Bit16 => {
            let word_idx = (x >> 1) as usize;
            let half_idx = 1 - (x & 1);
            (line[word_idx] >> (half_idx * 16)) & 0xFFFF
        }
        PixelDepth::Bit32 => line[x as usize],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gray_histogram_8bit_uniform() {
        // Create an 8-bit image with uniform value
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        assert_eq!(hist.len(), 256);
        // All pixels are 0 (initialized to zero)
        assert_eq!(hist[0], 10000.0);
        for i in 1..256 {
            assert_eq!(hist[i], 0.0);
        }
    }

    #[test]
    fn test_gray_histogram_with_subsampling() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();

        // Factor 2: sample every other pixel in both dimensions
        let hist = pix.gray_histogram(2).unwrap();
        // 50 * 50 = 2500 pixels sampled
        assert_eq!(hist[0], 2500.0);

        // Factor 10: 10 * 10 = 100 pixels sampled
        let hist = pix.gray_histogram(10).unwrap();
        assert_eq!(hist[0], 100.0);
    }

    #[test]
    fn test_gray_histogram_1bit() {
        let pix = Pix::new(32, 10, PixelDepth::Bit1).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        assert_eq!(hist.len(), 2);
        // All pixels are 0
        assert_eq!(hist[0], 320.0);
        assert_eq!(hist[1], 0.0);
    }

    #[test]
    fn test_gray_histogram_4bit() {
        let pix = Pix::new(16, 10, PixelDepth::Bit4).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        assert_eq!(hist.len(), 16);
        assert_eq!(hist[0], 160.0);
    }

    #[test]
    fn test_gray_histogram_16bit() {
        let pix = Pix::new(10, 10, PixelDepth::Bit16).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        assert_eq!(hist.len(), 65536);
        assert_eq!(hist[0], 100.0);
    }

    #[test]
    fn test_gray_histogram_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.gray_histogram(0).is_err());
    }

    #[test]
    fn test_gray_histogram_32bit_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.gray_histogram(1).is_err());
    }

    #[test]
    fn test_color_histogram_basic() {
        let pix = Pix::new(100, 100, PixelDepth::Bit32).unwrap();
        let hist = pix.color_histogram(1).unwrap();

        assert_eq!(hist.red.len(), 256);
        assert_eq!(hist.green.len(), 256);
        assert_eq!(hist.blue.len(), 256);

        // Default is all zeros (black pixels with alpha=255)
        // RGB values are 0,0,0 for all pixels
        assert_eq!(hist.red[0], 10000.0);
        assert_eq!(hist.green[0], 10000.0);
        assert_eq!(hist.blue[0], 10000.0);
    }

    #[test]
    fn test_color_histogram_with_colors() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.to_mut();

        // Set some pixels to different colors
        for y in 0..5 {
            for x in 0..10 {
                pix_mut.set_rgb(x, y, 255, 0, 0).unwrap(); // Red
            }
        }
        for y in 5..10 {
            for x in 0..10 {
                pix_mut.set_rgb(x, y, 0, 128, 255).unwrap(); // Blue-ish
            }
        }

        let pix: Pix = pix_mut.into();
        let hist = pix.color_histogram(1).unwrap();

        // 50 red pixels, 50 blue-ish pixels
        assert_eq!(hist.red[255], 50.0);
        assert_eq!(hist.red[0], 50.0);
        assert_eq!(hist.green[0], 50.0);
        assert_eq!(hist.green[128], 50.0);
        assert_eq!(hist.blue[0], 50.0);
        assert_eq!(hist.blue[255], 50.0);
    }

    #[test]
    fn test_color_histogram_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.color_histogram(1).is_err());
    }

    #[test]
    fn test_histogram_parameters() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        let (startx, deltax) = hist.parameters();
        assert_eq!(startx, 0.0);
        assert_eq!(deltax, 1.0);
    }

    // --- gray_histogram_in_rect tests ---

    #[test]

    fn test_gray_histogram_in_rect_full_image() {
        // When region is None, should behave like gray_histogram
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let hist = pix.gray_histogram_in_rect(None, 1).unwrap();
        assert_eq!(hist.len(), 256);
        assert_eq!(hist[0], 10000.0);
    }

    #[test]

    fn test_gray_histogram_in_rect_subregion() {
        // Create 8bpp image with known pixel pattern
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        // Fill top-left 50x50 with value 128
        for y in 0..50 {
            for x in 0..50 {
                pm.set_pixel(x, y, 128).unwrap();
            }
        }
        let pix: Pix = pm.into();

        // Histogram of top-left 50x50 region
        let region = crate::Box::new(0, 0, 50, 50).unwrap();
        let hist = pix.gray_histogram_in_rect(Some(&region), 1).unwrap();
        assert_eq!(hist.len(), 256);
        assert_eq!(hist[128], 2500.0); // 50*50 = 2500 pixels of value 128
        assert_eq!(hist[0], 0.0);
    }

    #[test]

    fn test_gray_histogram_in_rect_clipped() {
        // Region extends beyond image boundary
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let region = crate::Box::new(25, 25, 100, 100).unwrap();
        let hist = pix.gray_histogram_in_rect(Some(&region), 1).unwrap();
        assert_eq!(hist.len(), 256);
        // Only 25x25 = 625 pixels should be counted
        assert_eq!(hist[0], 625.0);
    }

    #[test]

    fn test_gray_histogram_in_rect_with_factor() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let region = crate::Box::new(0, 0, 100, 100).unwrap();
        let hist = pix.gray_histogram_in_rect(Some(&region), 2).unwrap();
        assert_eq!(hist[0], 2500.0); // 50*50 = 2500 pixels sampled
    }

    #[test]

    fn test_gray_histogram_in_rect_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.gray_histogram_in_rect(None, 0).is_err());
    }

    #[test]

    fn test_gray_histogram_in_rect_32bit_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.gray_histogram_in_rect(None, 1).is_err());
    }

    // --- gray_histogram_masked tests ---

    #[test]

    fn test_gray_histogram_masked_no_mask() {
        // When mask is None, should behave like gray_histogram
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let hist = pix.gray_histogram_masked(None, 0, 0, 1).unwrap();
        assert_eq!(hist.len(), 256);
        assert_eq!(hist[0], 10000.0);
    }

    #[test]

    fn test_gray_histogram_masked_with_mask() {
        // Create 8bpp source with mixed values
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        // Top half = 100, bottom half = 200
        for y in 0..5 {
            for x in 0..10 {
                pm.set_pixel(x, y, 100).unwrap();
            }
        }
        for y in 5..10 {
            for x in 0..10 {
                pm.set_pixel(x, y, 200).unwrap();
            }
        }
        let pix: Pix = pm.into();

        // Create 1bpp mask covering only top half
        let mask = Pix::new(10, 5, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        for y in 0..5 {
            for x in 0..10 {
                mask_mut.set_pixel(x, y, 1).unwrap();
            }
        }
        let mask: Pix = mask_mut.into();

        // Mask placed at (0, 0) - covers top half of source
        let hist = pix.gray_histogram_masked(Some(&mask), 0, 0, 1).unwrap();
        assert_eq!(hist[100], 50.0); // 10*5 = 50 pixels
        assert_eq!(hist[200], 0.0); // Bottom half not included
    }

    #[test]

    fn test_gray_histogram_masked_with_offset() {
        // Source: 10x10, all value 50
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.to_mut();
        for y in 0..10 {
            for x in 0..10 {
                pm.set_pixel(x, y, 50).unwrap();
            }
        }
        let pix: Pix = pm.into();

        // Small 3x3 mask, all ON
        let mask = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        for y in 0..3 {
            for x in 0..3 {
                mask_mut.set_pixel(x, y, 1).unwrap();
            }
        }
        let mask: Pix = mask_mut.into();

        // Place mask at offset (5, 5) -> covers source pixels (5..8, 5..8)
        let hist = pix.gray_histogram_masked(Some(&mask), 5, 5, 1).unwrap();
        assert_eq!(hist[50], 9.0); // 3*3 = 9 pixels
    }

    #[test]

    fn test_gray_histogram_masked_boundary_clip() {
        // Mask extends beyond image boundary
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();

        // 5x5 mask, all ON
        let mask = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        for y in 0..5 {
            for x in 0..5 {
                mask_mut.set_pixel(x, y, 1).unwrap();
            }
        }
        let mask: Pix = mask_mut.into();

        // Place at (8, 8) -> only 2x2 = 4 pixels within bounds
        let hist = pix.gray_histogram_masked(Some(&mask), 8, 8, 1).unwrap();
        assert_eq!(hist[0], 4.0);
    }

    #[test]

    fn test_gray_histogram_masked_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.gray_histogram_masked(None, 0, 0, 0).is_err());
    }

    #[test]

    fn test_gray_histogram_masked_32bit_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.gray_histogram_masked(None, 0, 0, 1).is_err());
    }

    #[test]

    fn test_gray_histogram_masked_non_1bpp_mask_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.gray_histogram_masked(Some(&mask), 0, 0, 1).is_err());
    }

    // --- color_histogram_masked tests ---

    #[test]

    fn test_color_histogram_masked_no_mask() {
        // When mask is None, should behave like color_histogram
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let hist = pix.color_histogram_masked(None, 0, 0, 1).unwrap();
        assert_eq!(hist.red.len(), 256);
        assert_eq!(hist.green.len(), 256);
        assert_eq!(hist.blue.len(), 256);
        // All black pixels
        assert_eq!(hist.red[0], 100.0);
    }

    #[test]

    fn test_color_histogram_masked_with_mask() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        // Top half red, bottom half blue
        for y in 0..5 {
            for x in 0..10 {
                pm.set_rgb(x, y, 255, 0, 0).unwrap();
            }
        }
        for y in 5..10 {
            for x in 0..10 {
                pm.set_rgb(x, y, 0, 0, 255).unwrap();
            }
        }
        let pix: Pix = pm.into();

        // Mask covering only top half (10x5, all ON)
        let mask = Pix::new(10, 5, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        for y in 0..5 {
            for x in 0..10 {
                mask_mut.set_pixel(x, y, 1).unwrap();
            }
        }
        let mask: Pix = mask_mut.into();

        let hist = pix.color_histogram_masked(Some(&mask), 0, 0, 1).unwrap();
        assert_eq!(hist.red[255], 50.0); // 50 red pixels
        assert_eq!(hist.blue[0], 50.0); // blue channel is 0 for red pixels
        assert_eq!(hist.blue[255], 0.0); // no blue pixels in masked area
    }

    #[test]

    fn test_color_histogram_masked_with_offset() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.to_mut();
        // Set pixel at (5,5) to green
        pm.set_rgb(5, 5, 0, 200, 0).unwrap();
        let pix: Pix = pm.into();

        // 1x1 mask covering (5,5)
        let mask = Pix::new(1, 1, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.to_mut();
        mask_mut.set_pixel(0, 0, 1).unwrap();
        let mask: Pix = mask_mut.into();

        let hist = pix.color_histogram_masked(Some(&mask), 5, 5, 1).unwrap();
        assert_eq!(hist.green[200], 1.0);
        assert_eq!(hist.red[0], 1.0);
        assert_eq!(hist.blue[0], 1.0);
    }

    #[test]

    fn test_color_histogram_masked_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.color_histogram_masked(None, 0, 0, 1).is_err());
    }

    #[test]

    fn test_color_histogram_masked_invalid_factor() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(pix.color_histogram_masked(Some(&mask), 0, 0, 0).is_err());
    }

    #[test]

    fn test_color_histogram_masked_non_1bpp_mask_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.color_histogram_masked(Some(&mask), 0, 0, 1).is_err());
    }

    #[test]
    fn test_histogram_stats_integration() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let hist = pix.gray_histogram(1).unwrap();

        // All pixels are 0, so stats should reflect that
        let stats = hist.histogram_stats(0.0, 1.0).unwrap();
        assert!((stats.mean - 0.0).abs() < 0.001);
        assert!((stats.mode - 0.0).abs() < 0.001);
        assert!((stats.variance - 0.0).abs() < 0.001);
    }

    // -- Pix::cmap_histogram_masked --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cmap_histogram_masked_basic() {
        use crate::PixColormap;
        // 8bpp image with colormap: fill all pixels with index 2
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            let mut cmap = PixColormap::new(8).unwrap();
            cmap.add_rgba(0, 0, 0, 255).unwrap();
            cmap.add_rgba(255, 0, 0, 255).unwrap();
            cmap.add_rgba(0, 255, 0, 255).unwrap();
            pm.set_colormap(Some(cmap)).unwrap();
            for y in 0..4 {
                for x in 0..4 {
                    pm.set_pixel_unchecked(x, y, 2);
                }
            }
            Pix::from(pm)
        };
        // Mask that only includes top-left 2x2
        let mask = {
            let m = Pix::new(2, 2, PixelDepth::Bit1).unwrap();
            let mut mm = m.try_into_mut().unwrap();
            for y in 0..2 {
                for x in 0..2 {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(mm)
        };
        let hist = pix.cmap_histogram_masked(&mask, 0, 0, 1).unwrap();
        assert_eq!(hist.len(), 256);
        assert_eq!(hist[2], 4.0); // 4 masked ON pixels, all index 2
        assert_eq!(hist[0], 0.0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cmap_histogram_masked_no_cmap() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mask = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        assert!(pix.cmap_histogram_masked(&mask, 0, 0, 1).is_err());
    }

    // -- Pix::cmap_histogram_in_rect --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cmap_histogram_in_rect_full() {
        use crate::PixColormap;
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            let mut cmap = PixColormap::new(8).unwrap();
            cmap.add_rgba(0, 0, 0, 255).unwrap();
            cmap.add_rgba(255, 0, 0, 255).unwrap();
            pm.set_colormap(Some(cmap)).unwrap();
            for y in 0..4 {
                for x in 0..4 {
                    pm.set_pixel_unchecked(x, y, 1);
                }
            }
            Pix::from(pm)
        };
        // No region → full image
        let hist = pix.cmap_histogram_in_rect(None, 1).unwrap();
        assert_eq!(hist.len(), 256);
        assert_eq!(hist[1], 16.0); // 4*4 pixels all index 1
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cmap_histogram_in_rect_subregion() {
        use crate::PixColormap;
        use crate::box_::Box as LepBox;
        // 4x4 image; top-left 2x2 = index 0, rest = index 1
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            let mut cmap = PixColormap::new(8).unwrap();
            cmap.add_rgba(0, 0, 0, 255).unwrap();
            cmap.add_rgba(255, 0, 0, 255).unwrap();
            pm.set_colormap(Some(cmap)).unwrap();
            for y in 0..4 {
                for x in 0..4 {
                    let idx = if x < 2 && y < 2 { 0 } else { 1 };
                    pm.set_pixel_unchecked(x, y, idx);
                }
            }
            Pix::from(pm)
        };
        // Subregion: top-left 2x2
        let region = LepBox::new(0, 0, 2, 2).unwrap();
        let hist = pix.cmap_histogram_in_rect(Some(&region), 1).unwrap();
        assert_eq!(hist[0], 4.0);
        assert_eq!(hist[1], 0.0);
    }

    // -- Pix::max_color_index --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_max_color_index_8bpp() {
        use crate::PixColormap;
        let pix = {
            let base = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
            let mut pm = base.try_into_mut().unwrap();
            let mut cmap = PixColormap::new(8).unwrap();
            for _ in 0..5 {
                cmap.add_rgba(0, 0, 0, 255).unwrap();
            }
            pm.set_colormap(Some(cmap)).unwrap();
            pm.set_pixel_unchecked(0, 0, 3);
            pm.set_pixel_unchecked(1, 0, 4);
            Pix::from(pm)
        };
        let max = pix.max_color_index().unwrap();
        assert_eq!(max, 4);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_max_color_index_no_cmap() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        assert!(pix.max_color_index().is_err());
    }
}
