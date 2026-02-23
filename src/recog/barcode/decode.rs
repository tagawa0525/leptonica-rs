//! Barcode decoding dispatcher
//!
//! This module provides the main dispatch logic for decoding barcodes
//! based on their format.

use crate::barcode::formats::{
    decode_codabar, decode_code2of5, decode_code39, decode_code93, decode_codei2of5, decode_ean13,
    decode_upca, verify_codabar, verify_code2of5, verify_code39, verify_code93, verify_codei2of5,
    verify_ean13, verify_upca,
};
use crate::barcode::types::{BarcodeFormat, BarcodeResult, SUPPORTED_FORMATS};
use crate::{RecogError, RecogResult};

/// Decodes a barcode from a bar width string
///
/// # Arguments
/// * `barstr` - String of bar widths (digits 1-4 representing relative widths)
/// * `format` - The barcode format to use, or `BarcodeFormat::Any` for auto-detection
///
/// # Returns
/// * `BarcodeResult` containing the decoded data and detected format
pub fn dispatch_decoder(barstr: &str, format: BarcodeFormat) -> RecogResult<BarcodeResult> {
    if barstr.is_empty() {
        return Err(RecogError::BarcodeError("barstr not defined".to_string()));
    }

    let format = if format == BarcodeFormat::Any {
        find_format(barstr)?
    } else {
        format
    };

    let data = match format {
        BarcodeFormat::Code2of5 => decode_code2of5(barstr)?,
        BarcodeFormat::CodeI2of5 => decode_codei2of5(barstr)?,
        BarcodeFormat::Code93 => decode_code93(barstr)?,
        BarcodeFormat::Code39 => decode_code39(barstr)?,
        BarcodeFormat::Codabar => decode_codabar(barstr)?,
        BarcodeFormat::UpcA => decode_upca(barstr)?,
        BarcodeFormat::Ean13 => decode_ean13(barstr, 0)?,
        _ => {
            return Err(RecogError::UnsupportedBarcodeFormat(
                format.name().to_string(),
            ));
        }
    };

    Ok(BarcodeResult::new(data, format).with_bar_widths(barstr.to_string()))
}

/// Automatically determines the barcode format from the bar width string
///
/// # Arguments
/// * `barstr` - String of bar widths
///
/// # Returns
/// * The detected format, or error if no format matches
fn find_format(barstr: &str) -> RecogResult<BarcodeFormat> {
    for &format in SUPPORTED_FORMATS {
        if verify_format(barstr, format) {
            return Ok(format);
        }
    }
    Err(RecogError::BarcodeError(
        "could not determine barcode format".to_string(),
    ))
}

/// Verifies if a bar string matches a specific format
///
/// # Arguments
/// * `barstr` - String of bar widths
/// * `format` - The format to verify against
///
/// # Returns
/// * `true` if the format is valid
fn verify_format(barstr: &str, format: BarcodeFormat) -> bool {
    match format {
        BarcodeFormat::Code2of5 => verify_code2of5(barstr).valid,
        BarcodeFormat::CodeI2of5 => verify_codei2of5(barstr).valid,
        BarcodeFormat::Code93 => verify_code93(barstr).valid,
        BarcodeFormat::Code39 => verify_code39(barstr).valid,
        BarcodeFormat::Codabar => verify_codabar(barstr).valid,
        BarcodeFormat::UpcA => verify_upca(barstr).valid,
        BarcodeFormat::Ean13 => verify_ean13(barstr).valid,
        _ => false,
    }
}

/// Checks if a barcode format is supported
///
/// # Arguments
/// * `format` - The format to check
///
/// # Returns
/// * `true` if the format is supported for decoding
pub fn is_format_supported(format: BarcodeFormat) -> bool {
    format.is_supported()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_format_supported() {
        assert!(is_format_supported(BarcodeFormat::UpcA));
        assert!(is_format_supported(BarcodeFormat::Ean13));
        assert!(is_format_supported(BarcodeFormat::Code39));
        assert!(!is_format_supported(BarcodeFormat::Unknown));
        assert!(!is_format_supported(BarcodeFormat::Code128));
    }

    #[test]
    fn test_dispatch_empty_string() {
        let result = dispatch_decoder("", BarcodeFormat::Any);
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatch_unsupported_format() {
        let result = dispatch_decoder("111222333", BarcodeFormat::Code128);
        assert!(result.is_err());
    }
}
