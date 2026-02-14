//! Error types for leptonica-transform

use thiserror::Error;

/// Errors that can occur during geometric transformations
#[derive(Debug, Error)]
pub enum TransformError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Invalid scale factor
    #[error("invalid scale factor: {0}")]
    InvalidScaleFactor(String),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: {0}")]
    UnsupportedDepth(String),

    /// Invalid transformation parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    /// Singular matrix (non-invertible)
    #[error("singular transformation matrix")]
    SingularMatrix,
}

/// Result type for transform operations
pub type TransformResult<T> = Result<T, TransformError>;
