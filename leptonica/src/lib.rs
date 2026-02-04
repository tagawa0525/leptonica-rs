//! Leptonica - Image processing library for Rust
//!
//! This is a Rust port of the [Leptonica](http://www.leptonica.org/) image
//! processing library.
//!
//! # Overview
//!
//! Leptonica provides a comprehensive set of image processing operations
//! including:
//!
//! - Image I/O (PNG, JPEG, TIFF, BMP, GIF, WebP)
//! - Morphological operations (dilation, erosion, opening, closing)
//! - Geometric transforms (rotation, scaling, affine, projective)
//! - Filtering and enhancement
//! - Color processing and quantization
//! - Document image processing (deskew, dewarp, binarization)
//!
//! # Example
//!
//! ```
//! use leptonica::{Pix, PixelDepth};
//!
//! // Create a new 8-bit grayscale image
//! let pix = Pix::new(640, 480, PixelDepth::Bit8).unwrap();
//! assert_eq!(pix.width(), 640);
//! assert_eq!(pix.height(), 480);
//! ```

// Re-export core types
pub use leptonica_core::*;
