//! Box/Boxa transform (ordered) and orthogonal rotation
//!
//! Functions for ordered transforms (shift, scale, rotate) and
//! orthogonal rotation (90/180/270 degrees).
//!
//! C Leptonica equivalents: boxfunc2.c

use crate::core::error::{Error, Result};
use crate::core::pta::Pta;

use super::{Box, Boxa};

/// Order of transform operations: translate, scale, rotate.
///
/// C Leptonica equivalents: `L_TR_SC_RO`, `L_SC_RO_TR`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformOrder {
    /// Translate, then Scale, then Rotate
    TrScRo,
    /// Scale, then Rotate, then Translate
    ScRoTr,
    /// Rotate, then Translate, then Scale
    RoTrSc,
    /// Translate, then Rotate, then Scale
    TrRoSc,
    /// Rotate, then Scale, then Translate
    RoScTr,
    /// Scale, then Translate, then Rotate
    ScTrRo,
}

impl Box {
    /// Apply an ordered sequence of shift, scale, and rotation transforms.
    ///
    /// The rotation is about the point (`xcen`, `ycen`) specified before
    /// any transforms. The center is adjusted internally based on the order.
    ///
    /// C Leptonica equivalent: `boxTransformOrdered`
    #[allow(clippy::too_many_arguments)]
    pub fn transform_ordered(
        &self,
        shiftx: i32,
        shifty: i32,
        scalex: f32,
        scaley: f32,
        xcen: i32,
        ycen: i32,
        angle: f32,
        order: TransformOrder,
    ) -> Box {
        let (bx, by, bw, bh) = (self.x, self.y, self.w, self.h);
        if bw <= 0 || bh <= 0 {
            return Box::new_unchecked(0, 0, 0, 0);
        }

        let (sina, cosa) = if angle != 0.0 {
            (angle.sin(), angle.cos())
        } else {
            (0.0, 1.0)
        };

        // Helper: rotate a box (tx,ty,tw,th) about (cx,cy)
        let rotate_box =
            |tx: f32, ty: f32, tw: f32, th: f32, cx: f32, cy: f32| -> (f32, f32, f32, f32) {
                if angle == 0.0 {
                    return (tx, ty, tw, th);
                }
                let xdif = tx + 0.5 * tw - cx;
                let ydif = ty + 0.5 * th - cy;
                let rw = (tw * cosa).abs() + (th * sina).abs();
                let rh = (th * cosa).abs() + (tw * sina).abs();
                let rx = cx + xdif * cosa - ydif * sina - 0.5 * rw;
                let ry = cy + ydif * cosa + xdif * sina - 0.5 * rh;
                (rx, ry, rw, rh)
            };

        match order {
            TransformOrder::TrScRo => {
                let tx = (scalex * (bx + shiftx) as f32 + 0.5) as i32;
                let ty = (scaley * (by + shifty) as f32 + 0.5) as i32;
                let tw = (1.0_f32.max(scalex * bw as f32 + 0.5)) as i32;
                let th = (1.0_f32.max(scaley * bh as f32 + 0.5)) as i32;
                let xcent = (scalex * xcen as f32 + 0.5) as i32;
                let ycent = (scaley * ycen as f32 + 0.5) as i32;
                if angle == 0.0 {
                    Box::new_unchecked(tx, ty, tw, th)
                } else {
                    let (rx, ry, rw, rh) = rotate_box(
                        tx as f32,
                        ty as f32,
                        tw as f32,
                        th as f32,
                        xcent as f32,
                        ycent as f32,
                    );
                    Box::new_unchecked(rx as i32, ry as i32, rw as i32, rh as i32)
                }
            }
            TransformOrder::ScTrRo => {
                let tx = (scalex * bx as f32 + shiftx as f32 + 0.5) as i32;
                let ty = (scaley * by as f32 + shifty as f32 + 0.5) as i32;
                let tw = (1.0_f32.max(scalex * bw as f32 + 0.5)) as i32;
                let th = (1.0_f32.max(scaley * bh as f32 + 0.5)) as i32;
                let xcent = (scalex * xcen as f32 + 0.5) as i32;
                let ycent = (scaley * ycen as f32 + 0.5) as i32;
                if angle == 0.0 {
                    Box::new_unchecked(tx, ty, tw, th)
                } else {
                    let (rx, ry, rw, rh) = rotate_box(
                        tx as f32,
                        ty as f32,
                        tw as f32,
                        th as f32,
                        xcent as f32,
                        ycent as f32,
                    );
                    Box::new_unchecked(rx as i32, ry as i32, rw as i32, rh as i32)
                }
            }
            TransformOrder::RoTrSc => {
                let (rx, ry, rw, rh) = rotate_box(
                    bx as f32,
                    by as f32,
                    bw as f32,
                    bh as f32,
                    xcen as f32,
                    ycen as f32,
                );
                let tx = (scalex * (rx + shiftx as f32) + 0.5) as i32;
                let ty = (scaley * (ry + shifty as f32) + 0.5) as i32;
                let tw = (1.0_f32.max(scalex * rw + 0.5)) as i32;
                let th = (1.0_f32.max(scaley * rh + 0.5)) as i32;
                Box::new_unchecked(tx, ty, tw, th)
            }
            TransformOrder::RoScTr => {
                let (rx, ry, rw, rh) = rotate_box(
                    bx as f32,
                    by as f32,
                    bw as f32,
                    bh as f32,
                    xcen as f32,
                    ycen as f32,
                );
                let tx = (scalex * rx + shiftx as f32 + 0.5) as i32;
                let ty = (scaley * ry + shifty as f32 + 0.5) as i32;
                let tw = (1.0_f32.max(scalex * rw + 0.5)) as i32;
                let th = (1.0_f32.max(scaley * rh + 0.5)) as i32;
                Box::new_unchecked(tx, ty, tw, th)
            }
            TransformOrder::TrRoSc => {
                let tx = bx + shiftx;
                let ty = by + shifty;
                let (rx, ry, rw, rh) = rotate_box(
                    tx as f32,
                    ty as f32,
                    bw as f32,
                    bh as f32,
                    xcen as f32,
                    ycen as f32,
                );
                let fx = (scalex * rx + 0.5) as i32;
                let fy = (scaley * ry + 0.5) as i32;
                let fw = (1.0_f32.max(scalex * rw + 0.5)) as i32;
                let fh = (1.0_f32.max(scaley * rh + 0.5)) as i32;
                Box::new_unchecked(fx, fy, fw, fh)
            }
            TransformOrder::ScRoTr => {
                let tx = (scalex * bx as f32 + 0.5) as i32;
                let ty = (scaley * by as f32 + 0.5) as i32;
                let tw = (1.0_f32.max(scalex * bw as f32 + 0.5)) as i32;
                let th = (1.0_f32.max(scaley * bh as f32 + 0.5)) as i32;
                let xcent = (scalex * xcen as f32 + 0.5) as i32;
                let ycent = (scaley * ycen as f32 + 0.5) as i32;
                let (rx, ry, rw, rh) = rotate_box(
                    tx as f32,
                    ty as f32,
                    tw as f32,
                    th as f32,
                    xcent as f32,
                    ycent as f32,
                );
                let fx = (rx + shiftx as f32 + 0.5) as i32;
                let fy = (ry + shifty as f32 + 0.5) as i32;
                let fw = (rw + 0.5) as i32;
                let fh = (rh + 0.5) as i32;
                Box::new_unchecked(fx, fy, fw, fh)
            }
        }
    }

    /// Rotate the box by an orthogonal angle (0, 90, 180, or 270 degrees CW).
    ///
    /// `w` and `h` are the image dimensions in which the box is embedded.
    /// `rotation`: 0=noop, 1=90° CW, 2=180°, 3=270° CW.
    ///
    /// C Leptonica equivalent: `boxRotateOrth`
    pub fn rotate_orth(&self, w: i32, h: i32, rotation: i32) -> Result<Box> {
        if !(0..=3).contains(&rotation) {
            return Err(Error::InvalidParameter(format!(
                "rotation must be 0..3, got {rotation}"
            )));
        }
        if rotation == 0 {
            return Ok(*self);
        }
        let (bx, by, bw, bh) = (self.x, self.y, self.w, self.h);
        if bw <= 0 || bh <= 0 {
            return Ok(Box::new_unchecked(0, 0, 0, 0));
        }
        let ydist = h - by - bh;
        let xdist = w - bx - bw;
        match rotation {
            1 => Ok(Box::new_unchecked(ydist, bx, bh, bw)),
            2 => Ok(Box::new_unchecked(xdist, ydist, bw, bh)),
            3 => Ok(Box::new_unchecked(by, xdist, bh, bw)),
            _ => unreachable!(),
        }
    }
}

impl Boxa {
    /// Apply an ordered transform to all boxes.
    ///
    /// C Leptonica equivalent: `boxaTransformOrdered`
    #[allow(clippy::too_many_arguments)]
    pub fn transform_ordered(
        &self,
        shiftx: i32,
        shifty: i32,
        scalex: f32,
        scaley: f32,
        xcen: i32,
        ycen: i32,
        angle: f32,
        order: TransformOrder,
    ) -> Boxa {
        self.iter()
            .map(|b| b.transform_ordered(shiftx, shifty, scalex, scaley, xcen, ycen, angle, order))
            .collect()
    }

    /// Rotate all boxes by an orthogonal angle.
    ///
    /// `w`, `h`: image dimensions; `rotation`: 0=noop, 1=90° CW, 2=180°, 3=270° CW.
    ///
    /// C Leptonica equivalent: `boxaRotateOrth`
    pub fn rotate_orth(&self, w: i32, h: i32, rotation: i32) -> Result<Boxa> {
        if !(0..=3).contains(&rotation) {
            return Err(Error::InvalidParameter(format!(
                "rotation must be 0..3, got {rotation}"
            )));
        }
        if rotation == 0 {
            return Ok(self.clone());
        }
        self.iter().map(|b| b.rotate_orth(w, h, rotation)).collect()
    }

    /// Shift each box by the corresponding point in the Pta.
    ///
    /// `dir`: +1 shifts by (x, y) from pta; -1 shifts by (-x, -y).
    ///
    /// C Leptonica equivalent: `boxaShiftWithPta`
    pub fn shift_with_pta(&self, pta: &Pta, dir: i32) -> Result<Boxa> {
        if dir != 1 && dir != -1 {
            return Err(Error::InvalidParameter(format!(
                "dir must be 1 or -1, got {dir}"
            )));
        }
        let n = self.len();
        if n != pta.len() {
            return Err(Error::InvalidParameter(format!(
                "boxa length {} != pta length {}",
                n,
                pta.len()
            )));
        }
        let mut result = Boxa::with_capacity(n);
        for i in 0..n {
            let b = self.get(i).unwrap();
            let (px, py) = pta.get(i).unwrap();
            let dx = dir as f32 * px;
            let dy = dir as f32 * py;
            result.push(b.translate(dx.round() as i32, dy.round() as i32));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::box_::Box;

    #[test]
    fn test_transform_ordered_no_rotation() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        let result = b.transform_ordered(5, 10, 2.0, 2.0, 0, 0, 0.0, TransformOrder::TrScRo);
        assert_eq!(result.x, 30);
        assert_eq!(result.y, 60);
    }

    #[test]
    fn test_transform_ordered_invalid_box() {
        let b = Box::new_unchecked(10, 20, 0, 0);
        let result = b.transform_ordered(5, 10, 2.0, 2.0, 0, 0, 0.0, TransformOrder::TrScRo);
        assert_eq!(result, Box::new_unchecked(0, 0, 0, 0));
    }

    #[test]
    fn test_rotate_orth_90() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        let r = b.rotate_orth(200, 300, 1).unwrap();
        // ydist = 300 - 20 - 40 = 240, xdist = 200 - 10 - 30 = 160
        assert_eq!(r, Box::new_unchecked(240, 10, 40, 30));
    }

    #[test]
    fn test_rotate_orth_180() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        let r = b.rotate_orth(200, 300, 2).unwrap();
        assert_eq!(r, Box::new_unchecked(160, 240, 30, 40));
    }

    #[test]
    fn test_rotate_orth_270() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        let r = b.rotate_orth(200, 300, 3).unwrap();
        assert_eq!(r, Box::new_unchecked(20, 160, 40, 30));
    }

    #[test]
    fn test_rotate_orth_noop() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        let r = b.rotate_orth(200, 300, 0).unwrap();
        assert_eq!(r, b);
    }

    #[test]
    fn test_rotate_orth_invalid() {
        let b = Box::new_unchecked(10, 20, 30, 40);
        assert!(b.rotate_orth(200, 300, 4).is_err());
    }

    #[test]
    fn test_boxa_rotate_orth() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        boxa.push(Box::new_unchecked(50, 60, 70, 80));
        let result = boxa.rotate_orth(200, 300, 1).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().w, 40);
    }

    #[test]
    fn test_shift_with_pta() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        boxa.push(Box::new_unchecked(50, 60, 70, 80));
        let mut pta = Pta::new();
        pta.push(5.0, 10.0);
        pta.push(15.0, 20.0);
        let result = boxa.shift_with_pta(&pta, 1).unwrap();
        assert_eq!(result.get(0).unwrap().x, 15);
        assert_eq!(result.get(0).unwrap().y, 30);
        assert_eq!(result.get(1).unwrap().x, 65);
    }

    #[test]
    fn test_shift_with_pta_negative() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        let mut pta = Pta::new();
        pta.push(5.0, 10.0);
        let result = boxa.shift_with_pta(&pta, -1).unwrap();
        assert_eq!(result.get(0).unwrap().x, 5);
        assert_eq!(result.get(0).unwrap().y, 10);
    }

    #[test]
    fn test_shift_with_pta_mismatched_length() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        let pta = Pta::new();
        assert!(boxa.shift_with_pta(&pta, 1).is_err());
    }
}
