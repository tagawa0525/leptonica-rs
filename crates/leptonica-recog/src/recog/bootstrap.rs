//! Bootstrap digit recognizer
//!
//! Provides a built-in recognizer for the ten Arabic numerals (0-9) that can
//! be used to seed training when only a small number of real samples are
//! available.

use leptonica_core::{Pix, PixelDepth};

use crate::error::{RecogError, RecogResult};

use super::train::create;
use super::types::{CharsetType, Recog};

/// Minimum scale height for the bootstrap recognizer.
const MIN_BOOT_SCALE_H: u32 = 20;

/// Width-to-height ratio used when generating digit templates.
const DIGIT_ASPECT: f32 = 0.6;

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
    pub fn make_boot_digit_recog(scale_h: u32) -> RecogResult<Recog> {
        if scale_h < MIN_BOOT_SCALE_H {
            return Err(RecogError::InvalidParameter(format!(
                "scale_h must be at least {MIN_BOOT_SCALE_H}, got {scale_h}"
            )));
        }

        let mut recog = create(0, scale_h as i32, 0, 150, 1)?;
        recog.charset_type = CharsetType::ArabicNumerals;
        recog.charset_size = 10;

        let w = ((scale_h as f32 * DIGIT_ASPECT).round() as u32).max(4);
        let h = scale_h;

        for digit in 0u32..10 {
            let label = digit.to_string();
            let pix = make_digit_pix(digit, w, h)?;
            recog.train_labeled(&pix, &label)?;
        }

        recog.finish_training()?;
        Ok(recog)
    }

    /// Returns `true` if the training set needs digit padding.
    ///
    /// Padding is needed when the recognizer uses an Arabic numeral charset
    /// and has fewer than 10 classes (i.e., some digits are missing).
    pub fn is_padding_needed(&self) -> bool {
        self.charset_type == CharsetType::ArabicNumerals && self.set_size < 10
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
    pub fn pad_digit_training_set(&mut self, scale_h: u32) -> RecogResult<()> {
        if self.charset_type != CharsetType::ArabicNumerals {
            return Err(RecogError::InvalidParameter(
                "pad_digit_training_set requires charset_type == ArabicNumerals".to_string(),
            ));
        }

        let boot = Recog::make_boot_digit_recog(scale_h)?;

        // Collect missing labels before mutating self.
        let missing: Vec<String> = (0u32..10)
            .map(|d| d.to_string())
            .filter(|label| !self.sa_text.iter().any(|s| s == label))
            .collect();

        // Reset training state so we can add more samples.
        self.train_done = false;
        self.ave_done = false;

        for label in missing {
            if let Some(idx) = boot.get_class_index(&label)
                && let Some(pix) = boot.pixa_u.get(idx)
            {
                let pix = pix.clone();
                self.add_sample(&pix, &label)?;
            }
        }

        self.finish_training()
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
    pub fn train_from_boot(&mut self, boot_recog: &Recog) -> RecogResult<()> {
        if self.train_done {
            return Err(RecogError::TrainingError(
                "training has already been completed; reset train_done first".to_string(),
            ));
        }

        // Collect missing labels before mutating self.
        let to_add: Vec<(usize, String)> = boot_recog
            .sa_text
            .iter()
            .enumerate()
            .filter(|(_, label)| !self.sa_text.iter().any(|s| s == *label))
            .map(|(idx, label)| (idx, label.clone()))
            .collect();

        for (idx, label) in to_add {
            if let Some(pix) = boot_recog.pixa_u.get(idx) {
                let pix = pix.clone();
                self.add_sample(&pix, &label)?;
            }
        }

        self.finish_training()
    }
}

/// Generates a simple programmatic bitmap for digit `d` in a `w×h` canvas.
///
/// Each digit is drawn as a combination of filled rectangles approximating
/// a seven-segment display style to ensure distinct visual patterns.
fn make_digit_pix(d: u32, w: u32, h: u32) -> RecogResult<Pix> {
    let pix = Pix::new(w, h, PixelDepth::Bit1).map_err(RecogError::Core)?;
    let mut m = pix.try_into_mut().unwrap_or_else(|p| p.to_mut());

    let top = 0u32;
    let mid = h / 2;
    let bot = h.saturating_sub(1);
    let lft = 0u32;
    let rgt = w.saturating_sub(1);

    // Collect (x, y) pairs to set, then set them all at once.
    let mut pts: Vec<(u32, u32)> = Vec::new();

    let hbar = |pts: &mut Vec<_>, y: u32, x0: u32, x1: u32| {
        for x in x0..=x1 {
            pts.push((x, y));
        }
    };
    let vbar = |pts: &mut Vec<_>, x: u32, y0: u32, y1: u32| {
        for y in y0..=y1 {
            pts.push((x, y));
        }
    };

    match d {
        0 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, lft, top, bot);
            vbar(&mut pts, rgt, top, bot);
        }
        1 => {
            vbar(&mut pts, rgt, top, bot);
        }
        2 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, rgt, top, mid);
            vbar(&mut pts, lft, mid, bot);
        }
        3 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, rgt, top, bot);
        }
        4 => {
            hbar(&mut pts, mid, lft, rgt);
            vbar(&mut pts, lft, top, mid);
            vbar(&mut pts, rgt, top, bot);
        }
        5 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, lft, top, mid);
            vbar(&mut pts, rgt, mid, bot);
        }
        6 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, lft, top, bot);
            vbar(&mut pts, rgt, mid, bot);
        }
        7 => {
            hbar(&mut pts, top, lft, rgt);
            vbar(&mut pts, rgt, top, bot);
        }
        8 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, lft, top, bot);
            vbar(&mut pts, rgt, top, bot);
        }
        9 => {
            hbar(&mut pts, top, lft, rgt);
            hbar(&mut pts, mid, lft, rgt);
            hbar(&mut pts, bot, lft, rgt);
            vbar(&mut pts, lft, top, mid);
            vbar(&mut pts, rgt, top, bot);
        }
        _ => {}
    }

    for (x, y) in pts {
        let _ = m.set_pixel(x, y, 1);
    }

    Ok(m.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

    use crate::recog::train::create;

    #[test]
    fn test_make_boot_digit_recog_produces_ten_classes() {
        let boot = Recog::make_boot_digit_recog(30).unwrap();
        assert_eq!(boot.get_class_count(), 10);
        assert!(boot.train_done);
    }

    #[test]
    fn test_make_boot_digit_recog_invalid_scale() {
        assert!(Recog::make_boot_digit_recog(10).is_err());
    }

    #[test]
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
    fn test_pad_digit_training_set_fills_missing_classes() {
        // Use the same scale_h as the boot recognizer so template heights
        // are compatible (height ratio check won't fire).
        let scale_h = 30u32;
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        // Train with a solid block of similar height to the boot templates.
        let p = make_solid_pix(18, scale_h);
        recog.train_labeled(&p, "0").unwrap();
        recog.finish_training().unwrap();
        recog.charset_type = crate::recog::CharsetType::ArabicNumerals;

        recog.pad_digit_training_set(scale_h).unwrap();
        assert_eq!(recog.get_class_count(), 10);
    }

    #[test]
    fn test_train_from_boot() {
        let scale_h = 30u32;
        let boot = Recog::make_boot_digit_recog(scale_h).unwrap();

        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        // Same height so the height ratio check passes.
        let p = make_solid_pix(18, scale_h);
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
