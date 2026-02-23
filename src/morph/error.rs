//! Error types for leptonica-morph

use thiserror::Error;

/// Errors that can occur during morphological operations
#[derive(Debug, Error)]
pub enum MorphError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Invalid structuring element
    #[error("invalid structuring element: {0}")]
    InvalidSel(String),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth { expected: &'static str, actual: u32 },

    /// Invalid parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    /// Invalid sequence format
    #[error("invalid sequence: {0}")]
    InvalidSequence(String),

    /// Unsupported operation in sequence
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Result type for morphological operations
pub type MorphResult<T> = Result<T, MorphError>;
