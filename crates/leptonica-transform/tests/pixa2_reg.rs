//! pixa2_reg - Pixa replacement operations regression test
//!
//! This test corresponds to pixa2_reg.c in the C version.
//! Tests pixaReplacePix (Pixa::replace), pixaInitFull (Pixa::init_full),
//! pixaExtendArrayToSize (Pixa::extend_to_size), and Pixa with image I/O.
//!
//! The C test:
//!   1. Finds jpg/tif images in the current directory, selects a subset
//!   2. Creates a Pixa, extends it, fills all slots with copies of scaled marge.jpg
//!   3. Replaces each slot with a different scaled image (forward order)
//!   4. Re-initializes pixa, replaces each slot in reverse order
//!   5. Writes tiled display images as output (skipped: pixaDisplayTiledInRows not implemented)
//!
//! Run with:
//! ```
//! cargo test -p leptonica-transform --test pixa2_reg -- --nocapture
//! ```

use leptonica_core::{Box, Pix, Pixa, PixelDepth};
use leptonica_test::{RegParams, load_test_image};
use leptonica_transform::scale_to_size;

/// Collect test image paths (jpg and tif) from the test data directory.
///
/// The C version uses getSortedPathnamesInDirectory to find images in ".",
/// then selects ranges [10..19] from each list. We adapt this by using
/// the known test images available in the test data directory.
fn collect_test_images() -> Vec<&'static str> {
    // These are the available test images, analogous to the C test's
    // directory listing and range selection. We use a mix of jpg and tif
    // to match the C test's behavior of combining both formats.
    let jpg_images = [
        "aneurisms8.jpg",
        "color-wheel-hue.jpg",
        "fish24.jpg",
        "karen8.jpg",
        "lighttext.jpg",
        "lucasta.150.jpg",
        "marge.jpg",
        "pedante.079.jpg",
        "test24.jpg",
        "test8.jpg",
        "tetons.jpg",
        "w91frag.jpg",
        "wet-day.jpg",
        "wyom.jpg",
    ];

    let tif_images = [
        "baseline2.tif",
        "baseline3.tif",
        "char.tif",
        "feyn-fract.tif",
        "feyn.tif",
        "pageseg1.tif",
        "pageseg2.tif",
        "pageseg3.tif",
        "pageseg4.tif",
        "test16.tif",
    ];

    // Select a subset like the C test does (it takes indices 10..19 from each list).
    // Since we have fewer images, we take a representative subset.
    // Use up to 5 from each to keep test fast while still covering the logic.
    let mut images: Vec<&str> = Vec::new();
    for img in jpg_images.iter().take(5) {
        images.push(img);
    }
    for img in tif_images.iter().take(5) {
        images.push(img);
    }
    images
}

#[test]
fn pixa2_reg() {
    let mut rp = RegParams::new("pixa2");

    let image_names = collect_test_images();
    let n = image_names.len();
    eprintln!("Number of test images: {}", n);
    assert!(n > 0, "Must have at least one test image");

    // ================================================================
    // Part 1: Use replace to fill up a pixa
    // C版: pixa = pixaCreate(1)
    //       pixaExtendArrayToSize(pixa, n)
    //       pixRead("marge.jpg") -> pixScaleToSize(pix0, 144, 108)
    //       pixaInitFull(pixa, pix1, NULL)
    // ================================================================

    let mut pixa = Pixa::with_capacity(1);

    // C版: pixaExtendArrayToSize(pixa, n)
    pixa.extend_to_size(n);

    // C版: pixRead("marge.jpg")
    let pix0 = load_test_image("marge.jpg").expect("Failed to load marge.jpg");
    eprintln!(
        "marge.jpg: {}x{} depth={:?}",
        pix0.width(),
        pix0.height(),
        pix0.depth()
    );

    // C版: pixScaleToSize(pix0, 144, 108)
    let pix1 = scale_to_size(&pix0, 144, 108).expect("scale_to_size failed");
    assert_eq!(pix1.width(), 144);
    assert_eq!(pix1.height(), 108);

    // C版: pixaInitFull(pixa, pix1, NULL)
    pixa.init_full(n, Some(&pix1), None);

    // Verify: pixa should now have n elements, all copies of pix1
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);
    assert_eq!(
        pixa.len(),
        n,
        "Pixa should have {} elements after init_full",
        n
    );

    // Verify all elements have the same dimensions as pix1
    for i in 0..n {
        let (w, h, d) = pixa
            .get_dimensions(i)
            .unwrap_or_else(|| panic!("Failed to get dimensions for index {}", i));
        assert_eq!(w, 144, "Element {} width should be 144", i);
        assert_eq!(h, 108, "Element {} height should be 108", i);
        assert_eq!(d, pix1.depth(), "Element {} depth should match pix1", i);
    }
    rp.compare_values(1.0, 1.0, 0.0); // init_full verification passed

    // C版: pixd = pixaDisplayTiledInRows(pixa, 32, 1000, 1.0, 0, 25, 2) -- Rust未実装のためスキップ
    // C版: pixWrite("/tmp/lept/regout/pixa2-1.jpg", pixd, IFF_JFIF_JPEG) -- Rust未実装のためスキップ

    // ================================================================
    // Part 2: Replace each slot with scaled versions of different images
    // C版: for (i = 0; i < n; i++) {
    //         pixRead(name) -> pixScaleToSize(pix0, 144, 108)
    //         pixaReplacePix(pixa, i, pix1, NULL)
    //       }
    // ================================================================

    let mut loaded_widths: Vec<u32> = Vec::with_capacity(n);
    let mut loaded_heights: Vec<u32> = Vec::with_capacity(n);

    for i in 0..n {
        let name = image_names[i];
        let pix0 = match load_test_image(name) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error in pixa2_reg: failed to read {}: {}", name, e);
                // Record original dimensions as fallback
                loaded_widths.push(0);
                loaded_heights.push(0);
                continue;
            }
        };

        // C版: pixScaleToSize(pix0, 144, 108)
        let pix_scaled = scale_to_size(&pix0, 144, 108).expect("scale_to_size failed");

        // Record the original dimensions for verification
        loaded_widths.push(pix0.width());
        loaded_heights.push(pix0.height());

        // C版: pixaReplacePix(pixa, i, pix1, NULL)
        let _old = pixa
            .replace(i, pix_scaled)
            .unwrap_or_else(|e| panic!("replace at index {} failed: {}", i, e));
    }

    // Verify: pixa still has n elements
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);

    // All elements should still be 144x108 (since we scaled to that size)
    for i in 0..n {
        let (w, h, _d) = pixa
            .get_dimensions(i)
            .unwrap_or_else(|| panic!("Failed to get dimensions for index {}", i));
        assert_eq!(w, 144, "After replace, element {} width should be 144", i);
        assert_eq!(h, 108, "After replace, element {} height should be 108", i);
    }
    rp.compare_values(1.0, 1.0, 0.0); // forward replacement verification passed

    // C版: pixd = pixaDisplayTiledInRows(pixa, 32, 1000, 1.0, 0, 25, 2) -- Rust未実装のためスキップ
    // C版: pixWrite("/tmp/lept/regout/pixa2-2.jpg", pixd, IFF_JFIF_JPEG) -- Rust未実装のためスキップ

    // ================================================================
    // Part 3: Reinitialize and replace in reverse order
    // C版: box = boxCreate(0, 0, 0, 0)
    //       pixaInitFull(pixa, NULL, box)
    //       for (i = 0; i < n; i++) {
    //         pixRead(name) -> pixScaleToSize(pix0, 144, 108)
    //         pixaReplacePix(pixa, n - 1 - i, pix1, NULL)
    //       }
    // ================================================================

    // C版: box = boxCreate(0, 0, 0, 0)
    // Note: Box::new requires w>0, h>0, so we use Box::new_unchecked for a zero-size box
    let box0 = Box::new_unchecked(0, 0, 0, 0);

    // C版: pixaInitFull(pixa, NULL, box) -- reinitialize with placeholder pix + box
    pixa.init_full(n, None, Some(&box0));

    // Verify: pixa should have n elements, all tiny 1x1 placeholders
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);
    for i in 0..n {
        let (w, h, d) = pixa
            .get_dimensions(i)
            .unwrap_or_else(|| panic!("Failed to get dimensions for index {}", i));
        assert_eq!(w, 1, "After reinit, element {} width should be 1", i);
        assert_eq!(h, 1, "After reinit, element {} height should be 1", i);
        assert_eq!(
            d,
            PixelDepth::Bit1,
            "After reinit, element {} depth should be Bit1",
            i
        );
    }

    // Verify boxes were also initialized
    rp.compare_values(n as f64, pixa.boxa_count() as f64, 0.0);
    for i in 0..n {
        let b = pixa
            .get_box(i)
            .unwrap_or_else(|| panic!("Failed to get box for index {}", i));
        assert_eq!(b.x, 0, "Box {} x should be 0", i);
        assert_eq!(b.y, 0, "Box {} y should be 0", i);
        assert_eq!(b.w, 0, "Box {} w should be 0", i);
        assert_eq!(b.h, 0, "Box {} h should be 0", i);
    }
    rp.compare_values(1.0, 1.0, 0.0); // reinit verification passed

    // Replace in reverse order
    for i in 0..n {
        let name = image_names[i];
        let pix0 = match load_test_image(name) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error in pixa2_reg: failed to read {}: {}", name, e);
                continue;
            }
        };

        // C版: pixScaleToSize(pix0, 144, 108)
        let pix_scaled = scale_to_size(&pix0, 144, 108).expect("scale_to_size failed");

        // C版: pixaReplacePix(pixa, n - 1 - i, pix1, NULL)
        let _old = pixa
            .replace(n - 1 - i, pix_scaled)
            .unwrap_or_else(|e| panic!("reverse replace at index {} failed: {}", n - 1 - i, e));
    }

    // Verify: pixa still has n elements, all 144x108
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);
    for i in 0..n {
        let (w, h, _d) = pixa
            .get_dimensions(i)
            .unwrap_or_else(|| panic!("Failed to get dimensions for index {}", i));
        assert_eq!(
            w, 144,
            "After reverse replace, element {} width should be 144",
            i
        );
        assert_eq!(
            h, 108,
            "After reverse replace, element {} height should be 108",
            i
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // reverse replacement verification passed

    // C版: pixd = pixaDisplayTiledInRows(pixa, 32, 1000, 1.0, 0, 25, 2) -- Rust未実装のためスキップ
    // C版: pixWrite("/tmp/lept/regout/pixa2-3.jpg", pixd, IFF_JFIF_JPEG) -- Rust未実装のためスキップ

    assert!(rp.cleanup(), "pixa2 regression test failed");
}

/// Test pixaExtendArrayToSize (Pixa::extend_to_size) in isolation
#[test]
fn pixa2_extend_array_to_size() {
    let mut rp = RegParams::new("pixa2_extend");

    // C版: pixa = pixaCreate(1); pixaExtendArrayToSize(pixa, n)
    let mut pixa = Pixa::with_capacity(1);

    // extend_to_size should pre-allocate capacity without changing len
    pixa.extend_to_size(20);
    rp.compare_values(0.0, pixa.len() as f64, 0.0); // len should still be 0

    // After extending, pushing up to 20 elements should not cause reallocation issues
    for i in 0..20 {
        pixa.push(Pix::new(i + 1, i + 1, PixelDepth::Bit8).unwrap());
    }
    rp.compare_values(20.0, pixa.len() as f64, 0.0);

    // Verify each element has correct dimensions
    for i in 0..20 {
        let (w, h, _) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, (i + 1) as u32);
        assert_eq!(h, (i + 1) as u32);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Extending to a smaller size should be a no-op
    pixa.extend_to_size(5);
    rp.compare_values(20.0, pixa.len() as f64, 0.0); // len unchanged

    assert!(rp.cleanup(), "pixa2_extend regression test failed");
}

/// Test pixaInitFull (Pixa::init_full) in isolation
#[test]
fn pixa2_init_full() {
    let mut rp = RegParams::new("pixa2_initfull");

    // Test 1: init_full with a specific Pix
    let mut pixa = Pixa::new();
    let template = Pix::new(50, 30, PixelDepth::Bit8).unwrap();

    pixa.init_full(5, Some(&template), None);
    rp.compare_values(5.0, pixa.len() as f64, 0.0);

    for i in 0..5 {
        let (w, h, d) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, 50);
        assert_eq!(h, 30);
        assert_eq!(d, PixelDepth::Bit8);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Test 2: init_full with None Pix (should use 1x1 placeholder)
    // C版: pixaInitFull(pixa, NULL, NULL)
    pixa.init_full(3, None, None);
    rp.compare_values(3.0, pixa.len() as f64, 0.0);

    for i in 0..3 {
        let (w, h, d) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(d, PixelDepth::Bit1);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Test 3: init_full with Box
    // C版: box = boxCreate(0, 0, 0, 0); pixaInitFull(pixa, NULL, box)
    let box0 = Box::new_unchecked(10, 20, 30, 40);
    pixa.init_full(4, None, Some(&box0));
    rp.compare_values(4.0, pixa.len() as f64, 0.0);
    rp.compare_values(4.0, pixa.boxa_count() as f64, 0.0);

    for i in 0..4 {
        let b = pixa.get_box(i).unwrap();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.w, 30);
        assert_eq!(b.h, 40);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Test 4: init_full replaces existing elements
    let mut pixa = Pixa::new();
    for _ in 0..10 {
        pixa.push(Pix::new(200, 200, PixelDepth::Bit32).unwrap());
    }
    assert_eq!(pixa.len(), 10);

    // Re-initialize with fewer elements
    pixa.init_full(3, None, None);
    rp.compare_values(3.0, pixa.len() as f64, 0.0);
    // All should now be 1x1 placeholders
    for i in 0..3 {
        let (w, h, _) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, 1);
        assert_eq!(h, 1);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    assert!(rp.cleanup(), "pixa2_initfull regression test failed");
}

/// Test pixaReplacePix (Pixa::replace) in isolation
#[test]
fn pixa2_replace_pix() {
    let mut rp = RegParams::new("pixa2_replace");

    let mut pixa = Pixa::new();

    // Fill with 5 identical images
    let original = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    pixa.init_full(5, Some(&original), None);
    rp.compare_values(5.0, pixa.len() as f64, 0.0);

    // Replace element 2 with a different sized image
    let replacement = Pix::new(20, 30, PixelDepth::Bit32).unwrap();
    let old = pixa.replace(2, replacement).unwrap();
    assert_eq!(old.width(), 10, "Old pix should have width 10");
    assert_eq!(old.height(), 10, "Old pix should have height 10");

    // Verify the replacement
    let (w, h, d) = pixa.get_dimensions(2).unwrap();
    assert_eq!(w, 20);
    assert_eq!(h, 30);
    assert_eq!(d, PixelDepth::Bit32);

    // Other elements should be unchanged
    for i in [0, 1, 3, 4] {
        let (w, h, d) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, 10, "Element {} should still have width 10", i);
        assert_eq!(h, 10, "Element {} should still have height 10", i);
        assert_eq!(
            d,
            PixelDepth::Bit8,
            "Element {} should still have depth Bit8",
            i
        );
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Replace out of bounds should fail
    let oob = pixa.replace(10, Pix::new(1, 1, PixelDepth::Bit1).unwrap());
    assert!(
        oob.is_err(),
        "Replace at out-of-bounds index should return error"
    );
    rp.compare_values(1.0, 1.0, 0.0);

    // Replace all elements in reverse order (like the C test does)
    for i in 0..5 {
        let new_pix = Pix::new((i + 1) as u32 * 10, (i + 1) as u32 * 10, PixelDepth::Bit8).unwrap();
        pixa.replace(4 - i, new_pix).unwrap();
    }

    // After reverse replacement:
    // index 0 should have been replaced when i=4 -> 50x50
    // index 1 should have been replaced when i=3 -> 40x40
    // index 2 should have been replaced when i=2 -> 30x30
    // index 3 should have been replaced when i=1 -> 20x20
    // index 4 should have been replaced when i=0 -> 10x10
    let expected_sizes: [(u32, u32); 5] = [(50, 50), (40, 40), (30, 30), (20, 20), (10, 10)];
    for (i, (ew, eh)) in expected_sizes.iter().enumerate() {
        let (w, h, _) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, *ew, "Element {} width mismatch after reverse replace", i);
        assert_eq!(
            h, *eh,
            "Element {} height mismatch after reverse replace",
            i
        );
    }
    rp.compare_values(1.0, 1.0, 0.0); // reverse replace verification

    assert!(rp.cleanup(), "pixa2_replace regression test failed");
}

/// Test Pixa with image I/O round-trip
///
/// The C test reads images and scales them into a Pixa. While Pixa
/// serialization (pixaRead/pixaWrite) is not implemented in Rust,
/// we test individual image I/O round-trips through the Pixa.
#[test]
fn pixa2_io_roundtrip() {
    let mut rp = RegParams::new("pixa2_io");

    // Load marge.jpg and scale it
    let pix0 = load_test_image("marge.jpg").expect("Failed to load marge.jpg");
    let pix_scaled = scale_to_size(&pix0, 144, 108).expect("scale_to_size failed");

    // Store in pixa
    let mut pixa = Pixa::new();
    pixa.push(pix_scaled.clone());

    // Verify round-trip: the pix in the pixa should match the original
    let retrieved = pixa.get(0).unwrap();
    assert_eq!(retrieved.width(), 144);
    assert_eq!(retrieved.height(), 108);
    assert_eq!(retrieved.depth(), pix_scaled.depth());
    rp.compare_values(144.0, retrieved.width() as f64, 0.0);
    rp.compare_values(108.0, retrieved.height() as f64, 0.0);

    // Pixel-by-pixel comparison with the original scaled image
    rp.compare_pix(&pix_scaled, retrieved);

    // Test with multiple images of different formats
    let test_images = ["marge.jpg", "feyn-fract.tif"];
    let mut pixa2 = Pixa::new();

    for name in &test_images {
        let pix = load_test_image(name).expect(&format!("Failed to load {}", name));
        let scaled = scale_to_size(&pix, 100, 100).expect("scale_to_size failed");
        pixa2.push(scaled);
    }

    rp.compare_values(test_images.len() as f64, pixa2.len() as f64, 0.0);

    // Verify each stored image
    for i in 0..pixa2.len() {
        let (w, h, _) = pixa2.get_dimensions(i).unwrap();
        assert_eq!(w, 100, "Image {} width should be 100", i);
        assert_eq!(h, 100, "Image {} height should be 100", i);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // C版: pixaRead/pixaWrite -- Rust未実装のためスキップ
    // Pixa serialization (native binary format) is not implemented in Rust.

    assert!(rp.cleanup(), "pixa2_io regression test failed");
}

/// Test the combination of extend + init_full + replace (full C test flow)
///
/// This tests the exact sequence from the C test:
///   pixaCreate(1) -> pixaExtendArrayToSize(n) -> pixaInitFull(pix) -> pixaReplacePix(...)
#[test]
fn pixa2_full_sequence() {
    let mut rp = RegParams::new("pixa2_sequence");

    let n = 8;

    // Step 1: Create and extend
    let mut pixa = Pixa::with_capacity(1);
    pixa.extend_to_size(n);
    rp.compare_values(0.0, pixa.len() as f64, 0.0); // still empty

    // Step 2: Init full with a template
    let template = Pix::new(144, 108, PixelDepth::Bit8).unwrap();
    pixa.init_full(n, Some(&template), None);
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);

    // Step 3: Replace in forward order
    for i in 0..n {
        let w = (i as u32 + 1) * 20;
        let h = (i as u32 + 1) * 15;
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        pixa.replace(i, pix).unwrap();
    }

    // Verify forward order
    for i in 0..n {
        let expected_w = (i as u32 + 1) * 20;
        let expected_h = (i as u32 + 1) * 15;
        let (w, h, _) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, expected_w, "Forward: element {} width", i);
        assert_eq!(h, expected_h, "Forward: element {} height", i);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    // Step 4: Re-init with NULL pix + box, then replace in reverse
    let box0 = Box::new_unchecked(0, 0, 0, 0);
    pixa.init_full(n, None, Some(&box0));
    rp.compare_values(n as f64, pixa.len() as f64, 0.0);

    // Verify all are placeholders
    for i in 0..n {
        let (w, h, d) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(d, PixelDepth::Bit1);
    }

    // Replace in reverse order
    for i in 0..n {
        let w = (i as u32 + 1) * 20;
        let h = (i as u32 + 1) * 15;
        let pix = Pix::new(w, h, PixelDepth::Bit8).unwrap();
        pixa.replace(n - 1 - i, pix).unwrap();
    }

    // Verify reverse order:
    // i=0 -> replaces index n-1 with 20x15
    // i=1 -> replaces index n-2 with 40x30
    // ...
    // i=n-1 -> replaces index 0 with (n*20)x(n*15)
    for i in 0..n {
        let expected_w = (n - i) as u32 * 20;
        let expected_h = (n - i) as u32 * 15;
        let (w, h, _) = pixa.get_dimensions(i).unwrap();
        assert_eq!(w, expected_w, "Reverse: element {} width", i);
        assert_eq!(h, expected_h, "Reverse: element {} height", i);
    }
    rp.compare_values(1.0, 1.0, 0.0);

    assert!(rp.cleanup(), "pixa2_sequence regression test failed");
}
