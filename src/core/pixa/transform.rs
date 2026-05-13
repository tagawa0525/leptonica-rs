//! Pixa transform helpers (plan 107 / C pixafunc1.c + pixafunc2.c).
//!
//! Each method iterates the source Pixa, applies the corresponding Pix
//! function, and rebuilds the Boxa with matching coordinates.

use crate::core::box_::{Box, Boxa};
use crate::core::error::{Error, Result};
use crate::core::pix::rop::InColor;

use super::Pixa;

impl Pixa {
    /// Scale each Pix by `(scale_x, scale_y)` and rebuild matching boxes.
    ///
    /// C Leptonica equivalent: `pixaScale`.
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Result<Pixa> {
        if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
            return Err(Error::InvalidParameter(format!(
                "scale factors must be > 0 (got {scale_x}, {scale_y})"
            )));
        }
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let scaled = crate::transform::scale::scale(
                pix,
                scale_x,
                scale_y,
                crate::transform::scale::ScaleMethod::Auto,
            )
            .map_err(|e| Error::InvalidParameter(format!("scale failed: {e}")))?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(scaled, scale_box(&b, scale_x, scale_y));
        }
        Ok(out)
    }

    /// Scale each Pix by sampling.
    ///
    /// C Leptonica equivalent: `pixaScaleBySampling`.
    pub fn scale_by_sampling(&self, scale_x: f32, scale_y: f32) -> Result<Pixa> {
        if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
            return Err(Error::InvalidParameter(format!(
                "scale factors must be > 0 (got {scale_x}, {scale_y})"
            )));
        }
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let scaled = crate::transform::scale::scale_by_sampling(pix, scale_x, scale_y)
                .map_err(|e| Error::InvalidParameter(format!("scale_by_sampling failed: {e}")))?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(scaled, scale_box(&b, scale_x, scale_y));
        }
        Ok(out)
    }

    /// Orthogonal rotation by `quads` * 90 degrees (0..=3).
    ///
    /// `quads == 0` returns a deep-cloned Pixa (independent pixel buffers),
    /// matching C `pixaRotateOrth`'s `pixaCopy(pixas, L_COPY)`.
    ///
    /// C Leptonica equivalent: `pixaRotateOrth`.
    pub fn rotate_orth(&self, quads: u32) -> Result<Pixa> {
        if quads > 3 {
            return Err(Error::InvalidParameter(format!(
                "quads must be in 0..=3 (got {quads})"
            )));
        }
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let b = self.boxa().get(i).copied().unwrap_or_default();
            if quads == 0 {
                // C `pixaCopy(pixas, L_COPY)` produces fresh pixel data.
                out.push_with_box(pix.deep_clone(), b);
            } else {
                let rotated = crate::transform::rotate::rotate_orth(pix, quads)
                    .map_err(|e| Error::InvalidParameter(format!("rotate_orth failed: {e}")))?;
                let new_box = b
                    .rotate_orth(pix.width() as i32, pix.height() as i32, quads as i32)
                    .unwrap_or_default();
                out.push_with_box(rotated, new_box);
            }
        }
        Ok(out)
    }

    /// Translate each Pix by `(hshift, vshift)`. `incolor` controls the
    /// background brought in by the shift.
    ///
    /// `(0, 0)` shift returns a deep-cloned Pixa (independent pixel
    /// buffers), matching `Pix::translate(0, 0, ...)`'s deep-copy
    /// semantics.
    ///
    /// C Leptonica equivalent: `pixaTranslate`.
    pub fn translate(&self, hshift: i32, vshift: i32, incolor: InColor) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let shifted = pix.translate(hshift, vshift, incolor);
            let b = self.boxa().get(i).copied().unwrap_or_default();
            let new_box = Box::new_unchecked(b.x + hshift, b.y + vshift, b.w, b.h);
            out.push_with_box(shifted, new_box);
        }
        Ok(out)
    }

    /// Convert every Pix to 1 bpp using a global threshold.
    ///
    /// Equivalent to calling `Pix::convert_to_1_by_sampling(1, thresh)`.
    /// C Leptonica equivalent: `pixaConvertTo1`.
    pub fn convert_to_1(&self, thresh: u32) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let converted = pix.convert_to_1_by_sampling(1, thresh)?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(converted, b);
        }
        Ok(out)
    }

    /// Convert every Pix to 8 bpp.
    ///
    /// Currently `cmap_flag` is accepted for C API parity but does not
    /// alter behaviour: every output Pix is 8 bpp without an attached
    /// gray colormap. Full cmap handling (matching the `cmap_flag = 1`
    /// path of C `pixConvertTo8`) is deferred to plan 107b.
    ///
    /// C Leptonica equivalent: `pixaConvertTo8`.
    pub fn convert_to_8(&self, cmap_flag: bool) -> Result<Pixa> {
        let _ = cmap_flag;
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let converted = pix.convert_to_8()?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(converted, b);
        }
        Ok(out)
    }

    /// Convert every Pix to 32 bpp.
    ///
    /// C Leptonica equivalent: `pixaConvertTo32`.
    pub fn convert_to_32(&self) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let converted = pix.convert_to_32()?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(converted, b);
        }
        Ok(out)
    }

    /// Add a uniform border to every Pix.
    ///
    /// `val` is the pixel value used to fill the border, expressed in
    /// the per-Pix native format. Each Box is shifted by
    /// `(-left, -top)` because the new image origin moves by that
    /// offset relative to the original coordinate system.
    ///
    /// C Leptonica equivalent: `pixaAddBorderGeneral`.
    pub fn add_border_general(
        &self,
        left: u32,
        right: u32,
        top: u32,
        bot: u32,
        val: u32,
    ) -> Result<Pixa> {
        // Border widths must fit in i32 because the Box shift uses
        // signed subtraction. Fail loudly on overflow rather than
        // silently truncating coordinates.
        let left_i = i32::try_from(left)
            .map_err(|_| Error::InvalidParameter(format!("left border {left} overflows i32")))?;
        let top_i = i32::try_from(top)
            .map_err(|_| Error::InvalidParameter(format!("top border {top} overflows i32")))?;

        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let bordered = pix.add_border_general(left, right, top, bot, val)?;
            let b = pix_box_or_full(pix, self.boxa().get(i).copied());
            let new_box = Box::new(b.x - left_i, b.y - top_i, b.w, b.h)?;
            out.push_with_box(bordered, new_box);
        }
        Ok(out)
    }

    /// Clip every Pix to its foreground bounding region.
    ///
    /// Returns `(pixa, boxa)`: the clipped Pixa and the per-entry crop
    /// Box (in original coordinates). Pix without any foreground are
    /// kept as a deep clone with a Box that covers the whole image.
    ///
    /// C Leptonica equivalent: `pixaClipToForeground`.
    pub fn clip_to_foreground_all(&self) -> Result<(Pixa, Boxa)> {
        let n = self.pix_slice().len();
        let mut pixad = Pixa::with_capacity(n);
        let mut boxa = Boxa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            match pix.clip_to_foreground()? {
                Some((clipped, b)) => {
                    pixad.push_with_box(clipped, b);
                    boxa.push(b);
                }
                None => {
                    let b = full_image_box(pix)?;
                    pixad.push_with_box(pix.deep_clone(), b);
                    boxa.push(b);
                }
            }
        }
        Ok((pixad, boxa))
    }

    /// Convert every Pix to the given target depth.
    ///
    /// Only `depth = 8` or `depth = 32` is supported, matching C
    /// `pixaConvertToGivenDepth`.
    ///
    /// C Leptonica equivalent: `pixaConvertToGivenDepth`.
    pub fn convert_to_given_depth(&self, depth: u32) -> Result<Pixa> {
        if depth != 8 && depth != 32 {
            return Err(Error::InvalidParameter(format!(
                "depth must be 8 or 32 (got {depth})"
            )));
        }
        if depth == 8 {
            self.convert_to_8(false)
        } else {
            self.convert_to_32()
        }
    }

    /// Bring every Pix to a common depth.
    ///
    /// Strips colormaps first (if any), then promotes every Pix to
    /// 8 bpp when the rendered depth is <= 16, else 32 bpp.
    /// Errors when the Pixa is empty (matches C).
    ///
    /// C Leptonica equivalent: `pixaConvertToSameDepth`.
    pub fn convert_to_same_depth(&self) -> Result<Pixa> {
        let n = self.pix_slice().len();
        if n == 0 {
            return Err(Error::InvalidParameter("pixa is empty".into()));
        }
        // 1. Drop colormaps by converting through 8/32 depending on the
        //    rendering depth.
        let rd = self.get_rendering_depth()?;
        let has_cmap = self.any_colormaps();
        let stage1: Pixa = if has_cmap {
            let mut tmp = Pixa::with_capacity(n);
            for i in 0..n {
                let pix = &self.pix_slice()[i];
                // For cmapped entries, first strip the colormap; for
                // non-cmapped entries, only do a depth conversion if
                // we need to promote.
                let conv = if pix.colormap().is_some() {
                    use crate::core::pix::convert::RemoveColormapTarget;
                    let target = if rd == 32 {
                        RemoveColormapTarget::ToFullColor
                    } else {
                        RemoveColormapTarget::ToGrayscale
                    };
                    pix.remove_colormap(target)?
                } else if rd == 32 && pix.depth() != crate::core::pix::PixelDepth::Bit32 {
                    pix.convert_to_32()?
                } else if rd != 32 && pix.depth() != crate::core::pix::PixelDepth::Bit8 {
                    pix.convert_to_8()?
                } else {
                    pix.deep_clone()
                };
                let b = pix_box_or_full(pix, self.boxa().get(i).copied());
                tmp.push_with_box(conv, b);
            }
            tmp
        } else {
            self.clone()
        };

        // 2. Promote all entries to the max depth.
        let (maxd, same) = stage1.get_depth_info()?;
        if same {
            return Ok(stage1);
        }
        let target = if maxd <= 16 { 8 } else { 32 };
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &stage1.pix_slice()[i];
            let conv = if target == 8 {
                pix.convert_to_8()?
            } else {
                pix.convert_to_32()?
            };
            let b = pix_box_or_full(pix, stage1.boxa().get(i).copied());
            out.push_with_box(conv, b);
        }
        Ok(out)
    }
}

/// Scale a Box by `(scale_x, scale_y)` (origin + dimensions),
/// matching C's `boxaTransform(boxa, 0, 0, sx, sy)`.
fn scale_box(b: &Box, scale_x: f32, scale_y: f32) -> Box {
    let nx = (b.x as f32 * scale_x + 0.5) as i32;
    let ny = (b.y as f32 * scale_y + 0.5) as i32;
    let nw = ((b.w as f32 * scale_x).max(1.0) + 0.5) as i32;
    let nh = ((b.h as f32 * scale_y).max(1.0) + 0.5) as i32;
    Box::new_unchecked(nx, ny, nw, nh)
}

/// Build a Box covering the whole image, with overflow check.
///
/// Used when a Pixa entry has no associated Box and we need a
/// meaningful fallback (the full image rectangle) rather than the
/// zero-sized default.
fn full_image_box(pix: &crate::core::pix::Pix) -> Result<Box> {
    let w = i32::try_from(pix.width())
        .map_err(|_| Error::InvalidParameter(format!("pix width {} overflows i32", pix.width())))?;
    let h = i32::try_from(pix.height()).map_err(|_| {
        Error::InvalidParameter(format!("pix height {} overflows i32", pix.height()))
    })?;
    Ok(Box::new_unchecked(0, 0, w, h))
}

/// Return `boxa[i]` if present, else a Box covering the whole image.
/// Falls back to a zero-Box (only) when the Pix dimensions overflow
/// i32 — which never happens for any real image.
fn pix_box_or_full(pix: &crate::core::pix::Pix, candidate: Option<Box>) -> Box {
    match candidate {
        Some(b) if b.w > 0 && b.h > 0 => b,
        _ => full_image_box(pix).unwrap_or_default(),
    }
}

// ============================================================================
// Plan 123: rotate / clip_to_pix / render_component
// ============================================================================

impl Pixa {
    /// Rotate every Pix by `angle` radians, returning a new Pixa.
    ///
    /// Matches C `pixaRotate`: for a sub-threshold angle the output is a
    /// deep_clone of the input. The output boxa is left empty because
    /// rotated geometry does not preserve the original boxes.
    ///
    /// C Leptonica equivalent: `pixaRotate`.
    pub fn rotate(&self, angle: f32, options: &crate::transform::RotateOptions) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for pix in self.pix_slice().iter() {
            let rotated = crate::transform::rotate(pix, angle, options)
                .map_err(|e| Error::InvalidParameter(format!("rotate failed: {e}")))?;
            out.push(rotated);
        }
        Ok(out)
    }

    /// Clip each box-rectangle from `pixs` and AND it with the matching Pix
    /// in this Pixa.
    ///
    /// Walks `min(pixa_count, boxa_count)` entries. Pix without a box are
    /// silently skipped (C `pixaClipToPix` assumes the two counts agree).
    ///
    /// C Leptonica equivalent: `pixaClipToPix`.
    pub fn clip_to_pix(&self, pixs: &crate::core::pix::Pix) -> Result<Pixa> {
        let n = self.pix_slice().len().min(self.boxa().len());
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let b = self.boxa().get(i).copied().unwrap_or_default();
            let x = u32::try_from(b.x).map_err(|_| {
                Error::InvalidParameter(format!("box[{i}].x ({}) is negative", b.x))
            })?;
            let y = u32::try_from(b.y).map_err(|_| {
                Error::InvalidParameter(format!("box[{i}].y ({}) is negative", b.y))
            })?;
            let w = u32::try_from(b.w).map_err(|_| {
                Error::InvalidParameter(format!("box[{i}].w ({}) is negative", b.w))
            })?;
            let h = u32::try_from(b.h).map_err(|_| {
                Error::InvalidParameter(format!("box[{i}].h ({}) is negative", b.h))
            })?;
            let clipped = pixs.clip_rectangle(x, y, w, h)?;
            let anded = clipped.and(pix)?;
            out.push_with_box(anded, b);
        }
        Ok(out)
    }

    /// Render the 1bpp component at `index` onto `pixs` (or a fresh
    /// boxa-extent canvas if `None`).
    ///
    /// `pixs` (when present) and every Pix in the Pixa must be 1 bpp.
    /// The component is OR-ed onto the destination at its associated box.
    ///
    /// C Leptonica equivalent: `pixaRenderComponent`.
    pub fn render_component(
        &self,
        pixs: Option<&crate::core::pix::Pix>,
        index: usize,
    ) -> Result<crate::core::pix::Pix> {
        let n = self.pix_slice().len();
        if index >= n {
            return Err(Error::IndexOutOfBounds { index, len: n });
        }
        if let Some(p) = pixs
            && p.depth() != crate::core::pix::PixelDepth::Bit1
        {
            return Err(Error::UnsupportedDepth(p.depth().bits()));
        }
        for pix in self.pix_slice().iter() {
            if pix.depth() != crate::core::pix::PixelDepth::Bit1 {
                return Err(Error::UnsupportedDepth(pix.depth().bits()));
            }
        }

        // Build the destination canvas: either the provided pixs (cloned) or
        // a fresh zero canvas sized to the boxa extent.
        let dst = match pixs {
            Some(p) => p.deep_clone(),
            None => {
                let (w, h, _) = self.boxa().get_extent().ok_or_else(|| {
                    Error::InvalidParameter("boxa is empty; cannot size canvas".into())
                })?;
                let w_u = u32::try_from(w).map_err(|_| {
                    Error::InvalidParameter(format!(
                        "boxa extent width is negative ({w}); a box has negative x/w"
                    ))
                })?;
                let h_u = u32::try_from(h).map_err(|_| {
                    Error::InvalidParameter(format!(
                        "boxa extent height is negative ({h}); a box has negative y/h"
                    ))
                })?;
                crate::core::pix::Pix::new(w_u, h_u, crate::core::pix::PixelDepth::Bit1)?
            }
        };

        let comp = &self.pix_slice()[index];
        let b = self.boxa().get(index).copied().unwrap_or_default();
        let mut dst_mut = dst.try_into_mut().unwrap();
        dst_mut.rop_region_inplace(
            b.x,
            b.y,
            comp.width(),
            comp.height(),
            crate::core::pix::rop::RopOp::Or,
            comp,
            0,
            0,
        )?;
        Ok(dst_mut.into())
    }
}

// ============================================================================
// Plan 124: bin_sort / scale_to_size_var
// ============================================================================

impl Pixa {
    /// O(n) bin sort by a single box dimension key.
    ///
    /// Mirrors C `pixaBinSort` which supports only `ByX`, `ByY`, `ByWidth`,
    /// `ByHeight`, `ByPerimeter`. Other [`PixaSortType`] variants return Err.
    ///
    /// Each entry's key is taken from its Box when present. When no Box is
    /// attached the fallback differs by sort type:
    ///
    /// - `ByX` / `ByY` → `0` (the implicit origin)
    /// - `ByWidth` / `ByHeight` / `ByPerimeter` → the Pix's own width and
    ///   height
    ///
    /// C Leptonica equivalent: `pixaBinSort`.
    pub fn bin_sort(
        &self,
        sort_type: crate::core::pixa::PixaSortType,
        order: crate::core::numa::SortOrder,
    ) -> Result<(Pixa, Vec<usize>)> {
        use crate::core::pixa::PixaSortType;
        let n = self.pix_slice().len();
        if n == 0 {
            return Ok((Pixa::new(), Vec::new()));
        }
        let mut keys = Vec::with_capacity(n);
        for (i, pix) in self.pix_slice().iter().enumerate() {
            let (x, y, w, h) = if let Some(b) = self.boxa().get(i) {
                (b.x, b.y, b.w, b.h)
            } else {
                let pw = i32::try_from(pix.width()).map_err(|_| {
                    Error::InvalidParameter(format!("pix[{i}] width overflows i32"))
                })?;
                let ph = i32::try_from(pix.height()).map_err(|_| {
                    Error::InvalidParameter(format!("pix[{i}] height overflows i32"))
                })?;
                (0, 0, pw, ph)
            };
            let key = match sort_type {
                PixaSortType::ByX => x,
                PixaSortType::ByY => y,
                PixaSortType::ByWidth => w,
                PixaSortType::ByHeight => h,
                PixaSortType::ByPerimeter => w.saturating_add(h),
                _ => {
                    return Err(Error::InvalidParameter(format!(
                        "pixaBinSort does not support {sort_type:?}; \
                         supported: ByX, ByY, ByWidth, ByHeight, ByPerimeter"
                    )));
                }
            };
            keys.push(key);
        }
        let na = crate::core::numa::Numa::from_i32_slice(&keys);
        let naindex = na
            .bin_sort_index(order)
            .map_err(|e| Error::InvalidParameter(format!("bin_sort_index failed: {e}")))?;
        let indices: Vec<usize> = (0..naindex.len())
            .map(|i| naindex.get_i32(i).map(|v| v as usize).unwrap_or(0))
            .collect();
        let sorted = self.sort_by_index(&indices)?;
        Ok((sorted, indices))
    }
}

impl crate::core::pixa::Pixaa {
    /// Scale each inner Pixa to per-image target sizes.
    ///
    /// `nawd[i]` (resp. `nahd[i]`) is the target width (resp. height) for
    /// every Pix in the `i`-th inner Pixa. At least one of `nawd`/`nahd`
    /// must be provided; the size of any provided Numa must equal
    /// `self.len()`.
    ///
    /// C Leptonica equivalent: `pixaaScaleToSizeVar`.
    pub fn scale_to_size_var(
        &self,
        nawd: Option<&crate::core::numa::Numa>,
        nahd: Option<&crate::core::numa::Numa>,
    ) -> Result<crate::core::pixa::Pixaa> {
        if nawd.is_none() && nahd.is_none() {
            return Err(Error::InvalidParameter(
                "scale_to_size_var requires at least one of nawd/nahd".into(),
            ));
        }
        let n = self.len();
        if let Some(na) = nawd
            && na.len() != n
        {
            return Err(Error::InvalidParameter(format!(
                "nawd length {} != pixaa size {n}",
                na.len()
            )));
        }
        if let Some(na) = nahd
            && na.len() != n
        {
            return Err(Error::InvalidParameter(format!(
                "nahd length {} != pixaa size {n}",
                na.len()
            )));
        }
        let mut out = crate::core::pixa::Pixaa::with_capacity(n);
        for i in 0..n {
            let inner = match self.get(i) {
                Some(p) => p,
                None => continue,
            };
            let wd = nawd
                .and_then(|na| na.get_i32(i))
                .filter(|&v| v > 0)
                .map(|v| v as u32)
                .unwrap_or(0);
            let hd = nahd
                .and_then(|na| na.get_i32(i))
                .filter(|&v| v > 0)
                .map(|v| v as u32)
                .unwrap_or(0);
            out.push(inner.scale_to_size(wd, hd));
        }
        Ok(out)
    }
}

// ============================================================================
// Plan 125: convert_to_8_colormap / Pix::make_tiled_pixa / Pixa::make_tiled_pixa
// ============================================================================

impl Pixa {
    /// Convert each Pix to 8 bpp with an attached colormap.
    ///
    /// Boxes are copied verbatim.
    ///
    /// C Leptonica equivalent: `pixaConvertTo8Colormap`.
    pub fn convert_to_8_colormap(&self, dither: bool) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let converted = pix.convert_to_8_colormap(dither)?;
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(converted, b);
        }
        Ok(out)
    }

    /// Build a Pixa by tiling every inner Pix and joining the results.
    ///
    /// Each inner Pix contributes up to `nsamp` tiles of size `w`×`h`,
    /// starting at index 0. `nsamp` must be > 0 (use `Pix::make_tiled_pixa`
    /// directly with `num = 0` if you want to take all tiles). Inner Pix
    /// that yield fewer than `nsamp` tiles still contribute what they have.
    ///
    /// C Leptonica equivalent: `pixaMakeFromTiledPixa`.
    pub fn make_tiled_pixa(&self, w: u32, h: u32, nsamp: u32) -> Result<Pixa> {
        if nsamp == 0 {
            return Err(Error::InvalidParameter(
                "nsamp must be > 0 (use Pix::make_tiled_pixa with num=0 \
                 directly for the all-tiles case)"
                    .into(),
            ));
        }
        let mut out = Pixa::new();
        for pix in self.pix_slice().iter() {
            let tiles = pix.make_tiled_pixa(w, h, 0, nsamp, None)?;
            out.join(&tiles, 0, None)?;
        }
        Ok(out)
    }
}

impl crate::core::pix::Pix {
    /// Split this Pix into a Pixa of `w`×`h` tiles, or extract sub-regions
    /// indicated by `boxa` (when `Some`).
    ///
    /// - `start` skips the first `start` tiles
    /// - `num == 0` means "take all remaining tiles" (matching C semantics)
    /// - When `boxa = Some(b)` the `w`/`h` arguments are ignored, and
    ///   sub-images are clipped from this Pix at each box position.
    ///   This path delegates to [`Pixa::create_from_boxa`], which silently
    ///   **skips** boxes that lie fully outside the image or clamp to zero
    ///   area. The output may therefore contain fewer tiles than the boxa
    ///   has entries, and `start`/`num` are applied to that already-filtered
    ///   sequence (not the original boxa indices).
    /// - When `boxa = None`, the image is split into a `nx`×`ny` grid where
    ///   `nx = width / w`, `ny = height / h`. The Pix text field may carry
    ///   an `"n = N"` tile count that limits how many tiles are produced.
    ///
    /// C Leptonica equivalent: `pixaMakeFromTiledPix`.
    pub fn make_tiled_pixa(
        &self,
        w: u32,
        h: u32,
        start: u32,
        num: u32,
        boxa: Option<&Boxa>,
    ) -> Result<Pixa> {
        if let Some(b) = boxa {
            let pa = Pixa::create_from_boxa(self, b);
            let len = pa.pix_slice().len();
            if start as usize >= len && len > 0 {
                return Ok(Pixa::new());
            }
            let begin = (start as usize).min(len);
            let end = if num == 0 {
                len
            } else {
                len.min(begin + num as usize)
            };
            return Ok(pa.select_range(begin, Some(end.saturating_sub(1))));
        }
        if w == 0 || h == 0 {
            return Err(Error::InvalidParameter(format!(
                "tile size must be > 0 (got w={w} h={h})"
            )));
        }
        let ws = self.width();
        let hs = self.height();
        let nx = ws / w;
        let ny = hs / h;
        if nx < 1 || ny < 1 {
            return Err(Error::InvalidParameter(format!(
                "image {ws}x{hs} cannot hold any {w}x{h} tile"
            )));
        }
        let grid_total = nx.saturating_mul(ny);
        let text_n = self.get_tile_count();
        // Use the text-encoded tile count only if it lies in
        // (nx * (ny - 1), nx * ny], matching the C heuristic.
        let n_isvalid =
            text_n > 0 && text_n <= grid_total && text_n > nx.saturating_mul(ny.saturating_sub(1));
        let ntiles = if n_isvalid { text_n } else { grid_total };
        if start >= ntiles {
            return Ok(Pixa::new());
        }
        let nmax = ntiles - start;
        let take = if num == 0 { nmax } else { num.min(nmax) };
        let mut out = Pixa::with_capacity(take as usize);
        let mut k: u32 = 0;
        for i in 0..ny {
            for j in 0..nx {
                if k < start {
                    k += 1;
                    continue;
                }
                if k >= start + take {
                    return Ok(out);
                }
                let tile = self.clip_rectangle(j * w, i * h, w, h)?;
                out.push(tile);
                k += 1;
            }
        }
        Ok(out)
    }
}

// ============================================================================
// Plan 126: Pixa::select_to_pdf
// ============================================================================

#[cfg(feature = "pdf-format")]
impl Pixa {
    /// Write a contiguous range of Pix to a multi-page PDF.
    ///
    /// Mirrors C `pixaSelectToPdf` with `fontsize <= 0` (no text annotation).
    /// `last = None` extends to the end of the Pixa. The selected range must
    /// be non-empty; otherwise the underlying PDF writer returns an error.
    ///
    /// C Leptonica equivalent: `pixaSelectToPdf` (text-annotation feature
    /// intentionally omitted; pair with the existing BMF helpers if needed).
    pub fn select_to_pdf<W: std::io::Write>(
        &self,
        first: usize,
        last: Option<usize>,
        options: &crate::io::pdf::PdfOptions,
        writer: W,
    ) -> crate::io::IoResult<()> {
        let n = self.pix_slice().len();
        if first >= n {
            return Err(crate::io::IoError::InvalidData(format!(
                "first ({first}) >= pixa size ({n}); select_to_pdf range is empty"
            )));
        }
        let begin = first;
        let end = match last {
            Some(l) => {
                if l < begin {
                    return Err(crate::io::IoError::InvalidData(format!(
                        "last ({l}) < first ({begin}); select_to_pdf range is empty"
                    )));
                }
                l.saturating_add(1).min(n)
            }
            None => n,
        };
        let refs: Vec<&crate::core::pix::Pix> = self.pix_slice()[begin..end].iter().collect();
        crate::io::pdf::write_pdf_multi(&refs, writer, options)
    }
}

// ============================================================================
// Plan 127: Pixa::convert_to_nup
// ============================================================================

impl Pixa {
    /// Tile this Pixa into N-up pages.
    ///
    /// Each page is one `nx × ny` grid laid out by
    /// [`Pixa::display_tiled_and_scaled`]; the result is a Pixa with one Pix
    /// per page (`ceil(self.len() / (nx * ny))` pages in total).
    ///
    /// Matches C `pixaConvertToNUpPixa` with `fontsize == 0` (no text
    /// overlay). BMF text annotation (`pixAddTextlines`) is intentionally
    /// omitted; consumers can layer that on top by pre-annotating each Pix.
    ///
    /// C Leptonica equivalent: `pixaConvertToNUpPixa`.
    pub fn convert_to_nup(
        &self,
        _nx: u32,
        _ny: u32,
        _tile_width: u32,
        _spacing: u32,
        _border: u32,
    ) -> Result<Pixa> {
        unimplemented!("plan 127: Pixa::convert_to_nup")
    }
}
