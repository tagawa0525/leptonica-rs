//! Rectangle clipping operations for images
//!
//! Functions for extracting rectangular sub-regions from images.
//! Corresponds to `pixClipRectangle()` in C Leptonica's `pix2.c`.

use super::{Pix, PixelDepth};
use crate::error::{Error, Result};

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
