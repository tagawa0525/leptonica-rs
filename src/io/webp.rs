//! WebP image format support
//!
//! Provides reading and writing support for WebP images.
//! Animated WebP images (multiple frames) are not supported.
//!
//! # Notes
//!
//! - Reading: Supports both lossy and lossless WebP images
//! - Writing: Currently only lossless encoding is supported by the underlying library

use crate::core::{ImageFormat, Pix, PixelDepth, pixel};
use crate::io::{IoError, IoResult, header::ImageHeader};
use image_webp::{ColorType, WebPDecoder, WebPEncoder};
use std::io::{BufRead, Read, Seek, Write};

/// Read WebP header metadata without decoding pixel data
pub fn read_header_webp(data: &[u8]) -> IoResult<ImageHeader> {
    let cursor = std::io::Cursor::new(data);
    let decoder = WebPDecoder::new(cursor)
        .map_err(|e| IoError::DecodeError(format!("WebP decode error: {}", e)))?;

    let (width, height) = decoder.dimensions();
    let spp: u32 = if decoder.has_alpha() { 4 } else { 3 };

    Ok(ImageHeader {
        width,
        height,
        depth: 32,
        bps: 8,
        spp,
        has_colormap: false,
        num_colors: 0,
        format: ImageFormat::WebP,
        x_resolution: None,
        y_resolution: None,
    })
}

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
                let pixel = pixel::compose_rgba(r, g, b, a);
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
                let pixel = pixel::compose_rgba(r, g, b, 255);
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
                let (r, g, b, a) = pixel::extract_rgba(pixel);
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
                let (r, g, b, _) = pixel::extract_rgba(pixel);
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
                let pixel = pixel::compose_rgba(r, g, b, 255);
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
                let pixel = pixel::compose_rgba(gray, gray, gray, 255);
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
                let pixel = pixel::compose_rgba(gray, gray, gray, 255);
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

/// Options for animated WebP encoding
#[derive(Debug, Clone)]
pub struct WebPAnimOptions {
    /// Loop count (0 = infinite)
    pub loop_count: u32,
    /// Duration per frame in milliseconds
    pub duration_ms: u32,
    /// Quality (0-100, for lossy mode)
    pub quality: u32,
    /// Use lossless encoding
    pub lossless: bool,
}

impl Default for WebPAnimOptions {
    fn default() -> Self {
        Self {
            loop_count: 0,
            duration_ms: 100,
            quality: 75,
            lossless: true,
        }
    }
}

/// Write animated WebP to memory from a Pixa
///
/// Each Pix in the Pixa becomes a frame. All frames must have the same dimensions
/// (the first frame's dimensions are used as the canvas size).
///
/// # See also
/// C Leptonica: `pixaWriteMemWebPAnim()` in `webpanimio.c`
pub fn write_webp_anim_mem(
    pixa: &crate::core::Pixa,
    options: &WebPAnimOptions,
) -> IoResult<Vec<u8>> {
    if pixa.is_empty() {
        return Err(IoError::InvalidData("pixa is empty".to_string()));
    }

    // Get canvas dimensions from first frame
    let first = pixa.get(0).unwrap();
    let canvas_w = first.width();
    let canvas_h = first.height();

    // Encode each frame as a standalone WebP image
    let mut frame_data: Vec<Vec<u8>> = Vec::new();
    for i in 0..pixa.len() {
        let pix = pixa.get(i).unwrap();
        let mut buf = Vec::new();
        write_webp(pix, &mut buf)?;
        frame_data.push(buf);
    }

    // Build animated WebP (RIFF container with ANIM + ANMF chunks)
    build_animated_webp(
        &frame_data,
        canvas_w,
        canvas_h,
        options.loop_count,
        options.duration_ms,
    )
}

/// Write animated WebP to a writer
///
/// # See also
/// C Leptonica: `pixaWriteStreamWebPAnim()` in `webpanimio.c`
pub fn write_webp_anim<W: Write>(
    pixa: &crate::core::Pixa,
    mut writer: W,
    options: &WebPAnimOptions,
) -> IoResult<()> {
    let data = write_webp_anim_mem(pixa, options)?;
    writer.write_all(&data).map_err(IoError::Io)?;
    Ok(())
}

/// Write animated WebP to a file
///
/// # See also
/// C Leptonica: `pixaWriteWebPAnim()` in `webpanimio.c`
pub fn write_webp_anim_file(
    pixa: &crate::core::Pixa,
    path: impl AsRef<std::path::Path>,
    options: &WebPAnimOptions,
) -> IoResult<()> {
    let data = write_webp_anim_mem(pixa, options)?;
    std::fs::write(path, &data).map_err(IoError::Io)?;
    Ok(())
}

/// Build an animated WebP file from individual WebP frame data
///
/// Uses the extended WebP format (VP8X + ANIM + ANMF chunks).
fn build_animated_webp(
    frames: &[Vec<u8>],
    canvas_w: u32,
    canvas_h: u32,
    loop_count: u32,
    duration_ms: u32,
) -> IoResult<Vec<u8>> {
    let mut buf = Vec::new();

    // Placeholder for RIFF header (will be filled at the end)
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&[0u8; 4]); // file size placeholder
    buf.extend_from_slice(b"WEBP");

    // VP8X chunk: extended format flags
    buf.extend_from_slice(b"VP8X");
    buf.extend_from_slice(&10u32.to_le_bytes()); // chunk size
    // Flags: bit 1 = animation
    buf.extend_from_slice(&0x02u32.to_le_bytes()); // flags
    // Canvas width - 1 (24 bits) and height - 1 (24 bits)
    let w_minus_1 = canvas_w.saturating_sub(1);
    let h_minus_1 = canvas_h.saturating_sub(1);
    buf.push((w_minus_1 & 0xFF) as u8);
    buf.push(((w_minus_1 >> 8) & 0xFF) as u8);
    buf.push(((w_minus_1 >> 16) & 0xFF) as u8);
    buf.push((h_minus_1 & 0xFF) as u8);
    buf.push(((h_minus_1 >> 8) & 0xFF) as u8);
    buf.push(((h_minus_1 >> 16) & 0xFF) as u8);

    // ANIM chunk: animation parameters
    buf.extend_from_slice(b"ANIM");
    buf.extend_from_slice(&6u32.to_le_bytes()); // chunk size
    // Background color (BGRA) = transparent
    buf.extend_from_slice(&[0u8; 4]);
    // Loop count
    buf.extend_from_slice(&(loop_count as u16).to_le_bytes());

    // ANMF chunks: one per frame
    for frame in frames {
        // Extract the VP8/VP8L bitstream from the WebP container
        let bitstream = extract_webp_bitstream(frame)?;
        let chunk_type = &bitstream.0;
        let bitstream_data = &bitstream.1;

        // ANMF payload: 16 bytes header + sub-chunk
        let sub_chunk_size = 8 + bitstream_data.len(); // chunk_type(4) + size(4) + data
        let anmf_payload_size = 16 + sub_chunk_size;
        // Pad to even
        let padded_sub = if bitstream_data.len() % 2 != 0 { 1 } else { 0 };

        buf.extend_from_slice(b"ANMF");
        buf.extend_from_slice(&((anmf_payload_size + padded_sub) as u32).to_le_bytes());

        // Frame X offset (24 bits, divided by 2)
        buf.extend_from_slice(&[0u8; 3]);
        // Frame Y offset (24 bits, divided by 2)
        buf.extend_from_slice(&[0u8; 3]);

        // Frame width - 1 (24 bits)
        buf.push((w_minus_1 & 0xFF) as u8);
        buf.push(((w_minus_1 >> 8) & 0xFF) as u8);
        buf.push(((w_minus_1 >> 16) & 0xFF) as u8);

        // Frame height - 1 (24 bits)
        buf.push((h_minus_1 & 0xFF) as u8);
        buf.push(((h_minus_1 >> 8) & 0xFF) as u8);
        buf.push(((h_minus_1 >> 16) & 0xFF) as u8);

        // Duration (24 bits)
        buf.push((duration_ms & 0xFF) as u8);
        buf.push(((duration_ms >> 8) & 0xFF) as u8);
        buf.push(((duration_ms >> 16) & 0xFF) as u8);

        // Flags: disposal=0, blending=0
        buf.push(0);

        // Sub-chunk (VP8 or VP8L)
        buf.extend_from_slice(chunk_type);
        buf.extend_from_slice(&(bitstream_data.len() as u32).to_le_bytes());
        buf.extend_from_slice(bitstream_data);
        if padded_sub > 0 {
            buf.push(0);
        }
    }

    // Fix RIFF file size
    let file_size = (buf.len() - 8) as u32;
    buf[4..8].copy_from_slice(&file_size.to_le_bytes());

    Ok(buf)
}

/// Extract the VP8/VP8L bitstream data from a WebP container
fn extract_webp_bitstream(webp_data: &[u8]) -> IoResult<([u8; 4], Vec<u8>)> {
    if webp_data.len() < 12 || &webp_data[0..4] != b"RIFF" || &webp_data[8..12] != b"WEBP" {
        return Err(IoError::InvalidData("invalid WebP data".to_string()));
    }

    let mut pos = 12;
    while pos + 8 <= webp_data.len() {
        let chunk_id: [u8; 4] = webp_data[pos..pos + 4].try_into().unwrap();
        let chunk_size =
            u32::from_le_bytes(webp_data[pos + 4..pos + 8].try_into().unwrap()) as usize;

        if &chunk_id == b"VP8 " || &chunk_id == b"VP8L" {
            let data_end = (pos + 8 + chunk_size).min(webp_data.len());
            return Ok((chunk_id, webp_data[pos + 8..data_end].to_vec()));
        }

        // Move to next chunk (padded to even boundary)
        pos += 8 + chunk_size + (chunk_size % 2);
    }

    Err(IoError::InvalidData(
        "no VP8/VP8L chunk found in WebP data".to_string(),
    ))
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
                let pixel = pixel::compose_rgba(r, g, b, 255);
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
                let pixel = pixel::compose_rgba(r, g, b, a);
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

        let pixel = pixel::compose_rgba(r, g, b, a);
        let (r2, g2, b2, a2) = pixel::extract_rgba(pixel);

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
