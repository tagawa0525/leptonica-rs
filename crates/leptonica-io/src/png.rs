//! PNG image format support

use crate::{IoError, IoResult};
use leptonica_core::{Pix, PixColormap, PixelDepth, color};
use png::{BitDepth, ColorType, Decoder, Encoder};
use std::io::{BufRead, Seek, Write};

/// Read a PNG image
pub fn read_png<R: BufRead + Seek>(reader: R) -> IoResult<Pix> {
    let decoder = Decoder::new(reader);
    let mut reader = decoder
        .read_info()
        .map_err(|e| IoError::DecodeError(format!("PNG decode error: {}", e)))?;

    let info = reader.info();
    let width = info.width;
    let height = info.height;
    let color_type = info.color_type;
    let bit_depth = info.bit_depth;

    // Determine pixel depth
    let (pix_depth, spp) = match (color_type, bit_depth) {
        (ColorType::Grayscale, BitDepth::One) => (PixelDepth::Bit1, 1),
        (ColorType::Grayscale, BitDepth::Two) => (PixelDepth::Bit2, 1),
        (ColorType::Grayscale, BitDepth::Four) => (PixelDepth::Bit4, 1),
        (ColorType::Grayscale, BitDepth::Eight) => (PixelDepth::Bit8, 1),
        (ColorType::Grayscale, BitDepth::Sixteen) => (PixelDepth::Bit16, 1),
        (ColorType::GrayscaleAlpha, _) => (PixelDepth::Bit32, 2),
        (ColorType::Rgb, _) => (PixelDepth::Bit32, 3),
        (ColorType::Rgba, _) => (PixelDepth::Bit32, 4),
        (ColorType::Indexed, BitDepth::One) => (PixelDepth::Bit1, 1),
        (ColorType::Indexed, BitDepth::Two) => (PixelDepth::Bit2, 1),
        (ColorType::Indexed, BitDepth::Four) => (PixelDepth::Bit4, 1),
        (ColorType::Indexed, BitDepth::Eight) => (PixelDepth::Bit8, 1),
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "unsupported PNG format: {:?} {:?}",
                color_type, bit_depth
            )));
        }
    };

    // Read image data
    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| IoError::DecodeError("failed to get output buffer size".to_string()))?;
    let mut buf = vec![0; buf_size];
    let output_info = reader
        .next_frame(&mut buf)
        .map_err(|e| IoError::DecodeError(format!("PNG frame error: {}", e)))?;

    let pix = Pix::new(width, height, pix_depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();
    pix_mut.set_spp(spp);

    // Handle palette if present
    if color_type == ColorType::Indexed
        && let Some(palette) = reader.info().palette.as_ref()
    {
        let mut cmap = PixColormap::new(bit_depth as u32).map_err(IoError::Core)?;

        let palette_bytes: &[u8] = palette;
        for chunk in palette_bytes.chunks(3) {
            if chunk.len() == 3 {
                cmap.add_rgb(chunk[0], chunk[1], chunk[2])
                    .map_err(IoError::Core)?;
            }
        }
        pix_mut.set_colormap(Some(cmap)).map_err(IoError::Core)?;
    }

    // Convert to PIX format
    let bytes_per_row = output_info.line_size;
    let data = &buf[..output_info.buffer_size()];

    match (color_type, bit_depth) {
        (ColorType::Grayscale, BitDepth::One) | (ColorType::Indexed, BitDepth::One) => {
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let byte_idx = row_start + (x / 8) as usize;
                    let bit_idx = 7 - (x % 8);
                    let val = (data[byte_idx] >> bit_idx) & 1;
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
        }
        (ColorType::Grayscale, BitDepth::Two) | (ColorType::Indexed, BitDepth::Two) => {
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let byte_idx = row_start + (x / 4) as usize;
                    let shift = 6 - ((x % 4) * 2);
                    let val = (data[byte_idx] >> shift) & 3;
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
        }
        (ColorType::Grayscale, BitDepth::Four) | (ColorType::Indexed, BitDepth::Four) => {
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let byte_idx = row_start + (x / 2) as usize;
                    let val = if x % 2 == 0 {
                        (data[byte_idx] >> 4) & 0xF
                    } else {
                        data[byte_idx] & 0xF
                    };
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
        }
        (ColorType::Grayscale, BitDepth::Eight) | (ColorType::Indexed, BitDepth::Eight) => {
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let val = data[row_start + x as usize];
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
        }
        (ColorType::Grayscale, BitDepth::Sixteen) => {
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let idx = row_start + (x as usize * 2);
                    let val = ((data[idx] as u32) << 8) | (data[idx + 1] as u32);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val) };
                }
            }
        }
        (ColorType::GrayscaleAlpha, _) => {
            let samples = if bit_depth == BitDepth::Sixteen { 4 } else { 2 };
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let idx = row_start + (x as usize * samples);
                    let (g, a) = if bit_depth == BitDepth::Sixteen {
                        (data[idx], data[idx + 2])
                    } else {
                        (data[idx], data[idx + 1])
                    };
                    let pixel = color::compose_rgba(g, g, g, a);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
        }
        (ColorType::Rgb, _) => {
            let samples = if bit_depth == BitDepth::Sixteen { 6 } else { 3 };
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let idx = row_start + (x as usize * samples);
                    let (r, g, b) = if bit_depth == BitDepth::Sixteen {
                        (data[idx], data[idx + 2], data[idx + 4])
                    } else {
                        (data[idx], data[idx + 1], data[idx + 2])
                    };
                    let pixel = color::compose_rgb(r, g, b);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
        }
        (ColorType::Rgba, _) => {
            let samples = if bit_depth == BitDepth::Sixteen { 8 } else { 4 };
            for y in 0..height {
                let row_start = y as usize * bytes_per_row;
                for x in 0..width {
                    let idx = row_start + (x as usize * samples);
                    let (r, g, b, a) = if bit_depth == BitDepth::Sixteen {
                        (data[idx], data[idx + 2], data[idx + 4], data[idx + 6])
                    } else {
                        (data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
                    };
                    let pixel = color::compose_rgba(r, g, b, a);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(pix_mut.into())
}

/// Write a PNG image
pub fn write_png<W: Write>(pix: &Pix, writer: W) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    // Determine PNG format
    let (color_type, bit_depth) = match pix.depth() {
        PixelDepth::Bit1 => {
            if pix.has_colormap() {
                (ColorType::Indexed, BitDepth::One)
            } else {
                (ColorType::Grayscale, BitDepth::One)
            }
        }
        PixelDepth::Bit2 => {
            if pix.has_colormap() {
                (ColorType::Indexed, BitDepth::Two)
            } else {
                (ColorType::Grayscale, BitDepth::Two)
            }
        }
        PixelDepth::Bit4 => {
            if pix.has_colormap() {
                (ColorType::Indexed, BitDepth::Four)
            } else {
                (ColorType::Grayscale, BitDepth::Four)
            }
        }
        PixelDepth::Bit8 => {
            if pix.has_colormap() {
                (ColorType::Indexed, BitDepth::Eight)
            } else {
                (ColorType::Grayscale, BitDepth::Eight)
            }
        }
        PixelDepth::Bit16 => (ColorType::Grayscale, BitDepth::Sixteen),
        PixelDepth::Bit32 => {
            if pix.spp() == 4 {
                (ColorType::Rgba, BitDepth::Eight)
            } else {
                (ColorType::Rgb, BitDepth::Eight)
            }
        }
    };

    let mut encoder = Encoder::new(writer, width, height);
    encoder.set_color(color_type);
    encoder.set_depth(bit_depth);

    // Write palette if present
    if color_type == ColorType::Indexed
        && let Some(cmap) = pix.colormap()
    {
        let mut palette = Vec::with_capacity(cmap.len() * 3);
        for i in 0..cmap.len() {
            if let Some((r, g, b)) = cmap.get_rgb(i) {
                palette.push(r);
                palette.push(g);
                palette.push(b);
            }
        }
        encoder.set_palette(palette);
    }

    let mut writer = encoder
        .write_header()
        .map_err(|e| IoError::EncodeError(format!("PNG header error: {}", e)))?;

    // Prepare pixel data
    let bytes_per_row = match (color_type, bit_depth) {
        (ColorType::Grayscale, BitDepth::One) | (ColorType::Indexed, BitDepth::One) => {
            width.div_ceil(8)
        }
        (ColorType::Grayscale, BitDepth::Two) | (ColorType::Indexed, BitDepth::Two) => {
            width.div_ceil(4)
        }
        (ColorType::Grayscale, BitDepth::Four) | (ColorType::Indexed, BitDepth::Four) => {
            width.div_ceil(2)
        }
        (ColorType::Grayscale, BitDepth::Eight) | (ColorType::Indexed, BitDepth::Eight) => width,
        (ColorType::Grayscale, BitDepth::Sixteen) => width * 2,
        (ColorType::Rgb, _) => width * 3,
        (ColorType::Rgba, _) => width * 4,
        _ => unreachable!(),
    } as usize;

    let mut data = vec![0u8; bytes_per_row * height as usize];

    for y in 0..height {
        let row_start = y as usize * bytes_per_row;

        match (color_type, bit_depth) {
            (ColorType::Grayscale, BitDepth::One) | (ColorType::Indexed, BitDepth::One) => {
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y)
                        && val != 0
                    {
                        let byte_idx = row_start + (x / 8) as usize;
                        let bit_idx = 7 - (x % 8);
                        data[byte_idx] |= 1 << bit_idx;
                    }
                }
            }
            (ColorType::Grayscale, BitDepth::Two) | (ColorType::Indexed, BitDepth::Two) => {
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y) {
                        let byte_idx = row_start + (x / 4) as usize;
                        let shift = 6 - ((x % 4) * 2);
                        data[byte_idx] |= ((val & 3) as u8) << shift;
                    }
                }
            }
            (ColorType::Grayscale, BitDepth::Four) | (ColorType::Indexed, BitDepth::Four) => {
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y) {
                        let byte_idx = row_start + (x / 2) as usize;
                        if x % 2 == 0 {
                            data[byte_idx] |= ((val & 0xF) as u8) << 4;
                        } else {
                            data[byte_idx] |= (val & 0xF) as u8;
                        }
                    }
                }
            }
            (ColorType::Grayscale, BitDepth::Eight) | (ColorType::Indexed, BitDepth::Eight) => {
                for x in 0..width {
                    data[row_start + x as usize] = pix.get_pixel(x, y).unwrap_or(0) as u8;
                }
            }
            (ColorType::Grayscale, BitDepth::Sixteen) => {
                for x in 0..width {
                    let val = pix.get_pixel(x, y).unwrap_or(0);
                    let idx = row_start + (x as usize * 2);
                    data[idx] = (val >> 8) as u8;
                    data[idx + 1] = val as u8;
                }
            }
            (ColorType::Rgb, _) => {
                for x in 0..width {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    let (r, g, b) = color::extract_rgb(pixel);
                    let idx = row_start + (x as usize * 3);
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                }
            }
            (ColorType::Rgba, _) => {
                for x in 0..width {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    let (r, g, b, a) = color::extract_rgba(pixel);
                    let idx = row_start + (x as usize * 4);
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = a;
                }
            }
            _ => unreachable!(),
        }
    }

    writer
        .write_image_data(&data)
        .map_err(|e| IoError::EncodeError(format!("PNG write error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_png_roundtrip_grayscale() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel(x, y, (x + y) * 10).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_png(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_png(cursor).unwrap();

        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);

        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_png_roundtrip_rgb() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_rgb(0, 0, 255, 0, 0).unwrap();
        pix_mut.set_rgb(1, 1, 0, 255, 0).unwrap();
        pix_mut.set_rgb(2, 2, 0, 0, 255).unwrap();

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_png(&pix, &mut buffer).unwrap();

        let cursor = Cursor::new(buffer);
        let pix2 = read_png(cursor).unwrap();

        assert_eq!(pix2.get_rgb(0, 0), Some((255, 0, 0)));
        assert_eq!(pix2.get_rgb(1, 1), Some((0, 255, 0)));
        assert_eq!(pix2.get_rgb(2, 2), Some((0, 0, 255)));
    }
}
