//! WebP image format support
//!
//! Reads and writes images in WebP format using the `image-webp` crate.
//! Currently only **lossless** encoding is supported (the `image-webp`
//! crate does not expose lossy encoding).  All images are handled as
//! 32-bit RGBA.
//!
//! # See also
//! C version: `pixReadStreamWebP()`, `pixWriteStreamWebP()` in `webpiostub.c`
//! (the C version wraps libwebp)

use crate::IoResult;
use leptonica_core::Pix;
use std::io::{BufRead, Read, Seek, Write};

/// Read a WebP image from a reader.
///
/// # Arguments
/// * `reader` - A reader positioned at the RIFF/WEBP header
///
/// # Returns
/// A 32-bpp `Pix`.
pub fn read_webp<R: Read + BufRead + Seek>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Write a `Pix` as lossless WebP to a writer.
///
/// # Arguments
/// * `pix`    - The image to encode (converted to RGBA internally)
/// * `writer` - Destination writer
pub fn write_webp<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    todo!()
}
