//! FPix - Floating-point image
//!
//! `FPix` is a 2D array of `f32` values, useful for intermediate computations
//! in image processing where integer precision is insufficient.
//!
//! See [`serial`] for serialization support.
//!
//! # Examples
//!
//! ```
//! use leptonica_core::FPix;
//!
//! // Create a 100x100 floating-point image
//! let mut fpix = FPix::new(100, 100).unwrap();
//!
//! // Set and get pixel values
//! fpix.set_pixel(10, 20, 0.5).unwrap();
//! assert_eq!(fpix.get_pixel(10, 20).unwrap(), 0.5);
//!
//! // Get statistics
//! let (min_val, min_x, min_y) = fpix.min().unwrap();
//! let (max_val, max_x, max_y) = fpix.max().unwrap();
//! ```

pub mod serial;

use crate::error::{Error, Result};
use crate::pix::{Pix, PixelDepth};

/// How to handle negative values when converting FPix to Pix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NegativeHandling {
    /// Clip negative values to zero
    #[default]
    ClipToZero,
    /// Take the absolute value
    TakeAbsValue,
}

/// Floating-point image
///
/// A 2D array of `f32` values. Unlike `Pix` which stores packed integer
/// pixel values, `FPix` stores one `f32` per pixel, allowing for high
/// precision intermediate computations.
///
/// # Memory Layout
///
/// Data is stored in row-major order with no padding. The pixel at (x, y)
/// is at index `y * width + x`.
#[derive(Debug, Clone)]
pub struct FPix {
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Pixel data (row-major, no padding)
    data: Vec<f32>,
    /// X resolution (ppi), 0 if unknown
    xres: i32,
    /// Y resolution (ppi), 0 if unknown
    yres: i32,
}

impl FPix {
    /// Create a new FPix with all pixels set to zero
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels (must be > 0)
    /// * `height` - Height in pixels (must be > 0)
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidDimension` if width or height is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::FPix;
    ///
    /// let fpix = FPix::new(640, 480).unwrap();
    /// assert_eq!(fpix.width(), 640);
    /// assert_eq!(fpix.height(), 480);
    /// ```
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }

        let size = (width as usize) * (height as usize);
        let data = vec![0.0f32; size];

        Ok(FPix {
            width,
            height,
            data,
            xres: 0,
            yres: 0,
        })
    }

    /// Create a new FPix with all pixels set to the specified value
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels (must be > 0)
    /// * `height` - Height in pixels (must be > 0)
    /// * `value` - Initial value for all pixels
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidDimension` if width or height is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::FPix;
    ///
    /// let fpix = FPix::new_with_value(100, 100, 0.5).unwrap();
    /// assert_eq!(fpix.get_pixel(50, 50).unwrap(), 0.5);
    /// ```
    pub fn new_with_value(width: u32, height: u32, value: f32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }

        let size = (width as usize) * (height as usize);
        let data = vec![value; size];

        Ok(FPix {
            width,
            height,
            data,
            xres: 0,
            yres: 0,
        })
    }

    /// Create a FPix from raw data
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `data` - Pixel data in row-major order
    ///
    /// # Errors
    ///
    /// Returns an error if dimensions are invalid or data length doesn't match.
    pub fn from_data(width: u32, height: u32, data: Vec<f32>) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }

        let expected_size = (width as usize) * (height as usize);
        if data.len() != expected_size {
            return Err(Error::InvalidParameter(format!(
                "data length {} doesn't match {}x{} = {}",
                data.len(),
                width,
                height,
                expected_size
            )));
        }

        Ok(FPix {
            width,
            height,
            data,
            xres: 0,
            yres: 0,
        })
    }

    /// Get the image width in pixels
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the image height in pixels
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the image dimensions as (width, height)
    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the X resolution (ppi)
    #[inline]
    pub fn xres(&self) -> i32 {
        self.xres
    }

    /// Get the Y resolution (ppi)
    #[inline]
    pub fn yres(&self) -> i32 {
        self.yres
    }

    /// Get both resolutions as (xres, yres)
    #[inline]
    pub fn resolution(&self) -> (i32, i32) {
        (self.xres, self.yres)
    }

    /// Set the X resolution (ppi)
    #[inline]
    pub fn set_xres(&mut self, xres: i32) {
        self.xres = xres;
    }

    /// Set the Y resolution (ppi)
    #[inline]
    pub fn set_yres(&mut self, yres: i32) {
        self.yres = yres;
    }

    /// Set both resolutions
    #[inline]
    pub fn set_resolution(&mut self, xres: i32, yres: i32) {
        self.xres = xres;
        self.yres = yres;
    }

    /// Get the pixel value at (x, y)
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    ///
    /// # Errors
    ///
    /// Returns `Error::IndexOutOfBounds` if coordinates are out of range.
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<f32> {
        if x >= self.width || y >= self.height {
            return Err(Error::IndexOutOfBounds {
                index: (y as usize) * (self.width as usize) + (x as usize),
                len: self.data.len(),
            });
        }

        let idx = (y as usize) * (self.width as usize) + (x as usize);
        Ok(self.data[idx])
    }

    /// Set the pixel value at (x, y)
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    /// * `value` - New pixel value
    ///
    /// # Errors
    ///
    /// Returns `Error::IndexOutOfBounds` if coordinates are out of range.
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, value: f32) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(Error::IndexOutOfBounds {
                index: (y as usize) * (self.width as usize) + (x as usize),
                len: self.data.len(),
            });
        }

        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.data[idx] = value;
        Ok(())
    }

    /// Get the pixel value at (x, y) without bounds checking
    ///
    /// # Panics
    ///
    /// Panics if `x >= width` or `y >= height`.
    #[inline]
    pub fn get_pixel_unchecked(&self, x: u32, y: u32) -> f32 {
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.data[idx]
    }

    /// Set the pixel value at (x, y) without bounds checking
    ///
    /// # Panics
    ///
    /// Panics if `x >= width` or `y >= height`.
    #[inline]
    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, value: f32) {
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.data[idx] = value;
    }

    /// Get raw access to the pixel data
    #[inline]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get mutable access to the pixel data
    #[inline]
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get a row of pixel data
    ///
    /// # Arguments
    ///
    /// * `y` - Row index
    ///
    /// # Panics
    ///
    /// Panics if `y >= height`.
    #[inline]
    pub fn row(&self, y: u32) -> &[f32] {
        let start = (y as usize) * (self.width as usize);
        let end = start + (self.width as usize);
        &self.data[start..end]
    }

    /// Get a mutable row of pixel data
    ///
    /// # Arguments
    ///
    /// * `y` - Row index
    ///
    /// # Panics
    ///
    /// Panics if `y >= height`.
    #[inline]
    pub fn row_mut(&mut self, y: u32) -> &mut [f32] {
        let start = (y as usize) * (self.width as usize);
        let end = start + (self.width as usize);
        &mut self.data[start..end]
    }

    /// Set all pixels to the specified value
    pub fn set_all(&mut self, value: f32) {
        self.data.fill(value);
    }

    /// Clear all pixels to zero
    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }

    // ========================================================================
    // Conversion from Pix
    // ========================================================================

    /// Create a FPix from a Pix
    ///
    /// Converts integer pixel values to floating-point. Supported depths:
    /// - 1, 2, 4, 8, 16 bpp: Direct conversion to f32
    /// - 32 bpp: Converts to grayscale luminance if RGB
    ///
    /// # Arguments
    ///
    /// * `pix` - Source image
    ///
    /// # Errors
    ///
    /// Returns `Error::UnsupportedDepth` for unsupported pixel depths.
    pub fn from_pix(pix: &Pix) -> Result<Self> {
        let width = pix.width();
        let height = pix.height();
        let depth = pix.depth();

        let mut fpix = FPix::new(width, height)?;
        fpix.xres = pix.xres();
        fpix.yres = pix.yres();

        let src_data = pix.data();
        let wpl = pix.wpl() as usize;

        for y in 0..height {
            let row_start = (y as usize) * wpl;
            let row_data = &src_data[row_start..row_start + wpl];

            for x in 0..width {
                let val: f32 = match depth {
                    PixelDepth::Bit1 => {
                        let word_idx = (x / 32) as usize;
                        let bit_idx = 31 - (x % 32);
                        let bit = (row_data[word_idx] >> bit_idx) & 1;
                        bit as f32
                    }
                    PixelDepth::Bit2 => {
                        let word_idx = (x / 16) as usize;
                        let shift = 30 - 2 * (x % 16);
                        let val = (row_data[word_idx] >> shift) & 0x3;
                        val as f32
                    }
                    PixelDepth::Bit4 => {
                        let word_idx = (x / 8) as usize;
                        let shift = 28 - 4 * (x % 8);
                        let val = (row_data[word_idx] >> shift) & 0xf;
                        val as f32
                    }
                    PixelDepth::Bit8 => {
                        let word_idx = (x / 4) as usize;
                        let shift = 24 - 8 * (x % 4);
                        let val = (row_data[word_idx] >> shift) & 0xff;
                        val as f32
                    }
                    PixelDepth::Bit16 => {
                        let word_idx = (x / 2) as usize;
                        let shift = 16 - 16 * (x % 2);
                        let val = (row_data[word_idx] >> shift) & 0xffff;
                        val as f32
                    }
                    PixelDepth::Bit32 => {
                        let pixel = row_data[x as usize];
                        // Convert RGB to luminance: 0.299*R + 0.587*G + 0.114*B
                        let r = ((pixel >> 24) & 0xff) as f32;
                        let g = ((pixel >> 16) & 0xff) as f32;
                        let b = ((pixel >> 8) & 0xff) as f32;
                        0.299 * r + 0.587 * g + 0.114 * b
                    }
                };

                let idx = (y as usize) * (width as usize) + (x as usize);
                fpix.data[idx] = val;
            }
        }

        Ok(fpix)
    }

    // ========================================================================
    // Conversion to Pix
    // ========================================================================

    /// Convert FPix to Pix
    ///
    /// # Arguments
    ///
    /// * `out_depth` - Output depth (8, 16, or 32). Use 0 for auto-detection.
    /// * `neg_handling` - How to handle negative values
    ///
    /// # Auto-detection
    ///
    /// When `out_depth` is 0:
    /// - If all values <= 255: use 8 bpp
    /// - Else if all values <= 65535: use 16 bpp
    /// - Else: use 32 bpp
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidParameter` if out_depth is not 0, 8, 16, or 32.
    pub fn to_pix(&self, out_depth: u32, neg_handling: NegativeHandling) -> Result<Pix> {
        // Determine output depth
        let depth = if out_depth == 0 {
            self.auto_detect_depth()
        } else if out_depth == 8 || out_depth == 16 || out_depth == 32 {
            out_depth
        } else {
            return Err(Error::InvalidParameter(format!(
                "out_depth must be 0, 8, 16, or 32, got {}",
                out_depth
            )));
        };

        let pixel_depth = PixelDepth::from_bits(depth)?;
        let pix = Pix::new(self.width, self.height, pixel_depth)?;
        let mut pix_mut = pix.to_mut();

        pix_mut.set_xres(self.xres);
        pix_mut.set_yres(self.yres);

        let max_val = match depth {
            8 => 255u32,
            16 => 65535u32,
            32 => u32::MAX,
            _ => unreachable!(),
        };

        let wpl = pix_mut.wpl() as usize;
        let dst_data = pix_mut.data_mut();

        for y in 0..self.height {
            let row_start = (y as usize) * wpl;
            let row_data = &mut dst_data[row_start..row_start + wpl];

            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let fval = self.data[idx];

                // Handle negative values
                let fval = if fval < 0.0 {
                    match neg_handling {
                        NegativeHandling::ClipToZero => 0.0,
                        NegativeHandling::TakeAbsValue => fval.abs(),
                    }
                } else {
                    fval
                };

                // Round and clamp
                let ival = (fval + 0.5) as u32;
                let ival = ival.min(max_val);

                match depth {
                    8 => {
                        let word_idx = (x / 4) as usize;
                        let shift = 24 - 8 * (x % 4);
                        let mask = !(0xffu32 << shift);
                        row_data[word_idx] = (row_data[word_idx] & mask) | (ival << shift);
                    }
                    16 => {
                        let word_idx = (x / 2) as usize;
                        let shift = 16 - 16 * (x % 2);
                        let mask = !(0xffffu32 << shift);
                        row_data[word_idx] = (row_data[word_idx] & mask) | (ival << shift);
                    }
                    32 => {
                        row_data[x as usize] = ival;
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(pix_mut.into())
    }

    /// Auto-detect appropriate output depth based on pixel values
    fn auto_detect_depth(&self) -> u32 {
        let mut max_val: f32 = 0.0;

        for &val in &self.data {
            let abs_val = val.abs();
            if abs_val > max_val {
                max_val = abs_val;
            }
        }

        if max_val <= 255.5 {
            8
        } else if max_val <= 65535.5 {
            16
        } else {
            32
        }
    }

    // ========================================================================
    // Arithmetic Operations
    // ========================================================================

    /// Add two FPix images element-wise
    ///
    /// # Errors
    ///
    /// Returns `Error::IncompatibleSizes` if dimensions don't match.
    pub fn add(&self, other: &FPix) -> Result<FPix> {
        self.check_same_size(other)?;

        let mut result = FPix::new(self.width, self.height)?;
        for (i, (&a, &b)) in self.data.iter().zip(other.data.iter()).enumerate() {
            result.data[i] = a + b;
        }

        Ok(result)
    }

    /// Subtract other FPix from this one element-wise
    ///
    /// # Errors
    ///
    /// Returns `Error::IncompatibleSizes` if dimensions don't match.
    pub fn sub(&self, other: &FPix) -> Result<FPix> {
        self.check_same_size(other)?;

        let mut result = FPix::new(self.width, self.height)?;
        for (i, (&a, &b)) in self.data.iter().zip(other.data.iter()).enumerate() {
            result.data[i] = a - b;
        }

        Ok(result)
    }

    /// Multiply two FPix images element-wise
    ///
    /// # Errors
    ///
    /// Returns `Error::IncompatibleSizes` if dimensions don't match.
    pub fn mul(&self, other: &FPix) -> Result<FPix> {
        self.check_same_size(other)?;

        let mut result = FPix::new(self.width, self.height)?;
        for (i, (&a, &b)) in self.data.iter().zip(other.data.iter()).enumerate() {
            result.data[i] = a * b;
        }

        Ok(result)
    }

    /// Divide this FPix by other element-wise
    ///
    /// Division by zero results in `f32::INFINITY` or `f32::NEG_INFINITY`.
    ///
    /// # Errors
    ///
    /// Returns `Error::IncompatibleSizes` if dimensions don't match.
    pub fn div(&self, other: &FPix) -> Result<FPix> {
        self.check_same_size(other)?;

        let mut result = FPix::new(self.width, self.height)?;
        for (i, (&a, &b)) in self.data.iter().zip(other.data.iter()).enumerate() {
            result.data[i] = a / b;
        }

        Ok(result)
    }

    /// Add a constant to all pixels (in-place)
    pub fn add_constant(&mut self, value: f32) {
        for v in &mut self.data {
            *v += value;
        }
    }

    /// Multiply all pixels by a constant (in-place)
    pub fn mul_constant(&mut self, value: f32) {
        for v in &mut self.data {
            *v *= value;
        }
    }

    /// Linear combination: result = a * self + b
    ///
    /// This performs `result[i] = multiplier * self[i] + addend` for each pixel.
    pub fn linear_combination(&self, multiplier: f32, addend: f32) -> FPix {
        let mut result = self.clone();
        for v in &mut result.data {
            *v = multiplier * *v + addend;
        }
        result
    }

    /// Create a template FPix with the same dimensions, zeroed data.
    ///
    /// Preserves the resolution (xres, yres) of the source.
    ///
    /// # See also
    ///
    /// C Leptonica: `fpixCreateTemplate()` in `fpix1.c`
    pub fn create_template(&self) -> FPix {
        FPix {
            width: self.width,
            height: self.height,
            data: vec![0.0; self.data.len()],
            xres: self.xres,
            yres: self.yres,
        }
    }

    /// Linear combination of two FPix: `a * fpix1 + b * fpix2`.
    ///
    /// Both images must have the same dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `fpixLinearCombination()` in `fpix2.c`
    pub fn linear_combination_two(a: f32, fpix1: &FPix, b: f32, fpix2: &FPix) -> Result<FPix> {
        fpix1.check_same_size(fpix2)?;
        let mut result = fpix1.create_template();
        for (i, val) in result.data.iter_mut().enumerate() {
            *val = a * fpix1.data[i] + b * fpix2.data[i];
        }
        Ok(result)
    }

    /// Check that two FPix have the same dimensions
    fn check_same_size(&self, other: &FPix) -> Result<()> {
        if self.width != other.width || self.height != other.height {
            return Err(Error::IncompatibleSizes(
                self.width,
                self.height,
                other.width,
                other.height,
            ));
        }
        Ok(())
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Find the minimum value and its location
    ///
    /// Returns `(min_value, x, y)` where (x, y) is the location of the first
    /// occurrence of the minimum value.
    ///
    /// Returns `None` if the image is empty (shouldn't happen with valid FPix).
    pub fn min(&self) -> Option<(f32, u32, u32)> {
        if self.data.is_empty() {
            return None;
        }

        let mut min_val = f32::MAX;
        let mut min_x = 0u32;
        let mut min_y = 0u32;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                if self.data[idx] < min_val {
                    min_val = self.data[idx];
                    min_x = x;
                    min_y = y;
                }
            }
        }

        Some((min_val, min_x, min_y))
    }

    /// Find the minimum value only
    pub fn min_value(&self) -> Option<f32> {
        self.min().map(|(v, _, _)| v)
    }

    /// Find the maximum value and its location
    ///
    /// Returns `(max_value, x, y)` where (x, y) is the location of the first
    /// occurrence of the maximum value.
    ///
    /// Returns `None` if the image is empty.
    pub fn max(&self) -> Option<(f32, u32, u32)> {
        if self.data.is_empty() {
            return None;
        }

        let mut max_val = f32::MIN;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                if self.data[idx] > max_val {
                    max_val = self.data[idx];
                    max_x = x;
                    max_y = y;
                }
            }
        }

        Some((max_val, max_x, max_y))
    }

    /// Find the maximum value only
    pub fn max_value(&self) -> Option<f32> {
        self.max().map(|(v, _, _)| v)
    }

    /// Calculate the mean (average) of all pixel values
    ///
    /// Returns `None` if the image is empty.
    pub fn mean(&self) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }

        let sum: f32 = self.data.iter().sum();
        Some(sum / self.data.len() as f32)
    }

    /// Calculate the sum of all pixel values
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    // -- Phase 17.2: Contour rendering --

    /// Render contour lines from the FPix data, returning an 8bpp colormapped Pix.
    ///
    /// Contours are drawn at every `incr` interval.  A pixel is classified as a
    /// contour if its fractional distance to the nearest contour level is ≤ `proxim`.
    /// Negative values are shown in red (index 2); non-negative in black (index 1).
    /// White (index 0) is the background.
    ///
    /// `proxim` defaults to 0.15 when ≤ 0.
    ///
    /// C equivalent: `fpixRenderContours()` in `graphics.c`
    pub fn render_contours(&self, incr: f32, proxim: f32) -> crate::error::Result<Pix> {
        use crate::colormap::PixColormap;
        if incr <= 0.0 {
            return Err(crate::error::Error::InvalidParameter(
                "incr must be > 0".to_string(),
            ));
        }
        let proxim = if proxim <= 0.0 { 0.15 } else { proxim };
        let (w, h) = self.dimensions();
        let pix = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut pixd = pix.to_mut();
        let mut cmap = PixColormap::new(8)?;
        cmap.add_rgb(255, 255, 255)?; // index 0: white
        cmap.add_rgb(0, 0, 0)?; // index 1: black
        cmap.add_rgb(255, 0, 0)?; // index 2: red
        pixd.set_colormap(Some(cmap))?;
        let inv_incr = 1.0 / incr;
        for y in 0..h {
            for x in 0..w {
                let val = self.get_pixel_unchecked(x, y);
                let finter = inv_incr * val;
                let above = finter - finter.floor();
                let below = finter.ceil() - finter;
                let diff = above.min(below);
                if diff <= proxim {
                    let idx: u32 = if val < 0.0 { 2 } else { 1 };
                    let _ = pixd.set_pixel(x, y, idx);
                }
            }
        }
        Ok(pixd.into())
    }

    /// Auto-render contours with approximately `ncontours` levels.
    ///
    /// C equivalent: `fpixAutoRenderContours()` in `graphics.c`
    pub fn auto_render_contours(&self, ncontours: i32) -> crate::error::Result<Pix> {
        if !(2..=500).contains(&ncontours) {
            return Err(crate::error::Error::InvalidParameter(
                "ncontours must be in [2, 500]".to_string(),
            ));
        }
        let min = self
            .min_value()
            .ok_or(crate::error::Error::NullInput("empty FPix"))?;
        let max = self
            .max_value()
            .ok_or(crate::error::Error::NullInput("empty FPix"))?;
        if (min - max).abs() < f32::EPSILON {
            return Err(crate::error::Error::InvalidParameter(
                "all values in FPix are equal".to_string(),
            ));
        }
        let incr = (max - min) / (ncontours as f32 - 1.0);
        self.render_contours(incr, 0.15)
    }
}

// ============================================================================
// Operator Overloading
// ============================================================================

impl std::ops::Add for &FPix {
    type Output = Result<FPix>;

    fn add(self, rhs: Self) -> Self::Output {
        FPix::add(self, rhs)
    }
}

impl std::ops::Sub for &FPix {
    type Output = Result<FPix>;

    fn sub(self, rhs: Self) -> Self::Output {
        FPix::sub(self, rhs)
    }
}

impl std::ops::Mul for &FPix {
    type Output = Result<FPix>;

    fn mul(self, rhs: Self) -> Self::Output {
        FPix::mul(self, rhs)
    }
}

impl std::ops::Div for &FPix {
    type Output = Result<FPix>;

    fn div(self, rhs: Self) -> Self::Output {
        FPix::div(self, rhs)
    }
}

// ============================================================================
// DPix - Double-precision floating-point image
// ============================================================================

/// Double-precision floating-point image
///
/// Similar to [`FPix`] but stores `f64` values for higher precision.
/// Used when f32 precision is insufficient.
///
/// # Memory Layout
///
/// Data is stored in row-major order with no padding.
///
/// # See also
///
/// C Leptonica: `DPIX` in `pix.h`
#[derive(Debug, Clone)]
pub struct DPix {
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Pixel data (row-major, no padding)
    data: Vec<f64>,
    /// X resolution (ppi), 0 if unknown
    xres: i32,
    /// Y resolution (ppi), 0 if unknown
    yres: i32,
}

impl DPix {
    /// Create a new DPix with all pixels set to zero.
    ///
    /// # See also
    ///
    /// C Leptonica: `dpixCreate()` in `fpix1.c`
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimension { width, height });
        }
        let size = (width as usize) * (height as usize);
        Ok(DPix {
            width,
            height,
            data: vec![0.0f64; size],
            xres: 0,
            yres: 0,
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the X resolution (ppi).
    #[inline]
    pub fn xres(&self) -> i32 {
        self.xres
    }

    /// Get the Y resolution (ppi).
    #[inline]
    pub fn yres(&self) -> i32 {
        self.yres
    }

    /// Get both resolutions as (xres, yres).
    #[inline]
    pub fn resolution(&self) -> (i32, i32) {
        (self.xres, self.yres)
    }

    /// Set the X resolution (ppi).
    #[inline]
    pub fn set_xres(&mut self, xres: i32) {
        self.xres = xres;
    }

    /// Set the Y resolution (ppi).
    #[inline]
    pub fn set_yres(&mut self, yres: i32) {
        self.yres = yres;
    }

    /// Set both resolutions.
    #[inline]
    pub fn set_resolution(&mut self, xres: i32, yres: i32) {
        self.xres = xres;
        self.yres = yres;
    }

    /// Get pixel value at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<f64> {
        if x >= self.width || y >= self.height {
            return Err(Error::IndexOutOfBounds {
                index: (y as usize) * (self.width as usize) + (x as usize),
                len: self.data.len(),
            });
        }
        Ok(self.data[(y as usize) * (self.width as usize) + (x as usize)])
    }

    /// Set pixel value at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, value: f64) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(Error::IndexOutOfBounds {
                index: (y as usize) * (self.width as usize) + (x as usize),
                len: self.data.len(),
            });
        }
        self.data[(y as usize) * (self.width as usize) + (x as usize)] = value;
        Ok(())
    }

    /// Raw read-only data access.
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// Convert DPix to Pix.
    ///
    /// # Arguments
    ///
    /// * `out_depth` - Output depth: 8, 16, or 32. Use 0 for auto-detection.
    /// * `neg_handling` - How to handle negative values
    ///
    /// # Auto-detection
    ///
    /// When `out_depth` is 0:
    /// - If all values <= 255: use 8 bpp
    /// - Else if all values <= 65535: use 16 bpp
    /// - Else: use 32 bpp
    ///
    /// # Errors
    ///
    /// Returns an error if `out_depth` is not 0, 8, 16, or 32.
    ///
    /// # See also
    ///
    /// C Leptonica: `dpixConvertToPix()` in `fpix2.c`
    pub fn to_pix(&self, out_depth: u32, neg_handling: NegativeHandling) -> Result<Pix> {
        // Validate and determine output depth
        let depth = if out_depth == 0 {
            let max_val = self.data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if max_val <= 255.5 {
                8
            } else if max_val <= 65535.5 {
                16
            } else {
                32
            }
        } else if out_depth == 8 || out_depth == 16 || out_depth == 32 {
            out_depth
        } else {
            return Err(Error::InvalidParameter(format!(
                "out_depth must be 0, 8, 16, or 32, got {out_depth}"
            )));
        };

        let pix_depth = PixelDepth::from_bits(depth)?;
        let pix = Pix::new(self.width, self.height, pix_depth)?;
        let mut pm = pix.to_mut();

        // Preserve resolution metadata
        pm.set_xres(self.xres);
        pm.set_yres(self.yres);

        let max_val = match depth {
            8 => 255u32,
            16 => 65535u32,
            32 => u32::MAX,
            _ => unreachable!(),
        };

        let wpl = pm.wpl() as usize;
        let dst_data = pm.data_mut();

        for y in 0..self.height {
            let row_start = (y as usize) * wpl;
            let row_data = &mut dst_data[row_start..row_start + wpl];

            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let fval = self.data[idx];

                // Handle negative values
                let fval = if fval < 0.0 {
                    match neg_handling {
                        NegativeHandling::ClipToZero => 0.0,
                        NegativeHandling::TakeAbsValue => fval.abs(),
                    }
                } else {
                    fval
                };

                // Round and clamp
                let ival = (fval + 0.5) as u32;
                let ival = ival.min(max_val);

                // Direct word manipulation for performance
                match depth {
                    8 => {
                        let word_idx = (x / 4) as usize;
                        let shift = 24 - 8 * (x % 4);
                        let mask = !(0xffu32 << shift);
                        row_data[word_idx] = (row_data[word_idx] & mask) | (ival << shift);
                    }
                    16 => {
                        let word_idx = (x / 2) as usize;
                        let shift = 16 - 16 * (x % 2);
                        let mask = !(0xffffu32 << shift);
                        row_data[word_idx] = (row_data[word_idx] & mask) | (ival << shift);
                    }
                    32 => {
                        row_data[x as usize] = ival;
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(pm.into())
    }

    /// Convert DPix to FPix (lossy: f64 → f32).
    ///
    /// # See also
    ///
    /// C Leptonica: `dpixConvertToFPix()` in `fpix2.c`
    pub fn to_fpix(&self) -> FPix {
        let data: Vec<f32> = self.data.iter().map(|&v| v as f32).collect();
        FPix {
            width: self.width,
            height: self.height,
            data,
            xres: self.xres,
            yres: self.yres,
        }
    }

    /// Create DPix from FPix (lossless: f32 → f64).
    pub fn from_fpix(fpix: &FPix) -> Self {
        let data: Vec<f64> = fpix.data.iter().map(|&v| v as f64).collect();
        DPix {
            width: fpix.width,
            height: fpix.height,
            data,
            xres: fpix.xres,
            yres: fpix.yres,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpix_creation() {
        let fpix = FPix::new(100, 200).unwrap();
        assert_eq!(fpix.width(), 100);
        assert_eq!(fpix.height(), 200);
        assert_eq!(fpix.dimensions(), (100, 200));

        // Check all zeros
        for &val in fpix.data() {
            assert_eq!(val, 0.0);
        }
    }

    #[test]
    fn test_fpix_creation_with_value() {
        let fpix = FPix::new_with_value(50, 50, 0.5).unwrap();

        for &val in fpix.data() {
            assert_eq!(val, 0.5);
        }
    }

    #[test]
    fn test_fpix_invalid_dimensions() {
        assert!(FPix::new(0, 100).is_err());
        assert!(FPix::new(100, 0).is_err());
        assert!(FPix::new(0, 0).is_err());
    }

    #[test]
    fn test_fpix_from_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let fpix = FPix::from_data(3, 2, data).unwrap();

        assert_eq!(fpix.get_pixel(0, 0).unwrap(), 1.0);
        assert_eq!(fpix.get_pixel(1, 0).unwrap(), 2.0);
        assert_eq!(fpix.get_pixel(2, 0).unwrap(), 3.0);
        assert_eq!(fpix.get_pixel(0, 1).unwrap(), 4.0);
        assert_eq!(fpix.get_pixel(1, 1).unwrap(), 5.0);
        assert_eq!(fpix.get_pixel(2, 1).unwrap(), 6.0);
    }

    #[test]
    fn test_fpix_from_data_wrong_size() {
        let data = vec![1.0, 2.0, 3.0]; // Wrong size for 3x2
        assert!(FPix::from_data(3, 2, data).is_err());
    }

    #[test]
    fn test_fpix_pixel_access() {
        let mut fpix = FPix::new(10, 10).unwrap();

        fpix.set_pixel(5, 5, 1.5).unwrap();
        assert_eq!(fpix.get_pixel(5, 5).unwrap(), 1.5);

        fpix.set_pixel(0, 0, -0.5).unwrap();
        assert_eq!(fpix.get_pixel(0, 0).unwrap(), -0.5);

        fpix.set_pixel(9, 9, 100.0).unwrap();
        assert_eq!(fpix.get_pixel(9, 9).unwrap(), 100.0);
    }

    #[test]
    fn test_fpix_pixel_access_out_of_bounds() {
        let fpix = FPix::new(10, 10).unwrap();

        assert!(fpix.get_pixel(10, 0).is_err());
        assert!(fpix.get_pixel(0, 10).is_err());
        assert!(fpix.get_pixel(10, 10).is_err());
    }

    #[test]
    fn test_fpix_resolution() {
        let mut fpix = FPix::new(10, 10).unwrap();

        assert_eq!(fpix.xres(), 0);
        assert_eq!(fpix.yres(), 0);

        fpix.set_resolution(300, 300);
        assert_eq!(fpix.resolution(), (300, 300));

        fpix.set_xres(150);
        fpix.set_yres(200);
        assert_eq!(fpix.xres(), 150);
        assert_eq!(fpix.yres(), 200);
    }

    #[test]
    fn test_fpix_row_access() {
        let mut fpix = FPix::new(5, 3).unwrap();

        // Set some values
        for x in 0..5 {
            fpix.set_pixel(x, 1, (x + 1) as f32).unwrap();
        }

        let row = fpix.row(1);
        assert_eq!(row, &[1.0, 2.0, 3.0, 4.0, 5.0]);

        // Test mutable row
        let row_mut = fpix.row_mut(0);
        row_mut[0] = 10.0;
        assert_eq!(fpix.get_pixel(0, 0).unwrap(), 10.0);
    }

    #[test]
    fn test_fpix_set_all_and_clear() {
        let mut fpix = FPix::new(10, 10).unwrap();

        fpix.set_all(5.0);
        for &val in fpix.data() {
            assert_eq!(val, 5.0);
        }

        fpix.clear();
        for &val in fpix.data() {
            assert_eq!(val, 0.0);
        }
    }

    #[test]
    fn test_fpix_add() {
        let fpix1 = FPix::new_with_value(5, 5, 1.0).unwrap();
        let fpix2 = FPix::new_with_value(5, 5, 2.0).unwrap();

        let result = fpix1.add(&fpix2).unwrap();

        for &val in result.data() {
            assert_eq!(val, 3.0);
        }
    }

    #[test]
    fn test_fpix_sub() {
        let fpix1 = FPix::new_with_value(5, 5, 5.0).unwrap();
        let fpix2 = FPix::new_with_value(5, 5, 2.0).unwrap();

        let result = fpix1.sub(&fpix2).unwrap();

        for &val in result.data() {
            assert_eq!(val, 3.0);
        }
    }

    #[test]
    fn test_fpix_mul() {
        let fpix1 = FPix::new_with_value(5, 5, 3.0).unwrap();
        let fpix2 = FPix::new_with_value(5, 5, 4.0).unwrap();

        let result = fpix1.mul(&fpix2).unwrap();

        for &val in result.data() {
            assert_eq!(val, 12.0);
        }
    }

    #[test]
    fn test_fpix_div() {
        let fpix1 = FPix::new_with_value(5, 5, 10.0).unwrap();
        let fpix2 = FPix::new_with_value(5, 5, 2.0).unwrap();

        let result = fpix1.div(&fpix2).unwrap();

        for &val in result.data() {
            assert_eq!(val, 5.0);
        }
    }

    #[test]
    fn test_fpix_arithmetic_size_mismatch() {
        let fpix1 = FPix::new(10, 10).unwrap();
        let fpix2 = FPix::new(5, 5).unwrap();

        assert!(fpix1.add(&fpix2).is_err());
        assert!(fpix1.sub(&fpix2).is_err());
        assert!(fpix1.mul(&fpix2).is_err());
        assert!(fpix1.div(&fpix2).is_err());
    }

    #[test]
    fn test_fpix_constant_operations() {
        let mut fpix = FPix::new_with_value(5, 5, 2.0).unwrap();

        fpix.add_constant(3.0);
        for &val in fpix.data() {
            assert_eq!(val, 5.0);
        }

        fpix.mul_constant(2.0);
        for &val in fpix.data() {
            assert_eq!(val, 10.0);
        }
    }

    #[test]
    fn test_fpix_linear_combination() {
        let fpix = FPix::new_with_value(5, 5, 2.0).unwrap();

        let result = fpix.linear_combination(3.0, 1.0); // 3*2 + 1 = 7

        for &val in result.data() {
            assert_eq!(val, 7.0);
        }
    }

    #[test]
    fn test_fpix_min() {
        let mut fpix = FPix::new_with_value(10, 10, 5.0).unwrap();
        fpix.set_pixel(3, 7, -2.0).unwrap();

        let (min_val, min_x, min_y) = fpix.min().unwrap();
        assert_eq!(min_val, -2.0);
        assert_eq!(min_x, 3);
        assert_eq!(min_y, 7);

        assert_eq!(fpix.min_value(), Some(-2.0));
    }

    #[test]
    fn test_fpix_max() {
        let mut fpix = FPix::new_with_value(10, 10, 5.0).unwrap();
        fpix.set_pixel(8, 2, 100.0).unwrap();

        let (max_val, max_x, max_y) = fpix.max().unwrap();
        assert_eq!(max_val, 100.0);
        assert_eq!(max_x, 8);
        assert_eq!(max_y, 2);

        assert_eq!(fpix.max_value(), Some(100.0));
    }

    #[test]
    fn test_fpix_mean() {
        let fpix = FPix::new_with_value(10, 10, 4.0).unwrap();
        assert_eq!(fpix.mean(), Some(4.0));

        // Test with varied values
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let fpix = FPix::from_data(3, 2, data).unwrap();
        assert_eq!(fpix.mean(), Some(3.5)); // (1+2+3+4+5+6)/6 = 3.5
    }

    #[test]
    fn test_fpix_sum() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let fpix = FPix::from_data(2, 2, data).unwrap();
        assert_eq!(fpix.sum(), 10.0);
    }

    #[test]
    fn test_fpix_from_pix_8bit() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.to_mut();

        // Set some pixel values using raw data access
        let data = pix_mut.data_mut();
        // For 8-bit, 4 pixels per 32-bit word
        // Pixel layout: [P0 P1 P2 P3] in MSB to LSB
        data[0] = 0x64_C8_00_FF; // Row 0: 100, 200, 0, 255

        let pix: Pix = pix_mut.into();
        let fpix = FPix::from_pix(&pix).unwrap();

        assert_eq!(fpix.get_pixel(0, 0).unwrap(), 100.0);
        assert_eq!(fpix.get_pixel(1, 0).unwrap(), 200.0);
        assert_eq!(fpix.get_pixel(2, 0).unwrap(), 0.0);
        assert_eq!(fpix.get_pixel(3, 0).unwrap(), 255.0);
    }

    #[test]
    fn test_fpix_to_pix_8bit() {
        let mut fpix = FPix::new(4, 2).unwrap();
        fpix.set_pixel(0, 0, 0.0).unwrap();
        fpix.set_pixel(1, 0, 127.5).unwrap(); // Should round to 128
        fpix.set_pixel(2, 0, 255.0).unwrap();
        fpix.set_pixel(3, 0, 300.0).unwrap(); // Should clamp to 255

        let pix = fpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();

        assert_eq!(pix.depth(), PixelDepth::Bit8);
        assert_eq!(pix.width(), 4);
        assert_eq!(pix.height(), 2);
    }

    #[test]
    fn test_fpix_to_pix_negative_handling() {
        let mut fpix = FPix::new(2, 1).unwrap();
        fpix.set_pixel(0, 0, -10.0).unwrap();
        fpix.set_pixel(1, 0, 10.0).unwrap();

        // Test ClipToZero
        let pix1 = fpix.to_pix(8, NegativeHandling::ClipToZero).unwrap();
        let data1 = pix1.data();
        let val0 = (data1[0] >> 24) & 0xff;
        let val1 = (data1[0] >> 16) & 0xff;
        assert_eq!(val0, 0); // -10 clipped to 0
        assert_eq!(val1, 10); // 10.0 + 0.5 = 10.5 -> truncates to 10

        // Test TakeAbsValue
        let pix2 = fpix.to_pix(8, NegativeHandling::TakeAbsValue).unwrap();
        let data2 = pix2.data();
        let val0_abs = (data2[0] >> 24) & 0xff;
        assert_eq!(val0_abs, 10); // |-10| + 0.5 = 10.5 -> truncates to 10
    }

    #[test]
    fn test_fpix_auto_detect_depth() {
        // Should detect 8-bit
        let fpix8 = FPix::new_with_value(10, 10, 200.0).unwrap();
        let pix = fpix8.to_pix(0, NegativeHandling::ClipToZero).unwrap();
        assert_eq!(pix.depth(), PixelDepth::Bit8);

        // Should detect 16-bit
        let fpix16 = FPix::new_with_value(10, 10, 1000.0).unwrap();
        let pix = fpix16.to_pix(0, NegativeHandling::ClipToZero).unwrap();
        assert_eq!(pix.depth(), PixelDepth::Bit16);

        // Should detect 32-bit
        let fpix32 = FPix::new_with_value(10, 10, 100000.0).unwrap();
        let pix = fpix32.to_pix(0, NegativeHandling::ClipToZero).unwrap();
        assert_eq!(pix.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_fpix_operator_overloading() {
        let fpix1 = FPix::new_with_value(5, 5, 3.0).unwrap();
        let fpix2 = FPix::new_with_value(5, 5, 2.0).unwrap();

        let result = (&fpix1 + &fpix2).unwrap();
        assert_eq!(result.data()[0], 5.0);

        let result = (&fpix1 - &fpix2).unwrap();
        assert_eq!(result.data()[0], 1.0);

        let result = (&fpix1 * &fpix2).unwrap();
        assert_eq!(result.data()[0], 6.0);

        let result = (&fpix1 / &fpix2).unwrap();
        assert_eq!(result.data()[0], 1.5);
    }

    #[test]
    fn test_fpix_clone() {
        let fpix1 = FPix::new_with_value(10, 10, 5.0).unwrap();
        let fpix2 = fpix1.clone();

        // Should be independent copies
        assert_eq!(fpix1.data(), fpix2.data());
        assert_ne!(fpix1.data().as_ptr(), fpix2.data().as_ptr());
    }

    // -- Phase 17.2 FPix rendering tests --

    #[test]
    fn test_fpix_render_contours() {
        // Create a simple gradient FPix
        let mut fpix = FPix::new(50, 50).unwrap();
        for y in 0..50u32 {
            for x in 0..50u32 {
                fpix.set_pixel_unchecked(x, y, (x + y) as f32);
            }
        }
        let pix = fpix.render_contours(10.0, 0.15).unwrap();
        // Output should be 8bpp with colormap
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        assert!(pix.has_colormap());
    }

    #[test]
    fn test_fpix_auto_render_contours() {
        let mut fpix = FPix::new(50, 50).unwrap();
        for y in 0..50u32 {
            for x in 0..50u32 {
                fpix.set_pixel_unchecked(x, y, x as f32);
            }
        }
        let pix = fpix.auto_render_contours(10).unwrap();
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        assert!(pix.has_colormap());
    }
}
