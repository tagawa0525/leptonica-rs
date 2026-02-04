//! Type definitions for character recognition
//!
//! This module contains the core data structures for template-based character recognition.

use leptonica_core::{Box as PixBox, Pix};

/// Character set type for recognition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetType {
    /// Character set type is not specified
    #[default]
    Unknown = 0,
    /// Arabic numerals: 0-9 (10 characters)
    ArabicNumerals = 1,
    /// Lowercase Roman numerals: i, v, x, l, c, d, m (7 characters)
    LcRomanNumerals = 2,
    /// Uppercase Roman numerals: I, V, X, L, C, D, M (7 characters)
    UcRomanNumerals = 3,
    /// Lowercase letters: a-z (26 characters)
    LcAlpha = 4,
    /// Uppercase letters: A-Z (26 characters)
    UcAlpha = 5,
}

impl CharsetType {
    /// Returns the expected number of characters in this charset
    pub fn expected_size(&self) -> usize {
        match self {
            CharsetType::Unknown => 0,
            CharsetType::ArabicNumerals => 10,
            CharsetType::LcRomanNumerals => 7,
            CharsetType::UcRomanNumerals => 7,
            CharsetType::LcAlpha => 26,
            CharsetType::UcAlpha => 26,
        }
    }

    /// Returns the characters in this charset
    pub fn characters(&self) -> &'static [char] {
        match self {
            CharsetType::Unknown => &[],
            CharsetType::ArabicNumerals => &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
            CharsetType::LcRomanNumerals => &['i', 'v', 'x', 'l', 'c', 'd', 'm'],
            CharsetType::UcRomanNumerals => &['I', 'V', 'X', 'L', 'C', 'D', 'M'],
            CharsetType::LcAlpha => &[
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ],
            CharsetType::UcAlpha => &[
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ],
        }
    }
}

/// Template usage mode for recognition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TemplateUse {
    /// Use all templates for matching (default)
    #[default]
    All = 0,
    /// Use averaged templates for matching (special cases)
    Average = 1,
}

/// Character recognizer
///
/// This structure holds all the data needed for training and recognizing
/// individual machine-printed text characters.
#[derive(Debug)]
pub struct Recog {
    // Scaling parameters
    /// Horizontal scale target (0 = no horizontal scaling)
    pub scale_w: i32,
    /// Vertical scale target (0 = no vertical scaling)
    pub scale_h: i32,
    /// Line width for skeleton-based recognition (0 = skip)
    pub line_w: i32,

    // Template settings
    /// Template usage mode (all or average)
    pub templ_use: TemplateUse,
    /// Maximum array size for containers
    pub max_array_size: usize,
    /// Number of character classes
    pub set_size: usize,

    // Identification parameters
    /// Binarization threshold (for depth > 1)
    pub threshold: i32,
    /// Maximum vertical shift allowed during matching (typically 0 or 1)
    pub max_y_shift: i32,

    // Character set
    /// Type of character set being recognized
    pub charset_type: CharsetType,
    /// Expected number of classes in charset
    pub charset_size: usize,

    // Training statistics
    /// Minimum number of samples without padding
    pub min_nopad: i32,
    /// Total number of training samples
    pub num_samples: usize,

    // Template size info (unscaled)
    /// Minimum width of unscaled templates
    pub minwidth_u: i32,
    /// Maximum width of unscaled templates
    pub maxwidth_u: i32,
    /// Minimum height of unscaled templates
    pub minheight_u: i32,
    /// Maximum height of unscaled templates
    pub maxheight_u: i32,

    // Template size info (scaled)
    /// Minimum width of scaled templates
    pub minwidth: i32,
    /// Maximum width of scaled templates
    pub maxwidth: i32,

    // State flags
    /// Whether averaged bitmaps have been computed
    pub ave_done: bool,
    /// Whether training is complete
    pub train_done: bool,

    // Splitting parameters
    /// Maximum width/height ratio for splitting
    pub max_wh_ratio: f32,
    /// Maximum template height ratio
    pub max_ht_ratio: f32,
    /// Minimum component width kept in splitting
    pub min_split_w: i32,
    /// Maximum component height kept in splitting
    pub max_split_h: i32,

    // Text mapping (for arbitrary character sets)
    /// Character strings for each class
    pub sa_text: Vec<String>,
    /// Index to character lookup table
    pub dna_tochar: Vec<f64>,

    // Lookup tables
    /// Table for finding centroids (256 entries)
    pub(crate) centtab: Vec<i32>,
    /// Table for finding pixel sums (256 entries)
    pub(crate) sumtab: Vec<i32>,

    // Templates (unscaled)
    /// All unscaled templates for each class
    pub pixaa_u: Vec<Vec<Pix>>,
    /// Centroids of all unscaled templates
    pub ptaa_u: Vec<Vec<(f32, f32)>>,
    /// Area of all unscaled templates
    pub naasum_u: Vec<Vec<i32>>,

    // Templates (scaled)
    /// All scaled templates for each class
    pub pixaa: Vec<Vec<Pix>>,
    /// Centroids of all scaled templates
    pub ptaa: Vec<Vec<(f32, f32)>>,
    /// Area of all scaled templates
    pub naasum: Vec<Vec<i32>>,

    // Averaged templates
    /// Averaged unscaled templates per class
    pub pixa_u: Vec<Pix>,
    /// Centroids of unscaled averaged templates
    pub pta_u: Vec<(f32, f32)>,
    /// Area of unscaled averaged templates
    pub nasum_u: Vec<i32>,

    /// Averaged scaled templates per class
    pub pixa: Vec<Pix>,
    /// Centroids of scaled averaged templates
    pub pta: Vec<(f32, f32)>,
    /// Area of scaled averaged templates
    pub nasum: Vec<i32>,

    // Debug/working data
    /// All input training images
    pub pixa_tr: Vec<Pix>,

    // Temp data (computed during identification)
    /// DID state for decoding
    pub(crate) did: Option<Rdid>,
    /// Best character result (reserved for future use)
    #[allow(dead_code)]
    pub(crate) rch: Option<Rch>,
    /// Array of best character results (reserved for future use)
    #[allow(dead_code)]
    pub(crate) rcha: Option<Rcha>,
}

/// Recognition result for a single character
#[derive(Debug, Clone)]
pub struct Rch {
    /// Index of best matching template
    pub index: i32,
    /// Correlation score of best match
    pub score: f32,
    /// Character string of best template
    pub text: String,
    /// Index of best sample within the template class
    pub sample: i32,
    /// X-location of template (delx + shiftx)
    pub xloc: i32,
    /// Y-location of template (dely + shifty)
    pub yloc: i32,
    /// Width of best template
    pub width: i32,
}

impl Default for Rch {
    fn default() -> Self {
        Self {
            index: -1,
            score: 0.0,
            text: String::new(),
            sample: 0,
            xloc: 0,
            yloc: 0,
            width: 0,
        }
    }
}

/// Recognition results for an array of characters
#[derive(Debug, Clone, Default)]
pub struct Rcha {
    /// Indices of best templates
    pub indices: Vec<i32>,
    /// Correlation scores of best templates
    pub scores: Vec<f32>,
    /// Character strings of best templates
    pub texts: Vec<String>,
    /// Indices of best samples
    pub samples: Vec<i32>,
    /// X-locations of templates
    pub xlocs: Vec<i32>,
    /// Y-locations of templates
    pub ylocs: Vec<i32>,
    /// Widths of best templates
    pub widths: Vec<i32>,
}

impl Rcha {
    /// Creates a new empty Rcha
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of recognized characters
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns true if no characters were recognized
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Adds a recognition result
    pub fn push(&mut self, rch: &Rch) {
        self.indices.push(rch.index);
        self.scores.push(rch.score);
        self.texts.push(rch.text.clone());
        self.samples.push(rch.sample);
        self.xlocs.push(rch.xloc);
        self.ylocs.push(rch.yloc);
        self.widths.push(rch.width);
    }

    /// Gets a single recognition result at the given index
    pub fn get(&self, i: usize) -> Option<Rch> {
        if i >= self.len() {
            return None;
        }
        Some(Rch {
            index: self.indices[i],
            score: self.scores[i],
            text: self.texts[i].clone(),
            sample: self.samples[i],
            xloc: self.xlocs[i],
            yloc: self.ylocs[i],
            width: self.widths[i],
        })
    }
}

impl std::fmt::Display for Rcha {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.texts.join(""))
    }
}

/// Document Image Decoding data
///
/// This structure holds data used for decoding a line of characters
/// using the Viterbi algorithm.
#[derive(Debug)]
pub struct Rdid {
    /// Clone of image to be decoded
    pub pixs: Pix,

    /// Count array for each averaged template
    pub counta: Vec<Vec<i32>>,
    /// Best y-shift array per averaged template
    pub delya: Vec<Vec<i32>>,

    /// Number of averaged templates
    pub narray: usize,
    /// Size of count array (width of pixs)
    pub size: usize,

    /// Set widths for each template
    pub setwidth: Vec<i32>,
    /// Pixel count in pixs by column
    pub nasum: Vec<i32>,
    /// First moment of pixels in pixs by column
    pub namoment: Vec<i32>,

    /// Whether full arrays have been computed
    pub fullarrays: bool,

    // Viterbi parameters
    /// Channel coefficients for template foreground term
    pub beta: Vec<f32>,
    /// Channel coefficients for bit-and term
    pub gamma: Vec<f32>,
    /// Score on trellis
    pub trellisscore: Vec<f32>,
    /// Template on trellis (for backtracking)
    pub trellistempl: Vec<i32>,

    // Best path results
    /// Indices of best path templates
    pub natempl: Vec<i32>,
    /// X-locations of best path templates
    pub naxloc: Vec<i32>,
    /// Y-shifts of best path templates
    pub nadely: Vec<i32>,
    /// Widths of best path templates
    pub nawidth: Vec<i32>,

    /// Viterbi result for splitting input pixs
    pub boxa: Vec<PixBox>,
    /// Correlation scores of best path templates
    pub nascore: Vec<f32>,

    // Rescored results
    /// Indices of best rescored templates
    pub natempl_r: Vec<i32>,
    /// Samples of best rescored templates
    pub nasample_r: Vec<i32>,
    /// X-locations of best rescored templates
    pub naxloc_r: Vec<i32>,
    /// Y-shifts of best rescored templates
    pub nadely_r: Vec<i32>,
    /// Widths of best rescored templates
    pub nawidth_r: Vec<i32>,
    /// Correlation scores of rescored templates
    pub nascore_r: Vec<f32>,
}

impl Rdid {
    /// Creates a new Rdid for the given image
    pub fn new(pixs: Pix, narray: usize) -> Self {
        let size = pixs.width() as usize;
        Self {
            pixs,
            counta: Vec::with_capacity(narray),
            delya: Vec::with_capacity(narray),
            narray,
            size,
            setwidth: vec![0; narray],
            nasum: vec![0; size],
            namoment: vec![0; size],
            fullarrays: false,
            beta: vec![0.0; narray],
            gamma: vec![0.0; narray],
            trellisscore: vec![0.0; size],
            trellistempl: vec![-1; size],
            natempl: Vec::new(),
            naxloc: Vec::new(),
            nadely: Vec::new(),
            nawidth: Vec::new(),
            boxa: Vec::new(),
            nascore: Vec::new(),
            natempl_r: Vec::new(),
            nasample_r: Vec::new(),
            naxloc_r: Vec::new(),
            nadely_r: Vec::new(),
            nawidth_r: Vec::new(),
            nascore_r: Vec::new(),
        }
    }
}

/// Default initial size for template arrays
pub const DEFAULT_MAX_ARRAY_SIZE: usize = 256;

/// Default binarization threshold
pub const DEFAULT_THRESHOLD: i32 = 150;

/// Default maximum y-shift for correlation
pub const DEFAULT_MAX_Y_SHIFT: i32 = 1;

/// Default maximum width/height ratio for splitting
pub const DEFAULT_MAX_WH_RATIO: f32 = 3.0;

/// Default maximum template height ratio
pub const DEFAULT_MAX_HT_RATIO: f32 = 2.5;

/// Default minimum split width
pub const DEFAULT_MIN_SPLIT_W: i32 = 6;

/// Default maximum split height
pub const DEFAULT_MAX_SPLIT_H: i32 = 60;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_type_expected_size() {
        assert_eq!(CharsetType::Unknown.expected_size(), 0);
        assert_eq!(CharsetType::ArabicNumerals.expected_size(), 10);
        assert_eq!(CharsetType::LcRomanNumerals.expected_size(), 7);
        assert_eq!(CharsetType::UcRomanNumerals.expected_size(), 7);
        assert_eq!(CharsetType::LcAlpha.expected_size(), 26);
        assert_eq!(CharsetType::UcAlpha.expected_size(), 26);
    }

    #[test]
    fn test_charset_type_characters() {
        let digits = CharsetType::ArabicNumerals.characters();
        assert_eq!(digits.len(), 10);
        assert_eq!(digits[0], '0');
        assert_eq!(digits[9], '9');

        let lc_roman = CharsetType::LcRomanNumerals.characters();
        assert_eq!(lc_roman.len(), 7);
        assert!(lc_roman.contains(&'i'));
        assert!(lc_roman.contains(&'m'));
    }

    #[test]
    fn test_rch_default() {
        let rch = Rch::default();
        assert_eq!(rch.index, -1);
        assert_eq!(rch.score, 0.0);
        assert!(rch.text.is_empty());
    }

    #[test]
    fn test_rcha_operations() {
        let mut rcha = Rcha::new();
        assert!(rcha.is_empty());
        assert_eq!(rcha.len(), 0);

        let rch = Rch {
            index: 5,
            score: 0.95,
            text: "A".to_string(),
            sample: 0,
            xloc: 10,
            yloc: 20,
            width: 30,
        };
        rcha.push(&rch);

        assert!(!rcha.is_empty());
        assert_eq!(rcha.len(), 1);

        let retrieved = rcha.get(0).unwrap();
        assert_eq!(retrieved.index, 5);
        assert_eq!(retrieved.score, 0.95);
        assert_eq!(retrieved.text, "A");

        assert!(rcha.get(1).is_none());
    }

    #[test]
    fn test_rcha_to_string() {
        let mut rcha = Rcha::new();
        rcha.texts.push("H".to_string());
        rcha.texts.push("i".to_string());
        rcha.indices.push(0);
        rcha.indices.push(1);
        rcha.scores.push(0.9);
        rcha.scores.push(0.9);
        rcha.samples.push(0);
        rcha.samples.push(0);
        rcha.xlocs.push(0);
        rcha.xlocs.push(0);
        rcha.ylocs.push(0);
        rcha.ylocs.push(0);
        rcha.widths.push(0);
        rcha.widths.push(0);

        assert_eq!(rcha.to_string(), "Hi");
    }
}
