//! Pixa property and inspection helpers (plan 108 / C pixafunc1.c, pixafunc2.c).
//!
//! Covered functions:
//!
//! - `pixaAnyColormaps` -> [`Pixa::any_colormaps`]
//! - `pixaHasColor` -> [`Pixa::has_color`]
//! - `pixaGetDepthInfo` -> [`Pixa::get_depth_info`]
//! - `pixaGetRenderingDepth` -> [`Pixa::get_rendering_depth`]
//! - `pixaSizeRange` -> [`Pixa::size_range`]
//! - `pixaSetFullSizeBoxa` -> [`Pixa::set_full_size_boxa`]
//! - `pixaEqual` (ordered variant) -> [`Pixa::equal_to_ordered`]
//! - `pixGetTileCount` -> [`Pix::get_tile_count`]

use crate::core::box_::{Box, Boxa};
use crate::core::error::{Error, Result};
use crate::core::pix::{Pix, PixelDepth};

use super::Pixa;

impl Pixa {
    /// Return `true` when any entry has an attached colormap.
    ///
    /// C Leptonica equivalent: `pixaAnyColormaps`.
    pub fn any_colormaps(&self) -> bool {
        self.pix_slice().iter().any(|p| p.colormap().is_some())
    }

    /// Return `true` when at least one entry actually carries colour
    /// information — either via a non-grayscale colormap entry or by
    /// being 32 bpp.
    ///
    /// C Leptonica equivalent: `pixaHasColor`.
    pub fn has_color(&self) -> bool {
        self.pix_slice().iter().any(|p| {
            if p.depth() == PixelDepth::Bit32 {
                return true;
            }
            match p.colormap() {
                Some(cmap) => cmap.has_color(),
                None => false,
            }
        })
    }

    /// Inspect Pix depths.
    ///
    /// Returns `(max_depth, all_same)`. Errors when the Pixa is empty
    /// (matching C `pixaGetDepthInfo` which rejects empty input).
    ///
    /// C Leptonica equivalent: `pixaGetDepthInfo`.
    pub fn get_depth_info(&self) -> Result<(u32, bool)> {
        let n = self.pix_slice().len();
        if n == 0 {
            return Err(Error::InvalidParameter("pixa is empty".into()));
        }
        let first = self.pix_slice()[0].depth().bits();
        let mut maxd = first;
        let mut same = true;
        for p in &self.pix_slice()[1..] {
            let d = p.depth().bits();
            if d != first {
                same = false;
            }
            if d > maxd {
                maxd = d;
            }
        }
        Ok((maxd, same))
    }

    /// Return the minimum depth needed to render every entry without loss.
    ///
    /// Result is one of `1` (all 1-bpp), `8` (grey entries with depth
    /// in 2/4/8/16), or `32` (any colour entry).
    ///
    /// C Leptonica equivalent: `pixaGetRenderingDepth`.
    pub fn get_rendering_depth(&self) -> Result<u32> {
        if self.has_color() {
            return Ok(32);
        }
        let (maxd, _) = self.get_depth_info()?;
        Ok(if maxd == 1 { 1 } else { 8 })
    }

    /// Return `(min_w, min_h, max_w, max_h)` across all entries.
    ///
    /// Returns `None` when the Pixa is empty.
    ///
    /// C Leptonica equivalent: `pixaSizeRange`.
    pub fn size_range(&self) -> Option<(u32, u32, u32, u32)> {
        let pixs = self.pix_slice();
        if pixs.is_empty() {
            return None;
        }
        let mut minw = u32::MAX;
        let mut minh = u32::MAX;
        let mut maxw = 0u32;
        let mut maxh = 0u32;
        for p in pixs {
            let w = p.width();
            let h = p.height();
            if w < minw {
                minw = w;
            }
            if h < minh {
                minh = h;
            }
            if w > maxw {
                maxw = w;
            }
            if h > maxh {
                maxh = h;
            }
        }
        Some((minw, minh, maxw, maxh))
    }

    /// Replace the Boxa with one whose entries are `(0, 0, w, h)` per
    /// Pix.
    ///
    /// No-op when the Pixa is empty (C version logs `L_INFO`).
    ///
    /// C Leptonica equivalent: `pixaSetFullSizeBoxa`.
    ///
    /// Pix dimensions are validated with `i32::try_from`; any entry whose
    /// width or height exceeds `i32::MAX` is clamped to `i32::MAX` (the
    /// maximum representable `Box` dimension). This is defensive against
    /// pathologically large Pix that should not occur in practice.
    pub fn set_full_size_boxa(&mut self) {
        let n = self.pix_slice().len();
        if n == 0 {
            return;
        }
        let mut boxa = Boxa::with_capacity(n);
        for p in self.pix_slice() {
            let w = i32::try_from(p.width()).unwrap_or(i32::MAX);
            let h = i32::try_from(p.height()).unwrap_or(i32::MAX);
            boxa.push(Box::new_unchecked(0, 0, w, h));
        }
        self.set_boxa(boxa);
    }

    /// Test whether two Pixa are entry-by-entry equal.
    ///
    /// This is the **ordered** variant of C `pixaEqual`: equal length,
    /// matching Pix at the same index (via `Pix::equals`), and boxes
    /// equal under `Boxa::equal_ordered(max_dist)`. The unordered
    /// variant that derives a reorder Numa from `boxaEqual` is deferred
    /// to plan 108b.
    ///
    /// C Leptonica equivalent: `pixaEqual` (ordered case).
    pub fn equal_to_ordered(&self, other: &Pixa, max_dist: u32) -> bool {
        let a = self.pix_slice();
        let b = other.pix_slice();
        if a.len() != b.len() {
            return false;
        }
        // Compare boxes when both have entries.
        let boxa1 = self.boxa();
        let boxa2 = other.boxa();
        if !boxa1.is_empty() && !boxa2.is_empty() {
            if boxa1.len() != boxa2.len() {
                return false;
            }
            if !boxa1.equal_ordered(boxa2, max_dist as usize) {
                return false;
            }
        } else if boxa1.is_empty() != boxa2.is_empty() {
            // One has boxes, the other does not: C returns 0 (not equal).
            return false;
        }
        for (x, y) in a.iter().zip(b.iter()) {
            if !x.equals(y) {
                return false;
            }
        }
        true
    }
}

impl Pix {
    /// Parse the Pix's text tag for a tile count expressed as `"n = N"`.
    ///
    /// Returns `0` when the text tag is empty, too short, or in an
    /// unexpected format. This matches the inexact-text C contract
    /// (`pixGetTileCount`), which silently treats malformed text as
    /// "no tile count".
    ///
    /// C Leptonica equivalent: `pixGetTileCount`.
    pub fn get_tile_count(&self) -> u32 {
        let text = match self.text() {
            Some(t) => t,
            None => return 0,
        };
        if text.len() <= 4 {
            return 0;
        }
        // Expected prefix: "n = "
        let body = match text.strip_prefix("n = ") {
            Some(s) => s,
            None => return 0,
        };
        // Take leading decimal digits.
        let digits: String = body.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u32>().unwrap_or(0)
    }
}
