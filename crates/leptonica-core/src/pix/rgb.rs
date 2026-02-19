//! RGB component operations
//!
//! Functions for extracting, setting, and composing individual color
//! channels of 32 bpp images.
//!
//! # See also
//!
//! C Leptonica: `pix2.c` (`pixGetRGBComponent`, `pixSetRGBComponent`, etc.)

use super::{Pix, PixMut, PixelDepth};
use crate::color;
use crate::error::{Error, Result};

/// Color component selector for RGB channel operations.
///
/// # See also
///
/// C Leptonica: `COLOR_RED`, `COLOR_GREEN`, `COLOR_BLUE`, `L_ALPHA_CHANNEL`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbComponent {
    /// Red channel (bits 24-31)
    Red,
    /// Green channel (bits 16-23)
    Green,
    /// Blue channel (bits 8-15)
    Blue,
    /// Alpha channel (bits 0-7)
    Alpha,
}

impl Pix {
    /// Extract a single color component as an 8 bpp grayscale image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRGBComponent()` in `pix2.c`
    pub fn get_rgb_component(&self, comp: RgbComponent) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let val = match comp {
                    RgbComponent::Red => color::red(pixel),
                    RgbComponent::Green => color::green(pixel),
                    RgbComponent::Blue => color::blue(pixel),
                    RgbComponent::Alpha => color::alpha(pixel),
                };
                result_mut.set_pixel_unchecked(x, y, val as u32);
            }
        }

        Ok(result_mut.into())
    }

    /// Create a 32 bpp RGB image from three 8 bpp component images.
    ///
    /// All three images must have the same dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCreateRGBImage()` in `pix2.c`
    pub fn create_rgb_image(pix_r: &Pix, pix_g: &Pix, pix_b: &Pix) -> Result<Pix> {
        // Validate depths
        if pix_r.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(pix_r.depth().bits()));
        }
        if pix_g.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(pix_g.depth().bits()));
        }
        if pix_b.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(pix_b.depth().bits()));
        }

        // Validate dimensions match
        let w = pix_r.width();
        let h = pix_r.height();
        if pix_g.width() != w || pix_g.height() != h || pix_b.width() != w || pix_b.height() != h {
            return Err(Error::InvalidParameter(
                "all component images must have the same dimensions".into(),
            ));
        }

        let result = Pix::new(w, h, PixelDepth::Bit32)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(pix_r.xres(), pix_r.yres());

        for y in 0..h {
            for x in 0..w {
                let r = pix_r.get_pixel_unchecked(x, y) as u8;
                let g = pix_g.get_pixel_unchecked(x, y) as u8;
                let b = pix_b.get_pixel_unchecked(x, y) as u8;
                result_mut.set_pixel_unchecked(x, y, color::compose_rgb(r, g, b));
            }
        }

        Ok(result_mut.into())
    }
}

impl Pix {
    /// Extract a single RGB component from a colormapped image.
    ///
    /// Returns an 8 bpp grayscale image where each pixel value is the
    /// specified component from the colormap lookup.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRGBComponentCmap()` in `pix2.c`
    pub fn get_rgb_component_cmap(&self, comp: RgbComponent) -> Result<Pix> {
        let cmap = self
            .colormap()
            .ok_or_else(|| Error::InvalidParameter("image has no colormap".into()))?;

        let w = self.width();
        let h = self.height();
        let result = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut result_mut = result.try_into_mut().unwrap();
        result_mut.set_resolution(self.xres(), self.yres());

        for y in 0..h {
            for x in 0..w {
                let index = self.get_pixel_unchecked(x, y) as usize;
                let val = match cmap.get(index) {
                    Some(rgba) => match comp {
                        RgbComponent::Red => rgba.red,
                        RgbComponent::Green => rgba.green,
                        RgbComponent::Blue => rgba.blue,
                        RgbComponent::Alpha => rgba.alpha,
                    },
                    None => 0,
                };
                result_mut.set_pixel_unchecked(x, y, val as u32);
            }
        }

        Ok(result_mut.into())
    }

    /// Extract R, G, B values from a single row of a 32 bpp image.
    ///
    /// Returns three vectors of length `width`, one for each channel.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRGBLine()` in `pix2.c`
    pub fn get_rgb_line(&self, row: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if row >= self.height() {
            return Err(Error::IndexOutOfBounds {
                index: row as usize,
                len: self.height() as usize,
            });
        }

        let w = self.width() as usize;
        let mut buf_r = Vec::with_capacity(w);
        let mut buf_g = Vec::with_capacity(w);
        let mut buf_b = Vec::with_capacity(w);

        for x in 0..self.width() {
            let pixel = self.get_pixel_unchecked(x, row);
            let (r, g, b) = color::extract_rgb(pixel);
            buf_r.push(r);
            buf_g.push(g);
            buf_b.push(b);
        }

        Ok((buf_r, buf_g, buf_b))
    }

    /// Check if all alpha values are 255 (fully opaque).
    ///
    /// Returns `true` if the image is 32 bpp with spp == 4 and all
    /// alpha values are 255.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixAlphaIsOpaque()` in `pix2.c`
    pub fn alpha_is_opaque(&self) -> Result<bool> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if self.spp() != 4 {
            return Err(Error::InvalidParameter(
                "alpha_is_opaque requires 32 bpp with spp == 4".into(),
            ));
        }

        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel = self.get_pixel_unchecked(x, y);
                if color::alpha(pixel) != 255 {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Infer image resolution from physical dimensions.
    ///
    /// Given the long side of the image in inches, returns the
    /// estimated resolution in PPI.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixInferResolution()` in `pix2.c`
    pub fn infer_resolution(&self, longside_inches: f32) -> Result<i32> {
        if longside_inches <= 0.0 {
            return Err(Error::InvalidParameter(
                "longside_inches must be positive".into(),
            ));
        }

        let w = self.width() as f32;
        let h = self.height() as f32;
        let longside_pixels = w.max(h);
        let res = (longside_pixels / longside_inches + 0.5) as i32;
        Ok(res)
    }

    /// Create a new image with endian byte-swapped data.
    ///
    /// Swaps bytes within each 32-bit word: ABCD -> DCBA.
    /// On big-endian systems, returns a clone.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixEndianByteSwapNew()` in `pix2.c`
    pub fn endian_byte_swap_new(&self) -> Pix {
        if cfg!(target_endian = "big") {
            return self.clone();
        }
        let mut result_mut = self.to_mut();
        result_mut.endian_byte_swap();
        result_mut.into()
    }

    /// Create a new image with endian two-byte swapped data.
    ///
    /// Swaps 16-bit half-words within each 32-bit word: AABB -> BBAA.
    /// On big-endian systems, returns a clone.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixEndianTwoByteSwapNew()` in `pix2.c`
    pub fn endian_two_byte_swap_new(&self) -> Pix {
        if cfg!(target_endian = "big") {
            return self.clone();
        }
        let mut result_mut = self.to_mut();
        result_mut.endian_two_byte_swap();
        result_mut.into()
    }

    /// Extract raster data as a flat byte vector.
    ///
    /// For 32 bpp images, extracts RGB data only (3 bytes per pixel),
    /// stripping the alpha channel. For other depths, returns raw
    /// raster data with pad bits cleared.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetRasterData()` in `pix2.c`
    pub fn get_raster_data(&self) -> Result<Vec<u8>> {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        let wpl = self.wpl();

        if d == PixelDepth::Bit32 {
            // 32 bpp: extract RGB only (3 bytes per pixel)
            let mut data = Vec::with_capacity((w * h * 3) as usize);
            for y in 0..h {
                for x in 0..w {
                    let pixel = self.get_pixel_unchecked(x, y);
                    let (r, g, b) = color::extract_rgb(pixel);
                    data.push(r);
                    data.push(g);
                    data.push(b);
                }
            }
            Ok(data)
        } else {
            // Other depths: return raw byte data, row by row, with pad bits cleared.
            //
            // For depths < 32 bpp, pixels are packed into 32-bit words with
            // padding in the least significant bits of the last word per row.
            let bytes_per_row = wpl * 4;
            let mut data = Vec::with_capacity((h * bytes_per_row) as usize);

            let bits_per_pixel = d.bits();
            let valid_bits_per_row = w * bits_per_pixel;
            let needed_words_per_row = valid_bits_per_row.div_ceil(32) as usize;
            let remaining_bits = valid_bits_per_row % 32;

            let raw_data = self.data();
            for y in 0..h {
                let row_start = (y * wpl) as usize;
                let row_end = row_start + wpl as usize;

                for (i, &word) in raw_data[row_start..row_end].iter().enumerate() {
                    let masked = if i >= needed_words_per_row {
                        // Entire word is padding beyond the image width.
                        0
                    } else if remaining_bits > 0 && i == needed_words_per_row - 1 {
                        // Last partially used word: clear pad bits.
                        // Pixels occupy MSB; padding is in the LSB.
                        let mask: u32 = u32::MAX << (32 - remaining_bits);
                        word & mask
                    } else {
                        word
                    };
                    data.extend_from_slice(&masked.to_be_bytes());
                }
            }
            Ok(data)
        }
    }
}

impl PixMut {
    /// Copy a single color component from source to destination.
    ///
    /// Both images must be 32 bpp. Only the specified component is
    /// copied; other components are preserved.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCopyRGBComponent()` in `pix2.c`
    pub fn copy_rgb_component(&mut self, src: &Pix, comp: RgbComponent) -> Result<()> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if src.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(src.depth().bits()));
        }

        if comp == RgbComponent::Alpha {
            self.set_spp(4);
        }

        let w = self.width().min(src.width());
        let h = self.height().min(src.height());

        for y in 0..h {
            for x in 0..w {
                let src_pixel = src.get_pixel_unchecked(x, y);
                let dst_pixel = self.get_pixel_unchecked(x, y);
                let (mut r, mut g, mut b, mut a) = color::extract_rgba(dst_pixel);
                let (sr, sg, sb, sa) = color::extract_rgba(src_pixel);

                match comp {
                    RgbComponent::Red => r = sr,
                    RgbComponent::Green => g = sg,
                    RgbComponent::Blue => b = sb,
                    RgbComponent::Alpha => a = sa,
                }

                self.set_pixel_unchecked(x, y, color::compose_rgba(r, g, b, a));
            }
        }

        Ok(())
    }

    /// Swap bytes within each 32-bit word in-place: ABCD -> DCBA.
    ///
    /// On big-endian systems, this is a no-op.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixEndianByteSwap()` in `pix2.c`
    pub fn endian_byte_swap(&mut self) {
        if cfg!(target_endian = "big") {
            return;
        }
        for word in self.inner.data.iter_mut() {
            *word = word.swap_bytes();
        }
    }

    /// Swap 16-bit half-words within each 32-bit word in-place: AABB -> BBAA.
    ///
    /// On big-endian systems, this is a no-op.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixEndianTwoByteSwap()` in `pix2.c`
    pub fn endian_two_byte_swap(&mut self) {
        if cfg!(target_endian = "big") {
            return;
        }
        for word in self.inner.data.iter_mut() {
            *word = (*word).rotate_right(16);
        }
    }

    /// Set a pixel in a colormapped image by RGB value.
    ///
    /// Finds the nearest color in the colormap and sets the pixel
    /// index accordingly. If no colormap is present, returns an error.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetCmapPixel()` in `pix2.c`
    pub fn set_cmap_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> Result<()> {
        let index = self
            .inner
            .colormap
            .as_ref()
            .ok_or_else(|| Error::InvalidParameter("image has no colormap".into()))?
            .find_nearest(r, g, b)
            .ok_or_else(|| Error::InvalidParameter("colormap is empty".into()))?;

        self.set_pixel(x, y, index as u32)
    }
}

impl PixMut {
    /// Set a single color component from an 8 bpp source image.
    ///
    /// The source image values replace the specified component channel
    /// in this 32 bpp image. If the source image is smaller than this
    /// image, only the overlapping region is modified; pixels outside
    /// the source bounds are left unchanged. If setting the alpha
    /// component, spp is automatically set to 4.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetRGBComponent()` in `pix2.c`
    pub fn set_rgb_component(&mut self, src: &Pix, comp: RgbComponent) -> Result<()> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if src.depth() != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(src.depth().bits()));
        }

        if comp == RgbComponent::Alpha {
            self.set_spp(4);
        }

        let w = self.width().min(src.width());
        let h = self.height().min(src.height());

        for y in 0..h {
            for x in 0..w {
                let src_val = src.get_pixel_unchecked(x, y) as u8;
                let pixel = self.get_pixel_unchecked(x, y);
                let (mut r, mut g, mut b, mut a) = color::extract_rgba(pixel);

                match comp {
                    RgbComponent::Red => r = src_val,
                    RgbComponent::Green => g = src_val,
                    RgbComponent::Blue => b = src_val,
                    RgbComponent::Alpha => a = src_val,
                }

                self.set_pixel_unchecked(x, y, color::compose_rgba(r, g, b, a));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use crate::pix::PixelDepth;

    #[test]
    fn test_get_rgb_component_red() {
        let pix = Pix::new(3, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 200, 100, 50).unwrap();
        pm.set_rgb(1, 0, 0, 255, 0).unwrap();
        pm.set_rgb(2, 0, 50, 50, 50).unwrap();
        let pix: Pix = pm.into();

        let red = pix.get_rgb_component(RgbComponent::Red).unwrap();
        assert_eq!(red.depth(), PixelDepth::Bit8);
        assert_eq!(red.get_pixel(0, 0), Some(200));
        assert_eq!(red.get_pixel(1, 0), Some(0));
        assert_eq!(red.get_pixel(2, 0), Some(50));
    }

    #[test]
    fn test_get_rgb_component_green() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 200, 50).unwrap();
        pm.set_rgb(1, 0, 0, 128, 255).unwrap();
        let pix: Pix = pm.into();

        let green = pix.get_rgb_component(RgbComponent::Green).unwrap();
        assert_eq!(green.get_pixel(0, 0), Some(200));
        assert_eq!(green.get_pixel(1, 0), Some(128));
    }

    #[test]
    fn test_get_rgb_component_blue() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 10, 20, 30).unwrap();
        let pix: Pix = pm.into();

        let blue = pix.get_rgb_component(RgbComponent::Blue).unwrap();
        assert_eq!(blue.get_pixel(0, 0), Some(30));
    }

    #[test]
    fn test_get_rgb_component_alpha() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        pm.set_pixel_unchecked(0, 0, color::compose_rgba(100, 150, 200, 128));
        let pix: Pix = pm.into();

        let alpha = pix.get_rgb_component(RgbComponent::Alpha).unwrap();
        assert_eq!(alpha.get_pixel(0, 0), Some(128));
    }

    #[test]
    fn test_get_rgb_component_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.get_rgb_component(RgbComponent::Red).is_err());
    }

    #[test]
    fn test_get_rgb_component_preserves_resolution() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_resolution(300, 300);
        let pix: Pix = pm.into();

        let red = pix.get_rgb_component(RgbComponent::Red).unwrap();
        assert_eq!(red.xres(), 300);
        assert_eq!(red.yres(), 300);
    }

    #[test]
    fn test_set_rgb_component() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 100, 100).unwrap();
        pm.set_rgb(1, 0, 100, 100, 100).unwrap();

        // Create an 8bpp source with new red values
        let src = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut src_m = src.try_into_mut().unwrap();
        src_m.set_pixel_unchecked(0, 0, 255);
        src_m.set_pixel_unchecked(1, 0, 0);
        let src: Pix = src_m.into();

        pm.set_rgb_component(&src, RgbComponent::Red).unwrap();
        let pix: Pix = pm.into();

        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (255, 100, 100));
        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (0, 100, 100));
    }

    #[test]
    fn test_set_rgb_component_alpha() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 100, 150, 200).unwrap();

        let src = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut src_m = src.try_into_mut().unwrap();
        src_m.set_pixel_unchecked(0, 0, 128);
        let src: Pix = src_m.into();

        pm.set_rgb_component(&src, RgbComponent::Alpha).unwrap();
        assert_eq!(pm.spp(), 4);
        let pix: Pix = pm.into();

        let (_, _, _, a) = color::extract_rgba(pix.get_pixel_unchecked(0, 0));
        assert_eq!(a, 128);
    }

    #[test]
    fn test_set_rgb_component_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let src = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pm.set_rgb_component(&src, RgbComponent::Red).is_err());
    }

    #[test]
    fn test_set_rgb_component_dimension_mismatch() {
        let pix = Pix::new(2, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..2 {
            for x in 0..2 {
                pm.set_rgb(x, y, 10, 20, 30).unwrap();
            }
        }

        // Smaller source: only overlapping region should be modified
        let src = Pix::new(1, 1, PixelDepth::Bit8).unwrap();
        let mut src_m = src.try_into_mut().unwrap();
        src_m.set_pixel_unchecked(0, 0, 255);
        let src: Pix = src_m.into();

        pm.set_rgb_component(&src, RgbComponent::Red).unwrap();
        let pix: Pix = pm.into();

        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(0, 0));
        assert_eq!((r, g, b), (255, 20, 30));
        // Other pixels unchanged
        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(1, 0));
        assert_eq!((r, g, b), (10, 20, 30));
        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(0, 1));
        assert_eq!((r, g, b), (10, 20, 30));
        let (r, g, b) = color::extract_rgb(pix.get_pixel_unchecked(1, 1));
        assert_eq!((r, g, b), (10, 20, 30));
    }

    #[test]
    fn test_create_rgb_image() {
        let r = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut rm = r.try_into_mut().unwrap();
        rm.set_pixel_unchecked(0, 0, 255);
        rm.set_pixel_unchecked(1, 0, 0);
        let r: Pix = rm.into();

        let g = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut gm = g.try_into_mut().unwrap();
        gm.set_pixel_unchecked(0, 0, 0);
        gm.set_pixel_unchecked(1, 0, 255);
        let g: Pix = gm.into();

        let b = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut bm = b.try_into_mut().unwrap();
        bm.set_pixel_unchecked(0, 0, 128);
        bm.set_pixel_unchecked(1, 0, 64);
        let b: Pix = bm.into();

        let rgb = Pix::create_rgb_image(&r, &g, &b).unwrap();
        assert_eq!(rgb.depth(), PixelDepth::Bit32);
        let (rv, gv, bv) = color::extract_rgb(rgb.get_pixel_unchecked(0, 0));
        assert_eq!((rv, gv, bv), (255, 0, 128));
        let (rv, gv, bv) = color::extract_rgb(rgb.get_pixel_unchecked(1, 0));
        assert_eq!((rv, gv, bv), (0, 255, 64));
    }

    #[test]
    fn test_create_rgb_image_dimension_mismatch() {
        let r = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let g = Pix::new(10, 20, PixelDepth::Bit8).unwrap();
        let b = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(Pix::create_rgb_image(&r, &g, &b).is_err());
    }

    #[test]
    fn test_create_rgb_image_invalid_depth() {
        let r = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let g = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let b = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(Pix::create_rgb_image(&r, &g, &b).is_err());
    }

    #[test]
    fn test_roundtrip_extract_compose() {
        // Extract R/G/B, then compose back, should match original
        let pix = Pix::new(3, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 10, 20, 30).unwrap();
        pm.set_rgb(1, 0, 100, 150, 200).unwrap();
        pm.set_rgb(2, 0, 255, 128, 0).unwrap();
        pm.set_rgb(0, 1, 0, 0, 0).unwrap();
        pm.set_rgb(1, 1, 255, 255, 255).unwrap();
        pm.set_rgb(2, 1, 50, 100, 150).unwrap();
        let pix: Pix = pm.into();

        let r = pix.get_rgb_component(RgbComponent::Red).unwrap();
        let g = pix.get_rgb_component(RgbComponent::Green).unwrap();
        let b = pix.get_rgb_component(RgbComponent::Blue).unwrap();

        let result = Pix::create_rgb_image(&r, &g, &b).unwrap();

        for y in 0..2 {
            for x in 0..3 {
                let (or, og, ob) = color::extract_rgb(pix.get_pixel_unchecked(x, y));
                let (rr, rg, rb) = color::extract_rgb(result.get_pixel_unchecked(x, y));
                assert_eq!((or, og, ob), (rr, rg, rb), "mismatch at ({}, {})", x, y);
            }
        }
    }

    // ================================================================
    // Phase 11.3: RGB/Alpha/Endian tests
    // ================================================================

    #[test]

    fn test_get_rgb_component_cmap() {
        use crate::PixColormap;
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // index 0: red
        cmap.add_rgb(0, 128, 0).unwrap(); // index 1: green
        cmap.add_rgb(0, 0, 200).unwrap(); // index 2: blue

        let pix = Pix::new(3, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_colormap(Some(cmap)).unwrap();
        pm.set_pixel(0, 0, 0).unwrap(); // red
        pm.set_pixel(1, 0, 1).unwrap(); // green
        pm.set_pixel(2, 0, 2).unwrap(); // blue
        let pix: Pix = pm.into();

        let red_comp = pix.get_rgb_component_cmap(RgbComponent::Red).unwrap();
        assert_eq!(red_comp.depth(), PixelDepth::Bit8);
        assert_eq!(red_comp.get_pixel(0, 0), Some(255));
        assert_eq!(red_comp.get_pixel(1, 0), Some(0));
        assert_eq!(red_comp.get_pixel(2, 0), Some(0));
    }

    #[test]

    fn test_get_rgb_line() {
        let pix = Pix::new(3, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 10, 20, 30).unwrap();
        pm.set_rgb(1, 0, 40, 50, 60).unwrap();
        pm.set_rgb(2, 0, 70, 80, 90).unwrap();
        let pix: Pix = pm.into();

        let (r, g, b) = pix.get_rgb_line(0).unwrap();
        assert_eq!(r, vec![10, 40, 70]);
        assert_eq!(g, vec![20, 50, 80]);
        assert_eq!(b, vec![30, 60, 90]);
    }

    #[test]

    fn test_copy_rgb_component() {
        let pix_d = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm_d = pix_d.try_into_mut().unwrap();
        pm_d.set_rgb(0, 0, 100, 100, 100).unwrap();
        pm_d.set_rgb(1, 0, 100, 100, 100).unwrap();

        let pix_s = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm_s = pix_s.try_into_mut().unwrap();
        pm_s.set_rgb(0, 0, 255, 0, 0).unwrap();
        pm_s.set_rgb(1, 0, 0, 255, 0).unwrap();
        let pix_s: Pix = pm_s.into();

        pm_d.copy_rgb_component(&pix_s, RgbComponent::Red).unwrap();
        let (r, _, _) = color::extract_rgb(pm_d.get_pixel_unchecked(0, 0));
        assert_eq!(r, 255);
        let (r, _, _) = color::extract_rgb(pm_d.get_pixel_unchecked(1, 0));
        assert_eq!(r, 0);
        // Green and blue should be unchanged
        let (_, g, b) = color::extract_rgb(pm_d.get_pixel_unchecked(0, 0));
        assert_eq!((g, b), (100, 100));
    }

    #[test]

    fn test_alpha_is_opaque_true() {
        let pix = Pix::new(3, 3, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        for y in 0..3 {
            for x in 0..3 {
                pm.set_pixel_unchecked(x, y, color::compose_rgba(100, 100, 100, 255));
            }
        }
        let pix: Pix = pm.into();
        assert!(pix.alpha_is_opaque().unwrap());
    }

    #[test]

    fn test_alpha_is_opaque_false() {
        let pix = Pix::new(3, 3, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_spp(4);
        for y in 0..3 {
            for x in 0..3 {
                pm.set_pixel_unchecked(x, y, color::compose_rgba(100, 100, 100, 255));
            }
        }
        // Set one pixel with alpha != 255
        pm.set_pixel_unchecked(1, 1, color::compose_rgba(100, 100, 100, 128));
        let pix: Pix = pm.into();
        assert!(!pix.alpha_is_opaque().unwrap());
    }

    #[test]

    fn test_infer_resolution() {
        // 3000x2000 image with 10 inch long side → 300 ppi
        let pix = Pix::new(3000, 2000, PixelDepth::Bit8).unwrap();
        let res = pix.infer_resolution(10.0).unwrap();
        assert_eq!(res, 300);
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_endian_byte_swap() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0xAABBCCDD);
        pm.endian_byte_swap();
        // ABCD -> DCBA: 0xAABBCCDD -> 0xDDCCBBAA
        assert_eq!(pm.get_pixel_unchecked(0, 0), 0xDDCCBBAA);
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_endian_byte_swap_new() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0x12345678);
        let pix: Pix = pm.into();

        let swapped = pix.endian_byte_swap_new();
        assert_eq!(swapped.get_pixel_unchecked(0, 0), 0x78563412);
        // Original should be unchanged
        assert_eq!(pix.get_pixel_unchecked(0, 0), 0x12345678);
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_endian_two_byte_swap() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0xAABBCCDD);
        pm.endian_two_byte_swap();
        // AABB|CCDD -> CCDD|AABB: 0xCCDDAABB
        assert_eq!(pm.get_pixel_unchecked(0, 0), 0xCCDDAABB);
    }

    #[test]

    fn test_get_raster_data_8bpp() {
        let pix = Pix::new(3, 2, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..2u32 {
            for x in 0..3u32 {
                pm.set_pixel(x, y, (y * 3 + x) as u32).unwrap();
            }
        }
        let pix: Pix = pm.into();
        let data = pix.get_raster_data().unwrap();
        // Each row is 4 bytes (wpl=1, 32 bits), 3 used + 1 pad
        assert_eq!(data.len(), 2 * 4);
        assert_eq!(data[0], 0);
        assert_eq!(data[1], 1);
        assert_eq!(data[2], 2);
    }

    #[test]

    fn test_get_raster_data_32bpp() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 10, 20, 30).unwrap();
        pm.set_rgb(1, 0, 40, 50, 60).unwrap();
        let pix: Pix = pm.into();
        let data = pix.get_raster_data().unwrap();
        // 32 bpp: 3 bytes per pixel (RGB only)
        assert_eq!(data.len(), 6);
        assert_eq!(data[0..3], [10, 20, 30]);
        assert_eq!(data[3..6], [40, 50, 60]);
    }

    #[test]

    fn test_set_cmap_pixel() {
        use crate::PixColormap;
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap(); // index 0
        cmap.add_rgb(0, 255, 0).unwrap(); // index 1

        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_colormap(Some(cmap)).unwrap();

        pm.set_cmap_pixel(0, 0, 255, 0, 0).unwrap(); // should set index 0
        pm.set_cmap_pixel(1, 0, 0, 255, 0).unwrap(); // should set index 1
        assert_eq!(pm.get_pixel(0, 0), Some(0));
        assert_eq!(pm.get_pixel(1, 0), Some(1));
    }
}
