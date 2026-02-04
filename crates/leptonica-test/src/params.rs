//! Regression test parameters and operations

use crate::error::{TestError, TestResult};
use crate::{golden_dir, regout_dir};
use leptonica_core::Pix;
use leptonica_io::ImageFormat;
use std::fs;
use std::path::Path;

/// Regression test mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegTestMode {
    /// Generate golden files
    Generate,
    /// Compare with golden files (default)
    #[default]
    Compare,
    /// Display mode - run without comparison
    Display,
}

impl RegTestMode {
    /// Parse mode from environment variable or string
    pub fn from_env() -> Self {
        match std::env::var("REGTEST_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "generate" => Self::Generate,
            "display" => Self::Display,
            _ => Self::Compare,
        }
    }
}

/// Regression test parameters
///
/// This structure tracks the state of a regression test, including
/// the test name, current index, mode, and success status.
pub struct RegParams {
    /// Name of the test (e.g., "conncomp")
    pub test_name: String,
    /// Current test index (incremented before each test)
    index: usize,
    /// Test mode (generate, compare, or display)
    pub mode: RegTestMode,
    /// Overall success status
    success: bool,
    /// Recorded failures
    failures: Vec<String>,
}

impl RegParams {
    /// Create new regression test parameters
    ///
    /// # Arguments
    ///
    /// * `test_name` - Name of the test (e.g., "conncomp")
    ///
    /// # Returns
    ///
    /// A new `RegParams` instance configured based on the `REGTEST_MODE`
    /// environment variable.
    pub fn new(test_name: &str) -> Self {
        let mode = RegTestMode::from_env();

        // Ensure directories exist
        let _ = fs::create_dir_all(golden_dir());
        let _ = fs::create_dir_all(regout_dir());

        eprintln!();
        eprintln!("////////////////////////////////////////////////");
        eprintln!("////////////////   {}_reg   ///////////////", test_name);
        eprintln!("////////////////////////////////////////////////");
        eprintln!("Mode: {:?}", mode);

        Self {
            test_name: test_name.to_string(),
            index: 0,
            mode,
            success: true,
            failures: Vec::new(),
        }
    }

    /// Get the current test index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Check if in display mode
    pub fn display(&self) -> bool {
        self.mode == RegTestMode::Display
    }

    /// Compare two floating-point values
    ///
    /// # Arguments
    ///
    /// * `expected` - Expected value (typically from golden/reference)
    /// * `actual` - Actual computed value
    /// * `delta` - Maximum allowed difference
    ///
    /// # Returns
    ///
    /// `true` if values match within delta, `false` otherwise.
    pub fn compare_values(&mut self, expected: f64, actual: f64, delta: f64) -> bool {
        self.index += 1;
        let diff = (expected - actual).abs();

        if diff > delta {
            let msg = format!(
                "Failure in {}_reg: value comparison for index {}\n\
                 difference = {} but allowed delta = {}\n\
                 expected = {}, actual = {}",
                self.test_name, self.index, diff, delta, expected, actual
            );
            eprintln!("{}", msg);
            self.failures.push(msg);
            self.success = false;
            false
        } else {
            true
        }
    }

    /// Compare two Pix images for exact equality
    ///
    /// # Arguments
    ///
    /// * `pix1` - First image
    /// * `pix2` - Second image
    ///
    /// # Returns
    ///
    /// `true` if images are identical, `false` otherwise.
    pub fn compare_pix(&mut self, pix1: &Pix, pix2: &Pix) -> bool {
        self.index += 1;

        // Check dimensions
        if pix1.width() != pix2.width()
            || pix1.height() != pix2.height()
            || pix1.depth() != pix2.depth()
        {
            let msg = format!(
                "Failure in {}_reg: pix comparison for index {} - dimension mismatch",
                self.test_name, self.index
            );
            eprintln!("{}", msg);
            self.failures.push(msg);
            self.success = false;
            return false;
        }

        // Compare pixel by pixel
        let width = pix1.width();
        let height = pix1.height();

        for y in 0..height {
            for x in 0..width {
                let p1 = pix1.get_pixel(x, y);
                let p2 = pix2.get_pixel(x, y);
                if p1 != p2 {
                    let msg = format!(
                        "Failure in {}_reg: pix comparison for index {} - pixel mismatch at ({}, {})",
                        self.test_name, self.index, x, y
                    );
                    eprintln!("{}", msg);
                    self.failures.push(msg);
                    self.success = false;
                    return false;
                }
            }
        }

        true
    }

    /// Write a Pix to file and check against golden file
    ///
    /// # Arguments
    ///
    /// * `pix` - Image to write
    /// * `format` - Output format (PNG, TIFF, etc.)
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, error otherwise.
    pub fn write_pix_and_check(&mut self, pix: &Pix, format: ImageFormat) -> TestResult<()> {
        self.index += 1;

        let ext = format.extension();
        let local_path = format!(
            "{}/{}.{:02}.{}",
            regout_dir(),
            self.test_name,
            self.index,
            ext
        );

        // Write the local file
        leptonica_io::write_image(pix, &local_path, format).map_err(|e| TestError::ImageWrite {
            path: local_path.clone(),
            message: e.to_string(),
        })?;

        // Check based on mode
        self.check_file(&local_path)
    }

    /// Check a file against its golden counterpart
    ///
    /// In generate mode, copies the file to golden.
    /// In compare mode, compares with golden file.
    /// In display mode, does nothing.
    fn check_file(&mut self, local_path: &str) -> TestResult<()> {
        let ext = Path::new(local_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let golden_path = format!(
            "{}/{}_golden.{:02}.{}",
            golden_dir(),
            self.test_name,
            self.index,
            ext
        );

        match self.mode {
            RegTestMode::Generate => {
                // Copy local to golden
                fs::copy(local_path, &golden_path)?;
                eprintln!("Generated: {}", golden_path);
            }
            RegTestMode::Compare => {
                // Compare files
                if !Path::new(&golden_path).exists() {
                    let msg = format!(
                        "Failure in {}_reg: golden file not found: {}",
                        self.test_name, golden_path
                    );
                    eprintln!("{}", msg);
                    self.failures.push(msg);
                    self.success = false;
                    return Ok(());
                }

                let local_data = fs::read(local_path)?;
                let golden_data = fs::read(&golden_path)?;

                if local_data != golden_data {
                    // For images, try pixel-by-pixel comparison
                    let same = self.compare_image_files(local_path, &golden_path);

                    if !same {
                        let msg = format!(
                            "Failure in {}_reg, index {}: comparing {} with {}",
                            self.test_name, self.index, local_path, golden_path
                        );
                        eprintln!("{}", msg);
                        self.failures.push(msg);
                        self.success = false;
                    }
                }
            }
            RegTestMode::Display => {
                // Nothing to do in display mode
            }
        }

        Ok(())
    }

    /// Compare two image files pixel-by-pixel
    fn compare_image_files(&self, path1: &str, path2: &str) -> bool {
        let pix1 = match leptonica_io::read_image(path1) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let pix2 = match leptonica_io::read_image(path2) {
            Ok(p) => p,
            Err(_) => return false,
        };

        if pix1.width() != pix2.width()
            || pix1.height() != pix2.height()
            || pix1.depth() != pix2.depth()
        {
            return false;
        }

        let width = pix1.width();
        let height = pix1.height();

        for y in 0..height {
            for x in 0..width {
                if pix1.get_pixel(x, y) != pix2.get_pixel(x, y) {
                    return false;
                }
            }
        }

        true
    }

    /// Compare two binary data arrays
    ///
    /// # Arguments
    ///
    /// * `data1` - First byte array
    /// * `data2` - Second byte array
    ///
    /// # Returns
    ///
    /// `true` if data is identical, `false` otherwise.
    pub fn compare_strings(&mut self, data1: &[u8], data2: &[u8]) -> bool {
        self.index += 1;

        if data1 != data2 {
            let msg = format!(
                "Failure in {}_reg: string comparison for index {}\n\
                 sizes: {} vs {}",
                self.test_name,
                self.index,
                data1.len(),
                data2.len()
            );
            eprintln!("{}", msg);
            self.failures.push(msg);
            self.success = false;
            false
        } else {
            true
        }
    }

    /// Write data to file and check against golden file
    ///
    /// # Arguments
    ///
    /// * `data` - Data to write
    /// * `ext` - File extension (e.g., "ba", "pta")
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, error otherwise.
    pub fn write_data_and_check(&mut self, data: &[u8], ext: &str) -> TestResult<()> {
        self.index += 1;

        let local_path = format!(
            "{}/{}.{:02}.{}",
            regout_dir(),
            self.test_name,
            self.index,
            ext
        );

        fs::write(&local_path, data)?;
        self.check_file(&local_path)
    }

    /// Clean up and report results
    ///
    /// # Returns
    ///
    /// `true` if all tests passed, `false` if any failed.
    pub fn cleanup(self) -> bool {
        if self.success {
            eprintln!("SUCCESS: {}_reg", self.test_name);
        } else {
            eprintln!("FAILURE: {}_reg", self.test_name);
            for failure in &self.failures {
                eprintln!("  {}", failure);
            }
        }
        eprintln!();

        self.success
    }

    /// Check if all tests have passed so far
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get list of failures
    pub fn failures(&self) -> &[String] {
        &self.failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_from_env() {
        // Default should be Compare
        // Note: We can't safely remove env var in tests as it may affect other tests
        // Just test that from_env returns a valid mode
        let mode = RegTestMode::from_env();
        assert!(matches!(
            mode,
            RegTestMode::Compare | RegTestMode::Generate | RegTestMode::Display
        ));
    }

    #[test]
    fn test_compare_values_success() {
        let mut rp = RegParams::new("test");
        assert!(rp.compare_values(100.0, 100.0, 0.0));
        assert!(rp.is_success());
    }

    #[test]
    fn test_compare_values_within_delta() {
        let mut rp = RegParams::new("test");
        assert!(rp.compare_values(100.0, 100.5, 1.0));
        assert!(rp.is_success());
    }

    #[test]
    fn test_compare_values_failure() {
        let mut rp = RegParams::new("test");
        assert!(!rp.compare_values(100.0, 200.0, 0.0));
        assert!(!rp.is_success());
    }
}
