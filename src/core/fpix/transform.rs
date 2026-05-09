//! Orthogonal rotations, flips and border extensions for FPix.
//!
//! C Leptonica equivalent: portions of `fpix2.c` (`fpixRotateOrth`,
//! `fpixRotate90`, `fpixRotate180`, `fpixFlipLR`, `fpixFlipTB`,
//! `fpixAddBorder`, `fpixAddMirroredBorder`, `fpixAddContinuedBorder`).

use crate::core::error::Result;
use crate::core::fpix::FPix;

/// Direction of a 90° rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateDirection {
    /// Clockwise rotation.
    Cw,
    /// Counter-clockwise rotation.
    Ccw,
}

impl FPix {
    /// Rotate by `quads * 90°` clockwise. `quads` must be in `0..=3`.
    ///
    /// C Leptonica equivalent: `fpixRotateOrth`
    pub fn rotate_orth(&self, _quads: u8) -> Result<FPix> {
        unimplemented!("FPix::rotate_orth: implemented in GREEN commit (plan 103)")
    }

    /// Rotate 90° in the given direction.
    ///
    /// C Leptonica equivalent: `fpixRotate90`
    pub fn rotate_90(&self, _direction: RotateDirection) -> Result<FPix> {
        unimplemented!("FPix::rotate_90: implemented in GREEN commit (plan 103)")
    }

    /// Rotate 180° (= LR flip + TB flip).
    ///
    /// C Leptonica equivalent: `fpixRotate180`
    pub fn rotate_180(&self) -> Result<FPix> {
        unimplemented!("FPix::rotate_180: implemented in GREEN commit (plan 103)")
    }

    /// Flip left-right.
    ///
    /// C Leptonica equivalent: `fpixFlipLR`
    pub fn flip_lr(&self) -> Result<FPix> {
        unimplemented!("FPix::flip_lr: implemented in GREEN commit (plan 103)")
    }

    /// Flip top-bottom.
    ///
    /// C Leptonica equivalent: `fpixFlipTB`
    pub fn flip_tb(&self) -> Result<FPix> {
        unimplemented!("FPix::flip_tb: implemented in GREEN commit (plan 103)")
    }

    /// Extend the FPix on each side, filling the border with `fill`.
    ///
    /// C Leptonica equivalent: `fpixAddBorder`
    pub fn add_border(
        &self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
        _fill: f32,
    ) -> Result<FPix> {
        unimplemented!("FPix::add_border: implemented in GREEN commit (plan 103)")
    }

    /// Extend the FPix on each side using mirror reflection.
    ///
    /// C Leptonica equivalent: `fpixAddMirroredBorder`
    pub fn add_mirrored_border(
        &self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
    ) -> Result<FPix> {
        unimplemented!("FPix::add_mirrored_border: implemented in GREEN commit (plan 103)")
    }

    /// Extend the FPix on each side by replicating the boundary value.
    ///
    /// C Leptonica equivalent: `fpixAddContinuedBorder`
    pub fn add_continued_border(
        &self,
        _left: u32,
        _right: u32,
        _top: u32,
        _bot: u32,
    ) -> Result<FPix> {
        unimplemented!("FPix::add_continued_border: implemented in GREEN commit (plan 103)")
    }
}
