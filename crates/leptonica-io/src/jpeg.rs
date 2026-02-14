//! JPEG image format support
//!
//! Reads JPEG images using the `jpeg-decoder` crate.
//! Supports 8-bit grayscale and 24-bit RGB (decoded to 32-bit RGBA internally).
//!
//! **Note:** JPEG *writing* is not yet implemented because the `jpeg-decoder`
//! crate is decode-only.  A future addition of `jpeg-encoder` or similar
//! would enable write support.
//!
//! # See also
//! C version: `pixReadStreamJpeg()`, `pixWriteStreamJpeg()` in `jpegio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::Read;

/// Read a JPEG image from a reader.
///
/// # Arguments
/// * `reader` - A reader positioned at the JPEG SOI marker (`FF D8`)
///
/// # Returns
/// A `Pix` at 8-bpp (grayscale) or 32-bpp (RGB).
pub fn read_jpeg<R: Read>(reader: R) -> IoResult<Pix> {
    todo!()
}
