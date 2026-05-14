//! Advanced Numa helpers (plan 109 / plan 119 / plans 130-133 /
//! C numafunc2.c).
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
//! - `numaEarthMoverDistance` -> [`Numa::earth_mover_distance`]
//! - `numaDiscretizeSortedInBins` -> [`Numa::discretize_sorted_in_bins`]
//! - `numaDiscretizeHistoInBins` -> [`Numa::discretize_histo_in_bins`]
//! - `numaGetRankBinValues` -> [`Numa::get_rank_bin_values`]
//! - `numaMakeHistogramAuto` -> [`Numa::make_histogram_auto`]
//! - `numaSplitDistribution` -> [`Numa::split_distribution`]
//!   (returns [`SplitDistribution`])
//! - `numaCrossingsByPeaks` -> [`Numa::crossings_by_peaks`]
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

/// Result of [`Numa::split_distribution`].
#[derive(Debug, Clone)]
pub struct SplitDistribution {
    /// Threshold-style split index (best_split + 1, capped at 255).
    pub split_index: i32,
    /// Average of the lower partition at the chosen split.
    pub ave1: f32,
    /// Average of the upper partition at the chosen split.
    pub ave2: f32,
    /// Count in the lower partition at the chosen split.
    pub num1: f32,
    /// Count in the upper partition at the chosen split.
    pub num2: f32,
    /// Per-bin Otsu-like score array (only when `want_score = true`).
    pub score: Option<Numa>,
}

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

    /// Find threshold-crossings located between successive peaks/troughs
    /// of a 1D signal.
    ///
    /// First locates the extrema of `self` (via [`Numa::find_extrema`]
    /// with the given `delta`) and appends the last index. Between each
    /// pair of consecutive extrema, walks the signal to find the point
    /// where it crosses `(prev_extremum_val + cur_extremum_val) / 2` and
    /// records that location in the result. Locations are expressed in
    /// `nax` units when provided, otherwise in `startx + i * delx`.
    ///
    /// C Leptonica equivalent: `numaCrossingsByPeaks` (`numafunc2.c`).
    pub fn crossings_by_peaks(&self, nax: Option<&Numa>, delta: f32) -> Result<Numa> {
        let n = self.len();
        if n < 2 {
            return Ok(Numa::new());
        }
        if let Some(x) = nax
            && x.len() != n
        {
            return Err(Error::InvalidParameter(format!(
                "nax length ({}) != nay length ({n})",
                x.len()
            )));
        }
        // Find the extrema, then append n-1 to capture the last transition.
        let mut nap = self.find_extrema(delta)?;
        nap.push((n - 1) as f32);
        let np = nap.len();

        let (startx, delx) = self.parameters();
        let mut nad = Numa::with_capacity(np);
        let mut prev_index: usize = 0;
        let mut prev_val = self.get(0).unwrap_or(0.0);
        for i in 0..np {
            let cur_index = nap.get(i).unwrap_or(0.0) as usize;
            let cur_val = self.get(cur_index).unwrap_or(0.0);
            let thresh = (prev_val + cur_val) / 2.0;
            let xval1_initial = match nax {
                Some(x) => x.get(prev_index).unwrap_or(0.0),
                None => startx + prev_index as f32 * delx,
            };
            let yval1_initial = self.get(prev_index).unwrap_or(0.0);
            let mut xval1 = xval1_initial;
            let mut yval1 = yval1_initial;
            for j in (prev_index + 1)..=cur_index {
                let xval2 = match nax {
                    Some(x) => x.get(j).unwrap_or(0.0),
                    None => startx + j as f32 * delx,
                };
                let yval2 = self.get(j).unwrap_or(0.0);
                let delta1 = yval1 - thresh;
                let delta2 = yval2 - thresh;
                if delta1 == 0.0 {
                    nad.push(xval1);
                    break;
                } else if delta2 == 0.0 {
                    nad.push(xval2);
                    break;
                } else if delta1 * delta2 < 0.0 {
                    let fract = delta1.abs() / (yval1 - yval2).abs();
                    let crossval = xval1 + fract * (xval2 - xval1);
                    nad.push(crossval);
                    break;
                }
                xval1 = xval2;
                yval1 = yval2;
            }
            prev_index = cur_index;
            prev_val = cur_val;
        }
        Ok(nad)
    }

    /// Otsu-style split of a histogram into a lower / upper partition.
    ///
    /// For each candidate split point `i`, computes an Otsu inter-class
    /// variance score with weights normalised by `4 / (n-1)^2`. The split
    /// point is then refined to the minimum-value bin within all contiguous
    /// scores at least `(1 - score_fract)` of the peak. The returned
    /// `split_index` is offset by `+1` (capped at 255) to match the
    /// threshold semantics of `pixThresholdToBinary` (which selects values
    /// strictly below the threshold).
    ///
    /// C Leptonica equivalent: `numaSplitDistribution` (`numafunc2.c`).
    pub fn split_distribution(
        &self,
        score_fract: f32,
        want_score: bool,
    ) -> Result<SplitDistribution> {
        if !(0.0..=1.0).contains(&score_fract) {
            return Err(Error::InvalidParameter(format!(
                "split_distribution: score_fract must be in [0.0, 1.0] (got {score_fract})"
            )));
        }
        let n = self.len();
        if n <= 1 {
            return Err(Error::InvalidParameter(format!(
                "split_distribution: n must be > 1 (got {n})"
            )));
        }
        let sum = self.sum().unwrap_or(0.0);
        if sum <= 0.0 {
            return Err(Error::InvalidParameter(format!(
                "split_distribution: histogram total must be > 0 (got {sum})"
            )));
        }
        let norm = 4.0 / ((n as f32 - 1.0).powi(2));
        let mut ave1_prev = 0.0;
        let stats = self
            .histogram_stats(0.0, 1.0)
            .ok_or_else(|| Error::InvalidParameter("histogram_stats failed".into()))?;
        let mut ave2_prev = stats.mean;
        let mut num1_prev = 0.0;
        let mut num2_prev = sum;
        let mut max_index: usize = n / 2;
        let mut max_score = 0.0_f32;

        let mut nascore = Numa::with_capacity(n);
        let mut naave1 = Numa::with_capacity(n);
        let mut naave2 = Numa::with_capacity(n);
        let mut nanum1 = Numa::with_capacity(n);
        let mut nanum2 = Numa::with_capacity(n);

        for i in 0..n {
            let val = self.get(i).unwrap_or(0.0);
            let num1 = num1_prev + val;
            let ave1 = if num1 == 0.0 {
                ave1_prev
            } else {
                (num1_prev * ave1_prev + i as f32 * val) / num1
            };
            let num2 = num2_prev - val;
            let ave2 = if num2 == 0.0 {
                ave2_prev
            } else {
                (num2_prev * ave2_prev - i as f32 * val) / num2
            };
            let fract1 = num1 / sum;
            let score = norm * (fract1 * (1.0 - fract1)) * (ave2 - ave1).powi(2);
            nascore.push(score);
            naave1.push(ave1);
            naave2.push(ave2);
            nanum1.push(num1);
            nanum2.push(num2);
            if score > max_score {
                max_score = score;
                max_index = i;
            }
            num1_prev = num1;
            num2_prev = num2;
            ave1_prev = ave1;
            ave2_prev = ave2;
        }

        // Refine split: choose minimum-value bin within all contiguous
        // scores at least (1 - score_fract) * max_score.
        let min_score = (1.0 - score_fract) * max_score;
        let mut min_range = max_index;
        // Sweep left.
        let mut i = max_index as i32 - 1;
        while i >= 0 {
            if nascore.get(i as usize).unwrap_or(0.0) < min_score {
                break;
            }
            min_range = i as usize;
            i -= 1;
        }
        let mut max_range = max_index;
        // Sweep right.
        let mut i = max_index + 1;
        while i < n {
            if nascore.get(i).unwrap_or(0.0) < min_score {
                break;
            }
            max_range = i;
            i += 1;
        }
        let mut best_split = min_range;
        let mut min_val = self.get(min_range).unwrap_or(0.0);
        for j in (min_range + 1)..=max_range {
            let v = self.get(j).unwrap_or(0.0);
            if v < min_val {
                min_val = v;
                best_split = j;
            }
        }
        // Match C: cap at 255 and add 1 to align with threshold semantics
        // ("pixThresholdToBinary" picks values strictly less than the
        // returned threshold).
        let split_index = (best_split + 1).min(255) as i32;
        let bs_idx = best_split.min(n - 1);

        Ok(SplitDistribution {
            split_index,
            ave1: naave1.get(bs_idx).unwrap_or(0.0),
            ave2: naave2.get(bs_idx).unwrap_or(0.0),
            num1: nanum1.get(bs_idx).unwrap_or(0.0),
            num2: nanum2.get(bs_idx).unwrap_or(0.0),
            score: if want_score { Some(nascore) } else { None },
        })
    }

    /// Make a histogram with automatic bin sizing.
    ///
    /// If all values are integers and the integer range fits within
    /// `maxbins`, returns a unit-width integer histogram with one bin per
    /// distinct integer value (parameters: `startx = minval, delx = 1.0`).
    /// Otherwise distributes the values into `maxbins` floating-point bins
    /// spanning `[minval, maxval]` (parameters: `startx = minval,
    /// delx = (maxval - minval) / maxbins`).
    ///
    /// C Leptonica equivalent: `numaMakeHistogramAuto` (`numafunc2.c`).
    pub fn make_histogram_auto(&self, maxbins: u32) -> Result<Numa> {
        if self.is_empty() {
            return Err(Error::InvalidParameter(
                "make_histogram_auto: input is empty".into(),
            ));
        }
        if maxbins == 0 {
            return Err(Error::InvalidParameter(
                "make_histogram_auto: maxbins must be >= 1".into(),
            ));
        }
        let maxbins = maxbins as usize;
        let minval = self.min_value().unwrap_or(0.0);
        let maxval = self.max_value().unwrap_or(0.0);
        let n = self.len();

        // Integer fast path.
        let allints = self.has_only_integers(0.0);
        if allints && (maxval - minval) < maxbins as f32 {
            let imin = minval as i32;
            let imax = maxval as i32;
            let irange = (imax - imin + 1) as usize;
            let mut hist = Numa::with_capacity(irange);
            for _ in 0..irange {
                hist.push(0.0);
            }
            hist.set_parameters(minval, 1.0);
            for i in 0..n {
                let v = self.get(i).unwrap_or(0.0) as i32;
                let bin = (v - imin) as usize;
                if bin < irange {
                    let prev = hist.get(bin).unwrap_or(0.0);
                    let _ = hist.set(bin, prev + 1.0);
                }
            }
            return Ok(hist);
        }

        // Float-bin path.
        let range = maxval - minval;
        if range == 0.0 {
            // All values identical: produce a single-bin histogram.
            // Use delx = 1.0 (matching the integer constant path and
            // `make_histogram`) so downstream histogram APIs that divide by
            // delx don't yield NaN/inf.
            let mut hist = Numa::with_capacity(1);
            hist.set_parameters(minval, 1.0);
            hist.push(n as f32);
            return Ok(hist);
        }
        let binsize = range / maxbins as f32;
        let mut hist = Numa::with_capacity(maxbins);
        for _ in 0..maxbins {
            hist.push(0.0);
        }
        hist.set_parameters(minval, binsize);
        for i in 0..n {
            let v = self.get(i).unwrap_or(0.0);
            let mut bin = ((v - minval) / binsize) as usize;
            if bin >= maxbins {
                bin = maxbins - 1;
            }
            let prev = hist.get(bin).unwrap_or(0.0);
            let _ = hist.set(bin, prev + 1.0);
        }
        Ok(hist)
    }

    /// Compute equal-population rank bin values from an arbitrary Numa.
    ///
    /// Internally chooses between a sort-based path (when the value range is
    /// large relative to length) and a histogram-based path (when the range
    /// is small, e.g. 8-bit pixel values). Returns the average value within
    /// each of the `nbins` equal-population buckets in the **original value
    /// domain** (the histogram path post-multiplies by the histogram's
    /// `binsize` and adds `binstart` so callers see real-world values rather
    /// than internal histogram indices — a small divergence from C, which
    /// returns indices).
    ///
    /// Note: matches C `numaGetRankBinValues` for the documented
    /// "no negative values" case. Negative-minimum inputs follow the
    /// histogram path with reduced resolution (C also uses only `maxval`
    /// to pick maxbins; this port mirrors that behaviour).
    ///
    /// C Leptonica equivalent: `numaGetRankBinValues` (`numafunc2.c`).
    pub fn get_rank_bin_values(&self, nbins: u32) -> Result<Numa> {
        if nbins < 2 {
            return Err(Error::InvalidParameter(format!(
                "nbins must be > 1 (got {nbins})"
            )));
        }
        if self.is_empty() {
            return Err(Error::InvalidParameter(
                "get_rank_bin_values: input is empty".into(),
            ));
        }
        let max_val = self.max_value().unwrap_or(0.0);
        let use_bin_sort = Numa::choose_sort_type(self.len(), max_val);
        if !use_bin_sort {
            // Sort-based path: shell sort the values, then equal-population
            // average across the sorted array.
            let mut sorted = self.clone();
            sorted.sort(crate::core::numa::SortOrder::Increasing);
            return sorted.discretize_sorted_in_bins(nbins);
        }
        // Histogram-based path: build a histogram up to 100002 entries and
        // discretize that. The discretizer returns averages in histogram-
        // index units; convert back to the original value domain with the
        // histogram's binstart / binsize.
        let maxbins = (max_val as i32 + 2).clamp(2, 100_002) as usize;
        let hist_result = self
            .make_histogram(maxbins)
            .ok_or_else(|| Error::InvalidParameter("make_histogram failed".into()))?;
        let (binval, _) = hist_result
            .histogram
            .discretize_histo_in_bins(nbins, false)?;
        let binstart = hist_result.binstart as f32;
        let binsize = hist_result.binsize as f32;
        let mut out = Numa::with_capacity(binval.len());
        for i in 0..binval.len() {
            out.push(binstart + binsize * binval.get(i).unwrap_or(0.0));
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
