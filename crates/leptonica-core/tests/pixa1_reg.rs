//! Pixa (image array) regression test
//!
//! Tests Pixa creation, push/pop, box management, verification,
//! iterators, and Pixaa operations.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/pixa1_reg.c`

use leptonica_core::{Box, Pix, Pixa, Pixaa, PixelDepth};
use leptonica_test::RegParams;

fn make_test_pix(width: u32, height: u32) -> Pix {
    Pix::new(width, height, PixelDepth::Bit8).unwrap()
}

// ==========================================================================
// Test 1: Pixa creation and basic operations
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_basic() {
    let mut rp = RegParams::new("pixa1_basic");

    let mut pixa = Pixa::new();
    rp.compare_values(0.0, pixa.len() as f64, 0.0);
    rp.compare_values(1.0, if pixa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    // Push and get
    pixa.push(make_test_pix(100, 200));
    rp.compare_values(1.0, pixa.len() as f64, 0.0);
    rp.compare_values(100.0, pixa.get(0).unwrap().width() as f64, 0.0);
    rp.compare_values(200.0, pixa.get(0).unwrap().height() as f64, 0.0);

    // Get cloned
    let cloned = pixa.get_cloned(0).unwrap();
    rp.compare_values(100.0, cloned.width() as f64, 0.0);

    // Get dimensions
    let (w, h, d) = pixa.get_dimensions(0).unwrap();
    rp.compare_values(100.0, w as f64, 0.0);
    rp.compare_values(200.0, h as f64, 0.0);
    rp.compare_values(8.0, d.bits() as f64, 0.0);

    assert!(rp.cleanup(), "pixa1_reg basic tests failed");
}

// ==========================================================================
// Test 2: Pixa with boxes
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_boxes() {
    let mut rp = RegParams::new("pixa1_boxes");

    let mut pixa = Pixa::new();
    let b = Box::new(10, 20, 30, 40).unwrap();
    pixa.push_with_box(make_test_pix(100, 100), b);

    rp.compare_values(1.0, pixa.len() as f64, 0.0);
    rp.compare_values(1.0, pixa.boxa_count() as f64, 0.0);

    let retrieved_box = pixa.get_box(0).unwrap();
    rp.compare_values(10.0, retrieved_box.x as f64, 0.0);
    rp.compare_values(20.0, retrieved_box.y as f64, 0.0);
    rp.compare_values(30.0, retrieved_box.w as f64, 0.0);
    rp.compare_values(40.0, retrieved_box.h as f64, 0.0);

    assert!(rp.cleanup(), "pixa1_reg boxes tests failed");
}

// ==========================================================================
// Test 3: Insert, remove, replace
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_modify() {
    let mut rp = RegParams::new("pixa1_modify");

    let mut pixa = Pixa::new();
    pixa.push(make_test_pix(100, 100));
    pixa.push(make_test_pix(200, 200));
    pixa.push(make_test_pix(300, 300));

    // Remove middle element
    let removed = pixa.remove(1).unwrap();
    rp.compare_values(200.0, removed.width() as f64, 0.0);
    rp.compare_values(2.0, pixa.len() as f64, 0.0);
    rp.compare_values(300.0, pixa.get(1).unwrap().width() as f64, 0.0);

    // Insert
    pixa.insert(1, make_test_pix(250, 250)).unwrap();
    rp.compare_values(3.0, pixa.len() as f64, 0.0);
    rp.compare_values(250.0, pixa.get(1).unwrap().width() as f64, 0.0);

    // Replace
    let old = pixa.replace(0, make_test_pix(150, 150)).unwrap();
    rp.compare_values(100.0, old.width() as f64, 0.0);
    rp.compare_values(150.0, pixa.get(0).unwrap().width() as f64, 0.0);

    // Pop
    let popped = pixa.pop().unwrap();
    rp.compare_values(300.0, popped.width() as f64, 0.0);
    rp.compare_values(2.0, pixa.len() as f64, 0.0);

    // Clear
    pixa.clear();
    rp.compare_values(0.0, pixa.len() as f64, 0.0);
    rp.compare_values(1.0, if pixa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixa1_reg modify tests failed");
}

// ==========================================================================
// Test 4: Verify depth and dimensions
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_verify() {
    let mut rp = RegParams::new("pixa1_verify");

    let mut pixa = Pixa::new();

    // Same depth
    pixa.push(make_test_pix(100, 100));
    pixa.push(make_test_pix(200, 200));
    let (same, depth) = pixa.verify_depth().unwrap();
    rp.compare_values(1.0, if same { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(8.0, depth.bits() as f64, 0.0);

    // Different depths
    pixa.push(Pix::new(50, 50, PixelDepth::Bit32).unwrap());
    let (same2, max_depth) = pixa.verify_depth().unwrap();
    rp.compare_values(0.0, if same2 { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(32.0, max_depth.bits() as f64, 0.0);

    // Verify dimensions (different)
    let not_same_dim = pixa.verify_dimensions().unwrap();
    rp.compare_values(0.0, if not_same_dim { 1.0 } else { 0.0 }, 0.0);

    // Same dimensions
    let mut pixa2 = Pixa::new();
    pixa2.push(make_test_pix(100, 100));
    pixa2.push(make_test_pix(100, 100));
    let same_dim = pixa2.verify_dimensions().unwrap();
    rp.compare_values(1.0, if same_dim { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixa1_reg verify tests failed");
}

// ==========================================================================
// Test 5: Iterator
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_iterator() {
    let mut rp = RegParams::new("pixa1_iter");

    let mut pixa = Pixa::new();
    pixa.push(make_test_pix(100, 100));
    pixa.push(make_test_pix(200, 200));
    pixa.push(make_test_pix(300, 300));

    let widths: Vec<u32> = pixa.iter().map(|p| p.width()).collect();
    rp.compare_values(100.0, widths[0] as f64, 0.0);
    rp.compare_values(200.0, widths[1] as f64, 0.0);
    rp.compare_values(300.0, widths[2] as f64, 0.0);

    // From iterator
    let pix_list = vec![make_test_pix(10, 10), make_test_pix(20, 20)];
    let pixa2: Pixa = pix_list.into_iter().collect();
    rp.compare_values(2.0, pixa2.len() as f64, 0.0);

    assert!(rp.cleanup(), "pixa1_reg iterator tests failed");
}

// ==========================================================================
// Test 6: Clone vs deep clone
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_clone() {
    let mut rp = RegParams::new("pixa1_clone");

    let mut pixa = Pixa::new();
    pixa.push(make_test_pix(100, 100));

    // Regular clone shares data via Arc
    let cloned = pixa.clone();
    let same_ptr = pixa[0].data().as_ptr() == cloned[0].data().as_ptr();
    rp.compare_values(1.0, if same_ptr { 1.0 } else { 0.0 }, 0.0);

    // Deep clone creates independent copies
    let deep = pixa.deep_clone();
    let diff_ptr = pixa[0].data().as_ptr() != deep[0].data().as_ptr();
    rp.compare_values(1.0, if diff_ptr { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "pixa1_reg clone tests failed");
}

// ==========================================================================
// Test 7: Pixaa operations
// ==========================================================================

#[test]
#[ignore = "not yet implemented"]
fn pixa1_reg_pixaa() {
    let mut rp = RegParams::new("pixa1_pixaa");

    let mut pixaa = Pixaa::new();
    rp.compare_values(0.0, pixaa.len() as f64, 0.0);
    rp.compare_values(1.0, if pixaa.is_empty() { 1.0 } else { 0.0 }, 0.0);

    let mut pixa1 = Pixa::new();
    pixa1.push(make_test_pix(100, 100));
    pixa1.push(make_test_pix(200, 200));
    pixaa.push(pixa1);

    let mut pixa2 = Pixa::new();
    pixa2.push(make_test_pix(300, 300));
    pixaa.push(pixa2);

    rp.compare_values(2.0, pixaa.len() as f64, 0.0);
    rp.compare_values(3.0, pixaa.total_pix() as f64, 0.0);

    // Flatten
    let flat = pixaa.flatten();
    rp.compare_values(3.0, flat.len() as f64, 0.0);
    rp.compare_values(100.0, flat[0].width() as f64, 0.0);
    rp.compare_values(200.0, flat[1].width() as f64, 0.0);
    rp.compare_values(300.0, flat[2].width() as f64, 0.0);

    // Get pix
    let pix = pixaa.get_pix(0, 1).unwrap();
    rp.compare_values(200.0, pix.width() as f64, 0.0);

    assert!(rp.cleanup(), "pixa1_reg pixaa tests failed");
}
