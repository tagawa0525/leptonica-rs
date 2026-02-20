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

use super::{Numa, Numaa};
use crate::error::{Error, Result};

/// Arithmetic operation for element-wise Numa arithmetic.
///
/// C equivalent: `L_ARITH_ADD` / `L_ARITH_SUBTRACT` / `L_ARITH_MULTIPLY` / `L_ARITH_DIVIDE`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    /// Add element-wise.
    Add,
    /// Subtract element-wise (`self - other`).
    Subtract,
    /// Multiply element-wise.
    Multiply,
    /// Divide element-wise (`self / other`).
    Divide,
}

/// Logical operation for element-wise Numa logical operations.
///
/// C equivalent: `L_UNION` / `L_INTERSECTION` / `L_SUBTRACTION` / `L_EXCLUSIVE_OR`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    /// Logical OR (`val1 || val2`).
    Union,
    /// Logical AND (`val1 && val2`).
    Intersection,
    /// Logical AND-NOT (`val1 && !val2`).
    Subtraction,
    /// Logical XOR (`val1 != val2`).
    ExclusiveOr,
}

/// Border fill type for `add_specified_border`.
///
/// C equivalent: `L_CONTINUED_BORDER` / `L_MIRRORED_BORDER`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    /// Fill with the nearest edge value (continued).
    Continued,
    /// Fill by mirroring values from the edge inward.
    Mirrored,
}

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

/// Interpolation method for Numa interpolation functions.
///
/// C equivalent: `L_LINEAR_INTERP` / `L_QUADRATIC_INTERP`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationType {
    /// Linear interpolation between adjacent sample points.
    Linear,
    /// Quadratic interpolation using three neighboring sample points.
    Quadratic,
}

/// Comparison type for threshold indicator generation.
///
/// C equivalent: `L_SELECT_IF_LT` / `L_SELECT_IF_GT` /
/// `L_SELECT_IF_LTE` / `L_SELECT_IF_GTE`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdComparison {
    /// Select values less than threshold.
    LessThan,
    /// Select values greater than threshold.
    GreaterThan,
    /// Select values less than or equal to threshold.
    LessThanOrEqual,
    /// Select values greater than or equal to threshold.
    GreaterThanOrEqual,
}

/// Count classification relative to zero.
///
/// C equivalent: `L_LESS_THAN_ZERO` / `L_EQUAL_TO_ZERO` /
/// `L_GREATER_THAN_ZERO`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountRelativeToZero {
    /// Count values less than zero.
    LessThan,
    /// Count values equal to zero.
    EqualTo,
    /// Count values greater than zero.
    GreaterThan,
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

    // ====================================================================
    // Sort index / Sort by index
    // ====================================================================

    /// Return a sorted copy using an automatically selected algorithm.
    ///
    /// For small arrays (n < 200) uses shell sort, for larger arrays
    /// uses Rust's standard sort (TimSort). This matches C Leptonica's
    /// auto-selection between shell sort and bin sort.
    ///
    /// C equivalent: `numaSortAutoSelect(nas, sortorder)`
    pub fn sort_auto_select(&self, order: SortOrder) -> Numa {
        // C version selects shell sort vs bin sort based on size.
        // Rust's standard sort (TimSort) is efficient for all sizes,
        // so we delegate directly.
        self.sorted(order)
    }

    /// Return the permutation indices that would sort the array.
    ///
    /// Automatically selects algorithm based on array size.
    /// The returned Numa contains indices `[i0, i1, ...]` such that
    /// `self[i0] <= self[i1] <= ...` for increasing order.
    ///
    /// C equivalent: `numaSortIndexAutoSelect(nas, sortorder)`
    pub fn sort_index_auto_select(&self, order: SortOrder) -> Numa {
        self.sort_index(order)
    }

    /// Return the permutation indices that would sort the array.
    ///
    /// Uses a stable sort. The returned Numa contains f32 index values.
    ///
    /// C equivalent: `numaGetSortIndex(nas, sortorder)`
    pub fn sort_index(&self, order: SortOrder) -> Numa {
        let n = self.len();
        let array = self.as_slice();

        // Create index array [0, 1, 2, ..., n-1]
        let mut indices: Vec<usize> = (0..n).collect();

        // Stable sort indices by comparing values
        match order {
            SortOrder::Increasing => {
                indices.sort_by(|&a, &b| f32::total_cmp(&array[a], &array[b]));
            }
            SortOrder::Decreasing => {
                indices.sort_by(|&a, &b| f32::total_cmp(&array[b], &array[a]));
            }
        }

        // Convert to Numa of f32
        let mut result = Numa::with_capacity(n);
        for idx in indices {
            result.push(idx as f32);
        }
        result
    }

    /// Reorder elements according to an index array.
    ///
    /// Given an index array `naindex`, produces a new Numa where
    /// `result[i] = self[naindex[i]]`.
    ///
    /// C equivalent: `numaSortByIndex(nas, naindex)`
    pub fn sort_by_index(&self, naindex: &Numa) -> Result<Numa> {
        let n = naindex.len();
        let mut result = Numa::with_capacity(n);
        let array = self.as_slice();
        let self_len = self.len();

        for i in 0..n {
            let idx = naindex[i] as usize;
            if idx >= self_len {
                return Err(Error::IndexOutOfBounds {
                    index: idx,
                    len: self_len,
                });
            }
            result.push(array[idx]);
        }
        Ok(result)
    }

    /// Check if the array is sorted in the given order.
    ///
    /// Returns `true` if the array is sorted (non-strictly) in the
    /// specified order. Empty and single-element arrays are always sorted.
    ///
    /// C equivalent: `numaIsSorted(nas, sortorder, &sorted)`
    pub fn is_sorted(&self, order: SortOrder) -> bool {
        let n = self.len();
        if n <= 1 {
            return true;
        }
        let array = self.as_slice();
        match order {
            SortOrder::Increasing => array.windows(2).all(|w| w[0] <= w[1]),
            SortOrder::Decreasing => array.windows(2).all(|w| w[0] >= w[1]),
        }
    }

    // ====================================================================
    // Interpolation
    // ====================================================================

    /// Interpolate a value from equally-spaced data.
    ///
    /// Uses the Numa's `startx` and `delx` parameters to map the
    /// target `xval` to the array index, then interpolates.
    ///
    /// C equivalent: `numaInterpolateEqxVal(startx, deltax, nay, type, xval, &yval)`
    pub fn interpolate_eqx_val(&self, interp_type: InterpolationType, xval: f32) -> Result<f32> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter(
                "interpolation requires at least 2 data points".to_string(),
            ));
        }
        let (startx, deltax) = self.parameters();
        if deltax <= 0.0 {
            return Err(Error::InvalidParameter("deltax must be > 0".to_string()));
        }

        let array = self.as_slice();
        let xmax = startx + (n - 1) as f32 * deltax;

        if xval < startx || xval > xmax {
            return Err(Error::InvalidParameter(format!(
                "xval {xval} out of range [{startx}, {xmax}]"
            )));
        }

        // Map xval to fractional index
        let findex = (xval - startx) / deltax;
        let i = findex as usize;
        let del = findex - i as f32;

        // Exact match at a knot
        if del.abs() < 1e-7 {
            return Ok(array[i.min(n - 1)]);
        }

        match interp_type {
            InterpolationType::Linear => {
                // Linear interpolation: y = y[i] + del * (y[i+1] - y[i])
                let i1 = (i + 1).min(n - 1);
                Ok(array[i] + del * (array[i1] - array[i]))
            }
            InterpolationType::Quadratic => {
                if n == 2 {
                    // Fall back to linear with only 2 points
                    let i1 = (i + 1).min(n - 1);
                    return Ok(array[i] + del * (array[i1] - array[i]));
                }
                // Select 3 points for quadratic interpolation
                let (p0, p1, p2) = if i == 0 {
                    (0, 1, 2)
                } else if i >= n - 2 {
                    (n - 3, n - 2, n - 1)
                } else {
                    (i - 1, i, i + 1)
                };

                // x-values for the 3 points
                let x0 = startx + p0 as f32 * deltax;
                let x1 = startx + p1 as f32 * deltax;
                let x2 = startx + p2 as f32 * deltax;

                // Lagrangian quadratic interpolation
                let d0 = (x0 - x1) * (x0 - x2);
                let d1 = (x1 - x0) * (x1 - x2);
                let d2 = (x2 - x0) * (x2 - x1);

                let val = array[p0] * (xval - x1) * (xval - x2) / d0
                    + array[p1] * (xval - x0) * (xval - x2) / d1
                    + array[p2] * (xval - x0) * (xval - x1) / d2;
                Ok(val)
            }
        }
    }

    /// Interpolate a value from arbitrarily-spaced (x, y) data.
    ///
    /// `self` contains the x-values (must be monotonically increasing),
    /// `nay` contains the corresponding y-values.
    ///
    /// C equivalent: `numaInterpolateArbxVal(nax, nay, type, xval, &yval)`
    pub fn interpolate_arbx_val(
        &self,
        interp_type: InterpolationType,
        nay: &Numa,
        xval: f32,
    ) -> Result<f32> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter(
                "interpolation requires at least 2 data points".to_string(),
            ));
        }
        if n != nay.len() {
            return Err(Error::InvalidParameter(format!(
                "nax length {} != nay length {}",
                n,
                nay.len()
            )));
        }

        let fax = self.as_slice();
        let fay = nay.as_slice();

        if xval < fax[0] || xval > fax[n - 1] {
            return Err(Error::InvalidParameter(format!(
                "xval {xval} out of range [{}, {}]",
                fax[0],
                fax[n - 1]
            )));
        }

        // Find the interval containing xval via linear search
        // (matching C Leptonica behavior)
        let mut i = 1;
        while i < n && fax[i] < xval {
            i += 1;
        }

        // Exact match
        if (fax[i] - xval).abs() < 1e-7 {
            return Ok(fay[i]);
        }
        // Also check left endpoint
        let im = i - 1;
        if (fax[im] - xval).abs() < 1e-7 {
            return Ok(fay[im]);
        }

        // Fractional position in interval [im, i]
        let fract = (xval - fax[im]) / (fax[i] - fax[im]);

        match interp_type {
            InterpolationType::Linear => Ok(fay[im] + fract * (fay[i] - fay[im])),
            InterpolationType::Quadratic => {
                if n == 2 {
                    return Ok(fay[im] + fract * (fay[i] - fay[im]));
                }
                // Select 3 points for quadratic
                let (p0, p1, p2) = if im == 0 {
                    (0, 1, 2)
                } else {
                    (im - 1, im, im + 1)
                };

                let x0 = fax[p0];
                let x1 = fax[p1];
                let x2 = fax[p2];

                let d0 = (x0 - x1) * (x0 - x2);
                let d1 = (x1 - x0) * (x1 - x2);
                let d2 = (x2 - x0) * (x2 - x1);

                let val = fay[p0] * (xval - x1) * (xval - x2) / d0
                    + fay[p1] * (xval - x0) * (xval - x2) / d1
                    + fay[p2] * (xval - x0) * (xval - x1) / d2;
                Ok(val)
            }
        }
    }

    // ====================================================================
    // Clipping / Indicator / Range / Subsample
    // ====================================================================

    /// Extract a sub-array for indices `[first..=last]`.
    ///
    /// If `last` exceeds the array length, it is clamped to `n-1`.
    /// The result's `startx` is adjusted: `new_startx = startx + first * delx`.
    ///
    /// C equivalent: `numaClipToInterval(nas, first, last)`
    pub fn clip_to_interval(&self, first: usize, last: usize) -> Result<Numa> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }
        if first >= n {
            return Err(Error::InvalidParameter(format!(
                "first index {first} >= array length {n}"
            )));
        }

        let last = last.min(n - 1);
        let array = self.as_slice();
        let mut result = Numa::with_capacity(last - first + 1);
        for &val in &array[first..=last] {
            result.push(val);
        }

        let (startx, delx) = self.parameters();
        result.set_parameters(startx + first as f32 * delx, delx);
        Ok(result)
    }

    /// Generate a binary indicator array based on a threshold.
    ///
    /// For each element, sets the output to 1.0 if the comparison
    /// with `thresh` is satisfied, otherwise 0.0.
    ///
    /// C equivalent: `numaMakeThresholdIndicator(nas, thresh, type)`
    pub fn make_threshold_indicator(&self, thresh: f32, cmp: ThresholdComparison) -> Numa {
        let n = self.len();
        let mut result = Numa::with_capacity(n);
        let array = self.as_slice();

        for &val in array {
            let indicator = match cmp {
                ThresholdComparison::LessThan => val < thresh,
                ThresholdComparison::GreaterThan => val > thresh,
                ThresholdComparison::LessThanOrEqual => val <= thresh,
                ThresholdComparison::GreaterThanOrEqual => val >= thresh,
            };
            result.push(if indicator { 1.0 } else { 0.0 });
        }
        result
    }

    /// Find the range of indices with non-zero values.
    ///
    /// Returns `Ok(Some((first, last)))` where `first` is the first index
    /// and `last` is the last index with `|value| > eps`.
    /// Returns `Ok(None)` if all values are within `eps` of zero.
    ///
    /// C equivalent: `numaGetNonzeroRange(na, eps, &first, &last)`
    pub fn get_nonzero_range(&self, eps: f32) -> Result<Option<(usize, usize)>> {
        let n = self.len();
        if n == 0 {
            return Err(Error::NullInput("empty Numa"));
        }

        let array = self.as_slice();
        let eps = eps.abs();

        // Forward scan for first nonzero
        let first = array.iter().position(|&v| v.abs() > eps);
        let first = match first {
            Some(f) => f,
            None => return Ok(None),
        };

        // Backward scan for last nonzero
        let last = array.iter().rposition(|&v| v.abs() > eps).unwrap();

        Ok(Some((first, last)))
    }

    /// Count elements by their sign relative to zero.
    ///
    /// C equivalent: `numaGetCountRelativeToZero(na, type, &count)`
    pub fn get_count_relative_to_zero(&self, rel: CountRelativeToZero) -> Result<usize> {
        if self.is_empty() {
            return Err(Error::NullInput("empty Numa"));
        }

        let count = self
            .as_slice()
            .iter()
            .filter(|&&v| match rel {
                CountRelativeToZero::LessThan => v < 0.0,
                CountRelativeToZero::EqualTo => v == 0.0,
                CountRelativeToZero::GreaterThan => v > 0.0,
            })
            .count();
        Ok(count)
    }

    /// Subsample the array, taking every `subfactor`-th element.
    ///
    /// Returns elements at indices `0, subfactor, 2*subfactor, ...`.
    ///
    /// C equivalent: `numaSubsample(nas, subfactor)`
    pub fn subsample(&self, subfactor: usize) -> Result<Numa> {
        if subfactor == 0 {
            return Err(Error::InvalidParameter(
                "subfactor must be >= 1".to_string(),
            ));
        }

        let array = self.as_slice();
        let mut result = Numa::with_capacity(self.len().div_ceil(subfactor));
        for (i, &val) in array.iter().enumerate() {
            if i % subfactor == 0 {
                result.push(val);
            }
        }
        Ok(result)
    }

    // ====================================================================
    // Arithmetic and logical operations
    // ====================================================================

    /// Apply an element-wise arithmetic operation in-place.
    ///
    /// Requires both arrays to have the same length.
    /// For `Divide`, all elements of `other` must be nonzero.
    ///
    /// C equivalent: `numaArithOp()` in `numafunc1.c`
    pub fn arith_op(&mut self, op: ArithOp, other: &Numa) -> Result<()> {
        let n = self.len();
        if n != other.len() {
            return Err(Error::InvalidParameter(format!(
                "length mismatch: {} vs {}",
                n,
                other.len()
            )));
        }
        if op == ArithOp::Divide {
            for i in 0..n {
                if other.get(i).unwrap() == 0.0 {
                    return Err(Error::InvalidParameter(format!(
                        "other[{i}] is zero in Divide"
                    )));
                }
            }
        }
        let data = self.as_slice_mut();
        for i in 0..n {
            let v2 = other.get(i).unwrap();
            match op {
                ArithOp::Add => data[i] += v2,
                ArithOp::Subtract => data[i] -= v2,
                ArithOp::Multiply => data[i] *= v2,
                ArithOp::Divide => data[i] /= v2,
            }
        }
        Ok(())
    }

    /// Apply an element-wise logical operation in-place.
    ///
    /// Treats non-zero as `true` and zero as `false`.
    /// Requires both arrays to have the same length.
    ///
    /// C equivalent: `numaLogicalOp()` in `numafunc1.c`
    pub fn logical_op(&mut self, op: LogicalOp, other: &Numa) -> Result<()> {
        let n = self.len();
        if n != other.len() {
            return Err(Error::InvalidParameter(format!(
                "length mismatch: {} vs {}",
                n,
                other.len()
            )));
        }
        let data = self.as_slice_mut();
        for i in 0..n {
            let v1 = data[i] != 0.0;
            let v2 = other.get(i).unwrap() != 0.0;
            data[i] = match op {
                LogicalOp::Union => (v1 || v2) as u8 as f32,
                LogicalOp::Intersection => (v1 && v2) as u8 as f32,
                LogicalOp::Subtraction => (v1 && !v2) as u8 as f32,
                LogicalOp::ExclusiveOr => (v1 != v2) as u8 as f32,
            };
        }
        Ok(())
    }

    /// Invert indicator array values in-place (0 → 1, nonzero → 0).
    ///
    /// C equivalent: `numaInvert()` in `numafunc1.c`
    pub fn invert(&mut self) {
        for v in self.as_slice_mut() {
            *v = if *v == 0.0 { 1.0 } else { 0.0 };
        }
    }

    /// Add `val` to the element at `index`.
    ///
    /// C equivalent: `numaAddToNumber()` in `numafunc1.c`
    pub fn add_to_element(&mut self, index: usize, val: f32) -> Result<()> {
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("Numa is empty".into()));
        }
        if index >= n {
            return Err(Error::IndexOutOfBounds { index, len: n });
        }
        self.as_slice_mut()[index] += val;
        Ok(())
    }

    // ====================================================================
    // Delta / absval
    // ====================================================================

    /// Return a new Numa containing the first-difference of `self`.
    ///
    /// Output length is `self.len() - 1`. Returns an empty Numa if
    /// `self.len() < 2`.
    ///
    /// C equivalent: `numaMakeDelta()` in `numafunc1.c`
    pub fn make_delta(&self) -> Numa {
        let n = self.len();
        if n < 2 {
            return Numa::new();
        }
        let mut nad = Numa::with_capacity(n - 1);
        for i in 1..n {
            nad.push(self.get(i).unwrap() - self.get(i - 1).unwrap());
        }
        nad
    }

    /// Replace each element with its absolute value in-place.
    ///
    /// C equivalent: `numaMakeAbsval()` in `numafunc1.c`
    pub fn abs_val(&mut self) {
        for v in self.as_slice_mut() {
            *v = v.abs();
        }
    }

    // ====================================================================
    // Border add / remove
    // ====================================================================

    /// Return a new Numa with `left` and `right` border elements prepended
    /// and appended, each initialised to `val`.
    ///
    /// The x-parameters are adjusted so that the sequence continues
    /// smoothly outside the original range.
    ///
    /// C equivalent: `numaAddBorder()` in `numafunc1.c`
    pub fn add_border(&self, left: usize, right: usize, val: f32) -> Numa {
        let left = left;
        let right = right;
        if left == 0 && right == 0 {
            return self.clone();
        }
        let n = self.len();
        let total = n + left + right;
        let (startx, delx) = self.parameters();
        let mut nad = Numa::with_capacity(total);
        for _ in 0..left {
            nad.push(val);
        }
        for v in self.iter() {
            nad.push(v);
        }
        for _ in 0..right {
            nad.push(val);
        }
        nad.set_parameters(startx - delx * left as f32, delx);
        nad
    }

    /// Return a new Numa with `left` and `right` border elements added
    /// using the specified border fill strategy.
    ///
    /// C equivalent: `numaAddSpecifiedBorder()` in `numafunc1.c`
    pub fn add_specified_border(
        &self,
        left: usize,
        right: usize,
        border_type: BorderType,
    ) -> Result<Numa> {
        let n = self.len();
        if left == 0 && right == 0 {
            return Ok(self.clone());
        }
        if border_type == BorderType::Mirrored && (left > n || right > n) {
            return Err(Error::InvalidParameter(
                "border too large for mirrored".into(),
            ));
        }
        let mut nad = self.add_border(left, right, 0.0);
        let total = nad.len();
        let data = nad.as_slice_mut();
        match border_type {
            BorderType::Continued => {
                let edge_left = if n > 0 { data[left] } else { 0.0 };
                let edge_right = if n > 0 { data[total - right - 1] } else { 0.0 };
                for i in 0..left {
                    data[i] = edge_left;
                }
                for i in (total - right)..total {
                    data[i] = edge_right;
                }
            }
            BorderType::Mirrored => {
                for i in 0..left {
                    data[i] = data[2 * left - 1 - i];
                }
                for i in 0..right {
                    data[total - right + i] = data[total - right - i - 1];
                }
            }
        }
        Ok(nad)
    }

    /// Return a new Numa with `left` elements removed from the start and
    /// `right` elements removed from the end.
    ///
    /// C equivalent: `numaRemoveBorder()` in `numafunc1.c`
    pub fn remove_border(&self, left: usize, right: usize) -> Result<Numa> {
        let n = self.len();
        if left == 0 && right == 0 {
            return Ok(self.clone());
        }
        let combined = left + right;
        if combined > n {
            return Err(Error::InvalidParameter(format!(
                "border ({left}+{right}={combined}) exceeds length {n}"
            )));
        }
        let len = n - combined;
        let (startx, delx) = self.parameters();
        let src = self.as_slice();
        let mut nad = Numa::with_capacity(len);
        for i in 0..len {
            nad.push(src[left + i]);
        }
        nad.set_parameters(startx + delx * left as f32, delx);
        Ok(nad)
    }

    // ====================================================================
    // Run counting
    // ====================================================================

    /// Count the number of contiguous nonzero runs.
    ///
    /// C equivalent: `numaCountNonzeroRuns()` in `numafunc1.c`
    pub fn count_nonzero_runs(&self) -> Result<usize> {
        let n = self.len();
        if n == 0 {
            return Err(Error::InvalidParameter("Numa is empty".into()));
        }
        let mut count = 0usize;
        let mut in_run = false;
        for i in 0..n {
            let val = self.get_i32(i).unwrap();
            if !in_run && val > 0 {
                count += 1;
                in_run = true;
            } else if in_run && val == 0 {
                in_run = false;
            }
        }
        Ok(count)
    }
}

impl Numaa {
    /// Append Numa arrays from `other` in the range `[istart, iend]`.
    ///
    /// `iend = None` means append through the last element.
    ///
    /// C equivalent: `numaaJoin()` in `numafunc1.c`
    pub fn join_range(&mut self, other: &Numaa, istart: usize, iend: Option<usize>) {
        let n = other.len();
        if n == 0 {
            return;
        }
        let end = iend.unwrap_or(n - 1).min(n - 1);
        if istart > end {
            return;
        }
        for i in istart..=end {
            self.push(other.get(i).unwrap().clone());
        }
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

    // -- Numa::arith_op --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_add() {
        let mut na1 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        let na2 = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        na1.arith_op(ArithOp::Add, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[11.0, 22.0, 33.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_subtract() {
        let mut na1 = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        let na2 = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        na1.arith_op(ArithOp::Subtract, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[9.0, 18.0, 27.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_multiply() {
        let mut na1 = Numa::from_vec(vec![2.0, 3.0, 4.0]);
        let na2 = Numa::from_vec(vec![5.0, 6.0, 7.0]);
        na1.arith_op(ArithOp::Multiply, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[10.0, 18.0, 28.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_divide() {
        let mut na1 = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        let na2 = Numa::from_vec(vec![2.0, 4.0, 5.0]);
        na1.arith_op(ArithOp::Divide, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[5.0, 5.0, 6.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_divide_by_zero() {
        let mut na1 = Numa::from_vec(vec![1.0, 2.0]);
        let na2 = Numa::from_vec(vec![1.0, 0.0]);
        assert!(na1.arith_op(ArithOp::Divide, &na2).is_err());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_arith_op_length_mismatch() {
        let mut na1 = Numa::from_vec(vec![1.0, 2.0]);
        let na2 = Numa::from_vec(vec![1.0]);
        assert!(na1.arith_op(ArithOp::Add, &na2).is_err());
    }

    // -- Numa::logical_op --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_logical_op_union() {
        let mut na1 = Numa::from_vec(vec![0.0, 1.0, 0.0, 1.0]);
        let na2 = Numa::from_vec(vec![0.0, 0.0, 1.0, 1.0]);
        na1.logical_op(LogicalOp::Union, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[0.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_logical_op_intersection() {
        let mut na1 = Numa::from_vec(vec![0.0, 1.0, 0.0, 1.0]);
        let na2 = Numa::from_vec(vec![0.0, 0.0, 1.0, 1.0]);
        na1.logical_op(LogicalOp::Intersection, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_logical_op_subtraction() {
        let mut na1 = Numa::from_vec(vec![0.0, 1.0, 0.0, 1.0]);
        let na2 = Numa::from_vec(vec![0.0, 0.0, 1.0, 1.0]);
        na1.logical_op(LogicalOp::Subtraction, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_logical_op_xor() {
        let mut na1 = Numa::from_vec(vec![0.0, 1.0, 0.0, 1.0]);
        let na2 = Numa::from_vec(vec![0.0, 0.0, 1.0, 1.0]);
        na1.logical_op(LogicalOp::ExclusiveOr, &na2).unwrap();
        assert_eq!(na1.as_slice(), &[0.0, 1.0, 1.0, 0.0]);
    }

    // -- Numa::invert --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_invert() {
        let mut na = Numa::from_vec(vec![0.0, 1.0, 0.0, 5.0]);
        na.invert();
        assert_eq!(na.as_slice(), &[1.0, 0.0, 1.0, 0.0]);
    }

    // -- Numa::add_to_element --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_to_element() {
        let mut na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        na.add_to_element(1, 10.0).unwrap();
        assert_eq!(na.as_slice(), &[1.0, 12.0, 3.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_to_element_out_of_bounds() {
        let mut na = Numa::from_vec(vec![1.0, 2.0]);
        assert!(na.add_to_element(5, 1.0).is_err());
    }

    // -- Numa::make_delta --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_make_delta() {
        let na = Numa::from_vec(vec![1.0, 3.0, 6.0, 10.0]);
        let d = na.make_delta();
        assert_eq!(d.as_slice(), &[2.0, 3.0, 4.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_make_delta_too_short() {
        let na = Numa::from_vec(vec![5.0]);
        let d = na.make_delta();
        assert!(d.is_empty());
    }

    // -- Numa::abs_val --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_abs_val() {
        let mut na = Numa::from_vec(vec![-3.0, 0.0, 2.0, -5.0]);
        na.abs_val();
        assert_eq!(na.as_slice(), &[3.0, 0.0, 2.0, 5.0]);
    }

    // -- Numa::add_border --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_border_basic() {
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        let bordered = na.add_border(2, 1, 0.0);
        assert_eq!(bordered.as_slice(), &[0.0, 0.0, 1.0, 2.0, 3.0, 0.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_border_no_border() {
        let na = Numa::from_vec(vec![1.0, 2.0]);
        let result = na.add_border(0, 0, 99.0);
        assert_eq!(result.as_slice(), na.as_slice());
    }

    // -- Numa::add_specified_border --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_specified_border_continued() {
        let na = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        let bordered = na
            .add_specified_border(2, 2, BorderType::Continued)
            .unwrap();
        // left 2 filled with 10, right 2 filled with 30
        assert_eq!(
            bordered.as_slice(),
            &[10.0, 10.0, 10.0, 20.0, 30.0, 30.0, 30.0]
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_specified_border_mirrored() {
        let na = Numa::from_vec(vec![10.0, 20.0, 30.0]);
        let bordered = na.add_specified_border(2, 2, BorderType::Mirrored).unwrap();
        // left 2: mirror of [10,20] → [20,10]; right 2: mirror of [20,30] → [30,20]
        assert_eq!(
            bordered.as_slice(),
            &[20.0, 10.0, 10.0, 20.0, 30.0, 30.0, 20.0]
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_specified_border_mirrored_too_large() {
        let na = Numa::from_vec(vec![1.0, 2.0]);
        assert!(na.add_specified_border(3, 0, BorderType::Mirrored).is_err());
    }

    // -- Numa::remove_border --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_border_basic() {
        let na = Numa::from_vec(vec![0.0, 0.0, 1.0, 2.0, 3.0, 0.0]);
        let removed = na.remove_border(2, 1).unwrap();
        assert_eq!(removed.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_border_too_large() {
        let na = Numa::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(na.remove_border(2, 2).is_err());
    }

    // -- Numa::count_nonzero_runs --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_count_nonzero_runs_basic() {
        let na = Numa::from_vec(vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 4.0, 5.0]);
        assert_eq!(na.count_nonzero_runs().unwrap(), 3);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_count_nonzero_runs_all_zero() {
        let na = Numa::from_vec(vec![0.0, 0.0, 0.0]);
        assert_eq!(na.count_nonzero_runs().unwrap(), 0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_count_nonzero_runs_empty() {
        let na = Numa::new();
        assert!(na.count_nonzero_runs().is_err());
    }

    // -- Numaa::join_range --

    #[test]
    #[ignore = "not yet implemented"]
    fn test_numaa_join_range_all() {
        let mut naad = Numaa::new();
        naad.push(Numa::from_vec(vec![1.0]));
        let mut naas = Numaa::new();
        naas.push(Numa::from_vec(vec![2.0]));
        naas.push(Numa::from_vec(vec![3.0]));
        naad.join_range(&naas, 0, None);
        assert_eq!(naad.len(), 3);
        assert_eq!(naad.get(1).unwrap().as_slice(), &[2.0]);
        assert_eq!(naad.get(2).unwrap().as_slice(), &[3.0]);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_numaa_join_range_partial() {
        let mut naad = Numaa::new();
        let mut naas = Numaa::new();
        naas.push(Numa::from_vec(vec![10.0]));
        naas.push(Numa::from_vec(vec![20.0]));
        naas.push(Numa::from_vec(vec![30.0]));
        naad.join_range(&naas, 1, Some(2));
        assert_eq!(naad.len(), 2);
        assert_eq!(naad.get(0).unwrap().as_slice(), &[20.0]);
        assert_eq!(naad.get(1).unwrap().as_slice(), &[30.0]);
    }
}
