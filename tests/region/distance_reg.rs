//! Distance function regression test
//!
//! Tests pixDistanceFunction for all 8 combinations of connectivity (4/8),
//! destination depth (8/16 bpp), and boundary condition (Background/Foreground).
//! The C version uses a clipped region of feyn.tif and also tests
//! seedfill_gray labeling of distance values.
//!
//! # See also
//!
//! C Leptonica: `prog/distance_reg.c`

use crate::common::RegParams;
use leptonica::PixelDepth;
use leptonica::filter::{DynamicRangeScale, max_dynamic_range};
use leptonica::io::ImageFormat;
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
    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = pix
        .clip_rectangle(383, 338, 800, 500)
        .expect("clip feyn region");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);

    let connectivities = [ConnectivityType::FourWay, ConnectivityType::EightWay];
    let depths = [PixelDepth::Bit8, PixelDepth::Bit16];
    let boundaries = [BoundaryCondition::Background, BoundaryCondition::Foreground];

    let mut combo_idx = 0usize;
    for &conn in &connectivities {
        for &depth in &depths {
            for &bc in &boundaries {
                // C: pixt1 = pixDistanceFunction(pixs, conn, depth, bc);
                let result = distance_function(&pixs, conn, depth, bc).expect("distance_function");
                rp.compare_values(pixs.width() as f64, result.width() as f64, 0.0);
                rp.compare_values(pixs.height() as f64, result.height() as f64, 0.0);
                assert_eq!(result.depth(), depth);
                if combo_idx == 0 {
                    rp.write_pix_and_check(&result, ImageFormat::Png)
                        .expect("write result dist_combos");
                }
                combo_idx += 1;
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

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
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
    rp.write_pix_and_check(&dist, ImageFormat::Png)
        .expect("write dist dist_seedfill");
    rp.write_pix_and_check(&labeled, ImageFormat::Png)
        .expect("write labeled dist_seedfill");

    assert!(rp.cleanup(), "distance seedfill labeling test failed");
}

/// Test max_dynamic_range (LOG and LINEAR) on distance function output (C checks a+2, a+5, a+6).
///
/// For each of the 8 connectivity/depth/boundary combinations:
/// - compute distance_function → apply max_dynamic_range(Log) → WPAC
/// - compute distance_function → apply max_dynamic_range(Linear) → WPAC
///
/// C version: `TestDistance()` in `distance_reg.c` (pixt2, pixt4, pixt5)
#[test]
fn distance_reg_max_dynamic_range() {
    let mut rp = RegParams::new("dist_dynrange");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = pix
        .clip_rectangle(383, 338, 400, 300)
        .expect("clip feyn region");

    let connectivities = [ConnectivityType::FourWay, ConnectivityType::EightWay];
    let depths = [PixelDepth::Bit8, PixelDepth::Bit16];
    let boundaries = [BoundaryCondition::Background, BoundaryCondition::Foreground];

    for &conn in &connectivities {
        for &depth in &depths {
            for &bc in &boundaries {
                let dist = distance_function(&pixs, conn, depth, bc).expect("distance_function");

                // C: pixt2 = pixMaxDynamicRange(pixt1, L_LOG_SCALE); /* a+2 */
                let log_scaled = max_dynamic_range(&dist, DynamicRangeScale::Log)
                    .expect("max_dynamic_range log");
                assert_eq!(log_scaled.depth(), PixelDepth::Bit8);
                rp.write_pix_and_check(&log_scaled, ImageFormat::Png)
                    .expect("write log scaled");

                // C: pixt4 = pixMaxDynamicRange(pixt3, L_LINEAR_SCALE); /* a+5 */
                let lin_scaled = max_dynamic_range(&dist, DynamicRangeScale::Linear)
                    .expect("max_dynamic_range linear");
                assert_eq!(lin_scaled.depth(), PixelDepth::Bit8);
                rp.write_pix_and_check(&lin_scaled, ImageFormat::Png)
                    .expect("write linear scaled");
            }
        }
    }

    assert!(rp.cleanup(), "distance max_dynamic_range test failed");
}

/// Test render_contours on distance function output (C check a+4).
///
/// For each of the 8 combinations, generates binary and overlay contour images.
///
/// C version: `TestDistance()` in `distance_reg.c` (pixt2 = pixRenderContours)
#[test]
fn distance_reg_render_contours() {
    use leptonica::ContourOutput;

    let mut rp = RegParams::new("dist_contours");

    let pix = crate::common::load_test_image("feyn.tif").expect("load feyn.tif");
    let pixs = pix
        .clip_rectangle(383, 338, 400, 300)
        .expect("clip feyn region");

    let connectivities = [ConnectivityType::FourWay, ConnectivityType::EightWay];
    let depths = [PixelDepth::Bit8, PixelDepth::Bit16];
    let boundaries = [BoundaryCondition::Background, BoundaryCondition::Foreground];

    for &conn in &connectivities {
        for &depth in &depths {
            for &bc in &boundaries {
                let dist = distance_function(&pixs, conn, depth, bc).expect("distance_function");

                // C: pixt2 = pixRenderContours(pixt1, 2, 4, 1); /* binary, a+4 */
                let contour_bin = dist
                    .render_contours(2, 4, ContourOutput::Binary)
                    .expect("render_contours binary");
                assert_eq!(contour_bin.depth(), PixelDepth::Bit1);
                rp.write_pix_and_check(&contour_bin, ImageFormat::Png)
                    .expect("write contour_bin");

                // C: pixt3 = pixRenderContours(pixt1, 2, 4, depth); /* overlay */
                // pixt5 = pixMaxDynamicRange(pixt3, L_LOG_SCALE); /* a+6 */
                let contour_ov = dist
                    .render_contours(2, 4, ContourOutput::Overlay)
                    .expect("render_contours overlay");
                let log_scaled = max_dynamic_range(&contour_ov, DynamicRangeScale::Log)
                    .expect("max_dynamic_range on contour overlay");
                rp.write_pix_and_check(&log_scaled, ImageFormat::Png)
                    .expect("write contour log scaled");
            }
        }
    }

    assert!(rp.cleanup(), "distance render_contours test failed");
}
