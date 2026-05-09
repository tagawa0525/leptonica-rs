//! Rectangle detection inside 1bpp images.
//!
//! C Leptonica equivalent: `pageseg.c::pixFindLargestRectangle`,
//! `pixFindLargeRectangles`, `pixFindRectangleInCC`.

use crate::core::box_::Box;
use crate::core::{Boxa, Pix, PixelDepth};
use crate::region::error::{RegionError, RegionResult};

/// Pixel polarity selector for largest-rectangle search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// Search inside the background (white = 0). Maps to C `polarity = 0`.
    Background,
    /// Search inside the foreground (black = 1). Maps to C `polarity = 1`.
    Foreground,
}

/// Fast-scan direction for [`find_rectangle_in_cc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    /// `L_SCAN_HORIZONTAL` — fast scan along rows.
    Horizontal,
    /// `L_SCAN_VERTICAL` — fast scan along columns.
    Vertical,
}

/// How [`find_rectangle_in_cc`] combines the box found from each slow-scan
/// direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectSelect {
    /// `L_GEOMETRIC_UNION` — bounding box of the two boxes.
    GeometricUnion,
    /// `L_GEOMETRIC_INTERSECTION` — overlap region.
    GeometricIntersection,
    /// `L_LARGEST_AREA` — the larger of the two by area.
    LargestArea,
    /// `L_SMALLEST_AREA` — the smaller of the two by area.
    SmallestArea,
}

fn require_1bpp(pix: &Pix) -> RegionResult<()> {
    if pix.depth() != PixelDepth::Bit1 {
        Err(RegionError::UnsupportedDepth {
            expected: "1bpp",
            actual: pix.depth().bits(),
        })
    } else {
        Ok(())
    }
}

/// C Leptonica equivalent: `pixFindLargestRectangle`.
///
/// O(W×H) DP: at each pixel `(j, i)` we maintain `(linew, lineh)` = the
/// (width, height) of the largest mono-coloured rectangle whose lower-right
/// corner sits there. The recurrence picks the better of "extend the rectangle
/// from the pixel above downward" vs "extend the rectangle from the pixel to
/// the left rightward", clipped by the most recent opposite-colour pixel seen
/// in the current row / column.
pub fn find_largest_rectangle(pix: &Pix, polarity: Polarity) -> RegionResult<Box> {
    require_1bpp(pix)?;
    let w = pix.width() as i32;
    let h = pix.height() as i32;
    if w == 0 || h == 0 {
        return Err(RegionError::EmptyImage);
    }

    let pol = match polarity {
        Polarity::Background => 0u32,
        Polarity::Foreground => 1u32,
    };

    let wsz = w as usize;
    let hsz = h as usize;
    let mut linew = vec![0i32; wsz * hsz];
    let mut lineh = vec![0i32; wsz * hsz];
    // Most recent row position (in this row) where the opposite-colour pixel
    // was seen. -1 means "never yet in this row".
    let mut lowestfg = vec![-1i32; wsz];

    let mut maxarea = 0i32;
    let (mut xmax, mut ymax, mut wmax, mut hmax) = (0i32, 0i32, 0i32, 0i32);

    for i in 0..h {
        let mut prevfg = -1i32;
        for j in 0..w {
            let val = pix.get_pixel_unchecked(j as u32, i as u32);
            let in_target = (val ^ pol) == 0;
            let (wp, hp) = if in_target {
                if i == 0 && j == 0 {
                    (1, 1)
                } else if i == 0 {
                    (linew[(j - 1) as usize] + 1, 1)
                } else if j == 0 {
                    (1, lineh[((i - 1) as usize) * wsz] + 1)
                } else {
                    let w1 = linew[((i - 1) as usize) * wsz + j as usize];
                    let h1 = lineh[((i - 1) as usize) * wsz + j as usize];
                    let horizdist = j - prevfg;
                    let wmin = w1.min(horizdist);
                    let area1 = wmin * (h1 + 1);

                    let w2 = linew[(i as usize) * wsz + (j - 1) as usize];
                    let h2 = lineh[(i as usize) * wsz + (j - 1) as usize];
                    let vertdist = i - lowestfg[j as usize];
                    let hmin = h2.min(vertdist);
                    let area2 = hmin * (w2 + 1);

                    if area1 == 0 && area2 == 0 {
                        // Isolated target pixel with non-target above and to
                        // the left: the C version's recurrence collapses to
                        // 0×0 here, which silently drops the pixel from the
                        // search. Treat the pixel itself as the 1×1 rectangle.
                        (1, 1)
                    } else if area1 > area2 {
                        (wmin, h1 + 1)
                    } else {
                        (w2 + 1, hmin)
                    }
                }
            } else {
                prevfg = j;
                lowestfg[j as usize] = i;
                (0, 0)
            };
            linew[(i as usize) * wsz + j as usize] = wp;
            lineh[(i as usize) * wsz + j as usize] = hp;
            if wp * hp > maxarea {
                maxarea = wp * hp;
                xmax = j;
                ymax = i;
                wmax = wp;
                hmax = hp;
            }
        }
    }

    if maxarea == 0 {
        // No pixel of the requested polarity exists; return a degenerate
        // 0-size box at origin rather than synthesising bogus coordinates.
        return Box::new(0, 0, 0, 0)
            .map_err(|e| RegionError::InvalidParameters(format!("Box::new(0,0,0,0): {e}")));
    }
    Box::new(xmax - wmax + 1, ymax - hmax + 1, wmax, hmax)
        .map_err(|e| RegionError::InvalidParameters(format!("Box::new: {e}")))
}

/// C Leptonica equivalent: `pixFindLargeRectangles`.
///
/// Greedy: repeatedly find the largest rectangle in `pix`, fill it with the
/// opposite colour and search again. `nrect` is clamped to `1000` to match the
/// C-version safety cap.
pub fn find_large_rectangles(pix: &Pix, polarity: Polarity, nrect: u32) -> RegionResult<Boxa> {
    require_1bpp(pix)?;
    let mut boxa = Boxa::with_capacity(nrect.min(1000) as usize);
    if nrect == 0 {
        return Ok(boxa);
    }
    let nrect = nrect.min(1000);

    // Work on a mutable copy.
    let mut work: Pix = pix.deep_clone();
    for _ in 0..nrect {
        let b = find_largest_rectangle(&work, polarity)?;
        if b.w == 0 || b.h == 0 {
            // No more selectable rectangles for this polarity; stop early
            // rather than padding the result with degenerate boxes.
            break;
        }
        boxa.push(b);
        // Fill the box with the opposite colour to prevent re-selection.
        let mut pm = work.to_mut();
        match polarity {
            Polarity::Background => pm.set_in_rect(&b)?,
            Polarity::Foreground => pm.clear_in_rect(&b)?,
        }
        work = pm.into();
    }
    Ok(boxa)
}

/// C Leptonica equivalent: `pixFindRectangleInCC`.
///
/// Walks scanlines from both top→bottom and bottom→top finding the first row
/// whose longest fg run is at least `fract * w`, then continues until the run
/// shrinks. Combines the two boxes via `select`.
pub fn find_rectangle_in_cc(
    pix: &Pix,
    boxs: Option<&Box>,
    fract: f32,
    dir: ScanDirection,
    select: RectSelect,
) -> RegionResult<Option<Box>> {
    use crate::filter::runlength::find_max_horizontal_run_on_line;
    use crate::transform::rotate::rotate_orth;

    require_1bpp(pix)?;
    if !(0.0 < fract && fract <= 1.0) {
        return Err(RegionError::InvalidParameters(format!(
            "fract must be in (0, 1], got {fract}"
        )));
    }

    // Optional clip + offset for the result coordinates. C Leptonica accepts
    // any boxs and silently clips, but in Rust we reject out-of-bounds boxs
    // so the result coordinates aren't ambiguous (the offset added below has
    // to match the *actual* clip origin, which differs from `boxs` if it was
    // partly outside the image).
    let (offset_x, offset_y, pix1) = if let Some(b) = boxs {
        let pw = pix.width() as i32;
        let ph = pix.height() as i32;
        if b.x < 0 || b.y < 0 || b.w <= 0 || b.h <= 0 || b.x + b.w > pw || b.y + b.h > ph {
            return Err(RegionError::InvalidParameters(format!(
                "boxs {:?} is out of bounds for {}x{} image",
                b, pw, ph
            )));
        }
        let clipped = pix.clip_rectangle(b.x as u32, b.y as u32, b.w as u32, b.h as u32)?;
        (b.x, b.y, clipped)
    } else {
        (0, 0, pix.clone())
    };

    // Always operate with horizontal fast-scan; rotate 90° cw if vertical.
    let pix2 = if dir == ScanDirection::Vertical {
        rotate_orth(&pix1, 1)
            .map_err(|e| RegionError::InvalidParameters(format!("rotate_orth: {e}")))?
    } else {
        pix1.clone()
    };
    let w = pix2.width() as i32;
    let h = pix2.height() as i32;
    let threshold = (fract * w as f32 + 0.5) as i32;

    // Top-down pass.
    let mut found = false;
    let (mut xfirst, mut yfirst, mut xlast, mut ylast) = (0i32, 0i32, 0i32, 0i32);
    for i in 0..h {
        let (xstart, length) = find_max_horizontal_run_on_line(&pix2, i as u32);
        if length as i32 >= threshold {
            yfirst = i;
            xfirst = xstart as i32;
            xlast = xfirst + length as i32 - 1;
            found = true;
            break;
        }
    }
    if !found {
        return Ok(None);
    }
    let mut top_h1 = h - yfirst;
    let (top_xfirst, top_yfirst) = (xfirst, yfirst);
    for i in (yfirst + 1)..h {
        let (xstart, length) = find_max_horizontal_run_on_line(&pix2, i as u32);
        let xs = xstart as i32;
        let xe = xs + length as i32 - 1;
        if xs > top_xfirst || xe < xlast || i == h - 1 {
            top_h1 = i - top_yfirst;
            break;
        }
    }
    let top_w1 = xlast - top_xfirst + 1;
    let box1 = Box::new(top_xfirst, top_yfirst, top_w1, top_h1)
        .map_err(|e| RegionError::InvalidParameters(format!("Box::new top: {e}")))?;

    // Bottom-up pass.
    for i in (0..h).rev() {
        let (xstart, length) = find_max_horizontal_run_on_line(&pix2, i as u32);
        if length as i32 >= threshold {
            ylast = i;
            xfirst = xstart as i32;
            xlast = xfirst + length as i32 - 1;
            break;
        }
    }
    let mut bot_h2 = ylast + 1;
    let (bot_xfirst, bot_ylast) = (xfirst, ylast);
    yfirst = 0;
    for i in (0..ylast).rev() {
        let (xstart, length) = find_max_horizontal_run_on_line(&pix2, i as u32);
        let xs = xstart as i32;
        let xe = xs + length as i32 - 1;
        if xs > bot_xfirst || xe < xlast || i == 0 {
            yfirst = i + 1;
            bot_h2 = bot_ylast - yfirst + 1;
            break;
        }
    }
    let bot_w2 = xlast - bot_xfirst + 1;
    let box2 = Box::new(bot_xfirst, yfirst, bot_w2, bot_h2)
        .map_err(|e| RegionError::InvalidParameters(format!("Box::new bot: {e}")))?;

    // Combine.
    let area1 = (box1.w as i64) * (box1.h as i64);
    let area2 = (box2.w as i64) * (box2.h as i64);
    let combined: Option<Box> = match select {
        RectSelect::GeometricUnion => Some(union_box(&box1, &box2)),
        RectSelect::GeometricIntersection => intersection_box(&box1, &box2),
        RectSelect::LargestArea => Some(if area1 >= area2 { box1 } else { box2 }),
        RectSelect::SmallestArea => Some(if area1 <= area2 { box1 } else { box2 }),
    };

    let Some(mut r) = combined else {
        return Ok(None);
    };

    // Rotate the box back to the source frame if we rotated the image.
    if dir == ScanDirection::Vertical {
        // 90° cw rotation took source (x_s, y_s) of size (w_s, h_s) into
        // pix2 with x = y_s, y = w_s - 1 - x_s. The inverse maps the box
        // (x, y, w, h) back to source coords:
        let src_h = pix1.height() as i32;
        let new_x = r.y;
        let new_y = src_h - 1 - (r.x + r.w - 1);
        r = Box::new(new_x, new_y, r.h, r.w)
            .map_err(|e| RegionError::InvalidParameters(format!("rotate-back: {e}")))?;
    }

    if offset_x != 0 || offset_y != 0 {
        r = Box::new(r.x + offset_x, r.y + offset_y, r.w, r.h)
            .map_err(|e| RegionError::InvalidParameters(format!("offset: {e}")))?;
    }

    Ok(Some(r))
}

fn union_box(a: &Box, b: &Box) -> Box {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let xr = (a.x + a.w).max(b.x + b.w);
    let yb = (a.y + a.h).max(b.y + b.h);
    Box::new(x, y, xr - x, yb - y).expect("non-degenerate union")
}

fn intersection_box(a: &Box, b: &Box) -> Option<Box> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let xr = (a.x + a.w).min(b.x + b.w);
    let yb = (a.y + a.h).min(b.y + b.h);
    if xr > x && yb > y {
        Some(Box::new(x, y, xr - x, yb - y).expect("intersection valid"))
    } else {
        None
    }
}
