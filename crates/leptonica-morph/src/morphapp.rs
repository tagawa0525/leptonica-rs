//! Morphological application functions
//!
//! Higher-level morphological operations including:
//! - Sequence operations with masking
//! - Union and intersection of morphological ops over a set of SELs
//! - Seedfill via dilation (binary reconstruction)
//! - Grayscale morphological gradient

use crate::{
    MorphError, MorphResult, Sel, close, dilate, erode, gradient_gray, hit_miss_transform,
    morph_sequence, open,
};
use leptonica_core::{Pix, PixelDepth};

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

/// Grayscale morphological gradient: dilation - original (after optional smoothing).
///
/// Emphasises edges and transitions in an 8-bpp image. Optional block
/// convolution smoothing can be applied first to reduce noise.
///
/// # Arguments
/// * `pix` - 8 bpp input image
/// * `hsize` - SEL width (will be rounded up to odd if even)
/// * `vsize` - SEL height (will be rounded up to odd if even)
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
    // Round up even sizes to odd (same convention as gradient_gray internally)
    let hsize = if hsize.is_multiple_of(2) {
        hsize + 1
    } else {
        hsize
    };
    let vsize = if vsize.is_multiple_of(2) {
        vsize + 1
    } else {
        vsize
    };
    gradient_gray(pix, hsize, vsize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica_core::{Pix, PixelDepth};

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
        let result_direct = crate::morph_sequence(&pix, "D3.3").unwrap();
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
        let union = union_of_morph_ops(&pix, &[sel.clone()], MorphOpType::Dilate).unwrap();
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
        let result = intersection_of_morph_ops(&pix, &[sel.clone()], MorphOpType::Erode).unwrap();
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
}
