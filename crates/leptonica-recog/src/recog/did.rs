//! Document Image Decoding (DID) using Viterbi algorithm
//!
//! This module implements the DID approach to character recognition,
//! which uses dynamic programming (Viterbi algorithm) to find the
//! optimal segmentation and labeling of text.
//!
//! The DID method was pioneered by Gary Kopec and provides a
//! maximum a posteriori (MAP) decoding of document images.

use leptonica_core::{Box as PixBox, Pix};

use crate::error::{RecogError, RecogResult};

use super::train::{binarize_pix, compute_centroid};
use super::types::{Rcha, Rdid, Recog};

/// Setwidth fraction (how far to advance after printing a character)
const SETWIDTH_FRACTION: f32 = 0.95;

/// Maximum vertical shift for template matching
const MAX_Y_SHIFT: i32 = 1;

/// Default channel parameters for 2-level templates
const DEFAULT_ALPHA_2: [f32; 2] = [0.95, 0.9];

impl Recog {
    /// Decodes a text line using Document Image Decoding
    ///
    /// This uses the Viterbi algorithm to find the optimal segmentation
    /// and labeling of characters in the input image.
    ///
    /// # Arguments
    ///
    /// * `pix` - Input image containing a text line (1 bpp)
    ///
    /// # Returns
    ///
    /// Recognition results for the decoded characters
    pub fn decode(&mut self, pix: &Pix) -> RecogResult<Rcha> {
        if !self.train_done {
            return Err(RecogError::IdentificationError(
                "training not finished".to_string(),
            ));
        }

        // Prepare for decoding
        self.create_did(pix)?;

        // Run Viterbi algorithm
        self.run_viterbi()?;

        // Extract results
        let result = self.extract_viterbi_result()?;

        Ok(result)
    }

    /// Creates DID state for decoding
    pub fn create_did(&mut self, pix: &Pix) -> RecogResult<()> {
        if self.pixa_u.is_empty() {
            return Err(RecogError::IdentificationError(
                "no averaged templates available".to_string(),
            ));
        }

        // Binarize if needed
        let pixs = binarize_pix(pix, self.threshold as u8)?;

        let narray = self.pixa_u.len();
        let mut did = Rdid::new(pixs, narray);

        // Set up channel parameters
        self.set_channel_params(&mut did)?;

        // Compute setwidths for each template
        for i in 0..narray {
            let w = self.pixa_u[i].width() as i32;
            did.setwidth[i] = (w as f32 * SETWIDTH_FRACTION) as i32;
        }

        // Compute column sums and moments for the input image
        self.compute_column_stats(&mut did)?;

        // Generate decoding arrays for each template
        for i in 0..narray {
            self.make_decoding_array(&mut did, i)?;
        }

        did.fullarrays = true;
        self.did = Some(did);

        Ok(())
    }

    /// Destroys the DID state
    pub fn destroy_did(&mut self) {
        self.did = None;
    }

    /// Runs the Viterbi dynamic programming algorithm
    pub fn run_viterbi(&mut self) -> RecogResult<()> {
        // Clone the nasum_u to avoid borrow issues
        let nasum_u = self.nasum_u.clone();

        let did = self
            .did
            .as_mut()
            .ok_or_else(|| RecogError::IdentificationError("DID not initialized".to_string()))?;

        let size = did.size;
        let narray = did.narray;

        if size == 0 || narray == 0 {
            return Err(RecogError::IdentificationError(
                "empty decoding arrays".to_string(),
            ));
        }

        // Initialize trellis
        for i in 0..size {
            did.trellisscore[i] = f32::NEG_INFINITY;
            did.trellistempl[i] = -1;
        }

        // Initialize first column - allow any template to start
        for t in 0..narray {
            if !did.counta.is_empty() && !did.counta[t].is_empty() {
                let setwidth = did.setwidth[t] as usize;
                if setwidth < size {
                    let score = compute_template_score_inner(did, &nasum_u, t, 0)?;
                    if score > did.trellisscore[setwidth] {
                        did.trellisscore[setwidth] = score;
                        did.trellistempl[setwidth] = t as i32;
                    }
                }
            }
        }

        // Forward pass through trellis
        for x in 1..size {
            // Check if we can extend from this position
            if did.trellisscore[x] > f32::NEG_INFINITY {
                // Try each template at this position
                for t in 0..narray {
                    let setwidth = did.setwidth[t] as usize;
                    let next_x = x + setwidth;

                    if next_x < size {
                        let template_score = compute_template_score_inner(did, &nasum_u, t, x)?;
                        let new_score = did.trellisscore[x] + template_score;

                        if new_score > did.trellisscore[next_x] {
                            did.trellisscore[next_x] = new_score;
                            did.trellistempl[next_x] = t as i32;
                        }
                    }
                }
            }
        }

        // Backtrack to find best path
        self.backtrack_viterbi()?;

        Ok(())
    }

    /// Rescores the DID result using full template matching
    pub fn rescore_did_result(&mut self) -> RecogResult<()> {
        // Collect data from did first
        let (segments, template_indices, pix_xs, pix_ws): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = {
            let did = self.did.as_ref().ok_or_else(|| {
                RecogError::IdentificationError("DID not initialized".to_string())
            })?;

            let mut segments = Vec::new();
            let mut template_indices = Vec::new();
            let mut pix_xs = Vec::new();
            let mut pix_ws = Vec::new();

            for i in 0..did.boxa.len() {
                let pix_box = &did.boxa[i];
                if let Ok(segment) = extract_rect(&did.pixs, pix_box) {
                    segments.push(segment);
                    template_indices.push(did.natempl[i]);
                    pix_xs.push(pix_box.x);
                    pix_ws.push(pix_box.w);
                }
            }

            (segments, template_indices, pix_xs, pix_ws)
        };

        // Now compute scores with &self
        let mut results = Vec::new();
        for (i, segment) in segments.iter().enumerate() {
            let template_idx = template_indices[i];
            let (best_sample, best_score, best_dely) =
                self.find_best_sample_match(segment, template_idx as usize)?;
            results.push((
                template_idx,
                best_sample,
                pix_xs[i],
                best_dely,
                pix_ws[i],
                best_score,
            ));
        }

        // Finally update did
        let did = self
            .did
            .as_mut()
            .ok_or_else(|| RecogError::IdentificationError("DID not initialized".to_string()))?;

        // Clear rescored results
        did.natempl_r.clear();
        did.nasample_r.clear();
        did.naxloc_r.clear();
        did.nadely_r.clear();
        did.nawidth_r.clear();
        did.nascore_r.clear();

        for (template_idx, best_sample, x, dely, w, score) in results {
            did.natempl_r.push(template_idx);
            did.nasample_r.push(best_sample);
            did.naxloc_r.push(x);
            did.nadely_r.push(dely);
            did.nawidth_r.push(w);
            did.nascore_r.push(score);
        }

        Ok(())
    }

    /// Sets channel parameters for scoring
    fn set_channel_params(&self, did: &mut Rdid) -> RecogResult<()> {
        let narray = did.narray;

        // Use 2-level channel model
        let alpha = DEFAULT_ALPHA_2;

        // Compute beta and gamma from alpha values
        // beta = log(alpha[1]) - log(1 - alpha[1])
        // gamma = log(alpha[0]) - log(1 - alpha[0]) + log(1 - alpha[1]) - log(alpha[1])
        let log_alpha0 = alpha[0].ln();
        let log_1_alpha0 = (1.0 - alpha[0]).ln();
        let log_alpha1 = alpha[1].ln();
        let log_1_alpha1 = (1.0 - alpha[1]).ln();

        for i in 0..narray {
            did.beta[i] = log_alpha1 - log_1_alpha1;
            did.gamma[i] = log_alpha0 - log_1_alpha0 + log_1_alpha1 - log_alpha1;
        }

        Ok(())
    }

    /// Computes column statistics for the input image
    fn compute_column_stats(&self, did: &mut Rdid) -> RecogResult<()> {
        let w = did.pixs.width();
        let h = did.pixs.height();

        for x in 0..w {
            let mut sum = 0i32;
            let mut moment = 0i64;

            for y in 0..h {
                if let Some(val) = did.pixs.get_pixel(x, y)
                    && val == 1
                {
                    sum += 1;
                    moment += y as i64;
                }
            }

            did.nasum[x as usize] = sum;
            did.namoment[x as usize] = if sum > 0 {
                (moment / sum as i64) as i32
            } else {
                (h / 2) as i32
            };
        }

        Ok(())
    }

    /// Makes decoding array for a single template
    fn make_decoding_array(&self, did: &mut Rdid, index: usize) -> RecogResult<()> {
        let template = &self.pixa_u[index];
        let template_w = template.width() as i32;
        let template_h = template.height() as i32;
        let size = did.size;

        let mut count_array = vec![0i32; size];
        let mut dely_array = vec![0i32; size];

        let (_, template_cy) = self.pta_u[index];
        let img_h = did.pixs.height() as i32;

        // For each starting position
        for x in 0..(size as i32 - template_w + 1) {
            let x_usize = x as usize;

            // Compute windowed centroid of input
            let mut sum_y = 0i64;
            let mut count = 0i32;
            for dx in 0..template_w {
                let col_x = (x + dx) as usize;
                if col_x < size {
                    sum_y += did.namoment[col_x] as i64 * did.nasum[col_x] as i64;
                    count += did.nasum[col_x];
                }
            }
            let img_cy = if count > 0 {
                (sum_y / count as i64) as i32
            } else {
                img_h / 2
            };

            // Try different y-shifts
            let mut best_count = 0i32;
            let mut best_dely = 0i32;

            for dely in -MAX_Y_SHIFT..=MAX_Y_SHIFT {
                let offset_y = img_cy - template_cy as i32 + dely;

                // Count bit-and overlap
                let mut and_count = 0i32;

                for ty in 0..template_h {
                    let iy = offset_y + ty;
                    if iy < 0 || iy >= img_h {
                        continue;
                    }

                    for tx in 0..template_w {
                        let ix = x + tx;
                        if ix < 0 || ix >= size as i32 {
                            continue;
                        }

                        if let Some(tv) = template.get_pixel(tx as u32, ty as u32)
                            && tv == 1
                            && let Some(iv) = did.pixs.get_pixel(ix as u32, iy as u32)
                            && iv == 1
                        {
                            and_count += 1;
                        }
                    }
                }

                if and_count > best_count {
                    best_count = and_count;
                    best_dely = dely;
                }
            }

            count_array[x_usize] = best_count;
            dely_array[x_usize] = best_dely;
        }

        did.counta.push(count_array);
        did.delya.push(dely_array);

        Ok(())
    }
}

/// Helper function to compute template score without borrowing self
fn compute_template_score_inner(
    did: &Rdid,
    nasum_u: &[i32],
    template_idx: usize,
    x: usize,
) -> RecogResult<f32> {
    if template_idx >= did.counta.len() || x >= did.counta[template_idx].len() {
        return Ok(f32::NEG_INFINITY);
    }

    let and_count = did.counta[template_idx][x];
    let template_area = nasum_u.get(template_idx).copied().unwrap_or(0);

    let beta = did.beta[template_idx];
    let gamma = did.gamma[template_idx];

    // Score = beta * and_count + gamma * template_area
    let score = beta * and_count as f32 + gamma * template_area as f32;

    Ok(score)
}

impl Recog {
    /// Backtracks through trellis to find best path
    fn backtrack_viterbi(&mut self) -> RecogResult<()> {
        // Clone data needed during the loop before mutable borrow
        let template_heights: Vec<u32> = self.pixa_u.iter().map(|p| p.height()).collect();
        let nasum_u_clone = self.nasum_u.clone();
        let maxwidth_u = self.maxwidth_u;

        let did = self
            .did
            .as_mut()
            .ok_or_else(|| RecogError::IdentificationError("DID not initialized".to_string()))?;

        // Find best ending position
        let size = did.size;
        let mut best_x = 0usize;
        let mut best_score = f32::NEG_INFINITY;

        // Look for best score in last portion of trellis
        let search_start = size.saturating_sub(maxwidth_u as usize);
        for x in search_start..size {
            if did.trellisscore[x] > best_score {
                best_score = did.trellisscore[x];
                best_x = x;
            }
        }

        if best_score == f32::NEG_INFINITY {
            return Err(RecogError::IdentificationError(
                "no valid path found".to_string(),
            ));
        }

        // Backtrack
        let mut path = Vec::new();
        let mut x = best_x;

        while x > 0 && did.trellistempl[x] >= 0 {
            let template_idx = did.trellistempl[x] as usize;
            let setwidth = did.setwidth[template_idx] as usize;
            let start_x = x.saturating_sub(setwidth);

            path.push((start_x, template_idx, setwidth));
            x = start_x;

            if x == 0 {
                break;
            }
        }

        path.reverse();

        // Store results
        did.natempl.clear();
        did.naxloc.clear();
        did.nadely.clear();
        did.nawidth.clear();
        did.boxa.clear();
        did.nascore.clear();

        for (start_x, template_idx, width) in path {
            let template_h = template_heights.get(template_idx).copied().unwrap_or(0);

            did.natempl.push(template_idx as i32);
            did.naxloc.push(start_x as i32);
            did.nadely.push(if start_x < did.delya[template_idx].len() {
                did.delya[template_idx][start_x]
            } else {
                0
            });
            did.nawidth.push(width as i32);

            let pix_box = PixBox::new(start_x as i32, 0, width as i32, template_h as i32)
                .map_err(RecogError::Core)?;
            did.boxa.push(pix_box);

            let score = compute_template_score_inner(did, &nasum_u_clone, template_idx, start_x)?;
            did.nascore.push(score);
        }

        Ok(())
    }

    /// Finds best sample match for rescoring
    fn find_best_sample_match(
        &self,
        segment: &Pix,
        class_idx: usize,
    ) -> RecogResult<(i32, f32, i32)> {
        if class_idx >= self.pixaa.len() {
            return Err(RecogError::IdentificationError(
                "invalid class index".to_string(),
            ));
        }

        let (seg_cx, seg_cy) = compute_centroid(segment, &self.centtab)?;
        let mut best_sample = 0i32;
        let mut best_score = 0.0f32;
        let mut best_dely = 0i32;

        for (sample_idx, template) in self.pixaa[class_idx].iter().enumerate() {
            let (templ_cx, templ_cy) = self.ptaa[class_idx][sample_idx];

            for dely in -self.max_y_shift..=self.max_y_shift {
                let score = compute_correlation_score_aligned(
                    segment,
                    template,
                    seg_cx,
                    seg_cy,
                    templ_cx,
                    templ_cy + dely as f32,
                )?;

                if score > best_score {
                    best_score = score;
                    best_sample = sample_idx as i32;
                    best_dely = dely;
                }
            }
        }

        Ok((best_sample, best_score, best_dely))
    }

    /// Extracts Viterbi result as Rcha
    fn extract_viterbi_result(&self) -> RecogResult<Rcha> {
        let did = self
            .did
            .as_ref()
            .ok_or_else(|| RecogError::IdentificationError("DID not initialized".to_string()))?;

        let mut rcha = Rcha::new();

        for i in 0..did.natempl.len() {
            let template_idx = did.natempl[i] as usize;

            if template_idx < self.sa_text.len() {
                rcha.indices.push(template_idx as i32);
                rcha.texts.push(self.sa_text[template_idx].clone());
                rcha.xlocs.push(did.naxloc[i]);
                rcha.ylocs.push(did.nadely[i]);
                rcha.widths.push(did.nawidth[i]);
                rcha.scores.push(did.nascore[i]);
                rcha.samples.push(0); // Will be filled by rescoring
            }
        }

        Ok(rcha)
    }
}

/// Extracts a rectangular region from an image
fn extract_rect(pix: &Pix, pix_box: &PixBox) -> RecogResult<Pix> {
    let w = pix_box.w.max(0) as u32;
    let h = pix_box.h.max(0) as u32;

    let result = Pix::new(w, h, pix.depth()).map_err(RecogError::Core)?;
    let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

    for y in 0..h {
        for x in 0..w {
            let src_x = pix_box.x as u32 + x;
            let src_y = pix_box.y as u32 + y;
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

/// Computes correlation score between two aligned images
fn compute_correlation_score_aligned(
    pix1: &Pix,
    pix2: &Pix,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
) -> RecogResult<f32> {
    let w1 = pix1.width() as i32;
    let h1 = pix1.height() as i32;
    let w2 = pix2.width() as i32;
    let h2 = pix2.height() as i32;

    let dx = (cx2 - cx1).round() as i32;
    let dy = (cy2 - cy1).round() as i32;

    let mut and_count = 0i32;
    let mut count1 = 0i32;
    let mut count2 = 0i32;

    for y1 in 0..h1 {
        for x1 in 0..w1 {
            if let Some(v1) = pix1.get_pixel(x1 as u32, y1 as u32)
                && v1 == 1
            {
                count1 += 1;
                let x2 = x1 + dx;
                let y2 = y1 + dy;
                if x2 >= 0
                    && x2 < w2
                    && y2 >= 0
                    && y2 < h2
                    && let Some(v2) = pix2.get_pixel(x2 as u32, y2 as u32)
                    && v2 == 1
                {
                    and_count += 1;
                }
            }
        }
    }

    for y2 in 0..h2 {
        for x2 in 0..w2 {
            if let Some(v2) = pix2.get_pixel(x2 as u32, y2 as u32)
                && v2 == 1
            {
                count2 += 1;
            }
        }
    }

    if count1 == 0 || count2 == 0 {
        return Ok(0.0);
    }

    Ok(2.0 * and_count as f32 / (count1 + count2) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recog::train::create;
    use leptonica_core::PixelDepth;

    #[test]
    fn test_decode_not_trained() {
        let mut recog = create(40, 40, 0, 150, 1).unwrap();
        let pix = Pix::new(100, 40, PixelDepth::Bit1).unwrap();

        let result = recog.decode(&pix);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_did_no_templates() {
        let mut recog = create(40, 40, 0, 150, 1).unwrap();
        let pix = Pix::new(100, 40, PixelDepth::Bit1).unwrap();

        let result = recog.create_did(&pix);
        assert!(result.is_err());
    }

    #[test]
    fn test_destroy_did() {
        let mut recog = create(40, 40, 0, 150, 1).unwrap();
        recog.destroy_did();
        assert!(recog.did.is_none());
    }

    #[test]
    fn test_correlation_score_aligned_empty() {
        let pix1 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let pix2 = Pix::new(10, 10, PixelDepth::Bit1).unwrap();

        let score = compute_correlation_score_aligned(&pix1, &pix2, 5.0, 5.0, 5.0, 5.0).unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_correlation_score_aligned_identical() {
        let pix_blank = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix_blank.try_into_mut().unwrap_or_else(|p| p.to_mut());
        for y in 2..8 {
            for x in 2..8 {
                let _ = pix_mut.set_pixel(x, y, 1);
            }
        }
        let pix: Pix = pix_mut.into();

        let score = compute_correlation_score_aligned(&pix, &pix, 5.0, 5.0, 5.0, 5.0).unwrap();
        assert!((score - 1.0).abs() < 0.01);
    }
}
