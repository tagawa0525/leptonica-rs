//! TIFF image format support
//!
//! Reads and writes single-page and multipage TIFF images using the
//! `tiff` crate.  Supports various compression methods: None, LZW,
//! Zip/Deflate, PackBits, and (with fallback) G3/G4/RLE.
//!
//! # Multipage TIFF
//! Use `write_tiff_multipage` / `read_tiff_multipage` for documents
//! containing multiple pages.  Individual pages can be accessed via
//! `read_tiff_page`.
//!
//! # See also
//! C version: `pixReadStreamTiff()`, `pixWriteStreamTiff()`,
//! `pixReadFromMultipageTiff()` in `tiffio.c`

use crate::IoResult;
use leptonica_core::{ImageFormat, Pix};
use std::io::{Read, Seek, Write};

/// TIFF compression method.
///
/// Maps to C `IFF_TIFF_*` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiffCompression {
    /// No compression
    #[default]
    None,
    /// CCITT Group 3 fax (falls back to None if unsupported)
    G3,
    /// CCITT Group 4 fax (falls back to None if unsupported)
    G4,
    /// Run-Length Encoding (falls back to None if unsupported)
    Rle,
    /// PackBits
    PackBits,
    /// Lempel-Ziv-Welch
    Lzw,
    /// Zip / Deflate
    Zip,
    /// JPEG-in-TIFF (falls back to None if unsupported)
    Jpeg,
}

impl TiffCompression {
    /// Convert this compression to the corresponding `ImageFormat`.
    pub fn to_image_format(self) -> ImageFormat {
        todo!()
    }

    /// Try to derive a `TiffCompression` from an `ImageFormat`.
    ///
    /// Returns `None` if the format is not a TIFF variant.
    pub fn from_image_format(format: ImageFormat) -> Option<Self> {
        todo!()
    }
}

/// Read the first page of a TIFF image.
///
/// # Arguments
/// * `reader` - A seekable reader positioned at the TIFF header
pub fn read_tiff<R: Read + Seek>(reader: R) -> IoResult<Pix> {
    todo!()
}

/// Read a specific page from a multipage TIFF.
///
/// # Arguments
/// * `reader` - A seekable reader
/// * `page`   - Zero-based page index
pub fn read_tiff_page<R: Read + Seek>(reader: R, page: usize) -> IoResult<Pix> {
    todo!()
}

/// Read all pages from a multipage TIFF.
///
/// # Arguments
/// * `reader` - A seekable reader
///
/// # Returns
/// A `Vec<Pix>` with one entry per page.
pub fn read_tiff_multipage<R: Read + Seek>(reader: R) -> IoResult<Vec<Pix>> {
    todo!()
}

/// Count the number of pages in a multipage TIFF.
///
/// # Arguments
/// * `reader` - A seekable reader
pub fn tiff_page_count<R: Read + Seek>(reader: R) -> IoResult<usize> {
    todo!()
}

/// Write a single `Pix` as a TIFF image.
///
/// # Arguments
/// * `pix`         - The image to encode
/// * `writer`      - A seekable writer
/// * `compression` - Compression method
pub fn write_tiff<W: Write + Seek>(
    pix: &Pix,
    writer: W,
    compression: TiffCompression,
) -> IoResult<()> {
    todo!()
}

/// Write multiple images as a multipage TIFF.
///
/// # Arguments
/// * `pages`       - Slice of images (one per page)
/// * `writer`      - A seekable writer
/// * `compression` - Compression method applied to every page
pub fn write_tiff_multipage<W: Write + Seek>(
    pages: &[&Pix],
    writer: W,
    compression: TiffCompression,
) -> IoResult<()> {
    todo!()
}
