//! Pixa transform helpers — RED stubs (plan 107 / C pixafunc1.c + pixafunc2.c).

use crate::core::error::Result;
use crate::core::pix::rop::InColor;

use super::Pixa;

impl Pixa {
    /// C: `pixaScale`.
    pub fn scale(&self, _scale_x: f32, _scale_y: f32) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaScaleBySampling`.
    pub fn scale_by_sampling(&self, _scale_x: f32, _scale_y: f32) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaRotateOrth`.
    pub fn rotate_orth(&self, _quads: u32) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaTranslate`.
    pub fn translate(&self, _hshift: i32, _vshift: i32, _incolor: InColor) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaConvertTo1`.
    pub fn convert_to_1(&self, _thresh: u32) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaConvertTo8`.
    pub fn convert_to_8(&self, _cmap_flag: bool) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }

    /// C: `pixaConvertTo32`.
    pub fn convert_to_32(&self) -> Result<Pixa> {
        unimplemented!("plan 107 RED stub")
    }
}
