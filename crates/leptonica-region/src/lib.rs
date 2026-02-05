//! leptonica-region - Region processing for Leptonica
//!
//! This crate provides region processing functionality including:
//!
//! - **Connected component analysis** - Finding and labeling connected regions
//! - **Seed fill operations** - Flood fill and morphological reconstruction
//! - **Watershed segmentation** - Image segmentation using watershed algorithm
//! - **Pixel labeling** - High-level labeling and analysis functions
//!
//! # Examples
//!
//! ## Finding connected components
//!
//! ```
//! use leptonica_region::{find_connected_components, ConnectivityType};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Create a binary image
//! let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
//! let mut pix_mut = pix.try_into_mut().unwrap();
//!
//! // Set some pixels to create components
//! pix_mut.set_pixel(10, 10, 1).unwrap();
//! pix_mut.set_pixel(11, 10, 1).unwrap();
//! pix_mut.set_pixel(50, 50, 1).unwrap();
//!
//! let pix: Pix = pix_mut.into();
//!
//! // Find connected components
//! let components = find_connected_components(&pix, ConnectivityType::FourWay).unwrap();
//! assert_eq!(components.len(), 2);
//! ```
//!
//! ## Flood fill
//!
//! ```
//! use leptonica_region::{floodfill, ConnectivityType};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
//! let mut pix_mut = pix.try_into_mut().unwrap();
//!
//! // Fill from seed point
//! let count = floodfill(&mut pix_mut, 5, 5, 1, ConnectivityType::FourWay).unwrap();
//! assert_eq!(count, 100); // All 100 pixels filled
//! ```
//!
//! ## Watershed segmentation
//!
//! ```
//! use leptonica_region::{watershed_segmentation, WatershedOptions};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
//! let options = WatershedOptions::default();
//! let segmented = watershed_segmentation(&pix, &options).unwrap();
//! ```

pub mod ccbord;
pub mod conncomp;
pub mod error;
pub mod label;
pub mod maze;
pub mod quadtree;
pub mod seedfill;
pub mod watershed;

// Re-export core types
pub use leptonica_core;

// Re-export error types
pub use error::{RegionError, RegionResult};

// Re-export conncomp types and functions
pub use conncomp::{
    ConnectedComponent, ConnectivityType, component_area_transform, extract_component,
    filter_components_by_size, find_connected_components, label_connected_components,
};

// Re-export label types and functions
pub use label::{
    ComponentStats, get_component_bounds_from_labels, get_component_sizes, get_component_stats,
    pix_count_components, pix_get_component_bounds, pix_label_connected_components,
};

// Re-export seedfill types and functions
pub use seedfill::{
    SeedFillOptions, clear_border, fill_holes, floodfill, seedfill_binary, seedfill_gray,
};

// Re-export watershed types and functions
pub use watershed::{
    WatershedOptions, compute_gradient, find_basins, find_local_maxima, find_local_minima,
    watershed_segmentation,
};

// Re-export ccbord types and functions
pub use ccbord::{
    Border, BorderPoint, BorderType, ComponentBorders, Direction, ImageBorders, from_chain_code,
    get_all_borders, get_component_borders, get_outer_border, get_outer_borders, render_borders,
    to_chain_code,
};

// Re-export quadtree types and functions
pub use quadtree::{
    IntegralImage, QuadtreeResult, SquaredIntegralImage, mean_in_rectangle, quadtree_max_levels,
    quadtree_mean, quadtree_mean_with_integral, quadtree_regions, quadtree_variance,
    quadtree_variance_with_integral, variance_in_rectangle,
};

// Re-export maze types and functions
pub use maze::{
    DEFAULT_ANISOTROPY_RATIO, DEFAULT_WALL_PROBABILITY, MIN_MAZE_HEIGHT, MIN_MAZE_WIDTH,
    MazeDirection, MazeGenerationOptions, MazePath, generate_binary_maze, render_maze_path,
    search_binary_maze,
};
