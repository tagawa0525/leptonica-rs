//! Regression tests for plan 124 (Pixa::bin_sort, Pixaa::scale_to_size_var).

use leptonica::core::Box;
use leptonica::core::numa::{Numa, SortOrder};
use leptonica::core::pixa::{PixaSortType, Pixaa};
use leptonica::{Pix, Pixa, PixelDepth};

fn pixa_with_widths(widths: &[u32]) -> Pixa {
    let mut pa = Pixa::with_capacity(widths.len());
    for (i, &w) in widths.iter().enumerate() {
        let pix = Pix::new(w, 4, PixelDepth::Bit8).unwrap();
        pa.push_with_box(pix, Box::new(i as i32 * 100, 0, w as i32, 4).unwrap());
    }
    pa
}

// -- bin_sort ----------------------------------------------------------

#[test]

fn bin_sort_by_width_ascending_matches_sort() {
    let pa = pixa_with_widths(&[8, 2, 5, 1, 9, 3]);
    let (sorted, indices) = pa
        .bin_sort(PixaSortType::ByWidth, SortOrder::Increasing)
        .unwrap();
    // Validate widths after sorting
    let got_widths: Vec<u32> = sorted.pix_slice().iter().map(|p| p.width()).collect();
    let mut expected_widths = vec![8, 2, 5, 1, 9, 3];
    expected_widths.sort();
    assert_eq!(got_widths, expected_widths);
    // indices should be a permutation of 0..6
    let mut sorted_indices = indices.clone();
    sorted_indices.sort();
    assert_eq!(sorted_indices, (0..6).collect::<Vec<_>>());
}

#[test]

fn bin_sort_descending_reverses() {
    let pa = pixa_with_widths(&[3, 1, 4, 1, 5, 9, 2, 6]);
    let (asc, _) = pa
        .bin_sort(PixaSortType::ByWidth, SortOrder::Increasing)
        .unwrap();
    let (desc, _) = pa
        .bin_sort(PixaSortType::ByWidth, SortOrder::Decreasing)
        .unwrap();
    let asc_w: Vec<u32> = asc.pix_slice().iter().map(|p| p.width()).collect();
    let mut desc_w: Vec<u32> = desc.pix_slice().iter().map(|p| p.width()).collect();
    desc_w.reverse();
    assert_eq!(asc_w, desc_w);
}

#[test]

fn bin_sort_empty_pixa() {
    let pa = Pixa::new();
    let (sorted, indices) = pa
        .bin_sort(PixaSortType::ByX, SortOrder::Increasing)
        .unwrap();
    assert_eq!(sorted.pix_slice().len(), 0);
    assert!(indices.is_empty());
}

#[test]

fn bin_sort_rejects_unsupported_type() {
    let pa = pixa_with_widths(&[2, 3]);
    // ByArea, ByMinDimension, etc. are not supported by C `pixaBinSort`.
    assert!(
        pa.bin_sort(PixaSortType::ByArea, SortOrder::Increasing)
            .is_err()
    );
    assert!(
        pa.bin_sort(PixaSortType::ByAspectRatio, SortOrder::Increasing)
            .is_err()
    );
}

// -- scale_to_size_var -------------------------------------------------

fn build_pixaa(layout: &[&[(u32, u32)]]) -> Pixaa {
    let mut paa = Pixaa::with_capacity(layout.len());
    for sizes in layout {
        let mut inner = Pixa::with_capacity(sizes.len());
        for &(w, h) in *sizes {
            inner.push(Pix::new(w, h, PixelDepth::Bit8).unwrap());
        }
        paa.push(inner);
    }
    paa
}

#[test]

fn scale_to_size_var_wd_only() {
    let paa = build_pixaa(&[&[(8, 4), (12, 6)], &[(20, 10)]]);
    let wd = Numa::from_i32_slice(&[16, 30]);
    let out = paa.scale_to_size_var(Some(&wd), None).unwrap();
    assert_eq!(out.len(), 2);
    // Inner 0 should all have width 16
    for pix in out.get(0).unwrap().pix_slice() {
        assert_eq!(pix.width(), 16);
    }
    // Inner 1 should all have width 30
    for pix in out.get(1).unwrap().pix_slice() {
        assert_eq!(pix.width(), 30);
    }
}

#[test]

fn scale_to_size_var_hd_only() {
    let paa = build_pixaa(&[&[(8, 4)], &[(20, 10), (40, 30)]]);
    let hd = Numa::from_i32_slice(&[8, 20]);
    let out = paa.scale_to_size_var(None, Some(&hd)).unwrap();
    for pix in out.get(0).unwrap().pix_slice() {
        assert_eq!(pix.height(), 8);
    }
    for pix in out.get(1).unwrap().pix_slice() {
        assert_eq!(pix.height(), 20);
    }
}

#[test]

fn scale_to_size_var_both() {
    let paa = build_pixaa(&[&[(8, 4)], &[(20, 10)]]);
    let wd = Numa::from_i32_slice(&[16, 30]);
    let hd = Numa::from_i32_slice(&[8, 15]);
    let out = paa.scale_to_size_var(Some(&wd), Some(&hd)).unwrap();
    assert_eq!(out.get(0).unwrap().pix_slice()[0].width(), 16);
    assert_eq!(out.get(0).unwrap().pix_slice()[0].height(), 8);
    assert_eq!(out.get(1).unwrap().pix_slice()[0].width(), 30);
    assert_eq!(out.get(1).unwrap().pix_slice()[0].height(), 15);
}

#[test]

fn scale_to_size_var_requires_one_numa() {
    let paa = build_pixaa(&[&[(8, 4)]]);
    assert!(paa.scale_to_size_var(None, None).is_err());
}

#[test]

fn scale_to_size_var_size_mismatch() {
    let paa = build_pixaa(&[&[(8, 4)], &[(20, 10)]]);
    let wd = Numa::from_i32_slice(&[16]); // length 1, but paa has 2 inner
    assert!(paa.scale_to_size_var(Some(&wd), None).is_err());
}
