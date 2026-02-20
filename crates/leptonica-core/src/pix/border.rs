//! Border operations for images
//!
//! Functions for adding, removing, and manipulating borders around images.
//! Corresponds to border functions in C Leptonica's `pix2.c`.

use super::{Pix, PixMut, PixelDepth};
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

    /// Add a mirrored border by reflecting pixels at the edges.
    ///
    /// Useful for convolution kernels that need valid edge data.
    /// The edge pixel is the axis of symmetry.
    ///
    /// C equivalent: `pixAddMirroredBorder()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `left` - Width of the border to add on the left side
    /// * `right` - Width of the border to add on the right side
    /// * `top` - Height of the border to add on the top side
    /// * `bot` - Height of the border to add on the bottom side
    ///
    /// # Errors
    ///
    /// Returns an error if any border size exceeds the corresponding
    /// image dimension.
    pub fn add_mirrored_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<Pix> {
        let w = self.width();
        let h = self.height();
        if left > w || right > w || top > h || bot > h {
            return Err(Error::InvalidParameter(
                "mirror border size exceeds image dimension".into(),
            ));
        }

        let bordered = self.add_border_general(left, right, top, bot, 0)?;
        let mut bm = bordered.try_into_mut().unwrap();
        let wd = w + left + right;

        // Mirror left border columns (within interior rows)
        for j in 0..left {
            for y in top..(top + h) {
                let src_x = left + j; // edge pixel is axis of symmetry
                let dst_x = left - 1 - j;
                bm.set_pixel_unchecked(dst_x, y, bm.get_pixel_unchecked(src_x, y));
            }
        }

        // Mirror right border columns (within interior rows)
        for j in 0..right {
            for y in top..(top + h) {
                let src_x = left + w - 1 - j;
                let dst_x = left + w + j;
                bm.set_pixel_unchecked(dst_x, y, bm.get_pixel_unchecked(src_x, y));
            }
        }

        // Mirror top border rows (full width including side borders)
        for j in 0..top {
            for x in 0..wd {
                let src_y = top + j;
                let dst_y = top - 1 - j;
                bm.set_pixel_unchecked(x, dst_y, bm.get_pixel_unchecked(x, src_y));
            }
        }

        // Mirror bottom border rows (full width including side borders)
        for j in 0..bot {
            for x in 0..wd {
                let src_y = top + h - 1 - j;
                let dst_y = top + h + j;
                bm.set_pixel_unchecked(x, dst_y, bm.get_pixel_unchecked(x, src_y));
            }
        }

        Ok(bm.into())
    }

    /// Add a repeated (tiled) border by wrapping pixels from opposite edges.
    ///
    /// C equivalent: `pixAddRepeatedBorder()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `left` - Width of the border to add on the left side
    /// * `right` - Width of the border to add on the right side
    /// * `top` - Height of the border to add on the top side
    /// * `bot` - Height of the border to add on the bottom side
    ///
    /// # Errors
    ///
    /// Returns an error if any border size exceeds the corresponding
    /// image dimension.
    pub fn add_repeated_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<Pix> {
        let w = self.width();
        let h = self.height();
        if left > w || right > w || top > h || bot > h {
            return Err(Error::InvalidParameter(
                "repeated border size exceeds image dimension".into(),
            ));
        }

        let bordered = self.add_border_general(left, right, top, bot, 0)?;
        let mut bm = bordered.try_into_mut().unwrap();
        let wd = w + left + right;

        // Left border: copy from right edge of original image
        for j in 0..left {
            for y in top..(top + h) {
                let src_x = left + w - left + j; // wrap from right
                let dst_x = j;
                bm.set_pixel_unchecked(dst_x, y, bm.get_pixel_unchecked(src_x, y));
            }
        }

        // Right border: copy from left edge of original image
        for j in 0..right {
            for y in top..(top + h) {
                let src_x = left + j; // wrap from left
                let dst_x = left + w + j;
                bm.set_pixel_unchecked(dst_x, y, bm.get_pixel_unchecked(src_x, y));
            }
        }

        // Top border: copy from bottom of image (full width after side borders)
        for j in 0..top {
            for x in 0..wd {
                let src_y = top + h - top + j; // wrap from bottom
                let dst_y = j;
                bm.set_pixel_unchecked(x, dst_y, bm.get_pixel_unchecked(x, src_y));
            }
        }

        // Bottom border: copy from top of image (full width after side borders)
        for j in 0..bot {
            for x in 0..wd {
                let src_y = top + j; // wrap from top
                let dst_y = top + h + j;
                bm.set_pixel_unchecked(x, dst_y, bm.get_pixel_unchecked(x, src_y));
            }
        }

        Ok(bm.into())
    }

    /// Add a black or white border with specified widths on each side.
    ///
    /// The border color value is automatically determined based on the
    /// image depth and the requested color.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAddBlackOrWhiteBorder()` in `pix2.c`
    pub fn add_black_or_white_border(
        &self,
        left: u32,
        right: u32,
        top: u32,
        bot: u32,
        color: super::InitColor,
    ) -> Result<Pix> {
        let val = super::PixMut::get_black_or_white_val(self, color);
        self.add_border_general(left, right, top, bot, val)
    }
}

impl PixMut {
    /// Set border pixels to a specified value in-place.
    ///
    /// Only supports 8, 16, and 32 bpp images.
    ///
    /// C equivalent: `pixSetBorderVal()` in `pix2.c`
    ///
    /// # Arguments
    ///
    /// * `left` - Width of border on the left side
    /// * `right` - Width of border on the right side
    /// * `top` - Height of border on the top side
    /// * `bot` - Height of border on the bottom side
    /// * `val` - Fill value for border pixels (masked to depth)
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 8, 16, or 32 bpp.
    pub fn set_border_val(
        &mut self,
        left: u32,
        right: u32,
        top: u32,
        bot: u32,
        val: u32,
    ) -> Result<()> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 && d != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }

        let w = self.width();
        let h = self.height();

        // Mask value to valid range for depth
        let val = match d {
            PixelDepth::Bit8 => val & 0xff,
            PixelDepth::Bit16 => val & 0xffff,
            _ => val,
        };

        // Top border rows (full width)
        for y in 0..top.min(h) {
            for x in 0..w {
                self.set_pixel_unchecked(x, y, val);
            }
        }

        // Bottom border rows (full width)
        let bot_start = h.saturating_sub(bot);
        for y in bot_start..h {
            for x in 0..w {
                self.set_pixel_unchecked(x, y, val);
            }
        }

        // Left and right borders (middle rows only)
        let y_start = top.min(h);
        let y_end = bot_start.max(y_start);
        for y in y_start..y_end {
            for x in 0..left.min(w) {
                self.set_pixel_unchecked(x, y, val);
            }
            let right_start = w.saturating_sub(right);
            for x in right_start..w {
                self.set_pixel_unchecked(x, y, val);
            }
        }

        Ok(())
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
    fn test_set_border_val_8bpp() {
        let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Fill interior with 100
        for y in 0..8 {
            for x in 0..10 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        pm.set_border_val(2, 2, 2, 2, 255).unwrap();
        let pix: Pix = pm.into();
        // Border pixels should be 255
        assert_eq!(pix.get_pixel(0, 0), Some(255));
        assert_eq!(pix.get_pixel(9, 7), Some(255));
        assert_eq!(pix.get_pixel(1, 0), Some(255)); // top row
        assert_eq!(pix.get_pixel(0, 3), Some(255)); // left col
        // Interior should remain 100
        assert_eq!(pix.get_pixel(5, 4), Some(100));
        assert_eq!(pix.get_pixel(2, 2), Some(100));
        assert_eq!(pix.get_pixel(7, 5), Some(100));
    }

    #[test]
    fn test_set_border_val_32bpp() {
        let pix = Pix::new(6, 4, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_border_val(1, 1, 1, 1, 0xff000000).unwrap();
        let pix: Pix = pm.into();
        // Corner should be set
        assert_eq!(pix.get_pixel(0, 0), Some(0xff000000));
        // Interior should be 0
        assert_eq!(pix.get_pixel(2, 2), Some(0));
    }

    #[test]
    fn test_set_border_val_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        assert!(pm.set_border_val(1, 1, 1, 1, 1).is_err());
    }

    #[test]
    fn test_add_mirrored_border() {
        // 4x3 image with known pixels
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..3u32 {
            for x in 0..4u32 {
                pm.set_pixel_unchecked(x, y, x * 10 + y);
            }
        }
        let pix: Pix = pm.into();

        let bordered = pix.add_mirrored_border(2, 2, 1, 1).unwrap();
        assert_eq!(bordered.width(), 8); // 4 + 2 + 2
        assert_eq!(bordered.height(), 5); // 3 + 1 + 1

        // Interior preserved: original (0,0) is at (2,1) in bordered
        assert_eq!(bordered.get_pixel(2, 1), Some(0)); // x=0, y=0
        assert_eq!(bordered.get_pixel(3, 1), Some(10)); // x=1, y=0

        // Left mirror: col at x=1 mirrors original x=0 (which is at bordered x=2)
        // x=1 in bordered = mirror of x=2 in bordered = original x=0
        assert_eq!(bordered.get_pixel(1, 1), bordered.get_pixel(2, 1));
        // x=0 in bordered = mirror of x=3 in bordered = original x=1
        assert_eq!(bordered.get_pixel(0, 1), bordered.get_pixel(3, 1));

        // Right mirror: x=6 mirrors x=5 (original x=3)
        assert_eq!(bordered.get_pixel(6, 1), bordered.get_pixel(5, 1));
        assert_eq!(bordered.get_pixel(7, 1), bordered.get_pixel(4, 1));

        // Top mirror: y=0 copies from y=1 (first interior row)
        for x in 0..8 {
            assert_eq!(
                bordered.get_pixel(x, 0),
                bordered.get_pixel(x, 1),
                "top mirror mismatch at x={}",
                x
            );
        }

        // Bottom mirror: y=4 copies from y=3 (last interior row)
        for x in 0..8 {
            assert_eq!(
                bordered.get_pixel(x, 4),
                bordered.get_pixel(x, 3),
                "bottom mirror mismatch at x={}",
                x
            );
        }
    }

    #[test]
    fn test_add_mirrored_border_too_large() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.add_mirrored_border(11, 0, 0, 0).is_err());
        assert!(pix.add_mirrored_border(0, 0, 11, 0).is_err());
    }

    #[test]
    fn test_add_repeated_border() {
        // 4x3 image
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..3u32 {
            for x in 0..4u32 {
                pm.set_pixel_unchecked(x, y, x * 10 + y + 1);
            }
        }
        let pix: Pix = pm.into();

        let bordered = pix.add_repeated_border(2, 2, 1, 1).unwrap();
        assert_eq!(bordered.width(), 8);
        assert_eq!(bordered.height(), 5);

        // Interior preserved
        assert_eq!(bordered.get_pixel(2, 1), Some(1)); // original (0,0)
        assert_eq!(bordered.get_pixel(5, 3), Some(33)); // original (3,2)

        // Left border: wraps from right edge of image
        // x=0 in bordered gets original x=(w-2)=2, x=1 gets original x=3
        assert_eq!(bordered.get_pixel(0, 1), pix.get_pixel(2, 0));
        assert_eq!(bordered.get_pixel(1, 1), pix.get_pixel(3, 0));

        // Right border: wraps from left edge of image
        // x=6 gets original x=0, x=7 gets original x=1
        assert_eq!(bordered.get_pixel(6, 1), pix.get_pixel(0, 0));
        assert_eq!(bordered.get_pixel(7, 1), pix.get_pixel(1, 0));

        // Top border: wraps from bottom edge
        // y=0 gets original y=(h-1)=2
        assert_eq!(bordered.get_pixel(2, 0), pix.get_pixel(0, 2));

        // Bottom border: wraps from top edge
        // y=4 gets original y=0
        assert_eq!(bordered.get_pixel(2, 4), pix.get_pixel(0, 0));
    }

    #[test]
    fn test_add_repeated_border_too_large() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.add_repeated_border(11, 0, 0, 0).is_err());
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

    // ================================================================
    // Phase 1.6: pixAddBlackOrWhiteBorder
    // ================================================================

    #[test]

    fn test_add_black_or_white_border_white_8bpp() {
        use super::super::InitColor;
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bordered = pix
            .add_black_or_white_border(5, 5, 5, 5, InitColor::White)
            .unwrap();
        assert_eq!(bordered.width(), 20);
        assert_eq!(bordered.height(), 20);
        // Border should be white (255 for 8bpp)
        assert_eq!(bordered.get_pixel(0, 0), Some(255));
    }

    #[test]

    fn test_add_black_or_white_border_black_8bpp() {
        use super::super::InitColor;
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let bordered = pix
            .add_black_or_white_border(3, 3, 3, 3, InitColor::Black)
            .unwrap();
        assert_eq!(bordered.width(), 16);
        assert_eq!(bordered.height(), 16);
        // Border should be black (0 for 8bpp)
        assert_eq!(bordered.get_pixel(0, 0), Some(0));
    }

    #[test]

    fn test_add_black_or_white_border_1bpp() {
        use super::super::InitColor;
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        // 1bpp: black=1, white=0
        let bordered = pix
            .add_black_or_white_border(2, 2, 2, 2, InitColor::Black)
            .unwrap();
        assert_eq!(bordered.get_pixel(0, 0), Some(1));

        let bordered = pix
            .add_black_or_white_border(2, 2, 2, 2, InitColor::White)
            .unwrap();
        assert_eq!(bordered.get_pixel(0, 0), Some(0));
    }
}
