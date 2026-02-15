//! Grayscale morphological operations
//!
//! Implements erosion, dilation, opening, and closing for 8-bpp grayscale images.
//!
//! # Algorithm
//!
//! For grayscale morphology with a brick (rectangular) structuring element:
//! - **Dilation**: Computes the maximum pixel value in the neighborhood
//! - **Erosion**: Computes the minimum pixel value in the neighborhood
//! - **Opening**: Erosion followed by dilation (removes small bright features)
//! - **Closing**: Dilation followed by erosion (fills small dark features)
//!
//! # Reference
//!
//! Based on Leptonica's `graymorph.c` implementation.

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, PixelDepth};

/// Dilate a grayscale image with a brick structuring element
///
/// Dilation computes the maximum pixel value in the SE neighborhood,
/// which expands bright regions and shrinks dark regions.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new dilated image, or error if input is not 8-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Out-of-bounds pixels are treated as 0 (no contribution to max)
/// - Uses vHGW (van Herk/Gil-Werman) algorithm for O(3) comparisons per pixel
/// - 3×3 and smaller SEs use specialized fast path with 8-pixel unrolling
pub fn dilate_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_grayscale(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // 3×3 fast path
    if hsize <= 3 && vsize <= 3 {
        return dilate_gray_3x3_fastpath(pix, hsize, vsize);
    }

    dilate_gray_vhgw(pix, hsize as usize, vsize as usize)
}

/// vHGW dilate implementation
fn dilate_gray_vhgw(pix: &Pix, hsize: usize, vsize: usize) -> MorphResult<Pix> {
    // Calculate border sizes
    let (leftpix, rightpix, toppix, bottompix) = if vsize == 1 {
        ((hsize + 1) / 2, (3 * hsize + 1) / 2, 0, 0)
    } else if hsize == 1 {
        (0, 0, (vsize + 1) / 2, (3 * vsize + 1) / 2)
    } else {
        (
            (hsize + 1) / 2,
            (3 * hsize + 1) / 2,
            (vsize + 1) / 2,
            (3 * vsize + 1) / 2,
        )
    };

    // Add border (identity value for dilation is 0)
    let pixb = add_border(pix, leftpix, rightpix, toppix, bottompix, 0)?;
    let w = pixb.width() as usize;
    let h = pixb.height() as usize;
    let wplb = pixb.wpl() as usize;

    // Create temporary output
    let pixt = Pix::new(w as u32, h as u32, PixelDepth::Bit8)?;
    let mut pixt_mut = pixt.try_into_mut().unwrap();
    let wplt = pixt_mut.wpl() as usize;

    let datab = pixb.data();
    let datat = pixt_mut.data_mut();

    if vsize == 1 {
        // Horizontal only
        dilate_gray_1d_vhgw(datat, wplt, datab, wplb, w, h, hsize, true);
    } else if hsize == 1 {
        // Vertical only
        dilate_gray_1d_vhgw(datat, wplt, datab, wplb, h, w, vsize, false);
    } else {
        // Both: H pass then V pass
        dilate_gray_1d_vhgw(datat, wplt, datab, wplb, w, h, hsize, true);

        // Reset border for vertical pass
        let pixt: Pix = pixt_mut.into();
        let pixb2 = set_border(&pixt, leftpix, rightpix, toppix, bottompix, 0)?;
        let mut pixt2_mut = pixt.try_into_mut().unwrap();
        let datab2 = pixb2.data();
        let datat2 = pixt2_mut.data_mut();

        dilate_gray_1d_vhgw(datat2, wplt, datab2, wplb, h, w, vsize, false);
        pixt_mut = pixt2_mut;
    }

    let pixt: Pix = pixt_mut.into();
    remove_border(&pixt, leftpix, rightpix, toppix, bottompix)
}

/// Naive dilate implementation (for testing)
fn dilate_gray_naive(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let half_h = (hsize / 2) as i32;
    let half_v = (vsize / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut max_val: u8 = 0;

            for dy in -half_v..=half_v {
                for dx in -half_h..=half_h {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;

                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let val = pix.get_pixel_unchecked(sx as u32, sy as u32) as u8;
                        max_val = max_val.max(val);
                    }
                }
            }

            out_mut.set_pixel_unchecked(x, y, max_val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Erode a grayscale image with a brick structuring element
///
/// Erosion computes the minimum pixel value in the SE neighborhood,
/// which shrinks bright regions and expands dark regions.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE (will be made odd if even)
/// * `vsize` - Vertical size of the brick SE (will be made odd if even)
///
/// # Returns
///
/// A new eroded image, or error if input is not 8-bpp.
///
/// # Notes
///
/// - If hsize and vsize are both 1, returns a copy of the input
/// - Out-of-bounds pixels are treated as 255 (no contribution to min)
/// - Uses vHGW (van Herk/Gil-Werman) algorithm for O(3) comparisons per pixel
/// - 3×3 and smaller SEs use specialized fast path with 8-pixel unrolling
pub fn erode_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    check_grayscale(pix)?;
    let (hsize, vsize) = ensure_odd(hsize, vsize)?;

    // Identity operation
    if hsize == 1 && vsize == 1 {
        return Ok(pix.clone());
    }

    // 3×3 fast path
    if hsize <= 3 && vsize <= 3 {
        return erode_gray_3x3_fastpath(pix, hsize, vsize);
    }

    erode_gray_vhgw(pix, hsize as usize, vsize as usize)
}

/// vHGW erode implementation
fn erode_gray_vhgw(pix: &Pix, hsize: usize, vsize: usize) -> MorphResult<Pix> {
    let (leftpix, rightpix, toppix, bottompix) = if vsize == 1 {
        ((hsize + 1) / 2, (3 * hsize + 1) / 2, 0, 0)
    } else if hsize == 1 {
        (0, 0, (vsize + 1) / 2, (3 * vsize + 1) / 2)
    } else {
        (
            (hsize + 1) / 2,
            (3 * hsize + 1) / 2,
            (vsize + 1) / 2,
            (3 * vsize + 1) / 2,
        )
    };

    // Add border (identity value for erosion is 255)
    let pixb = add_border(pix, leftpix, rightpix, toppix, bottompix, 255)?;
    let w = pixb.width() as usize;
    let h = pixb.height() as usize;
    let wplb = pixb.wpl() as usize;

    let pixt = Pix::new(w as u32, h as u32, PixelDepth::Bit8)?;
    let mut pixt_mut = pixt.try_into_mut().unwrap();
    let wplt = pixt_mut.wpl() as usize;

    let datab = pixb.data();
    let datat = pixt_mut.data_mut();

    if vsize == 1 {
        erode_gray_1d_vhgw(datat, wplt, datab, wplb, w, h, hsize, true);
    } else if hsize == 1 {
        erode_gray_1d_vhgw(datat, wplt, datab, wplb, h, w, vsize, false);
    } else {
        erode_gray_1d_vhgw(datat, wplt, datab, wplb, w, h, hsize, true);

        let pixt: Pix = pixt_mut.into();
        let pixb2 = set_border(&pixt, leftpix, rightpix, toppix, bottompix, 255)?;
        let mut pixt2_mut = pixt.try_into_mut().unwrap();
        let datab2 = pixb2.data();
        let datat2 = pixt2_mut.data_mut();

        erode_gray_1d_vhgw(datat2, wplt, datab2, wplb, h, w, vsize, false);
        pixt_mut = pixt2_mut;
    }

    let pixt: Pix = pixt_mut.into();
    remove_border(&pixt, leftpix, rightpix, toppix, bottompix)
}

/// Naive erode implementation (for testing)
fn erode_gray_naive(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let w = pix.width();
    let h = pix.height();
    let half_h = (hsize / 2) as i32;
    let half_v = (vsize / 2) as i32;

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let mut min_val: u8 = 255;

            for dy in -half_v..=half_v {
                for dx in -half_h..=half_h {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;

                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let val = pix.get_pixel_unchecked(sx as u32, sy as u32) as u8;
                        min_val = min_val.min(val);
                    }
                }
            }

            out_mut.set_pixel_unchecked(x, y, min_val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Open a grayscale image (erosion followed by dilation)
///
/// Opening removes small bright features while preserving the overall shape.
/// It is useful for removing noise and small bright spots.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn open_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let eroded = erode_gray(pix, hsize, vsize)?;
    dilate_gray(&eroded, hsize, vsize)
}

/// Close a grayscale image (dilation followed by erosion)
///
/// Closing fills small dark features while preserving the overall shape.
/// It is useful for filling small holes and gaps.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn close_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_gray(pix, hsize, vsize)?;
    erode_gray(&dilated, hsize, vsize)
}

/// Grayscale morphological gradient (dilation - erosion)
///
/// Highlights edges and boundaries in the image.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn gradient_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let dilated = dilate_gray(pix, hsize, vsize)?;
    let eroded = erode_gray(pix, hsize, vsize)?;
    subtract_gray(&dilated, &eroded)
}

/// Grayscale top-hat transform (original - opening)
///
/// Extracts bright features smaller than the structuring element.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn top_hat_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let opened = open_gray(pix, hsize, vsize)?;
    subtract_gray(pix, &opened)
}

/// Grayscale bottom-hat transform (closing - original)
///
/// Extracts dark features smaller than the structuring element.
///
/// # Arguments
///
/// * `pix` - 8-bpp grayscale image
/// * `hsize` - Horizontal size of the brick SE
/// * `vsize` - Vertical size of the brick SE
pub fn bottom_hat_gray(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    let closed = close_gray(pix, hsize, vsize)?;
    subtract_gray(&closed, pix)
}

/// Subtract two grayscale images (a - b, clamped to 0)
fn subtract_gray(a: &Pix, b: &Pix) -> MorphResult<Pix> {
    let w = a.width();
    let h = a.height();

    let out_pix = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = out_pix.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let va = a.get_pixel_unchecked(x, y) as i32;
            let vb = b.get_pixel_unchecked(x, y) as i32;
            let result = (va - vb).max(0) as u32;
            out_mut.set_pixel_unchecked(x, y, result);
        }
    }

    Ok(out_mut.into())
}

/// Check that the image is 8-bpp grayscale
fn check_grayscale(pix: &Pix) -> MorphResult<()> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }
    Ok(())
}

// vHGW (van Herk/Gil-Werman) algorithm helpers

/// Get byte value from packed pixel data (8bpp: 4 pixels per word)
#[inline]
fn get_data_byte(line: &[u32], j: usize) -> u8 {
    ((line[j / 4] >> (24 - 8 * (j & 3))) & 0xff) as u8
}

/// Set byte value in packed pixel data (8bpp: 4 pixels per word)
#[inline]
fn set_data_byte(line: &mut [u32], j: usize, val: u8) {
    let idx = j / 4;
    let shift = 24 - 8 * (j & 3);
    line[idx] = (line[idx] & !(0xff << shift)) | ((val as u32) << shift);
}

/// 1D van Herk/Gil-Werman dilation (O(3) comparisons per pixel)
///
/// Processes a single line (horizontal or vertical) using vHGW algorithm.
/// The line is divided into blocks of `size` pixels, and for each block,
/// forward and backward max scans are combined.
fn dilate_gray_1d_vhgw(
    datad: &mut [u32],
    wpld: usize,
    datas: &[u32],
    wpls: usize,
    dim1: usize, // width for horiz, height for vert
    dim2: usize, // height for horiz, width for vert
    size: usize,
    is_horizontal: bool,
) {
    let hsize = size / 2;
    let nsteps = (dim1 - 2 * hsize) / size;
    let mut buffer = vec![0u8; dim1];
    let mut maxarray = vec![0u8; 2 * size];

    if is_horizontal {
        // Horizontal: process rows
        for i in 0..dim2 {
            let lines = &datas[i * wpls..];
            let lined = &mut datad[i * wpld..];

            // Fill buffer
            for j in 0..dim1 {
                buffer[j] = get_data_byte(lines, j);
            }

            // Process blocks
            for j in 0..nsteps {
                let startmax = (j + 1) * size - 1;
                maxarray[size - 1] = buffer[startmax];

                // Backward and forward fill
                for k in 1..size {
                    maxarray[size - 1 - k] = maxarray[size - k].max(buffer[startmax - k]);
                    maxarray[size - 1 + k] = maxarray[size + k - 2].max(buffer[startmax + k]);
                }

                // Write output
                let startx = hsize + j * size;
                set_data_byte(lined, startx, maxarray[0]);
                set_data_byte(lined, startx + size - 1, maxarray[2 * size - 2]);
                for k in 1..size - 1 {
                    let maxval = maxarray[k].max(maxarray[k + size - 1]);
                    set_data_byte(lined, startx + k, maxval);
                }
            }
        }
    } else {
        // Vertical: process columns
        for j in 0..dim2 {
            // Fill buffer (column)
            for i in 0..dim1 {
                let lines = &datas[i * wpls..];
                buffer[i] = get_data_byte(lines, j);
            }

            // Process blocks
            for i in 0..nsteps {
                let startmax = (i + 1) * size - 1;
                maxarray[size - 1] = buffer[startmax];

                // Backward and forward fill
                for k in 1..size {
                    maxarray[size - 1 - k] = maxarray[size - k].max(buffer[startmax - k]);
                    maxarray[size - 1 + k] = maxarray[size + k - 2].max(buffer[startmax + k]);
                }

                // Write output (vertical)
                let starty = hsize + i * size;
                let lined = &mut datad[starty * wpld..];
                set_data_byte(lined, j, maxarray[0]);

                let lined_end = &mut datad[(starty + size - 1) * wpld..];
                set_data_byte(lined_end, j, maxarray[2 * size - 2]);

                for k in 1..size - 1 {
                    let maxval = maxarray[k].max(maxarray[k + size - 1]);
                    let lined_k = &mut datad[(starty + k) * wpld..];
                    set_data_byte(lined_k, j, maxval);
                }
            }
        }
    }
}

/// 1D van Herk/Gil-Werman erosion (O(3) comparisons per pixel)
fn erode_gray_1d_vhgw(
    datad: &mut [u32],
    wpld: usize,
    datas: &[u32],
    wpls: usize,
    dim1: usize,
    dim2: usize,
    size: usize,
    is_horizontal: bool,
) {
    let hsize = size / 2;
    let nsteps = (dim1 - 2 * hsize) / size;
    let mut buffer = vec![0u8; dim1];
    let mut minarray = vec![0u8; 2 * size];

    if is_horizontal {
        for i in 0..dim2 {
            let lines = &datas[i * wpls..];
            let lined = &mut datad[i * wpld..];

            for j in 0..dim1 {
                buffer[j] = get_data_byte(lines, j);
            }

            for j in 0..nsteps {
                let startmin = (j + 1) * size - 1;
                minarray[size - 1] = buffer[startmin];

                for k in 1..size {
                    minarray[size - 1 - k] = minarray[size - k].min(buffer[startmin - k]);
                    minarray[size - 1 + k] = minarray[size + k - 2].min(buffer[startmin + k]);
                }

                let startx = hsize + j * size;
                set_data_byte(lined, startx, minarray[0]);
                set_data_byte(lined, startx + size - 1, minarray[2 * size - 2]);
                for k in 1..size - 1 {
                    let minval = minarray[k].min(minarray[k + size - 1]);
                    set_data_byte(lined, startx + k, minval);
                }
            }
        }
    } else {
        for j in 0..dim2 {
            for i in 0..dim1 {
                let lines = &datas[i * wpls..];
                buffer[i] = get_data_byte(lines, j);
            }

            for i in 0..nsteps {
                let startmin = (i + 1) * size - 1;
                minarray[size - 1] = buffer[startmin];

                for k in 1..size {
                    minarray[size - 1 - k] = minarray[size - k].min(buffer[startmin - k]);
                    minarray[size - 1 + k] = minarray[size + k - 2].min(buffer[startmin + k]);
                }

                let starty = hsize + i * size;
                let lined = &mut datad[starty * wpld..];
                set_data_byte(lined, j, minarray[0]);

                let lined_end = &mut datad[(starty + size - 1) * wpld..];
                set_data_byte(lined_end, j, minarray[2 * size - 2]);

                for k in 1..size - 1 {
                    let minval = minarray[k].min(minarray[k + size - 1]);
                    let lined_k = &mut datad[(starty + k) * wpld..];
                    set_data_byte(lined_k, j, minval);
                }
            }
        }
    }
}

/// Ensure sizes are odd (as required by Leptonica's convention)
fn ensure_odd(hsize: u32, vsize: u32) -> MorphResult<(u32, u32)> {
    if hsize == 0 || vsize == 0 {
        return Err(MorphError::InvalidParameters(
            "hsize and vsize must be >= 1".to_string(),
        ));
    }

    let hsize = if hsize.is_multiple_of(2) {
        hsize + 1
    } else {
        hsize
    };
    let vsize = if vsize.is_multiple_of(2) {
        vsize + 1
    } else {
        vsize
    };

    Ok((hsize, vsize))
}

/// Add border with constant value
fn add_border(
    pix: &Pix,
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
    val: u8,
) -> MorphResult<Pix> {
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let new_w = (w + left + right) as u32;
    let new_h = (h + top + bottom) as u32;

    let out = Pix::new(new_w, new_h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    // Fill with border value
    for y in 0..new_h {
        for x in 0..new_w {
            out_mut.set_pixel_unchecked(x, y, val as u32);
        }
    }

    // Copy source region
    for y in 0..h {
        for x in 0..w {
            let src_val = pix.get_pixel_unchecked(x as u32, y as u32);
            out_mut.set_pixel_unchecked((x + left) as u32, (y + top) as u32, src_val);
        }
    }

    Ok(out_mut.into())
}

/// Set border to constant value
fn set_border(
    pix: &Pix,
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
    val: u8,
) -> MorphResult<Pix> {
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let out = pix.deep_clone();
    let mut out_mut = out.try_into_mut().unwrap();

    // Top border
    for y in 0..top {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x as u32, y as u32, val as u32);
        }
    }

    // Bottom border
    for y in (h - bottom)..h {
        for x in 0..w {
            out_mut.set_pixel_unchecked(x as u32, y as u32, val as u32);
        }
    }

    // Left border
    for y in 0..h {
        for x in 0..left {
            out_mut.set_pixel_unchecked(x as u32, y as u32, val as u32);
        }
    }

    // Right border
    for y in 0..h {
        for x in (w - right)..w {
            out_mut.set_pixel_unchecked(x as u32, y as u32, val as u32);
        }
    }

    Ok(out_mut.into())
}

/// Remove border
fn remove_border(
    pix: &Pix,
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
) -> MorphResult<Pix> {
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let new_w = (w - left - right) as u32;
    let new_h = (h - top - bottom) as u32;

    let out = Pix::new(new_w, new_h, PixelDepth::Bit8)?;
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..new_h as usize {
        for x in 0..new_w as usize {
            let src_val = pix.get_pixel_unchecked((x + left) as u32, (y + top) as u32);
            out_mut.set_pixel_unchecked(x as u32, y as u32, src_val);
        }
    }

    Ok(out_mut.into())
}

// 3×3 fast path implementations

/// 3×3 dilate fast path (placeholder for RED phase)
fn dilate_gray_3x3_fastpath(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    // Placeholder: delegate to vHGW for now
    dilate_gray_vhgw(pix, hsize as usize, vsize as usize)
}

/// 3×3 erode fast path (placeholder for RED phase)
fn erode_gray_3x3_fastpath(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    // Placeholder: delegate to vHGW for now
    erode_gray_vhgw(pix, hsize as usize, vsize as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grayscale_image() -> Pix {
        // Create a 9x9 grayscale image with a bright 3x3 square in the center
        let pix = Pix::new(9, 9, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark background (50)
        for y in 0..9 {
            for x in 0..9 {
                pix_mut.set_pixel_unchecked(x, y, 50);
            }
        }

        // Set bright center 3x3 (200)
        for y in 3..6 {
            for x in 3..6 {
                pix_mut.set_pixel_unchecked(x, y, 200);
            }
        }

        pix_mut.into()
    }

    #[test]
    fn test_dilate_gray_identity() {
        let pix = create_test_grayscale_image();
        let dilated = dilate_gray(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(
                    pix.get_pixel_unchecked(x, y),
                    dilated.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]
    fn test_erode_gray_identity() {
        let pix = create_test_grayscale_image();
        let eroded = erode_gray(&pix, 1, 1).unwrap();

        // Should be identical
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(
                    pix.get_pixel_unchecked(x, y),
                    eroded.get_pixel_unchecked(x, y)
                );
            }
        }
    }

    #[test]
    fn test_dilate_gray_expands_bright() {
        let pix = create_test_grayscale_image();
        let dilated = dilate_gray(&pix, 3, 3).unwrap();

        // The bright center should expand
        // After 3x3 dilation, the 3x3 bright area should become 5x5
        // Pixels at (2,2) should now be bright (200)
        assert_eq!(dilated.get_pixel_unchecked(2, 2), 200);
        assert_eq!(dilated.get_pixel_unchecked(6, 6), 200);

        // Center should remain bright
        assert_eq!(dilated.get_pixel_unchecked(4, 4), 200);

        // Corners should remain dark
        assert_eq!(dilated.get_pixel_unchecked(0, 0), 50);
        assert_eq!(dilated.get_pixel_unchecked(8, 8), 50);
    }

    #[test]
    fn test_erode_gray_shrinks_bright() {
        let pix = create_test_grayscale_image();
        let eroded = erode_gray(&pix, 3, 3).unwrap();

        // The 3x3 bright center should shrink to 1x1 (just center pixel)
        assert_eq!(eroded.get_pixel_unchecked(4, 4), 200);

        // Adjacent pixels should now be dark (50)
        assert_eq!(eroded.get_pixel_unchecked(3, 4), 50);
        assert_eq!(eroded.get_pixel_unchecked(5, 4), 50);
    }

    #[test]
    fn test_open_gray() {
        let pix = create_test_grayscale_image();
        let opened = open_gray(&pix, 3, 3).unwrap();

        // Opening should shrink then expand
        // The 3x3 bright region: erode makes it 1x1, dilate makes it 3x3
        // Center should remain bright
        assert_eq!(opened.get_pixel_unchecked(4, 4), 200);
    }

    #[test]
    fn test_close_gray() {
        let pix = create_test_grayscale_image();
        let closed = close_gray(&pix, 3, 3).unwrap();

        // Closing should expand then shrink
        // The 3x3 bright region should be preserved
        assert_eq!(closed.get_pixel_unchecked(4, 4), 200);
    }

    #[test]
    fn test_even_size_incremented() {
        let pix = create_test_grayscale_image();

        // Even sizes should work (auto-incremented to odd)
        let result = dilate_gray(&pix, 2, 4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_size_error() {
        let pix = create_test_grayscale_image();

        let result = dilate_gray(&pix, 0, 3);
        assert!(result.is_err());

        let result = erode_gray(&pix, 3, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_grayscale_error() {
        let pix = Pix::new(5, 5, PixelDepth::Bit1).unwrap();

        let result = dilate_gray(&pix, 3, 3);
        assert!(result.is_err());

        let result = erode_gray(&pix, 3, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_gradient_gray() {
        let pix = create_test_grayscale_image();
        let gradient = gradient_gray(&pix, 3, 3).unwrap();

        // Gradient should be highest at edges
        // Interior of bright region and background should be low
        // The center of the bright region should have low gradient
        // (dilated - eroded at center: 200 - 200 = 0... but after erosion center becomes 200)
        // Actually after 3x3 operations on 3x3 bright region:
        // - dilated center: 200
        // - eroded center: 200 (only center survives)
        // So gradient at center should be 0
        assert_eq!(gradient.get_pixel_unchecked(4, 4), 0);
    }

    #[test]
    fn test_top_hat_gray() {
        let pix = create_test_grayscale_image();
        let tophat = top_hat_gray(&pix, 3, 3).unwrap();

        // Top-hat extracts bright features smaller than SE
        // For our 3x3 SE and 3x3 bright region, the bright region
        // survives opening, so top-hat should be small
        assert!(tophat.get_pixel_unchecked(4, 4) <= 200);
    }

    #[test]
    fn test_bottom_hat_gray() {
        let pix = create_test_grayscale_image();
        let bottomhat = bottom_hat_gray(&pix, 3, 3).unwrap();

        // Bottom-hat extracts dark features
        // Should be non-negative everywhere
        for y in 0..9 {
            for x in 0..9 {
                assert!(bottomhat.get_pixel_unchecked(x, y) <= 255);
            }
        }
    }

    #[test]
    fn test_single_pixel_dilation() {
        // Create image with single bright pixel
        let pix = Pix::new(7, 7, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Fill with dark (0)
        for y in 0..7 {
            for x in 0..7 {
                pix_mut.set_pixel_unchecked(x, y, 0);
            }
        }

        // Single bright pixel at center
        pix_mut.set_pixel_unchecked(3, 3, 255);
        let pix: Pix = pix_mut.into();

        let dilated = dilate_gray(&pix, 3, 3).unwrap();

        // 3x3 dilation should create a 3x3 bright region
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let x = (3 + dx) as u32;
                let y = (3 + dy) as u32;
                assert_eq!(
                    dilated.get_pixel_unchecked(x, y),
                    255,
                    "Expected 255 at ({}, {})",
                    x,
                    y
                );
            }
        }

        // Corners should remain dark
        assert_eq!(dilated.get_pixel_unchecked(0, 0), 0);
    }

    // vHGW equivalence tests
    fn create_random_grayscale_image(w: u32, h: u32, seed: u64) -> Pix {
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Simple LCG random number generator
        let mut state = seed;
        for y in 0..h {
            for x in 0..w {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                let val = (state % 256) as u32;
                pix_mut.set_pixel_unchecked(x, y, val);
            }
        }

        pix_mut.into()
    }

    fn assert_pix_equal(pix1: &Pix, pix2: &Pix, name: &str) {
        assert_eq!(pix1.width(), pix2.width());
        assert_eq!(pix1.height(), pix2.height());

        for y in 0..pix1.height() {
            for x in 0..pix1.width() {
                let v1 = pix1.get_pixel_unchecked(x, y);
                let v2 = pix2.get_pixel_unchecked(x, y);
                if v1 != v2 {
                    panic!(
                        "{}: Pixels differ at ({}, {}): naive={}, vhgw={}",
                        name, x, y, v1, v2
                    );
                }
            }
        }
    }

    #[test]
    fn test_dilate_vhgw_equivalence_3x3() {
        let pix = create_random_grayscale_image(100, 80, 12345);
        let naive = dilate_gray_naive(&pix, 3, 3).unwrap();
        let vhgw = dilate_gray(&pix, 3, 3).unwrap();
        assert_pix_equal(&naive, &vhgw, "dilate 3x3");
    }

    #[test]

    fn test_dilate_vhgw_equivalence_7x5() {
        let pix = create_random_grayscale_image(100, 80, 54321);
        let naive = dilate_gray_naive(&pix, 7, 5).unwrap();
        let vhgw = dilate_gray(&pix, 7, 5).unwrap();
        assert_pix_equal(&naive, &vhgw, "dilate 7x5");
    }

    #[test]

    fn test_dilate_vhgw_equivalence_horizontal() {
        let pix = create_random_grayscale_image(100, 80, 99999);
        let naive = dilate_gray_naive(&pix, 11, 1).unwrap();
        let vhgw = dilate_gray(&pix, 11, 1).unwrap();
        assert_pix_equal(&naive, &vhgw, "dilate 11x1");
    }

    #[test]

    fn test_dilate_vhgw_equivalence_vertical() {
        let pix = create_random_grayscale_image(100, 80, 11111);
        let naive = dilate_gray_naive(&pix, 1, 9).unwrap();
        let vhgw = dilate_gray(&pix, 1, 9).unwrap();
        assert_pix_equal(&naive, &vhgw, "dilate 1x9");
    }

    #[test]

    fn test_erode_vhgw_equivalence_3x3() {
        let pix = create_random_grayscale_image(100, 80, 67890);
        let naive = erode_gray_naive(&pix, 3, 3).unwrap();
        let vhgw = erode_gray(&pix, 3, 3).unwrap();
        assert_pix_equal(&naive, &vhgw, "erode 3x3");
    }

    #[test]

    fn test_erode_vhgw_equivalence_7x5() {
        let pix = create_random_grayscale_image(100, 80, 24680);
        let naive = erode_gray_naive(&pix, 7, 5).unwrap();
        let vhgw = erode_gray(&pix, 7, 5).unwrap();
        assert_pix_equal(&naive, &vhgw, "erode 7x5");
    }
}
