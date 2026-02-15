//! Mask operations for images
//!
//! Functions for setting, combining, and creating masks.
//! Corresponds to mask functions in C Leptonica's `pix3.c`.

use super::{Pix, PixMut, PixelDepth};
use crate::error::{Error, Result};

impl PixMut {
    /// Set pixels to a value where a 1 bpp mask is ON.
    ///
    /// The mask is aligned to the upper-left corner. Only the
    /// overlapping region is processed.
    ///
    /// C equivalent: `pixSetMasked()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - 1 bpp mask image
    /// * `val` - Value to set (masked to valid range for depth)
    ///
    /// # Errors
    ///
    /// Returns an error if the mask is not 1 bpp or if this image
    /// has an unsupported depth.
    pub fn set_masked(&mut self, mask: &Pix, val: u32) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        let val = mask_val(val, self.depth());
        let w = self.width().min(mask.width());
        let h = self.height().min(mask.height());

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel_unchecked(x, y) != 0 {
                    self.set_pixel_unchecked(x, y, val);
                }
            }
        }
        Ok(())
    }

    /// Copy pixels from a source image where a 1 bpp mask is ON.
    ///
    /// The mask, source, and destination are aligned at the upper-left
    /// corner. Only the overlapping region is processed.
    ///
    /// C equivalent: `pixCombineMasked()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `src` - Source image (must have same depth as self)
    /// * `mask` - 1 bpp mask image
    ///
    /// # Errors
    ///
    /// Returns an error if depths don't match, if the mask is not
    /// 1 bpp, or if the depth is unsupported.
    pub fn combine_masked(&mut self, src: &Pix, mask: &Pix) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        if self.depth() != src.depth() {
            return Err(Error::InvalidParameter(
                "source and destination depths must match".into(),
            ));
        }
        let w = self.width().min(src.width()).min(mask.width());
        let h = self.height().min(src.height()).min(mask.height());

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel_unchecked(x, y) != 0 {
                    self.set_pixel_unchecked(x, y, src.get_pixel_unchecked(x, y));
                }
            }
        }
        Ok(())
    }

    /// Paint a value through a mask at a specified offset.
    ///
    /// Like `set_masked` but the mask is placed at position `(x, y)`
    /// relative to the destination image.
    ///
    /// C equivalent: `pixPaintThroughMask()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - 1 bpp mask image
    /// * `x` - Horizontal offset for mask placement
    /// * `y` - Vertical offset for mask placement
    /// * `val` - Value to paint (masked to valid range for depth)
    ///
    /// # Errors
    ///
    /// Returns an error if the mask is not 1 bpp or if this image
    /// has an unsupported depth.
    pub fn paint_through_mask(&mut self, mask: &Pix, x: i32, y: i32, val: u32) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        let val = mask_val(val, self.depth());
        let dw = self.width() as i32;
        let dh = self.height() as i32;
        let mw = mask.width() as i32;
        let mh = mask.height() as i32;

        for my in 0..mh {
            let dy = y + my;
            if dy < 0 || dy >= dh {
                continue;
            }
            for mx in 0..mw {
                let dx = x + mx;
                if dx < 0 || dx >= dw {
                    continue;
                }
                if mask.get_pixel_unchecked(mx as u32, my as u32) != 0 {
                    self.set_pixel_unchecked(dx as u32, dy as u32, val);
                }
            }
        }
        Ok(())
    }
}

impl Pix {
    /// Create a 1 bpp mask where pixels equal a given value.
    ///
    /// C equivalent: `pixMakeMaskFromVal()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `val` - Pixel value to match
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 2, 4, or 8 bpp.
    pub fn make_mask_from_val(&self, val: u32) -> Result<Pix> {
        let d = self.depth();
        if d != PixelDepth::Bit2 && d != PixelDepth::Bit4 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }

        let w = self.width();
        let h = self.height();
        let mask = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut mm = mask.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) == val {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        Ok(mm.into())
    }

    /// Create a 1 bpp mask using a lookup table.
    ///
    /// The LUT maps pixel values to mask values (0 or 1).
    ///
    /// C equivalent: `pixMakeMaskFromLUT()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `lut` - Lookup table of 256 entries; 1 means set mask bit
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 2, 4, or 8 bpp,
    /// or if the LUT has fewer than 256 entries.
    pub fn make_mask_from_lut(&self, lut: &[u8]) -> Result<Pix> {
        let d = self.depth();
        if d != PixelDepth::Bit2 && d != PixelDepth::Bit4 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if lut.len() < 256 {
            return Err(Error::InvalidParameter(
                "LUT must have at least 256 entries".into(),
            ));
        }

        let w = self.width();
        let h = self.height();
        let mask = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut mm = mask.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let v = self.get_pixel_unchecked(x, y) as usize;
                if lut[v] != 0 {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        Ok(mm.into())
    }
}

/// Mask a pixel value to the valid range for a given depth.
fn mask_val(val: u32, depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => val & 1,
        PixelDepth::Bit2 => val & 3,
        PixelDepth::Bit4 => val & 0xf,
        PixelDepth::Bit8 => val & 0xff,
        PixelDepth::Bit16 => val & 0xffff,
        PixelDepth::Bit32 => val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pix::PixelDepth;

    #[test]

    fn test_set_masked_8bpp() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Fill with 100
        for y in 0..3 {
            for x in 0..4 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }

        // Mask: ON at (0,0) and (2,1)
        let mask = Pix::new(4, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(2, 1, 1);
        let mask: Pix = mm.into();

        pm.set_masked(&mask, 255).unwrap();
        let pix: Pix = pm.into();

        assert_eq!(pix.get_pixel(0, 0), Some(255));
        assert_eq!(pix.get_pixel(2, 1), Some(255));
        assert_eq!(pix.get_pixel(1, 0), Some(100)); // unchanged
        assert_eq!(pix.get_pixel(3, 2), Some(100)); // unchanged
    }

    #[test]

    fn test_set_masked_32bpp() {
        let pix = Pix::new(3, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(3, 2, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(1, 0, 1);
        let mask: Pix = mm.into();

        pm.set_masked(&mask, 0xff000000).unwrap();
        let pix: Pix = pm.into();

        assert_eq!(pix.get_pixel(1, 0), Some(0xff000000));
        assert_eq!(pix.get_pixel(0, 0), Some(0)); // unchanged
    }

    #[test]

    fn test_set_masked_invalid_mask_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pm.set_masked(&mask, 0).is_err());
    }

    #[test]

    fn test_combine_masked_8bpp() {
        let dst = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        for y in 0..3 {
            for x in 0..4 {
                dm.set_pixel_unchecked(x, y, 50);
            }
        }

        let src = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut sm = src.try_into_mut().unwrap();
        for y in 0..3 {
            for x in 0..4 {
                sm.set_pixel_unchecked(x, y, 200);
            }
        }
        let src: Pix = sm.into();

        let mask = Pix::new(4, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(3, 2, 1);
        let mask: Pix = mm.into();

        dm.combine_masked(&src, &mask).unwrap();
        let dst: Pix = dm.into();

        assert_eq!(dst.get_pixel(0, 0), Some(200)); // from src
        assert_eq!(dst.get_pixel(3, 2), Some(200)); // from src
        assert_eq!(dst.get_pixel(1, 0), Some(50)); // unchanged
    }

    #[test]

    fn test_combine_masked_depth_mismatch() {
        let dst = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        let src = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(dm.combine_masked(&src, &mask).is_err());
    }

    #[test]

    fn test_paint_through_mask() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        // Small 3x3 mask with center ON
        let mask = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(1, 1, 1);
        let mask: Pix = mm.into();

        pm.paint_through_mask(&mask, 4, 4, 200).unwrap();
        let pix: Pix = pm.into();

        // Mask center (1,1) placed at (4,4) → pixel (5,5)
        assert_eq!(pix.get_pixel(5, 5), Some(200));
        assert_eq!(pix.get_pixel(4, 4), Some(0)); // not masked
    }

    #[test]

    fn test_paint_through_mask_negative_offset() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(3, 3, 1); // bottom-right of mask
        let mask: Pix = mm.into();

        pm.paint_through_mask(&mask, -2, -2, 128).unwrap();
        let pix: Pix = pm.into();

        // Mask (3,3) at offset (-2,-2) → pixel (1,1)
        assert_eq!(pix.get_pixel(1, 1), Some(128));
        assert_eq!(pix.get_pixel(0, 0), Some(0));
    }

    #[test]

    fn test_make_mask_from_val() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 42);
        pm.set_pixel_unchecked(2, 1, 42);
        pm.set_pixel_unchecked(3, 2, 42);
        pm.set_pixel_unchecked(1, 0, 100);
        let pix: Pix = pm.into();

        let mask = pix.make_mask_from_val(42).unwrap();
        assert_eq!(mask.depth(), PixelDepth::Bit1);
        assert_eq!(mask.width(), 4);
        assert_eq!(mask.height(), 3);
        assert_eq!(mask.get_pixel(0, 0), Some(1)); // matches
        assert_eq!(mask.get_pixel(2, 1), Some(1)); // matches
        assert_eq!(mask.get_pixel(3, 2), Some(1)); // matches
        assert_eq!(mask.get_pixel(1, 0), Some(0)); // doesn't match
        assert_eq!(mask.get_pixel(1, 1), Some(0)); // zero pixel
    }

    #[test]

    fn test_make_mask_from_val_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.make_mask_from_val(0).is_err());
    }

    #[test]

    fn test_make_mask_from_lut() {
        let pix = Pix::new(4, 2, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 10);
        pm.set_pixel_unchecked(1, 0, 20);
        pm.set_pixel_unchecked(2, 0, 30);
        pm.set_pixel_unchecked(3, 0, 10);
        pm.set_pixel_unchecked(0, 1, 20);
        let pix: Pix = pm.into();

        // LUT: mask values 10 and 30
        let mut lut = [0u8; 256];
        lut[10] = 1;
        lut[30] = 1;

        let mask = pix.make_mask_from_lut(&lut).unwrap();
        assert_eq!(mask.get_pixel(0, 0), Some(1)); // val=10 → 1
        assert_eq!(mask.get_pixel(1, 0), Some(0)); // val=20 → 0
        assert_eq!(mask.get_pixel(2, 0), Some(1)); // val=30 → 1
        assert_eq!(mask.get_pixel(3, 0), Some(1)); // val=10 → 1
        assert_eq!(mask.get_pixel(0, 1), Some(0)); // val=20 → 0
    }

    #[test]

    fn test_make_mask_from_lut_short_lut() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let lut = [0u8; 100]; // too short
        assert!(pix.make_mask_from_lut(&lut).is_err());
    }
}
