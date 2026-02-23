//! Serialization for Numa and Numaa
//!
//! Text-based serialization format compatible with C Leptonica.
//!
//! # Numa format
//!
//! ```text
//! \nNuma Version 1\n
//! Number of numbers = N\n
//!   [0] = <value>\n
//!   [1] = <value>\n
//!   ...
//! \nstartx = <value>, delx = <value>\n   (only if non-default)
//! ```
//!
//! # Numaa format
//!
//! ```text
//! \nNumaa Version 1\n
//! Number of numa = N\n\n
//! Numa[0]:
//! [embedded numa]
//! Numa[1]:
//! [embedded numa]
//! ...
//! ```
//!
//! # See also
//!
//! C Leptonica: `numabasic.c` (`numaReadStream`, `numaWriteStream`,
//! `numaaReadStream`, `numaaWriteStream`)

use crate::error::{Error, Result};
use crate::numa::{Numa, Numaa};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Numa/Numaa serialization format version (matches C Leptonica NUMA_VERSION_NUMBER)
const NUMA_VERSION: i32 = 1;

/// Maximum number of values in a Numa.
const MAX_NUMA_SIZE: usize = 10_000_000;

/// Maximum number of Numa in a Numaa.
const MAX_NUMAA_SIZE: usize = 1_000_000;

/// Maximum input size in bytes to prevent unbounded memory growth.
const MAX_INPUT_SIZE: usize = 100_000_000;

// ============================================================================
// Numa serialization
// ============================================================================

impl Numa {
    /// Read a Numa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaReadStream()`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Numa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaRead()`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Numa from a byte slice.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaReadMem()`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_numa(&mut lines)
    }

    /// Write a Numa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaWriteStream()`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        write_numa(writer, self)
    }

    /// Write a Numa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaWrite()`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Numa to a byte vector.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaWriteMem()`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// Numaa serialization
// ============================================================================

impl Numaa {
    /// Read a Numaa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaReadStream()`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Numaa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaRead()`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Numaa from a byte slice.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaReadMem()`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_numaa(&mut lines)
    }

    /// Write a Numaa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaWriteStream()`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let n = self.len();
        writeln!(writer, "\nNumaa Version {NUMA_VERSION}")?;
        writeln!(writer, "Number of numa = {n}\n")?;

        for (i, numa) in self.iter().enumerate() {
            write!(writer, "Numa[{i}]:")?;
            write_numa(writer, numa)?;
        }

        Ok(())
    }

    /// Write a Numaa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaWrite()`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Numaa to a byte vector.
    ///
    /// # See also
    ///
    /// C Leptonica: `numaaWriteMem()`
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

/// Write a Numa in the C Leptonica text format.
fn write_numa(writer: &mut impl Write, numa: &Numa) -> Result<()> {
    let n = numa.len();
    writeln!(writer, "\nNuma Version {NUMA_VERSION}")?;
    writeln!(writer, "Number of numbers = {n}")?;

    for (i, val) in numa.iter().enumerate() {
        writeln!(writer, "  [{i}] = {val:.6}")?;
    }

    // Write startx/delx only if non-default (C Leptonica behavior)
    let (startx, delx) = numa.parameters();
    if startx != 0.0 || delx != 1.0 {
        writeln!(writer, "\nstartx = {startx:.6}, delx = {delx:.6}")?;
    }

    Ok(())
}

/// Parse a Numa from a line iterator.
fn parse_numa<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Result<Numa> {
    // Find and parse version line
    let version = find_and_parse_int(lines, "Numa Version ")?;
    if version != NUMA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Numa version: {version}"
        )));
    }

    // Parse count (validate non-negative before usize cast)
    let n_i32 = find_and_parse_int(lines, "Number of numbers = ")?;
    if n_i32 < 0 {
        return Err(Error::DecodeError(format!("invalid number count: {n_i32}")));
    }
    let n = n_i32 as usize;
    if n > MAX_NUMA_SIZE {
        return Err(Error::DecodeError(format!("too many numbers: {n}")));
    }

    // Parse values: "  [i] = <float>"
    let mut data = Vec::with_capacity(n);
    for _ in 0..n {
        let val = parse_value_line(lines)?;
        data.push(val);
    }

    // Try to parse optional startx/delx line
    let (startx, delx) = parse_params(lines)?;

    let mut numa = Numa::from_vec(data);
    numa.set_parameters(startx, delx);
    Ok(numa)
}

/// Parse a Numaa from a line iterator.
fn parse_numaa<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Numaa> {
    // Find and parse version line
    let version = find_and_parse_int(lines, "Numaa Version ")?;
    if version != NUMA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Numaa version: {version}"
        )));
    }

    // Parse count (validate non-negative before usize cast)
    let n_i32 = find_and_parse_int(lines, "Number of numa = ")?;
    if n_i32 < 0 {
        return Err(Error::DecodeError(format!("invalid numa count: {n_i32}")));
    }
    let n = n_i32 as usize;
    if n > MAX_NUMAA_SIZE {
        return Err(Error::DecodeError(format!("too many numa: {n}")));
    }

    let mut numaa = Numaa::with_capacity(n);
    for _ in 0..n {
        // Skip "Numa[i]:" label line
        skip_until_prefix(lines, "Numa[")?;
        // Parse the embedded Numa
        let numa = parse_numa(lines)?;
        numaa.push(numa);
    }

    Ok(numaa)
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

/// Skip lines until one starting with `prefix` is found.
///
/// Returns an error if EOF is reached without finding the prefix.
fn skip_until_prefix<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    prefix: &str,
) -> Result<()> {
    for line in lines.by_ref() {
        if line.trim().starts_with(prefix) {
            return Ok(());
        }
    }
    Err(Error::DecodeError(format!(
        "expected line starting with '{prefix}' not found"
    )))
}

/// Parse a value line like "  [0] = 1.500000"
fn parse_value_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<f32> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        // Look for pattern "[N] = <value>"
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(after_bracket) = rest.split_once(']')
        {
            let value_part = after_bracket.1.trim();
            if let Some(val_str) = value_part.strip_prefix("= ") {
                return val_str
                    .trim()
                    .parse::<f32>()
                    .map_err(|e| Error::DecodeError(format!("failed to parse float value: {e}")));
            } else if let Some(val_str) = value_part.strip_prefix('=') {
                return val_str
                    .trim()
                    .parse::<f32>()
                    .map_err(|e| Error::DecodeError(format!("failed to parse float value: {e}")));
            }
        }
    }
    Err(Error::DecodeError(
        "expected value line '[N] = <value>' not found".into(),
    ))
}

/// Try to parse the optional "startx = <val>, delx = <val>" line.
///
/// Uses peek to avoid consuming lines that belong to the next section
/// (important when embedded in a Numaa). Returns defaults (0.0, 1.0) if
/// the next non-empty line doesn't start with "startx". Returns an error
/// if the startx line exists but contains unparseable values.
fn parse_params<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<(f32, f32)> {
    // Skip blank lines, but don't consume non-matching content lines
    while let Some(&line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.next();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("startx") {
            lines.next(); // Consume the startx line
            let rest = rest.trim_start_matches([' ', '=']);
            let parts: Vec<&str> = rest.split(',').collect();
            if parts.len() >= 2 {
                let startx = parts[0].trim().parse::<f32>().map_err(|e| {
                    Error::DecodeError(format!(
                        "failed to parse startx value '{}': {e}",
                        parts[0].trim()
                    ))
                })?;
                let delx_part = parts[1].trim();
                let delx_val = delx_part
                    .strip_prefix("delx")
                    .unwrap_or(delx_part)
                    .trim_start_matches([' ', '='])
                    .trim();
                let delx = delx_val.parse::<f32>().map_err(|e| {
                    Error::DecodeError(format!("failed to parse delx value '{delx_val}': {e}"))
                })?;
                return Ok((startx, delx));
            }
        }
        break; // Non-empty, non-startx line → stop without consuming
    }
    Ok((0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Numa serialization tests
    // ========================================================================

    #[test]
    fn test_numa_roundtrip() {
        let mut numa = Numa::from_vec(vec![1.5, 2.5, 3.5, -4.0, 0.0]);
        numa.set_parameters(10.0, 0.5);

        let bytes = numa.write_to_bytes().unwrap();
        let restored = Numa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), numa.len());
        for i in 0..numa.len() {
            assert!((restored[i] - numa[i]).abs() < 1e-4);
        }
        let (startx, delx) = restored.parameters();
        assert!((startx - 10.0).abs() < 1e-4);
        assert!((delx - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_numa_roundtrip_default_params() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        let bytes = numa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes.clone()).unwrap();

        let restored = Numa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 3);
        let (startx, delx) = restored.parameters();
        assert!((startx - 0.0).abs() < 1e-4);
        assert!((delx - 1.0).abs() < 1e-4);
        // Default params should not be written
        assert!(!text.contains("startx"));
    }

    #[test]
    fn test_numa_roundtrip_empty() {
        let numa = Numa::new();
        let bytes = numa.write_to_bytes().unwrap();
        let restored = Numa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_numa_write_format() {
        let numa = Numa::from_vec(vec![1.5, 2.0]);

        let bytes = numa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Numa Version 1"));
        assert!(text.contains("Number of numbers = 2"));
        assert!(text.contains("[0] ="));
        assert!(text.contains("[1] ="));
    }

    #[test]
    fn test_numa_reader_roundtrip() {
        let numa = Numa::from_vec(vec![100.0, 200.0, 300.0]);
        let mut buf = Vec::new();
        numa.write_to_writer(&mut buf).unwrap();

        let restored = Numa::read_from_reader(&mut &buf[..]).unwrap();
        assert_eq!(restored.len(), 3);
        assert!((restored[0] - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_numa_file_roundtrip() {
        let numa = Numa::from_vec(vec![1.0, 2.0, 3.0]);

        let dir = std::env::temp_dir().join("leptonica_test_numa");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_numa.txt");

        numa.write_to_file(&path).unwrap();
        let restored = Numa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 3);
        assert!((restored[0] - 1.0).abs() < 1e-4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_numa_invalid_data() {
        let result = Numa::read_from_bytes(b"not valid data");
        assert!(result.is_err());

        let result = Numa::read_from_bytes(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_numa_malformed_params_rejected() {
        let input =
            b"\nNuma Version 1\nNumber of numbers = 1\n  [0] = 1.0\n\nstartx = abc, delx = 0.5\n";
        let result = Numa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("startx"), "error should mention startx: {err}");
    }

    #[test]
    fn test_numa_negative_count_rejected() {
        let input = b"\nNuma Version 1\nNumber of numbers = -1\n";
        let result = Numa::read_from_bytes(input);
        assert!(result.is_err());
    }

    // ========================================================================
    // Numaa serialization tests
    // ========================================================================

    #[test]
    fn test_numaa_roundtrip() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0, 2.0, 3.0]));
        let mut numa2 = Numa::from_vec(vec![4.0, 5.0]);
        numa2.set_parameters(1.0, 0.25);
        numaa.push(numa2);

        let bytes = numaa.write_to_bytes().unwrap();
        let restored = Numaa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].len(), 3);
        assert_eq!(restored[1].len(), 2);
        assert!((restored[0][0] - 1.0).abs() < 1e-4);
        assert!((restored[1][1] - 5.0).abs() < 1e-4);

        let (startx, delx) = restored[1].parameters();
        assert!((startx - 1.0).abs() < 1e-4);
        assert!((delx - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_numaa_roundtrip_empty() {
        let numaa = Numaa::new();
        let bytes = numaa.write_to_bytes().unwrap();
        let restored = Numaa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_numaa_write_format() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0]));
        numaa.push(Numa::from_vec(vec![2.0]));

        let bytes = numaa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Numaa Version 1"));
        assert!(text.contains("Number of numa = 2"));
        assert!(text.contains("Numa[0]:"));
        assert!(text.contains("Numa[1]:"));
    }

    #[test]
    fn test_numaa_file_roundtrip() {
        let mut numaa = Numaa::new();
        numaa.push(Numa::from_vec(vec![1.0, 2.0]));
        numaa.push(Numa::from_vec(vec![3.0]));

        let dir = std::env::temp_dir().join("leptonica_test_numaa");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_numaa.txt");

        numaa.write_to_file(&path).unwrap();
        let restored = Numaa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].len(), 2);
        assert_eq!(restored[1].len(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_numaa_negative_count_rejected() {
        let input = b"\nNumaa Version 1\nNumber of numa = -1\n";
        let result = Numaa::read_from_bytes(input);
        assert!(result.is_err());
    }
}
