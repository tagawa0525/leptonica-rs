//! JPEG 2000 image format support
//!
//! Provides reading support for JPEG 2000 images using the `hayro-jpeg2000` crate -
//! a pure Rust, memory-safe decoder.
//!
//! Supports both JP2 container format (.jp2) and raw J2K codestreams (.j2k, .j2c).
//!
//! # Note
//!
//! Writing is not yet supported. The openjp2 crate's low-level API requires
//! additional work to integrate properly.

use crate::{IoError, IoResult, header::ImageHeader};
use hayro_jpeg2000::{ColorSpace, DecodeSettings, Image};
use leptonica_core::{ImageFormat, Pix, PixelDepth};
use std::io::{Read, Seek};

/// Read JPEG 2000 header metadata without decoding pixel data
pub fn read_header_jp2k(data: &[u8]) -> IoResult<ImageHeader> {
    let _ = data;
    Err(IoError::UnsupportedFormat(
        "JP2K header reading not yet implemented".to_string(),
    ))
}

/// Read a JPEG 2000 image
///
/// Supports both JP2 container format (.jp2) and raw codestream format (.j2k, .j2c).
///
/// # Supported color spaces
///
/// - Grayscale -> 8bpp
/// - RGB -> 32bpp (spp=3)
/// - RGB + alpha -> 32bpp (spp=4)
/// - CMYK -> 32bpp RGB (converted)
/// - ICC-based -> 8bpp or 32bpp RGB depending on channel count
pub fn read_jp2k<R: Read + Seek>(mut reader: R) -> IoResult<Pix> {
    // Read all data into memory (hayro-jpeg2000 requires &[u8])
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|e| IoError::DecodeError(format!("Failed to read JP2K data: {}", e)))?;

    read_jp2k_mem(&data)
}

/// Read a JPEG 2000 image from memory
pub fn read_jp2k_mem(data: &[u8]) -> IoResult<Pix> {
    // Create decoder with default settings
    let settings = DecodeSettings::default();

    let image = Image::new(data, &settings)
        .map_err(|e| IoError::DecodeError(format!("JP2K parse error: {}", e)))?;

    // Get image properties from the Image struct (before decoding)
    let width = image.width();
    let height = image.height();
    let color_space = image.color_space().clone();
    let has_alpha = image.has_alpha();

    // Decode the image
    let pixels = image
        .decode()
        .map_err(|e| IoError::DecodeError(format!("JP2K decode error: {}", e)))?;

    // Create Pix based on color space
    match &color_space {
        ColorSpace::Gray => {
            let num_channels: u32 = if has_alpha { 2 } else { 1 };

            if has_alpha {
                // Gray + alpha -> 32bpp RGBA
                let pix = Pix::new(width, height, PixelDepth::Bit32)?;
                let mut pix_mut = pix.try_into_mut().unwrap();
                pix_mut.set_spp(4);

                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * num_channels) as usize;
                        let g = pixels[idx];
                        let a = pixels[idx + 1];
                        let pixel = compose_rgba(g, g, g, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
                Ok(pix_mut.into())
            } else {
                // Pure grayscale -> 8bpp
                let pix = Pix::new(width, height, PixelDepth::Bit8)?;
                let mut pix_mut = pix.try_into_mut().unwrap();
                pix_mut.set_spp(1);

                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let val = pixels[idx];
                        pix_mut.set_pixel_unchecked(x, y, val as u32);
                    }
                }
                Ok(pix_mut.into())
            }
        }
        ColorSpace::RGB => {
            let pix = Pix::new(width, height, PixelDepth::Bit32)?;
            let mut pix_mut = pix.try_into_mut().unwrap();

            if has_alpha {
                pix_mut.set_spp(4);
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let r = pixels[idx];
                        let g = pixels[idx + 1];
                        let b = pixels[idx + 2];
                        let a = pixels[idx + 3];
                        let pixel = compose_rgba(r, g, b, a);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            } else {
                pix_mut.set_spp(3);
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 3) as usize;
                        let r = pixels[idx];
                        let g = pixels[idx + 1];
                        let b = pixels[idx + 2];
                        let pixel = compose_rgba(r, g, b, 255);
                        pix_mut.set_pixel_unchecked(x, y, pixel);
                    }
                }
            }

            Ok(pix_mut.into())
        }
        ColorSpace::CMYK => {
            // CMYK is output as 4 bytes per pixel, convert to RGB
            let pix = Pix::new(width, height, PixelDepth::Bit32)?;
            let mut pix_mut = pix.try_into_mut().unwrap();
            pix_mut.set_spp(3);

            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    let c = pixels[idx] as f32 / 255.0;
                    let m = pixels[idx + 1] as f32 / 255.0;
                    let y_val = pixels[idx + 2] as f32 / 255.0;
                    let k = pixels[idx + 3] as f32 / 255.0;

                    // CMYK to RGB conversion
                    let r = (255.0 * (1.0 - c) * (1.0 - k)) as u8;
                    let g = (255.0 * (1.0 - m) * (1.0 - k)) as u8;
                    let b = (255.0 * (1.0 - y_val) * (1.0 - k)) as u8;

                    let pixel = compose_rgba(r, g, b, 255);
                    pix_mut.set_pixel_unchecked(x, y, pixel);
                }
            }

            Ok(pix_mut.into())
        }
        ColorSpace::Unknown { num_channels } => {
            // Unknown color space - treat based on channel count
            let n = *num_channels as usize;
            let total_channels = if has_alpha { n + 1 } else { n };

            if n == 1 {
                // Single channel - treat as grayscale
                if has_alpha {
                    let pix = Pix::new(width, height, PixelDepth::Bit32)?;
                    let mut pix_mut = pix.try_into_mut().unwrap();
                    pix_mut.set_spp(4);

                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let g = pixels[idx];
                            let a = pixels.get(idx + 1).copied().unwrap_or(255);
                            let pixel = compose_rgba(g, g, g, a);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                    Ok(pix_mut.into())
                } else {
                    let pix = Pix::new(width, height, PixelDepth::Bit8)?;
                    let mut pix_mut = pix.try_into_mut().unwrap();
                    pix_mut.set_spp(1);

                    for y in 0..height {
                        for x in 0..width {
                            let idx = (y * width + x) as usize;
                            let val = pixels[idx];
                            pix_mut.set_pixel_unchecked(x, y, val as u32);
                        }
                    }
                    Ok(pix_mut.into())
                }
            } else {
                // Multi-channel - treat as RGB
                let pix = Pix::new(width, height, PixelDepth::Bit32)?;
                let mut pix_mut = pix.try_into_mut().unwrap();

                if has_alpha {
                    pix_mut.set_spp(4);
                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let r = pixels.get(idx).copied().unwrap_or(0);
                            let g = pixels.get(idx + 1).copied().unwrap_or(0);
                            let b = pixels.get(idx + 2).copied().unwrap_or(0);
                            let a = pixels.get(idx + n).copied().unwrap_or(255);
                            let pixel = compose_rgba(r, g, b, a);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                } else {
                    pix_mut.set_spp(3);
                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let r = pixels.get(idx).copied().unwrap_or(0);
                            let g = pixels.get(idx + 1).copied().unwrap_or(0);
                            let b = pixels.get(idx + 2).copied().unwrap_or(0);
                            let pixel = compose_rgba(r, g, b, 255);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                }
                Ok(pix_mut.into())
            }
        }
        ColorSpace::Icc {
            num_channels,
            profile: _,
        } => {
            // ICC-based color space - treat as RGB or grayscale depending on component count
            let n = *num_channels as usize;
            let total_channels = if has_alpha { n + 1 } else { n };

            if n == 1 {
                // Single channel ICC - treat as grayscale
                if has_alpha {
                    let pix = Pix::new(width, height, PixelDepth::Bit32)?;
                    let mut pix_mut = pix.try_into_mut().unwrap();
                    pix_mut.set_spp(4);

                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let g = pixels[idx];
                            let a = pixels.get(idx + 1).copied().unwrap_or(255);
                            let pixel = compose_rgba(g, g, g, a);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                    Ok(pix_mut.into())
                } else {
                    let pix = Pix::new(width, height, PixelDepth::Bit8)?;
                    let mut pix_mut = pix.try_into_mut().unwrap();
                    pix_mut.set_spp(1);

                    for y in 0..height {
                        for x in 0..width {
                            let idx = (y * width + x) as usize;
                            let val = pixels[idx];
                            pix_mut.set_pixel_unchecked(x, y, val as u32);
                        }
                    }
                    Ok(pix_mut.into())
                }
            } else {
                // Multi-channel ICC - treat as RGB
                let pix = Pix::new(width, height, PixelDepth::Bit32)?;
                let mut pix_mut = pix.try_into_mut().unwrap();

                if has_alpha {
                    pix_mut.set_spp(4);
                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let r = pixels.get(idx).copied().unwrap_or(0);
                            let g = pixels.get(idx + 1).copied().unwrap_or(0);
                            let b = pixels.get(idx + 2).copied().unwrap_or(0);
                            let a = pixels.get(idx + n).copied().unwrap_or(255);
                            let pixel = compose_rgba(r, g, b, a);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                } else {
                    pix_mut.set_spp(3);
                    for y in 0..height {
                        for x in 0..width {
                            let idx = ((y * width + x) * total_channels as u32) as usize;
                            let r = pixels.get(idx).copied().unwrap_or(0);
                            let g = pixels.get(idx + 1).copied().unwrap_or(0);
                            let b = pixels.get(idx + 2).copied().unwrap_or(0);
                            let pixel = compose_rgba(r, g, b, 255);
                            pix_mut.set_pixel_unchecked(x, y, pixel);
                        }
                    }
                }
                Ok(pix_mut.into())
            }
        }
    }
}

/// Compose RGBA values into a 32-bit pixel value
#[inline]
fn compose_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_rgba() {
        let r = 100u8;
        let g = 150u8;
        let b = 200u8;
        let a = 255u8;

        let pixel = compose_rgba(r, g, b, a);

        assert_eq!((pixel >> 24) & 0xFF, r as u32);
        assert_eq!((pixel >> 16) & 0xFF, g as u32);
        assert_eq!((pixel >> 8) & 0xFF, b as u32);
        assert_eq!(pixel & 0xFF, a as u32);
    }
}
