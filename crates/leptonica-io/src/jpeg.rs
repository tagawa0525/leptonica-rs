//! JPEG image format support
//!
//! Read-only support for JPEG images using the jpeg-decoder crate.

use crate::{IoError, IoResult};
use jpeg_decoder::{Decoder, PixelFormat};
use leptonica_core::{Pix, PixelDepth, color};
use std::io::Read;

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
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
        }
        PixelFormat::L16 => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    let val = ((pixels[idx] as u32) << 8) | (pixels[idx + 1] as u32);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val) };
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
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(pix_mut.into())
}

#[cfg(test)]
mod tests {
    // JPEG tests require actual JPEG files, which we don't have in the test environment
    // Real tests would be integration tests with test images
}
