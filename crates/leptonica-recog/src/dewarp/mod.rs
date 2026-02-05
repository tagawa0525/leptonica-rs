//! Dewarping - Page distortion correction
//!
//! This module provides functionality to correct distortion in scanned
//! document images, particularly the curvature that occurs when scanning
//! bound books or documents that don't lie flat on the scanner.
//!
//! # Overview
//!
//! Dewarping works by:
//! 1. Detecting text lines in the image
//! 2. Modeling the curvature of each text line
//! 3. Building vertical disparity arrays that describe the distortion
//! 4. Optionally building horizontal disparity arrays for perspective correction
//! 5. Applying the disparity maps to dewarp the image
//!
//! # Example
//!
//! ```no_run
//! use leptonica_recog::dewarp::{dewarp_single_page, DewarpOptions};
//! use leptonica_core::Pix;
//!
//! // Load a scanned document image
//! // let pix = ... load image ...
//! # let pix = Pix::new(800, 600, leptonica_core::PixelDepth::Bit1).unwrap();
//!
//! // Dewarp with default options
//! let options = DewarpOptions::default();
//! match dewarp_single_page(&pix, &options) {
//!     Ok(result) => {
//!         println!("Dewarped successfully");
//!         println!("Vertical correction applied: {}", result.v_applied);
//!         println!("Horizontal correction applied: {}", result.h_applied);
//!         // Use result.pix
//!     }
//!     Err(e) => {
//!         println!("Dewarping failed: {}", e);
//!         // Use original image
//!     }
//! }
//! ```
//!
//! # Configuration
//!
//! Use [`DewarpOptions`] to configure the dewarping behavior:
//!
//! ```
//! use leptonica_recog::dewarp::DewarpOptions;
//!
//! let options = DewarpOptions::new()
//!     .with_sampling(20)        // Finer sampling for better accuracy
//!     .with_min_lines(10)       // Lower threshold for pages with less text
//!     .with_use_both(true)      // Apply both vertical and horizontal correction
//!     .with_gray_in(255);       // White background for outside pixels
//! ```
//!
//! # Limitations
//!
//! - The current implementation focuses on basic dewarping scenarios
//! - Works best with images containing clear horizontal text lines
//! - May not handle extreme curvature or perspective distortion well
//! - Performance may vary for very large images

mod apply;
mod model;
mod textline;
mod types;

pub use apply::{
    apply_disparity, apply_horizontal_disparity, apply_vertical_disparity,
    estimate_disparity_magnitude,
};
pub use model::{build_horizontal_disparity, build_vertical_disparity, populate_full_resolution};
pub use textline::{
    find_textline_centers, is_line_coverage_valid, remove_short_lines, sort_lines_by_y,
};
pub use types::{Dewarp, DewarpOptions, DewarpResult, TextLine};

use crate::{RecogError, RecogResult};
use leptonica_color::{
    AdaptiveThresholdOptions, adaptive_threshold, pix_convert_to_gray, threshold_to_binary,
};
use leptonica_core::{Pix, PixelDepth};

/// Dewarp a single page
///
/// This is the main entry point for dewarping. It handles the complete
/// pipeline from text line detection to applying the disparity model.
///
/// # Arguments
///
/// * `pix` - Input image (any depth)
/// * `options` - Dewarping options
///
/// # Returns
///
/// A [`DewarpResult`] containing the dewarped image and model information.
/// If dewarping fails (e.g., not enough text lines), returns an error.
///
/// # Example
///
/// ```no_run
/// use leptonica_recog::dewarp::{dewarp_single_page, DewarpOptions};
/// use leptonica_core::Pix;
///
/// # let pix = Pix::new(800, 600, leptonica_core::PixelDepth::Bit1).unwrap();
/// let options = DewarpOptions::default();
/// match dewarp_single_page(&pix, &options) {
///     Ok(result) => {
///         // Use result.pix for the dewarped image
///     }
///     Err(_) => {
///         // Dewarping failed, use original image
///     }
/// }
/// ```
pub fn dewarp_single_page(pix: &Pix, options: &DewarpOptions) -> RecogResult<DewarpResult> {
    // Get binary image for text line detection
    let pix_binary = get_binary_image(pix, options)?;

    // Find text line centers
    let lines = find_textline_centers(&pix_binary)?;

    if lines.len() < options.min_lines as usize {
        return Err(RecogError::NoContent(format!(
            "not enough text lines found: {} (need at least {})",
            lines.len(),
            options.min_lines
        )));
    }

    // Create dewarp model
    let mut dewarp = Dewarp::new(pix.width(), pix.height(), 0, options);

    // Build vertical disparity model
    build_vertical_disparity(&mut dewarp, &lines, options)?;

    if !dewarp.v_success {
        return Err(RecogError::NoContent(
            "failed to build vertical disparity model".to_string(),
        ));
    }

    // Optionally build horizontal disparity model
    let mut h_applied = false;
    if options.use_both && build_horizontal_disparity(&mut dewarp, &lines, options).is_ok() {
        h_applied = dewarp.h_valid;
    }

    // Populate full resolution disparity arrays
    populate_full_resolution(&mut dewarp)?;

    // Apply disparity to the original image
    let dewarped = apply_disparity(pix, &dewarp, options.gray_in)?;

    Ok(DewarpResult::new(dewarped, dewarp, true, h_applied))
}

/// Get a binary image for text line detection
fn get_binary_image(pix: &Pix, _options: &DewarpOptions) -> RecogResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => {
            // Already binary
            Ok(pix.deep_clone())
        }
        PixelDepth::Bit8 => {
            // Use adaptive thresholding for better results
            let opts = AdaptiveThresholdOptions::default();
            match adaptive_threshold(pix, &opts) {
                Ok(binary) => Ok(binary),
                Err(_) => {
                    // Fall back to simple threshold
                    threshold_to_binary(pix, 128).map_err(|e| e.into())
                }
            }
        }
        PixelDepth::Bit32 => {
            // Convert to grayscale first, then threshold
            let gray = pix_convert_to_gray(pix)?;
            let opts = AdaptiveThresholdOptions::default();
            match adaptive_threshold(&gray, &opts) {
                Ok(binary) => Ok(binary),
                Err(_) => threshold_to_binary(&gray, 128).map_err(|e| e.into()),
            }
        }
        _ => {
            // Convert to 8-bit then threshold
            let gray = pix_convert_to_gray(pix)?;
            threshold_to_binary(&gray, 128).map_err(|e| e.into())
        }
    }
}

/// Check if an image likely needs dewarping
///
/// This performs a quick analysis to determine if dewarping would be beneficial.
/// It does not build a full model.
///
/// # Arguments
///
/// * `pix` - Input image
///
/// # Returns
///
/// `true` if the image likely has significant curvature, `false` otherwise.
pub fn needs_dewarping(pix: &Pix) -> RecogResult<bool> {
    let options = DewarpOptions::default().with_min_lines(6);
    let pix_binary = get_binary_image(pix, &options)?;

    let lines = find_textline_centers(&pix_binary)?;
    if lines.len() < 6 {
        // Not enough lines to determine
        return Ok(false);
    }

    // Estimate disparity magnitude
    let magnitude = estimate_disparity_magnitude(&lines);

    // If max line deviation exceeds threshold, dewarping is likely needed
    // Threshold of 5 pixels is a reasonable starting point
    Ok(magnitude > 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dewarp_options_default() {
        let options = DewarpOptions::default();
        assert_eq!(options.sampling, 30);
        assert_eq!(options.min_lines, 15);
        assert!(options.use_both);
    }

    #[test]
    fn test_dewarp_single_page_empty_image() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let options = DewarpOptions::default();

        let result = dewarp_single_page(&pix, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_needs_dewarping_empty_image() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let result = needs_dewarping(&pix);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Empty image doesn't need dewarping
    }
}
