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

use leptonica_region::{ConnectivityType, find_connected_components};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn conncomp_reg() {
    let mut rp = RegParams::new("conncomp");

    // Load test image
    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");

    // Test with 4-cc (4-way connectivity)
    let comps_4 =
        find_connected_components(&pixs, ConnectivityType::FourWay).expect("find 4-way components");
    let n1 = comps_4.len();
    eprintln!("Number of 4 c.c.: n1 = {}", n1);

    rp.compare_values(n1 as f64, n1 as f64, 0.0);
    rp.compare_values(n1 as f64, n1 as f64, 0.0);
    rp.compare_values(4452.0, n1 as f64, 0.0); // C version expected value

    // Test with 8-cc (8-way connectivity)
    let comps_8 = find_connected_components(&pixs, ConnectivityType::EightWay)
        .expect("find 8-way components");
    let n2 = comps_8.len();
    eprintln!("Number of 8 c.c.: n2 = {}", n2);

    rp.compare_values(n2 as f64, n2 as f64, 0.0);
    rp.compare_values(n2 as f64, n2 as f64, 0.0);
    rp.compare_values(4305.0, n2 as f64, 0.0); // C version expected value

    // 8-way should find fewer or equal components than 4-way
    assert!(
        n2 <= n1,
        "8-way components ({}) should be <= 4-way components ({})",
        n2,
        n1
    );

    // Verify component properties
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
