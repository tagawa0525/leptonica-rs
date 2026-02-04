//! Error types for leptonica-region

use thiserror::Error;

/// Errors that can occur during region processing operations
#[derive(Debug, Error)]
pub enum RegionError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },

    /// Invalid seed position
    #[error("invalid seed position: ({x}, {y})")]
    InvalidSeed { x: u32, y: u32 },

    /// Invalid parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    /// Segmentation error
    #[error("segmentation error: {0}")]
    SegmentationError(String),

    /// Empty image
    #[error("empty image: no pixels to process")]
    EmptyImage,
}

/// Result type for region operations
pub type RegionResult<T> = Result<T, RegionError>;
