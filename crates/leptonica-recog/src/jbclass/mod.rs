//! JBIG2 connected component classification
//!
//! This module provides JBIG2-style connected component classification
//! for document image compression.

pub mod classify;
mod types;

pub use classify::*;
pub use types::*;
