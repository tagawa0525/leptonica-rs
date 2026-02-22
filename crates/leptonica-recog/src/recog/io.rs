//! Serialization and deserialization for Recog
//!
//! Provides binary format read/write for trained recognizers, and
//! conversion to/from Pixa for template extraction and reconstruction.

use std::io::{Read, Write};
use std::path::Path;

use leptonica_core::Pixa;

use crate::error::{RecogError, RecogResult};

use super::types::{Recog, RecogParams};

impl Recog {
    /// Writes the recognizer to a binary stream.
    ///
    /// # Format
    ///
    /// Magic header `RECOG\x01` followed by parameters, class labels,
    /// and unscaled template pixels for each class.
    pub fn write<W: Write>(&self, _writer: W) -> RecogResult<()> {
        todo!("not yet implemented")
    }

    /// Writes the recognizer to a file.
    pub fn write_to_file(&self, path: &Path) -> RecogResult<()> {
        let file =
            std::fs::File::create(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let writer = std::io::BufWriter::new(file);
        self.write(writer)
    }

    /// Reads a recognizer from a binary stream.
    pub fn read<R: Read>(_reader: R) -> RecogResult<Recog> {
        todo!("not yet implemented")
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
    pub fn extract_pixa(&self) -> RecogResult<Pixa> {
        todo!("not yet implemented")
    }

    /// Creates a recognizer from a Pixa of averaged templates and labels.
    ///
    /// # Arguments
    ///
    /// * `pixa` - Averaged templates, one per class
    /// * `labels` - Class label for each template (must match `pixa.len()`)
    /// * `params` - Recognizer parameters
    pub fn create_from_pixa_templates(
        _pixa: &Pixa,
        _labels: &[&str],
        _params: &RecogParams,
    ) -> RecogResult<Recog> {
        todo!("not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

    use crate::recog::train::{create, create_from_pixa};

    fn make_simple_recog() -> Recog {
        // Build a minimal recognizer with two 1-sample classes.
        let mut recog = create(0, 0, 0, 150, 1).unwrap();

        let pix_a = {
            let p = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
            let mut m = p.try_into_mut().unwrap();
            for i in 0..5 {
                let _ = m.set_pixel(i, i, 1);
            }
            m.into()
        };
        let pix_b = {
            let p = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
            let mut m = p.try_into_mut().unwrap();
            for i in 0..5 {
                let _ = m.set_pixel(0, i, 1);
            }
            m.into()
        };
        recog.train_labeled(&pix_a, "A").unwrap();
        recog.train_labeled(&pix_b, "B").unwrap();
        recog.finish_training().unwrap();
        recog
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_empty_recog_write_read_roundtrip() {
        // An untrained (but valid) recognizer: create with just parameters,
        // no training. After roundtrip the parameters should be identical.
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
