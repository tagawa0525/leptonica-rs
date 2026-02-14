//! PNM (Portable Any Map) format support
//!
//! Supports PBM (P1/P4), PGM (P2/P5), and PPM (P3/P6) formats.

use crate::{IoError, IoResult};
use leptonica_core::{Pix, PixelDepth, color};
use std::io::{BufRead, BufReader, Read, Write};

/// PNM format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PnmType {
    PbmAscii,  // P1
    PgmAscii,  // P2
    PpmAscii,  // P3
    PbmBinary, // P4
    PgmBinary, // P5
    PpmBinary, // P6
}

impl PnmType {
    fn from_magic(magic: &[u8]) -> Option<Self> {
        match magic {
            b"P1" => Some(PnmType::PbmAscii),
            b"P2" => Some(PnmType::PgmAscii),
            b"P3" => Some(PnmType::PpmAscii),
            b"P4" => Some(PnmType::PbmBinary),
            b"P5" => Some(PnmType::PgmBinary),
            b"P6" => Some(PnmType::PpmBinary),
            _ => None,
        }
    }

    fn magic(&self) -> &'static [u8] {
        match self {
            PnmType::PbmAscii => b"P1",
            PnmType::PgmAscii => b"P2",
            PnmType::PpmAscii => b"P3",
            PnmType::PbmBinary => b"P4",
            PnmType::PgmBinary => b"P5",
            PnmType::PpmBinary => b"P6",
        }
    }

    fn is_binary(&self) -> bool {
        matches!(
            self,
            PnmType::PbmBinary | PnmType::PgmBinary | PnmType::PpmBinary
        )
    }
}

/// Read a PNM image
pub fn read_pnm<R: Read>(reader: R) -> IoResult<Pix> {
    let mut reader = BufReader::new(reader);

    // Read magic number
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic).map_err(IoError::Io)?;

    let pnm_type = PnmType::from_magic(&magic)
        .ok_or_else(|| IoError::InvalidData("invalid PNM magic number".to_string()))?;

    // Skip whitespace and comments
    skip_whitespace_and_comments(&mut reader)?;

    // Read width
    let width = read_number(&mut reader)?;
    skip_whitespace_and_comments(&mut reader)?;

    // Read height
    let height = read_number(&mut reader)?;

    // Read maxval (not for PBM)
    let maxval = match pnm_type {
        PnmType::PbmAscii | PnmType::PbmBinary => 1,
        _ => {
            skip_whitespace_and_comments(&mut reader)?;
            read_number(&mut reader)?
        }
    };

    // Skip whitespace before binary data.
    // The PNM spec says a single whitespace character separates the header
    // from binary data, but files may use CRLF or have trailing comments.
    if pnm_type.is_binary() {
        skip_whitespace_and_comments(&mut reader)?;
    }

    // Determine pixel depth
    let depth = match pnm_type {
        PnmType::PbmAscii | PnmType::PbmBinary => PixelDepth::Bit1,
        PnmType::PgmAscii | PnmType::PgmBinary => {
            if maxval <= 255 {
                PixelDepth::Bit8
            } else {
                PixelDepth::Bit16
            }
        }
        PnmType::PpmAscii | PnmType::PpmBinary => PixelDepth::Bit32,
    };

    let pix = Pix::new(width, height, depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();

    match pnm_type {
        PnmType::PbmAscii => read_pbm_ascii(&mut reader, &mut pix_mut, width, height)?,
        PnmType::PbmBinary => read_pbm_binary(&mut reader, &mut pix_mut, width, height)?,
        PnmType::PgmAscii => read_pgm_ascii(&mut reader, &mut pix_mut, width, height, maxval)?,
        PnmType::PgmBinary => read_pgm_binary(&mut reader, &mut pix_mut, width, height, maxval)?,
        PnmType::PpmAscii => read_ppm_ascii(&mut reader, &mut pix_mut, width, height, maxval)?,
        PnmType::PpmBinary => read_ppm_binary(&mut reader, &mut pix_mut, width, height, maxval)?,
    }

    Ok(pix_mut.into())
}

fn skip_whitespace_and_comments<R: BufRead>(reader: &mut R) -> IoResult<()> {
    loop {
        let buf = reader.fill_buf().map_err(IoError::Io)?;
        if buf.is_empty() {
            break;
        }

        let first = buf[0];
        if first == b'#' {
            // Skip comment line
            let mut line = String::new();
            reader.read_line(&mut line).map_err(IoError::Io)?;
        } else if first.is_ascii_whitespace() {
            reader.consume(1);
        } else {
            break;
        }
    }
    Ok(())
}

fn read_number<R: BufRead>(reader: &mut R) -> IoResult<u32> {
    let mut num_str = String::new();
    loop {
        let buf = reader.fill_buf().map_err(IoError::Io)?;
        if buf.is_empty() {
            break;
        }

        let first = buf[0];
        if first.is_ascii_digit() {
            num_str.push(first as char);
            reader.consume(1);
        } else {
            break;
        }
    }

    num_str
        .parse()
        .map_err(|_| IoError::InvalidData("invalid number".to_string()))
}

fn read_pbm_ascii<R: BufRead>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
) -> IoResult<()> {
    for y in 0..height {
        for x in 0..width {
            skip_whitespace_and_comments(reader)?;
            let val = read_number(reader)?;
            // PBM: 1 = black, 0 = white (same convention as leptonica_core 1bpp)
            pix.set_pixel_unchecked(x, y, if val != 0 { 1 } else { 0 });
        }
    }
    Ok(())
}

fn read_pbm_binary<R: Read>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
) -> IoResult<()> {
    let row_bytes = width.div_ceil(8);
    let mut row_buffer = vec![0u8; row_bytes as usize];

    for y in 0..height {
        reader.read_exact(&mut row_buffer).map_err(IoError::Io)?;

        for x in 0..width {
            let byte_idx = (x / 8) as usize;
            let bit_idx = 7 - (x % 8);
            let val = (row_buffer[byte_idx] >> bit_idx) & 1;
            pix.set_pixel_unchecked(x, y, val as u32);
        }
    }
    Ok(())
}

fn read_pgm_ascii<R: BufRead>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
    maxval: u32,
) -> IoResult<()> {
    for y in 0..height {
        for x in 0..width {
            skip_whitespace_and_comments(reader)?;
            let val = read_number(reader)?;
            let scaled = if maxval != 255 {
                (val * 255 / maxval) as u32
            } else {
                val
            };
            pix.set_pixel_unchecked(x, y, scaled);
        }
    }
    Ok(())
}

fn read_pgm_binary<R: Read>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
    maxval: u32,
) -> IoResult<()> {
    if maxval <= 255 {
        let mut row_buffer = vec![0u8; width as usize];
        for y in 0..height {
            reader.read_exact(&mut row_buffer).map_err(IoError::Io)?;
            for x in 0..width {
                let val = row_buffer[x as usize] as u32;
                let scaled = if maxval != 255 {
                    val * 255 / maxval
                } else {
                    val
                };
                pix.set_pixel_unchecked(x, y, scaled);
            }
        }
    } else {
        // 16-bit values (big-endian)
        let mut row_buffer = vec![0u8; (width * 2) as usize];
        for y in 0..height {
            reader.read_exact(&mut row_buffer).map_err(IoError::Io)?;
            for x in 0..width {
                let idx = (x * 2) as usize;
                let val = ((row_buffer[idx] as u32) << 8) | (row_buffer[idx + 1] as u32);
                let scaled = val * 255 / maxval;
                pix.set_pixel_unchecked(x, y, scaled);
            }
        }
    }
    Ok(())
}

fn read_ppm_ascii<R: BufRead>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
    maxval: u32,
) -> IoResult<()> {
    for y in 0..height {
        for x in 0..width {
            skip_whitespace_and_comments(reader)?;
            let r = read_number(reader)?;
            skip_whitespace_and_comments(reader)?;
            let g = read_number(reader)?;
            skip_whitespace_and_comments(reader)?;
            let b = read_number(reader)?;

            let (r, g, b) = if maxval != 255 {
                (
                    (r * 255 / maxval) as u8,
                    (g * 255 / maxval) as u8,
                    (b * 255 / maxval) as u8,
                )
            } else {
                (r as u8, g as u8, b as u8)
            };

            let pixel = color::compose_rgb(r, g, b);
            pix.set_pixel_unchecked(x, y, pixel);
        }
    }
    Ok(())
}

fn read_ppm_binary<R: Read>(
    reader: &mut R,
    pix: &mut leptonica_core::pix::PixMut,
    width: u32,
    height: u32,
    maxval: u32,
) -> IoResult<()> {
    let bytes_per_sample = if maxval <= 255 { 1 } else { 2 };
    let row_bytes = (width * 3 * bytes_per_sample) as usize;
    let mut row_buffer = vec![0u8; row_bytes];

    for y in 0..height {
        reader.read_exact(&mut row_buffer).map_err(IoError::Io)?;

        for x in 0..width {
            let (r, g, b) = if bytes_per_sample == 1 {
                let idx = (x * 3) as usize;
                (row_buffer[idx], row_buffer[idx + 1], row_buffer[idx + 2])
            } else {
                let idx = (x * 6) as usize;
                let r = ((row_buffer[idx] as u32) << 8 | row_buffer[idx + 1] as u32) * 255 / maxval;
                let g =
                    ((row_buffer[idx + 2] as u32) << 8 | row_buffer[idx + 3] as u32) * 255 / maxval;
                let b =
                    ((row_buffer[idx + 4] as u32) << 8 | row_buffer[idx + 5] as u32) * 255 / maxval;
                (r as u8, g as u8, b as u8)
            };

            let (r, g, b) = if maxval != 255 && bytes_per_sample == 1 {
                (
                    (r as u32 * 255 / maxval) as u8,
                    (g as u32 * 255 / maxval) as u8,
                    (b as u32 * 255 / maxval) as u8,
                )
            } else {
                (r, g, b)
            };

            let pixel = color::compose_rgb(r, g, b);
            pix.set_pixel_unchecked(x, y, pixel);
        }
    }
    Ok(())
}

/// Write a PNM image
pub fn write_pnm<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    // Determine PNM type based on depth.
    // Colormapped images are expanded to PPM (P6) to preserve color information,
    // since PGM would serialize palette indices as grayscale values.
    let pnm_type = if pix.has_colormap() {
        PnmType::PpmBinary
    } else {
        match pix.depth() {
            PixelDepth::Bit1 => PnmType::PbmBinary,
            PixelDepth::Bit8 | PixelDepth::Bit2 | PixelDepth::Bit4 => PnmType::PgmBinary,
            PixelDepth::Bit32 => PnmType::PpmBinary,
            _ => {
                return Err(IoError::UnsupportedFormat(format!(
                    "cannot write {:?} as PNM",
                    pix.depth()
                )));
            }
        }
    };

    // Write header
    writer.write_all(pnm_type.magic()).map_err(IoError::Io)?;
    writer.write_all(b"\n").map_err(IoError::Io)?;
    writeln!(writer, "{} {}", width, height).map_err(IoError::Io)?;

    match pnm_type {
        PnmType::PbmBinary => {
            // No maxval for PBM
        }
        PnmType::PgmBinary | PnmType::PpmBinary => {
            writer.write_all(b"255\n").map_err(IoError::Io)?;
        }
        _ => unreachable!(),
    }

    // Write pixel data
    match pnm_type {
        PnmType::PbmBinary => {
            let row_bytes = width.div_ceil(8);
            let mut row_buffer = vec![0u8; row_bytes as usize];

            for y in 0..height {
                row_buffer.fill(0);
                for x in 0..width {
                    if let Some(val) = pix.get_pixel(x, y)
                        && val != 0
                    {
                        let byte_idx = (x / 8) as usize;
                        let bit_idx = 7 - (x % 8);
                        row_buffer[byte_idx] |= 1 << bit_idx;
                    }
                }
                writer.write_all(&row_buffer).map_err(IoError::Io)?;
            }
        }
        PnmType::PgmBinary => {
            let mut row_buffer = vec![0u8; width as usize];

            for y in 0..height {
                for x in 0..width {
                    row_buffer[x as usize] = pix.get_pixel(x, y).unwrap_or(0) as u8;
                }
                writer.write_all(&row_buffer).map_err(IoError::Io)?;
            }
        }
        PnmType::PpmBinary => {
            let mut row_buffer = vec![0u8; (width * 3) as usize];
            let cmap = pix.colormap();

            for y in 0..height {
                for x in 0..width {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    let (r, g, b) = if let Some(ref cm) = cmap {
                        // Expand colormapped pixel through the colormap
                        cm.get_rgb(pixel as usize).unwrap_or((0, 0, 0))
                    } else {
                        color::extract_rgb(pixel)
                    };
                    let idx = (x * 3) as usize;
                    row_buffer[idx] = r;
                    row_buffer[idx + 1] = g;
                    row_buffer[idx + 2] = b;
                }
                writer.write_all(&row_buffer).map_err(IoError::Io)?;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgm_roundtrip() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        for y in 0..10 {
            for x in 0..10 {
                pix_mut.set_pixel(x, y, (x + y) * 10).unwrap();
            }
        }

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_pnm(&pix, &mut buffer).unwrap();

        let cursor = std::io::Cursor::new(buffer);
        let pix2 = read_pnm(cursor).unwrap();

        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 10);

        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_ppm_roundtrip() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        pix_mut.set_rgb(0, 0, 255, 0, 0).unwrap();
        pix_mut.set_rgb(1, 1, 0, 255, 0).unwrap();
        pix_mut.set_rgb(2, 2, 0, 0, 255).unwrap();

        let pix: Pix = pix_mut.into();

        let mut buffer = Vec::new();
        write_pnm(&pix, &mut buffer).unwrap();

        let cursor = std::io::Cursor::new(buffer);
        let pix2 = read_pnm(cursor).unwrap();

        assert_eq!(pix2.get_rgb(0, 0), Some((255, 0, 0)));
        assert_eq!(pix2.get_rgb(1, 1), Some((0, 255, 0)));
        assert_eq!(pix2.get_rgb(2, 2), Some((0, 0, 255)));
    }
}
