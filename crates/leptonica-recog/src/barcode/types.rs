//! Type definitions for barcode recognition

use leptonica_core::Box as PixBox;

/// Barcode format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarcodeFormat {
    /// Unknown format
    #[default]
    Unknown = 0,
    /// Try all known formats (auto-detect)
    Any = 1,
    /// Code 128
    Code128 = 2,
    /// EAN-8
    Ean8 = 3,
    /// EAN-13
    Ean13 = 4,
    /// Code 2 of 5
    Code2of5 = 5,
    /// Interleaved 2 of 5
    CodeI2of5 = 6,
    /// Code 39
    Code39 = 7,
    /// Code 93
    Code93 = 8,
    /// Codabar
    Codabar = 9,
    /// UPC-A
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
    /// Edge detection threshold
    pub edge_threshold: i32,
    /// Binarization threshold for crossing detection
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
