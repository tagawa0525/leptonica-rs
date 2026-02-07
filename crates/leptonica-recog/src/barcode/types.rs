//! Type definitions for barcode recognition
//!
//! This module contains the core data structures for barcode detection and decoding.

use leptonica_core::Box as PixBox;

/// Barcode format types
///
/// These identify both the barcode format and the decoding method to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarcodeFormat {
    /// Unknown format
    #[default]
    Unknown = 0,
    /// Try decoding with all known formats (auto-detect)
    Any = 1,
    /// Code 128 format
    Code128 = 2,
    /// EAN-8 format
    Ean8 = 3,
    /// EAN-13 format
    Ean13 = 4,
    /// Code 2 of 5 format
    Code2of5 = 5,
    /// Interleaved 2 of 5 format
    CodeI2of5 = 6,
    /// Code 39 format
    Code39 = 7,
    /// Code 93 format
    Code93 = 8,
    /// Codabar format
    Codabar = 9,
    /// UPC-A format
    UpcA = 10,
}

impl BarcodeFormat {
    /// Returns the name of this barcode format
    pub fn name(&self) -> &'static str {
        match self {
            BarcodeFormat::Unknown => "Unknown",
            BarcodeFormat::Any => "Any",
            BarcodeFormat::Code128 => "Code128",
            BarcodeFormat::Ean8 => "EAN-8",
            BarcodeFormat::Ean13 => "EAN-13",
            BarcodeFormat::Code2of5 => "Code2of5",
            BarcodeFormat::CodeI2of5 => "CodeI2of5",
            BarcodeFormat::Code39 => "Code39",
            BarcodeFormat::Code93 => "Code93",
            BarcodeFormat::Codabar => "Codabar",
            BarcodeFormat::UpcA => "UPC-A",
        }
    }

    /// Returns whether this format is currently supported for decoding
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            BarcodeFormat::Code2of5
                | BarcodeFormat::CodeI2of5
                | BarcodeFormat::Code93
                | BarcodeFormat::Code39
                | BarcodeFormat::Codabar
                | BarcodeFormat::UpcA
                | BarcodeFormat::Ean13
        )
    }
}

/// List of supported barcode formats (in detection order)
pub const SUPPORTED_FORMATS: &[BarcodeFormat] = &[
    BarcodeFormat::Code2of5,
    BarcodeFormat::CodeI2of5,
    BarcodeFormat::Code93,
    BarcodeFormat::Code39,
    BarcodeFormat::Codabar,
    BarcodeFormat::UpcA,
    BarcodeFormat::Ean13,
];

/// Method for extracting barcode widths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecodeMethod {
    /// Use histogram of barcode widths
    #[default]
    UseWidths = 1,
    /// Find best window for decoding transitions
    UseWindows = 2,
}

/// Result of barcode detection and decoding
#[derive(Debug, Clone)]
pub struct BarcodeResult {
    /// Decoded barcode data
    pub data: String,
    /// Detected barcode format
    pub format: BarcodeFormat,
    /// Bar width string (for debugging)
    pub bar_widths: Option<String>,
    /// Bounding box of the barcode in the original image
    pub bbox: Option<PixBox>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

impl BarcodeResult {
    /// Creates a new barcode result
    pub fn new(data: String, format: BarcodeFormat) -> Self {
        Self {
            data,
            format,
            bar_widths: None,
            bbox: None,
            confidence: 1.0,
        }
    }

    /// Sets the bar width string
    pub fn with_bar_widths(mut self, widths: String) -> Self {
        self.bar_widths = Some(widths);
        self
    }

    /// Sets the bounding box
    pub fn with_bbox(mut self, bbox: PixBox) -> Self {
        self.bbox = Some(bbox);
        self
    }

    /// Sets the confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Options for barcode processing
#[derive(Debug, Clone)]
pub struct BarcodeOptions {
    /// Target format (use Any for auto-detection)
    pub format: BarcodeFormat,
    /// Decoding method
    pub method: DecodeMethod,
    /// Enable debug output
    pub debug: bool,
    /// Edge detection threshold (typically ~20)
    pub edge_threshold: i32,
    /// Binarization threshold for crossing detection (typically ~120)
    pub crossing_threshold: f32,
}

impl Default for BarcodeOptions {
    fn default() -> Self {
        Self {
            format: BarcodeFormat::Any,
            method: DecodeMethod::UseWidths,
            debug: false,
            edge_threshold: 20,
            crossing_threshold: 120.0,
        }
    }
}

impl BarcodeOptions {
    /// Creates options for a specific format
    pub fn with_format(format: BarcodeFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Sets the decode method
    pub fn method(mut self, method: DecodeMethod) -> Self {
        self.method = method;
        self
    }

    /// Enables debug output
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

/// Verification result for barcode format
#[derive(Debug, Clone, Copy)]
pub struct FormatVerification {
    /// Whether the format is valid
    pub valid: bool,
    /// Whether the barcode is reversed
    pub reversed: bool,
}

impl FormatVerification {
    /// Creates a valid verification result
    pub fn valid(reversed: bool) -> Self {
        Self {
            valid: true,
            reversed,
        }
    }

    /// Creates an invalid verification result
    pub fn invalid() -> Self {
        Self {
            valid: false,
            reversed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barcode_format_name() {
        assert_eq!(BarcodeFormat::UpcA.name(), "UPC-A");
        assert_eq!(BarcodeFormat::Ean13.name(), "EAN-13");
        assert_eq!(BarcodeFormat::Code39.name(), "Code39");
    }

    #[test]
    fn test_barcode_format_supported() {
        assert!(BarcodeFormat::UpcA.is_supported());
        assert!(BarcodeFormat::Ean13.is_supported());
        assert!(BarcodeFormat::Code39.is_supported());
        assert!(!BarcodeFormat::Unknown.is_supported());
        assert!(!BarcodeFormat::Code128.is_supported()); // Not yet implemented
    }

    #[test]
    fn test_barcode_result() {
        let result = BarcodeResult::new("123456789012".to_string(), BarcodeFormat::UpcA)
            .with_confidence(0.95);
        assert_eq!(result.data, "123456789012");
        assert_eq!(result.format, BarcodeFormat::UpcA);
        assert_eq!(result.confidence, 0.95);
    }

    #[test]
    fn test_barcode_options_default() {
        let opts = BarcodeOptions::default();
        assert_eq!(opts.format, BarcodeFormat::Any);
        assert_eq!(opts.method, DecodeMethod::UseWidths);
        assert!(!opts.debug);
    }

    #[test]
    fn test_format_verification() {
        let valid = FormatVerification::valid(false);
        assert!(valid.valid);
        assert!(!valid.reversed);

        let invalid = FormatVerification::invalid();
        assert!(!invalid.valid);
    }
}
