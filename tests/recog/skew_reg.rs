//! Skew detection regression test
//!
//! C版: prog/skew_reg.c
//! テキスト画像のスキュー(傾き)検出と補正をテスト。

use crate::common::{RegParams, load_test_image};
use leptonica::PixelDepth;
use leptonica::io::ImageFormat;
use leptonica::recog::SkewDetectOptions;
use leptonica::recog::skew::{find_skew, find_skew_and_deskew};

#[test]
fn skew_reg() {
    let mut rp = RegParams::new("skew");
    let display_mode = crate::common::is_display_mode();

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("Image: {}x{}", w, h);

    let options = SkewDetectOptions::default();
    if display_mode {
        let fast = pixs
            .clip_rectangle(0, 0, (w / 2).max(64), (h / 2).max(64))
            .expect("clip display fast");
        let result = find_skew(&fast, &options).expect("find_skew display");
        rp.compare_values(1.0, if result.confidence >= 0.0 { 1.0 } else { 0.0 }, 0.0);
        assert!(rp.cleanup(), "skew regression test failed");
        return;
    }

    // --- Test 1: Find skew with default options ---
    eprintln!("=== Skew detection ===");
    let result = find_skew(&pixs, &options).expect("find_skew");
    eprintln!("  Detected skew angle: {:.3}°", result.angle);
    eprintln!("  Confidence: {:.3}", result.confidence);

    // feyn.tif should have near-zero skew
    rp.compare_values(0.0, result.angle as f64, 1.0); // within 1 degree
    rp.compare_values(1.0, if result.confidence > 0.0 { 1.0 } else { 0.0 }, 0.0);

    // --- Test 2: Find skew and deskew ---
    eprintln!("=== Deskew ===");
    let (deskewed, skew_result) =
        find_skew_and_deskew(&pixs, &options).expect("find_skew_and_deskew");
    // Deskew may expand image dimensions due to rotation
    rp.compare_values(1.0, if deskewed.width() >= w { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if deskewed.height() >= h { 1.0 } else { 0.0 }, 0.0);

    rp.write_pix_and_check(&deskewed, ImageFormat::Tiff)
        .expect("write deskewed skew");

    eprintln!(
        "  Deskewed: {}x{}, angle={:.3}°",
        deskewed.width(),
        deskewed.height(),
        skew_result.angle
    );

    // --- Test 3: Deskewed image should have less skew ---
    let result2 = find_skew(&deskewed, &options).expect("find_skew on deskewed");
    let less_skew = result2.angle.abs() <= result.angle.abs() + 0.1;
    rp.compare_values(1.0, if less_skew { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Deskewed skew: {:.3}° (original: {:.3}°)",
        result2.angle, result.angle
    );

    // --- Test 4: Zero-angle detection on text ---
    // feyn-fract.tif is a fractal, not text, so skew detection
    // results may vary. Test that it at least returns a result.
    let pixf = load_test_image("feyn-fract.tif").expect("load feyn-fract.tif");
    let result_f = find_skew(&pixf, &options);
    rp.compare_values(1.0, if result_f.is_ok() { 1.0 } else { 0.0 }, 0.0);

    // NOTE: C版では回転した画像でのスキュー検出テストも含まれるが、
    // ここではleptonica-transformへの依存を避けるためスキップ

    assert!(rp.cleanup(), "skew regression test failed");
}

// =====================================================================
// gap-fill 第2弾 (plan 803-K): pixFindDifferentialSquareSum +
// pixFindNormalizedSquareSum
// =====================================================================

use leptonica::Pix;
use leptonica::recog::skew::{find_differential_square_sum, find_normalized_square_sum};

/// C: pixFindDifferentialSquareSum — 平坦画像では sum=0 になる
#[test]

fn skew_reg_differential_square_sum_uniform() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    // 全 0 (BG only): すべての行で count=0 → diff=0 → sum=0
    let s = find_differential_square_sum(&pix).expect("differential square sum");
    assert!(s.abs() < 1e-3, "uniform image should give sum~0, got {}", s);
}

/// C: pixFindDifferentialSquareSum — 縞模様では sum > 0
#[test]

fn skew_reg_differential_square_sum_stripes() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    // odd rows fully on
    for y in (1..64u32).step_by(2) {
        for x in 0..64u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let pix2: Pix = pm.into();
    let s = find_differential_square_sum(&pix2).expect("differential");
    assert!(s > 0.0, "striped image should give sum>0, got {}", s);
}

/// C: pixFindNormalizedSquareSum — 全 0 画像では fract=0
#[test]

fn skew_reg_normalized_square_sum_empty() {
    let pix = Pix::new(64, 64, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let (h, v, f) = find_normalized_square_sum(&pix).expect("normalized");
    assert_eq!(h, 0.0);
    assert_eq!(v, 0.0);
    assert_eq!(f, 0.0);
}

/// C: pixFindNormalizedSquareSum — 一様塗り画像では hratio = vratio = 1.0
#[test]

fn skew_reg_normalized_square_sum_uniform_full() {
    let pix = Pix::new(32, 32, leptonica::PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for y in 0..32u32 {
        for x in 0..32u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let pix2: Pix = pm.into();
    let (h, v, f) = find_normalized_square_sum(&pix2).expect("normalized");
    assert!(
        (h - 1.0).abs() < 1e-3,
        "uniform: hratio = {}, expected 1.0",
        h
    );
    assert!(
        (v - 1.0).abs() < 1e-3,
        "uniform: vratio = {}, expected 1.0",
        v
    );
    assert!(
        (f - 1.0).abs() < 1e-3,
        "uniform: fract = {}, expected 1.0",
        f
    );
}

/// C: pixFindDifferentialSquareSum — non-1bpp 入力はエラー
#[test]
fn skew_reg_differential_square_sum_non_1bpp_errors() {
    let pix = Pix::new(32, 32, leptonica::PixelDepth::Bit8).expect("new 8bpp");
    assert!(find_differential_square_sum(&pix).is_err());
}

/// C: pixFindNormalizedSquareSum — non-1bpp 入力はエラー
#[test]
fn skew_reg_normalized_square_sum_non_1bpp_errors() {
    let pix = Pix::new(32, 32, leptonica::PixelDepth::Bit32).expect("new 32bpp");
    assert!(find_normalized_square_sum(&pix).is_err());
}

/// C-compat: `prog/skew_reg.c`, all 7 outputs.
///
/// `feyn.tif` is lossless and C writes every output as PNG, so the whole
/// program is bit-exactly comparable. Indices 2, 4 and 5 depend on the skew
/// angle found by `pixFindSkewSweepAndSearchScorePivot` /
/// `pixFindSkewOrthogonalRange`, so this also pins those search algorithms.
#[test]
#[ignore = "not yet implemented"]
fn skew_c_compat() {
    use leptonica::core::{InitColor, Pix, Pixa, RopOp};
    use leptonica::recog::skew::{
        SkewPivot, find_skew_orthogonal_range, find_skew_sweep_and_search_score_pivot,
    };
    use leptonica::transform::{
        RotateEmbed, RotateFill, RotateMethod, RotateOptions, reduce_rank_binary_cascade, rotate,
        rotate_by_sampling,
    };

    if crate::common::is_display_mode() {
        return;
    }

    const BORDER: u32 = 150;
    // C uses the literal 3.1415926535, not M_PI.
    #[allow(clippy::approx_constant, clippy::excessive_precision)]
    let deg2rad = 3.1415926535_f32 / 180.0;

    let mut rp = RegParams::new("skew_c");
    let mut pixa = Pixa::new();

    let pixs = load_test_image("feyn.tif").expect("load feyn.tif");
    // C: pixSetOrClearBorder(pixs, 100, 250, 100, 0, PIX_CLR)
    let pixs = {
        let mut m = pixs.try_into_mut().expect("into mut");
        m.set_or_clear_border(100, 250, 100, 0, InitColor::White);
        Pix::from(m)
    };

    // C: pixReduceRankBinaryCascade(pixs, 2, 2, 0, 0)
    let pixb1 = reduce_rank_binary_cascade(&pixs, &[2, 2]).expect("rank cascade");
    rp.write_pix_and_check(&pixb1, ImageFormat::Png)
        .expect("check: reduced source");

    let pixb2 = pixb1.add_border(BORDER, 0).expect("add border");
    let (w, h) = (pixb2.width(), pixb2.height());
    pixa.push(pixb2.clone());

    // C: rotate by sampling, 40 degrees about the center of pixb2.
    let pixr = rotate_by_sampling(
        &pixb2,
        (w / 2) as i32,
        (h / 2) as i32,
        deg2rad * 40.0,
        RotateFill::White,
    )
    .expect("rotate 40");
    rp.write_pix_and_check(&pixr, ImageFormat::Png)
        .expect("check: rotated 40 deg");
    pixa.push(pixr.clone());

    // C: pixFindSkewSweepAndSearchScorePivot(pixr, .., 1, 1, 0.0, 45.0, 2.0, 0.03, CENTER)
    let opts = SkewDetectOptions {
        sweep_range: 45.0,
        sweep_delta: 2.0,
        min_bs_delta: 0.03,
        sweep_reduction: 1,
        bs_reduction: 1,
    };
    let (angle, _conf, _score) =
        find_skew_sweep_and_search_score_pivot(&pixr, &opts, SkewPivot::Center)
            .expect("sweep and search");

    let pixf = rotate_by_sampling(
        &pixr,
        (w / 2) as i32,
        (h / 2) as i32,
        deg2rad * angle,
        RotateFill::White,
    )
    .expect("deskew rotate");
    let pixd = pixf.remove_border(BORDER).expect("remove border");
    rp.write_pix_and_check(&pixd, ImageFormat::Png)
        .expect("check: deskewed");
    pixa.push(pixd);

    // C: pixRotate(pixb1, 37 deg, SAMPLING, WHITE, w, h) with pixb1's own dims.
    let (w, h) = (pixb1.width(), pixb1.height());
    let ropts = RotateOptions {
        method: RotateMethod::Sampling,
        fill: RotateFill::White,
        center_x: None,
        center_y: None,
        embed: RotateEmbed::Explicit(w, h),
    };
    let pixr = rotate(&pixb1, deg2rad * 37.0, &ropts).expect("rotate 37");
    rp.write_pix_and_check(&pixr, ImageFormat::Png)
        .expect("check: rotated 37 deg");
    pixa.push(pixr.clone());

    // C: pixFindSkewOrthogonalRange(pixr, .., 2, 1, 47.0, 1.0, 0.03, 0.0)
    let (angle, _conf) =
        find_skew_orthogonal_range(&pixr, 2, 1, 47.0, 1.0, 0.03, 0.0).expect("orthogonal range");
    let pixd = rotate(&pixr, deg2rad * angle, &ropts).expect("orthogonal deskew");
    rp.write_pix_and_check(&pixd, ImageFormat::Png)
        .expect("check: orthogonally deskewed");

    // C: crop the (larger) rotated result back to pixb1's size, centered.
    let (wd, hd) = (pixd.width(), pixd.height());
    let pixc = Pix::new(w, h, PixelDepth::Bit1).expect("new 1bpp");
    let pixc = {
        let mut m = pixc.try_into_mut().expect("into mut");
        m.rop_region_inplace(
            0,
            0,
            w,
            h,
            RopOp::Src,
            &pixd,
            (wd as i32 - w as i32) / 2,
            (hd as i32 - h as i32) / 2,
        )
        .expect("rasterop");
        Pix::from(m)
    };
    rp.write_pix_and_check(&pixc, ImageFormat::Png)
        .expect("check: recentered");
    pixa.push(pixc);

    let tiled = pixa
        .display_tiled_in_columns(3, 0.5, 20, 3)
        .expect("tile skew");
    rp.write_pix_and_check(&tiled, ImageFormat::Png)
        .expect("check: skew summary");

    assert!(rp.cleanup(), "skew C-compat test failed");
}
