//! JPEG image format support
//!
//! Reading and writing JPEG images using the jpeg-decoder and jpeg-encoder crates.

use crate::{IoError, IoResult};
use jpeg_decoder::{Decoder, PixelFormat};
use leptonica_core::{Pix, PixelDepth, color, pix::RemoveColormapTarget};
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
pub fn write_jpeg<W: Write>(pix: &Pix, writer: W, options: &JpegOptions) -> IoResult<()> {
    let quality = options.quality.clamp(1, 100);

    // Convert pix to a form suitable for JPEG encoding.
    // Following C version logic: remove colormap based on source content,
    // then ensure depth is 8 (grayscale) or 32 (RGB).
    let pix = if pix.has_colormap() {
        pix.remove_colormap(RemoveColormapTarget::BasedOnSrc)
            .map_err(|e| IoError::EncodeError(format!("colormap removal failed: {}", e)))?
    } else {
        match pix.depth() {
            PixelDepth::Bit8 | PixelDepth::Bit32 => pix.deep_clone(),
            PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit16 => pix
                .convert_to_8()
                .map_err(|e| IoError::EncodeError(format!("depth conversion failed: {}", e)))?,
        }
    };

    let w = pix.width();
    let h = pix.height();

    // jpeg-encoder uses u16 for dimensions
    if w > u16::MAX as u32 || h > u16::MAX as u32 {
        return Err(IoError::EncodeError(format!(
            "image dimensions {}x{} exceed JPEG maximum of 65535",
            w, h
        )));
    }

    let encoder = jpeg_encoder::Encoder::new(writer, quality);

    match pix.depth() {
        PixelDepth::Bit8 => {
            // Grayscale: extract pixel values into a byte buffer
            let mut data = vec![0u8; (w * h) as usize];
            for y in 0..h {
                for x in 0..w {
                    data[(y * w + x) as usize] = pix.get_pixel_unchecked(x, y) as u8;
                }
            }
            encoder
                .encode(&data, w as u16, h as u16, jpeg_encoder::ColorType::Luma)
                .map_err(|e| IoError::EncodeError(format!("JPEG encode error: {}", e)))?;
        }
        PixelDepth::Bit32 => {
            // Validate spp: JPEG supports only RGB (spp=3) or RGBA (spp=4, alpha ignored).
            let spp = pix.spp();
            if spp != 3 && spp != 4 {
                return Err(IoError::EncodeError(format!(
                    "32bpp image has spp={}, expected 3 (RGB) or 4 (RGBA)",
                    spp
                )));
            }
            // RGB: extract R, G, B channels (alpha ignored)
            let mut data = vec![0u8; (w * h * 3) as usize];
            for y in 0..h {
                for x in 0..w {
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b) = color::extract_rgb(pixel);
                    let idx = ((y * w + x) * 3) as usize;
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                }
            }
            encoder
                .encode(&data, w as u16, h as u16, jpeg_encoder::ColorType::Rgb)
                .map_err(|e| IoError::EncodeError(format!("JPEG encode error: {}", e)))?;
        }
        _ => {
            return Err(IoError::EncodeError(format!(
                "unexpected depth {} after conversion",
                pix.depth().bits()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
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
