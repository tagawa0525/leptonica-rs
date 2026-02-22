//! Dewarp I/O serialization
//!
//! Provides binary read/write for [`Dewarp`] models so that a built
//! disparity model can be persisted and reloaded without re-running
//! the full dewarping pipeline.

use std::io::{Read, Write};
use std::path::Path;

use leptonica_core::FPix;

use crate::error::{RecogError, RecogResult};

use super::types::Dewarp;

/// Magic bytes that identify a Dewarp binary file.
const MAGIC: &[u8; 7] = b"DEWARP\x01";

impl Dewarp {
    /// Serializes this `Dewarp` model to `writer` in a compact binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn write<W: Write>(&self, mut writer: W) -> RecogResult<()> {
        writer
            .write_all(MAGIC)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // Scalar fields (all little-endian)
        writer
            .write_all(&self.page_number.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.width.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.height.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.nx.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.ny.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.sampling.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.reduction_factor.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.min_lines.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.n_lines.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        writer
            .write_all(&self.min_curvature.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_curvature.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.left_slope.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.right_slope.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.left_curvature.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.right_curvature.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // Bool flags as u8
        let flags: u8 = (self.v_success as u8)
            | ((self.h_success as u8) << 1)
            | ((self.v_valid as u8) << 2)
            | ((self.h_valid as u8) << 3);
        writer
            .write_all(&[flags])
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // FPix arrays (4 of them, each prefixed with has-data byte)
        write_fpix(&mut writer, self.sampled_v_disparity.as_ref())?;
        write_fpix(&mut writer, self.sampled_h_disparity.as_ref())?;
        write_fpix(&mut writer, self.full_v_disparity.as_ref())?;
        write_fpix(&mut writer, self.full_h_disparity.as_ref())?;

        Ok(())
    }

    /// Writes this model to a file at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - Destination file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or writing fails.
    pub fn write_to_file(&self, path: &Path) -> RecogResult<()> {
        let file =
            std::fs::File::create(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        self.write(std::io::BufWriter::new(file))
    }

    /// Deserializes a `Dewarp` model from `reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or reading fails.
    pub fn read<R: Read>(mut reader: R) -> RecogResult<Dewarp> {
        // Validate magic
        let mut magic = [0u8; 7];
        reader
            .read_exact(&mut magic)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        if &magic != MAGIC {
            return Err(RecogError::InvalidParameter(
                "invalid Dewarp magic bytes".to_string(),
            ));
        }

        let page_number = read_u32(&mut reader)?;
        let width = read_u32(&mut reader)?;
        let height = read_u32(&mut reader)?;
        let nx = read_u32(&mut reader)?;
        let ny = read_u32(&mut reader)?;
        let sampling = read_u32(&mut reader)?;
        let reduction_factor = read_u32(&mut reader)?;
        let min_lines = read_u32(&mut reader)?;
        let n_lines = read_u32(&mut reader)?;

        // Validate invariants enforced by Dewarp::new / DewarpOptions
        if sampling < 8 {
            return Err(RecogError::InvalidParameter(format!(
                "sampling must be >= 8, got {sampling}"
            )));
        }
        if reduction_factor != 1 && reduction_factor != 2 {
            return Err(RecogError::InvalidParameter(format!(
                "reduction_factor must be 1 or 2, got {reduction_factor}"
            )));
        }
        if width == 0 || height == 0 {
            return Err(RecogError::InvalidParameter(
                "width and height must be non-zero".to_string(),
            ));
        }

        let min_curvature = read_i32(&mut reader)?;
        let max_curvature = read_i32(&mut reader)?;
        let left_slope = read_i32(&mut reader)?;
        let right_slope = read_i32(&mut reader)?;
        let left_curvature = read_i32(&mut reader)?;
        let right_curvature = read_i32(&mut reader)?;

        let mut flags_buf = [0u8; 1];
        reader
            .read_exact(&mut flags_buf)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let flags = flags_buf[0];
        if flags & !0x0F != 0 {
            return Err(RecogError::InvalidParameter(format!(
                "reserved bits set in flags byte: {flags:#04x}"
            )));
        }
        let v_success = (flags & 1) != 0;
        let h_success = (flags & 2) != 0;
        let v_valid = (flags & 4) != 0;
        let h_valid = (flags & 8) != 0;

        let sampled_v_disparity = read_fpix(&mut reader)?;
        let sampled_h_disparity = read_fpix(&mut reader)?;
        let full_v_disparity = read_fpix(&mut reader)?;
        let full_h_disparity = read_fpix(&mut reader)?;

        Ok(Dewarp {
            page_number,
            width,
            height,
            nx,
            ny,
            sampling,
            reduction_factor,
            min_lines,
            n_lines,
            sampled_v_disparity,
            sampled_h_disparity,
            full_v_disparity,
            full_h_disparity,
            min_curvature,
            max_curvature,
            left_slope,
            right_slope,
            left_curvature,
            right_curvature,
            v_success,
            h_success,
            v_valid,
            h_valid,
        })
    }

    /// Reads a `Dewarp` model from a file at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - Source file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsing fails.
    pub fn read_from_file(path: &Path) -> RecogResult<Dewarp> {
        let file =
            std::fs::File::open(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        Dewarp::read(std::io::BufReader::new(file))
    }
}

/// Writes an optional FPix to `writer`.
///
/// Format: 1-byte flag (0=absent, 1=present), then if present:
/// w (u32), h (u32), data (f32 × w × h, little-endian).
fn write_fpix<W: Write>(writer: &mut W, fpix: Option<&FPix>) -> RecogResult<()> {
    match fpix {
        None => {
            writer
                .write_all(&[0u8])
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        }
        Some(f) => {
            writer
                .write_all(&[1u8])
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
            let w = f.width();
            let h = f.height();
            writer
                .write_all(&w.to_le_bytes())
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
            writer
                .write_all(&h.to_le_bytes())
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
            for &v in f.data() {
                writer
                    .write_all(&v.to_le_bytes())
                    .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
            }
        }
    }
    Ok(())
}

/// Maximum number of f32 pixels allowed in a single FPix (≈16 MiB).
const MAX_FPIX_PIXELS: usize = 4_096 * 4_096;

/// Reads an optional FPix from `reader`.
fn read_fpix<R: Read>(reader: &mut R) -> RecogResult<Option<FPix>> {
    let mut flag = [0u8; 1];
    reader
        .read_exact(&mut flag)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    match flag[0] {
        0 => return Ok(None),
        1 => {}
        _ => {
            return Err(RecogError::InvalidParameter(format!(
                "invalid FPix presence flag: {}",
                flag[0]
            )));
        }
    }

    let w = read_u32(reader)?;
    let h = read_u32(reader)?;
    let n = (w as usize)
        .checked_mul(h as usize)
        .ok_or_else(|| RecogError::InvalidParameter("FPix dimensions overflow".to_string()))?;

    if n > MAX_FPIX_PIXELS {
        return Err(RecogError::InvalidParameter(format!(
            "FPix pixel count {n} exceeds maximum {MAX_FPIX_PIXELS}"
        )));
    }

    let mut data = vec![0.0f32; n];
    for v in data.iter_mut() {
        let mut buf = [0u8; 4];
        reader
            .read_exact(&mut buf)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        *v = f32::from_le_bytes(buf);
    }

    FPix::from_data(w, h, data)
        .map(Some)
        .map_err(RecogError::Core)
}

fn read_u32<R: Read>(reader: &mut R) -> RecogResult<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32<R: Read>(reader: &mut R) -> RecogResult<i32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(i32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::super::types::DewarpOptions;
    use super::*;

    fn make_empty_dewarp() -> Dewarp {
        let options = DewarpOptions::default();
        Dewarp::new(800, 600, 0, &options)
    }

    #[test]
    fn test_empty_dewarp_write_read_roundtrip() {
        let dewarp = make_empty_dewarp();
        let mut buf = Vec::new();
        dewarp.write(&mut buf).unwrap();
        let restored = Dewarp::read(buf.as_slice()).unwrap();
        assert_eq!(restored.page_number, dewarp.page_number);
        assert_eq!(restored.width, dewarp.width);
        assert_eq!(restored.height, dewarp.height);
        assert_eq!(restored.sampling, dewarp.sampling);
        assert_eq!(restored.min_lines, dewarp.min_lines);
        assert!(restored.sampled_v_disparity.is_none());
        assert!(restored.full_v_disparity.is_none());
    }

    #[test]
    fn test_dewarp_with_disparity_roundtrip() {
        let mut dewarp = make_empty_dewarp();
        // Insert a small FPix as sampled_v_disparity
        let fpix = FPix::new_with_value(3, 4, 1.5f32).unwrap();
        dewarp.sampled_v_disparity = Some(fpix);
        dewarp.v_success = true;
        dewarp.v_valid = true;
        dewarp.min_curvature = -100;
        dewarp.max_curvature = 200;

        let mut buf = Vec::new();
        dewarp.write(&mut buf).unwrap();
        let r = Dewarp::read(buf.as_slice()).unwrap();

        assert!(r.sampled_v_disparity.is_some());
        let fpix_r = r.sampled_v_disparity.as_ref().unwrap();
        assert_eq!(fpix_r.width(), 3);
        assert_eq!(fpix_r.height(), 4);
        assert!((fpix_r.get_pixel(0, 0).unwrap() - 1.5f32).abs() < 1e-6);
        assert!(r.v_success);
        assert!(r.v_valid);
        assert_eq!(r.min_curvature, -100);
        assert_eq!(r.max_curvature, 200);
    }

    #[test]
    fn test_dewarp_invalid_magic() {
        let bad = b"NOTDEWARP_DATA";
        let result = Dewarp::read(bad.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_dewarp_file_roundtrip() {
        let mut dewarp = make_empty_dewarp();
        dewarp.page_number = 7;
        dewarp.n_lines = 20;

        let path = std::env::temp_dir().join("dewarp_test.bin");
        dewarp.write_to_file(&path).unwrap();
        let r = Dewarp::read_from_file(&path).unwrap();
        assert_eq!(r.page_number, 7);
        assert_eq!(r.n_lines, 20);
        let _ = std::fs::remove_file(&path);
    }
}
