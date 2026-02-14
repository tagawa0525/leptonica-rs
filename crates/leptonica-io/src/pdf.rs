//! PDF output support
//!
//! Writes images as single-page PDF documents using the `pdf-writer`
//! crate.  Image data is compressed with Deflate (`miniz_oxide`).
//!
//! This module is **write-only**; PDF reading is not supported.
//!
//! # See also
//! C version: `pixWriteStreamPdf()`, `convertToPdf()` in `pdfio1.c`

use crate::IoResult;
use leptonica_core::Pix;
use std::io::Write;

/// Options for PDF output.
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Document title (default: empty)
    pub title: String,
    /// Horizontal resolution in DPI (default: 300)
    pub x_res: f32,
    /// Vertical resolution in DPI (default: 300)
    pub y_res: f32,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            x_res: 300.0,
            y_res: 300.0,
        }
    }
}

/// Write a `Pix` as a single-page PDF.
///
/// # Arguments
/// * `pix`     - The image to embed
/// * `writer`  - Destination writer
/// * `options` - PDF metadata and resolution
pub fn write_pdf<W: Write>(pix: &Pix, writer: W, options: &PdfOptions) -> IoResult<()> {
    todo!()
}
