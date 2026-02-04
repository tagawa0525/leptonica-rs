//! Error types for leptonica-recog

use thiserror::Error;

/// Errors that can occur during recognition operations
#[derive(Debug, Error)]
pub enum RecogError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Transform library error
    #[error("transform error: {0}")]
    Transform(#[from] leptonica_transform::TransformError),

    /// Morphology library error
    #[error("morphology error: {0}")]
    Morph(#[from] leptonica_morph::MorphError),

    /// Region library error
    #[error("region error: {0}")]
    Region(#[from] leptonica_region::RegionError),

    /// Color library error
    #[error("color error: {0}")]
    Color(#[from] leptonica_color::ColorError),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },

    /// Invalid parameter provided
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// Skew detection failed
    #[error("skew detection failed: {0}")]
    SkewDetectionFailed(String),

    /// Segmentation operation failed
    #[error("segmentation error: {0}")]
    SegmentationError(String),

    /// Image is too small for the operation
    #[error(
        "image too small: minimum size is {min_width}x{min_height}, got {actual_width}x{actual_height}"
    )]
    ImageTooSmall {
        min_width: u32,
        min_height: u32,
        actual_width: u32,
        actual_height: u32,
    },

    /// No content found in image
    #[error("no content found: {0}")]
    NoContent(String),

    /// Training operation failed
    #[error("training error: {0}")]
    TrainingError(String),

    /// Identification operation failed
    #[error("identification error: {0}")]
    IdentificationError(String),

    /// Classification operation failed
    #[error("classification error: {0}")]
    ClassificationError(String),
}

/// Result type for recognition operations
pub type RecogResult<T> = Result<T, RecogError>;
