//! Pta + graphics helpers — RED stubs (plan 111 / C ptafunc1.c).

use crate::core::box_::Box;
use crate::core::error::Result;
use crate::core::numa::Numa;
use crate::core::pix::Pix;
use crate::core::pta::Pta;

pub enum PatternSource<'a> {
    Pix(&'a Pix),
    Pta(&'a Pta),
}

impl Pta {
    /// C: `ptaGetBoundingRegion`.
    pub fn bounding_region(&self) -> Option<Box> {
        unimplemented!("plan 111 RED stub")
    }

    /// C: `ptaConvertToNuma`.
    pub fn to_numa_pair(&self) -> (Numa, Numa) {
        unimplemented!("plan 111 RED stub")
    }

    /// C: `ptaReplicatePattern`.
    pub fn replicate_pattern(
        &self,
        _pattern: PatternSource<'_>,
        _cx: i32,
        _cy: i32,
        _w: i32,
        _h: i32,
    ) -> Result<Pta> {
        unimplemented!("plan 111 RED stub")
    }
}

impl Pix {
    /// C: `pixFindCornerPixels`.
    pub fn find_corner_pixels(&self) -> Result<Pta> {
        unimplemented!("plan 111 RED stub")
    }
}

/// C: `pixGenerateFromPta`.
pub fn pix_generate_from_pta(_pta: &Pta, _w: u32, _h: u32) -> Result<Pix> {
    unimplemented!("plan 111 RED stub")
}

/// C: `ptaGetPixelsFromPix`.
pub fn pta_get_pixels_from_pix(_pixs: &Pix, _region: Option<&Box>) -> Result<Pta> {
    unimplemented!("plan 111 RED stub")
}
