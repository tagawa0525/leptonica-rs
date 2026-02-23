//! Distance function regression test
//!
//! Tests pixDistanceFunction for all 8 combinations of connectivity (4/8),
//! destination depth (8/16 bpp), and boundary condition (Background/Foreground).
//! The C version uses a clipped region of feyn.tif and also tests
//! seedfill_gray labeling of distance values.
//!
//! Partial migration: distance_function with all 8 combinations is tested.
//! pixMaxDynamicRange and pixRenderContours are not available.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/distance_reg.c`

mod common;
use common::RegParams;
use leptonica::PixelDepth;
use leptonica::region::{BoundaryCondition, ConnectivityType, distance_function, seedfill_gray};

/// Test distance_function with all connectivity/depth/boundary combinations (C checks 1-8).
///
/// Computes the distance from each foreground pixel to the nearest background
/// pixel for 4-way and 8-way connectivity at 8bpp and 16bpp output depth.
#[test]
fn distance_reg_all_combos() {
    let mut rp = RegParams::new("dist_combos");

    // C: pix = pixRead("feyn.tif"); pixs = pixClipRectangle(pix, box, NULL);
    // box = boxCreate(383, 338, 1480, 1050);
    let pix = common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = pix
        .clip_rectangle(383, 338, 800, 500)
        .expect("clip feyn region");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    let connectivities = [ConnectivityType::FourWay, ConnectivityType::EightWay];
    let depths = [PixelDepth::Bit8, PixelDepth::Bit16];
    let boundaries = [BoundaryCondition::Background, BoundaryCondition::Foreground];

    for &conn in &connectivities {
        for &depth in &depths {
            for &bc in &boundaries {
                // C: pixt1 = pixDistanceFunction(pixs, conn, depth, bc);
                let result = distance_function(&pixs, conn, depth, bc).expect("distance_function");
                rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
                rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
                assert_eq!(result.depth(), depth);
            }
        }
    }

    assert!(rp.cleanup(), "distance all combos test failed");
}

/// Test seedfill_gray labeling using distance function output (C check b+1-3).
///
/// Uses the distance function output as both the seed and mask source,
/// then applies seedfill_gray to label each connected component with
/// the maximum distance value in that component.
#[test]
fn distance_reg_seedfill_labeling() {
    let mut rp = RegParams::new("dist_seedfill");

    let pix = common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = pix
        .clip_rectangle(383, 338, 400, 300)
        .expect("clip feyn region");

    // C: pixt1 = pixDistanceFunction(pixs, 4, 8, bc);
    let dist = distance_function(
        &pixs,
        ConnectivityType::FourWay,
        PixelDepth::Bit8,
        BoundaryCondition::Background,
    )
    .expect("distance_function");
    rp.compare_values(pixs.width() as f64, dist.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, dist.height() as f64, 0.0);

    // C: pixt2 = pixCreateTemplate(pixt1); pixSetMasked(pixt2, pixs, 255);
    // Create a mask image where foreground pixels are set to 255
    let mask = {
        let template = dist.create_template();
        let mut m = template.try_into_mut().expect("try_into_mut");
        m.set_masked(&pixs, 255).expect("set_masked");
        let p: leptonica::Pix = m.into();
        p
    };

    // C: pixSeedfillGray(pixt1, pixt2, 4); -- labels each cc with max dist
    let labeled =
        seedfill_gray(&dist, &mask, ConnectivityType::FourWay).expect("seedfill_gray for labeling");
    rp.compare_values(pixs.width() as f64, labeled.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, labeled.height() as f64, 0.0);
    assert_eq!(labeled.depth(), PixelDepth::Bit8);

    assert!(rp.cleanup(), "distance seedfill labeling test failed");
}
