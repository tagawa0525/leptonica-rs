//! 1D Barcode detection and decoding
//!
//! This module provides functionality for locating and decoding 1D barcodes in images.
//!
//! # Supported Formats
//!
//! - Code 2 of 5
//! - Interleaved 2 of 5
//! - Code 39
//! - Code 93
//! - Codabar
//! - UPC-A
//! - EAN-13
//!
//! # Example
//!
//! ```no_run
//! use leptonica_recog::barcode::{process_barcodes, BarcodeOptions, BarcodeFormat};
//! use leptonica_io::read_image;
//!
//! // Load an image containing a barcode
//! let pix = read_image("/path/to/barcode.png").unwrap();
//!
//! // Process with auto-detection
//! let results = process_barcodes(&pix, &BarcodeOptions::default()).unwrap();
//!
//! for result in results {
//!     println!("Format: {}, Data: {}", result.format.name(), result.data);
//! }
//! ```
//!
//! # Decoding Bar Width Strings
//!
//! If you have a pre-extracted bar width string:
//!
//! ```no_run
//! use leptonica_recog::barcode::{decode_barcode, BarcodeFormat};
//!
//! // Decode a bar width string with known format
//! let result = decode_barcode("111321121123...", BarcodeFormat::UpcA).unwrap();
//! println!("Decoded: {}", result.data);
//! ```

mod decode;
mod detect;
pub mod formats;
mod signal;
mod types;

pub use decode::{dispatch_decoder, is_format_supported};
pub use detect::{deskew_barcode, extract_barcodes, locate_barcodes};
pub use signal::{
    extract_crossings, quantize_crossings_by_width, quantize_crossings_by_window,
    widths_to_bar_string,
};
pub use types::{
    BarcodeFormat, BarcodeOptions, BarcodeResult, DecodeMethod, FormatVerification,
    SUPPORTED_FORMATS,
};

use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};

/// Processes an image to detect and decode barcodes
///
/// This is the top-level API that combines barcode detection, width extraction,
/// and decoding.
///
/// # Arguments
/// * `pix` - Input image (must be 8bpp grayscale)
/// * `options` - Processing options including format and method
///
/// # Returns
/// * Vector of `BarcodeResult` for each barcode found and decoded
///
/// # Example
///
/// ```no_run
/// use leptonica_recog::barcode::{process_barcodes, BarcodeOptions, BarcodeFormat};
/// use leptonica_io::read_image;
///
/// let pix = read_image("barcode.png").unwrap();
/// let results = process_barcodes(&pix, &BarcodeOptions::default()).unwrap();
///
/// for result in results {
///     println!("Decoded: {} (format: {})", result.data, result.format.name());
/// }
/// ```
pub fn process_barcodes(pix: &Pix, options: &BarcodeOptions) -> RecogResult<Vec<BarcodeResult>> {
    // Validate format if specified
    if options.format != BarcodeFormat::Any && !is_format_supported(options.format) {
        return Err(RecogError::UnsupportedBarcodeFormat(
            options.format.name().to_string(),
        ));
    }

    // Require 8bpp grayscale input
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RecogError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth() as u32,
        });
    }

    // Extract barcode regions
    let pixa = extract_barcodes(pix)?;
    let n = pixa.len();

    if n == 0 {
        return Err(RecogError::NoBarcodeFound);
    }

    let mut results = Vec::new();

    for i in 0..n {
        let barcode_pix = match pixa.get(i) {
            Some(p) => p,
            None => continue,
        };

        // Extract bar widths
        let crossings = match extract_crossings(barcode_pix, options.crossing_threshold) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Quantize to bar widths
        let widths = match options.method {
            DecodeMethod::UseWidths => match quantize_crossings_by_width(&crossings, 0.25) {
                Ok(w) => w,
                Err(_) => continue,
            },
            DecodeMethod::UseWindows => match quantize_crossings_by_window(&crossings, 2.0) {
                Ok((w, _)) => w,
                Err(_) => continue,
            },
        };

        // Convert to bar string
        let bar_string = widths_to_bar_string(&widths);

        // Decode
        match dispatch_decoder(&bar_string, options.format) {
            Ok(mut result) => {
                // Add bounding box if available
                if let Some(bbox) = pixa.get_box(i) {
                    result = result.with_bbox(*bbox);
                }
                results.push(result);
            }
            Err(_) => {
                // Failed to decode, skip this barcode
                continue;
            }
        }
    }

    if results.is_empty() {
        return Err(RecogError::BarcodeError(
            "no valid barcode data decoded".to_string(),
        ));
    }

    Ok(results)
}

/// Decodes a bar width string directly
///
/// This is useful when you have already extracted the bar width string
/// from a barcode image.
///
/// # Arguments
/// * `bar_str` - String of digits (1-4) representing bar widths
/// * `format` - The barcode format, or `BarcodeFormat::Any` for auto-detection
///
/// # Returns
/// * `BarcodeResult` containing the decoded data
///
/// # Example
///
/// ```no_run
/// use leptonica_recog::barcode::{decode_barcode, BarcodeFormat};
///
/// // Auto-detect format
/// let result = decode_barcode("111321121...", BarcodeFormat::Any).unwrap();
///
/// // Or specify format
/// let result = decode_barcode("111321121...", BarcodeFormat::Code39).unwrap();
/// ```
pub fn decode_barcode(bar_str: &str, format: BarcodeFormat) -> RecogResult<BarcodeResult> {
    dispatch_decoder(bar_str, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats() {
        assert!(is_format_supported(BarcodeFormat::UpcA));
        assert!(is_format_supported(BarcodeFormat::Ean13));
        assert!(is_format_supported(BarcodeFormat::Code39));
        assert!(is_format_supported(BarcodeFormat::Code93));
        assert!(is_format_supported(BarcodeFormat::Codabar));
        assert!(is_format_supported(BarcodeFormat::Code2of5));
        assert!(is_format_supported(BarcodeFormat::CodeI2of5));
        assert!(!is_format_supported(BarcodeFormat::Code128)); // Not implemented
    }

    #[test]
    fn test_barcode_options_builder() {
        let opts = BarcodeOptions::with_format(BarcodeFormat::UpcA)
            .method(DecodeMethod::UseWindows)
            .debug(true);

        assert_eq!(opts.format, BarcodeFormat::UpcA);
        assert_eq!(opts.method, DecodeMethod::UseWindows);
        assert!(opts.debug);
    }

    #[test]
    fn test_decode_empty_string() {
        let result = decode_barcode("", BarcodeFormat::Any);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_unsupported_format() {
        let result = decode_barcode("1234", BarcodeFormat::Code128);
        assert!(result.is_err());
    }
}
