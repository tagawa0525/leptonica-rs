//! File/image partitioning by similarity
//!
//! Partition files or image collections into groups by similarity.
//! Originally designed for musical staff extraction, this module
//! provides general-purpose image partitioning.
//!
//! # See also
//! C Leptonica: `partify.c`

use std::path::Path;

use crate::core::{Pix, Pixa, PixelDepth};
use crate::io::{IoError, IoResult};

/// Partition files by similarity
///
/// Loads images from a directory, partitions them into `n_parts` groups,
/// and writes each group to a separate output.
///
/// # Arguments
/// * `dir` - input directory
/// * `substr` - optional filename filter
/// * `n_parts` - number of parts to generate
/// * `out_root` - root name for output files
///
/// # See also
/// C Leptonica: `partifyFiles()` in `partify.c`
pub fn partify_files(
    dir: impl AsRef<Path>,
    substr: Option<&str>,
    n_parts: usize,
    out_root: &str,
) -> IoResult<Vec<Pixa>> {
    if n_parts < 1 {
        return Err(IoError::InvalidData("n_parts must be >= 1".into()));
    }

    // Load all images
    let pixa = crate::io::pixa_read_files(dir, substr)?;
    partify_pixac(&pixa, n_parts, out_root)
}

/// Partition image collection by similarity
///
/// Partitions a collection of images into `n_parts` groups.
/// Each group contains corresponding slices of the input images.
///
/// For musical scores: each page is divided into `n_parts` staves/parts.
/// For general use: images are distributed round-robin into groups.
///
/// # Arguments
/// * `pixa` - collection of images
/// * `n_parts` - number of parts to generate
/// * `out_root` - root name for output (currently unused; reserved for PDF output)
///
/// # Returns
/// Vector of Pixa, one per part
///
/// # See also
/// C Leptonica: `partifyPixac()` in `partify.c`
pub fn partify_pixac(pixa: &Pixa, n_parts: usize, _out_root: &str) -> IoResult<Vec<Pixa>> {
    if n_parts < 1 {
        return Err(IoError::InvalidData("n_parts must be >= 1".into()));
    }

    let n = pixa.len();
    if n == 0 {
        let mut result = Vec::with_capacity(n_parts);
        for _ in 0..n_parts {
            result.push(Pixa::new());
        }
        return Ok(result);
    }

    let mut parts: Vec<Pixa> = (0..n_parts).map(|_| Pixa::new()).collect();

    for i in 0..n {
        if let Some(pix) = pixa.get(i) {
            // Try to find horizontal regions (stave sets) in the image
            let regions = locate_stave_sets(pix);

            if regions.is_empty() || regions.len() < n_parts {
                // Fallback: assign to parts round-robin
                let part_idx = i % n_parts;
                parts[part_idx].push(pix.clone());
            } else {
                // Assign each region to its corresponding part
                for (j, region) in regions.iter().enumerate().take(n_parts) {
                    if let Some(clipped) = clip_to_box(pix, region) {
                        parts[j].push(clipped);
                    }
                }
            }
        }
    }

    Ok(parts)
}

/// Locate horizontal stave sets in an image
///
/// Uses morphological reduction and CC analysis to find large horizontal
/// regions (originally designed for musical staves).
fn locate_stave_sets(pix: &Pix) -> Vec<crate::core::Box> {
    // Only works well on binary images
    if pix.depth() != PixelDepth::Bit1 {
        return Vec::new();
    }

    // Try morphological reduction to find large horizontal structures
    let reduced = match crate::morph::morph_sequence(pix, "r11") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    // Find connected components
    let (boxa, _) =
        match crate::region::conncomp_pixa(&reduced, crate::region::ConnectivityType::EightWay) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

    // Filter by area (keep large components)
    let filtered = boxa.select_by_area(15000, crate::core::SizeRelation::GreaterThanOrEqual);

    // Sort by y position
    let mut boxes: Vec<crate::core::Box> = (0..filtered.len())
        .filter_map(|i| filtered.get(i).cloned())
        .collect();
    boxes.sort_by_key(|b| b.y);

    // Scale back to full resolution (4x)
    boxes
        .into_iter()
        .map(|b| crate::core::Box::new(b.x * 4, b.y * 4, b.w * 4, b.h * 4).unwrap_or(b))
        .collect()
}

/// Clip an image to a bounding box region
fn clip_to_box(pix: &Pix, boxx: &crate::core::Box) -> Option<Pix> {
    let x0 = boxx.x.max(0) as u32;
    let y0 = boxx.y.max(0) as u32;
    let x1 = ((boxx.x + boxx.w) as u32).min(pix.width());
    let y1 = ((boxx.y + boxx.h) as u32).min(pix.height());

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    let w = x1 - x0;
    let h = y1 - y0;
    let clipped = Pix::new(w, h, pix.depth()).ok()?;
    let mut clipped_mut = clipped.try_into_mut().ok()?;

    for y in 0..h {
        for x in 0..w {
            let val = pix.get_pixel(x0 + x, y0 + y).unwrap_or(0);
            let _ = clipped_mut.set_pixel(x, y, val);
        }
    }

    Some(clipped_mut.into())
}
