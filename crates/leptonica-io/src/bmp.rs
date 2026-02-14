//! BMP image format support
//!
//! Reads and writes images in Windows BMP format.
//! This is a pure-Rust implementation (no external crate dependency)
//! supporting 1/8/24/32-bit uncompressed BMPs.
//!
//! # See also
//! C version: `pixReadStreamBmp()`, `pixWriteStreamBmp()` in `bmpio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::{BufRead, Seek, Write};

/// Read a BMP image from a reader.
///
/// # Arguments
/// * `reader` - A buffered, seekable reader positioned at the `BM` signature
///
/// # Returns
/// A `Pix` at the appropriate depth.
pub fn read_bmp<R: BufRead + Seek>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Write a `Pix` as BMP to a writer.
///
/// # Arguments
/// * `pix`    - The image to encode
/// * `writer` - Destination writer
pub fn write_bmp<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    todo!()
}
