//! Signal processing for barcode width extraction
//!
//! This module provides functions for extracting and quantizing bar widths
//! from barcode images.

use crate::{RecogError, RecogResult};
use leptonica_core::{Pix, PixelDepth};

/// Extracts bar width crossings from a barcode image
///
/// # Arguments
/// * `pix` - 8 bpp grayscale barcode image
/// * `threshold` - Pixel threshold for crossing detection (typically ~120)
///
/// # Returns
/// * Vector of crossing locations in pixel units
pub fn extract_crossings(pix: &Pix, threshold: f32) -> RecogResult<Vec<f32>> {
    if pix.depth() != PixelDepth::Bit8 {
        return Err(RecogError::UnsupportedDepth {
            expected: "8 bpp",
            actual: pix.depth() as u32,
        });
    }

    // Average raster scans across the middle of the image
    let signal = average_raster_scans(pix, 50)?;

    if signal.is_empty() {
        return Err(RecogError::BarcodeError(
            "failed to extract signal".to_string(),
        ));
    }

    // Find the best threshold for crossing detection
    let best_threshold = select_crossing_threshold(&signal, threshold);

    // Detect crossings
    let crossings = find_crossings(&signal, best_threshold);

    if crossings.len() < 10 {
        return Err(RecogError::BarcodeError(format!(
            "only {} crossings found; need at least 10",
            crossings.len()
        )));
    }

    Ok(crossings)
}

/// Averages pixel values across horizontal raster scans
///
/// # Arguments
/// * `pix` - 8 bpp grayscale image
/// * `nscans` - Number of adjacent scans to average
fn average_raster_scans(pix: &Pix, nscans: u32) -> RecogResult<Vec<f32>> {
    let w = pix.width();
    let h = pix.height();

    let actual_scans = nscans.min(h);
    let first = (h - actual_scans) / 2;
    let last = first + actual_scans;

    let mut signal = vec![0.0f32; w as usize];

    for y in first..last {
        for x in 0..w {
            if let Some(pixel) = pix.get_pixel(x, y) {
                // For 8bpp, pixel value is in the low byte
                let value = (pixel & 0xFF) as f32;
                signal[x as usize] += value;
            }
        }
    }

    // Average
    for value in signal.iter_mut() {
        *value /= actual_scans as f32;
    }

    Ok(signal)
}

/// Selects the best threshold for crossing detection
///
/// Runs multiple times with different thresholds and chooses
/// a threshold in the center of the range that gives maximum crossings.
fn select_crossing_threshold(signal: &[f32], initial_threshold: f32) -> f32 {
    let mut best_threshold = initial_threshold;
    let mut max_crossings = 0;

    // Try thresholds in the range [initial - 40, initial + 40]
    for delta in -40..=40 {
        let thresh = initial_threshold + delta as f32;
        if !(20.0..=220.0).contains(&thresh) {
            continue;
        }
        let crossings = find_crossings(signal, thresh);
        if crossings.len() > max_crossings {
            max_crossings = crossings.len();
            best_threshold = thresh;
        }
    }

    best_threshold
}

/// Finds crossing points where the signal crosses the threshold
fn find_crossings(signal: &[f32], threshold: f32) -> Vec<f32> {
    let mut crossings = Vec::new();

    if signal.len() < 2 {
        return crossings;
    }

    let mut above = signal[0] > threshold;

    for i in 1..signal.len() {
        let current_above = signal[i] > threshold;
        if current_above != above {
            // Linear interpolation to find exact crossing point
            let x0 = (i - 1) as f32;
            let x1 = i as f32;
            let y0 = signal[i - 1];
            let y1 = signal[i];

            let crossing = if (y1 - y0).abs() > 0.001 {
                x0 + (threshold - y0) * (x1 - x0) / (y1 - y0)
            } else {
                (x0 + x1) / 2.0
            };

            crossings.push(crossing);
            above = current_above;
        }
    }

    crossings
}

/// Quantizes crossing distances to bar widths using histogram analysis
///
/// # Arguments
/// * `crossings` - Vector of crossing locations
/// * `bin_fract` - Histogram bin size as a fraction of minimum width (e.g., 0.25)
///
/// # Returns
/// * Vector of quantized widths (values 1-4)
pub fn quantize_crossings_by_width(crossings: &[f32], bin_fract: f32) -> RecogResult<Vec<u8>> {
    if crossings.len() < 10 {
        return Err(RecogError::BarcodeError("too few crossings".to_string()));
    }
    if bin_fract <= 0.0 {
        return Err(RecogError::InvalidParameter(
            "bin_fract must be positive".to_string(),
        ));
    }

    // Calculate distances between crossings
    let mut distances: Vec<f32> = Vec::with_capacity(crossings.len() - 1);
    for i in 1..crossings.len() {
        distances.push(crossings[i] - crossings[i - 1]);
    }

    // Find min and max distances (using 10th and 90th percentile)
    let mut sorted_distances = distances.clone();
    sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_idx = (sorted_distances.len() as f32 * 0.1) as usize;
    let max_idx = (sorted_distances.len() as f32 * 0.9) as usize;
    let min_size = sorted_distances[min_idx];
    let max_size = sorted_distances[max_idx.min(sorted_distances.len() - 1)];

    if min_size < 1.0 || max_size / min_size > 8.0 {
        return Err(RecogError::BarcodeError(format!(
            "bad data: min_size = {:.2}, max/min = {:.2}",
            min_size,
            max_size / min_size
        )));
    }

    // Quantize distances to width units
    let unit_width = min_size;
    let mut widths = Vec::with_capacity(distances.len());

    for dist in distances {
        let width = (dist / unit_width).round() as u8;
        // Clamp to valid range 1-4
        let clamped = width.clamp(1, 4);
        widths.push(clamped);
    }

    Ok(widths)
}

/// Quantizes crossing distances using window-based analysis
///
/// # Arguments
/// * `crossings` - Vector of crossing locations
/// * `ratio` - Ratio of max/min window size in search (typically 2.0)
///
/// # Returns
/// * Tuple of (quantized widths, best window width)
pub fn quantize_crossings_by_window(crossings: &[f32], ratio: f32) -> RecogResult<(Vec<u8>, f32)> {
    if crossings.len() < 10 {
        return Err(RecogError::BarcodeError("too few crossings".to_string()));
    }

    // Calculate distances between crossings
    let mut distances: Vec<f32> = Vec::with_capacity(crossings.len() - 1);
    for i in 1..crossings.len() {
        distances.push(crossings[i] - crossings[i - 1]);
    }

    // Find minimum distance (unit width)
    let mut sorted = distances.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let min_idx = (sorted.len() as f32 * 0.1) as usize;
    let min_size = sorted[min_idx];

    // Search for best window width
    let (best_width, _best_shift) = find_best_window(&distances, min_size, min_size * ratio)?;

    // Quantize using the best window
    let unit_width = best_width;
    let mut widths = Vec::with_capacity(distances.len());

    for dist in distances {
        let width = (dist / unit_width).round() as u8;
        let clamped = width.clamp(1, 4);
        widths.push(clamped);
    }

    Ok((widths, best_width))
}

/// Finds the best window width and shift for quantization
fn find_best_window(distances: &[f32], min_width: f32, max_width: f32) -> RecogResult<(f32, f32)> {
    const N_WIDTH: usize = 100;
    const N_SHIFT: usize = 10;

    let mut best_width = min_width;
    let mut best_shift = 0.0f32;
    let mut best_score = f32::MAX;

    let del_width = (max_width - min_width) / (N_WIDTH - 1) as f32;

    for i in 0..N_WIDTH {
        let width = min_width + del_width * i as f32;
        let del_shift = width / N_SHIFT as f32;

        for j in 0..N_SHIFT {
            let shift = -0.5 * (width - del_shift) + j as f32 * del_shift;
            let score = eval_sync_error(distances, width, shift);

            if score < best_score {
                best_score = score;
                best_width = width;
                best_shift = shift;
            }
        }
    }

    Ok((best_width, best_shift))
}

/// Evaluates synchronization error for a given window width and shift
fn eval_sync_error(distances: &[f32], width: f32, shift: f32) -> f32 {
    let mut score = 0.0f32;
    let mut position = shift;

    for &dist in distances {
        position += dist;
        // Calculate distance from nearest window center
        let window_idx = (position / width).round();
        let window_center = window_idx * width;
        let error = position - window_center;
        score += error * error;
    }

    // Normalize by number of distances and window width
    score / (distances.len() as f32 * width * width / 4.0)
}

/// Converts quantized widths to a bar string
///
/// # Arguments
/// * `widths` - Vector of widths (1-4)
///
/// # Returns
/// * String of digits representing bar widths
pub fn widths_to_bar_string(widths: &[u8]) -> String {
    widths.iter().map(|&w| char::from(b'0' + w)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_crossings() {
        // Simple test signal: low, high, low, high
        let signal = vec![50.0, 150.0, 50.0, 150.0, 50.0];
        let crossings = find_crossings(&signal, 100.0);
        assert_eq!(crossings.len(), 4); // 4 crossings
    }

    #[test]
    fn test_widths_to_bar_string() {
        let widths = vec![1, 2, 3, 4, 1, 2];
        let bar_string = widths_to_bar_string(&widths);
        assert_eq!(bar_string, "123412");
    }

    #[test]
    fn test_quantize_too_few_crossings() {
        let crossings = vec![0.0, 10.0, 20.0];
        let result = quantize_crossings_by_width(&crossings, 0.25);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantize_invalid_bin_fract() {
        let crossings: Vec<f32> = (0..20).map(|i| i as f32 * 10.0).collect();
        let result = quantize_crossings_by_width(&crossings, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sync_error() {
        // Perfect alignment should give low error
        let distances = vec![10.0, 10.0, 10.0, 10.0];
        let error = eval_sync_error(&distances, 10.0, 0.0);
        assert!(error < 0.1);
    }
}
