//! Box sequence smoothing, reconciliation, and analysis
//!
//! Functions for smoothing box sequences using windowed median,
//! reconciling box dimensions, and computing size variation metrics.
//!
//! C Leptonica equivalents: boxfunc5.c

use crate::core::error::{Error, Result};
use crate::core::numa::Numa;

use super::geometry::Direction;
use super::{Box, Boxa};

// ---- Enums ----

/// Flag for boxaModifyWithBoxa subflag parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifySubFlag {
    /// Use intersection (min size)
    UseMinSize,
    /// Use union (max size)
    UseMaxSize,
    /// Substitute based on location difference
    SubOnLocDiff,
    /// Substitute based on size difference
    SubOnSizeDiff,
    /// Use capped minimum
    UseCappedMin,
    /// Use capped maximum
    UseCappedMax,
}

/// Flag for boxaSizeConsistency and boxaReconcileSizeByMedian type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckType {
    /// Check width
    Width,
    /// Check height
    Height,
    /// Check both width and height
    Both,
}

/// Flag for boxaFillSequence useflag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillFlag {
    /// Use all boxes to find nearest valid
    UseAllBoxes,
    /// Use only same-parity boxes
    UseSameParity,
}

/// Flag for boxaSizeVariation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectDimension {
    /// Select width
    Width,
    /// Select height
    Height,
}

/// Which pair of sides to adjust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustSelect {
    /// Adjust left side only
    Left,
    /// Adjust right side only
    Right,
    /// Adjust top side only
    Top,
    /// Adjust bottom side only
    Bot,
    /// Adjust left and right
    LeftAndRight,
    /// Adjust top and bottom
    TopAndBot,
    /// Skip adjustment
    Skip,
}

/// Reconcile pair width operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustChoose {
    /// Choose minimum width
    Min,
    /// Choose maximum width
    Max,
}

/// Result of `size_consistency`
#[derive(Debug, Clone, Copy)]
pub struct SizeConsistencyResult {
    /// Average fractional pairwise variation
    pub fvar_pair: f32,
    /// Average fractional variation from median
    pub fvar_median: f32,
    /// Decision: 1 = same size, 0 = different, -1 = unknown
    pub same: i32,
}

/// Result of `plot_sides`
#[derive(Debug, Clone)]
pub struct PlotSidesResult {
    /// Left side coordinates
    pub left: Numa,
    /// Top side coordinates
    pub top: Numa,
    /// Right side coordinates
    pub right: Numa,
    /// Bottom side coordinates
    pub bottom: Numa,
}

/// Result of `plot_sizes`
#[derive(Debug, Clone)]
pub struct PlotSizesResult {
    /// Width values
    pub width: Numa,
    /// Height values
    pub height: Numa,
}

/// Result of `size_variation`
#[derive(Debug, Clone, Copy)]
pub struct SizeVariationResult {
    /// Average absolute difference of even-odd size pairs
    pub del_evenodd: f32,
    /// RMS deviation of even boxes
    pub rms_even: f32,
    /// RMS deviation of odd boxes
    pub rms_odd: f32,
    /// RMS deviation of all boxes
    pub rms_all: f32,
}

/// Result of `median_dimensions`
#[derive(Debug, Clone)]
pub struct MedianDimensionsResult {
    /// Median width of all boxes
    pub med_w: i32,
    /// Median height of all boxes
    pub med_h: i32,
    /// Median width of even boxes
    pub med_we: i32,
    /// Median width of odd boxes
    pub med_wo: i32,
    /// Median height of even boxes
    pub med_he: i32,
    /// Median height of odd boxes
    pub med_ho: i32,
    /// Width difference from median for each box
    pub na_del_w: Numa,
    /// Height difference from median for each box
    pub na_del_h: Numa,
}

// ---- Helper: check if a box is "invalid" (zero w or h) ----

fn is_valid_box(b: &Box) -> bool {
    b.w > 0 && b.h > 0
}

/// Count valid (non-zero w and h) boxes
fn valid_count(boxa: &Boxa) -> usize {
    boxa.iter().filter(|b| is_valid_box(b)).count()
}

// ---- Boxa methods ----

impl Boxa {
    /// Fill missing (invalid) boxes in a sequence by copying the nearest valid box.
    ///
    /// Invalid boxes have w == 0 or h == 0.
    ///
    /// C Leptonica equivalent: `boxaFillSequence`
    pub fn fill_sequence(&self, useflag: FillFlag, _debug: bool) -> Result<Boxa> {
        let n = self.len();
        let nv = valid_count(self);
        if n == nv {
            return Ok(self.clone());
        }
        if useflag == FillFlag::UseSameParity && n < 3 {
            return Ok(self.clone());
        }

        match useflag {
            FillFlag::UseAllBoxes => {
                let mut boxad = self.clone();
                fill_all(&mut boxad);
                Ok(boxad)
            }
            FillFlag::UseSameParity => {
                let (mut boxae, mut boxao) = self.split_even_odd(false);
                fill_all(&mut boxae);
                fill_all(&mut boxao);
                Boxa::merge_even_odd(&boxae, &boxao, false)
            }
        }
    }

    /// Compute windowed median smoothing of a box sequence.
    ///
    /// Each of the four sides (left, top, right, bottom) is independently
    /// smoothed using a windowed median filter. Invalid boxes are filled first.
    ///
    /// C Leptonica equivalent: `boxaWindowedMedian`
    pub fn windowed_median(&self, halfwin: usize, _debug: bool) -> Result<Boxa> {
        let n = self.len();
        if n < 3 {
            return Ok(self.clone());
        }
        if halfwin == 0 {
            return Ok(self.clone());
        }

        let boxaf = self.fill_sequence(FillFlag::UseAllBoxes, false)?;
        let (nal, nat, nar, nab, _, _) = boxaf.extract_as_numa(true);

        let naml = nal.windowed_median(halfwin);
        let namt = nat.windowed_median(halfwin);
        let namr = nar.windowed_median(halfwin);
        let namb = nab.windowed_median(halfwin);

        let n = boxaf.len();
        let mut boxad = Boxa::with_capacity(n);
        for i in 0..n {
            let left = naml.get_i32(i).unwrap_or(0);
            let top = namt.get_i32(i).unwrap_or(0);
            let right = namr.get_i32(i).unwrap_or(0);
            let bot = namb.get_i32(i).unwrap_or(0);
            boxad.push(Box::new_unchecked(
                left,
                top,
                right - left + 1,
                bot - top + 1,
            ));
        }
        Ok(boxad)
    }

    /// Smooth a box sequence using windowed median on even/odd sets separately.
    ///
    /// Splits into even/odd sub-sequences, applies windowed median to each,
    /// then modifies original boxes using `modify_with_boxa`.
    ///
    /// C Leptonica equivalent: `boxaSmoothSequenceMedian`
    pub fn smooth_sequence_median(
        &self,
        halfwin: usize,
        subflag: ModifySubFlag,
        maxdiff: i32,
        extrapixels: i32,
        _debug: bool,
    ) -> Result<Boxa> {
        if halfwin == 0 {
            return Ok(self.clone());
        }
        if maxdiff < 0 {
            return Ok(self.clone());
        }
        let n = self.len();
        if n < 6 {
            return Ok(self.clone());
        }

        let (boxae, boxao) = self.split_even_odd(false);
        let boxamede = boxae.windowed_median(halfwin, false)?;
        let boxamedo = boxao.windowed_median(halfwin, false)?;

        let boxame = boxae.modify_with_boxa(&boxamede, subflag, maxdiff, extrapixels)?;
        let boxamo = boxao.modify_with_boxa(&boxamedo, subflag, maxdiff, extrapixels)?;

        Boxa::merge_even_odd(&boxame, &boxamo, false)
    }

    /// Modify boxes using a reference boxa.
    ///
    /// For each box pair `(boxs, boxm)`, the output box depends on `subflag`.
    ///
    /// C Leptonica equivalent: `boxaModifyWithBoxa`
    pub fn modify_with_boxa(
        &self,
        boxam: &Boxa,
        subflag: ModifySubFlag,
        maxdiff: i32,
        extrapixels: i32,
    ) -> Result<Boxa> {
        let n = self.len();
        if n != boxam.len() {
            return Err(Error::InvalidParameter(format!(
                "boxas ({}) and boxam ({}) sizes differ",
                n,
                boxam.len()
            )));
        }

        let invalid = Box::new_unchecked(0, 0, 0, 0);
        let mut boxad = Boxa::with_capacity(n);

        for i in 0..n {
            let bs = self.get(i).unwrap();
            let bm = boxam.get(i).unwrap();

            if !is_valid_box(bs) || !is_valid_box(bm) {
                boxad.push(invalid);
                continue;
            }

            let (ls, ts, ws, hs) = (bs.x, bs.y, bs.w, bs.h);
            let (lm, tm, wm, hm) = (bm.x, bm.y, bm.w, bm.h);
            let rs = ls + ws - 1;
            let bots = ts + hs - 1;
            let rm = lm + wm - 1;
            let botm = tm + hm - 1;

            let (ld, td, rd, bd) = match subflag {
                ModifySubFlag::UseMinSize => (ls.max(lm), ts.max(tm), rs.min(rm), bots.min(botm)),
                ModifySubFlag::UseMaxSize => (ls.min(lm), ts.min(tm), rs.max(rm), bots.max(botm)),
                ModifySubFlag::SubOnLocDiff => (
                    if (lm - ls).abs() <= maxdiff {
                        ls
                    } else {
                        lm - extrapixels
                    },
                    if (tm - ts).abs() <= maxdiff {
                        ts
                    } else {
                        tm - extrapixels
                    },
                    if (rm - rs).abs() <= maxdiff {
                        rs
                    } else {
                        rm + extrapixels
                    },
                    if (botm - bots).abs() <= maxdiff {
                        bots
                    } else {
                        botm + extrapixels
                    },
                ),
                ModifySubFlag::SubOnSizeDiff => (
                    if (wm - ws).abs() <= maxdiff {
                        ls
                    } else {
                        lm - extrapixels
                    },
                    if (hm - hs).abs() <= maxdiff {
                        ts
                    } else {
                        tm - extrapixels
                    },
                    if (wm - ws).abs() <= maxdiff {
                        rs
                    } else {
                        rm + extrapixels
                    },
                    if (hm - hs).abs() <= maxdiff {
                        bots
                    } else {
                        botm + extrapixels
                    },
                ),
                ModifySubFlag::UseCappedMin => (
                    lm.max(ls.min(lm + maxdiff)),
                    tm.max(ts.min(tm + maxdiff)),
                    rm.min(rs.max(rm - maxdiff)),
                    botm.min(bots.max(botm - maxdiff)),
                ),
                ModifySubFlag::UseCappedMax => (
                    lm.min(ls.max(lm - maxdiff)),
                    tm.min(ts.max(tm - maxdiff)),
                    rm.max(rs.min(rm + maxdiff)),
                    botm.max(bots.min(botm + maxdiff)),
                ),
            };

            let wd = (rd - ld + 1).max(0);
            let hd = (bd - td + 1).max(0);
            boxad.push(Box::new_unchecked(ld, td, wd, hd));
        }

        Ok(boxad)
    }

    /// Reconcile widths of consecutive box pairs.
    ///
    /// Adjusts box widths where adjacent even/odd pairs differ by more than `delw`.
    ///
    /// C Leptonica equivalent: `boxaReconcilePairWidth`
    pub fn reconcile_pair_width(
        &self,
        delw: i32,
        op: AdjustChoose,
        factor: f32,
        na: Option<&Numa>,
    ) -> Result<Boxa> {
        let factor = if factor <= 0.0 { 1.0 } else { factor };

        let (mut boxae, mut boxao) = self.split_even_odd(false);
        let ne = boxae.len();
        let no = boxao.len();
        let nmin = ne.min(no);

        for i in 0..nmin {
            let (inde, indo) = if let Some(na) = na {
                (
                    na.get_i32(2 * i).unwrap_or(0),
                    na.get_i32(2 * i + 1).unwrap_or(0),
                )
            } else {
                (1, 1)
            };
            if inde == 0 && indo == 0 {
                continue;
            }

            let be = *boxae.get(i).unwrap();
            let bo = *boxao.get(i).unwrap();
            let (xe, we) = (be.x, be.w);
            let wo = bo.w;

            if we == 0 || wo == 0 {
                continue;
            }

            if (we - wo).abs() > delw {
                match op {
                    AdjustChoose::Min => {
                        if we > wo && inde == 1 {
                            let w = (factor * wo as f32) as i32;
                            let x = xe + (we - w);
                            boxae.replace(i, be.set_geometry(x, -1, w, -1))?;
                        } else if we < wo && indo == 1 {
                            let w = (factor * we as f32) as i32;
                            boxao.replace(i, bo.set_geometry(-1, -1, w, -1))?;
                        }
                    }
                    AdjustChoose::Max => {
                        if we < wo && inde == 1 {
                            let w = (factor * wo as f32) as i32;
                            let x = (xe + (we - w)).max(0);
                            let w = we + (xe - x);
                            boxae.replace(i, be.set_geometry(x, -1, w, -1))?;
                        } else if we > wo && indo == 1 {
                            let w = (factor * we as f32) as i32;
                            boxao.replace(i, bo.set_geometry(-1, -1, w, -1))?;
                        }
                    }
                }
            }
        }

        Boxa::merge_even_odd(&boxae, &boxao, false)
    }

    /// Check size consistency of a box sequence.
    ///
    /// Evaluates pairwise and median-based dimensional variation.
    ///
    /// C Leptonica equivalent: `boxaSizeConsistency`
    pub fn size_consistency(
        &self,
        check_type: CheckType,
        threshp: f32,
        threshm: f32,
    ) -> Result<SizeConsistencyResult> {
        if check_type == CheckType::Both {
            return Err(Error::InvalidParameter(
                "size_consistency requires Width or Height, not Both".into(),
            ));
        }
        if valid_count(self) < 6 {
            return Err(Error::InvalidParameter(
                "need at least 6 valid boxes".into(),
            ));
        }
        if !(0.0..0.5).contains(&threshp) {
            return Err(Error::InvalidParameter(format!(
                "invalid threshp: {threshp}"
            )));
        }
        if !(0.0..0.5).contains(&threshm) {
            return Err(Error::InvalidParameter(format!(
                "invalid threshm: {threshm}"
            )));
        }
        let threshp = if threshp == 0.0 { 0.02 } else { threshp };
        let threshm = if threshm == 0.0 { 0.015 } else { threshm };

        let n = self.len();
        let mut na1 = Numa::new();
        let mut npairs = 0;
        let mut sumdiff = 0.0f32;

        let mut i = 0;
        while i < n - 1 {
            let (_, _, bw1, bh1) = self.get_box_geometry(i).unwrap_or((0, 0, 0, 0));
            let (_, _, bw2, bh2) = self.get_box_geometry(i + 1).unwrap_or((0, 0, 0, 0));
            i += 2;

            if bw1 == 0 || bh1 == 0 || bw2 == 0 || bh2 == 0 {
                continue;
            }
            npairs += 1;

            match check_type {
                CheckType::Width => {
                    let ave = (bw1 + bw2) as f32 / 2.0;
                    sumdiff += (bw1 - bw2).abs() as f32 / ave;
                    na1.push(bw1 as f32);
                    na1.push(bw2 as f32);
                }
                CheckType::Height => {
                    let ave = (bh1 + bh2) as f32 / 2.0;
                    sumdiff += (bh1 - bh2).abs() as f32 / ave;
                    na1.push(bh1 as f32);
                    na1.push(bh2 as f32);
                }
                CheckType::Both => unreachable!(),
            }
        }

        if npairs == 0 {
            return Err(Error::InvalidParameter("no valid pairs found".into()));
        }

        let fvarp = sumdiff / npairs as f32;
        let med = na1.median().unwrap_or(0.0);
        let fvarm = if med == 0.0 {
            0.0
        } else {
            let dev = na1.mean_dev_from_median(med).unwrap_or(0.0);
            dev / med
        };

        let same = if fvarp < threshp && fvarm < threshm {
            1
        } else if fvarp < threshp && fvarm > threshm {
            0
        } else {
            -1
        };

        Ok(SizeConsistencyResult {
            fvar_pair: fvarp,
            fvar_median: fvarm,
            same,
        })
    }

    /// Reconcile all box dimensions by median.
    ///
    /// Applies `reconcile_sides_by_median` to even and odd boxes separately,
    /// for left/right and/or top/bottom sides.
    ///
    /// C Leptonica equivalent: `boxaReconcileAllByMedian`
    pub fn reconcile_all_by_median(
        &self,
        select1: AdjustSelect,
        select2: AdjustSelect,
        thresh: i32,
        extra: i32,
    ) -> Result<Boxa> {
        if select1 != AdjustSelect::LeftAndRight && select1 != AdjustSelect::Skip {
            return Ok(self.clone());
        }
        if select2 != AdjustSelect::TopAndBot && select2 != AdjustSelect::Skip {
            return Ok(self.clone());
        }
        if thresh < 0 {
            return Ok(self.clone());
        }
        if valid_count(self) < 3 {
            return Ok(self.clone());
        }

        let (boxa1e, boxa1o) = self.split_even_odd(false);

        let boxa2e = if select1 == AdjustSelect::LeftAndRight {
            boxa1e.reconcile_sides_by_median(select1, thresh, extra)?
        } else {
            boxa1e.clone()
        };
        let boxa3e = if select2 == AdjustSelect::TopAndBot {
            boxa2e.reconcile_sides_by_median(select2, thresh, extra)?
        } else {
            boxa2e
        };

        let boxa2o = if select1 == AdjustSelect::LeftAndRight {
            boxa1o.reconcile_sides_by_median(select1, thresh, extra)?
        } else {
            boxa1o.clone()
        };
        let boxa3o = if select2 == AdjustSelect::TopAndBot {
            boxa2o.reconcile_sides_by_median(select2, thresh, extra)?
        } else {
            boxa2o
        };

        Boxa::merge_even_odd(&boxa3e, &boxa3o, false)
    }

    /// Reconcile box sides by median.
    ///
    /// Modifies individual box sides if their location differs significantly
    /// from the median value.
    ///
    /// C Leptonica equivalent: `boxaReconcileSidesByMedian`
    pub fn reconcile_sides_by_median(
        &self,
        select: AdjustSelect,
        thresh: i32,
        extra: i32,
    ) -> Result<Boxa> {
        if thresh < 0 {
            return Ok(self.clone());
        }
        if valid_count(self) < 3 {
            return Ok(self.clone());
        }

        // Handle compound selects by chaining
        if select == AdjustSelect::LeftAndRight {
            let boxa1 = self.reconcile_sides_by_median(AdjustSelect::Left, thresh, extra)?;
            return boxa1.reconcile_sides_by_median(AdjustSelect::Right, thresh, extra);
        }
        if select == AdjustSelect::TopAndBot {
            let boxa1 = self.reconcile_sides_by_median(AdjustSelect::Top, thresh, extra)?;
            return boxa1.reconcile_sides_by_median(AdjustSelect::Bot, thresh, extra);
        }

        let n = self.len();
        let (medleft, medtop, medright, medbot, _, _) = self.get_median_vals()?;
        let mut boxad = Boxa::with_capacity(n);

        for i in 0..n {
            let mut b = *self.get(i).unwrap();
            match select {
                AdjustSelect::Left => {
                    let (left, _, _, _) = b.side_locations();
                    let diff = medleft - left;
                    if diff.abs() >= thresh {
                        b.set_side(Direction::FromLeft, left + diff - extra, 0);
                    }
                }
                AdjustSelect::Right => {
                    let (_, right, _, _) = b.side_locations();
                    let diff = medright - right;
                    if diff.abs() >= thresh {
                        b.set_side(Direction::FromRight, right + diff + extra, 0);
                    }
                }
                AdjustSelect::Top => {
                    let (_, _, top, _) = b.side_locations();
                    let diff = medtop - top;
                    if diff.abs() >= thresh {
                        b.set_side(Direction::FromTop, top + diff - extra, 0);
                    }
                }
                AdjustSelect::Bot => {
                    let (_, _, _, bot) = b.side_locations();
                    let diff = medbot - bot;
                    if diff.abs() >= thresh {
                        b.set_side(Direction::FromBottom, bot + diff + extra, 0);
                    }
                }
                _ => {}
            }
            boxad.push(b);
        }
        Ok(boxad)
    }

    /// Reconcile box sizes by median.
    ///
    /// Identifies outlier boxes whose width or height differs significantly
    /// from the median and adjusts them.
    ///
    /// C Leptonica equivalent: `boxaReconcileSizeByMedian`
    pub fn reconcile_size_by_median(
        &self,
        check_type: CheckType,
        dfract: f32,
        sfract: f32,
        factor: f32,
    ) -> Result<Boxa> {
        if dfract <= 0.0 || dfract >= 0.5 {
            return Ok(self.clone());
        }
        if sfract <= 0.0 || sfract >= 0.5 {
            return Ok(self.clone());
        }
        if valid_count(self) < 6 {
            return Ok(self.clone());
        }

        if check_type == CheckType::Both {
            let boxa1 = self.reconcile_size_by_median(CheckType::Width, dfract, sfract, factor)?;
            return boxa1.reconcile_size_by_median(CheckType::Height, dfract, sfract, factor);
        }

        let n = self.len();
        let med_dims = self.median_dimensions()?;
        let medw = med_dims.med_w;
        let medh = med_dims.med_h;

        // Identify outliers and collect inliers for even/odd
        let mut naind = Vec::with_capacity(n);
        let mut boxae = Boxa::new();
        let mut boxao = Boxa::new();
        let mut outfound = false;

        for i in 0..n {
            let b = self.get(i).unwrap();
            if !is_valid_box(b) {
                naind.push(0);
                continue;
            }
            let brat = match check_type {
                CheckType::Width => b.w as f32 / medw as f32,
                CheckType::Height => b.h as f32 / medh as f32,
                CheckType::Both => unreachable!(),
            };
            if brat < 1.0 - dfract || brat > 1.0 + dfract {
                outfound = true;
                naind.push(1);
            } else {
                naind.push(0);
                if i % 2 == 0 {
                    boxae.push(*b);
                } else {
                    boxao.push(*b);
                }
            }
        }

        if !outfound {
            return Ok(self.clone());
        }

        let mut boxad = Boxa::with_capacity(n);

        match check_type {
            CheckType::Width => {
                let ne = valid_count(&boxae);
                let no = valid_count(&boxao);
                let (mut medlefte, mut medrighte) = (0, 0);
                let (mut medlefto, mut medrighto) = (0, 0);
                if ne > 0 {
                    let (l, _, r, _, _, _) = boxae.get_median_vals().unwrap_or((0, 0, 0, 0, 0, 0));
                    medlefte = l;
                    medrighte = r;
                }
                if no > 0 {
                    let (l, _, r, _, _, _) = boxao.get_median_vals().unwrap_or((0, 0, 0, 0, 0, 0));
                    medlefto = l;
                    medrighto = r;
                }
                if ne == 0 {
                    medlefte = medlefto;
                    medrighte = medrighto;
                } else if no == 0 {
                    medlefto = medlefte;
                    medrighto = medrighte;
                }

                let maxdel = (sfract * medw as f32 + 0.5) as i32;
                for (i, &ind) in naind.iter().enumerate() {
                    let mut b = *self.get(i).unwrap();
                    let medleft = if i % 2 == 0 { medlefte } else { medlefto };
                    let medright = if i % 2 == 0 { medrighte } else { medrighto };
                    if ind == 1 && is_valid_box(&b) {
                        let (left, right, _, _) = b.side_locations();
                        let left = if (left - medleft).abs() > maxdel {
                            medleft
                        } else {
                            left
                        };
                        let right = if (right - medright).abs() > maxdel {
                            medright
                        } else {
                            right
                        };
                        let del = (factor * medw as f32 - (right - left) as f32) as i32 / 2;
                        b.set_side(Direction::FromLeft, left - del, 0);
                        b.set_side(Direction::FromRight, right + del, 0);
                    }
                    boxad.push(b);
                }
            }
            CheckType::Height => {
                let ne = valid_count(&boxae);
                let no = valid_count(&boxao);
                let (mut medtope, mut medbote) = (0, 0);
                let (mut medtopo, mut medboto) = (0, 0);
                if ne > 0 {
                    let (_, t, _, b, _, _) = boxae.get_median_vals().unwrap_or((0, 0, 0, 0, 0, 0));
                    medtope = t;
                    medbote = b;
                }
                if no > 0 {
                    let (_, t, _, b, _, _) = boxao.get_median_vals().unwrap_or((0, 0, 0, 0, 0, 0));
                    medtopo = t;
                    medboto = b;
                }
                if ne == 0 {
                    medtope = medtopo;
                    medbote = medboto;
                } else if no == 0 {
                    medtopo = medtope;
                    medboto = medbote;
                }

                let maxdel = (sfract * medh as f32 + 0.5) as i32;
                for (i, &ind) in naind.iter().enumerate() {
                    let mut b = *self.get(i).unwrap();
                    let medtop = if i % 2 == 0 { medtope } else { medtopo };
                    let medbot = if i % 2 == 0 { medbote } else { medboto };
                    if ind == 1 && is_valid_box(&b) {
                        let (_, _, top, bot) = b.side_locations();
                        let top = if (top - medtop).abs() > maxdel {
                            medtop
                        } else {
                            top
                        };
                        let bot = if (bot - medbot).abs() > maxdel {
                            medbot
                        } else {
                            bot
                        };
                        let del = (factor * medh as f32 - (bot - top) as f32) as i32 / 2;
                        b.set_side(Direction::FromTop, (top - del).max(0), 0);
                        b.set_side(Direction::FromBottom, bot + del, 0);
                    }
                    boxad.push(b);
                }
            }
            CheckType::Both => unreachable!(),
        }

        Ok(boxad)
    }

    /// Extract side coordinates as Numas.
    ///
    /// Returns left, top, right, bottom coordinates for all boxes.
    /// Invalid boxes are filled with nearest valid first.
    ///
    /// C Leptonica equivalent: `boxaPlotSides`
    pub fn plot_sides(&self, _plotname: Option<&str>) -> Result<PlotSidesResult> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter("need at least 2 boxes".into()));
        }

        let boxat = self.fill_sequence(FillFlag::UseAllBoxes, false)?;
        let mut nal = Numa::with_capacity(n);
        let mut nat = Numa::with_capacity(n);
        let mut nar = Numa::with_capacity(n);
        let mut nab = Numa::with_capacity(n);

        for i in 0..boxat.len() {
            let (left, top, w, h) = boxat.get_box_geometry(i).unwrap_or((0, 0, 0, 0));
            let right = left + w - 1;
            let bot = top + h - 1;
            nal.push(left as f32);
            nat.push(top as f32);
            nar.push(right as f32);
            nab.push(bot as f32);
        }

        Ok(PlotSidesResult {
            left: nal,
            top: nat,
            right: nar,
            bottom: nab,
        })
    }

    /// Extract width and height as Numas.
    ///
    /// Returns width and height values for all boxes.
    /// Invalid boxes are filled with nearest valid first.
    ///
    /// C Leptonica equivalent: `boxaPlotSizes`
    pub fn plot_sizes(&self, _plotname: Option<&str>) -> Result<PlotSizesResult> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter("need at least 2 boxes".into()));
        }

        let boxat = self.fill_sequence(FillFlag::UseAllBoxes, false)?;
        let mut naw = Numa::with_capacity(n);
        let mut nah = Numa::with_capacity(n);

        for i in 0..boxat.len() {
            let (_, _, w, h) = boxat.get_box_geometry(i).unwrap_or((0, 0, 0, 0));
            naw.push(w as f32);
            nah.push(h as f32);
        }

        Ok(PlotSizesResult {
            width: naw,
            height: nah,
        })
    }

    /// Compute size variation metrics.
    ///
    /// Returns RMS deviations for even, odd, and all boxes, plus
    /// average even-odd pair difference.
    ///
    /// C Leptonica equivalent: `boxaSizeVariation`
    pub fn size_variation(&self, dim: SelectDimension) -> Result<SizeVariationResult> {
        let n = self.len();
        if n < 4 {
            return Err(Error::InvalidParameter("need at least 4 boxes".into()));
        }

        let (boxae, boxao) = self.split_even_odd(false);
        let ne = boxae.len();
        let no = boxao.len();
        let nmin = ne.min(no);
        if nmin == 0 {
            return Err(Error::InvalidParameter(
                "either no even or no odd boxes".into(),
            ));
        }

        let (nae, nao, na_all) = match dim {
            SelectDimension::Width => {
                let (we, _) = boxae.get_sizes();
                let (wo, _) = boxao.get_sizes();
                let (wa, _) = self.get_sizes();
                (we, wo, wa)
            }
            SelectDimension::Height => {
                let (_, he) = boxae.get_sizes();
                let (_, ho) = boxao.get_sizes();
                let (_, ha) = self.get_sizes();
                (he, ho, ha)
            }
        };

        let mut sum = 0.0f32;
        for i in 0..nmin {
            let vale = nae.get_i32(i).unwrap_or(0);
            let valo = nao.get_i32(i).unwrap_or(0);
            sum += (vale - valo).abs() as f32;
        }
        let del_evenodd = sum / nmin as f32;

        let (_, _, rms_even) = nae.simple_stats(0, -1)?;
        let (_, _, rms_odd) = nao.simple_stats(0, -1)?;
        let (_, _, rms_all) = na_all.simple_stats(0, -1)?;

        Ok(SizeVariationResult {
            del_evenodd,
            rms_even,
            rms_odd,
            rms_all,
        })
    }

    /// Get median dimensions for the box sequence.
    ///
    /// Returns median width/height overall and for even/odd subsets,
    /// plus per-box deviations from the median.
    ///
    /// C Leptonica equivalent: `boxaMedianDimensions`
    pub fn median_dimensions(&self) -> Result<MedianDimensionsResult> {
        if valid_count(self) < 6 {
            return Err(Error::InvalidParameter(
                "need at least 6 valid boxes".into(),
            ));
        }

        let (boxae, boxao) = self.split_even_odd(false);
        if valid_count(&boxae) < 3 || valid_count(&boxao) < 3 {
            return Err(Error::InvalidParameter(
                "need at least 3 valid boxes of each parity".into(),
            ));
        }

        let (_, _, _, _, medw, medh) = self.get_median_vals()?;
        let (_, _, _, _, medwe, medhe) = boxae.get_median_vals()?;
        let (_, _, _, _, medwo, medho) = boxao.get_median_vals()?;

        let n = self.len();
        let mut nadelw = Numa::with_capacity(n);
        let mut nadelh = Numa::with_capacity(n);
        for i in 0..n {
            let (_, _, bw, bh) = self.get_box_geometry(i).unwrap_or((0, 0, 0, 0));
            if bw == 0 || bh == 0 {
                nadelw.push(0.0);
                nadelh.push(0.0);
            } else {
                nadelw.push((bw - medw) as f32);
                nadelh.push((bh - medh) as f32);
            }
        }

        Ok(MedianDimensionsResult {
            med_w: medw,
            med_h: medh,
            med_we: medwe,
            med_wo: medwo,
            med_he: medhe,
            med_ho: medho,
            na_del_w: nadelw,
            na_del_h: nadelh,
        })
    }
}

// ---- Private helpers ----

/// Replace every invalid box with the nearest valid box.
fn fill_all(boxa: &mut Boxa) {
    let n = boxa.len();
    let nv = valid_count(boxa);
    if n == nv || nv == 0 {
        return;
    }

    // Build indicator array
    let indic: Vec<bool> = boxa.iter().map(is_valid_box).collect();

    for i in 0..n {
        if indic[i] {
            continue;
        }
        // Find nearest valid box
        let mut span_down = i32::MAX;
        let mut span_up = i32::MAX;
        for (j, &valid) in indic[..i].iter().enumerate().rev() {
            if valid {
                span_down = (i - j) as i32;
                break;
            }
        }
        for (j, &valid) in indic.iter().enumerate().skip(i + 1) {
            if valid {
                span_up = (j - i) as i32;
                break;
            }
        }
        let src_idx = if span_down <= span_up {
            i - span_down as usize
        } else {
            i + span_up as usize
        };
        let replacement = *boxa.get(src_idx).unwrap();
        let _ = boxa.replace(i, replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_boxa() -> Boxa {
        let mut boxa = Boxa::new();
        // 8 boxes with slightly varying dimensions
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(12, 22, 98, 198));
        boxa.push(Box::new_unchecked(11, 19, 101, 201));
        boxa.push(Box::new_unchecked(13, 21, 99, 199));
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(11, 21, 100, 200));
        boxa.push(Box::new_unchecked(12, 20, 99, 201));
        boxa.push(Box::new_unchecked(10, 22, 101, 199));
        boxa
    }

    // -- fill_sequence --

    #[test]
    fn test_fill_sequence_all_valid() {
        let boxa = make_test_boxa();
        let filled = boxa.fill_sequence(FillFlag::UseAllBoxes, false).unwrap();
        assert_eq!(filled.len(), boxa.len());
    }

    #[test]
    fn test_fill_sequence_with_invalid() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(0, 0, 0, 0)); // invalid
        boxa.push(Box::new_unchecked(12, 22, 98, 198));
        boxa.push(Box::new_unchecked(0, 0, 0, 0)); // invalid
        boxa.push(Box::new_unchecked(11, 19, 101, 201));

        let filled = boxa.fill_sequence(FillFlag::UseAllBoxes, false).unwrap();
        assert_eq!(filled.len(), 5);
        // Box at index 1 should be filled from nearest valid (index 0 or 2)
        assert!(is_valid_box(filled.get(1).unwrap()));
        assert!(is_valid_box(filled.get(3).unwrap()));
    }

    #[test]
    fn test_fill_sequence_same_parity() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200)); // even 0
        boxa.push(Box::new_unchecked(0, 0, 0, 0)); // odd 1 (invalid)
        boxa.push(Box::new_unchecked(12, 22, 98, 198)); // even 2
        boxa.push(Box::new_unchecked(14, 24, 96, 196)); // odd 3
        boxa.push(Box::new_unchecked(11, 19, 101, 201)); // even 4

        let filled = boxa.fill_sequence(FillFlag::UseSameParity, false).unwrap();
        assert_eq!(filled.len(), 5);
        // Odd index 1 should be filled from odd index 3
        let b1 = filled.get(1).unwrap();
        assert!(is_valid_box(b1));
        assert_eq!(b1.x, 14);
    }

    // -- windowed_median --

    #[test]
    fn test_windowed_median_basic() {
        let boxa = make_test_boxa();
        let result = boxa.windowed_median(2, false).unwrap();
        assert_eq!(result.len(), boxa.len());
        // All resulting boxes should be valid
        for i in 0..result.len() {
            assert!(is_valid_box(result.get(i).unwrap()));
        }
    }

    #[test]
    fn test_windowed_median_small() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(12, 22, 98, 198));
        // Only 2 boxes: should return copy
        let result = boxa.windowed_median(1, false).unwrap();
        assert_eq!(result.len(), 2);
    }

    // -- smooth_sequence_median --

    #[test]
    fn test_smooth_sequence_median() {
        let boxa = make_test_boxa();
        let result = boxa
            .smooth_sequence_median(2, ModifySubFlag::UseMinSize, 10, 0, false)
            .unwrap();
        assert_eq!(result.len(), boxa.len());
    }

    #[test]
    fn test_smooth_sequence_median_too_few() {
        let mut boxa = Boxa::new();
        for _ in 0..4 {
            boxa.push(Box::new_unchecked(10, 20, 100, 200));
        }
        // Only 4 boxes: should return copy
        let result = boxa
            .smooth_sequence_median(2, ModifySubFlag::UseMinSize, 10, 0, false)
            .unwrap();
        assert_eq!(result.len(), 4);
    }

    // -- modify_with_boxa --

    #[test]
    fn test_modify_with_boxa_min_size() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));
        boxas.push(Box::new_unchecked(15, 25, 90, 180));

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(12, 18, 96, 204));
        boxam.push(Box::new_unchecked(14, 24, 92, 182));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::UseMinSize, 0, 0)
            .unwrap();
        assert_eq!(result.len(), 2);
        // MinSize: intersection
        let b0 = result.get(0).unwrap();
        assert_eq!(b0.x, 12); // max(10, 12)
        assert_eq!(b0.y, 20); // max(20, 18)
    }

    #[test]
    fn test_modify_with_boxa_max_size() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(12, 18, 96, 204));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::UseMaxSize, 0, 0)
            .unwrap();
        let b0 = result.get(0).unwrap();
        assert_eq!(b0.x, 10); // min(10, 12)
        assert_eq!(b0.y, 18); // min(20, 18)
    }

    #[test]
    fn test_modify_with_boxa_size_mismatch() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));
        let boxam = Boxa::new();
        assert!(
            boxas
                .modify_with_boxa(&boxam, ModifySubFlag::UseMinSize, 0, 0)
                .is_err()
        );
    }

    #[test]
    fn test_modify_with_boxa_invalid_box() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(0, 0, 0, 0)); // invalid

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(12, 18, 96, 204));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::UseMinSize, 0, 0)
            .unwrap();
        assert_eq!(result.get(0).unwrap().w, 0); // stays invalid
    }

    // -- reconcile_pair_width --

    #[test]
    fn test_reconcile_pair_width() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200)); // even: w=100
        boxa.push(Box::new_unchecked(10, 20, 120, 200)); // odd: w=120
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(10, 20, 100, 200));

        let result = boxa
            .reconcile_pair_width(10, AdjustChoose::Min, 1.0, None)
            .unwrap();
        assert_eq!(result.len(), 4);
        // First pair: w=100 vs w=120, diff=20 > delw=10
        // With Min: odd (larger) gets adjusted to factor * min = 1.0 * 100 = 100
        let b1 = result.get(1).unwrap();
        assert_eq!(b1.w, 100);
    }

    // -- size_consistency --

    #[test]
    fn test_size_consistency_same() {
        let boxa = make_test_boxa();
        let result = boxa.size_consistency(CheckType::Width, 0.0, 0.0).unwrap();
        // Nearly identical sizes should give same = 1
        assert_eq!(result.same, 1);
        assert!(result.fvar_pair < 0.1);
    }

    #[test]
    fn test_size_consistency_too_few() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        assert!(boxa.size_consistency(CheckType::Width, 0.0, 0.0).is_err());
    }

    // -- reconcile_all_by_median --

    #[test]
    fn test_reconcile_all_by_median() {
        let boxa = make_test_boxa();
        let result = boxa
            .reconcile_all_by_median(AdjustSelect::LeftAndRight, AdjustSelect::Skip, 5, 0)
            .unwrap();
        assert_eq!(result.len(), boxa.len());
    }

    // -- reconcile_sides_by_median --

    #[test]
    fn test_reconcile_sides_by_median_left() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(50, 20, 100, 200)); // left far from median
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(11, 20, 100, 200));

        let result = boxa
            .reconcile_sides_by_median(AdjustSelect::Left, 5, 0)
            .unwrap();
        // Index 1 had left=50, far from median ~10-11, should be adjusted
        let b1 = result.get(1).unwrap();
        assert!(b1.x < 50);
    }

    // -- reconcile_size_by_median --

    #[test]
    fn test_reconcile_size_by_median() {
        let mut boxa = Boxa::new();
        // 8 boxes, mostly w=100, one outlier
        for _ in 0..7 {
            boxa.push(Box::new_unchecked(10, 20, 100, 200));
        }
        boxa.push(Box::new_unchecked(10, 20, 200, 200)); // width outlier

        let result = boxa
            .reconcile_size_by_median(CheckType::Width, 0.05, 0.04, 1.0)
            .unwrap();
        assert_eq!(result.len(), 8);
        // The outlier at index 7 should have been adjusted closer to median width
        let b7 = result.get(7).unwrap();
        assert!(b7.w < 200);
    }

    // -- plot_sides --

    #[test]
    fn test_plot_sides() {
        let boxa = make_test_boxa();
        let result = boxa.plot_sides(None).unwrap();
        assert_eq!(result.left.len(), boxa.len());
        assert_eq!(result.top.len(), boxa.len());
        assert_eq!(result.right.len(), boxa.len());
        assert_eq!(result.bottom.len(), boxa.len());
    }

    #[test]
    fn test_plot_sides_too_few() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        assert!(boxa.plot_sides(None).is_err());
    }

    // -- plot_sizes --

    #[test]
    fn test_plot_sizes() {
        let boxa = make_test_boxa();
        let result = boxa.plot_sizes(None).unwrap();
        assert_eq!(result.width.len(), boxa.len());
        assert_eq!(result.height.len(), boxa.len());
    }

    // -- size_variation --

    #[test]
    fn test_size_variation() {
        let boxa = make_test_boxa();
        let result = boxa.size_variation(SelectDimension::Width).unwrap();
        // Nearly identical sizes should have small RMS
        assert!(result.rms_all > 0.0);
    }

    #[test]
    fn test_size_variation_too_few() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        assert!(boxa.size_variation(SelectDimension::Width).is_err());
    }

    // -- median_dimensions --

    #[test]
    fn test_median_dimensions() {
        let boxa = make_test_boxa();
        let result = boxa.median_dimensions().unwrap();
        // Median width should be around 100
        assert!((result.med_w - 100).abs() <= 2);
        // Median height should be around 200
        assert!((result.med_h - 200).abs() <= 2);
        assert_eq!(result.na_del_w.len(), boxa.len());
        assert_eq!(result.na_del_h.len(), boxa.len());
    }

    #[test]
    fn test_median_dimensions_too_few() {
        let mut boxa = Boxa::new();
        for _ in 0..3 {
            boxa.push(Box::new_unchecked(10, 20, 100, 200));
        }
        assert!(boxa.median_dimensions().is_err());
    }

    // -- fill_all --

    #[test]
    fn test_fill_all() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(10, 20, 100, 200));
        boxa.push(Box::new_unchecked(0, 0, 0, 0));
        boxa.push(Box::new_unchecked(0, 0, 0, 0));
        boxa.push(Box::new_unchecked(30, 40, 100, 200));

        fill_all(&mut boxa);
        // Index 1 should be filled from index 0 (distance 1)
        assert_eq!(boxa.get(1).unwrap().x, 10);
        // Index 2 should be filled from index 3 (distance 1) since equal distance prefers down
        assert_eq!(boxa.get(2).unwrap().x, 30);
    }

    #[test]
    fn test_fill_all_no_valid() {
        let mut boxa = Boxa::new();
        boxa.push(Box::new_unchecked(0, 0, 0, 0));
        boxa.push(Box::new_unchecked(0, 0, 0, 0));
        // No valid boxes: nothing to fill with
        fill_all(&mut boxa);
        assert_eq!(boxa.get(0).unwrap().w, 0);
    }

    // -- modify_with_boxa sub-flags --

    #[test]
    fn test_modify_with_boxa_sub_on_loc_diff() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(10, 20, 100, 200));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::SubOnLocDiff, 5, 2)
            .unwrap();
        // No difference => use source sides
        let b = result.get(0).unwrap();
        assert_eq!(b.x, 10);
    }

    #[test]
    fn test_modify_with_boxa_capped_min() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(10, 20, 100, 200));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::UseCappedMin, 5, 0)
            .unwrap();
        assert!(is_valid_box(result.get(0).unwrap()));
    }

    #[test]
    fn test_modify_with_boxa_capped_max() {
        let mut boxas = Boxa::new();
        boxas.push(Box::new_unchecked(10, 20, 100, 200));

        let mut boxam = Boxa::new();
        boxam.push(Box::new_unchecked(10, 20, 100, 200));

        let result = boxas
            .modify_with_boxa(&boxam, ModifySubFlag::UseCappedMax, 5, 0)
            .unwrap();
        assert!(is_valid_box(result.get(0).unwrap()));
    }
}
