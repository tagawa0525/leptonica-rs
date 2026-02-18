//! Serialization for PixColormap
//!
//! Text-based serialization format compatible with C Leptonica.
//! Note: The C Leptonica colormap format has no version number.
//!
//! # Format
//!
//! ```text
//! \nPixcmap: depth = D bpp; N colors\n
//! Color    R-val    G-val    B-val   Alpha\n
//! ----------------------------------------\n
//!   0       R        G        B        A\n
//!   1       R        G        B        A\n
//!   ...
//! \n
//! ```
//!
//! # See also
//!
//! C Leptonica: `colormap.c` (`pixcmapReadStream`, `pixcmapWriteStream`)

use crate::PixColormap;
use crate::error::{Error, Result};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Maximum input size in bytes.
const MAX_INPUT_SIZE: usize = 100_000_000;

impl PixColormap {
    /// Read a PixColormap from a reader.
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let buf = read_limited(reader)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a PixColormap from a file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        Self::read_from_reader(&mut BufReader::new(file))
    }

    /// Read a PixColormap from a byte slice.
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::DecodeError(format!("invalid UTF-8: {e}")))?;
        let mut lines = text.lines().peekable();

        // Parse header line: "Pixcmap: depth = D bpp; N colors"
        let (depth, ncolors) = parse_header_line(&mut lines)?;

        // Skip the column header lines ("Color  R-val ..." and "----...")
        skip_header_lines(&mut lines);

        // Parse color entries
        let mut cmap = PixColormap::new(depth)?;
        for _ in 0..ncolors {
            let (r, g, b, a) = parse_color_line(&mut lines)?;
            cmap.add_rgba(r, g, b, a)?;
        }

        Ok(cmap)
    }

    /// Write a PixColormap to a writer.
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let ncolors = self.len();
        writeln!(
            writer,
            "\nPixcmap: depth = {} bpp; {ncolors} colors",
            self.depth()
        )?;
        writeln!(writer, "Color    R-val    G-val    B-val   Alpha")?;
        writeln!(writer, "----------------------------------------")?;

        for (i, color) in self.colors().iter().enumerate() {
            writeln!(
                writer,
                "{i:3}       {r:3}      {g:3}      {b:3}      {a:3}",
                r = color.red,
                g = color.green,
                b = color.blue,
                a = color.alpha,
            )?;
        }

        // Trailing newline
        writeln!(writer)?;
        Ok(())
    }

    /// Write a PixColormap to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a PixColormap to a byte vector.
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

/// Parse "Pixcmap: depth = D bpp; N colors" and return (depth, ncolors).
fn parse_header_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<(u32, usize)> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Pixcmap: depth = ") {
            // Parse "D bpp; N colors"
            let parts: Vec<&str> = rest.split("bpp;").collect();
            if parts.len() < 2 {
                return Err(Error::DecodeError(format!(
                    "invalid colormap header: '{trimmed}'"
                )));
            }
            let depth: u32 = parts[0]
                .trim()
                .parse()
                .map_err(|e| Error::DecodeError(format!("failed to parse colormap depth: {e}")))?;
            let ncolors_str = parts[1].trim().trim_end_matches("colors").trim();
            let ncolors: usize = ncolors_str.parse().map_err(|e| {
                Error::DecodeError(format!("failed to parse colormap ncolors: {e}"))
            })?;
            return Ok((depth, ncolors));
        }
    }
    Err(Error::DecodeError(
        "colormap header 'Pixcmap:' not found".into(),
    ))
}

/// Skip the column header lines (if present).
fn skip_header_lines<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) {
    // Skip up to 2 lines: "Color  R-val ..." and "----..."
    for _ in 0..2 {
        if let Some(&line) = lines.peek() {
            let trimmed = line.trim();
            if trimmed.starts_with("Color") || trimmed.starts_with("---") {
                lines.next();
            }
        }
    }
}

/// Parse a color entry line: "  i       R      G      B      A"
fn parse_color_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<(u8, u8, u8, u8)> {
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(Error::DecodeError(format!(
                "invalid color entry: '{trimmed}' (expected 5 fields, got {})",
                parts.len()
            )));
        }
        // parts[0] = index, parts[1] = R, parts[2] = G, parts[3] = B, parts[4] = A
        let r: u8 = parts[1]
            .parse()
            .map_err(|e| Error::DecodeError(format!("failed to parse R value: {e}")))?;
        let g: u8 = parts[2]
            .parse()
            .map_err(|e| Error::DecodeError(format!("failed to parse G value: {e}")))?;
        let b: u8 = parts[3]
            .parse()
            .map_err(|e| Error::DecodeError(format!("failed to parse B value: {e}")))?;
        let a: u8 = parts[4]
            .parse()
            .map_err(|e| Error::DecodeError(format!("failed to parse A value: {e}")))?;
        return Ok((r, g, b, a));
    }
    Err(Error::DecodeError(
        "expected color entry line not found".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colormap_roundtrip() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(255, 0, 0, 255).unwrap();
        cmap.add_rgba(0, 255, 0, 255).unwrap();
        cmap.add_rgba(0, 0, 255, 128).unwrap();

        let bytes = cmap.write_to_bytes().unwrap();
        let restored = PixColormap::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.depth(), 8);
        assert_eq!(restored.len(), 3);
        let (r, g, b, a) = restored.get_rgba(0).unwrap();
        assert_eq!((r, g, b, a), (255, 0, 0, 255));
        let (r, g, b, a) = restored.get_rgba(2).unwrap();
        assert_eq!((r, g, b, a), (0, 0, 255, 128));
    }

    #[test]
    fn test_colormap_write_format() {
        let mut cmap = PixColormap::new(4).unwrap();
        cmap.add_rgba(10, 20, 30, 255).unwrap();

        let bytes = cmap.write_to_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("Pixcmap: depth = 4 bpp"));
        assert!(text.contains("1 colors"));
        assert!(text.contains("R-val"));
    }

    #[test]
    fn test_colormap_file_roundtrip() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgba(0, 0, 0, 255).unwrap();
        cmap.add_rgba(255, 255, 255, 255).unwrap();

        let dir = std::env::temp_dir().join("leptonica_test_colormap");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_cmap.txt");

        cmap.write_to_file(&path).unwrap();
        let restored = PixColormap::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 2);
        let (r, g, b, _) = restored.get_rgba(0).unwrap();
        assert_eq!((r, g, b), (0, 0, 0));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_colormap_invalid_data() {
        let result = PixColormap::read_from_bytes(b"not valid");
        assert!(result.is_err());
    }
}
