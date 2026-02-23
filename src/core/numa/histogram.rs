//! Histogram statistics for Numa
//!
//! Functions for computing statistics from histograms and
//! converting between values and cumulative ranks.

use super::Numa;

/// Statistics computed from a histogram
///
/// All values are in the x-domain of the histogram, not bin indices.
/// The x-value for bin i is: `startx + i * deltax`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramStats {
    /// Mean value (weighted average)
    pub mean: f32,
    /// Median value (50th percentile)
    pub median: f32,
    /// Mode value (most frequent)
    pub mode: f32,
    /// Variance
    pub variance: f32,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            mode: 0.0,
            variance: 0.0,
        }
    }
}

impl Numa {
    /// Get statistical measures from a histogram
    ///
    /// This function computes the mean, median, mode, and variance of
    /// the distribution represented by the histogram.
    ///
    /// # Arguments
    ///
    /// * `startx` - The x-value corresponding to bin 0
    /// * `deltax` - The spacing between consecutive x-values (bin width)
    ///
    /// # Returns
    ///
    /// `Some(HistogramStats)` if the histogram has non-zero total,
    /// `None` if the histogram is empty or has zero sum.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// // Create a simple histogram: values 0, 1, 2, 3 with counts 1, 2, 3, 4
    /// let hist = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
    /// let stats = hist.histogram_stats(0.0, 1.0).unwrap();
    ///
    /// // Mean = (0*1 + 1*2 + 2*3 + 3*4) / (1+2+3+4) = 20/10 = 2.0
    /// assert!((stats.mean - 2.0).abs() < 0.001);
    /// ```
    pub fn histogram_stats(&self, startx: f32, deltax: f32) -> Option<HistogramStats> {
        self.histogram_stats_on_interval(startx, deltax, 0, None)
    }

    /// Get histogram statistics on a specific interval
    ///
    /// Same as `histogram_stats` but only considers bins in the range
    /// `[ifirst, ilast]` inclusive.
    ///
    /// # Arguments
    ///
    /// * `startx` - The x-value corresponding to bin 0
    /// * `deltax` - The spacing between consecutive x-values
    /// * `ifirst` - First bin index to include
    /// * `ilast` - Last bin index to include, or `None` for all remaining bins
    ///
    /// # Returns
    ///
    /// `Some(HistogramStats)` if the interval has non-zero total,
    /// `None` if the histogram is empty, interval is invalid, or sum is zero.
    pub fn histogram_stats_on_interval(
        &self,
        startx: f32,
        deltax: f32,
        ifirst: usize,
        ilast: Option<usize>,
    ) -> Option<HistogramStats> {
        let n = self.len();
        if n == 0 || ifirst >= n {
            return None;
        }

        let ilast = ilast.map(|i| i.min(n - 1)).unwrap_or(n - 1);
        if ifirst > ilast {
            return None;
        }

        // Compute sum, moment, and second moment
        let mut sum = 0.0f32;
        let mut moment = 0.0f32;
        let mut moment2 = 0.0f32;

        for i in ifirst..=ilast {
            let x = startx + (i as f32) * deltax;
            let y = self.get(i).unwrap_or(0.0);
            sum += y;
            moment += x * y;
            moment2 += x * x * y;
        }

        if sum == 0.0 {
            return None;
        }

        let mean = moment / sum;
        let variance = moment2 / sum - mean * mean;

        // Compute median (value where cumulative sum reaches 50%)
        let half_sum = sum / 2.0;
        let mut cumsum = 0.0f32;
        let mut median = startx;
        for i in ifirst..=ilast {
            let y = self.get(i).unwrap_or(0.0);
            cumsum += y;
            if cumsum >= half_sum {
                median = startx + (i as f32) * deltax;
                break;
            }
        }

        // Compute mode (x-value with maximum count)
        let mut mode_idx = ifirst;
        let mut max_count = f32::NEG_INFINITY;
        for i in ifirst..=ilast {
            let y = self.get(i).unwrap_or(0.0);
            if y > max_count {
                max_count = y;
                mode_idx = i;
            }
        }
        let mode = startx + (mode_idx as f32) * deltax;

        Some(HistogramStats {
            mean,
            median,
            mode,
            variance,
        })
    }

    /// Get the rank (cumulative fraction) for a given value
    ///
    /// For a histogram representing y(x), this computes the integral of y
    /// from the start value to `rval`, normalized by the total sum.
    ///
    /// The rank represents the fraction of the distribution that falls
    /// below the given value.
    ///
    /// # Arguments
    ///
    /// * `rval` - The value for which to compute the rank
    ///
    /// # Returns
    ///
    /// `Some(rank)` where rank is in [0.0, 1.0], or `None` if the histogram
    /// is empty or has zero sum.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// // Uniform histogram: each bin has count 1
    /// let mut hist = Numa::from_vec(vec![1.0; 10]);
    /// hist.set_parameters(0.0, 1.0);
    ///
    /// // At x=5, we've covered 5 out of 10 bins
    /// let rank = hist.histogram_rank_from_val(5.0).unwrap();
    /// assert!((rank - 0.5).abs() < 0.001);
    /// ```
    pub fn histogram_rank_from_val(&self, rval: f32) -> Option<f32> {
        let n = self.len();
        if n == 0 {
            return None;
        }

        let (startx, deltax) = self.parameters();

        // Value below start
        if rval < startx {
            return Some(0.0);
        }

        // Value above end
        let maxval = startx + (n as f32) * deltax;
        if rval > maxval {
            return Some(1.0);
        }

        // Find the bin and fractional position
        let binval = (rval - startx) / deltax;
        let ibin = binval as usize;

        if ibin >= n {
            return Some(1.0);
        }

        let fract = binval - (ibin as f32);

        // Sum up to the bin
        let mut sum = 0.0f32;
        for i in 0..ibin {
            sum += self.get(i).unwrap_or(0.0);
        }

        // Add fractional part of current bin
        sum += fract * self.get(ibin).unwrap_or(0.0);

        // Normalize by total
        let total = self.sum()?;
        if total == 0.0 {
            return None;
        }

        Some(sum / total)
    }

    /// Get the value corresponding to a given rank (cumulative fraction)
    ///
    /// Returns the x-value such that the cumulative distribution function
    /// equals the given rank.
    ///
    /// # Arguments
    ///
    /// * `rank` - The target rank in [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// `Some(value)` where value is the x-coordinate, or `None` if the
    /// histogram is empty or has zero sum.
    ///
    /// # Example
    ///
    /// ```
    /// use leptonica_core::Numa;
    ///
    /// // Uniform histogram
    /// let mut hist = Numa::from_vec(vec![1.0; 10]);
    /// hist.set_parameters(0.0, 1.0);
    ///
    /// // Rank 0.5 should give approximately x=5
    /// let val = hist.histogram_val_from_rank(0.5).unwrap();
    /// assert!((val - 5.0).abs() < 0.5);
    /// ```
    pub fn histogram_val_from_rank(&self, rank: f32) -> Option<f32> {
        let n = self.len();
        if n == 0 {
            return None;
        }

        // Clamp rank to valid range
        let rank = rank.clamp(0.0, 1.0);

        let (startx, deltax) = self.parameters();
        let total = self.sum()?;

        if total == 0.0 {
            return None;
        }

        let target_count = rank * total;
        let mut sum = 0.0f32;

        for i in 0..n {
            let val = self.get(i).unwrap_or(0.0);
            if sum + val >= target_count {
                // Linear interpolation within the bin
                let fract = if val > 0.0 {
                    (target_count - sum) / val
                } else {
                    0.0
                };
                return Some(startx + deltax * ((i as f32) + fract));
            }
            sum += val;
        }

        // If we get here, return the end value
        Some(startx + deltax * (n as f32))
    }

    /// Normalize the histogram so that the sum equals 1.0
    ///
    /// Returns a new Numa with the same shape but values scaled
    /// so they sum to 1.0.
    ///
    /// # Returns
    ///
    /// `Some(normalized_histogram)` or `None` if the histogram has zero sum.
    pub fn normalize_histogram(&self) -> Option<Numa> {
        let total = self.sum()?;
        if total == 0.0 {
            return None;
        }

        let scale = 1.0 / total;
        let data: Vec<f32> = self.iter().map(|v| v * scale).collect();
        let mut result = Numa::from_vec(data);
        let (startx, deltax) = self.parameters();
        result.set_parameters(startx, deltax);
        Some(result)
    }

    /// Compute the cumulative distribution function (CDF)
    ///
    /// Returns a new Numa where each value is the sum of all values
    /// up to and including that bin, normalized by the total.
    ///
    /// # Returns
    ///
    /// `Some(cdf)` or `None` if the histogram has zero sum.
    pub fn cumulative_distribution(&self) -> Option<Numa> {
        let total = self.sum()?;
        if total == 0.0 {
            return None;
        }

        let mut cumsum = 0.0f32;
        let data: Vec<f32> = self
            .iter()
            .map(|v| {
                cumsum += v;
                cumsum / total
            })
            .collect();

        let mut result = Numa::from_vec(data);
        let (startx, deltax) = self.parameters();
        result.set_parameters(startx, deltax);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_stats_basic() {
        // Uniform distribution: [1, 1, 1, 1]
        let hist = Numa::from_vec(vec![1.0, 1.0, 1.0, 1.0]);
        let stats = hist.histogram_stats(0.0, 1.0).unwrap();

        // Mean = (0+1+2+3)/4 = 1.5
        assert!((stats.mean - 1.5).abs() < 0.001);
        // Median should be around 1.5
        assert!((stats.median - 2.0).abs() <= 1.0);
        // Mode: all equal, first one (0) should be returned
        assert!((stats.mode - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_stats_weighted() {
        // Weighted: bins 0,1,2,3 with counts 1,2,3,4
        let hist = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let stats = hist.histogram_stats(0.0, 1.0).unwrap();

        // Mean = (0*1 + 1*2 + 2*3 + 3*4) / 10 = 20/10 = 2.0
        assert!((stats.mean - 2.0).abs() < 0.001);
        // Mode = 3 (highest count = 4)
        assert!((stats.mode - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_stats_with_offset() {
        // Same histogram but with startx=10, deltax=2
        let hist = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let stats = hist.histogram_stats(10.0, 2.0).unwrap();

        // x values are: 10, 12, 14, 16
        // Mean = (10*1 + 12*2 + 14*3 + 16*4) / 10 = 140/10 = 14.0
        assert!((stats.mean - 14.0).abs() < 0.001);
        // Mode = 16 (bin 3)
        assert!((stats.mode - 16.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_stats_on_interval() {
        let hist = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = hist
            .histogram_stats_on_interval(0.0, 1.0, 1, Some(3))
            .unwrap();

        // Only bins 1,2,3 with counts 2,3,4
        // Mean = (1*2 + 2*3 + 3*4) / 9 = 20/9 = 2.222...
        assert!((stats.mean - 20.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_stats_empty() {
        let hist = Numa::new();
        assert!(hist.histogram_stats(0.0, 1.0).is_none());

        let zero_hist = Numa::from_vec(vec![0.0, 0.0, 0.0]);
        assert!(zero_hist.histogram_stats(0.0, 1.0).is_none());
    }

    #[test]
    fn test_rank_from_val_uniform() {
        let mut hist = Numa::from_vec(vec![1.0; 10]);
        hist.set_parameters(0.0, 1.0);

        // At start
        let rank = hist.histogram_rank_from_val(0.0).unwrap();
        assert!(rank.abs() < 0.001);

        // At middle
        let rank = hist.histogram_rank_from_val(5.0).unwrap();
        assert!((rank - 0.5).abs() < 0.001);

        // At end
        let rank = hist.histogram_rank_from_val(10.0).unwrap();
        assert!((rank - 1.0).abs() < 0.001);

        // Below start
        let rank = hist.histogram_rank_from_val(-1.0).unwrap();
        assert!(rank.abs() < 0.001);

        // Above end
        let rank = hist.histogram_rank_from_val(15.0).unwrap();
        assert!((rank - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_val_from_rank_uniform() {
        let mut hist = Numa::from_vec(vec![1.0; 10]);
        hist.set_parameters(0.0, 1.0);

        // Rank 0 -> start
        let val = hist.histogram_val_from_rank(0.0).unwrap();
        assert!(val.abs() < 0.001);

        // Rank 0.5 -> middle
        let val = hist.histogram_val_from_rank(0.5).unwrap();
        assert!((val - 5.0).abs() < 0.5);

        // Rank 1.0 -> end
        let val = hist.histogram_val_from_rank(1.0).unwrap();
        assert!((val - 10.0).abs() < 0.5);
    }

    #[test]
    fn test_rank_val_roundtrip() {
        let mut hist = Numa::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        hist.set_parameters(0.0, 1.0);

        // Test roundtrip for several values
        for target_rank in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let val = hist.histogram_val_from_rank(target_rank).unwrap();
            let rank = hist.histogram_rank_from_val(val).unwrap();
            assert!(
                (rank - target_rank).abs() < 0.05,
                "Roundtrip failed: {} -> {} -> {}",
                target_rank,
                val,
                rank
            );
        }
    }

    #[test]
    fn test_normalize_histogram() {
        let hist = Numa::from_vec(vec![2.0, 4.0, 6.0, 8.0]);
        let normalized = hist.normalize_histogram().unwrap();

        // Total should be 1.0
        let total = normalized.sum().unwrap();
        assert!((total - 1.0).abs() < 0.001);

        // Ratios should be preserved
        assert!((normalized[0] * 20.0 - 2.0).abs() < 0.001);
        assert!((normalized[1] * 20.0 - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_cumulative_distribution() {
        let hist = Numa::from_vec(vec![1.0, 1.0, 1.0, 1.0]);
        let cdf = hist.cumulative_distribution().unwrap();

        assert!((cdf[0] - 0.25).abs() < 0.001);
        assert!((cdf[1] - 0.50).abs() < 0.001);
        assert!((cdf[2] - 0.75).abs() < 0.001);
        assert!((cdf[3] - 1.00).abs() < 0.001);
    }

    #[test]
    fn test_variance() {
        // Single value -> variance = 0
        let hist = Numa::from_vec(vec![0.0, 0.0, 10.0, 0.0, 0.0]);
        let stats = hist.histogram_stats(0.0, 1.0).unwrap();
        assert!((stats.variance).abs() < 0.001);

        // Two equal peaks at 0 and 4 -> variance = 4
        let hist = Numa::from_vec(vec![1.0, 0.0, 0.0, 0.0, 1.0]);
        let stats = hist.histogram_stats(0.0, 1.0).unwrap();
        // Mean = 2, Variance = ((0-2)^2 + (4-2)^2) / 2 = 4
        assert!((stats.variance - 4.0).abs() < 0.001);
    }
}
