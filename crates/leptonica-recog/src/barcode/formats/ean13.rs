//! EAN-13 barcode decoder

use crate::RecogResult;
use crate::barcode::types::FormatVerification;

/// Verify EAN-13 format
pub fn verify_ean13(_barstr: &str) -> FormatVerification {
    todo!("verify_ean13 not yet implemented")
}

/// Decode EAN-13 barcode
pub fn decode_ean13(_barstr: &str, _first_digit: u8) -> RecogResult<String> {
    todo!("decode_ean13 not yet implemented")
}
