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

// Re-export core types (primary data structures used everywhere)
pub use leptonica_core::*;

// Re-export domain crates as modules to avoid name conflicts
pub use leptonica_color as color;
pub use leptonica_filter as filter;
pub use leptonica_io as io;
pub use leptonica_morph as morph;
pub use leptonica_recog as recog;
pub use leptonica_region as region;
pub use leptonica_transform as transform;
