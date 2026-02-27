//! Convolution kernels
//!
//! Defines kernel structures for image convolution operations.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use crate::core::{Pix, PixelDepth, PixelOp};
use crate::filter::{FilterError, FilterResult};

/// Kernel serialization version number (matches C KERNEL_VERSION_NUMBER)
const KERNEL_VERSION_NUMBER: u32 = 2;

/// A 2D convolution kernel
#[derive(Debug, Clone)]
pub struct Kernel {
    /// Width of the kernel
    width: u32,
    /// Height of the kernel
    height: u32,
    /// X coordinate of the center
    cx: u32,
    /// Y coordinate of the center
    cy: u32,
    /// Kernel data (row-major order)
    data: Vec<f32>,
}

impl Kernel {
    /// Create a new kernel with the given dimensions
    pub fn new(width: u32, height: u32) -> FilterResult<Self> {
        if width == 0 || height == 0 {
            return Err(FilterError::InvalidKernel(
                "width and height must be > 0".to_string(),
            ));
        }

        let size = (width * height) as usize;
        Ok(Kernel {
            width,
            height,
            cx: width / 2,
            cy: height / 2,
            data: vec![0.0; size],
        })
    }

    /// Create a kernel from a slice of values
    pub fn from_slice(width: u32, height: u32, data: &[f32]) -> FilterResult<Self> {
        let size = (width * height) as usize;
        if data.len() != size {
            return Err(FilterError::InvalidKernel(format!(
                "data length {} doesn't match dimensions {}x{}",
                data.len(),
                width,
                height
            )));
        }

        Ok(Kernel {
            width,
            height,
            cx: width / 2,
            cy: height / 2,
            data: data.to_vec(),
        })
    }

    /// Create a box (averaging) kernel
    pub fn box_kernel(size: u32) -> FilterResult<Self> {
        if size == 0 {
            return Err(FilterError::InvalidKernel("size must be > 0".to_string()));
        }

        let value = 1.0 / (size * size) as f32;
        let data = vec![value; (size * size) as usize];

        Ok(Kernel {
            width: size,
            height: size,
            cx: size / 2,
            cy: size / 2,
            data,
        })
    }

    /// Create a Gaussian kernel
    pub fn gaussian(size: u32, sigma: f32) -> FilterResult<Self> {
        if size == 0 || size.is_multiple_of(2) {
            return Err(FilterError::InvalidKernel(
                "Gaussian kernel size must be odd and > 0 to have a well-defined center"
                    .to_string(),
            ));
        }
        if sigma <= 0.0 {
            return Err(FilterError::InvalidKernel(
                "sigma must be positive".to_string(),
            ));
        }

        let half = (size / 2) as i32;
        let mut data = vec![0.0f32; (size * size) as usize];
        let mut sum = 0.0f32;

        let two_sigma_sq = 2.0 * sigma * sigma;

        for y in 0..size {
            for x in 0..size {
                let dx = (x as i32 - half) as f32;
                let dy = (y as i32 - half) as f32;
                let value = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
                data[(y * size + x) as usize] = value;
                sum += value;
            }
        }

        // Normalize
        for v in &mut data {
            *v /= sum;
        }

        Ok(Kernel {
            width: size,
            height: size,
            cx: size / 2,
            cy: size / 2,
            data,
        })
    }

    /// Create a Sobel kernel for horizontal edge detection
    pub fn sobel_horizontal() -> Self {
        Kernel {
            width: 3,
            height: 3,
            cx: 1,
            cy: 1,
            data: vec![
                1.0, 2.0, 1.0, //
                0.0, 0.0, 0.0, //
                -1.0, -2.0, -1.0,
            ],
        }
    }

    /// Create a Sobel kernel for vertical edge detection
    pub fn sobel_vertical() -> Self {
        Kernel {
            width: 3,
            height: 3,
            cx: 1,
            cy: 1,
            data: vec![
                1.0, 0.0, -1.0, //
                2.0, 0.0, -2.0, //
                1.0, 0.0, -1.0,
            ],
        }
    }

    /// Create a Laplacian kernel
    pub fn laplacian() -> Self {
        Kernel {
            width: 3,
            height: 3,
            cx: 1,
            cy: 1,
            data: vec![
                0.0, 1.0, 0.0, //
                1.0, -4.0, 1.0, //
                0.0, 1.0, 0.0,
            ],
        }
    }

    /// Create a sharpening kernel
    pub fn sharpen() -> Self {
        Kernel {
            width: 3,
            height: 3,
            cx: 1,
            cy: 1,
            data: vec![
                0.0, -1.0, 0.0, //
                -1.0, 5.0, -1.0, //
                0.0, -1.0, 0.0,
            ],
        }
    }

    /// Create an emboss kernel
    pub fn emboss() -> Self {
        Kernel {
            width: 3,
            height: 3,
            cx: 1,
            cy: 1,
            data: vec![
                -2.0, -1.0, 0.0, //
                -1.0, 1.0, 1.0, //
                0.0, 1.0, 2.0,
            ],
        }
    }

    /// Get the kernel width
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the kernel height
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the center X coordinate
    #[inline]
    pub fn center_x(&self) -> u32 {
        self.cx
    }

    /// Get the center Y coordinate
    #[inline]
    pub fn center_y(&self) -> u32 {
        self.cy
    }

    /// Set the center coordinates
    pub fn set_center(&mut self, cx: u32, cy: u32) -> FilterResult<()> {
        if cx >= self.width || cy >= self.height {
            return Err(FilterError::InvalidKernel(
                "center must be within kernel bounds".to_string(),
            ));
        }
        self.cx = cx;
        self.cy = cy;
        Ok(())
    }

    /// Get the kernel data
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get a value at (x, y)
    #[inline]
    pub fn get(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width && y < self.height {
            Some(self.data[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Set a value at (x, y)
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, value: f32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = value;
        }
    }

    /// Normalize the kernel so that values sum to 1
    pub fn normalize(&mut self) {
        let sum: f32 = self.data.iter().sum();
        if sum.abs() > f32::EPSILON {
            for v in &mut self.data {
                *v /= sum;
            }
        }
    }

    /// Get the sum of all kernel values
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    /// Get the minimum and maximum values in the kernel.
    ///
    /// C equivalent: `kernelGetMinMax`
    pub fn get_min_max(&self) -> (f32, f32) {
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        for &v in &self.data {
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
        (min_val, max_val)
    }

    /// Spatially invert the kernel about the origin.
    ///
    /// The returned kernel has data flipped both horizontally and vertically,
    /// and the center coordinates are adjusted accordingly.
    ///
    /// C equivalent: `kernelInvert`
    pub fn invert(&self) -> Kernel {
        let mut data = vec![0.0f32; self.data.len()];
        let sy = self.height;
        let sx = self.width;
        for i in 0..sy {
            for j in 0..sx {
                data[(i * sx + j) as usize] =
                    self.data[((sy - 1 - i) * sx + (sx - 1 - j)) as usize];
            }
        }
        Kernel {
            width: sx,
            height: sy,
            cx: sx - 1 - self.cx,
            cy: sy - 1 - self.cy,
            data,
        }
    }

    /// Read a kernel from a reader in the leptonica text format.
    ///
    /// C equivalent: `kernelReadStream`
    pub fn read<R: Read>(reader: R) -> FilterResult<Kernel> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        // Read version line
        let version_line = lines
            .next()
            .ok_or_else(|| FilterError::InvalidKernel("empty input".to_string()))?
            .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        let version_line = version_line.trim();
        let version: u32 = version_line
            .strip_prefix("Kernel Version ")
            .ok_or_else(|| FilterError::InvalidKernel("not a kernel file".to_string()))?
            .trim()
            .parse()
            .map_err(|_| FilterError::InvalidKernel("invalid version number".to_string()))?;
        if version != KERNEL_VERSION_NUMBER {
            return Err(FilterError::InvalidKernel(format!(
                "invalid kernel version: expected {KERNEL_VERSION_NUMBER}, got {version}"
            )));
        }

        // Read dimensions line: "  sy = %d, sx = %d, cy = %d, cx = %d"
        let dim_line = lines
            .next()
            .ok_or_else(|| FilterError::InvalidKernel("missing dimensions".to_string()))?
            .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        let (sy, sx, cy, cx) = parse_dimensions(&dim_line)?;

        // Read data
        let mut data = Vec::with_capacity((sy * sx) as usize);
        for _ in 0..sy {
            let line = lines
                .next()
                .ok_or_else(|| FilterError::InvalidKernel("missing data line".to_string()))?
                .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
            for token in line.split_whitespace() {
                let val: f32 = token
                    .parse()
                    .map_err(|_| FilterError::InvalidKernel(format!("invalid float: {token}")))?;
                data.push(val);
            }
        }

        if data.len() != (sy * sx) as usize {
            return Err(FilterError::InvalidKernel(format!(
                "expected {} values, got {}",
                sy * sx,
                data.len()
            )));
        }

        Ok(Kernel {
            width: sx,
            height: sy,
            cx,
            cy,
            data,
        })
    }

    /// Write a kernel to a writer in the leptonica text format.
    ///
    /// C equivalent: `kernelWriteStream`
    pub fn write<W: Write>(&self, mut writer: W) -> FilterResult<()> {
        writeln!(writer, "  Kernel Version {KERNEL_VERSION_NUMBER}")
            .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        writeln!(
            writer,
            "  sy = {}, sx = {}, cy = {}, cx = {}",
            self.height, self.width, self.cy, self.cx
        )
        .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        for i in 0..self.height {
            for j in 0..self.width {
                write!(writer, "{:15.4}", self.data[(i * self.width + j) as usize])
                    .map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
            }
            writeln!(writer).map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        }
        writeln!(writer).map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        Ok(())
    }

    /// Create a kernel from a string of space-separated values.
    ///
    /// The string contains space/tab/newline-separated numeric values in row-major order.
    ///
    /// C equivalent: `kernelCreateFromString`
    pub fn from_string(
        height: u32,
        width: u32,
        cy: u32,
        cx: u32,
        kdata: &str,
    ) -> FilterResult<Kernel> {
        if height == 0 || width == 0 {
            return Err(FilterError::InvalidKernel(
                "height and width must be > 0".to_string(),
            ));
        }
        if cy >= height || cx >= width {
            return Err(FilterError::InvalidKernel(
                "center must be within kernel bounds".to_string(),
            ));
        }

        let values: Vec<f32> = kdata
            .split_whitespace()
            .map(|s| {
                s.parse::<f32>()
                    .map_err(|_| FilterError::InvalidKernel(format!("invalid number: {s}")))
            })
            .collect::<FilterResult<Vec<f32>>>()?;

        let expected = (height * width) as usize;
        if values.len() != expected {
            return Err(FilterError::InvalidKernel(format!(
                "expected {} values, got {}",
                expected,
                values.len()
            )));
        }

        Ok(Kernel {
            width,
            height,
            cx,
            cy,
            data: values,
        })
    }

    /// Create a kernel from a file in the simple leptonica text format.
    ///
    /// The file format is:
    /// - Lines starting with '#' are comments
    /// - First non-comment line: `height width`
    /// - Second non-comment line: `cy cx`
    /// - Subsequent lines: kernel data values
    ///
    /// C equivalent: `kernelCreateFromFile`
    pub fn from_file(path: &Path) -> FilterResult<Kernel> {
        let content =
            std::fs::read_to_string(path).map_err(|e| FilterError::InvalidKernel(e.to_string()))?;
        if content.is_empty() {
            return Err(FilterError::InvalidKernel("file is empty".to_string()));
        }

        let lines: Vec<&str> = content.lines().collect();

        // Find first non-comment, non-blank line
        let mut first = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                first = Some(i);
                break;
            }
        }
        let first = first.ok_or_else(|| {
            FilterError::InvalidKernel(
                "no header line found (blank or comment-only file)".to_string(),
            )
        })?;

        // Parse dimensions
        let dim_parts: Vec<&str> = lines[first].split_whitespace().collect();
        if dim_parts.len() != 2 {
            return Err(FilterError::InvalidKernel("error reading h,w".to_string()));
        }
        let h: u32 = dim_parts[0]
            .parse()
            .map_err(|_| FilterError::InvalidKernel("invalid height".to_string()))?;
        let w: u32 = dim_parts[1]
            .parse()
            .map_err(|_| FilterError::InvalidKernel("invalid width".to_string()))?;

        // Parse origin
        if first + 1 >= lines.len() {
            return Err(FilterError::InvalidKernel(
                "missing origin line".to_string(),
            ));
        }
        let origin_parts: Vec<&str> = lines[first + 1].split_whitespace().collect();
        if origin_parts.len() != 2 {
            return Err(FilterError::InvalidKernel(
                "error reading cy,cx".to_string(),
            ));
        }
        let cy: u32 = origin_parts[0]
            .parse()
            .map_err(|_| FilterError::InvalidKernel("invalid cy".to_string()))?;
        let cx: u32 = origin_parts[1]
            .parse()
            .map_err(|_| FilterError::InvalidKernel("invalid cx".to_string()))?;

        // Parse data values
        let mut data = Vec::new();
        for &line in &lines[first + 2..] {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            for token in trimmed.split_whitespace() {
                let val: f32 = token
                    .parse()
                    .map_err(|_| FilterError::InvalidKernel(format!("invalid number: {token}")))?;
                data.push(val);
            }
        }

        let expected = (h * w) as usize;
        if data.len() != expected {
            return Err(FilterError::InvalidKernel(format!(
                "expected {} values, got {}",
                expected,
                data.len()
            )));
        }

        if cy >= h || cx >= w {
            return Err(FilterError::InvalidKernel(
                "center must be within kernel bounds".to_string(),
            ));
        }

        Ok(Kernel {
            width: w,
            height: h,
            cx,
            cy,
            data,
        })
    }

    /// Create a kernel from an 8bpp grayscale Pix.
    ///
    /// Each pixel value becomes the corresponding kernel element.
    ///
    /// C equivalent: `kernelCreateFromPix`
    pub fn from_pix(pix: &Pix, cy: u32, cx: u32) -> FilterResult<Kernel> {
        if pix.depth() != PixelDepth::Bit8 {
            return Err(FilterError::UnsupportedDepth {
                expected: "8-bpp grayscale",
                actual: pix.depth().bits(),
            });
        }
        let w = pix.width();
        let h = pix.height();
        if cx >= w || cy >= h {
            return Err(FilterError::InvalidKernel(
                "(cy, cx) must be within pix bounds".to_string(),
            ));
        }

        let mut data = vec![0.0f32; (h * w) as usize];
        for i in 0..h {
            for j in 0..w {
                let val = pix.get_pixel(j, i).unwrap_or(0);
                data[(i * w + j) as usize] = val as f32;
            }
        }

        Ok(Kernel {
            width: w,
            height: h,
            cx,
            cy,
            data,
        })
    }

    /// Visualize the kernel as an 8bpp Pix.
    ///
    /// Two modes:
    /// - `size == 1` and not `normalized`: one pixel per kernel element, absolute values
    ///   normalized so the max absolute value maps to 255.
    /// - `size >= 17` (odd) and `normalized`: grid display with cells of the given size,
    ///   grid lines of width 2, and a cross-hair at the origin.
    ///
    /// C equivalent: `kernelDisplayInPix`
    pub fn display_in_pix(&self, size: u32, normalized: bool) -> FilterResult<Pix> {
        let (min_val, max_val) = self.get_min_max();
        let max_abs = max_val.abs().max(min_val.abs());
        if max_abs == 0.0 {
            return Err(FilterError::InvalidKernel(
                "kernel elements all 0.0".to_string(),
            ));
        }
        let norm = 255.0 / max_abs;

        let sx = self.width;
        let sy = self.height;

        // Simple 1-pixel-per-element mode
        if size == 1 && !normalized {
            let pix = Pix::new(sx, sy, PixelDepth::Bit8)?;
            let mut pix_mut = pix.try_into_mut().unwrap();
            for i in 0..sy {
                for j in 0..sx {
                    let val = self.data[(i * sx + j) as usize];
                    let normval = (norm * val.abs()) as u32;
                    pix_mut.set_pixel_unchecked(j, i, normval.min(255));
                }
            }
            return Ok(pix_mut.into());
        }

        // Grid display mode
        let size = if size < 17 { 17u32 } else { size };
        let size = if size % 2 == 0 { size + 1 } else { size };
        let gthick: u32 = 2;

        let w = size * sx + gthick * (sx + 1);
        let h = size * sy + gthick * (sy + 1);
        let pix = Pix::new(w, h, PixelDepth::Bit8)?;
        let mut pix_mut = pix.try_into_mut().unwrap();

        // Draw grid lines
        for i in 0..=sy {
            let y = (gthick / 2 + i * (size + gthick)) as i32;
            pix_mut
                .render_line(0, y, w as i32 - 1, y, gthick, PixelOp::Set)
                .map_err(FilterError::Core)?;
        }
        for j in 0..=sx {
            let x = (gthick / 2 + j * (size + gthick)) as i32;
            pix_mut
                .render_line(x, 0, x, h as i32 - 1, gthick, PixelOp::Set)
                .map_err(FilterError::Core)?;
        }

        // Fill cells
        let mut y0 = gthick;
        for i in 0..sy {
            let mut x0 = gthick;
            for j in 0..sx {
                let val = self.data[(i * sx + j) as usize];
                let normval = (norm * val.abs()) as u32;
                let normval = normval.min(255);

                // Fill the cell with the normalized value
                for dy in 0..size {
                    for dx in 0..size {
                        pix_mut.set_pixel_unchecked(x0 + dx, y0 + dy, normval);
                    }
                }

                // Draw origin marker
                if i == self.cy && j == self.cx {
                    let line_width = size / 8;
                    let line_width = line_width.max(1);
                    let half = size / 2;

                    // Vertical bar
                    let y_start = (0.12 * size as f32) as u32;
                    let y_end = (0.88 * size as f32) as u32;
                    for dy in y_start..y_end {
                        for dw in 0..line_width {
                            let px = x0 + half.saturating_sub(line_width / 2) + dw;
                            let py = y0 + dy;
                            if px < x0 + size && py < y0 + size {
                                pix_mut.set_pixel_unchecked(px, py, 255u32.saturating_sub(normval));
                            }
                        }
                    }

                    // Horizontal bar
                    let x_start = (0.15 * size as f32) as u32;
                    let x_end = (0.85 * size as f32) as u32;
                    for dx in x_start..x_end {
                        for dw in 0..line_width {
                            let px = x0 + dx;
                            let py = y0 + half.saturating_sub(line_width / 2) + dw;
                            if px < x0 + size && py < y0 + size {
                                let current = pix_mut.get_pixel(px, py).unwrap_or(0);
                                // Flip - XOR effect for the crossing
                                let new_val = if current == 255u32.saturating_sub(normval) {
                                    normval
                                } else {
                                    255u32.saturating_sub(normval)
                                };
                                pix_mut.set_pixel_unchecked(px, py, new_val);
                            }
                        }
                    }
                }

                x0 += size + gthick;
            }
            y0 += size + gthick;
        }

        Ok(pix_mut.into())
    }
}

/// Parse the dimensions line: "  sy = %d, sx = %d, cy = %d, cx = %d"
fn parse_dimensions(line: &str) -> FilterResult<(u32, u32, u32, u32)> {
    let line = line.trim();
    // Parse "sy = N, sx = N, cy = N, cx = N"
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 4 {
        return Err(FilterError::InvalidKernel(
            "invalid dimensions format".to_string(),
        ));
    }

    fn extract_value(s: &str) -> FilterResult<u32> {
        let s = s.trim();
        let val_str = s
            .split('=')
            .nth(1)
            .ok_or_else(|| FilterError::InvalidKernel(format!("missing '=' in: {s}")))?
            .trim();
        val_str
            .parse()
            .map_err(|_| FilterError::InvalidKernel(format!("invalid number in: {s}")))
    }

    let sy = extract_value(parts[0])?;
    let sx = extract_value(parts[1])?;
    let cy = extract_value(parts[2])?;
    let cx = extract_value(parts[3])?;

    Ok((sy, sx, cy, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_kernel() {
        let k = Kernel::box_kernel(3).unwrap();
        assert_eq!(k.width(), 3);
        assert_eq!(k.height(), 3);

        // Sum should be approximately 1
        let sum: f32 = k.data().iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gaussian_kernel() {
        let k = Kernel::gaussian(5, 1.0).unwrap();
        assert_eq!(k.width(), 5);
        assert_eq!(k.height(), 5);

        // Sum should be approximately 1
        let sum: f32 = k.data().iter().sum();
        assert!((sum - 1.0).abs() < 0.001);

        // Center should be the maximum
        let center_val = k.get(2, 2).unwrap();
        for v in k.data() {
            assert!(*v <= center_val + f32::EPSILON);
        }
    }

    #[test]
    fn test_sobel_kernels() {
        let h = Kernel::sobel_horizontal();
        let v = Kernel::sobel_vertical();

        assert_eq!(h.width(), 3);
        assert_eq!(v.width(), 3);

        // Sobel kernels sum to 0
        assert!((h.sum()).abs() < 0.001);
        assert!((v.sum()).abs() < 0.001);
    }

    #[test]
    fn test_from_slice() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let k = Kernel::from_slice(3, 3, &data).unwrap();

        assert_eq!(k.get(0, 0), Some(1.0));
        assert_eq!(k.get(2, 2), Some(9.0));
    }

    #[test]
    fn test_get_min_max() {
        let data = [-3.0, 1.0, 0.0, 5.0, -1.0, 2.0, 4.0, 0.5, 3.0];
        let k = Kernel::from_slice(3, 3, &data).unwrap();
        let (min, max) = k.get_min_max();
        assert!((min - (-3.0)).abs() < f32::EPSILON);
        assert!((max - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_min_max_uniform() {
        let k = Kernel::box_kernel(3).unwrap();
        let (min, max) = k.get_min_max();
        assert!((min - max).abs() < f32::EPSILON);
    }

    #[test]
    fn test_invert() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let k = Kernel::from_slice(3, 3, &data).unwrap();
        let inv = k.invert();

        // Spatial inversion: data is flipped
        // get(x, y) maps to data[y*width + x]
        assert_eq!(inv.get(0, 0), Some(9.0)); // old(2,2)
        assert_eq!(inv.get(1, 1), Some(5.0)); // old(1,1) - center stays
        assert_eq!(inv.get(2, 2), Some(1.0)); // old(0,0)
        // (x=0,y=2) -> inv.data[6] = old.data[(0)*3+(2)] = old.data[2] = 3
        assert_eq!(inv.get(0, 2), Some(3.0));
        // (x=2,y=0) -> inv.data[2] = old.data[(2)*3+(0)] = old.data[6] = 7
        assert_eq!(inv.get(2, 0), Some(7.0));

        // Center is also inverted
        assert_eq!(inv.center_x(), 3 - 1 - k.center_x());
        assert_eq!(inv.center_y(), 3 - 1 - k.center_y());
    }

    #[test]
    fn test_write_read_roundtrip() {
        let data = [1.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.5];
        let k = Kernel::from_slice(3, 3, &data).unwrap();

        let mut buf = Vec::new();
        k.write(&mut buf).unwrap();

        let k2 = Kernel::read(buf.as_slice()).unwrap();
        assert_eq!(k2.width(), k.width());
        assert_eq!(k2.height(), k.height());
        assert_eq!(k2.center_x(), k.center_x());
        assert_eq!(k2.center_y(), k.center_y());
        for i in 0..9 {
            assert!((k2.data()[i] - k.data()[i]).abs() < 0.01);
        }
    }

    #[test]
    fn test_write_format() {
        let k = Kernel::from_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]).unwrap();
        let mut buf = Vec::new();
        k.write(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Kernel Version 2"));
        assert!(output.contains("sy = 2, sx = 2, cy = 1, cx = 1"));
    }

    #[test]
    fn test_from_string() {
        let k = Kernel::from_string(3, 3, 1, 1, "1 2 3 4 5 6 7 8 9").unwrap();
        assert_eq!(k.width(), 3);
        assert_eq!(k.height(), 3);
        assert_eq!(k.center_x(), 1);
        assert_eq!(k.center_y(), 1);
        assert_eq!(k.get(0, 0), Some(1.0));
        assert_eq!(k.get(2, 2), Some(9.0));
    }

    #[test]
    fn test_from_string_multiline() {
        let kdata = "20   50   20\n70  140   70\n20   50   20";
        let k = Kernel::from_string(3, 3, 1, 1, kdata).unwrap();
        assert_eq!(k.get(1, 1), Some(140.0));
    }

    #[test]
    fn test_from_string_wrong_count() {
        assert!(Kernel::from_string(3, 3, 1, 1, "1 2 3 4 5").is_err());
    }

    #[test]
    fn test_from_string_invalid_center() {
        assert!(Kernel::from_string(3, 3, 3, 1, "1 2 3 4 5 6 7 8 9").is_err());
    }

    #[test]
    fn test_from_file() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "test_kernel_{}_{:?}_.txt",
            std::process::id(),
            std::thread::current().id()
        ));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "# test kernel").unwrap();
            writeln!(f, "3 3").unwrap();
            writeln!(f, "1 1").unwrap();
            writeln!(f, "25.5 51 24.3").unwrap();
            writeln!(f, "70.2 146.3 73.4").unwrap();
            writeln!(f, "20 50.9 18.4").unwrap();
        }
        let k = Kernel::from_file(&path).unwrap();
        assert_eq!(k.width(), 3);
        assert_eq!(k.height(), 3);
        assert_eq!(k.center_x(), 1);
        assert_eq!(k.center_y(), 1);
        assert!((k.get(1, 1).unwrap() - 146.3).abs() < 0.01);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_from_pix() {
        let pix = Pix::new(3, 3, PixelDepth::Bit8).unwrap();
        let mut pix_mut = pix.try_into_mut().unwrap();
        pix_mut.set_pixel_unchecked(0, 0, 10);
        pix_mut.set_pixel_unchecked(1, 1, 128);
        pix_mut.set_pixel_unchecked(2, 2, 255);
        let pix: Pix = pix_mut.into();

        let k = Kernel::from_pix(&pix, 1, 1).unwrap();
        assert_eq!(k.width(), 3);
        assert_eq!(k.height(), 3);
        assert_eq!(k.get(0, 0), Some(10.0));
        assert_eq!(k.get(1, 1), Some(128.0));
        assert_eq!(k.get(2, 2), Some(255.0));
    }

    #[test]
    fn test_from_pix_wrong_depth() {
        let pix = Pix::new(3, 3, PixelDepth::Bit32).unwrap();
        assert!(Kernel::from_pix(&pix, 1, 1).is_err());
    }

    #[test]
    fn test_display_in_pix_simple() {
        let data = [0.0, 1.0, 0.0, 1.0, 4.0, 1.0, 0.0, 1.0, 0.0];
        let k = Kernel::from_slice(3, 3, &data).unwrap();
        let pix = k.display_in_pix(1, false).unwrap();
        assert_eq!(pix.width(), 3);
        assert_eq!(pix.height(), 3);
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        // Center (4.0 is max) should map to 255
        let center = pix.get_pixel(1, 1).unwrap();
        assert_eq!(center, 255);
    }

    #[test]
    fn test_display_in_pix_grid() {
        let data = [0.0, 1.0, 0.0, 1.0, 4.0, 1.0, 0.0, 1.0, 0.0];
        let k = Kernel::from_slice(3, 3, &data).unwrap();
        let pix = k.display_in_pix(17, true).unwrap();
        assert_eq!(pix.depth(), PixelDepth::Bit8);
        // Grid mode: size is 17*3 + 2*(3+1) = 51 + 8 = 59
        assert_eq!(pix.width(), 59);
        assert_eq!(pix.height(), 59);
    }

    #[test]
    fn test_display_in_pix_zero_kernel() {
        let k = Kernel::new(3, 3).unwrap(); // all zeros
        assert!(k.display_in_pix(1, false).is_err());
    }

    #[test]
    fn test_read_invalid_version() {
        let data = b"  Kernel Version 99\n  sy = 2, sx = 2, cy = 0, cx = 0\n 1.0 2.0\n 3.0 4.0\n\n";
        assert!(Kernel::read(data.as_slice()).is_err());
    }

    #[test]
    fn test_read_empty() {
        let data = b"";
        assert!(Kernel::read(data.as_slice()).is_err());
    }
}
