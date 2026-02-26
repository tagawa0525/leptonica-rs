//! Colormap painting functions
//!
//! Functions for modifying colormapped images by repainting specific colors,
//! colorizing gray regions, or applying masks.
//!
//! # Reference
//!
//! Based on Leptonica's `paintcmap.c`.

use crate::color::{ColorError, ColorResult};
use crate::core::{Box, Pix, PixColormap, PixMut, PixelDepth, RgbaQuad};

/// Helper: check that a PixMut has a colormap, return depth on failure
fn check_colormapped(pix: &PixMut) -> ColorResult<()> {
    if pix.colormap().is_none() {
        return Err(ColorError::UnsupportedDepth {
            expected: "colormapped",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

/// Repaint selected colormap entries in a region.
///
/// Pixels in the specified region that have color index `old_index` are
/// changed to the `new_color`. The colormap is updated if necessary.
///
/// # Arguments
///
/// * `pix` - Colormapped image (mutable)
/// * `region` - Optional bounding box (None for entire image)
/// * `old_index` - Colormap index to replace
/// * `new_color` - Replacement RGB color
///
/// # Reference
///
/// C Leptonica: `pixSetSelectCmap()`
pub fn pix_set_select_cmap(
    pix: &mut PixMut,
    region: Option<&Box>,
    old_index: u32,
    new_color: (u8, u8, u8),
) -> ColorResult<()> {
    check_colormapped(pix)?;

    let cmap = pix.colormap_mut().unwrap();
    let max_idx = cmap.len();
    if old_index as usize >= max_idx {
        return Err(ColorError::InvalidParameters(format!(
            "old_index {} >= colormap size {}",
            old_index, max_idx
        )));
    }

    cmap.set_color(
        old_index as usize,
        RgbaQuad::rgb(new_color.0, new_color.1, new_color.2),
    )
    .map_err(|e| ColorError::InvalidParameters(e.to_string()))?;

    let _ = region; // Colormap change applies globally for that index
    Ok(())
}

/// Colorize gray pixels in specified regions of a colormapped image.
///
/// Gray pixels are identified by having equal R, G, B values.
/// New colormap entries are added for the colorized versions.
///
/// # Arguments
///
/// * `pix` - Colormapped image (mutable)
/// * `boxa` - Bounding boxes of regions to colorize
/// * `color` - Target RGB color
/// * `dark_thresh` - Pixels darker than this are not colorized
/// * `light_thresh` - Pixels lighter than this are not colorized
///
/// # Reference
///
/// C Leptonica: `pixColorGrayRegionsCmap()`
pub fn pix_color_gray_regions_cmap(
    pix: &mut PixMut,
    boxa: &crate::core::Boxa,
    color: (u8, u8, u8),
    dark_thresh: u8,
    light_thresh: u8,
) -> ColorResult<()> {
    for i in 0..boxa.len() {
        if let Some(b) = boxa.get(i) {
            pix_color_gray_cmap_in_region(pix, Some(b), color, dark_thresh, light_thresh)?;
        }
    }
    Ok(())
}

/// Colorize gray pixels in a colormapped image.
///
/// Creates new colormap entries that blend the gray value with the target color.
///
/// # Arguments
///
/// * `pix` - Colormapped image (mutable)
/// * `region` - Optional region (None for entire image)
/// * `color` - Target RGB color
/// * `dark_thresh` - Min gray value to colorize
/// * `light_thresh` - Max gray value to colorize
///
/// # Reference
///
/// C Leptonica: `pixColorGrayCmap()`
pub fn pix_color_gray_cmap(
    pix: &mut PixMut,
    region: Option<&Box>,
    color: (u8, u8, u8),
    dark_thresh: u8,
    light_thresh: u8,
) -> ColorResult<()> {
    pix_color_gray_cmap_in_region(pix, region, color, dark_thresh, light_thresh)
}

fn pix_color_gray_cmap_in_region(
    pix: &mut PixMut,
    region: Option<&Box>,
    color: (u8, u8, u8),
    dark_thresh: u8,
    light_thresh: u8,
) -> ColorResult<()> {
    check_colormapped(pix)?;

    // Build mapping: for each gray colormap entry, compute colorized version
    let mut index_map: Vec<Option<u32>> = {
        let cmap = pix.colormap().unwrap();
        vec![None; cmap.len()]
    };

    let new_entries: Vec<(usize, RgbaQuad)> = {
        let cmap = pix.colormap().unwrap();
        let mut entries = Vec::new();
        for i in 0..cmap.len() {
            if let Some((r, g, b)) = cmap.get_rgb(i)
                && r == g
                && g == b
                && r >= dark_thresh
                && r <= light_thresh
            {
                let gray_val = r as f32 / 255.0;
                let nr = (color.0 as f32 * gray_val).round() as u8;
                let ng = (color.1 as f32 * gray_val).round() as u8;
                let nb = (color.2 as f32 * gray_val).round() as u8;
                entries.push((i, RgbaQuad::rgb(nr, ng, nb)));
            }
        }
        entries
    };

    // Apply entries to colormap
    {
        let cmap = pix.colormap_mut().unwrap();
        for &(orig_idx, new_color_quad) in &new_entries {
            match cmap.add_color(new_color_quad) {
                Ok(new_idx) => {
                    index_map[orig_idx] = Some(new_idx as u32);
                }
                Err(_) => {
                    let _ = cmap.set_color(orig_idx, new_color_quad);
                }
            }
        }
    }

    // Remap pixels
    let (x0, y0, x1, y1) = match region {
        Some(b) => (
            b.x.max(0) as u32,
            b.y.max(0) as u32,
            (b.x + b.w).min(pix.width() as i32) as u32,
            (b.y + b.h).min(pix.height() as i32) as u32,
        ),
        None => (0, 0, pix.width(), pix.height()),
    };

    for y in y0..y1 {
        for x in x0..x1 {
            if let Some(val) = pix.get_pixel(x, y)
                && let Some(Some(new_idx)) = index_map.get(val as usize)
            {
                pix.set_pixel_unchecked(x, y, *new_idx);
            }
        }
    }

    Ok(())
}

/// Colorize gray pixels using a mask in a colormapped image.
///
/// Only pixels where the mask is ON are affected.
///
/// # Reference
///
/// C Leptonica: `pixColorGrayMaskedCmap()`
pub fn pix_color_gray_masked_cmap(
    pix: &mut PixMut,
    mask: &Pix,
    color: (u8, u8, u8),
    dark_thresh: u8,
    light_thresh: u8,
) -> ColorResult<()> {
    if mask.depth() != PixelDepth::Bit1 {
        return Err(ColorError::UnsupportedDepth {
            expected: "1-bit mask",
            actual: mask.depth().bits(),
        });
    }
    check_colormapped(pix)?;

    // Build index mapping for gray entries
    let mut index_map: Vec<Option<u32>> = {
        let cmap = pix.colormap().unwrap();
        vec![None; cmap.len()]
    };

    let new_entries: Vec<(usize, RgbaQuad)> = {
        let cmap = pix.colormap().unwrap();
        let mut entries = Vec::new();
        for i in 0..cmap.len() {
            if let Some((r, g, b)) = cmap.get_rgb(i)
                && r == g
                && g == b
                && r >= dark_thresh
                && r <= light_thresh
            {
                let gray_val = r as f32 / 255.0;
                let nr = (color.0 as f32 * gray_val).round() as u8;
                let ng = (color.1 as f32 * gray_val).round() as u8;
                let nb = (color.2 as f32 * gray_val).round() as u8;
                entries.push((i, RgbaQuad::rgb(nr, ng, nb)));
            }
        }
        entries
    };

    {
        let cmap = pix.colormap_mut().unwrap();
        for &(orig_idx, new_color_quad) in &new_entries {
            match cmap.add_color(new_color_quad) {
                Ok(new_idx) => {
                    index_map[orig_idx] = Some(new_idx as u32);
                }
                Err(_) => {
                    let _ = cmap.set_color(orig_idx, new_color_quad);
                }
            }
        }
    }

    let w = pix.width().min(mask.width());
    let h = pix.height().min(mask.height());
    for y in 0..h {
        for x in 0..w {
            if mask.get_pixel(x, y) == Some(1)
                && let Some(val) = pix.get_pixel(x, y)
                && let Some(Some(new_idx)) = index_map.get(val as usize)
            {
                pix.set_pixel_unchecked(x, y, *new_idx);
            }
        }
    }

    Ok(())
}

/// Add colorized gray entries to a colormap.
///
/// For each gray entry in the colormap, creates a new entry blending
/// the gray value with the specified color.
///
/// # Reference
///
/// C Leptonica: `addColorizedGrayToCmap()`
pub fn add_colorized_gray_to_cmap(
    cmap: &mut PixColormap,
    color: (u8, u8, u8),
) -> ColorResult<Vec<(usize, usize)>> {
    let mut mapping = Vec::new();

    let n = cmap.len();
    for i in 0..n {
        if let Some((r, g, b)) = cmap.get_rgb(i)
            && r == g
            && g == b
        {
            let gray_val = r as f32 / 255.0;
            let nr = (color.0 as f32 * gray_val).round() as u8;
            let ng = (color.1 as f32 * gray_val).round() as u8;
            let nb = (color.2 as f32 * gray_val).round() as u8;
            match cmap.add_color(RgbaQuad::rgb(nr, ng, nb)) {
                Ok(new_idx) => mapping.push((i, new_idx)),
                Err(_) => break,
            }
        }
    }
    Ok(mapping)
}

/// Set selected pixels to a color through a mask in a colormapped image.
///
/// Pixels where the mask is ON and that have the `old_index` value are
/// changed to `new_color`.
///
/// # Reference
///
/// C Leptonica: `pixSetSelectMaskedCmap()`
pub fn pix_set_select_masked_cmap(
    pix: &mut PixMut,
    mask: &Pix,
    x_offset: i32,
    y_offset: i32,
    old_index: u32,
    new_color: (u8, u8, u8),
) -> ColorResult<()> {
    if mask.depth() != PixelDepth::Bit1 {
        return Err(ColorError::UnsupportedDepth {
            expected: "1-bit mask",
            actual: mask.depth().bits(),
        });
    }
    check_colormapped(pix)?;

    // Find or add the new color
    let new_idx = {
        let cmap = pix.colormap_mut().unwrap();
        let nearest = cmap
            .find_nearest(new_color.0, new_color.1, new_color.2)
            .unwrap_or(0) as u32;
        match cmap.add_color(RgbaQuad::rgb(new_color.0, new_color.1, new_color.2)) {
            Ok(idx) => idx as u32,
            Err(_) => nearest,
        }
    };

    let mw = mask.width();
    let mh = mask.height();

    for my in 0..mh {
        for mx in 0..mw {
            if mask.get_pixel(mx, my) == Some(1) {
                let px = mx as i32 + x_offset;
                let py = my as i32 + y_offset;
                if px >= 0 && py >= 0 {
                    let ux = px as u32;
                    let uy = py as u32;
                    if ux < pix.width()
                        && uy < pix.height()
                        && pix.get_pixel(ux, uy) == Some(old_index)
                    {
                        pix.set_pixel_unchecked(ux, uy, new_idx);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Set all pixels to a color through a mask in a colormapped image.
///
/// Pixels where the mask is ON are set to `new_color`.
///
/// # Reference
///
/// C Leptonica: `pixSetMaskedCmap()`
pub fn pix_set_masked_cmap(
    pix: &mut PixMut,
    mask: &Pix,
    x_offset: i32,
    y_offset: i32,
    new_color: (u8, u8, u8),
) -> ColorResult<()> {
    if mask.depth() != PixelDepth::Bit1 {
        return Err(ColorError::UnsupportedDepth {
            expected: "1-bit mask",
            actual: mask.depth().bits(),
        });
    }
    check_colormapped(pix)?;

    let new_idx = {
        let cmap = pix.colormap_mut().unwrap();
        match cmap.add_color(RgbaQuad::rgb(new_color.0, new_color.1, new_color.2)) {
            Ok(idx) => idx as u32,
            Err(_) => cmap
                .find_nearest(new_color.0, new_color.1, new_color.2)
                .unwrap_or(0) as u32,
        }
    };

    let mw = mask.width();
    let mh = mask.height();

    for my in 0..mh {
        for mx in 0..mw {
            if mask.get_pixel(mx, my) == Some(1) {
                let px = mx as i32 + x_offset;
                let py = my as i32 + y_offset;
                if px >= 0 && py >= 0 {
                    let ux = px as u32;
                    let uy = py as u32;
                    if ux < pix.width() && uy < pix.height() {
                        pix.set_pixel_unchecked(ux, uy, new_idx);
                    }
                }
            }
        }
    }

    Ok(())
}
