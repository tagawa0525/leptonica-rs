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
