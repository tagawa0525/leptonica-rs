//! Batch file conversion to 1 bpp
//!
//! Convert image files in a directory to 1 bpp binary images.
//!
//! # See also
//! C Leptonica: `convertfiles.c`

use std::path::Path;

use crate::core::{ImageFormat, Pix, PixelDepth};
use crate::io::{IoError, IoResult};

/// Batch convert image files to 1 bpp
///
/// Reads images from `dir_in`, converts to binary, and writes to `dir_out`.
///
/// # Arguments
/// * `dir_in` - input directory
/// * `substr` - optional filename substring filter
/// * `upscaling` - 1, 2, or 4
/// * `thresh` - binarization threshold (0 for default 128)
/// * `first_page` - first page index (0-based)
/// * `n_pages` - number of pages (0 for all)
/// * `dir_out` - output directory
/// * `out_format` - output format (typically PNG or TIFF-G4)
///
/// # See also
/// C Leptonica: `convertFilesTo1bpp()` in `convertfiles.c`
#[allow(clippy::too_many_arguments)]
pub fn convert_files_to_1bpp(
    dir_in: impl AsRef<Path>,
    substr: Option<&str>,
    upscaling: u32,
    thresh: u32,
    first_page: usize,
    n_pages: usize,
    dir_out: impl AsRef<Path>,
    out_format: ImageFormat,
) -> IoResult<()> {
    if upscaling != 1 && upscaling != 2 && upscaling != 4 {
        return Err(IoError::InvalidData("upscaling must be 1, 2, or 4".into()));
    }

    let thresh = if thresh == 0 { 128 } else { thresh };
    let dir_in = dir_in.as_ref();
    let dir_out = dir_out.as_ref();

    // Ensure output directory exists
    std::fs::create_dir_all(dir_out).map_err(IoError::Io)?;

    // Get sorted file list
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir_in)
        .map_err(IoError::Io)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            match substr {
                Some(s) => name.contains(s),
                None => true,
            }
        })
        .map(|e| e.path())
        .collect();
    paths.sort();

    // Apply page range
    let end = if n_pages == 0 {
        paths.len()
    } else {
        (first_page + n_pages).min(paths.len())
    };
    let paths = &paths[first_page.min(paths.len())..end];

    let ext = crate::io::get_format_extension(out_format);

    for path in paths {
        // Read image
        let pix = crate::io::read_image(path)?;

        // Convert to grayscale if needed
        let gray = convert_to_grayscale(&pix)?;

        // Binarize (with optional upscaling)
        let binary = binarize_with_upscaling(&gray, upscaling, thresh)?;

        // Build output filename
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let out_path = dir_out.join(format!("{stem}.{ext}"));

        crate::io::write_image(&binary, &out_path, out_format)?;
    }

    Ok(())
}

/// Convert a pixel image to 8-bit grayscale
fn convert_to_grayscale(pix: &Pix) -> IoResult<Pix> {
    match pix.depth() {
        PixelDepth::Bit1 => Ok(pix.clone()),
        PixelDepth::Bit8 => Ok(pix.clone()),
        PixelDepth::Bit32 => {
            // Convert RGB to luminance
            let w = pix.width();
            let h = pix.height();
            let gray = Pix::new(w, h, PixelDepth::Bit8).map_err(IoError::Core)?;
            let mut gray_mut = gray
                .try_into_mut()
                .map_err(|_| IoError::Core(crate::core::Error::AllocationFailed))?;

            for y in 0..h {
                for x in 0..w {
                    let pixel = pix.get_pixel(x, y).unwrap_or(0);
                    let r = (pixel >> 24) & 0xff;
                    let g = (pixel >> 16) & 0xff;
                    let b = (pixel >> 8) & 0xff;
                    // Standard luminance weights
                    let lum = ((77 * r + 150 * g + 29 * b) >> 8).min(255);
                    gray_mut.set_pixel(x, y, lum).map_err(IoError::Core)?;
                }
            }
            Ok(gray_mut.into())
        }
        _depth => {
            // For other depths, try direct use
            Ok(pix.clone())
        }
    }
}

/// Binarize a grayscale image with optional upscaling
fn binarize_with_upscaling(pix: &Pix, upscaling: u32, thresh: u32) -> IoResult<Pix> {
    if pix.depth() == PixelDepth::Bit1 {
        return Ok(pix.clone());
    }

    let w = pix.width();
    let h = pix.height();

    // For upscaling > 1, we scale then threshold
    let (work_pix, work_thresh) = if upscaling > 1 {
        let new_w = w * upscaling;
        let new_h = h * upscaling;
        let scaled = Pix::new(new_w, new_h, pix.depth()).map_err(IoError::Core)?;
        let mut scaled_mut = scaled
            .try_into_mut()
            .map_err(|_| IoError::Core(crate::core::Error::AllocationFailed))?;

        // Nearest-neighbor upscaling
        for y in 0..new_h {
            for x in 0..new_w {
                let sx = x / upscaling;
                let sy = y / upscaling;
                let val = pix.get_pixel(sx, sy).unwrap_or(0);
                scaled_mut.set_pixel(x, y, val).map_err(IoError::Core)?;
            }
        }
        (scaled_mut.into(), thresh)
    } else {
        (pix.clone(), thresh)
    };

    // Threshold
    let final_w = work_pix.width();
    let final_h = work_pix.height();
    let binary = Pix::new(final_w, final_h, PixelDepth::Bit1).map_err(IoError::Core)?;
    let mut binary_mut = binary
        .try_into_mut()
        .map_err(|_| IoError::Core(crate::core::Error::AllocationFailed))?;

    for y in 0..final_h {
        for x in 0..final_w {
            let val = work_pix.get_pixel(x, y).unwrap_or(0);
            // In 1 bpp: fg = 1 (black), bg = 0 (white)
            // If grayscale value < threshold, it's dark → foreground
            if val < work_thresh {
                binary_mut.set_pixel(x, y, 1).map_err(IoError::Core)?;
            }
        }
    }

    Ok(binary_mut.into())
}
