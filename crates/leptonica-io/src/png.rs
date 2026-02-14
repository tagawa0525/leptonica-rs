//! PNG image format support
//!
//! Reads and writes images in PNG format using the `png` crate.
//! Supports 1/2/4/8/16-bit grayscale, indexed color (with colormap),
//! RGB, RGBA, and grayscale-alpha.
//!
//! PNG is a lossless format, so roundtrip read/write preserves pixel
//! values exactly (at the same bit depth).
//!
//! # See also
//! C version: `readHeaderPng()`, `pixReadStreamPng()`, `pixWriteStreamPng()`
//! in `pngio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::{BufRead, Seek, Write};

/// Read a PNG image from a reader.
///
/// # Arguments
/// * `reader` - A buffered, seekable reader positioned at the PNG header
///
/// # Returns
/// A `Pix` with the appropriate depth and optional colormap.
pub fn read_png<R: BufRead + Seek>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Write a `Pix` as PNG to a writer.
///
/// # Arguments
/// * `pix` - The image to encode
/// * `writer` - Destination writer
pub fn write_png<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    todo!()
}
