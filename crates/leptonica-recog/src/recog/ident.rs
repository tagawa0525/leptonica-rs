//! Character identification
//!
//! This module provides functionality for identifying characters using
//! trained template recognizers.

use leptonica_core::{Box as PixBox, Pix, PixelDepth};
use leptonica_morph::binary as morph_binary;
use leptonica_region::{ConnectivityType, find_connected_components};

use crate::error::{RecogError, RecogResult};

use super::train::{binarize_pix, compute_centroid, compute_correlation_with_centering};
use super::types::{Rch, Rcha, Recog, TemplateUse};

/// Minimum fill factor for filtering components
const MIN_FILL_FACTOR: f32 = 0.10;

/// Default minimum height for components
const DEFAULT_MIN_HEIGHT: u32 = 15;

impl Recog {
    /// Identifies a single character image
    ///
    /// # Arguments
    ///
    /// * `pix` - Character image to identify
    ///
    /// # Returns
    ///
    /// Recognition result with best matching template
    pub fn identify_pix(&self, pix: &Pix) -> RecogResult<Rch> {
        if !self.train_done {
            return Err(RecogError::IdentificationError(
                "training not finished".to_string(),
            ));
        }

        // Prepare the input image
        let processed = self.process_for_identify(pix)?;

        // Find best matching template
        self.correlation_best_char(&processed)
    }

    /// Identifies multiple characters in an image
    ///
    /// This function handles connected component extraction, optional splitting
    /// of touching characters, and identification of each component.
    ///
    /// # Arguments
    ///
    /// * `pix` - Image containing multiple characters
    ///
    /// # Returns
    ///
    /// Recognition results for all identified characters
    pub fn identify_multiple(&self, pix: &Pix) -> RecogResult<Rcha> {
        if !self.train_done {
            return Err(RecogError::IdentificationError(
                "training not finished".to_string(),
            ));
        }

        // Split into individual characters
        let (chars, boxes) = self.split_into_characters(pix)?;

        // Identify each character
        let mut rcha = Rcha::new();
        for (i, char_pix) in chars.iter().enumerate() {
            match self.identify_pix(char_pix) {
                Ok(mut rch) => {
                    // Update location with bounding box
                    if i < boxes.len() {
                        rch.xloc = boxes[i].x;
                        rch.yloc = boxes[i].y;
                    }
                    rcha.push(&rch);
                }
                Err(_) => {
                    // Skip unrecognized characters
                }
            }
        }

        Ok(rcha)
    }

    /// Identifies all images in a Pixa
    ///
    /// # Arguments
    ///
    /// * `pixa` - Array of character images
    ///
    /// # Returns
    ///
    /// Recognition results for each image
    pub fn identify_pixa(&self, pixa: &[Pix]) -> RecogResult<Vec<Rcha>> {
        if !self.train_done {
            return Err(RecogError::IdentificationError(
                "training not finished".to_string(),
            ));
        }

        let mut results = Vec::with_capacity(pixa.len());
        for pix in pixa {
            let rcha = self.identify_multiple(pix)?;
            results.push(rcha);
        }

        Ok(results)
    }

    /// Finds the best matching row (class) using correlation
    ///
    /// This is used for greedy character splitting.
    ///
    /// # Arguments
    ///
    /// * `pix` - Input character image
    ///
    /// # Returns
    ///
    /// (best_class_index, best_score)
    pub fn correlation_best_row(&self, pix: &Pix) -> RecogResult<(i32, f32)> {
        if self.pixa_u.is_empty() {
            return Err(RecogError::IdentificationError(
                "no averaged templates available".to_string(),
            ));
        }

        let (cx, cy) = compute_centroid(pix, &self.centtab)?;
        let mut best_index = -1i32;
        let mut best_score = 0.0f32;

        for (idx, avg) in self.pixa_u.iter().enumerate() {
            let (avg_cx, avg_cy) = self.pta_u[idx];
            let score = compute_correlation_with_centering(
                pix,
                avg,
                cx,
                cy,
                avg_cx,
                avg_cy,
                self.max_y_shift,
                &self.sumtab,
            )?;

            if score > best_score {
                best_score = score;
                best_index = idx as i32;
            }
        }

        Ok((best_index, best_score))
    }

    /// Finds the best matching character using correlation
    ///
    /// # Arguments
    ///
    /// * `pix` - Input character image
    ///
    /// # Returns
    ///
    /// Recognition result with best matching template
    pub fn correlation_best_char(&self, pix: &Pix) -> RecogResult<Rch> {
        if self.set_size == 0 {
            return Err(RecogError::IdentificationError(
                "no templates available".to_string(),
            ));
        }

        // Prepare input
        let processed = if self.scale_w > 0 || self.scale_h > 0 {
            self.modify_template(pix)?
        } else {
            pix.clone()
        };

        let (cx, cy) = compute_centroid(&processed, &self.centtab)?;

        let mut best_rch = Rch::default();

        match self.templ_use {
            TemplateUse::All => {
                // Compare against all templates
                for class_idx in 0..self.set_size {
                    for (sample_idx, templ) in self.pixaa[class_idx].iter().enumerate() {
                        let (templ_cx, templ_cy) = self.ptaa[class_idx][sample_idx];
                        let score = compute_correlation_with_centering(
                            &processed,
                            templ,
                            cx,
                            cy,
                            templ_cx,
                            templ_cy,
                            self.max_y_shift,
                            &self.sumtab,
                        )?;

                        if score > best_rch.score {
                            best_rch = Rch {
                                index: class_idx as i32,
                                score,
                                text: self.sa_text[class_idx].clone(),
                                sample: sample_idx as i32,
                                xloc: 0,
                                yloc: 0,
                                width: templ.width() as i32,
                            };
                        }
                    }
                }
            }
            TemplateUse::Average => {
                // Compare against averaged templates only
                for (class_idx, avg) in self.pixa.iter().enumerate() {
                    let (avg_cx, avg_cy) = self.pta[class_idx];
                    let score = compute_correlation_with_centering(
                        &processed,
                        avg,
                        cx,
                        cy,
                        avg_cx,
                        avg_cy,
                        self.max_y_shift,
                        &self.sumtab,
                    )?;

                    if score > best_rch.score {
                        best_rch = Rch {
                            index: class_idx as i32,
                            score,
                            text: self.sa_text[class_idx].clone(),
                            sample: 0,
                            xloc: 0,
                            yloc: 0,
                            width: avg.width() as i32,
                        };
                    }
                }
            }
        }

        if best_rch.index < 0 {
            return Err(RecogError::IdentificationError(
                "no match found".to_string(),
            ));
        }

        Ok(best_rch)
    }

    /// Splits an image into individual characters
    ///
    /// This handles noise removal, connected component extraction,
    /// and optionally splits touching characters.
    ///
    /// # Arguments
    ///
    /// * `pix` - Input image containing characters
    ///
    /// # Returns
    ///
    /// (character_images, bounding_boxes)
    pub fn split_into_characters(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        // Binarize if needed
        let pix1 = binarize_pix(pix, self.threshold as u8)?;

        // Small vertical close for consolidation
        let pix2 = morph_binary::close_brick(&pix1, 1, 3).map_err(RecogError::Morph)?;

        // Filter out noise
        let pix3 = self.pre_splitting_filter(&pix2)?;

        // Get connected components
        let components = find_connected_components(&pix3, ConnectivityType::EightWay)
            .map_err(RecogError::Region)?;

        if components.is_empty() {
            return Err(RecogError::NoContent(
                "no components found after filtering".to_string(),
            ));
        }

        // Extract and optionally split each component
        let mut chars = Vec::new();
        let mut char_boxes = Vec::new();

        for comp in &components {
            // Extract component
            let comp_pix = extract_box(&pix3, &comp.bounds)?;

            // Check if component needs splitting
            let w = comp.bounds.w as f32;
            let h = comp.bounds.h as f32;

            if w / h > self.max_wh_ratio && self.ave_done {
                // Try to split touching characters using greedy method
                match self.try_split_component(&comp_pix) {
                    Ok((split_chars, split_offsets)) => {
                        for (i, c) in split_chars.into_iter().enumerate() {
                            let new_box = PixBox::new_unchecked(
                                comp.bounds.x + split_offsets[i] as i32,
                                comp.bounds.y,
                                c.width() as i32,
                                c.height() as i32,
                            );
                            chars.push(c);
                            char_boxes.push(new_box);
                        }
                    }
                    Err(_) => {
                        // Keep unsplit component
                        chars.push(comp_pix);
                        char_boxes.push(comp.bounds);
                    }
                }
            } else {
                chars.push(comp_pix);
                char_boxes.push(comp.bounds);
            }
        }

        // Sort by x-coordinate (left to right)
        let mut indexed: Vec<_> = chars.into_iter().zip(char_boxes).collect();
        indexed.sort_by_key(|(_, b)| b.x);
        let (chars, char_boxes): (Vec<_>, Vec<_>) = indexed.into_iter().unzip();

        Ok((chars, char_boxes))
    }

    /// Pre-splitting filter to remove noise
    fn pre_splitting_filter(&self, pix: &Pix) -> RecogResult<Pix> {
        let w = pix.width();
        let h = pix.height();

        // Get connected components
        let components = find_connected_components(pix, ConnectivityType::EightWay)
            .map_err(RecogError::Region)?;

        if components.is_empty() {
            return Err(RecogError::NoContent("no components found".to_string()));
        }

        // Filter components by size and fill factor
        let min_h = DEFAULT_MIN_HEIGHT.min(h / 2);

        let result = Pix::new(w, h, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

        for comp in components {
            // Skip components that are too small
            if (comp.bounds.h as u32) < min_h {
                continue;
            }

            // Calculate fill factor
            let fill =
                comp.pixel_count as f32 / (comp.bounds.w as u32 * comp.bounds.h as u32) as f32;

            if fill < MIN_FILL_FACTOR {
                continue;
            }

            // Copy component to result
            let comp_pix = extract_box(pix, &comp.bounds)?;
            copy_to_box(&mut result_mut, &comp_pix, comp.bounds.x, comp.bounds.y)?;
        }

        Ok(result_mut.into())
    }

    /// Attempts to split a touching character component
    fn try_split_component(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<u32>)> {
        // Use greedy splitting based on template matching
        let w = pix.width();

        let mut splits = Vec::new();
        let mut offsets = Vec::new();
        let mut current_x = 0u32;

        // Simple greedy splitting: find best template match at each position
        while current_x < w {
            let remaining_w = w - current_x;
            if remaining_w < self.min_split_w as u32 {
                break;
            }

            // Try different widths
            let mut best_score = 0.0f32;
            let mut best_width = remaining_w.min(self.maxwidth_u as u32);
            let mut best_template = None;

            for template_w in self.minwidth_u as u32..=remaining_w.min(self.maxwidth_u as u32) {
                // Extract window
                let window_box = PixBox::new_unchecked(
                    current_x as i32,
                    0,
                    template_w as i32,
                    pix.height() as i32,
                );
                if let Ok(window) = extract_box(pix, &window_box)
                    && let Ok((_, score)) = self.correlation_best_row(&window)
                    && score > best_score
                {
                    best_score = score;
                    best_width = template_w;
                    best_template = Some(window);
                }
            }

            if best_score > 0.5
                && let Some(template) = best_template
            {
                splits.push(template);
                offsets.push(current_x);
                current_x += best_width;
            } else {
                // No good match found, include rest as single component
                let rest_box = PixBox::new_unchecked(
                    current_x as i32,
                    0,
                    remaining_w as i32,
                    pix.height() as i32,
                );
                if let Ok(rest) = extract_box(pix, &rest_box) {
                    splits.push(rest);
                    offsets.push(current_x);
                }
                break;
            }
        }

        if splits.is_empty() {
            return Err(RecogError::IdentificationError(
                "could not split component".to_string(),
            ));
        }

        Ok((splits, offsets))
    }

    /// Processes an image for identification
    fn process_for_identify(&self, pix: &Pix) -> RecogResult<Pix> {
        // Binarize if needed
        let pix1 = binarize_pix(pix, self.threshold as u8)?;

        // Clip to foreground
        clip_to_foreground(&pix1)
    }
}

/// Computes correlation score between two images
///
/// # Arguments
///
/// * `pix1` - First image
/// * `pix2` - Second image
/// * `_tab` - Lookup table for sum calculation (unused in this simple version)
///
/// # Returns
///
/// Correlation score (0.0 to 1.0)
pub fn compute_correlation_score(pix1: &Pix, pix2: &Pix, _tab: &[i32]) -> RecogResult<f32> {
    let w1 = pix1.width();
    let h1 = pix1.height();
    let w2 = pix2.width();
    let h2 = pix2.height();

    // Use smaller dimensions
    let w = w1.min(w2);
    let h = h1.min(h2);

    let mut and_count = 0i32;
    let mut count1 = 0i32;
    let mut count2 = 0i32;

    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel(x, y).unwrap_or(0);
            let v2 = pix2.get_pixel(x, y).unwrap_or(0);

            if v1 == 1 {
                count1 += 1;
            }
            if v2 == 1 {
                count2 += 1;
            }
            if v1 == 1 && v2 == 1 {
                and_count += 1;
            }
        }
    }

    if count1 == 0 || count2 == 0 {
        return Ok(0.0);
    }

    Ok(2.0 * and_count as f32 / (count1 + count2) as f32)
}

/// Extracts a region from an image using a Box
fn extract_box(pix: &Pix, bounds: &PixBox) -> RecogResult<Pix> {
    let result =
        Pix::new(bounds.w as u32, bounds.h as u32, pix.depth()).map_err(RecogError::Core)?;
    let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

    for y in 0..bounds.h as u32 {
        for x in 0..bounds.w as u32 {
            let src_x = bounds.x as u32 + x;
            let src_y = bounds.y as u32 + y;
            if src_x < pix.width()
                && src_y < pix.height()
                && let Some(val) = pix.get_pixel(src_x, src_y)
            {
                let _ = result_mut.set_pixel(x, y, val);
            }
        }
    }

    Ok(result_mut.into())
}

/// Copies a region to a destination image at specified position
fn copy_to_box(dst: &mut leptonica_core::PixMut, src: &Pix, x: i32, y: i32) -> RecogResult<()> {
    for sy in 0..src.height() {
        for sx in 0..src.width() {
            if let Some(val) = src.get_pixel(sx, sy)
                && val == 1
            {
                let dx = x + sx as i32;
                let dy = y + sy as i32;
                if dx >= 0 && (dx as u32) < dst.width() && dy >= 0 && (dy as u32) < dst.height() {
                    let _ = dst.set_pixel(dx as u32, dy as u32, 1);
                }
            }
        }
    }
    Ok(())
}

/// Clips an image to its foreground bounding box
fn clip_to_foreground(pix: &Pix) -> RecogResult<Pix> {
    let w = pix.width();
    let h = pix.height();

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..h {
        for x in 0..w {
            if let Some(val) = pix.get_pixel(x, y)
                && val == 1
            {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if max_x < min_x || max_y < min_y {
        return Err(RecogError::NoContent(
            "image has no foreground pixels".to_string(),
        ));
    }

    let new_w = max_x - min_x + 1;
    let new_h = max_y - min_y + 1;

    let result = Pix::new(new_w, new_h, pix.depth()).map_err(RecogError::Core)?;
    let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

    for y in 0..new_h {
        for x in 0..new_w {
            if let Some(val) = pix.get_pixel(x + min_x, y + min_y) {
                let _ = result_mut.set_pixel(x, y, val);
            }
        }
    }

    Ok(result_mut.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recog::train::{create, make_sumtab};

    #[test]
    fn test_compute_correlation_score_identical() {
        let sumtab = make_sumtab();
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Create a simple pattern
        for y in 2..8 {
            for x in 2..8 {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }

        let pix: Pix = pix_mut.into();
        let score = compute_correlation_score(&pix, &pix, &sumtab).unwrap();
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_correlation_score_different() {
        let sumtab = make_sumtab();
        let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let pix2 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix1_mut = pix1.try_into_mut().unwrap();
        let mut pix2_mut = pix2.try_into_mut().unwrap();

        // Create non-overlapping patterns
        for y in 0..5 {
            for x in 0..5 {
                let _ = pix1_mut.set_pixel(x, y, 1);
            }
        }
        for y in 5..10 {
            for x in 5..10 {
                let _ = pix2_mut.set_pixel(x, y, 1);
            }
        }

        let pix1: Pix = pix1_mut.into();
        let pix2: Pix = pix2_mut.into();
        let score = compute_correlation_score(&pix1, &pix2, &sumtab).unwrap();
        assert!(score < 0.1);
    }

    #[test]
    fn test_extract_box() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set a 5x5 block
        for y in 5..10 {
            for x in 5..10 {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }

        let pix: Pix = pix_mut.into();
        let bounds = PixBox::new_unchecked(5, 5, 5, 5);
        let extracted = extract_box(&pix, &bounds).unwrap();

        assert_eq!(extracted.width(), 5);
        assert_eq!(extracted.height(), 5);

        // All pixels should be set
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(extracted.get_pixel(x, y), Some(1));
            }
        }
    }

    #[test]
    fn test_clip_to_foreground() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set a 5x5 block in the center
        for y in 7..12 {
            for x in 7..12 {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }

        let pix: Pix = pix_mut.into();
        let clipped = clip_to_foreground(&pix).unwrap();
        assert_eq!(clipped.width(), 5);
        assert_eq!(clipped.height(), 5);
    }

    #[test]
    fn test_identify_not_trained() {
        let recog = create(40, 40, 0, 150, 1).unwrap();
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let result = recog.identify_pix(&pix);
        assert!(result.is_err());
    }
}
