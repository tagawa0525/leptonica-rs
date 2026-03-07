//! Subpixel rendering regression test
//!
//! Tests subpixel RGB rendering for CRT/LCD displays. The C version
//! converts grayscale and color images to subpixel-rendered RGB using
//! five different component orderings (RGB, BGR, VRGB, VBGR, standard).
//!
//! Not yet migrated: pixConvertGrayToSubpixelRGB and pixConvertToSubpixelRGB
//! are not available in leptonica-transform.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/subpixel_reg.c`

/// Test grayscale-to-subpixel-RGB conversion (C check 0).
///
/// Requires pixConvertGrayToSubpixelRGB which is not available.
#[test]
#[ignore = "not yet implemented: pixConvertGrayToSubpixelRGB not available"]
fn subpixel_reg_gray_to_rgb() {
    // C version:
    // 1. pixConvertTo8(pixs, FALSE) for grayscale source
    // 2. pixConvertGrayToSubpixelRGB(pix, scalex, scaley, order) for
    //    orders: L_SUBPIXEL_ORDER_RGB, BGR, VRGB, VBGR
    // 3. pixScale for standard scale comparison
    // 4. regTestWritePixAndCheck() for each result
}

/// Test color-to-subpixel-RGB conversion (C check 1).
///
/// Requires pixConvertToSubpixelRGB which is not available.
#[test]
#[ignore = "not yet implemented: pixConvertToSubpixelRGB not available"]
fn subpixel_reg_color_to_rgb() {
    // C version:
    // 1. pixConvertTo32(pixs) for 32bpp color source
    // 2. pixConvertToSubpixelRGB(pix, scalex, scaley, order) for
    //    orders: L_SUBPIXEL_ORDER_RGB, BGR, VRGB, VBGR
    // 3. regTestWritePixAndCheck() for each result
}

/// Test 1bpp subpixel with convolution filter (C checks 2-8).
///
/// Requires pixConvertToSubpixelRGB, makeGaussianKernelSep,
/// pixConvolveSep, and makeGaussianKernel which are not available.
#[test]
#[ignore = "not yet implemented: pixConvertToSubpixelRGB / makeGaussianKernelSep not available"]
fn subpixel_reg_bpp1_with_filter() {
    // C version:
    // 1. pixConvertTo8(pixs, FALSE) for 1bpp input
    // 2. pixConvertToSubpixelRGB with separable Gaussian filter
    // 3. pixConvolveSep / pixConvolve for post-processing
    // 4. regTestWritePixAndCheck for checks 2-6
    // 5. regTestComparePix for checks 4, 7
}
