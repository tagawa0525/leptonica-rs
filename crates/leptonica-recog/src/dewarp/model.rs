//! Disparity model building
//!
//! This module provides functions to build vertical and horizontal
//! disparity models from text line centers.

use crate::{RecogError, RecogResult};
use leptonica_core::FPix;

use super::textline::{is_line_coverage_valid, remove_short_lines, sort_lines_by_y};
use super::types::{Dewarp, DewarpOptions, TextLine};

/// Build the vertical disparity model
///
/// The vertical disparity represents how much each pixel needs to be
/// shifted vertically to straighten the text lines.
///
/// # Arguments
///
/// * `dewarp` - Dewarp structure to populate
/// * `lines` - Text lines detected in the image
/// * `options` - Dewarping options
///
/// # Returns
///
/// `Ok(())` if model was built successfully, error otherwise.
pub fn build_vertical_disparity(
    dewarp: &mut Dewarp,
    lines: &[TextLine],
    options: &DewarpOptions,
) -> RecogResult<()> {
    // Check for enough lines with valid coverage
    if !is_line_coverage_valid(lines, dewarp.height, options.min_lines) {
        return Err(RecogError::NoContent(format!(
            "insufficient line coverage: found {} lines, need {} with proper distribution",
            lines.len(),
            options.min_lines
        )));
    }

    // Remove short lines (< 80% of longest)
    let mut lines = remove_short_lines(lines.to_vec(), 0.8);
    if lines.len() < options.min_lines as usize {
        return Err(RecogError::NoContent(format!(
            "not enough long lines: {} after filtering, need {}",
            lines.len(),
            options.min_lines
        )));
    }

    // Sort lines by y position
    sort_lines_by_y(&mut lines);
    dewarp.n_lines = lines.len() as u32;

    let w = dewarp.width;
    let h = dewarp.height;
    let sampling = dewarp.sampling;
    let nx = dewarp.nx;
    let ny = dewarp.ny;

    // Build sampled vertical disparity array
    let mut v_disparity = FPix::new(nx, ny)?;

    // For each line, fit a quadratic and compute disparities at sampled x positions
    let mut mid_ys = Vec::with_capacity(lines.len());
    let mut curvatures = Vec::with_capacity(lines.len());

    for line in &lines {
        // Fit quadratic y = a*x^2 + b*x + c to the line
        let fit = fit_quadratic(&line.points);
        if fit.is_none() {
            continue;
        }
        let (a, b, c) = fit.unwrap();

        // Compute mid y (at center x)
        let mid_x = w as f32 / 2.0;
        let mid_y = a * mid_x * mid_x + b * mid_x + c;
        mid_ys.push(mid_y);

        // Curvature in "micro-units" (1000 * height deviation per pixel)
        // Approximated as deviation from linear fit at edges
        let left_y = c;
        let right_y = a * (w as f32) * (w as f32) + b * (w as f32) + c;
        let center_y = a * mid_x * mid_x + b * mid_x + c;
        let linear_mid = (left_y + right_y) / 2.0;
        let curvature = ((center_y - linear_mid) * 1000.0 / (w as f32 / 2.0)) as i32;
        curvatures.push(curvature);
    }

    if mid_ys.is_empty() {
        return Err(RecogError::NoContent(
            "failed to fit quadratics to any line".to_string(),
        ));
    }

    // Record min/max curvature
    dewarp.min_curvature = curvatures.iter().cloned().min().unwrap_or(0);
    dewarp.max_curvature = curvatures.iter().cloned().max().unwrap_or(0);

    // Build the disparity array
    // For each sampled (x, y) position, interpolate the expected y-shift
    for iy in 0..ny {
        let y = (iy * sampling) as f32;

        for ix in 0..nx {
            let x = (ix * sampling) as f32;

            // Find the disparity at this point
            let disparity = compute_disparity_at_point(x, y, &lines, w as f32, h as f32);
            v_disparity.set_pixel(ix, iy, disparity)?;
        }
    }

    dewarp.sampled_v_disparity = Some(v_disparity);
    dewarp.v_success = true;

    // Validate the model
    let max_curv = dewarp.max_curvature.abs().max(dewarp.min_curvature.abs());
    let diff_curv = dewarp.max_curvature - dewarp.min_curvature;

    if max_curv <= options.max_line_curvature && diff_curv <= 2 * options.max_line_curvature {
        dewarp.v_valid = true;
    }

    Ok(())
}

/// Fit a quadratic curve y = a*x^2 + b*x + c to the given points
///
/// Uses least-squares fitting.
///
/// Returns None if fitting fails (e.g., too few points).
fn fit_quadratic(points: &[(f32, f32)]) -> Option<(f32, f32, f32)> {
    if points.len() < 3 {
        return None;
    }

    let n = points.len() as f64;
    let mut sx = 0.0f64;
    let mut sx2 = 0.0f64;
    let mut sx3 = 0.0f64;
    let mut sx4 = 0.0f64;
    let mut sy = 0.0f64;
    let mut sxy = 0.0f64;
    let mut sx2y = 0.0f64;

    for &(x, y) in points {
        let x = x as f64;
        let y = y as f64;
        let x2 = x * x;

        sx += x;
        sx2 += x2;
        sx3 += x2 * x;
        sx4 += x2 * x2;
        sy += y;
        sxy += x * y;
        sx2y += x2 * y;
    }

    // Solve the normal equations:
    // | sx4  sx3  sx2 | | a |   | sx2y |
    // | sx3  sx2  sx  | | b | = | sxy  |
    // | sx2  sx   n   | | c |   | sy   |

    let det = sx4 * (sx2 * n - sx * sx) - sx3 * (sx3 * n - sx * sx2) + sx2 * (sx3 * sx - sx2 * sx2);

    if det.abs() < 1e-10 {
        return None;
    }

    let a = (sx2y * (sx2 * n - sx * sx) - sxy * (sx3 * n - sx * sx2) + sy * (sx3 * sx - sx2 * sx2))
        / det;

    let b = (sx4 * (sxy * n - sy * sx) - sx3 * (sx2y * n - sy * sx2)
        + sx2 * (sx2y * sx - sxy * sx2))
        / det;

    let c = (sx4 * (sx2 * sy - sx * sxy) - sx3 * (sx3 * sy - sx * sx2y)
        + sx2 * (sx3 * sxy - sx2 * sx2y))
        / det;

    Some((a as f32, b as f32, c as f32))
}

/// Compute disparity at a point (x, y) based on nearby text lines
///
/// Uses vertical interpolation between the nearest lines above and below.
fn compute_disparity_at_point(
    x: f32,
    y: f32,
    lines: &[TextLine],
    _width: f32,
    _height: f32,
) -> f32 {
    // Find the line above and below this y position
    let mut above_line: Option<(f32, f32)> = None; // (mid_y, y_at_x)
    let mut below_line: Option<(f32, f32)> = None;

    for line in lines {
        let Some(mid_y) = line.mid_y() else { continue };

        // Find y at this x position by interpolation
        let y_at_x = interpolate_y_at_x(line, x);
        if y_at_x.is_none() {
            continue;
        }
        let y_at_x = y_at_x.unwrap();

        if mid_y <= y {
            // This line is above (or at) y
            if above_line.is_none() || mid_y > above_line.unwrap().0 {
                above_line = Some((mid_y, y_at_x));
            }
        }

        if mid_y >= y {
            // This line is below (or at) y
            if below_line.is_none() || mid_y < below_line.unwrap().0 {
                below_line = Some((mid_y, y_at_x));
            }
        }
    }

    // Compute disparity by interpolation
    match (above_line, below_line) {
        (Some((_, y_above)), Some((_, y_below))) if above_line == below_line => {
            // On the line itself
            y_above - above_line.unwrap().0
        }
        (Some((mid_above, y_above)), Some((mid_below, y_below))) => {
            // Between two lines - linear interpolation
            let t = (y - mid_above) / (mid_below - mid_above);
            let expected_y = mid_above + t * (mid_below - mid_above);
            let actual_y_at_x = y_above + t * (y_below - y_above);
            actual_y_at_x - expected_y
        }
        (Some((mid_above, y_above)), None) => {
            // Only line above - extrapolate
            y_above - mid_above
        }
        (None, Some((mid_below, y_below))) => {
            // Only line below - extrapolate
            y_below - mid_below
        }
        (None, None) => 0.0,
    }
}

/// Interpolate y at a given x position in a text line
fn interpolate_y_at_x(line: &TextLine, x: f32) -> Option<f32> {
    if line.points.is_empty() {
        return None;
    }

    // Find surrounding points
    let mut left: Option<(f32, f32)> = None;
    let mut right: Option<(f32, f32)> = None;

    for &(px, py) in &line.points {
        if px <= x && (left.is_none() || px > left.unwrap().0) {
            left = Some((px, py));
        }
        if px >= x && (right.is_none() || px < right.unwrap().0) {
            right = Some((px, py));
        }
    }

    match (left, right) {
        (Some((lx, ly)), Some((rx, ry))) => {
            if (rx - lx).abs() < 0.001 {
                Some(ly)
            } else {
                let t = (x - lx) / (rx - lx);
                Some(ly + t * (ry - ly))
            }
        }
        (Some((_, ly)), None) => Some(ly),
        (None, Some((_, ry))) => Some(ry),
        (None, None) => None,
    }
}

/// Build the horizontal disparity model
///
/// The horizontal disparity represents perspective distortion from page curl.
///
/// # Arguments
///
/// * `dewarp` - Dewarp structure to populate
/// * `lines` - Text lines detected in the image
/// * `options` - Dewarping options
pub fn build_horizontal_disparity(
    dewarp: &mut Dewarp,
    lines: &[TextLine],
    options: &DewarpOptions,
) -> RecogResult<()> {
    if lines.len() < options.min_lines as usize {
        return Err(RecogError::NoContent(
            "not enough lines for horizontal disparity".to_string(),
        ));
    }

    let _w = dewarp.width;
    let h = dewarp.height;
    let sampling = dewarp.sampling;
    let nx = dewarp.nx;
    let ny = dewarp.ny;

    // Get left and right endpoints of each line
    let mut left_points: Vec<(f32, f32)> = Vec::new();
    let mut right_points: Vec<(f32, f32)> = Vec::new();

    for line in lines {
        if line.points.len() < 2 {
            continue;
        }

        // Find leftmost and rightmost points
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_point = (0.0f32, 0.0f32);
        let mut max_point = (0.0f32, 0.0f32);

        for &(x, y) in &line.points {
            if x < min_x {
                min_x = x;
                min_point = (x, y);
            }
            if x > max_x {
                max_x = x;
                max_point = (x, y);
            }
        }

        left_points.push(min_point);
        right_points.push(max_point);
    }

    if left_points.len() < 4 || right_points.len() < 4 {
        return Err(RecogError::NoContent(
            "not enough edge points for horizontal disparity".to_string(),
        ));
    }

    // Fit linear models to left and right edges
    let left_fit = fit_linear_by_y(&left_points);
    let right_fit = fit_linear_by_y(&right_points);

    if left_fit.is_none() || right_fit.is_none() {
        return Err(RecogError::NoContent(
            "failed to fit edge models".to_string(),
        ));
    }

    let (left_a, left_b) = left_fit.unwrap(); // x = a * y + b
    let (right_a, right_b) = right_fit.unwrap();

    // Store edge slopes (in milli-units)
    dewarp.left_slope = (left_a * 1000.0) as i32;
    dewarp.right_slope = (right_a * 1000.0) as i32;

    // Build the horizontal disparity array
    let mut h_disparity = FPix::new(nx, ny)?;

    for iy in 0..ny {
        let y = (iy * sampling) as f32;

        // Expected edge positions at this y
        let expected_left = left_a * y + left_b;
        let expected_right = right_a * y + right_b;

        // Reference positions (at mid-height)
        let mid_y = h as f32 / 2.0;
        let ref_left = left_a * mid_y + left_b;
        let ref_right = right_a * mid_y + right_b;

        for ix in 0..nx {
            let x = (ix * sampling) as f32;

            // Horizontal disparity: how much to shift x to align with reference
            // Linear interpolation between left and right edge disparities
            let left_disp = expected_left - ref_left;
            let right_disp = expected_right - ref_right;

            let t = if expected_right > expected_left {
                (x - expected_left) / (expected_right - expected_left)
            } else {
                0.5
            };

            let disparity = left_disp + t * (right_disp - left_disp);
            h_disparity.set_pixel(ix, iy, disparity)?;
        }
    }

    dewarp.sampled_h_disparity = Some(h_disparity);
    dewarp.h_success = true;

    // Validate the model
    if dewarp.left_slope.abs() <= options.max_edge_slope
        && dewarp.right_slope.abs() <= options.max_edge_slope
    {
        dewarp.h_valid = true;
    }

    Ok(())
}

/// Fit a linear model x = a * y + b to points
fn fit_linear_by_y(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }

    let n = points.len() as f64;
    let mut sy = 0.0f64;
    let mut sy2 = 0.0f64;
    let mut sx = 0.0f64;
    let mut sxy = 0.0f64;

    for &(x, y) in points {
        let x = x as f64;
        let y = y as f64;

        sy += y;
        sy2 += y * y;
        sx += x;
        sxy += x * y;
    }

    let det = sy2 * n - sy * sy;
    if det.abs() < 1e-10 {
        return None;
    }

    let a = (sxy * n - sx * sy) / det;
    let b = (sx * sy2 - sxy * sy) / det;

    Some((a as f32, b as f32))
}

/// Populate full resolution disparity arrays from sampled arrays
///
/// This scales the sampled arrays to full resolution using bilinear interpolation.
pub fn populate_full_resolution(dewarp: &mut Dewarp) -> RecogResult<()> {
    let w = dewarp.width;
    let h = dewarp.height;
    let sampling = dewarp.sampling;
    let reduction_factor = dewarp.reduction_factor;

    // Generate full resolution vertical disparity
    if let Some(ref sampled_v) = dewarp.sampled_v_disparity {
        let full_v = scale_fpix_by_sampling(sampled_v, sampling * reduction_factor, w, h)?;
        dewarp.full_v_disparity = Some(full_v);
    }

    // Generate full resolution horizontal disparity
    if let Some(ref sampled_h) = dewarp.sampled_h_disparity {
        let full_h = scale_fpix_by_sampling(sampled_h, sampling * reduction_factor, w, h)?;
        dewarp.full_h_disparity = Some(full_h);
    }

    Ok(())
}

/// Scale a sampled FPix to full resolution using bilinear interpolation
fn scale_fpix_by_sampling(
    sampled: &FPix,
    sampling: u32,
    target_width: u32,
    target_height: u32,
) -> RecogResult<FPix> {
    let sw = sampled.width();
    let sh = sampled.height();

    let mut result = FPix::new(target_width, target_height)?;

    for y in 0..target_height {
        for x in 0..target_width {
            // Map to sampled coordinates
            let sx = x as f32 / sampling as f32;
            let sy = y as f32 / sampling as f32;

            // Bilinear interpolation
            let x0 = sx.floor() as u32;
            let y0 = sy.floor() as u32;
            let x1 = (x0 + 1).min(sw - 1);
            let y1 = (y0 + 1).min(sh - 1);

            let fx = sx - x0 as f32;
            let fy = sy - y0 as f32;

            let v00 = sampled
                .get_pixel(x0.min(sw - 1), y0.min(sh - 1))
                .unwrap_or(0.0);
            let v10 = sampled.get_pixel(x1, y0.min(sh - 1)).unwrap_or(0.0);
            let v01 = sampled.get_pixel(x0.min(sw - 1), y1).unwrap_or(0.0);
            let v11 = sampled.get_pixel(x1, y1).unwrap_or(0.0);

            let value = v00 * (1.0 - fx) * (1.0 - fy)
                + v10 * fx * (1.0 - fy)
                + v01 * (1.0 - fx) * fy
                + v11 * fx * fy;

            result.set_pixel(x, y, value)?;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_straight_line(y: f32, x_start: f32, x_end: f32, step: f32) -> TextLine {
        let mut points = Vec::new();
        let mut x = x_start;
        while x <= x_end {
            points.push((x, y));
            x += step;
        }
        TextLine::new(points)
    }

    #[allow(dead_code)]
    fn create_curved_line(
        y_base: f32,
        curvature: f32,
        x_start: f32,
        x_end: f32,
        width: f32,
    ) -> TextLine {
        let mut points = Vec::new();
        let center_x = (x_start + x_end) / 2.0;
        let mut x = x_start;
        while x <= x_end {
            // Quadratic curvature: y = y_base + curvature * ((x - center) / (width/2))^2
            let normalized_x = (x - center_x) / (width / 2.0);
            let y = y_base + curvature * normalized_x * normalized_x;
            points.push((x, y));
            x += 10.0;
        }
        TextLine::new(points)
    }

    #[test]
    fn test_fit_quadratic_straight_line() {
        let points: Vec<(f32, f32)> = (0..10).map(|i| (i as f32 * 10.0, 50.0)).collect();

        let fit = fit_quadratic(&points);
        assert!(fit.is_some());
        let (a, b, c) = fit.unwrap();
        assert!(a.abs() < 0.01); // Nearly zero quadratic coefficient
        assert!(b.abs() < 0.01); // Nearly zero linear coefficient
        assert!((c - 50.0).abs() < 1.0); // Constant term near 50
    }

    #[test]
    fn test_fit_quadratic_curved_line() {
        // y = 0.001 * x^2 + 50 (bowl-shaped curve)
        let points: Vec<(f32, f32)> = (0..10)
            .map(|i| {
                let x = i as f32 * 10.0;
                let y = 0.001 * x * x + 50.0;
                (x, y)
            })
            .collect();

        let fit = fit_quadratic(&points);
        assert!(fit.is_some());
        let (a, _b, c) = fit.unwrap();
        assert!((a - 0.001).abs() < 0.001);
        assert!((c - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_fit_quadratic_insufficient_points() {
        let points: Vec<(f32, f32)> = vec![(0.0, 50.0), (10.0, 51.0)];
        assert!(fit_quadratic(&points).is_none());
    }

    #[test]
    fn test_fit_linear_by_y() {
        // Points along a vertical line
        let points: Vec<(f32, f32)> = (0..10).map(|i| (50.0, i as f32 * 10.0)).collect();

        let fit = fit_linear_by_y(&points);
        assert!(fit.is_some());
        let (a, b) = fit.unwrap();
        assert!(a.abs() < 0.01); // Nearly zero slope
        assert!((b - 50.0).abs() < 1.0); // Intercept near 50
    }

    #[test]
    fn test_interpolate_y_at_x() {
        let points = vec![(0.0, 10.0), (50.0, 20.0), (100.0, 15.0)];
        let line = TextLine::new(points);

        let y = interpolate_y_at_x(&line, 25.0);
        assert!(y.is_some());
        assert!((y.unwrap() - 15.0).abs() < 0.01); // Linear interpolation between (0,10) and (50,20)
    }

    #[test]
    fn test_scale_fpix_by_sampling() {
        let mut sampled = FPix::new(3, 3).unwrap();
        sampled.set_pixel(0, 0, 0.0).unwrap();
        sampled.set_pixel(1, 0, 1.0).unwrap();
        sampled.set_pixel(2, 0, 2.0).unwrap();
        sampled.set_pixel(0, 1, 1.0).unwrap();
        sampled.set_pixel(1, 1, 2.0).unwrap();
        sampled.set_pixel(2, 1, 3.0).unwrap();
        sampled.set_pixel(0, 2, 2.0).unwrap();
        sampled.set_pixel(1, 2, 3.0).unwrap();
        sampled.set_pixel(2, 2, 4.0).unwrap();

        let scaled = scale_fpix_by_sampling(&sampled, 10, 25, 25).unwrap();
        assert_eq!(scaled.width(), 25);
        assert_eq!(scaled.height(), 25);

        // Check corner values
        assert!((scaled.get_pixel(0, 0).unwrap() - 0.0).abs() < 0.01);
        assert!((scaled.get_pixel(20, 20).unwrap() - 4.0).abs() < 0.5); // Near bottom-right
    }

    #[test]
    fn test_build_vertical_disparity() {
        let options = DewarpOptions::default().with_min_lines(4);
        let mut dewarp = Dewarp::new(800, 600, 0, &options);

        // Create enough straight lines
        let lines: Vec<TextLine> = (0..8)
            .map(|i| create_straight_line(50.0 + i as f32 * 70.0, 50.0, 750.0, 10.0))
            .collect();

        let result = build_vertical_disparity(&mut dewarp, &lines, &options);
        assert!(result.is_ok(), "Failed: {:?}", result);
        assert!(dewarp.v_success);
        assert!(dewarp.sampled_v_disparity.is_some());
    }
}
