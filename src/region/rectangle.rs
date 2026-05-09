//! Rectangle detection inside 1bpp images.
//!
//! C Leptonica equivalent: `pageseg.c::pixFindLargestRectangle`,
//! `pixFindLargeRectangles`, `pixFindRectangleInCC`.

use crate::core::box_::Box;
use crate::core::{Boxa, Pix, PixelDepth};
use crate::region::error::{RegionError, RegionResult};

/// Pixel polarity selector for largest-rectangle search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// Search inside the background (white = 0). Maps to C `polarity = 0`.
    Background,
    /// Search inside the foreground (black = 1). Maps to C `polarity = 1`.
    Foreground,
}

/// Fast-scan direction for [`find_rectangle_in_cc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    /// `L_SCAN_HORIZONTAL` — fast scan along rows.
    Horizontal,
    /// `L_SCAN_VERTICAL` — fast scan along columns.
    Vertical,
}

/// How [`find_rectangle_in_cc`] combines the box found from each slow-scan
/// direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectSelect {
    /// `L_GEOMETRIC_UNION` — bounding box of the two boxes.
    GeometricUnion,
    /// `L_GEOMETRIC_INTERSECTION` — overlap region.
    GeometricIntersection,
    /// `L_LARGEST_AREA` — the larger of the two by area.
    LargestArea,
    /// `L_SMALLEST_AREA` — the smaller of the two by area.
    SmallestArea,
}

/// C Leptonica equivalent: `pixFindLargestRectangle`.
pub fn find_largest_rectangle(_pix: &Pix, _polarity: Polarity) -> RegionResult<Box> {
    unimplemented!("find_largest_rectangle: implemented in GREEN commit (plan 801)")
}

/// C Leptonica equivalent: `pixFindLargeRectangles`.
pub fn find_large_rectangles(_pix: &Pix, _polarity: Polarity, _nrect: u32) -> RegionResult<Boxa> {
    unimplemented!("find_large_rectangles: implemented in GREEN commit (plan 801)")
}

/// C Leptonica equivalent: `pixFindRectangleInCC`.
pub fn find_rectangle_in_cc(
    _pix: &Pix,
    _boxs: Option<&Box>,
    _fract: f32,
    _dir: ScanDirection,
    _select: RectSelect,
) -> RegionResult<Option<Box>> {
    unimplemented!("find_rectangle_in_cc: implemented in GREEN commit (plan 801)")
}

#[allow(dead_code)]
fn require_1bpp(pix: &Pix, name: &str) -> RegionResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        Err(RegionError::UnsupportedDepth {
            expected: "1bpp",
            actual: pix.depth().bits(),
        })
    } else {
        let _ = name;
        Ok(())
    }
}
