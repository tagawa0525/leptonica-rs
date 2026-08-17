//! Colormap painting functions
//!
//! Functions for modifying colormapped images by repainting specific colors,
//! colorizing gray regions, or applying masks.
//!
//! # Reference
//!
//! Based on Leptonica's `paintcmap.c`.

use crate::color::coloring::PaintType;
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

/// Repaint the pixels of one colormap index inside a region.
///
/// Pixels in `region` whose index is `old_index` are given the index of
/// `new_color` instead. The colour is looked up in the colormap first and
/// only appended when it is not already present, so existing entries are
/// never overwritten and pixels **outside** the region keep `old_index`.
///
/// # Arguments
///
/// * `pix` - Colormapped image of depth 1, 2, 4 or 8 (mutable)
/// * `region` - Optional bounding box (None for entire image); clipped to
///   the image
/// * `old_index` - Colormap index whose pixels are repainted
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
    let d = pix.depth();
    if !matches!(
        d,
        PixelDepth::Bit1 | PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8
    ) {
        return Err(ColorError::UnsupportedDepth {
            expected: "1, 2, 4 or 8 bpp",
            actual: d.bits(),
        });
    }

    let (r, g, b) = new_color;
    let cmap = pix.colormap_mut().unwrap();
    let n = cmap.len();
    if old_index as usize >= n {
        return Err(ColorError::InvalidParameters(format!(
            "old_index {old_index} >= colormap size {n}"
        )));
    }

    // C: reuse the entry when the colour is already in the cmap, otherwise
    // append it. The old entry itself is left untouched.
    let new_index = match cmap.get_index(r, g, b) {
        Some(index) => index,
        None => cmap
            .add_color(RgbaQuad::rgb(r, g, b))
            .map_err(|e| ColorError::InvalidParameters(e.to_string()))?,
    } as u32;

    let w = pix.width();
    let h = pix.height();
    let (x1, y1, x2, y2) = match region {
        None => (0i64, 0i64, w as i64 - 1, h as i64 - 1),
        Some(b) => {
            let x1 = b.x as i64;
            let y1 = b.y as i64;
            (x1, y1, x1 + b.w as i64 - 1, y1 + b.h as i64 - 1)
        }
    };

    for y in y1.max(0)..=y2.min(h as i64 - 1) {
        for x in x1.max(0)..=x2.min(w as i64 - 1) {
            let (x, y) = (x as u32, y as u32);
            if pix.get_pixel_unchecked(x, y) == old_index {
                pix.set_pixel_unchecked(x, y, new_index);
            }
        }
    }

    Ok(())
}

/// Colorize the gray pixels of a colormapped 8 bpp image inside a set of
/// regions.
///
/// A colorized entry is appended to the colormap for every gray entry (see
/// [`add_colorized_gray_to_cmap`]), then the pixels inside each box are
/// remapped through it. Pixels whose index is at or beyond the original
/// colormap size are skipped, so overlapping boxes do not colorize twice.
///
/// # Arguments
///
/// * `pix` - colormapped 8 bpp image (mutable)
/// * `boxa` - regions to colorize; each is clipped to the image
/// * `paint_type` - colorize the light or the dark pixels
/// * `color` - target RGB color
///
/// # Reference
///
/// C Leptonica: `pixColorGrayRegionsCmap()`
pub fn pix_color_gray_regions_cmap(
    _pix: &mut PixMut,
    _boxa: &crate::core::Boxa,
    _paint_type: PaintType,
    _color: (u8, u8, u8),
) -> ColorResult<()> {
    Err(ColorError::InvalidParameters("not yet implemented".into()))
}

/// Colorize the gray pixels of a colormapped image, optionally restricted to
/// one box.
///
/// 2 and 4 bpp inputs are promoted to 8 bpp first, as in C. `None` colorizes
/// the whole image.
///
/// # Reference
///
/// C Leptonica: `pixColorGrayCmap()`
pub fn pix_color_gray_cmap(
    _pix: &mut PixMut,
    _region: Option<&Box>,
    _paint_type: PaintType,
    _color: (u8, u8, u8),
) -> ColorResult<()> {
    Err(ColorError::InvalidParameters("not yet implemented".into()))
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
/// Append a colorized version of every gray colormap entry, returning the
/// index map from old entry to new entry.
///
/// An entry qualifies when `r == g == b` and, following C, it is not the
/// extreme that the transform leaves unchanged: value 0 for
/// [`PaintType::Light`] and value 255 for [`PaintType::Dark`]. Non-qualifying
/// entries map to [`CMAP_NO_REMAP`] (C's sentinel 256).
///
/// The colorized colour is added with C's `pixcmapAddNewColor` semantics: an
/// identical entry already in the colormap is reused rather than duplicated,
/// and a full colormap is an error.
///
/// # Reference
///
/// C Leptonica: `addColorizedGrayToCmap()`
pub fn add_colorized_gray_to_cmap(
    _cmap: &mut PixColormap,
    _paint_type: PaintType,
    _color: (u8, u8, u8),
) -> ColorResult<Vec<u32>> {
    Err(ColorError::InvalidParameters("not yet implemented".into()))
}

/// Sentinel used by [`add_colorized_gray_to_cmap`] for entries that are not
/// remapped (C stores 256 in the numa).
pub const CMAP_NO_REMAP: u32 = 256;

/// C `pixcmapAddNewColor`: reuse an identical entry, otherwise append.
#[allow(dead_code)]
fn add_new_color(cmap: &mut PixColormap, color: (u8, u8, u8)) -> ColorResult<u32> {
    let (r, g, b) = color;
    match cmap.get_index(r, g, b) {
        Some(index) => Ok(index as u32),
        None => cmap
            .add_color(RgbaQuad::rgb(r, g, b))
            .map(|i| i as u32)
            .map_err(|_| ColorError::InvalidParameters("no room; colormap full".into())),
    }
}

/// The per-component colorization C applies to a gray colormap entry.
///
/// C evaluates `rval * (l_float32)erval / 255.` for [`PaintType::Light`] and
/// `rval + (l_int32)((255. - rval) * (l_float32)erval / 255.)` for
/// [`PaintType::Dark`], truncating rather than rounding. The `255.` literals
/// are doubles, so the dark case is evaluated in double precision.
#[allow(dead_code)]
fn colorize_gray_triple(
    paint_type: PaintType,
    target: (u8, u8, u8),
    entry: (u8, u8, u8),
) -> (u8, u8, u8) {
    let comp = |t: u8, e: u8| -> u8 {
        match paint_type {
            PaintType::Light => (t as f64 * e as f32 as f64 / 255.0) as u8,
            PaintType::Dark => t + ((255.0 - t as f64) * e as f32 as f64 / 255.0) as u8,
        }
    };
    (
        comp(target.0, entry.0),
        comp(target.1, entry.1),
        comp(target.2, entry.2),
    )
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
/// Pixels where the mask is ON, offset by `(x_offset, y_offset)`, are given
/// the index of `new_color`. As in C the colour is looked up in the colormap
/// first and only appended when absent, so repainting with a colour that is
/// already present reuses its entry instead of duplicating it. When the
/// colormap is full and the colour is not present this is an error — C's
/// nearest-colour fallback lives one level up, not here.
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
    let d = pix.depth();
    if !matches!(d, PixelDepth::Bit2 | PixelDepth::Bit4 | PixelDepth::Bit8) {
        return Err(ColorError::UnsupportedDepth {
            expected: "2, 4 or 8 bpp",
            actual: d.bits(),
        });
    }

    let (r, g, b) = new_color;
    let new_idx = {
        let cmap = pix.colormap_mut().unwrap();
        match cmap.get_index(r, g, b) {
            Some(index) => index as u32,
            None => cmap
                .add_color(RgbaQuad::rgb(r, g, b))
                .map_err(|_| ColorError::InvalidParameters("no room in cmap".into()))?
                as u32,
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
