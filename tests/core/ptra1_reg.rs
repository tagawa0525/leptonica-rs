//! Pointer-array regression test (part 1)
//!
//! Uses `Ptaa` operations as the Rust mapping for pointer-array behavior.
//!
//! # See also
//!
//! C Leptonica: `prog/ptra1_reg.c`

use crate::common::RegParams;
use leptonica::{Pta, Ptaa};

#[test]
fn ptra1_reg() {
    let mut rp = RegParams::new("ptra1");

    let mut ptaa = Ptaa::new();

    let mut pa0 = Pta::new();
    pa0.push(1.0, 1.0);
    pa0.push(2.0, 2.0);
    ptaa.push(pa0);

    let mut pa1 = Pta::new();
    pa1.push(10.0, 10.0);
    ptaa.push(pa1);

    ptaa.add_pt(1, 20.0, 20.0).expect("add_pt");
    rp.compare_values(2.0, ptaa.len() as f64, 0.0);
    rp.compare_values(4.0, ptaa.total_points() as f64, 0.0);

    let repl = Pta::from_vecs(vec![5.0, 6.0], vec![7.0, 8.0]).expect("from_vecs");
    ptaa.replace(0, repl).expect("replace");
    let (x, y) = ptaa.get(0).expect("pta").get(1).expect("point");
    rp.compare_values(6.0, x as f64, 0.0);
    rp.compare_values(8.0, y as f64, 0.0);

    // Additional: verify element integrity after operations
    let pta0 = ptaa.get(0).expect("pta0 after replace");
    rp.compare_values(2.0, pta0.len() as f64, 0.0);

    // Remove and verify
    let removed = ptaa.pop().expect("pop last");
    rp.compare_values(1.0, ptaa.len() as f64, 0.0);
    rp.compare_values(2.0, removed.len() as f64, 0.0);

    // Flatten remaining
    let flat = ptaa.flatten();
    rp.compare_values(2.0, flat.len() as f64, 0.0);

    assert!(rp.cleanup(), "ptra1 regression test failed");
}

/// C-compat: `prog/ptra1_reg.c`, all 18 outputs.
///
/// `lucasta.1.300.tif` is lossless and C writes every output as PNG, so the
/// whole program is bit-exactly comparable. It exercises `L_PTRA` insert /
/// remove / swap / compaction against the connected components of a page.
#[test]
fn ptra1_c_compat() {
    use leptonica::core::{Box, Compaction, DownShift, Pixa, Ptra};
    use leptonica::io::ImageFormat;
    use leptonica::region::{ConnectivityType, conncomp_pixa};
    use leptonica::{Pix, PixelDepth};

    if crate::common::is_display_mode() {
        return;
    }

    /// C: MakePtrasFromPixa — one Ptra of pix, one of boxes.
    fn make_ptras(pixa: &Pixa) -> (Ptra<Pix>, Ptra<Box>) {
        let n = pixa.len();
        let mut papix = Ptra::with_capacity(n);
        let mut pabox = Ptra::with_capacity(n);
        for i in 0..n {
            papix.add(pixa.get(i).expect("pix").clone());
            pabox.add(*pixa.boxa().get(i).expect("box"));
        }
        (papix, pabox)
    }

    /// C: ReconstructPixa1 — drain in index order, leaving holes.
    fn reconstruct1(papix: &mut Ptra<Pix>, pabox: &mut Ptra<Box>) -> Pixa {
        let imax = papix.max_index();
        let mut pixat = Pixa::new();
        for i in 0..=imax {
            let pix = papix.remove(i, Compaction::No).expect("remove pix");
            let b = pabox.remove(i, Compaction::No).expect("remove box");
            if let Some(p) = pix {
                pixat.push(p);
            }
            if let Some(b) = b {
                pixat.add_box(b);
            }
        }
        pixat
    }

    /// C: ReconstructPixa2 — take every other, compact, then drain from 0.
    fn reconstruct2(papix: &mut Ptra<Pix>, pabox: &mut Ptra<Box>) -> Pixa {
        let imax = papix.max_index();
        let mut pixat = Pixa::new();
        for i in (0..=imax).step_by(2) {
            let pix = papix.remove(i, Compaction::No).expect("remove pix");
            let b = pabox.remove(i, Compaction::No).expect("remove box");
            if let Some(p) = pix {
                pixat.push(p);
            }
            if let Some(b) = b {
                pixat.add_box(b);
            }
        }
        papix.compact();
        pabox.compact();
        while papix.actual_count() != 0 {
            let pix = papix.remove(0, Compaction::Yes).expect("remove pix");
            let b = pabox.remove(0, Compaction::Yes).expect("remove box");
            if let Some(p) = pix {
                pixat.push(p);
            }
            if let Some(b) = b {
                pixat.add_box(b);
            }
        }
        pixat
    }

    /// C: SaveResult — render into a w x h canvas and keep a copy.
    fn save_result(pixac: &mut Pixa, pixa: &Pixa, w: u32, h: u32) -> Pix {
        let pixd = pixa.display(w, h).expect("pixa display");
        pixac.push(pixd.clone());
        pixd
    }

    let mut rp = RegParams::new("ptra1_c");

    let pixs = crate::common::load_test_image("lucasta.1.300.tif").expect("load lucasta");
    assert_eq!(pixs.depth(), PixelDepth::Bit1);
    let (w, h) = (pixs.width(), pixs.height());
    let (_boxa, pixas) = conncomp_pixa(&pixs, ConnectivityType::EightWay).expect("conn comp");
    let n = pixas.len();

    let mut pixac1 = Pixa::new();
    let mut pixac2 = Pixa::new();

    // C 0, 1: fill with clones and reconstruct.
    for reconstruct2_pass in [false, true] {
        let (mut papix, mut pabox) = make_ptras(&pixas);
        let pixa = if reconstruct2_pass {
            reconstruct2(&mut papix, &mut pabox)
        } else {
            reconstruct1(&mut papix, &mut pabox)
        };
        let pixac = if reconstruct2_pass {
            &mut pixac2
        } else {
            &mut pixac1
        };
        let pixd = save_result(pixac, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: clones reconstructed");
    }

    // C 2, 3: remove every other in the first half, with compaction.
    for reconstruct2_pass in [false, true] {
        let (mut papix, mut pabox) = make_ptras(&pixas);
        for i in (0..n / 2).step_by(2) {
            papix.remove(i as i32, Compaction::Yes).expect("remove pix");
            pabox.remove(i as i32, Compaction::Yes).expect("remove box");
        }
        let pixa = if reconstruct2_pass {
            reconstruct2(&mut papix, &mut pabox)
        } else {
            reconstruct1(&mut papix, &mut pabox)
        };
        let pixac = if reconstruct2_pass {
            &mut pixac2
        } else {
            &mut pixac1
        };
        let pixd = save_result(pixac, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: half removed with compaction");
    }

    // C 4, 5: remove every other everywhere without compaction, then compact.
    for reconstruct2_pass in [false, true] {
        let (mut papix, mut pabox) = make_ptras(&pixas);
        for i in (0..n).step_by(2) {
            papix.remove(i as i32, Compaction::No).expect("remove pix");
            pabox.remove(i as i32, Compaction::No).expect("remove box");
        }
        papix.compact();
        pabox.compact();
        let pixa = if reconstruct2_pass {
            reconstruct2(&mut papix, &mut pabox)
        } else {
            reconstruct1(&mut papix, &mut pabox)
        };
        let pixac = if reconstruct2_pass {
            &mut pixac2
        } else {
            &mut pixac1
        };
        let pixd = save_result(pixac, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: compacted after holes");
    }

    // C 6, 7: insert at the head, reversing the order.
    for reconstruct2_pass in [false, true] {
        let mut papix: Ptra<Pix> = Ptra::with_capacity(n);
        let mut pabox: Ptra<Box> = Ptra::with_capacity(n);
        for i in 0..n {
            papix
                .insert(0, pixas.get(i).expect("pix").clone(), DownShift::Min)
                .expect("insert pix");
            pabox
                .insert(0, *pixas.boxa().get(i).expect("box"), DownShift::Full)
                .expect("insert box");
        }
        let pixa = if reconstruct2_pass {
            reconstruct2(&mut papix, &mut pabox)
        } else {
            reconstruct1(&mut papix, &mut pabox)
        };
        let pixac = if reconstruct2_pass {
            &mut pixac2
        } else {
            &mut pixac1
        };
        let pixd = save_result(pixac, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: inserted at head");
    }

    // C 8, 9: reverse by swapping; the compaction should be a no-op.
    for reconstruct2_pass in [false, true] {
        let (mut papix, mut pabox) = make_ptras(&pixas);
        for i in 0..n / 2 {
            papix.swap(i as i32, (n - i - 1) as i32).expect("swap pix");
            pabox.swap(i as i32, (n - i - 1) as i32).expect("swap box");
        }
        papix.compact();
        pabox.compact();
        let pixa = if reconstruct2_pass {
            reconstruct2(&mut papix, &mut pabox)
        } else {
            reconstruct1(&mut papix, &mut pabox)
        };
        let pixac = if reconstruct2_pass {
            &mut pixac2
        } else {
            &mut pixac1
        };
        let pixd = save_result(pixac, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: reversed by swapping");
    }

    // C 10, 11: remove at the top and push the hole to the end by swapping.
    {
        let (mut papix, mut pabox) = make_ptras(&pixas);
        let mut papix2: Ptra<Pix> = Ptra::new();
        let mut pabox2: Ptra<Box> = Ptra::new();
        while papix.actual_count() != 0 {
            let imax = papix.max_index();
            if let Some(p) = papix.remove(0, Compaction::No).expect("remove pix") {
                papix2.add(p);
            }
            if let Some(b) = pabox.remove(0, Compaction::No).expect("remove box") {
                pabox2.add(b);
            }
            for i in 1..=imax {
                papix.swap(i - 1, i).expect("swap pix");
                pabox.swap(i - 1, i).expect("swap box");
            }
        }
        papix.compact();
        pabox.compact();

        // C 10: the source is now empty.
        let pixa = reconstruct1(&mut papix, &mut pabox);
        let pixd = save_result(&mut pixac1, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: emptied by swapping");

        // C 11: everything ended up in the second pair of arrays.
        let pixa = reconstruct1(&mut papix2, &mut pabox2);
        let pixd = save_result(&mut pixac1, &pixa, w, h);
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: moved by swapping");
    }

    // C 12-15: remove and re-insert one position above.
    for shift in [DownShift::Min, DownShift::Auto] {
        for reconstruct2_pass in [false, true] {
            let (mut papix, mut pabox) = make_ptras(&pixas);
            for i in 1..n as i32 {
                let pix = papix.remove(i, Compaction::No).expect("remove pix");
                let b = pabox.remove(i, Compaction::No).expect("remove box");
                if let Some(p) = pix {
                    papix.insert(i - 1, p, shift).expect("insert pix");
                }
                if let Some(b) = b {
                    pabox.insert(i - 1, b, shift).expect("insert box");
                }
            }
            let pixa = if reconstruct2_pass {
                reconstruct2(&mut papix, &mut pabox)
            } else {
                reconstruct1(&mut papix, &mut pabox)
            };
            let pixac = if reconstruct2_pass {
                &mut pixac2
            } else {
                &mut pixac1
            };
            let pixd = save_result(pixac, &pixa, w, h);
            rp.write_pix_and_check(&pixd, ImageFormat::Png)
                .expect("check: shifted up one");
        }
    }

    // C 16, 17: tiled summaries of everything above.
    for pixac in [&pixac1, &pixac2] {
        let pixd = pixac
            .display_tiled_in_columns(10, 0.5, 15, 2)
            .expect("tile ptra results");
        rp.write_pix_and_check(&pixd, ImageFormat::Png)
            .expect("check: ptra summary");
    }

    assert!(rp.cleanup(), "ptra1 C-compat test failed");
}
