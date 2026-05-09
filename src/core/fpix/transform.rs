//! Orthogonal rotations, flips and border extensions for FPix.
//!
//! C Leptonica equivalent: portions of `fpix2.c` (`fpixRotateOrth`,
//! `fpixRotate90`, `fpixRotate180`, `fpixFlipLR`, `fpixFlipTB`,
//! `fpixAddBorder`, `fpixAddMirroredBorder`, `fpixAddContinuedBorder`).

use crate::core::error::{Error, Result};
use crate::core::fpix::FPix;

/// Direction of a 90° rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateDirection {
    /// Clockwise rotation.
    Cw,
    /// Counter-clockwise rotation.
    Ccw,
}

impl FPix {
    /// Rotate by `quads * 90°` clockwise. `quads` must be in `0..=3`.
    ///
    /// C Leptonica equivalent: `fpixRotateOrth`
    pub fn rotate_orth(&self, quads: u8) -> Result<FPix> {
        match quads {
            0 => Ok(self.clone()),
            1 => self.rotate_90(RotateDirection::Cw),
            2 => self.rotate_180(),
            3 => self.rotate_90(RotateDirection::Ccw),
            _ => Err(Error::InvalidParameter(format!(
                "quads must be in 0..=3, got {quads}"
            ))),
        }
    }

    /// Rotate 90° in the given direction.
    ///
    /// C Leptonica equivalent: `fpixRotate90`
    pub fn rotate_90(&self, direction: RotateDirection) -> Result<FPix> {
        let (ws, hs) = (self.width(), self.height());
        let (wd, hd) = (hs, ws);
        let mut dst = FPix::new(wd, hd)?;
        let (xres, yres) = self.resolution();
        dst.set_resolution(xres, yres);

        match direction {
            RotateDirection::Cw => {
                // dst[x, y] = src[y, hs - 1 - x]
                for y in 0..hd {
                    for x in 0..wd {
                        let v = self.get_pixel_unchecked(y, hs - 1 - x);
                        dst.set_pixel_unchecked(x, y, v);
                    }
                }
            }
            RotateDirection::Ccw => {
                // dst[x, y] = src[ws - 1 - y, x]
                for y in 0..hd {
                    for x in 0..wd {
                        let v = self.get_pixel_unchecked(ws - 1 - y, x);
                        dst.set_pixel_unchecked(x, y, v);
                    }
                }
            }
        }
        Ok(dst)
    }

    /// Rotate 180° (= LR flip + TB flip).
    ///
    /// C Leptonica equivalent: `fpixRotate180`
    pub fn rotate_180(&self) -> Result<FPix> {
        self.flip_lr()?.flip_tb()
    }

    /// Flip left-right.
    ///
    /// C Leptonica equivalent: `fpixFlipLR`
    pub fn flip_lr(&self) -> Result<FPix> {
        let (w, h) = (self.width(), self.height());
        let mut dst = FPix::new(w, h)?;
        let (xres, yres) = self.resolution();
        dst.set_resolution(xres, yres);
        for y in 0..h {
            for x in 0..w {
                let v = self.get_pixel_unchecked(w - 1 - x, y);
                dst.set_pixel_unchecked(x, y, v);
            }
        }
        Ok(dst)
    }

    /// Flip top-bottom.
    ///
    /// C Leptonica equivalent: `fpixFlipTB`
    pub fn flip_tb(&self) -> Result<FPix> {
        let (w, h) = (self.width(), self.height());
        let mut dst = FPix::new(w, h)?;
        let (xres, yres) = self.resolution();
        dst.set_resolution(xres, yres);
        for y in 0..h {
            // Copy row (h - 1 - y) of src into row y of dst.
            let src_y = h - 1 - y;
            for x in 0..w {
                let v = self.get_pixel_unchecked(x, src_y);
                dst.set_pixel_unchecked(x, y, v);
            }
        }
        Ok(dst)
    }

    /// Extend the FPix on each side, filling the border with `fill`.
    ///
    /// C Leptonica equivalent: `fpixAddBorder`
    pub fn add_border(&self, left: u32, right: u32, top: u32, bot: u32, fill: f32) -> Result<FPix> {
        let (ws, hs) = (self.width(), self.height());
        let wd = ws + left + right;
        let hd = hs + top + bot;
        let mut dst = FPix::new_with_value(wd, hd, fill)?;
        let (xres, yres) = self.resolution();
        dst.set_resolution(xres, yres);
        for y in 0..hs {
            for x in 0..ws {
                let v = self.get_pixel_unchecked(x, y);
                dst.set_pixel_unchecked(x + left, y + top, v);
            }
        }
        Ok(dst)
    }

    /// Extend the FPix on each side using mirror reflection.
    ///
    /// C Leptonica equivalent: `fpixAddMirroredBorder`
    pub fn add_mirrored_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<FPix> {
        let mut dst = self.add_border(left, right, top, bot, 0.0)?;
        let (ws, hs) = (self.width(), self.height());
        let wd = dst.width();

        // Left/right columns: mirror against the boundary just inside the border.
        for j in 0..left {
            for y in 0..hs {
                let v = dst.get_pixel_unchecked(left + j, top + y);
                dst.set_pixel_unchecked(left - 1 - j, top + y, v);
            }
        }
        for j in 0..right {
            for y in 0..hs {
                let v = dst.get_pixel_unchecked(left + ws - 1 - j, top + y);
                dst.set_pixel_unchecked(left + ws + j, top + y, v);
            }
        }
        // Top/bottom rows: mirror the entire row (now including the freshly
        // filled left/right border columns).
        for i in 0..top {
            for x in 0..wd {
                let v = dst.get_pixel_unchecked(x, top + i);
                dst.set_pixel_unchecked(x, top - 1 - i, v);
            }
        }
        for i in 0..bot {
            for x in 0..wd {
                let v = dst.get_pixel_unchecked(x, top + hs - 1 - i);
                dst.set_pixel_unchecked(x, top + hs + i, v);
            }
        }
        Ok(dst)
    }

    /// Extend the FPix on each side by replicating the boundary value.
    ///
    /// C Leptonica equivalent: `fpixAddContinuedBorder`
    pub fn add_continued_border(&self, left: u32, right: u32, top: u32, bot: u32) -> Result<FPix> {
        let mut dst = self.add_border(left, right, top, bot, 0.0)?;
        let (ws, hs) = (self.width(), self.height());
        let wd = dst.width();

        for j in 0..left {
            for y in 0..hs {
                let v = dst.get_pixel_unchecked(left, top + y);
                dst.set_pixel_unchecked(j, top + y, v);
            }
        }
        for j in 0..right {
            for y in 0..hs {
                let v = dst.get_pixel_unchecked(left + ws - 1, top + y);
                dst.set_pixel_unchecked(left + ws + j, top + y, v);
            }
        }
        for i in 0..top {
            for x in 0..wd {
                let v = dst.get_pixel_unchecked(x, top);
                dst.set_pixel_unchecked(x, i, v);
            }
        }
        for i in 0..bot {
            for x in 0..wd {
                let v = dst.get_pixel_unchecked(x, top + hs - 1);
                dst.set_pixel_unchecked(x, top + hs + i, v);
            }
        }
        Ok(dst)
    }
}
