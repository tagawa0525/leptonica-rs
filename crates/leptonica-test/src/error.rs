//! Error types for the test framework

use thiserror::Error;

/// Errors that can occur during regression testing
#[derive(Debug, Error)]
pub enum TestError {
    /// Failed to load a test image
    #[error("failed to load image '{path}': {message}")]
    ImageLoad { path: String, message: String },

    /// Failed to write an image
    #[error("failed to write image '{path}': {message}")]
    ImageWrite { path: String, message: String },

    /// Failed to create directory
    #[error("failed to create directory '{path}': {message}")]
    DirectoryCreate { path: String, message: String },

    /// Value comparison failed
    #[error(
        "value comparison failed at index {index}: expected {expected}, got {actual}, delta {delta}"
    )]
    ValueMismatch {
        index: usize,
        expected: f64,
        actual: f64,
        delta: f64,
    },

    /// Pix comparison failed
    #[error("pix comparison failed at index {index}")]
    PixMismatch { index: usize },

    /// File comparison failed
    #[error("file comparison failed at index {index}: {path}")]
    FileMismatch { index: usize, path: String },

    /// Golden file not found
    #[error("golden file not found: {path}")]
    GoldenNotFound { path: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for test operations
pub type TestResult<T> = Result<T, TestError>;
