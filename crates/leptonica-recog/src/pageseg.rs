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

use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};

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
/// use leptonica_recog::pageseg::{segment_regions, PageSegOptions};
/// use leptonica_core::{Pix, PixelDepth};
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
    leptonica_morph::open_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Simple morphological closing (dilate then erode)
fn morphological_close(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    leptonica_morph::close_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Morphological erosion with rectangular structuring element
fn morphological_erode(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    leptonica_morph::erode_brick(pix, se_w, se_h).map_err(Into::into)
}

/// Morphological dilation with rectangular structuring element
fn morphological_dilate(pix: &Pix, se_w: u32, se_h: u32) -> RecogResult<Pix> {
    leptonica_morph::dilate_brick(pix, se_w, se_h).map_err(Into::into)
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

    if seed.wpl() != mask.wpl() {
        return Err(RecogError::InvalidParameter(
            "seed and mask words-per-line must match".to_string(),
        ));
    }

    let mask_data = mask.data();
    let mut current = seed.deep_clone();

    loop {
        let dilated = leptonica_morph::dilate_brick(&current, 3, 3)?;
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

    let data1 = pix1.data();
    let data2 = pix2.data();

    let result = Pix::new(w, h, pix1.depth())?;
    let mut result_mut = result.try_into_mut().unwrap();
    let out_data = result_mut.data_mut();

    for i in 0..total_words {
        out_data[i] = data1[i] & !data2[i];
    }

    Ok(result_mut.into())
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
