//! leptonica-recog - OCR preprocessing and recognition support
//!
//! This crate provides OCR preprocessing and recognition functionality including:
//!
//! - **Skew detection and correction**: Detect and correct document rotation
//! - **Baseline detection**: Find text baselines for line segmentation
//! - **Page segmentation**: Separate text, images, and whitespace regions
//! - **Character recognition**: Template-based character recognition (recog)
//! - **JBIG2 classification**: Connected component clustering for compression (jbclass)
//!
//! # Quick Start
//!
//! ```no_run
//! use leptonica_recog::skew::{find_skew_and_deskew, SkewDetectOptions};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Load or create a document image
//! let pix = Pix::new(800, 600, PixelDepth::Bit1).unwrap();
//!
//! // Detect and correct skew
//! let (deskewed, result) = find_skew_and_deskew(&pix, &SkewDetectOptions::default()).unwrap();
//! println!("Detected skew: {} degrees (confidence: {})", result.angle, result.confidence);
//! ```
//!
//! # Character Recognition Example
//!
//! ```no_run
//! use leptonica_recog::recog::{Recog, create};
//!
//! // Create a recognizer with scaling
//! let mut recog = create(40, 40, 0, 150, 1).unwrap();
//!
//! // Train with labeled samples
//! // recog.train_labeled(&pix, "A").unwrap();
//!
//! // Finish training
//! // recog.finish_training().unwrap();
//!
//! // Identify characters
//! // let result = recog.identify_pix(&unknown_pix).unwrap();
//! ```
//!
//! # JBIG2 Classification Example
//!
//! ```no_run
//! use leptonica_recog::jbclass::{JbClasser, JbComponent, rank_haus_init};
//!
//! // Create a classifier
//! let mut classer = rank_haus_init(
//!     JbComponent::Characters,
//!     150, 150,  // max dimensions
//!     2,         // structuring element size
//!     0.97       // rank value
//! ).unwrap();
//!
//! // Add pages and classify
//! // classer.add_page(&pix).unwrap();
//! // let data = classer.get_data().unwrap();
//! ```
//!
//! # Modules
//!
//! - [`skew`]: Skew detection and correction
//! - [`baseline`]: Text baseline detection
//! - [`pageseg`]: Page segmentation into regions
//! - [`recog`]: Template-based character recognition
//! - [`jbclass`]: JBIG2 connected component classification
//! - [`dewarp`]: Page dewarping (curvature correction)
//! - [`barcode`]: 1D barcode detection and decoding

pub mod barcode;
pub mod baseline;
pub mod dewarp;
mod error;
pub mod jbclass;
pub mod pageseg;
pub mod recog;
pub mod skew;

pub use error::{RecogError, RecogResult};

// Re-export commonly used types from Phase 1
pub use baseline::{BaselineOptions, BaselineResult};
pub use pageseg::{PageSegOptions, SegmentationResult};
pub use skew::{SkewDetectOptions, SkewResult};

// Re-export commonly used types from Phase 2 - recog
pub use recog::{CharsetType, Rch, Rcha, Recog, TemplateUse};

// Re-export commonly used types from Phase 2 - jbclass
pub use jbclass::{JbClasser, JbComponent, JbData, JbMethod};

// Re-export commonly used types from dewarp
pub use dewarp::{Dewarp, DewarpOptions, DewarpResult, dewarp_single_page};

// Re-export commonly used types from barcode
pub use barcode::{
    BarcodeFormat, BarcodeOptions, BarcodeResult, DecodeMethod, decode_barcode, process_barcodes,
};

// Re-export core for convenience
pub use leptonica_core;
