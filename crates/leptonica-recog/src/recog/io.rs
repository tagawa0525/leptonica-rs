//! Serialization and deserialization for Recog
//!
//! Provides binary format read/write for trained recognizers, and
//! conversion to/from Pixa for template extraction and reconstruction.

use std::io::{Read, Write};
use std::path::Path;

use leptonica_core::{Pix, Pixa, PixelDepth};

use crate::error::{RecogError, RecogResult};

use super::train::create;
use super::types::{CharsetType, Recog, RecogParams};

/// Binary format magic header: ASCII "RECOG" + version byte 0x01.
const MAGIC: &[u8; 6] = b"RECOG\x01";

impl Recog {
    /// Writes the recognizer to a binary stream.
    ///
    /// # Format
    ///
    /// Magic header `RECOG\x01` followed by parameters, class labels,
    /// and unscaled template pixels for each class.
    pub fn write<W: Write>(&self, mut writer: W) -> RecogResult<()> {
        // Header
        writer
            .write_all(MAGIC)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // Scaling / matching parameters
        write_i32(&mut writer, self.scale_w)?;
        write_i32(&mut writer, self.scale_h)?;
        write_i32(&mut writer, self.line_w)?;
        write_i32(&mut writer, self.threshold)?;
        write_i32(&mut writer, self.max_y_shift)?;
        write_u8(&mut writer, self.charset_type as u8)?;
        write_u8(&mut writer, self.train_done as u8)?;

        // Class data
        let set_size = u32::try_from(self.set_size)
            .map_err(|_| RecogError::InvalidParameter("set_size exceeds u32::MAX".to_string()))?;
        write_u32(&mut writer, set_size)?;
        for class_idx in 0..self.set_size {
            // Label
            let label_bytes = self.sa_text[class_idx].as_bytes();
            let label_len = u32::try_from(label_bytes.len()).map_err(|_| {
                RecogError::InvalidParameter("label length exceeds u32::MAX".to_string())
            })?;
            write_u32(&mut writer, label_len)?;
            writer
                .write_all(label_bytes)
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

            // Unscaled templates
            let templates = &self.pixaa_u[class_idx];
            let template_count = u32::try_from(templates.len()).map_err(|_| {
                RecogError::InvalidParameter("template count exceeds u32::MAX".to_string())
            })?;
            write_u32(&mut writer, template_count)?;
            for pix in templates {
                write_pix(&mut writer, pix)?;
            }
        }

        Ok(())
    }

    /// Writes the recognizer to a file.
    pub fn write_to_file(&self, path: &Path) -> RecogResult<()> {
        let file =
            std::fs::File::create(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let writer = std::io::BufWriter::new(file);
        self.write(writer)
    }

    /// Reads a recognizer from a binary stream.
    pub fn read<R: Read>(mut reader: R) -> RecogResult<Recog> {
        // Verify magic
        let mut magic = [0u8; 6];
        reader
            .read_exact(&mut magic)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        if &magic != MAGIC {
            return Err(RecogError::InvalidParameter(
                "invalid RECOG binary format header".to_string(),
            ));
        }

        // Parameters
        let scale_w = read_i32(&mut reader)?;
        let scale_h = read_i32(&mut reader)?;
        let line_w = read_i32(&mut reader)?;
        let threshold = read_i32(&mut reader)?;
        let max_y_shift = read_i32(&mut reader)?;
        let charset_type = charset_type_from_byte(read_u8(&mut reader)?)?;
        let train_done = read_u8(&mut reader)? != 0;

        let mut recog = create(scale_w, scale_h, line_w, threshold, max_y_shift)?;
        recog.charset_type = charset_type;

        // Classes
        let set_size = read_u32(&mut reader)? as usize;
        for _ in 0..set_size {
            let label_len = read_u32(&mut reader)? as usize;
            let mut label_bytes = vec![0u8; label_len];
            reader
                .read_exact(&mut label_bytes)
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
            let label = String::from_utf8(label_bytes)
                .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

            let num_templates = read_u32(&mut reader)? as usize;
            for _ in 0..num_templates {
                let pix = read_pix(&mut reader)?;
                // Use add_sample to bypass re-processing: the stored pixels
                // were already binarized and noise-cleaned during training.
                recog.add_sample(&pix, &label)?;
            }
        }

        if train_done && recog.set_size > 0 {
            recog.finish_training()?;
        }

        Ok(recog)
    }

    /// Reads a recognizer from a file.
    pub fn read_from_file(path: &Path) -> RecogResult<Recog> {
        let file =
            std::fs::File::open(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let reader = std::io::BufReader::new(file);
        Recog::read(reader)
    }

    /// Extracts the averaged unscaled templates as a Pixa.
    ///
    /// Returns one image per class in class-index order.
    /// The caller can retrieve class labels with [`Recog::get_class_labels`].
    ///
    /// # Errors
    ///
    /// Returns an error if training has not been completed yet.
    pub fn extract_pixa(&self) -> RecogResult<Pixa> {
        if !self.train_done {
            return Err(RecogError::TrainingError(
                "training must be completed before extracting pixa".to_string(),
            ));
        }

        let mut pixa = Pixa::new();
        for pix in &self.pixa_u {
            pixa.push(pix.clone());
        }
        Ok(pixa)
    }

    /// Creates a recognizer from a Pixa of averaged templates and labels.
    ///
    /// Each image in `pixa` is treated as the sole training sample for the
    /// corresponding class in `labels`.  After inserting all samples,
    /// [`finish_training`](Recog::finish_training) is called automatically.
    ///
    /// # Arguments
    ///
    /// * `pixa` - Averaged templates, one per class
    /// * `labels` - Class label for each template (must match `pixa.len()`)
    /// * `params` - Recognizer parameters
    ///
    /// # Errors
    ///
    /// Returns an error if `pixa.len() != labels.len()` or if any template
    /// has no foreground pixels.
    pub fn create_from_pixa_templates(
        pixa: &Pixa,
        labels: &[&str],
        params: &RecogParams,
    ) -> RecogResult<Recog> {
        if pixa.len() != labels.len() {
            return Err(RecogError::InvalidParameter(
                "pixa and labels must have the same length".to_string(),
            ));
        }

        let mut recog = create(
            params.scale_w,
            params.scale_h,
            params.line_w,
            params.threshold,
            params.max_y_shift,
        )?;

        for (i, label) in labels.iter().enumerate() {
            let pix = pixa
                .get(i)
                .ok_or_else(|| RecogError::InvalidParameter(format!("pixa missing element {i}")))?;
            recog.train_labeled(pix, label)?;
        }

        recog.finish_training()?;
        Ok(recog)
    }
}

// --- Little-endian I/O helpers ---

fn write_i32<W: Write>(writer: &mut W, v: i32) -> RecogResult<()> {
    writer
        .write_all(&v.to_le_bytes())
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))
}

fn write_u32<W: Write>(writer: &mut W, v: u32) -> RecogResult<()> {
    writer
        .write_all(&v.to_le_bytes())
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))
}

fn write_u8<W: Write>(writer: &mut W, v: u8) -> RecogResult<()> {
    writer
        .write_all(&[v])
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))
}

fn read_i32<R: Read>(reader: &mut R) -> RecogResult<i32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(i32::from_le_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> RecogResult<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u8<R: Read>(reader: &mut R) -> RecogResult<u8> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(buf[0])
}

/// Writes a 1bpp Pix to `writer` as (width u32)(height u32)(packed rows).
///
/// Each row is `ceil(width / 8)` bytes, pixels packed MSB-first.
fn write_pix<W: Write>(writer: &mut W, pix: &Pix) -> RecogResult<()> {
    let w = pix.width();
    let h = pix.height();
    write_u32(writer, w)?;
    write_u32(writer, h)?;

    let bytes_per_row = w.div_ceil(8);
    for y in 0..h {
        for byte_idx in 0..bytes_per_row {
            let mut byte = 0u8;
            for bit in 0..8u32 {
                let x = byte_idx * 8 + bit;
                if x < w && pix.get_pixel(x, y).unwrap_or(0) == 1 {
                    byte |= 1 << (7 - bit);
                }
            }
            write_u8(writer, byte)?;
        }
    }
    Ok(())
}

/// Reads a 1bpp Pix from `reader` written by [`write_pix`].
fn read_pix<R: Read>(reader: &mut R) -> RecogResult<Pix> {
    let w = read_u32(reader)?;
    let h = read_u32(reader)?;

    let pix = Pix::new(w, h, PixelDepth::Bit1).map_err(RecogError::Core)?;
    let mut pix_mut = pix.try_into_mut().unwrap_or_else(|p| p.to_mut());

    let bytes_per_row = w.div_ceil(8);
    for y in 0..h {
        for byte_idx in 0..bytes_per_row {
            let byte = read_u8(reader)?;
            for bit in 0..8u32 {
                let x = byte_idx * 8 + bit;
                if x < w && (byte >> (7 - bit)) & 1 == 1 {
                    let _ = pix_mut.set_pixel(x, y, 1);
                }
            }
        }
    }

    Ok(pix_mut.into())
}

fn charset_type_from_byte(b: u8) -> RecogResult<CharsetType> {
    match b {
        0 => Ok(CharsetType::Unknown),
        1 => Ok(CharsetType::ArabicNumerals),
        2 => Ok(CharsetType::LcRomanNumerals),
        3 => Ok(CharsetType::UcRomanNumerals),
        4 => Ok(CharsetType::LcAlpha),
        5 => Ok(CharsetType::UcAlpha),
        _ => Err(RecogError::InvalidParameter(format!(
            "unknown charset type byte: {b}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

    use crate::recog::train::create;

    fn make_simple_recog() -> Recog {
        // Build a minimal recognizer with two 1-sample classes.
        // Images must be at least 5 rows tall (solid) to survive the 1×5
        // morphological opening in train_labeled.
        let mut recog = create(0, 0, 0, 150, 1).unwrap();

        let pix_a = {
            // Class "A": narrow solid 3×8 block
            let p = Pix::new(3, 8, PixelDepth::Bit1).unwrap();
            let mut m = p.try_into_mut().unwrap();
            for y in 0..8 {
                for x in 0..3 {
                    let _ = m.set_pixel(x, y, 1);
                }
            }
            m.into()
        };
        let pix_b = {
            // Class "B": wider solid 5×8 block (different shape from A)
            let p = Pix::new(5, 8, PixelDepth::Bit1).unwrap();
            let mut m = p.try_into_mut().unwrap();
            for y in 0..8 {
                for x in 0..5 {
                    let _ = m.set_pixel(x, y, 1);
                }
            }
            m.into()
        };
        recog.train_labeled(&pix_a, "A").unwrap();
        recog.train_labeled(&pix_b, "B").unwrap();
        recog.finish_training().unwrap();
        recog
    }

    #[test]
    fn test_empty_recog_write_read_roundtrip() {
        let recog = create(20, 30, 0, 128, 0).unwrap();

        let mut buf = Vec::new();
        recog.write(&mut buf).unwrap();

        let restored = Recog::read(buf.as_slice()).unwrap();
        assert_eq!(restored.scale_w, recog.scale_w);
        assert_eq!(restored.scale_h, recog.scale_h);
        assert_eq!(restored.threshold, recog.threshold);
        assert_eq!(restored.max_y_shift, recog.max_y_shift);
        assert_eq!(restored.set_size, recog.set_size);
    }

    #[test]
    fn test_trained_recog_write_read_roundtrip() {
        let recog = make_simple_recog();

        let mut buf = Vec::new();
        recog.write(&mut buf).unwrap();

        let restored = Recog::read(buf.as_slice()).unwrap();
        assert_eq!(restored.set_size, recog.set_size);
        assert_eq!(restored.num_samples, recog.num_samples);
        assert_eq!(restored.get_class_labels(), recog.get_class_labels());
        assert_eq!(restored.train_done, recog.train_done);
    }

    #[test]
    fn test_extract_pixa_create_from_pixa_roundtrip() {
        let recog = make_simple_recog();

        let pixa = recog.extract_pixa().unwrap();
        assert_eq!(pixa.len(), recog.set_size);

        let labels: Vec<&str> = recog
            .get_class_labels()
            .iter()
            .map(|s| s.as_str())
            .collect();
        let params = RecogParams {
            scale_w: recog.scale_w,
            scale_h: recog.scale_h,
            line_w: recog.line_w,
            threshold: recog.threshold,
            max_y_shift: recog.max_y_shift,
        };
        let restored = Recog::create_from_pixa_templates(&pixa, &labels, &params).unwrap();

        assert_eq!(restored.set_size, recog.set_size);
        assert_eq!(restored.get_class_labels(), recog.get_class_labels());
        assert!(restored.train_done);
    }

    #[test]
    fn test_write_read_file_roundtrip() {
        let recog = make_simple_recog();

        let tmp = std::env::temp_dir().join("test_recog.bin");
        recog.write_to_file(&tmp).unwrap();

        let restored = Recog::read_from_file(&tmp).unwrap();
        assert_eq!(restored.set_size, recog.set_size);
        assert_eq!(restored.get_class_labels(), recog.get_class_labels());

        let _ = std::fs::remove_file(&tmp);
    }
}
