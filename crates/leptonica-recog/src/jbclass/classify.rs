//! JBIG2 classification processing
//!
//! This module implements the classification algorithms for JBIG2-style
//! connected component clustering.

use leptonica_core::{Box as PixBox, Pix, PixelDepth};
use leptonica_morph::binary as morph_binary;
use leptonica_region::{ConnectivityType, find_connected_components};

use crate::error::{RecogError, RecogResult};

use super::types::{
    DEFAULT_MAX_HEIGHT, DEFAULT_MAX_WIDTH, DEFAULT_SIZE_HAUS, DEFAULT_THRESH, JbClasser,
    JbComponent, JbData, JbMethod, TEMPLATE_BORDER,
};

/// Maximum difference in width for matching
const MAX_DIFF_WIDTH: i32 = 2;

/// Maximum difference in height for matching
const MAX_DIFF_HEIGHT: i32 = 2;

/// Creates a rank Hausdorff distance classifier
///
/// # Arguments
///
/// * `components` - Type of components to extract (ConnComps, Characters, Words)
/// * `max_width` - Maximum component width allowed (0 for default)
/// * `max_height` - Maximum component height allowed (0 for default)
/// * `size_haus` - Size of structuring element for Hausdorff (typically 2)
/// * `rank_haus` - Rank value for Hausdorff matching (0.97 for good results)
///
/// # Returns
///
/// A new JbClasser configured for rank Hausdorff classification
pub fn rank_haus_init(
    components: JbComponent,
    max_width: i32,
    max_height: i32,
    size_haus: i32,
    rank_haus: f32,
) -> RecogResult<JbClasser> {
    if !(1..=10).contains(&size_haus) {
        return Err(RecogError::InvalidParameter(
            "size_haus must be between 1 and 10".to_string(),
        ));
    }
    if !(0.5..=1.0).contains(&rank_haus) {
        return Err(RecogError::InvalidParameter(
            "rank_haus must be between 0.5 and 1.0".to_string(),
        ));
    }

    let mut classer = JbClasser::new(JbMethod::RankHaus, components);
    classer.max_width = if max_width > 0 {
        max_width
    } else {
        DEFAULT_MAX_WIDTH
    };
    classer.max_height = if max_height > 0 {
        max_height
    } else {
        DEFAULT_MAX_HEIGHT
    };
    classer.size_haus = if size_haus > 0 {
        size_haus
    } else {
        DEFAULT_SIZE_HAUS
    };
    classer.rank_haus = rank_haus;

    Ok(classer)
}

/// Creates a correlation-based classifier
///
/// # Arguments
///
/// * `components` - Type of components to extract (ConnComps, Characters, Words)
/// * `max_width` - Maximum component width allowed (0 for default)
/// * `max_height` - Maximum component height allowed (0 for default)
/// * `thresh` - Correlation threshold (typically 0.85)
/// * `weight_factor` - Weight factor for heavy text correction (typically 0.7)
///
/// # Returns
///
/// A new JbClasser configured for correlation-based classification
pub fn correlation_init(
    components: JbComponent,
    max_width: i32,
    max_height: i32,
    thresh: f32,
    weight_factor: f32,
) -> RecogResult<JbClasser> {
    if !(0.4..=1.0).contains(&thresh) {
        return Err(RecogError::InvalidParameter(
            "thresh must be between 0.4 and 1.0".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&weight_factor) {
        return Err(RecogError::InvalidParameter(
            "weight_factor must be between 0.0 and 1.0".to_string(),
        ));
    }

    let mut classer = JbClasser::new(JbMethod::Correlation, components);
    classer.max_width = if max_width > 0 {
        max_width
    } else {
        DEFAULT_MAX_WIDTH
    };
    classer.max_height = if max_height > 0 {
        max_height
    } else {
        DEFAULT_MAX_HEIGHT
    };
    classer.thresh = if thresh > 0.0 { thresh } else { DEFAULT_THRESH };
    classer.weight_factor = weight_factor;

    Ok(classer)
}

impl JbClasser {
    /// Adds a single page to the classifier
    ///
    /// # Arguments
    ///
    /// * `pix` - Binary image of the page
    pub fn add_page(&mut self, pix: &Pix) -> RecogResult<()> {
        if pix.depth() != PixelDepth::Bit1 {
            return Err(RecogError::UnsupportedDepth {
                expected: "1 bpp",
                actual: pix.depth() as u32,
            });
        }

        // Update page dimensions
        let w = pix.width() as i32;
        let h = pix.height() as i32;
        self.w = self.w.max(w);
        self.h = self.h.max(h);

        // Get components from the page
        let (components, boxes) = self.get_components(pix)?;

        let num_comps = components.len();
        self.nacomps.push(num_comps);

        // Classify each component
        for (comp, pix_box) in components.iter().zip(boxes.iter()) {
            let class_idx = match self.method {
                JbMethod::RankHaus => self.classify_rank_haus(comp)?,
                JbMethod::Correlation => self.classify_correlation(comp)?,
            };

            // Store classification results
            self.naclass.push(class_idx);
            self.napage.push(self.npages);
            self.ptaul.push((pix_box.x, pix_box.y));
            self.ptall.push((pix_box.x, pix_box.y + pix_box.h));

            // Compute and store centroid
            let (cx, cy) = compute_centroid(comp)?;
            self.ptac.push((cx, cy));
        }

        self.base_index += num_comps;
        self.npages += 1;

        Ok(())
    }

    /// Adds multiple pages to the classifier
    ///
    /// # Arguments
    ///
    /// * `pixs` - Array of binary page images
    pub fn add_pages(&mut self, pixs: &[Pix]) -> RecogResult<()> {
        for pix in pixs {
            self.add_page(pix)?;
        }
        Ok(())
    }

    /// Extracts components from a page based on component type
    pub fn get_components(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        match self.components {
            JbComponent::ConnComps => self.get_conn_comps(pix),
            JbComponent::Characters => self.get_characters(pix),
            JbComponent::Words => self.get_words(pix),
        }
    }

    /// Extracts connected components
    fn get_conn_comps(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        // Get 8-connected components
        let conn_comps = find_connected_components(pix, ConnectivityType::EightWay)
            .map_err(RecogError::Region)?;

        let mut components = Vec::new();
        let mut valid_boxes = Vec::new();

        for cc in conn_comps {
            let bounds = cc.bounds;
            // Filter by size
            if bounds.w > self.max_width || bounds.h > self.max_height {
                continue;
            }

            // Extract component
            if let Ok(comp) = extract_rect(pix, &bounds) {
                components.push(comp);
                valid_boxes.push(bounds);
            }
        }

        Ok((components, valid_boxes))
    }

    /// Extracts character-level components (filtered connected components)
    fn get_characters(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        let (comps, boxes) = self.get_conn_comps(pix)?;

        // Additional filtering for characters
        let mut chars = Vec::new();
        let mut char_boxes = Vec::new();

        for (comp, pix_box) in comps.into_iter().zip(boxes.into_iter()) {
            // Filter out very small or very large components
            let area = count_fg_pixels(&comp)?;
            let box_area = pix_box.w * pix_box.h;
            let fill_factor = area as f32 / box_area.max(1) as f32;

            // Characters typically have fill factor > 0.1
            if fill_factor > 0.1 && pix_box.h >= 5 {
                chars.push(comp);
                char_boxes.push(pix_box);
            }
        }

        Ok((chars, char_boxes))
    }

    /// Extracts word-level components (grouped connected components)
    fn get_words(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        // Close horizontally to group characters into words
        let closed = morph_binary::close_brick(pix, 20, 1).map_err(RecogError::Morph)?;

        // Get word boxes
        let word_comps = find_connected_components(&closed, ConnectivityType::EightWay)
            .map_err(RecogError::Region)?;

        let mut words = Vec::new();
        let mut valid_boxes = Vec::new();

        for cc in word_comps {
            let bounds = cc.bounds;
            if bounds.w > self.max_width || bounds.h > self.max_height {
                continue;
            }

            // Extract word from original (not closed) image
            if let Ok(word) = extract_rect(pix, &bounds) {
                words.push(word);
                valid_boxes.push(bounds);
            }
        }

        Ok((words, valid_boxes))
    }

    /// Classifies a component using rank Hausdorff distance
    pub fn classify_rank_haus(&mut self, pix: &Pix) -> RecogResult<usize> {
        let w = pix.width() as i32;
        let h = pix.height() as i32;

        // Add border for processing
        let bordered = add_border(pix, TEMPLATE_BORDER as u32)?;

        // Dilate for Hausdorff matching
        let dilated =
            morph_binary::dilate_brick(&bordered, self.size_haus as u32, self.size_haus as u32)
                .map_err(RecogError::Morph)?;

        // Look for matching template
        for (class_idx, template) in self.pixat.iter().enumerate() {
            let tw = template.width() as i32 - 2 * TEMPLATE_BORDER;
            let th = template.height() as i32 - 2 * TEMPLATE_BORDER;

            // Size must be similar
            if (tw - w).abs() > MAX_DIFF_WIDTH || (th - h).abs() > MAX_DIFF_HEIGHT {
                continue;
            }

            // Check Hausdorff match
            let template_dilated = &self.pixatd[class_idx];
            if hausdorff_match(
                &bordered,
                &dilated,
                template,
                template_dilated,
                self.rank_haus,
            )? {
                // Match found - add to existing class
                if self.keep_pixaa {
                    self.pixaa[class_idx].push(pix.clone());
                }
                return Ok(class_idx);
            }
        }

        // No match - create new class
        let class_idx = self.nclass;
        self.nclass += 1;

        // Store template
        self.pixat.push(bordered.clone());
        self.pixatd.push(dilated);
        self.naarea.push(w * h);

        // Compute and store template centroid
        let (cx, cy) = compute_centroid(&bordered)?;
        self.ptact.push((cx, cy));

        // Compute foreground area for rank < 1.0
        if self.rank_haus < 1.0 {
            let fg = count_fg_pixels(&bordered)?;
            self.nafgt.push(fg);
        }

        // Add to hash table
        let key = (w, h);
        self.dahash.entry(key).or_default().push(class_idx);

        if self.keep_pixaa {
            self.pixaa.push(vec![pix.clone()]);
        }

        Ok(class_idx)
    }

    /// Classifies a component using correlation
    pub fn classify_correlation(&mut self, pix: &Pix) -> RecogResult<usize> {
        let w = pix.width() as i32;
        let h = pix.height() as i32;

        // Add border for alignment
        let bordered = add_border(pix, TEMPLATE_BORDER as u32)?;

        let pix_area = count_fg_pixels(&bordered)?;
        let pix_centroid = compute_centroid(&bordered)?;

        // Compute effective threshold based on fill factor
        let fill_factor =
            pix_area as f32 / ((w + 2 * TEMPLATE_BORDER) * (h + 2 * TEMPLATE_BORDER)) as f32;
        let effective_thresh = self.thresh + self.weight_factor * fill_factor;

        // Look for matching template
        for (class_idx, template) in self.pixat.iter().enumerate() {
            let tw = template.width() as i32 - 2 * TEMPLATE_BORDER;
            let th = template.height() as i32 - 2 * TEMPLATE_BORDER;

            // Size must be similar
            if (tw - w).abs() > MAX_DIFF_WIDTH || (th - h).abs() > MAX_DIFF_HEIGHT {
                continue;
            }

            let templ_area = self.naarea[class_idx];
            let templ_centroid = self.ptact[class_idx];

            // Compute correlation score
            let score = correlation_score_aligned(
                &bordered,
                template,
                pix_centroid,
                templ_centroid,
                pix_area,
                templ_area,
            )?;

            if score >= effective_thresh {
                // Match found
                if self.keep_pixaa {
                    self.pixaa[class_idx].push(pix.clone());
                }
                return Ok(class_idx);
            }
        }

        // No match - create new class
        let class_idx = self.nclass;
        self.nclass += 1;

        // Store template
        self.pixat.push(bordered);
        self.pixatd.push(Pix::new(1, 1, PixelDepth::Bit1).unwrap()); // Placeholder
        self.naarea.push(pix_area);
        self.ptact.push(pix_centroid);

        // Add to hash table
        let key = (w, h);
        self.dahash.entry(key).or_default().push(class_idx);

        if self.keep_pixaa {
            self.pixaa.push(vec![pix.clone()]);
        }

        Ok(class_idx)
    }

    /// Generates JbData from the classifier
    pub fn get_data(&self) -> RecogResult<JbData> {
        if self.nclass == 0 {
            return Err(RecogError::ClassificationError(
                "no classes have been created".to_string(),
            ));
        }

        // Find lattice dimensions
        let lattice_w = self.pixat.iter().map(|p| p.width()).max().unwrap_or(1) as i32;
        let lattice_h = self.pixat.iter().map(|p| p.height()).max().unwrap_or(1) as i32;

        // Create composite template image
        let composite = self.templates_to_composite(lattice_w as u32, lattice_h as u32)?;

        Ok(JbData::from_classer(self, composite, lattice_w, lattice_h))
    }

    /// Creates a composite image of all templates
    fn templates_to_composite(&self, lattice_w: u32, lattice_h: u32) -> RecogResult<Pix> {
        let n = self.nclass;
        let cols = ((n as f32).sqrt().ceil() as usize).max(1);
        let rows = n.div_ceil(cols);

        let width = cols as u32 * lattice_w;
        let height = rows as u32 * lattice_h;

        let composite = Pix::new(width, height, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut composite_mut = composite.try_into_mut().unwrap_or_else(|p| p.to_mut());

        for (i, template) in self.pixat.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = col as u32 * lattice_w;
            let y = row as u32 * lattice_h;

            copy_to(&mut composite_mut, template, x as i32, y as i32)?;
        }

        Ok(composite_mut.into())
    }

    /// Creates templates from composite grayscale images
    pub fn templates_from_composites(&self) -> RecogResult<Vec<Pix>> {
        // For each class, create an averaged template from instances
        let mut templates = Vec::with_capacity(self.nclass);

        for class_idx in 0..self.nclass {
            if class_idx < self.pixaa.len() && !self.pixaa[class_idx].is_empty() {
                let instances = &self.pixaa[class_idx];

                // Find max dimensions
                let max_w = instances.iter().map(|p| p.width()).max().unwrap_or(1);
                let max_h = instances.iter().map(|p| p.height()).max().unwrap_or(1);

                // Create averaged template
                let mut accum = vec![0u32; (max_w * max_h) as usize];
                let threshold = instances.len() as u32 / 2;

                for inst in instances {
                    for y in 0..inst.height().min(max_h) {
                        for x in 0..inst.width().min(max_w) {
                            if let Some(val) = inst.get_pixel(x, y)
                                && val == 1
                            {
                                accum[(y * max_w + x) as usize] += 1;
                            }
                        }
                    }
                }

                let template =
                    Pix::new(max_w, max_h, PixelDepth::Bit1).map_err(RecogError::Core)?;
                let mut template_mut = template.try_into_mut().unwrap_or_else(|p| p.to_mut());

                for y in 0..max_h {
                    for x in 0..max_w {
                        if accum[(y * max_w + x) as usize] > threshold {
                            let _ = template_mut.set_pixel(x, y, 1);
                        }
                    }
                }

                templates.push(template_mut.into());
            } else {
                // Use stored template
                templates.push(self.pixat[class_idx].clone());
            }
        }

        Ok(templates)
    }
}

impl JbData {
    /// Renders a single page from the compressed data
    ///
    /// # Arguments
    ///
    /// * `page` - Page number to render
    ///
    /// # Returns
    ///
    /// Reconstructed page image
    pub fn render_page(&self, page: usize) -> RecogResult<Pix> {
        if page >= self.npages {
            return Err(RecogError::InvalidParameter(format!(
                "page {} out of range (max {})",
                page,
                self.npages - 1
            )));
        }

        let result =
            Pix::new(self.w as u32, self.h as u32, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

        // Extract templates from composite
        let templates = self.extract_templates()?;

        // Place each component on the page
        for i in 0..self.naclass.len() {
            if self.napage[i] != page {
                continue;
            }

            let class_idx = self.naclass[i];
            if class_idx >= templates.len() {
                continue;
            }

            let template = &templates[class_idx];
            let (x, y) = self.ptaul[i];

            copy_to(&mut result_mut, template, x, y)?;
        }

        Ok(result_mut.into())
    }

    /// Renders all pages from the compressed data
    ///
    /// # Returns
    ///
    /// Array of reconstructed page images
    pub fn render_all(&self) -> RecogResult<Vec<Pix>> {
        let mut pages = Vec::with_capacity(self.npages);

        for page in 0..self.npages {
            pages.push(self.render_page(page)?);
        }

        Ok(pages)
    }

    /// Extracts individual templates from the composite image
    fn extract_templates(&self) -> RecogResult<Vec<Pix>> {
        let cols = ((self.nclass as f32).sqrt().ceil() as usize).max(1);

        let mut templates = Vec::with_capacity(self.nclass);

        for i in 0..self.nclass {
            let col = i % cols;
            let row = i / cols;
            let x = (col as i32) * self.lattice_w;
            let y = (row as i32) * self.lattice_h;

            let pix_box =
                PixBox::new(x, y, self.lattice_w, self.lattice_h).map_err(RecogError::Core)?;

            templates.push(extract_rect(&self.pix, &pix_box)?);
        }

        Ok(templates)
    }
}

/// Computes Hausdorff distance match
pub fn hausdorff_distance(pix1: &Pix, pix2: &Pix, size: i32, rank: f32) -> RecogResult<bool> {
    // Dilate both images
    let dil1 =
        morph_binary::dilate_brick(pix1, size as u32, size as u32).map_err(RecogError::Morph)?;
    let dil2 =
        morph_binary::dilate_brick(pix2, size as u32, size as u32).map_err(RecogError::Morph)?;

    hausdorff_match(pix1, &dil1, pix2, &dil2, rank)
}

/// Checks if two images match using Hausdorff criterion
fn hausdorff_match(pix1: &Pix, dil1: &Pix, pix2: &Pix, dil2: &Pix, rank: f32) -> RecogResult<bool> {
    // Forward direction: pix1 fg must be covered by dil2
    let fg1 = count_fg_pixels(pix1)?;
    let covered1 = count_and_pixels(pix1, dil2)?;
    let ratio1 = covered1 as f32 / fg1.max(1) as f32;

    if ratio1 < rank {
        return Ok(false);
    }

    // Reverse direction: pix2 fg must be covered by dil1
    let fg2 = count_fg_pixels(pix2)?;
    let covered2 = count_and_pixels(pix2, dil1)?;
    let ratio2 = covered2 as f32 / fg2.max(1) as f32;

    Ok(ratio2 >= rank)
}

/// Computes correlation score for aligned images
fn correlation_score_aligned(
    pix1: &Pix,
    pix2: &Pix,
    centroid1: (f32, f32),
    centroid2: (f32, f32),
    area1: i32,
    area2: i32,
) -> RecogResult<f32> {
    let w1 = pix1.width() as i32;
    let h1 = pix1.height() as i32;
    let w2 = pix2.width() as i32;
    let h2 = pix2.height() as i32;

    // Compute offset to align centroids
    let dx = (centroid2.0 - centroid1.0).round() as i32;
    let dy = (centroid2.1 - centroid1.1).round() as i32;

    let mut and_count = 0i32;

    for y1 in 0..h1 {
        for x1 in 0..w1 {
            if let Some(v1) = pix1.get_pixel(x1 as u32, y1 as u32)
                && v1 == 1
            {
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

    // Correlation score: and_count^2 / (area1 * area2)
    let product = (area1 as i64 * area2 as i64).max(1) as f32;
    Ok((and_count as f32 * and_count as f32) / product)
}

/// Helper: Extracts a rectangular region from an image
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

/// Helper: Adds a border around an image
fn add_border(pix: &Pix, border: u32) -> RecogResult<Pix> {
    let new_w = pix.width() + 2 * border;
    let new_h = pix.height() + 2 * border;

    let result = Pix::new(new_w, new_h, pix.depth()).map_err(RecogError::Core)?;
    let mut result_mut = result.try_into_mut().unwrap_or_else(|p| p.to_mut());

    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y) {
                let _ = result_mut.set_pixel(x + border, y + border, val);
            }
        }
    }

    Ok(result_mut.into())
}

/// Helper: Copies source image to destination at specified position
fn copy_to(dst: &mut leptonica_core::PixMut, src: &Pix, x: i32, y: i32) -> RecogResult<()> {
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

/// Helper: Computes the centroid of foreground pixels
fn compute_centroid(pix: &Pix) -> RecogResult<(f32, f32)> {
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
        Ok((w as f32 / 2.0, h as f32 / 2.0))
    } else {
        Ok((sum_x as f32 / count as f32, sum_y as f32 / count as f32))
    }
}

/// Helper: Counts foreground pixels in an image
fn count_fg_pixels(pix: &Pix) -> RecogResult<i32> {
    let mut count = 0i32;
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if let Some(val) = pix.get_pixel(x, y)
                && val == 1
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn count_and_pixels(pix1: &Pix, pix2: &Pix) -> RecogResult<i32> {
    let w = pix1.width().min(pix2.width());
    let h = pix1.height().min(pix2.height());
    let mut count = 0i32;

    for y in 0..h {
        for x in 0..w {
            let v1 = pix1.get_pixel(x, y).unwrap_or(0);
            let v2 = pix2.get_pixel(x, y).unwrap_or(0);
            if v1 == 1 && v2 == 1 {
                count += 1;
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_haus_init() {
        let classer = rank_haus_init(JbComponent::ConnComps, 150, 150, 2, 0.97).unwrap();
        assert_eq!(classer.method, JbMethod::RankHaus);
        assert_eq!(classer.components, JbComponent::ConnComps);
        assert_eq!(classer.size_haus, 2);
        assert!((classer.rank_haus - 0.97).abs() < 0.001);
    }

    #[test]
    fn test_rank_haus_init_invalid_size() {
        let result = rank_haus_init(JbComponent::ConnComps, 150, 150, 0, 0.97);
        assert!(result.is_err());

        let result = rank_haus_init(JbComponent::ConnComps, 150, 150, 11, 0.97);
        assert!(result.is_err());
    }

    #[test]
    fn test_correlation_init() {
        let classer = correlation_init(JbComponent::Characters, 150, 150, 0.85, 0.7).unwrap();
        assert_eq!(classer.method, JbMethod::Correlation);
        assert_eq!(classer.components, JbComponent::Characters);
        assert!((classer.thresh - 0.85).abs() < 0.001);
        assert!((classer.weight_factor - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_add_border() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let bordered = add_border(&pix, 4).unwrap();
        assert_eq!(bordered.width(), 18);
        assert_eq!(bordered.height(), 18);
    }

    #[test]
    fn test_compute_centroid() {
        let pix_blank = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix_blank.try_into_mut().unwrap_or_else(|p| p.to_mut());
        let _ = pix_mut.set_pixel(5, 5, 1);
        let pix: Pix = pix_mut.into();

        let (cx, cy) = compute_centroid(&pix).unwrap();
        assert!((cx - 5.0).abs() < 0.01);
        assert!((cy - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_count_fg_pixels() {
        let pix_blank = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let mut pix_mut = pix_blank.try_into_mut().unwrap_or_else(|p| p.to_mut());
        for i in 0..5 {
            let _ = pix_mut.set_pixel(i, 0, 1);
        }
        let pix: Pix = pix_mut.into();

        let count = count_fg_pixels(&pix).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_get_data_empty() {
        let classer = JbClasser::new(JbMethod::RankHaus, JbComponent::ConnComps);
        let result = classer.get_data();
        assert!(result.is_err());
    }
}
