//! Stroke width detection and modification for binary images
//!
//! Operations on 1 bpp images to measure stroke parameters (length and
//! average width) and change the average stroke width by eroding or
//! dilating the image.
//!
//! # See also
//! C Leptonica: `strokes.c`

use crate::core::{Numa, Pix, Pixa, PixelDepth};
use crate::morph;
use crate::morph::binary::BoundaryType;
use crate::recog::{RecogError, RecogResult};
use crate::region;

/// Find total stroke length in a 1 bpp image
///
/// Returns half the number of foreground boundary pixels.
///
/// # See also
/// C Leptonica: `pixFindStrokeLength()` in `strokes.c`
pub fn find_stroke_length(pix: &Pix) -> RecogResult<u32> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let boundary = morph::extract_boundary(pix, BoundaryType::Inner)?;
    let n = boundary.count_pixels();
    Ok((n / 2) as u32)
}

/// Find average stroke width in a 1 bpp image
///
/// Uses two methods:
/// 1. Width = pixel_count / stroke_length (from boundary)
/// 2. Distance transform histogram analysis
///
/// Returns the average of both methods, plus optionally the distance histogram.
///
/// # Arguments
/// * `pix` - 1 bpp input
/// * `thresh` - fractional threshold relative to distance 1 (typically 0.15)
///
/// # See also
/// C Leptonica: `pixFindStrokeWidth()` in `strokes.c`
pub fn find_stroke_width(pix: &Pix, thresh: f32) -> RecogResult<(f32, Option<Numa>)> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    // Method 1: via boundary length
    let length = find_stroke_length(pix)?;
    if length == 0 {
        return Ok((0.0, None));
    }
    let count = pix.count_pixels();
    let width1 = count as f32 / length as f32;

    // Method 2: via distance transform
    let dist_pix = region::distance_function(
        pix,
        region::ConnectivityType::EightWay,
        PixelDepth::Bit8,
        region::BoundaryCondition::Background,
    )?;
    let na1 = dist_pix.gray_histogram(1)?;

    // Find non-zero range
    let range = na1.get_nonzero_range(0.1)?;
    let last = match range {
        Some((_, last)) => last,
        None => return Ok((width1, None)),
    };

    let na2 = na1.clip_to_interval(0, last)?;
    let n = na2.len();
    if n <= 1 {
        return Ok((width1, Some(na2)));
    }

    // Find bucket with largest distance whose count exceeds threshold
    let val1 = na2.get(1).unwrap_or(1.0);
    if val1 == 0.0 {
        return Ok((width1, Some(na2)));
    }

    let mut last_i = 1usize;
    let mut ratio = 1.0f32;
    for i in (1..n).rev() {
        ratio = na2.get(i).unwrap_or(0.0) / val1;
        if ratio > thresh {
            last_i = i;
            break;
        }
    }

    let extra = if last_i < n - 1 {
        na2.get(last_i + 1).unwrap_or(0.0) / val1
    } else {
        0.0
    };
    let width2 = 2.0 * (last_i as f32 - 1.0 + ratio + extra);

    let width = (width1 + width2) / 2.0;

    Ok((width, Some(na2)))
}

/// Find stroke widths for multiple images
///
/// # See also
/// C Leptonica: `pixaFindStrokeWidth()` in `strokes.c`
pub fn pixa_find_stroke_width(pixa: &Pixa, thresh: f32) -> RecogResult<Numa> {
    let mut na = Numa::with_capacity(pixa.len());
    for i in 0..pixa.len() {
        let pix = pixa.get(i).ok_or_else(|| {
            RecogError::InvalidParameter(format!("pixa index {} out of range", i))
        })?;
        if pix.depth() != PixelDepth::Bit1 {
            return Err(RecogError::UnsupportedDepth {
                expected: "1 bpp",
                actual: pix.depth().bits(),
            });
        }
        let (width, _) = find_stroke_width(pix, thresh)?;
        na.push(width);
    }
    Ok(na)
}

/// Modify stroke widths of multiple images to a target width
///
/// # See also
/// C Leptonica: `pixaModifyStrokeWidth()` in `strokes.c`
pub fn pixa_modify_stroke_width(pixa: &Pixa, target_w: f32) -> RecogResult<Pixa> {
    if target_w < 1.0 {
        return Err(RecogError::InvalidParameter(
            "target width must be >= 1".into(),
        ));
    }

    let na = pixa_find_stroke_width(pixa, 0.1)?;
    let mut pixad = Pixa::with_capacity(pixa.len());

    for i in 0..pixa.len() {
        let pix = pixa.get(i).ok_or_else(|| {
            RecogError::InvalidParameter(format!("pixa index {} out of range", i))
        })?;
        let width = na.get(i).unwrap_or(0.0);
        let result = modify_stroke_width(pix, width, target_w)?;
        pixad.push(result);
    }

    Ok(pixad)
}

/// Modify stroke width of a single image
///
/// Computes the difference between target and current width, then applies
/// morphological erosion (if narrowing) or dilation (if widening).
///
/// # See also
/// C Leptonica: `pixModifyStrokeWidth()` in `strokes.c`
pub fn modify_stroke_width(pix: &Pix, width: f32, target_w: f32) -> RecogResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }
    if target_w < 1.0 {
        return Err(RecogError::InvalidParameter(
            "target width must be >= 1".into(),
        ));
    }

    let diff = (target_w - width).round() as i32;
    if diff == 0 {
        return Ok(pix.clone());
    }

    let size = diff.unsigned_abs() + 1;
    let seq = if diff < 0 {
        format!("e{size}.{size}")
    } else {
        format!("d{size}.{size}")
    };

    Ok(morph::morph_sequence(pix, &seq)?)
}

/// Set stroke width of multiple images to target
///
/// Optionally thins each image to a skeleton first, then dilates to target.
///
/// # Arguments
/// * `pixa` - array of 1 bpp images
/// * `width` - target stroke width [1..100]
/// * `thin_first` - true to thin to skeleton first
/// * `connectivity` - 4 or 8 connectivity for thinning
///
/// # See also
/// C Leptonica: `pixaSetStrokeWidth()` in `strokes.c`
pub fn pixa_set_stroke_width(
    pixa: &Pixa,
    width: u32,
    thin_first: bool,
    connectivity: morph::Connectivity,
) -> RecogResult<Pixa> {
    if !(1..=100).contains(&width) {
        return Err(RecogError::InvalidParameter(
            "width must be in [1..100]".into(),
        ));
    }

    let mut pixad = Pixa::with_capacity(pixa.len());
    for i in 0..pixa.len() {
        let pix = pixa.get(i).ok_or_else(|| {
            RecogError::InvalidParameter(format!("pixa index {} out of range", i))
        })?;
        if pix.depth() != PixelDepth::Bit1 {
            return Err(RecogError::UnsupportedDepth {
                expected: "1 bpp",
                actual: pix.depth().bits(),
            });
        }
        let result = set_stroke_width(pix, width, thin_first, connectivity)?;
        pixad.push(result);
    }

    Ok(pixad)
}

/// Set stroke width of a single image to target
///
/// Adds a white border, optionally thins to skeleton, then dilates.
///
/// # See also
/// C Leptonica: `pixSetStrokeWidth()` in `strokes.c`
pub fn set_stroke_width(
    pix: &Pix,
    width: u32,
    thin_first: bool,
    connectivity: morph::Connectivity,
) -> RecogResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }
    if !(1..=100).contains(&width) {
        return Err(RecogError::InvalidParameter(
            "width must be in [1..100]".into(),
        ));
    }

    if !thin_first && width == 1 {
        return Ok(pix.clone());
    }

    // Add white border
    let border = width / 2;
    let pix1 = pix.add_border(border, 0)?;

    // Thin to skeleton if requested
    let pix2 = if thin_first {
        morph::thin_connected(&pix1, morph::ThinType::Foreground, connectivity, 0)?
    } else {
        pix1
    };

    // Dilate to target width
    let seq = format!("D{width}.{width}");
    let pixd = morph::morph_sequence(&pix2, &seq)?;

    Ok(pixd)
}
