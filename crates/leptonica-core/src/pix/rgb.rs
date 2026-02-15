//! RGB component operations
//!
//! Functions for extracting, setting, and composing individual color
//! channels of 32 bpp images.
//!
//! # See also
//!
//! C Leptonica: `pix2.c` (`pixGetRGBComponent`, `pixSetRGBComponent`, etc.)

use super::{Pix, PixMut};
use crate::error::Result;

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
    pub fn get_rgb_component(&self, _comp: RgbComponent) -> Result<Pix> {
        todo!()
    }

    /// Create a 32 bpp RGB image from three 8 bpp component images.
    ///
    /// All three images must have the same dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixCreateRGBImage()` in `pix2.c`
    pub fn create_rgb_image(_pix_r: &Pix, _pix_g: &Pix, _pix_b: &Pix) -> Result<Pix> {
        todo!()
    }
}

impl PixMut {
    /// Set a single color component from an 8 bpp source image.
    ///
    /// The source image values replace the specified component channel
    /// in this 32 bpp image.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixSetRGBComponent()` in `pix2.c`
    pub fn set_rgb_component(&mut self, _src: &Pix, _comp: RgbComponent) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use crate::pix::PixelDepth;

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_get_rgb_component_blue() {
        let pix = Pix::new(1, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_rgb(0, 0, 10, 20, 30).unwrap();
        let pix: Pix = pm.into();

        let blue = pix.get_rgb_component(RgbComponent::Blue).unwrap();
        assert_eq!(blue.get_pixel(0, 0), Some(30));
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_get_rgb_component_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.get_rgb_component(RgbComponent::Red).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_set_rgb_component_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let src = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pm.set_rgb_component(&src, RgbComponent::Red).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn test_create_rgb_image_dimension_mismatch() {
        let r = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let g = Pix::new(10, 20, PixelDepth::Bit8).unwrap();
        let b = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(Pix::create_rgb_image(&r, &g, &b).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_create_rgb_image_invalid_depth() {
        let r = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let g = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let b = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(Pix::create_rgb_image(&r, &g, &b).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
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
}
