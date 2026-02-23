//! Type definitions for JBIG2 classification
//!
//! This module contains the core data structures for JBIG2-style
//! connected component classification and template compression.

use leptonica_core::Pix;
use std::collections::HashMap;

/// Classification method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JbMethod {
    /// Rank Hausdorff distance
    #[default]
    RankHaus = 0,
    /// Correlation-based matching
    Correlation = 1,
}

/// Component type to extract from images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JbComponent {
    /// Connected components (basic)
    #[default]
    ConnComps = 0,
    /// Characters (filtered components)
    Characters = 1,
    /// Words (grouped components)
    Words = 2,
}

/// JBIG2 classifier
///
/// This structure holds all the data accumulated during the classification
/// process that can be used for a compressed JBIG2-type representation
/// of a set of images.
#[derive(Debug)]
pub struct JbClasser {
    /// Input page image file names
    pub files: Vec<String>,

    /// Classification method (RankHaus or Correlation)
    pub method: JbMethod,

    /// Component type (ConnComps, Characters, or Words)
    pub components: JbComponent,

    /// Maximum component width allowed
    pub max_width: i32,

    /// Maximum component height allowed
    pub max_height: i32,

    /// Number of pages already processed
    pub npages: usize,

    /// Number of components already processed on fully processed pages
    pub base_index: usize,

    /// Number of components on each page
    pub nacomps: Vec<usize>,

    // Hausdorff parameters
    /// Size of square structuring element for Hausdorff
    pub size_haus: i32,

    /// Rank value of Hausdorff match, each way
    pub rank_haus: f32,

    // Correlation parameters
    /// Threshold value for correlation score
    pub thresh: f32,

    /// Weight factor to correct threshold for heavier components
    pub weight_factor: f32,

    // Template information
    /// Area (w * h) of each template, without extra border pixels
    pub naarea: Vec<i32>,

    /// Maximum width of original source images
    pub w: i32,

    /// Maximum height of original source images
    pub h: i32,

    /// Current number of classes
    pub nclass: usize,

    // Template images
    /// If true, pixaa is filled with instances for each class
    pub keep_pixaa: bool,

    /// Instances for each class (unbordered)
    pub pixaa: Vec<Vec<Pix>>,

    /// Templates for each class (bordered, not dilated)
    pub pixat: Vec<Pix>,

    /// Templates for each class (bordered and dilated)
    pub pixatd: Vec<Pix>,

    /// Hash table to find templates by size (width, height) -> template indices
    pub dahash: HashMap<(i32, i32), Vec<usize>>,

    /// Foreground areas of undilated templates (only used for rank < 1.0)
    pub nafgt: Vec<i32>,

    /// Centroids of all bordered connected components
    pub ptac: Vec<(f32, f32)>,

    /// Centroids of all bordered template connected components
    pub ptact: Vec<(f32, f32)>,

    // Classification results
    /// Class ID for each component
    pub naclass: Vec<usize>,

    /// Page number for each component
    pub napage: Vec<usize>,

    /// Upper-left corner positions for placing templates
    pub ptaul: Vec<(i32, i32)>,

    /// Lower-left corner positions for placing templates
    pub ptall: Vec<(i32, i32)>,
}

impl JbClasser {
    /// Creates a new JbClasser with default values
    pub fn new(method: JbMethod, components: JbComponent) -> Self {
        Self {
            files: Vec::new(),
            method,
            components,
            max_width: 150,
            max_height: 150,
            npages: 0,
            base_index: 0,
            nacomps: Vec::new(),
            size_haus: 2,
            rank_haus: 0.97,
            thresh: 0.85,
            weight_factor: 0.0,
            naarea: Vec::new(),
            w: 0,
            h: 0,
            nclass: 0,
            keep_pixaa: false,
            pixaa: Vec::new(),
            pixat: Vec::new(),
            pixatd: Vec::new(),
            dahash: HashMap::new(),
            nafgt: Vec::new(),
            ptac: Vec::new(),
            ptact: Vec::new(),
            naclass: Vec::new(),
            napage: Vec::new(),
            ptaul: Vec::new(),
            ptall: Vec::new(),
        }
    }

    /// Returns the total number of components classified
    pub fn total_components(&self) -> usize {
        self.naclass.len()
    }

    /// Returns the number of unique classes
    pub fn num_classes(&self) -> usize {
        self.nclass
    }

    /// Clears all classification data
    pub fn clear(&mut self) {
        self.files.clear();
        self.npages = 0;
        self.base_index = 0;
        self.nacomps.clear();
        self.naarea.clear();
        self.w = 0;
        self.h = 0;
        self.nclass = 0;
        self.pixaa.clear();
        self.pixat.clear();
        self.pixatd.clear();
        self.dahash.clear();
        self.nafgt.clear();
        self.ptac.clear();
        self.ptact.clear();
        self.naclass.clear();
        self.napage.clear();
        self.ptaul.clear();
        self.ptall.clear();
    }
}

/// JBIG2 compressed data
///
/// This structure holds all the data required for the compressed
/// JBIG2-type representation of a set of images.
#[derive(Debug)]
pub struct JbData {
    /// Template composite image for all classes
    pub pix: Pix,

    /// Number of pages
    pub npages: usize,

    /// Maximum width of original page images
    pub w: i32,

    /// Maximum height of original page images
    pub h: i32,

    /// Number of classes (unique templates)
    pub nclass: usize,

    /// Lattice width for template composite
    pub lattice_w: i32,

    /// Lattice height for template composite
    pub lattice_h: i32,

    /// Class ID for each component
    pub naclass: Vec<usize>,

    /// Page number for each component
    pub napage: Vec<usize>,

    /// Upper-left corner positions for placing templates
    pub ptaul: Vec<(i32, i32)>,
}

impl JbData {
    /// Creates new JbData from a classer
    pub fn from_classer(classer: &JbClasser, pix: Pix, lattice_w: i32, lattice_h: i32) -> Self {
        Self {
            pix,
            npages: classer.npages,
            w: classer.w,
            h: classer.h,
            nclass: classer.nclass,
            lattice_w,
            lattice_h,
            naclass: classer.naclass.clone(),
            napage: classer.napage.clone(),
            ptaul: classer.ptaul.clone(),
        }
    }

    /// Returns the total number of components
    pub fn num_components(&self) -> usize {
        self.naclass.len()
    }
}

/// Default maximum component width
pub const DEFAULT_MAX_WIDTH: i32 = 150;

/// Default maximum component height
pub const DEFAULT_MAX_HEIGHT: i32 = 150;

/// Default Hausdorff structuring element size
pub const DEFAULT_SIZE_HAUS: i32 = 2;

/// Default Hausdorff rank value
pub const DEFAULT_RANK_HAUS: f32 = 0.97;

/// Default correlation threshold
pub const DEFAULT_THRESH: f32 = 0.85;

/// Default weight factor for correlation
pub const DEFAULT_WEIGHT_FACTOR: f32 = 0.0;

/// Border size for templates
pub const TEMPLATE_BORDER: i32 = 4;

/// Template file extension
pub const JB_TEMPLATE_EXT: &str = ".templates.png";

/// Data file extension
pub const JB_DATA_EXT: &str = ".data";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jb_method_default() {
        assert_eq!(JbMethod::default(), JbMethod::RankHaus);
    }

    #[test]
    fn test_jb_component_default() {
        assert_eq!(JbComponent::default(), JbComponent::ConnComps);
    }

    #[test]
    fn test_jbclasser_new() {
        let classer = JbClasser::new(JbMethod::Correlation, JbComponent::Characters);
        assert_eq!(classer.method, JbMethod::Correlation);
        assert_eq!(classer.components, JbComponent::Characters);
        assert_eq!(classer.nclass, 0);
        assert_eq!(classer.npages, 0);
    }

    #[test]
    fn test_jbclasser_total_components() {
        let mut classer = JbClasser::new(JbMethod::RankHaus, JbComponent::ConnComps);
        assert_eq!(classer.total_components(), 0);

        classer.naclass.push(0);
        classer.naclass.push(1);
        classer.naclass.push(0);
        assert_eq!(classer.total_components(), 3);
    }

    #[test]
    fn test_jbclasser_clear() {
        let mut classer = JbClasser::new(JbMethod::RankHaus, JbComponent::ConnComps);
        classer.naclass.push(0);
        classer.npages = 5;
        classer.nclass = 10;

        classer.clear();

        assert_eq!(classer.total_components(), 0);
        assert_eq!(classer.npages, 0);
        assert_eq!(classer.nclass, 0);
    }
}
