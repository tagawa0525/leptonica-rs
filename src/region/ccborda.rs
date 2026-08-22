//! Connected-component border representation, ported from C Leptonica
//! `ccbord.c`.
//!
//! [`CcBorda`] holds one [`CcBord`] per 8-connected component of a binary
//! image. Each component records its exterior border and the border of every
//! hole inside it, in three interchangeable forms: pixel coordinates local to
//! the component, the same in global image coordinates, and a step chain code.
//!
//! # Relationship to [`crate::region::ccbord`]
//!
//! `ccbord`'s `Border` / `ComponentBorders` are a Rust-only convenience API
//! with no one-to-one C counterpart. [`CcBorda`] is the port of C's `CCBORDA`
//! and is what `prog/ccbord_reg.c` exercises.

use crate::core::{Box, Boxa, Numa, Numaa, Pix, PixelDepth, Pta, Ptaa};
use crate::region::conncomp::{ConnectivityType, conncomp_pixa, next_on_pixel_in_raster};
use crate::region::error::{RegionError, RegionResult};
use crate::region::seedfill::{holes_by_filling, seedfill_binary_restricted};

/// Upper bound on the points a single border trace may record.
///
/// C has no such bound: it assumes the termination condition (back at the
/// start pixel heading for the same second pixel) always fires. It can fail
/// on ill-formed input, so cap the walk rather than growing without limit.
///
/// The bound must scale with area, not perimeter: a comb-shaped component's
/// border walks up and down every tooth, so its length grows with the area.
/// The trace revisits a pixel at most once per incoming direction, so
/// `8 * w * h + 16` cannot reject well-formed input.
fn max_border_points(w: i32, h: i32) -> usize {
    (w.max(0) as u64)
        .saturating_mul(h.max(0) as u64)
        .saturating_mul(8)
        .saturating_add(16)
        .min(usize::MAX as u64) as usize
}

/// Which coordinate frame [`CcBorda::step_chains_to_pix_coords`] writes into.
///
/// C: `CCB_LOCAL_COORDS` / `CCB_GLOBAL_COORDS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CcbCoords {
    /// Relative to the component's bounding box.
    Local,
    /// Relative to the whole image.
    Global,
}

// Tables used to trace the border. The 8 neighbour positions of Q are
// labelled clockwise starting from the west:
//
//     1   2   3
//     0   P   4
//     7   6   5
//
// `XPOSTAB` / `YPOSTAB` give Q's pixel offset from P for each label, and
// `QPOSTAB[pos]` gives Q's new label once the pixel at `pos` becomes the new
// P. P and Q stay 4-connected.
const XPOSTAB: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
const YPOSTAB: [i32; 8] = [0, -1, -1, -1, 0, 1, 1, 1];
const QPOSTAB: [i32; 8] = [6, 6, 0, 0, 2, 2, 4, 4];

/// The borders of a single 8-connected component (C `CCBORD`).
#[derive(Debug, Clone, Default)]
pub struct CcBord {
    /// Region of each closed curve. Index 0 is the component's own bounding
    /// box in global coordinates; the rest are hole borders in coordinates
    /// relative to the component.
    pub boxa: Boxa,
    /// First pixel of each border.
    pub start: Pta,
    /// Border pixels, local to the component.
    pub local: Ptaa,
    /// Border pixels in global image coordinates.
    pub global: Ptaa,
    /// Step chain code of each border.
    pub step: Numaa,
}

/// Borders of every component in an image (C `CCBORDA`).
#[derive(Debug, Clone)]
pub struct CcBorda {
    w: u32,
    h: u32,
    ccb: Vec<CcBord>,
}

impl CcBorda {
    /// Trace the borders of every 8-connected component of `pixs`.
    ///
    /// # Errors
    ///
    /// Returns an error unless `pixs` is 1 bpp.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixGetAllCCBorders()` in `ccbord.c`
    pub fn from_pix(pixs: &Pix) -> RegionResult<Self> {
        if pixs.depth() != PixelDepth::Bit1 {
            return Err(RegionError::UnsupportedDepth {
                expected: "1-bit",
                actual: pixs.depth().bits(),
            });
        }

        let (boxa, pixa) = conncomp_pixa(pixs, ConnectivityType::EightWay)?;
        let mut ccba = Self {
            w: pixs.width(),
            h: pixs.height(),
            ccb: Vec::with_capacity(boxa.len()),
        };
        for i in 0..boxa.len() {
            let pix = pixa.get(i).ok_or_else(|| {
                RegionError::InvalidParameters(format!("component {i} has no pix"))
            })?;
            let b = boxa.get(i).ok_or_else(|| {
                RegionError::InvalidParameters(format!("component {i} has no box"))
            })?;
            ccba.ccb.push(cc_borders(pix, b)?);
        }
        Ok(ccba)
    }

    /// Width of the image the borders came from.
    pub fn width(&self) -> u32 {
        self.w
    }

    /// Height of the image the borders came from.
    pub fn height(&self) -> u32 {
        self.h
    }

    /// Number of components.
    pub fn len(&self) -> usize {
        self.ccb.len()
    }

    /// Whether there are no components.
    pub fn is_empty(&self) -> bool {
        self.ccb.is_empty()
    }

    /// The borders of one component.
    pub fn get(&self, index: usize) -> Option<&CcBord> {
        self.ccb.get(index)
    }

    /// Fill in [`CcBord::global`] from [`CcBord::local`], shifting each
    /// component's pixels by the upper-left corner of its bounding box.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaGenerateGlobalLocs()` in `ccbord.c`
    pub fn generate_global_locs(&mut self) {
        for ccb in &mut self.ccb {
            let (xul, yul) = match ccb.boxa.get(0) {
                Some(b) => (b.x, b.y),
                None => (0, 0),
            };
            let mut global = Ptaa::new();
            for j in 0..ccb.local.len() {
                let local = ccb.local.get(j).expect("border in range");
                let mut pta = Pta::with_capacity(local.len());
                for k in 0..local.len() {
                    let (x, y) = pta_get_ipt(local, k);
                    pta.push((x + xul) as f32, (y + yul) as f32);
                }
                global.push(pta);
            }
            ccb.global = global;
        }
    }

    /// Fill in [`CcBord::step`] from [`CcBord::local`].
    ///
    /// The step direction of each pixel relative to its predecessor is the
    /// label from the tracing table:
    ///
    /// ```text
    /// 1   2   3
    /// 0   P   4
    /// 7   6   5
    /// ```
    ///
    /// A border of a single pixel gets an empty chain, as in C.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaGenerateStepChains()` in `ccbord.c`
    pub fn generate_step_chains(&mut self) {
        const DIRTAB: [[i32; 3]; 3] = [[1, 2, 3], [0, -1, 4], [7, 6, 5]];

        for ccb in &mut self.ccb {
            let mut step = Numaa::new();
            for j in 0..ccb.local.len() {
                let local = ccb.local.get(j).expect("border in range");
                let mut na = Numa::new();
                if local.len() > 1 {
                    let (mut px, mut py) = pta_get_ipt(local, 0);
                    for k in 1..local.len() {
                        let (cx, cy) = pta_get_ipt(local, k);
                        let stepdir = DIRTAB[(1 + cy - py) as usize][(1 + cx - px) as usize];
                        na.push(stepdir as f32);
                        px = cx;
                        py = cy;
                    }
                }
                step.push(na);
            }
            ccb.step = step;
        }
    }

    /// Rebuild pixel coordinates from the step chains.
    ///
    /// Writes into [`CcBord::local`] or [`CcBord::global`] depending on
    /// `coords`, replacing whatever was there.
    ///
    /// # Errors
    ///
    /// Returns an error if the step chains have not been generated yet.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaStepChainsToPixCoords()` in `ccbord.c`
    pub fn step_chains_to_pix_coords(&mut self, coords: CcbCoords) -> RegionResult<()> {
        for (i, ccb) in self.ccb.iter_mut().enumerate() {
            if ccb.step.is_empty() && !ccb.local.is_empty() {
                return Err(RegionError::InvalidParameters(format!(
                    "component {i} has no step chains; call generate_step_chains first"
                )));
            }
            let (xul, yul) = match coords {
                CcbCoords::Local => (0, 0),
                CcbCoords::Global => match ccb.boxa.get(0) {
                    Some(b) => (b.x, b.y),
                    None => {
                        return Err(RegionError::InvalidParameters(format!(
                            "component {i} has no bounding box"
                        )));
                    }
                },
            };

            let mut out = Ptaa::new();
            for j in 0..ccb.step.len() {
                let na = ccb.step.get(j).expect("border in range");
                let mut pta = Pta::with_capacity(na.len() + 1);
                let (xstart, ystart) = pta_get_ipt(&ccb.start, j);
                let mut x = xul + xstart;
                let mut y = yul + ystart;
                pta.push(x as f32, y as f32);
                for k in 0..na.len() {
                    let stepdir = na.get(k).unwrap_or(0.0) as usize;
                    x += XPOSTAB[stepdir];
                    y += YPOSTAB[stepdir];
                    pta.push(x as f32, y as f32);
                }
                out.push(pta);
            }
            match coords {
                CcbCoords::Local => ccb.local = out,
                CcbCoords::Global => ccb.global = out,
            }
        }
        Ok(())
    }

    /// Paint every border pixel, using the global coordinates.
    ///
    /// [`CcBorda::generate_global_locs`] or
    /// [`CcBorda::step_chains_to_pix_coords`] with [`CcbCoords::Global`] must
    /// have run first; a component without global coordinates is skipped, as
    /// in C.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaDisplayBorder()` in `ccbord.c`
    pub fn display_border(&self) -> RegionResult<Pix> {
        let pixd = Pix::new(self.w, self.h, PixelDepth::Bit1).map_err(RegionError::Core)?;
        let mut pixd = pixd.try_into_mut().map_err(|_| {
            RegionError::InvalidParameters("border pix unexpectedly shared".to_string())
        })?;
        for ccb in &self.ccb {
            for j in 0..ccb.global.len() {
                let pta = ccb.global.get(j).expect("border in range");
                for k in 0..pta.len() {
                    let (x, y) = pta_get_ipt(pta, k);
                    let _ = pixd.set_pixel(x as u32, y as u32, 1);
                }
            }
        }
        Ok(pixd.into())
    }

    /// Reconstruct the image from the borders.
    ///
    /// Each component is rebuilt by seed-filling from a pixel just outside
    /// every closed border, using the border itself as a clipping mask, then
    /// XOR-ing the result into the destination.
    ///
    /// Uses [`CcBord::local`], so call
    /// [`CcBorda::step_chains_to_pix_coords`] with [`CcbCoords::Local`] first
    /// when working from step chains.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaDisplayImage2()` in `ccbord.c`
    pub fn display_image(&self) -> RegionResult<Pix> {
        let pixd = Pix::new(self.w, self.h, PixelDepth::Bit1).map_err(RegionError::Core)?;
        let mut pixd = pixd.try_into_mut().map_err(|_| {
            RegionError::InvalidParameters("image pix unexpectedly shared".to_string())
        })?;

        for (i, ccb) in self.ccb.iter().enumerate() {
            let b = ccb.boxa.get(0).ok_or_else(|| {
                RegionError::InvalidParameters(format!("component {i} has no bounding box"))
            })?;
            let (xul, yul, w, h) = (b.x, b.y, b.w, b.h);
            if ccb.local.is_empty() {
                continue; // C warns and skips
            }

            // Border pixels go into a mask one pixel larger on each side, so
            // the outside seed always has room.
            let pixc = Pix::new(w as u32 + 2, h as u32 + 2, PixelDepth::Bit1)
                .map_err(RegionError::Core)?;
            let mut pixc = pixc.try_into_mut().expect("fresh pix");
            let pixs = Pix::new(w as u32 + 2, h as u32 + 2, PixelDepth::Bit1)
                .map_err(RegionError::Core)?;
            let mut pixs = pixs.try_into_mut().expect("fresh pix");

            for j in 0..ccb.local.len() {
                let pta = ccb.local.get(j).expect("border in range");
                let n = pta.len();
                let (mut fpx, mut fpy, mut spx, mut spy) = (0i32, 0i32, 0i32, 0i32);
                for k in 0..n {
                    let (x, y) = pta_get_ipt(pta, k);
                    let _ = pixc.set_pixel((x + 1) as u32, (y + 1) as u32, 1);
                    if k == 0 {
                        fpx = x + 1;
                        fpy = y + 1;
                    } else if k == 1 {
                        spx = x + 1;
                        spy = y + 1;
                    }
                }
                let (xs, ys) = if n > 1 {
                    locate_outside_seed_pixel(fpx, fpy, spx, spy)
                } else {
                    (0, 0) // isolated c.c.
                };
                let _ = pixs.set_pixel(xs as u32, ys as u32, 1);
            }

            // Invert the border mask to turn "clip" into "fill", grow the
            // seeds into it, then invert back to get the component.
            let pixc: Pix = Pix::from(pixc).invert();
            let seed: Pix = pixs.into();
            let filled = seedfill_binary_restricted(&seed, &pixc, ConnectivityType::FourWay, 0, 0)?;
            let comp = filled.invert();

            pixd.rop_region_inplace(
                xul,
                yul,
                w as u32,
                h as u32,
                crate::core::pix::RopOp::Xor,
                &comp,
                1,
                1,
            )
            .map_err(RegionError::Core)?;
        }

        Ok(pixd.into())
    }
}

/// Trace the exterior border and every hole border of a single component.
///
/// `pixs` holds exactly one 8-connected component and `b` is its bounding box
/// in global coordinates. The component's own box is stored as given; hole
/// boxes are relative to the component.
///
/// # See also
///
/// C Leptonica: `pixGetCCBorders()` in `ccbord.c`
fn cc_borders(pixs: &Pix, b: &Box) -> RegionResult<CcBord> {
    let mut ccb = CcBord::default();
    get_outer_border(&mut ccb, pixs, b)?;

    let pixh = holes_by_filling(pixs, ConnectivityType::FourWay)?;
    if pixh.is_zero() {
        return Ok(ccb);
    }

    let (boxa, pixa) = conncomp_pixa(&pixh, ConnectivityType::FourWay)?;
    let w = pixs.width() as i32;
    for i in 0..boxa.len() {
        let bt = boxa.get(i).expect("hole box");
        let pixt = pixa.get(i).expect("hole pix");
        let ys = bt.y; // there must be a hole pixel on this raster line

        // Find a foreground pixel of the hole on its first row, then march
        // right until the first border pixel of the component.
        let Some(xh) = (0..bt.w).find(|&x| pixt.get_pixel(x as u32, 0).unwrap_or(0) == 1) else {
            continue; // C warns "no hole pixel found!" and skips
        };
        let Some(xs) =
            (xh + bt.x..w).find(|&x| pixs.get_pixel(x as u32, ys as u32).unwrap_or(0) == 1)
        else {
            continue;
        };

        // The border box is one pixel larger on each side than the hole.
        let boxe = Box::new_unchecked(bt.x - 1, bt.y - 1, bt.w + 2, bt.h + 2);
        get_hole_border(&mut ccb, pixs, &boxe, xs, ys)?;
    }

    Ok(ccb)
}

/// Trace the exterior border of a component.
///
/// The walk runs on a copy with a 1-pixel border added, so the tracer never
/// leaves the array, but the coordinates saved are those of `pixs`.
///
/// # See also
///
/// C Leptonica: `pixGetOuterBorder()` in `ccbord.c`
fn get_outer_border(ccb: &mut CcBord, pixs: &Pix, b: &Box) -> RegionResult<()> {
    let pixb = pixs.add_border(1, 0).map_err(RegionError::Core)?;
    let Some((px, py)) = next_on_pixel_in_raster(&pixb, 1, 1) else {
        return Err(RegionError::InvalidParameters(
            "no start pixel found on the component border".to_string(),
        ));
    };
    let (mut px, mut py) = (px as i32, py as i32);
    let (fpx, fpy) = (px, py);
    let mut qpos = 0i32;

    ccb.boxa.push(*b);
    ccb.start.push((px - 1) as f32, (py - 1) as f32);

    let mut pta = Pta::new();
    pta.push((px - 1) as f32, (py - 1) as f32);

    let (w, h) = (pixb.width() as i32, pixb.height() as i32);
    let Some((spx, spy)) = find_next_border_pixel(&pixb, w, h, px, py, &mut qpos) else {
        // Isolated pixel: the border is just the start point.
        ccb.local.push(pta);
        return Ok(());
    };
    pta.push((spx - 1) as f32, (spy - 1) as f32);
    px = spx;
    py = spy;

    // C ignores the return value here; a pixel already known to be on a
    // border always has a neighbour, so the search cannot fail. Stopping on
    // `None` is the same walk without relying on that.
    let cap = max_border_points(w, h);
    while let Some((npx, npy)) = find_next_border_pixel(&pixb, w, h, px, py, &mut qpos) {
        if px == fpx && py == fpy && npx == spx && npy == spy {
            break;
        }
        pta.push((npx - 1) as f32, (npy - 1) as f32);
        if pta.len() > cap {
            return Err(RegionError::InvalidParameters(format!(
                "outer border trace exceeded {cap} points without closing"
            )));
        }
        px = npx;
        py = npy;
    }

    ccb.local.push(pta);
    Ok(())
}

/// Trace one hole border, working directly on `pixs` in component-relative
/// coordinates.
///
/// # See also
///
/// C Leptonica: `pixGetHoleBorder()` in `ccbord.c`
fn get_hole_border(ccb: &mut CcBord, pixs: &Pix, b: &Box, xs: i32, ys: i32) -> RegionResult<()> {
    let (fpx, fpy) = (xs, ys);
    let mut qpos = 0i32;

    ccb.boxa.push(*b);
    ccb.start.push(xs as f32, ys as f32);

    let mut pta = Pta::new();
    pta.push(xs as f32, ys as f32);

    let (w, h) = (pixs.width() as i32, pixs.height() as i32);
    let Some((spx, spy)) = find_next_border_pixel(pixs, w, h, xs, ys, &mut qpos) else {
        return Err(RegionError::InvalidParameters(
            "isolated hole border point".to_string(),
        ));
    };
    pta.push(spx as f32, spy as f32);
    let (mut px, mut py) = (spx, spy);

    // See the note in `get_outer_border` about C ignoring the return value.
    let cap = max_border_points(w, h);
    while let Some((npx, npy)) = find_next_border_pixel(pixs, w, h, px, py, &mut qpos) {
        if px == fpx && py == fpy && npx == spx && npy == spy {
            break;
        }
        pta.push(npx as f32, npy as f32);
        if pta.len() > cap {
            return Err(RegionError::InvalidParameters(format!(
                "hole border trace exceeded {cap} points without closing"
            )));
        }
        px = npx;
        py = npy;
    }

    ccb.local.push(pta);
    Ok(())
}

/// Step to the next border pixel clockwise from the current Q position.
///
/// `qpos` grows clockwise from 0 to 7, with 0 putting Q to the left of P.
///
/// # See also
///
/// C Leptonica: `findNextBorderPixel()` in `ccbord.c`
fn find_next_border_pixel(
    pix: &Pix,
    w: i32,
    h: i32,
    px: i32,
    py: i32,
    qpos: &mut i32,
) -> Option<(i32, i32)> {
    for i in 1..8 {
        let pos = ((*qpos + i) % 8) as usize;
        let npx = px + XPOSTAB[pos];
        let npy = py + YPOSTAB[pos];
        if npx < 0 || npx >= w || npy < 0 || npy >= h {
            continue;
        }
        if pix.get_pixel_unchecked(npx as u32, npy as u32) != 0 {
            *qpos = QPOSTAB[pos];
            return Some((npx, npy));
        }
    }
    None
}

/// Pick a pixel just outside the component, given the first two border
/// pixels.
///
/// The two must be 8-adjacent. The rules assume the inside of the component
/// is on the right as the border is followed: clockwise for an exterior
/// border, counter-clockwise for a hole.
///
/// # See also
///
/// C Leptonica: `locateOutsideSeedPixel()` in `ccbord.c`
fn locate_outside_seed_pixel(fpx: i32, fpy: i32, spx: i32, spy: i32) -> (i32, i32) {
    let dx = spx - fpx;
    let dy = spy - fpy;

    if dx * dy == 1 {
        (fpx + dx, fpy)
    } else if dx * dy == -1 {
        (fpx, fpy + dy)
    } else if dx == 0 {
        (fpx + dy, fpy + dy)
    } else {
        // dy == 0
        (fpx + dx, fpy - dx)
    }
}

/// C `ptaGetIPt()`: round the stored float coordinates to integers.
fn pta_get_ipt(pta: &Pta, index: usize) -> (i32, i32) {
    let (x, y) = pta.get(index).unwrap_or((0.0, 0.0));
    ((x + 0.5) as i32, (y + 0.5) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 6x5 ring (so the component has a hole border as well as an outer
    /// one) plus an isolated pixel, which exercises the single-point border
    /// that gets an empty step chain.
    fn ring_and_dot() -> Pix {
        let pix = Pix::new(12, 10, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        for x in 1..=6u32 {
            pm.set_pixel_unchecked(x, 1, 1);
            pm.set_pixel_unchecked(x, 5, 1);
        }
        for y in 1..=5u32 {
            pm.set_pixel_unchecked(1, y, 1);
            pm.set_pixel_unchecked(6, y, 1);
        }
        pm.set_pixel_unchecked(10, 8, 1); // isolated pixel
        pm.into()
    }

    fn local_points(ccba: &CcBorda) -> Vec<Vec<(i32, i32)>> {
        let mut out = Vec::new();
        for i in 0..ccba.len() {
            let ccb = ccba.get(i).expect("component");
            for j in 0..ccb.local.len() {
                let pta = ccb.local.get(j).expect("border");
                out.push((0..pta.len()).map(|k| pta_get_ipt(pta, k)).collect());
            }
        }
        out
    }

    /// Expected borders are the verbatim output of C `pixGetAllCCBorders()`
    /// on this fixture.
    #[test]
    fn test_local_borders_match_c() {
        let ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        assert_eq!(ccba.len(), 2);

        let got = local_points(&ccba);
        assert_eq!(
            got,
            vec![
                // outer border of the ring
                vec![
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (3, 0),
                    (4, 0),
                    (5, 0),
                    (5, 1),
                    (5, 2),
                    (5, 3),
                    (5, 4),
                    (4, 4),
                    (3, 4),
                    (2, 4),
                    (1, 4),
                    (0, 4),
                    (0, 3),
                    (0, 2),
                    (0, 1),
                    (0, 0),
                ],
                // hole border of the ring
                vec![
                    (5, 1),
                    (4, 0),
                    (3, 0),
                    (2, 0),
                    (1, 0),
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    (1, 4),
                    (2, 4),
                    (3, 4),
                    (4, 4),
                    (5, 3),
                    (5, 2),
                    (5, 1),
                ],
                // the isolated pixel
                vec![(0, 0)],
            ]
        );
    }

    /// Rebuilding local coordinates from the step chains must reproduce what
    /// `from_pix` traced, including the isolated pixel whose chain is empty.
    /// Verified against C, which round-trips this fixture unchanged.
    #[test]
    fn test_step_chains_round_trip_to_local() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        let before = local_points(&ccba);

        ccba.generate_step_chains();
        ccba.step_chains_to_pix_coords(CcbCoords::Local).unwrap();
        assert_eq!(local_points(&ccba), before);

        // The reconstruction still works off the regenerated local coords.
        let rebuilt = ccba.display_image().unwrap();
        let original = ring_and_dot();
        for y in 0..10 {
            for x in 0..12 {
                assert_eq!(
                    rebuilt.get_pixel_unchecked(x, y),
                    original.get_pixel_unchecked(x, y),
                    "reconstruction at ({x}, {y})"
                );
            }
        }
    }

    /// The isolated pixel's chain is empty, and the global frame just shifts
    /// every point by the component's upper-left corner.
    #[test]
    fn test_step_chains_and_global_shift() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_step_chains();

        let dot = ccba.get(1).expect("isolated component");
        assert_eq!(dot.step.len(), 1);
        assert_eq!(dot.step.get(0).expect("chain").len(), 0);

        ccba.generate_global_locs();
        let dot = ccba.get(1).expect("isolated component");
        let b = dot.boxa.get(0).expect("box");
        assert_eq!((b.x, b.y), (10, 8));
        let g = dot.global.get(0).expect("global border");
        assert_eq!(pta_get_ipt(g, 0), (10, 8));
    }
}
