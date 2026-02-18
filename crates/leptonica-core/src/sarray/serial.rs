//! Serialization for Sarray
//!
//! Text-based serialization format compatible with C Leptonica.
//!
//! # Format
//!
//! ```text
//! \nSarray Version 1\n
//! Number of strings = N\n
//!   0[len0]:  string0\n
//!   1[len1]:  string1\n
//!   ...
//! \n
//! ```
//!
//! # See also
//!
//! C Leptonica: `sarray1.c` (`sarrayReadStream`, `sarrayWriteStream`)

use crate::error::{Error, Result};
use crate::sarray::Sarray;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Sarray serialization format version (matches C Leptonica SARRAY_VERSION_NUMBER)
const SARRAY_VERSION: i32 = 1;

/// Maximum number of strings in a Sarray.
const MAX_SARRAY_SIZE: usize = 50_000_000;

/// Maximum input size in bytes.
const MAX_INPUT_SIZE: usize = 100_000_000;

impl Sarray {
    /// Read a Sarray from a reader.
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Sarray from a file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        Self::read_from_reader(&mut BufReader::new(file))
    }

    /// Read a Sarray from a byte slice.
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();

        // Parse version line
        let version = find_and_parse_int(&mut lines, "Sarray Version ")?;
        if version != SARRAY_VERSION {
            return Err(Error::DecodeError(format!(
                "invalid Sarray version: {version}"
            )));
        }

        // Parse count (validate non-negative before usize cast)
        let n_i32 = find_and_parse_int(&mut lines, "Number of strings = ")?;
        if n_i32 < 0 {
            return Err(Error::DecodeError(format!("invalid string count: {n_i32}")));
        }
        let n = n_i32 as usize;
        if n > MAX_SARRAY_SIZE {
            return Err(Error::DecodeError(format!("too many strings: {n}")));
        }

        // Parse string entries: "  i[len]:  string"
        let mut sa = Sarray::with_capacity(n);
        for _ in 0..n {
            let s = parse_string_line(&mut lines)?;
            sa.push(s);
        }

        Ok(sa)
    }

    /// Write a Sarray to a writer.
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let n = self.len();
        writeln!(writer, "\nSarray Version {SARRAY_VERSION}")?;
        writeln!(writer, "Number of strings = {n}")?;

        for (i, s) in self.iter().enumerate() {
            // The text format is line-based and cannot safely represent strings
            // containing newline characters. Reject such strings to avoid
            // producing output that cannot be parsed by `read_from_bytes`.
            if s.contains('\n') || s.contains('\r') {
                return Err(Error::EncodeError(format!(
                    "Sarray element at index {i} contains newline characters, \
                     which cannot be serialized in the Sarray text format"
                )));
            }
            writeln!(writer, "  {i}[{}]:  {s}", s.len())?;
        }

        // Trailing newline
        writeln!(writer)?;
        Ok(())
    }

    /// Write a Sarray to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Sarray to a byte vector.
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Read from a reader with a size limit.
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

/// Parse a string entry line: "  i[len]:  string"
///
/// The C format writes `"  %d[%d]:  %s\n"` where the string content
/// follows the two spaces after the colon. This function preserves
/// trailing whitespace in the string content by only stripping the
/// leading indentation, not the full line.
fn parse_string_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<String> {
    for line in lines.by_ref() {
        // Only skip leading indentation (2 spaces per C format)
        let line_content = line.strip_prefix("  ").unwrap_or(line);
        if line_content.trim().is_empty() {
            continue;
        }

        // Match pattern: "i[len]:  string"
        if let Some(bracket_pos) = line_content.find('[')
            && let Some(colon_pos) = line_content.find("]:")
        {
            // Parse length for validation
            let len_str = &line_content[bracket_pos + 1..colon_pos];
            let declared_len: usize = len_str
                .parse()
                .map_err(|e| Error::DecodeError(format!("failed to parse string length: {e}")))?;

            // The string content is after "]:  " (colon + 2 spaces)
            let content_start = colon_pos + 2; // skip "]:"
            let content = if content_start < line_content.len() {
                // Skip the two spaces after ":" to get the string content
                let rest = &line_content[content_start..];
                rest.strip_prefix("  ").unwrap_or(rest)
            } else {
                ""
            };

            // For empty strings, declared_len == 0 and content == ""
            if content.len() != declared_len {
                return Err(Error::DecodeError(format!(
                    "string length mismatch: declared {declared_len} but got {}",
                    content.len()
                )));
            }

            return Ok(content.to_string());
        }
        return Err(Error::DecodeError(format!(
            "invalid string entry line: '{line_content}'"
        )));
    }
    Err(Error::DecodeError(
        "expected string entry line not found".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sarray_roundtrip() {
        let sa = Sarray::from_vec(vec![
            "hello".into(),
            "world".into(),
            "test string with spaces".into(),
        ]);

        let bytes = sa.write_to_bytes().unwrap();
        let restored = Sarray::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get(0).unwrap(), "hello");
        assert_eq!(restored.get(1).unwrap(), "world");
        assert_eq!(restored.get(2).unwrap(), "test string with spaces");
    }

    #[test]
    fn test_sarray_roundtrip_empty() {
        let sa = Sarray::new();
        let bytes = sa.write_to_bytes().unwrap();
        let restored = Sarray::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_sarray_write_format() {
        let sa = Sarray::from_vec(vec!["abc".into(), "de".into()]);

        let bytes = sa.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Sarray Version 1"));
        assert!(text.contains("Number of strings = 2"));
        assert!(text.contains("0[3]:"));
        assert!(text.contains("1[2]:"));
    }

    #[test]
    fn test_sarray_file_roundtrip() {
        let sa = Sarray::from_vec(vec!["one".into(), "two".into(), "three".into()]);

        let dir = std::env::temp_dir().join("leptonica_test_sarray");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_sarray.txt");

        sa.write_to_file(&path).unwrap();
        let restored = Sarray::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get(0).unwrap(), "one");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_sarray_empty_strings() {
        let sa = Sarray::from_vec(vec!["".into(), "nonempty".into(), "".into()]);

        let bytes = sa.write_to_bytes().unwrap();
        let restored = Sarray::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get(0).unwrap(), "");
        assert_eq!(restored.get(1).unwrap(), "nonempty");
        assert_eq!(restored.get(2).unwrap(), "");
    }

    #[test]
    fn test_sarray_invalid_data() {
        let result = Sarray::read_from_bytes(b"not valid");
        assert!(result.is_err());
    }

    #[test]
    fn test_sarray_negative_count_rejected() {
        let input = b"\nSarray Version 1\nNumber of strings = -1\n";
        let result = Sarray::read_from_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_sarray_trailing_spaces() {
        let sa = Sarray::from_vec(vec!["hello  ".into(), "world ".into(), "  ".into()]);

        let bytes = sa.write_to_bytes().unwrap();
        let restored = Sarray::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get(0).unwrap(), "hello  ");
        assert_eq!(restored.get(1).unwrap(), "world ");
        assert_eq!(restored.get(2).unwrap(), "  ");
    }

    #[test]
    fn test_sarray_rejects_newlines() {
        let sa = Sarray::from_vec(vec!["hello".into(), "world\ntest".into()]);
        let result = sa.write_to_bytes();
        assert!(result.is_err());
    }

    #[test]
    fn test_sarray_rejects_carriage_returns() {
        let sa = Sarray::from_vec(vec!["hello".into(), "world\rtest".into()]);
        let result = sa.write_to_bytes();
        assert!(result.is_err());
    }
}
