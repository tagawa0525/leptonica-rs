//! leptonica-io - Image I/O for Leptonica
//!
//! This crate provides reading and writing support for various image formats.

mod error;
mod format;

#[cfg(feature = "bmp")]
pub mod bmp;

#[cfg(feature = "pnm")]
pub mod pnm;

#[cfg(feature = "png-format")]
pub mod png;

#[cfg(feature = "jpeg")]
pub mod jpeg;

#[cfg(feature = "tiff-format")]
pub mod tiff;

#[cfg(feature = "gif-format")]
pub mod gif;

#[cfg(feature = "webp-format")]
pub mod webp;

#[cfg(feature = "jp2k-format")]
pub mod jp2k;

#[cfg(feature = "pdf-format")]
pub mod pdf;

#[cfg(feature = "ps-format")]
pub mod ps;

pub mod spix;

pub use error::{IoError, IoResult};
pub use format::{detect_format, detect_format_from_bytes};
pub use leptonica_core::{ImageFormat, Pix, PixMut, PixelDepth};

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

/// Read an image from a file path
///
/// The format is automatically detected from the file contents.
pub fn read_image<P: AsRef<Path>>(path: P) -> IoResult<Pix> {
    let path = path.as_ref();
    let file = File::open(path).map_err(IoError::Io)?;
    let mut reader = BufReader::new(file);

    // Read enough bytes to detect format
    let mut header = [0u8; 12];
    let bytes_read = reader.read(&mut header).map_err(IoError::Io)?;

    let format = detect_format_from_bytes(&header[..bytes_read])?;

    // Seek back to beginning
    reader
        .seek(std::io::SeekFrom::Start(0))
        .map_err(IoError::Io)?;

    read_image_format(reader, format)
}

/// Read an image from bytes
pub fn read_image_mem(data: &[u8]) -> IoResult<Pix> {
    let format = detect_format_from_bytes(data)?;
    read_image_format(std::io::Cursor::new(data), format)
}

/// Read an image with a specific format
pub fn read_image_format<R: Read + Seek + std::io::BufRead>(
    reader: R,
    format: ImageFormat,
) -> IoResult<Pix> {
    match format {
        #[cfg(feature = "bmp")]
        ImageFormat::Bmp => bmp::read_bmp(reader),

        #[cfg(feature = "pnm")]
        ImageFormat::Pnm => pnm::read_pnm(reader),

        #[cfg(feature = "png-format")]
        ImageFormat::Png => png::read_png(reader),

        #[cfg(feature = "jpeg")]
        ImageFormat::Jpeg => jpeg::read_jpeg(reader),

        #[cfg(feature = "tiff-format")]
        ImageFormat::Tiff
        | ImageFormat::TiffG3
        | ImageFormat::TiffG4
        | ImageFormat::TiffRle
        | ImageFormat::TiffPackbits
        | ImageFormat::TiffLzw
        | ImageFormat::TiffZip
        | ImageFormat::TiffJpeg => tiff::read_tiff(reader),

        #[cfg(feature = "gif-format")]
        ImageFormat::Gif => gif::read_gif(reader),

        #[cfg(feature = "webp-format")]
        ImageFormat::WebP => webp::read_webp(reader),

        #[cfg(feature = "jp2k-format")]
        ImageFormat::Jp2 => jp2k::read_jp2k(reader),

        ImageFormat::Spix => spix::read_spix(reader),

        _ => Err(IoError::UnsupportedFormat(format!("{:?}", format))),
    }
}

/// Write an image to a file path
pub fn write_image<P: AsRef<Path>>(pix: &Pix, path: P, format: ImageFormat) -> IoResult<()> {
    let file = File::create(path).map_err(IoError::Io)?;

    // TIFF requires Seek, so handle it specially
    #[cfg(feature = "tiff-format")]
    if let Some(compression) = tiff::TiffCompression::from_image_format(format) {
        let writer = BufWriter::new(file);
        return tiff::write_tiff(pix, writer, compression);
    }

    let writer = BufWriter::new(file);
    write_image_format(pix, writer, format)
}

/// Write an image to bytes
pub fn write_image_mem(pix: &Pix, format: ImageFormat) -> IoResult<Vec<u8>> {
    // TIFF requires Seek, so handle it specially with Cursor
    #[cfg(feature = "tiff-format")]
    if let Some(compression) = tiff::TiffCompression::from_image_format(format) {
        let mut cursor = std::io::Cursor::new(Vec::new());
        tiff::write_tiff(pix, &mut cursor, compression)?;
        return Ok(cursor.into_inner());
    }

    let mut buffer = Vec::new();
    write_image_format(pix, &mut buffer, format)?;
    Ok(buffer)
}

/// Write an image with a specific format
///
/// Note: TIFF format requires a seekable writer. Use `write_image` for file output
/// or `write_image_mem` for in-memory output, or use `tiff::write_tiff` directly.
pub fn write_image_format<W: Write>(pix: &Pix, writer: W, format: ImageFormat) -> IoResult<()> {
    match format {
        #[cfg(feature = "bmp")]
        ImageFormat::Bmp => bmp::write_bmp(pix, writer),

        #[cfg(feature = "pnm")]
        ImageFormat::Pnm => pnm::write_pnm(pix, writer),

        #[cfg(feature = "png-format")]
        ImageFormat::Png => png::write_png(pix, writer),

        #[cfg(feature = "jpeg")]
        ImageFormat::Jpeg => jpeg::write_jpeg(pix, writer, &jpeg::JpegOptions::default()),

        #[cfg(feature = "tiff-format")]
        ImageFormat::Tiff
        | ImageFormat::TiffG3
        | ImageFormat::TiffG4
        | ImageFormat::TiffRle
        | ImageFormat::TiffPackbits
        | ImageFormat::TiffLzw
        | ImageFormat::TiffZip
        | ImageFormat::TiffJpeg => {
            // TIFF requires Seek trait. Use write_image or write_image_mem instead.
            Err(IoError::UnsupportedFormat(
                "TIFF requires seekable writer; use write_image or write_image_mem".to_string(),
            ))
        }

        #[cfg(feature = "gif-format")]
        ImageFormat::Gif => gif::write_gif(pix, writer),

        #[cfg(feature = "webp-format")]
        ImageFormat::WebP => webp::write_webp(pix, writer),

        #[cfg(feature = "jp2k-format")]
        ImageFormat::Jp2 => Err(IoError::UnsupportedFormat(
            "JP2K writing not yet supported".to_string(),
        )),

        #[cfg(feature = "pdf-format")]
        ImageFormat::Lpdf => pdf::write_pdf(pix, writer, &pdf::PdfOptions::default()),

        #[cfg(feature = "ps-format")]
        ImageFormat::Ps => ps::write_ps(pix, writer, &ps::PsOptions::default()),

        ImageFormat::Spix => spix::write_spix(pix, writer),

        _ => Err(IoError::UnsupportedFormat(format!("{:?}", format))),
    }
}
