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
