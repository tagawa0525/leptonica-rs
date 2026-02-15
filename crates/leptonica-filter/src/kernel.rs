//! Convolution kernels
//!
//! Defines kernel structures for image convolution operations.

use crate::{FilterError, FilterResult};

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
}
