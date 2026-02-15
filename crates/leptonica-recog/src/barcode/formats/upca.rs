//! UPC-A barcode decoder

use crate::RecogResult;
use crate::barcode::types::FormatVerification;

/// Verify UPC-A format
pub fn verify_upca(_barstr: &str) -> FormatVerification {
    todo!("verify_upca not yet implemented")
}

/// Decode UPC-A barcode
pub fn decode_upca(_barstr: &str) -> RecogResult<String> {
    todo!("decode_upca not yet implemented")
}
