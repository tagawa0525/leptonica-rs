//! PNM (Portable Any Map) format support
//!
//! Reads and writes PBM (P4 binary), PGM (P5 binary), and
//! PPM (P6 binary) formats.  ASCII variants (P1/P2/P3) and
//! PAM (P7) are not yet supported.
//!
//! # See also
//! C version: `pixReadStreamPnm()`, `pixWriteStreamPnm()` in `pnmio.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::{BufRead, Seek, Write};

/// Read a PNM image (P4/P5/P6) from a reader.
///
/// # Arguments
/// * `reader` - A buffered, seekable reader positioned at the `P4`/`P5`/`P6` magic
///
/// # Returns
/// A `Pix` at 1 bpp (PBM), 8 bpp (PGM), or 32 bpp (PPM).
pub fn read_pnm<R: BufRead + Seek>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Write a `Pix` as binary PNM to a writer.
///
/// Chooses P4 (1 bpp), P5 (8 bpp grayscale), or P6 (32 bpp RGB)
/// based on the pixel depth.
///
/// # Arguments
/// * `pix`    - The image to encode
/// * `writer` - Destination writer
pub fn write_pnm<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    todo!()
}
