//! leptonica-recog - OCR preprocessing and recognition support
//!
//! This crate provides OCR preprocessing functionality including:
//!
//! - **Skew detection and correction**: Detect and correct document rotation
//! - **Baseline detection**: Find text baselines for line segmentation
//! - **Page segmentation**: Separate text, images, and whitespace regions
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
//! # Modules
//!
//! - [`skew`]: Skew detection and correction
//! - [`baseline`]: Text baseline detection
//! - [`pageseg`]: Page segmentation into regions

pub mod baseline;
mod error;
pub mod pageseg;
pub mod skew;

pub use error::{RecogError, RecogResult};

// Re-export commonly used types
pub use baseline::{BaselineOptions, BaselineResult};
pub use pageseg::{PageSegOptions, SegmentationResult};
pub use skew::{SkewDetectOptions, SkewResult};

// Re-export core for convenience
pub use leptonica_core;
