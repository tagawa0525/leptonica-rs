//! Tests for runlength module functions
//!
//! Tests run-length transform, membership computation, and MSB location table.

use leptonica::filter::runlength::{
    RunDirection, find_horizontal_runs, find_max_horizontal_run_on_line, find_max_runs,
    find_max_vertical_run_on_line, find_vertical_runs, make_msbit_loc_tab,
    runlength_membership_on_line, runlength_transform,
};
use leptonica::{Pix, PixelDepth};

/// Test make_msbit_loc_tab for bitval=1
#[test]
fn runlength_msbit_loc_tab_bitval1() {
    let tab = make_msbit_loc_tab(1);
    assert_eq!(tab.len(), 256);
    // 0x00 = no bits set → 8
    assert_eq!(tab[0x00], 8);
    // 0x80 = MSB set → position 0
    assert_eq!(tab[0x80], 0);
    // 0x01 = LSB set → position 7
    assert_eq!(tab[0x01], 7);
    // 0xFF = all bits → position 0
    assert_eq!(tab[0xFF], 0);
    // 0x40 = bit 6 → position 1
    assert_eq!(tab[0x40], 1);
    // 0x04 = bit 2 → position 5
    assert_eq!(tab[0x04], 5);
}

/// Test make_msbit_loc_tab for bitval=0
#[test]
fn runlength_msbit_loc_tab_bitval0() {
    let tab = make_msbit_loc_tab(0);
    assert_eq!(tab.len(), 256);
    // 0xFF = no zero bits → 8
    assert_eq!(tab[0xFF], 8);
    // 0x00 = all zeros → position 0
    assert_eq!(tab[0x00], 0);
    // 0x7F = bit 7 is 0 → position 0
    assert_eq!(tab[0x7F], 0);
    // 0xFE = bit 0 is 0 → position 7
    assert_eq!(tab[0xFE], 7);
}

/// Test runlength_membership_on_line
#[test]
fn runlength_membership_basic() {
    let mut buffer = vec![0i32; 10];
    let start = [2, 7];
    let end = [4, 8];
    runlength_membership_on_line(&mut buffer, 10, PixelDepth::Bit8, &start, &end, 2);
    // Positions 0,1 → 0 (no run)
    assert_eq!(buffer[0], 0);
    assert_eq!(buffer[1], 0);
    // Positions 2-4 → run length 3
    assert_eq!(buffer[2], 3);
    assert_eq!(buffer[3], 3);
    assert_eq!(buffer[4], 3);
    // Positions 5,6 → 0
    assert_eq!(buffer[5], 0);
    assert_eq!(buffer[6], 0);
    // Positions 7-8 → run length 2
    assert_eq!(buffer[7], 2);
    assert_eq!(buffer[8], 2);
    // Position 9 → 0
    assert_eq!(buffer[9], 0);
}

/// Test find_horizontal_runs on a small binary image
#[test]
fn runlength_find_horizontal_runs() {
    let pix = Pix::new(10, 1, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Set pixels 2-4 and 7-8
    pix_mut.set_pixel(2, 0, 1).unwrap();
    pix_mut.set_pixel(3, 0, 1).unwrap();
    pix_mut.set_pixel(4, 0, 1).unwrap();
    pix_mut.set_pixel(7, 0, 1).unwrap();
    pix_mut.set_pixel(8, 0, 1).unwrap();
    let pix: Pix = pix_mut.into();

    let mut start = vec![0i32; 6];
    let mut end = vec![0i32; 6];
    let n = find_horizontal_runs(&pix, 0, &mut start, &mut end);
    assert_eq!(n, 2);
    assert_eq!(start[0], 2);
    assert_eq!(end[0], 4);
    assert_eq!(start[1], 7);
    assert_eq!(end[1], 8);
}

/// Test find_vertical_runs
#[test]
fn runlength_find_vertical_runs() {
    let pix = Pix::new(1, 8, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Set pixels at rows 1-3
    pix_mut.set_pixel(0, 1, 1).unwrap();
    pix_mut.set_pixel(0, 2, 1).unwrap();
    pix_mut.set_pixel(0, 3, 1).unwrap();
    let pix: Pix = pix_mut.into();

    let mut start = vec![0i32; 5];
    let mut end = vec![0i32; 5];
    let n = find_vertical_runs(&pix, 0, &mut start, &mut end);
    assert_eq!(n, 1);
    assert_eq!(start[0], 1);
    assert_eq!(end[0], 3);
}

/// Test find_max_horizontal_run_on_line
#[test]
fn runlength_find_max_horizontal_run() {
    let pix = Pix::new(20, 1, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Two runs: length 3 (positions 2-4) and length 5 (positions 10-14)
    for x in 2..=4 {
        pix_mut.set_pixel(x, 0, 1).unwrap();
    }
    for x in 10..=14 {
        pix_mut.set_pixel(x, 0, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let (start, size) = find_max_horizontal_run_on_line(&pix, 0);
    assert_eq!(start, 10);
    assert_eq!(size, 5);
}

/// Test find_max_vertical_run_on_line
#[test]
fn runlength_find_max_vertical_run() {
    let pix = Pix::new(1, 20, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Run of length 4 at rows 5-8
    for y in 5..=8 {
        pix_mut.set_pixel(0, y, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let (start, size) = find_max_vertical_run_on_line(&pix, 0);
    assert_eq!(start, 5);
    assert_eq!(size, 4);
}

/// Test runlength_transform horizontal
#[test]
fn runlength_transform_horizontal() {
    let pix = Pix::new(10, 2, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Row 0: run of 5 pixels at positions 2-6
    for x in 2..=6 {
        pix_mut.set_pixel(x, 0, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let result = runlength_transform(&pix, 1, RunDirection::Horizontal, PixelDepth::Bit8).unwrap();
    assert_eq!(result.width(), 10);
    assert_eq!(result.height(), 2);
    assert_eq!(result.depth(), PixelDepth::Bit8);

    // Positions 2-6 in row 0 should have value 5
    assert_eq!(result.get_pixel(2, 0).unwrap(), 5);
    assert_eq!(result.get_pixel(4, 0).unwrap(), 5);
    assert_eq!(result.get_pixel(6, 0).unwrap(), 5);
    // Position 0 in row 0 should have value 0
    assert_eq!(result.get_pixel(0, 0).unwrap(), 0);
}

/// Test runlength_transform vertical
#[test]
fn runlength_transform_vertical() {
    let pix = Pix::new(2, 10, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Column 0: run of 3 pixels at rows 1-3
    for y in 1..=3 {
        pix_mut.set_pixel(0, y, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let result = runlength_transform(&pix, 1, RunDirection::Vertical, PixelDepth::Bit8).unwrap();
    // Positions in column 0, rows 1-3 should have value 3
    assert_eq!(result.get_pixel(0, 1).unwrap(), 3);
    assert_eq!(result.get_pixel(0, 2).unwrap(), 3);
    assert_eq!(result.get_pixel(0, 3).unwrap(), 3);
    // Row 0 should be 0
    assert_eq!(result.get_pixel(0, 0).unwrap(), 0);
}

/// Test find_max_runs horizontal
#[test]
fn runlength_find_max_runs_horizontal() {
    let pix = Pix::new(10, 3, PixelDepth::Bit1).unwrap();
    let mut pix_mut = pix.try_into_mut().unwrap();
    // Row 0: run of 3
    for x in 0..3 {
        pix_mut.set_pixel(x, 0, 1).unwrap();
    }
    // Row 1: empty
    // Row 2: run of 5
    for x in 2..7 {
        pix_mut.set_pixel(x, 2, 1).unwrap();
    }
    let pix: Pix = pix_mut.into();

    let (sizes, _starts) = find_max_runs(&pix, RunDirection::Horizontal).unwrap();
    assert_eq!(sizes.len(), 3);
    assert_eq!(sizes.get(0).unwrap() as u32, 3);
    assert_eq!(sizes.get(1).unwrap() as u32, 0);
    assert_eq!(sizes.get(2).unwrap() as u32, 5);
}
