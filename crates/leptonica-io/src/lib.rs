// Stub: minimal I/O definitions for workspace compilation.
// These will be replaced with full implementations in later phases.

pub use leptonica_core::ImageFormat;

use leptonica_core::Pix;
use std::path::Path;

/// I/O error type (stub).
#[derive(Debug)]
pub struct IoError(String);

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for IoError {}

/// Read an image from a file path (stub).
pub fn read_image<P: AsRef<Path>>(_path: P) -> Result<Pix, IoError> {
    Err(IoError("not implemented".to_string()))
}

/// Write an image to a file path (stub).
pub fn write_image<P: AsRef<Path>>(
    _pix: &Pix,
    _path: P,
    _format: ImageFormat,
) -> Result<(), IoError> {
    Err(IoError("not implemented".to_string()))
}
