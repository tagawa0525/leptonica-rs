//! Text line detection for dewarping
//!
//! This module provides functionality to detect text line centers
//! in a binary image. These centers are used to build the disparity model.

use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};
use leptonica_morph::{close_brick, erode_brick, open_brick};
use leptonica_region::{ConnectivityType, find_connected_components};

use super::types::TextLine;

/// Find the centers of text lines in a binary image
///
/// This function identifies text lines and returns points along their centers.
/// These points are used to build the vertical disparity model.
///
/// # Arguments
///
/// * `pix` - Input binary image (1 bpp)
///
/// # Returns
///
/// A vector of `TextLine` objects, each containing points along the center
/// of a text line.
pub fn find_textline_centers(pix: &Pix) -> RecogResult<Vec<TextLine>> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(RecogError::UnsupportedDepth {
            expected: "1 bpp",
            actual: pix.depth().bits(),
        });
    }

    let w = pix.width();
    let _h = pix.height();

    // Filter to solidify text lines within x-height region
    // and remove ascenders/descenders
    // Step 1: Small vertical opening to remove noise
    let pix1 = open_brick(pix, 1, 3)?;

    // Step 2: Small closing to bridge gaps between letters
    let csize1 = (w / 80).max(15);
    let pix2 = close_brick(&pix1, csize1, 1)?;

    // Step 3: Opening to remove thin connections
    let pix3 = open_brick(&pix2, csize1, 1)?;

    // Step 4: Large closing to bridge gaps between words
    let csize2 = (w / 30).max(40);
    let pix4 = close_brick(&pix3, csize2, 1)?;

    // Remove tall components (embedded images) by finding components
    // with long vertical runs
    let seed = erode_brick(&pix4, 1, 50)?;
    let tall_components = seed_fill_binary(&seed, &pix4)?;
    let filtered = xor_pix(&pix4, &tall_components)?;

    // Get connected components
    let components = find_connected_components(&filtered, ConnectivityType::EightWay)?;

    if components.is_empty() {
        return Ok(vec![]);
    }

    // Filter out small components (width < 100 or height < 4) and get centers
    let mut text_lines = Vec::new();

    for comp in components.iter() {
        let bx = comp.bounds.x;
        let by = comp.bounds.y;
        let bw = comp.bounds.w as u32;
        let bh = comp.bounds.h as u32;

        if bw < 100 || bh < 4 {
            continue;
        }

        // Get the weighted center of each vertical column for this component
        // We scan the original filtered image within the bounding box
        let centers = get_mean_verticals_from_box(&filtered, bx, by, bw, bh);
        if !centers.is_empty() {
            text_lines.push(TextLine::new(centers));
        }
    }

    Ok(text_lines)
}

/// Get the weighted center of each vertical column within a bounding box
///
/// For each x-coordinate in the bounding box, compute the centroid y-coordinate
/// of all foreground pixels in that column.
fn get_mean_verticals_from_box(pix: &Pix, bx: i32, by: i32, bw: u32, bh: u32) -> Vec<(f32, f32)> {
    let mut centers = Vec::with_capacity(bw as usize);
    let img_w = pix.width();
    let img_h = pix.height();

    // Iterate over columns in the bounding box
    for dx in 0..bw {
        let x = (bx + dx as i32) as u32;
        if x >= img_w {
            continue;
        }

        let mut sum_y = 0u32;
        let mut count = 0u32;

        for dy in 0..bh {
            let y = (by + dy as i32) as u32;
            if y >= img_h {
                continue;
            }

            // Check if this pixel is foreground
            let pixel = unsafe { pix.get_pixel_unchecked(x, y) };
            if pixel != 0 {
                sum_y += y;
                count += 1;
            }
        }

        if count > 0 {
            let mean_y = (sum_y as f32) / (count as f32);
            centers.push((x as f32, mean_y));
        }
    }

    centers
}

/// Perform seed fill (binary reconstruction)
///
/// Grow the seed under the constraint of the mask until no changes occur.
fn seed_fill_binary(seed: &Pix, mask: &Pix) -> RecogResult<Pix> {
    let w = seed.width();
    let h = seed.height();

    if w != mask.width() || h != mask.height() {
        return Err(RecogError::InvalidParameter(
            "seed and mask must have same dimensions".to_string(),
        ));
    }

    // Clone seed as starting point
    let result = seed.deep_clone();
    let mut result_mut = result.try_into_mut().unwrap();

    // Iterate until no changes
    let max_iterations = (w + h) as usize; // Maximum iterations needed
    for _ in 0..max_iterations {
        let mut changed = false;

        // Dilate and AND with mask
        for y in 0..h {
            for x in 0..w {
                if unsafe { result_mut.get_pixel_unchecked(x, y) } == 0 {
                    // Check if any 8-connected neighbor is set
                    let mut has_neighbor = false;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0
                                && nx < w as i32
                                && ny >= 0
                                && ny < h as i32
                                && unsafe { result_mut.get_pixel_unchecked(nx as u32, ny as u32) }
                                    != 0
                            {
                                has_neighbor = true;
                                break;
                            }
                        }
                        if has_neighbor {
                            break;
                        }
                    }

                    // If has neighbor and mask is set, fill
                    if has_neighbor && unsafe { mask.get_pixel_unchecked(x, y) } != 0 {
                        unsafe { result_mut.set_pixel_unchecked(x, y, 1) };
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    Ok(result_mut.into())
}

/// XOR two binary images
fn xor_pix(pix1: &Pix, pix2: &Pix) -> RecogResult<Pix> {
    let w = pix1.width();
    let h = pix1.height();

    if w != pix2.width() || h != pix2.height() {
        return Err(RecogError::InvalidParameter(
            "images must have same dimensions".to_string(),
        ));
    }

    let result = Pix::new(w, h, PixelDepth::Bit1)?;
    let mut result_mut = result.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let v1 = unsafe { pix1.get_pixel_unchecked(x, y) };
            let v2 = unsafe { pix2.get_pixel_unchecked(x, y) };
            unsafe { result_mut.set_pixel_unchecked(x, y, v1 ^ v2) };
        }
    }

    Ok(result_mut.into())
}

/// Remove short lines from the list
///
/// Lines shorter than `min_fraction` of the longest line are removed.
///
/// # Arguments
///
/// * `lines` - Vector of text lines
/// * `min_fraction` - Minimum fraction of longest line (typically 0.8)
///
/// # Returns
///
/// Filtered vector of text lines
pub fn remove_short_lines(lines: Vec<TextLine>, min_fraction: f32) -> Vec<TextLine> {
    if lines.is_empty() {
        return lines;
    }

    // Find the longest line
    let max_extent = lines
        .iter()
        .map(|l| l.horizontal_extent())
        .fold(0.0f32, f32::max);

    let min_extent = max_extent * min_fraction;

    // Keep only lines at least min_extent long
    lines
        .into_iter()
        .filter(|l| l.horizontal_extent() >= min_extent)
        .collect()
}

/// Check if lines have valid coverage of the image height
///
/// Returns true if there are enough lines in both the top and bottom halves.
pub fn is_line_coverage_valid(lines: &[TextLine], image_height: u32, min_lines: u32) -> bool {
    if lines.len() < min_lines as usize {
        return false;
    }

    let mid_y = (image_height / 2) as f32;
    let mut n_top = 0;
    let mut n_bot = 0;

    for line in lines {
        if let Some(y) = line.mid_y() {
            if y < mid_y {
                n_top += 1;
            } else {
                n_bot += 1;
            }
        }
    }

    // Need at least 3 lines in each half
    n_top >= 3 && n_bot >= 3
}

/// Sort lines by their vertical position (top to bottom)
pub fn sort_lines_by_y(lines: &mut [TextLine]) {
    lines.sort_by(|a, b| {
        let ya = a.mid_y().unwrap_or(0.0);
        let yb = b.mid_y().unwrap_or(0.0);
        ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_line(y: f32, x_start: f32, x_end: f32) -> TextLine {
        let mut points = Vec::new();
        let mut x = x_start;
        while x <= x_end {
            points.push((x, y));
            x += 10.0;
        }
        TextLine::new(points)
    }

    #[test]
    fn test_remove_short_lines() {
        let lines = vec![
            create_test_line(10.0, 0.0, 100.0), // extent = 100
            create_test_line(30.0, 0.0, 50.0),  // extent = 50 (short, < 100)
            create_test_line(50.0, 0.0, 90.0),  // extent = 90 (short, < 100)
            create_test_line(70.0, 0.0, 200.0), // extent = 200 (longest)
        ];

        let filtered = remove_short_lines(lines, 0.5);

        // Should keep lines with extent >= 100 (50% of 200)
        // Lines with extents 100 and 200 are kept
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_remove_short_lines_empty() {
        let lines: Vec<TextLine> = vec![];
        let filtered = remove_short_lines(lines, 0.8);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_is_line_coverage_valid() {
        let lines = vec![
            create_test_line(50.0, 0.0, 100.0),
            create_test_line(100.0, 0.0, 100.0),
            create_test_line(150.0, 0.0, 100.0),
            create_test_line(200.0, 0.0, 100.0),
            create_test_line(250.0, 0.0, 100.0),
            create_test_line(300.0, 0.0, 100.0),
            create_test_line(350.0, 0.0, 100.0),
            create_test_line(400.0, 0.0, 100.0),
        ];

        // Image height 500: mid = 250
        // Lines at y < 250: 50, 100, 150, 200 (4 lines in top half)
        // Lines at y >= 250: 250, 300, 350, 400 (4 lines in bottom half)
        assert!(is_line_coverage_valid(&lines, 500, 6));
    }

    #[test]
    fn test_is_line_coverage_invalid_not_enough_lines() {
        let lines = vec![
            create_test_line(50.0, 0.0, 100.0),
            create_test_line(100.0, 0.0, 100.0),
        ];

        assert!(!is_line_coverage_valid(&lines, 500, 6));
    }

    #[test]
    fn test_sort_lines_by_y() {
        let mut lines = vec![
            create_test_line(100.0, 0.0, 100.0),
            create_test_line(50.0, 0.0, 100.0),
            create_test_line(200.0, 0.0, 100.0),
            create_test_line(75.0, 0.0, 100.0),
        ];

        sort_lines_by_y(&mut lines);

        assert_eq!(lines[0].mid_y(), Some(50.0));
        assert_eq!(lines[1].mid_y(), Some(75.0));
        assert_eq!(lines[2].mid_y(), Some(100.0));
        assert_eq!(lines[3].mid_y(), Some(200.0));
    }
}
