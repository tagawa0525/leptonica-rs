//! Query and inspection methods for Recog
//!
//! Provides accessors for recognizer state and a `set_params` mutator.

use crate::error::RecogResult;

use super::types::{Recog, RecogParams};

impl Recog {
    /// Returns the total number of training templates across all classes.
    pub fn get_count(&self) -> usize {
        todo!("not yet implemented")
    }

    /// Returns the number of character classes.
    pub fn get_class_count(&self) -> usize {
        todo!("not yet implemented")
    }

    /// Returns the index of the class with the given label, if it exists.
    pub fn get_class_index(&self, _class_str: &str) -> Option<usize> {
        todo!("not yet implemented")
    }

    /// Returns the label of the class at the given index, if in bounds.
    pub fn get_class_string(&self, _index: usize) -> Option<&str> {
        todo!("not yet implemented")
    }

    /// Converts a single-character string to its class index via its
    /// UTF-8 code point value.
    ///
    /// The mapping used by C leptonica is: class index = first byte of the
    /// UTF-8 encoding of the first character in `class_str`.
    ///
    /// # Errors
    ///
    /// Returns an error if `class_str` is empty or contains no valid UTF-8
    /// character.
    pub fn string_to_index(_class_str: &str) -> RecogResult<usize> {
        todo!("not yet implemented")
    }

    /// Updates recognizer parameters.
    ///
    /// Only the scale and matching parameters are updated; template data is
    /// left unchanged.  Call [`finish_training`](Recog::finish_training)
    /// afterwards if scaled templates need to be recomputed.
    pub fn set_params(&mut self, _params: RecogParams) {
        todo!("not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

    use crate::recog::train::create;

    fn make_trained_recog() -> Recog {
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        for (label, w) in [("A", 3u32), ("B", 5u32)] {
            let p = Pix::new(w, 8, PixelDepth::Bit1).unwrap();
            let mut m = p.try_into_mut().unwrap();
            for y in 0..8 {
                for x in 0..w {
                    let _ = m.set_pixel(x, y, 1);
                }
            }
            let pix: Pix = m.into();
            recog.train_labeled(&pix, label).unwrap();
        }
        recog.finish_training().unwrap();
        recog
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_count() {
        let recog = make_trained_recog();
        // One sample per class, two classes.
        assert_eq!(recog.get_count(), 2);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_class_count() {
        let recog = make_trained_recog();
        assert_eq!(recog.get_class_count(), 2);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_class_index() {
        let recog = make_trained_recog();
        assert_eq!(recog.get_class_index("A"), Some(0));
        assert_eq!(recog.get_class_index("B"), Some(1));
        assert_eq!(recog.get_class_index("Z"), None);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_class_string() {
        let recog = make_trained_recog();
        assert_eq!(recog.get_class_string(0), Some("A"));
        assert_eq!(recog.get_class_string(1), Some("B"));
        assert_eq!(recog.get_class_string(99), None);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_get_class_index_and_string_roundtrip() {
        let recog = make_trained_recog();
        for idx in 0..recog.get_class_count() {
            let label = recog.get_class_string(idx).unwrap();
            let back = recog.get_class_index(label).unwrap();
            assert_eq!(back, idx);
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_string_to_index() {
        // 'A' has UTF-8 byte value 65
        let idx = Recog::string_to_index("A").unwrap();
        assert_eq!(idx, 65);

        // Empty string → error
        assert!(Recog::string_to_index("").is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_set_params() {
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let params = RecogParams {
            scale_w: 40,
            scale_h: 40,
            line_w: 0,
            threshold: 128,
            max_y_shift: 0,
        };
        recog.set_params(params);
        assert_eq!(recog.scale_w, 40);
        assert_eq!(recog.scale_h, 40);
        assert_eq!(recog.threshold, 128);
        assert_eq!(recog.max_y_shift, 0);
    }
}
