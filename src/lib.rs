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

// Internal modules
pub mod color;
pub mod core;
pub mod filter;
pub mod io;
pub mod morph;
pub mod recog;
pub mod region;
pub mod transform;

// Re-export core types at root level (maintaining public API)
pub use core::{
    BlendMode, Bmf, Box, BoxField, BoxSortType, Boxa, Boxaa, Color, ColorHistogram,
    ColormapArrays, CompareResult, CompareType, ComponentsPerColor, ContourOutput, CornerLocation,
    CountRelativeToZero, DPix, DiffDirection, Error, ExtremeResult, ExtremeType, FPix, FPixa,
    GrayBlendType, HistogramResult, HistogramStats, ImageFormat, InColor, InitColor,
    InterpolationType, MaskBlendType, MaxValueResult, NegativeHandling, NonOpaqueInfo, Numa, Numaa,
    Pix, PixAcc, PixColormap, PixComp, PixMut, PixTiling, Pixa, PixaComp, PixaSortType, Pixaa,
    PixelDepth, PixelDiffResult, PixelMaxType, PixelOp, PixelStatType, Pta, Ptaa, RangeComponent,
    RangeValues, Result, RgbaQuad, RopOp, RowColumnStats, Sarray, Sarraya, ScanDirection,
    SizeRelation, SortOrder, SpixHeader, StatsRequest, TextLocation, ThresholdComparison,
    TransformOrder, WindowedStats, blend_with_gray_mask, bmf_get_line_strings,
    bmf_get_string_width, bmf_get_word_widths, correlation_binary, decode_ascii85, decode_base64,
    encode_base64, make_mosaic_strips,
};
