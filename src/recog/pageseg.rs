//! Page segmentation
//!
//! This module provides functionality to segment document pages into regions:
//! - Halftone (image) regions
//! - Text line regions
//! - Text block regions
//!
//! # Algorithm Overview
//!
//! 1. **Halftone Detection**: Identify screened halftone regions using
//!    morphological operations to detect regular dot patterns.
//!
//! 2. **Text Line Detection**: Use horizontal morphological operations
//!    to connect characters into lines.
//!
//! 3. **Text Block Detection**: Group text lines into paragraphs/blocks
//!    by detecting vertical whitespace between blocks.

use crate::core::{Pix, PixelDepth};
use crate::recog::util::ensure_binary_with_threshold;
use crate::recog::{RecogError, RecogResult};

/// Minimum dimensions for page segmentation
const MIN_WIDTH: u32 = 100;
const MIN_HEIGHT: u32 = 100;

/// Options for page segmentation
#[derive(Debug, Clone)]
pub struct PageSegOptions {
    /// Minimum width for detected regions (default: 100)
    pub min_width: u32,

    /// Minimum height for detected regions (default: 100)
    pub min_height: u32,

    /// Whether to detect halftone regions (default: true)
    pub detect_halftone: bool,

    /// Horizontal closing size for text lines (default: 25)
    pub textline_close_h: u32,

    /// Vertical closing size for text lines (default: 1)
    pub textline_close_v: u32,
}

impl Default for PageSegOptions {
    fn default() -> Self {
        Self {
            min_width: MIN_WIDTH,
            min_height: MIN_HEIGHT,
            detect_halftone: true,
            textline_close_h: 25,
            textline_close_v: 1,
        }
    }
}

impl PageSegOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum region width
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = width;
        self
    }

    /// Set minimum region height
    pub fn with_min_height(mut self, height: u32) -> Self {
        self.min_height = height;
        self
    }

    /// Enable or disable halftone detection
    pub fn with_detect_halftone(mut self, detect: bool) -> Self {
        self.detect_halftone = detect;
        self
    }

    /// Set text line closing parameters
    pub fn with_textline_closing(mut self, h: u32, v: u32) -> Self {
        self.textline_close_h = h;
        self.textline_close_v = v;
        self
    }

    /// Validate options
    fn validate(&self) -> RecogResult<()> {
        if self.min_width < 10 {
            return Err(RecogError::InvalidParameter(
                "min_width must be at least 10".to_string(),
            ));
        }
        if self.min_height < 10 {
            return Err(RecogError::InvalidParameter(
                "min_height must be at least 10".to_string(),
            ));
        }
        if self.textline_close_h == 0 {
            return Err(RecogError::InvalidParameter(
                "textline_close_h must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

/// Result of page segmentation
#[derive(Debug)]
pub struct SegmentationResult {
    /// Halftone (image) region mask (if detected)
    pub halftone_mask: Option<Pix>,

    /// Text line mask
    pub textline_mask: Pix,

    /// Text block mask
    pub textblock_mask: Pix,
}

/// Segment a page into halftone, textline, and textblock regions
///
/// # Arguments
/// * `pix` - Input image (1 bpp binary recommended)
/// * `options` - Segmentation options
///
/// # Returns
/// SegmentationResult containing masks for each region type
///
/// # Example
/// ```no_run
/// use leptonica::recog::pageseg::{segment_regions, PageSegOptions};
/// use leptonica::core::{Pix, PixelDepth};
///
/// let pix = Pix::new(500, 700, PixelDepth::Bit1).unwrap();
/// let result = segment_regions(&pix, &PageSegOptions::default()).unwrap();
/// println!("Text lines found in textline_mask");
/// ```
pub fn segment_regions(pix: &Pix, options: &PageSegOptions) -> RecogResult<SegmentationResult> {
    options.validate()?;

    let w = pix.width();
    let h = pix.height();

    if w < MIN_WIDTH || h < MIN_HEIGHT {
        return Err(RecogError::ImageTooSmall {
            min_width: MIN_WIDTH,
            min_height: MIN_HEIGHT,
            actual_width: w,
            actual_height: h,
        });
    }

    // Ensure binary
    let binary = ensure_binary(pix)?;

    // Reduce to half resolution for processing
    let reduced = reduce_by_2(&binary)?;

    // Step 1: Detect halftone regions (optional)
    let (halftone_mask, text_pixels) = if options.detect_halftone {
        let (hm, tp) = generate_halftone_mask(&reduced)?;
        (Some(expand_by_2(&hm, w, h)?), tp)
    } else {
        (None, reduced.deep_clone())
    };

    // Step 2: Generate text line mask
    let (textline_mask_2x, vws) = generate_textline_mask_internal(
        &text_pixels,
        options.textline_close_h / 2,
        options.textline_close_v,
    )?;

    // Step 3: Generate text block mask
    let textblock_mask_2x = generate_textblock_mask_internal(&textline_mask_2x, &vws)?;

    // Expand masks to full resolution
    let textline_mask = expand_by_2(&textline_mask_2x, w, h)?;
    let textblock_mask = expand_by_2(&textblock_mask_2x, w, h)?;

    Ok(SegmentationResult {
        halftone_mask,
        textline_mask,
        textblock_mask,
    })
}

/// Estimate the background gray level of an 8 bpp image.
///
/// Optionally crops the inner part by `edgecrop` (in `[0.0, 1.0)`) and
/// excludes pixels darker than `darkthresh` from the median. Returns the
/// median gray level in `[0, 255]`.
///
/// C Leptonica equivalent: `pixEstimateBackground` (`pageseg.c`).
pub fn estimate_background(pix: &Pix, darkthresh: u32, edgecrop: f32) -> RecogResult<u32> {
    use crate::core::pix::convert::RemoveColormapTarget;
    if !(0.0..1.0).contains(&edgecrop) || !edgecrop.is_finite() {
        return Err(RecogError::InvalidParameter(format!(
            "edgecrop must be in [0.0, 1.0); got {edgecrop}"
        )));
    }

    // Convert (or pass through) to 8 bpp grayscale.
    let pix1 = if pix.depth() == PixelDepth::Bit8 && pix.colormap().is_none() {
        pix.deep_clone()
    } else if pix.depth() == PixelDepth::Bit8 {
        pix.remove_colormap(RemoveColormapTarget::ToGrayscale)
            .map_err(|e| RecogError::InvalidParameter(format!("remove_colormap: {e}")))?
    } else {
        return Err(RecogError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth().bits(),
        });
    };

    let w = pix1.width();
    let h = pix1.height();

    // Optionally crop the inner part.
    let pix2 = if edgecrop > 0.0 {
        let cx = (0.5 * edgecrop * w as f32) as u32;
        let cy = (0.5 * edgecrop * h as f32) as u32;
        let cw = ((1.0 - edgecrop) * w as f32) as u32;
        let ch = ((1.0 - edgecrop) * h as f32) as u32;
        if cw == 0 || ch == 0 {
            return Err(RecogError::InvalidParameter(format!(
                "edgecrop {edgecrop} leaves no inner area for {w}x{h}"
            )));
        }
        pix1.clip_rectangle(cx, cy, cw, ch)
            .map_err(|e| RecogError::InvalidParameter(format!("clip_rectangle: {e}")))?
    } else {
        pix1
    };

    // No more than 50K samples.
    let cw = pix2.width() as f64;
    let ch = pix2.height() as f64;
    let sampling = (((cw * ch) / 50000.0 + 0.5).sqrt() as u32).max(1);

    // Optional dark-pixel mask.
    let mask = if darkthresh > 0 {
        let dark = crate::color::threshold::threshold_to_binary(&pix2, darkthresh as u8)
            .map_err(|e| RecogError::InvalidParameter(format!("threshold_to_binary: {e}")))?;
        Some(dark.invert())
    } else {
        None
    };

    let (val, _) = pix2
        .rank_value_masked(mask.as_ref(), 0, 0, sampling, 0.5)
        .map_err(|e| RecogError::InvalidParameter(format!("rank_value_masked: {e}")))?;
    Ok((val + 0.5) as u32)
}

/// Generate a text line mask from a binary image
///
/// # Arguments
/// * `pix` - Input binary image
///
/// # Returns
/// Tuple of (text line mask, vertical whitespace mask)
pub fn generate_textline_mask(pix: &Pix) -> RecogResult<(Pix, Pix)> {
    let binary = ensure_binary(pix)?;
    generate_textline_mask_internal(&binary, 25, 1)
}

/// Generate a text block mask from text lines and vertical whitespace
///
/// # Arguments
/// * `textline_mask` - Text line mask
/// * `vws` - Vertical whitespace mask
///
/// # Returns
/// Text block mask
pub fn generate_textblock_mask(textline_mask: &Pix, vws: &Pix) -> RecogResult<Pix> {
    generate_textblock_mask_internal(textline_mask, vws)
}

/// Extract individual text lines from an image
///
/// # Arguments
/// * `pix` - Input image
///
/// # Returns
/// Vector of images, each containing one text line
pub fn extract_textlines(pix: &Pix) -> RecogResult<Vec<Pix>> {
    let binary = ensure_binary(pix)?;
    let (textline_mask, _) = generate_textline_mask(&binary)?;

    // Find connected components in the text line mask
    let components = find_connected_components(&textline_mask)?;

    // Extract each component as a separate image
    let mut lines = Vec::new();
    for (x, y, w, h) in components {
        if w >= 20 && h >= 5 {
            // Minimum line dimensions
            let line = extract_region(&binary, x, y, w, h)?;
            lines.push(line);
        }
    }

    // Sort by y-coordinate (top to bottom)
    // Components are already in order from find_connected_components

    Ok(lines)
}

/// Determine if a region contains primarily text (vs image)
///
/// # Arguments
/// * `pix` - Input region image
///
/// # Returns
/// true if the region appears to be text
pub fn is_text_region(pix: &Pix) -> RecogResult<bool> {
    let binary = ensure_binary(pix)?;
    let w = binary.width();
    let h = binary.height();

    // Count pixels and compute row/column counts simultaneously
    let mut black_count = 0u64;
    let mut row_counts = vec![0u32; h as usize];
    let mut col_counts = vec![0u32; w as usize];
    for y in 0..h {
        for x in 0..w {
            let val = binary.get_pixel_unchecked(x, y);
            if val != 0 {
                black_count += 1;
                row_counts[y as usize] += 1;
                col_counts[x as usize] += 1;
            }
        }
    }

    let total = (w as u64) * (h as u64);
    let density = black_count as f64 / total as f64;

    // Text typically has 5-40% black pixel density.
    // Photos binarized at 128 threshold often have higher density.
    let is_text_density = density > 0.02 && density < 0.40;

    // Compute variance of row counts and column counts.
    // Text has highly structured row patterns: alternating text lines
    // and whitespace create large row-count variance.
    let h_variance = compute_variance(&col_counts);
    let v_variance = compute_variance(&row_counts);

    // Use coefficient of variation (CV = stddev / mean) to normalize for image size.
    // Text has high CV in at least one direction because of the alternating
    // line/gap pattern. Photos tend to have more uniform distribution.
    let mean_row = if h > 0 {
        black_count as f64 / h as f64
    } else {
        0.0
    };
    let mean_col = if w > 0 {
        black_count as f64 / w as f64
    } else {
        0.0
    };

    let cv_row = if mean_row > 0.0 {
        v_variance.sqrt() / mean_row
    } else {
        0.0
    };
    let cv_col = if mean_col > 0.0 {
        h_variance.sqrt() / mean_col
    } else {
        0.0
    };

    // Text documents have high CV in the row direction (vertical variance)
    // because text lines create strong horizontal banding.
    // A CV > 0.3 indicates significant structured variation.
    // Photos typically have CV < 0.3 because pixel density is more uniform.
    let max_cv = cv_row.max(cv_col);
    let has_text_pattern = max_cv > 0.3;

    // Additional check: the variance ratio between directions.
    // Text is strongly directional (one direction has much higher variance).
    // Photos tend to have similar variance in both directions.
    let min_variance = h_variance.min(v_variance).max(1.0);
    let max_variance = h_variance.max(v_variance);
    let variance_ratio = max_variance / min_variance;
    let has_directional_pattern = variance_ratio > 1.5;

    Ok(is_text_density && has_text_pattern && has_directional_pattern)
}

/// Finds the bounding box of the foreground region of a scanned page.
///
/// Uses morphological operations to detect the main content area, removing
/// border noise based on `mindist` and `erasedist`.
///
/// Corresponds to `pixFindPageForeground` in C Leptonica.
pub fn find_page_foreground(
    pix: &Pix,
    threshold: u32,
    mindist: i32,
    erasedist: i32,
) -> RecogResult<crate::core::Box> {
    let binary = ensure_binary_with_threshold(pix, threshold)?;
    let w = binary.width();
    let h = binary.height();

    // Find foreground bounding box
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..h {
        for x in 0..w {
            if binary.get_pixel_unchecked(x, y) != 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if max_x < min_x || max_y < min_y {
        return Err(RecogError::NoContent(
            "no foreground pixels found".to_string(),
        ));
    }

    // Apply mindist: shrink from edges
    let md = mindist.max(0) as u32;
    let ed = erasedist.max(0) as u32;
    let inset = md + ed;
    let fx = if min_x < inset { 0 } else { min_x };
    let fy = if min_y < inset { 0 } else { min_y };
    let fw = (max_x + 1).min(w) - fx;
    let fh = (max_y + 1).min(h) - fy;

    if fw == 0 || fh == 0 {
        return Err(RecogError::NoContent(
            "foreground region too small after edge removal".to_string(),
        ));
    }

    Ok(crate::core::Box::new_unchecked(
        fx as i32, fy as i32, fw as i32, fh as i32,
    ))
}

/// Splits a text line image into individual character bounding boxes.
///
/// Filters small components, consolidates with vertical closing, then
/// extracts 8-connected components and optionally splits wide components
/// using projection profiles.
///
/// Corresponds to `pixSplitIntoCharacters` in C Leptonica.
pub fn pix_split_into_characters(
    pix: &Pix,
    minw: u32,
    minh: u32,
) -> RecogResult<(Vec<crate::core::Box>, Option<Vec<Pix>>)> {
    let binary = ensure_binary(pix)?;

    // Vertical close for consolidation
    let closed = morphological_close(&binary, 1, 10)?;

    // Find connected components
    let components = find_connected_components(&closed)?;

    let mut boxes = Vec::new();
    let mut images = Vec::new();

    for (cx, cy, cw, ch) in &components {
        if *cw < minw || *ch < minh {
            continue;
        }
        boxes.push(crate::core::Box::new_unchecked(
            *cx as i32, *cy as i32, *cw as i32, *ch as i32,
        ));
        let region = extract_region(&binary, *cx, *cy, *cw, *ch)?;
        images.push(region);
    }

    // Sort by x coordinate
    let mut indexed: Vec<(crate::core::Box, Pix)> = boxes.into_iter().zip(images).collect();
    indexed.sort_by_key(|(b, _)| b.x);
    let (sorted_boxes, sorted_images): (Vec<_>, Vec<_>) = indexed.into_iter().unzip();

    if sorted_boxes.is_empty() {
        return Err(RecogError::NoContent(
            "no character components found".to_string(),
        ));
    }

    Ok((sorted_boxes, Some(sorted_images)))
}

/// Splits a single connected component using its vertical projection profile.
///
/// Finds minima in the column-sum projection and uses them as split points.
///
/// Corresponds to `pixSplitComponentWithProfile` in C Leptonica.
pub fn split_component_with_profile(
    pix: &Pix,
    delta: i32,
    mindel: i32,
) -> RecogResult<Vec<crate::core::Box>> {
    let binary = ensure_binary(pix)?;
    let w = binary.width();
    let h = binary.height();

    if w < 4 || h < 4 {
        return Ok(vec![crate::core::Box::new_unchecked(
            0, 0, w as i32, h as i32,
        )]);
    }

    // Compute column projection
    let mut col_sums = vec![0u32; w as usize];
    for y in 0..h {
        for x in 0..w {
            if binary.get_pixel_unchecked(x, y) != 0 {
                col_sums[x as usize] += 1;
            }
        }
    }

    // Find split points: look for valleys in the projection
    let delta = delta.max(1) as usize;
    let mindel = mindel.max(1) as u32;
    let mut split_points = Vec::new();

    // Find gaps (runs of columns below a threshold)
    let max_sum = col_sums.iter().copied().max().unwrap_or(0);
    let gap_thresh = (max_sum as f32 * 0.1) as u32;

    let mut in_gap = false;
    let mut gap_start = 0usize;

    for (x, &sum) in col_sums.iter().enumerate() {
        if sum <= gap_thresh {
            if !in_gap {
                in_gap = true;
                gap_start = x;
            }
        } else if in_gap {
            in_gap = false;
            let left_max = if gap_start >= delta {
                col_sums[gap_start.saturating_sub(delta)..gap_start]
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(0)
            } else {
                0
            };
            let right_max = col_sums[x..(x + delta).min(col_sums.len())]
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
            if left_max >= mindel && right_max >= mindel {
                split_points.push(((gap_start + x) / 2) as u32);
            }
        }
    }

    // Build boxes from split points
    let mut boxes = Vec::new();
    let mut start_x = 0u32;
    for &sp in &split_points {
        if sp > start_x {
            boxes.push(crate::core::Box::new_unchecked(
                start_x as i32,
                0,
                (sp - start_x) as i32,
                h as i32,
            ));
        }
        start_x = sp;
    }
    if start_x < w {
        boxes.push(crate::core::Box::new_unchecked(
            start_x as i32,
            0,
            (w - start_x) as i32,
            h as i32,
        ));
    }

    if boxes.is_empty() {
        boxes.push(crate::core::Box::new_unchecked(0, 0, w as i32, h as i32));
    }

    Ok(boxes)
}

/// Gets word bounding boxes and images from text lines.
///
/// Detects words using dilation-based masking, filters by size, sorts into
/// textline order, and returns images, boxes, and textline indices.
///
/// Corresponds to `pixGetWordsInTextlines` in C Leptonica.
#[allow(clippy::type_complexity)]
pub fn get_words_in_textlines(
    pix: &Pix,
    minwidth: u32,
    minheight: u32,
    maxwidth: u32,
    maxheight: u32,
) -> RecogResult<(Vec<crate::core::Box>, Option<Vec<Pix>>, Vec<usize>)> {
    let binary = ensure_binary(pix)?;
    let w = binary.width();
    let h = binary.height();

    // Generate word mask by dilation
    let dilated = morphological_dilate(&binary, 7, 3)?;

    // Extract components from word mask
    let components = find_connected_components(&dilated)?;

    let mut word_boxes = Vec::new();
    let mut word_images = Vec::new();

    for (cx, cy, cw, ch) in &components {
        if *cw < minwidth || *ch < minheight || *cw > maxwidth || *ch > maxheight {
            continue;
        }
        word_boxes.push(crate::core::Box::new_unchecked(
            *cx as i32, *cy as i32, *cw as i32, *ch as i32,
        ));
        let region = extract_region(&binary, *cx, *cy, (*cw).min(w - *cx), (*ch).min(h - *cy))?;
        word_images.push(region);
    }

    if word_boxes.is_empty() {
        return Err(RecogError::NoContent("no words found".to_string()));
    }

    // Sort by y then x (textline order)
    let mut indexed: Vec<(usize, crate::core::Box, Pix)> = word_boxes
        .into_iter()
        .zip(word_images)
        .enumerate()
        .map(|(i, (b, p))| (i, b, p))
        .collect();
    indexed.sort_by(|a, b| {
        let ay = a.1.y;
        let by = b.1.y;
        // Same textline if y-centers are within half avg height
        let ah = a.1.h;
        let bh = b.1.h;
        let a_center = ay + ah / 2;
        let b_center = by + bh / 2;
        let tolerance = (ah + bh) / 4;
        if (a_center - b_center).abs() <= tolerance {
            a.1.x.cmp(&b.1.x)
        } else {
            ay.cmp(&by)
        }
    });

    // Assign textline indices
    let mut nai = Vec::new();
    let mut sorted_boxes = Vec::new();
    let mut sorted_images = Vec::new();
    let mut current_line = 0usize;
    let mut prev_y_center = i32::MIN;

    for (_, b, p) in indexed {
        let y_center = b.y + b.h / 2;
        let tolerance = b.h / 2;
        if prev_y_center != i32::MIN && (y_center - prev_y_center).abs() > tolerance {
            current_line += 1;
        }
        prev_y_center = y_center;
        nai.push(current_line);
        sorted_boxes.push(b);
        sorted_images.push(p);
    }

    Ok((sorted_boxes, Some(sorted_images), nai))
}

/// Gets word bounding boxes from text lines (boxes only, no images).
///
/// Simpler version of [`get_words_in_textlines`] that returns only boxes
/// and textline indices.
///
/// Corresponds to `pixGetWordBoxesInTextlines` in C Leptonica.
pub fn get_word_boxes_in_textlines(
    pix: &Pix,
    minwidth: u32,
    minheight: u32,
    maxwidth: u32,
    maxheight: u32,
) -> RecogResult<(Vec<crate::core::Box>, Vec<usize>)> {
    let (boxes, _, nai) = get_words_in_textlines(pix, minwidth, minheight, maxwidth, maxheight)?;
    Ok((boxes, nai))
}

/// Converts an RGB image to grayscale (using luminance)
#[allow(dead_code)]
fn rgb_to_grayscale_pageseg(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let gray = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut gray_mut = gray.try_into_mut().unwrap();
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let r = (val >> 24) & 0xff;
            let g = (val >> 16) & 0xff;
            let b = (val >> 8) & 0xff;
            let lum = (r * 77 + g * 150 + b * 29) >> 8;
            gray_mut.set_pixel_unchecked(x, y, lum);
        }
    }
    Ok(gray_mut.into())
}

// ============================================================================
// Internal functions
// ============================================================================

/// Ensure image is binary
fn ensure_binary(pix: &Pix) -> RecogResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.deep_clone()),
        PixelDepth::Bit8 => {
            let w = pix.width();
            let h = pix.height();
            let binary = Pix::new(w, h, PixelDepth::Bit1)?;
            let mut binary_mut = binary.try_into_mut().unwrap();

            for y in 0..h {
                for x in 0..w {
                    let val = pix.get_pixel_unchecked(x, y);
                    let bit = if val < 128 { 1 } else { 0 };
                    binary_mut.set_pixel_unchecked(x, y, bit);
                }
            }
            Ok(binary_mut.into())
        }
        _ => Err(RecogError::UnsupportedDepth {
            expected: "1 or 8 bpp",
            actual: pix.depth().bits(),
        }),
    }
}

/// Reduce image by factor of 2 using OR (for binary)
fn reduce_by_2(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let new_w = w / 2;
    let new_h = h / 2;

    if new_w == 0 || new_h == 0 {
        return Err(RecogError::ImageTooSmall {
            min_width: 2,
            min_height: 2,
            actual_width: w,
            actual_height: h,
        });
    }

    let reduced = Pix::new(new_w, new_h, pix.depth())?;
    let mut reduced_mut = reduced.try_into_mut().unwrap();

    for ny in 0..new_h {
        for nx in 0..new_w {
            let mut has_black = false;
            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = nx * 2 + dx;
                    let sy = ny * 2 + dy;
                    if sx < w && sy < h {
                        let val = pix.get_pixel_unchecked(sx, sy);
                        if val != 0 {
                            has_black = true;
                            break;
                        }
                    }
                }
                if has_black {
                    break;
                }
            }
            let out_val = if has_black { 1 } else { 0 };
            reduced_mut.set_pixel_unchecked(nx, ny, out_val);
        }
    }

    Ok(reduced_mut.into())
}

/// Expand image by factor of 2 (replicate)
fn expand_by_2(pix: &Pix, target_w: u32, target_h: u32) -> RecogResult<Pix> {
    let src_w = pix.width();
    let src_h = pix.height();

    let expanded = Pix::new(target_w, target_h, pix.depth())?;
    let mut expanded_mut = expanded.try_into_mut().unwrap();

    for y in 0..target_h {
        for x in 0..target_w {
            let sx = (x / 2).min(src_w - 1);
            let sy = (y / 2).min(src_h - 1);
            let val = pix.get_pixel_unchecked(sx, sy);
            expanded_mut.set_pixel_unchecked(x, y, val);
        }
    }

    Ok(expanded_mut.into())
}

/// Generate halftone mask and extract text pixels
fn generate_halftone_mask(pix: &Pix) -> RecogResult<(Pix, Pix)> {
    let w = pix.width();
    let h = pix.height();

    // Halftone detection: look for regular patterns
    // Use opening at reduced scale to find screened regions

    // Create seed by further reduction and opening
    let reduced_4x = reduce_by_2(&reduce_by_2(pix)?)?;
    let seed = morphological_open(&reduced_4x, 5, 5)?;
    let seed_expanded = expand_by_2(&expand_by_2(&seed, w / 2, h / 2)?, w, h)?;

    // Create mask by closing
    let mask = morphological_close(pix, 4, 4)?;

    // Seed fill to get halftone mask
    let halftone = seed_fill(&seed_expanded, &mask)?;

    // Text pixels = original - halftone
    let text = subtract_images(pix, &halftone)?;

    Ok((halftone, text))
}

/// Generate text line mask
fn generate_textline_mask_internal(
    pix: &Pix,
    close_h: u32,
    close_v: u32,
) -> RecogResult<(Pix, Pix)> {
    // Close up text characters horizontally
    let closed = morphological_close(pix, close_h, close_v)?;

    // Erode slightly to remove noise
    let textline_mask = morphological_erode(&closed, 3, 1)?;

    // Detect vertical whitespace for block segmentation
    let vws = detect_vertical_whitespace(&textline_mask)?;

    Ok((textline_mask, vws))
}

/// Generate text block mask
fn generate_textblock_mask_internal(textline_mask: &Pix, vws: &Pix) -> RecogResult<Pix> {
    // Close vertically to connect text lines into blocks
    let closed = morphological_close(textline_mask, 1, 30)?;

    // Subtract vertical whitespace to separate columns
    let textblock = subtract_images(&closed, vws)?;

    Ok(textblock)
}

/// Simple morphological opening (erode then dilate)
fn morphological_open(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    crate::morph::open_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Simple morphological closing (dilate then erode)
fn morphological_close(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    crate::morph::close_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Morphological erosion with rectangular structuring element
fn morphological_erode(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    crate::morph::erode_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Morphological dilation with rectangular structuring element
#[allow(dead_code)]
fn morphological_dilate(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    crate::morph::dilate_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Seed fill operation using word-level operations
fn seed_fill(seed: &Pix, mask: &Pix) -> RecogResult<Pix> {
    let w = seed.width();
    let h = seed.height();
    let wpl = seed.wpl() as usize;
    let total_words = h as usize * wpl;

    if mask.width() != w || mask.height() != h {
        return Err(RecogError::InvalidParameter(
            "seed and mask dimensions must match".to_string(),
        ));
    }

    if seed.depth() != mask.depth() {
        return Err(RecogError::InvalidParameter(
            "seed and mask depths must match".to_string(),
        ));
    }

    if seed.depth() != PixelDepth::Bit1 {
        return Err(RecogError::InvalidParameter(
            "seed and mask must be 1 bpp for seed fill".to_string(),
        ));
    }

    if seed.wpl() != mask.wpl() {
        return Err(RecogError::InvalidParameter(
            "seed and mask words-per-line must match".to_string(),
        ));
    }

    let mask_data = mask.data();
    let mut current = seed.deep_clone();

    loop {
        let dilated = crate::morph::dilate_brick(&current, 3, 3)?;
        let dilated_data = dilated.data();
        let current_data = current.data();

        let out = Pix::new(w, h, seed.depth())?;
        let mut out_mut = out.try_into_mut().unwrap();
        let out_data = out_mut.data_mut();

        let mut changed = false;
        for i in 0..total_words {
            out_data[i] = dilated_data[i] & mask_data[i];
            if out_data[i] != current_data[i] {
                changed = true;
            }
        }

        // Clear unused padding bits in the last word of each scanline
        let bit_remainder = w % 32;
        if bit_remainder != 0 {
            let last_mask: u32 = u32::MAX << (32 - bit_remainder);
            for y in 0..(h as usize) {
                let idx = y * wpl + (wpl - 1);
                out_data[idx] &= last_mask;
            }
        }

        current = out_mut.into();
        if !changed {
            break;
        }
    }

    Ok(current)
}

/// Subtract second image from first using word-level operations
fn subtract_images(pix1: &Pix, pix2: &Pix) -> RecogResult<Pix> {
    let w = pix1.width();
    let h = pix1.height();
    let wpl = pix1.wpl() as usize;
    let total_words = h as usize * wpl;

    if pix2.width() != w || pix2.height() != h {
        return Err(RecogError::InvalidParameter(
            "image dimensions must match".to_string(),
        ));
    }

    if pix1.depth() != pix2.depth() {
        return Err(RecogError::InvalidParameter(
            "image depths must match".to_string(),
        ));
    }

    if pix1.depth() != PixelDepth::Bit1 {
        return Err(RecogError::InvalidParameter(
            "subtract_images requires 1 bpp images".to_string(),
        ));
    }

    let data1 = pix1.data();
    let data2 = pix2.data();

    let result = Pix::new(w, h, pix1.depth())?;
    let mut result_mut = result.try_into_mut().unwrap();
    let out_data = result_mut.data_mut();

    for i in 0..total_words {
        out_data[i] = data1[i] & !data2[i];
    }

    // Clear unused padding bits in the last word of each scanline
    let bit_remainder = w % 32;
    if bit_remainder != 0 {
        let last_mask: u32 = u32::MAX << (32 - bit_remainder);
        for y in 0..(h as usize) {
            let idx = y * wpl + (wpl - 1);
            out_data[idx] &= last_mask;
        }
    }

    Ok(result_mut.into())
}

/// Page orientation for [`decide_if_table`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOrientation {
    /// `L_PORTRAIT_MODE` — assume the image is upright.
    Portrait,
    /// `L_LANDSCAPE_MODE` — rotate 90° cw before analysing.
    Landscape,
}

/// Helper: clip to `box_` if any, then ensure 1bpp.
fn prepare_1bpp(pix: &Pix, box_: Option<&crate::core::Box>) -> RecogResult<Pix> {
    let clipped = if let Some(b) = box_ {
        let pw = pix.width() as i32;
        let ph = pix.height() as i32;
        if b.x < 0 || b.y < 0 || b.w <= 0 || b.h <= 0 || b.x + b.w > pw || b.y + b.h > ph {
            return Err(RecogError::InvalidParameter(format!(
                "box {:?} is out of bounds for {}x{} image",
                b, pw, ph
            )));
        }
        pix.clip_rectangle(b.x as u32, b.y as u32, b.w as u32, b.h as u32)
            .map_err(RecogError::from)?
    } else {
        pix.clone()
    };
    if clipped.depth() == PixelDepth::Bit1 {
        Ok(clipped)
    } else {
        ensure_binary_with_threshold(&clipped, 128)
    }
}

/// Decide whether `pix` likely contains a table.
///
/// Returns `Ok(0..=4)` where higher values indicate stronger evidence of a
/// table; `>= 2` is the conventional threshold (see C `pixDecideIfTable`
/// notes). The C version uses `-1` as a sentinel for "undetermined" via an
/// out-parameter; the Rust port instead surfaces failures as `Err` so the
/// `Ok` value is always a valid score.
///
/// C Leptonica equivalent: `pixDecideIfTable`.
pub fn decide_if_table(
    pix: &Pix,
    box_: Option<&crate::core::Box>,
    orient: PageOrientation,
) -> RecogResult<i32> {
    use crate::morph::{dilate_brick, sequence::morph_sequence};
    use crate::region::conncomp::{ConnectivityType, count_conn_comp, find_connected_components};
    use crate::region::seedfill::seedfill_binary_restricted;
    use crate::transform::rotate::rotate_90;

    // 1bpp + halftone detection: if a halftone region is present, the page
    // likely contains an image rather than a table — return 0 immediately.
    let pix_for_ht = prepare_1bpp(pix, box_)?;
    let (halftone, _text) = generate_halftone_mask(&pix_for_ht)?;
    if !halftone.is_zero() {
        return Ok(0);
    }

    // Same 1bpp pix, dilated 2x2 to merge near-adjacent text strokes.
    let pix1 = prepare_1bpp(pix, box_)?;
    if pix1.is_zero() {
        return Ok(0);
    }
    let pix2 = dilate_brick(&pix1, 2, 2)?;

    // Deskew and optionally rotate for landscape pages.
    let (pix3, _angle) = crate::recog::skew::deskew_both(&pix2)?;
    let pix1 = match orient {
        PageOrientation::Portrait => pix3,
        PageOrientation::Landscape => rotate_90(&pix3, true)?,
    };

    // Horizontal/vertical foreground line detection.
    let pix_h = morph_sequence(&pix1, "o100.1 + c1.4")?;
    let pix_v = morph_sequence(&pix1, "o1.100 + c4.1")?;
    let nhb = count_conn_comp(&pix_h, ConnectivityType::EightWay)?;
    let nvb = count_conn_comp(&pix_v, ConnectivityType::EightWay)?;

    // Vertical whitespace detection: invert to make whitespace foreground.
    // (xmax, ymax) = (0, 0) is the unlimited fast-path in
    // `seedfill_binary_restricted`; passing u32::MAX would force per-pixel
    // distance checks for no benefit.
    let h_lines = seedfill_binary_restricted(&pix_h, &pix1, ConnectivityType::EightWay, 0, 0)?;
    let v_lines = seedfill_binary_restricted(&pix_v, &pix1, ConnectivityType::EightWay, 0, 0)?;
    let lines = h_lines.or(&v_lines)?;
    let no_lines = pix1.subtract(&lines)?;
    let cleaned = morph_sequence(&no_lines, "c4.1 + o8.1")?;
    let inverted = cleaned.invert();
    let vws_mask = morph_sequence(&inverted, "r1 + o1.100")?;
    // C: pixSelectBySize(pix8, 5, 0, 8, L_SELECT_WIDTH, L_SELECT_IF_GTE) — keep
    // components whose bounding-box width is at least 5px. The Rust
    // pix_select_by_size only offers IfBoth/IfEither, so filter via the
    // connected-component list instead.
    let vws_comps = find_connected_components(&vws_mask, ConnectivityType::EightWay)?;
    let nvw = vws_comps.iter().filter(|c| c.bounds.w >= 5).count() as u32;

    let mut score = 0;
    if nhb > 1 {
        score += 1;
    }
    if nvb > 2 {
        score += 1;
    }
    if nvw > 3 {
        score += 1;
    }
    if nvw > 6 {
        score += 1;
    }
    Ok(score)
}

/// Detect inverted-text regions and re-invert them.
///
/// Returns `(processed_1bpp, optional_mask)` where `processed_1bpp` is the
/// post-photoinvert binary image and the mask flags inverted regions.
///
/// C Leptonica equivalent: `pixAutoPhotoinvert`.
pub fn auto_photoinvert(pix: &Pix, thresh: u32) -> RecogResult<(Pix, Option<Pix>)> {
    use crate::morph::sequence::morph_sequence;
    use crate::region::conncomp::{ConnectivityType, find_connected_components};
    use crate::region::seedfill::fill_holes_to_bounding_rect;

    let thresh = if thresh == 0 { 128 } else { thresh };
    let pix1 = ensure_binary_with_threshold(pix, thresh)?;

    // Identify candidate inverted-text regions via halftone-style mask.
    let (ht, _text) = generate_halftone_mask(&pix1)?;
    let denoised = morph_sequence(&ht, "o15.15 + c25.25")?;
    let mut mask = fill_holes_to_bounding_rect(&denoised, 1, 0.5, 1.0)?;
    if mask.is_zero() {
        return Ok((pix1, None));
    }

    // Validate each component: the underlying region must be ≥ 60% fg in pix1
    // (i.e. dark background with light text). Erase regions that do not meet
    // the criterion from the mask.
    let comps = find_connected_components(&mask, ConnectivityType::EightWay)?;
    let mut mask_mut = mask.to_mut();
    for cc in &comps {
        let b = cc.bounds;
        let region = pix1.clip_rectangle(b.x as u32, b.y as u32, b.w as u32, b.h as u32)?;
        let frac = region.foreground_fraction()?;
        if frac < 0.6 {
            // Erase this component from the mask.
            mask_mut.clear_in_rect(&b)?;
        }
    }
    mask = mask_mut.into();

    if mask.is_zero() {
        return Ok((pix1, None));
    }

    // Photoinvert: combine inverted pix into pix1 over the mask.
    let inverted = pix1.invert();
    let mut combined_mut = pix1.to_mut();
    combined_mut.combine_masked(&inverted, &mask)?;
    let combined: Pix = combined_mut.into();

    Ok((combined, Some(mask)))
}

// Old pixel-by-pixel implementations preserved for testing
#[cfg(test)]
mod old_impl {
    use super::*;

    /// Simple morphological opening (erode then dilate) - old implementation
    pub fn morphological_open_old(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
        let eroded = morphological_erode_old(pix, se_w, se_h)?;
        morphological_dilate_old(&eroded, se_w, se_h)
    }

    /// Simple morphological closing (dilate then erode) - old implementation
    pub fn morphological_close_old(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
        let dilated = morphological_dilate_old(pix, se_w, se_h)?;
        morphological_erode_old(&dilated, se_w, se_h)
    }

    /// Morphological erosion with rectangular structuring element - old implementation
    pub fn morphological_erode_old(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
        let w = pix.width();
        let h = pix.height();
        let hw = se_w / 2;
        let hh = se_h / 2;

        let eroded = Pix::new(w, h, pix.depth())?;
        let mut eroded_mut = eroded.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let mut all_black = true;
                'outer: for dy in 0..se_h {
                    for dx in 0..se_w {
                        let sx = x as i32 + dx as i32 - hw as i32;
                        let sy = y as i32 + dy as i32 - hh as i32;
                        if sx < 0 || sx >= w as i32 || sy < 0 || sy >= h as i32 {
                            all_black = false;
                            break 'outer;
                        }
                        let val = pix.get_pixel_unchecked(sx as u32, sy as u32);
                        if val == 0 {
                            all_black = false;
                            break 'outer;
                        }
                    }
                }
                let out_val = if all_black { 1 } else { 0 };
                eroded_mut.set_pixel_unchecked(x, y, out_val);
            }
        }

        Ok(eroded_mut.into())
    }

    /// Morphological dilation with rectangular structuring element - old implementation
    pub fn morphological_dilate_old(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
        let w = pix.width();
        let h = pix.height();
        let hw = se_w / 2;
        let hh = se_h / 2;

        let dilated = Pix::new(w, h, pix.depth())?;
        let mut dilated_mut = dilated.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let mut any_black = false;
                'outer: for dy in 0..se_h {
                    for dx in 0..se_w {
                        let sx = x as i32 + dx as i32 - hw as i32;
                        let sy = y as i32 + dy as i32 - hh as i32;
                        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                            let val = pix.get_pixel_unchecked(sx as u32, sy as u32);
                            if val != 0 {
                                any_black = true;
                                break 'outer;
                            }
                        }
                    }
                }
                let out_val = if any_black { 1 } else { 0 };
                dilated_mut.set_pixel_unchecked(x, y, out_val);
            }
        }

        Ok(dilated_mut.into())
    }

    /// Seed fill operation - old implementation
    pub fn seed_fill_old(seed: &Pix, mask: &Pix) -> RecogResult<Pix> {
        let w = seed.width();
        let h = seed.height();

        if mask.width() != w || mask.height() != h {
            return Err(RecogError::InvalidParameter(
                "seed and mask dimensions must match".to_string(),
            ));
        }

        // Start with seed
        let mut current = seed.deep_clone();
        let mut changed = true;

        // Iterate until convergence
        while changed {
            changed = false;
            let dilated = morphological_dilate_old(&current, 3, 3)?;

            // Intersect with mask
            let next = Pix::new(w, h, seed.depth())?;
            let mut next_mut = next.try_into_mut().unwrap();

            for y in 0..h {
                for x in 0..w {
                    let d_val = dilated.get_pixel_unchecked(x, y);
                    let m_val = mask.get_pixel_unchecked(x, y);
                    let c_val = current.get_pixel_unchecked(x, y);
                    let new_val = if d_val != 0 && m_val != 0 { 1 } else { 0 };
                    next_mut.set_pixel_unchecked(x, y, new_val);

                    if new_val != c_val {
                        changed = true;
                    }
                }
            }

            current = next_mut.into();
        }

        Ok(current)
    }

    /// Subtract second image from first - old implementation
    pub fn subtract_images_old(pix1: &Pix, pix2: &Pix) -> RecogResult<Pix> {
        let w = pix1.width();
        let h = pix1.height();

        let result = Pix::new(w, h, pix1.depth())?;
        let mut result_mut = result.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let v1 = pix1.get_pixel_unchecked(x, y);
                let sx = x.min(pix2.width() - 1);
                let sy = y.min(pix2.height() - 1);
                let v2 = pix2.get_pixel_unchecked(sx, sy);
                let out = if v1 != 0 && v2 == 0 { 1 } else { 0 };
                result_mut.set_pixel_unchecked(x, y, out);
            }
        }

        Ok(result_mut.into())
    }
}

/// Detect vertical whitespace between text blocks
fn detect_vertical_whitespace(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let vws = Pix::new(w, h, pix.depth())?;
    let mut vws_mut = vws.try_into_mut().unwrap();

    // Find columns with mostly white pixels (vertical gutters)
    let min_gap = 20u32; // Minimum vertical gap to be considered whitespace

    for x in 0..w {
        let mut gap_start: Option<u32> = None;

        for y in 0..h {
            let val = pix.get_pixel_unchecked(x, y);
            if val == 0 {
                // White pixel
                if gap_start.is_none() {
                    gap_start = Some(y);
                }
            } else {
                // Black pixel - check if we had a significant gap
                if let Some(start) = gap_start
                    && y - start >= min_gap
                {
                    // Mark this gap as vertical whitespace
                    for gy in start..y {
                        vws_mut.set_pixel_unchecked(x, gy, 1);
                    }
                }
                gap_start = None;
            }
        }

        // Check final gap
        if let Some(start) = gap_start
            && h - start >= min_gap
        {
            for gy in start..h {
                vws_mut.set_pixel_unchecked(x, gy, 1);
            }
        }
    }

    Ok(vws_mut.into())
}

/// Find connected components and return bounding boxes
fn find_connected_components(pix: &Pix) -> RecogResult<Vec<(u32, u32, u32, u32)>> {
    let w = pix.width();
    let h = pix.height();

    // Simple connected component labeling using flood fill
    let mut labels = vec![0u32; (w * h) as usize];
    let mut components = Vec::new();
    let mut current_label = 0u32;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if labels[idx] == 0 {
                let val = pix.get_pixel_unchecked(x, y);
                if val != 0 {
                    // New component found
                    current_label += 1;
                    let bbox = flood_fill_label(pix, &mut labels, x, y, w, h, current_label);
                    components.push(bbox);
                }
            }
        }
    }

    Ok(components)
}

/// Flood fill to label a connected component
fn flood_fill_label(
    pix: &Pix,
    labels: &mut [u32],
    start_x: u32,
    start_y: u32,
    w: u32,
    h: u32,
    label: u32,
) -> (u32, u32, u32, u32) {
    let mut min_x = start_x;
    let mut max_x = start_x;
    let mut min_y = start_y;
    let mut max_y = start_y;

    let mut stack = vec![(start_x, start_y)];

    while let Some((x, y)) = stack.pop() {
        let idx = (y * w + x) as usize;
        if labels[idx] != 0 {
            continue;
        }

        let val = pix.get_pixel_unchecked(x, y);
        if val == 0 {
            continue;
        }

        labels[idx] = label;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);

        // Add neighbors (4-connectivity)
        if x > 0 {
            stack.push((x - 1, y));
        }
        if x < w - 1 {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y < h - 1 {
            stack.push((x, y + 1));
        }
    }

    (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

/// Extract a rectangular region from an image
fn extract_region(pix: &Pix, x: u32, y: u32, w: u32, h: u32) -> RecogResult<Pix> {
    let src_w = pix.width();
    let src_h = pix.height();

    if x + w > src_w || y + h > src_h {
        return Err(RecogError::InvalidParameter(
            "region extends beyond image bounds".to_string(),
        ));
    }

    let region = Pix::new(w, h, pix.depth())?;
    let mut region_mut = region.try_into_mut().unwrap();

    for dy in 0..h {
        for dx in 0..w {
            let val = pix.get_pixel_unchecked(x + dx, y + dy);
            region_mut.set_pixel_unchecked(dx, dy, val);
        }
    }

    Ok(region_mut.into())
}

/// Compute horizontal variance (spread of black pixels horizontally)
#[allow(dead_code)]
fn compute_horizontal_variance(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();

    let mut col_counts = vec![0u32; w as usize];
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                col_counts[x as usize] += 1;
            }
        }
    }

    compute_variance(&col_counts)
}

/// Compute vertical variance (spread of black pixels vertically)
#[allow(dead_code)]
fn compute_vertical_variance(pix: &Pix) -> f64 {
    let w = pix.width();
    let h = pix.height();

    let mut row_counts = vec![0u32; h as usize];
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            if val != 0 {
                row_counts[y as usize] += 1;
            }
        }
    }

    compute_variance(&row_counts)
}

/// Compute variance of a series
fn compute_variance(data: &[u32]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let sum: u64 = data.iter().map(|&x| x as u64).sum();
    let mean = sum as f64 / data.len() as f64;

    let variance: f64 = data
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / data.len() as f64;

    variance
}

// ============================================================================
// pageseg.c gap-fill (plan 803)
// ============================================================================

/// Find the vertical extent of foreground rows whose pixel count meets
/// `thresh`.
///
/// Returns `(top, bot)` — row indices (inclusive) of the first and last
/// rows with `count_pixels_in_row >= thresh`. If no row qualifies, both
/// fields are `0` (matching C's "no row found" return).
///
/// Requires a 1 bpp image. Subsumes the `&top`/`&bot` output pointers from
/// the C signature.
///
/// C Leptonica equivalent: `pixFindThreshFgExtent`.
pub fn pix_find_thresh_fg_extent(pixs: &Pix, thresh: u32) -> RecogResult<(u32, u32)> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    let counts = pixs.count_by_row(None)?;
    let n = counts.len();
    let mut top = 0u32;
    let mut bot = 0u32;
    for i in 0..n {
        if (counts.get(i).unwrap_or(0.0) as u32) >= thresh {
            top = i as u32;
            break;
        }
    }
    for i in (0..n).rev() {
        if (counts.get(i).unwrap_or(0.0) as u32) >= thresh {
            bot = i as u32;
            break;
        }
    }
    Ok((top, bot))
}

/// Generate the halftone mask of `pixs` and a companion text image.
///
/// Returns `(halftone_mask, text_pix, halftone_found)`. The text image is
/// `pixs` with halftone-flagged pixels removed. `halftone_found` is `true`
/// when the halftone mask has any FG pixel.
///
/// Requires a 1 bpp image.
///
/// C Leptonica equivalent: `pixGenHalftoneMask` (delegates to
/// `pixGenerateHalftoneMask`).
pub fn pix_gen_halftone_mask(pixs: &Pix) -> RecogResult<(Pix, Pix, bool)> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    let (halftone, text) = generate_halftone_mask(pixs)?;
    let found = !halftone.is_zero();
    Ok((halftone, text, found))
}

/// Generate the textline mask of `pixs` plus the vertical-whitespace mask
/// it relies on.
///
/// Returns `(textline_mask, vertical_whitespace_mask, textline_found)`.
/// `textline_found` is `true` when the textline mask has any FG pixel.
///
/// Requires a 1 bpp image of at least `MIN_WIDTH x MIN_HEIGHT`.
///
/// C Leptonica equivalent: `pixGenTextlineMask`.
pub fn pix_gen_textline_mask(pixs: &Pix) -> RecogResult<(Pix, Pix, bool)> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    if pixs.width() < MIN_WIDTH || pixs.height() < MIN_HEIGHT {
        return Err(RecogError::ImageTooSmall {
            min_width: MIN_WIDTH,
            min_height: MIN_HEIGHT,
            actual_width: pixs.width(),
            actual_height: pixs.height(),
        });
    }
    use crate::morph::sequence::morph_comp_sequence;

    // Invert and isolate large bg regions, then subtract them.
    let mut pix1 = pixs.invert();
    let pix2 = morph_comp_sequence(&pix1, "o80.60")?;
    pix1 = pix1.subtract(&pix2)?;

    // Vertical whitespace = open the remaining background.
    let pixvws = morph_comp_sequence(&pix1, "o5.1 + o1.200")?;

    // Textline mask: close characters, subtract vws, open small noise.
    let pix3 = crate::morph::sequence::morph_sequence(pixs, "c30.1")?;
    let pix4 = pix3.subtract(&pixvws)?;
    let pixd = crate::morph::binary::open_brick(&pix4, 3, 3)?;

    let found = !pixd.is_zero();
    Ok((pixd, pixvws, found))
}

/// Generate a textblock mask from `pixs` and the vertical-whitespace mask
/// returned by [`pix_gen_textline_mask`].
///
/// Returns `None` when the morphology step leaves no FG pixels (matching
/// C's `NULL` return + `L_INFO` log).
///
/// C Leptonica equivalent: `pixGenTextblockMask`.
pub fn pix_gen_textblock_mask(pixs: &Pix, pixvws: &Pix) -> RecogResult<Option<Pix>> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    if pixs.width() < MIN_WIDTH || pixs.height() < MIN_HEIGHT {
        return Err(RecogError::ImageTooSmall {
            min_width: MIN_WIDTH,
            min_height: MIN_HEIGHT,
            actual_width: pixs.width(),
            actual_height: pixs.height(),
        });
    }
    if pixvws.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixvws.depth().bits(),
        });
    }
    if pixvws.width() != pixs.width() || pixvws.height() != pixs.height() {
        return Err(RecogError::InvalidParameter(format!(
            "pixvws dimensions ({}x{}) do not match pixs ({}x{})",
            pixvws.width(),
            pixvws.height(),
            pixs.width(),
            pixs.height()
        )));
    }
    use crate::morph::binary::close_safe_brick;
    use crate::morph::morphapp::morph_sequence_by_component;
    use crate::morph::sequence::morph_sequence;

    let pix1 = morph_sequence(pixs, "c1.10 + o4.1")?;
    if pix1.is_zero() {
        return Ok(None);
    }

    let mut pix2 = morph_sequence_by_component(&pix1, "c30.30 + d3.3", 0, 0, 8)?;
    pix2 = close_safe_brick(&pix2, 10, 1)?;
    let pix3 = pix2.subtract(pixvws)?;

    let pixd = crate::region::pix_select_by_size(
        &pix3,
        25,
        5,
        crate::region::ConnectivityType::EightWay,
        crate::region::SizeSelectType::IfBoth,
        crate::region::SizeSelectRelation::Gte,
    )?;
    Ok(Some(pixd))
}

// ============================================================================
// plan 804: pageseg.c の重い高レベル関数 (5)
// ============================================================================

/// Clean an image for printing/OCR: optional rotation, deskew, background
/// whitening, binarisation, and optional small-noise removal.
///
/// # Parameters
///
/// - `contrast`: 1..=10 (1 = lightest, 10 = darkest).
/// - `rotation`: 0..=3 (cw 90° quads applied before deskew).
/// - `scale`: 1 (threshold only) or 2 (2× upscale before threshold).
/// - `opensize`: 0 or 1 = skip; 2 or 3 = open with square SE of this size.
///
/// C Leptonica equivalent: `pixCleanImage`.
pub fn pix_clean_image(
    pixs: &Pix,
    contrast: u32,
    rotation: u32,
    scale: u32,
    opensize: u32,
) -> RecogResult<Pix> {
    use crate::filter::background_norm_to_1_min_max;
    use crate::morph::sequence::morph_sequence;
    use crate::recog::skew::deskew;
    use crate::transform::scale::{ScaleMethod, scale as scale_pix};
    use crate::transform::{expand_binary_replicate, rotate_orth};

    if rotation > 3 {
        return Err(RecogError::InvalidParameter(format!(
            "rotation must be in 0..=3, got {rotation}"
        )));
    }
    if !(1..=10).contains(&contrast) {
        return Err(RecogError::InvalidParameter(format!(
            "contrast must be in 1..=10, got {contrast}"
        )));
    }
    if scale != 1 && scale != 2 {
        return Err(RecogError::InvalidParameter(format!(
            "scale must be 1 or 2, got {scale}"
        )));
    }
    if opensize > 3 {
        return Err(RecogError::InvalidParameter(format!(
            "opensize must be <= 3, got {opensize}"
        )));
    }

    let intermediate = if pixs.depth() == PixelDepth::Bit1 {
        let rotated = if rotation > 0 {
            rotate_orth(pixs, rotation)?
        } else {
            pixs.clone()
        };
        let deskewed = deskew(&rotated)?;
        if scale == 2 {
            expand_binary_replicate(&deskewed, 2, 2)?
        } else {
            deskewed
        }
    } else {
        let gray = pixs.convert_to_8()?;
        let rotated = if rotation > 0 {
            rotate_orth(&gray, rotation)?
        } else {
            gray
        };
        let deskewed = deskew(&rotated)?;
        // C's pixBackgroundNormTo1MinMax handles `scale == 2` internally by
        // bilinear-upscaling the grayscale before threshold. The Rust public
        // helper only does scale=1, so we pre-scale here to match dims.
        let to_norm = if scale == 2 {
            scale_pix(&deskewed, 2.0, 2.0, ScaleMethod::Linear)?
        } else {
            deskewed
        };
        background_norm_to_1_min_max(&to_norm, contrast)?
    };

    if opensize == 2 || opensize == 3 {
        let seq = format!("o{opensize}.{opensize}");
        Ok(morph_sequence(&intermediate, &seq)?)
    } else {
        Ok(intermediate)
    }
}

/// Estimate the number of text columns on a page from the column-FG profile.
///
/// Returns `0` when no significant content is detected. The C version
/// returns the count via an out-parameter and uses `-1` as the unset
/// sentinel; the Rust port surfaces failures as `Err` so the count is
/// always meaningful.
///
/// # Parameters
///
/// - `deltafract`: 0.15..=0.75 (extrema-detection delta as fraction of range)
/// - `peakfract`: 0.25..=0.9 (peak threshold as fraction of range)
/// - `clipfract`: 0.0..0.5 (border fraction clipped before analysis)
///
/// C Leptonica equivalent: `pixCountTextColumns`.
pub fn pix_count_text_columns(
    pixs: &Pix,
    deltafract: f32,
    peakfract: f32,
    clipfract: f32,
) -> RecogResult<u32> {
    use crate::morph::binary::close_safe_brick;
    use crate::recog::skew::deskew;
    use crate::transform::reduce_rank_binary_cascade;
    use crate::transform::scale::{ScaleMethod, scale as scale_pix};

    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    if !(0.0..0.5).contains(&clipfract) {
        return Err(RecogError::InvalidParameter(format!(
            "clipfract must be in [0.0, 0.5), got {clipfract}"
        )));
    }
    // deltafract / peakfract: C only warns; we accept any value (consistent
    // with C's permissive behaviour outside the recommended range).

    // Scale to between 37.5 and 75 ppi.
    let res = if pixs.xres() <= 0 { 300 } else { pixs.xres() };
    let pix1 = if res < 37 {
        let factor = 37.5 / res as f32;
        scale_pix(pixs, factor, factor, ScaleMethod::Sampling)?
    } else {
        let redfact = res as f32 / 37.5;
        if redfact < 2.0 {
            pixs.clone()
        } else if redfact < 4.0 {
            reduce_rank_binary_cascade(pixs, &[1])?
        } else if redfact < 8.0 {
            reduce_rank_binary_cascade(pixs, &[1, 2])?
        } else if redfact < 16.0 {
            reduce_rank_binary_cascade(pixs, &[1, 2, 2])?
        } else {
            reduce_rank_binary_cascade(pixs, &[1, 2, 2, 2])?
        }
    };

    // Crop inner (1 - 2*clipfract) of image.
    let w1 = pix1.width() as f32;
    let h1 = pix1.height() as f32;
    let cx = (clipfract * w1) as i32;
    let cy = (clipfract * h1) as i32;
    let cw = ((1.0 - 2.0 * clipfract) * w1) as i32;
    let ch = ((1.0 - 2.0 * clipfract) * h1) as i32;
    if cw <= 0 || ch <= 0 {
        return Ok(0);
    }
    let pix2 = pix1.clip_rectangle(cx as u32, cy as u32, cw as u32, ch as u32)?;

    // Deskew (may fail on empty input → 0 columns).
    let pix3 = match deskew(&pix2) {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };
    let w = pix3.width();
    let h = pix3.height();

    // Close to merge text into bands, then invert so that text-band columns
    // become low counts and gutters become high counts.
    let pix4 = close_safe_brick(&pix3, 5, 21)?;
    let pix4_inv = pix4.invert();
    let na1 = pix4_inv.count_by_column(None)?;

    let max_val = na1.max_value().unwrap_or(0.0);
    let min_val = na1.min_value().unwrap_or(0.0);
    let range = max_val - min_val;
    let fraction_active = range / h as f32;
    if fraction_active < 0.05 {
        return Ok(0);
    }

    // numaFindExtrema with `delta = deltafract * (max - min)`.
    let (na_loc, na_val) = na1.find_extrema_with_values(deltafract * range)?;
    // Normalised location (0..1) and normalised value (0..1).
    let na_loc_norm = na_loc.transform(0.0, 1.0 / w as f32);
    let na_val_norm = na_val.transform(-min_val, 1.0 / range);

    let mut peaks = 0u32;
    for i in 0..na_loc_norm.len() {
        let loc = na_loc_norm.get(i).unwrap_or(0.0);
        let val = na_val_norm.get(i).unwrap_or(0.0);
        if loc > 0.3 && loc < 0.7 && val >= peakfract {
            peaks += 1;
        }
    }

    Ok(peaks + 1)
}

/// Decide whether `pixs` is more likely text or photo.
///
/// Returns `Some(true)` for text, `Some(false)` for photo, and `None` when
/// the input is empty or the decision cannot be made (mirrors C's `-1`
/// sentinel via the out-parameter `*pistext`).
///
/// C Leptonica equivalent: `pixDecideIfText`.
pub fn pix_decide_if_text(
    pixs: &Pix,
    box_: Option<&crate::core::Box>,
) -> RecogResult<Option<bool>> {
    use crate::morph::sequence::morph_comp_sequence;
    use crate::morph::{Sel, SelElement, hit_miss_transform};
    use crate::region::seedfill::seedfill_binary_restricted;
    use crate::region::{ConnectivityType, conncomp_pixa};

    // Crop and convert to 1 bpp at ~300 ppi.
    let pix1 = prepare_1bpp(pixs, box_)?;
    if pix1.is_zero() {
        return Ok(None);
    }
    let w = pix1.width() as i32;

    // Build a vertical-line hit-miss SEL (81 px tall, 11 wide; column 5 is
    // the hit column; misses at columns 0 and 10 at three vertical
    // positions). Removes thin vertical lines as found in tables.
    let sel_template = Pix::new(11, 81, PixelDepth::Bit1)?;
    let mut t = sel_template.try_into_mut().unwrap();
    for y in 0..81 {
        t.set_pixel(5, y, 1)?;
    }
    let template: Pix = t.into();
    // Rust API: from_pix(pix, cx, cy) — origin at (5, 40) in (w=11, h=81).
    let mut sel1 = Sel::from_pix(&template, 5, 40)?;
    // set_element(x, y, elem) — pair of misses on either side of the hit
    // column at rows 20, 40, 60.
    for &y in &[20u32, 40, 60] {
        sel1.set_element(0, y, SelElement::Miss);
        sel1.set_element(10, y, SelElement::Miss);
    }

    let pix3 = hit_miss_transform(&pix1, &sel1)?;
    let pix4 = seedfill_binary_restricted(&pix3, &pix1, ConnectivityType::EightWay, 5, 1000)?;
    let pix5 = pix1.xor(&pix4)?;

    // Merge cleaned residual into long horizontal components.
    let pix6 = morph_comp_sequence(&pix5, "c30.1 + o15.1 + c60.1 + o2.2")?;

    // Region height for minlines: full height with explicit box, otherwise
    // textline-band extent on pix6.
    let h = if box_.is_some() {
        pix6.height() as i32
    } else {
        let (_top, bot) = pix_find_thresh_fg_extent(&pix6, 400)?;
        bot as i32
    };

    let (boxa1, _) = conncomp_pixa(&pix6, ConnectivityType::EightWay)?;
    if boxa1.is_empty() {
        return Ok(Some(false));
    }

    // Width of the 2nd-widest component (mirror C: boxaSort by width, idx 1).
    let mut widths: Vec<i32> = boxa1.iter().map(|b| b.w).collect();
    widths.sort_unstable_by(|a, b| b.cmp(a));
    let maxw = if widths.len() >= 2 {
        widths[1]
    } else {
        widths[0]
    };

    let min_w = ((0.4 * maxw as f32) as i32).max(1);
    let n_long = boxa1.iter().filter(|b| b.w >= min_w).count() as i32;
    let n_thin = boxa1.iter().filter(|b| b.w >= min_w && b.h <= 60).count() as i32;
    let big_comp = boxa1.iter().any(|b| b.w > 400 && b.h > 175);

    let ratio1 = maxw as f32 / w as f32;
    let ratio2 = if n_long > 0 {
        n_thin as f32 / n_long as f32
    } else {
        0.0
    };
    let minlines = 2i32.max(h / 125);
    let is_text = !(big_comp || ratio1 < 0.6 || ratio2 < 0.8 || n_thin < minlines);
    Ok(Some(is_text))
}

/// Extract raw text lines from `pixs` as a `Pixa` of sub-images.
///
/// Returns an empty `Pixa` when `pixs` has no foreground pixels (instead of
/// C's `NULL` + log).
///
/// # Parameters
///
/// - `maxw`, `maxh`: maximum component width/height kept before clustering.
///   Pass `0` for the default `0.5 * resolution`.
/// - `adjw`, `adjh`: amounts subtracted from each band's left/right and
///   top/bottom respectively before clipping. `0` keeps the bounding box.
///
/// C Leptonica equivalent: `pixExtractRawTextlines`.
pub fn pix_extract_raw_textlines(
    pixs: &Pix,
    maxw: i32,
    maxh: i32,
    adjw: i32,
    adjh: i32,
) -> RecogResult<crate::core::Pixa> {
    use crate::color::threshold_to_binary;
    use crate::filter::clean_background_to_white;
    use crate::morph::sequence::morph_comp_sequence;
    use crate::region::{ConnectivityType, conncomp_pixa};

    let res = if pixs.xres() <= 0 { 300 } else { pixs.xres() };
    let maxw = if maxw != 0 {
        maxw
    } else {
        (0.5 * res as f32) as i32
    };
    let maxh = if maxh != 0 {
        maxh
    } else {
        (0.5 * res as f32) as i32
    };
    if maxw <= 0 || maxh <= 0 {
        return Err(RecogError::InvalidParameter(format!(
            "maxw and maxh must resolve to positive values (got {maxw}, {maxh})"
        )));
    }

    let pix1 = if pixs.depth() != PixelDepth::Bit1 {
        let gray = pixs.convert_to_8()?;
        let cleaned = clean_background_to_white(&gray, None, None)?;
        threshold_to_binary(&cleaned, 150)?
    } else {
        pixs.clone()
    };

    if pix1.is_zero() {
        return Ok(crate::core::Pixa::new());
    }

    // Drop very tall or very wide components.
    let pix2 = crate::region::pix_select_by_size(
        &pix1,
        maxw,
        maxh,
        crate::region::ConnectivityType::EightWay,
        crate::region::SizeSelectType::IfBoth,
        crate::region::SizeSelectRelation::Lte,
    )?;
    if pix2.is_zero() {
        return Ok(crate::core::Pixa::new());
    }

    // Close horizontally to solidify text lines.
    let csize = 120i32.min((60 * res / 300).max(1));
    let seq = format!("c{csize}.1");
    let pix3 = morph_comp_sequence(&pix2, &seq)?;

    // Connected components are dilated text lines; group them via boxaSort2d
    // and take the extent of each cluster.
    let (boxa1, _) = conncomp_pixa(&pix3, ConnectivityType::FourWay)?;
    if boxa1.is_empty() {
        return Ok(crate::core::Pixa::new());
    }

    let baa = boxa1.sort_2d(-1, -1, 5)?;
    let (_w, _h, _bbox, boxa2) = baa.get_extent()?;
    let boxa3 = boxa2.adjust_all_sides(-adjw, adjw, -adjh, adjh);

    Ok(pix2.clip_rectangles(&boxa3)?)
}

/// Locate the largest (or two comparable) connected component(s) after a
/// strong vertical closing. Returns a bounding box in the input resolution
/// of `pixs` (which is itself a 2x reduction of the original).
///
/// C Leptonica equivalent (static): `pixMaxCompAfterVClosing`.
fn pix_max_comp_after_v_closing(pixs: &Pix) -> RecogResult<crate::core::Box> {
    use crate::morph::sequence::morph_sequence;
    use crate::region::{ConnectivityType, find_connected_components};

    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }

    let pix1 = morph_sequence(pixs, "r11 + c3.80 + o3.80 + x4")?;
    if pix1.is_zero() {
        return Err(RecogError::NoContent(
            "pixMaxCompAfterVClosing: closed pix is empty".into(),
        ));
    }

    let mut comps = find_connected_components(&pix1, ConnectivityType::EightWay)?;
    if comps.is_empty() {
        return Err(RecogError::NoContent(
            "pixMaxCompAfterVClosing: no components".into(),
        ));
    }
    comps.sort_by_key(|c| -((c.bounds.w as i64) * (c.bounds.h as i64)));

    if comps.len() == 1 {
        return Ok(comps[0].bounds);
    }
    let b1 = comps[0].bounds;
    let b2 = comps[1].bounds;
    let a1 = (b1.w as f32) * (b1.h as f32);
    let a2 = (b2.w as f32) * (b2.h as f32);
    Ok(if a2 / a1 > 0.7 { b1.union(&b2) } else { b1 })
}

/// Extract the page region inside a solid black border. Returns a bounding
/// box in the input resolution of `pixs` (a 2x reduction of the original).
///
/// C Leptonica equivalent (static): `pixFindPageInsideBlackBorder`.
fn pix_find_page_inside_black_border(pixs: &Pix) -> RecogResult<crate::core::Box> {
    use crate::core::Box as LeptBox;
    use crate::morph::sequence::morph_sequence;
    use crate::region::{ConnectivityType, find_connected_components};

    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }

    let pix1 = morph_sequence(pixs, "r22 + c5.5 + o7.7")?;
    if pix1.is_zero() {
        return Err(RecogError::NoContent(
            "pixFindPageInsideBlackBorder: closed pix is empty".into(),
        ));
    }
    let pix1_inv = pix1.invert();
    let pix2 = morph_sequence(&pix1_inv, "c11.11 + o11.11")?;
    let mut comps = find_connected_components(&pix2, ConnectivityType::EightWay)?;
    if comps.is_empty() {
        return Err(RecogError::NoContent(
            "pixFindPageInsideBlackBorder: no components".into(),
        ));
    }
    comps.sort_by_key(|c| -((c.bounds.w as i64) * (c.bounds.h as i64)));

    let box1 = comps[0]
        .bounds
        .adjust_sides(5, -5, 5, -5)
        .ok_or_else(|| RecogError::NoContent("largest component too small after inset".into()))?;
    let box2 = box1.scale(4.0);

    let cx = box2.x.max(0) as u32;
    let cy = box2.y.max(0) as u32;
    let cw = box2.w.min(pixs.width() as i32 - cx as i32).max(0) as u32;
    let ch = box2.h.min(pixs.height() as i32 - cy as i32).max(0) as u32;
    if cw == 0 || ch == 0 {
        return Err(RecogError::NoContent(
            "pixFindPageInsideBlackBorder: box outside image".into(),
        ));
    }
    let pix3 = pixs.clip_rectangle(cx, cy, cw, ch)?;
    let box3 = match pix3.clip_to_foreground()? {
        Some((_, b)) => b,
        None => LeptBox {
            x: 0,
            y: 0,
            w: cw as i32,
            h: ch as i32,
        },
    };
    Ok(box3.translate(box2.x, box2.y))
}

/// Rescale `pixs` to fit isomorphically (with optional horizontal stretch)
/// inside a `(w x h)` page with cleared borders.
///
/// C Leptonica equivalent (static): `pixRescaleForCropping`.
fn pix_rescale_for_cropping(
    pixs: &Pix,
    w: i32,
    h: i32,
    lr_border: i32,
    tb_border: i32,
    maxwiden: f32,
) -> RecogResult<Pix> {
    use crate::core::{InitColor, RopOp};
    use crate::transform::scale::{ScaleMethod, scale as scale_pix};

    if pixs.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1-bit",
            actual: pixs.depth().bits(),
        });
    }
    let lr_border = lr_border.max(0);
    let tb_border = tb_border.max(0);
    let maxwiden = maxwiden.max(1.0);

    let wi = pixs.width() as i32;
    let hi = pixs.height() as i32;
    let wmax = w - 2 * lr_border;
    let hmax = h - 2 * tb_border;
    if wmax <= 0 || hmax <= 0 {
        return Err(RecogError::InvalidParameter(
            "border parameters leave no usable area".into(),
        ));
    }
    let ratio = (wmax * hi) as f32 / (hmax * wi) as f32;

    let (pix1, wf, hf, xf) = if ratio >= 1.0 {
        let scaleh = hmax as f32 / hi as f32;
        let wn = (scaleh * wi as f32) as i32;
        let scalewid = maxwiden.min(wmax as f32 / wn.max(1) as f32);
        let scalew = scaleh * scalewid;
        let wf = (scalew * wi as f32) as i32;
        let hf = hmax;
        let pix1 = scale_pix(pixs, scalew, scaleh, ScaleMethod::Linear)?;
        let xf = (w - wf) / 2;
        (pix1, wf, hf, xf)
    } else {
        let scalew = wmax as f32 / wi as f32;
        let pix1 = scale_pix(pixs, scalew, scalew, ScaleMethod::Linear)?;
        let wf = wmax;
        let hf = (scalew * hi as f32) as i32;
        let xf = lr_border;
        (pix1, wf, hf, xf)
    };

    let pixd = Pix::new(w as u32, h as u32, PixelDepth::Bit1)?;
    let mut pixd_mut = pixd.try_into_mut().unwrap();
    pixd_mut.set_or_clear_border(0, 0, 0, 0, InitColor::White);
    pixd_mut.rop_region_inplace(
        xf,
        tb_border,
        wf.max(0) as u32,
        hf.max(0) as u32,
        RopOp::Src,
        &pix1,
        0,
        0,
    )?;
    Ok(pixd_mut.into())
}

/// Crop the foreground of a page and rescale it for printing.
///
/// Returns `(cropped_image, crop_box_at_input_resolution)`.
///
/// # Parameters
///
/// - `edgeclean`: `-2` extracts the page from a solid black border; `-1`
///   removes left/right noise via a strong vertical closing; `0` keeps all
///   foreground; `1..=15` applies an open/close of that size for noise
///   removal.
/// - `printwiden`: `0` skips; `1` widens for 8.5x11 paper, `2` for A4.
///
/// C Leptonica equivalent: `pixCropImage`.
#[allow(clippy::too_many_arguments)]
pub fn pix_crop_image(
    pixs: &Pix,
    lr_clear: i32,
    tb_clear: i32,
    edgeclean: i32,
    lr_border: i32,
    tb_border: i32,
    maxwiden: f32,
    printwiden: u32,
) -> RecogResult<(Pix, crate::core::Box)> {
    use crate::core::InitColor;
    use crate::filter::background_norm_to_1_min_max;
    use crate::morph::sequence::morph_sequence;
    use crate::transform::reduce_rank_binary_2;
    use crate::transform::scale::{ScaleMethod, scale as scale_pix};

    let mut edgeclean = edgeclean;
    if edgeclean > 15 {
        edgeclean = 15;
    }
    if edgeclean < -1 {
        edgeclean = -2;
    }

    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    if w < MIN_WIDTH as i32 || h < MIN_HEIGHT as i32 {
        return Err(RecogError::ImageTooSmall {
            min_width: MIN_WIDTH,
            min_height: MIN_HEIGHT,
            actual_width: pixs.width(),
            actual_height: pixs.height(),
        });
    }
    let lr_clear = lr_clear.max(0);
    let tb_clear = tb_clear.max(0);
    let lr_border = lr_border.max(0);
    let tb_border = tb_border.max(0);
    if lr_clear > w / 6 || tb_clear > h / 6 {
        return Err(RecogError::InvalidParameter(format!(
            "lr_clear/tb_clear too large; must be <= {}/{}",
            w / 6,
            h / 6
        )));
    }
    let printwiden = if printwiden > 2 { 0 } else { printwiden };

    let pix1 = background_norm_to_1_min_max(pixs, 1)?;
    let pix2 = reduce_rank_binary_2(&pix1, 2)?;

    let mut pix2_mut = pix2.try_into_mut().unwrap();
    pix2_mut.set_or_clear_border(
        (lr_clear / 2) as u32,
        (lr_clear / 2) as u32,
        (tb_clear / 2) as u32,
        (tb_clear / 2) as u32,
        InitColor::White,
    );
    let pix2: Pix = pix2_mut.into();

    let box1 = if edgeclean == 0 {
        pix2.clip_to_foreground()?
            .ok_or_else(|| RecogError::NoContent("no foreground in pix2".into()))?
            .1
    } else if edgeclean > 0 {
        let val = edgeclean + 1;
        let seq = format!("c{val}.{val} + o{val}.{val}");
        let pix3 = morph_sequence(&pix2, &seq)?;
        pix3.clip_to_foreground()?
            .ok_or_else(|| RecogError::NoContent("no foreground after edge clean".into()))?
            .1
    } else if edgeclean == -1 {
        pix_max_comp_after_v_closing(&pix2)?
    } else {
        pix_find_page_inside_black_border(&pix2)?
    };

    let box2 = box1.scale(2.0);

    let cx = box2.x.max(0) as u32;
    let cy = box2.y.max(0) as u32;
    let cw = box2.w.min(pix1.width() as i32 - cx as i32).max(1) as u32;
    let ch = box2.h.min(pix1.height() as i32 - cy as i32).max(1) as u32;
    let pix_fg = pix1.clip_rectangle(cx, cy, cw, ch)?;

    // Slightly thicken long horizontal lines.
    let pix_thick = morph_sequence(&pix_fg, "o80.1 + d1.2")?;
    let mut pix_fg_mut = pix_fg.to_mut();
    pix_fg_mut.or_inplace(&pix_thick)?;
    let pix_fg: Pix = pix_fg_mut.into();

    let pix_rescaled = pix_rescale_for_cropping(&pix_fg, w, h, lr_border, tb_border, maxwiden)?;

    let r1 = h as f32 / w as f32;
    let r2 = match printwiden {
        1 => r1 / 1.294,
        2 => r1 / 1.414,
        _ => 0.0,
    };
    let pix_final = if r2 > 1.03 {
        let r2 = r2.min(1.20);
        scale_pix(&pix_rescaled, r2, 1.0, ScaleMethod::Sampling)?
    } else {
        pix_rescaled
    };

    Ok((pix_final, box2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document(w: u32, h: u32) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create text-like horizontal lines that fit in the image
        let line_height = 5;
        let line_spacing = h / 6; // Space for 5 lines

        for line in 1..=5 {
            let y_base = line * line_spacing;
            for dy in 0..line_height {
                let y = y_base + dy;
                if y < h {
                    for x in (w / 10)..(w * 9 / 10) {
                        pix_mut.set_pixel_unchecked(x, y, 1);
                    }
                }
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_pageseg_options_default() {
        let opts = PageSegOptions::default();
        assert_eq!(opts.min_width, 100);
        assert_eq!(opts.min_height, 100);
        assert!(opts.detect_halftone);
    }

    #[test]
    fn test_pageseg_options_validation() {
        let opts = PageSegOptions::default();
        assert!(opts.validate().is_ok());

        let invalid = PageSegOptions::default().with_min_width(5);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_reduce_by_2() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let reduced = reduce_by_2(&pix).unwrap();
        assert_eq!(reduced.width(), 50);
        assert_eq!(reduced.height(), 50);
    }

    #[test]
    fn test_expand_by_2() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let expanded = expand_by_2(&pix, 100, 100).unwrap();
        assert_eq!(expanded.width(), 100);
        assert_eq!(expanded.height(), 100);
    }

    #[test]
    fn test_morphological_erode() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a 10x10 filled square
        for y in 20..30 {
            for x in 20..30 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix: Pix = pix_mut.into();

        let eroded = morphological_erode(&pix, 3, 3).unwrap();
        // Center should still be black
        assert_eq!(eroded.get_pixel_unchecked(25, 25), 1);
    }

    #[test]
    fn test_morphological_dilate() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Single pixel
        pix_mut.set_pixel_unchecked(25, 25, 1);
        let pix: Pix = pix_mut.into();

        let dilated = morphological_dilate(&pix, 3, 3).unwrap();
        // Neighbors should now be black
        assert_eq!(dilated.get_pixel_unchecked(24, 25), 1);
        assert_eq!(dilated.get_pixel_unchecked(26, 25), 1);
    }

    #[test]
    fn test_segment_regions() {
        let pix = create_test_document(400, 300);
        let opts = PageSegOptions::default().with_detect_halftone(false);

        let result = segment_regions(&pix, &opts).unwrap();
        assert!(result.halftone_mask.is_none());
        assert_eq!(result.textline_mask.width(), 400);
        assert_eq!(result.textblock_mask.height(), 300);
    }

    #[test]
    fn test_is_text_region() {
        let pix = create_test_document(200, 100);
        let result = is_text_region(&pix).unwrap();
        assert!(result);
    }

    #[test]
    fn test_find_connected_components() {
        let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create two separate blobs
        for y in 5..15 {
            for x in 5..15 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        for y in 30..40 {
            for x in 30..40 {
                pix_mut.set_pixel_unchecked(x, y, 1);
            }
        }

        let pix: Pix = pix_mut.into();
        let components = find_connected_components(&pix).unwrap();

        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_extract_region() {
        let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(50, 50, 1);
        let pix: Pix = pix_mut.into();

        let region = extract_region(&pix, 40, 40, 20, 20).unwrap();
        assert_eq!(region.width(), 20);
        assert_eq!(region.height(), 20);
        // Pixel at (50,50) is now at (10,10) in the region
        assert_eq!(region.get_pixel_unchecked(10, 10), 1);
    }

    // Equivalence tests for morphology delegation
    use old_impl::*;

    fn assert_pix_equal(pix1: &Pix, pix2: &Pix) {
        assert_eq!(pix1.width(), pix2.width());
        assert_eq!(pix1.height(), pix2.height());
        assert_eq!(pix1.depth(), pix2.depth());

        for y in 0..pix1.height() {
            for x in 0..pix1.width() {
                let v1 = pix1.get_pixel_unchecked(x, y);
                let v2 = pix2.get_pixel_unchecked(x, y);
                if v1 != v2 {
                    panic!("Pixels differ at ({}, {}): old={}, new={}", x, y, v1, v2);
                }
            }
        }
    }

    #[test]
    fn test_erode_equivalence() {
        let pix = create_test_document(200, 100);

        let old_result = morphological_erode_old(&pix, 3, 3).unwrap();
        let new_result = morphological_erode(&pix, 3, 3).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }

    #[test]
    fn test_dilate_equivalence() {
        let pix = create_test_document(200, 100);

        let old_result = morphological_dilate_old(&pix, 3, 3).unwrap();
        let new_result = morphological_dilate(&pix, 3, 3).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }

    #[test]
    fn test_open_equivalence() {
        let pix = create_test_document(200, 100);

        let old_result = morphological_open_old(&pix, 5, 5).unwrap();
        let new_result = morphological_open(&pix, 5, 5).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }

    #[test]
    fn test_close_equivalence() {
        let pix = create_test_document(200, 100);

        let old_result = morphological_close_old(&pix, 4, 4).unwrap();
        let new_result = morphological_close(&pix, 4, 4).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }

    #[test]
    fn test_seed_fill_equivalence() {
        let seed = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let mut seed_mut = seed.try_into_mut().unwrap();
        for y in 45..55 {
            for x in 45..55 {
                seed_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let seed: Pix = seed_mut.into();

        let mask = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
        let mut mask_mut = mask.try_into_mut().unwrap();
        for y in 40..60 {
            for x in 40..60 {
                mask_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let mask: Pix = mask_mut.into();

        let old_result = seed_fill_old(&seed, &mask).unwrap();
        let new_result = seed_fill(&seed, &mask).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }

    #[test]
    fn test_subtract_equivalence() {
        let pix1 = create_test_document(200, 100);
        let pix2 = Pix::new(200, 100, PixelDepth::Bit1).unwrap();
        let mut pix2_mut = pix2.try_into_mut().unwrap();
        for y in 20..40 {
            for x in 40..120 {
                pix2_mut.set_pixel_unchecked(x, y, 1);
            }
        }
        let pix2: Pix = pix2_mut.into();

        let old_result = subtract_images_old(&pix1, &pix2).unwrap();
        let new_result = subtract_images(&pix1, &pix2).unwrap();

        assert_pix_equal(&old_result, &new_result);
    }
}
