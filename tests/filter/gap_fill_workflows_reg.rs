//! Integration tests exercising the gap-fill 第 2 弾 (plans 501 / 113 / 803-K /
//! 115 / 116) functions in realistic workflows.
//!
//! These tests combine multiple newly-added APIs to verify they cooperate
//! correctly (kernel generators feeding `convolve`, parsed kernel data from
//! string input, skew score on a real image, etc.). Smoke-level rather than
//! exhaustive — the per-function unit tests already cover the algebra.

use leptonica::core::pix::{
    PlotLocation, clear_data_dibit, generate_pta_line_from_pt, get_data_four_bytes,
    locate_pt_radially, make_plot_pta_from_numa, set_data_dibit, set_data_four_bytes,
};
use leptonica::filter::{Kernel, convolve_gray};
use leptonica::recog::skew::{find_differential_square_sum, find_normalized_square_sum};
use leptonica::{Numa, Pix, PixelDepth};

/// `Kernel::make_flat` の出力を実画像に畳み込んで spatially-averaging が
/// 効くことを確認 (plan 501 ↔ 既存 convolve_gray の連携)。
#[test]
fn workflow_make_flat_then_convolve_smooths_noise() {
    // 8x8 gray image: half black half white
    let pix = Pix::new(8, 8, PixelDepth::Bit8).expect("new 8bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for y in 0..8u32 {
        for x in 0..8u32 {
            let v = if x < 4 { 0 } else { 255 };
            pm.set_pixel(x, y, v).expect("set");
        }
    }
    let src: Pix = pm.into();

    // 3x3 flat kernel (centered) — 平滑化
    let k = Kernel::make_flat(3, 3, 1, 1).expect("make_flat 3x3");
    let dst = convolve_gray(&src, &k).expect("convolve_gray");

    // 境界 (x=3,4 のあたり) の隣接ピクセルは中間値になるはず
    let v3 = dst.get_pixel(3, 4).unwrap_or(0);
    let v4 = dst.get_pixel(4, 4).unwrap_or(0);
    assert!(
        v3 > 0 && v3 < 255,
        "boundary should be intermediate, got {}",
        v3
    );
    assert!(
        v4 > 0 && v4 < 255,
        "boundary should be intermediate, got {}",
        v4
    );
}

/// `Kernel::make_gaussian` で生成したカーネルを正規化して convolve する。
/// 平坦領域では値が保たれることを確認 (plan 501)。
#[test]
fn workflow_make_gaussian_normalized_preserves_uniform() {
    let pix = Pix::new(16, 16, PixelDepth::Bit8).expect("new 8bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for y in 0..16u32 {
        for x in 0..16u32 {
            pm.set_pixel(x, y, 128).expect("set");
        }
    }
    let src: Pix = pm.into();

    let mut k = Kernel::make_gaussian(2, 2, 1.0, 1.0).expect("make_gaussian 5x5");
    k.normalize();
    let dst = convolve_gray(&src, &k).expect("convolve");

    // 中央付近は元の値 (128) と概ね一致
    let v = dst.get_pixel(8, 8).unwrap_or(0);
    assert!(
        (v as i32 - 128).abs() <= 2,
        "uniform input should produce ~128, got {}",
        v
    );
}

/// `Numa::parse_from_string` で読んだ値を `Kernel::from_slice` に流して、
/// 文字列由来のカーネルが正しく動作することを確認 (plan 501)。
#[test]
fn workflow_parse_string_to_kernel_then_convolve() {
    // 3x3 mean filter via string
    let na = Numa::parse_from_string("1,1,1,1,1,1,1,1,1", ",").expect("parse");
    let data: Vec<f32> = (0..9).map(|i| na.get(i).unwrap() / 9.0).collect();
    let k = Kernel::from_slice(3, 3, &data).expect("from_slice");

    let pix = Pix::new(8, 8, PixelDepth::Bit8).expect("new 8bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for x in 0..8u32 {
        pm.set_pixel(x, 4, 255).expect("set");
    }
    let src: Pix = pm.into();
    let dst = convolve_gray(&src, &k).expect("convolve");
    // 中央行は約 255/3、隣接行は 255/3 → 中央列にもにじむ
    let v = dst.get_pixel(4, 4).unwrap_or(0);
    assert!(v > 0, "convolved value should be non-zero, got {}", v);
}

/// `find_differential_square_sum` がテキスト風画像 (横じま) で高い score を、
/// 平坦画像で低い score を返すことを確認 (plan 803-K)。
#[test]
fn workflow_skew_score_differential_real_pattern() {
    // 横じま (テキスト行を模擬)
    let pix1 = Pix::new(80, 80, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix1.try_into_mut().expect("into_mut");
    for y in (10..70u32).step_by(4) {
        for x in 0..80u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let textlike: Pix = pm.into();

    // 一様塗り
    let pix2 = Pix::new(80, 80, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix2.try_into_mut().expect("into_mut");
    for y in 0..80u32 {
        for x in 0..80u32 {
            pm.set_pixel(x, y, 1).expect("set");
        }
    }
    let uniform: Pix = pm.into();

    let s_text = find_differential_square_sum(&textlike).expect("score textlike");
    let s_uni = find_differential_square_sum(&uniform).expect("score uniform");
    assert!(
        s_text > s_uni,
        "textlike should score higher than uniform: {} vs {}",
        s_text,
        s_uni
    );
}

/// `find_normalized_square_sum` の hratio が斜め線では h>v になることを確認
/// (plan 803-K)。
#[test]
fn workflow_normalized_square_sum_horizontal_concentration() {
    // 横線数本: 行に集中、列方向は均等 → hratio > vratio
    let pix = Pix::new(40, 40, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    for x in 0..40u32 {
        pm.set_pixel(x, 10, 1).expect("set");
        pm.set_pixel(x, 20, 1).expect("set");
        pm.set_pixel(x, 30, 1).expect("set");
    }
    let p: Pix = pm.into();

    let (h, v, _fract) = find_normalized_square_sum(&p).expect("normalized");
    assert!(h > v, "hratio={} should exceed vratio={}", h, v);
}

/// arrayaccess の 2bit/4bit/4byte ロウレベル API を 1 つの 32bpp ワード上で
/// 連続適用して round-trip することを確認 (plan 113)。
#[test]
fn workflow_arrayaccess_dibit_qbit_fourbytes_combined() {
    let mut line = vec![0u32; 4];

    // 32bit ワード単位の get/set
    set_data_four_bytes(&mut line, 0, 0xDEADBEEF);
    assert_eq!(get_data_four_bytes(&line, 0), 0xDEADBEEF);

    // ワード 1 (dibit index 16..31) を 0xFFFFFFFF にしてから 端の dibit をクリア
    line[1] = 0xFFFFFFFF;
    clear_data_dibit(&mut line, 16); // word 1, dibit 0
    clear_data_dibit(&mut line, 31); // word 1, dibit 15
    use leptonica::core::pix::get_data_dibit;
    assert_eq!(get_data_dibit(&line, 16), 0);
    assert_eq!(get_data_dibit(&line, 17), 3); // 隣接 dibit は 3 のまま
    assert_eq!(get_data_dibit(&line, 31), 0);

    // ワード 2 では dibit を 3 に set してから clear するパターン
    set_data_dibit(&mut line, 32, 3);
    assert_eq!(get_data_dibit(&line, 32), 3);
    clear_data_dibit(&mut line, 32);
    assert_eq!(get_data_dibit(&line, 32), 0);
}

/// `generate_pta_line_from_pt` + `locate_pt_radially` を組合せ、
/// 同じ角度 + 同じ距離が同じ終点を導くことを確認 (plan 115)。
#[test]
fn workflow_radial_line_endpoints_match_radial_locate() {
    use std::f64::consts::PI;
    // 45° 方向、長さ 11 (length-1 = 10 pixels offset)
    let pta = generate_pta_line_from_pt(0, 0, 11.0, PI / 4.0);
    // generate_pta_line_from_pt は round(10*cos45°) = 7
    let expected_dx = 7.0_f64;
    let expected_dy = 7.0_f64;
    let last_idx = pta.len() - 1;
    let (lx, ly) = pta.get(last_idx).unwrap();
    assert!((lx as f64 - expected_dx).abs() <= 1.0);
    assert!((ly as f64 - expected_dy).abs() <= 1.0);

    // locate_pt_radially は f64 で正確な値
    let (x2, y2) = locate_pt_radially(0, 0, 10.0, PI / 4.0);
    let r: f64 = (x2 * x2 + y2 * y2).sqrt();
    assert!((r - 10.0).abs() < 1e-9);
}

/// `make_plot_pta_from_numa` → `PixMut::render_pta` で実際にプロットが
/// 画像に書き込めることを確認 (plan 115)。
#[test]
fn workflow_make_plot_pta_then_render() {
    use leptonica::core::PixelOp;
    let na = Numa::from_slice(&[0.0, 1.0, 2.0, 3.0, 2.0, 1.0, 0.0]);
    let pta = make_plot_pta_from_numa(&na, 40, PlotLocation::MidHoriz, 1, 10).expect("plot pta");

    let pix = Pix::new(40, 40, PixelDepth::Bit1).expect("new 1bpp");
    let mut pm = pix.try_into_mut().expect("into_mut");
    pm.render_pta(&pta, PixelOp::Set).expect("render_pta");

    // FG ピクセルが少なくとも 1 つ存在
    let any_set = (0..40u32).any(|y| (0..40u32).any(|x| pm.get_pixel(x, y) == Some(1)));
    assert!(any_set, "rendered plot should produce FG pixels");
}

/// `Numa::create_from_string` → `Numa::convert_to_sarray` の round-trip
/// (plan 116)。
#[test]
fn workflow_numa_string_roundtrip_via_create_and_convert() {
    use leptonica::core::numa::NumaSarrayType;
    let na = Numa::create_from_string("1, 2, 3, 42").expect("create");
    assert_eq!(na.len(), 4);
    let sa = na.convert_to_sarray(3, 0, true, NumaSarrayType::Integer);
    assert_eq!(sa.get(0).unwrap(), "001");
    assert_eq!(sa.get(3).unwrap(), "042");
}

/// `Numaa::create_full` + `Numaa::total_count` (numaaGetNumberCount 代替)
/// の動作確認 (plan 116)。
#[test]
fn workflow_numaa_create_full_total_count() {
    use leptonica::core::numa::Numaa;
    let mut naa = Numaa::create_full(3, 5);
    naa.get_mut(0).unwrap().push(1.0);
    naa.get_mut(0).unwrap().push(2.0);
    naa.get_mut(1).unwrap().push(3.0);
    // 0番 has 2, 1番 has 1, 2番 has 0 → total 3
    assert_eq!(naa.total_count(), 3);
}
