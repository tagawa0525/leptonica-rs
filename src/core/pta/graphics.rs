//! Pta + graphics helpers (plan 111 / C ptafunc1.c).
//!
//! Covered functions:
//!
//! - `ptaGetBoundingRegion` -> [`Pta::bounding_region`]
//! - `ptaConvertToNuma` -> [`Pta::to_numa_pair`]
//! - `pixGenerateFromPta` -> [`pix_generate_from_pta`]
//! - `ptaGetPixelsFromPix` -> [`pta_get_pixels_from_pix`]
//! - `pixFindCornerPixels` -> [`Pix::find_corner_pixels`]
//! - `ptaReplicatePattern` -> [`Pta::replicate_pattern`]
//!
//! `numaConvertToPta1` / `numaConvertToPta2` are already covered by
//! [`Pta::create_from_numa`].

use crate::core::box_::Box;
use crate::core::error::{Error, Result};
use crate::core::numa::Numa;
use crate::core::pix::{Pix, PixelDepth};
use crate::core::pta::Pta;

/// Pattern source for [`Pta::replicate_pattern`].
///
/// Mirrors the C signature of `ptaReplicatePattern(ptas, pixp, ptap, ...)`
/// where exactly one of `pixp`/`ptap` is non-null. Rust uses a single
/// enum to make the alternatives explicit.
pub enum PatternSource<'a> {
    /// 1 bpp pattern image; foreground pixels are the pattern points.
    Pix(&'a Pix),
    /// Direct pattern point set.
    Pta(&'a Pta),
}

impl Pta {
    /// Return the integer-coordinate bounding [`Box`] of all points.
    ///
    /// Returns `None` when the Pta is empty.
    /// Returns `Err` when the resulting width/height would overflow `i32`
    /// (extreme inputs only — every coordinate must be near `i32::MAX`/
    /// `i32::MIN` for this to trigger).
    ///
    /// Coordinates are rounded with `f32::round` (half-away-from-zero),
    /// which is symmetric across positive and negative values. This is a
    /// slight departure from C's `(x + 0.5) as i32` (which truncates
    /// toward zero for negatives) but matches the box semantics expected
    /// by callers that pass floating-point points.
    ///
    /// C Leptonica equivalent: `ptaGetBoundingRegion`.
    pub fn bounding_region(&self) -> Result<Option<Box>> {
        if self.is_empty() {
            return Ok(None);
        }
        let mut minx = i32::MAX;
        let mut maxx = i32::MIN;
        let mut miny = i32::MAX;
        let mut maxy = i32::MIN;
        for i in 0..self.len() {
            let (xf, yf) = self.get(i).expect("index in 0..len must be valid");
            let x = xf.round() as i32;
            let y = yf.round() as i32;
            if x < minx {
                minx = x;
            }
            if x > maxx {
                maxx = x;
            }
            if y < miny {
                miny = y;
            }
            if y > maxy {
                maxy = y;
            }
        }
        let w = maxx
            .checked_sub(minx)
            .and_then(|d| d.checked_add(1))
            .ok_or_else(|| Error::InvalidParameter("bounding box width overflows i32".into()))?;
        let h = maxy
            .checked_sub(miny)
            .and_then(|d| d.checked_add(1))
            .ok_or_else(|| Error::InvalidParameter("bounding box height overflows i32".into()))?;
        Ok(Some(Box::new(minx, miny, w, h)?))
    }

    /// Split this Pta into two parallel Numa arrays (x, y).
    ///
    /// C Leptonica equivalent: `ptaConvertToNuma`.
    pub fn to_numa_pair(&self) -> (Numa, Numa) {
        let n = self.len();
        let mut nax = Numa::with_capacity(n);
        let mut nay = Numa::with_capacity(n);
        for i in 0..n {
            if let Some((x, y)) = self.get(i) {
                nax.push(x);
                nay.push(y);
            }
        }
        (nax, nay)
    }

    /// Replicate a pattern at each point in `self`, clipping points
    /// that would fall outside the `w x h` canvas.
    ///
    /// `(cx, cy)` is the pattern centre; for each `(x, y)` in `self`
    /// every pattern point `(xp, yp)` lands at `(x - cx + xp,
    /// y - cy + yp)`.
    ///
    /// C Leptonica equivalent: `ptaReplicatePattern`.
    pub fn replicate_pattern(
        &self,
        pattern: PatternSource<'_>,
        cx: i32,
        cy: i32,
        w: i32,
        h: i32,
    ) -> Result<Pta> {
        if w <= 0 || h <= 0 {
            return Err(Error::InvalidParameter(format!(
                "canvas dimensions must be positive (got w={w}, h={h})"
            )));
        }
        // Borrow the pattern Pta when caller supplies one; only the Pix
        // arm has to allocate (extracting FG points produces a fresh Pta).
        let owned_from_pix;
        let ptap: &Pta = match pattern {
            PatternSource::Pta(p) => p,
            PatternSource::Pix(pix) => {
                owned_from_pix = pta_get_pixels_from_pix(pix, None)?;
                &owned_from_pix
            }
        };
        let n = self.len();
        let np = ptap.len();
        let mut out = Pta::with_capacity(n.saturating_mul(np));
        for i in 0..n {
            let (x, y) = self.get_i_pt(i).expect("index in 0..len must be valid");
            for j in 0..np {
                let (xp, yp) = ptap.get_i_pt(j).expect("index in 0..np must be valid");
                let xf = x - cx + xp;
                let yf = y - cy + yp;
                if xf >= 0 && xf < w && yf >= 0 && yf < h {
                    out.push(xf as f32, yf as f32);
                }
            }
        }
        Ok(out)
    }
}

impl Pix {
    /// Find the four "corner" foreground pixels of a 1 bpp image.
    ///
    /// Scans along the diagonals from each corner inward and records
    /// the first foreground pixel found in each direction. Up to 4
    /// points are returned (one per corner that has a foreground
    /// pixel); when an entire diagonal traversal is empty the
    /// corresponding corner is skipped (C also leaves it out of the
    /// returned Pta).
    ///
    /// Requires a 1 bpp image.
    ///
    /// C Leptonica equivalent: `pixFindCornerPixels`.
    pub fn find_corner_pixels(&self) -> Result<Pta> {
        if self.depth() != PixelDepth::Bit1 {
            return Err(Error::UnsupportedDepth(self.depth().bits()));
        }
        let w = self.width() as i32;
        let h = self.height() as i32;
        let mindim = w.min(h);
        let mut pta = Pta::with_capacity(4);

        // Inline-friendly generic helper: scan anti-diagonals from a
        // corner. Using a generic `F: Fn(i32, i32) -> (i32, i32)` lets
        // the compiler monomorphize each call and inline the closure,
        // avoiding per-pixel dynamic dispatch.
        fn scan_corner<F: Fn(i32, i32) -> (i32, i32)>(
            pix: &Pix,
            w: i32,
            h: i32,
            mindim: i32,
            pta: &mut Pta,
            xy: F,
        ) {
            for i in 0..mindim {
                for j in 0..=i {
                    let (x, y) = xy(i, j);
                    if x < 0 || x >= w || y < 0 || y >= h {
                        continue;
                    }
                    if pix.get_pixel_unchecked(x as u32, y as u32) != 0 {
                        pta.push(x as f32, y as f32);
                        return;
                    }
                }
            }
        }

        scan_corner(self, w, h, mindim, &mut pta, |i, j| (j, i - j)); // TL
        scan_corner(self, w, h, mindim, &mut pta, |i, j| (w - 1 - j, i - j)); // TR
        scan_corner(self, w, h, mindim, &mut pta, |i, j| (j, h - 1 - (i - j))); // BL
        scan_corner(self, w, h, mindim, &mut pta, |i, j| {
            (w - 1 - j, h - 1 - (i - j))
        }); // BR

        Ok(pta)
    }
}

/// Render a Pta as foreground pixels on a fresh `w x h` 1 bpp Pix.
///
/// Points outside the canvas are silently dropped (matching C's
/// `pixGenerateFromPta`).
///
/// C Leptonica equivalent: `pixGenerateFromPta`.
pub fn pix_generate_from_pta(pta: &Pta, w: u32, h: u32) -> Result<Pix> {
    let pix = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut m = pix.try_into_mut().expect("freshly created");
    let wi = w as i32;
    let hi = h as i32;
    for i in 0..pta.len() {
        let (x, y) = pta.get_i_pt(i).unwrap_or((-1, -1));
        if x < 0 || x >= wi || y < 0 || y >= hi {
            continue;
        }
        m.set_pixel(x as u32, y as u32, 1)?;
    }
    Ok(m.into())
}

/// Extract foreground pixel coordinates from a 1 bpp image as a Pta.
///
/// Optionally restrict to a sub-rectangle. Requires 1 bpp.
///
/// C Leptonica equivalent: `ptaGetPixelsFromPix`.
pub fn pta_get_pixels_from_pix(pixs: &Pix, region: Option<&Box>) -> Result<Pta> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    let (xstart, ystart, xend, yend) = match region {
        Some(b) => (b.x, b.y, b.x + b.w - 1, b.y + b.h - 1),
        None => (0, 0, w - 1, h - 1),
    };
    // Clamp the iteration range to the image so we never read out of bounds.
    let xstart = xstart.max(0);
    let ystart = ystart.max(0);
    let xend = xend.min(w - 1);
    let yend = yend.min(h - 1);

    let mut pta = Pta::new();
    for y in ystart..=yend {
        for x in xstart..=xend {
            if pixs.get_pixel_unchecked(x as u32, y as u32) != 0 {
                pta.push(x as f32, y as f32);
            }
        }
    }
    Ok(pta)
}

/// Boundary-pixel type for [`pta_get_boundary_pixels`] and
/// [`ptaa_get_boundary_pixels`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    /// Foreground boundary pixels (those that border background).
    Foreground,
    /// Background boundary pixels (those that border foreground).
    Background,
}

/// Extract foreground- or background-boundary pixels of a 1 bpp image.
///
/// For `Foreground`, `pixs` is eroded with a 3×3 SE and XOR-ed back with
/// itself; for `Background` the same is done with dilation. The remaining
/// pixels are the boundary, returned as their `(x, y)` coordinates.
///
/// C Leptonica equivalent: `ptaGetBoundaryPixels` (`ptafunc1.c`).
pub fn pta_get_boundary_pixels(pixs: &Pix, btype: BoundaryType) -> Result<Pta> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let seq = match btype {
        BoundaryType::Foreground => "e3.3",
        BoundaryType::Background => "d3.3",
    };
    let processed = crate::morph::sequence::morph_sequence(pixs, seq)
        .map_err(|e| Error::InvalidParameter(format!("morph_sequence({seq}): {e}")))?;
    let xored = processed.xor(pixs)?;
    pta_get_pixels_from_pix(&xored, None)
}

/// Generate a Pta of valid 4- or 8-connected neighbour locations of
/// `(x, y)` within `pixs`.
///
/// Out-of-image neighbours are omitted (the image acts as an implicit
/// boundary). `(x, y)` itself must lie inside `pixs`.
///
/// C Leptonica equivalent: `ptaGetNeighborPixLocs` (`ptafunc1.c`).
pub fn pta_get_neighbor_pix_locs(pixs: &Pix, x: i32, y: i32, conn: u32) -> Result<Pta> {
    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    if x < 0 || x >= w || y < 0 || y >= h {
        return Err(Error::InvalidParameter(format!(
            "(x, y) = ({x}, {y}) outside pixs {w}x{h}"
        )));
    }
    if conn != 4 && conn != 8 {
        return Err(Error::InvalidParameter(format!(
            "conn must be 4 or 8 (got {conn})"
        )));
    }
    let mut pta = Pta::with_capacity(conn as usize);
    if x > 0 {
        pta.push((x - 1) as f32, y as f32);
    }
    if x < w - 1 {
        pta.push((x + 1) as f32, y as f32);
    }
    if y > 0 {
        pta.push(x as f32, (y - 1) as f32);
    }
    if y < h - 1 {
        pta.push(x as f32, (y + 1) as f32);
    }
    if conn == 8 {
        if x > 0 {
            if y > 0 {
                pta.push((x - 1) as f32, (y - 1) as f32);
            }
            if y < h - 1 {
                pta.push((x - 1) as f32, (y + 1) as f32);
            }
        }
        if x < w - 1 {
            if y > 0 {
                pta.push((x + 1) as f32, (y - 1) as f32);
            }
            if y < h - 1 {
                pta.push((x + 1) as f32, (y + 1) as f32);
            }
        }
    }
    Ok(pta)
}

/// Boundary pixels of every connected component of a 1 bpp image.
///
/// Each component is processed in isolation (per its bounding box). When
/// `btype == Background`, a 1-pixel border is added on each side of the
/// component that does **not** already touch the parent image edge, so
/// that interior bg-boundary pixels exist (sides that already touch the
/// image edge are left unpadded, matching C `ptaaGetBoundaryPixels`).
/// The result is a Ptaa with one Pta per component, expressed in the
/// original image's coordinate frame. Optional `Boxa` and `Pixa` of the
/// components are returned when their respective `want_*` flags are set.
///
/// C Leptonica equivalent: `ptaaGetBoundaryPixels` (`ptafunc1.c`).
pub fn ptaa_get_boundary_pixels(
    pixs: &Pix,
    btype: BoundaryType,
    connectivity: u32,
    want_boxa: bool,
    want_pixa: bool,
) -> Result<(
    crate::core::pta::Ptaa,
    Option<crate::core::Boxa>,
    Option<crate::core::Pixa>,
)> {
    if pixs.depth() != PixelDepth::Bit1 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let conn = match connectivity {
        4 => crate::region::ConnectivityType::FourWay,
        8 => crate::region::ConnectivityType::EightWay,
        other => {
            return Err(Error::InvalidParameter(format!(
                "connectivity must be 4 or 8 (got {other})"
            )));
        }
    };
    let w = pixs.width() as i32;
    let h = pixs.height() as i32;
    let (boxa, pixa) = crate::region::conncomp_pixa(pixs, conn)
        .map_err(|e| Error::InvalidParameter(format!("conncomp_pixa: {e}")))?;
    let n = boxa.len();
    let mut ptaa = crate::core::pta::Ptaa::with_capacity(n);

    for i in 0..n {
        let comp = pixa
            .get(i)
            .ok_or_else(|| Error::InvalidParameter(format!("pixa[{i}] missing")))?;
        let b = boxa
            .get(i)
            .ok_or_else(|| Error::InvalidParameter(format!("boxa[{i}] missing")))?;
        let (x, y, bw, bh) = (b.x, b.y, b.w, b.h);

        // For background-boundary, pad by 1 pixel on any side that does
        // *not* already touch the image edge, so the bg ring exists.
        let (left, right, top, bot) = if btype == BoundaryType::Background {
            (
                (x > 0) as u32,
                ((x + bw) < w) as u32,
                (y > 0) as u32,
                ((y + bh) < h) as u32,
            )
        } else {
            (0, 0, 0, 0)
        };
        // Pix is Arc-backed; cheap-clone only when we need to materialise
        // the padded version, otherwise just reuse `comp` by reference.
        let padded;
        let processed: &Pix = if left + right + top + bot > 0 {
            padded = comp.add_border_general(left, right, top, bot, 0)?;
            &padded
        } else {
            comp
        };
        let mut pta = pta_get_boundary_pixels(processed, btype)?;
        // Translate from the (padded) component frame back to the parent
        // image frame: shift by `(x - left, y - top)`.
        pta.translate((x - left as i32) as f32, (y - top as i32) as f32);
        ptaa.push(pta);
    }

    let boxa_out = if want_boxa { Some(boxa) } else { None };
    let pixa_out = if want_pixa { Some(pixa) } else { None };
    Ok((ptaa, boxa_out, pixa_out))
}

/// Bucket the pixels of a 32 bpp labeled image into a Ptaa, one Pta per
/// connected-component label.
///
/// `pixs` must be 32 bpp; pixel values are treated as integer labels
/// (typically the output of a connected-component labeler). The returned
/// Ptaa has one Pta per label index `0..=max_label`, with `(x, y)`
/// coordinates of all pixels carrying that label. Returns
/// `(ptaa, max_label)` so callers can read the count of components.
///
/// C Leptonica equivalent: `ptaaIndexLabeledPixels` (`ptafunc1.c`).
pub fn ptaa_index_labeled_pixels(pixs: &Pix) -> Result<(crate::core::pta::Ptaa, u32)> {
    if pixs.depth() != PixelDepth::Bit32 {
        return Err(Error::UnsupportedDepth(pixs.depth().bits()));
    }
    let w = pixs.width();
    let h = pixs.height();
    // First pass: find the max label.
    let mut maxval: u32 = 0;
    for y in 0..h {
        for x in 0..w {
            let v = pixs.get_pixel_unchecked(x, y);
            if v > maxval {
                maxval = v;
            }
        }
    }
    // Sanity-cap maxval before allocating `(maxval + 1)` empty Pta entries.
    // A plausible connected-component count is in the millions; values
    // beyond `MAX_LABELS` almost always indicate non-label 32 bpp data
    // (e.g. an RGB image mistakenly passed in) and would explode memory.
    const MAX_LABELS: u32 = 1_048_576; // 2^20
    if maxval > MAX_LABELS {
        return Err(Error::InvalidParameter(format!(
            "ptaa_index_labeled_pixels: max label {maxval} exceeds the \
             {MAX_LABELS}-label cap; pixs is likely not a labeled image"
        )));
    }
    // Pre-fill the Ptaa with (maxval + 1) empty Pta entries so add_pt can
    // index any label up to maxval. `with_capacity` only reserves space,
    // not slots, so we push explicitly.
    let mut ptaa = crate::core::pta::Ptaa::with_capacity((maxval + 1) as usize);
    for _ in 0..=maxval {
        ptaa.push(Pta::new());
    }
    // Second pass: bucket each non-zero pixel by its label.
    for y in 0..h {
        for x in 0..w {
            let index = pixs.get_pixel_unchecked(x, y);
            if index > 0 {
                ptaa.add_pt(index as usize, x as f32, y as f32)?;
            }
        }
    }
    Ok((ptaa, maxval))
}
