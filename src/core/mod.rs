//! Leptonica Core - Basic data structures for image processing
//!
//! This crate provides the fundamental data structures used throughout
//! the Leptonica image processing library:
//!
//! - [`Pix`] / [`PixMut`] - The main image container (immutable / mutable)
//! - [`Box`] / [`Boxa`] / [`Boxaa`] - Rectangle regions
//! - [`Pta`] / [`Ptaa`] - Point arrays
//! - [`Numa`] / [`Numaa`] - Numeric arrays
//! - [`FPix`] - Floating-point image
//! - [`Pixa`] / [`Pixaa`] - Arrays of images
//! - [`Sarray`] / [`Sarraya`] - String arrays
//! - [`PixColormap`] - Color palette for indexed images
//!
//! # See also
//!
//! C Leptonica: `pix.h`, `box.h`, `pts.h`, `environ.h` (struct definitions)

pub mod box_;
pub mod colormap;
pub mod error;
pub mod fpix;
pub mod numa;
pub mod pix;
pub mod pixa;
pub mod pta;
pub mod sarray;

pub use box_::{Box, Boxa, Boxaa, SizeRelation};
pub use colormap::convert::{ColormapArrays, ComponentsPerColor};
pub use colormap::query::{NonOpaqueInfo, RangeComponent, RangeValues};
pub use colormap::{PixColormap, RgbaQuad};
pub use error::{Error, Result};
pub use fpix::{DPix, FPix, NegativeHandling};
pub use numa::{
    CountRelativeToZero, HistogramResult, HistogramStats, InterpolationType, Numa, Numaa,
    SortOrder, ThresholdComparison, WindowedStats,
};
pub use pix::serial::SpixHeader;
pub use pix::statistics::{
    DiffDirection, ExtremeResult, ExtremeType, MaxValueResult, PixelMaxType, PixelStatType,
    RowColumnStats, StatsRequest,
};
pub use pix::{
    BlendMode, Color, ColorHistogram, CompareResult, CompareType, ContourOutput, GrayBlendType,
    ImageFormat, InColor, InitColor, MaskBlendType, Pix, PixMut, PixelDepth, PixelDiffResult,
    PixelOp, RopOp, ScanDirection, blend_with_gray_mask, correlation_binary,
};
pub use pixa::{Pixa, PixaSortType, Pixaa};
pub use pta::{Pta, Ptaa};
pub use sarray::{Sarray, Sarraya};

pub mod pixel;
