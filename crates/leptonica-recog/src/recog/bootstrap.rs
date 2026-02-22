//! Bootstrap digit recognizer
//!
//! Provides a built-in recognizer for the ten Arabic numerals (0-9) that can
//! be used to seed training when only a small number of real samples are
//! available.

use crate::error::RecogResult;

use super::types::Recog;

impl Recog {
    /// Creates a bootstrap digit recognizer for the Arabic numerals 0-9.
    ///
    /// The recognizer is built from programmatically generated templates
    /// at the requested scale height.
    ///
    /// # Arguments
    ///
    /// * `scale_h` - Target template height in pixels (minimum 20)
    ///
    /// # Errors
    ///
    /// Returns an error if `scale_h` is less than 20.
    pub fn make_boot_digit_recog(_scale_h: u32) -> RecogResult<Recog> {
        todo!("not yet implemented")
    }

    /// Returns `true` if the training set needs digit padding.
    ///
    /// Padding is needed when the recognizer uses an Arabic numeral charset
    /// and has fewer than 10 classes (i.e., some digits are missing).
    pub fn is_padding_needed(&self) -> bool {
        todo!("not yet implemented")
    }

    /// Pads the training set with templates from a bootstrap digit recognizer.
    ///
    /// Any missing digit class (0-9) is supplemented with templates from a
    /// freshly created bootstrap recognizer at the same scale height.
    ///
    /// After padding, training is finished automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if `scale_h` is less than 20, or if the recognizer's
    /// charset type is not `ArabicNumerals`.
    pub fn pad_digit_training_set(&mut self, _scale_h: u32) -> RecogResult<()> {
        todo!("not yet implemented")
    }

    /// Supplements this recognizer with templates from `boot_recog`.
    ///
    /// For each class present in `boot_recog` but absent in `self`, the
    /// averaged template from `boot_recog` is added to `self` as a training
    /// sample.
    ///
    /// After merging, training is finished automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if training has already been completed.
    pub fn train_from_boot(&mut self, _boot_recog: &Recog) -> RecogResult<()> {
        todo!("not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

    use crate::recog::train::create;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_make_boot_digit_recog_produces_ten_classes() {
        let boot = Recog::make_boot_digit_recog(30).unwrap();
        assert_eq!(boot.get_class_count(), 10);
        assert!(boot.train_done);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_make_boot_digit_recog_invalid_scale() {
        assert!(Recog::make_boot_digit_recog(10).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_is_padding_needed_true_for_incomplete_digits() {
        // Create a numeral recognizer with only one digit
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let p = make_solid_pix(5, 8);
        recog.train_labeled(&p, "0").unwrap();
        recog.finish_training().unwrap();
        // set charset so is_padding_needed can reason about it
        recog.charset_type = crate::recog::CharsetType::ArabicNumerals;
        assert!(recog.is_padding_needed());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_pad_digit_training_set_fills_missing_classes() {
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let p = make_solid_pix(5, 8);
        recog.train_labeled(&p, "0").unwrap();
        recog.finish_training().unwrap();
        recog.charset_type = crate::recog::CharsetType::ArabicNumerals;

        recog.pad_digit_training_set(30).unwrap();
        assert_eq!(recog.get_class_count(), 10);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_train_from_boot() {
        let boot = Recog::make_boot_digit_recog(30).unwrap();

        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let p = make_solid_pix(5, 8);
        recog.train_labeled(&p, "0").unwrap();
        recog.finish_training().unwrap();

        // train_from_boot should add missing digits from boot
        let classes_before = recog.get_class_count();
        recog.train_done = false; // reset to allow re-training
        recog.ave_done = false;
        recog.train_from_boot(&boot).unwrap();
        assert!(recog.get_class_count() > classes_before);
    }

    fn make_solid_pix(w: u32, h: u32) -> Pix {
        let p = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut m = p.try_into_mut().unwrap();
        for y in 0..h {
            for x in 0..w {
                let _ = m.set_pixel(x, y, 1);
            }
        }
        m.into()
    }
}
