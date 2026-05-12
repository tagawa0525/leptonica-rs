//! Pixa property/inspection helpers — RED stubs (plan 108 / C pixafunc1/2.c).

use crate::core::error::Result;
use crate::core::pix::Pix;

use super::Pixa;

impl Pixa {
    /// C: `pixaAnyColormaps`.
    pub fn any_colormaps(&self) -> bool {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaHasColor`.
    pub fn has_color(&self) -> bool {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaGetDepthInfo`.
    pub fn get_depth_info(&self) -> Result<(u32, bool)> {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaGetRenderingDepth`.
    pub fn get_rendering_depth(&self) -> Result<u32> {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaSizeRange`.
    pub fn size_range(&self) -> Option<(u32, u32, u32, u32)> {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaSetFullSizeBoxa`.
    pub fn set_full_size_boxa(&mut self) {
        unimplemented!("plan 108 RED stub")
    }

    /// C: `pixaEqual` (ordered variant).
    pub fn equal_to_ordered(&self, _other: &Pixa, _max_dist: u32) -> bool {
        unimplemented!("plan 108 RED stub")
    }
}

impl Pix {
    /// C: `pixGetTileCount`.
    pub fn get_tile_count(&self) -> u32 {
        unimplemented!("plan 108 RED stub")
    }
}
