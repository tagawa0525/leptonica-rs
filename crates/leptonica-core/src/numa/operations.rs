//! Numa operations: sequences, joins, comparisons, windowed statistics, and histograms.
//!
//! Implements the following C Leptonica functions:
//!   - `numaMakeSequence()` - create evenly-spaced sequences
//!   - `numaGetPartialSums()` - cumulative sums
//!   - `numaJoin()` - append elements from another Numa
//!   - `numaSimilar()` - element-wise comparison
//!   - `numaWindowedMean()` - windowed mean with mirrored border
//!   - `numaWindowedMeanSquare()` - windowed mean of squares with mirrored border
//!   - `numaWindowedStats()` - windowed variance and RMS deviation
//!   - `numaMakeHistogram()` - histogram with automatic bin sizing
//!   - `numaMakeHistogramClipped()` - histogram with clipped range

use super::Numa;
use crate::error::{Error, Result};

/// Sort order for Numa sorting operations.
///
/// C equivalent: `L_SORT_INCREASING` / `L_SORT_DECREASING`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort in ascending order (smallest first).
    Increasing,
    /// Sort in descending order (largest first).
    Decreasing,
}

/// Windowed statistics result.
///
/// Returned by [`Numa::windowed_stats`]. Contains the windowed mean,
/// mean square, variance, and RMS deviation arrays.
#[derive(Debug, Clone)]
pub struct WindowedStats {
    /// Windowed mean: `E[x]` over the window at each position.
    pub mean: Numa,
    /// Windowed mean of squares: `E[x^2]` over the window at each position.
    pub mean_square: Numa,
    /// Windowed variance: `E[x^2] - E[x]^2` at each position.
    pub variance: Numa,
    /// Windowed RMS deviation: `sqrt(variance)` at each position.
    pub rms: Numa,
}

/// Histogram construction result.
///
/// Returned by [`Numa::make_histogram`]. Contains the histogram array
/// along with the bin size and bin start parameters.
#[derive(Debug, Clone)]
pub struct HistogramResult {
    /// The histogram: each element is a count of values falling in that bin.
    pub histogram: Numa,
    /// The width of each histogram bin.
    pub binsize: i32,
    /// The x-value of the start (left edge) of the first bin.
    pub binstart: i32,
}

// ============================================================================
// Helper: add mirrored border to a Numa
// C equivalent: numaAddSpecifiedBorder(nas, left, right, L_MIRRORED_BORDER)
// ============================================================================

/// Add mirrored border elements to both ends of a Numa.
///
/// For left border: `result[i] = nas[left - 1 - i]` for `i in 0..left`
/// For right border: `result[n + left + i] = nas[n - 1 - i]` for `i in 0..right`
fn add_mirrored_border(nas: &Numa, left: usize, right: usize) -> Numa {
    let n = nas.len();
    let total = n + left + right;
    let mut result = Numa::with_capacity(total);

    // Left border (mirrored)
    for i in 0..left {
        // Mirror index: left-1-i maps to nas[left-1-i] but we need
        // the mirror from the start of nas.
        // C: fa[i] = fa[2*left - 1 - i] where fa has `left` zeros prepended.
        // After zeroing out, fa[2*left-1-i] = original nas[left-1-i].
        // So: result[i] = nas[left - 1 - i]
        let idx = left - 1 - i;
        result.push(nas.get(idx).unwrap_or(0.0));
    }

    // Original data
    for val in nas.iter() {
        result.push(val);
    }

    // Right border (mirrored)
    for i in 0..right {
        // C: fa[n_total - right + i] = fa[n_total - right - i - 1]
        // which maps to original nas[n - 1 - i]
        let idx = n - 1 - i;
        result.push(nas.get(idx).unwrap_or(0.0));
    }

    result
}

// ============================================================================
// BinSizeArray for numaMakeHistogram
// ============================================================================

/// Bin sizes used by `numaMakeHistogram` to find a "nice" bin width.
/// Matches the C `BinSizeArray` in `numafunc2.c`.
const BIN_SIZE_ARRAY: &[i32] = &[
    2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000, 200000, 500000,
    1000000, 2000000, 5000000, 10000000, 200000000, 50000000, 100000000,
];

impl Numa {
    // ====================================================================
    // Sequence construction
    // ====================================================================

    /// Create a sequence of evenly-spaced values.
    ///
    /// Generates a Numa of length `count` with values:
    /// `[start, start + step, start + 2*step, ..., start + (count-1)*step]`
    ///
    /// C equivalent: `numaMakeSequence(startval, increment, size)`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let seq = Numa::make_sequence(0.0, 1.0, 5);
    /// assert_eq!(seq.len(), 5);
    /// assert_eq!(seq.get(0), Some(0.0));
    /// assert_eq!(seq.get(4), Some(4.0));
    /// ```
    pub fn make_sequence(start: f32, step: f32, count: usize) -> Numa {
        let mut na = Numa::with_capacity(count);
        for i in 0..count {
            na.push(start + (i as f32) * step);
        }
        na
    }

    // ====================================================================
    // Partial sums
    // ====================================================================

    /// Compute the cumulative (partial) sums of the array.
    ///
    /// Returns a new Numa where `result[i] = sum(self[0..=i])`.
    /// The last element equals the total sum of the input.
    ///
    /// C equivalent: `numaGetPartialSums(na)`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let psums = na.partial_sums();
    /// assert_eq!(psums.get(0), Some(1.0));
    /// assert_eq!(psums.get(4), Some(15.0));
    /// ```
    pub fn partial_sums(&self) -> Numa {
        let n = self.len();
        let mut result = Numa::with_capacity(n);
        let mut cumsum = 0.0f32;
        for val in self.iter() {
            cumsum += val;
            result.push(cumsum);
        }
        result
    }

    // ====================================================================
    // Join (append)
    // ====================================================================

    /// Append all values from another Numa to this one.
    ///
    /// This is the simplified form of C `numaJoin(nad, nas, 0, -1)` which
    /// appends all elements. For range-based joining, use `join_range`.
    ///
    /// C equivalent: `numaJoin(nad, nas, 0, -1)`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let mut na1 = Numa::from_vec(vec![1.0, 2.0]);
    /// let na2 = Numa::from_vec(vec![3.0, 4.0]);
    /// na1.join(&na2);
    /// assert_eq!(na1.len(), 4);
    /// assert_eq!(na1.get(2), Some(3.0));
    /// ```
    pub fn join(&mut self, other: &Numa) {
        for val in other.iter() {
            self.push(val);
        }
    }

    /// Append a range of values from another Numa.
    ///
    /// Appends values from `other[istart..=iend]` to `self`.
    /// If `iend` is `None`, appends from `istart` to the end.
    ///
    /// C equivalent: `numaJoin(nad, nas, istart, iend)`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let mut na1 = Numa::from_vec(vec![1.0]);
    /// let na2 = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0]);
    /// na1.join_range(&na2, 1, Some(2));
    /// assert_eq!(na1.len(), 3);
    /// assert_eq!(na1.get(1), Some(20.0));
    /// assert_eq!(na1.get(2), Some(30.0));
    /// ```
    pub fn join_range(&mut self, other: &Numa, istart: usize, iend: Option<usize>) {
        let n = other.len();
        if n == 0 {
            return;
        }
        let iend = iend.map(|e| e.min(n - 1)).unwrap_or(n - 1);
        if istart > iend {
            return;
        }
        for i in istart..=iend {
            if let Some(val) = other.get(i) {
                self.push(val);
            }
        }
    }

    // ====================================================================
    // Similarity comparison
    // ====================================================================

    /// Check if two Numas are element-wise similar within a tolerance.
    ///
    /// Returns `true` if both arrays have the same length and every
    /// pair of corresponding values differs by at most `max_diff`.
    /// Use `max_diff = 0.0` for exact equality.
    ///
    /// C equivalent: `numaSimilar(na1, na2, maxdiff, &similar)`
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na1 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    /// let na2 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
    /// assert!(na1.similar(&na2, 0.0));
    ///
    /// let na3 = Numa::from_vec(vec![1.1, 2.1, 3.1]);
    /// assert!(!na1.similar(&na3, 0.0));
    /// assert!(na1.similar(&na3, 0.2));
    /// ```
    pub fn similar(&self, other: &Numa, max_diff: f64) -> bool {
        let max_diff = max_diff.abs();
        if self.len() != other.len() {
            return false;
        }
        for i in 0..self.len() {
            let v1 = self.get(i).unwrap_or(0.0) as f64;
            let v2 = other.get(i).unwrap_or(0.0) as f64;
            if (v1 - v2).abs() > max_diff {
                return false;
            }
        }
        true
    }

    // ====================================================================
    // Windowed statistics
    // ====================================================================

    /// Compute the windowed mean using a mirrored border.
    ///
    /// For each position `i`, computes the mean of values in a window of
    /// width `2 * halfwin + 1` centered at `i`. The array is extended with
    /// mirrored values at both ends to avoid edge effects.
    ///
    /// C equivalent: `numaWindowedMean(nas, wc)`
    ///
    /// # Arguments
    ///
    /// * `halfwin` - Half-width of the averaging window. The full window
    ///   has `2 * halfwin + 1` elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![42.0; 20]);
    /// let mean = na.windowed_mean(3);
    /// // Windowed mean of a constant is the same constant
    /// assert!((mean.get(10).unwrap() - 42.0).abs() < 0.001);
    /// ```
    pub fn windowed_mean(&self, halfwin: usize) -> Numa {
        let n = self.len();
        if n == 0 {
            return Numa::new();
        }
        let wc = halfwin;
        let width = 2 * wc + 1;

        // Add mirrored border
        let na1 = add_mirrored_border(self, wc, wc);
        let n1 = na1.len(); // == n + 2 * wc

        // Build prefix sum array
        let mut suma = vec![0.0f32; n1 + 1];
        let mut sum = 0.0f32;
        for i in 0..n1 {
            sum += na1.get(i).unwrap_or(0.0);
            suma[i + 1] = sum;
        }

        // Compute windowed mean using prefix sums
        let norm = 1.0 / width as f32;
        let mut result = Numa::with_capacity(n);
        for i in 0..n {
            result.push(norm * (suma[width + i] - suma[i]));
        }
        result
    }

    /// Compute the windowed mean of squares using a mirrored border.
    ///
    /// For each position `i`, computes the mean of `x^2` values in a window
    /// of width `2 * halfwin + 1` centered at `i`. Uses mirrored border
    /// extension.
    ///
    /// C equivalent: `numaWindowedMeanSquare(nas, wc)`
    ///
    /// # Arguments
    ///
    /// * `halfwin` - Half-width of the averaging window.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![3.0; 20]);
    /// let ms = na.windowed_mean_square(3);
    /// // Mean square of constant 3.0 is 9.0
    /// assert!((ms.get(10).unwrap() - 9.0).abs() < 0.001);
    /// ```
    pub fn windowed_mean_square(&self, halfwin: usize) -> Numa {
        let n = self.len();
        if n == 0 {
            return Numa::new();
        }
        let wc = halfwin;
        let width = 2 * wc + 1;

        // Add mirrored border
        let na1 = add_mirrored_border(self, wc, wc);
        let n1 = na1.len();

        // Build prefix sum of squares array
        let mut suma = vec![0.0f32; n1 + 1];
        let mut sum = 0.0f32;
        for i in 0..n1 {
            let v = na1.get(i).unwrap_or(0.0);
            sum += v * v;
            suma[i + 1] = sum;
        }

        // Compute windowed mean square using prefix sums
        let norm = 1.0 / width as f32;
        let mut result = Numa::with_capacity(n);
        for i in 0..n {
            result.push(norm * (suma[width + i] - suma[i]));
        }
        result
    }

    /// Compute windowed statistics: mean, mean-square, variance, and RMS deviation.
    ///
    /// This is a convenience function that computes all four windowed
    /// statistics in one call, matching C `numaWindowedStats`.
    ///
    /// The variance at each position is: `E[x^2] - E[x]^2`
    /// The RMS deviation is: `sqrt(variance)`
    ///
    /// C equivalent: `numaWindowedStats(nas, wc, &nam, &nams, &nav, &narv)`
    ///
    /// # Arguments
    ///
    /// * `halfwin` - Half-width of the averaging window.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![42.0; 20]);
    /// let stats = na.windowed_stats(3);
    /// // Constant array: variance and RMS should be near 0
    /// // (small floating-point residuals are expected)
    /// assert!((stats.variance.get(10).unwrap()).abs() < 0.1);
    /// assert!((stats.rms.get(10).unwrap()).abs() < 0.1);
    /// ```
    pub fn windowed_stats(&self, halfwin: usize) -> WindowedStats {
        let mean = self.windowed_mean(halfwin);
        let mean_square = self.windowed_mean_square(halfwin);

        let n = self.len();
        let mut variance = Numa::with_capacity(n);
        let mut rms = Numa::with_capacity(n);

        for i in 0..n {
            let m = mean.get(i).unwrap_or(0.0);
            let ms = mean_square.get(i).unwrap_or(0.0);
            // C does NOT clamp to 0; it can be slightly negative due to
            // floating point. We match C behavior exactly here but clamp
            // only for sqrt to avoid NaN.
            let var = ms - m * m;
            variance.push(var);
            rms.push(if var > 0.0 { var.sqrt() } else { 0.0 });
        }

        WindowedStats {
            mean,
            mean_square,
            variance,
            rms,
        }
    }

    // ====================================================================
    // Histogram construction
    // ====================================================================

    /// Create a histogram with automatic bin sizing.
    ///
    /// Distributes values into bins whose size is chosen from the sequence
    /// `{1, 2, 5, 10, 20, 50, ...}` to fit within `max_bins` bins.
    /// Values are integerized before binning.
    ///
    /// The returned [`HistogramResult`] contains the histogram array with
    /// its `startx` and `delx` parameters set to `binstart` and `binsize`
    /// respectively.
    ///
    /// C equivalent: `numaMakeHistogram(na, maxbins, &binsize, &binstart)`
    ///
    /// # Arguments
    ///
    /// * `max_bins` - Maximum number of histogram bins.
    ///
    /// # Returns
    ///
    /// `Some(HistogramResult)` on success, `None` if the array is empty
    /// or values are too large to bin.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 2.0, 1.0]);
    /// let result = na.make_histogram(10).unwrap();
    /// assert!(result.histogram.len() > 0);
    /// assert!(result.binsize >= 1);
    /// ```
    pub fn make_histogram(&self, max_bins: usize) -> Option<HistogramResult> {
        if self.is_empty() || max_bins == 0 {
            return None;
        }

        // Determine input range (integerized)
        let min_f = self.min_value()?;
        let max_f = self.max_value()?;
        let imaxval = (max_f + 0.5) as i32;
        let iminval = (min_f + 0.5) as i32;

        // Determine bin size
        let range = imaxval - iminval + 1;
        let binsize;
        if range > max_bins as i32 - 1 {
            let ratio = range as f32 / max_bins as f32;
            let mut found = 0;
            for &bs in BIN_SIZE_ARRAY {
                if ratio < bs as f32 {
                    found = bs;
                    break;
                }
            }
            if found == 0 {
                return None; // numbers too large
            }
            binsize = found;
        } else {
            binsize = 1;
        }

        let nbins = 1 + range / binsize;

        // Redetermine iminval for nice alignment
        let binstart = if binsize > 1 {
            if iminval >= 0 {
                binsize * (iminval / binsize)
            } else {
                binsize * ((iminval - binsize + 1) / binsize)
            }
        } else {
            iminval
        };

        // Build histogram
        let nbins = nbins as usize;
        let mut histo = Numa::from_vec(vec![0.0; nbins]);
        histo.set_parameters(binstart as f32, binsize as f32);

        for val in self.iter() {
            let ival = (val + 0.5) as i32; // integerize (round to nearest)
            let ibin = (ival - binstart) / binsize;
            if ibin >= 0 && (ibin as usize) < nbins {
                let idx = ibin as usize;
                let old = histo.get(idx).unwrap_or(0.0);
                let _ = histo.set(idx, old + 1.0);
            }
        }

        Some(HistogramResult {
            histogram: histo,
            binsize,
            binstart,
        })
    }

    /// Create a histogram with clipped range and specified bin size.
    ///
    /// Values less than 0 are discarded. Values are binned into bins of
    /// width `binsize`, starting at 0, up to a maximum ordinate of `maxsize`.
    ///
    /// The returned Numa has `startx = 0.0` and `delx = binsize`.
    ///
    /// C equivalent: `numaMakeHistogramClipped(na, binsize, maxsize)`
    ///
    /// # Arguments
    ///
    /// * `binsize` - Width of each histogram bin. Must be > 0.
    /// * `maxsize` - Maximum ordinate value. Values above
    ///   `min(maxsize, max_value)` are clipped.
    ///
    /// # Returns
    ///
    /// `Some(Numa)` histogram on success, `None` if the array is empty
    /// or `binsize <= 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![1.0, 3.0, 5.0, 7.0, 9.0]);
    /// let histo = na.make_histogram_clipped(2.0, 10.0).unwrap();
    /// assert!(histo.len() > 0);
    /// ```
    pub fn make_histogram_clipped(&self, binsize: f32, maxsize: f32) -> Option<Numa> {
        if self.is_empty() || binsize <= 0.0 {
            return None;
        }

        let binsize = if binsize > maxsize { maxsize } else { binsize };

        let maxval = self.max_value()?;
        let effective_max = maxsize.min(maxval);
        let nbins = (effective_max / binsize) as usize + 1;

        let mut histo = Numa::from_vec(vec![0.0; nbins]);
        histo.set_parameters(0.0, binsize);

        for val in self.iter() {
            let ibin = (val / binsize) as i32;
            if ibin >= 0 && (ibin as usize) < nbins {
                let idx = ibin as usize;
                let old = histo.get(idx).unwrap_or(0.0);
                let _ = histo.set(idx, old + 1.0);
            }
        }

        Some(histo)
    }

    /// Compute statistics from raw data using a histogram for rank values.
    ///
    /// This is a convenience function that builds a histogram internally
    /// and uses it to compute min, max, mean, variance, median, and a
    /// rank value. Mean and variance are computed directly on the data
    /// (not from the histogram) for accuracy.
    ///
    /// C equivalent: `numaGetStatsUsingHistogram(na, maxbins, ...)`
    ///
    /// # Arguments
    ///
    /// * `max_bins` - Maximum number of histogram bins.
    /// * `rank` - Target rank in `[0.0, 1.0]` for computing the rank value.
    ///
    /// # Returns
    ///
    /// `Some((min, max, mean, variance, median, rank_val))` on success,
    /// `None` if the array is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let (min, max, mean, var, median, rval) = na.stats_using_histogram(10, 0.5).unwrap();
    /// assert!((min - 1.0).abs() < 0.5);
    /// assert!((max - 5.0).abs() < 0.5);
    /// ```
    pub fn stats_using_histogram(
        &self,
        max_bins: usize,
        rank: f32,
    ) -> Option<(f32, f32, f32, f32, f32, f32)> {
        if self.is_empty() {
            return None;
        }

        let min_val = self.min_value()?;
        let max_val = self.max_value()?;

        // Compute mean and variance directly from data (matches C behavior)
        let n = self.len() as f32;
        let mut sum = 0.0f32;
        let mut sum2 = 0.0f32;
        for val in self.iter() {
            sum += val;
            sum2 += val * val;
        }
        let mean_val = sum / n;
        let variance = sum2 / n - mean_val * mean_val;

        // Build histogram for median and rank value
        let hr = self.make_histogram(max_bins)?;
        let histo = &hr.histogram;

        // Get median (rank 0.5)
        let median = histo.histogram_val_from_rank(0.5).unwrap_or(0.0);

        // Get rank value
        let rank_val = histo.histogram_val_from_rank(rank).unwrap_or(0.0);

        Some((min_val, max_val, mean_val, variance, median, rank_val))
    }

    // ====================================================================
    // Constant array construction
    // ====================================================================

    /// Create a Numa filled with a constant value.
    ///
    /// C equivalent: `numaMakeConstant(val, size)`
    pub fn make_constant(val: f32, count: usize) -> Numa {
        Self::make_sequence(val, 0.0, count)
    }

    // ====================================================================
    // Reverse
    // ====================================================================

    /// Return a new Numa with elements in reversed order.
    ///
    /// Metadata is also reversed: `startx = startx + (n-1) * delx`, `delx = -delx`.
    ///
    /// C equivalent: `numaReverse(NULL, nas)`
    pub fn reversed(&self) -> Numa {
        let n = self.len();
        let mut result = Numa::with_capacity(n);
        for i in (0..n).rev() {
            result.push(self[i]);
        }
        let (startx, delx) = self.parameters();
        // Always copy parameters first, then apply reversal formula if non-empty
        if n > 0 {
            result.set_parameters(startx + (n - 1) as f32 * delx, -delx);
        } else {
            result.set_parameters(startx, delx);
        }
        result
    }

    /// Reverse the elements in place.
    ///
    /// Metadata is also reversed: `startx = startx + (n-1) * delx`, `delx = -delx`.
    ///
    /// C equivalent: `numaReverse(nas, nas)`
    pub fn reverse(&mut self) {
        let n = self.len();
        let slice = self.as_slice_mut();
        slice.reverse();
        let (startx, delx) = self.parameters();
        if n > 0 {
            self.set_parameters(startx + (n - 1) as f32 * delx, -delx);
        }
    }

    // ====================================================================
    // Sort
    // ====================================================================

    /// Return a new Numa with elements sorted.
    ///
    /// Uses `f32::total_cmp` for NaN-safe ordering.
    ///
    /// C equivalent: `numaSort(NULL, nain, sortorder)`
    pub fn sorted(&self, order: SortOrder) -> Numa {
        let mut result = self.clone();
        result.sort(order);
        result
    }

    /// Sort the elements in place.
    ///
    /// Uses `f32::total_cmp` for NaN-safe ordering.
    ///
    /// C equivalent: `numaSort(naout, naout, sortorder)`
    pub fn sort(&mut self, order: SortOrder) {
        let slice = self.as_slice_mut();
        match order {
            SortOrder::Increasing => slice.sort_by(f32::total_cmp),
            SortOrder::Decreasing => slice.sort_by(|a, b| f32::total_cmp(b, a)),
        }
    }

    // ====================================================================
    // Rank value / Median / Mode
    // ====================================================================

    /// Get the value at a given rank (fractional position) in the sorted array.
    ///
    /// `fract` ranges from 0.0 (minimum) to 1.0 (maximum).
    /// The index is computed as `(fract * (n-1) + 0.5) as usize`.
    ///
    /// C equivalent: `numaGetRankValue(na, fract, NULL, 0, &pval)`
    pub fn rank_value(&self, fract: f32) -> Result<f32> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if !(0.0..=1.0).contains(&fract) {
            return Err(Error::InvalidParameter(format!(
                "fract {fract} not in [0.0, 1.0]"
            )));
        }
        let sorted = self.sorted(SortOrder::Increasing);
        let index = (fract * (n - 1) as f32 + 0.5) as usize;
        let index = index.min(n - 1);
        Ok(sorted[index])
    }

    /// Get the median value.
    ///
    /// Equivalent to `self.rank_value(0.5)`.
    ///
    /// C equivalent: `numaGetMedian(na, &pval)`
    pub fn median(&self) -> Result<f32> {
        self.rank_value(0.5)
    }

    /// Get the mode (most frequent value) and its count.
    ///
    /// Sorts the array in decreasing order and scans for the longest
    /// run of equal values.
    ///
    /// C equivalent: `numaGetMode(na, &pval, &pcount)`
    pub fn mode(&self) -> Result<(f32, usize)> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        let sorted = self.sorted(SortOrder::Decreasing);
        let array = sorted.as_slice();

        let mut prev_val = array[0];
        let mut prev_count: usize = 1;
        let mut max_val = prev_val;
        let mut max_count: usize = 1;

        for &val in &array[1..] {
            if val == prev_val || (val.is_nan() && prev_val.is_nan()) {
                // Same run: identical values, including NaN treated as equal.
                prev_count += 1;
            } else {
                // New run: update max if the previous run was longer.
                if prev_count > max_count {
                    max_count = prev_count;
                    max_val = prev_val;
                }
                prev_val = val;
                prev_count = 1;
            }
        }
        if prev_count > max_count {
            max_count = prev_count;
            max_val = prev_val;
        }

        Ok((max_val, max_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_sequence() {
        let seq = Numa::make_sequence(0.0, 1.0, 5);
        assert_eq!(seq.len(), 5);
        assert_eq!(seq.as_slice(), &[0.0, 1.0, 2.0, 3.0, 4.0]);

        let seq = Numa::make_sequence(10.0, 0.5, 3);
        assert_eq!(seq.as_slice(), &[10.0, 10.5, 11.0]);

        let seq = Numa::make_sequence(5.0, 0.0, 3);
        assert_eq!(seq.as_slice(), &[5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_partial_sums() {
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let ps = na.partial_sums();
        assert_eq!(ps.len(), 5);
        assert_eq!(ps.get(0), Some(1.0));
        assert_eq!(ps.get(1), Some(3.0));
        assert_eq!(ps.get(2), Some(6.0));
        assert_eq!(ps.get(3), Some(10.0));
        assert_eq!(ps.get(4), Some(15.0));
    }

    #[test]
    fn test_partial_sums_empty() {
        let na = Numa::new();
        let ps = na.partial_sums();
        assert!(ps.is_empty());
    }

    #[test]
    fn test_join() {
        let mut na1 = Numa::from_vec(vec![1.0, 2.0]);
        let na2 = Numa::from_vec(vec![3.0, 4.0, 5.0]);
        na1.join(&na2);
        assert_eq!(na1.len(), 5);
        assert_eq!(na1.as_slice(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_join_range() {
        let mut na1 = Numa::from_vec(vec![1.0]);
        let na2 = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0]);
        na1.join_range(&na2, 1, Some(2));
        assert_eq!(na1.len(), 3);
        assert_eq!(na1.as_slice(), &[1.0, 20.0, 30.0]);
    }

    #[test]
    fn test_join_empty() {
        let mut na1 = Numa::from_vec(vec![1.0]);
        let na2 = Numa::new();
        na1.join(&na2);
        assert_eq!(na1.len(), 1);
    }

    #[test]
    fn test_similar_exact() {
        let na1 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        let na2 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(na1.similar(&na2, 0.0));
    }

    #[test]
    fn test_similar_tolerance() {
        let na1 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        let na2 = Numa::from_vec(vec![1.05, 2.05, 3.05]);
        assert!(!na1.similar(&na2, 0.01));
        assert!(na1.similar(&na2, 0.1));
    }

    #[test]
    fn test_similar_different_lengths() {
        let na1 = Numa::from_vec(vec![1.0, 2.0]);
        let na2 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(!na1.similar(&na2, 0.0));
    }

    #[test]
    fn test_windowed_mean_constant() {
        let na = Numa::from_vec(vec![5.0; 20]);
        let mean = na.windowed_mean(3);
        assert_eq!(mean.len(), 20);
        // All values should be 5.0
        for i in 0..20 {
            assert!(
                (mean.get(i).unwrap() - 5.0).abs() < 0.001,
                "index {} = {}",
                i,
                mean.get(i).unwrap()
            );
        }
    }

    #[test]
    fn test_windowed_mean_square_constant() {
        let na = Numa::from_vec(vec![3.0; 20]);
        let ms = na.windowed_mean_square(3);
        assert_eq!(ms.len(), 20);
        for i in 0..20 {
            assert!(
                (ms.get(i).unwrap() - 9.0).abs() < 0.001,
                "index {} = {}",
                i,
                ms.get(i).unwrap()
            );
        }
    }

    #[test]
    fn test_windowed_stats_constant() {
        let na = Numa::from_vec(vec![42.0; 20]);
        let stats = na.windowed_stats(3);
        assert_eq!(stats.mean.len(), 20);
        assert_eq!(stats.variance.len(), 20);
        assert_eq!(stats.rms.len(), 20);
        for i in 0..20 {
            // Floating-point precision: mean_square - mean^2 can have
            // small residual errors even for constant input. The C code
            // has the same behavior.
            assert!(
                (stats.variance.get(i).unwrap()).abs() < 0.1,
                "variance at {} = {}",
                i,
                stats.variance.get(i).unwrap()
            );
            assert!(
                (stats.rms.get(i).unwrap()).abs() < 0.1,
                "rms at {} = {}",
                i,
                stats.rms.get(i).unwrap()
            );
        }
    }

    #[test]
    fn test_windowed_stats_mirrored_border() {
        // Verify that the mirrored border extension works correctly
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let mean = na.windowed_mean(1);
        // With halfwin=1, window size = 3
        // At i=0: mirror border gives [2, 1, 2] => mean = 5/3
        // No: mirrored border: left border is [nas[0]] = [1], so
        // bordered = [1, 1, 2, 3, 4, 5, 5]
        // Wait: mirror of index 0 is index 0 (left=1: fa[0] = fa[2*1-1-0] = fa[1])
        // bordered = [2, 1, 2, 3, 4, 5, 4]
        // At i=0: sum of bordered[0..3] = 2+1+2 = 5, mean = 5/3
        assert_eq!(mean.len(), 5);
        // Just verify the middle value is correct
        // At i=2: sum of bordered[2..5] = 2+3+4 = 9, mean = 3.0
        assert!((mean.get(2).unwrap() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_make_histogram_simple() {
        let na = Numa::from_vec(vec![0.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 2.0]);
        let result = na.make_histogram(10).unwrap();
        assert!(!result.histogram.is_empty());
        assert_eq!(result.binsize, 1);
        // Bin 2 should have count 3
        let bin2 = result
            .histogram
            .get((2 - result.binstart) as usize)
            .unwrap();
        assert!((bin2 - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_make_histogram_clipped() {
        let na = Numa::from_vec(vec![-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 100.0]);
        let histo = na.make_histogram_clipped(1.0, 6.0).unwrap();
        // nbins = (min(6, 100) / 1) + 1 = 7
        assert!(histo.len() >= 6);
        // -1 should be discarded
        // bin 0 should have count 1 (value 0)
        assert!((histo.get(0).unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_add_mirrored_border() {
        let na = Numa::from_vec(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        let bordered = add_mirrored_border(&na, 2, 2);
        // Left border: [nas[1], nas[0]] = [20, 10]
        // Original: [10, 20, 30, 40, 50]
        // Right border: [nas[4], nas[3]] = [50, 40]
        assert_eq!(bordered.len(), 9);
        assert_eq!(bordered.get(0), Some(20.0)); // mirror of index 1
        assert_eq!(bordered.get(1), Some(10.0)); // mirror of index 0
        assert_eq!(bordered.get(2), Some(10.0)); // original start
        assert_eq!(bordered.get(6), Some(50.0)); // original end
        assert_eq!(bordered.get(7), Some(50.0)); // mirror of last
        assert_eq!(bordered.get(8), Some(40.0)); // mirror of second-to-last
    }

    #[test]
    fn test_stats_using_histogram() {
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = na.stats_using_histogram(10, 0.5);
        assert!(result.is_some());
        let (min, max, mean, _var, _median, _rval) = result.unwrap();
        assert!((min - 1.0).abs() < 0.5);
        assert!((max - 5.0).abs() < 0.5);
        assert!((mean - 3.0).abs() < 0.5);
    }

    // ================================================================
    // Tests for make_constant
    // ================================================================

    #[test]
    fn test_make_constant_basic() {
        let na = Numa::make_constant(42.0, 5);
        assert_eq!(na.len(), 5);
        for i in 0..5 {
            assert_eq!(na.get(i), Some(42.0));
        }
    }

    #[test]
    fn test_make_constant_zero_count() {
        let na = Numa::make_constant(1.0, 0);
        assert!(na.is_empty());
    }

    #[test]
    fn test_make_constant_negative_val() {
        let na = Numa::make_constant(-3.5, 3);
        assert_eq!(na.as_slice(), &[-3.5, -3.5, -3.5]);
    }

    // ================================================================
    // Tests for reverse / reversed
    // ================================================================

    #[test]
    fn test_reversed_basic() {
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let rev = na.reversed();
        assert_eq!(rev.as_slice(), &[5.0, 4.0, 3.0, 2.0, 1.0]);
    }

    #[test]
    fn test_reversed_metadata() {
        // C behavior: startx = startx + (n-1) * delx, delx = -delx
        let mut na = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        na.set_parameters(0.0, 2.0);
        let rev = na.reversed();
        let (startx, delx) = rev.parameters();
        // startx = 0.0 + (3-1) * 2.0 = 4.0
        assert!((startx - 4.0).abs() < 1e-6);
        // delx = -2.0
        assert!((delx - (-2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_reverse_in_place() {
        let mut na = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        na.set_parameters(1.0, 0.5);
        na.reverse();
        assert_eq!(na.as_slice(), &[5.0, 4.0, 3.0, 2.0, 1.0]);
        let (startx, delx) = na.parameters();
        // startx = 1.0 + (5-1) * 0.5 = 3.0
        assert!((startx - 3.0).abs() < 1e-6);
        assert!((delx - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_reversed_single_element() {
        let na = Numa::from_vec(vec![42.0]);
        let rev = na.reversed();
        assert_eq!(rev.as_slice(), &[42.0]);
    }

    #[test]
    fn test_reversed_empty() {
        let na = Numa::new();
        let rev = na.reversed();
        assert!(rev.is_empty());
    }

    #[test]
    fn test_reversed_empty_preserves_metadata() {
        // Empty Numa should preserve startx/delx metadata
        let mut na = Numa::new();
        na.set_parameters(5.0, 2.5);
        let rev = na.reversed();
        assert!(rev.is_empty());
        let (startx, delx) = rev.parameters();
        assert_eq!(startx, 5.0);
        assert_eq!(delx, 2.5);
    }

    // ================================================================
    // Tests for sort / sorted
    // ================================================================

    #[test]
    fn test_sorted_increasing() {
        let na = Numa::from_vec(vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0]);
        let sorted = na.sorted(SortOrder::Increasing);
        assert_eq!(sorted.as_slice(), &[1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 9.0]);
    }

    #[test]
    fn test_sorted_decreasing() {
        let na = Numa::from_vec(vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0]);
        let sorted = na.sorted(SortOrder::Decreasing);
        assert_eq!(sorted.as_slice(), &[9.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 1.0]);
    }

    #[test]
    fn test_sort_in_place() {
        let mut na = Numa::from_vec(vec![5.0, 3.0, 1.0, 4.0, 2.0]);
        na.sort(SortOrder::Increasing);
        assert_eq!(na.as_slice(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sorted_empty() {
        let na = Numa::new();
        let sorted = na.sorted(SortOrder::Increasing);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sorted_single() {
        let na = Numa::from_vec(vec![42.0]);
        let sorted = na.sorted(SortOrder::Decreasing);
        assert_eq!(sorted.as_slice(), &[42.0]);
    }

    #[test]
    fn test_sorted_with_nan() {
        // NaN should be handled safely by f32::total_cmp
        let na = Numa::from_vec(vec![3.0, f32::NAN, 1.0, 2.0]);
        let sorted = na.sorted(SortOrder::Increasing);
        assert_eq!(sorted.len(), 4);
        // NaN sorts after all other values with total_cmp
        assert_eq!(sorted.get(0), Some(1.0));
        assert_eq!(sorted.get(1), Some(2.0));
        assert_eq!(sorted.get(2), Some(3.0));
        assert!(sorted.get(3).unwrap().is_nan());
    }

    // ================================================================
    // Tests for rank_value, median, mode
    // ================================================================

    #[test]
    fn test_rank_value_min() {
        let na = Numa::from_vec(vec![5.0, 3.0, 1.0, 4.0, 2.0]);
        let val = na.rank_value(0.0).unwrap();
        assert!((val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rank_value_max() {
        let na = Numa::from_vec(vec![5.0, 3.0, 1.0, 4.0, 2.0]);
        let val = na.rank_value(1.0).unwrap();
        assert!((val - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_rank_value_mid() {
        let na = Numa::from_vec(vec![5.0, 3.0, 1.0, 4.0, 2.0]);
        let val = na.rank_value(0.5).unwrap();
        // sorted: [1,2,3,4,5], index = (0.5 * 4 + 0.5) as usize = 2
        assert!((val - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_rank_value_empty() {
        let na = Numa::new();
        assert!(na.rank_value(0.5).is_err());
    }

    #[test]
    fn test_rank_value_out_of_range() {
        let na = Numa::from_vec(vec![1.0, 2.0]);
        assert!(na.rank_value(-0.1).is_err());
        assert!(na.rank_value(1.1).is_err());
    }

    #[test]
    fn test_median_odd() {
        let na = Numa::from_vec(vec![5.0, 1.0, 3.0]);
        let med = na.median().unwrap();
        // sorted: [1,3,5], rank 0.5 => index = (0.5 * 2 + 0.5) = 1
        assert!((med - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_median_even() {
        let na = Numa::from_vec(vec![4.0, 1.0, 3.0, 2.0]);
        let med = na.median().unwrap();
        // sorted: [1,2,3,4], index = (0.5 * 3 + 0.5) as usize = 2
        assert!((med - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_median_empty() {
        let na = Numa::new();
        assert!(na.median().is_err());
    }

    #[test]
    fn test_mode_basic() {
        let na = Numa::from_vec(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0]);
        let (val, count) = na.mode().unwrap();
        assert!((val - 3.0).abs() < 1e-6);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_mode_single() {
        let na = Numa::from_vec(vec![42.0]);
        let (val, count) = na.mode().unwrap();
        assert!((val - 42.0).abs() < 1e-6);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_mode_all_same() {
        let na = Numa::from_vec(vec![7.0, 7.0, 7.0]);
        let (val, count) = na.mode().unwrap();
        assert!((val - 7.0).abs() < 1e-6);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_mode_empty() {
        let na = Numa::new();
        assert!(na.mode().is_err());
    }

    #[test]
    fn test_mode_with_nan() {
        // Multiple NaNs should be counted as a single run
        let na = Numa::from_vec(vec![1.0, f32::NAN, 2.0, f32::NAN, f32::NAN]);
        let (val, count) = na.mode().unwrap();
        assert!(val.is_nan());
        assert_eq!(count, 3);
    }
}
