//! JPEG 2000 (JP2 / J2K) format support
//!
//! Reads JPEG 2000 images using the `hayro-jpeg2000` crate.
//! Writing is not yet supported.
//!
//! # See also
//! C version: `pixReadStreamJp2k()` in `jp2kio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::Read;

/// Read a JPEG 2000 image from a reader.
///
/// # Arguments
/// * `reader` - A reader positioned at the JP2/J2K header
///
/// # Returns
/// A `Pix` at 8 bpp (grayscale) or 32 bpp (RGB/RGBA).
pub fn read_jp2k<R: Read>(reader: R) -> IoResult<Pix> {
    todo!()
}
