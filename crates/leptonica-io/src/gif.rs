//! GIF image format support
//!
//! Reads and writes images in GIF format using the `gif` crate.
//! GIF supports up to 256 colors via a color table.
//!
//! For images with depth <= 8 bpp and a colormap, GIF roundtrip is
//! lossless.  Higher-depth images are quantized to 8 bpp during
//! encoding, which is lossy.
//!
//! # See also
//! C version: `pixReadStreamGif()`, `pixWriteStreamGif()` in `gifio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::{Read, Write};

/// Read a GIF image from a reader.
///
/// # Arguments
/// * `reader` - A reader positioned at the GIF header (`GIF87a` / `GIF89a`)
///
/// # Returns
/// A colormapped `Pix` (typically 8 bpp).
pub fn read_gif<R: Read>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Write a `Pix` as GIF to a writer.
///
/// Images deeper than 8 bpp are quantized to 256 colors.
///
/// # Arguments
/// * `pix`    - The image to encode
/// * `writer` - Destination writer
pub fn write_gif<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    todo!()
}
