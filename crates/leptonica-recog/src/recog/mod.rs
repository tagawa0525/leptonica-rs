//! Template-based character recognition
//!
//! This module provides OCR functionality based on template matching.

mod did;
pub mod ident;
pub mod train;
mod types;

pub use ident::*;
pub use train::*;
pub use types::*;
