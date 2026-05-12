//! Advanced Numa helpers — RED stubs (plan 109 / C numafunc2.c).

use crate::core::error::Result;
use crate::core::numa::Numa;

impl Numa {
    /// C: `numaCountReversals`.
    pub fn count_reversals(&self, _min_reversal: f32) -> Result<(u32, f32)> {
        unimplemented!("plan 109 RED stub")
    }

    /// C: `numaFindPeaks`.
    pub fn find_peaks(&self, _nmax: u32, _fract1: f32, _fract2: f32) -> Numa {
        unimplemented!("plan 109 RED stub")
    }
}

/// C: `numaCrossingsByThreshold`.
pub fn numa_crossings_by_threshold(_nay: &Numa, _nax: Option<&Numa>, _thresh: f32) -> Result<Numa> {
    unimplemented!("plan 109 RED stub")
}

/// C: `numaGetUniformBinSizes`.
pub fn numa_uniform_bin_sizes(_ntotal: i32, _nbins: i32) -> Result<Numa> {
    unimplemented!("plan 109 RED stub")
}

/// C: `genConstrainedNumaInRange`.
pub fn gen_constrained_numa_in_range(
    _first: i32,
    _last: i32,
    _nmax: i32,
    _use_pairs: bool,
) -> Result<Numa> {
    unimplemented!("plan 109 RED stub")
}
