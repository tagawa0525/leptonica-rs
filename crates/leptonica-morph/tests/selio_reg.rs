//! Structuring Element (SEL) regression test
//!
//! This test corresponds to selio_reg.c in the C version.
//! Tests Sel creation, string parsing, reflection, rotation, and hit/miss offsets.
//!
//! C版の selio_reg.c に対応するテスト。
//!
//! Run with:
//! ```
//! cargo test -p leptonica-morph --test selio_reg -- --nocapture
//! ```

use leptonica_morph::{Sel, SelElement};
use leptonica_test::RegParams;

// ---------------------------------------------------------------
//  C版の textsel 定義（selCreateFromString用）
//  C版: 'x' = hit, 'o' = miss, ' ' = don't care
//       'O' = miss + origin, 'X' = hit + origin
//  Rust版: from_string は同じ文字規約だが、origin は別引数で指定
// ---------------------------------------------------------------

// C版 textsel1: 5行x6列, origin at (3, 1) -- 'O' is at row=1, col=3
//   "x  oo "
//   "x oOo "
//   "x  o  "
//   "x     "
//   "xxxxxx"
const TEXTSEL1: &str = "x  oo \nx oOo \nx  o  \nx     \nxxxxxx";

// C版 textsel2: textsel1 を左右反転
//   " oo  x"
//   " oOo x"
//   "  o  x"
//   "     x"
//   "xxxxxx"
const TEXTSEL2: &str = " oo  x\n oOo x\n  o  x\n     x\nxxxxxx";

// C版 textsel3: textsel1 を上下反転
//   "xxxxxx"
//   "x     "
//   "x  o  "
//   "x oOo "
//   "x  oo "
const TEXTSEL3: &str = "xxxxxx\nx     \nx  o  \nx oOo \nx  oo ";

// C版 textsel4: textsel1 を180度回転
//   "xxxxxx"
//   "     x"
//   "  o  x"
//   " oOo x"
//   " oo  x"
const TEXTSEL4: &str = "xxxxxx\n     x\n  o  x\n oOo x\n oo  x";

// C版 textsel5: origin なし（'O'がない）→ C版では NULL を返す
//   "xxxxxx"
//   "     x"
//   "  o  x"
//   " ooo x"
//   " oo  x"
const TEXTSEL5: &str = "xxxxxx\n     x\n  o  x\n ooo x\n oo  x";

// C版 textsel6: origin が2つ（'X' と 'O'）→ C版では NULL を返す
//   "xxXxxx"
//   "     x"
//   "  o  x"
//   " oOo x"
//   " oo  x"
const TEXTSEL6: &str = "xxXxxx\n     x\n  o  x\n oOo x\n oo  x";

// ==============================================================
//  Test 0-2: SELA I/O (selaAddBasic / selaWrite / selaRead)
// ==============================================================

#[test]
#[ignore = "C版: selaAddBasic() / selaWrite() / selaRead() -- SELA I/O は Rust未実装のためスキップ"]
fn test_00_02_sela_io() {
    // C版: sela1 = selaAddBasic(NULL);
    //       selaWrite("/tmp/lept/regout/sel.0.sela", sela1);
    //       regTestCheckFile(rp, "/tmp/lept/regout/sel.0.sela");  /* 0 */
    //       sela2 = selaRead("/tmp/lept/regout/sel.0.sela");
    //       selaWrite("/tmp/lept/regout/sel.1.sela", sela2);
    //       regTestCheckFile(rp, "/tmp/lept/regout/sel.1.sela");  /* 1 */
    //       regTestCompareFiles(rp, 0, 1);  /* 2 */
}

// ==============================================================
//  Test 3-4: selaCreateFromFile / selaDisplayInPix
// ==============================================================

#[test]
#[ignore = "C版: selaCreateFromFile() / selaDisplayInPix() -- Rust未実装のためスキップ"]
fn test_03_04_sela_create_from_file() {
    // C版: sela1 = selaCreateFromFile("flipsels.txt");
    //       pix = selaDisplayInPix(sela1, 31, 3, 15, 4);
    //       regTestWritePixAndCheck(rp, pix, IFF_PNG);  /* 3 */
    //       selaWrite("/tmp/lept/regout/sel.3.sela", sela1);
    //       regTestCheckFile(rp, "/tmp/lept/regout/sel.3.sela");  /* 4 */
}

// ==============================================================
//  Test 5-6: selCreateFromString + SELA comparison
// ==============================================================

#[test]
#[ignore = "C版: selaCreate() / selaAddSel() / selaWrite() / regTestCompareFiles() -- SELA I/O は Rust未実装のためスキップ"]
fn test_05_06_sela_from_string_comparison() {
    // C版: sela2 = selaCreate(4);
    //       sel = selCreateFromString(textsel1, 5, 6, "textsel1");
    //       selaAddSel(sela2, sel, NULL, 0);
    //       ...
    //       selaWrite("/tmp/lept/regout/sel.4.sela", sela2);
    //       regTestCheckFile(rp, "/tmp/lept/regout/sel.4.sela");  /* 5 */
    //       regTestCompareFiles(rp, 4, 5);  /* 6 */
}

// ==============================================================
//  Test 6-7: selCreateFromString with invalid strings
//  C版ではoriginが0個 / 2個の場合にNULLを返す
//  Rust版ではoriginは別引数で指定するためstring内の検証はない
// ==============================================================

#[test]
#[ignore = "C版: selCreateFromString()のorigin検証 -- Rust版はoriginを別引数で指定するため、文字列内のorigin検証は不要"]
fn test_06_07_invalid_string_origin() {
    // C版: sel = selCreateFromString(textsel5, 5, 6, "textsel5");
    //       val = (sel) ? 1.0 : 0.0;
    //       regTestCompareValues(rp, val, 0.0, 0.0);  /* 6 */
    //       sel = selCreateFromString(textsel6, 5, 6, "textsel6");
    //       val = (sel) ? 1.0 : 0.0;
    //       regTestCompareValues(rp, val, 0.0, 0.0);  /* 7 */
}

// ==============================================================
//  以下: C版の selio_reg.c に含まれない追加テスト
//  Rust版の Sel API を網羅的にテストする
//  C版のSel操作関数に対応するRust APIの動作確認
// ==============================================================

/// selCreateFromString のRust版テスト
/// C版 textsel1-4 をパースし、hit/miss/don't-care の要素数を検証
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel1() {
    let mut rp = RegParams::new("selio_from_string");

    // textsel1: 5 rows x 6 cols
    // "x  oo "  -> hits: x(1), misses: oo(2), dc: 3
    // "x oOo "  -> hits: x(1), misses: oOo(3), dc: 2
    // "x  o  "  -> hits: x(1), misses: o(1), dc: 4
    // "x     "  -> hits: x(1), dc: 5
    // "xxxxxx"  -> hits: xxxxxx(6)
    //
    // C版の 'O' は miss+origin。Rust版では 'O' は Miss扱い
    // Total hits: 1+1+1+1+6 = 10
    // Total misses: 2+3+1+0+0 = 6
    // Total don't care: 3+2+4+5+0 = 14

    // C版のorigin: 'O' at row=1, col=3
    let sel = Sel::from_string(TEXTSEL1, 3, 1).expect("Failed to create sel from textsel1");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);
    assert_eq!(sel.origin_x(), 3);
    assert_eq!(sel.origin_y(), 1);

    // Count hits and misses
    let hit_count = sel.hit_count();
    let miss_count = sel.miss_count();
    let total = (sel.width() * sel.height()) as usize;
    let dc_count = total - hit_count - miss_count;

    eprintln!(
        "textsel1: hits={}, misses={}, don't-care={}",
        hit_count, miss_count, dc_count
    );

    rp.compare_values(10.0, hit_count as f64, 0.0);
    rp.compare_values(6.0, miss_count as f64, 0.0);
    rp.compare_values(14.0, dc_count as f64, 0.0);

    // Verify specific elements
    // (0,0) = 'x' = Hit
    assert_eq!(sel.get_element(0, 0), Some(SelElement::Hit));
    // (3,0) = 'o' = Miss
    assert_eq!(sel.get_element(3, 0), Some(SelElement::Miss));
    // (1,0) = ' ' = DontCare
    assert_eq!(sel.get_element(1, 0), Some(SelElement::DontCare));
    // (3,1) = 'O' = Miss (origin)
    assert_eq!(sel.get_element(3, 1), Some(SelElement::Miss));

    assert!(rp.cleanup(), "selio_from_string regression test failed");
}

/// textsel2 は textsel1 の左右反転
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel2() {
    // textsel2: 5 rows x 6 cols (textsel1 の左右反転)
    // " oo  x"  -> hits: x(1), misses: oo(2), dc: 3
    // " oOo x"  -> hits: x(1), misses: oOo(3), dc: 2
    // "  o  x"  -> hits: x(1), misses: o(1), dc: 4
    // "     x"  -> hits: x(1), dc: 5
    // "xxxxxx"  -> hits: xxxxxx(6)
    // Same counts as textsel1
    // C版のorigin: 'O' at row=1, col=2
    let sel = Sel::from_string(TEXTSEL2, 2, 1).expect("Failed to create sel from textsel2");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);
    assert_eq!(sel.origin_x(), 2);
    assert_eq!(sel.origin_y(), 1);

    assert_eq!(sel.hit_count(), 10);
    assert_eq!(sel.miss_count(), 6);
}

/// textsel3 は textsel1 の上下反転
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel3() {
    // textsel3: 5 rows x 6 cols (textsel1 の上下反転)
    // C版のorigin: 'O' at row=3, col=3
    let sel = Sel::from_string(TEXTSEL3, 3, 3).expect("Failed to create sel from textsel3");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);
    assert_eq!(sel.origin_x(), 3);
    assert_eq!(sel.origin_y(), 3);

    assert_eq!(sel.hit_count(), 10);
    assert_eq!(sel.miss_count(), 6);
}

/// textsel4 は textsel1 の180度回転
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel4() {
    // textsel4: 5 rows x 6 cols (textsel1 の180度回転)
    // C版のorigin: 'O' at row=3, col=2
    let sel = Sel::from_string(TEXTSEL4, 2, 3).expect("Failed to create sel from textsel4");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);
    assert_eq!(sel.origin_x(), 2);
    assert_eq!(sel.origin_y(), 3);

    assert_eq!(sel.hit_count(), 10);
    assert_eq!(sel.miss_count(), 6);
}

/// textsel5: originなし -- Rust版ではoriginは別引数なので、文字列自体は有効にパースできる
/// C版では NULL を返す（origin検証に失敗）
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel5_no_origin() {
    // C版: selCreateFromString(textsel5, 5, 6, "textsel5") returns NULL
    // Rust版: from_string は origin を別引数で受け取るため、パース自体は成功する
    // textsel5 は 'O' がなく代わりに 'ooo' があるだけ
    let sel = Sel::from_string(TEXTSEL5, 2, 1).expect("Rust version should parse textsel5 OK");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);

    // 'o' が一つ増え、'O' が一つ減った分の差異
    // textsel4 と比較: "oOo" -> "ooo"
    // textsel5: hits=10, misses=7 (textsel4のmiss 6 + Oの代わりのo 1 - Oのmissが残っているので同じ... )
    // Actually let's count:
    // "xxxxxx" -> 6 hits
    // "     x" -> 1 hit, 5 dc
    // "  o  x" -> 1 hit, 1 miss, 4 dc
    // " ooo x" -> 1 hit, 3 miss, 2 dc  (vs textsel4's " oOo x" which has 1 hit, 3 miss, 2 dc)
    // " oo  x" -> 1 hit, 2 miss, 3 dc
    // total hits: 6+1+1+1+1 = 10
    // total misses: 0+0+1+3+2 = 6
    // Wait -- 'O' in Rust is also Miss, so textsel4 " oOo x" has misses: o, O, o = 3
    // And textsel5 " ooo x" has misses: o, o, o = 3
    // So they have the same miss count!
    assert_eq!(sel.hit_count(), 10);
    assert_eq!(sel.miss_count(), 6);
}

/// textsel6: 2つのorigin -- Rust版ではoriginは別引数なので文字列自体は有効
/// C版では NULL を返す（origin が2つ検出される）
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_from_string_textsel6_two_origins() {
    // C版: selCreateFromString(textsel6, 5, 6, "textsel6") returns NULL
    // Rust版: from_string は origin を別引数で受け取るため、パース自体は成功する
    // "xxXxxx" -> 'X' はRust版では Hit扱い
    let sel = Sel::from_string(TEXTSEL6, 2, 1).expect("Rust version should parse textsel6 OK");
    assert_eq!(sel.width(), 6);
    assert_eq!(sel.height(), 5);

    // "xxXxxx" -> 6 hits (X is Hit in Rust)
    // "     x" -> 1 hit, 5 dc
    // "  o  x" -> 1 hit, 1 miss, 4 dc
    // " oOo x" -> 1 hit, 3 miss (O is Miss), 2 dc
    // " oo  x" -> 1 hit, 2 miss, 3 dc
    // hits: 6+1+1+1+1 = 10
    // misses: 0+0+1+3+2 = 6
    assert_eq!(sel.hit_count(), 10);
    assert_eq!(sel.miss_count(), 6);
}

// ==============================================================
//  Sel creation factory methods
// ==============================================================

/// selCreate (brick) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_brick() {
    let mut rp = RegParams::new("selio_brick");

    // 各種サイズの brick SEL を作成
    for &(w, h) in &[(1, 1), (3, 3), (5, 5), (7, 7), (21, 15), (1, 10), (10, 1)] {
        let sel = Sel::create_brick(w, h)
            .unwrap_or_else(|_| panic!("Failed to create brick {}x{}", w, h));

        // 寸法チェック
        rp.compare_values(w as f64, sel.width() as f64, 0.0);
        rp.compare_values(h as f64, sel.height() as f64, 0.0);

        // Origin は中心にあるべき
        rp.compare_values((w / 2) as f64, sel.origin_x() as f64, 0.0);
        rp.compare_values((h / 2) as f64, sel.origin_y() as f64, 0.0);

        // 全要素が Hit であるべき
        let expected_hits = (w * h) as usize;
        rp.compare_values(expected_hits as f64, sel.hit_count() as f64, 0.0);

        // Miss は0
        rp.compare_values(0.0, sel.miss_count() as f64, 0.0);
    }

    // Zero size は error
    assert!(Sel::create_brick(0, 1).is_err());
    assert!(Sel::create_brick(1, 0).is_err());
    assert!(Sel::create_brick(0, 0).is_err());

    assert!(rp.cleanup(), "selio_brick regression test failed");
}

/// selCreate (square) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_square() {
    let mut rp = RegParams::new("selio_square");

    for &size in &[1, 3, 5, 7, 11] {
        let sel =
            Sel::create_square(size).unwrap_or_else(|_| panic!("Failed to create square {}", size));

        rp.compare_values(size as f64, sel.width() as f64, 0.0);
        rp.compare_values(size as f64, sel.height() as f64, 0.0);
        rp.compare_values((size / 2) as f64, sel.origin_x() as f64, 0.0);
        rp.compare_values((size / 2) as f64, sel.origin_y() as f64, 0.0);
        rp.compare_values((size * size) as f64, sel.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "selio_square regression test failed");
}

/// selCreate (horizontal / vertical) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_horizontal_vertical() {
    let mut rp = RegParams::new("selio_hv");

    for &len in &[1, 3, 5, 11, 21] {
        // Horizontal: width=len, height=1
        let sel_h = Sel::create_horizontal(len)
            .unwrap_or_else(|_| panic!("Failed to create horizontal {}", len));
        rp.compare_values(len as f64, sel_h.width() as f64, 0.0);
        rp.compare_values(1.0, sel_h.height() as f64, 0.0);
        rp.compare_values(len as f64, sel_h.hit_count() as f64, 0.0);

        // Vertical: width=1, height=len
        let sel_v = Sel::create_vertical(len)
            .unwrap_or_else(|_| panic!("Failed to create vertical {}", len));
        rp.compare_values(1.0, sel_v.width() as f64, 0.0);
        rp.compare_values(len as f64, sel_v.height() as f64, 0.0);
        rp.compare_values(len as f64, sel_v.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "selio_hv regression test failed");
}

/// selCreate (cross) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_cross() {
    let mut rp = RegParams::new("selio_cross");

    // Cross of size n: horizontal line + vertical line through center
    // Hit count = 2*n - 1 (center counted once)
    for &size in &[1, 3, 5, 7, 11] {
        let sel =
            Sel::create_cross(size).unwrap_or_else(|_| panic!("Failed to create cross {}", size));

        rp.compare_values(size as f64, sel.width() as f64, 0.0);
        rp.compare_values(size as f64, sel.height() as f64, 0.0);
        rp.compare_values((size / 2) as f64, sel.origin_x() as f64, 0.0);
        rp.compare_values((size / 2) as f64, sel.origin_y() as f64, 0.0);

        let expected_hits = 2 * size - 1;
        rp.compare_values(expected_hits as f64, sel.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "selio_cross regression test failed");
}

/// selCreate (diamond) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_diamond() {
    let mut rp = RegParams::new("selio_diamond");

    // Diamond with radius r: size = 2r+1, hit count = 2*r*(r+1)+1
    // r=1: size=3, hits=5
    // r=2: size=5, hits=13
    // r=3: size=7, hits=25
    for &radius in &[1, 2, 3, 4, 5] {
        let sel = Sel::create_diamond(radius)
            .unwrap_or_else(|_| panic!("Failed to create diamond {}", radius));

        let size = 2 * radius + 1;
        rp.compare_values(size as f64, sel.width() as f64, 0.0);
        rp.compare_values(size as f64, sel.height() as f64, 0.0);
        rp.compare_values(radius as f64, sel.origin_x() as f64, 0.0);
        rp.compare_values(radius as f64, sel.origin_y() as f64, 0.0);

        // Diamond hit count: sum of (2*min(y, 2r-y) + 1) for y=0..2r
        let expected_hits = 2 * radius * (radius + 1) + 1;
        rp.compare_values(expected_hits as f64, sel.hit_count() as f64, 0.0);
    }

    assert!(rp.cleanup(), "selio_diamond regression test failed");
}

/// selCreate (disk) のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_create_disk() {
    let mut rp = RegParams::new("selio_disk");

    for &radius in &[1, 2, 3, 4, 5] {
        let sel =
            Sel::create_disk(radius).unwrap_or_else(|_| panic!("Failed to create disk {}", radius));

        let size = 2 * radius + 1;
        rp.compare_values(size as f64, sel.width() as f64, 0.0);
        rp.compare_values(size as f64, sel.height() as f64, 0.0);
        rp.compare_values(radius as f64, sel.origin_x() as f64, 0.0);
        rp.compare_values(radius as f64, sel.origin_y() as f64, 0.0);

        // Disk should have at least as many hits as the diamond of same radius
        let diamond_hits = 2 * radius * (radius + 1) + 1;
        let disk_hits = sel.hit_count();
        eprintln!(
            "  disk radius={}: hits={}, diamond_hits={}",
            radius, disk_hits, diamond_hits
        );
        assert!(
            disk_hits >= diamond_hits as usize,
            "Disk should have >= diamond hits for radius {}",
            radius
        );

        // Disk should have at most all elements as hits
        let max_hits = (size * size) as usize;
        assert!(
            disk_hits <= max_hits,
            "Disk hits should not exceed total elements"
        );
    }

    assert!(rp.cleanup(), "selio_disk regression test failed");
}

// ==============================================================
//  Sel reflect / rotate_orth
// ==============================================================

/// selReflect (180度回転) のテスト
/// C版: textsel1 を reflect すると textsel4 と同じパターンになるべき
#[test]
#[ignore = "not yet implemented"]
fn test_sel_reflect() {
    let mut rp = RegParams::new("selio_reflect");

    // textsel1 を reflect すると textsel4 と同じパターンになるべき
    // textsel1 origin: (3, 1), reflect → origin: (6-1-3, 5-1-1) = (2, 3)
    let sel1 = Sel::from_string(TEXTSEL1, 3, 1).expect("textsel1");
    let sel4 = Sel::from_string(TEXTSEL4, 2, 3).expect("textsel4");

    let reflected = sel1.reflect();

    // 寸法は同じ
    rp.compare_values(sel4.width() as f64, reflected.width() as f64, 0.0);
    rp.compare_values(sel4.height() as f64, reflected.height() as f64, 0.0);

    // Origin チェック
    rp.compare_values(sel4.origin_x() as f64, reflected.origin_x() as f64, 0.0);
    rp.compare_values(sel4.origin_y() as f64, reflected.origin_y() as f64, 0.0);

    // 全要素が一致
    let mut all_match = true;
    for y in 0..reflected.height() {
        for x in 0..reflected.width() {
            if reflected.get_element(x, y) != sel4.get_element(x, y) {
                eprintln!(
                    "  Mismatch at ({}, {}): reflected={:?}, sel4={:?}",
                    x,
                    y,
                    reflected.get_element(x, y),
                    sel4.get_element(x, y)
                );
                all_match = false;
            }
        }
    }
    rp.compare_values(1.0, if all_match { 1.0 } else { 0.0 }, 0.0);

    // reflect を2回適用すると元に戻る
    let double_reflected = reflected.reflect();
    let mut round_trip = true;
    for y in 0..sel1.height() {
        for x in 0..sel1.width() {
            if double_reflected.get_element(x, y) != sel1.get_element(x, y) {
                round_trip = false;
            }
        }
    }
    rp.compare_values(1.0, if round_trip { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(
        sel1.origin_x() as f64,
        double_reflected.origin_x() as f64,
        0.0,
    );
    rp.compare_values(
        sel1.origin_y() as f64,
        double_reflected.origin_y() as f64,
        0.0,
    );

    assert!(rp.cleanup(), "selio_reflect regression test failed");
}

/// selReflect: symmetric SEL (brick) は reflect しても同じ
#[test]
#[ignore = "not yet implemented"]
fn test_sel_reflect_symmetric() {
    let sel = Sel::create_brick(3, 3).expect("brick 3x3");
    let reflected = sel.reflect();

    assert_eq!(sel.width(), reflected.width());
    assert_eq!(sel.height(), reflected.height());
    assert_eq!(sel.origin_x(), reflected.origin_x());
    assert_eq!(sel.origin_y(), reflected.origin_y());

    for y in 0..sel.height() {
        for x in 0..sel.width() {
            assert_eq!(
                sel.get_element(x, y),
                reflected.get_element(x, y),
                "Symmetric SEL should be unchanged by reflect at ({}, {})",
                x,
                y
            );
        }
    }
}

/// selRotateOrth のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_rotate_orth() {
    let mut rp = RegParams::new("selio_rotate");

    // テスト用の非対称 SEL: 3x2 (width=3, height=2)
    // "xx."
    // "..."
    let sel = Sel::from_string("xx.\n...", 0, 0).expect("test sel");

    // rotation=0: identity
    let rot0 = sel.rotate_orth(0);
    rp.compare_values(sel.width() as f64, rot0.width() as f64, 0.0);
    rp.compare_values(sel.height() as f64, rot0.height() as f64, 0.0);
    rp.compare_values(sel.origin_x() as f64, rot0.origin_x() as f64, 0.0);
    rp.compare_values(sel.origin_y() as f64, rot0.origin_y() as f64, 0.0);

    // rotation=1: 90 degrees CW → width/height swap
    let rot1 = sel.rotate_orth(1);
    rp.compare_values(sel.height() as f64, rot1.width() as f64, 0.0);
    rp.compare_values(sel.width() as f64, rot1.height() as f64, 0.0);

    // rotation=2: 180 degrees = reflect
    let rot2 = sel.rotate_orth(2);
    let reflected = sel.reflect();
    rp.compare_values(reflected.width() as f64, rot2.width() as f64, 0.0);
    rp.compare_values(reflected.height() as f64, rot2.height() as f64, 0.0);
    rp.compare_values(reflected.origin_x() as f64, rot2.origin_x() as f64, 0.0);
    rp.compare_values(reflected.origin_y() as f64, rot2.origin_y() as f64, 0.0);

    let mut match_180 = true;
    for y in 0..rot2.height() {
        for x in 0..rot2.width() {
            if rot2.get_element(x, y) != reflected.get_element(x, y) {
                match_180 = false;
            }
        }
    }
    rp.compare_values(1.0, if match_180 { 1.0 } else { 0.0 }, 0.0);

    // rotation=3: 270 degrees CW
    let rot3 = sel.rotate_orth(3);
    rp.compare_values(sel.height() as f64, rot3.width() as f64, 0.0);
    rp.compare_values(sel.width() as f64, rot3.height() as f64, 0.0);

    // rotation=4: same as 0 (identity)
    let rot4 = sel.rotate_orth(4);
    let mut match_identity = true;
    for y in 0..sel.height() {
        for x in 0..sel.width() {
            if rot4.get_element(x, y) != sel.get_element(x, y) {
                match_identity = false;
            }
        }
    }
    rp.compare_values(1.0, if match_identity { 1.0 } else { 0.0 }, 0.0);

    // 4回90度回転すると元に戻る
    let round_trip = sel
        .rotate_orth(1)
        .rotate_orth(1)
        .rotate_orth(1)
        .rotate_orth(1);
    let mut match_round = true;
    for y in 0..sel.height() {
        for x in 0..sel.width() {
            if round_trip.get_element(x, y) != sel.get_element(x, y) {
                match_round = false;
            }
        }
    }
    rp.compare_values(1.0, if match_round { 1.0 } else { 0.0 }, 0.0);
    rp.compare_values(sel.origin_x() as f64, round_trip.origin_x() as f64, 0.0);
    rp.compare_values(sel.origin_y() as f64, round_trip.origin_y() as f64, 0.0);

    assert!(rp.cleanup(), "selio_rotate regression test failed");
}

/// selRotateOrth: 対称 SEL (square) の回転テスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_rotate_orth_symmetric() {
    let sel = Sel::create_square(5).expect("square 5");

    for rot in 0..4 {
        let rotated = sel.rotate_orth(rot);
        assert_eq!(sel.width(), rotated.width(), "rot={}", rot);
        assert_eq!(sel.height(), rotated.height(), "rot={}", rot);
        assert_eq!(sel.hit_count(), rotated.hit_count(), "rot={}", rot);
    }
}

// ==============================================================
//  Hit / Miss offsets
// ==============================================================

/// selGetHitMissOffsets のテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_hit_miss_offsets() {
    let mut rp = RegParams::new("selio_offsets");

    // from_string でhit/miss/don't-care の混在パターン
    // "xo."
    // ".ox"
    // Origin at (1, 1) -- center
    let sel = Sel::from_string("xo.\n.ox", 1, 0).expect("test sel");

    // hit offsets: (0,0)->(0-1,0-0)=(-1,0), (2,1)->(2-1,1-0)=(1,1)
    // Wait, origin is (1, 0), so offsets are (x - origin_x, y - origin_y)
    let hit_offsets: Vec<(i32, i32)> = sel.hit_offsets().collect();
    eprintln!("Hit offsets: {:?}", hit_offsets);
    rp.compare_values(2.0, hit_offsets.len() as f64, 0.0);

    // Hits are at: (0,0)='x' and (2,1)='x'
    // Offsets: (0-1, 0-0) = (-1, 0) and (2-1, 1-0) = (1, 1)
    assert!(
        hit_offsets.contains(&(-1, 0)),
        "Should contain (-1, 0), got {:?}",
        hit_offsets
    );
    assert!(
        hit_offsets.contains(&(1, 1)),
        "Should contain (1, 1), got {:?}",
        hit_offsets
    );

    // miss offsets: (1,0)='o' and (1,1)='o'
    // Offsets: (1-1, 0-0) = (0, 0) and (1-1, 1-0) = (0, 1)
    let miss_offsets: Vec<(i32, i32)> = sel.miss_offsets().collect();
    eprintln!("Miss offsets: {:?}", miss_offsets);
    rp.compare_values(2.0, miss_offsets.len() as f64, 0.0);

    assert!(
        miss_offsets.contains(&(0, 0)),
        "Should contain (0, 0), got {:?}",
        miss_offsets
    );
    assert!(
        miss_offsets.contains(&(0, 1)),
        "Should contain (0, 1), got {:?}",
        miss_offsets
    );

    assert!(rp.cleanup(), "selio_offsets regression test failed");
}

/// brick SEL の hit offsets テスト: 全要素が Hit なので offset 数 = w*h
#[test]
#[ignore = "not yet implemented"]
fn test_sel_hit_offsets_brick() {
    let sel = Sel::create_brick(3, 3).expect("brick 3x3");
    let offsets: Vec<(i32, i32)> = sel.hit_offsets().collect();

    assert_eq!(offsets.len(), 9);

    // Origin is (1, 1), so offsets range from (-1,-1) to (1,1)
    for dy in -1..=1_i32 {
        for dx in -1..=1_i32 {
            assert!(
                offsets.contains(&(dx, dy)),
                "Should contain ({}, {})",
                dx,
                dy
            );
        }
    }
}

/// miss のない SEL の miss_offsets は空
#[test]
#[ignore = "not yet implemented"]
fn test_sel_miss_offsets_empty_for_brick() {
    let sel = Sel::create_brick(5, 5).expect("brick 5x5");
    let miss_offsets: Vec<(i32, i32)> = sel.miss_offsets().collect();
    assert_eq!(miss_offsets.len(), 0);
}

// ==============================================================
//  Sel new / set_element / get_element
// ==============================================================

/// 空の SEL を作成し、要素を個別に設定するテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_new_and_element_access() {
    let mut sel = Sel::new(4, 3).expect("new 4x3");

    assert_eq!(sel.width(), 4);
    assert_eq!(sel.height(), 3);
    // 全要素が DontCare
    for y in 0..3 {
        for x in 0..4 {
            assert_eq!(sel.get_element(x, y), Some(SelElement::DontCare));
        }
    }

    // 要素を設定
    sel.set_element(0, 0, SelElement::Hit);
    sel.set_element(1, 1, SelElement::Miss);
    sel.set_element(3, 2, SelElement::Hit);

    assert_eq!(sel.get_element(0, 0), Some(SelElement::Hit));
    assert_eq!(sel.get_element(1, 1), Some(SelElement::Miss));
    assert_eq!(sel.get_element(3, 2), Some(SelElement::Hit));
    assert_eq!(sel.get_element(2, 2), Some(SelElement::DontCare));

    // 範囲外
    assert_eq!(sel.get_element(4, 0), None);
    assert_eq!(sel.get_element(0, 3), None);

    assert_eq!(sel.hit_count(), 2);
    assert_eq!(sel.miss_count(), 1);
}

/// SEL のゼロサイズは error を返す
#[test]
#[ignore = "not yet implemented"]
fn test_sel_new_zero_size() {
    assert!(Sel::new(0, 1).is_err());
    assert!(Sel::new(1, 0).is_err());
    assert!(Sel::new(0, 0).is_err());
}

// ==============================================================
//  Sel name / origin
// ==============================================================

/// SEL の name 管理テスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_name() {
    let mut sel = Sel::new(3, 3).expect("new 3x3");
    assert!(sel.name().is_none());

    sel.set_name("test_sel");
    assert_eq!(sel.name(), Some("test_sel"));

    // brick には自動で名前がつく
    let brick = Sel::create_brick(5, 3).expect("brick 5x3");
    assert!(brick.name().is_some());
    eprintln!("Brick name: {:?}", brick.name());
}

/// SEL の origin 管理テスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_set_origin() {
    let mut sel = Sel::new(5, 5).expect("new 5x5");

    // デフォルトの origin は中心
    assert_eq!(sel.origin_x(), 2);
    assert_eq!(sel.origin_y(), 2);

    // origin を変更
    sel.set_origin(0, 0).expect("set origin (0,0)");
    assert_eq!(sel.origin_x(), 0);
    assert_eq!(sel.origin_y(), 0);

    sel.set_origin(4, 4).expect("set origin (4,4)");
    assert_eq!(sel.origin_x(), 4);
    assert_eq!(sel.origin_y(), 4);

    // 範囲外はエラー
    assert!(sel.set_origin(5, 0).is_err());
    assert!(sel.set_origin(0, 5).is_err());
}

// ==============================================================
//  Sel data access
// ==============================================================

/// data() メソッドのテスト
#[test]
#[ignore = "not yet implemented"]
fn test_sel_data() {
    let sel = Sel::create_brick(2, 2).expect("brick 2x2");
    let data = sel.data();
    assert_eq!(data.len(), 4);
    assert!(data.iter().all(|&e| e == SelElement::Hit));

    let sel2 = Sel::new(2, 2).expect("new 2x2");
    let data2 = sel2.data();
    assert_eq!(data2.len(), 4);
    assert!(data2.iter().all(|&e| e == SelElement::DontCare));
}

// ==============================================================
//  Sel clone
// ==============================================================

/// Sel の clone テスト（Clone derive）
#[test]
#[ignore = "not yet implemented"]
fn test_sel_clone() {
    let sel = Sel::from_string("xo.\n.ox", 1, 0).expect("test sel");
    let cloned = sel.clone();

    assert_eq!(sel.width(), cloned.width());
    assert_eq!(sel.height(), cloned.height());
    assert_eq!(sel.origin_x(), cloned.origin_x());
    assert_eq!(sel.origin_y(), cloned.origin_y());
    assert_eq!(sel.hit_count(), cloned.hit_count());
    assert_eq!(sel.miss_count(), cloned.miss_count());

    for y in 0..sel.height() {
        for x in 0..sel.width() {
            assert_eq!(sel.get_element(x, y), cloned.get_element(x, y));
        }
    }
}

// ==============================================================
//  from_string edge cases
// ==============================================================

/// 空文字列は error を返す
#[test]
#[ignore = "not yet implemented"]
fn test_sel_from_string_empty() {
    assert!(Sel::from_string("", 0, 0).is_err());
}

/// 1x1 の SEL
#[test]
#[ignore = "not yet implemented"]
fn test_sel_from_string_single() {
    let sel = Sel::from_string("x", 0, 0).expect("single hit");
    assert_eq!(sel.width(), 1);
    assert_eq!(sel.height(), 1);
    assert_eq!(sel.hit_count(), 1);
    assert_eq!(sel.origin_x(), 0);
    assert_eq!(sel.origin_y(), 0);
}

/// 大きめの SEL
#[test]
#[ignore = "not yet implemented"]
fn test_sel_from_string_large() {
    // 7x7 cross pattern
    let pattern = "\
...x...\n\
...x...\n\
...x...\n\
xxxxxxx\n\
...x...\n\
...x...\n\
...x...";
    let sel = Sel::from_string(pattern, 3, 3).expect("7x7 cross");
    assert_eq!(sel.width(), 7);
    assert_eq!(sel.height(), 7);
    assert_eq!(sel.origin_x(), 3);
    assert_eq!(sel.origin_y(), 3);

    // Cross: 7 horizontal + 7 vertical - 1 center = 13
    assert_eq!(sel.hit_count(), 13);
}
