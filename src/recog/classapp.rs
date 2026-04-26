//! Word and character box detection and image comparison by box patterns.
//!
//! This module provides functions for:
//! - Finding word and character bounding boxes in document images
//! - Extracting sorted spatial patterns from box arrays
//! - Comparing document images by their box patterns
//!
//! # C Leptonica equivalents
//!
//! - `pixFindWordAndCharacterBoxes` → [`find_word_and_character_boxes`]
//! - `boxaExtractSortedPattern` → [`boxa_extract_sorted_pattern`]
//! - `numaaCompareImagesByBoxes` → [`numaa_compare_images_by_boxes`]

use crate::core::{Box, Boxa, Boxaa, Numa, Numaa, Pix, PixelDepth, SizeRelation};
use crate::recog::util::ensure_binary_with_threshold;
use crate::recog::{RecogError, RecogResult};

/// Find word and character bounding boxes in a document image.
///
/// Given an input image (any depth), this function:
/// 1. Optionally clips to a sub-region (`box_s`)
/// 2. Converts to grayscale, then binarizes with `thresh`
/// 3. Scales to ~120 ppi to find word boxes via textline detection
/// 4. For each word, finds character boxes via morphological closing and
///    connected component extraction at full (300 ppi) resolution
///
/// Returns `(word_boxes, char_boxes_per_word)` where `char_boxes_per_word`
/// is a [`Boxaa`] with one [`Boxa`] per word containing character boxes.
///
/// All returned coordinates are relative to the full input image `pix_s`.
///
/// # Arguments
///
/// * `pix_s` - Input image (any depth, but not 1 bpp)
/// * `box_s` - Optional clipping region within `pix_s`
/// * `thresh` - Binarization threshold (typically 100-150)
///
/// # C Leptonica equivalent
///
/// `pixFindWordAndCharacterBoxes`
pub fn find_word_and_character_boxes(
    pix_s: &Pix,
    box_s: Option<&Box>,
    thresh: u32,
) -> RecogResult<(Boxa, Boxaa)> {
    if pix_s.depth() == PixelDepth::Bit1 {
        return Err(RecogError::InvalidParameter(
            "input image must not be 1 bpp".to_string(),
        ));
    }

    // Clip to sub-region if specified
    let (clipped, xs, ys) = if let Some(bs) = box_s {
        let c = pix_s.clip_rectangle(
            bs.x.max(0) as u32,
            bs.y.max(0) as u32,
            bs.w.max(1) as u32,
            bs.h.max(1) as u32,
        )?;
        (Some(c), bs.x, bs.y)
    } else {
        (None, 0, 0)
    };

    // Convert to 8 bpp grayscale
    let pix2 = clipped.as_ref().unwrap_or(pix_s).convert_to_8()?;

    // Binarize with low threshold to reduce touching characters
    let pix3 = ensure_binary_with_threshold(&pix2, thresh)?;

    // Scale to ~120 ppi for word detection (assuming 300 ppi source)
    let xres = pix3.xres();
    let effective_res = if xres > 0 { xres as f32 } else { 300.0 };
    let scalefact = 120.0 / effective_res;
    let pix3a = crate::transform::scale_by_sampling(&pix3, scalefact, scalefact)?;

    // Find word boxes at reduced resolution.
    // Use permissive size filters; the dilation-based algorithm handles word grouping.
    let (word_boxes_scaled, _line_indices) =
        crate::recog::pageseg::get_word_boxes_in_textlines(&pix3a, 2, 2, 5000, 5000)?;

    // Scale word boxes back to full resolution
    let inv_scale = 1.0 / scalefact;
    let boxa1: Boxa = word_boxes_scaled
        .into_iter()
        .map(|b| {
            Box::new_unchecked(
                (b.x as f32 * inv_scale).round() as i32,
                (b.y as f32 * inv_scale).round() as i32,
                (b.w as f32 * inv_scale).round() as i32,
                (b.h as f32 * inv_scale).round() as i32,
            )
        })
        .collect();

    // Find character boxes within each word at full resolution
    let nb = boxa1.len();
    let mut boxaw = Boxa::with_capacity(nb);
    let mut boxaac = Boxaa::with_capacity(nb);

    for i in 0..nb {
        let box1 = *boxa1.get(i).unwrap();
        let xb = box1.x;
        let yb = box1.y;

        // Clip word region from the binary image
        let pix4 = pix3.clip_rectangle(
            box1.x.max(0) as u32,
            box1.y.max(0) as u32,
            box1.w.max(1) as u32,
            box1.h.max(1) as u32,
        )?;

        // Join detached parts of characters vertically
        let pix5 = crate::morph::sequence::morph_sequence(&pix4, "c1.10")?;

        // Find connected components (mostly characters)
        let components =
            crate::region::conncomp::find_connected_components(&pix5, Default::default())?;

        // Extract bounding boxes and filter small pieces
        let boxa2: Boxa = components.iter().map(|c| c.bounds).collect();
        let boxa3 = boxa2.select_by_size(2, 5, SizeRelation::GreaterThanOrEqual);

        // Sort left-to-right by x position
        let mut boxa4 = boxa3;
        boxa4.boxes_mut().sort_by_key(|b| b.x);

        // Transform to coordinates relative to full input image
        let boxa5: Boxa = boxa4
            .iter()
            .map(|b| Box::new_unchecked((b.x + xs + xb).max(0), (b.y + ys + yb).max(0), b.w, b.h))
            .collect();
        let box2 = Box::new_unchecked((box1.x + xs).max(0), (box1.y + ys).max(0), box1.w, box1.h);

        // Only include words that have characters after filtering
        if !boxa5.is_empty() {
            boxaw.push(box2);
            boxaac.push(boxa5);
        }
    }

    Ok((boxaw, boxaac))
}

/// Extract sorted spatial pattern from word/character boxes.
///
/// Given a [`Boxa`] of character boxes and a [`Numa`] of word indices
/// (indicating which word each character belongs to), produces a [`Numaa`]
/// where each [`Numa`] represents one word/line and contains:
/// - First element: y-center of the line (y + h/2)
/// - Subsequent pairs: (x_left, x_right) for each character box
///
/// This pattern can be used to compare document layouts.
///
/// # Arguments
///
/// * `boxa` - Character bounding boxes sorted by position
/// * `na` - Word/row index for each character box
///
/// # C Leptonica equivalent
///
/// `boxaExtractSortedPattern`
pub fn boxa_extract_sorted_pattern(boxa: &Boxa, na: &Numa) -> RecogResult<Numaa> {
    let nbox = boxa.len();
    let mut naa = Numaa::new();

    if nbox == 0 {
        return Ok(naa);
    }

    let mut nad = Numa::new();
    let mut prevrow: i32 = -1;

    for index in 0..nbox {
        let b = boxa.get(index).ok_or_else(|| {
            RecogError::InvalidParameter(format!("box index {index} out of range"))
        })?;
        let row = na.get_i32(index).ok_or_else(|| {
            RecogError::InvalidParameter(format!("numa index {index} out of range"))
        })?;

        if row > prevrow {
            if index > 0 {
                // Push the previous row's data
                let finished = std::mem::replace(&mut nad, Numa::new());
                naa.push(finished);
            }
            prevrow = row;
            // First entry for a new row: y-center
            nad.push((b.y + b.h / 2) as f32);
        }
        // Left and right x coordinates for this box
        nad.push(b.x as f32);
        nad.push((b.x + b.w - 1) as f32);
    }

    // Push the last row
    naa.push(nad);

    Ok(naa)
}

/// Compare two document images by their box patterns.
///
/// Determines whether two document images are "the same" by comparing
/// the spatial patterns of their word/character bounding boxes.
///
/// The algorithm:
/// 1. For each line (row) in both patterns, checks if it has enough boxes
///    (`>= nperline`)
/// 2. Finds candidate line-to-line matches where the x/y shifts of the
///    first box are within `maxshiftx`/`maxshifty`
/// 3. For each candidate, verifies that `nperline` consecutive box edges
///    align within `delx` tolerance
/// 4. Among all valid matches, finds `nreq` mutually consistent matches
///    (shifts within `delx`/`dely` of each other, no row reuse)
///
/// Returns `true` if the images are considered the same.
///
/// # Arguments
///
/// * `naa1`, `naa2` - Box patterns from [`boxa_extract_sorted_pattern`]
/// * `nperline` - Minimum boxes per line to consider (>= 1)
/// * `nreq` - Required number of matching lines (>= 1)
/// * `maxshiftx` - Maximum horizontal shift for first-box matching
/// * `maxshifty` - Maximum vertical shift for first-box matching
/// * `delx` - Tolerance for box edge alignment
/// * `dely` - Tolerance for shift consistency between matched lines
///
/// # C Leptonica equivalent
///
/// `numaaCompareImagesByBoxes`
#[allow(clippy::too_many_arguments)]
pub fn numaa_compare_images_by_boxes(
    naa1: &Numaa,
    naa2: &Numaa,
    nperline: usize,
    nreq: usize,
    maxshiftx: i32,
    maxshifty: i32,
    delx: i32,
    dely: i32,
) -> RecogResult<bool> {
    if nperline < 1 {
        return Err(RecogError::InvalidParameter(
            "nperline must be >= 1".to_string(),
        ));
    }
    if nreq < 1 {
        return Err(RecogError::InvalidParameter(
            "nreq must be >= 1".to_string(),
        ));
    }

    let n1 = naa1.len();
    let n2 = naa2.len();

    if n1 < nreq || n2 < nreq {
        return Ok(false);
    }

    // Extract metadata for each line: sufficient boxes, y-location, left-x
    let mut line1 = vec![false; n1];
    let mut yloc1 = vec![0i32; n1];
    let mut xleft1 = vec![0i32; n1];
    let mut line2 = vec![false; n2];
    let mut yloc2 = vec![0i32; n2];
    let mut xleft2 = vec![0i32; n2];

    for i in 0..n1 {
        let na = naa1.get(i).unwrap();
        yloc1[i] = na.get_i32(0).unwrap_or(0);
        xleft1[i] = na.get_i32(1).unwrap_or(0);
        let nbox = (na.len().saturating_sub(1)) / 2;
        if nbox >= nperline {
            line1[i] = true;
        }
    }
    for i in 0..n2 {
        let na = naa2.get(i).unwrap();
        yloc2[i] = na.get_i32(0).unwrap_or(0);
        xleft2[i] = na.get_i32(1).unwrap_or(0);
        let nbox = (na.len().saturating_sub(1)) / 2;
        if nbox >= nperline {
            line2[i] = true;
        }
    }

    // Find candidate line matches
    let mut match_i1 = Vec::new();
    let mut match_i2 = Vec::new();
    let mut match_sx = Vec::new();
    let mut match_sy = Vec::new();

    for i in 0..n1 {
        if !line1[i] {
            continue;
        }
        let y1 = yloc1[i];
        let xl1 = xleft1[i];
        let na1 = naa1.get(i).unwrap();

        for j in 0..n2 {
            if !line2[j] {
                continue;
            }
            let y2 = yloc2[j];
            if (y1 - y2).abs() > maxshifty {
                continue;
            }
            let xl2 = xleft2[j];
            if (xl1 - xl2).abs() > maxshiftx {
                continue;
            }
            let shiftx = xl1 - xl2;
            let shifty = y1 - y2;
            let na2 = naa2.get(j).unwrap();

            if test_line_alignment_x(na1, na2, shiftx, delx, nperline) {
                match_i1.push(i);
                match_i2.push(j);
                match_sx.push(shiftx);
                match_sy.push(shifty);
            }
        }
    }

    // Find sufficient mutually aligned matches
    count_aligned_matches(
        &match_i1, &match_i2, &match_sx, &match_sy, n1, n2, delx, dely, nreq,
    )
}

/// Test if two lines align by checking `nperline` consecutive box edges.
///
/// Each Numa contains: [y_center, xl_0, xr_0, xl_1, xr_1, ...]
/// Checks that left and right edges of boxes 0..nperline match within `delx`
/// after compensating for the horizontal offset `shiftx` (image1 − image2).
fn test_line_alignment_x(na1: &Numa, na2: &Numa, shiftx: i32, delx: i32, nperline: usize) -> bool {
    for i in 0..nperline {
        // Box edges are at indices: 1 + 2*i (left), 2 + 2*i (right)
        let xl1 = match na1.get_i32(1 + 2 * i) {
            Some(v) => v,
            None => return false,
        };
        let xr1 = match na1.get_i32(2 + 2 * i) {
            Some(v) => v,
            None => return false,
        };
        let xl2 = match na2.get_i32(1 + 2 * i) {
            Some(v) => v,
            None => return false,
        };
        let xr2 = match na2.get_i32(2 + 2 * i) {
            Some(v) => v,
            None => return false,
        };

        let diffl = (xl1 - xl2 - shiftx).abs();
        let diffr = (xr1 - xr2 - shiftx).abs();
        if diffl > delx || diffr > delx {
            return false;
        }
    }
    true
}

/// Find sufficient mutually aligned matches among candidate line matches.
///
/// For each candidate match as a reference, counts how many other matches
/// have compatible shifts (within `delx`/`dely`) without reusing rows.
/// Returns `true` if `nreq` aligned matches are found.
#[allow(clippy::too_many_arguments)]
fn count_aligned_matches(
    ia1: &[usize],
    ia2: &[usize],
    iasx: &[i32],
    iasy: &[i32],
    n1: usize,
    n2: usize,
    delx: i32,
    dely: i32,
    nreq: usize,
) -> RecogResult<bool> {
    let nm = ia1.len();
    if nm < nreq {
        return Ok(false);
    }

    let mut index1 = vec![0u32; n1];
    let mut index2 = vec![0u32; n2];

    for i in 0..nm {
        // Reset row tracking
        index1.fill(0);
        index2.fill(0);

        let mut nmatch = 1u32;
        index1[ia1[i]] = nmatch;
        index2[ia2[i]] = nmatch;
        let shiftx = iasx[i];
        let shifty = iasy[i];

        if nreq == 1 {
            return Ok(true);
        }

        for j in 0..nm {
            if j == i {
                continue;
            }
            // Rows must not have been used already
            if index1[ia1[j]] > 0 || index2[ia2[j]] > 0 {
                continue;
            }
            // Check shift compatibility
            let diffx = (shiftx - iasx[j]).abs();
            let diffy = (shifty - iasy[j]).abs();
            if diffx > delx || diffy > dely {
                continue;
            }
            nmatch += 1;
            index1[ia1[j]] = nmatch;
            index2[ia2[j]] = nmatch;
            if nmatch as usize >= nreq {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
