//! TIFF image format support
//!
//! This module provides reading and writing support for TIFF images,
//! including multipage TIFFs and various compression formats.

use crate::{IoError, IoResult, header::ImageHeader};
use leptonica_core::{ImageFormat, Pix, PixelDepth, color};
use std::io::{Read, Seek, Write};
use tiff::ColorType;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::colortype::{Gray8, Gray16, RGB8, RGBA8};
use tiff::encoder::{Compression, TiffEncoder};
use tiff::tags::PhotometricInterpretation;

/// Read TIFF header metadata without decoding pixel data
pub fn read_header_tiff(data: &[u8]) -> IoResult<ImageHeader> {
    let cursor = std::io::Cursor::new(data);
    let mut decoder = Decoder::new(cursor)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    let (width, height) = decoder
        .dimensions()
        .map_err(|e| IoError::DecodeError(format!("TIFF dimensions: {}", e)))?;

    let color_type = decoder
        .colortype()
        .map_err(|e| IoError::DecodeError(format!("TIFF colortype: {}", e)))?;

    let (depth, spp, bps) = match color_type {
        ColorType::Gray(n) => {
            let d = if n <= 8 { 8u32 } else { 16 };
            (d, 1u32, n as u32)
        }
        ColorType::GrayA(n) => (32, 4, n as u32),
        ColorType::Palette(n) => (n as u32, 1, n as u32),
        ColorType::RGB(n) => (32, 3, n as u32),
        ColorType::RGBA(n) => (32, 4, n as u32),
        _ => (32, 3, 8),
    };

    // DPI from TIFF tags
    let x_dpi = decoder
        .get_tag_f32(tiff::tags::Tag::XResolution)
        .ok()
        .map(|v| v.round() as u32);
    let y_dpi = decoder
        .get_tag_f32(tiff::tags::Tag::YResolution)
        .ok()
        .map(|v| v.round() as u32);

    Ok(ImageHeader {
        width,
        height,
        depth,
        bps,
        spp,
        has_colormap: matches!(color_type, ColorType::Palette(_)),
        num_colors: 0,
        format: ImageFormat::Tiff,
        x_resolution: x_dpi,
        y_resolution: y_dpi,
    })
}

/// TIFF compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiffCompression {
    /// No compression
    #[default]
    None,
    /// CCITT Group 3 (fax) - not directly supported, falls back to None
    G3,
    /// CCITT Group 4 (most efficient binary compression) - not directly supported, falls back to None
    G4,
    /// Run-Length Encoding - not directly supported, falls back to None
    Rle,
    /// PackBits compression
    PackBits,
    /// LZW compression
    Lzw,
    /// ZIP/Deflate compression
    Zip,
}

impl TiffCompression {
    /// Convert to ImageFormat
    pub fn to_image_format(self) -> ImageFormat {
        match self {
            TiffCompression::None => ImageFormat::Tiff,
            TiffCompression::G3 => ImageFormat::TiffG3,
            TiffCompression::G4 => ImageFormat::TiffG4,
            TiffCompression::Rle => ImageFormat::TiffRle,
            TiffCompression::PackBits => ImageFormat::TiffPackbits,
            TiffCompression::Lzw => ImageFormat::TiffLzw,
            TiffCompression::Zip => ImageFormat::TiffZip,
        }
    }

    /// Create from ImageFormat
    pub fn from_image_format(format: ImageFormat) -> Option<Self> {
        match format {
            ImageFormat::Tiff => Some(TiffCompression::None),
            ImageFormat::TiffG3 => Some(TiffCompression::G3),
            ImageFormat::TiffG4 => Some(TiffCompression::G4),
            ImageFormat::TiffRle => Some(TiffCompression::Rle),
            ImageFormat::TiffPackbits => Some(TiffCompression::PackBits),
            ImageFormat::TiffLzw => Some(TiffCompression::Lzw),
            ImageFormat::TiffZip => Some(TiffCompression::Zip),
            _ => None,
        }
    }

    /// Convert to tiff crate's Compression enum
    fn to_tiff_compression(self) -> Compression {
        match self {
            TiffCompression::None
            | TiffCompression::G3
            | TiffCompression::G4
            | TiffCompression::Rle => Compression::Uncompressed,
            TiffCompression::PackBits => Compression::Packbits,
            TiffCompression::Lzw => Compression::Lzw,
            TiffCompression::Zip => Compression::Deflate(tiff::encoder::DeflateLevel::default()),
        }
    }
}

/// Read a single-page TIFF image
///
/// If the TIFF file contains multiple pages, only the first page is read.
/// Use `read_tiff_multipage` to read all pages.
pub fn read_tiff<R: Read + Seek>(reader: R) -> IoResult<Pix> {
    read_tiff_page(reader, 0)
}

/// Read a specific page from a TIFF file
///
/// # Arguments
///
/// * `reader` - The reader to read from
/// * `page` - The page index (0-based)
pub fn read_tiff_page<R: Read + Seek>(reader: R, page: usize) -> IoResult<Pix> {
    let mut decoder = Decoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    // Navigate to the requested page
    for _ in 0..page {
        if !decoder.more_images() {
            return Err(IoError::InvalidData(format!(
                "TIFF file has fewer than {} pages",
                page + 1
            )));
        }
        decoder
            .next_image()
            .map_err(|e| IoError::DecodeError(format!("TIFF page navigation error: {}", e)))?;
    }

    decode_tiff_image(&mut decoder)
}

/// Read all pages from a multipage TIFF file
pub fn read_tiff_multipage<R: Read + Seek>(reader: R) -> IoResult<Vec<Pix>> {
    let mut decoder = Decoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    let mut pages = Vec::new();

    loop {
        let pix = decode_tiff_image(&mut decoder)?;
        pages.push(pix);

        if !decoder.more_images() {
            break;
        }

        decoder
            .next_image()
            .map_err(|e| IoError::DecodeError(format!("TIFF page navigation error: {}", e)))?;
    }

    Ok(pages)
}

/// Get the number of pages in a TIFF file
pub fn tiff_page_count<R: Read + Seek>(reader: R) -> IoResult<usize> {
    let mut decoder = Decoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    let mut count = 1;
    while decoder.more_images() {
        decoder
            .next_image()
            .map_err(|e| IoError::DecodeError(format!("TIFF page navigation error: {}", e)))?;
        count += 1;
    }

    Ok(count)
}

/// Get the resolution (DPI) of a TIFF file
///
/// Returns `(x_dpi, y_dpi)` if resolution information is available.
pub fn tiff_resolution<R: Read + Seek>(reader: R) -> IoResult<Option<(f32, f32)>> {
    let mut decoder = Decoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    // Try to get resolution from TIFF tags
    let x_res = decoder.get_tag_f32(tiff::tags::Tag::XResolution).ok();
    let y_res = decoder.get_tag_f32(tiff::tags::Tag::YResolution).ok();

    match (x_res, y_res) {
        (Some(x), Some(y)) => Ok(Some((x, y))),
        _ => Ok(None),
    }
}

/// Detect the compression method used in a TIFF file
///
/// Reads the Compression tag from the first page of the TIFF.
pub fn tiff_compression<R: Read + Seek>(reader: R) -> IoResult<TiffCompression> {
    let mut decoder = Decoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("TIFF decode error: {}", e)))?;

    let compression_val = decoder
        .get_tag_u32(tiff::tags::Tag::Compression)
        .unwrap_or(1); // Default: no compression

    Ok(match compression_val {
        1 => TiffCompression::None,
        2 => TiffCompression::Rle, // CCITT modified Huffman RLE
        3 => TiffCompression::G3,
        4 => TiffCompression::G4,
        5 => TiffCompression::Lzw,
        8 | 0x80B2 => TiffCompression::Zip, // Deflate or OldDeflate
        0x8005 => TiffCompression::PackBits,
        _ => TiffCompression::None, // Unknown → treat as None
    })
}

/// Append one or more pages to an existing multipage TIFF
///
/// Reads all existing pages from the reader, then writes them plus the new
/// pages to the writer. The `tiff` crate does not support true append mode,
/// so the file is rewritten with the additional pages. This involves
/// decoding all existing pages and re-encoding them into a new multipage
/// TIFF, so the specified `compression` is applied to all pages in the
/// output (both existing and new), and original TIFF tags/metadata may not
/// be preserved.
///
/// # Arguments
///
/// * `existing` - Reader for the existing TIFF data
/// * `new_pages` - New pages to append
/// * `writer` - Writer for the output (can be the same file via a buffer)
/// * `compression` - Compression to use for all pages in the output TIFF
pub fn write_tiff_append<R: Read + Seek, W: Write + Seek>(
    existing: R,
    new_pages: &[&Pix],
    writer: W,
    compression: TiffCompression,
) -> IoResult<()> {
    if new_pages.is_empty() {
        return Err(IoError::InvalidData("no pages to append".to_string()));
    }

    // Read all existing pages
    let existing_pages = read_tiff_multipage(existing)?;

    // Write all pages (existing + new) as a single multipage TIFF
    let mut all_pages: Vec<&Pix> = existing_pages.iter().collect();
    all_pages.extend_from_slice(new_pages);

    write_tiff_multipage(&all_pages, writer, compression)
}

/// Decode a TIFF image from the current decoder position
fn decode_tiff_image<R: Read + Seek>(decoder: &mut Decoder<R>) -> IoResult<Pix> {
    let (width, height) = decoder
        .dimensions()
        .map_err(|e| IoError::DecodeError(format!("Failed to get TIFF dimensions: {}", e)))?;
    let color_type = decoder
        .colortype()
        .map_err(|e| IoError::DecodeError(format!("Failed to get TIFF color type: {}", e)))?;

    let photometric = decoder
        .get_tag_u32(tiff::tags::Tag::PhotometricInterpretation)
        .ok()
        .map(|v| match v {
            0 => PhotometricInterpretation::WhiteIsZero,
            1 => PhotometricInterpretation::BlackIsZero,
            2 => PhotometricInterpretation::RGB,
            3 => PhotometricInterpretation::RGBPalette,
            _ => PhotometricInterpretation::BlackIsZero,
        })
        .unwrap_or(PhotometricInterpretation::BlackIsZero);

    let image_data = decoder
        .read_image()
        .map_err(|e| IoError::DecodeError(format!("Failed to read TIFF image data: {}", e)))?;

    // Determine pixel depth and create Pix
    let (pix_depth, spp) = match color_type {
        ColorType::Gray(1) => (PixelDepth::Bit1, 1),
        ColorType::Gray(2) => (PixelDepth::Bit2, 1),
        ColorType::Gray(4) => (PixelDepth::Bit4, 1),
        ColorType::Gray(8) => (PixelDepth::Bit8, 1),
        ColorType::Gray(16) => (PixelDepth::Bit16, 1),
        ColorType::RGB(8) => (PixelDepth::Bit32, 3),
        ColorType::RGB(16) => (PixelDepth::Bit32, 3),
        ColorType::RGBA(8) => (PixelDepth::Bit32, 4),
        ColorType::RGBA(16) => (PixelDepth::Bit32, 4),
        ColorType::GrayA(8) => (PixelDepth::Bit32, 2),
        ColorType::GrayA(16) => (PixelDepth::Bit32, 2),
        ColorType::Palette(bits) => (
            PixelDepth::from_bits(bits as u32).unwrap_or(PixelDepth::Bit8),
            1,
        ),
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "unsupported TIFF color type: {:?}",
                color_type
            )));
        }
    };

    let pix = Pix::new(width, height, pix_depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();
    pix_mut.set_spp(spp);

    // Set resolution if available
    if let Ok(x_res) = decoder.get_tag_f32(tiff::tags::Tag::XResolution) {
        pix_mut.set_xres(x_res as i32);
    }
    if let Ok(y_res) = decoder.get_tag_f32(tiff::tags::Tag::YResolution) {
        pix_mut.set_yres(y_res as i32);
    }

    // Handle inverted photometric interpretation for binary images
    let invert = matches!(
        (pix_depth, photometric),
        (PixelDepth::Bit1, PhotometricInterpretation::WhiteIsZero)
    );

    // Convert decoded data to Pix format
    match image_data {
        DecodingResult::U8(data) => {
            convert_u8_to_pix(&data, &mut pix_mut, color_type, invert)?;
        }
        DecodingResult::U16(data) => {
            convert_u16_to_pix(&data, &mut pix_mut, color_type)?;
        }
        DecodingResult::U32(data) => {
            convert_u32_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::U64(data) => {
            convert_u64_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::F32(data) => {
            convert_f32_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::F64(data) => {
            convert_f64_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::I8(data) => {
            convert_i8_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::I16(data) => {
            convert_i16_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::I32(data) => {
            convert_i32_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::I64(data) => {
            convert_i64_to_pix(&data, &mut pix_mut)?;
        }
        DecodingResult::F16(data) => {
            convert_f16_to_pix(&data, &mut pix_mut)?;
        }
    }

    pix_mut.set_informat(ImageFormat::Tiff);
    Ok(pix_mut.into())
}

/// Convert U8 data to Pix format
fn convert_u8_to_pix(
    data: &[u8],
    pix_mut: &mut leptonica_core::PixMut,
    color_type: ColorType,
    invert: bool,
) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    match color_type {
        ColorType::Gray(1) => {
            // 1-bit grayscale - data is packed 8 pixels per byte
            let bytes_per_row = width.div_ceil(8) as usize;
            for y in 0..height {
                for x in 0..width {
                    let byte_idx = (y as usize * bytes_per_row) + (x / 8) as usize;
                    let bit_idx = 7 - (x % 8);
                    if byte_idx < data.len() {
                        let mut val = (data[byte_idx] >> bit_idx) & 1;
                        if invert {
                            val = 1 - val;
                        }
                        pix_mut.set_pixel_unchecked(x, y, val as u32);
                    }
                }
            }
        }
        ColorType::Gray(2) => {
            // 2-bit grayscale
            let bytes_per_row = width.div_ceil(4) as usize;
            for y in 0..height {
                for x in 0..width {
                    let byte_idx = (y as usize * bytes_per_row) + (x / 4) as usize;
                    let shift = 6 - ((x % 4) * 2);
                    if byte_idx < data.len() {
                        let val = (data[byte_idx] >> shift) & 3;
                        pix_mut.set_pixel_unchecked(x, y, val as u32);
                    }
                }
            }
        }
        ColorType::Gray(4) => {
            // 4-bit grayscale
            let bytes_per_row = width.div_ceil(2) as usize;
            for y in 0..height {
                for x in 0..width {
                    let byte_idx = (y as usize * bytes_per_row) + (x / 2) as usize;
                    if byte_idx < data.len() {
                        let val = if x % 2 == 0 {
                            (data[byte_idx] >> 4) & 0xF
                        } else {
                            data[byte_idx] & 0xF
                        };
                        pix_mut.set_pixel_unchecked(x, y, val as u32);
                    }
                }
            }
        }
        ColorType::Gray(8) => {
            // 8-bit grayscale
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    if idx < data.len() {
                        pix_mut.set_pixel_unchecked(x, y, data[idx] as u32);
                    }
                }
            }
        }
        ColorType::RGB(8) => {
            // 24-bit RGB
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 3) as usize;
                    if idx + 2 < data.len() {
                        let r = data[idx];
                        let g = data[idx + 1];
                        let b = data[idx + 2];
                        let pixel = color::compose_rgb(r, g, b);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        ColorType::RGBA(8) => {
            // 32-bit RGBA
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    if idx + 3 < data.len() {
                        let r = data[idx];
                        let g = data[idx + 1];
                        let b = data[idx + 2];
                        let a = data[idx + 3];
                        let pixel = color::compose_rgba(r, g, b, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        ColorType::GrayA(8) => {
            // Grayscale with alpha - convert to RGBA
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    if idx + 1 < data.len() {
                        let g = data[idx];
                        let a = data[idx + 1];
                        let pixel = color::compose_rgba(g, g, g, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        ColorType::Palette(bits) => {
            // Palette-based - treat as indexed (would need colormap support)
            let bits_per_pixel = bits as u32;
            let pixels_per_byte = 8 / bits_per_pixel;
            let bytes_per_row = width.div_ceil(pixels_per_byte) as usize;

            for y in 0..height {
                for x in 0..width {
                    let idx = if bits == 8 {
                        (y * width + x) as usize
                    } else {
                        let byte_idx = y as usize * bytes_per_row + (x / pixels_per_byte) as usize;
                        let shift = (8 - bits_per_pixel) - ((x % pixels_per_byte) * bits_per_pixel);
                        if byte_idx < data.len() {
                            ((data[byte_idx] >> shift) & ((1 << bits_per_pixel) - 1)) as usize
                        } else {
                            0
                        }
                    };
                    if idx < data.len() {
                        pix_mut.set_pixel_unchecked(x, y, data[idx] as u32);
                    }
                }
            }
        }
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "unsupported color type for U8 data: {:?}",
                color_type
            )));
        }
    }
    Ok(())
}

/// Convert U16 data to Pix format
fn convert_u16_to_pix(
    data: &[u16],
    pix_mut: &mut leptonica_core::PixMut,
    color_type: ColorType,
) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    match color_type {
        ColorType::Gray(16) => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    if idx < data.len() {
                        pix_mut.set_pixel_unchecked(x, y, data[idx] as u32);
                    }
                }
            }
        }
        ColorType::RGB(16) => {
            // 48-bit RGB - convert to 24-bit
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 3) as usize;
                    if idx + 2 < data.len() {
                        let r = (data[idx] >> 8) as u8;
                        let g = (data[idx + 1] >> 8) as u8;
                        let b = (data[idx + 2] >> 8) as u8;
                        let pixel = color::compose_rgb(r, g, b);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        ColorType::RGBA(16) => {
            // 64-bit RGBA - convert to 32-bit
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    if idx + 3 < data.len() {
                        let r = (data[idx] >> 8) as u8;
                        let g = (data[idx + 1] >> 8) as u8;
                        let b = (data[idx + 2] >> 8) as u8;
                        let a = (data[idx + 3] >> 8) as u8;
                        let pixel = color::compose_rgba(r, g, b, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        ColorType::GrayA(16) => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    if idx + 1 < data.len() {
                        let g = (data[idx] >> 8) as u8;
                        let a = (data[idx + 1] >> 8) as u8;
                        let pixel = color::compose_rgba(g, g, g, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }
        }
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "unsupported color type for U16 data: {:?}",
                color_type
            )));
        }
    }
    Ok(())
}

/// Convert U32 data to Pix format
fn convert_u32_to_pix(data: &[u32], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                pix_mut.set_pixel_unchecked(x, y, data[idx]);
            }
        }
    }
    Ok(())
}

/// Convert U64 data to Pix format (downsample to 32-bit)
fn convert_u64_to_pix(data: &[u64], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                pix_mut.set_pixel_unchecked(x, y, (data[idx] >> 32) as u32);
            }
        }
    }
    Ok(())
}

/// Convert F16 data to Pix format
fn convert_f16_to_pix(data: &[half::f16], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    // Find min/max for normalization
    let min = data
        .iter()
        .map(|v| v.to_f32())
        .fold(f32::INFINITY, f32::min);
    let max = data
        .iter()
        .map(|v| v.to_f32())
        .fold(f32::NEG_INFINITY, f32::max);
    let range = if (max - min).abs() < f32::EPSILON {
        1.0
    } else {
        max - min
    };

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                let normalized = ((data[idx].to_f32() - min) / range * 255.0) as u32;
                pix_mut.set_pixel_unchecked(x, y, normalized.min(255));
            }
        }
    }
    Ok(())
}

/// Convert F32 data to Pix format
fn convert_f32_to_pix(data: &[f32], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    // Find min/max for normalization
    let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = if (max - min).abs() < f32::EPSILON {
        1.0
    } else {
        max - min
    };

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                let normalized = ((data[idx] - min) / range * 255.0) as u32;
                pix_mut.set_pixel_unchecked(x, y, normalized.min(255));
            }
        }
    }
    Ok(())
}

/// Convert F64 data to Pix format
fn convert_f64_to_pix(data: &[f64], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                let normalized = ((data[idx] - min) / range * 255.0) as u32;
                pix_mut.set_pixel_unchecked(x, y, normalized.min(255));
            }
        }
    }
    Ok(())
}

/// Convert I8 data to Pix format
fn convert_i8_to_pix(data: &[i8], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                // Map -128..127 to 0..255
                let val = ((data[idx] as i32) + 128) as u32;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }
    Ok(())
}

/// Convert I16 data to Pix format
fn convert_i16_to_pix(data: &[i16], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                // Map to 0..65535
                let val = ((data[idx] as i32) + 32768) as u32;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }
    Ok(())
}

/// Convert I32 data to Pix format
fn convert_i32_to_pix(data: &[i32], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                // Just take lower 32 bits, shifted to unsigned range
                let val = (data[idx] as u32).wrapping_add(0x80000000);
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }
    Ok(())
}

/// Convert I64 data to Pix format
fn convert_i64_to_pix(data: &[i64], pix_mut: &mut leptonica_core::PixMut) -> IoResult<()> {
    let width = pix_mut.width();
    let height = pix_mut.height();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx < data.len() {
                let val = ((data[idx] >> 32) as u32).wrapping_add(0x80000000);
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }
    Ok(())
}

/// Write a single-page TIFF image
///
/// # Arguments
///
/// * `pix` - The image to write
/// * `writer` - The writer to write to
/// * `compression` - The compression format to use
pub fn write_tiff<W: Write + Seek>(
    pix: &Pix,
    writer: W,
    compression: TiffCompression,
) -> IoResult<()> {
    let tiff_compression = compression.to_tiff_compression();
    let encoder = TiffEncoder::new(writer)
        .map_err(|e| IoError::EncodeError(format!("TIFF encoder error: {}", e)))?
        .with_compression(tiff_compression);

    write_pix_to_encoder(encoder, pix)
}

/// Write a multipage TIFF image
///
/// # Arguments
///
/// * `pages` - The images to write
/// * `writer` - The writer to write to
/// * `compression` - The compression format to use for all pages
pub fn write_tiff_multipage<W: Write + Seek>(
    pages: &[&Pix],
    writer: W,
    compression: TiffCompression,
) -> IoResult<()> {
    if pages.is_empty() {
        return Err(IoError::InvalidData("no pages to write".to_string()));
    }

    let tiff_compression = compression.to_tiff_compression();
    let mut encoder = TiffEncoder::new(writer)
        .map_err(|e| IoError::EncodeError(format!("TIFF encoder error: {}", e)))?
        .with_compression(tiff_compression);

    for pix in pages {
        write_pix_page_to_encoder(&mut encoder, pix)?;
    }

    Ok(())
}

/// Write a Pix to a TiffEncoder (consumes the encoder)
fn write_pix_to_encoder<W: Write + Seek>(mut encoder: TiffEncoder<W>, pix: &Pix) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    match pix.depth() {
        PixelDepth::Bit1 => {
            // 1-bit binary - convert to 8-bit for simplicity
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    data[(y * width + x) as usize] = if val != 0 { 255 } else { 0 };
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit2 | PixelDepth::Bit4 => {
            // 2-bit and 4-bit - convert to 8-bit
            let max_val = pix.depth().max_value();
            let scale = 255 / max_val;
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    data[(y * width + x) as usize] = (val * scale) as u8;
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit8 => {
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    data[(y * width + x) as usize] = pix.get_pixel(x, y).unwrap_or(0) as u8;
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit16 => {
            let mut data = vec![0u16; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    data[(y * width + x) as usize] = pix.get_pixel(x, y).unwrap_or(0) as u16;
                }
            }
            encoder
                .write_image::<Gray16>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit32 => {
            if pix.spp() == 4 {
                // RGBA
                let mut data = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b, a) = color::extract_rgba(pixel);
                        let idx = ((y * width + x) * 4) as usize;
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                        data[idx + 3] = a;
                    }
                }
                encoder
                    .write_image::<RGBA8>(width, height, &data)
                    .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
            } else {
                // RGB
                let mut data = vec![0u8; (width * height * 3) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b) = color::extract_rgb(pixel);
                        let idx = ((y * width + x) * 3) as usize;
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                    }
                }
                encoder
                    .write_image::<RGB8>(width, height, &data)
                    .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
            }
        }
    }

    Ok(())
}

/// Write a Pix page to a TiffEncoder (for multipage support)
fn write_pix_page_to_encoder<W: Write + Seek>(
    encoder: &mut TiffEncoder<W>,
    pix: &Pix,
) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    match pix.depth() {
        PixelDepth::Bit1 => {
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    data[(y * width + x) as usize] = if val != 0 { 255 } else { 0 };
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit2 | PixelDepth::Bit4 => {
            let max_val = pix.depth().max_value();
            let scale = 255 / max_val;
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    data[(y * width + x) as usize] = (val * scale) as u8;
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit8 => {
            let mut data = vec![0u8; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    data[(y * width + x) as usize] = pix.get_pixel(x, y).unwrap_or(0) as u8;
                }
            }
            encoder
                .write_image::<Gray8>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit16 => {
            let mut data = vec![0u16; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    data[(y * width + x) as usize] = pix.get_pixel(x, y).unwrap_or(0) as u16;
                }
            }
            encoder
                .write_image::<Gray16>(width, height, &data)
                .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
        }
        PixelDepth::Bit32 => {
            if pix.spp() == 4 {
                let mut data = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b, a) = color::extract_rgba(pixel);
                        let idx = ((y * width + x) * 4) as usize;
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                        data[idx + 3] = a;
                    }
                }
                encoder
                    .write_image::<RGBA8>(width, height, &data)
                    .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
            } else {
                let mut data = vec![0u8; (width * height * 3) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel(x, y).unwrap_or(0);
                        let (r, g, b) = color::extract_rgb(pixel);
                        let idx = ((y * width + x) * 3) as usize;
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                    }
                }
                encoder
                    .write_image::<RGB8>(width, height, &data)
                    .map_err(|e| IoError::EncodeError(format!("TIFF write error: {}", e)))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tiff_roundtrip_gray8() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel(x, y, (x + y) * 10).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix, &mut buffer, TiffCompression::None).unwrap();

        buffer.set_position(0);
        let pix2 = read_tiff(buffer).unwrap();

        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);

        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_tiff_roundtrip_rgb() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_rgb(0, 0, 255, 0, 0).unwrap();
        pix_mut.set_rgb(1, 1, 0, 255, 0).unwrap();
        pix_mut.set_rgb(2, 2, 0, 0, 255).unwrap();

        let pix: Pix = pix_mut.into();

        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix, &mut buffer, TiffCompression::None).unwrap();

        buffer.set_position(0);
        let pix2 = read_tiff(buffer).unwrap();

        assert_eq!(pix2.get_rgb(0, 0), Some((255, 0, 0)));
        assert_eq!(pix2.get_rgb(1, 1), Some((0, 255, 0)));
        assert_eq!(pix2.get_rgb(2, 2), Some((0, 0, 255)));
    }

    #[test]
    fn test_tiff_compression_formats() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..8 {
            for x in 0..8 {
                pix_mut.set_pixel(x, y, ((x + y) * 16) % 256).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        for compression in [TiffCompression::None, TiffCompression::PackBits] {
            let mut buffer = Cursor::new(Vec::new());
            write_tiff(&pix, &mut buffer, compression).unwrap();

            buffer.set_position(0);
            let pix2 = read_tiff(buffer).unwrap();

            assert_eq!(pix2.width(), 8);
            assert_eq!(pix2.height(), 8);
        }
    }

    #[test]
    fn test_tiff_multipage() {
        let pix1 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(6, 6, PixelDepth::Bit8).unwrap();

        let pages: Vec<&Pix> = vec![&pix1, &pix2];
        let mut buffer = Cursor::new(Vec::new());
        write_tiff_multipage(&pages, &mut buffer, TiffCompression::None).unwrap();

        buffer.set_position(0);
        let count = tiff_page_count(buffer.clone()).unwrap();
        assert_eq!(count, 2);

        buffer.set_position(0);
        let loaded = read_tiff_multipage(buffer).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].width(), 4);
        assert_eq!(loaded[1].width(), 6);
    }

    #[test]
    fn test_tiff_page_navigation() {
        let pix1 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(8, 8, PixelDepth::Bit8).unwrap();

        let pages: Vec<&Pix> = vec![&pix1, &pix2];
        let mut buffer = Cursor::new(Vec::new());
        write_tiff_multipage(&pages, &mut buffer, TiffCompression::None).unwrap();

        buffer.set_position(0);
        let page0 = read_tiff_page(buffer.clone(), 0).unwrap();
        assert_eq!(page0.width(), 4);

        buffer.set_position(0);
        let page1 = read_tiff_page(buffer, 1).unwrap();
        assert_eq!(page1.width(), 8);
    }

    #[test]
    fn test_tiff_compression_enum() {
        assert_eq!(TiffCompression::None.to_image_format(), ImageFormat::Tiff);
        assert_eq!(TiffCompression::G4.to_image_format(), ImageFormat::TiffG4);
        assert_eq!(
            TiffCompression::from_image_format(ImageFormat::TiffLzw),
            Some(TiffCompression::Lzw)
        );
        assert_eq!(TiffCompression::from_image_format(ImageFormat::Png), None);
    }

    #[test]
    fn test_tiff_compression_detect_none() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix, &mut buffer, TiffCompression::None).unwrap();

        buffer.set_position(0);
        let compression = tiff_compression(buffer).unwrap();
        assert_eq!(compression, TiffCompression::None);
    }

    #[test]
    fn test_tiff_compression_detect_lzw() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix, &mut buffer, TiffCompression::Lzw).unwrap();

        buffer.set_position(0);
        let compression = tiff_compression(buffer).unwrap();
        assert_eq!(compression, TiffCompression::Lzw);
    }

    #[test]
    fn test_tiff_compression_detect_zip() {
        let pix = Pix::new(8, 8, PixelDepth::Bit8).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix, &mut buffer, TiffCompression::Zip).unwrap();

        buffer.set_position(0);
        let compression = tiff_compression(buffer).unwrap();
        assert_eq!(compression, TiffCompression::Zip);
    }

    #[test]
    fn test_write_tiff_append_single() {
        // Create an initial 2-page TIFF
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let pages: Vec<&Pix> = vec![&pix1, &pix2];
        let mut buffer = Cursor::new(Vec::new());
        write_tiff_multipage(&pages, &mut buffer, TiffCompression::None).unwrap();

        // Append a third page
        let pix3 = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let existing = Cursor::new(buffer.into_inner());
        let mut output = Cursor::new(Vec::new());
        write_tiff_append(existing, &[&pix3], &mut output, TiffCompression::None).unwrap();

        // Verify 3 pages
        output.set_position(0);
        let count = tiff_page_count(output.clone()).unwrap();
        assert_eq!(count, 3);

        output.set_position(0);
        let loaded = read_tiff_multipage(output).unwrap();
        assert_eq!(loaded[0].width(), 10);
        assert_eq!(loaded[1].width(), 20);
        assert_eq!(loaded[2].width(), 30);
    }

    #[test]
    fn test_write_tiff_append_multiple() {
        // Create an initial single-page TIFF
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        write_tiff(&pix1, &mut buffer, TiffCompression::Lzw).unwrap();

        // Append two more pages
        let pix2 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let pix3 = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
        let existing = Cursor::new(buffer.into_inner());
        let mut output = Cursor::new(Vec::new());
        write_tiff_append(existing, &[&pix2, &pix3], &mut output, TiffCompression::Lzw).unwrap();

        // Verify 3 pages
        output.set_position(0);
        let loaded = read_tiff_multipage(output).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].width(), 10);
        assert_eq!(loaded[1].width(), 20);
        assert_eq!(loaded[2].width(), 30);
    }
}
