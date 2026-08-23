//! JBIG2 classification processing
//!
//! This module implements the classification algorithms for JBIG2-style
//! connected component clustering.

use crate::core::{Box as PixBox, Boxa, Pix, PixelDepth};
use crate::morph::binary as morph_binary;
use crate::region::{ConnectivityType, conncomp_pixa, find_connected_components};

use crate::recog::error::{RecogError, RecogResult};

use super::types::{
    DEFAULT_MAX_HEIGHT, DEFAULT_MAX_WIDTH, DEFAULT_MAX_WORD_WIDTH, DEFAULT_SIZE_HAUS,
    DEFAULT_THRESH, JbClasser, JbComponent, JbData, JbMethod, TEMPLATE_BORDER,
};

/// Generates a word mask by progressive dilation.
///
/// Dilates `pix` horizontally in steps of 1, stopping when successive steps
/// produce the same count of connected components.  Returns the mask at that
/// dilation level and the dilation size used.
///
/// The algorithm mirrors Leptonica's `pixWordMaskByDilation`:
/// - Dilate by 1 pixel at a time up to `max_dil`
/// - Stop when the component count stabilises (delta == 0)
///
/// # Arguments
///
/// * `pix`     - Input binary image (1 bpp)
/// * `max_dil` - Maximum horizontal dilation to attempt (clamped to ≥ 1)
///
/// # Errors
///
/// Returns an error if `pix` is not 1 bpp or morphological operations fail.
pub fn pix_word_mask_by_dilation(pix: &Pix, max_dil: u32) -> RecogResult<(Pix, u32)> {
    let max_dil = max_dil.max(1);
    let mut best = pix.clone();
    let mut best_size = 0u32;
    let mut prev_count = find_connected_components(pix, ConnectivityType::FourWay)?.len() as u32;
    // If there are no components, no dilation is needed; return the original image.
    if prev_count == 0 {
        return Ok((best, best_size));
    }
    for size in 1..=max_dil {
        let dil_w = 2 * size + 1;
        let dilated = morph_binary::dilate_brick(pix, dil_w, 1)?;
        let count = find_connected_components(&dilated, ConnectivityType::FourWay)?.len() as u32;
        if count >= prev_count {
            break;
        }
        prev_count = count;
        best = dilated;
        best_size = size;
    }
    Ok((best, best_size))
}

/// Detects word bounding boxes by progressive dilation.
///
/// Calls [`pix_word_mask_by_dilation`] internally and returns the bounding boxes
/// of each connected component in the resulting word mask.
///
/// # Arguments
///
/// * `pix`     - Input binary image (1 bpp)
/// * `max_dil` - Maximum horizontal dilation to attempt
///
/// # Errors
///
/// Returns an error if `pix` is not 1 bpp or component detection fails.
pub fn pix_word_boxes_by_dilation(pix: &Pix, max_dil: u32) -> RecogResult<Boxa> {
    let (mask, _) = pix_word_mask_by_dilation(pix, max_dil)?;
    let comps = find_connected_components(&mask, ConnectivityType::FourWay)?;
    let mut boxa = Boxa::with_capacity(comps.len());
    for comp in comps {
        boxa.push(PixBox::new_unchecked(
            comp.bounds.x,
            comp.bounds.y,
            comp.bounds.w,
            comp.bounds.h,
        ));
    }
    Ok(boxa)
}

/// Maximum difference in width for matching
const MAX_DIFF_WIDTH: i32 = 2;

/// Maximum difference in height for matching
const MAX_DIFF_HEIGHT: i32 = 2;

/// Order in which candidate template sizes are visited, as `(dw, dh)` offsets
/// from the instance size.
///
/// Both classifiers take the *first* template that matches, so this order is
/// part of the result: it starts at the exact size and spirals outwards, so a
/// same-sized template always wins over a near-sized one.
///
/// C Leptonica: `two_by_two_walk` in `jbclass.c`
const TWO_BY_TWO_WALK: [(i32, i32); 25] = [
    (0, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 0),
    (-1, 1),
    (1, 1),
    (-1, -1),
    (1, -1),
    (0, -2),
    (2, 0),
    (0, 2),
    (-2, 0),
    (-1, -2),
    (1, -2),
    (2, -1),
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (-2, -2),
    (2, -2),
    (2, 2),
    (-2, 2),
];

/// Default maximum component width for a component type.
///
/// Words are allowed to be much wider than single characters.
///
/// C Leptonica: the `maxwidth == 0` branch of `jbCorrelationInitInternal()`
/// and `jbRankHausInit()` in `jbclass.c`
fn default_max_width(components: JbComponent) -> i32 {
    match components {
        JbComponent::ConnComps | JbComponent::Characters => DEFAULT_MAX_WIDTH,
        JbComponent::Words => DEFAULT_MAX_WORD_WIDTH,
    }
}

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
        default_max_width(components)
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
    // C keeps every instance so callers can inspect or improve the templates.
    classer.keep_pixaa = true;

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
        default_max_width(components)
    };
    classer.max_height = if max_height > 0 {
        max_height
    } else {
        DEFAULT_MAX_HEIGHT
    };
    classer.thresh = if thresh > 0.0 { thresh } else { DEFAULT_THRESH };
    classer.weight_factor = weight_factor;
    // C keeps every instance so callers can inspect or improve the templates.
    classer.keep_pixaa = true;

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

        // C stores the most recent page's size, not the largest.
        self.w = pix.width() as i32;
        self.h = pix.height() as i32;

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

            self.naclass.push(class_idx);
            self.napage.push(self.npages);
            self.ptac.push(instance_centroid(&add_border(
                comp,
                TEMPLATE_BORDER as u32,
            )?)?);

            let (ulx, uly) = self.ul_corner(pix, class_idx, self.ptac.len() - 1, pix_box)?;
            self.ptaul.push((ulx, uly));
            self.ptall.push((ulx, uly + pix_box.h));
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

    /// Where to draw this component's template so that the template and the
    /// instance line up.
    ///
    /// The centroid difference gives a first guess; the final pixel is chosen
    /// by trying all nine one-pixel shifts and keeping the one that differs
    /// least from the page.
    ///
    /// # See also
    ///
    /// C Leptonica: `jbGetULCorners()` in `jbclass.c`
    fn ul_corner(
        &self,
        pixs: &Pix,
        class_idx: usize,
        comp_idx: usize,
        bounds: &PixBox,
    ) -> RecogResult<(i32, i32)> {
        let (x1, y1) = self.ptac[comp_idx];
        let (x2, y2) = *self.ptact.get(class_idx).ok_or_else(|| {
            RecogError::ClassificationError(format!("class {class_idx} has no centroid"))
        })?;
        // C rounds away from zero, which `f32::round` also does.
        let idelx = (x2 - x1).round() as i32;
        let idely = (y2 - y1).round() as i32;

        let template = self.pixat.get(class_idx).ok_or_else(|| {
            RecogError::ClassificationError(format!("class {class_idx} has no template"))
        })?;
        let (dx, dy) =
            final_positioning_for_alignment(pixs, bounds.x, bounds.y, idelx, idely, template);

        Ok((bounds.x - idelx + dx, bounds.y - idely + dy))
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
    ///
    /// Each component is its own bitmap, holding only that component's
    /// pixels. Cropping the page to the bounding box instead would drag in
    /// whatever neighbouring components overlap that rectangle, which changes
    /// the templates and therefore the classification.
    fn get_conn_comps(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        let (boxa, pixa) =
            conncomp_pixa(pix, ConnectivityType::EightWay).map_err(RecogError::Region)?;

        let mut components = Vec::new();
        let mut valid_boxes = Vec::new();

        for i in 0..boxa.len() {
            let Some(bounds) = boxa.get(i).copied() else {
                continue;
            };
            // Filter by size
            if bounds.w > self.max_width || bounds.h > self.max_height {
                continue;
            }
            let Some(comp) = pixa.get(i) else {
                continue;
            };
            components.push(comp.clone());
            valid_boxes.push(bounds);
        }

        Ok((components, valid_boxes))
    }

    /// Extracts character-level components (filtered connected components)
    fn get_characters(&self, pix: &Pix) -> RecogResult<(Vec<Pix>, Vec<PixBox>)> {
        let (comps, boxes) = self.get_conn_comps(pix)?;

        // Additional filtering for characters
        let mut chars = Vec::new();
        let mut char_boxes = Vec::new();

        for (comp, pix_box) in comps.into_iter().zip(boxes) {
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
        let key = (w, h);

        // Add border for processing
        let bordered = add_border(pix, TEMPLATE_BORDER as u32)?;

        // Dilate for Hausdorff matching
        let dilated =
            morph_binary::dilate_brick(&bordered, self.size_haus as u32, self.size_haus as u32)
                .map_err(RecogError::Morph)?;
        // C counts fg on the unbordered component; the border is white so the
        // count is the same, but keep the source explicit.
        let inst_area = if self.rank_haus < 1.0 {
            Some(count_fg_pixels(pix)?)
        } else {
            None
        };
        let inst_centroid = compute_centroid(&bordered)?;

        // Same greedy walk as the correlation classifier: nearest sizes first.
        for (dw, dh) in TWO_BY_TWO_WALK {
            let (cw, ch) = (key.0 + dw, key.1 + dh);
            if cw < 1 || ch < 1 {
                continue;
            }
            let Some(candidates) = self.dahash.get(&(cw, ch)) else {
                continue;
            };
            for &class_idx in candidates {
                if class_idx >= self.pixat.len() || class_idx >= self.pixatd.len() {
                    continue;
                }
                if class_idx >= self.ptact.len() {
                    continue;
                }
                let template = &self.pixat[class_idx];
                let template_dilated = &self.pixatd[class_idx];
                let template_area = if self.rank_haus < 1.0 {
                    self.nafgt.get(class_idx).copied()
                } else {
                    None
                };
                let templ_centroid = self.ptact[class_idx];
                if hausdorff_match_with_areas(
                    &bordered,
                    &dilated,
                    template,
                    template_dilated,
                    self.rank_haus,
                    (
                        inst_centroid.0 - templ_centroid.0,
                        inst_centroid.1 - templ_centroid.1,
                    ),
                    inst_area,
                    template_area,
                )? {
                    if self.keep_pixaa {
                        self.pixaa[class_idx].push(pix.clone());
                    }
                    return Ok(class_idx);
                }
            }
        }

        // No match - create new class
        let class_idx = self.nclass;
        self.nclass += 1;

        // Store template
        self.pixat.push(bordered.clone());
        self.pixatd.push(dilated);
        self.naarea.push(w * h);

        self.ptact.push(inst_centroid);

        // Only consulted below rank 1.0, but keep the index aligned with
        // `pixat` either way.
        self.nafgt.push(inst_area.unwrap_or(count_fg_pixels(pix)?));

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
        let key = (w, h);

        // Add border for alignment
        let bordered = add_border(pix, TEMPLATE_BORDER as u32)?;

        let pix_area = count_fg_pixels(&bordered)?;
        let pix_centroid = compute_centroid(&bordered)?;

        // Take the first template whose score clears its own threshold. The
        // threshold rises with how heavy the template is, so that thick
        // characters are not merged as readily as thin ones.
        for (dw, dh) in TWO_BY_TWO_WALK {
            let (cw, ch) = (key.0 + dw, key.1 + dh);
            if cw < 1 || ch < 1 {
                continue;
            }
            let Some(candidates) = self.dahash.get(&(cw, ch)) else {
                continue;
            };
            for &class_idx in candidates {
                if class_idx >= self.pixat.len()
                    || class_idx >= self.ptact.len()
                    || class_idx >= self.naarea.len()
                    || class_idx >= self.nafgt.len()
                {
                    continue;
                }
                let template = &self.pixat[class_idx];
                let templ_fg = self.nafgt[class_idx];
                let templ_box_area = self.naarea[class_idx];
                let templ_centroid = self.ptact[class_idx];

                let threshold = if self.weight_factor > 0.0 {
                    self.thresh
                        + (1.0 - self.thresh) * self.weight_factor * templ_fg as f32
                            / templ_box_area.max(1) as f32
                } else {
                    self.thresh
                };

                let score = correlation_score_aligned(
                    &bordered,
                    template,
                    pix_centroid,
                    templ_centroid,
                    pix_area,
                    templ_fg,
                )?;

                if score >= threshold {
                    if self.keep_pixaa {
                        self.pixaa[class_idx].push(pix.clone());
                    }
                    return Ok(class_idx);
                }
            }
        }

        // No match - create new class
        let class_idx = self.nclass;
        self.nclass += 1;

        // Store template. `nafgt` is the fg count of the bordered template and
        // `naarea` the area of its unbordered bounding box; the threshold
        // above is their ratio.
        self.pixat.push(bordered);
        self.pixatd.push(Pix::new(1, 1, PixelDepth::Bit1).unwrap()); // unused for correlation
        self.nafgt.push(pix_area);
        self.naarea.push(w * h);
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

        // One pixel larger than the biggest template, so neighbouring cells
        // never touch.
        let lattice_w = self.pixat.iter().map(|p| p.width()).max().unwrap_or(0) as i32 + 1;
        let lattice_h = self.pixat.iter().map(|p| p.height()).max().unwrap_or(0) as i32 + 1;

        // Create composite template image
        let composite = self.templates_to_composite(lattice_w as u32, lattice_h as u32)?;

        Ok(JbData::from_classer(self, composite, lattice_w, lattice_h))
    }

    /// Tiles every template onto a lattice, row by row.
    ///
    /// The grid is roughly square, with `floor(sqrt(n))` columns. A template
    /// too big for its cell is skipped rather than clipped, as in C.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixaDisplayOnLattice()` in `pixafunc2.c`
    fn templates_to_composite(&self, lattice_w: u32, lattice_h: u32) -> RecogResult<Pix> {
        let n = self.nclass;
        let cols = ((n as f64).sqrt() as usize).max(1);
        let rows = n.div_ceil(cols);

        let width = cols as u32 * lattice_w;
        let height = rows as u32 * lattice_h;

        let composite = Pix::new(width, height, PixelDepth::Bit1).map_err(RecogError::Core)?;
        let mut composite_mut = composite.try_into_mut().unwrap_or_else(|p| p.to_mut());

        for (i, template) in self.pixat.iter().enumerate() {
            if template.width() > lattice_w || template.height() > lattice_h {
                continue; // C logs and omits it
            }
            let x = (i % cols) as u32 * lattice_w;
            let y = (i / cols) as u32 * lattice_h;
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
                            if inst.get_pixel_unchecked(x, y) == 1 {
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
                            template_mut.set_pixel_unchecked(x, y, 1);
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
        // Derive the column count from the composite itself rather than
        // recomputing the layout, so this cannot drift from whatever produced
        // the image. C does the same in `pixaCreateFromPix()`.
        let cols = self
            .pix
            .width()
            .div_ceil((self.lattice_w.max(1)) as u32)
            .max(1) as usize;

        let mut templates = Vec::with_capacity(self.nclass);

        for i in 0..self.nclass {
            let col = i % cols;
            let row = i / cols;
            let x = (col as i32) * self.lattice_w;
            let y = (row as i32) * self.lattice_h;

            let pix_box =
                PixBox::new(x, y, self.lattice_w, self.lattice_h).map_err(RecogError::Core)?;

            // C clips each 1 bpp cell back to its foreground, so the template
            // comes out at its own size rather than the lattice size. An
            // all-white cell has nothing to clip to; keep it as-is.
            let cell = extract_rect(&self.pix, &pix_box)?;
            let clipped = cell
                .clip_to_foreground()
                .map_err(RecogError::Core)?
                .map(|(pix, _)| pix)
                .unwrap_or(cell);
            templates.push(clipped);
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

/// Checks if two images match using the Hausdorff criterion, comparing them
/// as given rather than aligning their centroids.
fn hausdorff_match(pix1: &Pix, dil1: &Pix, pix2: &Pix, dil2: &Pix, rank: f32) -> RecogResult<bool> {
    hausdorff_match_with_areas(pix1, dil1, pix2, dil2, rank, (0.0, 0.0), None, None)
}

/// Tests whether an instance and a template match within the Hausdorff
/// distance implied by the dilation, allowing a `rank` fraction of pixels to
/// go uncovered in each direction.
///
/// Both directions must hold: the dilated template must cover the instance,
/// and the dilated instance must cover the template. The two images are
/// aligned by the rounded difference of their centroids before comparing.
///
/// `rank == 1.0` demands complete coverage. `area1` / `area2` are the
/// foreground counts of the unbordered instance and template; they are only
/// needed below rank 1.0.
///
/// # See also
///
/// C Leptonica: `pixHaustest()` and `pixRankHaustest()` in `jbclass.c`
#[allow(clippy::too_many_arguments)]
fn hausdorff_match_with_areas(
    pix1: &Pix,
    dil1: &Pix,
    pix2: &Pix,
    dil2: &Pix,
    rank: f32,
    delta: (f32, f32),
    area1: Option<i32>,
    area2: Option<i32>,
) -> RecogResult<bool> {
    // Too different in size to be the same character.
    if (pix1.width() as i32 - pix2.width() as i32).abs() > MAX_DIFF_WIDTH
        || (pix1.height() as i32 - pix2.height() as i32).abs() > MAX_DIFF_HEIGHT
    {
        return Ok(false);
    }

    // C rounds away from zero, which `f32::round` also does.
    let idelx = delta.0.round() as i32;
    let idely = delta.1.round() as i32;

    // How many pixels may stay uncovered in each direction.
    let allowed = |area: Option<i32>| -> i32 {
        match area {
            Some(a) if rank < 1.0 => (a as f32 * (1.0 - rank) + 0.5) as i32,
            _ => 0,
        }
    };

    // Forward: every fg pixel of the instance must fall inside the dilated
    // template, once the template is shifted by the centroid difference.
    let uncovered1 = count_uncovered(pix1, dil2, -idelx, -idely)?;
    if uncovered1 > allowed(area1) {
        return Ok(false);
    }

    // Reverse: every fg pixel of the template must fall inside the dilated
    // instance. The shift shows up with the opposite sign here.
    let uncovered2 = count_uncovered(pix2, dil1, idelx, idely)?;
    Ok(uncovered2 <= allowed(area2))
}

/// Counts the foreground pixels of `pix` that `cover`, shifted by
/// `(dx, dy)`, does not switch on.
fn count_uncovered(pix: &Pix, cover: &Pix, dx: i32, dy: i32) -> RecogResult<i32> {
    let (w, h) = (pix.width() as i32, pix.height() as i32);
    let (cw, ch) = (cover.width() as i32, cover.height() as i32);
    let mut count = 0;
    for y in 0..h {
        for x in 0..w {
            if pix.get_pixel_unchecked(x as u32, y as u32) != 1 {
                continue;
            }
            let (cx, cy) = (x + dx, y + dy);
            let covered = cx >= 0
                && cy >= 0
                && cx < cw
                && cy < ch
                && cover.get_pixel_unchecked(cx as u32, cy as u32) == 1;
            if !covered {
                count += 1;
            }
        }
    }
    Ok(count)
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
            if pix1.get_pixel_unchecked(x1 as u32, y1 as u32) == 1 {
                let x2 = x1 + dx;
                let y2 = y1 + dy;
                if x2 >= 0
                    && x2 < w2
                    && y2 >= 0
                    && y2 < h2
                    && pix2.get_pixel_unchecked(x2 as u32, y2 as u32) == 1
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
            let src_x = pix_box.x + x as i32;
            let src_y = pix_box.y + y as i32;
            if src_x >= 0 && src_x < pix.width() as i32 && src_y >= 0 && src_y < pix.height() as i32
            {
                let val = pix.get_pixel_unchecked(src_x as u32, src_y as u32);
                result_mut.set_pixel_unchecked(x, y, val);
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
            let val = pix.get_pixel_unchecked(x, y);
            result_mut.set_pixel_unchecked(x + border, y + border, val);
        }
    }

    Ok(result_mut.into())
}

/// Helper: Copies source image to destination at specified position
fn copy_to(dst: &mut crate::core::PixMut, src: &Pix, x: i32, y: i32) -> RecogResult<()> {
    for sy in 0..src.height() {
        for sx in 0..src.width() {
            if src.get_pixel_unchecked(sx, sy) == 1 {
                let dx = x + sx as i32;
                let dy = y + sy as i32;
                if dx >= 0 && (dx as u32) < dst.width() && dy >= 0 && (dy as u32) < dst.height() {
                    dst.set_pixel_unchecked(dx as u32, dy as u32, 1);
                }
            }
        }
    }
    Ok(())
}

/// Picks the one-pixel shift, out of the nine in a 3x3 neighbourhood, that
/// makes the template differ least from what is actually on the page.
///
/// The window is the template's own size, padded by [`TEMPLATE_BORDER`] and
/// placed at the centroid-corrected position. Where it runs off the page, C
/// clips it and compares only the part that remains, so components near an
/// edge can be pulled towards it; this reproduces that.
///
/// # See also
///
/// C Leptonica: `finalPositioningForAlignment()` in `jbclass.c`
fn final_positioning_for_alignment(
    pixs: &Pix,
    x: i32,
    y: i32,
    idelx: i32,
    idely: i32,
    template: &Pix,
) -> (i32, i32) {
    let (w, h) = (template.width() as i32, template.height() as i32);
    let (pw, ph) = (pixs.width() as i32, pixs.height() as i32);

    // The window, clipped to the page as C's `pixClipRectangle` does.
    let bx = x - idelx - TEMPLATE_BORDER;
    let by = y - idely - TEMPLATE_BORDER;
    let cx = bx.max(0);
    let cy = by.max(0);
    let cw = (bx + w).min(pw) - cx;
    let ch = (by + h).min(ph) - cy;
    if cw <= 0 || ch <= 0 {
        return (0, 0);
    }

    let (mut best, mut mincount) = ((0, 0), i32::MAX);
    for i in -1..=1 {
        for j in -1..=1 {
            let mut count = 0;
            for v in 0..ch {
                for u in 0..cw {
                    let mut val = pixs.get_pixel_unchecked((cx + u) as u32, (cy + v) as u32);
                    let (tu, tv) = (u - j, v - i);
                    if tu >= 0 && tv >= 0 && tu < w && tv < h {
                        val ^= template.get_pixel_unchecked(tu as u32, tv as u32);
                    }
                    count += val as i32;
                }
            }
            if count < mincount {
                mincount = count;
                best = (j, i);
            }
        }
    }
    best
}

/// The centroid an instance contributes to [`JbClasser::ptac`].
///
/// C measures it on the bordered component, the same image the classifiers
/// compare. It then appends it with `ptaJoin()`, which passes every point
/// through `ptaGetIPt()` and so **rounds it to whole pixels**; the template
/// centroids in `ptact` are stored directly and keep their fraction. The
/// asymmetry is visible in the placement of a few components, so reproduce
/// it rather than keeping the exact value here.
///
/// # See also
///
/// C Leptonica: `ptaJoin()` in `ptabasic.c`, called from
/// `jbClassifyRankHaus()` / `jbClassifyCorrelation()`
fn instance_centroid(bordered: &Pix) -> RecogResult<(f32, f32)> {
    let (x, y) = compute_centroid(bordered)?;
    // C rounds with `(l_int32)(v + 0.5)`, which truncates towards zero after
    // the offset; centroids are never negative here.
    Ok(((x + 0.5).floor(), (y + 0.5).floor()))
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
            if pix.get_pixel_unchecked(x, y) == 1 {
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
            if pix.get_pixel_unchecked(x, y) == 1 {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Creates a correlation-based classifier without keeping component instances.
///
/// Like [`correlation_init`], but sets `keep_pixaa = false` for lower memory.
///
/// Corresponds to `jbCorrelationInitWithoutComponents` in C Leptonica.
pub fn correlation_init_without_components(
    components: JbComponent,
    max_width: i32,
    max_height: i32,
    thresh: f32,
    weight_factor: f32,
) -> RecogResult<JbClasser> {
    let mut classer = correlation_init(components, max_width, max_height, thresh, weight_factor)?;
    classer.keep_pixaa = false;
    Ok(classer)
}

/// Adds pre-extracted page components to a classifier.
///
/// The components in `pixa` with corresponding bounding boxes `boxa` are
/// classified according to the method stored in the classer.
///
/// Corresponds to `jbAddPageComponents` in C Leptonica.
pub fn add_page_components(
    classer: &mut JbClasser,
    pix: &Pix,
    boxa: &[PixBox],
    pixa: &[Pix],
) -> RecogResult<()> {
    // Update page dimensions
    classer.w = pix.width() as i32;
    classer.h = pix.height() as i32;

    let page = classer.npages;
    let mut n_added = 0usize;

    for (comp, bounds) in pixa.iter().zip(boxa.iter()) {
        let cw = comp.width() as i32;
        let ch = comp.height() as i32;
        if cw > classer.max_width || ch > classer.max_height || cw == 0 || ch == 0 {
            continue;
        }

        // Same classifiers as `JbClasser::add_page`, so the two entry points
        // cannot drift apart.
        let class_id = match classer.method {
            JbMethod::RankHaus => classer.classify_rank_haus(comp)?,
            JbMethod::Correlation => classer.classify_correlation(comp)?,
        };

        classer.naclass.push(class_id);
        classer.napage.push(page);
        classer.ptac.push(instance_centroid(&add_border(
            comp,
            TEMPLATE_BORDER as u32,
        )?)?);

        let comp_idx = classer.ptac.len() - 1;
        let (ulx, uly) = classer.ul_corner(pix, class_id, comp_idx, bounds)?;
        classer.ptaul.push((ulx, uly));
        classer.ptall.push((ulx, uly + bounds.h));

        n_added += 1;
    }

    classer.nacomps.push(n_added);
    classer.base_index += n_added;
    classer.npages += 1;

    Ok(())
}

/// Runs full correlation-based JB classification on a set of page images.
///
/// Creates a correlation classifier, processes all pages, and returns
/// the compressed [`JbData`].
///
/// Corresponds to `jbCorrelation` in C Leptonica.
pub fn jb_correlation(
    pages: &[Pix],
    thresh: f32,
    weight: f32,
    components: JbComponent,
) -> RecogResult<JbData> {
    if pages.is_empty() {
        return Err(RecogError::InvalidParameter(
            "pages slice is empty".to_string(),
        ));
    }
    let mut classer = correlation_init(components, 0, 0, thresh, weight)?;
    classer.add_pages(pages)?;
    classer.get_data()
}

/// Runs full rank-Hausdorff-based JB classification on a set of page images.
///
/// Creates a RankHaus classifier, processes all pages, and returns
/// the compressed [`JbData`].
///
/// Corresponds to `jbRankHaus` in C Leptonica.
pub fn jb_rank_haus(
    pages: &[Pix],
    size: i32,
    rank: f32,
    components: JbComponent,
) -> RecogResult<JbData> {
    if pages.is_empty() {
        return Err(RecogError::InvalidParameter(
            "pages slice is empty".to_string(),
        ));
    }
    let mut classer = rank_haus_init(components, 0, 0, size, rank)?;
    classer.add_pages(pages)?;
    classer.get_data()
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

    #[test]
    fn test_pix_word_mask_by_dilation_empty_image() {
        // An empty binary image should return a zero-dilation mask.
        let pix = Pix::new(200, 100, PixelDepth::Bit1).unwrap();
        let (mask, dil) = pix_word_mask_by_dilation(&pix, 10).unwrap();
        assert_eq!(mask.width(), pix.width());
        assert_eq!(mask.height(), pix.height());
        let _ = dil; // dilation level is implementation detail
    }

    #[test]
    fn test_pix_word_boxes_by_dilation_empty_image() {
        let pix = Pix::new(200, 100, PixelDepth::Bit1).unwrap();
        let boxa = pix_word_boxes_by_dilation(&pix, 10).unwrap();
        assert_eq!(boxa.len(), 0);
    }

    #[test]
    fn test_pix_word_mask_wrong_depth() {
        let pix = Pix::new(200, 100, PixelDepth::Bit8).unwrap();
        let result = pix_word_mask_by_dilation(&pix, 10);
        assert!(result.is_err());
    }

    /// Four components: two identical solid blocks, a hollow ring of the same
    /// bounding box, and a smaller solid block. Verbatim from the C fixture
    /// used to measure the expectations below.
    fn c_fixture() -> Pix {
        let pix = Pix::new(80, 24, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let oy = 6u32;
        for ox in [4u32, 22] {
            for y in 0..7 {
                for x in 0..5 {
                    pm.set_pixel_unchecked(ox + x, oy + y, 1);
                }
            }
        }
        for y in 0..7u32 {
            for x in 0..5u32 {
                if y == 0 || y == 6 || x == 0 || x == 4 {
                    pm.set_pixel_unchecked(40 + x, oy + y, 1);
                }
            }
        }
        for y in 0..5u32 {
            for x in 0..3u32 {
                pm.set_pixel_unchecked(58 + x, oy + y, 1);
            }
        }
        pm.into()
    }

    /// C `JB_ADDED_PIXELS` is 6, so a 5x7 component yields a 17x19 template.
    #[test]
    fn test_template_border_matches_c() {
        assert_eq!(TEMPLATE_BORDER, 6);
    }

    /// C `jbCorrelationInit(JB_CONN_COMPS, 0, 0, ...)` fills in
    /// `MAX_CONN_COMP_WIDTH` = 350 and `MAX_COMP_HEIGHT` = 120.
    #[test]
    fn test_default_max_size_matches_c() {
        let c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        assert_eq!((c.max_width, c.max_height), (350, 120));
        let c = rank_haus_init(JbComponent::ConnComps, 0, 0, 2, 0.97).unwrap();
        assert_eq!((c.max_width, c.max_height), (350, 120));
        let c = correlation_init(JbComponent::Words, 0, 0, 0.8, 0.6).unwrap();
        assert_eq!((c.max_width, c.max_height), (1000, 120));
    }

    /// Expectations are the verbatim output of C on [`c_fixture`]: the two
    /// identical blocks share a class, the ring and the smaller block each
    /// get their own.
    #[test]
    fn test_correlation_classes_match_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        assert_eq!(c.nclass, 3);
        assert_eq!(c.naclass, vec![0, 0, 1, 2]);
        let sizes: Vec<_> = c.pixat.iter().map(|p| (p.width(), p.height())).collect();
        assert_eq!(sizes, [(17, 19), (17, 19), (15, 17)]);
    }

    #[test]
    fn test_rank_haus_classes_match_c() {
        let mut c = rank_haus_init(JbComponent::ConnComps, 0, 0, 2, 0.97).unwrap();
        c.add_page(&c_fixture()).unwrap();
        assert_eq!(c.nclass, 3);
        assert_eq!(c.naclass, vec![0, 0, 1, 2]);
        let sizes: Vec<_> = c.pixat.iter().map(|p| (p.width(), p.height())).collect();
        assert_eq!(sizes, [(17, 19), (17, 19), (15, 17)]);
    }

    /// C `jbDataSave` uses a lattice one pixel larger than the biggest
    /// template, and `pixaDisplayOnLattice` lays the cells out with
    /// `nw = floor(sqrt(n))` columns, so 3 classes make a single column.
    #[test]
    fn test_data_lattice_matches_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        let d = c.get_data().unwrap();
        assert_eq!((d.lattice_w, d.lattice_h), (18, 20));
        assert_eq!((d.pix.width(), d.pix.height()), (18, 60));
        assert_eq!(d.nclass, 3);
    }

    /// C `jbGetULCorners()` stores the centroid of the *bordered* component,
    /// so a solid 5x7 block padded by 6 has its centroid at (8, 9).
    #[test]
    fn test_instance_centroids_match_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        let rounded: Vec<_> = c.ptac.iter().map(|&(x, y)| (x, y)).collect();
        assert_eq!(rounded, [(8.0, 9.0), (8.0, 9.0), (8.0, 9.0), (7.0, 8.0)]);
    }

    /// Verbatim from C. Component 0 sits at x = 4 but is placed at x = 3:
    /// its alignment window runs off the left edge of the page, which C
    /// clips, and the shifted position then scores better.
    #[test]
    fn test_ul_corners_match_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        assert_eq!(c.ptaul, [(3, 6), (22, 6), (40, 6), (58, 6)]);
    }

    /// C `pixaCreateFromPix()` clips each 1 bpp cell back to its foreground,
    /// so the templates come out at their own size, not the lattice size.
    #[test]
    fn test_extracted_templates_are_clipped_like_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        let d = c.get_data().unwrap();
        let t = d.extract_templates().unwrap();
        let sizes: Vec<_> = t.iter().map(|p| (p.width(), p.height())).collect();
        assert_eq!(sizes, [(5, 7), (5, 7), (3, 5)]);
    }

    /// The whole pipeline: the page C reconstructs from the templates.
    #[test]
    fn test_render_page_matches_c() {
        let mut c = correlation_init(JbComponent::ConnComps, 0, 0, 0.8, 0.6).unwrap();
        c.add_page(&c_fixture()).unwrap();
        let d = c.get_data().unwrap();
        let page = d.render_page(0).unwrap();
        assert_eq!((page.width(), page.height()), (80, 24));

        // Rows 6..13 of C's output; every other row is blank.
        let expected: [&str; 7] = [
            "...#####..............#####.............#####.............###...................",
            "...#####..............#####.............#...#.............###...................",
            "...#####..............#####.............#...#.............###...................",
            "...#####..............#####.............#...#.............###...................",
            "...#####..............#####.............#...#.............###...................",
            "...#####..............#####.............#...#...................................",
            "...#####..............#####.............#####...................................",
        ];
        for (i, row) in expected.iter().enumerate() {
            let y = 6 + i as u32;
            let got: String = (0..80)
                .map(|x| {
                    if page.get_pixel_unchecked(x, y) == 1 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect();
            assert_eq!(&got, row, "row {y}");
        }
        for y in (0..6).chain(13..24) {
            let fg = (0..80)
                .filter(|&x| page.get_pixel_unchecked(x, y) == 1)
                .count();
            assert_eq!(fg, 0, "row {y} should be blank");
        }
    }
}
