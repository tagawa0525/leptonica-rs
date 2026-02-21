//! JPEG image format support
//!
//! Reading and writing JPEG images using the jpeg-decoder and jpeg-encoder crates.

use crate::{IoError, IoResult};
use jpeg_decoder::{Decoder, PixelFormat};
use leptonica_core::{Pix, PixelDepth, color};
use std::io::{Read, Write};

/// Options for JPEG encoding
pub struct JpegOptions {
    /// Quality setting (1-100, default 75)
    pub quality: u8,
}

impl Default for JpegOptions {
    fn default() -> Self {
        Self { quality: 75 }
    }
}

/// Read a JPEG image
pub fn read_jpeg<R: Read>(reader: R) -> IoResult<Pix> {
    let mut decoder = Decoder::new(reader);

    let pixels = decoder
        .decode()
        .map_err(|e| IoError::DecodeError(format!("JPEG decode error: {}", e)))?;

    let info = decoder
        .info()
        .ok_or_else(|| IoError::InvalidData("missing JPEG info".to_string()))?;

    let width = info.width as u32;
    let height = info.height as u32;

    let (depth, spp) = match info.pixel_format {
        PixelFormat::L8 => (PixelDepth::Bit8, 1),
        PixelFormat::L16 => (PixelDepth::Bit16, 1),
        PixelFormat::RGB24 => (PixelDepth::Bit32, 3),
        PixelFormat::CMYK32 => {
            return Err(IoError::UnsupportedFormat(
                "CMYK JPEG not supported".to_string(),
            ));
        }
    };

    let pix = Pix::new(width, height, depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();
    pix_mut.set_spp(spp);

    match info.pixel_format {
        PixelFormat::L8 => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let val = pixels[idx];
                    pix_mut.set_pixel_unchecked(x, y, val as u32);
                }
            }
        }
        PixelFormat::L16 => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    let val = ((pixels[idx] as u32) << 8) | (pixels[idx + 1] as u32);
                    pix_mut.set_pixel_unchecked(x, y, val);
                }
            }
        }
        PixelFormat::RGB24 => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 3) as usize;
                    let r = pixels[idx];
                    let g = pixels[idx + 1];
                    let b = pixels[idx + 2];
                    let pixel = color::compose_rgb(r, g, b);
                    pix_mut.set_pixel_unchecked(x, y, pixel);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(pix_mut.into())
}

/// Write a Pix as JPEG
///
/// Conversion rules:
/// - **1/2/4 bpp**: converted to 8bpp grayscale
/// - **8 bpp with colormap**: colormap removed (grayscale or RGB based on content)
/// - **8 bpp grayscale**: encoded directly as grayscale
/// - **16 bpp**: converted to 8bpp grayscale (upper byte)
/// - **32 bpp (spp=3/4)**: encoded as RGB (alpha ignored)
///
/// # See also
///
/// C Leptonica: `pixWriteStreamJpeg()` in `jpegio.c`
pub fn write_jpeg<W: Write>(_pix: &Pix, _writer: W, _options: &JpegOptions) -> IoResult<()> {
    todo!("JPEG writing not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jpeg_write_grayscale_roundtrip() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..10u32 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, (x * 25) as u32);
            }
        }
        let pix: Pix = pix_mut.into();

        let mut buf = Vec::new();
        write_jpeg(&pix, &mut buf, &JpegOptions::default()).unwrap();

        // Verify JPEG magic
        assert!(buf.starts_with(&[0xFF, 0xD8, 0xFF]));

        // Read back and verify dimensions
        let pix2 = read_jpeg(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert_eq!(pix2.depth(), PixelDepth::Bit8);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jpeg_write_rgb_roundtrip() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(3);
        for y in 0..10u32 {
            for x in 0..10u32 {
                let pixel = color::compose_rgb((x * 25) as u8, (y * 25) as u8, 128);
                pix_mut.set_pixel_unchecked(x, y, pixel);
            }
        }
        let pix: Pix = pix_mut.into();

        let mut buf = Vec::new();
        write_jpeg(&pix, &mut buf, &JpegOptions::default()).unwrap();

        // Verify JPEG magic
        assert!(buf.starts_with(&[0xFF, 0xD8, 0xFF]));

        // Read back
        let pix2 = read_jpeg(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert_eq!(pix2.depth(), PixelDepth::Bit32);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jpeg_write_1bpp_converts() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let mut buf = Vec::new();
        write_jpeg(&pix, &mut buf, &JpegOptions::default()).unwrap();
        assert!(buf.starts_with(&[0xFF, 0xD8, 0xFF]));

        let pix2 = read_jpeg(Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_jpeg_quality_affects_size() {
        let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..100u32 {
            for x in 0..100u32 {
                pix_mut.set_pixel_unchecked(x, y, ((x + y) % 256) as u32);
            }
        }
        let pix: Pix = pix_mut.into();

        let mut buf_low = Vec::new();
        write_jpeg(&pix, &mut buf_low, &JpegOptions { quality: 10 }).unwrap();

        let mut buf_high = Vec::new();
        write_jpeg(&pix, &mut buf_high, &JpegOptions { quality: 95 }).unwrap();

        // Higher quality should produce larger output for non-trivial images
        assert!(buf_high.len() > buf_low.len());
    }
}
