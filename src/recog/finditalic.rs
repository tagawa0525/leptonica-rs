//! Italic word detection using hit-miss morphology
//!
//! Detect italic words in text images using hit-miss transforms
//! with diagonal structuring elements and binary reconstruction.
//!
//! # See also
//! C Leptonica: `finditalic.c`

use crate::core::{Boxa, Pix, PixelDepth};
use crate::morph;
use crate::morph::Sel;
use crate::recog::{RecogError, RecogResult};
use crate::region;
use crate::region::ConnectivityType;

// Hit-miss sels for italic slant detection (right edge)
const STR_ITAL1: &str = "\
   o x\n\
      \n\
      \n\
      \n\
  o x \n\
      \n\
  C   \n\
      \n\
 o x  \n\
      \n\
      \n\
      \n\
o x   ";

// Shorter version for ~200 ppi
#[allow(dead_code)]
const STR_ITAL2: &str = "\
   o x\n\
      \n\
      \n\
  o x \n\
  C   \n\
      \n\
 o x  \n\
      \n\
      \n\
o x   ";

// Noise removal sel
const STR_ITAL3: &str = "\
 x\n\
Cx\n\
x \n\
x ";

/// Detect italic words in a text image
///
/// Uses hit-miss morphology with diagonal sels to find italic edges,
/// then binary reconstruction to fill word masks containing italic seeds.
///
/// # Arguments
/// * `pix` - 1 bpp text image
/// * `boxaw` - optional word bounding boxes
/// * `pixw` - optional word mask (mutually exclusive with boxaw)
///
/// # Returns
/// Boxa of detected italic word bounding boxes
///
/// # See also
/// C Leptonica: `pixItalicWords()` in `finditalic.c`
pub fn italic_words(pix: &Pix, boxaw: Option<&Boxa>, pixw: Option<&Pix>) -> RecogResult<Boxa> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    if boxaw.is_some() && pixw.is_some() {
        return Err(RecogError::InvalidParameter(
            "both boxaw and pixw are defined".into(),
        ));
    }

    // Create italic detection sels
    let sel_ital1 = Sel::from_string(STR_ITAL1, 2, 6)?;
    let sel_ital3 = Sel::from_string(STR_ITAL3, 0, 1)?;

    // Make italic seed: HMT with diagonal sel, then denoise
    let pixsd = morph::hit_miss_transform(pix, &sel_ital1)?;
    let pixsd = morph::close(&pixsd, &sel_ital3)?;
    let pixsd = morph::open(&pixsd, &sel_ital3)?;

    // Make word mask
    let pixm = if let Some(boxes) = boxaw {
        // Create mask from word bounding boxes
        let mask = Pix::new(pix.width(), pix.height(), PixelDepth::Bit1)?;
        let mut mask_mut = mask
            .try_into_mut()
            .map_err(|_| RecogError::Core(crate::core::Error::AllocationFailed))?;
        for i in 0..boxes.len() {
            if let Some(b) = boxes.get(i) {
                let x0 = b.x.max(0) as u32;
                let y0 = b.y.max(0) as u32;
                let x1 = ((b.x + b.w) as u32).min(pix.width());
                let y1 = ((b.y + b.h) as u32).min(pix.height());
                for y in y0..y1 {
                    for x in x0..x1 {
                        mask_mut.set_pixel(x, y, 1)?;
                    }
                }
            }
        }
        let mask: Pix = mask_mut.into();
        mask
    } else if let Some(pw) = pixw {
        pw.clone()
    } else {
        // Generate word mask via morphology
        morph::morph_sequence(pix, "d1.5 + c15.1")?
    };

    // Binary reconstruction: fill mask components that have seed pixels
    // This is pixSeedfillBinary(NULL, pixsd, pixm, 8)
    let pixd = binary_seedfill(&pixsd, &pixm, ConnectivityType::EightWay)?;

    // Extract connected component bounding boxes
    let (boxa, _) = region::conncomp_pixa(&pixd, ConnectivityType::EightWay)?;

    Ok(boxa)
}

/// Binary morphological reconstruction: expand seed within mask
///
/// Iteratively dilates seed, AND with mask, until stable.
fn binary_seedfill(seed: &Pix, mask: &Pix, connectivity: ConnectivityType) -> RecogResult<Pix> {
    let w = mask.width();
    let h = mask.height();

    let mut current = seed.clone();

    // Iterative dilation + AND with mask
    let sel = match connectivity {
        ConnectivityType::FourWay => morph::Sel::create_cross(3)?,
        ConnectivityType::EightWay => morph::Sel::create_square(3)?,
    };

    loop {
        let dilated = morph::dilate(&current, &sel)?;

        // AND with mask
        let new_pix = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut new_mut = new_pix
            .try_into_mut()
            .map_err(|_| RecogError::Core(crate::core::Error::AllocationFailed))?;
        for y in 0..h {
            for x in 0..w {
                let dv = dilated.get_pixel(x, y).unwrap_or(0);
                let mv = mask.get_pixel(x, y).unwrap_or(0);
                if dv != 0 && mv != 0 {
                    new_mut.set_pixel_unchecked(x, y, 1);
                }
            }
        }
        let result: Pix = new_mut.into();

        // Check if converged
        if result.count_pixels() == current.count_pixels() {
            return Ok(result);
        }
        current = result;
    }
}
