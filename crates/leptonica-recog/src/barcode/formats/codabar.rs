//! Codabar barcode decoder

use crate::RecogResult;
use crate::barcode::types::FormatVerification;

/// Verify Codabar format
pub fn verify_codabar(_barstr: &str) -> FormatVerification {
    todo!("verify_codabar not yet implemented")
}

/// Decode Codabar barcode
pub fn decode_codabar(_barstr: &str) -> RecogResult<String> {
    todo!("decode_codabar not yet implemented")
}
