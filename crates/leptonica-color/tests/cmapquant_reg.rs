//! Colormap quantization regression test
//!
//! C版: reference/leptonica/prog/cmapquant_reg.c
//!
//! C版はグレースケール画像を4bppカラーマップ付きに変換し、色を追加して
//! スケール後に再量子化するワークフローをテストしている。
//!
//! C版のテストフロー:
//!   1. pixThresholdTo4bpp(pixs, 6, 1) -- 4bppカラーマップ画像生成
//!   2. pixColorGray(pix1, box, L_PAINT_DARK, ...) -- カラーマップに色追加
//!   3. pixScale(pix1, 1.5, 1.5) -- スケールしてカラーマップを除去
//!   4. pixOctcubeQuantFromCmap(pix2, cmap, ...) -- 指定カラーマップで再量子化
//!   5. pixConvertTo32(pix3) -- RGB変換
//!   6. pixMedianCutQuant(pix4, 0) -- MedianCut再量子化
//!   7. pixFewColorsMedianCutQuantMixed -- 少色数Mixed量子化
//!   8. pixOctcubeQuantMixedWithGray -- グレー混合Octcube量子化
//!   9. pixFixedOctcubeQuant256 -- 固定256色Octcube量子化
//!   10. pixRemoveUnusedColors -- 未使用色除去
//!   11. pix8とpix9のピクセル比較 -- 未使用色除去前後の視覚的一致確認
//!
//! Rust側で利用可能なAPI:
//!   - median_cut_quant / median_cut_quant_simple
//!   - octree_quant / octree_quant_256
//!   - pix_color_gray (32bpp用)
//!   - Colormap検査 (len, get_rgb, is_grayscale, find_nearest)
//!
//! Rust未実装のためスキップ:
//!   - pixThresholdTo4bpp -- Rust未実装
//!   - pixOctcubeQuantFromCmap -- Rust未実装
//!   - pixFewColorsMedianCutQuantMixed -- Rust未実装
//!   - pixOctcubeQuantMixedWithGray -- Rust未実装
//!   - pixFixedOctcubeQuant256 -- octree_quant_256で代替
//!   - pixRemoveUnusedColors -- Rust未実装
//!   - pixScale -- leptonica-transformクレートに依存が必要なためスキップ

use leptonica_color::{
    MedianCutOptions, OctreeOptions, median_cut_quant, median_cut_quant_simple, octree_quant,
    octree_quant_256,
};
use leptonica_core::{Pix, PixelDepth, color};
use leptonica_test::{RegParams, load_test_image};

/// C版の lucasta-frag.jpg に近い32bppテスト画像を生成する。
/// 実画像読み込みを試み、失敗した場合はグレースケール風の合成画像を生成する。
fn load_source_image() -> Pix {
    // lucasta.150.jpg が利用可能か試みる（C版は lucasta-frag.jpg を使用）
    if let Ok(pix) = load_test_image("lucasta.150.jpg") {
        if pix.depth() == PixelDepth::Bit32 {
            return pix;
        }
        // 8bppの場合は32bppに変換
        if pix.depth() == PixelDepth::Bit8 {
            return convert_gray_to_rgb(&pix);
        }
    }
    // test24.jpg をフォールバックとして使用
    if let Ok(pix) = load_test_image("test24.jpg")
        && pix.depth() == PixelDepth::Bit32
    {
        return pix;
    }
    // 合成画像を生成: グレースケール風の文字画像シミュレーション
    create_synthetic_text_image(300, 200)
}

/// 8bppグレースケール画像を32bpp RGBに変換
fn convert_gray_to_rgb(pix: &Pix) -> Pix {
    let w = pix.width();
    let h = pix.height();
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let gray = pix.get_pixel(x, y).unwrap_or(0) as u8;
            let pixel = color::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

/// テキスト画像風の合成画像を生成
fn create_synthetic_text_image(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            // 背景は白っぽく、一部にグレーのテキスト様パターンを配置
            let gray = if (x / 10 + y / 15) % 3 == 0 {
                // テキスト風の暗い部分
                (40 + (x % 7) * 5 + (y % 11) * 3) as u8
            } else {
                // 背景の明るい部分
                (200 + ((x * 3 + y * 7) % 56)) as u8
            };
            let pixel = color::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

/// C版のpixColorGrayに対応:
/// 32bpp画像の指定領域に色を付加する（C版はカラーマップ画像に対して行うが
/// Rust版はpix_color_grayを使って32bpp画像に直接適用する）
#[allow(clippy::too_many_arguments)]
fn apply_color_to_region(pix: &Pix, x0: u32, y0: u32, w: u32, h: u32, r: u8, g: u8, b: u8) -> Pix {
    let pw = pix.width();
    let ph = pix.height();
    let out = Pix::new(pw, ph, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..ph {
        for x in 0..pw {
            let pixel = pix.get_pixel_unchecked(x, y);
            let (pr, pg, pb) = color::extract_rgb(pixel);

            let new_pixel = if x >= x0 && x < x0 + w && y >= y0 && y < y0 + h {
                // C版: pixColorGray with L_PAINT_DARK -- 暗いピクセルに色を付加
                // Gray値が220未満のピクセルに色を付加する
                let avg = ((pr as u32 + pg as u32 + pb as u32) / 3) as u8;
                if avg < 220 {
                    // 暗いピクセルに対してC版のL_PAINT_DARKと同様の着色
                    let factor = avg as f32 / 255.0;
                    let nr = (r as f32 * (1.0 - factor) + pr as f32 * factor) as u8;
                    let ng = (g as f32 * (1.0 - factor) + pg as f32 * factor) as u8;
                    let nb = (b as f32 * (1.0 - factor) + pb as f32 * factor) as u8;
                    color::compose_rgb(nr, ng, nb)
                } else {
                    pixel
                }
            } else {
                pixel
            };

            out_mut.set_pixel_unchecked(x, y, new_pixel);
        }
    }
    out_mut.into()
}

/// C版テストフローの忠実なポート（利用可能なAPIのみ）
///
/// C版テスト番号 (regTestWritePixAndCheck):
///   0: pixThresholdTo4bpp + pixColorGray結果 -- スキップ（pixThresholdTo4bpp未実装）
///   1: pixScale結果 -- スキップ（leptonica-transform依存が必要）
///   2: pixOctcubeQuantFromCmap結果 -- スキップ（pixOctcubeQuantFromCmap未実装）
///   3: pixMedianCutQuant結果 -- テスト可能
///   4: pixFewColorsMedianCutQuantMixed結果 -- スキップ（未実装）
///   5: pixOctcubeQuantMixedWithGray結果 -- スキップ（未実装）
///   6: pixFixedOctcubeQuant256結果 -- octree_quant_256で代替テスト
///   7: pixRemoveUnusedColors結果 -- スキップ（未実装）
///   8: pix8とpix9のピクセル比較 -- スキップ（pixRemoveUnusedColors未実装）
#[test]
fn cmapquant_reg() {
    let mut rp = RegParams::new("cmapquant");

    // 元画像を読み込み（C版: pixs = pixRead("lucasta-frag.jpg")）
    let pixs = load_source_image();
    let w = pixs.width();
    let h = pixs.height();
    eprintln!("  Source image: {}x{} depth={}", w, h, pixs.depth().bits());

    // --- C版テスト0: pixThresholdTo4bpp + pixColorGray ---
    // C版: pixThresholdTo4bpp(pixs, 6, 1) -- Rust未実装のためスキップ
    // C版: pixColorGray(pix1, box, L_PAINT_DARK, 220, 0, 0, 255) -- カラーマップ版は未実装
    // 代わりに32bpp画像に直接色を適用する
    let pix_colored = apply_color_to_region(&pixs, 120, 30, 200, 200, 0, 0, 255);
    eprintln!("  Applied color region (C版: pixColorGray代替)");

    // --- C版テスト1: pixScale(pix1, 1.5, 1.5) ---
    // C版: pixScale() -- leptonica-transformの依存が必要のためスキップ
    // 代わりにそのまま32bpp画像を使用する
    let pix_rgb = &pix_colored;
    eprintln!("  Using colored 32bpp image directly (C版: pixScale代替)");

    // --- C版テスト2: pixOctcubeQuantFromCmap ---
    // C版: pixOctcubeQuantFromCmap(pix2, cmap, MIN_DEPTH, LEVEL, L_EUCLIDEAN_DISTANCE)
    // Rust未実装のためスキップ
    eprintln!("  [SKIP] pixOctcubeQuantFromCmap -- Rust未実装");

    // --- C版テスト3: pixMedianCutQuant(pix4, 0) ---
    // C版はpixConvertTo32で8bpp->32bpp変換後にMedianCutを適用
    // Rust: 32bpp画像に直接median_cut_quantを適用
    {
        let result = median_cut_quant(pix_rgb, &MedianCutOptions::default());
        match result {
            Ok(pix_mc) => {
                // 8bpp出力であること
                rp.compare_values(8.0, pix_mc.depth().bits() as f64, 0.0);
                // カラーマップが存在すること
                let has_cmap = pix_mc.colormap().is_some();
                rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
                // カラーマップが256色以内であること
                if let Some(cmap) = pix_mc.colormap() {
                    rp.compare_values(1.0, if cmap.len() <= 256 { 1.0 } else { 0.0 }, 0.0);
                    eprintln!(
                        "  Test3 (MedianCutQuant default): {} colors, {}x{}",
                        cmap.len(),
                        pix_mc.width(),
                        pix_mc.height()
                    );
                }
                // 出力画像サイズが入力と一致すること
                rp.compare_values(pix_rgb.width() as f64, pix_mc.width() as f64, 0.0);
                rp.compare_values(pix_rgb.height() as f64, pix_mc.height() as f64, 0.0);
            }
            Err(e) => {
                eprintln!("  Test3 FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // --- C版テスト4: pixFewColorsMedianCutQuantMixed ---
    // C版: pixFewColorsMedianCutQuantMixed(pix4, 30, 30, 100, 0, 0, 0)
    // Rust未実装のためスキップ
    eprintln!("  [SKIP] pixFewColorsMedianCutQuantMixed -- Rust未実装");

    // --- C版テスト5: pixOctcubeQuantMixedWithGray ---
    // C版: pixOctcubeQuantMixedWithGray(pix2, 4, 5, 5)
    // Rust未実装のためスキップ
    eprintln!("  [SKIP] pixOctcubeQuantMixedWithGray -- Rust未実装");

    // --- C版テスト6: pixFixedOctcubeQuant256 ---
    // C版: pixFixedOctcubeQuant256(pix2, 0)
    // Rust: octree_quant_256で代替
    {
        let result = octree_quant_256(pix_rgb);
        match result {
            Ok(pix_oct) => {
                rp.compare_values(8.0, pix_oct.depth().bits() as f64, 0.0);
                let has_cmap = pix_oct.colormap().is_some();
                rp.compare_values(1.0, if has_cmap { 1.0 } else { 0.0 }, 0.0);
                if let Some(cmap) = pix_oct.colormap() {
                    rp.compare_values(1.0, if cmap.len() <= 256 { 1.0 } else { 0.0 }, 0.0);
                    eprintln!(
                        "  Test6 (OctreeQuant256): {} colors, {}x{}",
                        cmap.len(),
                        pix_oct.width(),
                        pix_oct.height()
                    );
                }
                rp.compare_values(pix_rgb.width() as f64, pix_oct.width() as f64, 0.0);
                rp.compare_values(pix_rgb.height() as f64, pix_oct.height() as f64, 0.0);
            }
            Err(e) => {
                eprintln!("  Test6 FAILED: {}", e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }

    // --- C版テスト7,8: pixRemoveUnusedColors + 比較 ---
    // C版: pixRemoveUnusedColors(pix9) + regTestComparePix(rp, pix8, pix9)
    // Rust未実装のためスキップ
    eprintln!("  [SKIP] pixRemoveUnusedColors + compare -- Rust未実装");

    assert!(rp.cleanup(), "cmapquant regression test failed");
}

// =============================================================================
// 追加テスト: C版の概念を利用可能なRust APIで検証
// =============================================================================

/// 量子化→カラーマップ検査→再量子化のワークフローテスト
///
/// C版のテスト3に対応: 量子化結果を検査し、再量子化が正しく動作するか検証する。
/// C版ではpixConvertTo32→pixMedianCutQuantだが、
/// Rust版では量子化→カラーマップ検査→別パラメータで再量子化の流れをテストする。
#[test]
fn cmapquant_requantize_workflow() {
    let mut rp = RegParams::new("cmapquant_requant");

    let pixs = load_source_image();
    eprintln!("  Source: {}x{}", pixs.width(), pixs.height());

    // Step 1: 初回量子化（少ない色数で）
    let pix_quant1 = median_cut_quant_simple(&pixs, 32).unwrap();
    let cmap1 = pix_quant1.colormap().expect("should have colormap");
    let n_colors1 = cmap1.len();
    eprintln!("  First quantization: {} colors", n_colors1);
    rp.compare_values(1.0, if n_colors1 <= 32 { 1.0 } else { 0.0 }, 0.0);

    // Step 2: カラーマップの内容を検査
    // 全エントリが有効なRGB値を持つことを確認
    for i in 0..n_colors1 {
        let (_r, _g, _b) = cmap1.get_rgb(i).expect("color should exist");
        // RGB値は0-255の範囲にあるべき（u8型で保証されている）
    }
    eprintln!("  All colormap entries valid");

    // Step 3: 量子化された画像のピクセル値がカラーマップ範囲内であることを確認
    for y in 0..pix_quant1.height() {
        for x in 0..pix_quant1.width() {
            let idx = pix_quant1.get_pixel(x, y).unwrap_or(0) as usize;
            assert!(
                idx < n_colors1,
                "pixel ({},{}) index {} >= colormap size {}",
                x,
                y,
                idx,
                n_colors1
            );
        }
    }
    eprintln!("  All pixel indices within colormap range");

    // Step 4: カラーマップから32bpp画像を再構築してから再量子化
    // C版のpixConvertTo32 + pixMedianCutQuantに対応
    let pix_rgb = pix_quant1.convert_to_32().unwrap();
    rp.compare_values(32.0, pix_rgb.depth().bits() as f64, 0.0);
    rp.compare_values(pix_quant1.width() as f64, pix_rgb.width() as f64, 0.0);

    // Step 5: 再量子化（異なる色数で）
    let pix_quant2 = median_cut_quant_simple(&pix_rgb, 64).unwrap();
    let cmap2 = pix_quant2.colormap().expect("should have colormap");
    eprintln!("  Re-quantization: {} colors", cmap2.len());
    rp.compare_values(1.0, if cmap2.len() <= 64 { 1.0 } else { 0.0 }, 0.0);

    // Step 6: Octree量子化でも同様にテスト
    let pix_quant3 = octree_quant(&pix_rgb, &OctreeOptions { max_colors: 128 }).unwrap();
    let cmap3 = pix_quant3.colormap().expect("should have colormap");
    eprintln!("  Octree re-quantization: {} colors", cmap3.len());
    rp.compare_values(1.0, if cmap3.len() <= 128 { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup(), "cmapquant_requant regression test failed");
}

/// カラーマップの色数検査テスト
///
/// 量子化の結果、指定したmax_colorsを超えないことを各パラメータで検証する。
#[test]
fn cmapquant_colormap_size_limits() {
    let mut rp = RegParams::new("cmapquant_cmap_size");

    let pixs = load_source_image();

    // MedianCut: 様々なmax_colorsでカラーマップサイズ制約を検証
    for &max_colors in &[2u32, 4, 8, 16, 32, 64, 128, 256] {
        let result = median_cut_quant_simple(&pixs, max_colors);
        match result {
            Ok(quantized) => {
                let cmap = quantized.colormap().expect("should have colormap");
                let ok = cmap.len() <= max_colors as usize;
                rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
                if !ok {
                    eprintln!(
                        "  FAIL: median_cut(max={}) produced {} colors",
                        max_colors,
                        cmap.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("  median_cut(max={}) error: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }
    eprintln!("  MedianCut colormap size limits: checked");

    // Octree: 様々なmax_colorsでカラーマップサイズ制約を検証
    for &max_colors in &[4u32, 16, 64, 128, 256] {
        let result = octree_quant(&pixs, &OctreeOptions { max_colors });
        match result {
            Ok(quantized) => {
                let cmap = quantized.colormap().expect("should have colormap");
                let ok = cmap.len() <= max_colors as usize;
                rp.compare_values(1.0, if ok { 1.0 } else { 0.0 }, 0.0);
                if !ok {
                    eprintln!(
                        "  FAIL: octree(max={}) produced {} colors",
                        max_colors,
                        cmap.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("  octree(max={}) error: {}", max_colors, e);
                rp.compare_values(1.0, 0.0, 0.0);
            }
        }
    }
    eprintln!("  Octree colormap size limits: checked");

    assert!(rp.cleanup(), "cmapquant_cmap_size regression test failed");
}

/// カラーマップ内容の品質検査テスト
///
/// C版のpixRemoveUnusedColorsに関連するテスト。
/// 量子化結果のカラーマップ内容が合理的であることを検証する:
/// - 全ピクセルが有効なカラーマップインデックスを使用
/// - find_nearestが一貫した結果を返す
/// - カラーマップ色が元画像の色に近い
#[test]
fn cmapquant_colormap_quality() {
    let mut rp = RegParams::new("cmapquant_cmap_quality");

    // 既知の3色画像を使用
    let pix_3color = create_3color_image(60, 60);

    // 十分な色数で量子化
    let quantized = median_cut_quant_simple(&pix_3color, 16).unwrap();
    let cmap = quantized.colormap().expect("should have colormap");
    eprintln!("  3-color image quantized to {} colors", cmap.len());

    // カラーマップのfind_nearestが元の色に近い結果を返すことを検証
    // 元の色: (255, 0, 0), (0, 255, 0), (0, 0, 255)
    let idx_r = cmap.find_nearest(255, 0, 0).unwrap();
    let idx_g = cmap.find_nearest(0, 255, 0).unwrap();
    let idx_b = cmap.find_nearest(0, 0, 255).unwrap();

    // それぞれ異なるインデックスに割り当てられるべき
    rp.compare_values(1.0, if idx_r != idx_g { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if idx_g != idx_b { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if idx_r != idx_b { 1.0 } else { 0.0 }, 0.0);

    // find_nearestで取得した色が元の色に近いことを検証
    let (cr, cg, cb) = cmap.get_rgb(idx_r).unwrap();
    let dist_r = color_distance(cr, cg, cb, 255, 0, 0);
    rp.compare_values(1.0, if dist_r < 100.0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  Red: nearest=({},{},{}) dist={:.1}", cr, cg, cb, dist_r);

    let (cr, cg, cb) = cmap.get_rgb(idx_g).unwrap();
    let dist_g = color_distance(cr, cg, cb, 0, 255, 0);
    rp.compare_values(1.0, if dist_g < 100.0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  Green: nearest=({},{},{}) dist={:.1}", cr, cg, cb, dist_g);

    let (cr, cg, cb) = cmap.get_rgb(idx_b).unwrap();
    let dist_b = color_distance(cr, cg, cb, 0, 0, 255);
    rp.compare_values(1.0, if dist_b < 100.0 { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  Blue: nearest=({},{},{}) dist={:.1}", cr, cg, cb, dist_b);

    // is_grayscale: 3色画像の量子化結果はグレースケールではないはず
    rp.compare_values(0.0, if cmap.is_grayscale() { 1.0 } else { 0.0 }, 0.0);
    eprintln!("  is_grayscale: {} (expected: false)", cmap.is_grayscale());

    // グレースケール画像の量子化結果はグレースケールであるべき
    let pix_gray = create_gray_gradient(60, 60);
    let quant_gray = median_cut_quant_simple(&pix_gray, 16).unwrap();
    let cmap_gray = quant_gray.colormap().expect("should have colormap");
    rp.compare_values(1.0, if cmap_gray.is_grayscale() { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Gray gradient is_grayscale: {} (expected: true)",
        cmap_gray.is_grayscale()
    );

    assert!(
        rp.cleanup(),
        "cmapquant_cmap_quality regression test failed"
    );
}

/// MedianCutとOctreeの比較テスト
///
/// 同じ入力画像に対して両アルゴリズムが合理的な結果を返すことを検証する。
/// C版テスト3とテスト6の対応。
#[test]
fn cmapquant_algorithm_comparison() {
    let mut rp = RegParams::new("cmapquant_algo_cmp");

    let pixs = load_source_image();

    let max_colors = 64u32;

    // MedianCut量子化
    let mc_result = median_cut_quant_simple(&pixs, max_colors).unwrap();
    let mc_cmap = mc_result.colormap().unwrap();
    let mc_colors = mc_cmap.len();

    // Octree量子化
    let oct_result = octree_quant(&pixs, &OctreeOptions { max_colors }).unwrap();
    let oct_cmap = oct_result.colormap().unwrap();
    let oct_colors = oct_cmap.len();

    eprintln!(
        "  MedianCut: {} colors, Octree: {} colors",
        mc_colors, oct_colors
    );

    // 両方とも指定色数以内
    rp.compare_values(
        1.0,
        if mc_colors <= max_colors as usize {
            1.0
        } else {
            0.0
        },
        0.0,
    );
    rp.compare_values(
        1.0,
        if oct_colors <= max_colors as usize {
            1.0
        } else {
            0.0
        },
        0.0,
    );

    // 両方とも同じサイズの出力を生成
    rp.compare_values(pixs.width() as f64, mc_result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, mc_result.height() as f64, 0.0);
    rp.compare_values(pixs.width() as f64, oct_result.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, oct_result.height() as f64, 0.0);

    // 両方とも8bpp出力
    rp.compare_values(8.0, mc_result.depth().bits() as f64, 0.0);
    rp.compare_values(8.0, oct_result.depth().bits() as f64, 0.0);

    // 全ピクセルがカラーマップ範囲内であることを確認
    let mc_valid = verify_pixel_indices(&mc_result, mc_colors);
    let oct_valid = verify_pixel_indices(&oct_result, oct_colors);
    rp.compare_values(1.0, if mc_valid { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(1.0, if oct_valid { 1.0 } else { 0.0 }, 0.0);
    eprintln!(
        "  Pixel index validation: MC={}, Oct={}",
        mc_valid, oct_valid
    );

    assert!(rp.cleanup(), "cmapquant_algo_cmp regression test failed");
}

/// C版テスト未実装機能の明示的スキップテスト
#[test]
#[ignore = "pixThresholdTo4bpp -- Rust未実装"]
fn cmapquant_threshold_to_4bpp() {
    // C版: pix1 = pixThresholdTo4bpp(pixs, 6, 1)
    // 入力: グレースケール画像
    // 出力: 4bpp（16色以内）カラーマップ付き画像、6レベルのグレー量子化
    // Rust未実装
    unimplemented!("pixThresholdTo4bpp is not implemented in Rust");
}

#[test]
#[ignore = "pixOctcubeQuantFromCmap -- Rust未実装"]
fn cmapquant_octcube_from_cmap() {
    // C版: pix3 = pixOctcubeQuantFromCmap(pix2, cmap, MIN_DEPTH, LEVEL, L_EUCLIDEAN_DISTANCE)
    // 指定されたカラーマップに対して最近傍色で量子化する
    // Rust未実装
    unimplemented!("pixOctcubeQuantFromCmap is not implemented in Rust");
}

#[test]
#[ignore = "pixFewColorsMedianCutQuantMixed -- Rust未実装"]
fn cmapquant_few_colors_mixed() {
    // C版: pix6 = pixFewColorsMedianCutQuantMixed(pix4, 30, 30, 100, 0, 0, 0)
    // 少色数画像に対するMedianCut＋グレー混合量子化
    // Rust未実装
    unimplemented!("pixFewColorsMedianCutQuantMixed is not implemented in Rust");
}

#[test]
#[ignore = "pixOctcubeQuantMixedWithGray -- Rust未実装"]
fn cmapquant_octcube_mixed_gray() {
    // C版: pix7 = pixOctcubeQuantMixedWithGray(pix2, 4, 5, 5)
    // Octcubeとグレーの混合量子化
    // Rust未実装
    unimplemented!("pixOctcubeQuantMixedWithGray is not implemented in Rust");
}

#[test]
#[ignore = "pixRemoveUnusedColors -- Rust未実装"]
fn cmapquant_remove_unused_colors() {
    // C版: pixRemoveUnusedColors(pix9)
    // カラーマップから未使用色を除去し、ピクセルインデックスを再マッピング
    // C版ではpixFixedOctcubeQuant256の結果に適用し、除去前後でピクセルが一致することを確認
    // Rust未実装
    unimplemented!("pixRemoveUnusedColors is not implemented in Rust");
}

// =============================================================================
// ヘルパー関数
// =============================================================================

/// RGB, G, Bの3色のみの画像を生成
fn create_3color_image(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let pixel = if x < w / 3 {
                color::compose_rgb(255, 0, 0)
            } else if x < 2 * w / 3 {
                color::compose_rgb(0, 255, 0)
            } else {
                color::compose_rgb(0, 0, 255)
            };
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

/// グレースケールグラデーション画像を32bppで生成
fn create_gray_gradient(w: u32, h: u32) -> Pix {
    let out = Pix::new(w, h, PixelDepth::Bit32).unwrap();
    let mut out_mut = out.try_into_mut().unwrap();

    for y in 0..h {
        for x in 0..w {
            let gray = ((x * 255) / w.max(1)) as u8;
            let pixel = color::compose_rgb(gray, gray, gray);
            out_mut.set_pixel_unchecked(x, y, pixel);
        }
    }
    out_mut.into()
}

/// RGB色間のユークリッド距離
fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f64 {
    let dr = r1 as f64 - r2 as f64;
    let dg = g1 as f64 - g2 as f64;
    let db = b1 as f64 - b2 as f64;
    (dr * dr + dg * dg + db * db).sqrt()
}

/// 全ピクセルが有効なカラーマップインデックスを使用しているか検証
fn verify_pixel_indices(pix: &Pix, cmap_size: usize) -> bool {
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            let idx = pix.get_pixel(x, y).unwrap_or(0) as usize;
            if idx >= cmap_size {
                eprintln!(
                    "  Invalid pixel index at ({},{}): {} >= {}",
                    x, y, idx, cmap_size
                );
                return false;
            }
        }
    }
    true
}
