//! Morphological application functions
//!
//! Higher-level morphological operations including:
//! - Sequence operations with masking
//! - Sequence operations by connected component or region
//! - Union and intersection of morphological ops over a set of SELs
//! - Seedfill via dilation (binary reconstruction)
//! - Grayscale morphological gradient

use crate::core::{Numa, Pix, Pixa, PixelDepth, Pta};
use crate::filter::blockconv;
use crate::morph::{
    MorphError, MorphResult, Sel, close, dilate, erode, gradient_gray, hit_miss_transform,
    morph_sequence, open,
};
use crate::region::{ConnectivityType, conncomp_pixa, fill_holes, seedfill_gray};
use crate::transform::{
    GrayMinMaxMode, scale_by_sampling, scale_by_sampling_to_size, scale_gray_min_max, scale_to_size,
};

/// Type of morphological operation for union/intersection functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphOpType {
    /// Binary dilation
    Dilate,
    /// Binary erosion
    Erode,
    /// Binary opening (erosion then dilation)
    Open,
    /// Binary closing (dilation then erosion)
    Close,
    /// Hit-Miss Transform
    HitMiss,
}

/// Scaling direction for [`pixa_extend_by_scaling`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleDirection {
    Horizontal,
    Vertical,
    BothDirections,
}

/// Run polarity for [`run_histogram_morph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunType {
    Off,
    On,
}

/// Run direction for [`run_histogram_morph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDirection {
    Horizontal,
    Vertical,
}

/// Polarity for [`fast_tophat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TophatType {
    White,
    Black,
}

/// Apply a morphological sequence to an image, then restore source pixels
/// where the mask is ON.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `mask` - optional 1 bpp mask; ON pixels get restored from `pix`
/// * `sequence` - morphological sequence string (e.g., `"D3.3 + E3.3"`)
///
/// If `mask` is `None`, this is equivalent to `morph_sequence(pix, sequence)`.
///
/// Based on C leptonica `pixMorphSequenceMasked`.
pub fn morph_sequence_masked(pix: &Pix, mask: Option<&Pix>, sequence: &str) -> MorphResult<Pix> {
    let result = morph_sequence(pix, sequence)?;
    if let Some(m) = mask {
        let mut rm = match result.try_into_mut() {
            Ok(pm) => pm,
            Err(p) => p.to_mut(),
        };
        rm.combine_masked(pix, m)?;
        Ok(rm.into())
    } else {
        Ok(result)
    }
}

/// Compute the union (OR) of a morphological operation applied to an image
/// with each SEL in the given slice.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sels` - slice of structuring elements
/// * `op` - the morphological operation to apply
///
/// Returns a new image where each pixel is ON if the operation with ANY sel
/// produces ON at that location.
///
/// Based on C leptonica `pixUnionOfMorphOps`.
pub fn union_of_morph_ops(pix: &Pix, sels: &[Sel], op: MorphOpType) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    if sels.is_empty() {
        return Err(MorphError::InvalidParameters(
            "sels must not be empty".into(),
        ));
    }
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut result = out.try_into_mut().unwrap();
    for sel in sels {
        let partial = apply_morph_op(pix, sel, op)?;
        result.or_inplace(&partial)?;
    }
    Ok(result.into())
}

/// Compute the intersection (AND) of a morphological operation applied to an
/// image with each SEL in the given slice.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sels` - slice of structuring elements
/// * `op` - the morphological operation to apply
///
/// Returns a new image where each pixel is ON if the operation with ALL sels
/// produces ON at that location.
///
/// Based on C leptonica `pixIntersectionOfMorphOps`.
pub fn intersection_of_morph_ops(pix: &Pix, sels: &[Sel], op: MorphOpType) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    if sels.is_empty() {
        return Err(MorphError::InvalidParameters(
            "sels must not be empty".into(),
        ));
    }
    // Start with all-ON image, AND each result in
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut result = out.try_into_mut().unwrap();
    result.set_all();
    for sel in sels {
        let partial = apply_morph_op(pix, sel, op)?;
        result.and_inplace(&partial)?;
    }
    Ok(result.into())
}

/// Apply a single morphological operation with the given SEL.
fn apply_morph_op(pix: &Pix, sel: &Sel, op: MorphOpType) -> MorphResult<Pix> {
    match op {
        MorphOpType::Dilate => dilate(pix, sel),
        MorphOpType::Erode => erode(pix, sel),
        MorphOpType::Open => open(pix, sel),
        MorphOpType::Close => close(pix, sel),
        MorphOpType::HitMiss => hit_miss_transform(pix, sel),
    }
}

/// Binary seedfill via morphological dilation (binary reconstruction).
///
/// Iteratively dilates the seed image with a 3x3 SEL (4- or 8-connected),
/// AND-ing with the mask after each step, until convergence or `max_iters`.
///
/// # Arguments
/// * `seed` - 1 bpp seed image
/// * `mask` - 1 bpp filling mask (must have same dimensions as `seed`)
/// * `max_iters` - maximum iterations (0 means use default of 1000)
/// * `connectivity` - 4 or 8 connected
///
/// Based on C leptonica `pixSeedfillMorph`.
pub fn seedfill_morph(
    seed: &Pix,
    mask: &Pix,
    max_iters: u32,
    connectivity: u8,
) -> MorphResult<Pix> {
    if seed.depth() != PixelDepth::Bit1 || mask.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: seed.depth().bits(),
        });
    }
    if connectivity != 4 && connectivity != 8 {
        return Err(MorphError::InvalidParameters(
            "connectivity must be 4 or 8".into(),
        ));
    }
    if seed.width() != mask.width() || seed.height() != mask.height() {
        return Err(MorphError::InvalidParameters(
            "seed and mask must have the same dimensions".into(),
        ));
    }

    let max_iters = if max_iters == 0 { 1000 } else { max_iters };

    // Build a 3x3 SEL: for 8-connected, use full 3x3 brick; for 4-connected,
    // use a '+' shape (no corner hits)
    let sel = if connectivity == 8 {
        Sel::create_brick(3, 3)?
    } else {
        // 4-connected: 3x3 brick minus corners = plus sign
        Sel::create_cross(3)?
    };

    let mut current = seed.clone();
    let mut next;
    for _ in 0..max_iters {
        next = dilate(&current, &sel)?;
        let mut next_mut = match next.try_into_mut() {
            Ok(pm) => pm,
            Err(p) => p.to_mut(),
        };
        next_mut.and_inplace(mask)?;
        let candidate: Pix = next_mut.into();
        if candidate.equals(&current) {
            return Ok(candidate);
        }
        current = candidate;
    }
    Ok(current)
}

/// Grayscale morphological gradient: dilation - erosion (after optional smoothing).
///
/// Emphasises edges and transitions in an 8-bpp image. Optional block
/// convolution smoothing can be applied first to reduce noise.
///
/// # Arguments
/// * `pix` - 8 bpp input image
/// * `hsize` - SEL width
/// * `vsize` - SEL height
/// * `smoothing` - half-width of smoothing convolution (0 = no smoothing)
///
/// Based on C leptonica `pixMorphGradient`.
pub fn morph_gradient(pix: &Pix, hsize: u32, vsize: u32, smoothing: u32) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    if smoothing > 0 {
        return Err(MorphError::InvalidParameters(
            "smoothing > 0 requires block convolution (not yet implemented)".into(),
        ));
    }
    // Note: size rounding to odd values is handled by lower-level grayscale
    // morphology functions (via ensure_odd) called inside `gradient_gray`.
    gradient_gray(pix, hsize, vsize)
}

/// Apply a morphological sequence to each connected component separately.
///
/// Extracts connected components, applies the sequence to each one
/// independently, then composites results back into a single image.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `sequence` - morphological sequence string (e.g., `"D3.3"`)
/// * `min_w` - minimum component width to process (0 = no filter)
/// * `min_h` - minimum component height to process (0 = no filter)
/// * `connectivity` - 4 or 8 connected
///
/// Based on C leptonica `pixMorphSequenceByComponent`.
pub fn morph_sequence_by_component(
    pix: &Pix,
    sequence: &str,
    min_w: u32,
    min_h: u32,
    connectivity: u8,
) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }

    let conn = match connectivity {
        4 => ConnectivityType::FourWay,
        8 => ConnectivityType::EightWay,
        _ => {
            return Err(MorphError::InvalidParameters(
                "connectivity must be 4 or 8".into(),
            ));
        }
    };

    let (boxa, pixa) = conncomp_pixa(pix, conn)
        .map_err(|e| MorphError::InvalidParameters(format!("conncomp error: {}", e)))?;

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..pixa.len() {
        let comp = pixa.get(i).unwrap();
        let b = boxa.get(i).unwrap();
        let bw = b.w as u32;
        let bh = b.h as u32;

        // Filter by minimum size
        if (min_w > 0 && bw < min_w) || (min_h > 0 && bh < min_h) {
            continue;
        }

        // Apply morph sequence to this component
        let processed = morph_sequence(comp, sequence)?;

        // Paste result back at original position
        let ox = b.x;
        let oy = b.y;
        let pw = processed.width();
        let ph = processed.height();
        for py in 0..ph {
            let dy = oy + py as i32;
            if dy < 0 || dy as u32 >= h {
                continue;
            }
            for px in 0..pw {
                let dx = ox + px as i32;
                if dx < 0 || dx as u32 >= w {
                    continue;
                }
                if processed.get_pixel_unchecked(px, py) != 0 {
                    out_mut.set_pixel_unchecked(dx as u32, dy as u32, 1);
                }
            }
        }
    }

    Ok(out_mut.into())
}

/// Apply a morphological sequence to each region defined by a mask.
///
/// For each connected component in the mask, extracts the corresponding
/// region from the source, applies the sequence, and pastes back.
///
/// # Arguments
/// * `pix` - 1 bpp input image
/// * `mask` - 1 bpp mask defining regions (connected components)
/// * `sequence` - morphological sequence string
/// * `connectivity` - 4 or 8 connected
/// * `min_w` - minimum region width to process (0 = no filter)
/// * `min_h` - minimum region height to process (0 = no filter)
///
/// Based on C leptonica `pixMorphSequenceByRegion`.
pub fn morph_sequence_by_region(
    pix: &Pix,
    mask: &Pix,
    sequence: &str,
    connectivity: u8,
    min_w: u32,
    min_h: u32,
) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    if mask.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: mask.depth().bits(),
        });
    }

    let conn = match connectivity {
        4 => ConnectivityType::FourWay,
        8 => ConnectivityType::EightWay,
        _ => {
            return Err(MorphError::InvalidParameters(
                "connectivity must be 4 or 8".into(),
            ));
        }
    };

    let (boxa, mask_pixa) = conncomp_pixa(mask, conn)
        .map_err(|e| MorphError::InvalidParameters(format!("conncomp error: {}", e)))?;

    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for i in 0..mask_pixa.len() {
        let mask_comp = mask_pixa.get(i).unwrap();
        let b = boxa.get(i).unwrap();
        let bw = b.w as u32;
        let bh = b.h as u32;

        if (min_w > 0 && bw < min_w) || (min_h > 0 && bh < min_h) {
            continue;
        }

        // Clip source region to match the mask component bounds
        let clip = pix.clip_rectangle(b.x as u32, b.y as u32, bw, bh)?;

        // AND with mask component to get only pixels within this region
        let region = clip.and(mask_comp)?;

        // Apply morph sequence
        let processed = morph_sequence(&region, sequence)?;

        // Paste back
        let ox = b.x;
        let oy = b.y;
        let pw = processed.width();
        let ph = processed.height();
        for py in 0..ph {
            let dy = oy + py as i32;
            if dy < 0 || dy as u32 >= h {
                continue;
            }
            for px in 0..pw {
                let dx = ox + px as i32;
                if dx < 0 || dx as u32 >= w {
                    continue;
                }
                if processed.get_pixel_unchecked(px, py) != 0 {
                    out_mut.set_pixel_unchecked(dx as u32, dy as u32, 1);
                }
            }
        }
    }

    Ok(out_mut.into())
}

/// Fill holes only in selected connected components.
///
/// Components are selected by width/height thresholds.
/// Based on C leptonica `pixSelectiveConnCompFill`.
pub fn selective_conn_comp_fill(
    pix: &Pix,
    connectivity: u8,
    min_w: u32,
    min_h: u32,
) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    let conn = match connectivity {
        4 => ConnectivityType::FourWay,
        8 => ConnectivityType::EightWay,
        _ => {
            return Err(MorphError::InvalidParameters(
                "connectivity must be 4 or 8".into(),
            ));
        }
    };
    let hole_conn = if conn == ConnectivityType::FourWay {
        ConnectivityType::EightWay
    } else {
        ConnectivityType::FourWay
    };
    let min_w = min_w.max(1);
    let min_h = min_h.max(1);

    let (boxa, pixa) = conncomp_pixa(pix, conn)
        .map_err(|e| MorphError::InvalidParameters(format!("conncomp error: {e}")))?;

    let out = pix.clone();
    let mut out_mut = match out.try_into_mut() {
        Ok(pm) => pm,
        Err(p) => p.to_mut(),
    };
    let ow = out_mut.width();
    let oh = out_mut.height();

    for i in 0..pixa.len() {
        let comp = pixa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing pix at index {i}")))?;
        let b = boxa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing box at index {i}")))?;
        if b.w < min_w as i32 || b.h < min_h as i32 {
            continue;
        }

        let filled = fill_holes(comp, hole_conn)
            .map_err(|e| MorphError::InvalidParameters(format!("fill_holes error: {e}")))?;
        for y in 0..filled.height() {
            let dy = b.y + y as i32;
            if dy < 0 || dy as u32 >= oh {
                continue;
            }
            for x in 0..filled.width() {
                if filled.get_pixel_unchecked(x, y) == 0 {
                    continue;
                }
                let dx = b.x + x as i32;
                if dx >= 0 && (dx as u32) < ow {
                    out_mut.set_pixel_unchecked(dx as u32, dy as u32, 1);
                }
            }
        }
    }

    Ok(out_mut.into())
}

/// Remove matched binary patterns from an image.
///
/// Returns a modified copy of `pix`.
/// Based on C leptonica `pixRemoveMatchedPattern`.
pub fn remove_matched_pattern(
    pix: &Pix,
    pattern: &Pix,
    eroded_matches: &Pix,
    x0: i32,
    y0: i32,
    dsize: u32,
) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1
        || pattern.depth() != PixelDepth::Bit1
        || eroded_matches.depth() != PixelDepth::Bit1
    {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    if dsize > 4 {
        return Err(MorphError::InvalidParameters(
            "dsize must be in [0, 4]".into(),
        ));
    }

    let (boxa, pixa) = conncomp_pixa(eroded_matches, ConnectivityType::EightWay)
        .map_err(|e| MorphError::InvalidParameters(format!("conncomp error: {e}")))?;
    if boxa.is_empty() {
        return Ok(pix.clone());
    }
    let centroids = pixa_centroids(&pixa)?;

    let pattern_expanded = if dsize > 0 {
        let bordered = pattern.add_border(dsize, 0)?;
        let sel = Sel::create_brick(2 * dsize + 1, 2 * dsize + 1)?;
        dilate(&bordered, &sel)?
    } else {
        pattern.clone()
    };

    let out = pix.clone();
    let mut out_mut = match out.try_into_mut() {
        Ok(pm) => pm,
        Err(p) => p.to_mut(),
    };
    let ow = out_mut.width();
    let oh = out_mut.height();
    let pw = pattern_expanded.width();
    let ph = pattern_expanded.height();

    for i in 0..boxa.len() {
        let b = boxa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing box at index {i}")))?;
        let (cx, cy) = centroids.get(i).unwrap_or((0.0, 0.0));
        let ox = b.x + cx.round() as i32 - x0 - dsize as i32;
        let oy = b.y + cy.round() as i32 - y0 - dsize as i32;

        for y in 0..ph {
            let dy = oy + y as i32;
            if dy < 0 || dy as u32 >= oh {
                continue;
            }
            for x in 0..pw {
                if pattern_expanded.get_pixel_unchecked(x, y) == 0 {
                    continue;
                }
                let dx = ox + x as i32;
                if dx >= 0 && (dx as u32) < ow {
                    out_mut.set_pixel_unchecked(dx as u32, dy as u32, 0);
                }
            }
        }
    }

    Ok(out_mut.into())
}

/// Display matched pattern locations by painting a binary stencil in color.
///
/// Returns a 32 bpp image.
/// Based on C leptonica `pixDisplayMatchedPattern`.
pub fn display_matched_pattern(
    pix: &Pix,
    pattern: &Pix,
    eroded_matches: &Pix,
    x0: i32,
    y0: i32,
    color: u32,
    scale: f32,
) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1
        || pattern.depth() != PixelDepth::Bit1
        || eroded_matches.depth() != PixelDepth::Bit1
    {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(MorphError::InvalidParameters("scale must be > 0".into()));
    }
    let scale = scale.min(1.0);

    let (boxa, pixa) = conncomp_pixa(eroded_matches, ConnectivityType::EightWay)
        .map_err(|e| MorphError::InvalidParameters(format!("conncomp error: {e}")))?;
    let centroids = if pixa.is_empty() {
        Pta::new()
    } else {
        pixa_centroids(&pixa)?
    };

    let (base, pattern_scaled) = if (scale - 1.0).abs() <= f32::EPSILON {
        (pix.convert_to_32()?, pattern.clone())
    } else {
        let pixs = scale_by_sampling(pix, scale, scale)
            .map_err(|e| MorphError::InvalidParameters(format!("scale_by_sampling error: {e}")))?;
        let pats = scale_by_sampling(pattern, scale, scale)
            .map_err(|e| MorphError::InvalidParameters(format!("scale_by_sampling error: {e}")))?;
        (pixs.convert_to_32()?, pats)
    };

    let mut out_mut = match base.try_into_mut() {
        Ok(pm) => pm,
        Err(p) => p.to_mut(),
    };
    let paint = if (color & 0xff) == 0 {
        color | 0xff
    } else {
        color
    };

    for i in 0..boxa.len() {
        let b = boxa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing box at index {i}")))?;
        let (cx, cy) = centroids.get(i).unwrap_or((0.0, 0.0));
        let mut ox = b.x as f32 + cx - x0 as f32;
        let mut oy = b.y as f32 + cy - y0 as f32;
        if scale < 1.0 {
            ox *= scale;
            oy *= scale;
        }
        out_mut.set_masked_general(&pattern_scaled, paint, ox.round() as i32, oy.round() as i32)?;
    }

    Ok(out_mut.into())
}

/// Extend a pixa by iterative erosion or dilation.
///
/// Based on C leptonica `pixaExtendByMorph`.
pub fn pixa_extend_by_morph(
    pixa: &Pixa,
    op: MorphOpType,
    niters: u32,
    sel: Option<&Sel>,
    include: bool,
) -> MorphResult<Pixa> {
    if niters == 0 {
        return Ok(pixa.clone());
    }
    if op != MorphOpType::Dilate && op != MorphOpType::Erode {
        return Err(MorphError::InvalidParameters(
            "op must be Dilate or Erode".into(),
        ));
    }
    for i in 0..pixa.len() {
        let pix = pixa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing pix at index {i}")))?;
        if pix.depth() != PixelDepth::Bit1 {
            return Err(MorphError::UnsupportedDepth {
                expected: "1-bpp binary",
                actual: pix.depth().bits(),
            });
        }
    }

    let default_sel;
    let sel = if let Some(s) = sel {
        s
    } else {
        default_sel = Sel::create_brick(2, 2)?;
        &default_sel
    };

    let extra = if include { 1 } else { 0 };
    let mut out = Pixa::with_capacity(pixa.len() * (niters as usize + extra));
    for i in 0..pixa.len() {
        let mut current = pixa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing pix at index {i}")))?
            .clone();
        if include {
            out.push(current.clone());
        }
        for _ in 0..niters {
            current = if op == MorphOpType::Dilate {
                dilate(&current, sel)?
            } else {
                erode(&current, sel)?
            };
            out.push(current.clone());
        }
    }

    Ok(out)
}

/// Extend a pixa by scaling each image using all provided factors.
///
/// Based on C leptonica `pixaExtendByScaling`.
pub fn pixa_extend_by_scaling(
    pixa: &Pixa,
    scales: &[f32],
    direction: ScaleDirection,
    include: bool,
) -> MorphResult<Pixa> {
    if scales.is_empty() {
        return Err(MorphError::InvalidParameters(
            "scales must not be empty".into(),
        ));
    }
    if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
        return Err(MorphError::InvalidParameters(
            "all scales must be finite and > 0".into(),
        ));
    }

    let extra = if include { 1 } else { 0 };
    let mut out = Pixa::with_capacity(pixa.len() * (scales.len() + extra));
    for i in 0..pixa.len() {
        let pix = pixa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing pix at index {i}")))?;
        let w = pix.width() as f32;
        let h = pix.height() as f32;
        if include {
            out.push(pix.clone());
        }

        for &scale in scales {
            let mut new_w = w;
            let mut new_h = h;
            if direction == ScaleDirection::Horizontal
                || direction == ScaleDirection::BothDirections
            {
                new_w = (w * scale).round();
            }
            if direction == ScaleDirection::Vertical || direction == ScaleDirection::BothDirections
            {
                new_h = (h * scale).round();
            }
            let new_w = (new_w as i32).max(1) as u32;
            let new_h = (new_h as i32).max(1) as u32;
            let scaled = scale_to_size(pix, new_w, new_h)
                .map_err(|e| MorphError::InvalidParameters(format!("scale_to_size error: {e}")))?;
            out.push(scaled);
        }
    }

    Ok(out)
}

/// Run-length histogram via iterative binary erosion.
///
/// Based on C leptonica `pixRunHistogramMorph`.
pub fn run_histogram_morph(
    pix: &Pix,
    run_type: RunType,
    direction: RunDirection,
    max_size: u32,
) -> MorphResult<Numa> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    let max_size = max_size.max(1);

    let sel = if direction == RunDirection::Horizontal {
        Sel::create_brick(1, 2)?
    } else {
        Sel::create_brick(2, 1)?
    };
    let pix1 = if run_type == RunType::Off {
        pix.invert()
    } else {
        pix.clone()
    };

    let mut counts = Vec::new();
    counts.push(pix1.count_pixels() as f32);
    let mut pix2 = erode(&pix1, &sel)?;
    counts.push(pix2.count_pixels() as f32);

    for _ in 0..(max_size / 2) {
        let pix3 = erode(&pix2, &sel)?;
        counts.push(pix3.count_pixels() as f32);
        pix2 = erode(&pix3, &sel)?;
        counts.push(pix2.count_pixels() as f32);
    }

    let mut hist = Vec::with_capacity(counts.len().saturating_sub(1));
    hist.push(0.0);
    for i in 1..counts.len().saturating_sub(1) {
        let val = counts[i + 1] - 2.0 * counts[i] + counts[i - 1];
        hist.push(val);
    }

    Ok(Numa::from_vec(hist))
}

/// HDome extraction on grayscale images.
///
/// Based on C leptonica `pixHDome`.
pub fn h_dome(pix: &Pix, height: i32, connectivity: u8) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    if height < 0 {
        return Err(MorphError::InvalidParameters("height must be >= 0".into()));
    }
    if height == 0 {
        return Ok(pix.create_template());
    }
    let conn = match connectivity {
        4 => ConnectivityType::FourWay,
        8 => ConnectivityType::EightWay,
        _ => {
            return Err(MorphError::InvalidParameters(
                "connectivity must be 4 or 8".into(),
            ));
        }
    };

    let seed = pix.add_constant(-height)?;
    let filled = seedfill_gray(&seed, pix, conn)
        .map_err(|e| MorphError::InvalidParameters(format!("seedfill_gray error: {e}")))?;
    pix.arith_subtract(&filled).map_err(MorphError::Core)
}

/// Fast tophat-like background removal.
///
/// Based on C leptonica `pixFastTophat`.
pub fn fast_tophat(pix: &Pix, xsize: u32, ysize: u32, top_type: TophatType) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    if xsize == 0 || ysize == 0 {
        return Err(MorphError::InvalidParameters(
            "xsize and ysize must be >= 1".into(),
        ));
    }
    if xsize == 1 && ysize == 1 {
        return Ok(pix.create_template());
    }

    let mode = if top_type == TophatType::White {
        GrayMinMaxMode::Min
    } else {
        GrayMinMaxMode::Max
    };
    let reduced = scale_gray_min_max(pix, xsize, ysize, mode)
        .map_err(|e| MorphError::InvalidParameters(format!("scale_gray_min_max error: {e}")))?;
    let smooth = blockconv(&reduced, 1, 1)
        .map_err(|e| MorphError::InvalidParameters(format!("blockconv error: {e}")))?;
    let expanded = scale_by_sampling_to_size(&smooth, pix.width(), pix.height()).map_err(|e| {
        MorphError::InvalidParameters(format!("scale_by_sampling_to_size error: {e}"))
    })?;

    if top_type == TophatType::White {
        pix.arith_subtract(&expanded).map_err(MorphError::Core)
    } else {
        expanded.arith_subtract(pix).map_err(MorphError::Core)
    }
}

/// Compute the centroid of foreground content in a 1 bpp or 8 bpp image.
///
/// For 1 bpp images, foreground pixels (value != 0) are weighted uniformly.
/// For 8 bpp images, pixel values are used as weights.
///
/// Returns `(0.0, 0.0)` when the image has no foreground/weight.
///
/// Based on C leptonica `pixCentroid`.
pub fn pix_centroid(pix: &Pix) -> MorphResult<(f32, f32)> {
    let depth = pix.depth();
    if depth != PixelDepth::Bit1 && depth != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1 or 8-bpp",
            actual: depth.bits(),
        });
    }

    let mut xsum = 0.0f64;
    let mut ysum = 0.0f64;
    let mut wsum = 0.0f64;

    let w = pix.width();
    let h = pix.height();
    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel_unchecked(x, y);
            let weight = if depth == PixelDepth::Bit1 {
                if val != 0 { 1.0 } else { 0.0 }
            } else {
                val as f64
            };
            if weight > 0.0 {
                xsum += weight * x as f64;
                ysum += weight * y as f64;
                wsum += weight;
            }
        }
    }

    if wsum <= f64::EPSILON {
        return Ok((0.0, 0.0));
    }
    Ok(((xsum / wsum) as f32, (ysum / wsum) as f32))
}

/// Compute centroids for each Pix in a Pixa.
///
/// Centroids are relative to each Pix origin.
/// If a Pix has an unsupported depth, `(0.0, 0.0)` is stored for that entry.
///
/// Based on C leptonica `pixaCentroids`.
pub fn pixa_centroids(pixa: &Pixa) -> MorphResult<Pta> {
    if pixa.is_empty() {
        return Err(MorphError::InvalidParameters("no pix in pixa".into()));
    }

    let mut pta = Pta::with_capacity(pixa.len());
    for i in 0..pixa.len() {
        let pix = pixa
            .get(i)
            .ok_or_else(|| MorphError::InvalidParameters(format!("missing pix at index {i}")))?;
        match pix_centroid(pix) {
            Ok((x, y)) => pta.push(x, y),
            Err(MorphError::UnsupportedDepth { .. }) => pta.push(0.0, 0.0),
            Err(e) => return Err(e),
        }
    }

    Ok(pta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Pix, Pixa, PixelDepth};

    fn create_1bpp_test() -> Pix {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 8..24 {
            for x in 8..24 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        pm.into()
    }

    fn create_8bpp_test() -> Pix {
        let pix = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 8..24 {
            for x in 8..24 {
                pm.set_pixel_unchecked(x, y, 128);
            }
        }
        pm.into()
    }

    // --- morph_sequence_masked tests ---

    #[test]
    fn test_morph_sequence_masked_no_mask_equals_sequence() {
        let pix = create_1bpp_test();
        let result_masked = morph_sequence_masked(&pix, None, "D3.3").unwrap();
        let result_direct = crate::morph::morph_sequence(&pix, "D3.3").unwrap();
        assert!(result_masked.equals(&result_direct));
    }

    #[test]
    fn test_morph_sequence_masked_restores_under_mask() {
        let pix = create_1bpp_test();
        // mask covers center 8x8 region
        let mask = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        for y in 12..20 {
            for x in 12..20 {
                mm.set_pixel_unchecked(x, y, 1);
            }
        }
        let mask: Pix = mm.into();
        let result = morph_sequence_masked(&pix, Some(&mask), "E3.3").unwrap();
        // Under the mask, pixels should match original
        for y in 12..20u32 {
            for x in 12..20u32 {
                assert_eq!(
                    result.get_pixel_unchecked(x, y),
                    pix.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]
    fn test_morph_sequence_masked_preserves_dimensions() {
        let pix = create_1bpp_test();
        let result = morph_sequence_masked(&pix, None, "D3.3").unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    // --- union_of_morph_ops tests ---

    #[test]
    fn test_union_of_morph_ops_single_sel_equals_direct() {
        let pix = create_1bpp_test();
        let sel = Sel::create_brick(3, 3).unwrap();
        let union =
            union_of_morph_ops(&pix, std::slice::from_ref(&sel), MorphOpType::Dilate).unwrap();
        let direct = dilate(&pix, &sel).unwrap();
        assert!(union.equals(&direct));
    }

    #[test]
    fn test_union_of_morph_ops_superset() {
        let pix = create_1bpp_test();
        let sels: Vec<_> = [3u32, 5, 7]
            .iter()
            .map(|&s| Sel::create_brick(s, s).unwrap())
            .collect();
        let union = union_of_morph_ops(&pix, &sels, MorphOpType::Dilate).unwrap();
        // Union should have at least as many pixels as any single result
        let direct = dilate(&pix, &sels[0]).unwrap();
        assert!(union.count_pixels() >= direct.count_pixels());
    }

    #[test]
    fn test_union_of_morph_ops_requires_1bpp() {
        let pix = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();
        assert!(union_of_morph_ops(&pix, &[sel], MorphOpType::Dilate).is_err());
    }

    #[test]
    fn test_union_of_morph_ops_empty_sels_error() {
        let pix = create_1bpp_test();
        assert!(union_of_morph_ops(&pix, &[], MorphOpType::Dilate).is_err());
    }

    // --- intersection_of_morph_ops tests ---

    #[test]
    fn test_intersection_of_morph_ops_single_sel_equals_direct() {
        let pix = create_1bpp_test();
        let sel = Sel::create_brick(3, 3).unwrap();
        let result =
            intersection_of_morph_ops(&pix, std::slice::from_ref(&sel), MorphOpType::Erode)
                .unwrap();
        let direct = erode(&pix, &sel).unwrap();
        assert!(result.equals(&direct));
    }

    #[test]
    fn test_intersection_of_morph_ops_subset() {
        let pix = create_1bpp_test();
        let sels: Vec<_> = [3u32, 5]
            .iter()
            .map(|&s| Sel::create_brick(s, s).unwrap())
            .collect();
        let result = intersection_of_morph_ops(&pix, &sels, MorphOpType::Erode).unwrap();
        let direct = erode(&pix, &sels[1]).unwrap();
        // Intersection of erosions with larger SEL should have fewer or equal pixels
        assert!(result.count_pixels() <= direct.count_pixels());
    }

    #[test]
    fn test_intersection_of_morph_ops_requires_1bpp() {
        let pix = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let sel = Sel::create_brick(3, 3).unwrap();
        assert!(intersection_of_morph_ops(&pix, &[sel], MorphOpType::Erode).is_err());
    }

    // --- seedfill_morph tests ---

    #[test]
    fn test_seedfill_morph_grows_into_mask() {
        // Seed: single pixel; mask: larger region
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(15, 15, 1); // single seed pixel
        let seed: Pix = pm.into();

        let mask = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        for y in 10..22u32 {
            for x in 10..22u32 {
                mm.set_pixel_unchecked(x, y, 1);
            }
        }
        let mask: Pix = mm.into();

        let result = seedfill_morph(&seed, &mask, 0, 4).unwrap();
        // Seed should have grown to fill the mask
        assert_eq!(result.count_pixels(), mask.count_pixels());
    }

    #[test]
    fn test_seedfill_morph_bounded_by_mask() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Fill entire image as seed
        for y in 0..32u32 {
            for x in 0..32u32 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        let seed: Pix = pm.into();

        // Mask: only center region
        let mask = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        for y in 10..22u32 {
            for x in 10..22u32 {
                mm.set_pixel_unchecked(x, y, 1);
            }
        }
        let mask: Pix = mm.into();

        let result = seedfill_morph(&seed, &mask, 0, 4).unwrap();
        // Result should be bounded by mask
        assert!(result.count_pixels() <= mask.count_pixels());
    }

    #[test]
    fn test_seedfill_morph_requires_1bpp() {
        let seed = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let mask = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        assert!(seedfill_morph(&seed, &mask, 0, 4).is_err());
    }

    #[test]
    fn test_seedfill_morph_invalid_connectivity() {
        let seed = create_1bpp_test();
        let mask = create_1bpp_test();
        assert!(seedfill_morph(&seed, &mask, 0, 6).is_err());
    }

    // --- morph_gradient tests ---

    #[test]
    fn test_morph_gradient_preserves_dimensions() {
        let pix = create_8bpp_test();
        let result = morph_gradient(&pix, 3, 3, 0).unwrap();
        assert_eq!(result.width(), pix.width());
        assert_eq!(result.height(), pix.height());
    }

    #[test]
    fn test_morph_gradient_requires_8bpp() {
        let pix = Pix::new(32, 32, PixelDepth::Bit1).unwrap();
        assert!(morph_gradient(&pix, 3, 3, 0).is_err());
    }

    #[test]
    fn test_morph_gradient_uniform_image_is_zero() {
        // Uniform gray image → gradient should be all zeros
        let pix = Pix::new(32, 32, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..32u32 {
            for x in 0..32u32 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pm.into();
        let result = morph_gradient(&pix, 3, 3, 0).unwrap();
        // Interior should be 0 (no gradient in uniform region)
        // Borders may have edge effects, check interior
        for y in 3..29u32 {
            for x in 3..29u32 {
                assert_eq!(result.get_pixel_unchecked(x, y), 0);
            }
        }
    }

    #[test]
    fn test_selective_conn_comp_fill_fills_hole() {
        let pix = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 2..14u32 {
            for x in 2..14u32 {
                pm.set_pixel_unchecked(x, y, 1);
            }
        }
        for y in 6..10u32 {
            for x in 6..10u32 {
                pm.set_pixel_unchecked(x, y, 0);
            }
        }
        let pix: Pix = pm.into();

        let out = selective_conn_comp_fill(&pix, 8, 1, 1).unwrap();
        assert_eq!(out.get_pixel_unchecked(7, 7), 1);
    }

    #[test]
    fn test_remove_matched_pattern_simple() {
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(4, 4, 1);
        let pix: Pix = pm.into();

        let pattern = Pix::new(1, 1, PixelDepth::Bit1).unwrap();
        let mut patm = pattern.try_into_mut().unwrap();
        patm.set_pixel_unchecked(0, 0, 1);
        let pattern: Pix = patm.into();

        let matches = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let mut mm = matches.try_into_mut().unwrap();
        mm.set_pixel_unchecked(4, 4, 1);
        let matches: Pix = mm.into();

        let out = remove_matched_pattern(&pix, &pattern, &matches, 0, 0, 0).unwrap();
        assert_eq!(out.get_pixel_unchecked(4, 4), 0);
    }

    #[test]
    fn test_display_matched_pattern_paints_color() {
        let pix = Pix::new(8, 8, PixelDepth::Bit1).unwrap();

        let pattern = Pix::new(1, 1, PixelDepth::Bit1).unwrap();
        let mut patm = pattern.try_into_mut().unwrap();
        patm.set_pixel_unchecked(0, 0, 1);
        let pattern: Pix = patm.into();

        let matches = Pix::new(8, 8, PixelDepth::Bit1).unwrap();
        let mut mm = matches.try_into_mut().unwrap();
        mm.set_pixel_unchecked(3, 2, 1);
        let matches: Pix = mm.into();

        let color = 0xff0000ff;
        let out = display_matched_pattern(&pix, &pattern, &matches, 0, 0, color, 1.0).unwrap();
        assert_eq!(out.depth(), PixelDepth::Bit32);
        assert_eq!(out.get_pixel_unchecked(3, 2), color);
    }

    #[test]
    fn test_pixa_extend_by_morph_dilate_iters() {
        let src = Pix::new(9, 9, PixelDepth::Bit1).unwrap();
        let mut sm = src.try_into_mut().unwrap();
        sm.set_pixel_unchecked(4, 4, 1);
        let mut pixa = Pixa::new();
        pixa.push(sm.into());

        let out = pixa_extend_by_morph(&pixa, MorphOpType::Dilate, 2, None, false).unwrap();
        assert_eq!(out.len(), 2);
        let p0 = out.get(0).unwrap();
        let p1 = out.get(1).unwrap();
        assert!(p1.count_pixels() >= p0.count_pixels());
    }

    #[test]
    fn test_pixa_extend_by_scaling_both() {
        let mut pixa = Pixa::new();
        pixa.push(Pix::new(10, 6, PixelDepth::Bit1).unwrap());

        let out = pixa_extend_by_scaling(&pixa, &[0.5, 2.0], ScaleDirection::BothDirections, false)
            .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out.get(0).unwrap().width(), 5);
        assert_eq!(out.get(0).unwrap().height(), 3);
        assert_eq!(out.get(1).unwrap().width(), 20);
        assert_eq!(out.get(1).unwrap().height(), 12);
    }

    #[test]
    fn test_run_histogram_morph_basic() {
        let pix = Pix::new(8, 1, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for x in 2..6u32 {
            pm.set_pixel_unchecked(x, 0, 1);
        }
        let pix: Pix = pm.into();

        let na = run_histogram_morph(&pix, RunType::On, RunDirection::Horizontal, 4).unwrap();
        assert!(!na.is_empty());
        assert_eq!(na.get(0), Some(0.0));
    }

    #[test]
    fn test_h_dome_extracts_peak() {
        let pix = Pix::new(7, 7, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..7u32 {
            for x in 0..7u32 {
                pm.set_pixel_unchecked(x, y, 10);
            }
        }
        pm.set_pixel_unchecked(3, 3, 40);
        let pix: Pix = pm.into();

        let out = h_dome(&pix, 15, 4).unwrap();
        assert_eq!(out.get_pixel_unchecked(0, 0), 0);
        assert!(out.get_pixel_unchecked(3, 3) > 0);
    }

    #[test]
    fn test_fast_tophat_uniform_zero() {
        let pix = Pix::new(12, 12, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..12u32 {
            for x in 0..12u32 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pm.into();

        let white = fast_tophat(&pix, 3, 3, TophatType::White).unwrap();
        let black = fast_tophat(&pix, 3, 3, TophatType::Black).unwrap();
        assert_eq!(white.get_pixel_unchecked(6, 6), 0);
        assert_eq!(black.get_pixel_unchecked(6, 6), 0);
    }

    #[test]
    fn test_pix_centroid_binary() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(1, 1, 1);
        pm.set_pixel_unchecked(3, 3, 1);
        let pix: Pix = pm.into();

        let (cx, cy) = pix_centroid(&pix).unwrap();
        assert!((cx - 2.0).abs() < 1e-6);
        assert!((cy - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_pix_centroid_gray_weighted() {
        let pix = Pix::new(4, 1, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 1);
        pm.set_pixel_unchecked(2, 0, 3);
        let pix: Pix = pm.into();

        let (cx, cy) = pix_centroid(&pix).unwrap();
        assert!((cx - 1.5).abs() < 1e-6);
        assert!(cy.abs() < 1e-6);
    }

    #[test]
    fn test_pixa_centroids() {
        let mut pixa = Pixa::new();

        let pix1 = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut p1m = pix1.try_into_mut().unwrap();
        p1m.set_pixel_unchecked(2, 1, 1);
        pixa.push(p1m.into());

        let pix2 = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut p2m = pix2.try_into_mut().unwrap();
        p2m.set_pixel_unchecked(0, 0, 1);
        p2m.set_pixel_unchecked(0, 2, 1);
        pixa.push(p2m.into());

        let pta = pixa_centroids(&pixa).unwrap();
        assert_eq!(pta.len(), 2);
        let (x0, y0) = pta.get(0).unwrap();
        let (x1, y1) = pta.get(1).unwrap();
        assert!((x0 - 2.0).abs() < 1e-6);
        assert!((y0 - 1.0).abs() < 1e-6);
        assert!(x1.abs() < 1e-6);
        assert!((y1 - 1.0).abs() < 1e-6);
    }
}
