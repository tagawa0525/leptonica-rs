#![allow(dead_code, unused_imports)]
//! Regression test framework for Leptonica
//!
//! This module provides a regression test framework similar to the C version's
//! regutils.c, supporting three modes:
//!
//! - **Generate**: Create golden files for comparison
//! - **Compare**: Compare results with golden files
//! - **Display**: Run tests without comparison (visual inspection)
//!
//! # Usage
//!
//! ```ignore
//! mod common;
//! use common::{RegParams, RegTestMode};
//!
//! let mut rp = RegParams::new("conncomp");
//! rp.compare_values(4452.0, count as f64, 0.0);
//! assert!(rp.cleanup());
//! ```
//!
//! # Environment Variables
//!
//! - `REGTEST_MODE`: Set to "generate", "compare", or "display"

mod error;
mod params;

pub use error::{TestError, TestResult};
pub use params::{RegParams, RegTestMode};

/// Load a test image from the test data directory
///
/// # Arguments
///
/// * `name` - Image filename (e.g., "feyn.tif")
///
/// # Returns
///
/// The loaded image, or an error if loading fails.
pub fn load_test_image(name: &str) -> TestResult<leptonica::Pix> {
    let path = test_data_path(name);
    leptonica::io::read_image(&path).map_err(|e| TestError::ImageLoad {
        path: path.clone(),
        message: e.to_string(),
    })
}

/// Get the path to the workspace root
fn workspace_root() -> String {
    env!("CARGO_MANIFEST_DIR").to_string()
}

/// Get the path to a test data file
pub fn test_data_path(name: &str) -> String {
    format!("{}/tests/data/images/{}", workspace_root(), name)
}

/// Get the path to the golden files directory
pub fn golden_dir() -> String {
    format!("{}/tests/golden", workspace_root())
}

/// Get the path to the regout (regression output) directory
pub fn regout_dir() -> String {
    format!("{}/tests/regout", workspace_root())
}
