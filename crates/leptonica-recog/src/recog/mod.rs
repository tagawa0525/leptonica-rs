//! Character recognition module
//!
//! This module provides template-based character recognition functionality.
//!
//! # Overview
//!
//! The recognizer works by:
//! 1. Training on labeled character images to build templates
//! 2. Computing averaged templates for each character class
//! 3. Matching new images against templates using correlation
//!
//! # Example
//!
//! ```no_run
//! use leptonica_recog::recog::{Recog, create};
//!
//! // Create a recognizer with default scaling
//! let mut recog = create(40, 40, 0, 150, 1).unwrap();
//!
//! // Train with labeled samples
//! // recog.train_labeled(&pix, "A").unwrap();
//!
//! // Finish training
//! recog.finish_training().unwrap();
//!
//! // Identify characters
//! // let result = recog.identify_pix(&unknown_pix).unwrap();
//! ```

mod did;
mod ident;
mod train;
mod types;

pub use ident::*;
pub use train::*;
pub use types::*;
