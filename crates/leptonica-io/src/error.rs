//! I/O error types
//!
//! Provides a unified error type for all image I/O operations.
//! Each format-specific module maps its underlying library errors
//! into `IoError` variants so that callers only need to handle
//! one error type.

use thiserror::Error;

/// Error type for image I/O operations.
///
/// Wraps format-specific decoding/encoding errors as well as
/// standard I/O and core-library errors.
///
/// # See also
/// C version: most I/O functions return 0/1 status codes.  The Rust API
/// uses `Result<T, IoError>` instead, giving callers richer context.
#[derive(Error, Debug)]
pub enum IoError {
    /// Standard I/O error (file not found, permission denied, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The image format is not supported or not enabled via features
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// The image data is structurally invalid
    #[error("invalid image data: {0}")]
    InvalidData(String),

    /// A format-specific decoder returned an error
    #[error("decode error: {0}")]
    DecodeError(String),

    /// A format-specific encoder returned an error
    #[error("encode error: {0}")]
    EncodeError(String),

    /// An error from the core library (e.g. pixel depth mismatch)
    #[error("core error: {0}")]
    Core(#[from] leptonica_core::Error),
}

/// Convenience alias for I/O results.
pub type IoResult<T> = Result<T, IoError>;
