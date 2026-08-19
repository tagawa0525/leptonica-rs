//! Checkerboard corner detection
//!
//! Find corners where four squares meet in a checkerboard pattern
//! using hit-miss morphology.
//!
//! # See also
//! C Leptonica: `checkerboard.c`

use crate::core::Pix;
use crate::core::{CornerLocation, Pta};
use crate::morph;
use crate::morph::{MorphOpType, Sel, SelElement};
use crate::region;
use crate::region::{ConnectivityType, RegionError, RegionResult};

/// Find corners in a checkerboard pattern
///
/// Uses hit-miss transforms with diagonal structuring elements to
/// detect the points where four squares meet.
///
/// # Arguments
/// * `pix` - checkerboard image (any depth, converted to 1 bpp internally)
/// * `size` - size of HMT sel (>= 7, default 7)
/// * `dilation` - dilation size for hit/miss squares (1-5, typically 1 or 3)
/// * `nsels` - number of sels to use (2 or 4; use 4 for >20° rotation)
///
/// # Returns
/// `(corner_pix, corner_points)` - 1 bpp image of corners and their coordinates
///
/// # See also
/// C Leptonica: `pixFindCheckerboardCorners()` in `checkerboard.c`
pub fn find_checkerboard_corners(
    pix: &Pix,
    size: u32,
    dilation: u32,
    nsels: u32,
) -> RegionResult<(Pix, Pta)> {
    let size = if size == 0 { 7 } else { size };
    if size < 7 {
        return Err(RegionError::InvalidParameters("size must be >= 7".into()));
    }
    if !(1..=5).contains(&dilation) {
        return Err(RegionError::InvalidParameters(
            "dilation must be in [1..5]".into(),
        ));
    }
    if nsels != 2 && nsels != 4 {
        return Err(RegionError::InvalidParameters(
            "nsels must be 2 or 4".into(),
        ));
    }

    // Generate hit-miss sels for corners
    let sels = make_checkerboard_corner_sels(size, dilation, nsels)?;

    // Do HMT to find corner locations
    let pix1 = morph::union_of_morph_ops(pix, &sels, MorphOpType::HitMiss)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;

    // Remove large noise CCs
    let pix2 = region::pix_select_by_size(
        &pix1,
        size as i32,
        size as i32,
        ConnectivityType::EightWay,
        region::SizeSelectType::IfBoth,
        region::SizeRelation::LessThanOrEqual,
    )?;

    // Thin remaining CCs to single pixels
    let pix3 = morph::thin_connected(
        &pix2,
        morph::ThinType::Foreground,
        morph::Connectivity::Eight,
        0,
    )
    .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;

    // Extract center coordinates of each CC
    // C: boxaExtractCorners(boxa1, L_BOX_CENTER)
    let (boxa, _) = region::conncomp_pixa(&pix3, ConnectivityType::EightWay)?;
    let pta = boxa.extract_corners(CornerLocation::Center);

    Ok((pix3, pta))
}

/// Generate the hit-miss structuring elements for corner detection.
///
/// C builds each sel from a *sparse* pair of dilated pixels rather than
/// filled quadrants: a 1 bpp mask with two set pixels near opposite corners
/// (or on the mid-line, for the cross sels), dilated by a `dilation` brick,
/// gives the hits; the same mask rotated 90 degrees gives the misses. The
/// second sel of each pair swaps the two roles. Everything else in the
/// `size` x `size` window stays don't-care, and the origin is the centre.
///
/// C equivalent: `makeCheckerboardCornerPixa()` + `selCreateFromColorPix()`
/// in `checkerboard.c` / `sel1.c`
fn make_checkerboard_corner_sels(size: u32, dilation: u32, nsels: u32) -> RegionResult<Vec<Sel>> {
    let half = size / 2;
    let mut sels = Vec::with_capacity(nsels as usize);

    // Diagonal (negative slope) mask: the UL and LR inset corners.
    let diag = corner_mask(size, dilation, &[(1, 1), (size - 2, size - 2)])?;
    let anti = rotate_mask_90(&diag)?;
    sels.push(sel_from_masks(size, half, &diag, &anti)?);
    sels.push(sel_from_masks(size, half, &anti, &diag)?);

    if nsels == 4 {
        // Vertical mask: two points on the mid-column.
        let vert = corner_mask(size, dilation, &[(half, 1), (half, size - 2)])?;
        let horiz = rotate_mask_90(&vert)?;
        sels.push(sel_from_masks(size, half, &vert, &horiz)?);
        sels.push(sel_from_masks(size, half, &horiz, &vert)?);
    }

    Ok(sels)
}

/// A `size` x `size` 1 bpp mask with `points` set and dilated by a
/// `dilation` x `dilation` brick (C only dilates when `dilation > 1`).
fn corner_mask(size: u32, dilation: u32, points: &[(u32, u32)]) -> RegionResult<Pix> {
    let pix = Pix::new(size, size, crate::core::PixelDepth::Bit1).map_err(RegionError::Core)?;
    let mut pm = pix.try_into_mut().unwrap();
    for &(x, y) in points {
        pm.set_pixel_unchecked(x, y, 1);
    }
    let pix: Pix = pm.into();
    if dilation > 1 {
        morph::dilate_brick(&pix, dilation, dilation)
            .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))
    } else {
        Ok(pix)
    }
}

/// C `pixRotate90(pix, 1)`: a clockwise quarter turn.
fn rotate_mask_90(pix: &Pix) -> RegionResult<Pix> {
    crate::transform::rotate_90(pix, true)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))
}

/// Combine a hit mask and a miss mask into a sel centred on `half`.
fn sel_from_masks(size: u32, half: u32, hits: &Pix, misses: &Pix) -> RegionResult<Sel> {
    let mut sel = Sel::new(size, size)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;
    for y in 0..size {
        for x in 0..size {
            if hits.get_pixel_unchecked(x, y) == 1 {
                sel.set_element(x, y, SelElement::Hit);
            } else if misses.get_pixel_unchecked(x, y) == 1 {
                sel.set_element(x, y, SelElement::Miss);
            }
        }
    }
    sel.set_origin(half, half)
        .map_err(|e| RegionError::Core(crate::core::Error::NotSupported(e.to_string())))?;
    Ok(sel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morph::SelElement;

    /// C builds each corner sel from two *dilated pixels* per role, not from
    /// filled quadrants: `pixSetPixel(pix2, 1, 1)` and
    /// `pixSetPixel(pix2, size - 2, size - 2)`, dilated by a
    /// `dilation x dilation` brick, are the hits, and the same mask rotated
    /// 90 degrees gives the misses. Everything else is don't-care.
    #[test]
    fn test_corner_sels_are_sparse_like_c() {
        let sels = make_checkerboard_corner_sels(15, 3, 2).unwrap();
        assert_eq!(sels.len(), 2);
        for sel in &sels {
            assert_eq!(sel.width(), 15);
            assert_eq!(sel.height(), 15);
            assert_eq!(sel.origin_x(), 7);
            assert_eq!(sel.origin_y(), 7);
            // Two 3x3 blocks of hits and two of misses.
            assert_eq!(sel.hit_count(), 18);
            assert_eq!(sel.miss_count(), 18);
        }
        // The pair swaps hits and misses.
        assert_eq!(
            sels[0].get_element(0, 0).unwrap(),
            sels[1].get_element(14, 0).unwrap()
        );
    }

    /// Without dilation each role is a single pixel per corner.
    #[test]
    fn test_corner_sels_undilated() {
        let sels = make_checkerboard_corner_sels(15, 1, 4).unwrap();
        assert_eq!(sels.len(), 4);
        for sel in &sels {
            assert_eq!(sel.hit_count(), 2);
            assert_eq!(sel.miss_count(), 2);
        }
        // Diagonal sel: hits on the negative-slope inset corners.
        assert_eq!(sels[0].get_element(1, 1).unwrap(), SelElement::Hit);
        assert_eq!(sels[0].get_element(13, 13).unwrap(), SelElement::Hit);
        assert_eq!(sels[0].get_element(13, 1).unwrap(), SelElement::Miss);
        assert_eq!(sels[0].get_element(1, 13).unwrap(), SelElement::Miss);
        // Cross sel: hits on the mid-column.
        assert_eq!(sels[2].get_element(7, 1).unwrap(), SelElement::Hit);
        assert_eq!(sels[2].get_element(7, 13).unwrap(), SelElement::Hit);
        assert_eq!(sels[2].get_element(1, 7).unwrap(), SelElement::Miss);
        assert_eq!(sels[2].get_element(13, 7).unwrap(), SelElement::Miss);
    }
}
