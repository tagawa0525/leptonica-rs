//! Multi-type image operations regression test
//!
//! Tests geometric transformations (rotation, affine, projective, bilinear)
//! across multiple image types (1, 2, 4, 8, 32 bpp).
//!
//! The C version applies each transformation to 10 image types and
//! verifies output via golden files (17 checks total).
//! This Rust port verifies that transformations run without error
//! and preserve image dimensions.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/multitype_reg.c`

use leptonica_core::PixelDepth;
use leptonica_test::RegParams;
use leptonica_transform::{
    AffineFill, AffineMatrix, BilinearCoeffs, Point, ProjectiveCoeffs, RotateFill, RotateOptions,
    affine, affine_sampled, bilinear, bilinear_sampled, expand_replicate, projective,
    projective_sampled, rotate, scale_to_size,
};

/// Target dimensions for scaled test images.
const TARGET_W: u32 = 200;
const TARGET_H: u32 = 200;

/// Helper: scale image to fixed size for consistent transform testing.
fn to_target_size(pix: &leptonica_core::Pix) -> leptonica_core::Pix {
    scale_to_size(pix, TARGET_W, TARGET_H).expect("scale_to_size")
}

/// Test rotate() with area mapping across all supported bit depths (C checks 0-4).
///
/// Verifies that rotation runs without error on 1, 2, 4, 8, 32 bpp images.
#[test]
fn multitype_reg_rotate() {
    let mut rp = RegParams::new("multitype_rotate");

    let images = [
        "test1.png",
        "weasel2.4g.png",
        "weasel4.16g.png",
        "test8.jpg",
        "marge.jpg",
    ];

    let opts = RotateOptions {
        fill: RotateFill::White,
        ..Default::default()
    };

    for img in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_scaled = to_target_size(&pix);

        let rotated = rotate(&pix_scaled, 0.3, &opts).expect("rotate");

        // rotate() may expand the canvas to fit rotated content,
        // so we only verify the operation succeeds and produces non-empty output.
        rp.compare_values(1.0, if rotated.width() > 0 { 1.0 } else { 0.0 }, 0.0);
        rp.compare_values(1.0, if rotated.height() > 0 { 1.0 } else { 0.0 }, 0.0);
    }

    assert!(rp.cleanup(), "multitype rotate test failed");
}

/// Test affine transforms across multiple bit depths (C checks 5-7).
///
/// Verifies affine and affine_sampled on 1, 8, 32 bpp images.
#[test]
fn multitype_reg_affine() {
    let mut rp = RegParams::new("multitype_affine");

    let images = ["test1.png", "test8.jpg", "marge.jpg"];

    // Rotation by 0.2 radians affine matrix
    let matrix = AffineMatrix::rotation(100.0f32, 100.0f32, 0.2f32);

    for img in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_scaled = to_target_size(&pix);

        // Sampled affine
        let result =
            affine_sampled(&pix_scaled, &matrix, AffineFill::White).expect("affine_sampled");
        rp.compare_values(TARGET_W as f64, result.width() as f64, 0.0);
        rp.compare_values(TARGET_H as f64, result.height() as f64, 0.0);

        // Interpolated affine
        let result2 = affine(&pix_scaled, &matrix, AffineFill::White).expect("affine");
        rp.compare_values(TARGET_W as f64, result2.width() as f64, 0.0);
        rp.compare_values(TARGET_H as f64, result2.height() as f64, 0.0);
    }

    assert!(rp.cleanup(), "multitype affine test failed");
}

/// Test projective transforms across multiple bit depths (C checks 8-10).
#[test]
fn multitype_reg_projective() {
    let mut rp = RegParams::new("multitype_projective");

    let images = ["test1.png", "test8.jpg", "marge.jpg"];

    // Small projective transform (slight perspective)
    let src = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 200.0, y: 0.0 },
        Point { x: 200.0, y: 200.0 },
        Point { x: 0.0, y: 200.0 },
    ];
    let dst = [
        Point { x: 10.0, y: 10.0 },
        Point { x: 190.0, y: 5.0 },
        Point { x: 195.0, y: 195.0 },
        Point { x: 5.0, y: 190.0 },
    ];
    let coeffs = ProjectiveCoeffs::from_four_points(src, dst).expect("projective coeffs");

    for img in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_scaled = to_target_size(&pix);

        // Sampled projective
        let result = projective_sampled(&pix_scaled, &coeffs, AffineFill::White)
            .expect("projective_sampled");
        rp.compare_values(TARGET_W as f64, result.width() as f64, 0.0);
        rp.compare_values(TARGET_H as f64, result.height() as f64, 0.0);

        // Interpolated projective
        let result2 = projective(&pix_scaled, &coeffs, AffineFill::White).expect("projective");
        rp.compare_values(TARGET_W as f64, result2.width() as f64, 0.0);
        rp.compare_values(TARGET_H as f64, result2.height() as f64, 0.0);
    }

    assert!(rp.cleanup(), "multitype projective test failed");
}

/// Test bilinear transforms across multiple bit depths (C checks 11-13).
#[test]
fn multitype_reg_bilinear() {
    let mut rp = RegParams::new("multitype_bilinear");

    let images = ["test1.png", "test8.jpg", "marge.jpg"];

    // Small bilinear transform
    let src = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 200.0, y: 0.0 },
        Point { x: 200.0, y: 200.0 },
        Point { x: 0.0, y: 200.0 },
    ];
    let dst = [
        Point { x: 8.0, y: 8.0 },
        Point { x: 192.0, y: 3.0 },
        Point { x: 197.0, y: 197.0 },
        Point { x: 3.0, y: 192.0 },
    ];
    let coeffs = BilinearCoeffs::from_four_points(src, dst).expect("bilinear coeffs");

    for img in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));
        let pix_scaled = to_target_size(&pix);

        // Sampled bilinear
        let result =
            bilinear_sampled(&pix_scaled, &coeffs, AffineFill::White).expect("bilinear_sampled");
        rp.compare_values(TARGET_W as f64, result.width() as f64, 0.0);

        // Interpolated bilinear
        let result2 = bilinear(&pix_scaled, &coeffs, AffineFill::White).expect("bilinear");
        rp.compare_values(TARGET_W as f64, result2.width() as f64, 0.0);
    }

    assert!(rp.cleanup(), "multitype bilinear test failed");
}

/// Test scale_to_size across all image types (C scale checks).
#[test]
fn multitype_reg_scale() {
    let mut rp = RegParams::new("multitype_scale");

    let images = [
        "test1.png",
        "weasel2.4g.png",
        "weasel4.16g.png",
        "test8.jpg",
        "marge.jpg",
    ];

    for img in &images {
        let pix = leptonica_test::load_test_image(img).unwrap_or_else(|_| panic!("load {img}"));

        let scaled = scale_to_size(&pix, TARGET_W, TARGET_H).expect("scale_to_size");
        rp.compare_values(TARGET_W as f64, scaled.width() as f64, 0.0);
        rp.compare_values(TARGET_H as f64, scaled.height() as f64, 0.0);

        // expand_replicate factor 1 = identity
        let identity = expand_replicate(&pix, 1).expect("expand 1x");
        rp.compare_pix(&pix, &identity);
    }

    assert!(rp.cleanup(), "multitype scale test failed");
}

/// Test with alpha-channel image (C check for test-gray-alpha.png).
#[test]
fn multitype_reg_alpha() {
    let mut rp = RegParams::new("multitype_alpha");

    let pix =
        leptonica_test::load_test_image("test-gray-alpha.png").expect("load test-gray-alpha.png");
    assert_eq!(pix.depth(), PixelDepth::Bit32);

    // Remove alpha before transformation
    let pix_no_alpha = pix.remove_alpha().expect("remove alpha");

    let scaled = scale_to_size(&pix_no_alpha, TARGET_W, TARGET_H).expect("scale_to_size");
    rp.compare_values(TARGET_W as f64, scaled.width() as f64, 0.0);

    // Rotate after alpha removal
    let opts = RotateOptions {
        fill: RotateFill::White,
        ..Default::default()
    };
    let rotated = rotate(&scaled, 0.2, &opts).expect("rotate");
    // rotate() may expand the canvas, so only verify non-empty output.
    rp.compare_values(1.0, if rotated.width() > 0 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "multitype alpha test failed");
}
