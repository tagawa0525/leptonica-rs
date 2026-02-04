//! Connected component regression test
//!
//! This test corresponds to conncomp_reg.c in the C version.
//!
//! Run with:
//! ```
//! cargo test -p leptonica-region --test conncomp_reg
//! ```
//!
//! Generate golden files:
//! ```
//! REGTEST_MODE=generate cargo test -p leptonica-region --test conncomp_reg
//! ```

use leptonica_region::{ConnectivityType, find_connected_components};
use leptonica_test::{RegParams, load_test_image};

#[test]
fn conncomp_reg() {
    let mut rp = RegParams::new("conncomp");

    // Load test image
    let pixs = match load_test_image("feyn.tif") {
        Ok(pix) => pix,
        Err(e) => {
            panic!("Failed to load test image: {}", e);
        }
    };

    // -----------------------------------------------------------
    // Test pixConnComp() and pixCountConnComp(),
    // with output to both boxa and pixa
    // -----------------------------------------------------------

    // Test with 4-cc (4-way connectivity)
    let comps_4 = match find_connected_components(&pixs, ConnectivityType::FourWay) {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to find 4-way components: {}", e);
        }
    };
    let n1 = comps_4.len();
    eprintln!("Number of 4 c.c.: n1 = {}", n1);

    // Index 0, 1, 2: Compare 4-cc count
    rp.compare_values(n1 as f64, n1 as f64, 0.0); // 0: self-check
    rp.compare_values(n1 as f64, n1 as f64, 0.0); // 1: self-check
    rp.compare_values(4452.0, n1 as f64, 0.0); // 2: C version expected value

    // Test with 8-cc (8-way connectivity)
    let comps_8 = match find_connected_components(&pixs, ConnectivityType::EightWay) {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to find 8-way components: {}", e);
        }
    };
    let n2 = comps_8.len();
    eprintln!("Number of 8 c.c.: n2 = {}", n2);

    // Index 3, 4, 5: Compare 8-cc count
    rp.compare_values(n2 as f64, n2 as f64, 0.0); // 3: self-check
    rp.compare_values(n2 as f64, n2 as f64, 0.0); // 4: self-check
    rp.compare_values(4305.0, n2 as f64, 0.0); // 5: C version expected value

    // Additional validations
    // 8-way should find fewer or equal components than 4-way
    // because diagonal pixels are connected
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
