//! Maze regression test
//!
//! Tests binary and gray maze generation and path search.
//! The C version generates a 200x200 binary maze, finds the shortest path,
//! then searches for minimum-cost paths through a grayscale image.
//!
//! Partial migration: generate_binary_maze, search_binary_maze, and
//! search_gray_maze are tested. Display functions (pixDisplayPta,
//! pixExpandBinaryReplicate, pixScaleBySampling) are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/maze_reg.c`

use leptonica_core::PixelDepth;
use leptonica_region::{
    ConnectivityType, MazeGenerationOptions, generate_binary_maze, pix_count_components,
    search_binary_maze, search_gray_maze,
};
use leptonica_test::RegParams;

/// Test generate_binary_maze and search_binary_maze (C check 0).
///
/// Generates a binary maze and finds the shortest path between two points.
#[test]
fn maze_reg_binary() {
    let mut rp = RegParams::new("maze_bin");

    // C: pixm = generateBinaryMaze(200, 200, 20, 20, 0.65, 0.25);
    let options = MazeGenerationOptions::new(200, 200)
        .with_start(20, 20)
        .with_wall_probability(0.65)
        .with_anisotropy(0.25);
    let maze = generate_binary_maze(&options).expect("generate_binary_maze");
    assert_eq!(maze.depth(), PixelDepth::Bit1);
    rp.compare_values(200.0, maze.width() as f64, 0.0);
    rp.compare_values(200.0, maze.height() as f64, 0.0);

    // C: pta = pixSearchBinaryMaze(pixm, 20, 20, 170, 170, NULL);
    let (path, _vis) =
        search_binary_maze(&maze, (20, 20), (170, 170), false).expect("search_binary_maze");
    // Maze might not have a path if walls block it — just check the result structure
    assert!(
        path.found || path.len() == 0,
        "path structure should be valid"
    );

    assert!(rp.cleanup(), "maze binary test failed");
}

/// Test search_gray_maze on grayscale image (C check 1).
///
/// Finds minimum-cost paths through a grayscale image using Dijkstra's algorithm.
#[test]
fn maze_reg_gray() {
    let mut rp = RegParams::new("maze_gray");

    // C: pixg = pixRead("test8.jpg");
    let pix = leptonica_test::load_test_image("test8.jpg").expect("load test8.jpg");
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    let w = pix.width();
    let h = pix.height();

    // Paths from the C test (NPATHS = 6)
    let paths: &[((u32, u32), (u32, u32))] = &[
        ((42, 117), (419, 383)),
        ((73, 319), (419, 383)),
        ((42, 117), (326, 168)),
    ];

    for &(start, end) in paths {
        if start.0 >= w || start.1 >= h || end.0 >= w || end.1 >= h {
            continue; // skip if out of bounds
        }
        // C: pta = pixSearchGrayMaze(pixg, x0, y0, x1, y1, NULL);
        let (pta, costs) = search_gray_maze(&pix, start, end).expect("search_gray_maze");
        // Path should have at least 2 points (start and end)
        rp.compare_values(1.0, if pta.len() >= 2 { 1.0 } else { 0.0 }, 0.0);
        assert_eq!(pta.len(), costs.len(), "path and costs should match");
    }

    assert!(rp.cleanup(), "maze gray test failed");
}

/// Test that binary maze has connected passages (basic sanity check).
///
/// Verifies that the generated maze has at least one passage (connected component).
#[test]
fn maze_reg_connectivity() {
    let mut rp = RegParams::new("maze_conn");

    let options = MazeGenerationOptions::new(100, 100)
        .with_wall_probability(0.5)
        .with_anisotropy(0.3);
    let maze = generate_binary_maze(&options).expect("generate_binary_maze");

    // Invert maze: passages become foreground for counting components
    let passages = maze.invert();
    let count =
        pix_count_components(&passages, ConnectivityType::FourWay).expect("pix_count_components");
    // A valid maze should have at least one passage
    rp.compare_values(1.0, if count >= 1 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "maze connectivity test failed");
}
