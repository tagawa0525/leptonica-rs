//! Histogram statistics for Numa
//!
//! Functions for computing statistics from histograms and
//! converting between values and cumulative ranks.
//!
//! # See also
//!
//! C Leptonica: `numafunc1.c` (histogram statistics functions)

use super::Numa;

/// Statistics computed from a histogram
///
/// All values are in the x-domain of the histogram, not bin indices.
/// The x-value for bin i is: `startx + i * deltax`
///
/// # See also
///
/// C Leptonica: `numaHistogramGetStats()`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

impl Numa {
    /// Get statistical measures from a histogram
    ///
    /// # See also
    ///
    /// C Leptonica: `numaHistogramGetStats()`
    pub fn histogram_stats(&self, startx: f32, deltax: f32) -> Option<HistogramStats> {
        todo!()
    }

    /// Get histogram statistics on a specific interval
    ///
    /// # See also
    ///
    /// C Leptonica: `numaHistogramGetStatsOnInterval()`
    pub fn histogram_stats_on_interval(
        &self,
        startx: f32,
        deltax: f32,
        ifirst: usize,
        ilast: Option<usize>,
    ) -> Option<HistogramStats> {
        todo!()
    }

    /// Get the rank (cumulative fraction) for a given value
    ///
    /// # See also
    ///
    /// C Leptonica: `numaHistogramGetRankFromVal()`
    pub fn histogram_rank_from_val(&self, rval: f32) -> Option<f32> {
        todo!()
    }

    /// Get the value corresponding to a given rank (cumulative fraction)
    ///
    /// # See also
    ///
    /// C Leptonica: `numaHistogramGetValFromRank()`
    pub fn histogram_val_from_rank(&self, rank: f32) -> Option<f32> {
        todo!()
    }

    /// Normalize the histogram so that the sum equals 1.0
    pub fn normalize_histogram(&self) -> Option<Numa> {
        todo!()
    }

    /// Compute the cumulative distribution function (CDF)
    pub fn cumulative_distribution(&self) -> Option<Numa> {
        todo!()
    }
}
