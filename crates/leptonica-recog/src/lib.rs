//! leptonica-recog - OCR preprocessing and recognition support
//!
//! This crate provides OCR preprocessing and recognition functionality including:
//!
//! - **Skew detection and correction**: Detect and correct document rotation
//! - **Baseline detection**: Find text baselines for line segmentation
//! - **Page segmentation**: Separate text, images, and whitespace regions
//! - **Character recognition**: Template-based character recognition (recog)
//! - **JBIG2 classification**: Connected component clustering for compression (jbclass)
//! - **Dewarping**: Page curvature correction
//! - **Barcode**: 1D barcode detection and decoding

pub mod barcode;
pub mod baseline;
pub mod dewarp;
mod error;
pub mod jbclass;
pub mod pageseg;
pub mod recog;
pub mod skew;

pub use error::{RecogError, RecogResult};

// Re-export commonly used types
pub use baseline::{BaselineOptions, BaselineResult};
pub use pageseg::{PageSegOptions, SegmentationResult};
pub use skew::{SkewDetectOptions, SkewResult};

// Re-export recog types
pub use recog::{CharsetType, Rch, Rcha, Recog, TemplateUse};

// Re-export jbclass types
pub use jbclass::{JbClasser, JbComponent, JbData, JbMethod};

// Re-export dewarp types
pub use dewarp::{Dewarp, DewarpOptions, DewarpResult, dewarp_single_page};

// Re-export barcode types
pub use barcode::{
    BarcodeFormat, BarcodeOptions, BarcodeResult, DecodeMethod, decode_barcode, process_barcodes,
};

// Re-export core for convenience
pub use leptonica_core;
