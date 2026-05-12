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

/// Which dimension(s) [`Pixa::make_size_indicator`] tests against the
/// threshold `(width, height)`.
///
/// C Leptonica constants: `L_SELECT_WIDTH`, `L_SELECT_HEIGHT`,
/// `L_SELECT_IF_EITHER`, `L_SELECT_IF_BOTH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeIndicatorAxis {
    /// Compare width only.
    Width,
    /// Compare height only.
    Height,
    /// Match when either width *or* height satisfies the relation.
    IfEither,
    /// Match only when both width *and* height satisfy the relation.
    IfBoth,
}

impl Pixa {
    /// Produce a 0/1 indicator [`Numa`] flagging Pix whose
    /// `(width, height)` satisfy `relation` against the given
    /// `(width, height)` threshold.
    ///
    /// C Leptonica equivalent: `pixaMakeSizeIndicator`.
    pub fn make_size_indicator(
        &self,
        width: u32,
        height: u32,
        axis: SizeIndicatorAxis,
        relation: super::ThresholdSelect,
    ) -> crate::core::numa::Numa {
        let pixs = self.pix_slice();
        let mut na = crate::core::numa::Numa::with_capacity(pixs.len());
        let w_t = width as f32;
        let h_t = height as f32;
        for p in pixs {
            let w = p.width() as f32;
            let h = p.height() as f32;
            let w_match = match_relation(w, w_t, relation);
            let h_match = match_relation(h, h_t, relation);
            let val = match axis {
                SizeIndicatorAxis::Width => w_match,
                SizeIndicatorAxis::Height => h_match,
                SizeIndicatorAxis::IfEither => w_match || h_match,
                SizeIndicatorAxis::IfBoth => w_match && h_match,
            };
            na.push(if val { 1.0 } else { 0.0 });
        }
        na
    }

    /// Split this Pixa into a [`Pixaa`] using an index Numaa.
    ///
    /// Each inner Numa lists 0-based indices into `self`; the
    /// corresponding Pix (and Box, when present) are gathered into a
    /// fresh inner Pixa. The resulting Pixaa has the same length as
    /// the input Numaa.
    ///
    /// Returns `Err` when the total index count in the Numaa does not
    /// equal the Pixa size (matching C's "element count mismatch"
    /// check).
    ///
    /// C Leptonica equivalent: `pixaSort2dByIndex`.
    pub fn sort_2d_by_index(
        &self,
        naa: &crate::core::numa::Numaa,
    ) -> Result<crate::core::pixa::Pixaa> {
        let pix_total = self.pix_slice().len();
        let total: usize = (0..naa.len())
            .map(|i| naa.get(i).map(|n| n.len()).unwrap_or(0))
            .sum();
        if total != pix_total {
            return Err(Error::InvalidParameter(format!(
                "naa element count ({total}) != pixa size ({pix_total})"
            )));
        }
        let mut paa = crate::core::pixa::Pixaa::with_capacity(naa.len());
        for i in 0..naa.len() {
            let na = naa.get(i).expect("0..naa.len() must be valid");
            let mut pa = Pixa::with_capacity(na.len());
            for j in 0..na.len() {
                let idx = na.get(j).unwrap_or(-1.0) as i64;
                if idx < 0 || (idx as usize) >= pix_total {
                    return Err(Error::InvalidParameter(format!(
                        "index {idx} out of range (pixa size {pix_total})"
                    )));
                }
                let idx = idx as usize;
                let pix = self.pix_slice()[idx].clone();
                let b = self.boxa().get(idx).copied().unwrap_or_default();
                pa.push_with_box(pix, b);
            }
            paa.push(pa);
        }
        Ok(paa)
    }

    /// Select a constrained, evenly-spaced subset of Pix.
    ///
    /// Wraps `gen_constrained_numa_in_range(first, last, nmax,
    /// use_pairs)` to compute the index list and gathers the
    /// corresponding entries (deep-cloned). Returns an empty Pixa
    /// when the constraint yields no indices.
    ///
    /// C Leptonica equivalent: `pixaConstrainedSelect`.
    pub fn constrained_select(
        &self,
        first: i32,
        last: i32,
        nmax: i32,
        use_pairs: bool,
    ) -> Result<Pixa> {
        let n = self.pix_slice().len();
        let first = first.max(0);
        let last = if last < 0 {
            (n as i32) - 1
        } else {
            last.min((n as i32) - 1)
        };
        if last < first {
            return Err(Error::InvalidParameter(format!(
                "last ({last}) < first ({first})"
            )));
        }
        if nmax < 1 {
            return Err(Error::InvalidParameter(format!("nmax < 1 (got {nmax})")));
        }
        let na = crate::core::numa::gen_constrained_numa_in_range(first, last, nmax, use_pairs)?;
        let mut out = Pixa::with_capacity(na.len());
        for i in 0..na.len() {
            let idx = na.get(i).unwrap_or(-1.0) as i64;
            if idx < 0 || (idx as usize) >= n {
                continue;
            }
            let idx = idx as usize;
            let pix = self.pix_slice()[idx].clone();
            let b = self.boxa().get(idx).copied().unwrap_or_default();
            out.push_with_box(pix, b);
        }
        Ok(out)
    }
}

#[inline]
fn match_relation(v: f32, t: f32, rel: super::ThresholdSelect) -> bool {
    use super::ThresholdSelect::*;
    match rel {
        LessThan => v < t,
        GreaterThan => v > t,
        LessOrEqual => v <= t,
        GreaterOrEqual => v >= t,
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
