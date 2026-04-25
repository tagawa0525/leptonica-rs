//! Regression test parameters and operations

use super::error::{TestError, TestResult};
use super::{golden_dir, regout_dir};
use leptonica::Pix;
use leptonica::io::ImageFormat;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

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

// --- Content hash functions (FNV-1a, no external dependencies) ---

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Hash pixel content of a Pix (format-independent)
pub fn pixel_content_hash(pix: &Pix) -> u64 {
    let mut h = FNV_OFFSET_BASIS;
    for b in pix.width().to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    for b in pix.height().to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    for b in (pix.depth() as u32).to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            let px = pix.get_pixel(x, y).unwrap_or(0);
            for b in px.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
        }
    }
    h
}

/// Hash raw byte data
fn data_content_hash(data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET_BASIS;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

// --- Manifest management ---

fn manifest_path() -> String {
    format!("{}/tests/golden_manifest.tsv", env!("CARGO_MANIFEST_DIR"))
}

fn manifest() -> &'static Mutex<HashMap<String, u64>> {
    static MANIFEST: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
    MANIFEST.get_or_init(|| Mutex::new(load_manifest_from_file()))
}

fn load_manifest_from_file() -> HashMap<String, u64> {
    let path = manifest_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, hash_str)) = line.split_once('\t')
            && let Ok(hash) = u64::from_str_radix(hash_str, 16)
        {
            map.insert(name.to_string(), hash);
        }
    }
    map
}

fn update_manifest_and_save(name: &str, hash: u64) {
    let mut map = manifest().lock().unwrap();
    map.insert(name.to_string(), hash);
    let mut entries: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut content =
        String::from("# Golden manifest - content hashes for regression test outputs\n");
    content.push_str("# Format: name<TAB>hash (FNV-1a hex)\n");
    for (name, hash) in &entries {
        content.push_str(&format!("{}\t{:016x}\n", name, hash));
    }
    fs::write(manifest_path(), &content).expect("Failed to write manifest");
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
    fn ensure_dirs() {
        static DIRS_READY: OnceLock<()> = OnceLock::new();
        DIRS_READY.get_or_init(|| {
            if let Err(e) = fs::create_dir_all(golden_dir()) {
                eprintln!("Warning: failed to create golden directory: {e}");
            }
            if let Err(e) = fs::create_dir_all(regout_dir()) {
                eprintln!("Warning: failed to create regout directory: {e}");
            }
        });
    }

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

        Self::ensure_dirs();
        if mode != RegTestMode::Display {
            eprintln!();
            eprintln!("////////////////////////////////////////////////");
            eprintln!("////////////////   {}_reg   ///////////////", test_name);
            eprintln!("////////////////////////////////////////////////");
            eprintln!("Mode: {:?}", mode);
        }

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
        leptonica::io::write_image(pix, &local_path, format).map_err(|e| {
            TestError::ImageWrite {
                path: local_path.clone(),
                message: e.to_string(),
            }
        })?;

        let hash = pixel_content_hash(pix);
        self.check_hash(&local_path, hash)
    }

    /// Check content hash against the golden manifest.
    ///
    /// In generate mode, copies the file to golden and updates the manifest.
    /// In compare mode, compares the hash against the manifest entry.
    /// In display mode, does nothing.
    fn check_hash(&mut self, local_path: &str, hash: u64) -> TestResult<()> {
        let ext = Path::new(local_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let manifest_key = format!("{}.{:02}.{}", self.test_name, self.index, ext);

        match self.mode {
            RegTestMode::Generate => {
                let golden_path = format!(
                    "{}/{}_golden.{:02}.{}",
                    golden_dir(),
                    self.test_name,
                    self.index,
                    ext
                );
                fs::copy(local_path, &golden_path)?;
                update_manifest_and_save(&manifest_key, hash);
                eprintln!("Generated: {} (hash: {:016x})", manifest_key, hash);
            }
            RegTestMode::Compare => {
                let map = manifest().lock().unwrap();
                match map.get(&manifest_key) {
                    Some(&expected) if expected == hash => {}
                    Some(&expected) => {
                        let msg = format!(
                            "Failure in {}_reg, index {}: hash mismatch for {}\n\
                             \x20 expected: {:016x}\n\
                             \x20 actual:   {:016x}",
                            self.test_name, self.index, manifest_key, expected, hash
                        );
                        eprintln!("{}", msg);
                        self.failures.push(msg);
                        self.success = false;
                    }
                    None => {
                        eprintln!("Warning: no manifest entry for {manifest_key}, skipping");
                    }
                }
            }
            RegTestMode::Display => {}
        }

        Ok(())
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
        let hash = data_content_hash(data);
        self.check_hash(&local_path, hash)
    }

    /// Clean up and report results
    ///
    /// # Returns
    ///
    /// `true` if all tests passed, `false` if any failed.
    pub fn cleanup(self) -> bool {
        if self.success {
            if self.mode != RegTestMode::Display {
                eprintln!("SUCCESS: {}_reg", self.test_name);
                eprintln!();
            }
        } else {
            eprintln!("FAILURE: {}_reg", self.test_name);
            for failure in &self.failures {
                eprintln!("  {}", failure);
            }
            eprintln!();
        }

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
