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
#[cfg(feature = "ccb-format")]
use miniz_oxide::{deflate::compress_to_vec_zlib, inflate::decompress_to_vec_zlib};

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

/// Which points [`CcBorda::generate_sp_global_locs`] keeps.
///
/// C: `CCB_SAVE_ALL_PTS` / `CCB_SAVE_TURNING_PTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CcbPoints {
    /// Every point of the single path.
    All,
    /// Only the points where the direction changes. This still describes the
    /// same polygon and is about half the size.
    Turning,
}

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
    /// Single path through the outer border and every hole border, local to
    /// the component.
    pub splocal: Pta,
    /// The same single path in global image coordinates.
    pub spglobal: Pta,
    /// The component's bitmap, which [`CcBorda::generate_single_path`] scans
    /// to find a cut from each hole out to the exterior border.
    ///
    /// `None` for a component read from a `.ccb` stream, which does not
    /// store it, as in C.
    pub pix: Option<Pix>,
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
                    // `step` is public and can also come from a `.ccb` file,
                    // so a direction outside the tables must not index it.
                    // C indexes `xpostab` unchecked and reads out of bounds.
                    let stepdir = na.get_i32(k).unwrap_or(-1);
                    let dir = usize::try_from(stepdir)
                        .ok()
                        .filter(|&d| d < XPOSTAB.len())
                        .ok_or_else(|| {
                            RegionError::InvalidParameters(format!(
                                "component {i} border {j} has step direction {stepdir}, \
                                 which is not one of 0-7"
                            ))
                        })?;
                    x += XPOSTAB[dir];
                    y += YPOSTAB[dir];
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
    let mut ccb = CcBord {
        // C `ccbCreate(pixs)` keeps the component bitmap for cut finding.
        pix: Some(pixs.clone()),
        ..CcBord::default()
    };
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

/// Upper bound on holes C will connect into the single path. Above it, the
/// cut finding is not worth the work and the outer border is used alone.
///
/// C: `NMAX_HOLES` in `ccbord.c`.
const NMAX_HOLES: usize = 150;

/// Single path representation and SVG output.
impl CcBorda {
    /// Join every border of each component into one closed path.
    ///
    /// A component with no holes just copies its exterior border. Otherwise
    /// each hole is connected to the exterior by a cut path found through the
    /// component's own pixels, and the path walks out along the cut, around
    /// the hole, and back.
    ///
    /// A hole whose cut cannot be found is dropped from the path, as in C;
    /// the borders themselves still describe the image exactly.
    ///
    /// # Errors
    ///
    /// Returns an error if a component has borders but no bitmap, which
    /// happens when the component was read from a `.ccb` stream.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaGenerateSinglePath()` in `ccbord.c`
    pub fn generate_single_path(&mut self) -> RegionResult<()> {
        for (i, ccb) in self.ccb.iter_mut().enumerate() {
            if ccb.local.is_empty() {
                continue; // C warns and skips
            }
            let nb = ccb.local.len();
            let outer = ccb.local.get(0).expect("outer border").clone();

            let mut path = Pta::new();
            if nb == 1 || nb > NMAX_HOLES + 1 {
                path.join(&outer, 0, None).map_err(RegionError::Core)?;
                ccb.splocal = path;
                continue;
            }

            let pix = ccb.pix.as_ref().ok_or_else(|| {
                RegionError::InvalidParameters(format!(
                    "component {i} has holes but no bitmap to cut through"
                ))
            })?;

            // One cut per hole, plus the point where it meets each border.
            // A hole with no cut gets (-1, -1), which no border point equals.
            let mut cuts: Vec<Pta> = Vec::with_capacity(nb - 1);
            let mut first = Pta::with_capacity(nb - 1);
            let mut last = Pta::with_capacity(nb - 1);
            for j in 1..nb {
                let boxinner = ccb.boxa.get(j).ok_or_else(|| {
                    RegionError::InvalidParameters(format!(
                        "component {i} has no bounding box for hole {j}"
                    ))
                })?;
                let cut = get_cut_path_for_hole(pix, &outer, boxinner);
                if cut.is_empty() {
                    first.push(-1.0, -1.0);
                    last.push(-1.0, -1.0);
                } else {
                    let (fx, fy) = pta_get_ipt(&cut, 0);
                    let (lx, ly) = pta_get_ipt(&cut, cut.len() - 1);
                    first.push(fx as f32, fy as f32);
                    last.push(lx as f32, ly as f32);
                }
                cuts.push(cut);
            }

            // Walk the outer border; at each point that a cut reaches, detour
            // out along the cut, around the hole, and back.
            for k in 0..outer.len() {
                let (x, y) = pta_get_ipt(&outer, k);
                if k == 0 {
                    // A cut at the first point can wait until the walk ends.
                    path.push(x as f32, y as f32);
                    continue;
                }
                let mut taken = false;
                for (j, cut) in cuts.iter().enumerate() {
                    let (xl, yl) = pta_get_ipt(&last, j);
                    if x != xl || y != yl {
                        continue;
                    }
                    taken = true;
                    let (xf, yf) = pta_get_ipt(&first, j);
                    let hole = ccb.local.get(j + 1).expect("hole border");
                    // Start the hole at the point the cut arrives on.
                    let hole = hole.cyclic_perm(xf, yf).map_err(RegionError::Core)?;
                    path.join(&cut.reverse(), 0, None)
                        .map_err(RegionError::Core)?;
                    path.join(&hole, 0, None).map_err(RegionError::Core)?;
                    path.join(cut, 0, None).map_err(RegionError::Core)?;
                    break;
                }
                if !taken {
                    path.push(x as f32, y as f32);
                }
            }
            ccb.splocal = path;
        }
        Ok(())
    }

    /// Fill in [`CcBord::spglobal`] from [`CcBord::splocal`].
    ///
    /// Unlike C, which quietly calls `ccbaGenerateSinglePath()` when a
    /// component has no single path, this reports the omission. Every
    /// generation step in this module is explicit, matching
    /// [`CcBorda::step_chains_to_pix_coords`] and `to_bytes`, which is
    /// gated behind the `ccb-format` feature.
    ///
    /// # Errors
    ///
    /// Returns an error if the single path has not been generated yet, or if
    /// a component has no bounding box.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaGenerateSPGlobalLocs()` in `ccbord.c`
    pub fn generate_sp_global_locs(&mut self, points: CcbPoints) -> RegionResult<()> {
        for (i, ccb) in self.ccb.iter_mut().enumerate() {
            if ccb.splocal.is_empty() {
                return Err(RegionError::InvalidParameters(format!(
                    "component {i} has no single path; call generate_single_path first"
                )));
            }
            let b = ccb.boxa.get(0).ok_or_else(|| {
                RegionError::InvalidParameters(format!("component {i} has no bounding box"))
            })?;
            let (xul, yul) = (b.x, b.y);
            let local = &ccb.splocal;
            let npt = local.len();
            let mut global = Pta::with_capacity(npt);

            match points {
                CcbPoints::All => {
                    for j in 0..npt {
                        let (x, y) = pta_get_ipt(local, j);
                        global.push((x + xul) as f32, (y + yul) as f32);
                    }
                }
                CcbPoints::Turning => {
                    let (mut xp, mut yp) = pta_get_ipt(local, 0);
                    global.push((xp + xul) as f32, (yp + yul) as f32);
                    if npt == 2 {
                        let (x, y) = pta_get_ipt(local, 1);
                        global.push((x + xul) as f32, (y + yul) as f32);
                    } else if npt > 2 {
                        let (x, y) = pta_get_ipt(local, 1);
                        let (mut delxp, mut delyp) = (x - xp, y - yp);
                        (xp, yp) = (x, y);
                        for j in 2..npt {
                            let (x, y) = pta_get_ipt(local, j);
                            let (delx, dely) = (x - xp, y - yp);
                            if delx != delxp || dely != delyp {
                                global.push((xp + xul) as f32, (yp + yul) as f32);
                            }
                            (xp, yp) = (x, y);
                            (delxp, delyp) = (delx, dely);
                        }
                        global.push((xp + xul) as f32, (yp + yul) as f32);
                    }
                }
            }
            ccb.spglobal = global;
        }
        Ok(())
    }

    /// Paint every pixel of the single path, using the global coordinates.
    ///
    /// [`CcBorda::generate_sp_global_locs`] must have run first; a component
    /// without a global single path is skipped, as in C.
    ///
    /// # Errors
    ///
    /// Returns an error if the destination pix cannot be created.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaDisplaySPBorder()` in `ccbord.c`
    pub fn display_sp_border(&self) -> RegionResult<Pix> {
        let pixd = Pix::new(self.w, self.h, PixelDepth::Bit1).map_err(RegionError::Core)?;
        let mut pixd = pixd.try_into_mut().map_err(|_| {
            RegionError::InvalidParameters("border pix unexpectedly shared".to_string())
        })?;
        for ccb in &self.ccb {
            for k in 0..ccb.spglobal.len() {
                let (x, y) = pta_get_ipt(&ccb.spglobal, k);
                let _ = pixd.set_pixel(x as u32, y as u32, 1);
            }
        }
        Ok(pixd.into())
    }

    /// Render the single paths as one SVG `polygon` element per component.
    ///
    /// C builds this from an `SARRAY` joined with a newline after every
    /// element, so each point sits on its own line and a lone space line
    /// follows `</svg>`.
    ///
    /// # Errors
    ///
    /// Returns an error if a component has no single path in global
    /// coordinates.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaWriteSVGString()` in `ccbord.c`. `ccbaWriteSVG()`
    /// only writes the string to a file and is not ported.
    pub fn to_svg_string(&self) -> RegionResult<String> {
        let mut svg = String::from("<?xml version=\"1.0\" encoding=\"iso-8859-1\"?>\n");
        svg.push_str(
            "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 20000303 Stylable//EN\" \
             \"http://www.w3.org/TR/2000/03/WD-SVG-20000303/DTD/svg-20000303-stylable.dtd\">\n",
        );
        svg.push_str("<svg>\n");
        for (i, ccb) in self.ccb.iter().enumerate() {
            if ccb.spglobal.is_empty() {
                return Err(RegionError::InvalidParameters(format!(
                    "component {i} has no global single path; \
                     call generate_sp_global_locs first"
                )));
            }
            svg.push_str("<polygon style=\"stroke-width:1;stroke:black;\" points=\"\n");
            for k in 0..ccb.spglobal.len() {
                let (x, y) = pta_get_ipt(&ccb.spglobal, k);
                svg.push_str(&format!("{x},{y}\n"));
            }
            svg.push_str("\" />\n");
        }
        svg.push_str("</svg>\n \n");
        Ok(svg)
    }
}

/// Find a short path from inside a hole out to the exterior border, staying
/// on foreground pixels of the component.
///
/// Tries up, then down, then left, then right from the middle of the hole's
/// bounding box: run to the first foreground pixel, keep going while the run
/// stays foreground, and accept the direction if the run ends on `outer`.
/// Returns an empty path when no direction works, in which case C drops the
/// hole from the single path.
///
/// # See also
///
/// C Leptonica: `getCutPathForHole()` in `ccbord.c`
fn get_cut_path_for_hole(pix: &Pix, outer: &Pta, boxinner: &Box) -> Pta {
    let (w, h) = (pix.width() as i32, pix.height() as i32);
    let xmid = boxinner.x + boxinner.w / 2;
    let ymid = boxinner.y + boxinner.h / 2;

    let on = |x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || x >= w || y >= h {
            return false;
        }
        pix.get_pixel(x as u32, y as u32).unwrap_or(0) == 1
    };

    // (dx, dy) for up, down, left, right, in C's order of preference.
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let mut ptac = Pta::new();
        // Run to the first foreground pixel.
        let (mut x, mut y) = (xmid, ymid);
        while x >= 0 && y >= 0 && x < w && y < h {
            if on(x, y) {
                ptac.push(x as f32, y as f32);
                break;
            }
            x += dx;
            y += dy;
        }
        // Then stay on foreground for as long as it lasts.
        x += dx;
        y += dy;
        while on(x, y) {
            ptac.push(x as f32, y as f32);
            x += dx;
            y += dy;
        }
        if ptac.is_empty() {
            continue;
        }
        let (xl, yl) = pta_get_ipt(&ptac, ptac.len() - 1);
        if outer.contains_pt(xl, yl) {
            return ptac;
        }
    }

    // Sometimes there is nothing.
    Pta::new()
}

/// Bytes of the `.ccb` header: `"ccba: %7d cc\n"` is 17 characters, and C
/// takes an 18th byte from its `snprintf` buffer, which is the NUL.
#[cfg(feature = "ccb-format")]
const CCB_HEADER_SIZE: usize = 18;

/// Cursor over an uncompressed `.ccb` stream.
///
/// C's `ccbaReadStream()` trusts the counts in the file and `memcpy`s past
/// the end of a truncated one; every read here is bounds-checked instead.
#[cfg(feature = "ccb-format")]
struct CcbReader<'a> {
    data: &'a [u8],
    offset: usize,
}

#[cfg(feature = "ccb-format")]
impl<'a> CcbReader<'a> {
    fn take(&mut self, n: usize) -> RegionResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(n)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| {
                RegionError::InvalidParameters(format!(
                    "ccba stream is truncated: wanted {n} bytes at offset {}, have {}",
                    self.offset,
                    self.data.len()
                ))
            })?;
        let out = &self.data[self.offset..end];
        self.offset = end;
        Ok(out)
    }

    fn take4(&mut self) -> RegionResult<[u8; 4]> {
        Ok(self.take(4)?.try_into().expect("four bytes"))
    }

    fn byte(&mut self) -> RegionResult<u8> {
        Ok(self.take(1)?[0])
    }
}

/// Serialization of the C `.ccb` format.
///
/// The stream holds only the step chain representation: image size, and per
/// component its bounding box, border start points, and step chains. Local
/// and global pixel coordinates, and the bounding boxes of hole borders, are
/// not written; rebuild the coordinates with
/// [`CcBorda::step_chains_to_pix_coords`] after reading.
///
/// C's file-level `ccbaWrite()` / `ccbaRead()` are thin `fopen`/`fclose`
/// wrappers and are not ported: use
/// `std::fs::write(path, ccba.to_bytes()?)` and
/// `CcBorda::from_bytes(&std::fs::read(path)?)`.
#[cfg(feature = "ccb-format")]
impl CcBorda {
    /// Serialize to the zlib-compressed C `.ccb` format.
    ///
    /// Unlike C, which quietly calls `ccbaGenerateStepChains()` when a
    /// component has none, this reports the omission, matching
    /// [`CcBorda::step_chains_to_pix_coords`].
    ///
    /// # Errors
    ///
    /// Returns an error if the step chains have not been generated yet, or if
    /// a component has no bounding box.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaWriteStream()` in `ccbord.c`
    pub fn to_bytes(&self) -> RegionResult<Vec<u8>> {
        let mut buf = format!("ccba: {:7} cc\n", self.ccb.len()).into_bytes();
        buf.resize(CCB_HEADER_SIZE, 0);
        buf.extend_from_slice(&self.w.to_le_bytes());
        buf.extend_from_slice(&self.h.to_le_bytes());

        for (i, ccb) in self.ccb.iter().enumerate() {
            let b = ccb.boxa.get(0).ok_or_else(|| {
                RegionError::InvalidParameters(format!("component {i} has no bounding box"))
            })?;
            if ccb.step.is_empty() {
                return Err(RegionError::InvalidParameters(format!(
                    "component {i} has no step chains; call generate_step_chains first"
                )));
            }
            // C writes w and h too, though reconstruction does not need them.
            for v in [b.x, b.y, b.w, b.h, ccb.step.len() as i32] {
                buf.extend_from_slice(&v.to_le_bytes());
            }

            for j in 0..ccb.step.len() {
                let na = ccb.step.get(j).expect("border in range");
                let (startx, starty) = pta_get_ipt(&ccb.start, j);
                buf.extend_from_slice(&startx.to_le_bytes());
                buf.extend_from_slice(&starty.to_le_bytes());

                // Two steps per byte, the earlier one in the high nibble.
                let mut bval = 0u8;
                for k in 0..na.len() {
                    // 8 is the terminator, so only 0-7 are representable.
                    let stepdir = na.get_i32(k).unwrap_or(-1);
                    let val = u8::try_from(stepdir)
                        .ok()
                        .filter(|&v| v < 8)
                        .ok_or_else(|| {
                            RegionError::InvalidParameters(format!(
                                "component {i} border {j} has step direction {stepdir}, \
                             which is not one of 0-7"
                            ))
                        })?;
                    if k % 2 == 0 {
                        bval = val << 4;
                    } else {
                        bval |= val;
                        buf.push(bval);
                    }
                }
                // 8 is not a step direction, so it terminates: after an odd
                // count the last step keeps the high nibble, otherwise both
                // nibbles are the sentinel.
                buf.push(if na.len() % 2 == 1 { bval | 0x8 } else { 0x88 });
            }
        }

        Ok(compress_to_vec_zlib(&buf, 6))
    }

    /// Parse the zlib-compressed C `.ccb` format.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not zlib, does not start with the
    /// `ccba:` magic, or ends in the middle of a record.
    ///
    /// # See also
    ///
    /// C Leptonica: `ccbaReadStream()` in `ccbord.c`
    pub fn from_bytes(data: &[u8]) -> RegionResult<Self> {
        let data = decompress_to_vec_zlib(data).map_err(|e| {
            RegionError::InvalidParameters(format!("ccba stream is not zlib data: {e}"))
        })?;
        let mut r = CcbReader {
            data: &data,
            offset: 0,
        };

        let header = r.take(CCB_HEADER_SIZE)?;
        let ncc = std::str::from_utf8(&header[..CCB_HEADER_SIZE - 1])
            .ok()
            .and_then(|s| s.strip_prefix("ccba:"))
            .and_then(|s| s.strip_suffix(" cc\n"))
            .and_then(|s| s.trim().parse::<usize>().ok())
            .ok_or_else(|| {
                RegionError::InvalidParameters("data is not a ccba stream".to_string())
            })?;

        let w = u32::from_le_bytes(r.take4()?);
        let h = u32::from_le_bytes(r.take4()?);

        // The counts come from the file, so they are not trusted for
        // preallocation; the bounds checks in `take` are what limit growth.
        let mut ccba = Self {
            w,
            h,
            ccb: Vec::new(),
        };
        for _ in 0..ncc {
            let mut ccb = CcBord::default();
            let bx = i32::from_le_bytes(r.take4()?);
            let by = i32::from_le_bytes(r.take4()?);
            let bw = i32::from_le_bytes(r.take4()?);
            let bh = i32::from_le_bytes(r.take4()?);
            ccb.boxa
                .push(Box::new(bx, by, bw, bh).map_err(RegionError::Core)?);

            let nb = i32::from_le_bytes(r.take4()?);
            let nb = usize::try_from(nb).map_err(|_| {
                RegionError::InvalidParameters(format!("negative border count {nb}"))
            })?;
            for _ in 0..nb {
                let startx = i32::from_le_bytes(r.take4()?);
                let starty = i32::from_le_bytes(r.take4()?);
                ccb.start.push(startx as f32, starty as f32);

                let mut na = Numa::new();
                'steps: loop {
                    let bval = r.byte()?;
                    for nib in [bval >> 4, bval & 0xf] {
                        match nib {
                            // 8 is not a direction, so it terminates the chain.
                            8 => break 'steps,
                            0..=7 => na.push(f32::from(nib)),
                            _ => {
                                return Err(RegionError::InvalidParameters(format!(
                                    "ccba stream has step direction {nib}, \
                                     which is not one of 0-7"
                                )));
                            }
                        }
                    }
                }
                ccb.step.push(na);
            }
            ccba.ccb.push(ccb);
        }

        Ok(ccba)
    }
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

    /// The uncompressed `.ccb` payload C writes for [`ring_and_dot`], taken
    /// verbatim from `ccbaWrite()` piped through `zlibUncompress()`.
    ///
    /// It exercises two components, a hole border, and both an even-length
    /// and an empty step chain.
    #[cfg(feature = "ccb-format")]
    const C_STREAM_RING_AND_DOT: &[u8] = &[
        0x63, 0x63, 0x62, 0x61, 0x3a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x32, 0x20, 0x63,
        0x63, 0x0a, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x44, 0x46, 0x66, 0x60, 0x00,
        0x00, 0x22, 0x22, 0x88, 0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x00, 0x76,
        0x65, 0x44, 0x43, 0x22, 0x88, 0x0a, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x88,
    ];

    /// A 3-pixel L, the smallest shape whose step chain has odd length, so the
    /// stream ends with the `0xz8` half-byte terminator rather than `0x88`.
    /// Steps are `4 7 2`, packed as `0x47` then `0x2 | 0x8`.
    #[cfg(feature = "ccb-format")]
    const C_STREAM_L_TRIOMINO: &[u8] = &[
        0x63, 0x63, 0x62, 0x61, 0x3a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x31, 0x20, 0x63,
        0x63, 0x0a, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x47, 0x28,
    ];

    #[cfg(feature = "ccb-format")]
    fn l_triomino() -> Pix {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();
        let mut pm = pix.try_into_mut().unwrap();
        pm.set_pixel_unchecked(1, 1, 1);
        pm.set_pixel_unchecked(2, 1, 1);
        pm.set_pixel_unchecked(1, 2, 1);
        pm.into()
    }

    #[cfg(feature = "ccb-format")]
    fn inflate(data: &[u8]) -> Vec<u8> {
        miniz_oxide::inflate::decompress_to_vec_zlib(data).expect("zlib data")
    }

    #[cfg(feature = "ccb-format")]
    fn deflate(data: &[u8]) -> Vec<u8> {
        miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
    }

    /// Compare against the uncompressed payload, not the file: the compressed
    /// bytes depend on the deflate implementation, the payload does not.
    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_to_bytes_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_step_chains();
        let written = ccba.to_bytes().expect("to_bytes");
        assert_eq!(inflate(&written), C_STREAM_RING_AND_DOT);
    }

    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_to_bytes_odd_step_chain_matches_c() {
        let mut ccba = CcBorda::from_pix(&l_triomino()).unwrap();
        ccba.generate_step_chains();
        let written = ccba.to_bytes().expect("to_bytes");
        assert_eq!(inflate(&written), C_STREAM_L_TRIOMINO);
    }

    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_to_bytes_without_step_chains_is_an_error() {
        let ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        assert!(ccba.to_bytes().is_err());
    }

    /// Parse C's own stream, so the reader is checked against C's writer
    /// rather than only against our own.
    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_from_bytes_reads_c_stream() {
        let ccba = CcBorda::from_bytes(&deflate(C_STREAM_RING_AND_DOT)).expect("from_bytes");
        assert_eq!((ccba.width(), ccba.height()), (12, 10));
        assert_eq!(ccba.len(), 2);

        let ring = ccba.get(0).expect("ring");
        assert_eq!(ring.boxa.len(), 1, "hole boxes are not serialized");
        let b = ring.boxa.get(0).unwrap();
        assert_eq!((b.x, b.y, b.w, b.h), (1, 1, 6, 5));
        assert_eq!(ring.step.len(), 2);
        assert_eq!(
            ring.step.get(0).unwrap().as_slice(),
            [
                4., 4., 4., 4., 4., 6., 6., 6., 6., 0., 0., 0., 0., 0., 2., 2., 2., 2.
            ]
        );
        assert_eq!(
            ring.step.get(1).unwrap().as_slice(),
            [1., 0., 0., 0., 7., 6., 6., 5., 4., 4., 4., 3., 2., 2.]
        );
        assert_eq!(pta_get_ipt(&ring.start, 0), (0, 0));
        assert_eq!(pta_get_ipt(&ring.start, 1), (5, 1));

        let dot = ccba.get(1).expect("dot");
        assert_eq!(dot.step.len(), 1);
        assert!(dot.step.get(0).unwrap().is_empty());

        // Only the step representation survives; coordinates are rebuilt.
        assert!(ring.local.is_empty() && ring.global.is_empty());
    }

    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_from_bytes_reads_odd_step_chain() {
        let ccba = CcBorda::from_bytes(&deflate(C_STREAM_L_TRIOMINO)).expect("from_bytes");
        let ccb = ccba.get(0).expect("component");
        assert_eq!(ccb.step.get(0).unwrap().as_slice(), [4., 7., 2.]);
    }

    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_from_bytes_rejects_foreign_data() {
        let not_ccba = deflate(b"nope: not a ccba stream at all");
        assert!(CcBorda::from_bytes(&not_ccba).is_err());
        assert!(CcBorda::from_bytes(b"not even zlib").is_err());
        // Truncated in the middle of a component record.
        let short = deflate(&C_STREAM_RING_AND_DOT[..40]);
        assert!(CcBorda::from_bytes(&short).is_err());
    }

    /// The round trip is what `prog/ccbord_reg.c` checks 3 and 4 rely on: the
    /// borders drawn after a write/read must be identical to the originals.
    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_round_trip_reproduces_borders_and_image() {
        let pixs = ring_and_dot();
        let mut ccba = CcBorda::from_pix(&pixs).unwrap();
        ccba.generate_step_chains();
        ccba.step_chains_to_pix_coords(CcbCoords::Global).unwrap();
        let border = ccba.display_border().unwrap();

        let mut back = CcBorda::from_bytes(&ccba.to_bytes().unwrap()).unwrap();
        back.step_chains_to_pix_coords(CcbCoords::Global).unwrap();
        assert!(back.display_border().unwrap().equals(&border));

        back.step_chains_to_pix_coords(CcbCoords::Local).unwrap();
        assert!(back.display_image().unwrap().equals(&pixs));
    }

    /// 9 is neither a direction (0-7) nor the terminator (8). C would index
    /// `xpostab` out of bounds; this must be an error, not a panic.
    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_from_bytes_rejects_invalid_step_direction() {
        let mut stream = C_STREAM_L_TRIOMINO.to_vec();
        let steps = stream.len() - 2; // the byte holding steps 4 and 7
        assert_eq!(stream[steps], 0x47);
        stream[steps] = 0x97;
        assert!(CcBorda::from_bytes(&deflate(&stream)).is_err());
    }

    /// `CcBord::step` is public, so a direction outside the tables can reach
    /// `step_chains_to_pix_coords` without going through `from_bytes`.
    #[test]
    fn test_step_chains_reject_invalid_step_direction() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_step_chains();
        ccba.ccb[0].step.get_mut(0).unwrap().set(0, 9.0).unwrap();
        assert!(ccba.step_chains_to_pix_coords(CcbCoords::Local).is_err());

        ccba.ccb[0].step.get_mut(0).unwrap().set(0, -1.0).unwrap();
        assert!(ccba.step_chains_to_pix_coords(CcbCoords::Local).is_err());
    }

    /// The format reserves 8 as the terminator, so a chain that a reader
    /// would reject must not be writable either.
    #[test]
    #[cfg(feature = "ccb-format")]
    fn test_to_bytes_rejects_invalid_step_direction() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_step_chains();
        ccba.ccb[0].step.get_mut(0).unwrap().set(0, 9.0).unwrap();
        assert!(ccba.to_bytes().is_err());
    }

    /// C `ccbaGenerateSinglePath()` on the ring of [`ring_and_dot`]. The cut
    /// path out to the hole and back repeats points, e.g. `(3, 0)` twice.
    const C_SP_LOCAL_RING: &[(i32, i32)] = &[
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
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
        (4, 0),
        (3, 0),
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
    ];

    /// The ring's single path in global coordinates, turning points only.
    const C_SP_GLOBAL_TURNING_RING: &[(i32, i32)] = &[
        (1, 1),
        (4, 1),
        (4, 1),
        (2, 1),
        (1, 2),
        (1, 4),
        (2, 5),
        (5, 5),
        (6, 4),
        (6, 2),
        (5, 1),
        (4, 1),
        (4, 1),
        (6, 1),
        (6, 5),
        (1, 5),
        (1, 1),
    ];

    /// Pixels C `ccbaDisplaySPBorder()` sets after
    /// `generate_sp_global_locs(Turning)`, in raster order.
    const C_SP_BORDER_PIXELS: &[(i32, i32)] = &[
        (1, 1),
        (2, 1),
        (4, 1),
        (5, 1),
        (6, 1),
        (1, 2),
        (6, 2),
        (1, 4),
        (6, 4),
        (1, 5),
        (2, 5),
        (5, 5),
        (6, 5),
        (10, 8),
    ];

    fn sp_points(pta: &Pta) -> Vec<(i32, i32)> {
        (0..pta.len()).map(|k| pta_get_ipt(pta, k)).collect()
    }

    /// The exact SVG C emits for [`ring_and_dot`]. `sarrayToString()` puts
    /// every array element on its own line, so each point is one line, and
    /// the lone space element becomes a space line after `</svg>`.
    fn c_svg_ring_and_dot() -> String {
        let mut s = String::from("<?xml version=\"1.0\" encoding=\"iso-8859-1\"?>\n");
        s.push_str(
            "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 20000303 Stylable//EN\" \
             \"http://www.w3.org/TR/2000/03/WD-SVG-20000303/DTD/svg-20000303-stylable.dtd\">\n",
        );
        s.push_str("<svg>\n");
        for pts in [C_SP_GLOBAL_TURNING_RING, &[(10, 8)]] {
            s.push_str("<polygon style=\"stroke-width:1;stroke:black;\" points=\"\n");
            for (x, y) in pts {
                s.push_str(&format!("{x},{y}\n"));
            }
            s.push_str("\" />\n");
        }
        s.push_str("</svg>\n \n");
        s
    }

    #[test]
    fn test_single_path_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_single_path().expect("generate_single_path");
        assert_eq!(sp_points(&ccba.get(0).unwrap().splocal), C_SP_LOCAL_RING);
        // An isolated pixel has one border of one point and no hole to cut to.
        assert_eq!(sp_points(&ccba.get(1).unwrap().splocal), [(0, 0)]);
    }

    #[test]
    fn test_sp_global_all_points_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_single_path().unwrap();
        ccba.generate_sp_global_locs(CcbPoints::All)
            .expect("generate_sp_global_locs");

        // The ring's box starts at (1, 1), so every local point shifts by it.
        let expected: Vec<_> = C_SP_LOCAL_RING
            .iter()
            .map(|&(x, y)| (x + 1, y + 1))
            .collect();
        assert_eq!(sp_points(&ccba.get(0).unwrap().spglobal), expected);
        assert_eq!(sp_points(&ccba.get(1).unwrap().spglobal), [(10, 8)]);
    }

    #[test]
    fn test_sp_global_turning_points_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_single_path().unwrap();
        ccba.generate_sp_global_locs(CcbPoints::Turning).unwrap();
        assert_eq!(
            sp_points(&ccba.get(0).unwrap().spglobal),
            C_SP_GLOBAL_TURNING_RING
        );
        assert_eq!(sp_points(&ccba.get(1).unwrap().spglobal), [(10, 8)]);
    }

    #[test]
    fn test_display_sp_border_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_single_path().unwrap();
        ccba.generate_sp_global_locs(CcbPoints::Turning).unwrap();
        let pixd = ccba.display_sp_border().expect("display_sp_border");

        let mut set = Vec::new();
        for y in 0..pixd.height() {
            for x in 0..pixd.width() {
                if pixd.get_pixel(x, y).unwrap_or(0) != 0 {
                    set.push((x as i32, y as i32));
                }
            }
        }
        assert_eq!(set, C_SP_BORDER_PIXELS);
    }

    #[test]
    fn test_svg_string_matches_c() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        ccba.generate_single_path().unwrap();
        ccba.generate_sp_global_locs(CcbPoints::Turning).unwrap();
        let svg = ccba.to_svg_string().expect("to_svg_string");
        assert_eq!(svg, c_svg_ring_and_dot());
        assert_eq!(svg.len(), 391, "C emits 391 bytes for this fixture");
    }

    #[test]
    fn test_sp_global_without_single_path_is_an_error() {
        let mut ccba = CcBorda::from_pix(&ring_and_dot()).unwrap();
        assert!(ccba.generate_sp_global_locs(CcbPoints::All).is_err());
    }
}
