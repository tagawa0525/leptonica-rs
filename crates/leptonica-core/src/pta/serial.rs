//! Serialization for Pta and Ptaa
//!
//! Text-based serialization format compatible with C Leptonica.
//!
//! # Pta format
//!
//! ```text
//! \n Pta Version 1\n
//!  Number of pts = N; format = float\n
//!    (x0, y0)\n
//!    (x1, y1)\n
//!    ...
//! ```
//!
//! # Ptaa format
//!
//! ```text
//! \nPtaa Version 1\n
//! Number of Pta = N\n
//! [embedded pta 0]
//! [embedded pta 1]
//! ...
//! ```
//!
//! # See also
//!
//! C Leptonica: `ptabasic.c` (`ptaReadStream`, `ptaWriteStream`,
//! `ptaaReadStream`, `ptaaWriteStream`)

use crate::error::{Error, Result};
use crate::pta::{Pta, Ptaa};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Pta/Ptaa serialization format version (matches C Leptonica PTA_VERSION_NUMBER)
const PTA_VERSION: i32 = 1;

/// Maximum number of points in a Pta.
const MAX_PTA_SIZE: usize = 100_000_000;

/// Maximum number of Pta in a Ptaa.
const MAX_PTAA_SIZE: usize = 10_000_000;

/// Maximum input size in bytes to prevent unbounded memory growth.
const MAX_INPUT_SIZE: usize = 100_000_000;

// ============================================================================
// Pta serialization
// ============================================================================

impl Pta {
    /// Read a Pta from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaReadStream()`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Pta from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaRead()`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Pta from a byte slice.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaReadMem()`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_pta(&mut lines)
    }

    /// Write a Pta to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaWriteStream()`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        write_pta(writer, self)
    }

    /// Write a Pta to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaWrite()`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Pta to a byte vector.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaWriteMem()`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// Ptaa serialization
// ============================================================================

impl Ptaa {
    /// Read a Ptaa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaReadStream()`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Ptaa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaRead()`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Ptaa from a byte slice.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaReadMem()`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_ptaa(&mut lines)
    }

    /// Write a Ptaa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaWriteStream()`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let n = self.len();
        writeln!(writer, "\nPtaa Version {PTA_VERSION}")?;
        writeln!(writer, "Number of Pta = {n}")?;

        for pta in self.iter() {
            write_pta(writer, pta)?;
        }

        Ok(())
    }

    /// Write a Ptaa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaWrite()`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Ptaa to a byte vector.
    ///
    /// # See also
    ///
    /// C Leptonica: `ptaaWriteMem()`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Read from a reader with a size limit, returning a clear error if exceeded.
fn read_limited(reader: &mut impl Read) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader
        .take((MAX_INPUT_SIZE + 1) as u64)
        .read_to_end(&mut buf)?;
    if buf.len() > MAX_INPUT_SIZE {
        return Err(Error::DecodeError(format!(
            "input too large: exceeds maximum allowed size of {MAX_INPUT_SIZE} bytes"
        )));
    }
    Ok(buf)
}

/// Write a Pta in the C Leptonica text format (always float).
fn write_pta(writer: &mut impl Write, pta: &Pta) -> Result<()> {
    let n = pta.len();
    writeln!(writer, "\n Pta Version {PTA_VERSION}")?;
    writeln!(writer, " Number of pts = {n}; format = float")?;

    for (x, y) in pta.iter() {
        writeln!(writer, "   ({x}, {y})")?;
    }

    Ok(())
}

/// Parse a Pta from a line iterator.
fn parse_pta<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Result<Pta> {
    // Find and parse version line: " Pta Version N"
    let version = find_and_parse_version(lines, "Pta Version ")?;
    if version != PTA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Pta version: {version}"
        )));
    }

    // Parse count and format line: " Number of pts = N; format = float|integer"
    let (n, is_integer) = parse_pts_header(lines)?;

    // Parse point values
    let mut pta = Pta::with_capacity(n);
    for _ in 0..n {
        let (x, y) = parse_point_line(lines, is_integer)?;
        pta.push(x, y);
    }

    Ok(pta)
}

/// Parse a Ptaa from a line iterator.
fn parse_ptaa<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Result<Ptaa> {
    // Find and parse version line: "Ptaa Version N"
    let version = find_and_parse_version(lines, "Ptaa Version ")?;
    if version != PTA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Ptaa version: {version}"
        )));
    }

    // Parse count: "Number of Pta = N"
    let n_i32 = find_and_parse_int(lines, "Number of Pta = ")?;
    if n_i32 < 0 {
        return Err(Error::DecodeError(format!("invalid Pta count: {n_i32}")));
    }
    let n = n_i32 as usize;
    if n > MAX_PTAA_SIZE {
        return Err(Error::DecodeError(format!("too many Pta: {n}")));
    }

    let mut ptaa = Ptaa::with_capacity(n);
    for _ in 0..n {
        let pta = parse_pta(lines)?;
        ptaa.push(pta);
    }

    Ok(ptaa)
}

/// Find a line containing `prefix` and parse the integer value after it.
fn find_and_parse_version<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    prefix: &str,
) -> Result<i32> {
    find_and_parse_int(lines, prefix)
}

/// Find a line containing `prefix` and parse the integer after it.
fn find_and_parse_int<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    prefix: &str,
) -> Result<i32> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.trim().parse::<i32>().map_err(|e| {
                Error::DecodeError(format!("failed to parse integer after '{prefix}': {e}"))
            });
        }
    }
    Err(Error::DecodeError(format!(
        "expected line with '{prefix}' not found"
    )))
}

/// Parse the Pta header line: "Number of pts = N; format = float|integer"
///
/// Returns (count, is_integer).
fn parse_pts_header<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<(usize, bool)> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Number of pts = ") {
            // Parse "N; format = float|integer"
            let parts: Vec<&str> = rest.splitn(2, ';').collect();
            if parts.is_empty() {
                return Err(Error::DecodeError(
                    "malformed pts header: missing count".into(),
                ));
            }

            let n_i32 = parts[0]
                .trim()
                .parse::<i32>()
                .map_err(|e| Error::DecodeError(format!("failed to parse point count: {e}")))?;
            if n_i32 < 0 {
                return Err(Error::DecodeError(format!("invalid point count: {n_i32}")));
            }
            let n = n_i32 as usize;
            if n > MAX_PTA_SIZE {
                return Err(Error::DecodeError(format!("too many points: {n}")));
            }

            let is_integer = if parts.len() >= 2 {
                parts[1].trim().contains("integer")
            } else {
                false
            };

            return Ok((n, is_integer));
        }
    }
    Err(Error::DecodeError(
        "expected 'Number of pts' line not found".into(),
    ))
}

/// Parse a point line: "   (x, y)" in float or integer format.
fn parse_point_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    is_integer: bool,
) -> Result<(f32, f32)> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        // Look for "(x, y)" pattern
        if let Some(inner) = trimmed.strip_prefix('(')
            && let Some(inner) = inner.strip_suffix(')')
        {
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(Error::DecodeError(format!(
                    "malformed point line: expected (x, y), got '{trimmed}'"
                )));
            }
            let x =
                if is_integer {
                    parts[0].trim().parse::<i32>().map_err(|e| {
                        Error::DecodeError(format!("failed to parse integer x: {e}"))
                    })? as f32
                } else {
                    parts[0]
                        .trim()
                        .parse::<f32>()
                        .map_err(|e| Error::DecodeError(format!("failed to parse float x: {e}")))?
                };
            let y =
                if is_integer {
                    parts[1].trim().parse::<i32>().map_err(|e| {
                        Error::DecodeError(format!("failed to parse integer y: {e}"))
                    })? as f32
                } else {
                    parts[1]
                        .trim()
                        .parse::<f32>()
                        .map_err(|e| Error::DecodeError(format!("failed to parse float y: {e}")))?
                };
            return Ok((x, y));
        }
    }
    Err(Error::DecodeError(
        "expected point line '(x, y)' not found".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Pta serialization tests
    // ========================================================================

    #[test]
    fn test_pta_roundtrip() {
        let pta: Pta = [(1.5, 2.5), (3.0, -4.0), (0.0, 100.0)]
            .into_iter()
            .collect();

        let bytes = pta.write_to_bytes().unwrap();
        let restored = Pta::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), pta.len());
        for i in 0..pta.len() {
            let (x1, y1) = pta.get(i).unwrap();
            let (x2, y2) = restored.get(i).unwrap();
            assert!((x1 - x2).abs() < 1e-4);
            assert!((y1 - y2).abs() < 1e-4);
        }
    }

    #[test]
    fn test_pta_roundtrip_empty() {
        let pta = Pta::new();
        let bytes = pta.write_to_bytes().unwrap();
        let restored = Pta::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_pta_write_format() {
        let pta: Pta = [(1.5, 2.0)].into_iter().collect();

        let bytes = pta.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Pta Version 1"));
        assert!(text.contains("Number of pts = 1; format = float"));
        assert!(text.contains("("));
    }

    #[test]
    fn test_pta_reader_roundtrip() {
        let pta: Pta = [(100.0, 200.0), (300.0, 400.0)].into_iter().collect();
        let mut buf = Vec::new();
        pta.write_to_writer(&mut buf).unwrap();

        let restored = Pta::read_from_reader(&mut &buf[..]).unwrap();
        assert_eq!(restored.len(), 2);
        let (x, y) = restored.get(0).unwrap();
        assert!((x - 100.0).abs() < 1e-4);
        assert!((y - 200.0).abs() < 1e-4);
    }

    #[test]
    fn test_pta_file_roundtrip() {
        let pta: Pta = [(1.0, 2.0), (3.0, 4.0)].into_iter().collect();

        let dir = std::env::temp_dir().join("leptonica_test_pta");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_pta.txt");

        pta.write_to_file(&path).unwrap();
        let restored = Pta::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);
        let (x, y) = restored.get(0).unwrap();
        assert!((x - 1.0).abs() < 1e-4);
        assert!((y - 2.0).abs() < 1e-4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_pta_integer_format_read() {
        // C Leptonica can write integer format; we should read it
        let input =
            b"\n Pta Version 1\n Number of pts = 2; format = integer\n   (10, 20)\n   (30, 40)\n";
        let pta = Pta::read_from_bytes(input).unwrap();
        assert_eq!(pta.len(), 2);
        let (x, y) = pta.get(0).unwrap();
        assert!((x - 10.0).abs() < 1e-4);
        assert!((y - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_pta_invalid_data() {
        let result = Pta::read_from_bytes(b"not valid data");
        assert!(result.is_err());

        let result = Pta::read_from_bytes(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_pta_negative_count_rejected() {
        let input = b"\n Pta Version 1\n Number of pts = -1; format = float\n";
        let result = Pta::read_from_bytes(input);
        assert!(result.is_err());
    }

    // ========================================================================
    // Ptaa serialization tests
    // ========================================================================

    #[test]
    fn test_ptaa_roundtrip() {
        let mut ptaa = Ptaa::new();
        ptaa.push([(1.0, 2.0), (3.0, 4.0)].into_iter().collect());
        ptaa.push([(5.0, 6.0)].into_iter().collect());

        let bytes = ptaa.write_to_bytes().unwrap();
        let restored = Ptaa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get(0).unwrap().len(), 2);
        assert_eq!(restored.get(1).unwrap().len(), 1);

        let (x, y) = restored.get(0).unwrap().get(0).unwrap();
        assert!((x - 1.0).abs() < 1e-4);
        assert!((y - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_ptaa_roundtrip_empty() {
        let ptaa = Ptaa::new();
        let bytes = ptaa.write_to_bytes().unwrap();
        let restored = Ptaa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_ptaa_write_format() {
        let mut ptaa = Ptaa::new();
        ptaa.push([(1.0, 2.0)].into_iter().collect());
        ptaa.push([(3.0, 4.0)].into_iter().collect());

        let bytes = ptaa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Ptaa Version 1"));
        assert!(text.contains("Number of Pta = 2"));
    }

    #[test]
    fn test_ptaa_file_roundtrip() {
        let mut ptaa = Ptaa::new();
        ptaa.push([(1.0, 2.0)].into_iter().collect());
        ptaa.push([(3.0, 4.0)].into_iter().collect());

        let dir = std::env::temp_dir().join("leptonica_test_ptaa");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_ptaa.txt");

        ptaa.write_to_file(&path).unwrap();
        let restored = Ptaa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ptaa_negative_count_rejected() {
        let input = b"\nPtaa Version 1\nNumber of Pta = -1\n";
        let result = Ptaa::read_from_bytes(input);
        assert!(result.is_err());
    }
}
