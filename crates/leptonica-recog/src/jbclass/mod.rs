//! JBIG2 classification module
//!
//! This module provides JBIG2-style connected component classification
//! for document image compression.
//!
//! # Overview
//!
//! JBIG2 classification works by:
//! 1. Extracting connected components from document images
//! 2. Clustering similar components into classes
//! 3. Representing each instance by its class template and position
//!
//! Two classification methods are supported:
//! - Rank Hausdorff distance: Robust to noise, uses morphological matching
//! - Correlation: Uses pixel-wise correlation for matching
//!
//! # Example
//!
//! ```no_run
//! use leptonica_recog::jbclass::{JbClasser, JbComponent, rank_haus_init};
//!
//! // Create a classifier using rank Hausdorff method
//! let mut classer = rank_haus_init(
//!     JbComponent::Characters,
//!     150,  // max width
//!     150,  // max height
//!     2,    // structuring element size
//!     0.97  // rank value
//! ).unwrap();
//!
//! // Add pages and get compressed data
//! // classer.add_page(&pix).unwrap();
//! // let data = classer.get_data().unwrap();
//! ```

mod classify;
mod types;

pub use classify::*;
pub use types::*;
