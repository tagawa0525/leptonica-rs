//! Mask operations for images
//!
//! Functions for setting, combining, and creating masks.
//! Corresponds to mask functions in C Leptonica's `pix3.c`.

use super::{Pix, PixMut};
use crate::error::Result;

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
    pub fn set_masked(&mut self, _mask: &Pix, _val: u32) -> Result<()> {
        todo!()
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
    pub fn combine_masked(&mut self, _src: &Pix, _mask: &Pix) -> Result<()> {
        todo!()
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
    pub fn paint_through_mask(&mut self, _mask: &Pix, _x: i32, _y: i32, _val: u32) -> Result<()> {
        todo!()
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
    pub fn make_mask_from_val(&self, _val: u32) -> Result<Pix> {
        todo!()
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
    pub fn make_mask_from_lut(&self, _lut: &[u8]) -> Result<Pix> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pix::PixelDepth;

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_set_masked_invalid_mask_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pm.set_masked(&mask, 0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_combine_masked_depth_mismatch() {
        let dst = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        let src = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(dm.combine_masked(&src, &mask).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_make_mask_from_val_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.make_mask_from_val(0).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_make_mask_from_lut_short_lut() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let lut = [0u8; 100]; // too short
        assert!(pix.make_mask_from_lut(&lut).is_err());
    }
}
