//! 1D Barcode detection and decoding
//!
//! This module provides functionality for locating and decoding 1D barcodes.
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
use leptonica_core::Pix;

/// Processes an image to detect and decode barcodes
pub fn process_barcodes(_pix: &Pix, _options: &BarcodeOptions) -> RecogResult<Vec<BarcodeResult>> {
    todo!("process_barcodes not yet implemented")
}

/// Decodes a bar width string directly
pub fn decode_barcode(bar_str: &str, format: BarcodeFormat) -> RecogResult<BarcodeResult> {
    if !is_format_supported(format) && format != BarcodeFormat::Any {
        return Err(RecogError::UnsupportedBarcodeFormat(
            format.name().to_string(),
        ));
    }
    dispatch_decoder(bar_str, format)
}
