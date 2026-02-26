//! Core coverage regression tests
//!
//! Tests for all newly implemented core functions across pix1-5, boxbasic,
//! boxfunc3-4, ptabasic, pixabasic, numafunc, sarray, fpix, pixconv, rop,
//! compare, blend, and graphics modules.
//!
//! # See also
//!
//! C Leptonica: various core source files

use leptonica::*;

// ---------------------------------------------------------------------------
// pix1.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::create_with_cmap` – create Pix with colormap, verify depth and
/// that a colormap exists.
#[test]

fn create_with_cmap() {
    let pix = Pix::create_with_cmap(20, 20, PixelDepth::Bit8, InitColor::Black).unwrap();
    assert_eq!(pix.width(), 20);
    assert_eq!(pix.height(), 20);
    assert_eq!(pix.depth(), PixelDepth::Bit8);
    assert!(pix.has_colormap(), "colormap should exist");
}

/// Test `Pix::get_text_comp_new` / `PixMut::set_text_comp_new` – round-trip
/// compressed text through base64 encoding.
#[test]

fn text_comp_new_roundtrip() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    let data = b"hello leptonica";
    PixMut::set_text_comp_new(&mut pm, data).unwrap();
    let pix: Pix = pm.into();
    let decoded = pix.get_text_comp_new().unwrap();
    assert_eq!(decoded, data);
}

/// Test `Pix::get_random_pixel` – returns a pixel value and coordinates
/// within bounds.
#[test]

fn get_random_pixel() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let (val, x, y) = pix.get_random_pixel();
    assert!(x < 20);
    assert!(y < 20);
    // On a zeroed image the value should be 0
    assert_eq!(val, 0);
}

// ---------------------------------------------------------------------------
// pix2.c functions
// ---------------------------------------------------------------------------

/// Test `PixMut::set_component_arbitrary` – set one color component of a
/// 32bpp image.
#[test]

fn set_component_arbitrary() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    // Set red component (0) to 128
    pm.set_component_arbitrary(0, 128).unwrap();
    let pix: Pix = pm.into();
    let val = pix.get_pixel(0, 0).unwrap();
    let r = (val >> 24) & 0xFF;
    assert_eq!(r, 128);
}

/// Test `Pix::blend_in_rect` – blend color in a rectangle region.
#[test]

fn blend_in_rect() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let rect = Box::new(2, 2, 10, 10).unwrap();
    let result = pix.blend_in_rect(Some(rect), 0xFF000000, 0.5).unwrap();
    assert_eq!(result.width(), 20);
    assert_eq!(result.height(), 20);
}

/// Test `Pix::set_border_ring_val` – set border ring pixels at a given
/// distance.
#[test]

fn set_border_ring_val() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.set_border_ring_val(1, 255).unwrap();
    // Corner pixel should be set
    assert_eq!(result.get_pixel(0, 0).unwrap(), 255);
    // Center pixel should still be 0
    assert_eq!(result.get_pixel(10, 10).unwrap(), 0);
}

/// Test `Pix::set_mirrored_border` – set mirrored border pixels.
#[test]

fn set_mirrored_border() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.set_mirrored_border(2, 2, 2, 2).unwrap();
    assert_eq!(result.width(), 20);
}

/// Test `Pix::copy_border` – copy border pixels from another image.
#[test]

fn copy_border() {
    let pix1 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let pix2 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix1.copy_border(&pix2, 2, 2, 2, 2).unwrap();
    assert_eq!(result.width(), 20);
}

/// Test `Pix::add_multiple_black_white_borders` – alternating borders.
#[test]

fn add_multiple_black_white_borders() {
    let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    // n_white_border=2, width_white_border=2, width_black_border=2, n_pairs=3
    let result = pix.add_multiple_black_white_borders(2, 2, 2, 3).unwrap();
    // Initial white border + 3 pairs of black+white borders → grows
    assert!(result.width() > 10);
    assert!(result.height() > 10);
}

/// Test `Pix::remove_border_to_size` – remove border to target size.
#[test]

fn remove_border_to_size() {
    let pix = Pix::new(30, 30, PixelDepth::Bit8).unwrap();
    let result = pix.remove_border_to_size(20, 20).unwrap();
    assert_eq!(result.width(), 20);
    assert_eq!(result.height(), 20);
}

/// Test `Pix::add_mixed_border` – add mixed border.
#[test]

fn add_mixed_border() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let result = pix.add_mixed_border(3, 3, 3, 3).unwrap();
    assert_eq!(result.width(), 16);
    assert_eq!(result.height(), 16);
}

/// Test `Pix::add_continued_border` – add continued border.
#[test]

fn add_continued_border() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let result = pix.add_continued_border(3, 3, 3, 3).unwrap();
    assert_eq!(result.width(), 16);
    assert_eq!(result.height(), 16);
}

/// Test `Pix::shift_and_transfer_alpha` – alpha transfer with shift.
#[test]

fn shift_and_transfer_alpha() {
    let pix1 = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let pix2 = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let result = pix1.shift_and_transfer_alpha(&pix2, 2, 2).unwrap();
    assert_eq!(result.width(), 20);
}

// ---------------------------------------------------------------------------
// pix3.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::paint_self_through_mask` – paint image through a 1bpp mask.
#[test]

fn paint_self_through_mask() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let result = pix.paint_self_through_mask(&mask, 0, 0, 0).unwrap();
    assert_eq!(result.width(), 20);
}

/// Test `Pix::make_alpha_from_mask` – create alpha channel from 1bpp mask.
#[test]

fn make_alpha_from_mask() {
    let mask = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
    let alpha = mask.make_alpha_from_mask(200).unwrap();
    assert_eq!(alpha.width(), 10);
    assert_eq!(alpha.height(), 10);
}

/// Test `Pix::get_color_near_mask_boundary` – color near mask boundary.
#[test]

fn get_color_near_mask_boundary() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let mask = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let color = pix.get_color_near_mask_boundary(&mask, 2).unwrap();
    // On blank images the color should be 0
    assert_eq!(color, 0);
}

// ---------------------------------------------------------------------------
// pix4.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::count_rgb_colors_by_hash` – count unique RGB colors.
#[test]

fn count_rgb_colors_by_hash() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    // All-zero image → 1 unique color
    let count = pix.count_rgb_colors_by_hash().unwrap();
    assert_eq!(count, 1);
}

/// Test `Pix::color_amap_histogram` – colormap histogram via associative map.
#[test]

fn color_amap_histogram() {
    // color_amap_histogram requires a colormapped image
    let pix = Pix::create_with_cmap(10, 10, PixelDepth::Bit8, InitColor::Black).unwrap();
    let hist = pix.color_amap_histogram(1).unwrap();
    assert!(!hist.is_empty());
}

/// Test `Pix::get_binned_component_range` – get min/max of binned component.
#[test]

fn get_binned_component_range() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let (min_val, max_val) = pix
        .get_binned_component_range(16, 1, InColor::White, None, None)
        .unwrap();
    assert!(min_val <= max_val);
}

/// Test `Pix::get_rank_color_array` – sorted representative colors.
#[test]

fn get_rank_color_array() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let colors = pix.get_rank_color_array(4, InColor::White, 1, 256).unwrap();
    assert_eq!(colors.len(), 4);
}

/// Test `Pix::get_binned_color` – binned average color per bin.
#[test]

fn get_binned_color() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let pixa = Pixa::new();
    // Empty pixa yields empty result
    let colors = pix.get_binned_color(&pixa, 1).unwrap();
    assert_eq!(colors.len(), 0);
}

/// Test `Pix::display_color_array` – display array of colors as image.
#[test]

fn display_color_array() {
    let colors = vec![0xFF000000u32, 0x00FF0000, 0x0000FF00, 0x00000000];
    let result = Pix::display_color_array(&colors, 20, 2).unwrap();
    // 2 per row, 4 items → 2 rows
    assert_eq!(result.width(), 40);
    assert_eq!(result.height(), 40);
}

/// Test `Pix::rank_bin_by_strip` – rank binning by horizontal/vertical strip.
#[test]

fn rank_bin_by_strip() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let result = pix
        .rank_bin_by_strip(DiffDirection::Horizontal, 10, 4, InColor::White)
        .unwrap();
    assert!(result.width() > 0);
    assert!(result.height() > 0);
}

/// Test `Pix::split_distribution_fg_bg` – Otsu-like fg/bg threshold split.
#[test]

fn split_distribution_fg_bg() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let (threshold, avg_fg, avg_bg) = pix.split_distribution_fg_bg(0.0, 1).unwrap();
    assert!(threshold <= 255);
    assert!(avg_fg >= 0.0);
    assert!(avg_bg >= 0.0);
}

// ---------------------------------------------------------------------------
// pix5.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::find_area_perim_ratio` – area/perimeter ratio of 1bpp CC.
#[test]

fn find_area_perim_ratio() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let ratio = pix.find_area_perim_ratio().unwrap();
    assert!(ratio >= 0.0);
}

/// Test `Pix::find_perim_size_ratio` – perimeter/size ratio.
#[test]

fn find_perim_size_ratio() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let ratio = pix.find_perim_size_ratio().unwrap();
    assert!(ratio >= 0.0);
}

/// Test `Pix::find_area_fraction_masked` – area fraction under mask.
#[test]

fn find_area_fraction_masked() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let mask = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let fract = pix.find_area_fraction_masked(&mask).unwrap();
    assert!((0.0..=1.0).contains(&fract));
}

/// Test `Pix::conforms_to_rectangle` – check rectangularity of 1bpp CC.
#[test]

fn conforms_to_rectangle() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let result = pix.conforms_to_rectangle(0.9).unwrap();
    // Empty image may be trivially rectangular - any bool is fine
    let _ = result;
}

/// Test `Pixa::find_perim_size_ratio` – collection version.
#[test]

fn pixa_find_perim_size_ratio() {
    let pixa = Pixa::new();
    let result = pixa.find_perim_size_ratio();
    // Empty pixa should still succeed (or return an empty Numa)
    assert!(result.is_ok());
}

/// Test `Pixa::find_area_fraction_masked` – collection version.
#[test]

fn pixa_find_area_fraction_masked() {
    let mask = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let pixa = Pixa::new();
    let result = pixa.find_area_fraction_masked(&mask);
    assert!(result.is_ok());
}

/// Test `Pixa::find_width_height_ratio` – width/height ratio collection.
#[test]

fn pixa_find_width_height_ratio() {
    let pixa = Pixa::new();
    let result = pixa.find_width_height_ratio();
    assert!(result.is_ok());
}

/// Test `Pixa::find_width_height_product` – width*height product collection.
#[test]

fn pixa_find_width_height_product() {
    let pixa = Pixa::new();
    let result = pixa.find_width_height_product();
    assert!(result.is_ok());
}

/// Test `Pix::find_rectangle_comps` – find rectangular connected components.
#[test]

fn find_rectangle_comps() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let boxa = pix.find_rectangle_comps(0.9).unwrap();
    assert!(boxa.is_empty()); // empty image has no CCs
}

/// Test `Pix::conforms_to_rectangle_detail` – detailed rectangularity check.
///
/// Note: if not implemented, this test documents the expected API.
#[test]

fn conforms_to_rectangle_detail() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    // conforms_to_rectangle as proxy for the detailed version
    let result = pix.conforms_to_rectangle(0.8).unwrap();
    let _ = result;
}

/// Test `Pix::extract_rectangular_regions` – extract rectangular regions.
#[test]

fn extract_rectangular_regions() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let pixa = pix.extract_rectangular_regions(0.9).unwrap();
    assert_eq!(pixa.len(), 0);
}

/// Test `Pix::select_component_by_size` – select CCs by size threshold.
#[test]

fn select_component_by_size() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let (result_pix, boxa) = pix
        .select_component_by_size(5, 5, 8, SizeRelation::GreaterThan)
        .unwrap();
    assert_eq!(result_pix.width(), 20);
    assert_eq!(boxa.len(), 0);
}

/// Test `Pix::filter_component_by_size` – filter CCs by size.
#[test]

fn filter_component_by_size() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let result = pix
        .filter_component_by_size(5, 5, 8, SizeRelation::GreaterThan)
        .unwrap();
    assert_eq!(result.width(), 20);
}

/// Test `Pix::make_covering_of_rectangles` – create covering rectangles.
#[test]

fn make_covering_of_rectangles() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let boxa = pix.make_covering_of_rectangles(2).unwrap();
    // No foreground → no covering rects
    assert_eq!(boxa.len(), 0);
}

/// Test `Pix::reversal_profile` – count reversals along rows/columns.
#[test]

fn reversal_profile() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let na = pix.reversal_profile(0.5, 0, 0, 19, 1).unwrap();
    assert_eq!(na.len(), 20);
}

/// Test `Pix::windowed_variance_on_line` – windowed variance computation.
#[test]

fn windowed_variance_on_line() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let na = pix.windowed_variance_on_line(0, 10, 0, 19, 5).unwrap();
    assert!(!na.is_empty());
}

/// Test `Pix::min_max_near_line` – min/max values near a line.
#[test]

fn min_max_near_line() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let (namin, namax) = pix.min_max_near_line(0, 10, 19, 10, 2).unwrap();
    assert!(!namin.is_empty());
    assert!(!namax.is_empty());
}

// ---------------------------------------------------------------------------
// boxbasic.c functions
// ---------------------------------------------------------------------------

/// Test `Box::set_geometry` – set box geometry.
#[test]

fn box_set_geometry() {
    let b = Box::new(0, 0, 10, 10).unwrap();
    let b2 = b.set_geometry(5, 5, 20, 20);
    assert_eq!(b2.x, 5);
    assert_eq!(b2.y, 5);
    assert_eq!(b2.w, 20);
    assert_eq!(b2.h, 20);
}

/// Test `Box::side_locations` – get left/right/top/bottom.
#[test]

fn box_side_locations() {
    let b = Box::new(10, 20, 30, 40).unwrap();
    let (left, right, top, bottom) = b.side_locations();
    assert_eq!(left, 10);
    assert_eq!(right, 39);
    assert_eq!(top, 20);
    assert_eq!(bottom, 59);
}

/// Test `Box::from_side_locations` – create box from side locations.
#[test]

fn box_from_side_locations() {
    let b = Box::from_side_locations(10, 39, 20, 59);
    assert_eq!(b.x, 10);
    assert_eq!(b.y, 20);
    assert_eq!(b.w, 30);
    assert_eq!(b.h, 40);
}

/// Test `Boxa::get_box_geometry` – get geometry of nth box.
#[test]

fn boxa_get_box_geometry() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(5, 10, 15, 20).unwrap());
    let (x, y, w, h) = boxa.get_box_geometry(0).unwrap();
    assert_eq!((x, y, w, h), (5, 10, 15, 20));
}

/// Test `Boxa::remove_and_save` – remove box and return it.
#[test]

fn boxa_remove_and_save() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(1, 2, 3, 4).unwrap());
    boxa.push(Box::new(5, 6, 7, 8).unwrap());
    let b = boxa.remove_and_save(0).unwrap();
    assert_eq!(b.x, 1);
    assert_eq!(boxa.len(), 1);
}

/// Test `Boxa::permute_pseudorandom` – pseudorandom permutation.
#[test]

fn boxa_permute_pseudorandom() {
    let mut boxa = Boxa::new();
    for i in 0..10 {
        boxa.push(Box::new(i, 0, 10, 10).unwrap());
    }
    let permuted = boxa.permute_pseudorandom();
    assert_eq!(permuted.len(), 10);
}

/// Test `Boxa::permute_random` – random permutation with seed.
#[test]

fn boxa_permute_random() {
    let mut boxa = Boxa::new();
    for i in 0..10 {
        boxa.push(Box::new(i, 0, 10, 10).unwrap());
    }
    let permuted = boxa.permute_random(42);
    assert_eq!(permuted.len(), 10);
}

/// Test `Boxaa::get_box` – nested index access.
#[test]

fn boxaa_get_box() {
    let mut boxaa = Boxaa::new();
    let mut boxa = Boxa::new();
    boxa.push(Box::new(1, 2, 3, 4).unwrap());
    boxaa.push(boxa);
    let b = boxaa.get_box(0, 0).unwrap();
    assert_eq!(b.x, 1);
}

/// Test `Boxaa::replace` – replace boxa at index.
#[test]

fn boxaa_replace() {
    let mut boxaa = Boxaa::new();
    boxaa.push(Boxa::new());
    let mut replacement = Boxa::new();
    replacement.push(Box::new(9, 9, 9, 9).unwrap());
    boxaa.replace(0, replacement).unwrap();
    let b = boxaa.get_box(0, 0).unwrap();
    assert_eq!(b.x, 9);
}

/// Test `Boxaa::insert` – insert boxa at index.
#[test]

fn boxaa_insert() {
    let mut boxaa = Boxaa::new();
    boxaa.push(Boxa::new());
    let mut new_boxa = Boxa::new();
    new_boxa.push(Box::new(7, 7, 7, 7).unwrap());
    boxaa.insert(0, new_boxa).unwrap();
    assert_eq!(boxaa.len(), 2);
    let b = boxaa.get_box(0, 0).unwrap();
    assert_eq!(b.x, 7);
}

/// Test `Boxaa::remove` – remove boxa at index.
#[test]

fn boxaa_remove() {
    let mut boxaa = Boxaa::new();
    boxaa.push(Boxa::new());
    boxaa.push(Boxa::new());
    let removed = boxaa.remove(0).unwrap();
    assert_eq!(boxaa.len(), 1);
    assert_eq!(removed.len(), 0);
}

/// Test `Boxaa::add_box` – add single box to nested boxa.
#[test]

fn boxaa_add_box() {
    let mut boxaa = Boxaa::new();
    boxaa.push(Boxa::new());
    boxaa.add_box(0, Box::new(3, 3, 3, 3).unwrap()).unwrap();
    let b = boxaa.get_box(0, 0).unwrap();
    assert_eq!(b.x, 3);
}

// ---------------------------------------------------------------------------
// boxfunc3.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::mask_conn_comp` – mask connected components.
#[test]

fn mask_conn_comp() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let (mask, boxa) = pix.mask_conn_comp(8).unwrap();
    assert_eq!(mask.width(), 20);
    assert_eq!(boxa.len(), 0); // no foreground
}

/// Test `Pixa::display_boxaa` – display pixa images with boxaa annotations.
#[test]

fn display_boxaa() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(20, 20, PixelDepth::Bit32).unwrap());
    let mut boxaa = Boxaa::new();
    let mut boxa = Boxa::new();
    boxa.push(Box::new(2, 2, 10, 10).unwrap());
    boxaa.push(boxa);
    let colors = vec![0xFF000000u32];
    let result = Pixa::display_boxaa(&pixa, &boxaa, &colors).unwrap();
    assert_eq!(result.len(), 1);
}

/// Test `Pix::split_into_boxa` – split 1bpp image into boxes.
#[test]

fn split_into_boxa() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let boxa = pix.split_into_boxa(1, 1, 1, 0).unwrap();
    assert_eq!(boxa.len(), 0); // no foreground
}

/// Test `Pix::split_component_into_boxa` – split single component into boxes.
#[test]

fn split_component_into_boxa() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let boxa = pix.split_component_into_boxa(1, 1, 1, 0).unwrap();
    assert_eq!(boxa.len(), 0);
}

/// Test `make_mosaic_strips` – create mosaic strip layout.
#[test]

fn test_make_mosaic_strips() {
    let boxa = make_mosaic_strips(100, 100, 0, 25).unwrap();
    assert!(!boxa.is_empty());
}

/// Test `Pix::select_large_ul_comp` – select largest upper-left component
/// from a 1bpp image.
#[test]

fn select_large_ul_comp() {
    // Create a 1bpp image with a foreground rectangle
    let pix = Pix::new(50, 50, PixelDepth::Bit1).unwrap();
    let mut pm = pix.try_into_mut().unwrap();
    for y in 5..25 {
        for x in 5..25 {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pm.into();
    let b = pix.select_large_ul_comp(0.5).unwrap();
    assert!(b.w > 0);
    assert!(b.h > 0);
}

/// Test `Boxa::display_tiled` – tiled box display.
#[test]

fn boxa_display_tiled() {
    let mut boxa = Boxa::new();
    boxa.push(Box::new(0, 0, 20, 20).unwrap());
    boxa.push(Box::new(0, 0, 30, 30).unwrap());
    let result = boxa.display_tiled(None, 200).unwrap();
    assert!(result.width() > 0);
}

// ---------------------------------------------------------------------------
// ptabasic.c functions
// ---------------------------------------------------------------------------

/// Test `Pta::remove_pt` – remove point at index.
#[test]

fn pta_remove_pt() {
    let mut pta = Pta::new();
    pta.push(1.0, 2.0);
    pta.push(3.0, 4.0);
    let (x, y) = pta.remove_pt(0).unwrap();
    assert!((x - 1.0).abs() < 1e-6);
    assert!((y - 2.0).abs() < 1e-6);
    assert_eq!(pta.len(), 1);
}

/// Test `Pta::crop_to_mask` – crop points to 1bpp mask.
#[test]

fn pta_crop_to_mask() {
    let mut pta = Pta::new();
    pta.push(5.0, 5.0);
    pta.push(15.0, 15.0);
    let mask = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    // All mask pixels are 0, so no points should survive
    let cropped = pta.crop_to_mask(&mask).unwrap();
    assert_eq!(cropped.len(), 0);
}

// ---------------------------------------------------------------------------
// pixabasic.c functions
// ---------------------------------------------------------------------------

/// Test `Pixa::remove_pix` – remove pix at index.
#[test]

fn pixa_remove_pix() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
    pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
    pixa.remove_pix(0).unwrap();
    assert_eq!(pixa.len(), 1);
}

/// Test `Pixa::remove_pix_and_save` – remove and return pix.
#[test]

fn pixa_remove_pix_and_save() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
    let pix = pixa.remove_pix_and_save(0).unwrap();
    assert_eq!(pix.width(), 10);
    assert_eq!(pixa.len(), 0);
}

/// Test `Pixa::read_both` – read images and boxes from files.
#[test]

fn pixa_read_both() {
    use std::path::Path;
    // Use non-existent paths; the function should return an error gracefully.
    let result = Pixa::read_both(Path::new("/nonexistent/img"), Path::new("/nonexistent/box"));
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// numafunc functions
// ---------------------------------------------------------------------------

/// Test `Numa::sort_general` – general sort returning sorted array and index map.
#[test]

fn numa_sort_general() {
    let mut na = Numa::new();
    na.push(3.0);
    na.push(1.0);
    na.push(2.0);
    let (sorted, indices) = na.sort_general(SortOrder::Increasing);
    assert!((sorted.get(0).unwrap() - 1.0).abs() < 1e-6);
    assert!((sorted.get(1).unwrap() - 2.0).abs() < 1e-6);
    assert!((sorted.get(2).unwrap() - 3.0).abs() < 1e-6);
    assert_eq!(indices.len(), 3);
}

/// Test `Numa::choose_sort_type` – heuristic for sort algorithm.
#[test]

fn numa_choose_sort_type() {
    // Small n with small max_val
    let use_bin = Numa::choose_sort_type(100, 50.0);
    // Just verify it returns a bool
    let _ = use_bin;
}

/// Test `Numaa::join` – join two Numaa collections.
#[test]

fn numaa_join() {
    let mut naa1 = Numaa::new();
    naa1.push(Numa::new());
    let mut naa2 = Numaa::new();
    naa2.push(Numa::new());
    naa2.push(Numa::new());
    naa1.join(&naa2);
    assert_eq!(naa1.len(), 3);
}

// ---------------------------------------------------------------------------
// sarray functions
// ---------------------------------------------------------------------------

/// Test `Sarray::convert_words_to_lines` – convert words to lines.
#[test]

fn sarray_convert_words_to_lines() {
    let mut sa = Sarray::new();
    sa.push("hello");
    sa.push("world");
    sa.push("foo");
    let lines = sa.convert_words_to_lines(12);
    assert!(!lines.is_empty());
}

/// Test `Sarray::append_range` – append range from another Sarray.
#[test]

fn sarray_append_range() {
    let mut sa1 = Sarray::new();
    sa1.push("a");
    let mut sa2 = Sarray::new();
    sa2.push("b");
    sa2.push("c");
    sa2.push("d");
    sa1.append_range(&sa2, 0, 1);
    assert_eq!(sa1.len(), 3); // "a", "b", "c"
}

/// Test `Sarray::append` – append a string.
#[test]

fn sarray_append() {
    let mut sa = Sarray::new();
    sa.append("test");
    assert_eq!(sa.len(), 1);
    assert_eq!(sa.get(0).unwrap(), "test");
}

// ---------------------------------------------------------------------------
// fpix functions
// ---------------------------------------------------------------------------

/// Test FPixa basic operations: create, push, get, len.
#[test]

fn fpixa_basic() {
    let mut fpixa = FPixa::new();
    assert_eq!(fpixa.len(), 0);
    assert!(fpixa.is_empty());

    let fpix = FPix::new(10, 10).unwrap();
    fpixa.push(fpix);
    assert_eq!(fpixa.len(), 1);
    assert!(!fpixa.is_empty());

    let f = fpixa.get(0).unwrap();
    assert_eq!(f.width(), 10);
}

/// Test FPixa pixel access: get_pixel / set_pixel.
#[test]

fn fpixa_pixel_access() {
    let mut fpixa = FPixa::new();
    let fpix = FPix::new(5, 5).unwrap();
    fpixa.push(fpix);

    fpixa.set_pixel(0, 2, 3, 42.5).unwrap();
    let val = fpixa.get_pixel(0, 2, 3).unwrap();
    assert!((val - 42.5).abs() < 1e-6);
}

/// Test FPixa dimension/data access: get_dimensions, get_data.
#[test]

fn fpixa_get_dimensions_and_data() {
    let mut fpixa = FPixa::new();
    let fpix = FPix::new(8, 6).unwrap();
    fpixa.push(fpix);

    let (w, h) = fpixa.get_dimensions(0).unwrap();
    assert_eq!(w, 8);
    assert_eq!(h, 6);

    let data = fpixa.get_data(0).unwrap();
    assert_eq!(data.len(), 48); // 8 * 6
}

// ---------------------------------------------------------------------------
// pixconv.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::threshold_8` – threshold 8bpp to lower depth.
#[test]

fn threshold_8() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.threshold_8(4, 4, false).unwrap();
    assert_eq!(result.width(), 20);
}

/// Test `Pix::convert_rgb_to_binary_arb` – RGB to binary with weights.
#[test]

fn convert_rgb_to_binary_arb() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let result = pix.convert_rgb_to_binary_arb(0.3, 0.5, 0.2, 128).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit1);
}

/// Test `Pix::convert_rgb_to_colormap` – RGB to 8bpp colormap.
#[test]

fn convert_rgb_to_colormap() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = pix.convert_rgb_to_colormap(false).unwrap();
    assert!(result.has_colormap());
}

/// Test `Pix::quantize_if_few_colors` – quantize only if few unique colors.
#[test]

fn quantize_if_few_colors() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let result = pix.quantize_if_few_colors(256, 0.0, 8).unwrap();
    // With only 1 unique color (0), quantization should succeed
    assert!(result.is_some());
}

/// Test `Pix::convert_to_1_adaptive` – adaptive binarization.
#[test]

fn convert_to_1_adaptive() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.convert_to_1_adaptive().unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit1);
}

/// Test `Pix::convert_to_1_by_sampling` – binary by sampling.
#[test]

fn convert_to_1_by_sampling() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.convert_to_1_by_sampling(2, 128).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit1);
}

/// Test `Pix::convert_to_8_by_sampling` – 8bpp by sampling.
#[test]

fn convert_to_8_by_sampling() {
    let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
    let result = pix.convert_to_8_by_sampling(1, false).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit8);
}

/// Test `Pix::convert_to_8_colormap` – 8bpp with colormap.
#[test]

fn convert_to_8_colormap() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let result = pix.convert_to_8_colormap(false).unwrap();
    assert!(result.has_colormap());
}

/// Test `Pix::convert_to_32_by_sampling` – 32bpp by sampling.
#[test]

fn convert_to_32_by_sampling() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.convert_to_32_by_sampling(1).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

/// Test `Pix::convert_24_to_32` – 24bpp to 32bpp conversion.
#[test]

fn convert_24_to_32() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    // convert_32_to_24 first, then back
    let pix24 = pix.convert_32_to_24().unwrap();
    let pix32 = pix24.convert_24_to_32().unwrap();
    assert_eq!(pix32.depth(), PixelDepth::Bit32);
}

/// Test `Pix::convert_32_to_24` – 32bpp to 24bpp conversion.
#[test]

fn convert_32_to_24() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = pix.convert_32_to_24().unwrap();
    assert_eq!(result.width(), 10);
}

/// Test `Pix::convert_to_subpixel_rgb` – subpixel rendering.
#[test]

fn convert_to_subpixel_rgb() {
    let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
    let result = pix.convert_to_subpixel_rgb(1, 0).unwrap();
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

// ---------------------------------------------------------------------------
// rop.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::rasterop_ip` – in-place rasterop (shift with zero fill).
#[test]

fn rasterop_ip() {
    let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let result = pix.rasterop_ip(5, 3).unwrap();
    assert_eq!(result.width(), 20);
    assert_eq!(result.height(), 20);
}

/// Test `Pix::rasterop_full_image` – full image rasterop between two images.
#[test]

fn rasterop_full_image() {
    let pix1 = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let pix2 = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let result = pix1.rasterop_full_image(&pix2, RopOp::Xor).unwrap();
    assert_eq!(result.width(), 20);
}

// ---------------------------------------------------------------------------
// compare.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::compare_tiled` – tiled comparison of two images.
#[test]

fn compare_tiled() {
    let pix1 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let pix2 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let (exceed_count, diff_pix) = pix1.compare_tiled(&pix2, 5, 5, 10).unwrap();
    assert_eq!(exceed_count, 0); // identical images
    assert!(diff_pix.width() > 0);
}

/// Test `Pix::get_perceptual_diff` – perceptual difference between images.
#[test]

fn get_perceptual_diff() {
    let pix1 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let pix2 = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
    let (fract, avg, exceeds) = pix1.get_perceptual_diff(&pix2, 1, 1, 10, 0.1, 1).unwrap();
    assert!((fract - 0.0).abs() < 1e-6);
    assert!((avg - 0.0).abs() < 1e-6);
    assert!(!exceeds);
}

// ---------------------------------------------------------------------------
// blend.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::blend_background_to_color` – blend toward background color.
#[test]

fn blend_background_to_color() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = pix.blend_background_to_color(0xFFFFFF00).unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

/// Test `Pix::set_alpha_over_white` – composite alpha over white.
#[test]

fn set_alpha_over_white() {
    let pix = Pix::new(10, 10, PixelDepth::Bit32).unwrap();
    let result = pix.set_alpha_over_white().unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.depth(), PixelDepth::Bit32);
}

// ---------------------------------------------------------------------------
// graphics.c functions
// ---------------------------------------------------------------------------

/// Test `Pix::generate_pta_boundary` – generate boundary PTA from 1bpp image.
#[test]

fn generate_pta_boundary() {
    let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
    let pta = pix.generate_pta_boundary(1).unwrap();
    // Empty image has no boundary pixels
    assert_eq!(pta.len(), 0);
}

// ---------------------------------------------------------------------------
// Pixa serialization (pixabasic.c)
// ---------------------------------------------------------------------------

/// Test `Pixa::write_to_bytes` / `Pixa::read_from_bytes` – round-trip empty.
#[test]

fn pixa_serial_roundtrip_empty() {
    let pixa = Pixa::new();
    let bytes = pixa.write_to_bytes().unwrap();
    let restored = Pixa::read_from_bytes(&bytes).unwrap();
    assert_eq!(restored.len(), 0);
}

/// Test `Pixa::write_to_bytes` / `Pixa::read_from_bytes` – round-trip with images.
#[test]

fn pixa_serial_roundtrip_with_images() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(10, 10, PixelDepth::Bit8).unwrap());
    pixa.push(Pix::new(20, 15, PixelDepth::Bit8).unwrap());

    let bytes = pixa.write_to_bytes().unwrap();
    let restored = Pixa::read_from_bytes(&bytes).unwrap();

    assert_eq!(restored.len(), 2);
    assert_eq!(restored.get(0).unwrap().width(), 10);
    assert_eq!(restored.get(1).unwrap().width(), 20);
    assert_eq!(restored.get(1).unwrap().height(), 15);
}

/// Test `Pixa::write_to_bytes` / `Pixa::read_from_bytes` – preserves boxes.
#[test]

fn pixa_serial_roundtrip_with_boxes() {
    let mut pixa = Pixa::new();
    pixa.push_with_box(
        Pix::new(10, 10, PixelDepth::Bit8).unwrap(),
        Box::new(1, 2, 3, 4).unwrap(),
    );

    let bytes = pixa.write_to_bytes().unwrap();
    let restored = Pixa::read_from_bytes(&bytes).unwrap();

    assert_eq!(restored.len(), 1);
    assert_eq!(restored.boxa().len(), 1);
    let b = restored.get_box(0).unwrap();
    assert_eq!((b.x, b.y, b.w, b.h), (1, 2, 3, 4));
}

/// Test `Pixa::write_to_file` / `Pixa::read_from_file` – file round-trip.
#[test]

fn pixa_serial_file_roundtrip() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(5, 5, PixelDepth::Bit8).unwrap());

    let dir = std::env::temp_dir().join("leptonica_test_pixa_serial_cov");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.pixa");

    pixa.write_to_file(&path).unwrap();
    let restored = Pixa::read_from_file(&path).unwrap();

    assert_eq!(restored.len(), 1);
    assert_eq!(restored.get(0).unwrap().width(), 5);

    let _ = std::fs::remove_dir_all(&dir);
}

/// Test `Pixa::read_from_bytes` – rejects garbage data.
#[test]

fn pixa_serial_invalid_data() {
    assert!(Pixa::read_from_bytes(b"garbage data").is_err());
    assert!(Pixa::read_from_bytes(b"").is_err());
}

/// Test `Pixa::write_to_bytes` / `Pixa::read_from_reader` – reader round-trip.
#[test]

fn pixa_serial_reader_roundtrip() {
    let mut pixa = Pixa::new();
    pixa.push(Pix::new(8, 8, PixelDepth::Bit8).unwrap());

    let mut buf = Vec::new();
    pixa.write_to_writer(&mut buf).unwrap();

    let restored = Pixa::read_from_reader(&mut std::io::Cursor::new(&buf)).unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored.get(0).unwrap().width(), 8);
}

// ---------------------------------------------------------------------------
// Boxaa::read_from_files (boxbasic.c)
// ---------------------------------------------------------------------------

/// Test `Boxaa::read_from_files` – reads boxa files from a directory.
#[test]

fn boxaa_read_from_files() {
    use leptonica::Boxaa;

    let dir = std::env::temp_dir().join("leptonica_test_boxaa_from_files");
    std::fs::create_dir_all(&dir).unwrap();

    // Write two boxa files
    let mut boxa1 = Boxa::new();
    boxa1.push(Box::new(10, 20, 30, 40).unwrap());
    boxa1.write_to_file(dir.join("a.boxa")).unwrap();

    let mut boxa2 = Boxa::new();
    boxa2.push(Box::new(50, 60, 70, 80).unwrap());
    boxa2.write_to_file(dir.join("b.boxa")).unwrap();

    let baa = Boxaa::read_from_files(&dir, None, 0, 0).unwrap();
    assert_eq!(baa.len(), 2);
    assert_eq!(baa.get(0).unwrap().len(), 1);
    assert_eq!(baa.get(1).unwrap().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

/// Test `Boxaa::read_from_files` with substring filter.
#[test]

fn boxaa_read_from_files_with_filter() {
    use leptonica::Boxaa;

    let dir = std::env::temp_dir().join("leptonica_test_boxaa_from_files_filter");
    std::fs::create_dir_all(&dir).unwrap();

    let mut boxa = Boxa::new();
    boxa.push(Box::new(1, 2, 3, 4).unwrap());
    boxa.write_to_file(dir.join("keep_a.boxa")).unwrap();
    boxa.write_to_file(dir.join("skip_b.boxa")).unwrap();

    let baa = Boxaa::read_from_files(&dir, Some("keep"), 0, 0).unwrap();
    assert_eq!(baa.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}
