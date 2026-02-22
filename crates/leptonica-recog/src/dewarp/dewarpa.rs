//! Dewarpa - multi-page dewarp container
//!
//! [`Dewarpa`] manages a sparse array of [`Dewarp`] models, one per page,
//! together with shared curvature and configuration settings.

use std::io::{Read, Write};

use crate::error::{RecogError, RecogResult};

use super::types::{Dewarp, DewarpOptions};

/// Multi-page dewarp container.
///
/// Stores one optional [`Dewarp`] model per page index and exposes shared
/// configuration for curvature limits and reference-model insertion.
#[derive(Debug)]
pub struct Dewarpa {
    /// Sparse array of per-page models (`None` = no model for that page).
    dewarp_array: Vec<Option<Dewarp>>,

    /// Maximum number of pages this container can hold.
    pub max_pages: usize,

    // Sampling/reduction settings (used when creating new child Dewarp models).
    /// Sampling interval forwarded to child Dewarp models.
    pub sampling: u32,
    /// Reduction factor forwarded to child Dewarp models.
    pub reduction_factor: u32,

    // Curvature and validation limits
    /// Maximum acceptable line curvature (micro-units).
    pub max_linecurv: i32,
    /// Minimum number of text lines required per page.
    pub min_lines: u32,
    /// Maximum acceptable edge slope (milli-units).
    pub max_edgeslope: i32,
    /// Maximum acceptable edge curvature (micro-units).
    pub max_edgecurv: i32,
    /// Maximum allowed difference between adjacent page parameters.
    pub max_diff: u32,

    /// When `true`, both vertical and horizontal disparity are applied.
    pub use_both: bool,
    /// When `true`, enables column-validity checking.
    pub check_columns: bool,

    /// Maximum page distance for borrowing a reference model.
    pub max_dist: u32,
}

impl Dewarpa {
    /// Creates a new `Dewarpa` for up to `max_pages` pages.
    ///
    /// # Arguments
    ///
    /// * `max_pages` - Maximum number of pages
    /// * `sampling` - Sampling interval for child Dewarp models (≥ 8)
    /// * `reduction_factor` - Reduction factor for child models (1 or 2)
    /// * `min_lines` - Minimum text lines required for a valid model
    /// * `max_dist` - Maximum page distance for borrowing a reference model
    pub fn new(
        max_pages: usize,
        sampling: u32,
        reduction_factor: u32,
        min_lines: u32,
        max_dist: u32,
    ) -> Self {
        let cap = max_pages;
        Self {
            dewarp_array: (0..cap).map(|_| None).collect(),
            max_pages,
            sampling,
            reduction_factor,
            max_linecurv: 150,
            min_lines,
            max_edgeslope: 80,
            max_edgecurv: 50,
            max_diff: 100,
            use_both: true,
            check_columns: false,
            max_dist,
        }
    }

    /// Inserts a `Dewarp` model at its page index.
    ///
    /// If `dewarp.page_number` is ≥ `max_pages`, the container is extended.
    ///
    /// # Arguments
    ///
    /// * `dewarp` - Model to insert
    ///
    /// # Errors
    ///
    /// Returns an error if the page index overflows `usize`.
    pub fn insert(&mut self, dewarp: Dewarp) -> RecogResult<()> {
        let page = dewarp.page_number() as usize;
        if page >= self.dewarp_array.len() {
            let new_len = page.checked_add(1).ok_or_else(|| {
                RecogError::InvalidParameter("page index overflows usize".to_string())
            })?;
            self.dewarp_array.resize_with(new_len, || None);
            self.max_pages = self.dewarp_array.len();
        }
        self.dewarp_array[page] = Some(dewarp);
        Ok(())
    }

    /// Returns a reference to the `Dewarp` model for `page`, if any.
    ///
    /// # Arguments
    ///
    /// * `page` - 0-indexed page number
    pub fn get(&self, page: usize) -> Option<&Dewarp> {
        self.dewarp_array.get(page)?.as_ref()
    }

    /// Removes the `Dewarp` model for `page`, if present.
    ///
    /// # Arguments
    ///
    /// * `page` - 0-indexed page number
    pub fn destroy_dewarp(&mut self, page: usize) {
        if let Some(slot) = self.dewarp_array.get_mut(page) {
            *slot = None;
        }
    }

    /// Returns the number of pages that have a model.
    pub fn model_count(&self) -> usize {
        self.dewarp_array.iter().filter(|s| s.is_some()).count()
    }

    /// Sets curvature and slope limits for model validation.
    ///
    /// # Arguments
    ///
    /// * `max_linecurv` - Maximum line curvature (micro-units)
    /// * `min_lines` - Minimum text lines required
    /// * `max_edgeslope` - Maximum edge slope (milli-units)
    /// * `max_edgecurv` - Maximum edge curvature (micro-units)
    /// * `max_diff` - Maximum allowed parameter difference between adjacent pages
    pub fn set_curvatures(
        &mut self,
        max_linecurv: i32,
        min_lines: u32,
        max_edgeslope: i32,
        max_edgecurv: i32,
        max_diff: u32,
    ) {
        self.max_linecurv = max_linecurv;
        self.min_lines = min_lines;
        self.max_edgeslope = max_edgeslope;
        self.max_edgecurv = max_edgecurv;
        self.max_diff = max_diff;
    }

    /// Sets whether both vertical and horizontal disparity are applied.
    ///
    /// # Arguments
    ///
    /// * `use_both` - `true` to apply both, `false` for vertical only
    pub fn use_both_arrays(&mut self, use_both: bool) {
        self.use_both = use_both;
    }

    /// Sets whether column-validity checking is enabled.
    ///
    /// # Arguments
    ///
    /// * `check` - `true` to enable column checking
    pub fn set_check_columns(&mut self, check: bool) {
        self.check_columns = check;
    }

    /// Sets the maximum page distance for borrowing a reference model.
    ///
    /// # Arguments
    ///
    /// * `max_dist` - Maximum distance in pages
    pub fn set_max_distance(&mut self, max_dist: u32) {
        self.max_dist = max_dist;
    }

    /// Returns a [`DewarpOptions`] reflecting the container's sampling settings.
    pub fn dewarp_options(&self) -> DewarpOptions {
        DewarpOptions::default()
            .with_sampling(self.sampling)
            .with_reduction_factor(self.reduction_factor)
            .with_min_lines(self.min_lines)
            .with_use_both(self.use_both)
    }

    /// Fill empty page slots by inserting reference models pointing to the nearest valid page.
    ///
    /// For each page without a model, finds the nearest page with a valid non-reference model
    /// within `max_dist` pages, and inserts a reference Dewarp pointing to it.
    ///
    /// # Arguments
    ///
    /// * `use_both` - If `true`, reference models use both V and H disparity
    ///
    /// # Returns
    ///
    /// The number of reference models inserted.
    ///
    /// # Errors
    ///
    /// Returns an error if model insertion fails.
    pub fn insert_ref_models(&mut self, _use_both: bool) -> RecogResult<u32> {
        let n = self.dewarp_array.len();
        let max_d = self.max_dist as usize;

        // Collect pages that need a reference model
        let missing: Vec<usize> = (0..n).filter(|&i| self.dewarp_array[i].is_none()).collect();

        let mut count = 0u32;
        for page in missing {
            // Find nearest real (non-ref, v_success) model within max_dist
            let mut best: Option<usize> = None;
            let mut best_dist = usize::MAX;
            for ref_page in 0..n {
                let is_valid = self.dewarp_array[ref_page]
                    .as_ref()
                    .is_some_and(|d| !d.is_ref() && d.v_success);
                if is_valid {
                    let dist = ref_page.abs_diff(page);
                    if dist <= max_d && dist < best_dist {
                        best = Some(ref_page);
                        best_dist = dist;
                    }
                }
            }
            if let Some(ref_page) = best {
                self.dewarp_array[page] = Some(Dewarp::create_ref(page as u32, ref_page as u32));
                count = count.saturating_add(1);
            }
        }
        Ok(count)
    }

    /// Apply a single page's model to all other pages via reference models.
    ///
    /// Replaces every slot (except `page`) with a reference model pointing to `page`.
    ///
    /// # Arguments
    ///
    /// * `page` - The page whose model to use
    /// * `use_both` - If `true`, apply both V and H disparity
    ///
    /// # Errors
    ///
    /// Returns an error if `page` has no model.
    pub fn use_single_model(&mut self, page: usize, _use_both: bool) -> RecogResult<()> {
        if self
            .dewarp_array
            .get(page)
            .and_then(|s| s.as_ref())
            .is_none()
        {
            return Err(RecogError::InvalidParameter(format!(
                "page {page} has no Dewarp model"
            )));
        }
        let n = self.dewarp_array.len();
        let page_u32 = u32::try_from(page)
            .map_err(|_| RecogError::InvalidParameter("page index overflows u32".to_string()))?;
        for i in 0..n {
            if i != page {
                let i_u32 = u32::try_from(i).map_err(|_| {
                    RecogError::InvalidParameter("page index overflows u32".to_string())
                })?;
                self.dewarp_array[i] = Some(Dewarp::create_ref(i_u32, page_u32));
            }
        }
        Ok(())
    }

    /// Swap the Dewarp models at two page positions.
    ///
    /// # Arguments
    ///
    /// * `page1` - First page index
    /// * `page2` - Second page index
    ///
    /// # Errors
    ///
    /// Returns an error if either index is out of bounds.
    pub fn swap_pages(&mut self, page1: usize, page2: usize) -> RecogResult<()> {
        let n = self.dewarp_array.len();
        if page1 >= n {
            return Err(RecogError::InvalidParameter(format!(
                "page1 index {page1} out of bounds (len={n})"
            )));
        }
        if page2 >= n {
            return Err(RecogError::InvalidParameter(format!(
                "page2 index {page2} out of bounds (len={n})"
            )));
        }
        self.dewarp_array.swap(page1, page2);
        Ok(())
    }

    /// Remove all reference models from the container.
    ///
    /// Non-reference models are kept unchanged.
    pub fn strip_ref_models(&mut self) {
        for slot in self.dewarp_array.iter_mut() {
            if slot.as_ref().is_some_and(|d| d.is_ref()) {
                *slot = None;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// I/O
// ---------------------------------------------------------------------------

/// Magic bytes that identify a Dewarpa binary file.
const MAGIC: &[u8; 8] = b"DEWARPA\x01";

impl Dewarpa {
    /// Serializes this container to `writer` in binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `writer` fails.
    pub fn write<W: Write>(&self, mut writer: W) -> RecogResult<()> {
        writer
            .write_all(MAGIC)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        let max_pages = u32::try_from(self.max_pages)
            .map_err(|_| RecogError::InvalidParameter("max_pages exceeds u32::MAX".to_string()))?;
        writer
            .write_all(&max_pages.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.sampling.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.reduction_factor.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_linecurv.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.min_lines.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_edgeslope.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_edgecurv.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_diff.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        writer
            .write_all(&self.max_dist.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        let flags: u8 = (self.use_both as u8) | ((self.check_columns as u8) << 1);
        writer
            .write_all(&[flags])
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // Number of Dewarp entries that follow
        let n_entries = self.dewarp_array.iter().filter(|s| s.is_some()).count();
        let n_entries_u32 = u32::try_from(n_entries).map_err(|_| {
            RecogError::InvalidParameter("entry count exceeds u32::MAX".to_string())
        })?;
        writer
            .write_all(&n_entries_u32.to_le_bytes())
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;

        // Write each present Dewarp
        for dw in self.dewarp_array.iter().flatten() {
            dw.write(&mut writer)?;
        }

        Ok(())
    }

    /// Deserializes a `Dewarpa` from `reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or reading fails.
    pub fn read<R: Read>(mut reader: R) -> RecogResult<Dewarpa> {
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        if &magic != MAGIC {
            return Err(RecogError::InvalidParameter(
                "invalid Dewarpa magic bytes".to_string(),
            ));
        }

        let max_pages = read_u32(&mut reader)? as usize;
        let sampling = read_u32(&mut reader)?;
        let reduction_factor = read_u32(&mut reader)?;
        let max_linecurv = read_i32(&mut reader)?;
        let min_lines = read_u32(&mut reader)?;
        let max_edgeslope = read_i32(&mut reader)?;
        let max_edgecurv = read_i32(&mut reader)?;
        let max_diff = read_u32(&mut reader)?;
        let max_dist = read_u32(&mut reader)?;

        let mut flags_buf = [0u8; 1];
        reader
            .read_exact(&mut flags_buf)
            .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        let flags = flags_buf[0];
        if flags & !0x03 != 0 {
            return Err(RecogError::InvalidParameter(format!(
                "reserved bits set in flags byte: {flags:#04x}"
            )));
        }
        let use_both = (flags & 1) != 0;
        let check_columns = (flags & 2) != 0;

        // Guard against unreasonable max_pages values from input
        const MAX_PAGES: usize = 100_000;
        if max_pages > MAX_PAGES {
            return Err(RecogError::InvalidParameter(format!(
                "Dewarpa max_pages {max_pages} exceeds maximum {MAX_PAGES}"
            )));
        }

        let n_entries = read_u32(&mut reader)? as usize;
        // Guard against unreasonable entry counts
        if n_entries > MAX_PAGES {
            return Err(RecogError::InvalidParameter(format!(
                "Dewarpa entry count {n_entries} exceeds maximum {MAX_PAGES}"
            )));
        }

        let mut dewarp_array: Vec<Option<Dewarp>> = (0..max_pages).map(|_| None).collect();

        for _ in 0..n_entries {
            let dw = Dewarp::read(&mut reader)?;
            let page = dw.page_number() as usize;
            // Reject entries outside the declared page range
            if page >= max_pages {
                return Err(RecogError::InvalidParameter(format!(
                    "Dewarpa entry has page index {page} outside allowed range [0, {max_pages})"
                )));
            }
            dewarp_array[page] = Some(dw);
        }

        Ok(Dewarpa {
            dewarp_array,
            max_pages,
            sampling,
            reduction_factor,
            max_linecurv,
            min_lines,
            max_edgeslope,
            max_edgecurv,
            max_diff,
            use_both,
            check_columns,
            max_dist,
        })
    }

    /// Writes this container to a file at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - Destination file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or writing fails.
    pub fn write_to_file(&self, path: &std::path::Path) -> RecogResult<()> {
        let file =
            std::fs::File::create(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        self.write(std::io::BufWriter::new(file))
    }

    /// Reads a `Dewarpa` from a file at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - Source file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsing fails.
    pub fn read_from_file(path: &std::path::Path) -> RecogResult<Dewarpa> {
        let file =
            std::fs::File::open(path).map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
        Dewarpa::read(std::io::BufReader::new(file))
    }
}

fn read_u32<R: Read>(reader: &mut R) -> RecogResult<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32<R: Read>(reader: &mut R) -> RecogResult<i32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| RecogError::InvalidParameter(e.to_string()))?;
    Ok(i32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dewarpa(max_pages: usize) -> Dewarpa {
        Dewarpa::new(max_pages, 30, 1, 15, 5)
    }

    fn make_dewarp(page: u32) -> Dewarp {
        let opts = DewarpOptions::default();
        let mut d = Dewarp::new(800, 600, page, &opts);
        d.v_success = true;
        d
    }

    #[test]
    fn test_insert_and_get() {
        let mut da = make_dewarpa(10);
        let dw = make_dewarp(3);
        da.insert(dw).unwrap();
        assert!(da.get(3).is_some());
        assert!(da.get(0).is_none());
        assert!(da.get(9).is_none());
    }

    #[test]
    fn test_insert_extends_array() {
        let mut da = make_dewarpa(5);
        let dw = make_dewarp(10);
        da.insert(dw).unwrap();
        assert!(da.get(10).is_some());
        assert!(da.max_pages >= 11);
    }

    #[test]
    fn test_destroy_dewarp() {
        let mut da = make_dewarpa(10);
        da.insert(make_dewarp(2)).unwrap();
        assert!(da.get(2).is_some());
        da.destroy_dewarp(2);
        assert!(da.get(2).is_none());
    }

    #[test]
    fn test_model_count() {
        let mut da = make_dewarpa(10);
        assert_eq!(da.model_count(), 0);
        da.insert(make_dewarp(0)).unwrap();
        da.insert(make_dewarp(5)).unwrap();
        assert_eq!(da.model_count(), 2);
    }

    #[test]
    fn test_set_curvatures() {
        let mut da = make_dewarpa(10);
        da.set_curvatures(200, 10, 60, 40, 50);
        assert_eq!(da.max_linecurv, 200);
        assert_eq!(da.min_lines, 10);
        assert_eq!(da.max_edgeslope, 60);
        assert_eq!(da.max_edgecurv, 40);
        assert_eq!(da.max_diff, 50);
    }

    #[test]
    fn test_use_both_and_check_columns() {
        let mut da = make_dewarpa(10);
        da.use_both_arrays(false);
        assert!(!da.use_both);
        da.set_check_columns(true);
        assert!(da.check_columns);
        da.set_max_distance(3);
        assert_eq!(da.max_dist, 3);
    }

    #[test]
    fn test_write_read_roundtrip_empty() {
        let da = make_dewarpa(5);
        let mut buf = Vec::new();
        da.write(&mut buf).unwrap();
        let da2 = Dewarpa::read(buf.as_slice()).unwrap();
        assert_eq!(da2.max_pages, 5);
        assert_eq!(da2.model_count(), 0);
    }

    #[test]
    fn test_write_read_roundtrip_with_models() {
        let mut da = make_dewarpa(10);
        da.insert(make_dewarp(1)).unwrap();
        da.insert(make_dewarp(4)).unwrap();
        da.set_curvatures(200, 8, 70, 60, 80);
        da.use_both_arrays(false);
        da.set_check_columns(true);

        let mut buf = Vec::new();
        da.write(&mut buf).unwrap();
        let da2 = Dewarpa::read(buf.as_slice()).unwrap();

        assert_eq!(da2.model_count(), 2);
        assert!(da2.get(1).is_some());
        assert!(da2.get(4).is_some());
        assert!(da2.get(0).is_none());
        assert_eq!(da2.max_linecurv, 200);
        assert!(!da2.use_both);
        assert!(da2.check_columns);
    }

    #[test]
    fn test_invalid_magic() {
        let bad = b"BAD_MAGIC_BYTES";
        let result = Dewarpa::read(bad.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_ref_models() {
        let mut da = make_dewarpa(5);
        let mut dw = make_dewarp(2);
        dw.v_success = true;
        da.insert(dw).unwrap();
        let n = da.insert_ref_models(false).unwrap();
        // Pages 0, 1, 3, 4 are within max_dist=5 of page 2 and should get ref models
        assert!(n > 0);
        let p0 = da.get(0);
        let p2 = da.get(2);
        assert!(p0.is_some_and(|d| d.is_ref()));
        assert!(p2.is_some_and(|d| !d.is_ref()));
    }

    #[test]
    fn test_use_single_model() {
        let mut da = make_dewarpa(5);
        da.insert(make_dewarp(2)).unwrap();
        da.use_single_model(2, false).unwrap();
        // Page 2 is the real model; all others are refs pointing to page 2
        for i in 0..5 {
            let m = da.get(i);
            if i == 2 {
                assert!(m.is_some_and(|d| !d.is_ref()));
            } else {
                assert!(m.is_some_and(|d| d.is_ref() && d.ref_page() == Some(2)));
            }
        }
    }

    #[test]
    fn test_use_single_model_no_model() {
        let mut da = make_dewarpa(5);
        assert!(da.use_single_model(2, false).is_err());
    }

    #[test]
    fn test_swap_pages() {
        let mut da = make_dewarpa(5);
        da.insert(make_dewarp(1)).unwrap();
        da.swap_pages(1, 3).unwrap();
        // After swap: page 1 is empty, page 3 has the model
        assert!(da.get(1).is_none());
        assert!(da.get(3).is_some_and(|d| d.page_number() == 1));
    }

    #[test]
    fn test_swap_pages_out_of_bounds() {
        let mut da = make_dewarpa(5);
        assert!(da.swap_pages(0, 10).is_err());
        assert!(da.swap_pages(10, 0).is_err());
    }

    #[test]
    fn test_strip_ref_models() {
        let mut da = make_dewarpa(5);
        da.insert(make_dewarp(2)).unwrap();
        da.use_single_model(2, false).unwrap();
        // 5 total models: 1 real + 4 refs
        assert_eq!(da.model_count(), 5);
        da.strip_ref_models();
        // Only the real model at page 2 remains
        assert_eq!(da.model_count(), 1);
        assert!(da.get(2).is_some_and(|d| !d.is_ref()));
    }

    #[test]
    fn test_file_roundtrip() {
        let mut da = make_dewarpa(3);
        da.insert(make_dewarp(2)).unwrap();

        let path = std::env::temp_dir().join("dewarpa_test.bin");
        da.write_to_file(&path).unwrap();
        let da2 = Dewarpa::read_from_file(&path).unwrap();
        assert_eq!(da2.model_count(), 1);
        assert!(da2.get(2).is_some());
        let _ = std::fs::remove_file(&path);
    }
}
