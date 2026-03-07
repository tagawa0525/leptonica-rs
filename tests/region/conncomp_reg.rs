//! Connected component regression test
//!
//! This test corresponds to conncomp_reg.c in the C version.
//!
//! C reference: reference/leptonica/prog/conncomp_reg.c
//!
//! Verifies:
//! 1. 4-way and 8-way connected component counting
//! 2. Component count matches C version expected values
//! 3. 8-way count <= 4-way count (diagonal connections reduce count)
//! 4. Each component has positive pixel count and bounding box

use crate::common::{RegParams, load_test_image};
use leptonica::io::ImageFormat;
use leptonica::region::{ConnectivityType, conncomp_pixa, find_connected_components};

#[test]
fn conncomp_reg() {
    if crate::common::is_display_mode() {
        return;
    }

    let mut rp = RegParams::new("conncomp");

    // Load test image
    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");

    // Test with 4-cc (4-way connectivity)
    let comps_4 =
        find_connected_components(&pixs, ConnectivityType::FourWay).expect("find 4-way components");
    let n1 = comps_4.len();

    // C check 0-1: pixa count matches find_connected_components
    let (boxa4, pixa4) =
        conncomp_pixa(&pixs, ConnectivityType::FourWay).expect("conncomp_pixa 4-way");
    rp.compare_values(n1 as f64, pixa4.len() as f64, 0.0);
    rp.compare_values(n1 as f64, boxa4.len() as f64, 0.0);
    // C check 2: absolute expected value
    rp.compare_values(4452.0, n1 as f64, 0.0);

    // C check 3: reconstruct from pixa and compare (WPAC)
    let display4 = pixa4
        .display(pixs.width(), pixs.height())
        .expect("display 4-way");
    rp.write_pix_and_check(&display4, ImageFormat::Png)
        .expect("check: conncomp 4-way display");

    // C check 4: reconstructed image dimensions match
    rp.compare_values(pixs.width() as f64, display4.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, display4.height() as f64, 0.0);

    // Test with 8-cc (8-way connectivity)
    let comps_8 = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find 8-way components");
    let n2 = comps_8.len();

    // C check 5-6: pixa count matches
    let (boxa8, pixa8) =
        conncomp_pixa(&pixs, ConnectivityType::EightWay).expect("conncomp_pixa 8-way");
    rp.compare_values(n2 as f64, pixa8.len() as f64, 0.0);
    rp.compare_values(n2 as f64, boxa8.len() as f64, 0.0);
    // C check 7: absolute expected value
    rp.compare_values(4305.0, n2 as f64, 0.0);

    // C check 8: reconstruct 8-way (WPAC)
    let display8 = pixa8
        .display(pixs.width(), pixs.height())
        .expect("display 8-way");
    rp.write_pix_and_check(&display8, ImageFormat::Png)
        .expect("check: conncomp 8-way display");

    // C check 10: Boxa serialization roundtrip
    let boxa_data = boxa4.write_to_bytes().expect("serialize boxa4");
    let boxa_rt = leptonica::Boxa::read_from_bytes(&boxa_data).expect("deserialize boxa4");
    rp.compare_values(n1 as f64, boxa_rt.len() as f64, 0.0);

    // C checks 12-17: covering rectangles with increasing distance
    for dist in [1, 2, 3] {
        let covering = pixs
            .make_covering_of_rectangles(dist)
            .expect("covering rects");
        rp.compare_values(1.0, if !covering.is_empty() { 1.0 } else { 0.0 }, 0.0);
    }

    // 8-way should find fewer or equal components than 4-way
    assert!(n2 <= n1);

    // Verify per-component invariants (pixel_count > 0, positive bounds)
    for comp in &comps_4 {
        assert!(
            comp.pixel_count > 0,
            "Component should have at least 1 pixel"
        );
        assert!(
            comp.bounds.w > 0 && comp.bounds.h > 0,
            "Bounds should be positive"
        );
    }

    assert!(rp.cleanup(), "conncomp regression test failed");
}
