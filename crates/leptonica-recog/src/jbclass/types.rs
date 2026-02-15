//! Type definitions for JBIG2 classification

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
#[derive(Debug)]
pub struct JbClasser {
    /// Input page image file names
    pub files: Vec<String>,
    /// Classification method
    pub method: JbMethod,
    /// Component type
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
    /// Size of square structuring element for Hausdorff
    pub size_haus: i32,
    /// Rank value of Hausdorff match
    pub rank_haus: f32,
    /// Threshold value for correlation score
    pub thresh: f32,
    /// Weight factor for correlation
    pub weight_factor: f32,
    /// Area of each template
    pub naarea: Vec<i32>,
    /// Maximum width of original source images
    pub w: i32,
    /// Maximum height of original source images
    pub h: i32,
    /// Current number of classes
    pub nclass: usize,
    /// If true, pixaa is filled with instances for each class
    pub keep_pixaa: bool,
    /// Instances for each class
    pub pixaa: Vec<Vec<Pix>>,
    /// Templates for each class (bordered, not dilated)
    pub pixat: Vec<Pix>,
    /// Templates for each class (bordered and dilated)
    pub pixatd: Vec<Pix>,
    /// Hash table to find templates by size
    pub dahash: HashMap<(i32, i32), Vec<usize>>,
    /// Foreground areas of undilated templates
    pub nafgt: Vec<i32>,
    /// Centroids of all bordered connected components
    pub ptac: Vec<(f32, f32)>,
    /// Centroids of all bordered template connected components
    pub ptact: Vec<(f32, f32)>,
    /// Class ID for each component
    pub naclass: Vec<usize>,
    /// Page number for each component
    pub napage: Vec<usize>,
    /// Upper-left corner positions
    pub ptaul: Vec<(i32, i32)>,
    /// Lower-left corner positions
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
#[derive(Debug)]
pub struct JbData {
    /// Template composite image
    pub pix: Pix,
    /// Number of pages
    pub npages: usize,
    /// Maximum width of original page images
    pub w: i32,
    /// Maximum height of original page images
    pub h: i32,
    /// Number of classes
    pub nclass: usize,
    /// Lattice width for template composite
    pub lattice_w: i32,
    /// Lattice height for template composite
    pub lattice_h: i32,
    /// Class ID for each component
    pub naclass: Vec<usize>,
    /// Page number for each component
    pub napage: Vec<usize>,
    /// Upper-left corner positions
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
/// Default weight factor
pub const DEFAULT_WEIGHT_FACTOR: f32 = 0.0;
/// Border size for templates
pub const TEMPLATE_BORDER: i32 = 4;
/// Template file extension
pub const JB_TEMPLATE_EXT: &str = ".templates.png";
/// Data file extension
pub const JB_DATA_EXT: &str = ".data";
