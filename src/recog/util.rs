//! Shared utilities used across recog submodules.

use crate::core::{Pix, PixelDepth};
use crate::recog::RecogResult;

/// Convert any-depth image to a 1bpp binary image using `threshold`.
///
/// - 1bpp input is returned as a clone (already binary, threshold ignored).
/// - 8bpp input is thresholded directly: pixels with `val < threshold` map to 1.
/// - Other depths are first converted to 8bpp via [`Pix::convert_to_8`] and
///   then thresholded.
///
/// Used by `pageseg` and `classapp` to normalize input before binary operations.
pub(super) fn ensure_binary_with_threshold(pix: &Pix, threshold: u32) -> RecogResult<Pix> {
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
