//! Bitmap font and text operations regression test
//!
//! Tests Bmf creation, glyph access, text measurement, and rendering.
//!
//! # See also
//!
//! C Leptonica: `reference/leptonica/prog/textops_reg.c`

use crate::common::RegParams;
use leptonica::core::bmf;
use leptonica::{Bmf, Pix, Pixa, PixelDepth, TextLocation};

// ==========================================================================
// Test 1: Bmf creation at various sizes
// ==========================================================================

#[test]
fn bmf_create_sizes() {
    let mut rp = RegParams::new("bmf_create_sizes");

    for &size in &[4, 6, 8, 10, 12, 14, 16, 20] {
        let bmf = Bmf::new(size).unwrap();
        rp.compare_values(size as f64, bmf.size() as f64, 0.0);

        // Should have 95 glyphs (ASCII 32-126)
        let pixa = bmf.get_font_pixa();
        rp.compare_values(95.0, pixa.len() as f64, 0.0);

        // Line height should be > 0
        assert!(
            bmf.line_height() > 0,
            "line_height must be > 0 for size {}",
            size
        );
        assert!(
            bmf.kern_width() > 0,
            "kern_width must be > 0 for size {}",
            size
        );
    }
}

// ==========================================================================
// Test 2: Get individual character glyphs
// ==========================================================================

#[test]
fn bmf_get_pix_chars() {
    let mut rp = RegParams::new("bmf_get_pix_chars");
    let bmf = Bmf::new(10).unwrap();

    // Printable ASCII should return Some
    let pix_a = bmf.get_pix('A');
    assert!(pix_a.is_some(), "should get glyph for 'A'");
    let pix_a = pix_a.unwrap();
    rp.compare_values(1.0, pix_a.depth().bits() as f64, 0.0); // 1bpp
    assert!(pix_a.width() > 0);
    assert!(pix_a.height() > 0);

    // Space should return Some (it's printable)
    assert!(bmf.get_pix(' ').is_some(), "should get glyph for space");

    // Newline should return None
    assert!(bmf.get_pix('\n').is_none(), "newline should return None");

    // Non-ASCII should return None
    assert!(
        bmf.get_pix('\x01').is_none(),
        "control char should return None"
    );
    assert!(bmf.get_pix('€').is_none(), "non-ASCII should return None");
}

// ==========================================================================
// Test 3: Character widths and baselines
// ==========================================================================

#[test]
fn bmf_widths_and_baselines() {
    let mut rp = RegParams::new("bmf_widths_baselines");
    let bmf = Bmf::new(10).unwrap();

    // Width of 'A' should be > 0
    let w_a = bmf.get_width('A');
    assert!(w_a.is_some());
    assert!(w_a.unwrap() > 0);

    // Width of 'M' should be >= width of 'i' (proportional)
    let w_m = bmf.get_width('M').unwrap();
    let w_i = bmf.get_width('i').unwrap();
    assert!(
        w_m >= w_i,
        "M ({}) should be at least as wide as i ({})",
        w_m,
        w_i
    );

    // Baseline should exist
    let bl = bmf.get_baseline('A');
    assert!(bl.is_some());
    assert!(bl.unwrap() > 0);
    rp.compare_values(1.0, 1.0, 0.0); // placeholder to ensure rp is used
}

// ==========================================================================
// Test 4: String width measurement
// ==========================================================================

#[test]
fn bmf_string_width() {
    let mut rp = RegParams::new("bmf_string_width");
    let bmf = Bmf::new(10).unwrap();

    // Empty string has width 0
    rp.compare_values(0.0, bmf.get_string_width("") as f64, 0.0);

    // Single char width should match get_width
    let w_a = bmf.get_width('A').unwrap();
    rp.compare_values(w_a as f64, bmf.get_string_width("A") as f64, 0.0);

    // "AB" width = w_A + kern + w_B
    let w_b = bmf.get_width('B').unwrap();
    let expected_ab = w_a + bmf.kern_width() + w_b;
    rp.compare_values(expected_ab as f64, bmf.get_string_width("AB") as f64, 0.0);

    // Free function should match method
    let w = bmf::bmf_get_string_width(&bmf, "Hello");
    rp.compare_values(bmf.get_string_width("Hello") as f64, w as f64, 0.0);
}

// ==========================================================================
// Test 5: Word widths
// ==========================================================================

#[test]
fn bmf_word_widths() {
    let mut rp = RegParams::new("bmf_word_widths");
    let bmf = Bmf::new(10).unwrap();

    let widths = bmf.get_word_widths("Hello World");
    rp.compare_values(2.0, widths.len() as f64, 0.0);
    assert_eq!(widths[0], bmf.get_string_width("Hello"));
    assert_eq!(widths[1], bmf.get_string_width("World"));

    // Free function
    let widths2 = bmf::bmf_get_word_widths(&bmf, "one two three");
    rp.compare_values(3.0, widths2.len() as f64, 0.0);
}

// ==========================================================================
// Test 6: Line breaking
// ==========================================================================

#[test]
fn bmf_line_strings() {
    let mut rp = RegParams::new("bmf_line_strings");
    let bmf = Bmf::new(10).unwrap();

    // Short text that fits in one line
    let (lines, h) = bmf.get_line_strings("Hello World", 1000, 0);
    rp.compare_values(1.0, lines.len() as f64, 0.0);
    assert_eq!(lines[0], "Hello World");
    assert!(h > 0);

    // Very narrow width should force wrapping
    let narrow_w = bmf.get_string_width("Hello") + 5;
    let (lines, _) = bmf.get_line_strings("Hello World", narrow_w, 0);
    assert!(lines.len() >= 2, "should wrap to multiple lines");

    // Free function
    let lines2 = bmf::bmf_get_line_strings("a b c", 1000, 0, &bmf);
    rp.compare_values(1.0, lines2.len() as f64, 0.0);
}

// ==========================================================================
// Test 7: Render a single text line
// ==========================================================================

#[test]
fn bmf_set_textline() {
    let _rp = RegParams::new("bmf_set_textline");
    let bmf = Bmf::new(10).unwrap();

    // 1bpp image
    let pix = Pix::new(200, 50, PixelDepth::Bit1).unwrap();
    let (result, width) = bmf.set_textline(&pix, "Test", 10, 20, 1).unwrap();
    assert_eq!(result.width(), 200);
    assert_eq!(result.height(), 50);
    assert!(width > 0, "rendered text should have positive width");

    // 8bpp image
    let pix8 = Pix::new(200, 50, PixelDepth::Bit8).unwrap();
    let (result8, _) = bmf.set_textline(&pix8, "Test", 10, 20, 128).unwrap();
    assert_eq!(result8.width(), 200);

    // 32bpp image
    let pix32 = Pix::new(200, 50, PixelDepth::Bit32).unwrap();
    let (result32, _) = bmf
        .set_textline(&pix32, "Test", 10, 20, 0xFF000000)
        .unwrap();
    assert_eq!(result32.width(), 200);
}

// ==========================================================================
// Test 8: Render text block
// ==========================================================================

#[test]
fn bmf_set_textblock() {
    let _rp = RegParams::new("bmf_set_textblock");
    let bmf = Bmf::new(10).unwrap();

    let pix = Pix::new(300, 200, PixelDepth::Bit8).unwrap();
    let result = bmf
        .set_textblock(
            &pix,
            "The quick brown fox jumps over the lazy dog",
            0,
            10,
            20,
            200,
            0,
        )
        .unwrap();

    assert_eq!(result.width(), 300);
    assert_eq!(result.height(), 200);
}

// ==========================================================================
// Test 9: Add text lines to image
// ==========================================================================

#[test]
fn bmf_add_textlines() {
    let _rp = RegParams::new("bmf_add_textlines");
    let bmf = Bmf::new(10).unwrap();
    let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();

    // Add text above
    let result = bmf
        .add_textlines(&pix, "Above", 0, TextLocation::Above)
        .unwrap();
    assert!(result.height() > pix.height(), "height should increase");
    assert_eq!(result.width(), pix.width());

    // Add text below
    let result = bmf
        .add_textlines(&pix, "Below", 0, TextLocation::Below)
        .unwrap();
    assert!(result.height() > pix.height(), "height should increase");

    // Add text left
    let result = bmf
        .add_textlines(&pix, "Left", 0, TextLocation::Left)
        .unwrap();
    assert!(result.width() > pix.width(), "width should increase");

    // Add text right
    let result = bmf
        .add_textlines(&pix, "Right", 0, TextLocation::Right)
        .unwrap();
    assert!(result.width() > pix.width(), "width should increase");
}

// ==========================================================================
// Test 10: Pixa text number
// ==========================================================================

#[test]
fn bmf_pixa_add_text_number() {
    let _rp = RegParams::new("bmf_pixa_text_number");
    let bmf = Bmf::new(8).unwrap();

    let mut pixa = Pixa::new();
    for _ in 0..3 {
        pixa.push(Pix::new(50, 40, PixelDepth::Bit8).unwrap());
    }

    let result = bmf
        .pixa_add_text_number(&pixa, None, 0, TextLocation::Below)
        .unwrap();
    assert_eq!(result.len(), 3);

    // Each image should be taller (text added below)
    for i in 0..3 {
        let orig = pixa.get(i).unwrap();
        let labeled = result.get(i).unwrap();
        assert!(
            labeled.height() > orig.height(),
            "image {} should be taller after adding number",
            i
        );
    }
}

// ==========================================================================
// Test 11: Pixa add text lines
// ==========================================================================

#[test]
fn bmf_pixa_add_textlines() {
    let _rp = RegParams::new("bmf_pixa_add_textlines");
    let bmf = Bmf::new(8).unwrap();

    let mut pixa = Pixa::new();
    pixa.push(Pix::new(60, 40, PixelDepth::Bit8).unwrap());
    pixa.push(Pix::new(60, 40, PixelDepth::Bit8).unwrap());

    let texts = vec!["First".to_string(), "Second".to_string()];
    let result = bmf
        .pixa_add_textlines(&pixa, Some(&texts), 0, TextLocation::Above)
        .unwrap();
    assert_eq!(result.len(), 2);
}

// ==========================================================================
// Test 12: Pixa add pix with text
// ==========================================================================

#[test]
fn bmf_pixa_add_pix_with_text() {
    let _rp = RegParams::new("bmf_pixa_add_pix_with_text");
    let bmf = Bmf::new(8).unwrap();

    let mut pixa = Pixa::new();
    let pix = Pix::new(100, 80, PixelDepth::Bit8).unwrap();

    bmf.pixa_add_pix_with_text(&mut pixa, &pix, 1, "Label", 0, TextLocation::Below)
        .unwrap();
    assert_eq!(pixa.len(), 1);

    // With reduction
    bmf.pixa_add_pix_with_text(&mut pixa, &pix, 2, "Reduced", 0, TextLocation::Below)
        .unwrap();
    assert_eq!(pixa.len(), 2);
    // Reduced image should be smaller
    let reduced = pixa.get(1).unwrap();
    assert!(reduced.width() <= pix.width());
}

// ==========================================================================
// Test 13: Empty/edge cases
// ==========================================================================

#[test]
fn bmf_edge_cases() {
    let mut rp = RegParams::new("bmf_edge_cases");
    let bmf = Bmf::new(10).unwrap();

    // Empty text
    rp.compare_values(0.0, bmf.get_string_width("") as f64, 0.0);
    let (lines, h) = bmf.get_line_strings("", 100, 0);
    assert!(lines.is_empty());
    rp.compare_values(0.0, h as f64, 0.0);

    // Empty text for add_textlines returns clone
    let pix = Pix::new(50, 50, PixelDepth::Bit8).unwrap();
    let result = bmf.add_textlines(&pix, "", 0, TextLocation::Below).unwrap();
    assert_eq!(result.width(), pix.width());
    assert_eq!(result.height(), pix.height());
}

// ==========================================================================
// Test 14: Font pixa accessor
// ==========================================================================

#[test]
fn bmf_get_font_pixa() {
    let mut rp = RegParams::new("bmf_get_font_pixa");
    let bmf = Bmf::new(12).unwrap();

    let pixa = bmf.get_font_pixa();
    rp.compare_values(95.0, pixa.len() as f64, 0.0);

    // First glyph (space) should be valid
    let space_pix = pixa.get(0).unwrap();
    assert_eq!(space_pix.depth(), PixelDepth::Bit1);
}

// ==========================================================================
// Test 15: Glyph pixels are actually set
// ==========================================================================

#[test]
fn bmf_glyph_has_pixels() {
    let _rp = RegParams::new("bmf_glyph_has_pixels");
    let bmf = Bmf::new(10).unwrap();

    // 'A' glyph should have some ON pixels
    let pix = bmf.get_pix('A').unwrap();
    let mut on_count = 0u32;
    for y in 0..pix.height() {
        for x in 0..pix.width() {
            if pix.get_pixel(x, y) == Some(1) {
                on_count += 1;
            }
        }
    }
    assert!(on_count > 0, "'A' glyph should have ON pixels, got 0");

    // Space should have NO ON pixels
    let space = bmf.get_pix(' ').unwrap();
    let mut space_on = 0u32;
    for y in 0..space.height() {
        for x in 0..space.width() {
            if space.get_pixel(x, y) == Some(1) {
                space_on += 1;
            }
        }
    }
    assert_eq!(space_on, 0, "space should have no ON pixels");
}

// ==========================================================================
// Test 16: Rendered text has pixels set
// ==========================================================================

#[test]
fn bmf_rendered_text_has_pixels() {
    let _rp = RegParams::new("bmf_rendered_text_pixels");
    let bmf = Bmf::new(10).unwrap();

    let pix = Pix::new(100, 30, PixelDepth::Bit1).unwrap();
    let (result, _) = bmf.set_textline(&pix, "Hi", 5, 15, 1).unwrap();

    // Count ON pixels in the result
    let mut on_count = 0u32;
    for y in 0..result.height() {
        for x in 0..result.width() {
            if result.get_pixel(x, y) == Some(1) {
                on_count += 1;
            }
        }
    }
    assert!(on_count > 0, "rendered text should have ON pixels");
}
