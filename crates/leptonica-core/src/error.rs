//! Error types for leptonica-core
//!
//! Provides a unified error type for all operations in the core crate.
//! Each variant captures enough context for diagnostics without exposing
//! internal implementation details.
//!
//! # See also
//!
//! C Leptonica uses integer return codes and `L_WARNING` / `L_ERROR` macros.
//! This module replaces those with Rust's `Result<T, Error>` pattern.

use thiserror::Error;

/// Leptonica-rs error type
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid image dimensions
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimension { width: u32, height: u32 },

    /// Invalid pixel depth
    #[error("invalid pixel depth: {0} bpp")]
    InvalidDepth(u32),

    /// Colormap required but not present
    #[error("colormap required but not present")]
    ColormapRequired,

    /// Colormap not allowed for this depth
    #[error("colormap not allowed for depth {0} bpp")]
    ColormapNotAllowed(u32),

    /// Index out of bounds
    #[error("index out of bounds: {index} >= {len}")]
    IndexOutOfBounds { index: usize, len: usize },

    /// Incompatible image sizes
    #[error("incompatible image sizes: {0}x{1} vs {2}x{3}")]
    IncompatibleSizes(u32, u32, u32, u32),

    /// Incompatible pixel depths
    #[error("incompatible pixel depths: {0} bpp vs {1} bpp")]
    IncompatibleDepths(u32, u32),

    /// Image dimension mismatch
    #[error("dimension mismatch: expected {}x{}, got {}x{}", .expected.0, .expected.1, .actual.0, .actual.1)]
    DimensionMismatch {
        expected: (u32, u32),
        actual: (u32, u32),
    },

    /// Unsupported pixel depth for this operation
    #[error("unsupported pixel depth: {0} bpp")]
    UnsupportedDepth(u32),

    /// Invalid parameter value
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// Null pointer or empty input
    #[error("null or empty input: {0}")]
    NullInput(&'static str),

    /// Operation not supported
    #[error("operation not supported: {0}")]
    NotSupported(String),

    /// Memory allocation failed
    #[error("memory allocation failed")]
    AllocationFailed,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Unsupported image format
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Image decode error
    #[error("decode error: {0}")]
    DecodeError(String),

    /// Image encode error
    #[error("encode error: {0}")]
    EncodeError(String),
}

/// Result type alias for Leptonica operations
pub type Result<T> = std::result::Result<T, Error>;
