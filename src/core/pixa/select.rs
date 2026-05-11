//! Pixa selection helpers — RED stubs (plan 106 / C pixafunc1.c).
//!
//! All functions here are placeholders; they panic if called. The companion
//! tests in `tests/core/pixa_select_reg.rs` are gated with
//! `#[ignore = "not yet implemented"]` and will be enabled by the GREEN commit.

use crate::core::error::Result;
use crate::core::{Pix, PixMut};

use super::Pixa;

/// Threshold relation used by the metric-based `select_by_*` helpers.
///
/// C Leptonica constants: `L_SELECT_IF_LT`, `L_SELECT_IF_GT`,
/// `L_SELECT_IF_LTE`, `L_SELECT_IF_GTE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdSelect {
    /// Keep when `metric < thresh`.
    LessThan,
    /// Keep when `metric > thresh`.
    GreaterThan,
    /// Keep when `metric <= thresh`.
    LessOrEqual,
    /// Keep when `metric >= thresh`.
    GreaterOrEqual,
}

impl Pixa {
    /// C: `pixaSelectRange`.
    pub fn select_range(&self, _first: usize, _last: Option<usize>) -> Self {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectWithIndicator`.
    pub fn select_with_indicator(&self, _indicator: &[bool]) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectWithString`.
    pub fn select_with_string(&self, _s: &str) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectByNumConnComp`.
    pub fn select_by_num_conn_comp(
        &self,
        _nmin: u32,
        _nmax: u32,
        _connectivity: crate::region::ConnectivityType,
    ) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectByAreaFraction`.
    pub fn select_by_area_fraction(
        &self,
        _thresh: f32,
        _sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectByPerimSizeRatio`.
    pub fn select_by_perim_size_ratio(
        &self,
        _thresh: f32,
        _sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectByPerimToAreaRatio`.
    pub fn select_by_perim_to_area_ratio(
        &self,
        _thresh: f32,
        _sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }

    /// C: `pixaSelectByWidthHeightRatio`.
    pub fn select_by_width_height_ratio(
        &self,
        _thresh: f32,
        _sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        unimplemented!("plan 106 RED stub")
    }
}

/// C: `pixaFindAreaFraction`.
pub fn pixa_find_area_fraction(_pixa: &Pixa) -> Vec<f32> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixaFindPerimSizeRatio`.
pub fn pixa_find_perim_size_ratio(_pixa: &Pixa) -> Vec<f32> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixaFindPerimToAreaRatio`.
pub fn pixa_find_perim_to_area_ratio(_pixa: &Pixa) -> Vec<f32> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixaFindWidthHeightRatio`.
pub fn pixa_find_width_height_ratio(_pixa: &Pixa) -> Vec<f32> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixAddWithIndicator`.
pub fn pix_add_with_indicator(_pixs: &Pix, _pixad: &mut PixMut, _indicator: &[bool]) -> Result<()> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixRemoveWithIndicator`.
pub fn pix_remove_with_indicator(
    _pixs: &Pix,
    _pixad: &mut PixMut,
    _indicator: &[bool],
) -> Result<()> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixSelectByAreaFraction`.
pub fn pix_select_by_area_fraction(
    _pixs: &Pix,
    _thresh: f32,
    _connectivity: crate::region::ConnectivityType,
    _sel: ThresholdSelect,
) -> Result<Pix> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixSelectByPerimSizeRatio`.
pub fn pix_select_by_perim_size_ratio(
    _pixs: &Pix,
    _thresh: f32,
    _connectivity: crate::region::ConnectivityType,
    _sel: ThresholdSelect,
) -> Result<Pix> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixSelectByPerimToAreaRatio`.
pub fn pix_select_by_perim_to_area_ratio(
    _pixs: &Pix,
    _thresh: f32,
    _connectivity: crate::region::ConnectivityType,
    _sel: ThresholdSelect,
) -> Result<Pix> {
    unimplemented!("plan 106 RED stub")
}

/// C: `pixSelectByWidthHeightRatio`.
pub fn pix_select_by_width_height_ratio(
    _pixs: &Pix,
    _thresh: f32,
    _connectivity: crate::region::ConnectivityType,
    _sel: ThresholdSelect,
) -> Result<Pix> {
    unimplemented!("plan 106 RED stub")
}
