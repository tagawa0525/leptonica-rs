//! Serialization for Box, Boxa, and Boxaa
//!
//! Text-based serialization format compatible with C Leptonica.
//!
//! # Boxa format
//!
//! ```text
//! \nBoxa Version 2\n
//! Number of boxes = N\n
//!   Box[0]: x = X, y = Y, w = W, h = H\n
//!   Box[1]: x = X, y = Y, w = W, h = H\n
//!   ...
//! ```
//!
//! # Boxaa format
//!
//! ```text
//! \nBoxaa Version 3\n
//! Number of boxa = N\n
//! \nBoxa[0] extent: x = X, y = Y, w = W, h = H
//! [embedded boxa]
//! \nBoxa[1] extent: x = X, y = Y, w = W, h = H
//! [embedded boxa]
//! ...
//! ```
//!
//! # See also
//!
//! C Leptonica: `boxbasic.c` (`boxaReadStream`, `boxaWriteStream`,
//! `boxaaReadStream`, `boxaaWriteStream`)

use crate::box_::{Box, Boxa, Boxaa};
use crate::error::{Error, Result};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Boxa serialization format version (matches C Leptonica BOXA_VERSION_NUMBER)
const BOXA_VERSION: i32 = 2;
/// Boxaa serialization format version (matches C Leptonica BOXAA_VERSION_NUMBER)
const BOXAA_VERSION: i32 = 3;

/// Maximum number of boxes in a Boxa
const MAX_BOXA_SIZE: usize = 10_000_000;
/// Maximum number of Boxa in a Boxaa
const MAX_BOXAA_SIZE: usize = 1_000_000;

/// Maximum input size in bytes to prevent unbounded memory growth.
/// Generous limit (~100 MB) that accommodates any realistic Boxa/Boxaa data.
const MAX_INPUT_SIZE: usize = 100_000_000;

impl Boxa {
    /// Read a Boxa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaReadStream()` in `boxbasic.c`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let mut buf = String::new();
        reader
            .take(MAX_INPUT_SIZE as u64)
            .read_to_string(&mut buf)?;
        Self::read_from_bytes(buf.as_bytes())
    }

    /// Read a Boxa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaRead()` in `boxbasic.c`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Boxa from bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaReadMem()` in `boxbasic.c`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_boxa(&mut lines)
    }

    /// Write a Boxa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaWriteStream()` in `boxbasic.c`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        write_boxa(writer, self)
    }

    /// Write a Boxa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaWrite()` in `boxbasic.c`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Boxa to bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaWriteMem()` in `boxbasic.c`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

impl Boxaa {
    /// Read a Boxaa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaReadStream()` in `boxbasic.c`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let mut buf = String::new();
        reader
            .take(MAX_INPUT_SIZE as u64)
            .read_to_string(&mut buf)?;
        Self::read_from_bytes(buf.as_bytes())
    }

    /// Read a Boxaa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaRead()` in `boxbasic.c`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Boxaa from bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaReadMem()` in `boxbasic.c`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();
        parse_boxaa(&mut lines)
    }

    /// Write a Boxaa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaWriteStream()` in `boxbasic.c`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let n = self.len();
        writeln!(writer, "\nBoxaa Version {BOXAA_VERSION}")?;
        writeln!(writer, "Number of boxa = {n}")?;

        for (i, boxa) in self.boxas().iter().enumerate() {
            let bb = boxa.bounding_box().unwrap_or_default();
            write!(
                writer,
                "\nBoxa[{i}] extent: x = {}, y = {}, w = {}, h = {}",
                bb.x, bb.y, bb.w, bb.h
            )?;
            write_boxa(writer, boxa)?;
        }

        Ok(())
    }

    /// Write a Boxaa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaWrite()` in `boxbasic.c`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Boxaa to bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `boxaaWriteMem()` in `boxbasic.c`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// --- Internal parsing/writing helpers ---

/// Write a Boxa in the C Leptonica text format.
fn write_boxa(writer: &mut impl Write, boxa: &Boxa) -> Result<()> {
    let n = boxa.len();
    writeln!(writer, "\nBoxa Version {BOXA_VERSION}")?;
    writeln!(writer, "Number of boxes = {n}")?;

    for (i, b) in boxa.iter().enumerate() {
        writeln!(
            writer,
            "  Box[{i}]: x = {}, y = {}, w = {}, h = {}",
            b.x, b.y, b.w, b.h
        )?;
    }

    Ok(())
}

/// Parse a Boxa from a line iterator.
///
/// Expects the iterator to be positioned before the version header line.
fn parse_boxa<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Result<Boxa> {
    // Find and parse version line
    let version = find_and_parse_int(lines, "Boxa Version ")?;
    if version != BOXA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Boxa version: {version}"
        )));
    }

    // Parse count
    let n = find_and_parse_int(lines, "Number of boxes = ")? as usize;
    if n > MAX_BOXA_SIZE {
        return Err(Error::DecodeError(format!("too many boxes: {n}")));
    }

    let mut boxa = Boxa::with_capacity(n);
    for _ in 0..n {
        let b = parse_box_line(lines)?;
        boxa.push(b);
    }

    Ok(boxa)
}

/// Parse a Boxaa from a line iterator.
fn parse_boxaa<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Boxaa> {
    // Find and parse version line
    let version = find_and_parse_int(lines, "Boxaa Version ")?;
    if version != BOXAA_VERSION {
        return Err(Error::DecodeError(format!(
            "invalid Boxaa version: {version}"
        )));
    }

    // Parse count
    let n = find_and_parse_int(lines, "Number of boxa = ")? as usize;
    if n > MAX_BOXAA_SIZE {
        return Err(Error::DecodeError(format!("too many boxa: {n}")));
    }

    let mut boxaa = Boxaa::with_capacity(n);
    for _ in 0..n {
        // Skip extent line (Boxa[i] extent: x = ...)
        skip_until_contains(lines, "extent:")?;
        // Parse the embedded Boxa
        let boxa = parse_boxa(lines)?;
        boxaa.push(boxa);
    }

    Ok(boxaa)
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

/// Skip lines until one containing `needle` is found.
///
/// Returns an error if EOF is reached without finding the needle.
fn skip_until_contains<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    needle: &str,
) -> Result<()> {
    for line in lines.by_ref() {
        if line.contains(needle) {
            return Ok(());
        }
    }
    Err(Error::DecodeError(format!(
        "expected line containing '{needle}' not found"
    )))
}

/// Parse a Box line like "  Box[0]: x = 10, y = 20, w = 30, h = 40"
fn parse_box_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Box> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if !trimmed.starts_with("Box[") {
            continue;
        }
        // Parse "Box[N]: x = X, y = Y, w = W, h = H"
        let after_colon = trimmed
            .split_once(": ")
            .ok_or_else(|| Error::DecodeError(format!("invalid box line: {trimmed}")))?
            .1;

        let (x, y, w, h) = parse_box_fields(after_colon)?;

        if w < 0 || h < 0 {
            return Err(Error::DecodeError(format!(
                "box has negative dimensions: w = {w}, h = {h}"
            )));
        }

        return Ok(Box::new_unchecked(x, y, w, h));
    }
    Err(Error::DecodeError("expected Box line not found".into()))
}

/// Parse "x = 10, y = 20, w = 30, h = 40" into (x, y, w, h).
///
/// Parses by named keys, rejecting unknown/duplicate/missing fields.
fn parse_box_fields(s: &str) -> Result<(i32, i32, i32, i32)> {
    let mut x: Option<i32> = None;
    let mut y: Option<i32> = None;
    let mut w: Option<i32> = None;
    let mut h: Option<i32> = None;

    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (key, val) = part
            .split_once('=')
            .ok_or_else(|| Error::DecodeError(format!("invalid key/value pair: '{part}'")))?;

        let key = key.trim();
        let val = val.trim();
        let parsed = val
            .parse::<i32>()
            .map_err(|e| Error::DecodeError(format!("failed to parse value in '{part}': {e}")))?;

        let slot = match key {
            "x" => &mut x,
            "y" => &mut y,
            "w" => &mut w,
            "h" => &mut h,
            other => {
                return Err(Error::DecodeError(format!(
                    "unknown field '{other}' in box line"
                )));
            }
        };

        if slot.is_some() {
            return Err(Error::DecodeError(format!(
                "duplicate field '{key}' in box line"
            )));
        }
        *slot = Some(parsed);
    }

    let x = x.ok_or_else(|| Error::DecodeError("missing field 'x' in box line".into()))?;
    let y = y.ok_or_else(|| Error::DecodeError("missing field 'y' in box line".into()))?;
    let w = w.ok_or_else(|| Error::DecodeError("missing field 'w' in box line".into()))?;
    let h = h.ok_or_else(|| Error::DecodeError("missing field 'h' in box line".into()))?;

    Ok((x, y, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boxa_roundtrip_empty() {
        let boxa = Boxa::new();
        let bytes = boxa.write_to_bytes().unwrap();
        let restored = Boxa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_boxa_roundtrip() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 30, 40).unwrap());
        boxa.push(Box::new(0, 0, 100, 200).unwrap());
        boxa.push(Box::new(50, 60, 70, 80).unwrap());

        let bytes = boxa.write_to_bytes().unwrap();
        let restored = Boxa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get(0), Some(&Box::new_unchecked(10, 20, 30, 40)));
        assert_eq!(restored.get(1), Some(&Box::new_unchecked(0, 0, 100, 200)));
        assert_eq!(restored.get(2), Some(&Box::new_unchecked(50, 60, 70, 80)));
    }

    #[test]
    fn test_boxa_write_format() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 30, 40).unwrap());

        let bytes = boxa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Boxa Version 2"));
        assert!(text.contains("Number of boxes = 1"));
        assert!(text.contains("Box[0]: x = 10, y = 20, w = 30, h = 40"));
    }

    #[test]
    fn test_boxa_file_roundtrip() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(1, 2, 3, 4).unwrap());
        boxa.push(Box::new(5, 6, 7, 8).unwrap());

        let dir = std::env::temp_dir().join("leptonica_test_boxa");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.boxa");

        boxa.write_to_file(&path).unwrap();
        let restored = Boxa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get(0), Some(&Box::new_unchecked(1, 2, 3, 4)));
        assert_eq!(restored.get(1), Some(&Box::new_unchecked(5, 6, 7, 8)));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_boxa_reader_roundtrip() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(100, 200, 300, 400).unwrap());

        let mut buf = Vec::new();
        boxa.write_to_writer(&mut buf).unwrap();

        let restored = Boxa::read_from_reader(&mut std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(
            restored.get(0),
            Some(&Box::new_unchecked(100, 200, 300, 400))
        );
    }

    #[test]
    fn test_boxa_invalid_data() {
        assert!(Boxa::read_from_bytes(b"garbage data").is_err());
        assert!(Boxa::read_from_bytes(b"").is_err());
    }

    #[test]
    fn test_boxa_negative_dimensions_rejected() {
        // Negative w/h should be rejected during deserialization
        let input =
            b"\nBoxa Version 2\nNumber of boxes = 1\n  Box[0]: x = 10, y = 20, w = -5, h = 30\n";
        let result = Boxa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("negative dimensions"));
    }

    #[test]
    fn test_boxa_unknown_field_rejected() {
        let input =
            b"\nBoxa Version 2\nNumber of boxes = 1\n  Box[0]: x = 10, y = 20, w = 30, z = 40\n";
        let result = Boxa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown field"));
    }

    #[test]
    fn test_boxa_duplicate_field_rejected() {
        let input =
            b"\nBoxa Version 2\nNumber of boxes = 1\n  Box[0]: x = 10, y = 20, w = 30, w = 40\n";
        let result = Boxa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("duplicate field"));
    }

    #[test]
    fn test_boxa_missing_field_rejected() {
        let input = b"\nBoxa Version 2\nNumber of boxes = 1\n  Box[0]: x = 10, y = 20, w = 30\n";
        let result = Boxa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing field"));
    }

    #[test]
    fn test_boxaa_missing_extent_line() {
        // Boxaa with count=1 but no extent line should produce a clear error
        let input = b"\nBoxaa Version 3\nNumber of boxa = 1\n";
        let result = Boxaa::read_from_bytes(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("extent"));
    }

    #[test]
    fn test_boxaa_roundtrip_empty() {
        let boxaa = Boxaa::new();
        let bytes = boxaa.write_to_bytes().unwrap();
        let restored = Boxaa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_boxaa_roundtrip() {
        let mut boxaa = Boxaa::new();

        let mut boxa1 = Boxa::new();
        boxa1.push(Box::new(10, 20, 30, 40).unwrap());
        boxa1.push(Box::new(50, 60, 70, 80).unwrap());

        let mut boxa2 = Boxa::new();
        boxa2.push(Box::new(0, 0, 100, 100).unwrap());

        boxaa.push(boxa1);
        boxaa.push(boxa2);

        let bytes = boxaa.write_to_bytes().unwrap();
        let restored = Boxaa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get(0).unwrap().len(), 2);
        assert_eq!(restored.get(1).unwrap().len(), 1);
        assert_eq!(
            restored.get(0).unwrap().get(0),
            Some(&Box::new_unchecked(10, 20, 30, 40))
        );
        assert_eq!(
            restored.get(1).unwrap().get(0),
            Some(&Box::new_unchecked(0, 0, 100, 100))
        );
    }

    #[test]
    fn test_boxaa_write_format() {
        let mut boxaa = Boxaa::new();
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 30, 40).unwrap());
        boxaa.push(boxa);

        let bytes = boxaa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Boxaa Version 3"));
        assert!(text.contains("Number of boxa = 1"));
    }

    #[test]
    fn test_boxaa_file_roundtrip() {
        let mut boxaa = Boxaa::new();
        let mut boxa = Boxa::new();
        boxa.push(Box::new(1, 2, 3, 4).unwrap());
        boxaa.push(boxa);

        let dir = std::env::temp_dir().join("leptonica_test_boxaa");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.boxaa");

        boxaa.write_to_file(&path).unwrap();
        let restored = Boxaa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored.get(0).unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_boxa_with_zero_dim_boxes() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new(10, 20, 0, 0).unwrap());
        boxa.push(Box::new(30, 40, 50, 0).unwrap());

        let bytes = boxa.write_to_bytes().unwrap();
        let restored = Boxa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get(0), Some(&Box::new_unchecked(10, 20, 0, 0)));
        assert_eq!(restored.get(1), Some(&Box::new_unchecked(30, 40, 50, 0)));
    }
}
