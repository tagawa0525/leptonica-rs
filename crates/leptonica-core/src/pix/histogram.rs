//! Histogram generation for Pix images
//!
//! Functions to compute pixel value distributions from images.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};
use crate::numa::Numa;
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
}
