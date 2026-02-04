//! I/O error types

use thiserror::Error;

/// I/O error type
#[derive(Error, Debug)]
pub enum IoError {
    /// Standard I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Unsupported image format
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Invalid image data
    #[error("invalid image data: {0}")]
    InvalidData(String),

    /// Decode error
    #[error("decode error: {0}")]
    DecodeError(String),

    /// Encode error
    #[error("encode error: {0}")]
    EncodeError(String),

    /// Core library error
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),
}

/// Result type for I/O operations
pub type IoResult<T> = Result<T, IoError>;
