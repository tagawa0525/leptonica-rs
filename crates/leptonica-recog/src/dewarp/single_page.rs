//! One-shot single-page dewarp helpers
//!
//! These thin wrappers combine Dewarpa initialisation and application into
//! two simple calls, mirroring the C `dewarpSinglePage*` family.

use crate::RecogResult;
use leptonica_core::Pix;

use super::dewarpa::Dewarpa;

/// Initialise a single-page dewarping pipeline from a source image.
///
/// Builds a complete [`Dewarpa`] container pre-loaded with a model for page 0.
/// The model is constructed by running the full page-model pipeline
/// (text-line detection → vertical disparity → optional horizontal disparity →
/// full-resolution population) on `pix`.
///
/// # Arguments
///
/// * `pix` - Source image (any depth)
///
/// # Errors
///
/// Returns an error if the page model cannot be built (e.g. too few text lines).
///
/// # Example
///
/// ```no_run
/// use leptonica_recog::dewarp::{dewarp_single_page_init, dewarp_single_page_run};
/// use leptonica_core::{Pix, PixelDepth};
///
/// # let pix = Pix::new(800, 600, PixelDepth::Bit1).unwrap();
/// let dewarpa = dewarp_single_page_init(&pix).unwrap();
/// let dewarped = dewarp_single_page_run(&dewarpa, &pix).unwrap();
/// ```
pub fn dewarp_single_page_init(_pix: &Pix) -> RecogResult<Dewarpa> {
    todo!("Phase 9: implement dewarp_single_page_init")
}

/// Apply a pre-built single-page dewarping pipeline to an image.
///
/// Uses the model for page 0 stored in `dewarpa` (as built by
/// [`dewarp_single_page_init`]) to dewarp `pix`.
///
/// # Arguments
///
/// * `dewarpa` - Container built by [`dewarp_single_page_init`]
/// * `pix` - Source image to dewarp
///
/// # Errors
///
/// Returns an error if page 0 has no model or if applying the disparity fails.
pub fn dewarp_single_page_run(_dewarpa: &Dewarpa, _pix: &Pix) -> RecogResult<Pix> {
    todo!("Phase 9: implement dewarp_single_page_run")
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::PixelDepth;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_dewarp_single_page_init_empty_image() {
        // An empty image has no text lines; init should return an error.
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let result = dewarp_single_page_init(&pix);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_dewarp_single_page_run_no_model() {
        // A Dewarpa with no models should return an error when run.
        let da = Dewarpa::new(1, 30, 1, 15, 5);
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let result = dewarp_single_page_run(&da, &pix);
        assert!(result.is_err());
    }
}
