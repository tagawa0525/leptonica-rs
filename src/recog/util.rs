//! Shared utilities used across recog submodules.

use crate::core::pix::RemoveColormapTarget;
use crate::core::{Pix, PixelDepth};
use crate::recog::RecogResult;

/// Convert any-depth image to a 1bpp binary image using `threshold`.
///
/// - 1bpp input is returned as a clone (already binary, threshold ignored).
/// - Colormapped non-1bpp input has its colormap decoded to grayscale first;
///   without this step we would threshold raw palette indices and produce
///   nonsense for any palette that isn't ordered by intensity.
/// - 8bpp input is thresholded directly: pixels with `val < threshold` map to 1.
/// - Other depths are first converted to 8bpp via [`Pix::convert_to_8`] and
///   then thresholded.
///
/// Used by `pageseg` and `classapp` to normalize input before binary operations.
pub(super) fn ensure_binary_with_threshold(pix: &Pix, threshold: u32) -> RecogResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 && pix.colormap().is_some() {
        let gray = pix.remove_colormap(RemoveColormapTarget::ToGrayscale)?;
        return ensure_binary_with_threshold(&gray, threshold);
    }

    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.deep_clone()),
        PixelDepth::Bit8 => {
            let w = pix.width();
            let h = pix.height();
            let binary = Pix::new(w, h, PixelDepth::Bit1)?;
            let mut binary_mut = binary.try_into_mut().unwrap();
            for y in 0..h {
                for x in 0..w {
                    let val = pix.get_pixel_unchecked(x, y);
                    let bit = if val < threshold { 1 } else { 0 };
                    binary_mut.set_pixel_unchecked(x, y, bit);
                }
            }
            Ok(binary_mut.into())
        }
        _ => {
            let gray = pix.convert_to_8()?;
            ensure_binary_with_threshold(&gray, threshold)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PixColormap;
    use crate::core::pixel;

    #[test]
    fn bit1_input_is_passed_through() {
        let pix = Pix::new(2, 2, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(1, 1, 1);
        let pix: Pix = pm.into();

        let out = ensure_binary_with_threshold(&pix, 130).expect("pass through 1bpp");
        assert_eq!(out.depth(), PixelDepth::Bit1);
        assert_eq!(out.get_pixel_unchecked(0, 0), 1);
        assert_eq!(out.get_pixel_unchecked(1, 0), 0);
        assert_eq!(out.get_pixel_unchecked(0, 1), 0);
        assert_eq!(out.get_pixel_unchecked(1, 1), 1);
    }

    #[test]
    fn bit8_thresholds_dark_pixels_to_one() {
        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 50); // below threshold -> 1
        pm.set_pixel_unchecked(1, 0, 200); // at/above threshold -> 0
        let pix: Pix = pm.into();

        let out = ensure_binary_with_threshold(&pix, 130).expect("8bpp threshold");
        assert_eq!(out.depth(), PixelDepth::Bit1);
        assert_eq!(out.get_pixel_unchecked(0, 0), 1);
        assert_eq!(out.get_pixel_unchecked(1, 0), 0);
    }

    #[test]
    fn bit32_converts_to_luminance_then_thresholds() {
        let pix = Pix::new(2, 1, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, pixel::compose_rgba(0, 0, 0, 255)); // black -> dark
        pm.set_pixel_unchecked(1, 0, pixel::compose_rgba(255, 255, 255, 255)); // white -> bright
        let pix: Pix = pm.into();

        let out = ensure_binary_with_threshold(&pix, 130).expect("32bpp threshold");
        assert_eq!(out.depth(), PixelDepth::Bit1);
        assert_eq!(out.get_pixel_unchecked(0, 0), 1);
        assert_eq!(out.get_pixel_unchecked(1, 0), 0);
    }

    #[test]
    fn colormapped_8bpp_decodes_palette_before_threshold() {
        // Palette intentionally inverts the natural index→intensity ordering:
        // index 0 -> white (bright), index 1 -> black (dark). Without
        // colormap decoding the helper would treat index 0 as "darker" than
        // index 1 and produce inverted output.
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 255, 255, 255).unwrap();
        cmap.add_rgba(0, 0, 0, 255).unwrap();

        let pix = Pix::new(2, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 0); // palette[0] = white
        pm.set_pixel_unchecked(1, 0, 1); // palette[1] = black
        pm.set_colormap(Some(cmap)).unwrap();
        let pix: Pix = pm.into();
        assert!(pix.colormap().is_some());

        let out = ensure_binary_with_threshold(&pix, 130).expect("colormap decode");
        assert_eq!(out.depth(), PixelDepth::Bit1);
        assert_eq!(
            out.get_pixel_unchecked(0, 0),
            0,
            "white palette entry must threshold to 0"
        );
        assert_eq!(
            out.get_pixel_unchecked(1, 0),
            1,
            "black palette entry must threshold to 1"
        );
    }
}
