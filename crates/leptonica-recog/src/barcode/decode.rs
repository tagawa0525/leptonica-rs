//! Barcode decoding dispatch

use super::types::{BarcodeFormat, BarcodeResult};
use crate::RecogResult;

/// Dispatch to the appropriate decoder based on format
pub fn dispatch_decoder(_barstr: &str, _format: BarcodeFormat) -> RecogResult<BarcodeResult> {
    todo!("dispatch_decoder not yet implemented")
}

/// Check if a format is supported
pub fn is_format_supported(format: BarcodeFormat) -> bool {
    format.is_supported()
}
