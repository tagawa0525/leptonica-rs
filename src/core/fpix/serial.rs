//! Serialization for FPix and DPix
//!
//! Mixed text header + binary data format compatible with C Leptonica.
//!
//! # FPix format
//!
//! ```text
//! \nFPix Version 2\n
//! w = W, h = H, nbytes = N\n
//! xres = X, yres = Y\n
//! <raw f32 data, little-endian, N bytes>
//! \n
//! ```
//!
//! # DPix format
//!
//! ```text
//! \nDPix Version 2\n
//! w = W, h = H, nbytes = N\n
//! xres = X, yres = Y\n
//! <raw f64 data, little-endian, N bytes>
//! \n
//! ```
//!
//! # See also
//!
//! C Leptonica: `fpix1.c` (`fpixReadStream`, `fpixWriteStream`,
//! `dpixReadStream`, `dpixWriteStream`)

use crate::error::{Error, Result};
use crate::fpix::{DPix, FPix};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// FPix format version (matches C Leptonica FPIX_VERSION_NUMBER)
const FPIX_VERSION: i32 = 2;

/// DPix format version (matches C Leptonica DPIX_VERSION_NUMBER)
const DPIX_VERSION: i32 = 2;

/// Maximum pixel count for FPix (2^29)
const MAX_FPIX_PIXELS: u64 = 1 << 29;

/// Maximum pixel count for DPix (2^28)
const MAX_DPIX_PIXELS: u64 = 1 << 28;

/// Maximum input size in bytes.
const MAX_INPUT_SIZE: u64 = 500_000_000;

// ============================================================================
// FPix serialization
// ============================================================================

impl FPix {
    /// Read an FPix from a reader.
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let mut buf = Vec::new();
        reader.take(MAX_INPUT_SIZE + 1).read_to_end(&mut buf)?;
        if buf.len() as u64 > MAX_INPUT_SIZE {
            return Err(Error::DecodeError(format!(
                "input too large: exceeds maximum allowed size of {MAX_INPUT_SIZE} bytes"
            )));
        }
        Self::read_from_bytes(&buf)
    }

    /// Read an FPix from a file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        Self::read_from_reader(&mut BufReader::new(file))
    }

    /// Read an FPix from a byte slice.
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        // Find end of text header (3 lines after the leading newline).
        // Format: \nFPix Version 2\nw = W, h = H, nbytes = N\nxres = X, yres = Y\n<binary>
        let (w, h, nbytes, xres, yres, header_end) =
            parse_binary_pix_header(data, "FPix", FPIX_VERSION)?;

        // Validate dimensions
        let npixels = w as u64 * h as u64;
        if npixels > MAX_FPIX_PIXELS {
            return Err(Error::DecodeError(format!(
                "FPix too large: {npixels} pixels exceeds maximum {MAX_FPIX_PIXELS}"
            )));
        }
        let expected_nbytes = npixels * 4;
        if nbytes != expected_nbytes {
            return Err(Error::DecodeError(format!(
                "FPix nbytes mismatch: header says {nbytes} but w*h*4 = {expected_nbytes}"
            )));
        }

        // Read binary data
        let binary_start = header_end;
        let binary_end = binary_start + nbytes as usize;
        if data.len() < binary_end {
            return Err(Error::DecodeError(format!(
                "FPix data truncated: need {binary_end} bytes but only have {}",
                data.len()
            )));
        }
        let binary = &data[binary_start..binary_end];

        // Convert little-endian bytes to f32 values
        let pixel_data: Vec<f32> = binary
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(FPix {
            width: w,
            height: h,
            data: pixel_data,
            xres,
            yres,
        })
    }

    /// Write an FPix to a writer.
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let nbytes = (self.width as u64) * (self.height as u64) * 4;
        // Text header
        writeln!(writer, "\nFPix Version {FPIX_VERSION}")?;
        writeln!(
            writer,
            "w = {}, h = {}, nbytes = {nbytes}",
            self.width, self.height
        )?;
        writeln!(writer, "xres = {}, yres = {}", self.xres, self.yres)?;

        // Binary data (little-endian)
        for &val in &self.data {
            writer.write_all(&val.to_le_bytes())?;
        }

        // Trailing newline
        writeln!(writer)?;
        Ok(())
    }

    /// Write an FPix to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write an FPix to a byte vector.
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// DPix serialization
// ============================================================================

impl DPix {
    /// Read a DPix from a reader.
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let mut buf = Vec::new();
        reader.take(MAX_INPUT_SIZE + 1).read_to_end(&mut buf)?;
        if buf.len() as u64 > MAX_INPUT_SIZE {
            return Err(Error::DecodeError(format!(
                "input too large: exceeds maximum allowed size of {MAX_INPUT_SIZE} bytes"
            )));
        }
        Self::read_from_bytes(&buf)
    }

    /// Read a DPix from a file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        Self::read_from_reader(&mut BufReader::new(file))
    }

    /// Read a DPix from a byte slice.
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let (w, h, nbytes, xres, yres, header_end) =
            parse_binary_pix_header(data, "DPix", DPIX_VERSION)?;

        let npixels = w as u64 * h as u64;
        if npixels > MAX_DPIX_PIXELS {
            return Err(Error::DecodeError(format!(
                "DPix too large: {npixels} pixels exceeds maximum {MAX_DPIX_PIXELS}"
            )));
        }
        let expected_nbytes = npixels * 8;
        if nbytes != expected_nbytes {
            return Err(Error::DecodeError(format!(
                "DPix nbytes mismatch: header says {nbytes} but w*h*8 = {expected_nbytes}"
            )));
        }

        let binary_start = header_end;
        let binary_end = binary_start + nbytes as usize;
        if data.len() < binary_end {
            return Err(Error::DecodeError(format!(
                "DPix data truncated: need {binary_end} bytes but only have {}",
                data.len()
            )));
        }
        let binary = &data[binary_start..binary_end];

        let pixel_data: Vec<f64> = binary
            .chunks_exact(8)
            .map(|chunk| {
                f64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ])
            })
            .collect();

        Ok(DPix {
            width: w,
            height: h,
            data: pixel_data,
            xres,
            yres,
        })
    }

    /// Write a DPix to a writer.
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let nbytes = (self.width as u64) * (self.height as u64) * 8;
        writeln!(writer, "\nDPix Version {DPIX_VERSION}")?;
        writeln!(
            writer,
            "w = {}, h = {}, nbytes = {nbytes}",
            self.width, self.height
        )?;
        writeln!(writer, "xres = {}, yres = {}", self.xres, self.yres)?;

        for &val in &self.data {
            writer.write_all(&val.to_le_bytes())?;
        }

        writeln!(writer)?;
        Ok(())
    }

    /// Write a DPix to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a DPix to a byte vector.
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Parse FPix/DPix text header from raw bytes.
///
/// The header is pure ASCII text followed by binary data. We scan for
/// newlines byte-by-byte to extract exactly the text header portion,
/// then parse it as UTF-8. Returns (w, h, nbytes, xres, yres, header_end_offset).
fn parse_binary_pix_header(
    data: &[u8],
    type_name: &str,
    expected_version: i32,
) -> Result<(u32, u32, u64, i32, i32, usize)> {
    // The header has exactly 3 text lines (after a leading \n):
    //   \n<Type> Version V\n
    //   w = W, h = H, nbytes = N\n
    //   xres = X, yres = Y\n
    // Binary data follows immediately after the 3rd line's \n.

    // Find header end by counting newlines from the start.
    // We need to find the newline after "yres = ..." line.
    // Strategy: find 3 non-empty lines after the leading newline(s).
    let header_end = find_header_end_by_lines(data)?;
    let header_bytes = &data[..header_end];
    let header_text = std::str::from_utf8(header_bytes)
        .map_err(|e| Error::DecodeError(format!("{type_name} header is not valid UTF-8: {e}")))?;

    let lines: Vec<&str> = header_text.lines().collect();

    // Find version line
    let version_prefix = format!("{type_name} Version ");
    let version_line = lines
        .iter()
        .find(|l| l.trim().starts_with(&version_prefix))
        .ok_or_else(|| Error::DecodeError(format!("{type_name} version line not found")))?;
    let version: i32 = version_line
        .trim()
        .strip_prefix(&version_prefix)
        .unwrap()
        .trim()
        .parse()
        .map_err(|e| Error::DecodeError(format!("failed to parse {type_name} version: {e}")))?;
    if version != expected_version {
        return Err(Error::DecodeError(format!(
            "invalid {type_name} version: {version}"
        )));
    }

    // Find dimensions line: "w = W, h = H, nbytes = N"
    let dim_line = lines
        .iter()
        .find(|l| l.trim().starts_with("w = "))
        .ok_or_else(|| Error::DecodeError(format!("{type_name} dimension line not found")))?;
    let (w, h, nbytes) = parse_dim_line(dim_line)?;

    // Find resolution line: "xres = X, yres = Y"
    let res_line = lines
        .iter()
        .find(|l| l.trim().starts_with("xres = "))
        .ok_or_else(|| Error::DecodeError(format!("{type_name} resolution line not found")))?;
    let (xres, yres) = parse_res_line(res_line)?;

    Ok((w, h, nbytes, xres, yres, header_end))
}

/// Find the byte offset where binary data begins by scanning for text header lines.
///
/// The header format has 3 content lines (version, dimensions, resolution),
/// possibly preceded by empty lines. We find 3 non-empty lines and return
/// the byte offset right after the last one's newline.
fn find_header_end_by_lines(data: &[u8]) -> Result<usize> {
    let scan_limit = data.len().min(512);
    let mut content_lines_found = 0;
    let mut pos = 0;

    while pos < scan_limit {
        // Find next newline
        let newline_pos = data[pos..scan_limit].iter().position(|&b| b == b'\n');
        match newline_pos {
            Some(offset) => {
                let line_end = pos + offset;
                let line = &data[pos..line_end];
                // Count non-empty lines (trimming whitespace)
                let trimmed: Vec<u8> = line
                    .iter()
                    .copied()
                    .filter(|&b| b != b' ' && b != b'\r')
                    .collect();
                if !trimmed.is_empty() {
                    content_lines_found += 1;
                    if content_lines_found == 3 {
                        // Binary data starts right after this newline
                        return Ok(line_end + 1);
                    }
                }
                pos = line_end + 1;
            }
            None => break,
        }
    }
    Err(Error::DecodeError(
        "could not find end of text header (expected 3 header lines)".into(),
    ))
}

/// Parse "w = W, h = H, nbytes = N"
fn parse_dim_line(line: &str) -> Result<(u32, u32, u64)> {
    let trimmed = line.trim();
    // Split by commas and parse each part
    let parts: Vec<&str> = trimmed.split(',').collect();
    if parts.len() < 3 {
        return Err(Error::DecodeError(format!(
            "invalid dimension line: '{trimmed}'"
        )));
    }

    let w = parse_key_value_u32(parts[0], "w")?;
    let h = parse_key_value_u32(parts[1], "h")?;
    let nbytes = parse_key_value_u64(parts[2], "nbytes")?;

    if w == 0 || h == 0 {
        return Err(Error::DecodeError(format!(
            "invalid dimensions: w={w}, h={h}"
        )));
    }

    Ok((w, h, nbytes))
}

/// Parse "xres = X, yres = Y"
fn parse_res_line(line: &str) -> Result<(i32, i32)> {
    let trimmed = line.trim();
    let parts: Vec<&str> = trimmed.split(',').collect();
    if parts.len() < 2 {
        return Err(Error::DecodeError(format!(
            "invalid resolution line: '{trimmed}'"
        )));
    }

    let xres = parse_key_value_i32(parts[0], "xres")?;
    let yres = parse_key_value_i32(parts[1], "yres")?;
    Ok((xres, yres))
}

/// Parse "key = value" where value is u32
fn parse_key_value_u32(s: &str, key: &str) -> Result<u32> {
    let trimmed = s.trim();
    let val_str = trimmed
        .split('=')
        .nth(1)
        .ok_or_else(|| Error::DecodeError(format!("missing '=' in {key} field")))?
        .trim();
    val_str
        .parse()
        .map_err(|e| Error::DecodeError(format!("failed to parse {key}: {e}")))
}

/// Parse "key = value" where value is u64
fn parse_key_value_u64(s: &str, key: &str) -> Result<u64> {
    let trimmed = s.trim();
    let val_str = trimmed
        .split('=')
        .nth(1)
        .ok_or_else(|| Error::DecodeError(format!("missing '=' in {key} field")))?
        .trim();
    val_str
        .parse()
        .map_err(|e| Error::DecodeError(format!("failed to parse {key}: {e}")))
}

/// Parse "key = value" where value is i32
fn parse_key_value_i32(s: &str, key: &str) -> Result<i32> {
    let trimmed = s.trim();
    let val_str = trimmed
        .split('=')
        .nth(1)
        .ok_or_else(|| Error::DecodeError(format!("missing '=' in {key} field")))?
        .trim();
    val_str
        .parse()
        .map_err(|e| Error::DecodeError(format!("failed to parse {key}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // FPix serialization tests
    // ========================================================================

    #[test]
    fn test_fpix_roundtrip() {
        let mut fpix = FPix::new(4, 3).unwrap();
        fpix.set_pixel(0, 0, 1.5).unwrap();
        fpix.set_pixel(3, 2, -42.0).unwrap();
        fpix.set_resolution(72, 72);

        let bytes = fpix.write_to_bytes().unwrap();
        let restored = FPix::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.width(), 4);
        assert_eq!(restored.height(), 3);
        assert!((restored.get_pixel(0, 0).unwrap() - 1.5).abs() < 1e-6);
        assert!((restored.get_pixel(3, 2).unwrap() - (-42.0)).abs() < 1e-6);
        let (xres, yres) = restored.resolution();
        assert_eq!(xres, 72);
        assert_eq!(yres, 72);
    }

    #[test]
    fn test_fpix_file_roundtrip() {
        let fpix = FPix::new_with_value(3, 2, 7.5).unwrap();

        let dir = std::env::temp_dir().join("leptonica_test_fpix");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_fpix.dat");

        fpix.write_to_file(&path).unwrap();
        let restored = FPix::read_from_file(&path).unwrap();

        assert_eq!(restored.width(), 3);
        assert_eq!(restored.height(), 2);
        assert!((restored.get_pixel(0, 0).unwrap() - 7.5).abs() < 1e-6);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_fpix_invalid_data() {
        let result = FPix::read_from_bytes(b"not valid data");
        assert!(result.is_err());
    }

    // ========================================================================
    // DPix serialization tests
    // ========================================================================

    #[test]
    fn test_dpix_roundtrip() {
        let mut dpix = DPix::new(3, 2).unwrap();
        dpix.set_pixel(0, 0, 1.5).unwrap();
        dpix.set_pixel(2, 1, -999.125).unwrap();
        dpix.set_resolution(150, 150);

        let bytes = dpix.write_to_bytes().unwrap();
        let restored = DPix::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.width(), 3);
        assert_eq!(restored.height(), 2);
        assert!((restored.get_pixel(0, 0).unwrap() - 1.5).abs() < 1e-10);
        assert!((restored.get_pixel(2, 1).unwrap() - (-999.125)).abs() < 1e-10);
        let (xres, yres) = restored.resolution();
        assert_eq!(xres, 150);
        assert_eq!(yres, 150);
    }

    #[test]
    fn test_dpix_file_roundtrip() {
        let mut dpix = DPix::new(2, 2).unwrap();
        for y in 0..2 {
            for x in 0..2 {
                dpix.set_pixel(x, y, std::f64::consts::FRAC_PI_2).unwrap();
            }
        }

        let dir = std::env::temp_dir().join("leptonica_test_dpix");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_dpix.dat");

        dpix.write_to_file(&path).unwrap();
        let restored = DPix::read_from_file(&path).unwrap();

        assert_eq!(restored.width(), 2);
        assert!((restored.get_pixel(0, 0).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-10);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_dpix_invalid_data() {
        let result = DPix::read_from_bytes(b"not valid data");
        assert!(result.is_err());
    }
}
