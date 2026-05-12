//! Advanced Numa helpers (plan 109 / C numafunc2.c).
//!
//! Covered functions:
//!
//! - `numaCountReversals` -> [`Numa::count_reversals`]
//! - `numaCrossingsByThreshold` -> [`numa_crossings_by_threshold`]
//! - `numaFindPeaks` -> [`Numa::find_peaks`]
//! - `numaGetUniformBinSizes` -> [`numa_uniform_bin_sizes`]
//! - `genConstrainedNumaInRange` -> [`gen_constrained_numa_in_range`]
//!
//! `numaHistogramGetRankFromVal` / `numaHistogramGetValFromRank` are
//! already covered by existing [`Numa::histogram_rank_from_val`] /
//! [`Numa::histogram_val_from_rank`] in `numa::histogram`.

use crate::core::error::{Error, Result};
use crate::core::numa::Numa;

impl Numa {
    /// Count the number of significant "reversals" (sign / level changes
    /// large enough to exceed `min_reversal`).
    ///
    /// Returns `(reversal_count, reversal_density)` where density is
    /// `count / (delx * n)` matching C's `prd` output.
    ///
    /// Binary inputs (`0`/`1`) are special-cased: every 0↔1 transition
    /// is counted regardless of `min_reversal` (matching C's
    /// `binvals` fast path).
    ///
    /// C Leptonica equivalent: `numaCountReversals`.
    pub fn count_reversals(&self, min_reversal: f32) -> Result<(u32, f32)> {
        if min_reversal < 0.0 {
            return Err(Error::InvalidParameter("min_reversal < 0".into()));
        }
        let n = self.len();
        if n == 0 {
            return Ok((0, 0.0));
        }

        // Detect binary 0/1 input.
        let mut bin_vals = true;
        for i in 0..n {
            let v = self.get(i).unwrap_or(0.0);
            if v != 0.0 && v != 1.0 {
                bin_vals = false;
                break;
            }
        }

        let nr: u32 = if bin_vals && min_reversal <= 1.0 {
            let mut count = 0u32;
            let mut prev = self.get(0).unwrap_or(0.0) as i32;
            for i in 1..n {
                let cur = self.get(i).unwrap_or(0.0) as i32;
                if cur != prev {
                    count += 1;
                    prev = cur;
                }
            }
            count
        } else {
            // Use find_extrema with delta=min_reversal.
            let extrema = self.find_extrema(min_reversal)?;
            extrema.len() as u32
        };

        let (_start, delx) = self.parameters();
        // Density follows C's formula `nr / (delx * n)`. Guard against
        // `delx == 0` (and the rare NaN/inf case) so we never divide by
        // zero; valid sub-unit sampling rates (delx < 1.0) are preserved
        // as-is.
        let len = delx * (n as f32);
        let rd = if len.is_finite() && len != 0.0 {
            nr as f32 / len
        } else {
            0.0
        };
        Ok((nr, rd))
    }

    /// Find up to `nmax` peaks in this Numa.
    ///
    /// Returns a Numa with `4 * actual_peaks` floats laid out as
    /// `(left_loc, max_loc, right_loc, peak_fraction)` per peak. After
    /// each peak is extracted the affected range is zeroed before
    /// looking for the next.
    ///
    /// The sweep extends through every neighbour that satisfies **either**
    /// of these conditions (matching C `numaFindPeaks`):
    ///
    /// 1. `val > fract1 * fmax_val` — value is still close to the peak
    /// 2. `lastval - val > fract2 * lastval` — value drops sharply from
    ///    the previous one, which is treated as a "transition through"
    ///    rather than the end of the peak region
    ///
    /// The sweep stops on the first neighbour that fails both conditions,
    /// or on a zero value.
    ///
    /// C Leptonica equivalent: `numaFindPeaks`.
    pub fn find_peaks(&self, nmax: u32, fract1: f32, fract2: f32) -> Numa {
        let n = self.len();
        let total: f32 = self.sum().unwrap_or(0.0);
        let mut work = self.clone();
        let mut napeak = Numa::with_capacity((4 * nmax) as usize);
        if n == 0 || total == 0.0 {
            return napeak;
        }
        for _ in 0..nmax {
            let new_total: f32 = work.sum().unwrap_or(0.0);
            if new_total == 0.0 {
                break;
            }
            let (fmax_val, max_loc) = match work.max() {
                Some(m) => m,
                None => break,
            };
            let max_loc = max_loc as i32;

            // Sweep left.
            let mut sum = fmax_val;
            let mut lastval = fmax_val;
            let mut lloc = 0i32;
            let mut i = max_loc - 1;
            while i >= 0 {
                let val = work.get(i as usize).unwrap_or(0.0);
                if val == 0.0 {
                    lloc = i + 1;
                    break;
                }
                // Either the value is high enough OR the drop is small
                // enough; both cases continue the sweep with the same
                // bookkeeping. Otherwise stop.
                if val > fract1 * fmax_val || lastval - val > fract2 * lastval {
                    sum += val;
                    lastval = val;
                } else {
                    lloc = i;
                    break;
                }
                i -= 1;
            }

            // Sweep right.
            lastval = fmax_val;
            let mut rloc = (n - 1) as i32;
            let mut i = max_loc + 1;
            while i < n as i32 {
                let val = work.get(i as usize).unwrap_or(0.0);
                if val == 0.0 {
                    rloc = i - 1;
                    break;
                }
                if val > fract1 * fmax_val || lastval - val > fract2 * lastval {
                    sum += val;
                    lastval = val;
                } else {
                    rloc = i;
                    break;
                }
                i += 1;
            }

            let peak_fract = if total > 0.0 { sum / total } else { 0.0 };
            napeak.push(lloc as f32);
            napeak.push(max_loc as f32);
            napeak.push(rloc as f32);
            napeak.push(peak_fract);

            for k in lloc..=rloc {
                let _ = work.set(k as usize, 0.0);
            }
        }
        napeak
    }
}

/// Find x-positions where a y-series crosses `thresh`, optionally with
/// an explicit x-array.
///
/// The result is a Numa of `x` values where linear interpolation
/// indicates `y == thresh`. When `nax` is `None`, the x positions are
/// derived from `nay.parameters()` (`startx + i*delx`).
///
/// C Leptonica equivalent: `numaCrossingsByThreshold`.
pub fn numa_crossings_by_threshold(nay: &Numa, nax: Option<&Numa>, thresh: f32) -> Result<Numa> {
    let n = nay.len();
    if let Some(x) = nax
        && x.len() != n
    {
        return Err(Error::InvalidParameter(format!(
            "nax length ({}) != nay length ({})",
            x.len(),
            n
        )));
    }
    let mut out = Numa::new();
    if n < 2 {
        return Ok(out);
    }
    let (startx, delx) = nay.parameters();
    let mut yval1 = nay.get(0).unwrap_or(0.0);
    let mut xval1 = match nax {
        Some(x) => x.get(0).unwrap_or(startx),
        None => startx,
    };
    for i in 1..n {
        let yval2 = nay.get(i).unwrap_or(0.0);
        let xval2 = match nax {
            Some(x) => x.get(i).unwrap_or(startx + (i as f32) * delx),
            None => startx + (i as f32) * delx,
        };
        let delta1 = yval1 - thresh;
        let delta2 = yval2 - thresh;
        if delta1 == 0.0 {
            out.push(xval1);
        } else if delta2 == 0.0 {
            out.push(xval2);
        } else if delta1 * delta2 < 0.0 {
            let fract = delta1.abs() / (yval1 - yval2).abs();
            let crossval = xval1 + fract * (xval2 - xval1);
            out.push(crossval);
        }
        xval1 = xval2;
        yval1 = yval2;
    }
    Ok(out)
}

/// Divide `ntotal` items evenly into `nbins` bins, returning the size
/// of each bin.
///
/// When `ntotal < nbins`, the first `ntotal` bins get size `1` and the
/// rest are absent (matching C's special path).
///
/// C Leptonica equivalent: `numaGetUniformBinSizes`.
pub fn numa_uniform_bin_sizes(ntotal: i32, nbins: i32) -> Result<Numa> {
    if ntotal <= 0 {
        return Err(Error::InvalidParameter("ntotal <= 0".into()));
    }
    if nbins <= 0 {
        return Err(Error::InvalidParameter("nbins <= 0".into()));
    }
    let mut na = Numa::with_capacity(nbins as usize);
    if ntotal < nbins {
        for _ in 0..ntotal {
            na.push(1.0);
        }
        return Ok(na);
    }
    let mut start = 0i64;
    for i in 0..nbins {
        let end = (ntotal as i64) * ((i + 1) as i64) / (nbins as i64);
        na.push((end - start) as f32);
        start = end;
    }
    Ok(na)
}

/// Generate up to `nmax` integer values evenly spread across the
/// inclusive range `[first, last]`.
///
/// When `use_pairs` is `true`, each selected value is paired with its
/// successor (so the result has `2 * nsets` entries).
///
/// C Leptonica equivalent: `genConstrainedNumaInRange`.
pub fn gen_constrained_numa_in_range(
    first: i32,
    last: i32,
    nmax: i32,
    use_pairs: bool,
) -> Result<Numa> {
    let first = first.max(0);
    if last < first {
        return Err(Error::InvalidParameter("last < first".into()));
    }
    if nmax < 1 {
        return Err(Error::InvalidParameter("nmax < 1".into()));
    }
    let mut nsets = nmax.min(last - first + 1);
    if use_pairs {
        nsets /= 2;
    }
    if nsets == 0 {
        return Err(Error::InvalidParameter("nsets == 0".into()));
    }
    let delta = if nsets == 1 {
        0.0
    } else if !use_pairs {
        (last - first) as f32 / (nsets - 1) as f32
    } else {
        (last - first - 1) as f32 / (nsets - 1) as f32
    };
    let mut na = Numa::with_capacity(if use_pairs {
        (2 * nsets) as usize
    } else {
        nsets as usize
    });
    for i in 0..nsets {
        let val = (first as f32 + (i as f32) * delta + 0.5) as i32;
        na.push(val as f32);
        if use_pairs {
            na.push((val + 1) as f32);
        }
    }
    Ok(na)
}
