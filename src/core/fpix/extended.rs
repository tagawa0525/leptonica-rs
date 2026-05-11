//! FPix extended helpers — RED stubs (plan 110 / C fpix2.c gap-fill).

use crate::core::error::Result;
use crate::core::pix::Pix;

use super::FPix;

impl FPix {
    /// C: `fpixGetMin` (alias for [`FPix::min`]).
    pub fn get_min(&self) -> Option<(f32, u32, u32)> {
        unimplemented!("plan 110 RED stub")
    }

    /// C: `fpixGetMax` (alias for [`FPix::max`]).
    pub fn get_max(&self) -> Option<(f32, u32, u32)> {
        unimplemented!("plan 110 RED stub")
    }

    /// C: `fpixThresholdToPix`.
    pub fn threshold_to_pix(&self, _thresh: f32) -> Result<Pix> {
        unimplemented!("plan 110 RED stub")
    }

    /// C: `fpixRasterop`.
    #[allow(clippy::too_many_arguments)]
    pub fn rasterop(
        &mut self,
        _dx: i32,
        _dy: i32,
        _dw: i32,
        _dh: i32,
        _src: &FPix,
        _sx: i32,
        _sy: i32,
    ) -> Result<()> {
        unimplemented!("plan 110 RED stub")
    }

    /// C: `fpixScaleByInteger`.
    pub fn scale_by_integer(&self, _factor: u32) -> Result<FPix> {
        unimplemented!("plan 110 RED stub")
    }

    /// C: `fpixRemoveBorder`.
    pub fn remove_border(&self, _left: i32, _right: i32, _top: i32, _bot: i32) -> Result<FPix> {
        unimplemented!("plan 110 RED stub")
    }
}

/// C: `linearInterpolatePixelFloat`.
pub fn linear_interpolate_pixel_float(
    _data: &[f32],
    _w: i32,
    _h: i32,
    _x: f32,
    _y: f32,
    _inval: f32,
) -> f32 {
    unimplemented!("plan 110 RED stub")
}
