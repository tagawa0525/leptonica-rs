//! Border operations for images
//!
//! Functions for adding and removing uniform borders around images.
//! Corresponds to `pixAddBorder()`, `pixAddBorderGeneral()`,
//! `pixRemoveBorder()`, and `pixRemoveBorderGeneral()` in C Leptonica's
//! `pix2.c`.

use super::Pix;
use crate::error::{Error, Result};

impl Pix {
    /// Add a uniform border of `npix` pixels on all sides.
    ///
    /// Creates a new image with dimensions `(w + 2*npix, h + 2*npix)`
    /// where the border region is filled with `val` and the interior
    /// contains a copy of the original image.
    ///
    /// C equivalent: `pixAddBorder()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `npix` - Number of pixels to add to each side
    /// * `val` - Fill value for the border pixels.
    ///   - For 1 bpp: 0 (white) or 1 (black)
    ///   - For 8 bpp: 0..255 (e.g. 0 for black, 255 for white)
    ///   - For 32 bpp: packed RGBA (e.g. 0 for black, 0xffffff00 for white)
    ///
    /// # Returns
    ///
    /// A new `Pix` with the added border. If `npix` is 0, returns a
    /// deep clone of the original.
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting image dimensions are invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let bordered = pix.add_border(10, 0).unwrap();
    /// assert_eq!(bordered.width(), 120);
    /// assert_eq!(bordered.height(), 100);
    /// ```
    pub fn add_border(&self, npix: u32, val: u32) -> Result<Pix> {
        if npix == 0 {
            return Ok(self.deep_clone());
        }
        self.add_border_general(npix, npix, npix, npix, val)
    }

    /// Add asymmetric borders with different widths on each side.
    ///
    /// Creates a new image with dimensions `(w + left + right, h + top + bot)`
    /// where the border region is filled with `val` and the interior
    /// contains a copy of the original image.
    ///
    /// C equivalent: `pixAddBorderGeneral()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `left` - Pixels to add on the left
    /// * `right` - Pixels to add on the right
    /// * `top` - Pixels to add on the top
    /// * `bot` - Pixels to add on the bottom
    /// * `val` - Fill value for the border pixels
    ///
    /// # Returns
    ///
    /// A new `Pix` with the added border.
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting image dimensions are invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
    /// let bordered = pix.add_border_general(5, 10, 15, 20, 255).unwrap();
    /// assert_eq!(bordered.width(), 115);   // 100 + 5 + 10
    /// assert_eq!(bordered.height(), 115);  // 80 + 15 + 20
    /// ```
    pub fn add_border_general(
        &self,
        left: u32,
        right: u32,
        top: u32,
        bot: u32,
        val: u32,
    ) -> Result<Pix> {
        let ws = self.width();
        let hs = self.height();
        let wd = ws + left + right;
        let hd = hs + top + bot;

        let pixd = Pix::new(wd, hd, self.depth())?;
        let mut pixd_mut = pixd.try_into_mut().unwrap();

        // Copy resolution from source
        pixd_mut.set_resolution(self.xres(), self.yres());

        // Fill border with val if non-zero (new image is already zero-filled)
        if val != 0 {
            for y in 0..hd {
                for x in 0..wd {
                    pixd_mut.set_pixel_unchecked(x, y, val);
                }
            }
        }

        // Copy source image into the interior
        for y in 0..hs {
            for x in 0..ws {
                let pixel = self.get_pixel_unchecked(x, y);
                pixd_mut.set_pixel_unchecked(x + left, y + top, pixel);
            }
        }

        Ok(pixd_mut.into())
    }

    /// Remove a uniform border of `npix` pixels from all sides.
    ///
    /// Creates a new image containing only the interior region, with the
    /// outer `npix` pixels removed from each side.
    ///
    /// C equivalent: `pixRemoveBorder()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `npix` - Number of pixels to remove from each side
    ///
    /// # Returns
    ///
    /// A new `Pix` with the border removed. If `npix` is 0, returns a
    /// deep clone of the original.
    ///
    /// # Errors
    ///
    /// Returns an error if removing the border would result in zero or
    /// negative dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(120, 100, PixelDepth::Bit8).unwrap();
    /// let inner = pix.remove_border(10).unwrap();
    /// assert_eq!(inner.width(), 100);
    /// assert_eq!(inner.height(), 80);
    /// ```
    pub fn remove_border(&self, npix: u32) -> Result<Pix> {
        if npix == 0 {
            return Ok(self.deep_clone());
        }
        self.remove_border_general(npix, npix, npix, npix)
    }

    /// Remove asymmetric borders with different widths from each side.
    ///
    /// Creates a new image containing only the interior region after
    /// removing the specified number of pixels from each side.
    ///
    /// C equivalent: `pixRemoveBorderGeneral()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `left` - Pixels to remove from the left
    /// * `right` - Pixels to remove from the right
    /// * `top` - Pixels to remove from the top
    /// * `bot` - Pixels to remove from the bottom
    ///
    /// # Returns
    ///
    /// A new `Pix` with the border removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting width or height would be zero
    /// or negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::{Pix, PixelDepth};
    ///
    /// let pix = Pix::new(115, 115, PixelDepth::Bit8).unwrap();
    /// let inner = pix.remove_border_general(5, 10, 15, 20).unwrap();
    /// assert_eq!(inner.width(), 100);   // 115 - 5 - 10
    /// assert_eq!(inner.height(), 80);   // 115 - 15 - 20
    /// ```
    pub fn remove_border_general(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<Pix> {
        let ws = self.width();
        let hs = self.height();

        if left + right >= ws {
            return Err(Error::InvalidParameter(format!(
                "border left({}) + right({}) >= width({})",
                left, right, ws
            )));
        }
        if top + bot >= hs {
            return Err(Error::InvalidParameter(format!(
                "border top({}) + bot({}) >= height({})",
                top, bot, hs
            )));
        }

        let wd = ws - left - right;
        let hd = hs - top - bot;

        let pixd = Pix::new(wd, hd, self.depth())?;
        let mut pixd_mut = pixd.try_into_mut().unwrap();

        // Copy resolution from source
        pixd_mut.set_resolution(self.xres(), self.yres());

        // Copy the interior region
        for y in 0..hd {
            for x in 0..wd {
                let pixel = self.get_pixel_unchecked(x + left, y + top);
                pixd_mut.set_pixel_unchecked(x, y, pixel);
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
    fn test_add_border_uniform() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(50, 40, 128).unwrap();
        let pix: Pix = pix_mut.into();

        let bordered = pix.add_border(10, 0).unwrap();
        assert_eq!(bordered.width(), 120);
        assert_eq!(bordered.height(), 100);
        // Original pixel at (50,40) should now be at (60,50)
        assert_eq!(bordered.get_pixel(60, 50), Some(128));
        // Border pixel should be 0
        assert_eq!(bordered.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_add_border_with_fill_value() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bordered = pix.add_border(5, 255).unwrap();
        assert_eq!(bordered.width(), 20);
        assert_eq!(bordered.height(), 20);
        // Border pixel should be 255
        assert_eq!(bordered.get_pixel(0, 0), Some(255));
        // Interior pixel should be 0 (original was all zeros)
        assert_eq!(bordered.get_pixel(5, 5), Some(0));
    }

    #[test]
    fn test_add_border_zero() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bordered = pix.add_border(0, 0).unwrap();
        assert_eq!(bordered.width(), 10);
        assert_eq!(bordered.height(), 10);
    }

    #[test]
    fn test_remove_border_uniform() {
        let pix = Pix::new(120, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(60, 50, 200).unwrap();
        let pix: Pix = pix_mut.into();

        let inner = pix.remove_border(10).unwrap();
        assert_eq!(inner.width(), 100);
        assert_eq!(inner.height(), 80);
        // Pixel at (60,50) should now be at (50,40)
        assert_eq!(inner.get_pixel(50, 40), Some(200));
    }

    #[test]
    fn test_remove_border_zero() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let inner = pix.remove_border(0).unwrap();
        assert_eq!(inner.width(), 10);
        assert_eq!(inner.height(), 10);
    }

    #[test]
    fn test_remove_border_too_large() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.remove_border(5).is_err());
        assert!(pix.remove_border(6).is_err());
    }

    #[test]
    fn test_add_remove_roundtrip() {
        let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..50u32 {
            for x in 0..50u32 {
                pix_mut.set_pixel(x, y, (x * 5 + y) % 256).unwrap();
            }
        }
        let pix: Pix = pix_mut.into();

        let bordered = pix.add_border(20, 0).unwrap();
        let recovered = bordered.remove_border(20).unwrap();

        assert_eq!(recovered.width(), pix.width());
        assert_eq!(recovered.height(), pix.height());
        for y in 0..50u32 {
            for x in 0..50u32 {
                assert_eq!(recovered.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_add_border_general() {
        let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();
        let bordered = pix.add_border_general(5, 10, 15, 20, 0).unwrap();
        assert_eq!(bordered.width(), 115);
        assert_eq!(bordered.height(), 115);
    }

    #[test]
    fn test_remove_border_general() {
        let pix = Pix::new(115, 115, PixelDepth::Bit8).unwrap();
        let inner = pix.remove_border_general(5, 10, 15, 20).unwrap();
        assert_eq!(inner.width(), 100);
        assert_eq!(inner.height(), 80);
    }

    #[test]
    fn test_add_border_1bpp() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(16, 16, 1).unwrap();
        let pix: Pix = pix_mut.into();

        let bordered = pix.add_border(10, 0).unwrap();
        assert_eq!(bordered.width(), 52);
        assert_eq!(bordered.height(), 52);
        assert_eq!(bordered.get_pixel(26, 26), Some(1));
        assert_eq!(bordered.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_remove_border_1bpp() {
        let pix = Pix::new(52, 52, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel(26, 26, 1).unwrap();
        let pix: Pix = pix_mut.into();

        let inner = pix.remove_border(10).unwrap();
        assert_eq!(inner.width(), 32);
        assert_eq!(inner.height(), 32);
        assert_eq!(inner.get_pixel(16, 16), Some(1));
    }

    #[test]
    fn test_add_border_preserves_resolution() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_resolution(300, 300);
        let pix: Pix = pix_mut.into();

        let bordered = pix.add_border(10, 0).unwrap();
        assert_eq!(bordered.xres(), 300);
        assert_eq!(bordered.yres(), 300);
    }
}
