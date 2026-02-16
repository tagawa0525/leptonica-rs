//! Error types for leptonica-filter

use thiserror::Error;

/// Errors that can occur during filtering operations
#[derive(Debug, Error)]
pub enum FilterError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Morphology operation error
    #[error("morph error: {0}")]
    Morph(#[from] leptonica_morph::MorphError),

    /// Transform operation error
    #[error("transform error: {0}")]
    Transform(#[from] leptonica_transform::TransformError),

    /// Invalid kernel
    #[error("invalid kernel: {0}")]
    InvalidKernel(String),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },

    /// Invalid parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Result type for filter operations
pub type FilterResult<T> = Result<T, FilterError>;
