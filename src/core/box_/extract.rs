//! Box array extraction and statistics
//!
//! Functions for extracting box parameters as Numa or Pta arrays,
//! and computing statistics (rank, median, average).
//!
//! C Leptonica equivalents: boxfunc2.c

use crate::core::error::{Error, Result};
use crate::core::numa::Numa;
use crate::core::pta::Pta;

use super::Boxa;

/// Which box field to extract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxField {
    /// Left x coordinate
    X,
    /// Top y coordinate
    Y,
    /// Right side (x + w - 1)
    Right,
    /// Bottom side (y + h - 1)
    Bottom,
    /// Width
    Width,
    /// Height
    Height,
}

/// Corner location to extract.
///
/// C Leptonica equivalents: `L_UPPER_LEFT`, `L_UPPER_RIGHT`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerLocation {
    /// Upper-left corner
    UpperLeft,
    /// Upper-right corner
    UpperRight,
    /// Lower-left corner
    LowerLeft,
    /// Lower-right corner
    LowerRight,
    /// Center of the box
    Center,
}

impl Boxa {
    /// Extract box parameters as separate Numa arrays.
    ///
    /// Returns `(left, top, right, bottom, width, height)`.
    /// If `keep_invalid` is false, boxes with w<=0 or h<=0 are skipped.
    ///
    /// C Leptonica equivalent: `boxaExtractAsNuma`
    pub fn extract_as_numa(&self, keep_invalid: bool) -> (Numa, Numa, Numa, Numa, Numa, Numa) {
        let n = self.len();
        let mut nal = Numa::with_capacity(n);
        let mut nat = Numa::with_capacity(n);
        let mut nar = Numa::with_capacity(n);
        let mut nab = Numa::with_capacity(n);
        let mut naw = Numa::with_capacity(n);
        let mut nah = Numa::with_capacity(n);

        for b in self.iter() {
            if !keep_invalid && (b.w <= 0 || b.h <= 0) {
                continue;
            }
            let right = b.x + b.w - 1;
            let bot = b.y + b.h - 1;
            nal.push(b.x as f32);
            nat.push(b.y as f32);
            nar.push(right as f32);
            nab.push(bot as f32);
            naw.push(b.w as f32);
            nah.push(b.h as f32);
        }

        (nal, nat, nar, nab, naw, nah)
    }

    /// Extract box parameters as separate Pta arrays.
    ///
    /// Each Pta stores (index, value) pairs.
    /// Returns `(left, top, right, bottom, width, height)`.
    ///
    /// C Leptonica equivalent: `boxaExtractAsPta`
    pub fn extract_as_pta(&self, keep_invalid: bool) -> (Pta, Pta, Pta, Pta, Pta, Pta) {
        let n = self.len();
        let mut pal = Pta::with_capacity(n);
        let mut pat = Pta::with_capacity(n);
        let mut par = Pta::with_capacity(n);
        let mut pab = Pta::with_capacity(n);
        let mut paw = Pta::with_capacity(n);
        let mut pah = Pta::with_capacity(n);

        for (i, b) in self.iter().enumerate() {
            if !keep_invalid && (b.w <= 0 || b.h <= 0) {
                continue;
            }
            let right = b.x + b.w - 1;
            let bot = b.y + b.h - 1;
            let idx = i as f32;
            pal.push(idx, b.x as f32);
            pat.push(idx, b.y as f32);
            par.push(idx, right as f32);
            pab.push(idx, bot as f32);
            paw.push(idx, b.w as f32);
            pah.push(idx, b.h as f32);
        }

        (pal, pat, par, pab, paw, pah)
    }

    /// Extract specified corners as a Pta.
    ///
    /// Invalid boxes (w==0 or h==0) produce (0, 0).
    ///
    /// C Leptonica equivalent: `boxaExtractCorners`
    pub fn extract_corners(&self, loc: CornerLocation) -> Pta {
        let n = self.len();
        let mut pta = Pta::with_capacity(n);

        for b in self.iter() {
            let (left, top, w, h) = (b.x, b.y, b.w, b.h);
            if w == 0 || h == 0 {
                pta.push(0.0, 0.0);
                continue;
            }
            let right = left + w - 1;
            let bot = top + h - 1;
            match loc {
                CornerLocation::UpperLeft => pta.push(left as f32, top as f32),
                CornerLocation::UpperRight => pta.push(right as f32, top as f32),
                CornerLocation::LowerLeft => pta.push(left as f32, bot as f32),
                CornerLocation::LowerRight => pta.push(right as f32, bot as f32),
                CornerLocation::Center => {
                    pta.push((left + right) as f32 / 2.0, (top + bot) as f32 / 2.0);
                }
            }
        }

        pta
    }

    /// Get rank-order values for box parameters.
    ///
    /// `fract` is 0.0 for smallest, 1.0 for largest.
    /// x and y are sorted in decreasing order; r, b, w, h in increasing order.
    /// Returns `(x, y, right, bottom, width, height)`.
    ///
    /// C Leptonica equivalent: `boxaGetRankVals`
    pub fn get_rank_vals(&self, fract: f32) -> Result<(i32, i32, i32, i32, i32, i32)> {
        if !(0.0..=1.0).contains(&fract) {
            return Err(Error::InvalidParameter(format!(
                "fract {fract} not in [0.0, 1.0]"
            )));
        }
        let (nax, nay, nar, nab, naw, nah) = self.extract_as_numa(false);
        if nax.is_empty() {
            return Err(Error::InvalidParameter("no valid boxes in boxa".into()));
        }

        // x and y sorted in decreasing order (rank = 1.0 - fract)
        let xval = nax.rank_value(1.0 - fract)? as i32;
        let yval = nay.rank_value(1.0 - fract)? as i32;
        // r, b, w, h sorted in increasing order
        let rval = nar.rank_value(fract)? as i32;
        let bval = nab.rank_value(fract)? as i32;
        let wval = naw.rank_value(fract)? as i32;
        let hval = nah.rank_value(fract)? as i32;

        Ok((xval, yval, rval, bval, wval, hval))
    }

    /// Get median values for box parameters.
    ///
    /// Returns `(x, y, right, bottom, width, height)`.
    ///
    /// C Leptonica equivalent: `boxaGetMedianVals`
    pub fn get_median_vals(&self) -> Result<(i32, i32, i32, i32, i32, i32)> {
        self.get_rank_vals(0.5)
    }

    /// Get average width and height.
    ///
    /// Returns `(average_width, average_height)`.
    ///
    /// C Leptonica equivalent: `boxaGetAverageSize`
    pub fn get_average_size(&self) -> Result<(f32, f32)> {
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("boxa is empty".into()));
        }
        let mut sum_w: f32 = 0.0;
        let mut sum_h: f32 = 0.0;
        for b in self.iter() {
            sum_w += b.w as f32;
            sum_h += b.h as f32;
        }
        Ok((sum_w / n as f32, sum_h / n as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::box_::Box;

    fn sample_boxa() -> Boxa {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        boxa.push(Box::new_unchecked(50, 60, 70, 80));
        boxa.push(Box::new_unchecked(100, 110, 120, 130));
        boxa
    }

    #[test]
    fn test_extract_as_numa() {
        let boxa = sample_boxa();
        let (nal, nat, nar, nab, naw, nah) = boxa.extract_as_numa(true);
        assert_eq!(nal.len(), 3);
        assert_eq!(nal[0], 10.0);
        assert_eq!(nat[1], 60.0);
        assert_eq!(nar[0], 39.0); // 10 + 30 - 1
        assert_eq!(nab[0], 59.0); // 20 + 40 - 1
        assert_eq!(naw[2], 120.0);
        assert_eq!(nah[2], 130.0);
    }

    #[test]
    fn test_extract_as_numa_skip_invalid() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 30, 40));
        boxa.push(Box::new_unchecked(0, 0, 0, 0)); // invalid
        boxa.push(Box::new_unchecked(50, 60, 70, 80));
        let (nal, _, _, _, _, _) = boxa.extract_as_numa(false);
        assert_eq!(nal.len(), 2);
    }

    #[test]
    fn test_extract_as_pta() {
        let boxa = sample_boxa();
        let (pal, pat, _, _, _, _) = boxa.extract_as_pta(true);
        assert_eq!(pal.len(), 3);
        let (idx, val) = pal.get(0).unwrap();
        assert_eq!(idx, 0.0);
        assert_eq!(val, 10.0);
        let (idx, val) = pat.get(1).unwrap();
        assert_eq!(idx, 1.0);
        assert_eq!(val, 60.0);
    }

    #[test]
    fn test_extract_corners_upper_left() {
        let boxa = sample_boxa();
        let pta = boxa.extract_corners(CornerLocation::UpperLeft);
        assert_eq!(pta.len(), 3);
        assert_eq!(pta.get(0).unwrap(), (10.0, 20.0));
    }

    #[test]
    fn test_extract_corners_lower_right() {
        let boxa = sample_boxa();
        let pta = boxa.extract_corners(CornerLocation::LowerRight);
        assert_eq!(pta.get(0).unwrap(), (39.0, 59.0)); // (10+30-1, 20+40-1)
    }

    #[test]
    fn test_extract_corners_center() {
        let boxa = sample_boxa();
        let pta = boxa.extract_corners(CornerLocation::Center);
        // box(10,20,30,40): center = ((10+39)/2, (20+59)/2) = (24.5, 39.5)
        let (cx, cy) = pta.get(0).unwrap();
        assert!((cx - 24.5).abs() < 0.01);
        assert!((cy - 39.5).abs() < 0.01);
    }

    #[test]
    fn test_extract_corners_invalid_box() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 0, 0));
        let pta = boxa.extract_corners(CornerLocation::UpperLeft);
        assert_eq!(pta.get(0).unwrap(), (0.0, 0.0));
    }

    #[test]
    fn test_get_rank_vals() {
        let boxa = sample_boxa();
        let (x, y, r, b, w, h) = boxa.get_rank_vals(0.5).unwrap();
        assert_eq!(w, 70); // median width
        assert_eq!(h, 80); // median height
        assert!(x > 0);
        assert!(y > 0);
        assert!(r > 0);
        assert!(b > 0);
    }

    #[test]
    fn test_get_rank_vals_invalid_fract() {
        let boxa = sample_boxa();
        assert!(boxa.get_rank_vals(1.5).is_err());
        assert!(boxa.get_rank_vals(-0.1).is_err());
    }

    #[test]
    fn test_get_median_vals() {
        let boxa = sample_boxa();
        let (_, _, _, _, w, h) = boxa.get_median_vals().unwrap();
        assert_eq!(w, 70);
        assert_eq!(h, 80);
    }

    #[test]
    fn test_get_average_size() {
        let boxa = sample_boxa();
        let (avg_w, avg_h) = boxa.get_average_size().unwrap();
        // (30 + 70 + 120) / 3 = 73.33..
        assert!((avg_w - 73.333).abs() < 0.01);
        // (40 + 80 + 130) / 3 = 83.33..
        assert!((avg_h - 83.333).abs() < 0.01);
    }

    #[test]
    fn test_get_average_size_empty() {
        let boxa = Boxa::new();
        assert!(boxa.get_average_size().is_err());
    }
}
