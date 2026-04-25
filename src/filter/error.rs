//! Error types for leptonica-filter

use thiserror::Error;

/// Errors that can occur during filtering operations
#[derive(Debug, Error)]
pub enum FilterError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] crate::core::Error),

    /// Morphology operation error
    #[error("morph error: {0}")]
    Morph(#[from] crate::morph::MorphError),

    /// Color operation error (filter helpers occasionally call into the
    /// color crate, e.g. `threshold_to_binary`).
    #[error("color error: {0}")]
    Color(#[from] crate::color::ColorError),

    /// Transform operation error
    #[error("transform error: {0}")]
    Transform(#[from] crate::transform::TransformError),

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
