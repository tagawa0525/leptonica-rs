# Color Segmentation Implementation Plan

## Overview

Implement unsupervised color segmentation for 32-bpp RGB images in the
`leptonica-color` crate. This corresponds to `colorseg.c` in the
original Leptonica C library.

## Reference

- C source: `reference/leptonica/src/colorseg.c`
- Existing pattern: `crates/leptonica-color/src/quantize.rs`

## Scope

### Functions to Implement

| C Function | Rust Function | Description |
| --- | --- | --- |
| `pixColorSegment` | `color_segment` | Full 4-phase segmentation |
| `pixColorSegmentCluster` | `color_segment_cluster` | Phase 1 clustering |
| `pixAssignToNearestColor` | `assign_to_nearest_color` | Phase 2 reassign |
| `pixColorSegmentClean` | (internal) | Phase 3 cleanup |
| `pixColorSegmentRemoveColors` | (internal) | Phase 4 reduce |

### Design Decision: Simplified API

The C implementation has 4 phases with complex parameters. For the Rust
implementation, provide:

1. **Full API** (`color_segment`): Complete 4-phase algorithm
2. **Simple API** (`color_segment_simple`): Reasonable defaults

## Design

### File Structure

```text
crates/leptonica-color/src/
  lib.rs          # Add pub mod segment; and re-exports
  segment.rs      # NEW: Color segmentation implementation
```

### API Design

```rust
// crates/leptonica-color/src/segment.rs

/// Options for color segmentation
#[derive(Debug, Clone)]
pub struct ColorSegmentOptions {
    /// Maximum Euclidean distance to existing cluster (Phase 1)
    /// Typical: 60-100 for 3-6 final colors
    pub max_dist: u32,
    /// Maximum colors allowed in first pass
    /// Should be larger than final_colors (typically 2x)
    pub max_colors: u32,
    /// Linear size of structuring element for morphological cleanup
    /// Set to 0 or 1 to skip cleanup
    pub sel_size: u32,
    /// Maximum number of final colors after Phase 4
    pub final_colors: u32,
}

impl Default for ColorSegmentOptions {
    fn default() -> Self {
        Self {
            max_dist: 75,
            max_colors: 10,
            sel_size: 4,
            final_colors: 5,
        }
    }
}

/// Perform unsupervised color segmentation
///
/// Returns an 8-bpp colormapped image.
///
/// # Algorithm (4 phases)
///
/// 1. **Cluster**: Greedy assignment to color clusters
/// 2. **Refine**: Reassign pixels to nearest cluster average
/// 3. **Clean**: Morphological closing to remove noise (optional)
/// 4. **Reduce**: Remove unpopular colors
pub fn color_segment(pix: &Pix, options: &ColorSegmentOptions)
    -> ColorResult<Pix>;

/// Simple color segmentation with default options
pub fn color_segment_simple(pix: &Pix, final_colors: u32)
    -> ColorResult<Pix>;

/// Greedy color clustering (Phase 1 only)
///
/// Useful when you need more control over the process.
pub fn color_segment_cluster(
    pix: &Pix,
    max_dist: u32,
    max_colors: u32,
) -> ColorResult<Pix>;

/// Assign pixels to nearest color in colormap
///
/// This is Phase 2 of color segmentation.
/// Can be used independently for remapping colors.
pub fn assign_to_nearest_color(
    dest: &mut PixMut,
    src: &Pix,
    mask: Option<&Pix>,
) -> ColorResult<Vec<u32>>;
```

### Parameters Guidelines

Based on C source comments:

| final_colors | max_colors | max_dist |
| ------------ | ---------- | -------- |
| 3            | 6          | 100      |
| 4            | 8          | 90       |
| 5            | 10         | 75       |
| 6            | 12         | 60       |

## Implementation Details

### Phase 1: Greedy Clustering

```rust
fn color_segment_cluster_impl(
    pix_src: &Pix,
    max_dist: u32,
    max_colors: u32,
) -> ColorResult<Pix> {
    // Create 8-bpp output with colormap
    let (w, h) = (pix_src.width(), pix_src.height());
    let pix_out = Pix::new(w, h, PixelDepth::Bit8)?;
    let mut out_mut = pix_out.try_into_mut().unwrap();
    let mut colormap = PixColormap::new(8)?;

    // Track cluster colors and sums for averaging
    let mut cluster_colors: Vec<(u8, u8, u8)> = Vec::new();
    let mut cluster_sums: Vec<(u64, u64, u64, u64)> = Vec::new(); // (r, g, b, count)

    let max_dist_sq = (max_dist * max_dist) as i32;

    for y in 0..h {
        for x in 0..w {
            let pixel = pix_src.get_pixel(x, y);
            let (r, g, b) = extract_rgb(pixel);

            // Find nearest cluster within max_dist
            let mut found = false;
            for (i, &(cr, cg, cb)) in cluster_colors.iter().enumerate() {
                let dist_sq = color_distance_sq(r, g, b, cr, cg, cb);
                if dist_sq <= max_dist_sq {
                    // Assign to this cluster
                    out_mut.set_pixel(x, y, i as u32);
                    cluster_sums[i].0 += r as u64;
                    cluster_sums[i].1 += g as u64;
                    cluster_sums[i].2 += b as u64;
                    cluster_sums[i].3 += 1;
                    found = true;
                    break;
                }
            }

            if !found {
                if cluster_colors.len() >= max_colors as usize {
                    // Too many colors - need to increase max_dist
                    return Err(ColorError::QuantizationError(
                        "exceeded max_colors".into()
                    ));
                }
                // Create new cluster
                let idx = cluster_colors.len();
                cluster_colors.push((r, g, b));
                cluster_sums.push((r as u64, g as u64, b as u64, 1));
                out_mut.set_pixel(x, y, idx as u32);
            }
        }
    }

    // Update colormap with average colors
    for (i, (sr, sg, sb, count)) in cluster_sums.iter().enumerate() {
        let avg_r = (*sr / count) as u8;
        let avg_g = (*sg / count) as u8;
        let avg_b = (*sb / count) as u8;
        colormap.add_rgb(avg_r, avg_g, avg_b)?;
        cluster_colors[i] = (avg_r, avg_g, avg_b);
    }

    out_mut.set_colormap(Some(colormap))?;
    Ok(out_mut.into())
}
```

### Iterative Distance Expansion

Like the C version, if initial max_dist produces too many colors,
automatically expand:

```rust
const MAX_ITERATIONS: u32 = 20;
const DIST_EXPAND_FACTOR: f32 = 1.3;

fn cluster_with_expansion(...) -> ColorResult<Pix> {
    let mut current_dist = max_dist;
    for _ in 0..MAX_ITERATIONS {
        match color_segment_cluster_impl(pix, current_dist, max_colors) {
            Ok(result) => return Ok(result),
            Err(ColorError::QuantizationError(_)) => {
                current_dist = (current_dist as f32 * DIST_EXPAND_FACTOR) as u32;
            }
            Err(e) => return Err(e),
        }
    }
    Err(ColorError::QuantizationError("failed after max iterations".into()))
}
```

### Phase 2: Nearest Color Assignment

Use octcube-based LUT for efficiency (like the C version), or simpler
brute-force for initial implementation:

```rust
fn assign_to_nearest_color(
    dest: &mut PixMut,
    src: &Pix,
    mask: Option<&Pix>,
) -> ColorResult<Vec<u32>> {
    let colormap = dest.colormap()
        .ok_or(ColorError::InvalidParameters("no colormap"))?;

    let mut counts = vec![0u32; colormap.len()];

    for y in 0..src.height() {
        for x in 0..src.width() {
            // Skip if masked
            if let Some(m) = mask {
                if m.get_pixel(x, y) == 0 {
                    continue;
                }
            }

            let pixel = src.get_pixel(x, y);
            let (r, g, b) = extract_rgb(pixel);

            // Find nearest color in colormap
            let idx = colormap.find_nearest(r, g, b).unwrap_or(0);
            dest.set_pixel(x, y, idx as u32);
            counts[idx] += 1;
        }
    }

    Ok(counts)
}
```

### Phase 3: Morphological Cleanup (Simplified)

Note: Full implementation requires morphological operations from
leptonica-morph. For initial version, can skip or use simplified
approach.

```rust
fn color_segment_clean(
    pix: &mut PixMut,
    sel_size: u32,
    counts: &[u32],
) -> ColorResult<()> {
    if sel_size <= 1 {
        return Ok(()); // No cleanup needed
    }

    // Sort colors by count (descending)
    let mut indices: Vec<usize> = (0..counts.len()).collect();
    indices.sort_by(|a, b| counts[*b].cmp(&counts[*a]));

    // For each color (largest first), do closing
    // This absorbs small regions of other colors
    for &idx in &indices {
        // Generate mask for this color
        // Apply closing
        // Set absorbed pixels
    }

    Ok(())
}
```

### Phase 4: Remove Unpopular Colors

```rust
fn remove_unpopular_colors(
    pix_dest: &mut PixMut,
    pix_src: &Pix,
    final_colors: u32,
) -> ColorResult<()> {
    let colormap = pix_dest.colormap_mut().unwrap();
    let counts = /* get histogram */;

    if counts.len() <= final_colors as usize {
        return Ok(()); // Already few enough colors
    }

    // Find top N colors by count
    let mut indices: Vec<usize> = (0..counts.len()).collect();
    indices.sort_by(|a, b| counts[*b].cmp(&counts[*a]));
    let keep_indices: HashSet<usize> = indices[..final_colors as usize]
        .iter().copied().collect();

    // Create mask of pixels to reassign
    // Reassign those pixels to nearest kept color

    Ok(())
}
```

## Error Handling

```rust
// Add to ColorError enum if needed
pub enum ColorError {
    // ... existing variants ...
    /// Segmentation exceeded maximum iterations
    SegmentationFailed(String),
}
```

## Test Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_color_segment_simple_image() {
        // Create image with 3 distinct color regions
        // Verify output has 3 or fewer colors
    }

    #[test]
    fn test_color_segment_gradient() {
        // Create smooth gradient
        // Verify reasonable segmentation
    }

    #[test]
    fn test_cluster_phase_only() {
        // Test Phase 1 in isolation
    }

    #[test]
    fn test_assign_to_nearest() {
        // Test Phase 2 in isolation
    }

    #[test]
    fn test_wrong_depth() {
        // Non-32bpp should return error
    }

    #[test]
    fn test_options_validation() {
        // Invalid parameters should error
    }
}
```

## Status

- [x] Create feature branch
- [x] Create plan document
- [x] Create `segment.rs` with types and options
- [x] Implement `color_segment_cluster` (Phase 1)
- [x] Implement `assign_to_nearest_color` (Phase 2)
- [x] Implement full `color_segment` (Phase 3-4 simplified)
- [x] Add unit tests
- [x] Update `lib.rs` with exports
- [x] Run `cargo fmt && cargo clippy`
- [x] Run tests
- [x] Commit changes (73bb37a)

## Questions

None at this time.

## Future Enhancements

- Octcube LUT for faster nearest-color lookup (Phase 2)
- Full morphological cleanup (Phase 3) when leptonica-morph integration is available
- Debug output options (like C version's debugflag)
