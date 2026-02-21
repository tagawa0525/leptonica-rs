//! PNM (Portable Any Map) format support
//!
//! Supports PBM (P1/P4), PGM (P2/P5), and PPM (P3/P6) formats.

use crate::{IoError, IoResult, header::ImageHeader};
use leptonica_core::{ImageFormat, Pix, PixelDepth, color};
use std::io::{BufRead, BufReader, Read, Write};

/// Read PNM header metadata without decoding pixel data
pub fn read_header_pnm(data: &[u8]) -> IoResult<ImageHeader> {
    let mut reader = BufReader::new(data);

    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic).map_err(IoError::Io)?;

    // P7 (PAM) has a different header format
    if &magic == b"P7" {
        return read_header_pam(&mut reader);
    }

    let pnm_type = PnmType::from_magic(&magic)
        .ok_or_else(|| IoError::InvalidData("invalid PNM magic number".to_string()))?;

    skip_whitespace_and_comments(&mut reader)?;
    let width = read_number(&mut reader)?;
    skip_whitespace_and_comments(&mut reader)?;
    let height = read_number(&mut reader)?;

    let maxval = match pnm_type {
        PnmType::PbmAscii | PnmType::PbmBinary => 1,
        _ => {
            skip_whitespace_and_comments(&mut reader)?;
            read_number(&mut reader)?
        }
    };

    let (depth, spp, bps) = match pnm_type {
        PnmType::PbmAscii | PnmType::PbmBinary => (1u32, 1u32, 1u32),
        PnmType::PgmAscii | PnmType::PgmBinary => {
            if maxval > 255 {
                (16, 1, 16)
            } else {
                (8, 1, 8)
            }
        }
        // PPM images use 32 bpp internally (24-bit RGB in 32-bit word), 8 bits per sample
        PnmType::PpmAscii | PnmType::PpmBinary => (32, 3, 8),
    };

    Ok(ImageHeader {
        width,
        height,
        depth,
        bps,
        spp,
        has_colormap: false,
        num_colors: 0,
        format: ImageFormat::Pnm,
        x_resolution: None,
        y_resolution: None,
    })
}

/// Parse a PAM (P7) header and return an ImageHeader
fn read_header_pam<R: BufRead>(reader: &mut R) -> IoResult<ImageHeader> {
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut pam_depth: Option<u32> = None; // spp
    let mut maxval: Option<u32> = None;
    let mut found_endhdr = false;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(IoError::Io)?;
        if n == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "ENDHDR" {
            found_endhdr = true;
            break;
        }
        if let Some(rest) = line.strip_prefix("WIDTH ") {
            width = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM WIDTH".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("HEIGHT ") {
            height = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM HEIGHT".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("DEPTH ") {
            pam_depth = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM DEPTH".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("MAXVAL ") {
            maxval = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM MAXVAL".to_string()))?,
            );
        }
    }

    if !found_endhdr {
        return Err(IoError::InvalidData(
            "PAM header missing ENDHDR".to_string(),
        ));
    }

    let w = width.ok_or_else(|| IoError::InvalidData("missing PAM WIDTH".to_string()))?;
    let h = height.ok_or_else(|| IoError::InvalidData("missing PAM HEIGHT".to_string()))?;
    let spp = pam_depth.ok_or_else(|| IoError::InvalidData("missing PAM DEPTH".to_string()))?;
    let mv = maxval.ok_or_else(|| IoError::InvalidData("missing PAM MAXVAL".to_string()))?;

    let bps: u32 = if mv == 1 {
        1
    } else if mv <= 3 {
        2
    } else if mv <= 15 {
        4
    } else if mv <= 255 {
        8
    } else {
        16
    };

    let depth = match spp {
        1 => bps,
        2..=4 => 32,
        _ => 32,
    };

    Ok(ImageHeader {
        width: w,
        height: h,
        depth,
        bps,
        spp,
        has_colormap: false,
        num_colors: 0,
        format: ImageFormat::Pnm,
        x_resolution: None,
        y_resolution: None,
    })
}

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

    // P7 (PAM) is handled by read_pam - delegate to it with the consumed magic bytes prepended
    if &magic == b"P7" {
        let prefix = std::io::Cursor::new(b"P7" as &[u8]);
        return read_pam(prefix.chain(reader));
    }

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

/// Write a Pix as ASCII PNM (P1/P2/P3)
///
/// Selection rules (following C Leptonica `pixWriteStreamAsciiPnm`):
/// - 1 bpp → P1 (PBM ASCII)
/// - 2/4/8/16 bpp, no colormap or grayscale colormap → P2 (PGM ASCII)
/// - 2/4/8 bpp with color-valued colormap, or 32 bpp → P3 (PPM ASCII)
///
/// # See also
///
/// C Leptonica: `pixWriteStreamAsciiPnm()` in `pnmio.c`
pub fn write_pnm_ascii<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    // Colormapped → remove colormap first
    let owned;
    let pix = if pix.has_colormap() {
        owned = pix
            .remove_colormap(leptonica_core::pix::RemoveColormapTarget::BasedOnSrc)
            .map_err(|e| IoError::EncodeError(format!("colormap removal failed: {}", e)))?;
        &owned
    } else {
        pix
    };

    match pix.depth() {
        PixelDepth::Bit1 => {
            // P1: 1=black, 0=white; values separated by spaces, wrap at ~70 chars
            write!(writer, "P1\n{} {}\n", width, height).map_err(IoError::Io)?;
            let mut count = 0usize;
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y);
                    let ch = if val != 0 { b'1' } else { b'0' };
                    writer.write_all(&[ch, b' ']).map_err(IoError::Io)?;
                    count += 2;
                    if count >= 70 {
                        writer.write_all(b"\n").map_err(IoError::Io)?;
                        count = 0;
                    }
                }
            }
        }
        PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8 | PixelDepth::Bit16 => {
            let depth_bits = pix.depth().bits();
            let maxval = (1u32 << depth_bits) - 1;
            write!(writer, "P2\n{} {}\n{}\n", width, height, maxval).map_err(IoError::Io)?;
            let mut count = 0usize;
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y);
                    let field_width = match depth_bits {
                        2 => 2,
                        4 => 3,
                        8 => 4,
                        _ => 6, // 16-bit: up to 65535 = 5 digits + space
                    };
                    let s = format!("{:width$} ", val, width = field_width - 1);
                    writer.write_all(s.as_bytes()).map_err(IoError::Io)?;
                    count += field_width;
                    if count >= 60 {
                        writer.write_all(b"\n").map_err(IoError::Io)?;
                        count = 0;
                    }
                }
            }
        }
        PixelDepth::Bit32 => {
            write!(writer, "P3\n{} {}\n255\n", width, height).map_err(IoError::Io)?;
            let mut count = 0usize;
            for y in 0..height {
                for x in 0..width {
                    let pixel = pix.get_pixel_unchecked(x, y);
                    let (r, g, b) = color::extract_rgb(pixel);
                    for &ch in &[r, g, b] {
                        let s = format!("{:3} ", ch);
                        writer.write_all(s.as_bytes()).map_err(IoError::Io)?;
                        count += 4;
                        if count >= 60 {
                            writer.write_all(b"\n").map_err(IoError::Io)?;
                            count = 0;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Read a PAM (P7) image
///
/// Supports the standard PAM tuple types `BLACKANDWHITE`, `GRAYSCALE`, `RGB`,
/// and `RGB_ALPHA`. Additionally, images with `DEPTH 2` are accepted and
/// treated as grayscale + alpha: the grayscale sample is replicated into the
/// R, G, and B channels, and the alpha sample is used as the alpha component
/// of the resulting 32 bpp `Pix`.
///
/// # See also
///
/// C Leptonica: `pixReadStreamPnm()` type-7 branch in `pnmio.c`
pub fn read_pam<R: Read>(reader: R) -> IoResult<Pix> {
    let mut reader = BufReader::new(reader);

    // Read and verify magic
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic).map_err(IoError::Io)?;
    if &magic != b"P7" {
        return Err(IoError::InvalidData(
            "not a PAM file (expected P7)".to_string(),
        ));
    }

    // Parse header lines until ENDHDR
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut depth: Option<u32> = None; // spp
    let mut maxval: Option<u32> = None;
    let mut tupltype: Option<String> = None;
    let mut found_endhdr = false;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(IoError::Io)?;
        if n == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "ENDHDR" {
            found_endhdr = true;
            break;
        }
        if let Some(rest) = line.strip_prefix("WIDTH ") {
            width = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM WIDTH".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("HEIGHT ") {
            height = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM HEIGHT".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("DEPTH ") {
            depth = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM DEPTH".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("MAXVAL ") {
            maxval = Some(
                rest.trim()
                    .parse()
                    .map_err(|_| IoError::InvalidData("invalid PAM MAXVAL".to_string()))?,
            );
        } else if let Some(rest) = line.strip_prefix("TUPLTYPE ") {
            tupltype = Some(rest.trim().to_string());
        }
    }

    if !found_endhdr {
        return Err(IoError::InvalidData(
            "PAM header missing ENDHDR".to_string(),
        ));
    }

    let w = width.ok_or_else(|| IoError::InvalidData("missing PAM WIDTH".to_string()))?;
    let h = height.ok_or_else(|| IoError::InvalidData("missing PAM HEIGHT".to_string()))?;
    let spp = depth.ok_or_else(|| IoError::InvalidData("missing PAM DEPTH".to_string()))?;
    let mv = maxval.ok_or_else(|| IoError::InvalidData("missing PAM MAXVAL".to_string()))?;

    // Determine bps and pixel depth from maxval
    let bps: u32 = if mv == 1 {
        1
    } else if mv == 3 {
        2
    } else if mv == 15 {
        4
    } else if mv == 255 {
        8
    } else if mv == 65535 {
        16
    } else {
        return Err(IoError::InvalidData(format!(
            "unsupported PAM MAXVAL {}",
            mv
        )));
    };

    // Determine Pix depth from spp and bps
    let pix_depth = match spp {
        1 => match bps {
            1 => PixelDepth::Bit1,
            2 => PixelDepth::Bit2,
            4 => PixelDepth::Bit4,
            8 => PixelDepth::Bit8,
            16 => PixelDepth::Bit16,
            _ => return Err(IoError::InvalidData(format!("unsupported PAM bps {}", bps))),
        },
        2..=4 => PixelDepth::Bit32,
        _ => return Err(IoError::InvalidData(format!("unsupported PAM spp {}", spp))),
    };

    let mask8 = if bps < 8 { (1u8 << bps) - 1 } else { 0xFF };

    let pix = Pix::new(w, h, pix_depth)?;
    let mut pix_mut = pix.try_into_mut().unwrap();

    match spp {
        1 => {
            // Grayscale / binary
            for y in 0..h {
                for x in 0..w {
                    if bps == 16 {
                        // 16-bit grayscale: read two bytes per sample (big-endian)
                        let mut buf = [0u8; 2];
                        reader.read_exact(&mut buf).map_err(IoError::Io)?;
                        let val = u16::from_be_bytes(buf) as u32;
                        pix_mut.set_pixel_unchecked(x, y, val);
                    } else {
                        let mut buf = [0u8; 1];
                        reader.read_exact(&mut buf).map_err(IoError::Io)?;
                        let val = buf[0] & mask8;
                        let val = if bps == 1 { val ^ 1 } else { val }; // PAM white-is-1 → leptonica 0=white
                        pix_mut.set_pixel_unchecked(x, y, val as u32);
                    }
                }
            }
        }
        2 => {
            // Grayscale + alpha → 32bpp
            pix_mut.set_spp(4);
            let bytes_per_sample = if mv > 255 { 2 } else { 1 };
            let mut sample_buf = vec![0u8; 2 * bytes_per_sample];
            for y in 0..h {
                for x in 0..w {
                    reader.read_exact(&mut sample_buf).map_err(IoError::Io)?;
                    let (g, a) = if bytes_per_sample == 2 {
                        let g = read_sample_16(&sample_buf[0..2], mv);
                        let a = read_sample_16(&sample_buf[2..4], mv);
                        (g, a)
                    } else {
                        let g = scale_sample(sample_buf[0] & mask8, mv);
                        let a = scale_sample(sample_buf[1] & mask8, mv);
                        (g, a)
                    };
                    let pixel = color::compose_rgba(g, g, g, a);
                    pix_mut.set_pixel_unchecked(x, y, pixel);
                }
            }
        }
        3 => {
            // RGB
            pix_mut.set_spp(3);
            let bytes_per_sample = if mv > 255 { 2 } else { 1 };
            let mut sample_buf = vec![0u8; 3 * bytes_per_sample];
            for y in 0..h {
                for x in 0..w {
                    reader.read_exact(&mut sample_buf).map_err(IoError::Io)?;
                    let (r, g, b) = if bytes_per_sample == 2 {
                        (
                            read_sample_16(&sample_buf[0..2], mv),
                            read_sample_16(&sample_buf[2..4], mv),
                            read_sample_16(&sample_buf[4..6], mv),
                        )
                    } else {
                        (
                            scale_sample(sample_buf[0] & mask8, mv),
                            scale_sample(sample_buf[1] & mask8, mv),
                            scale_sample(sample_buf[2] & mask8, mv),
                        )
                    };
                    let pixel = color::compose_rgb(r, g, b);
                    pix_mut.set_pixel_unchecked(x, y, pixel);
                }
            }
        }
        4 => {
            // RGBA
            pix_mut.set_spp(4);
            let bytes_per_sample = if mv > 255 { 2 } else { 1 };
            let mut sample_buf = vec![0u8; 4 * bytes_per_sample];
            for y in 0..h {
                for x in 0..w {
                    reader.read_exact(&mut sample_buf).map_err(IoError::Io)?;
                    let (r, g, b, a) = if bytes_per_sample == 2 {
                        (
                            read_sample_16(&sample_buf[0..2], mv),
                            read_sample_16(&sample_buf[2..4], mv),
                            read_sample_16(&sample_buf[4..6], mv),
                            read_sample_16(&sample_buf[6..8], mv),
                        )
                    } else {
                        (
                            scale_sample(sample_buf[0] & mask8, mv),
                            scale_sample(sample_buf[1] & mask8, mv),
                            scale_sample(sample_buf[2] & mask8, mv),
                            scale_sample(sample_buf[3] & mask8, mv),
                        )
                    };
                    let pixel = color::compose_rgba(r, g, b, a);
                    pix_mut.set_pixel_unchecked(x, y, pixel);
                }
            }
        }
        _ => unreachable!(),
    }

    // Note tupltype but don't use it to override pix_depth determination
    let _ = tupltype;

    Ok(pix_mut.into())
}

/// Scale an 8-bit sample value from 0..maxval to 0..255
fn scale_sample(val: u8, maxval: u32) -> u8 {
    if maxval == 255 {
        val
    } else {
        (val as u32 * 255 / maxval) as u8
    }
}

/// Read a 16-bit big-endian sample and scale to 0..255
fn read_sample_16(buf: &[u8], maxval: u32) -> u8 {
    let val = u16::from_be_bytes([buf[0], buf[1]]) as u32;
    (val * 255 / maxval) as u8
}

/// Write a Pix as PAM (P7 Portable Arbitrary Map)
///
/// Supports 1/2/4/8/16 bpp grayscale and 32 bpp RGB/RGBA.
/// Colormapped images have their colormap removed before writing.
///
/// # See also
///
/// C Leptonica: `pixWriteStreamPam()` in `pnmio.c`
pub fn write_pam<W: Write>(pix: &Pix, mut writer: W) -> IoResult<()> {
    let width = pix.width();
    let height = pix.height();

    // Colormapped → remove colormap first
    let owned;
    let pix = if pix.has_colormap() {
        owned = pix
            .remove_colormap(leptonica_core::pix::RemoveColormapTarget::BasedOnSrc)
            .map_err(|e| IoError::EncodeError(format!("colormap removal failed: {}", e)))?;
        &owned
    } else {
        pix
    };

    let depth_bits = pix.depth().bits();
    let spp = pix.spp();

    // For 32bpp, normalize spp to 3 (RGB) if it's not 3 or 4
    let effective_spp = if depth_bits == 32 && spp != 3 && spp != 4 {
        3
    } else {
        spp
    };

    let maxval: u32 = if depth_bits < 32 {
        (1u32 << depth_bits) - 1
    } else {
        255
    };

    // PAM header
    writeln!(writer, "P7").map_err(IoError::Io)?;
    writeln!(writer, "WIDTH {}", width).map_err(IoError::Io)?;
    writeln!(writer, "HEIGHT {}", height).map_err(IoError::Io)?;
    writeln!(writer, "DEPTH {}", effective_spp).map_err(IoError::Io)?;
    writeln!(writer, "MAXVAL {}", maxval).map_err(IoError::Io)?;
    let tupltype = match (effective_spp, depth_bits) {
        (1, 1) => "BLACKANDWHITE",
        (1, _) => "GRAYSCALE",
        (3, _) => "RGB",
        (4, _) => "RGB_ALPHA",
        _ => "GRAYSCALE",
    };
    writeln!(writer, "TUPLTYPE {}", tupltype).map_err(IoError::Io)?;
    writeln!(writer, "ENDHDR").map_err(IoError::Io)?;

    // Pixel data
    match pix.depth() {
        PixelDepth::Bit1 => {
            // 1 byte per pixel; PAM uses white-is-1 photometry (flip leptonica convention)
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y);
                    let byte = (val ^ 1) as u8; // flip: leptonica 1=black → PAM 0=black
                    writer.write_all(&[byte]).map_err(IoError::Io)?;
                }
            }
        }
        PixelDepth::Bit2 => {
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y) as u8;
                    writer.write_all(&[val]).map_err(IoError::Io)?;
                }
            }
        }
        PixelDepth::Bit4 => {
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y) as u8;
                    writer.write_all(&[val]).map_err(IoError::Io)?;
                }
            }
        }
        PixelDepth::Bit8 => {
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y) as u8;
                    writer.write_all(&[val]).map_err(IoError::Io)?;
                }
            }
        }
        PixelDepth::Bit16 => {
            for y in 0..height {
                for x in 0..width {
                    let val = pix.get_pixel_unchecked(x, y) as u16;
                    // PAM 16-bit values are stored big-endian
                    writer.write_all(&val.to_be_bytes()).map_err(IoError::Io)?;
                }
            }
        }
        PixelDepth::Bit32 => match spp {
            3 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel_unchecked(x, y);
                        let (r, g, b) = color::extract_rgb(pixel);
                        writer.write_all(&[r, g, b]).map_err(IoError::Io)?;
                    }
                }
            }
            4 => {
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel_unchecked(x, y);
                        let (r, g, b, a) = color::extract_rgba(pixel);
                        writer.write_all(&[r, g, b, a]).map_err(IoError::Io)?;
                    }
                }
            }
            _ => {
                // Default: treat as RGB
                for y in 0..height {
                    for x in 0..width {
                        let pixel = pix.get_pixel_unchecked(x, y);
                        let (r, g, b) = color::extract_rgb(pixel);
                        writer.write_all(&[r, g, b]).map_err(IoError::Io)?;
                    }
                }
            }
        },
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
                    let (r, g, b) = if let Some(cm) = &cmap {
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
    fn test_write_pnm_ascii_1bpp() {
        let pix = Pix::new(8, 4, PixelDepth::Bit1).unwrap();
        let mut buf = Vec::new();
        write_pnm_ascii(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P1"));
        // P1 output should be readable back
        let pix2 = read_pnm(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 8);
        assert_eq!(pix2.height(), 4);
        assert_eq!(pix2.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn test_write_pnm_ascii_8bpp() {
        let pix = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..4 {
            for x in 0..4u32 {
                pix_mut.set_pixel_unchecked(x, y, x * 60 + y * 15);
            }
        }
        let pix: Pix = pix_mut.into();
        let mut buf = Vec::new();
        write_pnm_ascii(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P2"));
        let pix2 = read_pnm(std::io::Cursor::new(&buf)).unwrap();
        for y in 0..4 {
            for x in 0..4u32 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_write_pnm_ascii_32bpp() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_rgb(0, 0, 255, 0, 0).unwrap();
        let pix: Pix = pix_mut.into();
        let mut buf = Vec::new();
        write_pnm_ascii(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P3"));
        let pix2 = read_pnm(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.get_rgb(0, 0), Some((255, 0, 0)));
    }

    #[test]
    fn test_pam_roundtrip_8bpp() {
        let pix = Pix::new(10, 8, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        for y in 0..8 {
            for x in 0..10u32 {
                pix_mut.set_pixel_unchecked(x, y, (x + y * 10) % 256);
            }
        }
        let pix: Pix = pix_mut.into();
        let mut buf = Vec::new();
        write_pam(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P7"));
        let pix2 = read_pam(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.width(), 10);
        assert_eq!(pix2.height(), 8);
        assert_eq!(pix2.depth(), PixelDepth::Bit8);
        for y in 0..8 {
            for x in 0..10u32 {
                assert_eq!(pix2.get_pixel(x, y), pix.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_pam_roundtrip_rgb() {
        let pix = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(3);
        pix_mut.set_rgb(0, 0, 255, 128, 64).unwrap();
        let pix: Pix = pix_mut.into();
        let mut buf = Vec::new();
        write_pam(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P7"));
        let pix2 = read_pam(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.get_rgb(0, 0), Some((255, 128, 64)));
    }

    #[test]
    fn test_pam_roundtrip_rgba() {
        let pix = Pix::new(4, 4, PixelDepth::Bit32).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_spp(4);
        pix_mut.set_rgba(1, 1, 100, 150, 200, 128).unwrap();
        let pix: Pix = pix_mut.into();
        let mut buf = Vec::new();
        write_pam(&pix, &mut buf).unwrap();
        assert!(buf.starts_with(b"P7"));
        let pix2 = read_pam(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(pix2.spp(), 4);
        assert_eq!(pix2.get_rgba(1, 1), Some((100, 150, 200, 128)));
    }

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
