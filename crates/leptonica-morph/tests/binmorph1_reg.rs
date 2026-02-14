//! Binary morphology regression test
//!
//! This test corresponds to binmorph1_reg.c in the C version.
//! Tests dilation, erosion, opening, and closing operations.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test binmorph1_reg
//! ```

use leptonica_core::PixelDepth;
use leptonica_morph::{Sel, close_brick, dilate_brick, erode_brick, open_brick};
use leptonica_test::{RegParams, load_test_image};

// Brick sel dimensions (matching C version)
const WIDTH: u32 = 21;
const HEIGHT: u32 = 15;

#[test]
fn binmorph1_reg() {
    let mut rp = RegParams::new("binmorph1");

    // Load test image
    let pixs = match load_test_image("feyn-fract.tif") {
        Ok(pix) => pix,
        Err(e) => {
            panic!("Failed to load test image: {}", e);
        }
    };

    assert_eq!(
        pixs.depth(),
        PixelDepth::Bit1,
        "Test image should be binary"
    );

    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image size: {}x{}", w, h);

    // Count foreground pixels in original
    let orig_count = pixs.count_pixels();
    eprintln!("Original foreground pixels: {}", orig_count);

    // Test dilation
    eprintln!("  Testing dilation");
    let dilated = dilate_brick(&pixs, WIDTH, HEIGHT).expect("Dilation failed");
    let dilated_count = dilated.count_pixels();
    eprintln!("  Dilated foreground pixels: {}", dilated_count);

    // Dilation should increase foreground pixels
    assert!(
        dilated_count >= orig_count,
        "Dilation should not decrease foreground pixels"
    );
    rp.compare_values(
        1.0,
        if dilated_count >= orig_count {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // Test erosion
    eprintln!("  Testing erosion");
    let eroded = erode_brick(&pixs, WIDTH, HEIGHT).expect("Erosion failed");
    let eroded_count = eroded.count_pixels();
    eprintln!("  Eroded foreground pixels: {}", eroded_count);

    // Erosion should decrease foreground pixels
    assert!(
        eroded_count <= orig_count,
        "Erosion should not increase foreground pixels"
    );
    rp.compare_values(1.0, if eroded_count <= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Test opening (erosion then dilation)
    eprintln!("  Testing opening");
    let opened = open_brick(&pixs, WIDTH, HEIGHT).expect("Opening failed");
    let opened_count = opened.count_pixels();
    eprintln!("  Opened foreground pixels: {}", opened_count);

    // Opening should not increase foreground pixels (anti-extensive)
    assert!(
        opened_count <= orig_count,
        "Opening should be anti-extensive"
    );
    rp.compare_values(1.0, if opened_count <= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Test closing (dilation then erosion)
    eprintln!("  Testing closing");
    let closed = close_brick(&pixs, WIDTH, HEIGHT).expect("Closing failed");
    let closed_count = closed.count_pixels();
    eprintln!("  Closed foreground pixels: {}", closed_count);

    // Closing should not decrease foreground pixels (extensive)
    assert!(closed_count >= orig_count, "Closing should be extensive");
    rp.compare_values(1.0, if closed_count >= orig_count { 1.0 } else { 0.0 }, 0.0);

    // Test idempotence: opening twice should equal opening once
    eprintln!("  Testing opening idempotence");
    let opened2 = open_brick(&opened, WIDTH, HEIGHT).expect("Second opening failed");
    let is_idempotent = opened.equals(&opened2);
    rp.compare_values(1.0, if is_idempotent { 1.0 } else { 0.0 }, 0.0);

    // Test idempotence: closing twice should equal closing once
    eprintln!("  Testing closing idempotence");
    let closed2 = close_brick(&closed, WIDTH, HEIGHT).expect("Second closing failed");
    let is_idempotent = closed.equals(&closed2);
    rp.compare_values(1.0, if is_idempotent { 1.0 } else { 0.0 }, 0.0);

    // Test with SEL
    eprintln!("  Testing with explicit SEL");
    let sel = Sel::create_brick(WIDTH, HEIGHT).expect("Failed to create SEL");
    assert_eq!(sel.width(), WIDTH);
    assert_eq!(sel.height(), HEIGHT);

    // Verify SEL-based operations match brick operations
    let dilated_sel = leptonica_morph::dilate(&pixs, &sel).expect("SEL dilation failed");
    let match_dilate = dilated.equals(&dilated_sel);
    rp.compare_values(1.0, if match_dilate { 1.0 } else { 0.0 }, 0.0);

    let eroded_sel = leptonica_morph::erode(&pixs, &sel).expect("SEL erosion failed");
    let match_erode = eroded.equals(&eroded_sel);
    rp.compare_values(1.0, if match_erode { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "binmorph1 regression test failed");
}
