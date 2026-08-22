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
//! C's own header warns that `wshedApply()` "is buggy: it seems to locate
//! watersheds that are duplicates". That behaviour is reproduced here, not
//! corrected — the point of this port is to agree with C bit for bit.

use crate::core::{Numa, Pix, Pixa, PixelDepth, Pta};
use crate::region::error::{RegionError, RegionResult};

/// C: `static const l_uint32 MAX_LABEL_VALUE = 0x7fffffff` — the label
/// written into every pixel of `pixlab` before filling starts.
const MAX_LABEL_VALUE: u32 = 0x7fff_ffff;

/// Watershed transform state (C `L_WSHED`).
///
/// Build it with [`Wshed::new`], run [`Wshed::apply`], then read the
/// basins with [`Wshed::basins`] or render them with
/// [`Wshed::render_fill`] / [`Wshed::render_colors`].
// The stub does not touch most of the state yet; `apply()` fills it in.
#[allow(dead_code)]
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
    /// Initial heights of the minima that carry no seed.
    namh: Numa,
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
            namh: Numa::new(),
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

    /// Run the flooding.
    ///
    /// Not implemented yet.
    ///
    /// # See also
    ///
    /// C Leptonica: `wshedApply()` in `watershed.c`
    pub fn apply(&mut self) -> RegionResult<()> {
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
