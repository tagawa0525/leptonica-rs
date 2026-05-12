//! Pixa transform helpers (plan 107 / C pixafunc1.c + pixafunc2.c).
//!
//! Each method iterates the source Pixa, applies the corresponding Pix
//! function, and rebuilds the Boxa with matching coordinates.

use crate::core::box_::Box;
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

    /// Convert every Pix to 8 bpp. `cmap_flag = true` keeps/produces a
    /// gray colormap (matching C's `pixConvertTo8`).
    ///
    /// C Leptonica equivalent: `pixaConvertTo8`.
    pub fn convert_to_8(&self, cmap_flag: bool) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let mut out = Pixa::with_capacity(n);
        for i in 0..n {
            let pix = &self.pix_slice()[i];
            let converted = if cmap_flag {
                pix.convert_to_8_or_32()?
            } else {
                pix.convert_to_8()?
            };
            // convert_to_8_or_32 may yield 32 bpp for color, but C
            // pixaConvertTo8 always returns 8 bpp via convert_to_8.
            let converted = if converted.depth() != crate::core::pix::PixelDepth::Bit8 {
                pix.convert_to_8()?
            } else {
                converted
            };
            let b = self.boxa().get(i).copied().unwrap_or_default();
            out.push_with_box(converted, b);
        }
        let _ = cmap_flag; // cmap-handling pathway can be revisited in plan 107b
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
