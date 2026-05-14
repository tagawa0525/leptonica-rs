//! Advanced Numa helpers (plan 109 / plan 119 / C numafunc2.c).
//!
//! Covered functions:
//!
//! - `numaCountReversals` -> [`Numa::count_reversals`]
//! - `numaCrossingsByThreshold` -> [`numa_crossings_by_threshold`]
//! - `numaFindPeaks` -> [`Numa::find_peaks`]
//! - `numaGetUniformBinSizes` -> [`numa_uniform_bin_sizes`]
//! - `genConstrainedNumaInRange` -> [`gen_constrained_numa_in_range`]
//! - `numaRebinHistogram` -> [`numa_rebin_histogram`]
//! - `numaMakeRankFromHistogram` -> [`make_rank_from_histogram`]
//!
//! Already covered by other modules:
//!
//! - `numaHistogramGetRankFromVal` / `numaHistogramGetValFromRank` ->
//!   [`Numa::histogram_rank_from_val`] / [`Numa::histogram_val_from_rank`]
//!   (`numa::histogram`)
//! - `numaGetHistogramStats` / `numaGetHistogramStatsOnInterval` ->
//!   [`Numa::histogram_stats`] / [`Numa::histogram_stats_on_interval`]
//!   (`numa::histogram`)
//! - `numaGetStatsUsingHistogram` ->
//!   [`Numa::stats_using_histogram`] (`numa::operations`)

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

/// Rebin a histogram by accumulating every `new_size` consecutive bins
/// into one bin of the output.
///
/// The output has `ceil(ns / new_size)` bins; trailing partial groups
/// accumulate whatever remains. The bin width parameter is scaled by
/// `new_size` so the output's `(startx, deltax)` reflects the coarser
/// resolution.
///
/// C Leptonica equivalent: `numaRebinHistogram`.
pub fn numa_rebin_histogram(nas: &Numa, new_size: usize) -> Result<Numa> {
    if new_size <= 1 {
        return Err(Error::InvalidParameter("newsize must be > 1".into()));
    }
    let ns = nas.len();
    if ns == 0 {
        return Err(Error::InvalidParameter("nas is empty".into()));
    }
    let nd = ns.div_ceil(new_size);
    let mut nad = Numa::with_capacity(nd);
    let (start, oldsize) = nas.parameters();
    nad.set_parameters(start, oldsize * (new_size as f32));
    for i in 0..nd {
        let base = i * new_size;
        let mut count = 0.0_f32;
        for j in 0..new_size {
            let idx = base + j;
            if idx < ns {
                count += nas.get(idx).unwrap_or(0.0);
            }
        }
        nad.push(count);
    }
    Ok(nad)
}

/// Build a rank (cumulative fraction) curve sampled at `npts` evenly
/// spaced x positions covering the original histogram's range.
///
/// Returns `(nax, nay)` where each entry pair `(x, r)` describes the
/// rank `r` at sample point `x`. Internally the histogram is
/// normalised, cumulatively summed, and resampled with linear
/// interpolation.
///
/// C Leptonica equivalent: `numaMakeRankFromHistogram`.
pub fn make_rank_from_histogram(
    startx: f32,
    deltax: f32,
    nasy: &Numa,
    npts: usize,
) -> Result<(Numa, Numa)> {
    let n = nasy.len();
    if n == 0 {
        return Err(Error::InvalidParameter("no bins in nasy".into()));
    }
    if npts < 3 {
        return Err(Error::InvalidParameter("npts must be >= 3".into()));
    }
    if deltax <= 0.0 {
        return Err(Error::InvalidParameter("deltax must be > 0".into()));
    }
    let nan = nasy
        .normalize_histogram()
        .ok_or_else(|| Error::InvalidParameter("nasy has zero sum: cannot normalize".into()))?;

    // Build the cumulative rank Numa (length n+1) and tag it with
    // (startx, deltax) so interpolate_eqx_interval can map index -> x.
    let mut nar = Numa::with_capacity(n + 1);
    nar.set_parameters(startx, deltax);
    let mut sum = 0.0_f32;
    nar.push(sum);
    for i in 0..n {
        sum += nan.get(i).unwrap_or(0.0);
        nar.push(sum);
    }

    let x_end = startx + (n as f32) * deltax;
    let nay = nar.interpolate_eqx_interval(
        crate::core::numa::InterpolationType::Linear,
        startx,
        x_end,
        npts,
    )?;
    let dx = (x_end - startx) / ((npts - 1) as f32);
    let mut nax = Numa::with_capacity(npts);
    for i in 0..npts {
        nax.push(startx + (i as f32) * dx);
    }
    Ok((nax, nay))
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

// ============================================================================
// Plan 130: earth_mover_distance / discretize_sorted_in_bins /
//           discretize_histo_in_bins
// ============================================================================

impl Numa {
    /// 1D Earth-Mover Distance between two equal-length distributions.
    ///
    /// `other` is renormalized so its total matches `self`, then mass is
    /// pushed bin-by-bin to align with `self`. The total work
    /// (sum of |movement|) is divided by `sum(self)` to produce the EMD.
    ///
    /// C Leptonica equivalent: `numaEarthMoverDistance` (`numafunc2.c`).
    pub fn earth_mover_distance(&self, other: &Numa) -> Result<f32> {
        let n = self.len();
        if other.len() != n {
            return Err(Error::InvalidParameter(format!(
                "earth_mover_distance: na1 length {n} != na2 length {}",
                other.len()
            )));
        }
        if n == 0 {
            return Err(Error::InvalidParameter(
                "earth_mover_distance: empty inputs".into(),
            ));
        }
        let sum1 = self.sum().unwrap_or(0.0);
        let sum2 = other.sum().unwrap_or(0.0);
        if sum1 == 0.0 {
            return Ok(0.0);
        }
        if sum2 == 0.0 {
            return Err(Error::InvalidParameter(
                "earth_mover_distance: other has zero total mass".into(),
            ));
        }
        let normalized = (sum1 - sum2).abs() < 1e-5 * sum1.abs();
        let scale = if normalized { 1.0 } else { sum1 / sum2 };
        let mut na3: Vec<f32> = (0..n)
            .map(|i| other.get(i).unwrap_or(0.0) * scale)
            .collect();
        let mut total = 0.0_f32;
        for i in 1..n {
            let diff = self.get(i - 1).unwrap_or(0.0) - na3[i - 1];
            na3[i] -= diff;
            total += diff.abs();
        }
        Ok(total / sum1)
    }

    /// Discretize an already-sorted Numa into `nbins` equal-population
    /// buckets, returning the average value in each bucket.
    ///
    /// C Leptonica equivalent: `numaDiscretizeSortedInBins`.
    pub fn discretize_sorted_in_bins(&self, nbins: u32) -> Result<Numa> {
        if nbins < 2 {
            return Err(Error::InvalidParameter(format!(
                "nbins must be > 1 (got {nbins})"
            )));
        }
        let ntot = self.len();
        if ntot == 0 {
            return Err(Error::InvalidParameter(
                "discretize_sorted_in_bins: input is empty".into(),
            ));
        }
        let ntot_i = i32::try_from(ntot)
            .map_err(|_| Error::InvalidParameter("input length overflows i32".into()))?;
        let nbins_i = i32::try_from(nbins)
            .map_err(|_| Error::InvalidParameter("nbins overflows i32".into()))?;
        let naeach = numa_uniform_bin_sizes(ntot_i, nbins_i)?;
        let mut out = Numa::with_capacity(nbins as usize);
        let mut sum = 0.0_f32;
        let mut bincount: u32 = 0;
        let mut binindex: usize = 0;
        let mut binsize = naeach.get(0).unwrap_or(0.0) as u32;
        for i in 0..ntot {
            sum += self.get(i).unwrap_or(0.0);
            bincount += 1;
            if bincount == binsize {
                out.push(sum / binsize as f32);
                sum = 0.0;
                bincount = 0;
                binindex += 1;
                if binindex == nbins as usize {
                    break;
                }
                binsize = naeach.get(binindex).unwrap_or(0.0) as u32;
            }
        }
        Ok(out)
    }

    /// Discretize a histogram Numa (count per index) into `nbins` equal-
    /// population buckets. Returns `(average index per bucket, optional
    /// cumulative normalized rank)`.
    ///
    /// C Leptonica equivalent: `numaDiscretizeHistoInBins`.
    pub fn discretize_histo_in_bins(
        &self,
        nbins: u32,
        want_rank: bool,
    ) -> Result<(Numa, Option<Numa>)> {
        if nbins < 2 {
            return Err(Error::InvalidParameter(format!(
                "nbins must be > 1 (got {nbins})"
            )));
        }
        let nxvals = self.len();
        if nxvals == 0 {
            return Err(Error::InvalidParameter(
                "discretize_histo_in_bins: input is empty".into(),
            ));
        }
        let ntot = self.sum().unwrap_or(0.0) as i32;
        if ntot <= 0 {
            return Err(Error::InvalidParameter(
                "discretize_histo_in_bins: histogram total is zero".into(),
            ));
        }
        let nbins_i = i32::try_from(nbins)
            .map_err(|_| Error::InvalidParameter("nbins overflows i32".into()))?;
        let naeach = numa_uniform_bin_sizes(ntot, nbins_i)?;

        let mut binval = Numa::with_capacity(nbins as usize);
        let mut sum = 0.0_f32;
        let mut bincount: u32 = 0;
        let mut binindex: usize = 0;
        let mut binsize = naeach.get(0).unwrap_or(0.0) as u32;
        'outer: for i in 0..nxvals {
            let count = self.get(i).unwrap_or(0.0) as u32;
            for _ in 0..count {
                bincount += 1;
                sum += i as f32;
                if bincount == binsize {
                    binval.push(sum / binsize as f32);
                    sum = 0.0;
                    bincount = 0;
                    binindex += 1;
                    if binindex == nbins as usize {
                        break 'outer;
                    }
                    binsize = naeach.get(binindex).unwrap_or(0.0) as u32;
                }
            }
        }

        let rank = if want_rank {
            // Cumulative normalized histogram: rank[i] = Σ_{k<=i} na[k] / ntot.
            let mut rank = Numa::with_capacity(nxvals);
            let mut cum = 0.0_f32;
            let total = ntot as f32;
            for i in 0..nxvals {
                cum += self.get(i).unwrap_or(0.0);
                rank.push(cum / total);
            }
            Some(rank)
        } else {
            None
        };

        Ok((binval, rank))
    }
}
