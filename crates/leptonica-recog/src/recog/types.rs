//! Type definitions for character recognition

use leptonica_core::{Box as PixBox, Pix};

/// Character set type for recognition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetType {
    /// Not specified
    #[default]
    Unknown = 0,
    /// Arabic numerals: 0-9
    ArabicNumerals = 1,
    /// Lowercase Roman numerals
    LcRomanNumerals = 2,
    /// Uppercase Roman numerals
    UcRomanNumerals = 3,
    /// Lowercase letters: a-z
    LcAlpha = 4,
    /// Uppercase letters: A-Z
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
    /// Use all templates for matching
    #[default]
    All = 0,
    /// Use averaged templates
    Average = 1,
}

/// Character recognizer
#[derive(Debug)]
pub struct Recog {
    /// Horizontal scale target
    pub scale_w: i32,
    /// Vertical scale target
    pub scale_h: i32,
    /// Line width for skeleton-based recognition
    pub line_w: i32,
    /// Template usage mode
    pub templ_use: TemplateUse,
    /// Maximum array size
    pub max_array_size: usize,
    /// Number of character classes
    pub set_size: usize,
    /// Binarization threshold
    pub threshold: i32,
    /// Maximum vertical shift
    pub max_y_shift: i32,
    /// Character set type
    pub charset_type: CharsetType,
    /// Expected charset size
    pub charset_size: usize,
    /// Minimum samples without padding
    pub min_nopad: i32,
    /// Total training samples
    pub num_samples: usize,
    /// Min width unscaled
    pub minwidth_u: i32,
    /// Max width unscaled
    pub maxwidth_u: i32,
    /// Min height unscaled
    pub minheight_u: i32,
    /// Max height unscaled
    pub maxheight_u: i32,
    /// Min width scaled
    pub minwidth: i32,
    /// Max width scaled
    pub maxwidth: i32,
    /// Whether averaged bitmaps computed
    pub ave_done: bool,
    /// Whether training is complete
    pub train_done: bool,
    /// Max width/height ratio for splitting
    pub max_wh_ratio: f32,
    /// Max template height ratio
    pub max_ht_ratio: f32,
    /// Min component width kept in splitting
    pub min_split_w: i32,
    /// Max component height kept in splitting
    pub max_split_h: i32,
    /// Character strings for each class
    pub sa_text: Vec<String>,
    /// Index to character lookup
    pub dna_tochar: Vec<f64>,
    pub(crate) centtab: Vec<i32>,
    pub(crate) sumtab: Vec<i32>,
    /// Unscaled templates per class
    pub pixaa_u: Vec<Vec<Pix>>,
    /// Centroids of unscaled templates
    pub ptaa_u: Vec<Vec<(f32, f32)>>,
    /// Area of unscaled templates
    pub naasum_u: Vec<Vec<i32>>,
    /// Scaled templates per class
    pub pixaa: Vec<Vec<Pix>>,
    /// Centroids of scaled templates
    pub ptaa: Vec<Vec<(f32, f32)>>,
    /// Area of scaled templates
    pub naasum: Vec<Vec<i32>>,
    /// Averaged unscaled templates
    pub pixa_u: Vec<Pix>,
    /// Centroids of unscaled averaged
    pub pta_u: Vec<(f32, f32)>,
    /// Area of unscaled averaged
    pub nasum_u: Vec<i32>,
    /// Averaged scaled templates
    pub pixa: Vec<Pix>,
    /// Centroids of scaled averaged
    pub pta: Vec<(f32, f32)>,
    /// Area of scaled averaged
    pub nasum: Vec<i32>,
    /// All input training images
    pub pixa_tr: Vec<Pix>,
    pub(crate) did: Option<Rdid>,
    #[allow(dead_code)]
    pub(crate) rch: Option<Rch>,
    #[allow(dead_code)]
    pub(crate) rcha: Option<Rcha>,
}

/// Recognition result for a single character
#[derive(Debug, Clone)]
pub struct Rch {
    /// Index of best matching template
    pub index: i32,
    /// Correlation score
    pub score: f32,
    /// Character string
    pub text: String,
    /// Best sample index
    pub sample: i32,
    /// X-location
    pub xloc: i32,
    /// Y-location
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

/// Recognition results for character array
#[derive(Debug, Clone, Default)]
pub struct Rcha {
    /// Indices of best templates
    pub indices: Vec<i32>,
    /// Correlation scores
    pub scores: Vec<f32>,
    /// Character strings
    pub texts: Vec<String>,
    /// Sample indices
    pub samples: Vec<i32>,
    /// X-locations
    pub xlocs: Vec<i32>,
    /// Y-locations
    pub ylocs: Vec<i32>,
    /// Widths
    pub widths: Vec<i32>,
}

impl Rcha {
    /// Creates a new empty Rcha
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of recognized characters
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    /// Returns true if empty
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

    /// Gets result at index
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
#[derive(Debug)]
pub struct Rdid {
    /// Clone of image to be decoded
    pub pixs: Pix,
    /// Count array per averaged template
    pub counta: Vec<Vec<i32>>,
    /// Best y-shift per averaged template
    pub delya: Vec<Vec<i32>>,
    /// Number of averaged templates
    pub narray: usize,
    /// Size of count array
    pub size: usize,
    /// Set widths per template
    pub setwidth: Vec<i32>,
    /// Pixel count by column
    pub nasum: Vec<i32>,
    /// First moment by column
    pub namoment: Vec<i32>,
    /// Whether full arrays computed
    pub fullarrays: bool,
    /// Channel coefficients for foreground term
    pub beta: Vec<f32>,
    /// Channel coefficients for bit-and term
    pub gamma: Vec<f32>,
    /// Score on trellis
    pub trellisscore: Vec<f32>,
    /// Template on trellis
    pub trellistempl: Vec<i32>,
    /// Best path template indices
    pub natempl: Vec<i32>,
    /// Best path x-locations
    pub naxloc: Vec<i32>,
    /// Best path y-shifts
    pub nadely: Vec<i32>,
    /// Best path widths
    pub nawidth: Vec<i32>,
    /// Viterbi result boxes
    pub boxa: Vec<PixBox>,
    /// Best path correlation scores
    pub nascore: Vec<f32>,
    /// Rescored template indices
    pub natempl_r: Vec<i32>,
    /// Rescored samples
    pub nasample_r: Vec<i32>,
    /// Rescored x-locations
    pub naxloc_r: Vec<i32>,
    /// Rescored y-shifts
    pub nadely_r: Vec<i32>,
    /// Rescored widths
    pub nawidth_r: Vec<i32>,
    /// Rescored correlation scores
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
/// Default maximum y-shift
pub const DEFAULT_MAX_Y_SHIFT: i32 = 1;
/// Default max width/height ratio
pub const DEFAULT_MAX_WH_RATIO: f32 = 3.0;
/// Default max template height ratio
pub const DEFAULT_MAX_HT_RATIO: f32 = 2.5;
/// Default min split width
pub const DEFAULT_MIN_SPLIT_W: i32 = 6;
/// Default max split height
pub const DEFAULT_MAX_SPLIT_H: i32 = 60;
