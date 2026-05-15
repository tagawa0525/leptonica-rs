//! FPix extended helpers (plan 110 / C fpix2.c gap-fill).
//!
//! Port targets:
//!
//! - `fpixGetMin` / `fpixGetMax` (thin aliases over [`FPix::min`] / [`FPix::max`])
//! - `fpixThresholdToPix` -> [`FPix::threshold_to_pix`]
//! - `fpixRasterop` -> [`FPix::rasterop`]
//! - `fpixScaleByInteger` -> [`FPix::scale_by_integer`]
//! - `fpixRemoveBorder` -> [`FPix::remove_border`]
//! - `linearInterpolatePixelFloat` -> [`linear_interpolate_pixel_float`]

use crate::core::error::{Error, Result};
use crate::core::pix::{Pix, PixelDepth};

use super::FPix;

impl FPix {
    /// Find the minimum value and its location.
    ///
    /// Thin alias for [`FPix::min`] kept for C API parity (`fpixGetMin`).
    pub fn get_min(&self) -> Option<(f32, u32, u32)> {
        self.min()
    }

    /// Find the maximum value and its location.
    ///
    /// Thin alias for [`FPix::max`] kept for C API parity (`fpixGetMax`).
    pub fn get_max(&self) -> Option<(f32, u32, u32)> {
        self.max()
    }

    /// Threshold to a 1 bpp Pix: pixels with `value <= thresh` become FG (1).
    ///
    /// C Leptonica equivalent: `fpixThresholdToPix`.
    pub fn threshold_to_pix(&self, thresh: f32) -> Result<Pix> {
        let w = self.width();
        let h = self.height();
        let pix = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut pix_mut = pix.try_into_mut().expect("freshly created");
        for y in 0..h {
            let row = self.row(y);
            for (x, &v) in row.iter().enumerate() {
                if v <= thresh {
                    pix_mut.set_pixel(x as u32, y, 1)?;
                }
            }
        }
        Ok(pix_mut.into())
    }

    /// Copy a clipped rectangle from `src` into `self` (a la C `pixRasterop`
    /// with `PIX_SRC`).
    ///
    /// Negative offsets and overhanging rectangles are clipped to the
    /// intersection of source and destination; if the clipped rect is
    /// empty the call is a no-op.
    ///
    /// C Leptonica equivalent: `fpixRasterop`.
    #[allow(clippy::too_many_arguments)]
    pub fn rasterop(
        &mut self,
        mut dx: i32,
        mut dy: i32,
        mut dw: i32,
        mut dh: i32,
        src: &FPix,
        mut sx: i32,
        mut sy: i32,
    ) -> Result<()> {
        let fsw = src.width() as i32;
        let fsh = src.height() as i32;
        let fdw = self.width() as i32;
        let fdh = self.height() as i32;

        // Horizontal clipping
        if dx < 0 {
            sx -= dx;
            dw += dx;
            dx = 0;
        }
        if sx < 0 {
            dx -= sx;
            dw += sx;
            sx = 0;
        }
        let dhang_w = dx + dw - fdw;
        if dhang_w > 0 {
            dw -= dhang_w;
        }
        let shang_w = sx + dw - fsw;
        if shang_w > 0 {
            dw -= shang_w;
        }

        // Vertical clipping
        if dy < 0 {
            sy -= dy;
            dh += dy;
            dy = 0;
        }
        if sy < 0 {
            dy -= sy;
            dh += sy;
            sy = 0;
        }
        let dhang_h = dy + dh - fdh;
        if dhang_h > 0 {
            dh -= dhang_h;
        }
        let shang_h = sy + dh - fsh;
        if shang_h > 0 {
            dh -= shang_h;
        }

        if dw <= 0 || dh <= 0 {
            return Ok(());
        }

        let sw = src.width() as usize;
        let dw_dst = self.width() as usize;
        for row in 0..dh {
            let s_row = (sy + row) as usize;
            let d_row = (dy + row) as usize;
            let s_off = s_row * sw + sx as usize;
            let d_off = d_row * dw_dst + dx as usize;
            let n = dw as usize;
            let src_slice = &src.data()[s_off..s_off + n];
            self.data_mut()[d_off..d_off + n].copy_from_slice(src_slice);
        }
        Ok(())
    }

    /// Upscale by an integer factor using bilinear interpolation on a
    /// `(factor*(ws-1)+1) x (factor*(hs-1)+1)` grid.
    ///
    /// `factor = 1` returns a deep copy; `factor >= 2` interpolates.
    /// Returns `InvalidParameter` for `factor == 0`.
    ///
    /// C Leptonica equivalent: `fpixScaleByInteger`.
    pub fn scale_by_integer(&self, factor: u32) -> Result<FPix> {
        if factor == 0 {
            return Err(Error::InvalidParameter("factor must be >= 1".into()));
        }
        // Reject factors that overflow i32 arithmetic below.
        if factor > i32::MAX as u32 {
            return Err(Error::InvalidParameter(format!(
                "factor too large for i32 arithmetic: {factor}"
            )));
        }
        let factor_i = factor as i32;
        let ws = self.width() as i32;
        let hs = self.height() as i32;
        if ws < 2 || hs < 2 {
            // C version has UB in this case (wd/hd <= 0 possible); we follow
            // the spirit by returning a deep clone.
            return Ok(self.clone());
        }
        // Use checked arithmetic to avoid silent i32 overflow on huge inputs.
        let wd = factor_i
            .checked_mul(ws - 1)
            .and_then(|p| p.checked_add(1))
            .ok_or_else(|| Error::InvalidParameter("scale dimensions overflow".into()))?;
        let hd = factor_i
            .checked_mul(hs - 1)
            .and_then(|p| p.checked_add(1))
            .ok_or_else(|| Error::InvalidParameter("scale dimensions overflow".into()))?;
        let mut dst = FPix::new(wd as u32, hd as u32)?;
        let f = factor as f32;
        let fract: Vec<f32> = (0..factor_i).map(|i| (i as f32) / f).collect();

        let sw = self.width() as usize;
        let src = self.data();
        let dw_dst = wd as usize;

        for i in 0..(hs - 1) {
            let row0 = i as usize * sw;
            let row1 = row0 + sw;
            for j in 0..(ws - 1) {
                let v0 = src[row0 + j as usize];
                let v1 = src[row0 + j as usize + 1];
                let v2 = src[row1 + j as usize];
                let v3 = src[row1 + j as usize + 1];
                for k in 0..factor_i {
                    let dline = ((i * factor_i + k) as usize) * dw_dst;
                    let fk = fract[k as usize];
                    for m in 0..factor_i {
                        let fm = fract[m as usize];
                        let val = v0 * (1.0 - fm) * (1.0 - fk)
                            + v1 * fm * (1.0 - fk)
                            + v2 * (1.0 - fm) * fk
                            + v3 * fm * fk;
                        dst.data_mut()[dline + (j * factor_i + m) as usize] = val;
                    }
                }
            }
        }

        // Final column/row (j = ws-1 or i = hs-1): copy verbatim from source
        // edges to fill the (factor*(ws-1)+1) coordinate space.
        for i in 0..(hs - 1) {
            let v_right = src[i as usize * sw + (ws - 1) as usize];
            let v_right_next = src[(i as usize + 1) * sw + (ws - 1) as usize];
            for k in 0..factor_i {
                let dline = ((i * factor_i + k) as usize) * dw_dst;
                let fk = fract[k as usize];
                let val = v_right * (1.0 - fk) + v_right_next * fk;
                dst.data_mut()[dline + (wd - 1) as usize] = val;
            }
        }
        for j in 0..(ws - 1) {
            let v_bot = src[(hs - 1) as usize * sw + j as usize];
            let v_bot_next = src[(hs - 1) as usize * sw + j as usize + 1];
            let dline = (hd - 1) as usize * dw_dst;
            for m in 0..factor_i {
                let fm = fract[m as usize];
                let val = v_bot * (1.0 - fm) + v_bot_next * fm;
                dst.data_mut()[dline + (j * factor_i + m) as usize] = val;
            }
        }
        // Bottom-right corner
        dst.data_mut()[(hd - 1) as usize * dw_dst + (wd - 1) as usize] =
            src[(hs - 1) as usize * sw + (ws - 1) as usize];

        Ok(dst)
    }

    /// Return a new FPix with the specified borders removed.
    ///
    /// All borders default to zero (`<= 0` means no removal on that side).
    /// Returns an error if the resulting width or height would be zero
    /// or negative.
    ///
    /// C Leptonica equivalent: `fpixRemoveBorder`.
    pub fn remove_border(&self, left: i32, right: i32, top: i32, bot: i32) -> Result<FPix> {
        if left <= 0 && right <= 0 && top <= 0 && bot <= 0 {
            return Ok(self.clone());
        }
        let ws = self.width() as i32;
        let hs = self.height() as i32;
        let wd = ws - left.max(0) - right.max(0);
        let hd = hs - top.max(0) - bot.max(0);
        if wd <= 0 || hd <= 0 {
            return Err(Error::InvalidParameter(format!(
                "removing borders leaves no image: wd={wd}, hd={hd}"
            )));
        }
        let mut dst = FPix::new(wd as u32, hd as u32)?;
        dst.set_xres(self.xres());
        dst.set_yres(self.yres());
        dst.rasterop(0, 0, wd, hd, self, left.max(0), top.max(0))?;
        Ok(dst)
    }
}

/// 16-step fixed-point bilinear interpolation on raw `f32` row-major data.
///
/// `data` must be at least `w * h` elements. `(x, y)` may be fractional.
/// When `(x, y)` is outside `[0, w-2] x [0, h-2]` the function returns
/// `inval` (matching C's "skip if off the edge" behaviour). Invalid
/// inputs (`w < 2`, `h < 2`, undersized `data`, or non-finite
/// `x`/`y`) also return `inval` rather than indexing blindly.
///
/// C Leptonica equivalent: `linearInterpolatePixelFloat`.
pub fn linear_interpolate_pixel_float(
    data: &[f32],
    w: i32,
    h: i32,
    x: f32,
    y: f32,
    inval: f32,
) -> f32 {
    // Reject malformed inputs up front: bilinear needs a 2x2 neighborhood,
    // the backing buffer must hold w*h floats, and NaN/inf coordinates
    // would otherwise bypass the bounds check before indexing.
    if w < 2 || h < 2 {
        return inval;
    }
    if !x.is_finite() || !y.is_finite() {
        return inval;
    }
    let required = (w as usize).saturating_mul(h as usize);
    if data.len() < required {
        return inval;
    }
    if x < 0.0 || y < 0.0 || x > (w - 2) as f32 || y > (h - 2) as f32 {
        return inval;
    }
    let xpm = (16.0 * x + 0.5) as i32;
    let ypm = (16.0 * y + 0.5) as i32;
    let xp = (xpm >> 4) as usize;
    let yp = (ypm >> 4) as usize;
    let xf = (xpm & 0x0f) as f32;
    let yf = (ypm & 0x0f) as f32;

    let wu = w as usize;
    let line = yp * wu;
    let v00 = (16.0 - xf) * (16.0 - yf) * data[line + xp];
    let v10 = xf * (16.0 - yf) * data[line + xp + 1];
    let v01 = (16.0 - xf) * yf * data[line + wu + xp];
    let v11 = xf * yf * data[line + wu + xp + 1];
    (v00 + v01 + v10 + v11) / 256.0
}

/// Affine-transform an [`FPix`] by a 6-element matrix that maps destination
/// coordinates back to source coordinates.
///
/// For each destination pixel `(j, i)`, computes the source coordinate
/// `(x, y) = (vc[0]*j + vc[1]*i + vc[2], vc[3]*j + vc[4]*i + vc[5])` and
/// samples `fpixs` via [`linear_interpolate_pixel_float`]. Out-of-bounds
/// samples are filled with `inval`.
///
/// C Leptonica equivalent: `fpixAffine` (`fpix2.c`).
pub fn fpix_affine(fpixs: &FPix, vc: &[f32; 6], inval: f32) -> Result<FPix> {
    let w = fpixs.width() as i32;
    let h = fpixs.height() as i32;
    let datas = fpixs.data();
    // `create_template` already produces a zero-filled buffer; since we
    // write every destination pixel exactly once below (in-bounds samples
    // get the interpolated value, out-of-bounds get `inval`), we skip the
    // extra `set_all(inval)` pass to avoid a redundant O(w*h) write.
    let mut fpixd = fpixs.create_template();
    let datad = fpixd.data_mut();
    let wu = w as usize;
    for i in 0..h {
        for j in 0..w {
            let x = vc[0] * j as f32 + vc[1] * i as f32 + vc[2];
            let y = vc[3] * j as f32 + vc[4] * i as f32 + vc[5];
            let val = linear_interpolate_pixel_float(datas, w, h, x, y, inval);
            datad[i as usize * wu + j as usize] = val;
        }
    }
    Ok(fpixd)
}

/// Projective-transform an [`FPix`] by an 8-element vector that maps
/// destination coordinates back to source coordinates.
///
/// For each destination pixel `(j, i)`:
/// - `denom = vc[6] * j + vc[7] * i + 1`
/// - `(x, y) = ((vc[0]*j + vc[1]*i + vc[2]) / denom,
///             (vc[3]*j + vc[4]*i + vc[5]) / denom)`
///
/// and samples `fpixs` via [`linear_interpolate_pixel_float`].
/// Out-of-bounds samples are filled with `inval`.
///
/// C Leptonica equivalent: `fpixProjective` (`fpix2.c`).
pub fn fpix_projective(fpixs: &FPix, vc: &[f32; 8], inval: f32) -> Result<FPix> {
    let w = fpixs.width() as i32;
    let h = fpixs.height() as i32;
    let datas = fpixs.data();
    // See fpix_affine: every destination pixel is written exactly once
    // below, so we skip the redundant `set_all(inval)` pre-fill.
    let mut fpixd = fpixs.create_template();
    let datad = fpixd.data_mut();
    let wu = w as usize;
    for i in 0..h {
        for j in 0..w {
            let denom = vc[6] * j as f32 + vc[7] * i as f32 + 1.0;
            let val = if denom == 0.0 {
                inval
            } else {
                let x = (vc[0] * j as f32 + vc[1] * i as f32 + vc[2]) / denom;
                let y = (vc[3] * j as f32 + vc[4] * i as f32 + vc[5]) / denom;
                linear_interpolate_pixel_float(datas, w, h, x, y, inval)
            };
            datad[i as usize * wu + j as usize] = val;
        }
    }
    Ok(fpixd)
}
