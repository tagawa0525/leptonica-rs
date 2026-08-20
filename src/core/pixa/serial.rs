//! Serialization for Pixa
//!
//! Binary serialization format for Pixa arrays. Each Pix is stored as a PNG
//! stream directly after its header line; the stream is self-terminating, so
//! no length prefix is written. The format is:
//!
//! ```text
//! Pixa Version 2\n
//! Number of pix = N\n
//! [embedded boxa in text format]
//!  pix[0]: xres = X, yres = Y\n
//! [PNG stream, ending with its IEND chunk]
//!  pix[1]: xres = X, yres = Y\n
//! [PNG stream]
//! ...
//! ```
//!
//! Earlier versions of this crate appended `, size = S` to the header line.
//! Such files are still read, but they are not readable by C Leptonica.
//!
//! # See also
//!
//! C Leptonica: `pixabasic.c` (`pixaReadStream`, `pixaWriteStream`)

use crate::core::box_::Boxa;
use crate::core::error::{Error, Result};
use crate::core::pixa::Pixa;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;

/// Pixa serialization format version (matches C Leptonica PIXA_VERSION_NUMBER)
const PIXA_VERSION: i32 = 2;

/// Maximum number of Pix in a serialized Pixa.
const MAX_PIXA_SIZE: usize = 10_000_000;

/// Maximum allowed PNG blob size (100 MiB).
const MAX_PNG_SIZE: usize = 100_000_000;

impl Pixa {
    /// Read a Pixa from a reader.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaReadStream()` in `pixabasic.c`
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Self::read_from_bytes(&buf)
    }

    /// Read a Pixa from a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaRead()` in `pixabasic.c`
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read a Pixa from bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaReadMem()` in `pixabasic.c`
    pub fn read_from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let mut line_buf = String::new();

        // Read version line
        read_line_trimmed(&mut cursor, &mut line_buf)?;
        let version = parse_after_prefix(&line_buf, "Pixa Version ")?;
        if version != PIXA_VERSION {
            return Err(Error::DecodeError(format!(
                "invalid Pixa version: {version}"
            )));
        }

        // Read count
        read_line_trimmed(&mut cursor, &mut line_buf)?;
        let n = parse_after_prefix(&line_buf, "Number of pix = ")? as usize;
        if n > MAX_PIXA_SIZE {
            return Err(Error::DecodeError(format!("too many pix: {n}")));
        }

        // Read embedded boxa by extracting only the text portion up to the
        // first pix header (or end of data if n == 0).
        let boxa = parse_boxa_from_mixed(&mut cursor)?;

        let mut pixa = Pixa::with_capacity(n);
        pixa.set_boxa(boxa);

        for i in 0..n {
            // Read pix header line
            read_line_trimmed(&mut cursor, &mut line_buf)?;
            let (xres, yres, size) = parse_pix_header(&line_buf, i)?;
            let pos = cursor.position() as usize;
            let size = match size {
                Some(size) => size,
                None => png_stream_len(data.get(pos..).unwrap_or_default())?,
            };
            if size > MAX_PNG_SIZE {
                return Err(Error::DecodeError(format!(
                    "PNG data too large for pix[{i}]: {size}"
                )));
            }

            // Read PNG data
            let end = pos.checked_add(size).ok_or_else(|| {
                Error::DecodeError(format!(
                    "integer overflow computing PNG data end position for pix[{i}]"
                ))
            })?;
            if end > data.len() {
                return Err(Error::DecodeError(format!(
                    "unexpected end of data reading pix[{i}]"
                )));
            }
            let png_data = &data[pos..end];
            cursor.set_position(end as u64);

            let pix = crate::io::png::read_png(Cursor::new(png_data))
                .map_err(|e| Error::DecodeError(format!("failed to read PNG for pix[{i}]: {e}")))?;
            // Just-decoded Pix has refcount 1, so try_into_mut always succeeds
            let mut pix_mut = pix
                .try_into_mut()
                .unwrap_or_else(|p| p.deep_clone().try_into_mut().unwrap());
            pix_mut.set_xres(xres);
            pix_mut.set_yres(yres);
            pixa.push(pix_mut.into());
        }

        Ok(pixa)
    }

    /// Write a Pixa to a writer.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaWriteStream()` in `pixabasic.c`
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()> {
        let n = self.len();
        writeln!(writer, "Pixa Version {PIXA_VERSION}")?;
        writeln!(writer, "Number of pix = {n}")?;

        // Write embedded boxa
        self.boxa().write_to_writer(writer)?;

        // Write each pix as PNG
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let png_data = crate::io::png::write_png_to_vec(pix).map_err(|e| {
                Error::EncodeError(format!("failed to write PNG for pix[{i}]: {e}"))
            })?;
            // C writes no size field; the PNG stream terminates itself.
            writeln!(
                writer,
                " pix[{i}]: xres = {}, yres = {}",
                pix.xres(),
                pix.yres()
            )?;
            writer.write_all(&png_data)?;
        }

        Ok(())
    }

    /// Write a Pixa to a file.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaWrite()` in `pixabasic.c`
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write a Pixa to bytes.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaWriteMem()` in `pixabasic.c`
    pub fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }
}

/// Read a line from the cursor, trimming leading/trailing whitespace.
fn read_line_trimmed(cursor: &mut Cursor<&[u8]>, buf: &mut String) -> Result<()> {
    loop {
        buf.clear();
        let n = cursor.read_line(buf)?;
        if n == 0 {
            return Err(Error::DecodeError("unexpected end of data".into()));
        }
        let trimmed = buf.trim();
        if !trimmed.is_empty() {
            *buf = trimmed.to_string();
            return Ok(());
        }
    }
}

/// Parse an embedded Boxa from a mixed text/binary buffer.
///
/// Reads the Boxa text format line by line, extracting only the text portion.
/// Updates the cursor position to just after the last box line.
fn parse_boxa_from_mixed(cursor: &mut Cursor<&[u8]>) -> Result<Boxa> {
    let mut line_buf = String::new();
    // Collect text lines for the boxa
    let mut boxa_text = Vec::new();

    // Read lines until we have enough for the boxa
    // First: find the "Boxa Version" line
    read_line_trimmed(cursor, &mut line_buf)?;
    boxa_text.push(line_buf.clone());

    // Then: "Number of boxes = N"
    read_line_trimmed(cursor, &mut line_buf)?;
    boxa_text.push(line_buf.clone());

    // Parse count from the number line
    let n = parse_after_prefix(boxa_text[1].trim(), "Number of boxes = ")? as usize;

    // Read N box lines
    for _ in 0..n {
        read_line_trimmed(cursor, &mut line_buf)?;
        boxa_text.push(line_buf.clone());
    }

    // Reconstruct the boxa text and parse it
    let text = boxa_text.join("\n");
    Boxa::read_from_bytes(text.as_bytes())
}

/// Parse an integer value after a known prefix.
fn parse_after_prefix(line: &str, prefix: &str) -> Result<i32> {
    line.strip_prefix(prefix)
        .ok_or_else(|| Error::DecodeError(format!("expected '{prefix}', got '{line}'")))
        .and_then(|rest| {
            rest.trim().parse::<i32>().map_err(|e| {
                Error::DecodeError(format!("failed to parse integer in '{line}': {e}"))
            })
        })
}

/// Parse pix header line like " pix[0]: xres = 72, yres = 72, size = 1234"
/// Parse a `pix[i]:` header line.
///
/// C writes only `xres` and `yres`. Older files written by this crate also
/// carry a `size` field, which is accepted and returned when present.
fn parse_pix_header(line: &str, expected_index: usize) -> Result<(i32, i32, Option<usize>)> {
    let line = line.trim();
    let prefix = format!("pix[{expected_index}]:");
    let rest = line
        .strip_prefix(&prefix)
        .ok_or_else(|| Error::DecodeError(format!("expected '{prefix}', got '{line}'")))?;

    let mut xres: Option<i32> = None;
    let mut yres: Option<i32> = None;
    let mut size: Option<usize> = None;

    for part in rest.split(',') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("xres = ") {
            xres = Some(
                val.trim()
                    .parse()
                    .map_err(|e| Error::DecodeError(format!("failed to parse xres: {e}")))?,
            );
        } else if let Some(val) = part.strip_prefix("yres = ") {
            yres = Some(
                val.trim()
                    .parse()
                    .map_err(|e| Error::DecodeError(format!("failed to parse yres: {e}")))?,
            );
        } else if let Some(val) = part.strip_prefix("size = ") {
            size = Some(
                val.trim()
                    .parse()
                    .map_err(|e| Error::DecodeError(format!("failed to parse size: {e}")))?,
            );
        }
    }

    Ok((
        xres.ok_or_else(|| Error::DecodeError("missing xres".into()))?,
        yres.ok_or_else(|| Error::DecodeError("missing yres".into()))?,
        size,
    ))
}

/// Length of the PNG stream starting at `data[0]`.
///
/// C relies on the PNG decoder consuming exactly one image from the stream.
/// Walking the chunk headers to `IEND` gives the same boundary without
/// decoding.
fn png_stream_len(data: &[u8]) -> Result<usize> {
    const SIGNATURE: usize = 8;
    if data.len() < SIGNATURE || data[..SIGNATURE] != *b"\x89PNG\r\n\x1a\n" {
        return Err(Error::DecodeError("not a PNG stream".into()));
    }
    let mut pos = SIGNATURE;
    loop {
        // Each chunk is [4-byte length][4-byte type][data][4-byte CRC].
        if pos + 8 > data.len() {
            return Err(Error::DecodeError("truncated PNG chunk header".into()));
        }
        let len = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let ctype = &data[pos + 4..pos + 8];
        let end = pos
            .checked_add(12)
            .and_then(|p| p.checked_add(len as usize))
            .ok_or_else(|| Error::DecodeError("PNG chunk length overflow".into()))?;
        if end > data.len() {
            return Err(Error::DecodeError("truncated PNG chunk".into()));
        }
        if ctype == b"IEND" {
            return Ok(end);
        }
        pos = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::box_::Box;
    use crate::core::pix::{Pix, PixelDepth};

    #[test]
    fn test_pixa_roundtrip_empty() {
        let pixa = Pixa::new();
        let bytes = pixa.write_to_bytes().unwrap();
        let restored = Pixa::read_from_bytes(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_pixa_roundtrip() {
        let mut pixa = Pixa::new();
        let pix1 = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let pix2 = Pix::new(20, 15, PixelDepth::Bit8).unwrap();
        pixa.push(pix1);
        pixa.push(pix2);

        let bytes = pixa.write_to_bytes().unwrap();
        let restored = Pixa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get(0).unwrap().width(), 10);
        assert_eq!(restored.get(0).unwrap().height(), 10);
        assert_eq!(restored.get(1).unwrap().width(), 20);
        assert_eq!(restored.get(1).unwrap().height(), 15);
    }

    #[test]
    fn test_pixa_roundtrip_with_boxes() {
        let mut pixa = Pixa::new();
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        pixa.push_with_box(pix, Box::new(1, 2, 3, 4).unwrap());

        let bytes = pixa.write_to_bytes().unwrap();
        let restored = Pixa::read_from_bytes(&bytes).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored.boxa().len(), 1);
        assert_eq!(restored.get_box(0), Some(&Box::new_unchecked(1, 2, 3, 4)));
    }

    #[test]
    fn test_pixa_file_roundtrip() {
        let mut pixa = Pixa::new();
        pixa.push(Pix::new(5, 5, PixelDepth::Bit8).unwrap());

        let dir = std::env::temp_dir().join("leptonica_test_pixa_serial");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.pixa");

        pixa.write_to_file(&path).unwrap();
        let restored = Pixa::read_from_file(&path).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored.get(0).unwrap().width(), 5);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pixa_invalid_data() {
        assert!(Pixa::read_from_bytes(b"garbage data").is_err());
        assert!(Pixa::read_from_bytes(b"").is_err());
    }
}

#[cfg(test)]
mod c_format_tests {
    use super::*;
    use crate::core::{Pix, PixelDepth};

    fn sample_pixa() -> Pixa {
        let mut pixa = Pixa::new();
        for d in [PixelDepth::Bit1, PixelDepth::Bit8] {
            let pix = Pix::new(7, 5, d).unwrap();
            let mut m = pix.try_into_mut().unwrap();
            m.set_xres(300);
            m.set_yres(300);
            let _ = m.set_pixel(1, 1, 1);
            pixa.push(m.into());
        }
        pixa
    }

    /// C `pixaWriteStream` emits only `xres` and `yres`; the PNG stream is
    /// self-terminating, so there is no `size` field. Writing one makes the
    /// file unreadable by C.
    #[test]
    fn test_write_matches_c_header() {
        let bytes = sample_pixa().write_to_bytes().unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains(" pix[0]: xres = 300, yres = 300\n"),
            "header should match C exactly"
        );
        assert!(!text.contains("size = "), "C writes no size field");
    }

    /// A file written by C has no `size` field, so the reader has to find the
    /// end of each PNG stream itself.
    #[test]
    fn test_read_c_written_header() {
        let pixa = sample_pixa();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\nPixa Version 2\n");
        bytes.extend_from_slice(format!("Number of pix = {}\n", pixa.len()).as_bytes());
        pixa.boxa().write_to_writer(&mut bytes).unwrap();
        for i in 0..pixa.len() {
            let pix = pixa.get(i).unwrap();
            bytes.extend_from_slice(
                format!(" pix[{i}]: xres = {}, yres = {}\n", pix.xres(), pix.yres()).as_bytes(),
            );
            bytes.extend_from_slice(&crate::io::png::write_png_to_vec(pix).unwrap());
        }

        let got = Pixa::read_from_bytes(&bytes).unwrap();
        assert_eq!(got.len(), pixa.len());
        for i in 0..pixa.len() {
            let a = pixa.get(i).unwrap();
            let b = got.get(i).unwrap();
            assert_eq!(
                (a.width(), a.height(), a.depth()),
                (b.width(), b.height(), b.depth())
            );
            assert_eq!((b.xres(), b.yres()), (300, 300));
        }
    }
}
