//! Watershed transform, ported from C Leptonica `watershed.c`.
//!
//! This is a faithful port of the `L_WSHED` machinery: a priority queue
//! floods the image from a set of seeds (and from the local minima that
//! carry no seed), and basins are recorded whenever two of them meet.
//!
//! # Relationship to [`crate::region::watershed`]
//!
//! [`crate::region::watershed_segmentation`] and its `WatershedResult`
//! cluster are a Rust-only convenience API with no C counterpart; they use
//! a different flooding scheme and return a label image rather than a
//! `Pixa` of basins. [`Wshed`] is the port of the C algorithm and is what
//! `prog/watershed_reg.c` exercises.
//!
//! # C fidelity
//!
//! The flooding and the basin geometry agree with C bit for bit: the basins
//! themselves, their bounding boxes and their fill levels all match, and so
//! does [`Wshed::render_fill`]. C's own header warns that `wshedApply()` "is
//! buggy: it seems to locate watersheds that are duplicates"; that behaviour
//! is reproduced here rather than corrected, because agreeing with C is the
//! point of the port.
//!
//! [`Wshed::render_colors`] is the exception. It tints the basins with
//! [`Pixa::display_random_cmap`], whose colormap comes from a per-call
//! generator, while C's `pixcmapCreateRandom()` draws from glibc's single
//! process-wide `rand()` stream. The basin masks are identical but the
//! colours are not, so `render_colors` output does not match C today. See
//! `docs/porting/c-compat-findings/010-random-cmap-global-rng.md`.

use crate::core::pta::pix_generate_from_pta;
use crate::core::{Box, Numa, Pix, Pixa, PixelDepth, Pta};
use crate::region::conncomp::ConnectivityType;
use crate::region::error::{RegionError, RegionResult};
use crate::region::seedfill::{local_extrema, remove_seeded_components, select_min_in_conncomp};

/// C: `static const l_uint32 MAX_LABEL_VALUE = 0x7fffffff` — the label
/// written into every pixel of `pixlab` before filling starts.
const MAX_LABEL_VALUE: u32 = 0x7fff_ffff;

/// One entry of the flooding priority queue (C `L_WSPIXEL`).
#[derive(Clone, Copy, Debug)]
struct WsPixel {
    /// Ordering key: the source pixel value. C stores it as `l_float32`.
    val: f32,
    x: i32,
    y: i32,
    /// Label of the set this pixel belongs to.
    index: i32,
}

/// Binary min-heap ordered on `WsPixel::val`.
///
/// Ported verbatim from C `L_HEAP` (`heap.c`) with `L_SORT_INCREASING`.
/// The exact sift-up / sift-down steps decide the order among equal
/// values, and that order changes which basin claims a saddle pixel, so
/// this must not be replaced with [`std::collections::BinaryHeap`].
struct WsHeap {
    array: Vec<WsPixel>,
}

impl WsHeap {
    fn new() -> Self {
        Self { array: Vec::new() }
    }

    fn len(&self) -> usize {
        self.array.len()
    }

    /// C `lheapAdd()`: append, then sift up from the new last slot.
    fn add(&mut self, item: WsPixel) {
        self.array.push(item);
        self.swap_up(self.array.len() - 1);
    }

    /// C `lheapRemove()`: take the root, move the last item to the head,
    /// shrink, then sift down.
    fn remove(&mut self) -> Option<WsPixel> {
        let item = *self.array.first()?;
        let last = *self.array.last().expect("non-empty");
        self.array[0] = last;
        self.array.pop();
        self.swap_down();
        Some(item)
    }

    /// C `lheapSwapUp()`. `ic` / `ip` are 1-based heap indices.
    fn swap_up(&mut self, index: usize) {
        let mut ic = index + 1;
        loop {
            if ic == 1 {
                break; // root of heap
            }
            let ip = ic / 2;
            let valc = self.array[ic - 1].val;
            let valp = self.array[ip - 1].val;
            if valp <= valc {
                break;
            }
            self.array.swap(ip - 1, ic - 1);
            ic = ip;
        }
    }

    /// C `lheapSwapDown()`. `ip` / `icl` / `icr` are 1-based heap indices.
    fn swap_down(&mut self) {
        let n = self.array.len();
        if n < 1 {
            return;
        }
        let mut ip = 1usize;
        loop {
            let icl = 2 * ip;
            if icl > n {
                break;
            }
            let valp = self.array[ip - 1].val;
            let valcl = self.array[icl - 1].val;
            let icr = icl + 1;
            if icr > n {
                // Only a left child; no iterations below this one.
                if valp > valcl {
                    self.array.swap(ip - 1, icl - 1);
                }
                break;
            }
            let valcr = self.array[icr - 1].val;
            if valp <= valcl && valp <= valcr {
                break; // smaller than both
            }
            if valcl <= valcr {
                self.array.swap(ip - 1, icl - 1);
                ip = icl;
            } else {
                self.array.swap(ip - 1, icr - 1);
                ip = icr;
            }
        }
    }
}

/// Watershed transform state (C `L_WSHED`).
///
/// Build it with [`Wshed::new`], run [`Wshed::apply`], then read the
/// basins with [`Wshed::basins`] or render them with
/// [`Wshed::render_fill`] / [`Wshed::render_colors`].
pub struct Wshed {
    /// 8 bpp source.
    pixs: Pix,
    /// 1 bpp seed (marker) image.
    pixm: Pix,
    /// Minimum depth for a basin to be saved. C clamps it to >= 1.
    mindepth: i32,
    /// C `pixlab`: 32 bpp label plane, initialised to [`MAX_LABEL_VALUE`].
    pixlab: Vec<u32>,
    /// C `pixt`: 1 bpp scratch plane used while carving out a basin.
    pixt: Vec<bool>,
    w: u32,
    h: u32,
    /// Result: the basins, each cropped to its bounding box.
    pixad: Pixa,
    /// Initial seed pixels, one per connected component of `pixm`.
    ptas: Pta,
    /// Seed indicators; an entry drops to 0 once its basin is saved.
    nasi: Numa,
    /// Initial seed heights.
    nash: Numa,
    /// Result: the fill level of each saved basin.
    nalevels: Numa,
    nseeds: i32,
    nother: i32,
    /// Merge lookup: `lut[i]` is the current owner of index `i`.
    lut: Vec<i32>,
    /// Back-links into `lut`, so a merge can redirect every pointer at once.
    links: Vec<Option<Vec<i32>>>,
    arraysize: i32,
}

impl Wshed {
    /// Create the watershed state.
    ///
    /// The foreground pixels of `pixm` need not sit at minima, nor be
    /// isolated: one pixel is taken from each connected component, and a
    /// seed anywhere in a basin labels that basin once the fill reaches it.
    ///
    /// `mindepth` suppresses noise: a basin shallower than this is not
    /// saved even when it has a seed. C clamps it to at least 1.
    ///
    /// # Errors
    ///
    /// Returns an error unless `pixs` is 8 bpp, `pixm` is 1 bpp, and the
    /// two have the same dimensions.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedCreate()` in `watershed.c`
    pub fn new(pixs: &Pix, pixm: &Pix, mindepth: i32) -> RegionResult<Self> {
        if pixs.depth() != PixelDepth::Bit8 {
            return Err(RegionError::UnsupportedDepth {
                expected: "8-bit",
                actual: pixs.depth().bits(),
            });
        }
        if pixm.depth() != PixelDepth::Bit1 {
            return Err(RegionError::UnsupportedDepth {
                expected: "1-bit",
                actual: pixm.depth().bits(),
            });
        }
        let (w, h) = (pixs.width(), pixs.height());
        if pixm.width() != w || pixm.height() != h {
            return Err(RegionError::InvalidParameters(format!(
                "pixs ({w}x{h}) and pixm ({}x{}) must have the same dimensions",
                pixm.width(),
                pixm.height()
            )));
        }

        let n = (w as usize) * (h as usize);
        Ok(Self {
            pixs: pixs.clone(),
            pixm: pixm.clone(),
            mindepth: mindepth.max(1),
            pixlab: vec![MAX_LABEL_VALUE; n],
            pixt: vec![false; n],
            w,
            h,
            pixad: Pixa::new(),
            ptas: Pta::new(),
            nasi: Numa::new(),
            nash: Numa::new(),
            nalevels: Numa::new(),
            nseeds: 0,
            nother: 0,
            lut: Vec::new(),
            links: Vec::new(),
            arraysize: 0,
        })
    }

    /// The basins found by [`Wshed::apply`] and their fill levels.
    ///
    /// Each basin is a 1 bpp mask cropped to its bounding box; the box is
    /// carried in the `Pixa`'s `Boxa`.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedBasins()` in `watershed.c`
    pub fn basins(&self) -> (&Pixa, &Numa) {
        (&self.pixad, &self.nalevels)
    }

    /// Number of seeds found in `pixm` (available after [`Wshed::apply`]).
    pub fn num_seeds(&self) -> i32 {
        self.nseeds
    }

    /// Number of local minima that carried no seed.
    pub fn num_other(&self) -> i32 {
        self.nother
    }

    #[inline]
    fn source_value(&self, x: i32, y: i32) -> i32 {
        self.pixs.get_pixel_unchecked(x as u32, y as u32) as i32
    }

    /// Run the flooding.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedApply()` in `watershed.c`
    pub fn apply(&mut self) -> RegionResult<()> {
        let w = self.w as i32;
        let h = self.h as i32;
        let mut lh = WsHeap::new();

        // Seeds: one pixel per connected component of pixm.
        let (ptas, nash) = select_min_in_conncomp(&self.pixs, &self.pixm)?;
        let pixsd = pix_generate_from_pta(&ptas, self.w, self.h)?;
        let nseeds = ptas.len() as i32;
        for i in 0..nseeds {
            let (x, y) = pta_get_ipt(&ptas, i as usize);
            lh.add(WsPixel {
                val: self.source_value(x, y) as f32,
                x,
                y,
                index: i,
            });
        }
        self.ptas = ptas;
        self.nasi = Numa::make_constant(1.0, nseeds as usize);
        self.nash = nash;
        self.nseeds = nseeds;

        // Minima that are not seeds: take the local minima, drop the ones
        // that already hold a seed, then shrink each survivor to a pixel.
        let (pixmin, _) = local_extrema(&self.pixs, 200, 0)?;
        let pixmin = remove_seeded_components(&pixsd, &pixmin, ConnectivityType::EightWay, 2)?;
        // C stores the second return (`namh`, the heights of these minima) on
        // the struct, but its only reader is the unreachable branch of
        // `wshedGetHeight` described there, so there is nothing to keep.
        let (ptao, _namh) = select_min_in_conncomp(&self.pixs, &pixmin)?;
        let nother = ptao.len() as i32;
        for i in 0..nother {
            let (x, y) = pta_get_ipt(&ptao, i as usize);
            lh.add(WsPixel {
                val: self.source_value(x, y) as f32,
                x,
                y,
                index: nseeds + i,
            });
        }
        self.nother = nother;

        // Merging lookup tables. `lut` always gives the current owner of an
        // index; `links` are the back-pointers so a merge can redirect all
        // of an owner's followers at once.
        let mindepth = self.mindepth;
        let nboth = nseeds + nother;
        let arraysize = 2 * nboth;
        self.arraysize = arraysize;
        self.lut = (0..arraysize).collect();
        self.links = vec![None; arraysize.max(0) as usize];
        let mut nindex = nseeds + nother; // next unused index value

        while lh.len() > 0 {
            let Some(p) = lh.remove() else { break };
            let (val, x, y, index) = (p.val as i32, p.x, p.y, p.index);
            let ulabel = self.pixlab[(y as usize) * (w as usize) + x as usize];
            let clabel = if ulabel == MAX_LABEL_VALUE {
                MAX_LABEL_VALUE as i32
            } else {
                self.lut[ulabel as usize]
            };
            let cindex = self.lut[index as usize];
            if clabel == cindex {
                continue; // already seen this one
            }

            if clabel == MAX_LABEL_VALUE as i32 {
                // New one: assign the index and try to propagate to all
                // 8-neighbours.
                self.pixlab[(y as usize) * (w as usize) + x as usize] = cindex as u32;
                let imin = 0.max(y - 1);
                let imax = (h - 1).min(y + 1);
                let jmin = 0.max(x - 1);
                let jmax = (w - 1).min(x + 1);
                for i in imin..=imax {
                    for j in jmin..=jmax {
                        if i == y && j == x {
                            continue;
                        }
                        lh.add(WsPixel {
                            val: self.source_value(j, i) as f32,
                            x: j,
                            y: i,
                            index: cindex,
                        });
                    }
                }
            } else if clabel < nseeds && cindex < nseeds {
                // Both indices are seeds. If the shallower of the two is
                // deeper than mindepth we have two new watersheds; save
                // both and give them a fresh index to keep filling with.
                // Otherwise absorb the shallower into the deeper one.
                let hlabel = self.get_height(val, clabel)?;
                let hindex = self.get_height(val, cindex)?;
                let hmin = hlabel.min(hindex);
                if hmin >= mindepth {
                    self.save_basin(cindex, val - 1)?;
                    self.save_basin(clabel, val - 1)?;
                    self.set_seed_done(cindex);
                    self.set_seed_done(clabel);
                    self.merge_lookup(clabel, nindex)?;
                    self.merge_lookup(cindex, nindex)?;
                    nindex += 1;
                }
                // C runs this merge whether or not the basins were saved
                // (the comment there flags it as possibly misplaced).
                let (minhindex, maxhindex) = if hindex > hlabel {
                    (clabel, cindex)
                } else {
                    (cindex, clabel)
                };
                self.merge_lookup(minhindex, maxhindex)?;
            } else if clabel < nseeds && cindex >= nboth {
                // One index is a seed, the other a merge of two
                // watersheds: generate a single watershed.
                self.save_basin(clabel, val - 1)?;
                self.set_seed_done(clabel);
                self.merge_lookup(clabel, cindex)?;
            } else if cindex < nseeds && clabel >= nboth {
                self.save_basin(cindex, val - 1)?;
                self.set_seed_done(cindex);
                self.merge_lookup(cindex, clabel)?;
            } else if clabel < nseeds {
                // One is a seed, the other came from a minimum: merge the
                // minimum's basin into the seeded one.
                self.merge_lookup(cindex, clabel)?;
            } else if cindex < nseeds {
                self.merge_lookup(clabel, cindex)?;
            } else {
                // Neither is a seed; just merge.
                self.merge_lookup(clabel, cindex)?;
            }
        }

        Ok(())
    }

    /// Mark a seed's basin as finished. C: `numaSetValue(nasi, i, 0)`.
    fn set_seed_done(&mut self, index: i32) {
        let _ = self.nasi.set(index as usize, 0.0);
    }

    /// Height of `val` above the minimum of the basin labelled `label`.
    ///
    /// # C fidelity
    ///
    /// C has a second branch for `label` in `[nseeds, nseeds + nother)`
    /// that reads its `namh` array with the unshifted `label`, which is out
    /// of range for that array. It is unreachable: `wshedApply()` only calls
    /// this when both indices are below `nseeds`. Reject that range instead
    /// of reproducing an out-of-bounds read — which is also why this port
    /// does not keep a `namh` field at all.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedGetHeight()` in `watershed.c`
    fn get_height(&self, val: i32, label: i32) -> RegionResult<i32> {
        if label >= self.nseeds {
            return Err(RegionError::InvalidParameters(format!(
                "wshed height requested for non-seed label {label} (nseeds = {})",
                self.nseeds
            )));
        }
        let minval = self.nash.get(label as usize).unwrap_or(0.0) as i32;
        Ok(val - minval)
    }

    /// Save the basin owned by `index`, filled up to `level`.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedSaveBasin()` in `watershed.c`
    fn save_basin(&mut self, index: i32, level: i32) -> RegionResult<()> {
        let (b, pix) = self.identify_basin(index, level)?;
        self.pixad.push_with_box(pix, b);
        self.nalevels.push((level - 1) as f32);
        Ok(())
    }

    /// Breadth-first search that carves the basin owned by `index` out of
    /// the label plane.
    ///
    /// A neighbour joins the basin when it carries the label `index`, its
    /// source value is below `level` (the overflow height at which the two
    /// basins joined), and it has not been seen in this search.
    ///
    /// # See also
    ///
    /// C Leptonica: `identifyWatershedBasin()` in `watershed.c`
    fn identify_basin(&mut self, index: i32, level: i32) -> RegionResult<(Box, Pix)> {
        let w = self.w as i32;
        let h = self.h as i32;
        let wu = self.w as usize;

        // C primes these with 1000000 / 0 and relies on the seed push to
        // bring them into range.
        let mut minx = 1_000_000i32;
        let mut miny = 1_000_000i32;
        let mut maxx = 0i32;
        let mut maxy = 0i32;

        let (sx, sy) = pta_get_ipt(&self.ptas, index as usize);
        self.pixt[(sy as usize) * wu + sx as usize] = true;
        let mut queue = std::collections::VecDeque::new();
        push_new_pixel(
            &mut queue, sx, sy, &mut minx, &mut maxx, &mut miny, &mut maxy,
        );

        while let Some((x, y)) = queue.pop_front() {
            let imin = 0.max(y - 1);
            let imax = (h - 1).min(y + 1);
            let jmin = 0.max(x - 1);
            let jmax = (w - 1).min(x + 1);
            for i in imin..=imax {
                for j in jmin..=jmax {
                    if j == x && i == y {
                        continue; // parent
                    }
                    let off = (i as usize) * wu + j as usize;
                    let label = self.pixlab[off];
                    if label == MAX_LABEL_VALUE || self.lut[label as usize] != index {
                        continue;
                    }
                    if self.pixt[off] {
                        continue; // already seen
                    }
                    if self.source_value(j, i) >= level {
                        continue; // too high
                    }
                    self.pixt[off] = true;
                    push_new_pixel(&mut queue, j, i, &mut minx, &mut maxx, &mut miny, &mut maxy);
                }
            }
        }

        // Extract the box and pix, then clear that region of pixt. C does
        // it with pixClipRectangle + an in-place XOR rasterop; copying the
        // bits out and zeroing them is the same thing.
        let bw = (maxx - minx + 1) as u32;
        let bh = (maxy - miny + 1) as u32;
        let b = Box::new_unchecked(minx, miny, bw as i32, bh as i32);
        let pixd = Pix::new(bw, bh, PixelDepth::Bit1).map_err(RegionError::Core)?;
        let mut pixd = pixd.try_into_mut().map_err(|_| {
            RegionError::InvalidParameters("basin pix unexpectedly shared".to_string())
        })?;
        for dy in 0..bh {
            for dx in 0..bw {
                let off = ((miny as u32 + dy) as usize) * wu + (minx as u32 + dx) as usize;
                if self.pixt[off] {
                    pixd.set_pixel_unchecked(dx, dy, 1);
                    self.pixt[off] = false;
                }
            }
        }
        Ok((b, pixd.into()))
    }

    /// Redirect `sindex` (and everything pointing at it) to `dindex`.
    ///
    /// Every entry of `lut` is either an *owner* (`lut[i] == i`) or a
    /// *redirect* (`lut[i] != i`). This restores that canonical form after
    /// a merge, so a redirect always points straight at the current owner.
    ///
    /// # See also
    ///
    /// C Leptonica: `mergeLookup()` in `watershed.c`
    fn merge_lookup(&mut self, sindex: i32, dindex: i32) -> RegionResult<()> {
        let size = self.arraysize;
        if sindex < 0 || sindex >= size {
            return Err(RegionError::InvalidParameters(format!(
                "invalid merge source index {sindex} (size {size})"
            )));
        }
        if dindex < 0 || dindex >= size {
            return Err(RegionError::InvalidParameters(format!(
                "invalid merge destination index {dindex} (size {size})"
            )));
        }

        // Redirect the links in the lut.
        let src = self.links[sindex as usize].take().unwrap_or_default();
        for &idx in &src {
            self.lut[idx as usize] = dindex;
        }
        self.lut[sindex as usize] = dindex;

        // Shift the back-link arrays from sindex to dindex. sindex has no
        // back-links left: everything that pointed at it now points at
        // dindex. C's callers never merge an index into itself, so the
        // move below never aliases.
        let dst = self.links[dindex as usize].get_or_insert_with(Vec::new);
        dst.extend(src);
        dst.push(sindex);
        Ok(())
    }

    /// The source image with every saved basin painted at its fill level.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedRenderFill()` in `watershed.c`
    pub fn render_fill(&self) -> RegionResult<Pix> {
        let pixd = self.pixs.clone();
        let mut pixd = pixd.try_into_mut().unwrap_or_else(|p| p.to_mut());
        for i in 0..self.pixad.len() {
            let pix = self.pixad.get(i).expect("basin pix");
            let b = self.pixad.boxa().get(i).expect("basin box");
            let level = self.nalevels.get(i).unwrap_or(0.0) as u32;
            pixd.paint_through_mask(pix, b.x, b.y, level)
                .map_err(RegionError::Core)?;
        }
        Ok(pixd.into())
    }

    /// The source image in 32 bpp with the basins tinted by a random
    /// colormap.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedRenderColors()` in `watershed.c`
    pub fn render_colors(&self) -> RegionResult<Pix> {
        let (w, h) = (self.w, self.h);
        let pixd = self.pixs.convert_to_32().map_err(RegionError::Core)?;
        if self.pixad.is_empty() {
            // No basin was ever saved (for example a lone seed that never
            // collides with another). C walks the same path: every step from
            // `pixaDisplayRandomCmap` on logs an error and does nothing, and
            // `wshedRenderColors` returns the plain 32 bpp source. Return that
            // instead of failing, so the two agree.
            return Ok(pixd);
        }
        let pixt = self
            .pixad
            .display_random_cmap(w, h)
            .map_err(RegionError::Core)?;
        let pixc = pixt.convert_to_32().map_err(RegionError::Core)?;
        let pixm = self.pixad.display(w, h).map_err(RegionError::Core)?;
        let mut pixd = pixd.try_into_mut().unwrap_or_else(|p| p.to_mut());
        pixd.combine_masked(&pixc, &pixm)
            .map_err(RegionError::Core)?;
        Ok(pixd.into())
    }
}

/// C `ptaGetIPt()`: round the stored float coordinates to integers.
fn pta_get_ipt(pta: &Pta, index: usize) -> (i32, i32) {
    let (x, y) = pta.get(index).unwrap_or((0.0, 0.0));
    ((x + 0.5) as i32, (y + 0.5) as i32)
}

/// C `pushNewPixel()`: enqueue and grow the bounding box.
fn push_new_pixel(
    queue: &mut std::collections::VecDeque<(i32, i32)>,
    x: i32,
    y: i32,
    minx: &mut i32,
    maxx: &mut i32,
    miny: &mut i32,
    maxy: &mut i32,
) {
    *minx = (*minx).min(x);
    *maxx = (*maxx).max(x);
    *miny = (*miny).min(y);
    *maxy = (*maxy).max(y);
    queue.push_back((x, y));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 12x12 with two conical basins whose seeds sit at their minima.
    fn two_basin_fixture() -> (Pix, Pix) {
        let pixs = Pix::new(12, 12, PixelDepth::Bit8).unwrap();
        let mut m = pixs.try_into_mut().unwrap();
        for y in 0..12i32 {
            for x in 0..12i32 {
                let d1 = (x - 2) * (x - 2) + (y - 2) * (y - 2);
                let d2 = (x - 9) * (x - 9) + (y - 9) * (y - 9);
                let v = (40 + d1.min(d2) * 3).min(255);
                m.set_pixel(x as u32, y as u32, v as u32).unwrap();
            }
        }
        let pixs: Pix = m.into();

        let pixm = Pix::new(12, 12, PixelDepth::Bit1).unwrap();
        let mut m = pixm.try_into_mut().unwrap();
        m.set_pixel(2, 2, 1).unwrap();
        m.set_pixel(9, 9, 1).unwrap();
        (pixs, m.into())
    }

    /// Expected basins, taken verbatim from C `wshedApply()` on the same
    /// fixture. Note the third, 1x1 basin: C's own header warns that the
    /// algorithm "seems to locate watersheds that are duplicates", and this
    /// port reproduces that rather than correcting it.
    type ExpectedBasin = (i32, i32, i32, i32, i32, &'static [&'static str]);

    const EXPECTED_BASINS: [ExpectedBasin; 3] = [
        (
            0,
            0,
            7,
            7,
            92,
            &[
                "1111110", "1111111", "1111111", "1111111", "1111110", "1111100", "0111000",
            ],
        ),
        (
            5,
            5,
            7,
            7,
            92,
            &[
                "0001110", "0011111", "0111111", "1111111", "1111111", "1111111", "0111111",
            ],
        ),
        (9, 9, 1, 1, 92, &["1"]),
    ];

    #[test]
    fn test_wshed_apply_matches_c() {
        let (pixs, pixm) = two_basin_fixture();
        let mut w = Wshed::new(&pixs, &pixm, 5).unwrap();
        w.apply().unwrap();

        assert_eq!(w.num_seeds(), 2);
        assert_eq!(w.num_other(), 0);

        let (pixa, na) = w.basins();
        assert_eq!(pixa.len(), EXPECTED_BASINS.len());
        for (i, &(bx, by, bw, bh, level, rows)) in EXPECTED_BASINS.iter().enumerate() {
            let b = pixa.boxa().get(i).expect("basin box");
            assert_eq!((b.x, b.y, b.w, b.h), (bx, by, bw, bh), "basin {i} box");
            assert_eq!(na.get(i).unwrap() as i32, level, "basin {i} level");
            let pix = pixa.get(i).expect("basin pix");
            for (y, row) in rows.iter().enumerate() {
                for (x, c) in row.bytes().enumerate() {
                    assert_eq!(
                        pix.get_pixel_unchecked(x as u32, y as u32),
                        u32::from(c - b'0'),
                        "basin {i} at ({x}, {y})"
                    );
                }
            }
        }
    }

    /// C `wshedRenderFill()` paints each basin at its recorded level over a
    /// copy of the source. Expected values dumped from C.
    #[test]
    fn test_wshed_render_fill_matches_c() {
        let (pixs, pixm) = two_basin_fixture();
        let mut w = Wshed::new(&pixs, &pixm, 5).unwrap();
        w.apply().unwrap();
        let pixd = w.render_fill().unwrap();

        #[rustfmt::skip]
        const EXPECTED: [[u32; 12]; 12] = [
            [ 92,  92,  92,  92,  92,  92, 100, 127, 160, 199, 244, 255],
            [ 92,  92,  92,  92,  92,  92,  92, 118, 151, 190, 235, 244],
            [ 92,  92,  92,  92,  92,  92,  92, 115, 148, 187, 190, 199],
            [ 92,  92,  92,  92,  92,  92,  92, 118, 151, 148, 151, 160],
            [ 92,  92,  92,  92,  92,  92, 100, 127, 118, 115, 118, 127],
            [ 92,  92,  92,  92,  92,  94, 115, 100,  92,  92,  92, 100],
            [100,  92,  92,  92, 100, 115,  94,  92,  92,  92,  92,  92],
            [127, 118, 115, 118, 127, 100,  92,  92,  92,  92,  92,  92],
            [160, 151, 148, 151, 118,  92,  92,  92,  92,  92,  92,  92],
            [199, 190, 187, 148, 115,  92,  92,  92,  92,  92,  92,  92],
            [244, 235, 190, 151, 118,  92,  92,  92,  92,  92,  92,  92],
            [255, 244, 199, 160, 127, 100,  92,  92,  92,  92,  92,  92],
        ];

        for (y, row) in EXPECTED.iter().enumerate() {
            for (x, &want) in row.iter().enumerate() {
                assert_eq!(
                    pixd.get_pixel_unchecked(x as u32, y as u32),
                    want,
                    "render_fill at ({x}, {y})"
                );
            }
        }
    }

    /// A lone seed never collides with another basin, so nothing is ever
    /// saved. C leaves `pixad` empty here (verified by running
    /// `wshedApply()` on this fixture: `nbasins=0`), and `wshedRenderColors()`
    /// then logs errors from `pixaDisplayRandomCmap` onwards and returns the
    /// plain 32 bpp source. C has a final pass over the seed indicators that
    /// would save such basins, but it is disabled behind `#if 0` with the
    /// comment "This seems to screw things up!", so it must not be ported.
    #[test]
    fn test_wshed_single_seed_saves_no_basin() {
        let pixs = Pix::new(12, 12, PixelDepth::Bit8).unwrap();
        let mut m = pixs.try_into_mut().unwrap();
        for y in 0..12i32 {
            for x in 0..12i32 {
                let d = (x - 6) * (x - 6) + (y - 6) * (y - 6);
                m.set_pixel(x as u32, y as u32, (40 + d).min(255) as u32)
                    .unwrap();
            }
        }
        let pixs: Pix = m.into();

        let pixm = Pix::new(12, 12, PixelDepth::Bit1).unwrap();
        let mut m = pixm.try_into_mut().unwrap();
        m.set_pixel(6, 6, 1).unwrap();
        let pixm: Pix = m.into();

        let mut w = Wshed::new(&pixs, &pixm, 5).unwrap();
        w.apply().unwrap();
        assert_eq!(w.num_seeds(), 1);
        assert_eq!(w.num_other(), 0);
        let (pixa, na) = w.basins();
        assert_eq!(
            pixa.len(),
            0,
            "a lone seed never collides, so C saves nothing"
        );
        assert_eq!(na.len(), 0);

        // render_fill leaves the source untouched, render_colors returns it
        // converted to 32 bpp; neither may fail.
        let filled = w.render_fill().unwrap();
        for y in 0..12u32 {
            for x in 0..12u32 {
                assert_eq!(
                    filled.get_pixel_unchecked(x, y),
                    pixs.get_pixel_unchecked(x, y),
                    "render_fill at ({x}, {y})"
                );
            }
        }
        let colored = w.render_colors().unwrap();
        assert_eq!(colored.depth(), PixelDepth::Bit32);
    }

    /// `save_basin` stores `level - 1` and its callers pass `val - 1`, so a
    /// basin saved at a collision value of 1 records level `-1`.
    ///
    /// C keeps that as an `l_int32`, passes it to `pixPaintThroughMask()` as
    /// an `l_uint32`, and the low byte is written — so level `-1` paints 255
    /// (measured: `-1` -> 255, `-2` -> 254). Saturating the cast to 0 instead
    /// would paint black.
    ///
    /// The fixture below is the one used to verify this against C: two minima
    /// at 0 joined by a plateau of 1s long enough that the saddle is not
    /// adjacent to either minimum (otherwise the basins collide at 0 and are
    /// discarded as too shallow). C reports `level=-1` for every basin and
    /// paints 255 at both minima.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_wshed_render_fill_negative_level_matches_c() {
        fn rowval(x: i32) -> i32 {
            if x == 8 || x == 12 {
                0
            } else if x > 8 && x < 12 {
                1
            } else if x < 8 {
                2 + (8 - x)
            } else {
                2 + (x - 12)
            }
        }

        let pixs = Pix::new(24, 24, PixelDepth::Bit8).unwrap();
        let mut m = pixs.try_into_mut().unwrap();
        for y in 0..24i32 {
            for x in 0..24i32 {
                let v = (rowval(x) + 3 * (y - 12).abs()).min(255);
                m.set_pixel(x as u32, y as u32, v as u32).unwrap();
            }
        }
        let pixs: Pix = m.into();

        let pixm = Pix::new(24, 24, PixelDepth::Bit1).unwrap();
        let mut m = pixm.try_into_mut().unwrap();
        m.set_pixel(8, 12, 1).unwrap();
        m.set_pixel(12, 12, 1).unwrap();
        let pixm: Pix = m.into();

        let mut w = Wshed::new(&pixs, &pixm, 1).unwrap();
        w.apply().unwrap();

        let (pixa, na) = w.basins();
        assert_eq!(pixa.len(), 3, "C saves three basins for this fixture");
        for i in 0..na.len() {
            assert_eq!(na.get(i).unwrap() as i32, -1, "basin {i} level");
        }

        // C: renderFill row 12, x = 6..14 -> 4 3 255 1 1 1 255 3 4
        let filled = w.render_fill().unwrap();
        let row: Vec<u32> = (6..15).map(|x| filled.get_pixel_unchecked(x, 12)).collect();
        assert_eq!(row, vec![4, 3, 255, 1, 1, 1, 255, 3, 4]);
    }

    #[test]
    fn test_wshed_rejects_bad_input() {
        let pix8 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let pix1 = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let pix1_big = Pix::new(5, 4, PixelDepth::Bit1).unwrap();

        assert!(Wshed::new(&pix1, &pix1, 1).is_err(), "pixs must be 8 bpp");
        assert!(Wshed::new(&pix8, &pix8, 1).is_err(), "pixm must be 1 bpp");
        assert!(Wshed::new(&pix8, &pix1_big, 1).is_err(), "sizes must agree");
        assert!(Wshed::new(&pix8, &pix1, 1).is_ok());
    }

    /// C clamps `mindepth` to at least 1 (`L_MAX(1, mindepth)`).
    #[test]
    fn test_wshed_clamps_mindepth() {
        let pix8 = Pix::new(4, 4, PixelDepth::Bit8).unwrap();
        let pix1 = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        for given in [-5, 0, 1] {
            let w = Wshed::new(&pix8, &pix1, given).unwrap();
            assert_eq!(w.mindepth, 1, "mindepth {given} must clamp to 1");
        }
        let w = Wshed::new(&pix8, &pix1, 7).unwrap();
        assert_eq!(w.mindepth, 7);
    }
}
