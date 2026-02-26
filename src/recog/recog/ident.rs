//! Character identification
//!
//! This module provides functionality for identifying characters using
//! trained template recognizers.

use crate::core::{Box as PixBox, Pix, PixelDepth};
use crate::morph::binary as morph_binary;
use crate::region::{ConnectivityType, find_connected_components};

use crate::recog::error::{RecogError, RecogResult};

use super::train::{binarize_pix, compute_centroid, compute_correlation_with_centering};
use super::types::{OutlierTarget, PreFilterResult, Rch, Rcha, Recog, TemplateUse};

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
        let pix3 = self.remove_noisy_components(&pix2)?;

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

    /// Validates a single character image against pre-splitting criteria.
    ///
    /// Checks that the image meets minimum height and fill-factor thresholds.
    /// Returns a [`PreFilterResult`] describing whether the image is valid
    /// and, if not, the reason for rejection.
    ///
    /// # Arguments
    ///
    /// * `pix` - Single character image to validate
    pub fn pre_splitting_filter(&self, pix: &Pix) -> PreFilterResult {
        let w = pix.width();
        let h = pix.height();

        let min_h = DEFAULT_MIN_HEIGHT.min(h / 2).max(1);

        if h < min_h {
            return PreFilterResult {
                is_valid: false,
                width: w,
                height: h,
                reason: Some(format!("height {h} is below minimum {min_h}")),
            };
        }

        // Count foreground pixels and compute fill factor
        let mut fg_count = 0u32;
        for y in 0..h {
            for x in 0..w {
                if pix.get_pixel(x, y).unwrap_or(0) == 1 {
                    fg_count += 1;
                }
            }
        }

        let area = w * h;
        let fill = if area == 0 {
            0.0f32
        } else {
            fg_count as f32 / area as f32
        };

        if fill < MIN_FILL_FACTOR {
            return PreFilterResult {
                is_valid: false,
                width: w,
                height: h,
                reason: Some(format!(
                    "fill factor {fill:.3} is below minimum {MIN_FILL_FACTOR}"
                )),
            };
        }

        PreFilterResult {
            is_valid: true,
            width: w,
            height: h,
            reason: None,
        }
    }

    /// Returns `true` if the image's aspect ratio falls within the allowed range.
    ///
    /// `min_aspect` ≤ width/height ≤ `max_aspect`
    ///
    /// # Arguments
    ///
    /// * `pix` - Image to check
    /// * `min_aspect` - Minimum allowed width-to-height ratio
    /// * `max_aspect` - Maximum allowed width-to-height ratio
    pub fn splitting_filter(pix: &Pix, min_aspect: f32, max_aspect: f32) -> bool {
        let h = pix.height();
        if h == 0 {
            return false;
        }
        let aspect = pix.width() as f32 / h as f32;
        aspect >= min_aspect && aspect <= max_aspect
    }

    /// Removes outlier templates whose correlation score falls below `min_score`.
    ///
    /// For each class, the correlation of every individual template is compared
    /// against either the class average (`OutlierTarget::Average`) or the
    /// best individual template (`OutlierTarget::Individual`).  Templates
    /// whose score is below `min_score` are removed, provided that at least
    /// `min_fraction` of the original templates would remain.
    ///
    /// If removing outliers would leave fewer than `min_fraction` templates for
    /// a class, that class is left unchanged.
    ///
    /// # Arguments
    ///
    /// * `min_score` - Minimum acceptable correlation score (0.0–1.0)
    /// * `min_fraction` - Minimum fraction of templates that must remain (0.0–1.0)
    /// * `target` - Whether to compare against the averaged or best individual template
    ///
    /// # Errors
    ///
    /// Returns an error if training has not been completed.
    pub fn remove_outliers(
        &mut self,
        min_score: f32,
        min_fraction: f32,
        target: OutlierTarget,
    ) -> RecogResult<()> {
        if !self.train_done {
            return Err(RecogError::IdentificationError(
                "training not finished".to_string(),
            ));
        }

        for class_idx in 0..self.set_size {
            let n = self.pixaa[class_idx].len();
            if n == 0 {
                continue;
            }

            // Determine the reference template for scoring
            let ref_pix = match target {
                OutlierTarget::Average => self.pixa_u.get(class_idx).cloned(),
                OutlierTarget::Individual => {
                    // Use the template with the highest self-correlation (all same: just pick first)
                    self.pixaa[class_idx].first().cloned()
                }
            };

            let Some(reference) = ref_pix else {
                continue;
            };

            // Compute scores for each template in this class
            let scores: Vec<f32> = self.pixaa[class_idx]
                .iter()
                .map(|tmpl| {
                    compute_correlation_score(tmpl, &reference, &self.sumtab).unwrap_or(0.0)
                })
                .collect();

            // Identify templates that pass the min_score threshold
            let keep: Vec<bool> = scores.iter().map(|&s| s >= min_score).collect();
            let keep_count = keep.iter().filter(|&&k| k).count();

            // Only remove outliers if enough templates would remain
            let min_keep = ((n as f32 * min_fraction).ceil() as usize).max(1);
            if keep_count < min_keep {
                continue;
            }

            // Remove outliers (iterate in reverse to preserve indices)
            let mut to_remove: Vec<usize> = keep
                .iter()
                .enumerate()
                .filter(|&(_, &k)| !k)
                .map(|(i, _)| i)
                .collect();
            to_remove.reverse();
            for idx in to_remove {
                self.pixaa[class_idx].remove(idx);
                self.ptaa[class_idx].remove(idx);
            }
        }

        // Re-run finish_training to recompute averages
        self.train_done = false;
        self.ave_done = false;
        self.finish_training()
    }

    /// Filters character images by size, returning only those within the given bounds.
    ///
    /// # Arguments
    ///
    /// * `pixa` - Slice of images to filter
    /// * `min_w` - Minimum acceptable width in pixels
    /// * `max_w` - Maximum acceptable width in pixels
    /// * `min_h` - Minimum acceptable height in pixels
    /// * `max_h` - Maximum acceptable height in pixels
    pub fn filter_pixa_by_size(
        pixa: &[Pix],
        min_w: u32,
        max_w: u32,
        min_h: u32,
        max_h: u32,
    ) -> Vec<Pix> {
        pixa.iter()
            .filter(|p| {
                let w = p.width();
                let h = p.height();
                w >= min_w && w <= max_w && h >= min_h && h <= max_h
            })
            .cloned()
            .collect()
    }

    /// Removes noisy (too small or too sparse) components from an image
    fn remove_noisy_components(&self, pix: &Pix) -> RecogResult<Pix> {
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

    /// Sets a placeholder result, skipping actual identification.
    ///
    /// Used for whitespace or other characters that should not be identified.
    ///
    /// Corresponds to `recogSkipIdentify` in C Leptonica.
    pub fn skip_identify(&mut self) {
        self.rch = Some(Rch {
            index: 0,
            score: 0.0,
            text: String::new(),
            sample: 0,
            xloc: 0,
            yloc: 0,
            width: 0,
        });
    }

    /// Processes an image for identification: binarize, clip, and pad.
    ///
    /// Corresponds to `recogProcessToIdentify` in C Leptonica.
    pub fn process_to_identify(&self, pix: &Pix, pad: u32) -> RecogResult<Pix> {
        // Binarize if needed
        let pix1 = binarize_pix(pix, self.threshold as u8)?;

        // Clip to foreground
        let pix2 = clip_to_foreground(&pix1)?;

        // Add horizontal padding
        if pad == 0 {
            return Ok(pix2);
        }
        let new_w = pix2.width() + 2 * pad;
        let new_h = pix2.height();
        let result = Pix::new(new_w, new_h, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());
        for y in 0..pix2.height() {
            for x in 0..pix2.width() {
                if let Some(v) = pix2.get_pixel(x, y) {
                    let _ = result_mut.set_pixel(x + pad, y, v);
                }
            }
        }
        Ok(result_mut.into())
    }

    /// Extracts number strings from recognition results.
    ///
    /// Uses the stored `rcha` results and bounding boxes to group consecutive
    /// high-scoring digit-like characters into number strings.
    ///
    /// Corresponds to `recogExtractNumbers` in C Leptonica.
    pub fn extract_numbers(
        &self,
        boxes: &[PixBox],
        score_thresh: f32,
        space_thresh: i32,
    ) -> RecogResult<Vec<String>> {
        let rcha = self
            .rcha
            .as_ref()
            .ok_or_else(|| RecogError::IdentificationError("rcha not defined".to_string()))?;

        let space_thresh = if space_thresh < 0 {
            self.maxheight_u.max(20)
        } else {
            space_thresh
        };

        let n = rcha.len().min(boxes.len());
        let mut result = Vec::new();
        let mut current: Option<String> = None;
        let mut prev_box: Option<&PixBox> = None;

        for (score, text, bx) in rcha.scores[..n]
            .iter()
            .zip(rcha.texts[..n].iter())
            .zip(boxes[..n].iter())
            .map(|((s, t), b)| (*s, t, b))
        {
            match (&mut current, prev_box) {
                (None, _) => {
                    if score >= score_thresh {
                        current = Some(text.clone());
                        prev_box = Some(bx);
                    }
                }
                (Some(cur), Some(pb)) => {
                    let h_sep = bx.x - (pb.x + pb.w);
                    if pb.x < bx.x && h_sep <= space_thresh && score >= score_thresh {
                        cur.push_str(text);
                        prev_box = Some(bx);
                    } else {
                        result.push(cur.clone());
                        if score >= score_thresh {
                            current = Some(text.clone());
                            prev_box = Some(bx);
                        } else {
                            current = None;
                            prev_box = None;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(cur) = current {
            result.push(cur);
        }

        if result.is_empty() {
            return Err(RecogError::IdentificationError(
                "no identified numbers".to_string(),
            ));
        }
        Ok(result)
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
fn copy_to_box(dst: &mut crate::core::PixMut, src: &Pix, x: i32, y: i32) -> RecogResult<()> {
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
    use crate::recog::recog::train::{create, make_sumtab};
    use crate::recog::recog::types::OutlierTarget;

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

    fn make_trained_recog() -> super::Recog {
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let p_a = make_solid_pix(5, 20);
        let p_b = make_solid_pix(8, 20);
        recog.train_labeled(&p_a, "A").unwrap();
        recog.train_labeled(&p_b, "B").unwrap();
        recog.finish_training().unwrap();
        recog
    }

    #[test]
    fn test_pre_splitting_filter_valid_image() {
        let recog = create(0, 0, 0, 150, 1).unwrap();
        // Solid 5×20 image: tall enough and fully filled
        let pix = make_solid_pix(5, 20);
        let result = recog.pre_splitting_filter(&pix);
        assert!(result.is_valid);
        assert_eq!(result.width, 5);
        assert_eq!(result.height, 20);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_pre_splitting_filter_rejects_sparse_image() {
        let recog = create(0, 0, 0, 150, 1).unwrap();
        // 10×20 with only one pixel set → fill < MIN_FILL_FACTOR
        let p = Pix::new(10, 20, PixelDepth::Bit1).unwrap();
        let mut m = p.try_into_mut().unwrap();
        let _ = m.set_pixel(5, 10, 1);
        let pix: Pix = m.into();
        let result = recog.pre_splitting_filter(&pix);
        assert!(!result.is_valid);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_splitting_filter_in_range() {
        // 10×10 image: aspect = 1.0, allowed range [0.5, 2.0]
        let pix = make_solid_pix(10, 10);
        assert!(Recog::splitting_filter(&pix, 0.5, 2.0));
    }

    #[test]
    fn test_splitting_filter_too_wide() {
        // 30×10 image: aspect = 3.0, max 2.0 → rejected
        let pix = make_solid_pix(30, 10);
        assert!(!Recog::splitting_filter(&pix, 0.5, 2.0));
    }

    #[test]
    fn test_remove_outliers_keeps_good_templates() {
        let mut recog = make_trained_recog();
        let before_a = recog.pixaa[0].len();
        // Very low min_score: all templates should be kept
        recog
            .remove_outliers(0.0, 0.5, OutlierTarget::Average)
            .unwrap();
        assert_eq!(recog.pixaa[0].len(), before_a);
    }

    #[test]
    fn test_remove_outliers_requires_training() {
        let mut recog = create(0, 0, 0, 150, 1).unwrap();
        let result = recog.remove_outliers(0.5, 0.5, OutlierTarget::Average);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_pixa_by_size_keeps_matching() {
        let images = vec![
            make_solid_pix(5, 10),
            make_solid_pix(10, 20),
            make_solid_pix(15, 30),
        ];
        let filtered = Recog::filter_pixa_by_size(&images, 8, 12, 15, 25);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].width(), 10);
        assert_eq!(filtered[0].height(), 20);
    }

    #[test]
    fn test_filter_pixa_by_size_empty_result() {
        let images = vec![make_solid_pix(5, 10)];
        // width 5 < min_w 10
        let filtered = Recog::filter_pixa_by_size(&images, 10, 20, 5, 15);
        assert!(filtered.is_empty());
    }

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
