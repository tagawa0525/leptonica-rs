//! BMP image format support
//!
//! Reads and writes Windows Bitmap (BMP) files.

use crate::{IoError, IoResult};
use leptonica_core::{Pix, PixelDepth, color};
use std::io::{Read, Write};

/// BMP file header size
const BMP_FILE_HEADER_SIZE: usize = 14;

/// BMP info header size (BITMAPINFOHEADER)
const BMP_INFO_HEADER_SIZE: u32 = 40;

/// Read a BMP image
pub fn read_bmp<R: Read>(mut reader: R) -> IoResult<Pix> {
    // Read file header (14 bytes)
    let mut file_header = [0u8; BMP_FILE_HEADER_SIZE];
    reader.read_exact(&mut file_header).map_err(IoError::Io)?;

    // Verify magic number
    if &file_header[0..2] != b"BM" {
        return Err(IoError::InvalidData("not a BMP file".to_string()));
    }

    // Get pixel data offset
    let pixel_offset = u32::from_le_bytes([
        file_header[10],
        file_header[11],
        file_header[12],
        file_header[13],
    ]) as usize;

    // Read info header (minimum 40 bytes)
    let mut info_header = [0u8; 40];
    reader.read_exact(&mut info_header).map_err(IoError::Io)?;

    let header_size = u32::from_le_bytes([
        info_header[0],
        info_header[1],
        info_header[2],
        info_header[3],
    ]);

    if header_size < BMP_INFO_HEADER_SIZE {
        return Err(IoError::InvalidData(format!(
            "unsupported BMP header size: {}",
            header_size
        )));
    }

    let width = i32::from_le_bytes([
        info_header[4],
        info_header[5],
        info_header[6],
        info_header[7],
    ]);

    let height = i32::from_le_bytes([
        info_header[8],
        info_header[9],
        info_header[10],
        info_header[11],
    ]);

    let planes = u16::from_le_bytes([info_header[12], info_header[13]]);
    if planes != 1 {
        return Err(IoError::InvalidData(format!(
            "unsupported number of planes: {}",
            planes
        )));
    }

    let bits_per_pixel = u16::from_le_bytes([info_header[14], info_header[15]]);

    let compression = u32::from_le_bytes([
        info_header[16],
        info_header[17],
        info_header[18],
        info_header[19],
    ]);

    // Only support uncompressed BMP for now
    if compression != 0 && compression != 3 {
        return Err(IoError::UnsupportedFormat(format!(
            "unsupported BMP compression: {}",
            compression
        )));
    }

    let width = width.unsigned_abs();
    let top_down = height < 0;
    let height = height.unsigned_abs();

    // Determine pixel depth
    let depth = match bits_per_pixel {
        1 => PixelDepth::Bit1,
        4 => PixelDepth::Bit4,
        8 => PixelDepth::Bit8,
        24 | 32 => PixelDepth::Bit32,
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "unsupported BMP bit depth: {}",
                bits_per_pixel
            )));
        }
    };

    // Read colormap if present (for 1, 4, 8 bit images)
    let colormap = if bits_per_pixel <= 8 {
        let num_colors = 1usize << bits_per_pixel;
        let bytes_to_skip = header_size as usize - 40;
        if bytes_to_skip > 0 {
            let mut skip = vec![0u8; bytes_to_skip];
            reader.read_exact(&mut skip).map_err(IoError::Io)?;
        }

        let mut palette = vec![0u8; num_colors * 4];
        reader.read_exact(&mut palette).map_err(IoError::Io)?;

        let mut cmap = leptonica_core::PixColormap::new(bits_per_pixel as u32)?;
        for i in 0..num_colors {
            let b = palette[i * 4];
            let g = palette[i * 4 + 1];
            let r = palette[i * 4 + 2];
            cmap.add_rgb(r, g, b)?;
        }
        Some(cmap)
    } else {
        None
    };

    // Skip to pixel data
    let current_pos =
        BMP_FILE_HEADER_SIZE + header_size as usize + colormap.as_ref().map_or(0, |c| c.len() * 4);
    if pixel_offset > current_pos {
        let skip_bytes = pixel_offset - current_pos;
        let mut skip = vec![0u8; skip_bytes];
        reader.read_exact(&mut skip).map_err(IoError::Io)?;
    }

    // Create PIX
    let pix = Pix::new(width, height, depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();

    if let Some(cmap) = colormap {
        pix_mut.set_colormap(Some(cmap))?;
    }

    // Calculate row stride (BMP rows are 4-byte aligned)
    let row_stride = ((width as usize * bits_per_pixel as usize + 31) / 32) * 4;
    let mut row_buffer = vec![0u8; row_stride];

    // Read pixel data
    for row in 0..height {
        reader.read_exact(&mut row_buffer).map_err(IoError::Io)?;

        let y = if top_down { row } else { height - 1 - row };

        match bits_per_pixel {
            1 => {
                for x in 0..width {
                    let byte_idx = (x / 8) as usize;
                    let bit_idx = 7 - (x % 8);
                    let val = (row_buffer[byte_idx] >> bit_idx) & 1;
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
            4 => {
                for x in 0..width {
                    let byte_idx = (x / 2) as usize;
                    let val = if x % 2 == 0 {
                        (row_buffer[byte_idx] >> 4) & 0xF
                    } else {
                        row_buffer[byte_idx] & 0xF
                    };
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
            8 => {
                for x in 0..width {
                    let val = row_buffer[x as usize];
                    unsafe { pix_mut.set_pixel_unchecked(x, y, val as u32) };
                }
            }
            24 => {
                for x in 0..width {
                    let idx = (x as usize) * 3;
                    let b = row_buffer[idx];
                    let g = row_buffer[idx + 1];
                    let r = row_buffer[idx + 2];
                    let pixel = color::compose_rgb(r, g, b);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
            32 => {
                for x in 0..width {
                    let idx = (x as usize) * 4;
                    let b = row_buffer[idx];
                    let g = row_buffer[idx + 1];
                    let r = row_buffer[idx + 2];
                    let a = row_buffer[idx + 3];
                    let pixel = color::compose_rgba(r, g, b, a);
                    unsafe { pix_mut.set_pixel_unchecked(x, y, pixel) };
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(pix_mut.into())
}

/// Write a BMP image
pub fn write_bmp<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();
    let depth = pix.depth();

    // Determine output bit depth
    let (bits_per_pixel, has_colormap): (u16, bool) = match depth {
        PixelDepth::Bit1 => (1, pix.has_colormap()),
        PixelDepth::Bit4 => (4, pix.has_colormap()),
        PixelDepth::Bit8 => (8, pix.has_colormap() || true), // Always use colormap for 8-bit
        PixelDepth::Bit32 => (24, false),                    // Write as 24-bit RGB
        _ => {
            return Err(IoError::UnsupportedFormat(format!(
                "cannot write {:?} as BMP",
                depth
            )));
        }
    };

    // Calculate sizes
    let row_stride = ((width as usize * bits_per_pixel as usize + 31) / 32) * 4;
    let pixel_data_size = row_stride * height as usize;

    let colormap_size = if has_colormap {
        (1usize << bits_per_pixel) * 4
    } else {
        0
    };

    let pixel_offset = BMP_FILE_HEADER_SIZE + BMP_INFO_HEADER_SIZE as usize + colormap_size;
    let file_size = pixel_offset + pixel_data_size;

    // Write file header
    writer.write_all(b"BM").map_err(IoError::Io)?;
    writer
        .write_all(&(file_size as u32).to_le_bytes())
        .map_err(IoError::Io)?;
    writer.write_all(&[0u8; 4]).map_err(IoError::Io)?; // Reserved
    writer
        .write_all(&(pixel_offset as u32).to_le_bytes())
        .map_err(IoError::Io)?;

    // Write info header
    writer
        .write_all(&BMP_INFO_HEADER_SIZE.to_le_bytes())
        .map_err(IoError::Io)?;
    writer
        .write_all(&(width as i32).to_le_bytes())
        .map_err(IoError::Io)?;
    writer
        .write_all(&(height as i32).to_le_bytes())
        .map_err(IoError::Io)?; // Bottom-up
    writer.write_all(&1u16.to_le_bytes()).map_err(IoError::Io)?; // Planes
    writer
        .write_all(&bits_per_pixel.to_le_bytes())
        .map_err(IoError::Io)?;
    writer.write_all(&0u32.to_le_bytes()).map_err(IoError::Io)?; // Compression
    writer
        .write_all(&(pixel_data_size as u32).to_le_bytes())
        .map_err(IoError::Io)?;
    writer.write_all(&0i32.to_le_bytes()).map_err(IoError::Io)?; // X pixels per meter
    writer.write_all(&0i32.to_le_bytes()).map_err(IoError::Io)?; // Y pixels per meter
    writer.write_all(&0u32.to_le_bytes()).map_err(IoError::Io)?; // Colors used
    writer.write_all(&0u32.to_le_bytes()).map_err(IoError::Io)?; // Important colors

    // Write colormap
    if has_colormap {
        let num_colors = 1usize << bits_per_pixel;
        if let Some(cmap) = pix.colormap() {
            for i in 0..num_colors {
                let (r, g, b) = cmap.get_rgb(i).unwrap_or((0, 0, 0));
                writer.write_all(&[b, g, r, 0]).map_err(IoError::Io)?;
            }
        } else {
            // Create grayscale colormap
            for i in 0..num_colors {
                let val = ((i * 255) / (num_colors - 1)) as u8;
                writer.write_all(&[val, val, val, 0]).map_err(IoError::Io)?;
            }
        }
    }

    // Write pixel data (bottom-up)
    let mut row_buffer = vec![0u8; row_stride];

    for row in 0..height {
        let y = height - 1 - row;

        match depth {
            PixelDepth::Bit1 => {
                row_buffer.fill(0);
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y) {
                        if val != 0 {
                            let byte_idx = (x / 8) as usize;
                            let bit_idx = 7 - (x % 8);
                            row_buffer[byte_idx] |= 1 << bit_idx;
                        }
                    }
                }
            }
            PixelDepth::Bit4 => {
                row_buffer.fill(0);
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y) {
                        let byte_idx = (x / 2) as usize;
                        if x % 2 == 0 {
                            row_buffer[byte_idx] |= ((val & 0xF) as u8) << 4;
                        } else {
                            row_buffer[byte_idx] |= (val & 0xF) as u8;
                        }
                    }
                }
            }
            PixelDepth::Bit8 => {
                for x in 0..width {
                    row_buffer[x as usize] = pix.get_pixel(x, y).unwrap_or(0) as u8;
                }
            }
            PixelDepth::Bit32 => {
                for x in 0..width {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    let (r, g, b) = color::extract_rgb(pixel);
                    let idx = (x as usize) * 3;
                    row_buffer[idx] = b;
                    row_buffer[idx + 1] = g;
                    row_buffer[idx + 2] = r;
                }
            }
            _ => {
                return Err(IoError::UnsupportedFormat(format!(
                    "cannot write {:?} as BMP",
                    depth
                )));
            }
        }

        writer.write_all(&row_buffer).map_err(IoError::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmp_roundtrip_8bit() {
        // Create a simple 8-bit grayscale image
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set some pixels
        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel(x, y, ((x + y) * 10) as u32).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        // Write to memory
        let mut buffer = Vec::new();
        write_bmp(&pix, &mut buffer).unwrap();

        // Read back
        let cursor = std::io::Cursor::new(buffer);
        let pix2 = read_bmp(cursor).unwrap();

        // Verify
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);
        assert_eq!(pix2.depth(), PixelDepth::Bit8);

        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_bmp_roundtrip_32bit() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set RGB pixels
        pix_mut.set_rgb(0, 0, 255, 0, 0).unwrap(); // Red
        pix_mut.set_rgb(1, 1, 0, 255, 0).unwrap(); // Green
        pix_mut.set_rgb(2, 2, 0, 0, 255).unwrap(); // Blue

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_bmp(&pix, &mut buffer).unwrap();

        let cursor = std::io::Cursor::new(buffer);
        let pix2 = read_bmp(cursor).unwrap();

        assert_eq!(pix2.get_rgb(0, 0), Some((255, 0, 0)));
        assert_eq!(pix2.get_rgb(1, 1), Some((0, 255, 0)));
        assert_eq!(pix2.get_rgb(2, 2), Some((0, 0, 255)));
    }
}
