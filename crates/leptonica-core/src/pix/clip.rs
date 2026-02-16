//! Rectangle clipping operations for images
//!
//! Functions for extracting rectangular sub-regions from images,
//! foreground detection, mask generation, and line averaging.
//!
//! # See also
//!
//! C Leptonica: `pix2.c`, `pix5.c`

use super::{Pix, PixelDepth};
use crate::Box;
use crate::error::{Error, Result};

/// Direction for scanning an image to find the foreground edge.
///
/// # See also
///
/// C Leptonica: `L_FROM_LEFT`, `L_FROM_RIGHT`, `L_FROM_TOP`, `L_FROM_BOT`
/// in `pix5.c`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    /// Scan from left edge toward right
    FromLeft,
    /// Scan from right edge toward left
    FromRight,
    /// Scan from top edge toward bottom
    FromTop,
    /// Scan from bottom edge toward top
    FromBot,
}

impl Pix {
    /// Extract a rectangular sub-region from the image.
    ///
    /// Creates a new image containing the specified rectangle. If the
    /// rectangle extends beyond the image bounds, it is clipped to the
    /// valid region. Returns an error if the rectangle is entirely outside
    /// the image.
    ///
    /// For 32-bit images, the output preserves the samples-per-pixel
    /// value from the source.
    ///
    /// C equivalent: `pixClipRectangle()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `x` - Left edge of the rectangle
    /// * `y` - Top edge of the rectangle
    /// * `w` - Width of the rectangle
    /// * `h` - Height of the rectangle
    ///
    /// # Returns
    ///
    /// A new `Pix` containing the clipped region.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The requested width or height is 0
    /// - The rectangle is entirely outside the image bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let clipped = pix.clip_rectangle(10, 20, 50, 40).unwrap();
    /// assert_eq!(clipped.width(), 50);
    /// assert_eq!(clipped.height(), 40);
    /// ```
    ///
    /// Regions extending beyond the image are clipped:
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let clipped = pix.clip_rectangle(80, 60, 50, 50).unwrap();
    /// assert_eq!(clipped.width(), 20);   // clipped: 100 - 80
    /// assert_eq!(clipped.height(), 20);  // clipped: 80 - 60
    /// ```
    pub fn clip_rectangle(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Pix> {
        if w == 0 || h == 0 {
            return Err(Error::InvalidParameter(format!(
                "clip rectangle has zero dimension: {}x{}",
                w, h
            )));
        }

        let src_w = self.width();
        let src_h = self.height();

        // Check if the rectangle is entirely outside the image
        if x >= src_w || y >= src_h {
            return Err(Error::InvalidParameter(format!(
                "clip rectangle origin ({}, {}) is outside image bounds ({}x{})",
                x, y, src_w, src_h
            )));
        }

        // Clip the rectangle to the image bounds
        let clip_w = w.min(src_w - x);
        let clip_h = h.min(src_h - y);

        let depth = self.depth();
        let pixd = Pix::new(clip_w, clip_h, depth)?;
        let mut pixd_mut = pixd.try_into_mut().unwrap();

        // Preserve spp for 32-bit images
        if depth == PixelDepth::Bit32 {
            pixd_mut.set_spp(self.spp());
        }

        // Copy resolution from source
        pixd_mut.set_resolution(self.xres(), self.yres());

        // Copy pixel data
        for dy in 0..clip_h {
            for dx in 0..clip_w {
                let val = self.get_pixel_unchecked(x + dx, y + dy);
                pixd_mut.set_pixel_unchecked(dx, dy, val);
            }
        }

        Ok(pixd_mut.into())
    }
}

// ============================================================================
// Advanced clipping, foreground detection, and mask generation (pix5.c)
// ============================================================================

impl Pix {
    /// Extract a rectangular sub-region with an additional border.
    ///
    /// Clips the region to the image bounds, then adds a border of the
    /// specified width around the clipped area (filled with zeros).
    /// The border is clamped so it does not exceed the distance from
    /// the region to the image edge.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipRectangleWithBorder()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `region` - The rectangle to extract
    /// * `border` - Border width to add around the region
    ///
    /// # Errors
    ///
    /// Returns an error if the region is entirely outside the image.
    pub fn clip_rectangle_with_border(&self, _region: &Box, _border: u32) -> Result<(Pix, Box)> {
        todo!()
    }

    /// Crop two images to their overlapping region so they have the same size.
    ///
    /// Both images are cropped to the minimum of their widths and heights,
    /// taken from the upper-left corner.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCropToMatch()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `other` - The second image to match sizes with
    ///
    /// # Errors
    ///
    /// Returns an error if either image has zero dimensions after cropping.
    pub fn crop_to_match(&self, _other: &Pix) -> Result<(Pix, Pix)> {
        todo!()
    }

    /// Clip the image to the bounding box of its foreground pixels.
    ///
    /// Only works on 1bpp images. Foreground pixels have value 1.
    /// Returns `None` if no foreground pixels are found.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipToForeground()` in `pix5.c`
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp.
    pub fn clip_to_foreground(&self) -> Result<Option<(Pix, Box)>> {
        todo!()
    }

    /// Scan from the specified direction to find the first foreground pixel.
    ///
    /// Only works on 1bpp images. Scans within the given region.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixScanForForeground()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `region` - The region to scan within
    /// * `direction` - The direction to scan from
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp or no foreground is found.
    pub fn scan_for_foreground(&self, _region: &Box, _direction: ScanDirection) -> Result<u32> {
        todo!()
    }

    /// Clip a box to the foreground region of a 1bpp image.
    ///
    /// If `input_box` is `None`, the entire image is used. Returns the
    /// clipped image and its bounding box. Returns `None` if no foreground
    /// is found in the region.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixClipBoxToForeground()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `input_box` - Optional region to search within; `None` for entire image
    ///
    /// # Errors
    ///
    /// Returns an error if the image is not 1bpp.
    pub fn clip_box_to_foreground(&self, _input_box: Option<&Box>) -> Result<Option<(Pix, Box)>> {
        todo!()
    }

    /// Create a 1bpp frame mask with an annular ring of ON pixels.
    ///
    /// The mask has a rectangular ring of foreground (ON) pixels,
    /// with inner and outer boundaries specified as fractions of
    /// the image dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixMakeFrameMask()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `w` - Width of the output mask
    /// * `h` - Height of the output mask
    /// * `hf1` - Horizontal fraction for inner left/right boundary
    /// * `hf2` - Horizontal fraction for outer left/right boundary
    /// * `vf1` - Vertical fraction for inner top/bottom boundary
    /// * `vf2` - Vertical fraction for outer top/bottom boundary
    ///
    /// # Errors
    ///
    /// Returns an error if any fraction is not in [0.0, 1.0] or
    /// if inner fractions exceed outer fractions.
    pub fn make_frame_mask(
        _w: u32,
        _h: u32,
        _hf1: f32,
        _hf2: f32,
        _vf1: f32,
        _vf2: f32,
    ) -> Result<Pix> {
        todo!()
    }

    /// Compute the fraction of foreground pixels in the source that
    /// are also foreground in the mask.
    ///
    /// Both images must be 1bpp and the same size.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixFractionFgInMask()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - The mask image (1bpp)
    ///
    /// # Errors
    ///
    /// Returns an error if either image is not 1bpp or sizes differ.
    pub fn fraction_fg_in_mask(&self, _mask: &Pix) -> Result<f32> {
        todo!()
    }

    /// Compute the average pixel value along a line.
    ///
    /// Extracts pixel values along a line from `(x1, y1)` to `(x2, y2)`
    /// and returns their average. Works on 8bpp grayscale images.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAverageOnLine()` in `pix5.c`
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` - One endpoint of the line
    /// * `x2`, `y2` - Other endpoint of the line
    /// * `factor` - Sampling factor (>= 1)
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 8bpp.
    pub fn average_on_line(
        &self,
        _x1: i32,
        _y1: i32,
        _x2: i32,
        _y2: i32,
        _factor: i32,
    ) -> Result<f32> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::super::PixelDepth;
    use super::*;

    #[test]
    fn test_clip_rectangle_basic() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(50, 40, 128).unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(40, 30, 20, 20).unwrap();
        assert_eq!(clipped.width(), 20);
        assert_eq!(clipped.height(), 20);
        // Original pixel at (50,40) should now be at (10,10) in clipped image
        assert_eq!(clipped.get_pixel(10, 10), Some(128));
    }

    #[test]
    fn test_clip_rectangle_full_image() {
        let pix = Pix::new(50, 30, PixelDepth::Bit8).unwrap();
        let clipped = pix.clip_rectangle(0, 0, 50, 30).unwrap();
        assert_eq!(clipped.width(), 50);
        assert_eq!(clipped.height(), 30);
    }

    #[test]
    fn test_clip_rectangle_clips_to_bounds() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        // Request extends beyond the right and bottom edges
        let clipped = pix.clip_rectangle(80, 60, 50, 50).unwrap();
        assert_eq!(clipped.width(), 20); // 100 - 80
        assert_eq!(clipped.height(), 20); // 80 - 60
    }

    #[test]
    fn test_clip_rectangle_entirely_outside() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        assert!(pix.clip_rectangle(100, 0, 10, 10).is_err());
        assert!(pix.clip_rectangle(0, 80, 10, 10).is_err());
        assert!(pix.clip_rectangle(200, 200, 10, 10).is_err());
    }

    #[test]
    fn test_clip_rectangle_zero_size() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        assert!(pix.clip_rectangle(0, 0, 0, 10).is_err());
        assert!(pix.clip_rectangle(0, 0, 10, 0).is_err());
    }

    #[test]
    fn test_clip_rectangle_1bpp() {
        let pix = Pix::new(64, 64, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(32, 32, 1).unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(20, 20, 30, 30).unwrap();
        assert_eq!(clipped.width(), 30);
        assert_eq!(clipped.height(), 30);
        // Pixel at (32,32) in source -> (12,12) in clipped
        assert_eq!(clipped.get_pixel(12, 12), Some(1));
        assert_eq!(clipped.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_clip_rectangle_32bpp() {
        use crate::color::compose_rgb;

        let pix = Pix::new(100, 80, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut
            .set_pixel(50, 40, compose_rgb(200, 100, 50))
            .unwrap();
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(40, 30, 20, 20).unwrap();
        assert_eq!(clipped.width(), 20);
        assert_eq!(clipped.height(), 20);
        assert_eq!(clipped.depth(), PixelDepth::Bit32);
        assert_eq!(clipped.spp(), pix.spp());

        let (r, g, b) = clipped.get_rgb(10, 10).unwrap();
        assert_eq!((r, g, b), (200, 100, 50));
    }

    #[test]
    fn test_clip_rectangle_preserves_resolution() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_resolution(300, 300);
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(10, 10, 50, 50).unwrap();
        assert_eq!(clipped.xres(), 300);
        assert_eq!(clipped.yres(), 300);
    }

    #[test]
    fn test_clip_rectangle_pixel_values() {
        // Verify that all pixels are correctly copied
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..20u32 {
            for x in 0..20u32 {
                pix_mut.set_pixel(x, y, (x + y * 20) % 256).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let clipped = pix.clip_rectangle(5, 5, 10, 10).unwrap();
        for y in 0..10u32 {
            for x in 0..10u32 {
                let expected = ((x + 5) + (y + 5) * 20) % 256;
                assert_eq!(clipped.get_pixel(x, y), Some(expected));
            }
        }
    }
}
