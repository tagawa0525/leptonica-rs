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

use crate::core::{ImageFormat, Pix, PixelDepth};
use crate::io::{IoError, IoResult, header::ImageHeader};
use hayro_jpeg2000::{ColorSpace, DecodeSettings, Image};
use std::io::{Read, Seek};

/// Read JPEG 2000 header metadata without decoding pixel data
pub fn read_header_jp2k(data: &[u8]) -> IoResult<ImageHeader> {
    let settings = DecodeSettings::default();
    let image = Image::new(data, &settings)
        .map_err(|e| IoError::DecodeError(format!("JP2K decode error: {}", e)))?;

    let width = image.width();
    let height = image.height();
    let has_alpha = image.has_alpha();
    let spp: u32 = match image.color_space() {
        ColorSpace::Gray => {
            if has_alpha {
                4
            } else {
                1
            }
        }
        _ => {
            if has_alpha {
                4
            } else {
                3
            }
        }
    };
    let depth: u32 = if spp == 1 { 8 } else { 32 };

    Ok(ImageHeader {
        width,
        height,
        depth,
        bps: 8,
        spp,
        has_colormap: false,
        num_colors: 0,
        format: ImageFormat::Jp2,
        x_resolution: None,
        y_resolution: None,
    })
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
    let settings = DecodeSettings::default();
    let image = Image::new(data, &settings)
        .map_err(|e| IoError::DecodeError(format!("JP2K parse error: {}", e)))?;
    decode_image_to_pix(image)
}

/// Convert a hayro `Image` into a [`Pix`], dispatching on color space.
fn decode_image_to_pix(image: Image<'_>) -> IoResult<Pix> {
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

/// Options for JPEG 2000 writing
#[derive(Debug, Clone, Default)]
pub struct Jp2kWriteOptions {
    /// Quality factor (0 = default ~34 SNR, 100 = lossless)
    pub quality: u32,
}

/// Write a Pix as JPEG 2000 to memory
///
/// # Note
/// JPEG 2000 encoding is not currently supported. A pure-Rust JP2K encoder
/// is not yet available. Returns `UnsupportedFormat` error.
///
/// # See also
/// C Leptonica: `pixWriteMemJp2k()` in `jp2kio.c`
pub fn write_jp2k_mem(_pix: &Pix, _options: &Jp2kWriteOptions) -> IoResult<Vec<u8>> {
    Err(IoError::UnsupportedFormat(
        "JP2K writing not yet supported: no pure-Rust encoder available".to_string(),
    ))
}

/// Write a Pix as JPEG 2000 to a writer
///
/// # See also
/// C Leptonica: `pixWriteStreamJp2k()` in `jp2kio.c`
pub fn write_jp2k<W: std::io::Write>(
    _pix: &Pix,
    _writer: W,
    _options: &Jp2kWriteOptions,
) -> IoResult<()> {
    Err(IoError::UnsupportedFormat(
        "JP2K writing not yet supported: no pure-Rust encoder available".to_string(),
    ))
}

/// Read a JPEG 2000 image from memory at a reduced resolution.
///
/// `scale_denom` is the reciprocal of the scaling factor, e.g. `2` requests
/// half-size and `4` requests quarter-size. `scale_denom == 0` is rejected.
/// `scale_denom == 1` is equivalent to [`read_jp2k_mem`].
///
/// Implementation: passes `target_resolution` to hayro-jpeg2000's
/// `DecodeSettings`. The hint is honoured when the codec's wavelet pyramid
/// allows it (typically powers of two); for other denominators the decoder
/// may return a slightly different size.
///
/// C Leptonica equivalent: `pixReadMemJp2k(data, size, reduction, ...)` with
/// `box == NULL`.
pub fn read_jp2k_scaled_mem(data: &[u8], scale_denom: u32) -> IoResult<Pix> {
    if scale_denom == 0 {
        return Err(IoError::InvalidData("scale_denom must be >= 1".to_string()));
    }

    // Probe the original dimensions to compute the target resolution.
    let header_settings = DecodeSettings::default();
    let probe = Image::new(data, &header_settings)
        .map_err(|e| IoError::DecodeError(format!("JP2K parse error: {}", e)))?;
    let target_w = (probe.width() / scale_denom).max(1);
    let target_h = (probe.height() / scale_denom).max(1);
    drop(probe);

    let settings = DecodeSettings {
        target_resolution: Some((target_w, target_h)),
        ..DecodeSettings::default()
    };
    let image = Image::new(data, &settings)
        .map_err(|e| IoError::DecodeError(format!("JP2K parse error: {}", e)))?;

    decode_image_to_pix(image)
}

/// Read a JPEG 2000 image from a stream at a reduced resolution.
pub fn read_jp2k_scaled<R: Read + Seek>(mut reader: R, scale_denom: u32) -> IoResult<Pix> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    read_jp2k_scaled_mem(&data, scale_denom)
}

/// Read a JPEG 2000 image from memory and crop to `box_`.
///
/// hayro-jpeg2000 does not currently expose a tile-based partial decoder, so
/// this implementation decodes the whole image then clips with
/// [`Pix::clip_rectangle`]. For very large images this is wasteful — consider
/// adding a tile-aware decoder when one becomes available.
///
/// C Leptonica equivalent: `pixReadMemJp2k(data, size, 1, &box, ...)`.
pub fn read_jp2k_cropped_mem(data: &[u8], box_: &crate::core::Box) -> IoResult<Pix> {
    // Validate the crop box first so degenerate parameters are rejected
    // before we spend time parsing/decoding the JP2K stream.
    if box_.x < 0 || box_.y < 0 || box_.w <= 0 || box_.h <= 0 {
        return Err(IoError::InvalidData(format!("invalid crop box {:?}", box_)));
    }
    let pix = read_jp2k_mem(data)?;
    pix.clip_rectangle(box_.x as u32, box_.y as u32, box_.w as u32, box_.h as u32)
        .map_err(|e| IoError::DecodeError(format!("clip_rectangle failed: {e}")))
}

/// Read a JPEG 2000 image from a stream and crop to `box_`.
pub fn read_jp2k_cropped<R: Read + Seek>(mut reader: R, box_: &crate::core::Box) -> IoResult<Pix> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    read_jp2k_cropped_mem(&data, box_)
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
