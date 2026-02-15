//! Template training for character recognition
//!
//! This module provides functionality for training character recognizers
//! using labeled template images.

use leptonica_color::threshold_to_binary;
use leptonica_core::{Pix, PixelDepth};
use leptonica_morph::binary as morph_binary;
use leptonica_transform::scale;

use crate::error::{RecogError, RecogResult};

use super::types::{
    CharsetType, DEFAULT_MAX_ARRAY_SIZE, DEFAULT_MAX_HT_RATIO, DEFAULT_MAX_SPLIT_H,
    DEFAULT_MAX_WH_RATIO, DEFAULT_MIN_SPLIT_W, DEFAULT_THRESHOLD, Recog, TemplateUse,
};

/// Binarizes a Pix image using a threshold
///
/// This helper converts grayscale or color images to 1bpp binary.
pub fn binarize_pix(pix: &Pix, threshold: u8) -> RecogResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.clone()),
        PixelDepth::Bit8 => threshold_to_binary(pix, threshold).map_err(RecogError::Color),
        _ => {
            // For other depths, convert to grayscale first
            let gray = leptonica_color::pix_convert_to_gray(pix).map_err(RecogError::Color)?;
            threshold_to_binary(&gray, threshold).map_err(RecogError::Color)
        }
    }
}

/// Creates an empty character recognizer
///
/// # Arguments
///
/// * `scale_w` - Target width for scaling (0 = no horizontal scaling)
/// * `scale_h` - Target height for scaling (0 = no vertical scaling)
/// * `line_w` - Line width for skeleton-based recognition (0 = skip)
/// * `threshold` - Binarization threshold for depth > 1
/// * `max_y_shift` - Maximum vertical shift during matching (typically 0 or 1)
///
/// # Returns
///
/// A new empty recognizer ready for training
pub fn create(
    scale_w: i32,
    scale_h: i32,
    line_w: i32,
    threshold: i32,
    max_y_shift: i32,
) -> RecogResult<Recog> {
    if !(0..=2).contains(&max_y_shift) {
        return Err(RecogError::InvalidParameter(
            "max_y_shift must be 0, 1, or 2".to_string(),
        ));
    }

    // Initialize lookup tables
    let centtab = make_centtab();
    let sumtab = make_sumtab();

    Ok(Recog {
        scale_w,
        scale_h,
        line_w,
        templ_use: TemplateUse::All,
        max_array_size: DEFAULT_MAX_ARRAY_SIZE,
        set_size: 0,
        threshold: if threshold > 0 {
            threshold
        } else {
            DEFAULT_THRESHOLD
        },
        max_y_shift,
        charset_type: CharsetType::Unknown,
        charset_size: 0,
        min_nopad: 0,
        num_samples: 0,
        minwidth_u: i32::MAX,
        maxwidth_u: 0,
        minheight_u: i32::MAX,
        maxheight_u: 0,
        minwidth: i32::MAX,
        maxwidth: 0,
        ave_done: false,
        train_done: false,
        max_wh_ratio: DEFAULT_MAX_WH_RATIO,
        max_ht_ratio: DEFAULT_MAX_HT_RATIO,
        min_split_w: DEFAULT_MIN_SPLIT_W,
        max_split_h: DEFAULT_MAX_SPLIT_H,
        sa_text: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        dna_tochar: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        centtab,
        sumtab,
        pixaa_u: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        ptaa_u: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        naasum_u: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        pixaa: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        ptaa: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        naasum: Vec::with_capacity(DEFAULT_MAX_ARRAY_SIZE),
        pixa_u: Vec::new(),
        pta_u: Vec::new(),
        nasum_u: Vec::new(),
        pixa: Vec::new(),
        pta: Vec::new(),
        nasum: Vec::new(),
        pixa_tr: Vec::new(),
        did: None,
        rch: None,
        rcha: None,
    })
}

/// Creates a recognizer from labeled images
///
/// # Arguments
///
/// * `pixa` - Array of labeled character images (1 bpp with text labels)
/// * `labels` - Text labels for each image
/// * `scale_w` - Target width for scaling (0 = no horizontal scaling)
/// * `scale_h` - Target height for scaling (0 = no vertical scaling)
/// * `line_w` - Line width for skeleton-based recognition (0 = skip)
/// * `threshold` - Binarization threshold for depth > 1
/// * `max_y_shift` - Maximum vertical shift during matching (typically 0 or 1)
///
/// # Returns
///
/// A new recognizer trained on the provided images
pub fn create_from_pixa(
    pixa: &[Pix],
    labels: &[&str],
    scale_w: i32,
    scale_h: i32,
    line_w: i32,
    threshold: i32,
    max_y_shift: i32,
) -> RecogResult<Recog> {
    if pixa.len() != labels.len() {
        return Err(RecogError::InvalidParameter(
            "pixa and labels must have the same length".to_string(),
        ));
    }

    let mut recog = create(scale_w, scale_h, line_w, threshold, max_y_shift)?;

    for (pix, label) in pixa.iter().zip(labels.iter()) {
        recog.train_labeled(pix, label)?;
    }

    recog.finish_training()?;

    Ok(recog)
}

impl Recog {
    /// Adds a labeled sample to the recognizer
    ///
    /// # Arguments
    ///
    /// * `pix` - Character image (will be converted to 1 bpp if needed)
    /// * `label` - Text label for this character
    pub fn train_labeled(&mut self, pix: &Pix, label: &str) -> RecogResult<()> {
        if self.train_done {
            return Err(RecogError::TrainingError(
                "training has already been completed".to_string(),
            ));
        }

        if label.is_empty() {
            return Err(RecogError::InvalidParameter(
                "label cannot be empty".to_string(),
            ));
        }

        // Process the image: binarize and clean
        let processed = self.process_labeled(pix)?;

        // Add the sample
        self.add_sample(&processed, label)?;

        Ok(())
    }

    /// Processes a labeled image for training
    fn process_labeled(&self, pix: &Pix) -> RecogResult<Pix> {
        // Convert to 1 bpp if needed
        let pix1 = binarize_pix(pix, self.threshold as u8)?;

        // Remove isolated noise using morphological opening
        let pix2 = morph_binary::open_brick(&pix1, 1, 5).map_err(RecogError::Morph)?;

        // AND with original to preserve shape while removing noise
        let pix3 = and_images(&pix2, &pix1)?;

        // Clip to foreground
        let pix4 = clip_to_foreground(&pix3)?;

        Ok(pix4)
    }

    /// Adds a processed sample to the recognizer
    fn add_sample(&mut self, pix: &Pix, label: &str) -> RecogResult<()> {
        // Find or create the class index
        let index = self.get_or_create_class_index(label);

        // Ensure we have storage for this class
        while self.pixaa_u.len() <= index {
            self.pixaa_u.push(Vec::new());
            self.ptaa_u.push(Vec::new());
            self.naasum_u.push(Vec::new());
        }

        // Calculate centroid and area
        let (cx, cy) = compute_centroid(pix, &self.centtab)?;
        let area = compute_area(pix, &self.sumtab)?;

        // Update size statistics
        let w = pix.width() as i32;
        let h = pix.height() as i32;
        self.minwidth_u = self.minwidth_u.min(w);
        self.maxwidth_u = self.maxwidth_u.max(w);
        self.minheight_u = self.minheight_u.min(h);
        self.maxheight_u = self.maxheight_u.max(h);

        // Add to class storage
        self.pixaa_u[index].push(pix.clone());
        self.ptaa_u[index].push((cx, cy));
        self.naasum_u[index].push(area);

        // Store in training array
        self.pixa_tr.push(pix.clone());

        self.num_samples += 1;

        Ok(())
    }

    /// Gets or creates a class index for the given label
    fn get_or_create_class_index(&mut self, label: &str) -> usize {
        // Check if class already exists
        if let Some(index) = self.sa_text.iter().position(|s| s == label) {
            return index;
        }

        // Create new class
        let index = self.sa_text.len();
        self.sa_text.push(label.to_string());
        self.set_size = index + 1;

        index
    }

    /// Computes averaged templates for each class
    pub fn average_samples(&mut self) -> RecogResult<()> {
        if self.ave_done {
            return Ok(());
        }

        if self.set_size == 0 {
            return Err(RecogError::TrainingError(
                "no samples to average".to_string(),
            ));
        }

        // Clear existing averages
        self.pixa_u.clear();
        self.pta_u.clear();
        self.nasum_u.clear();
        self.pixa.clear();
        self.pta.clear();
        self.nasum.clear();

        // Process each class
        for class_idx in 0..self.set_size {
            let samples = &self.pixaa_u[class_idx];
            if samples.is_empty() {
                return Err(RecogError::TrainingError(format!(
                    "class {} has no samples",
                    class_idx
                )));
            }

            // Compute unscaled average
            let (avg_u, cx_u, cy_u, area_u) =
                self.compute_class_average(samples, &self.ptaa_u[class_idx])?;
            self.pixa_u.push(avg_u);
            self.pta_u.push((cx_u, cy_u));
            self.nasum_u.push(area_u);

            // Compute scaled average (if scaling is enabled)
            if self.scale_w > 0 || self.scale_h > 0 {
                let scaled_samples: Vec<Pix> = samples
                    .iter()
                    .map(|p| self.modify_template(p))
                    .collect::<RecogResult<Vec<_>>>()?;

                let scaled_centroids: Vec<(f32, f32)> = scaled_samples
                    .iter()
                    .map(|p| compute_centroid(p, &self.centtab))
                    .collect::<RecogResult<Vec<_>>>()?;

                let (avg, cx, cy, area) =
                    self.compute_class_average(&scaled_samples, &scaled_centroids)?;
                self.pixa.push(avg);
                self.pta.push((cx, cy));
                self.nasum.push(area);
            } else {
                // No scaling: use unscaled as scaled
                self.pixa.push(self.pixa_u[class_idx].clone());
                self.pta.push(self.pta_u[class_idx]);
                self.nasum.push(self.nasum_u[class_idx]);
            }
        }

        // Compute min/max widths for scaled templates
        self.minwidth = i32::MAX;
        self.maxwidth = 0;
        for pix in &self.pixa {
            let w = pix.width() as i32;
            self.minwidth = self.minwidth.min(w);
            self.maxwidth = self.maxwidth.max(w);
        }

        self.ave_done = true;
        Ok(())
    }

    /// Computes the average template for a class
    fn compute_class_average(
        &self,
        samples: &[Pix],
        centroids: &[(f32, f32)],
    ) -> RecogResult<(Pix, f32, f32, i32)> {
        if samples.is_empty() {
            return Err(RecogError::TrainingError(
                "cannot compute average of empty class".to_string(),
            ));
        }

        // Find the target size (max dimensions)
        let max_w = samples.iter().map(|p| p.width()).max().unwrap_or(1);
        let max_h = samples.iter().map(|p| p.height()).max().unwrap_or(1);

        // Create accumulator
        let mut accum = vec![0u32; (max_w * max_h) as usize];
        let threshold = (samples.len() / 2) as u32;

        // Accumulate aligned samples
        for (pix, &(cx, cy)) in samples.iter().zip(centroids.iter()) {
            let w = pix.width();
            let h = pix.height();

            // Calculate offset to center
            let offset_x = (max_w as f32 / 2.0 - cx) as i32;
            let offset_y = (max_h as f32 / 2.0 - cy) as i32;

            // Add pixels to accumulator
            for y in 0..h {
                for x in 0..w {
                    if let Some(val) = pix.get_pixel(x, y)
                        && val == 1
                    {
                        let dst_x = x as i32 + offset_x;
                        let dst_y = y as i32 + offset_y;
                        if dst_x >= 0 && dst_x < max_w as i32 && dst_y >= 0 && dst_y < max_h as i32
                        {
                            accum[(dst_y as u32 * max_w + dst_x as u32) as usize] += 1;
                        }
                    }
                }
            }
        }

        // Create averaged image
        let avg = Pix::new(max_w, max_h, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut avg_mut = avg.try_into_mut().unwrap_or_else(|p| p.to_mut());
        let mut area = 0i32;

        for y in 0..max_h {
            for x in 0..max_w {
                let count = accum[(y * max_w + x) as usize];
                if count > threshold {
                    let _ = avg_mut.set_pixel(x, y, 1);
                    area += 1;
                }
            }
        }

        // Compute centroid of average
        let avg_pix: Pix = avg_mut.into();
        let (cx, cy) = compute_centroid(&avg_pix, &self.centtab)?;

        Ok((avg_pix, cx, cy, area))
    }

    /// Modifies a template by scaling and/or converting to lines
    pub fn modify_template(&self, pix: &Pix) -> RecogResult<Pix> {
        let w = pix.width();
        let h = pix.height();

        // Scale if needed
        let pix1 = if (self.scale_w == 0 || self.scale_w == w as i32)
            && (self.scale_h == 0 || self.scale_h == h as i32)
        {
            pix.clone()
        } else {
            let target_w = if self.scale_w > 0 {
                self.scale_w as u32
            } else {
                w
            };
            let target_h = if self.scale_h > 0 {
                self.scale_h as u32
            } else {
                h
            };
            scale::scale_to_size(pix, target_w, target_h).map_err(RecogError::Transform)?
        };

        // Convert to lines if needed
        let pix2 = if self.line_w > 0 {
            set_stroke_width(&pix1, self.line_w as u32)?
        } else {
            pix1
        };

        Ok(pix2)
    }

    /// Finishes training and prepares for identification
    pub fn finish_training(&mut self) -> RecogResult<()> {
        if self.train_done {
            return Ok(());
        }

        if self.set_size == 0 {
            return Err(RecogError::TrainingError("no classes defined".to_string()));
        }

        // Compute averages if not already done
        self.average_samples()?;

        // Build scaled templates for each class
        self.pixaa.clear();
        self.ptaa.clear();
        self.naasum.clear();

        for class_idx in 0..self.set_size {
            let mut scaled_samples = Vec::new();
            let mut scaled_centroids = Vec::new();
            let mut scaled_areas = Vec::new();

            for pix in &self.pixaa_u[class_idx] {
                let modified = self.modify_template(pix)?;
                let (cx, cy) = compute_centroid(&modified, &self.centtab)?;
                let area = compute_area(&modified, &self.sumtab)?;

                scaled_samples.push(modified);
                scaled_centroids.push((cx, cy));
                scaled_areas.push(area);
            }

            self.pixaa.push(scaled_samples);
            self.ptaa.push(scaled_centroids);
            self.naasum.push(scaled_areas);
        }

        // Update scaled size statistics
        self.minwidth = i32::MAX;
        self.maxwidth = 0;
        for samples in &self.pixaa {
            for pix in samples {
                let w = pix.width() as i32;
                self.minwidth = self.minwidth.min(w);
                self.maxwidth = self.maxwidth.max(w);
            }
        }

        // Validate template heights
        let min_h = self.pixa_u.iter().map(|p| p.height()).min().unwrap_or(1) as f32;
        let max_h = self.pixa_u.iter().map(|p| p.height()).max().unwrap_or(1) as f32;
        if min_h > 0.0 && max_h / min_h > self.max_ht_ratio {
            return Err(RecogError::TrainingError(format!(
                "template height ratio {:.2} exceeds maximum {:.2}",
                max_h / min_h,
                self.max_ht_ratio
            )));
        }

        self.train_done = true;
        Ok(())
    }

    /// Removes outliers using method 1 (correlation with class average)
    ///
    /// Templates with correlation score below `min_score` are removed.
    ///
    /// # Arguments
    ///
    /// * `min_score` - Minimum correlation score to keep a template
    ///
    /// # Returns
    ///
    /// Number of outliers removed
    pub fn remove_outliers1(&mut self, min_score: f32) -> RecogResult<usize> {
        if !self.ave_done {
            self.average_samples()?;
        }

        let mut removed = 0;

        for class_idx in 0..self.set_size {
            let avg = &self.pixa_u[class_idx];
            let avg_centroid = self.pta_u[class_idx];

            let mut keep_indices = Vec::new();

            for (sample_idx, pix) in self.pixaa_u[class_idx].iter().enumerate() {
                let centroid = self.ptaa_u[class_idx][sample_idx];
                let score = compute_correlation_with_centering(
                    pix,
                    avg,
                    centroid.0,
                    centroid.1,
                    avg_centroid.0,
                    avg_centroid.1,
                    self.max_y_shift,
                    &self.sumtab,
                )?;

                if score >= min_score {
                    keep_indices.push(sample_idx);
                } else {
                    removed += 1;
                }
            }

            // Rebuild arrays with kept samples
            if keep_indices.len() < self.pixaa_u[class_idx].len() {
                let new_samples: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.pixaa_u[class_idx][i].clone())
                    .collect();
                let new_centroids: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.ptaa_u[class_idx][i])
                    .collect();
                let new_areas: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.naasum_u[class_idx][i])
                    .collect();

                self.pixaa_u[class_idx] = new_samples;
                self.ptaa_u[class_idx] = new_centroids;
                self.naasum_u[class_idx] = new_areas;
            }
        }

        // Recalculate averages after removal
        if removed > 0 {
            self.ave_done = false;
            self.num_samples -= removed;
            self.average_samples()?;
        }

        Ok(removed)
    }

    /// Removes outliers using method 2 (comparison with other classes)
    ///
    /// Templates that have higher correlation with another class's average
    /// than their own class are removed.
    ///
    /// # Returns
    ///
    /// Number of outliers removed
    pub fn remove_outliers2(&mut self) -> RecogResult<usize> {
        if !self.ave_done {
            self.average_samples()?;
        }

        let mut removed = 0;

        for class_idx in 0..self.set_size {
            let own_avg = &self.pixa_u[class_idx];
            let own_centroid = self.pta_u[class_idx];

            let mut keep_indices = Vec::new();

            for (sample_idx, pix) in self.pixaa_u[class_idx].iter().enumerate() {
                let centroid = self.ptaa_u[class_idx][sample_idx];

                // Score with own class
                let own_score = compute_correlation_with_centering(
                    pix,
                    own_avg,
                    centroid.0,
                    centroid.1,
                    own_centroid.0,
                    own_centroid.1,
                    self.max_y_shift,
                    &self.sumtab,
                )?;

                // Check if any other class has higher score
                let mut is_outlier = false;
                for other_idx in 0..self.set_size {
                    if other_idx == class_idx {
                        continue;
                    }

                    let other_avg = &self.pixa_u[other_idx];
                    let other_centroid = self.pta_u[other_idx];

                    let other_score = compute_correlation_with_centering(
                        pix,
                        other_avg,
                        centroid.0,
                        centroid.1,
                        other_centroid.0,
                        other_centroid.1,
                        self.max_y_shift,
                        &self.sumtab,
                    )?;

                    if other_score > own_score {
                        is_outlier = true;
                        break;
                    }
                }

                if !is_outlier {
                    keep_indices.push(sample_idx);
                } else {
                    removed += 1;
                }
            }

            // Rebuild arrays with kept samples
            if keep_indices.len() < self.pixaa_u[class_idx].len() {
                let new_samples: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.pixaa_u[class_idx][i].clone())
                    .collect();
                let new_centroids: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.ptaa_u[class_idx][i])
                    .collect();
                let new_areas: Vec<_> = keep_indices
                    .iter()
                    .map(|&i| self.naasum_u[class_idx][i])
                    .collect();

                self.pixaa_u[class_idx] = new_samples;
                self.ptaa_u[class_idx] = new_centroids;
                self.naasum_u[class_idx] = new_areas;
            }
        }

        // Recalculate averages after removal
        if removed > 0 {
            self.ave_done = false;
            self.num_samples -= removed;
            self.average_samples()?;
        }

        Ok(removed)
    }

    /// Returns the number of training samples for each class
    pub fn get_sample_counts(&self) -> Vec<usize> {
        self.pixaa_u.iter().map(|v| v.len()).collect()
    }

    /// Returns the class labels
    pub fn get_class_labels(&self) -> &[String] {
        &self.sa_text
    }
}

/// Creates a lookup table for centroid calculation
pub fn make_centtab() -> Vec<i32> {
    let mut tab = vec![0i32; 256];
    for (i, entry) in tab.iter_mut().enumerate() {
        let mut count = 0i32;
        for j in 0..8 {
            if (i >> j) & 1 != 0 {
                count += 7 - j;
            }
        }
        *entry = count;
    }
    tab
}

/// Creates a lookup table for pixel sum calculation
pub fn make_sumtab() -> Vec<i32> {
    let mut tab = vec![0i32; 256];
    for (i, entry) in tab.iter_mut().enumerate() {
        *entry = (i as u32).count_ones() as i32;
    }
    tab
}

/// Computes the centroid of a binary image
pub fn compute_centroid(pix: &Pix, _centtab: &[i32]) -> RecogResult<(f32, f32)> {
    let w = pix.width();
    let h = pix.height();

    let mut sum_x = 0i64;
    let mut sum_y = 0i64;
    let mut count = 0i64;

    for y in 0..h {
        for x in 0..w {
            if let Some(val) = pix.get_pixel(x, y)
                && val == 1
            {
                sum_x += x as i64;
                sum_y += y as i64;
                count += 1;
            }
        }
    }

    if count == 0 {
        return Ok((w as f32 / 2.0, h as f32 / 2.0));
    }

    Ok((sum_x as f32 / count as f32, sum_y as f32 / count as f32))
}

/// Computes the foreground area of a binary image
pub fn compute_area(pix: &Pix, _sumtab: &[i32]) -> RecogResult<i32> {
    let w = pix.width();
    let h = pix.height();
    let mut area = 0i32;

    for y in 0..h {
        for x in 0..w {
            if let Some(val) = pix.get_pixel(x, y)
                && val == 1
            {
                area += 1;
            }
        }
    }

    Ok(area)
}

/// Computes correlation score with centroid alignment
#[allow(clippy::too_many_arguments)]
pub fn compute_correlation_with_centering(
    pix1: &Pix,
    pix2: &Pix,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    max_y_shift: i32,
    _sumtab: &[i32],
) -> RecogResult<f32> {
    let mut best_score = 0.0f32;

    for dy in -max_y_shift..=max_y_shift {
        let score = compute_correlation_score_aligned(pix1, pix2, cx1, cy1, cx2, cy2 + dy as f32)?;
        if score > best_score {
            best_score = score;
        }
    }

    Ok(best_score)
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

    // Compute offset to align centroids
    let dx = (cx2 - cx1).round() as i32;
    let dy = (cy2 - cy1).round() as i32;

    let mut and_count = 0i32;
    let mut count1 = 0i32;
    let mut count2 = 0i32;

    // Count foreground pixels and AND overlap
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

    // Correlation score: 2 * AND / (count1 + count2)
    let score = 2.0 * and_count as f32 / (count1 + count2) as f32;
    Ok(score)
}

/// Computes bitwise AND of two binary images
fn and_images(pix1: &Pix, pix2: &Pix) -> RecogResult<Pix> {
    let w = pix1.width().min(pix2.width());
    let h = pix1.height().min(pix2.height());

    let result = Pix::new(w, h, PixelDepth::Bit1).map_err(RecogError::Core)?;
    let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel(x, y).unwrap_or(0);
            let v2 = pix2.get_pixel(x, y).unwrap_or(0);
            if v1 == 1 && v2 == 1 {
                let _ = result_mut.set_pixel(x, y, 1);
            }
        }
    }

    Ok(result_mut.into())
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

    let result = Pix::new(new_w, new_h, PixelDepth::Bit1).map_err(RecogError::Core)?;
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

/// Sets stroke width by skeletonization and dilation
fn set_stroke_width(pix: &Pix, width: u32) -> RecogResult<Pix> {
    // Simple implementation: dilate by width/2
    let half = (width / 2).max(1);
    let dilated = morph_binary::dilate_brick(pix, half, half).map_err(RecogError::Morph)?;
    Ok(dilated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_recog() {
        let recog = create(40, 40, 0, 150, 1).unwrap();
        assert_eq!(recog.scale_w, 40);
        assert_eq!(recog.scale_h, 40);
        assert_eq!(recog.line_w, 0);
        assert_eq!(recog.threshold, 150);
        assert_eq!(recog.max_y_shift, 1);
        assert!(!recog.train_done);
    }

    #[test]
    fn test_create_recog_invalid_y_shift() {
        let result = create(40, 40, 0, 150, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_make_sumtab() {
        let tab = make_sumtab();
        assert_eq!(tab.len(), 256);
        assert_eq!(tab[0], 0);
        assert_eq!(tab[1], 1);
        assert_eq!(tab[255], 8);
        assert_eq!(tab[0b10101010], 4);
    }

    #[test]
    fn test_make_centtab() {
        let tab = make_centtab();
        assert_eq!(tab.len(), 256);
        // For byte with bit 0 set (rightmost), contribution is 7
        assert_eq!(tab[1], 7);
        // For byte with bit 7 set (leftmost), contribution is 0
        assert_eq!(tab[128], 0);
    }

    #[test]
    fn test_compute_centroid() {
        let centtab = make_centtab();
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set a single pixel at (5, 5)
        let _ = pix_mut.set_pixel(5, 5, 1);

        let pix: Pix = pix_mut.into();
        let (cx, cy) = compute_centroid(&pix, &centtab).unwrap();
        assert!((cx - 5.0).abs() < 0.01);
        assert!((cy - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_area() {
        let sumtab = make_sumtab();
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Set 5 pixels
        for i in 0..5 {
            let _ = pix_mut.set_pixel(i, 0, 1);
        }

        let pix: Pix = pix_mut.into();
        let area = compute_area(&pix, &sumtab).unwrap();
        assert_eq!(area, 5);
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
}
