//! WebP image format support
//!
//! Provides reading and writing support for WebP images.
//! Animated WebP images (multiple frames) are not supported.
//!
//! # Notes
//!
//! - Reading: Supports both lossy and lossless WebP images
//! - Writing: Currently only lossless encoding is supported by the underlying library

use crate::{IoError, IoResult};
use image_webp::{ColorType, WebPDecoder, WebPEncoder};
use leptonica_core::{Pix, PixelDepth};
use std::io::{BufRead, Read, Seek, Write};

/// Read a WebP image
///
/// Reads the first frame of a WebP image. Animated WebP images (multiple frames)
/// will return an error.
///
/// The resulting Pix will be 32bpp with:
/// - spp=4 if the image has an alpha channel
/// - spp=3 if the image has no alpha channel
pub fn read_webp<R: Read + BufRead + Seek>(reader: R) -> IoResult<Pix> {
    let decoder = WebPDecoder::new(reader)
        .map_err(|e| IoError::DecodeError(format!("WebP decode error: {}", e)))?;

    // Check for animated WebP
    if decoder.is_animated() {
        return Err(IoError::UnsupportedFormat(
            "animated WebP not supported".to_string(),
        ));
    }

    let (width, height) = decoder.dimensions();
    let has_alpha = decoder.has_alpha();

    // Determine output buffer size
    let buffer_size = decoder.output_buffer_size().ok_or_else(|| {
        IoError::DecodeError("failed to determine output buffer size".to_string())
    })?;

    // Read image data
    let mut buffer = vec![0u8; buffer_size];
    let mut decoder = decoder;
    decoder
        .read_image(&mut buffer)
        .map_err(|e| IoError::DecodeError(format!("WebP read error: {}", e)))?;

    // Create 32bpp Pix
    let pix = Pix::new(width, height, PixelDepth::Bit32)?;
    let mut pix_mut = pix.try_into_mut().unwrap();

    // Set spp based on alpha channel
    if has_alpha {
        pix_mut.set_spp(4);
    } else {
        pix_mut.set_spp(3);
    }

    // Convert pixel data
    // WebP output is either RGB8 or RGBA8
    if has_alpha {
        // RGBA format: 4 bytes per pixel
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let r = buffer[idx];
                let g = buffer[idx + 1];
                let b = buffer[idx + 2];
                let a = buffer[idx + 3];
                // Pix stores RGBA in 32-bit word (R is MSB, A is LSB on big-endian)
                let pixel = compose_rgba(r, g, b, a);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
    } else {
        // RGB format: 3 bytes per pixel
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                let r = buffer[idx];
                let g = buffer[idx + 1];
                let b = buffer[idx + 2];
                // Set alpha to fully opaque
                let pixel = compose_rgba(r, g, b, 255);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
    }

    Ok(pix_mut.into())
}

/// Write a WebP image
///
/// Writes a Pix as a WebP image using lossless compression.
///
/// # Supported depths
/// - 32 bpp: Written directly as RGBA
/// - 1/2/4/8/16 bpp: Converted to 32bpp before encoding
///
/// # Notes
/// Currently, only lossless encoding is supported by the underlying library.
pub fn write_webp<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    write_webp_with_options(pix, writer, &WebPOptions::default())
}

/// WebP encoding options
#[derive(Debug, Clone)]
pub struct WebPOptions {
    /// Use predictor transform (improves compression for lossless encoding)
    pub use_predictor_transform: bool,
}

impl Default for WebPOptions {
    fn default() -> Self {
        Self {
            use_predictor_transform: true,
        }
    }
}

/// Write a WebP image with options
///
/// Writes a Pix as a WebP image with the specified options.
pub fn write_webp_with_options<W: Write>(
    pix: &Pix,
    writer: W,
    options: &WebPOptions,
) -> IoResult<()> {
    let (write_pix, has_alpha) = prepare_pix_for_webp(pix)?;

    let width = write_pix.width();
    let height = write_pix.height();

    // Build RGBA/RGB buffer
    let (buffer, color_type) = if has_alpha {
        // RGBA format
        let mut buffer = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let pixel = write_pix.get_pixel(x, y).unwrap_or(0);
                let (r, g, b, a) = decompose_rgba(pixel);
                buffer.push(r);
                buffer.push(g);
                buffer.push(b);
                buffer.push(a);
            }
        }
        (buffer, ColorType::Rgba8)
    } else {
        // RGB format
        let mut buffer = Vec::with_capacity((width * height * 3) as usize);
        for y in 0..height {
            for x in 0..width {
                let pixel = write_pix.get_pixel(x, y).unwrap_or(0);
                let (r, g, b, _) = decompose_rgba(pixel);
                buffer.push(r);
                buffer.push(g);
                buffer.push(b);
            }
        }
        (buffer, ColorType::Rgb8)
    };

    // Create encoder with options
    let mut encoder = WebPEncoder::new(writer);

    // EncoderParams is non-exhaustive, so we use Default and modify
    let mut params = image_webp::EncoderParams::default();
    params.use_predictor_transform = options.use_predictor_transform;
    encoder.set_params(params);

    // Encode
    encoder
        .encode(&buffer, width, height, color_type)
        .map_err(|e| IoError::EncodeError(format!("WebP encode error: {}", e)))?;

    Ok(())
}

/// Prepare pix for WebP output
///
/// Converts the input pix to 32bpp format suitable for WebP encoding.
/// Returns the converted pix and whether it has alpha channel.
fn prepare_pix_for_webp(pix: &Pix) -> IoResult<(Pix, bool)> {
    match pix.depth() {
        PixelDepth::Bit32 => {
            // Check if it has colormap (shouldn't happen for 32bpp, but handle it)
            if pix.has_colormap() {
                let converted = convert_colormapped_to_32bpp(pix)?;
                Ok((converted, false))
            } else {
                // Check spp for alpha
                let has_alpha = pix.spp() == 4;
                // Clone the pix
                let cloned = clone_pix_32bpp(pix)?;
                Ok((cloned, has_alpha))
            }
        }
        PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 => {
            if pix.has_colormap() {
                let converted = convert_colormapped_to_32bpp(pix)?;
                Ok((converted, false))
            } else {
                // Grayscale - convert to 32bpp
                let converted = convert_grayscale_to_32bpp(pix)?;
                Ok((converted, false))
            }
        }
        PixelDepth::Bit16 => {
            // 16bpp grayscale - convert to 32bpp
            let converted = convert_16bpp_to_32bpp(pix)?;
            Ok((converted, false))
        }
    }
}

/// Convert colormapped pix to 32bpp RGB
fn convert_colormapped_to_32bpp(pix: &Pix) -> IoResult<Pix> {
    let cmap = pix
        .colormap()
        .ok_or_else(|| IoError::InvalidData("expected colormap".to_string()))?;

    let new_pix = Pix::new(pix.width(), pix.height(), PixelDepth::Bit32)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();
    new_mut.set_spp(3);

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(idx) = pix.get_pixel(x, y)
                && let Some((r, g, b)) = cmap.get_rgb(idx as usize)
            {
                let pixel = compose_rgba(r, g, b, 255);
                new_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
    }

    Ok(new_mut.into())
}

/// Convert grayscale pix to 32bpp RGB
fn convert_grayscale_to_32bpp(pix: &Pix) -> IoResult<Pix> {
    let new_pix = Pix::new(pix.width(), pix.height(), PixelDepth::Bit32)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();
    new_mut.set_spp(3);

    let max_val = match pix.depth() {
        PixelDepth::Bit1 => 1,
        PixelDepth::Bit2 => 3,
        PixelDepth::Bit4 => 15,
        PixelDepth::Bit8 => 255,
        _ => return Err(IoError::UnsupportedFormat("unsupported depth".to_string())),
    };

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y) {
                // Scale to 0-255
                let gray = ((val * 255) / max_val) as u8;
                let pixel = compose_rgba(gray, gray, gray, 255);
                new_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
    }

    Ok(new_mut.into())
}

/// Convert 16bpp grayscale to 32bpp RGB
fn convert_16bpp_to_32bpp(pix: &Pix) -> IoResult<Pix> {
    let new_pix = Pix::new(pix.width(), pix.height(), PixelDepth::Bit32)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();
    new_mut.set_spp(3);

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val16) = pix.get_pixel(x, y) {
                // Scale 16-bit to 8-bit
                let gray = (val16 >> 8) as u8;
                let pixel = compose_rgba(gray, gray, gray, 255);
                new_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
    }

    Ok(new_mut.into())
}

/// Clone a 32bpp pix
fn clone_pix_32bpp(pix: &Pix) -> IoResult<Pix> {
    let new_pix = Pix::new(pix.width(), pix.height(), PixelDepth::Bit32)?;
    let mut new_mut = new_pix.try_into_mut().unwrap();
    new_mut.set_spp(pix.spp());

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y) {
                new_mut.set_pixel_unchecked(x, y, val);
            }
        }
    }

    Ok(new_mut.into())
}

/// Compose RGBA values into a 32-bit pixel value
///
/// Pix stores pixels in a host-dependent manner:
/// - The 32-bit word is stored in native endianness
/// - Conceptually: R is in the most significant byte, A in the least significant
#[inline]
fn compose_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Decompose a 32-bit pixel value into RGBA components
#[inline]
fn decompose_rgba(pixel: u32) -> (u8, u8, u8, u8) {
    let r = ((pixel >> 24) & 0xFF) as u8;
    let g = ((pixel >> 16) & 0xFF) as u8;
    let b = ((pixel >> 8) & 0xFF) as u8;
    let a = (pixel & 0xFF) as u8;
    (r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_pix_32bpp() -> Pix {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(3);

        // Fill with a gradient pattern
        for y in 0..10 {
            for x in 0..10 {
                let r = (x * 25) as u8;
                let g = (y * 25) as u8;
                let b = 128u8;
                let pixel = compose_rgba(r, g, b, 255);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    fn create_test_pix_with_alpha() -> Pix {
        let pix = Pix::new(8, 8, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(4);

        // Fill with pattern including alpha
        for y in 0..8 {
            for x in 0..8 {
                let r = (x * 32) as u8;
                let g = (y * 32) as u8;
                let b = 100u8;
                let a = if (x + y) % 2 == 0 { 255 } else { 128 };
                let pixel = compose_rgba(r, g, b, a);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_webp_roundtrip_rgb() {
        let pix = create_test_pix_32bpp();

        let mut buffer = Vec::new();
        write_webp(&pix, &mut buffer).unwrap();

        // Check WebP signature
        assert!(buffer.len() > 12);
        assert_eq!(&buffer[0..4], b"RIFF");
        assert_eq!(&buffer[8..12], b"WEBP");

        let cursor = Cursor::new(buffer);
        let pix2 = read_webp(cursor).unwrap();

        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert_eq!(pix2.depth(), PixelDepth::Bit32);

        // Verify pixel values (lossless should be exact)
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    pix2.get_pixel(x, y),
                    pix.get_pixel(x, y),
                    "mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_webp_roundtrip_rgba() {
        let pix = create_test_pix_with_alpha();

        let mut buffer = Vec::new();
        write_webp(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_webp(cursor).unwrap();

        assert_eq!(pix2.width(), 8);
        assert_eq!(pix2.height(), 8);
        assert_eq!(pix2.spp(), 4);

        // Verify pixel values (lossless should be exact)
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    pix2.get_pixel(x, y),
                    pix.get_pixel(x, y),
                    "mismatch at ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_webp_grayscale_conversion() {
        // Test 8bpp grayscale conversion
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..4 {
            for x in 0..4 {
                let val = (x + y) * 32;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_webp(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_webp(cursor).unwrap();

        assert_eq!(pix2.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_webp_1bpp_conversion() {
        // Test 1bpp conversion
        let pix = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Checkerboard pattern
        for y in 0..16 {
            for x in 0..16 {
                let val = (x + y) % 2;
                pix_mut.set_pixel(x, y, val).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_webp(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_webp(cursor).unwrap();

        assert_eq!(pix2.depth(), PixelDepth::Bit32);
    }

    #[test]
    fn test_compose_decompose_rgba() {
        let r = 100u8;
        let g = 150u8;
        let b = 200u8;
        let a = 255u8;

        let pixel = compose_rgba(r, g, b, a);
        let (r2, g2, b2, a2) = decompose_rgba(pixel);

        assert_eq!(r, r2);
        assert_eq!(g, g2);
        assert_eq!(b, b2);
        assert_eq!(a, a2);
    }

    #[test]
    fn test_webp_options() {
        let pix = create_test_pix_32bpp();

        let options = WebPOptions {
            use_predictor_transform: false,
        };

        let mut buffer = Vec::new();
        write_webp_with_options(&pix, &mut buffer, &options).unwrap();

        // Should still produce valid WebP
        assert!(buffer.len() > 12);
        assert_eq!(&buffer[0..4], b"RIFF");
    }
}
