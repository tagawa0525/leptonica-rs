//! Interpolation, sampling, differentiation, integration, and signal
//! feature extraction for Numa arrays.
//!
//! Corresponds to interpolation-related functions in C Leptonica's
//! `numafunc1.c`.

use super::{InterpolationType, Numa};
use crate::error::{Error, Result};

impl Numa {
    /// Interpolate at evenly-spaced x values over an interval.
    ///
    /// `self` is the y-value array with x-parameters `(startx, delx)`.
    /// Evaluates the interpolated function at `npts` equally-spaced points
    /// from `x0` to `x1` inclusive. Returns the y-value array.
    ///
    /// C equivalent: `numaInterpolateEqxInterval()` in `numafunc1.c`
    pub fn interpolate_eqx_interval(
        &self,
        interp_type: InterpolationType,
        x0: f32,
        x1: f32,
        npts: usize,
    ) -> Result<Numa> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter("n < 2".to_string()));
        }
        let (startx, deltax) = self.parameters();
        if deltax <= 0.0 {
            return Err(Error::InvalidParameter("deltax must be > 0".to_string()));
        }
        if npts < 3 {
            return Err(Error::InvalidParameter("npts must be >= 3".to_string()));
        }
        let maxx = startx + deltax * (n as f32 - 1.0);
        if x0 < startx || x1 > maxx || x1 <= x0 {
            return Err(Error::InvalidParameter(
                "[x0, x1] interval is out of range".to_string(),
            ));
        }
        let delx = (x1 - x0) / (npts as f32 - 1.0);
        let mut nay = Numa::new();
        nay.set_parameters(x0, delx);
        for i in 0..npts {
            let x = x0 + i as f32 * delx;
            let yval = self.interpolate_eqx_val(interp_type, x)?;
            nay.push(yval);
        }
        Ok(nay)
    }

    /// Interpolate at evenly-spaced x values over an interval using
    /// arbitrary x-value arrays.
    ///
    /// `self` is the x-value array, `nay` is the corresponding y-value array.
    /// Returns interpolated y values at `npts` equally-spaced points from
    /// `x0` to `x1`.
    ///
    /// C equivalent: `numaInterpolateArbxInterval()` in `numafunc1.c`
    pub fn interpolate_arbx_interval(
        &self,
        nay: &Numa,
        interp_type: InterpolationType,
        x0: f32,
        x1: f32,
        npts: usize,
    ) -> Result<Numa> {
        let nx = self.len();
        let ny = nay.len();
        if nx != ny {
            return Err(Error::InvalidParameter(
                "nax and nay must have same length".to_string(),
            ));
        }
        if ny < 2 {
            return Err(Error::InvalidParameter("not enough points".to_string()));
        }
        if npts < 2 {
            return Err(Error::InvalidParameter("npts must be >= 2".to_string()));
        }
        if x0 > x1 {
            return Err(Error::InvalidParameter("x0 > x1".to_string()));
        }
        let minx = self.min_value().unwrap();
        let maxx = self.max_value().unwrap();
        if x0 < minx || x1 > maxx {
            return Err(Error::InvalidParameter("xval is out of bounds".to_string()));
        }

        // Ensure nax is sorted increasing; sort if needed
        let (nasx, nasy) = if self.is_sorted(super::SortOrder::Increasing) {
            (self.clone(), nay.clone())
        } else {
            self.sort_pair(nay, super::SortOrder::Increasing)
        };

        let fax = nasx.as_slice();
        let fay = nasy.as_slice();

        // Build index array: for each output point, find the position in fax
        let del = (x1 - x0) / (npts as f32 - 1.0);
        let mut index = vec![0usize; npts];
        let mut j = 0usize;
        for (i, idx) in index.iter_mut().enumerate() {
            let xval = x0 + i as f32 * del;
            while j < nx - 1 && xval > fax[j] {
                j += 1;
            }
            *idx = if xval == fax[j] {
                j.min(nx - 1)
            } else if j > 0 {
                j - 1
            } else {
                0
            };
        }

        // Choose effective interpolation type
        let effective_type = if interp_type == InterpolationType::Quadratic && ny == 2 {
            InterpolationType::Linear
        } else {
            interp_type
        };

        let mut nady = Numa::new();
        for (i, &im) in index.iter().enumerate() {
            let xval = x0 + i as f32 * del;

            // Exact match
            if (xval - fax[im]).abs() < f32::EPSILON {
                nady.push(fay[im]);
                continue;
            }

            let excess = xval - fax[im];
            let denom = fax[im + 1] - fax[im];
            let fract = if denom != 0.0 { excess / denom } else { 0.0 };

            match effective_type {
                InterpolationType::Linear => {
                    let yval = fay[im] + fract * (fay[im + 1] - fay[im]);
                    nady.push(yval);
                }
                InterpolationType::Quadratic => {
                    let (i1, i2, i3) = if im == 0 {
                        (im, im + 1, im + 2)
                    } else {
                        (im - 1, im, im + 1)
                    };
                    // Guard against out-of-bounds
                    if i3 >= nx {
                        let yval = fay[im] + fract * (fay[im + 1] - fay[im]);
                        nady.push(yval);
                        continue;
                    }
                    let d1 = (fax[i1] - fax[i2]) * (fax[i1] - fax[i3]);
                    let d2 = (fax[i2] - fax[i1]) * (fax[i2] - fax[i3]);
                    let d3 = (fax[i3] - fax[i1]) * (fax[i3] - fax[i2]);
                    if d1 == 0.0 || d2 == 0.0 || d3 == 0.0 {
                        let yval = fay[im] + fract * (fay[im + 1] - fay[im]);
                        nady.push(yval);
                        continue;
                    }
                    let yval = fay[i1] * (xval - fax[i2]) * (xval - fax[i3]) / d1
                        + fay[i2] * (xval - fax[i1]) * (xval - fax[i3]) / d2
                        + fay[i3] * (xval - fax[i1]) * (xval - fax[i2]) / d3;
                    nady.push(yval);
                }
            }
        }
        Ok(nady)
    }

    /// Find the maximum value and its location using quadratic interpolation.
    ///
    /// Returns `(max_val, max_loc)`. If `naloc` is `None`, `max_loc` is an
    /// interpolated index; if `naloc` is provided, `max_loc` is the
    /// corresponding x-value.
    ///
    /// C equivalent: `numaFitMax()` in `numafunc1.c`
    pub fn fit_max(&self, naloc: Option<&Numa>) -> Result<(f32, f32)> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if naloc.is_some_and(|loc| loc.len() != n) {
            return Err(Error::InvalidParameter(
                "na and naloc must have same size".to_string(),
            ));
        }

        let (smaxval, imaxloc) = self.max().ok_or(Error::NullInput("empty Numa"))?;

        // Max at endpoint: no interpolation possible
        if imaxloc == 0 || imaxloc == n - 1 {
            let maxloc = if let Some(loc) = naloc {
                loc.get(imaxloc).unwrap()
            } else {
                imaxloc as f32
            };
            return Ok((smaxval, maxloc));
        }

        // Interior point: quadratic interpolation
        let y2 = smaxval;
        let y1 = self.get(imaxloc - 1).unwrap();
        let y3 = self.get(imaxloc + 1).unwrap();

        let (x1, x2, x3) = if let Some(loc) = naloc {
            (
                loc.get(imaxloc - 1).unwrap(),
                loc.get(imaxloc).unwrap(),
                loc.get(imaxloc + 1).unwrap(),
            )
        } else {
            ((imaxloc - 1) as f32, imaxloc as f32, (imaxloc + 1) as f32)
        };

        if x1 == x2 || x1 == x3 || x2 == x3 {
            return Ok((y2, x2));
        }

        let c1 = y1 / ((x1 - x2) * (x1 - x3));
        let c2 = y2 / ((x2 - x1) * (x2 - x3));
        let c3 = y3 / ((x3 - x1) * (x3 - x2));
        let a = c1 + c2 + c3;
        if a == 0.0 {
            return Ok((y2, x2));
        }
        let b = c1 * (x2 + x3) + c2 * (x1 + x3) + c3 * (x1 + x2);
        let xmax = b / (2.0 * a);
        let ymax = c1 * (xmax - x2) * (xmax - x3)
            + c2 * (xmax - x1) * (xmax - x3)
            + c3 * (xmax - x1) * (xmax - x2);
        Ok((ymax, xmax))
    }

    /// Differentiate over an interval using linear interpolation.
    ///
    /// `self` is the x-value array; `nay` is the corresponding y-value array.
    /// Returns the derivative values at `npts` equally-spaced points from
    /// `x0` to `x1`.
    ///
    /// C equivalent: `numaDifferentiateInterval()` in `numafunc1.c`
    pub fn differentiate_interval(
        &self,
        nay: &Numa,
        x0: f32,
        x1: f32,
        npts: usize,
    ) -> Result<Numa> {
        if x0 > x1 {
            return Err(Error::InvalidParameter("x0 > x1".to_string()));
        }
        if npts < 2 {
            return Err(Error::InvalidParameter("npts must be >= 2".to_string()));
        }
        let naiy = self.interpolate_arbx_interval(nay, InterpolationType::Linear, x0, x1, npts)?;
        let fay = naiy.as_slice();
        let invdel = 0.5 * (npts as f32 - 1.0) / (x1 - x0);
        let mut nady = Numa::new();
        // Endpoint (left)
        nady.push(0.5 * invdel * (fay[1] - fay[0]));
        // Interior points
        for i in 1..npts - 1 {
            nady.push(invdel * (fay[i + 1] - fay[i - 1]));
        }
        // Endpoint (right)
        nady.push(0.5 * invdel * (fay[npts - 1] - fay[npts - 2]));
        Ok(nady)
    }

    /// Integrate over an interval using the trapezoidal rule.
    ///
    /// `self` is the x-value array; `nay` is the corresponding y-value array.
    /// Uses linear interpolation to sample `npts` points and applies the
    /// trapezoidal rule.
    ///
    /// C equivalent: `numaIntegrateInterval()` in `numafunc1.c`
    pub fn integrate_interval(&self, nay: &Numa, x0: f32, x1: f32, npts: usize) -> Result<f32> {
        if x0 > x1 {
            return Err(Error::InvalidParameter("x0 > x1".to_string()));
        }
        if npts < 2 {
            return Err(Error::InvalidParameter("npts must be >= 2".to_string()));
        }
        let naiy = self.interpolate_arbx_interval(nay, InterpolationType::Linear, x0, x1, npts)?;
        let fay = naiy.as_slice();
        let del = (x1 - x0) / (npts as f32 - 1.0);
        let sum = 0.5 * (fay[0] + fay[npts - 1]) + fay[1..npts - 1].iter().sum::<f32>();
        Ok(del * sum)
    }

    /// Resample this Numa to `nsamp` uniformly spaced samples.
    ///
    /// C equivalent: `numaUniformSampling()` in `numafunc1.c`
    pub fn uniform_sampling(&self, nsamp: usize) -> Result<Numa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if nsamp == 0 {
            return Err(Error::InvalidParameter("nsamp must be > 0".to_string()));
        }
        let array = self.as_slice();
        let (startx, delx) = self.parameters();
        let binsize = n as f32 / nsamp as f32;
        let mut nad = Numa::new();
        nad.set_parameters(startx, binsize * delx);

        let mut left = 0.0f32;
        for _ in 0..nsamp {
            let mut sum = 0.0f32;
            let right = left + binsize;
            let ileft = left as usize;
            let mut lfract = 1.0 - left + ileft as f32;
            if lfract >= 1.0 {
                lfract = 0.0;
            }
            let iright_raw = right as usize;
            let rfract = right - iright_raw as f32;
            let iright = iright_raw.min(n - 1);

            if ileft == iright {
                sum += (lfract + rfract - 1.0) * array[ileft];
            } else {
                if lfract > 0.0001 {
                    sum += lfract * array[ileft];
                }
                if rfract > 0.0001 {
                    sum += rfract * array[iright];
                }
                for &v in &array[(ileft + 1)..iright] {
                    sum += v;
                }
            }
            nad.push(sum);
            left = right;
        }
        Ok(nad)
    }

    /// Find intervals where values are below a threshold fraction of the max.
    ///
    /// Returns a Numa where the first element is the max value, followed by
    /// pairs `(x_start, x_end)` for each below-threshold interval.
    ///
    /// `maxn`: if 0.0 (or `None`), the maximum of `self` is used.
    ///
    /// C equivalent: `numaLowPassIntervals()` in `numafunc1.c`
    pub fn low_pass_intervals(&self, thresh: f32, maxn: Option<f32>) -> Result<Numa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if !(0.0..=1.0).contains(&thresh) {
            return Err(Error::InvalidParameter(
                "thresh must be in [0.0, 1.0]".to_string(),
            ));
        }
        let maxval = match maxn {
            Some(v) if v != 0.0 => v,
            _ => self.max_value().unwrap(),
        };
        let (startx, delx) = self.parameters();
        let threshval = thresh * maxval;
        let mut nad = Numa::new();
        nad.push(maxval);

        let mut inrun = false;
        let mut x0 = 0.0f32;
        for i in 0..n {
            let fval = self.get(i).unwrap();
            let x = startx + i as f32 * delx;
            if fval < threshval && !inrun {
                inrun = true;
                x0 = x;
            } else if fval > threshval && inrun {
                inrun = false;
                nad.push(x0);
                nad.push(x);
            }
        }
        if inrun {
            let x1 = startx + (n - 1) as f32 * delx;
            nad.push(x0);
            nad.push(x1);
        }
        Ok(nad)
    }

    /// Find edge intervals where values transition through a threshold band.
    ///
    /// Returns a Numa where the first element is the max value, followed by
    /// triplets `(x_start, x_end, sign)` for each edge, where sign is +1
    /// (rising) or -1 (falling).
    ///
    /// C equivalent: `numaThresholdEdges()` in `numafunc1.c`
    pub fn threshold_edges(&self, thresh1: f32, thresh2: f32, maxn: Option<f32>) -> Result<Numa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if !(0.0..=1.0).contains(&thresh1) || !(0.0..=1.0).contains(&thresh2) {
            return Err(Error::InvalidParameter(
                "thresholds must be in [0.0, 1.0]".to_string(),
            ));
        }
        if thresh2 < thresh1 {
            return Err(Error::InvalidParameter("thresh2 < thresh1".to_string()));
        }
        let maxval = match maxn {
            Some(v) if v != 0.0 => v,
            _ => self.max_value().unwrap(),
        };
        let (startx, delx) = self.parameters();
        let tv1 = thresh1 * maxval;
        let tv2 = thresh2 * maxval;
        let mut nad = Numa::new();
        nad.push(maxval);

        // Find istart: first index where value is outside the band
        let mut istart = n;
        let mut belowlast = false;
        let mut abovelast = false;
        for i in 0..n {
            let fval = self.get(i).unwrap();
            belowlast = fval < tv1;
            abovelast = fval > tv2;
            if belowlast || abovelast {
                istart = i;
                break;
            }
        }
        if istart == n {
            return Ok(nad);
        }

        let mut inband = false;
        let mut startbelow = belowlast;
        let mut output = false;
        let mut sign = 0i32;
        let mut x0 = startx + istart as f32 * delx;
        let mut x1 = x0;
        let mut out_x0 = x0;
        let mut out_x1 = x0;

        for i in (istart + 1)..n {
            let fval = self.get(i).unwrap();
            let x = startx + i as f32 * delx;
            let below = fval < tv1;
            let above = fval > tv2;

            if !inband && belowlast && above {
                x1 = x;
                sign = 1;
                startbelow = false;
                output = true;
            } else if !inband && abovelast && below {
                x1 = x;
                sign = -1;
                startbelow = true;
                output = true;
            } else if inband && startbelow && above {
                x1 = x;
                sign = 1;
                inband = false;
                startbelow = false;
                output = true;
            } else if inband && !startbelow && below {
                x1 = x;
                sign = -1;
                inband = false;
                startbelow = true;
                output = true;
            } else if inband && ((!startbelow && above) || (startbelow && below)) {
                // exit without crossing: reset x0
                x0 = x;
                inband = false;
            } else if !inband && !above && !below {
                inband = true;
                startbelow = belowlast;
            } else if !inband && (above || below) {
                x0 = x;
            }

            belowlast = below;
            abovelast = above;

            if output {
                out_x0 = x0;
                out_x1 = x1;
                nad.push(out_x0);
                nad.push(out_x1);
                nad.push(sign as f32);
                output = false;
                x0 = x;
            }
        }
        let _ = (out_x0, out_x1);
        Ok(nad)
    }

    /// Get the start and end x-values for a span from `low_pass_intervals` output.
    ///
    /// `span` is zero-based. Returns `(x_start, x_end)`.
    ///
    /// C equivalent: `numaGetSpanValues()` in `numafunc1.c`
    pub fn get_span_values(&self, span: usize) -> Result<(f32, f32)> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if n % 2 != 1 {
            return Err(Error::InvalidParameter(
                "n is not odd (invalid low_pass_intervals output)".to_string(),
            ));
        }
        let nspans = n / 2;
        if span >= nspans {
            return Err(Error::InvalidParameter(
                "span index out of range".to_string(),
            ));
        }
        let start = self.get(2 * span + 1).unwrap();
        let end = self.get(2 * span + 2).unwrap();
        Ok((start, end))
    }

    /// Get the start, end, and sign for an edge from `threshold_edges` output.
    ///
    /// `edge` is zero-based. Returns `(x_start, x_end, sign)` where sign is
    /// +1 (rising) or -1 (falling).
    ///
    /// C equivalent: `numaGetEdgeValues()` in `numafunc1.c`
    pub fn get_edge_values(&self, edge: usize) -> Result<(f32, f32, i32)> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if n % 3 != 1 {
            return Err(Error::InvalidParameter(
                "n % 3 != 1 (invalid threshold_edges output)".to_string(),
            ));
        }
        let nedges = (n - 1) / 3;
        if edge >= nedges {
            return Err(Error::InvalidParameter(
                "edge index out of range".to_string(),
            ));
        }
        let start = self.get(3 * edge + 1).unwrap();
        let end = self.get(3 * edge + 2).unwrap();
        let sign = self.get(3 * edge + 3).unwrap() as i32;
        Ok((start, end, sign))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- interpolate_eqx_interval --

    #[test]
    fn test_interpolate_eqx_interval_linear() {
        // y = x: values [0, 1, 2, 3, 4] with startx=0, delx=1
        let mut nasy = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        nasy.set_parameters(0.0, 1.0);
        let nay = nasy
            .interpolate_eqx_interval(InterpolationType::Linear, 0.0, 4.0, 5)
            .unwrap();
        assert_eq!(nay.len(), 5);
        for i in 0..5 {
            let yval = nay.get(i).unwrap();
            assert!((yval - i as f32).abs() < 1e-4, "at i={i}: got {yval}");
        }
    }

    #[test]
    fn test_interpolate_eqx_interval_out_of_range_error() {
        let mut nasy = Numa::from_slice(&[0.0, 1.0, 2.0]);
        nasy.set_parameters(0.0, 1.0);
        // x0=0, x1=5 exceeds range [0, 2]
        assert!(
            nasy.interpolate_eqx_interval(InterpolationType::Linear, 0.0, 5.0, 3)
                .is_err()
        );
    }

    // -- interpolate_arbx_interval --

    #[test]
    fn test_interpolate_arbx_interval_linear() {
        // y = 2x: nax=[0,1,2,3], nay=[0,2,4,6]
        let nax = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0]);
        let nay = Numa::from_slice(&[0.0, 2.0, 4.0, 6.0]);
        let result = nax
            .interpolate_arbx_interval(&nay, InterpolationType::Linear, 0.0, 3.0, 4)
            .unwrap();
        assert_eq!(result.len(), 4);
        // at x=0,1,2,3 expect y=0,2,4,6
        for i in 0..4 {
            let expected = 2.0 * i as f32;
            let got = result.get(i).unwrap();
            assert!(
                (got - expected).abs() < 0.01,
                "at i={i}: got {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn test_interpolate_arbx_interval_midpoint() {
        // y = x: nax=[0,2,4], nay=[0,2,4], sample at midpoint x=1
        let nax = Numa::from_slice(&[0.0, 2.0, 4.0]);
        let nay = Numa::from_slice(&[0.0, 2.0, 4.0]);
        let result = nax
            .interpolate_arbx_interval(&nay, InterpolationType::Linear, 0.0, 4.0, 3)
            .unwrap();
        // expect y at [0, 2, 4]
        assert!((result.get(0).unwrap()).abs() < 0.01);
        assert!((result.get(1).unwrap() - 2.0).abs() < 0.01);
        assert!((result.get(2).unwrap() - 4.0).abs() < 0.01);
    }

    // -- fit_max --

    #[test]
    fn test_fit_max_interior() {
        // Parabola peaking at index 2: y = -(x-2)^2 + 4
        // Values at x=0,1,2,3,4: 0, 3, 4, 3, 0
        let na = Numa::from_slice(&[0.0, 3.0, 4.0, 3.0, 0.0]);
        let (maxval, maxloc) = na.fit_max(None).unwrap();
        assert!(
            (maxval - 4.0).abs() < 0.1,
            "expected maxval≈4.0, got {maxval}"
        );
        assert!(
            (maxloc - 2.0).abs() < 0.1,
            "expected maxloc≈2.0, got {maxloc}"
        );
    }

    #[test]
    fn test_fit_max_endpoint() {
        let na = Numa::from_slice(&[5.0, 3.0, 1.0]);
        let (maxval, maxloc) = na.fit_max(None).unwrap();
        assert_eq!(maxval, 5.0);
        assert_eq!(maxloc as usize, 0);
    }

    #[test]
    fn test_fit_max_with_naloc() {
        // Peak at naloc[2]=10.0
        let na = Numa::from_slice(&[1.0, 3.0, 4.0, 3.0, 1.0]);
        let naloc = Numa::from_slice(&[6.0, 8.0, 10.0, 12.0, 14.0]);
        let (maxval, maxloc) = na.fit_max(Some(&naloc)).unwrap();
        assert!(
            (maxval - 4.0).abs() < 0.5,
            "expected maxval≈4.0, got {maxval}"
        );
        assert!(
            (maxloc - 10.0).abs() < 0.5,
            "expected maxloc≈10.0, got {maxloc}"
        );
    }

    // -- differentiate_interval --

    #[test]
    fn test_differentiate_interval_linear() {
        // y = x: interior derivative should be 1.0
        // (endpoints use a one-sided formula, so they may differ)
        let nax = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        let nay = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        let nady = nax.differentiate_interval(&nay, 0.0, 4.0, 5).unwrap();
        assert_eq!(nady.len(), 5);
        // Check interior points (indices 1..3)
        for i in 1..4 {
            let der = nady.get(i).unwrap();
            assert!((der - 1.0).abs() < 0.1, "at i={i}: derivative={der}");
        }
    }

    // -- integrate_interval --

    #[test]
    fn test_integrate_interval_constant() {
        // y = 2: integral over [0, 4] = 8
        let nax = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        let nay = Numa::from_slice(&[2.0, 2.0, 2.0, 2.0, 2.0]);
        let result = nax.integrate_interval(&nay, 0.0, 4.0, 5).unwrap();
        assert!((result - 8.0).abs() < 0.01, "expected 8.0, got {result}");
    }

    #[test]
    fn test_integrate_interval_linear() {
        // y = x: integral over [0, 2] = 2
        let nax = Numa::from_slice(&[0.0, 1.0, 2.0]);
        let nay = Numa::from_slice(&[0.0, 1.0, 2.0]);
        let result = nax.integrate_interval(&nay, 0.0, 2.0, 3).unwrap();
        assert!((result - 2.0).abs() < 0.1, "expected 2.0, got {result}");
    }

    // -- uniform_sampling --

    #[test]
    fn test_uniform_sampling_downsample() {
        // Downsample [1,2,3,4,5,6,7,8] to 4 samples
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let result = na.uniform_sampling(4).unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_uniform_sampling_identity() {
        // Same number of samples: length must be preserved
        let na = Numa::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        let result = na.uniform_sampling(4).unwrap();
        assert_eq!(result.len(), 4);
    }

    // -- low_pass_intervals --

    #[test]
    fn test_low_pass_intervals_basic() {
        // Values [10, 3, 3, 10, 3, 10], thresh=0.5 → intervals where val < 5
        let na = Numa::from_slice(&[10.0, 3.0, 3.0, 10.0, 3.0, 10.0]);
        let result = na.low_pass_intervals(0.5, None).unwrap();
        // First element is maxval=10
        assert!((result.get(0).unwrap() - 10.0).abs() < 1e-5);
        // Should have 2 spans: indices 1-2 and 4
        let n = result.len();
        assert!(n >= 5, "expected at least 5 elements, got {n}");
    }

    #[test]
    fn test_low_pass_intervals_no_intervals() {
        // All values above threshold
        let na = Numa::from_slice(&[10.0, 10.0, 10.0]);
        let result = na.low_pass_intervals(0.5, None).unwrap();
        // Only maxval, no spans
        assert_eq!(result.len(), 1);
    }

    // -- threshold_edges --

    #[test]
    fn test_threshold_edges_rising() {
        // Values: low (0), then high (10): one rising edge
        let na = Numa::from_slice(&[0.0, 0.0, 10.0, 10.0]);
        let result = na.threshold_edges(0.2, 0.8, None).unwrap();
        // First element is maxval
        assert!((result.get(0).unwrap() - 10.0).abs() < 1e-4);
        // Should find 1 edge
        let n = result.len();
        // n = 1 + 3*nedges; nedges should be 1 → n=4
        assert_eq!(n, 4, "expected 4 elements, got {n}");
        // sign = +1 (rising)
        let sign = result.get(3).unwrap() as i32;
        assert_eq!(sign, 1);
    }

    // -- get_span_values / get_edge_values --

    #[test]
    fn test_get_span_values_basic() {
        // Manually construct low_pass_intervals output
        // [maxval, x0_span0, x1_span0, x0_span1, x1_span1]
        let na = Numa::from_slice(&[10.0, 1.0, 3.0, 5.0, 7.0]);
        let (s, e) = na.get_span_values(0).unwrap();
        assert_eq!(s, 1.0);
        assert_eq!(e, 3.0);
        let (s2, e2) = na.get_span_values(1).unwrap();
        assert_eq!(s2, 5.0);
        assert_eq!(e2, 7.0);
    }

    #[test]
    fn test_get_edge_values_basic() {
        // Manually construct threshold_edges output
        // [maxval, x0_edge0, x1_edge0, sign0]
        let na = Numa::from_slice(&[10.0, 1.0, 3.0, 1.0]);
        let (s, e, sign) = na.get_edge_values(0).unwrap();
        assert_eq!(s, 1.0);
        assert_eq!(e, 3.0);
        assert_eq!(sign, 1);
    }

    #[test]
    fn test_get_span_values_out_of_range() {
        let na = Numa::from_slice(&[10.0, 1.0, 3.0]);
        assert!(na.get_span_values(1).is_err());
    }
}
