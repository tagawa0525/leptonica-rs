//! Mask operations for images
//!
//! Functions for setting, combining, and creating masks.
//! Corresponds to mask functions in C Leptonica's `pix3.c`.

use super::{Pix, PixMut, PixelDepth};
use crate::core::error::{Error, Result};

impl PixMut {
    /// Set pixels to a value where a 1 bpp mask is ON.
    ///
    /// The mask is aligned to the upper-left corner. Only the
    /// overlapping region is processed.
    ///
    /// C equivalent: `pixSetMasked()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - 1 bpp mask image
    /// * `val` - Value to set (masked to valid range for depth)
    ///
    /// # Errors
    ///
    /// Returns an error if the mask is not 1 bpp or if this image
    /// has an unsupported depth.
    pub fn set_masked(&mut self, mask: &Pix, val: u32) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        let val = mask_val(val, self.depth());
        let w = self.width().min(mask.width());
        let h = self.height().min(mask.height());

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel_unchecked(x, y) != 0 {
                    self.set_pixel_unchecked(x, y, val);
                }
            }
        }
        Ok(())
    }

    /// Copy pixels from a source image where a 1 bpp mask is ON.
    ///
    /// The mask, source, and destination are aligned at the upper-left
    /// corner. Only the overlapping region is processed.
    ///
    /// C equivalent: `pixCombineMasked()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `src` - Source image (must have same depth as self)
    /// * `mask` - 1 bpp mask image
    ///
    /// # Errors
    ///
    /// Returns an error if depths don't match, if the mask is not
    /// 1 bpp, or if the depth is unsupported.
    pub fn combine_masked(&mut self, src: &Pix, mask: &Pix) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        if self.depth() != src.depth() {
            return Err(Error::InvalidParameter(
                "source and destination depths must match".into(),
            ));
        }
        let w = self.width().min(src.width()).min(mask.width());
        let h = self.height().min(src.height()).min(mask.height());

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel_unchecked(x, y) != 0 {
                    self.set_pixel_unchecked(x, y, src.get_pixel_unchecked(x, y));
                }
            }
        }
        Ok(())
    }

    /// Paint a value through a mask at a specified offset.
    ///
    /// Like `set_masked` but the mask is placed at position `(x, y)`
    /// relative to the destination image.
    ///
    /// C equivalent: `pixPaintThroughMask()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `mask` - 1 bpp mask image
    /// * `x` - Horizontal offset for mask placement
    /// * `y` - Vertical offset for mask placement
    /// * `val` - Value to paint (masked to valid range for depth)
    ///
    /// # Errors
    ///
    /// Returns an error if the mask is not 1 bpp or if this image
    /// has an unsupported depth.
    pub fn paint_through_mask(&mut self, mask: &Pix, x: i32, y: i32, val: u32) -> Result<()> {
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        let val = mask_val(val, self.depth());
        let dw = self.width() as i32;
        let dh = self.height() as i32;
        let mw = mask.width() as i32;
        let mh = mask.height() as i32;

        for my in 0..mh {
            let dy = y + my;
            if dy < 0 || dy >= dh {
                continue;
            }
            for mx in 0..mw {
                let dx = x + mx;
                if dx < 0 || dx >= dw {
                    continue;
                }
                if mask.get_pixel_unchecked(mx as u32, my as u32) != 0 {
                    self.set_pixel_unchecked(dx as u32, dy as u32, val);
                }
            }
        }
        Ok(())
    }
}

impl Pix {
    /// Create a 1 bpp mask where pixels equal a given value.
    ///
    /// C equivalent: `pixMakeMaskFromVal()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `val` - Pixel value to match
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 2, 4, or 8 bpp.
    pub fn make_mask_from_val(&self, val: u32) -> Result<Pix> {
        let d = self.depth();
        if d != PixelDepth::Bit2 && d != PixelDepth::Bit4 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }

        let w = self.width();
        let h = self.height();
        let mask = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut mm = mask.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) == val {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        Ok(mm.into())
    }

    /// Create a 1 bpp mask using a lookup table.
    ///
    /// The LUT maps pixel values to mask values (0 or 1).
    ///
    /// C equivalent: `pixMakeMaskFromLUT()` in `pix3.c`
    ///
    /// # Arguments
    ///
    /// * `lut` - Lookup table of 256 entries; 1 means set mask bit
    ///
    /// # Errors
    ///
    /// Returns an error if the image depth is not 2, 4, or 8 bpp,
    /// or if the LUT has fewer than 256 entries.
    pub fn make_mask_from_lut(&self, lut: &[u8]) -> Result<Pix> {
        let d = self.depth();
        if d != PixelDepth::Bit2 && d != PixelDepth::Bit4 && d != PixelDepth::Bit8 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if lut.len() < 256 {
            return Err(Error::InvalidParameter(
                "LUT must have at least 256 entries".into(),
            ));
        }

        let w = self.width();
        let h = self.height();
        let mask = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut mm = mask.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let v = self.get_pixel_unchecked(x, y) as usize;
                if lut[v] != 0 {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        Ok(mm.into())
    }
}

impl PixMut {
    /// Set pixels to a value where a 1 bpp mask is ON, with explicit alignment.
    ///
    /// Like `set_masked`, but the mask is placed at `(x, y)` relative to the
    /// destination, and only 8, 16, or 32 bpp destinations are supported.
    ///
    /// C equivalent: `pixSetMaskedGeneral()` in `pix3.c`
    pub fn set_masked_general(&mut self, mask: &Pix, val: u32, x: i32, y: i32) -> Result<()> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit16 && d != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        let val = mask_val(val, d);
        let dw = self.width() as i32;
        let dh = self.height() as i32;
        let mw = mask.width() as i32;
        let mh = mask.height() as i32;

        for my in 0..mh {
            let dy = y + my;
            if dy < 0 || dy >= dh {
                continue;
            }
            for mx in 0..mw {
                let dx = x + mx;
                if dx < 0 || dx >= dw {
                    continue;
                }
                if mask.get_pixel_unchecked(mx as u32, my as u32) != 0 {
                    self.set_pixel_unchecked(dx as u32, dy as u32, val);
                }
            }
        }
        Ok(())
    }

    /// Copy pixels from a source image where a 1 bpp mask is ON, with explicit alignment.
    ///
    /// Both source and mask are aligned to `(x, y)` relative to the destination.
    ///
    /// C equivalent: `pixCombineMaskedGeneral()` in `pix3.c`
    pub fn combine_masked_general(&mut self, src: &Pix, mask: &Pix, x: i32, y: i32) -> Result<()> {
        let d = self.depth();
        if d != src.depth() {
            return Err(Error::InvalidParameter(
                "source and destination depths must match".into(),
            ));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        let dw = self.width() as i32;
        let dh = self.height() as i32;
        let sw = src.width() as i32;
        let sh = src.height() as i32;
        let mw = mask.width() as i32;
        let mh = mask.height() as i32;
        let h_min = sh.min(mh);
        let w_min = sw.min(mw);

        for my in 0..h_min {
            let dy = y + my;
            if dy < 0 || dy >= dh {
                continue;
            }
            for mx in 0..w_min {
                let dx = x + mx;
                if dx < 0 || dx >= dw {
                    continue;
                }
                if mask.get_pixel_unchecked(mx as u32, my as u32) != 0 {
                    let val = src.get_pixel_unchecked(mx as u32, my as u32);
                    self.set_pixel_unchecked(dx as u32, dy as u32, val);
                }
            }
        }
        Ok(())
    }
}

impl Pix {
    /// Copy pixels from box regions, filling the rest with a background color.
    ///
    /// Creates a new image of the same size and depth, filled with the
    /// background color, then copies pixels from `self` that fall within
    /// any of the boxes in `boxa`.
    ///
    /// C equivalent: `pixCopyWithBoxa()` in `pix3.c`
    pub fn copy_with_boxa(
        &self,
        boxa: &crate::core::box_::Boxa,
        background: super::InitColor,
    ) -> Result<Pix> {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        let bg_val = PixMut::get_black_or_white_val(self, background);

        let dst = Pix::new(w, h, d)?;
        let mut dm = dst.try_into_mut().unwrap();

        // Fill with background
        for y in 0..h {
            for x in 0..w {
                dm.set_pixel_unchecked(x, y, bg_val);
            }
        }

        // Copy pixels inside each box
        for b in boxa.iter() {
            let x0 = b.x.max(0) as u32;
            let y0 = b.y.max(0) as u32;
            let x1 = ((b.x + b.w) as u32).min(w);
            let y1 = ((b.y + b.h) as u32).min(h);
            for y in y0..y1 {
                for x in x0..x1 {
                    dm.set_pixel_unchecked(x, y, self.get_pixel_unchecked(x, y));
                }
            }
        }

        Ok(dm.into())
    }

    /// Create a 1 bpp mask from a 32 bpp RGB image using weighted coefficients.
    ///
    /// Computes `rc*R + gc*G + bc*B` for each pixel, then thresholds.
    /// Pixels where the weighted sum exceeds `thresh` are ON in the mask.
    ///
    /// C equivalent: `pixMakeArbMaskFromRGB()` in `pix3.c`
    pub fn make_arb_mask_from_rgb(&self, rc: f32, gc: f32, bc: f32, thresh: f32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let mask = Pix::new(w, h, PixelDepth::Bit1)?;
        let mut mm = mask.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                let pixel = self.get_pixel_unchecked(x, y);
                let (r, g, b, _) = crate::core::pixel::extract_rgba(pixel);
                let val = rc * r as f32 + gc * g as f32 + bc * b as f32;
                if val > thresh {
                    mm.set_pixel_unchecked(x, y, 1);
                }
            }
        }

        Ok(mm.into())
    }

    /// Replace all pixels under the 1-pixels of mask, at offset (x, y),
    /// with the average value of surrounding non-masked pixels.
    ///
    /// `search_dir`: 0 = 8-connected neighborhood, 1 = 4-connected.
    /// Must be 8 or 32 bpp.
    ///
    /// C equivalent: `pixPaintSelfThroughMask()` in `pix3.c`
    pub fn paint_self_through_mask(
        &self,
        mask: &Pix,
        x: u32,
        y: u32,
        search_dir: u32,
    ) -> Result<Pix> {
        let d = self.depth();
        if d != PixelDepth::Bit8 && d != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(d.bits()));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let mw = mask.width();
        let mh = mask.height();
        let result = self.deep_clone();
        let mut rm = result.try_into_mut().unwrap();

        let neighbors_8: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        let neighbors_4: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

        for my in 0..mh {
            let dy = y + my;
            if dy >= h {
                continue;
            }
            for mx in 0..mw {
                let dx = x + mx;
                if dx >= w {
                    continue;
                }
                if mask.get_pixel_unchecked(mx, my) == 0 {
                    continue;
                }

                if d == PixelDepth::Bit32 {
                    let mut r_sum = 0u32;
                    let mut g_sum = 0u32;
                    let mut b_sum = 0u32;
                    let mut count = 0u32;
                    let neighbors: &[(i32, i32)] = if search_dir == 1 {
                        &neighbors_4
                    } else {
                        &neighbors_8
                    };
                    for &(ox, oy) in neighbors {
                        let nx = dx as i32 + ox;
                        let ny = dy as i32 + oy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let nxu = nx as u32;
                        let nyu = ny as u32;
                        // Check if this neighbor is masked
                        let in_mask = nxu >= x
                            && nyu >= y
                            && nxu - x < mw
                            && nyu - y < mh
                            && mask.get_pixel_unchecked(nxu - x, nyu - y) != 0;
                        if !in_mask {
                            let p = self.get_pixel_unchecked(nxu, nyu);
                            let (r, g, b, _) = crate::core::pixel::extract_rgba(p);
                            r_sum += r as u32;
                            g_sum += g as u32;
                            b_sum += b as u32;
                            count += 1;
                        }
                    }
                    if let Some(count) = std::num::NonZeroU32::new(count) {
                        let count = count.get();
                        let avg = crate::core::pixel::compose_rgb(
                            (r_sum / count) as u8,
                            (g_sum / count) as u8,
                            (b_sum / count) as u8,
                        );
                        rm.set_pixel_unchecked(dx, dy, avg);
                    }
                } else {
                    // 8bpp
                    let mut sum = 0u32;
                    let mut count = 0u32;
                    let neighbors: &[(i32, i32)] = if search_dir == 1 {
                        &neighbors_4
                    } else {
                        &neighbors_8
                    };
                    for &(ox, oy) in neighbors {
                        let nx = dx as i32 + ox;
                        let ny = dy as i32 + oy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let nxu = nx as u32;
                        let nyu = ny as u32;
                        let in_mask = nxu >= x
                            && nyu >= y
                            && nxu - x < mw
                            && nyu - y < mh
                            && mask.get_pixel_unchecked(nxu - x, nyu - y) != 0;
                        if !in_mask {
                            sum += self.get_pixel_unchecked(nxu, nyu);
                            count += 1;
                        }
                    }
                    if let Some(avg) = sum.checked_div(count) {
                        rm.set_pixel_unchecked(dx, dy, avg);
                    }
                }
            }
        }
        Ok(rm.into())
    }

    /// Create 8bpp alpha image from 1bpp mask.
    ///
    /// Pixels in mask get `val` alpha, others get 0.
    ///
    /// C equivalent: `pixMakeAlphaFromMask()` in `pix3.c`
    pub fn make_alpha_from_mask(&self, val: u8) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let alpha = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut am = alpha.try_into_mut().unwrap();

        for y in 0..h {
            for x in 0..w {
                if self.get_pixel_unchecked(x, y) != 0 {
                    am.set_pixel_unchecked(x, y, val as u32);
                }
            }
        }
        Ok(am.into())
    }

    /// Get average color of pixels near boundary of mask, within distance `dist`.
    ///
    /// For 32bpp source and 1bpp mask. Erode mask by `dist`, XOR with original
    /// to get boundary ring. Average the colors of source pixels in that ring.
    ///
    /// C equivalent: `pixGetColorNearMaskBoundary()` in `pix3.c`
    pub fn get_color_near_mask_boundary(&self, mask: &Pix, dist: u32) -> Result<u32> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        if mask.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(mask.depth().bits()));
        }
        if dist == 0 {
            return Err(Error::InvalidParameter("dist must be >= 1".into()));
        }

        let w = self.width().min(mask.width());
        let h = self.height().min(mask.height());

        // Erode the mask by `dist` to get inner region
        let size = 2 * dist + 1;
        let eroded = crate::morph::erode_brick(mask, size, size)
            .map_err(|e| Error::InvalidParameter(format!("erode failed: {e}")))?;

        // XOR original mask with eroded to get the boundary ring
        let boundary = mask
            .xor(&eroded)
            .map_err(|e| Error::InvalidParameter(format!("xor failed: {e}")))?;

        let mut r_sum = 0u64;
        let mut g_sum = 0u64;
        let mut b_sum = 0u64;
        let mut count = 0u64;

        for y in 0..h {
            for x in 0..w {
                if boundary.get_pixel_unchecked(x, y) != 0 {
                    let p = self.get_pixel_unchecked(x, y);
                    let (r, g, b, _) = crate::core::pixel::extract_rgba(p);
                    r_sum += r as u64;
                    g_sum += g as u64;
                    b_sum += b as u64;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Ok(0);
        }

        Ok(crate::core::pixel::compose_rgb(
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        ))
    }

    /// Set RGB values under fully transparent (alpha == 0) pixels.
    ///
    /// Returns a new 32 bpp image with RGB replaced where alpha is 0.
    /// The alpha channel is preserved.
    ///
    /// C equivalent: `pixSetUnderTransparency()` in `pix3.c`
    pub fn set_under_transparency(&self, val: u32) -> Result<Pix> {
        if self.depth() != PixelDepth::Bit32 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }

        let w = self.width();
        let h = self.height();
        let result = self.deep_clone();
        let mut rm = result.try_into_mut().unwrap();

        // Extract replacement RGB from val (ignore alpha byte of val)
        let (new_r, new_g, new_b, _) = crate::core::pixel::extract_rgba(val);

        for y in 0..h {
            for x in 0..w {
                let pixel = rm.get_pixel_unchecked(x, y);
                let (_, _, _, a) = crate::core::pixel::extract_rgba(pixel);
                if a == 0 {
                    rm.set_pixel_unchecked(
                        x,
                        y,
                        crate::core::pixel::compose_rgba(new_r, new_g, new_b, 0),
                    );
                }
            }
        }

        Ok(rm.into())
    }
}

/// Mask a pixel value to the valid range for a given depth.
fn mask_val(val: u32, depth: PixelDepth) -> u32 {
    match depth {
        PixelDepth::Bit1 => val & 1,
        PixelDepth::Bit2 => val & 3,
        PixelDepth::Bit4 => val & 0xf,
        PixelDepth::Bit8 => val & 0xff,
        PixelDepth::Bit16 => val & 0xffff,
        PixelDepth::Bit32 => val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pix::PixelDepth;

    #[test]
    fn test_set_masked_8bpp() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Fill with 100
        for y in 0..3 {
            for x in 0..4 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }

        // Mask: ON at (0,0) and (2,1)
        let mask = Pix::new(4, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(2, 1, 1);
        let mask: Pix = mm.into();

        pm.set_masked(&mask, 255).unwrap();
        let pix: Pix = pm.into();

        assert_eq!(pix.get_pixel(0, 0), Some(255));
        assert_eq!(pix.get_pixel(2, 1), Some(255));
        assert_eq!(pix.get_pixel(1, 0), Some(100)); // unchanged
        assert_eq!(pix.get_pixel(3, 2), Some(100)); // unchanged
    }

    #[test]
    fn test_set_masked_32bpp() {
        let pix = Pix::new(3, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(3, 2, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(1, 0, 1);
        let mask: Pix = mm.into();

        pm.set_masked(&mask, 0xff000000).unwrap();
        let pix: Pix = pm.into();

        assert_eq!(pix.get_pixel(1, 0), Some(0xff000000));
        assert_eq!(pix.get_pixel(0, 0), Some(0)); // unchanged
    }

    #[test]
    fn test_set_masked_invalid_mask_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pm.set_masked(&mask, 0).is_err());
    }

    #[test]
    fn test_combine_masked_8bpp() {
        let dst = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        for y in 0..3 {
            for x in 0..4 {
                dm.set_pixel_unchecked(x, y, 50);
            }
        }

        let src = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut sm = src.try_into_mut().unwrap();
        for y in 0..3 {
            for x in 0..4 {
                sm.set_pixel_unchecked(x, y, 200);
            }
        }
        let src: Pix = sm.into();

        let mask = Pix::new(4, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        mm.set_pixel_unchecked(3, 2, 1);
        let mask: Pix = mm.into();

        dm.combine_masked(&src, &mask).unwrap();
        let dst: Pix = dm.into();

        assert_eq!(dst.get_pixel(0, 0), Some(200)); // from src
        assert_eq!(dst.get_pixel(3, 2), Some(200)); // from src
        assert_eq!(dst.get_pixel(1, 0), Some(50)); // unchanged
    }

    #[test]
    fn test_combine_masked_depth_mismatch() {
        let dst = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        let src = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        assert!(dm.combine_masked(&src, &mask).is_err());
    }

    #[test]
    fn test_paint_through_mask() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        // Small 3x3 mask with center ON
        let mask = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(1, 1, 1);
        let mask: Pix = mm.into();

        pm.paint_through_mask(&mask, 4, 4, 200).unwrap();
        let pix: Pix = pm.into();

        // Mask center (1,1) placed at (4,4) → pixel (5,5)
        assert_eq!(pix.get_pixel(5, 5), Some(200));
        assert_eq!(pix.get_pixel(4, 4), Some(0)); // not masked
    }

    #[test]
    fn test_paint_through_mask_negative_offset() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(3, 3, 1); // bottom-right of mask
        let mask: Pix = mm.into();

        pm.paint_through_mask(&mask, -2, -2, 128).unwrap();
        let pix: Pix = pm.into();

        // Mask (3,3) at offset (-2,-2) → pixel (1,1)
        assert_eq!(pix.get_pixel(1, 1), Some(128));
        assert_eq!(pix.get_pixel(0, 0), Some(0));
    }

    #[test]
    fn test_make_mask_from_val() {
        let pix = Pix::new(4, 3, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 42);
        pm.set_pixel_unchecked(2, 1, 42);
        pm.set_pixel_unchecked(3, 2, 42);
        pm.set_pixel_unchecked(1, 0, 100);
        let pix: Pix = pm.into();

        let mask = pix.make_mask_from_val(42).unwrap();
        assert_eq!(mask.depth(), PixelDepth::Bit1);
        assert_eq!(mask.width(), 4);
        assert_eq!(mask.height(), 3);
        assert_eq!(mask.get_pixel(0, 0), Some(1)); // matches
        assert_eq!(mask.get_pixel(2, 1), Some(1)); // matches
        assert_eq!(mask.get_pixel(3, 2), Some(1)); // matches
        assert_eq!(mask.get_pixel(1, 0), Some(0)); // doesn't match
        assert_eq!(mask.get_pixel(1, 1), Some(0)); // zero pixel
    }

    #[test]
    fn test_make_mask_from_val_invalid_depth() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        assert!(pix.make_mask_from_val(0).is_err());
    }

    #[test]
    fn test_make_mask_from_lut() {
        let pix = Pix::new(4, 2, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(0, 0, 10);
        pm.set_pixel_unchecked(1, 0, 20);
        pm.set_pixel_unchecked(2, 0, 30);
        pm.set_pixel_unchecked(3, 0, 10);
        pm.set_pixel_unchecked(0, 1, 20);
        let pix: Pix = pm.into();

        // LUT: mask values 10 and 30
        let mut lut = [0u8; 256];
        lut[10] = 1;
        lut[30] = 1;

        let mask = pix.make_mask_from_lut(&lut).unwrap();
        assert_eq!(mask.get_pixel(0, 0), Some(1)); // val=10 → 1
        assert_eq!(mask.get_pixel(1, 0), Some(0)); // val=20 → 0
        assert_eq!(mask.get_pixel(2, 0), Some(1)); // val=30 → 1
        assert_eq!(mask.get_pixel(3, 0), Some(1)); // val=10 → 1
        assert_eq!(mask.get_pixel(0, 1), Some(0)); // val=20 → 0
    }

    #[test]
    fn test_make_mask_from_lut_short_lut() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let lut = [0u8; 100]; // too short
        assert!(pix.make_mask_from_lut(&lut).is_err());
    }

    // -- PixMut::set_masked_general --

    #[test]
    fn test_set_masked_general_8bpp() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        // Small 3x3 mask with center ON
        let mask = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(1, 1, 1);
        let mask: Pix = mm.into();

        // Place mask at (5, 5)
        pm.set_masked_general(&mask, 200, 5, 5).unwrap();
        let pix: Pix = pm.into();

        // Mask center (1,1) at offset (5,5) → pixel (6,6) = 200
        assert_eq!(pix.get_pixel(6, 6), Some(200));
        assert_eq!(pix.get_pixel(5, 5), Some(0)); // not masked
    }

    #[test]
    fn test_set_masked_general_negative_offset() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(4, 4, 1); // bottom-right of mask
        let mask: Pix = mm.into();

        pm.set_masked_general(&mask, 128, -2, -2).unwrap();
        let pix: Pix = pm.into();

        // Mask (4,4) at offset (-2,-2) → pixel (2,2) = 128
        assert_eq!(pix.get_pixel(2, 2), Some(128));
    }

    #[test]
    fn test_set_masked_general_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();

        let mask = Pix::new(3, 3, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(0, 0, 1);
        let mask: Pix = mm.into();

        pm.set_masked_general(&mask, 0xFF000000, 2, 3).unwrap();
        let pix: Pix = pm.into();

        assert_eq!(pix.get_pixel(2, 3), Some(0xFF000000));
        assert_eq!(pix.get_pixel(3, 3), Some(0));
    }

    // -- PixMut::combine_masked_general --

    #[test]
    fn test_combine_masked_general_8bpp() {
        // Destination: all 50
        let dst = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                dm.set_pixel_unchecked(x, y, 50);
            }
        }

        // Source: all 200
        let src = Pix::new(5, 5, PixelDepth::Bit8).unwrap();
        let mut sm = src.try_into_mut().unwrap();
        for y in 0..5 {
            for x in 0..5 {
                sm.set_pixel_unchecked(x, y, 200);
            }
        }
        let src: Pix = sm.into();

        // Mask: ON at (2,2)
        let mask = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut mm = mask.try_into_mut().unwrap();
        mm.set_pixel_unchecked(2, 2, 1);
        let mask: Pix = mm.into();

        // Place src and mask at (10, 10)
        dm.combine_masked_general(&src, &mask, 10, 10).unwrap();
        let dst: Pix = dm.into();

        // Mask (2,2) at offset (10,10) → dst(12,12) = src(2,2) = 200
        assert_eq!(dst.get_pixel(12, 12), Some(200));
        assert_eq!(dst.get_pixel(10, 10), Some(50)); // not masked
    }

    #[test]
    fn test_combine_masked_general_depth_mismatch() {
        let dst = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut dm = dst.try_into_mut().unwrap();
        let src = Pix::new(5, 5, PixelDepth::Bit32).unwrap();
        let mask = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        assert!(dm.combine_masked_general(&src, &mask, 0, 0).is_err());
    }

    // -- Pix::copy_with_boxa --

    #[test]
    fn test_copy_with_boxa_white_bg() {
        use crate::core::box_::{Box, Boxa};
        use crate::core::pix::InitColor;

        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pm.into();

        let mut boxa = Boxa::new();
        boxa.push(Box::new(5, 5, 3, 3).unwrap());

        let result = pix.copy_with_boxa(&boxa, InitColor::White).unwrap();
        // Inside box: copied from source (100)
        assert_eq!(result.get_pixel(6, 6), Some(100));
        // Outside box: white background (255 for 8bpp)
        assert_eq!(result.get_pixel(0, 0), Some(255));
    }

    #[test]
    fn test_copy_with_boxa_black_bg() {
        use crate::core::box_::{Box, Boxa};
        use crate::core::pix::InitColor;

        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                pm.set_pixel_unchecked(x, y, 100);
            }
        }
        let pix: Pix = pm.into();

        let mut boxa = Boxa::new();
        boxa.push(Box::new(2, 2, 4, 4).unwrap());

        let result = pix.copy_with_boxa(&boxa, InitColor::Black).unwrap();
        assert_eq!(result.get_pixel(3, 3), Some(100));
        assert_eq!(result.get_pixel(0, 0), Some(0));
    }

    // -- Pix::make_arb_mask_from_rgb --

    #[test]
    fn test_make_arb_mask_from_rgb() {
        // Create a 32bpp image with some red pixels
        let pix = Pix::new(4, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Red pixel: R=200, G=0, B=0 → 200*1.0 + 0 + 0 = 200 > 100 → mask ON
        pm.set_pixel_unchecked(0, 0, crate::core::pixel::compose_rgba(200, 0, 0, 255));
        // Blue pixel: R=0, G=0, B=200 → 0*1.0 + 0 + 0 = 0 < 100 → mask OFF
        pm.set_pixel_unchecked(1, 0, crate::core::pixel::compose_rgba(0, 0, 200, 255));
        let pix: Pix = pm.into();

        // rc=1.0, gc=0.0, bc=0.0, thresh=100 → mask ON where red > 100
        let mask = pix.make_arb_mask_from_rgb(1.0, 0.0, 0.0, 100.0).unwrap();
        assert_eq!(mask.depth(), PixelDepth::Bit1);
        assert_eq!(mask.get_pixel(0, 0), Some(1)); // red pixel → ON
        assert_eq!(mask.get_pixel(1, 0), Some(0)); // blue pixel → OFF
    }

    #[test]
    fn test_make_arb_mask_from_rgb_not_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.make_arb_mask_from_rgb(1.0, 0.0, 0.0, 100.0).is_err());
    }

    // -- Pix::set_under_transparency --

    #[test]
    fn test_set_under_transparency() {
        let pix = Pix::new(4, 2, PixelDepth::Bit32).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        // Pixel with alpha=0 (fully transparent): R=10, G=20, B=30
        pm.set_pixel_unchecked(0, 0, crate::core::pixel::compose_rgba(10, 20, 30, 0));
        // Pixel with alpha=255 (opaque): R=100, G=150, B=200
        pm.set_pixel_unchecked(1, 0, crate::core::pixel::compose_rgba(100, 150, 200, 255));
        let pix: Pix = pm.into();

        // Set transparent pixels to white (0xFFFFFF00)
        let result = pix.set_under_transparency(0xFFFFFF00).unwrap();

        // Transparent pixel → replaced with white RGB, alpha preserved at 0
        let (r, g, b, a) = crate::core::pixel::extract_rgba(result.get_pixel(0, 0).unwrap());
        assert_eq!((r, g, b), (255, 255, 255));
        assert_eq!(a, 0);

        // Opaque pixel → unchanged
        let (r, g, b, a) = crate::core::pixel::extract_rgba(result.get_pixel(1, 0).unwrap());
        assert_eq!((r, g, b, a), (100, 150, 200, 255));
    }

    #[test]
    fn test_set_under_transparency_not_32bpp() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(pix.set_under_transparency(0xFFFFFF00).is_err());
    }
}
