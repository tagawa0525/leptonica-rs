//! Pixa selection helpers (gap-fill plan 106 / C pixafunc1.c).
//!
//! These functions cover [`Pixa::select_range`], indicator-based filtering,
//! and metric-based selection (area fraction, perimeter ratios, width/height
//! ratio, connected-component count).
//!
//! C Leptonica equivalents are noted on each method.

use crate::core::box_::Box;
use crate::core::error::{Error, Result};
use crate::core::{Pix, PixMut, PixelDepth};

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

#[inline]
fn matches(metric: f32, thresh: f32, sel: ThresholdSelect) -> bool {
    match sel {
        ThresholdSelect::LessThan => metric < thresh,
        ThresholdSelect::GreaterThan => metric > thresh,
        ThresholdSelect::LessOrEqual => metric <= thresh,
        ThresholdSelect::GreaterOrEqual => metric >= thresh,
    }
}

impl Pixa {
    // --------------------------------------------------------------------
    // Basic selection (slice / indicator / string)
    // --------------------------------------------------------------------

    /// Return a new Pixa containing items `first..=last_inclusive`.
    ///
    /// If `last` is `None`, the slice extends to the end. Out-of-range
    /// `first` returns an empty Pixa.
    ///
    /// C Leptonica equivalent: `pixaSelectRange`.
    pub fn select_range(&self, first: usize, last: Option<usize>) -> Self {
        let n = self.pix_slice().len();
        if first >= n {
            return Pixa::new();
        }
        let end = match last {
            Some(l) => (l + 1).min(n),
            None => n,
        };
        let mut out = Pixa::with_capacity(end - first);
        for i in first..end {
            out.push_with_box(
                self.pix_slice()[i].clone(),
                self.boxa().get(i).copied().unwrap_or_default(),
            );
        }
        out
    }

    /// Filter by a boolean indicator (length must match the Pixa).
    ///
    /// Returns `(filtered, changed)` where `changed` is true when at least one
    /// element was dropped.
    ///
    /// C Leptonica equivalent: `pixaSelectWithIndicator`.
    pub fn select_with_indicator(&self, indicator: &[bool]) -> Result<(Self, bool)> {
        if indicator.len() != self.pix_slice().len() {
            return Err(Error::InvalidParameter(format!(
                "indicator length {} != pixa length {}",
                indicator.len(),
                self.pix_slice().len()
            )));
        }
        let kept: usize = indicator.iter().filter(|&&b| b).count();
        let changed = kept != indicator.len();
        let mut out = Pixa::with_capacity(kept);
        for (i, &keep) in indicator.iter().enumerate() {
            if keep {
                out.push_with_box(
                    self.pix_slice()[i].clone(),
                    self.boxa().get(i).copied().unwrap_or_default(),
                );
            }
        }
        Ok((out, changed))
    }

    /// Filter by a '0'/'1' character string indicator.
    ///
    /// Each '1' (or any non-'0') keeps the element; '0' drops it. The string
    /// length must match the Pixa.
    ///
    /// C Leptonica equivalent: `pixaSelectWithString`.
    pub fn select_with_string(&self, s: &str) -> Result<(Self, bool)> {
        if s.len() != self.pix_slice().len() {
            return Err(Error::InvalidParameter(format!(
                "string length {} != pixa length {}",
                s.len(),
                self.pix_slice().len()
            )));
        }
        let ind: Vec<bool> = s.bytes().map(|b| b != b'0').collect();
        self.select_with_indicator(&ind)
    }

    // --------------------------------------------------------------------
    // Metric-based selection
    // --------------------------------------------------------------------

    /// Filter components by their internal connected-component count
    /// (`nmin <= count <= nmax`).
    ///
    /// Each Pix is expected to be 1 bpp; non-1bpp images are skipped via the
    /// connected-component routine.
    ///
    /// C Leptonica equivalent: `pixaSelectByNumConnComp`.
    pub fn select_by_num_conn_comp(
        &self,
        nmin: u32,
        nmax: u32,
        connectivity: crate::region::ConnectivityType,
    ) -> Result<(Self, bool)> {
        if nmax < nmin {
            return Err(Error::InvalidParameter(format!(
                "nmax ({nmax}) < nmin ({nmin})"
            )));
        }
        let mut indicator = Vec::with_capacity(self.pix_slice().len());
        for pix in self.pix_slice() {
            let count = crate::region::count_conn_comp(pix, connectivity).unwrap_or(0);
            indicator.push(count >= nmin && count <= nmax);
        }
        self.select_with_indicator(&indicator)
    }

    /// Filter by FG area fraction (`fg_pixels / (w * h)`) against `thresh`.
    ///
    /// Each Pix must be 1 bpp; non-1bpp entries are treated as area fraction 0.
    ///
    /// C Leptonica equivalent: `pixaSelectByAreaFraction`.
    pub fn select_by_area_fraction(
        &self,
        thresh: f32,
        sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        let metrics = pixa_find_area_fraction(self);
        let indicator: Vec<bool> = metrics.iter().map(|&m| matches(m, thresh, sel)).collect();
        self.select_with_indicator(&indicator)
    }

    /// Filter by perimeter/size ratio (`fg_boundary / (2*(w+h))`) against
    /// `thresh`.
    ///
    /// C Leptonica equivalent: `pixaSelectByPerimSizeRatio`.
    pub fn select_by_perim_size_ratio(
        &self,
        thresh: f32,
        sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        let metrics = pixa_find_perim_size_ratio(self);
        let indicator: Vec<bool> = metrics.iter().map(|&m| matches(m, thresh, sel)).collect();
        self.select_with_indicator(&indicator)
    }

    /// Filter by perimeter/area ratio (`fg_boundary / fg_pixels`) against
    /// `thresh`.
    ///
    /// C Leptonica equivalent: `pixaSelectByPerimToAreaRatio`.
    pub fn select_by_perim_to_area_ratio(
        &self,
        thresh: f32,
        sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        let metrics = pixa_find_perim_to_area_ratio(self);
        let indicator: Vec<bool> = metrics.iter().map(|&m| matches(m, thresh, sel)).collect();
        self.select_with_indicator(&indicator)
    }

    /// Filter by width/height ratio against `thresh`.
    ///
    /// C Leptonica equivalent: `pixaSelectByWidthHeightRatio`.
    pub fn select_by_width_height_ratio(
        &self,
        thresh: f32,
        sel: ThresholdSelect,
    ) -> Result<(Self, bool)> {
        let metrics = pixa_find_width_height_ratio(self);
        let indicator: Vec<bool> = metrics.iter().map(|&m| matches(m, thresh, sel)).collect();
        self.select_with_indicator(&indicator)
    }
}

// ------------------------------------------------------------------------
// Internal find helpers (return a Numa-like Vec<f32> for indicator building)
// ------------------------------------------------------------------------

fn count_fg(pix: &Pix) -> u64 {
    if pix.depth() == PixelDepth::Bit1 {
        pix.count_pixels()
    } else {
        0
    }
}

fn boundary_count(pix: &Pix) -> u64 {
    if pix.depth() != PixelDepth::Bit1 || pix.width() == 0 || pix.height() == 0 {
        return 0;
    }
    let eroded = match crate::morph::erode_brick(pix, 3, 3) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    // boundary = fg(pix) XOR fg(eroded)
    let xor = match pix.xor(&eroded) {
        Ok(x) => x,
        Err(_) => return 0,
    };
    xor.count_pixels()
}

/// `pixaFindAreaFraction` — fg / (w*h) per pix.
pub fn pixa_find_area_fraction(pixa: &Pixa) -> Vec<f32> {
    pixa.pix_slice()
        .iter()
        .map(|p| {
            let area = (p.width() as u64) * (p.height() as u64);
            if area == 0 {
                0.0
            } else {
                count_fg(p) as f32 / area as f32
            }
        })
        .collect()
}

/// `pixaFindPerimSizeRatio` — fg boundary / (2*(w+h)) per pix.
pub fn pixa_find_perim_size_ratio(pixa: &Pixa) -> Vec<f32> {
    pixa.pix_slice()
        .iter()
        .map(|p| {
            let denom = 2.0_f32 * (p.width() + p.height()) as f32;
            if denom == 0.0 {
                0.0
            } else {
                boundary_count(p) as f32 / denom
            }
        })
        .collect()
}

/// `pixaFindPerimToAreaRatio` — fg boundary / fg pixels per pix.
pub fn pixa_find_perim_to_area_ratio(pixa: &Pixa) -> Vec<f32> {
    pixa.pix_slice()
        .iter()
        .map(|p| {
            let fg = count_fg(p);
            if fg == 0 {
                0.0
            } else {
                boundary_count(p) as f32 / fg as f32
            }
        })
        .collect()
}

/// `pixaFindWidthHeightRatio` — w / h per pix.
pub fn pixa_find_width_height_ratio(pixa: &Pixa) -> Vec<f32> {
    pixa.pix_slice()
        .iter()
        .map(|p| {
            let h = p.height();
            if h == 0 {
                0.0
            } else {
                p.width() as f32 / h as f32
            }
        })
        .collect()
}

// ------------------------------------------------------------------------
// Pix-level: add / remove components via Pixa indicator
// ------------------------------------------------------------------------

/// Render only the indicated components of `pixs` (1 bpp) on top of `pixd`.
///
/// `pixs` is broken into connected components, the indicator (one entry per
/// component) decides which are painted onto the destination buffer.
///
/// C Leptonica equivalent: `pixAddWithIndicator`.
pub fn pix_add_with_indicator(pixs: &Pix, pixad: &mut PixMut, indicator: &[bool]) -> Result<()> {
    apply_with_indicator(pixs, pixad, indicator, true)
}

/// Erase the indicated components of `pixs` (1 bpp) from `pixd`.
///
/// `pixs` is broken into connected components, the indicator (one entry per
/// component) decides which are erased.
///
/// C Leptonica equivalent: `pixRemoveWithIndicator`.
pub fn pix_remove_with_indicator(pixs: &Pix, pixad: &mut PixMut, indicator: &[bool]) -> Result<()> {
    apply_with_indicator(pixs, pixad, indicator, false)
}

fn apply_with_indicator(
    pixs: &Pix,
    pixad: &mut PixMut,
    indicator: &[bool],
    add: bool,
) -> Result<()> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let (boxa, pixa) =
        crate::region::conncomp_pixa(pixs, crate::region::ConnectivityType::EightWay)
            .map_err(|e| Error::InvalidParameter(format!("conncomp failed: {e}")))?;
    if indicator.len() != boxa.len() {
        return Err(Error::InvalidParameter(format!(
            "indicator length {} != component count {}",
            indicator.len(),
            boxa.len()
        )));
    }
    let cw = pixad.width() as i32;
    let ch = pixad.height() as i32;
    for (i, b) in boxa.boxes().iter().enumerate() {
        if !indicator[i] {
            continue;
        }
        let comp = pixa
            .get(i)
            .ok_or_else(|| Error::InvalidParameter(format!("missing component at index {i}")))?;
        paint_box(pixad, comp, b, cw, ch, add);
    }
    Ok(())
}

fn paint_box(pixad: &mut PixMut, comp: &Pix, b: &Box, cw: i32, ch: i32, add: bool) {
    let w = comp.width() as i32;
    let h = comp.height() as i32;
    for j in 0..h {
        for i in 0..w {
            if comp.get_pixel(i as u32, j as u32) != Some(1) {
                continue;
            }
            let dx = b.x + i;
            let dy = b.y + j;
            if dx < 0 || dy < 0 || dx >= cw || dy >= ch {
                continue;
            }
            let _ = pixad.set_pixel(dx as u32, dy as u32, if add { 1 } else { 0 });
        }
    }
}

// ------------------------------------------------------------------------
// Pix-level: select_by_*  (conncomp -> select -> render into fresh image)
// ------------------------------------------------------------------------

fn pix_select_via_pixa<F>(
    pixs: &Pix,
    connectivity: crate::region::ConnectivityType,
    select: F,
) -> Result<Pix>
where
    F: FnOnce(&Pixa) -> Result<(Pixa, bool)>,
{
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let (boxa, pixa) = crate::region::conncomp_pixa(pixs, connectivity)
        .map_err(|e| Error::InvalidParameter(format!("conncomp failed: {e}")))?;
    if pixa.pix_slice().is_empty() {
        // No components — return a clone (C uses pixCopy).
        return Ok(pixs.deep_clone());
    }
    let (filtered_pixa, _changed) = select(&pixa)?;
    // pixa indices may shrink relative to boxa — but we kept boxes inside
    // the returned Pixa (via push_with_box), so use those boxes.
    let out = Pix::new(pixs.width(), pixs.height(), PixelDepth::Bit1)?;
    let mut out_mut = out.try_into_mut().expect("freshly created");
    let cw = pixs.width() as i32;
    let ch = pixs.height() as i32;
    for (i, comp) in filtered_pixa.pix_slice().iter().enumerate() {
        let b = filtered_pixa.boxa().get(i).copied().unwrap_or_default();
        paint_box(&mut out_mut, comp, &b, cw, ch, true);
    }
    let _ = boxa; // not used directly; conncomp boxes were carried through pixa
    Ok(out_mut.into())
}

/// `pixSelectByAreaFraction`.
pub fn pix_select_by_area_fraction(
    pixs: &Pix,
    thresh: f32,
    connectivity: crate::region::ConnectivityType,
    sel: ThresholdSelect,
) -> Result<Pix> {
    pix_select_via_pixa(pixs, connectivity, |pixa| {
        pixa.select_by_area_fraction(thresh, sel)
    })
}

/// `pixSelectByPerimSizeRatio`.
pub fn pix_select_by_perim_size_ratio(
    pixs: &Pix,
    thresh: f32,
    connectivity: crate::region::ConnectivityType,
    sel: ThresholdSelect,
) -> Result<Pix> {
    pix_select_via_pixa(pixs, connectivity, |pixa| {
        pixa.select_by_perim_size_ratio(thresh, sel)
    })
}

/// `pixSelectByPerimToAreaRatio`.
pub fn pix_select_by_perim_to_area_ratio(
    pixs: &Pix,
    thresh: f32,
    connectivity: crate::region::ConnectivityType,
    sel: ThresholdSelect,
) -> Result<Pix> {
    pix_select_via_pixa(pixs, connectivity, |pixa| {
        pixa.select_by_perim_to_area_ratio(thresh, sel)
    })
}

/// `pixSelectByWidthHeightRatio`.
pub fn pix_select_by_width_height_ratio(
    pixs: &Pix,
    thresh: f32,
    connectivity: crate::region::ConnectivityType,
    sel: ThresholdSelect,
) -> Result<Pix> {
    pix_select_via_pixa(pixs, connectivity, |pixa| {
        pixa.select_by_width_height_ratio(thresh, sel)
    })
}
