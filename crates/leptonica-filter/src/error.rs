//! Error types for leptonica-filter
//!
//! Corresponds to error handling in C Leptonica's filter functions.
//! C version returns NULL or error codes; Rust uses typed errors.

use thiserror::Error;

/// Errors that can occur during filtering operations
#[derive(Debug, Error)]
pub enum FilterError {
    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),

    /// Invalid kernel
    #[error("invalid kernel: {0}")]
    InvalidKernel(String),

    /// Unsupported pixel depth for this operation
    #[error("unsupported depth: expected {expected}, got {actual}")]
    UnsupportedDepth {
        /// Expected depth description
        expected: &'static str,
        /// Actual depth in bits
        actual: u32,
    },

    /// Invalid parameters
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Result type for filter operations
pub type FilterResult<T> = Result<T, FilterError>;
