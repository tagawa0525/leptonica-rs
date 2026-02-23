//! Error types for leptonica-color

use thiserror::Error;

/// Errors that can occur during color processing operations
#[derive(Debug, Error)]
pub enum ColorError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },

    /// Invalid color value
    #[error("invalid color value: {0}")]
    InvalidColorValue(String),

    /// Invalid parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    /// Quantization error
    #[error("quantization error: {0}")]
    QuantizationError(String),

    /// Empty image
    #[error("empty image: no pixels to process")]
    EmptyImage,
}

/// Result type for color operations
pub type ColorResult<T> = Result<T, ColorError>;
