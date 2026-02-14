//! Structuring elements for connectivity-preserving thinning operations
//!
//! Based on Leptonica's sel2.c implementation and the paper:
//! "Connectivity-preserving morphological image transformations"
//! (http://www.leptonica.com/papers/conn.pdf)

use crate::{MorphResult, Sel};

// ============================================================================
// 4-connected thinning SELs
// ============================================================================

// sel_4_1: Basic 4-cc thinning
//   x
// oCx
//   x
const SEL_4_1: &str = "  x\noCx\n  x";

// sel_4_2:
//   x
// oCx
//  o
const SEL_4_2: &str = "  x\noCx\n o ";

// sel_4_3:
//  o
// oCx
//   x
const SEL_4_3: &str = " o \noCx\n  x";

// sel_4_4:
//  o
// oCx
//  o
const SEL_4_4: &str = " o \noCx\n o ";

// sel_4_5:
//  ox
// oCx
//  o
const SEL_4_5: &str = " ox\noCx\n o ";

// sel_4_6:
//  o
// oCx
//  ox
const SEL_4_6: &str = " o \noCx\n ox";

// sel_4_7:
//  xx
// oCx
//  o
const SEL_4_7: &str = " xx\noCx\n o ";

// sel_4_8:
//   x
// oCx
// o x
const SEL_4_8: &str = "  x\noCx\no x";

// sel_4_9:
// o x
// oCx
//   x
const SEL_4_9: &str = "o x\noCx\n  x";

// ============================================================================
// 8-connected thinning SELs
// ============================================================================

// sel_8_1: Basic 8-cc thinning
//  x
// oCx
//  x
const SEL_8_1: &str = " x \noCx\n x ";

// sel_8_2:
//  x
// oCx
// o
const SEL_8_2: &str = " x \noCx\no  ";

// sel_8_3:
// o
// oCx
//  x
const SEL_8_3: &str = "o  \noCx\n x ";

// sel_8_4:
// o
// oCx
// o
const SEL_8_4: &str = "o  \noCx\no  ";

// sel_8_5:
// o x
// oCx
// o
const SEL_8_5: &str = "o x\noCx\no  ";

// sel_8_6:
// o
// oCx
// o x
const SEL_8_6: &str = "o  \noCx\no x";

// sel_8_7:
//  x
// oCx
// oo
const SEL_8_7: &str = " x \noCx\noo ";

// sel_8_8:
//  x
// oCx
// ox
const SEL_8_8: &str = " x \noCx\nox ";

// sel_8_9:
// ox
// oCx
//  x
const SEL_8_9: &str = "ox \noCx\n x ";

// ============================================================================
// SELs for both 4 and 8-connected thinning
// ============================================================================

// sel_48_1:
//  xx
// oCx
// oo
const SEL_48_1: &str = " xx\noCx\noo ";

// sel_48_2:
// o x
// oCx
// o x
const SEL_48_2: &str = "o x\noCx\no x";

/// Create a thinning SEL from a string pattern
///
/// The pattern uses:
/// - 'x' or 'X' for hit (foreground match)
/// - 'o' or 'O' for miss (background match)
/// - ' ' for don't care
/// - 'C' marks the center/origin (treated as hit)
fn sel_from_thin_pattern(pattern: &str, name: &str) -> MorphResult<Sel> {
    // Parse the pattern to find dimensions and center
    let lines: Vec<&str> = pattern.lines().collect();
    let height = lines.len() as u32;
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u32;

    let mut sel = Sel::new(width, height)?;

    // Find center position (marked with 'C')
    let mut cx = width / 2;
    let mut cy = height / 2;

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == 'C' || ch == 'c' {
                cx = x as u32;
                cy = y as u32;
            }
        }
    }

    sel.set_origin(cx, cy)?;

    // Set the elements
    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let elem = match ch {
                'x' | 'X' | 'C' | 'c' => crate::SelElement::Hit,
                'o' | 'O' => crate::SelElement::Miss,
                _ => crate::SelElement::DontCare,
            };
            sel.set_element(x as u32, y as u32, elem);
        }
    }

    sel.set_name(name);
    Ok(sel)
}

/// Create the 9 basic SELs for 4-connected thinning
pub fn sels_4cc_thin() -> Vec<Sel> {
    vec![
        sel_from_thin_pattern(SEL_4_1, "sel_4_1").unwrap(),
        sel_from_thin_pattern(SEL_4_2, "sel_4_2").unwrap(),
        sel_from_thin_pattern(SEL_4_3, "sel_4_3").unwrap(),
        sel_from_thin_pattern(SEL_4_4, "sel_4_4").unwrap(),
        sel_from_thin_pattern(SEL_4_5, "sel_4_5").unwrap(),
        sel_from_thin_pattern(SEL_4_6, "sel_4_6").unwrap(),
        sel_from_thin_pattern(SEL_4_7, "sel_4_7").unwrap(),
        sel_from_thin_pattern(SEL_4_8, "sel_4_8").unwrap(),
        sel_from_thin_pattern(SEL_4_9, "sel_4_9").unwrap(),
    ]
}

/// Create the 9 basic SELs for 8-connected thinning
pub fn sels_8cc_thin() -> Vec<Sel> {
    vec![
        sel_from_thin_pattern(SEL_8_1, "sel_8_1").unwrap(),
        sel_from_thin_pattern(SEL_8_2, "sel_8_2").unwrap(),
        sel_from_thin_pattern(SEL_8_3, "sel_8_3").unwrap(),
        sel_from_thin_pattern(SEL_8_4, "sel_8_4").unwrap(),
        sel_from_thin_pattern(SEL_8_5, "sel_8_5").unwrap(),
        sel_from_thin_pattern(SEL_8_6, "sel_8_6").unwrap(),
        sel_from_thin_pattern(SEL_8_7, "sel_8_7").unwrap(),
        sel_from_thin_pattern(SEL_8_8, "sel_8_8").unwrap(),
        sel_from_thin_pattern(SEL_8_9, "sel_8_9").unwrap(),
    ]
}

/// Create the 2 SELs for both 4 and 8-connected thinning
pub fn sels_4and8cc_thin() -> Vec<Sel> {
    vec![
        sel_from_thin_pattern(SEL_48_1, "sel_48_1").unwrap(),
        sel_from_thin_pattern(SEL_48_2, "sel_48_2").unwrap(),
    ]
}

/// Index into specific thinning SEL sets
///
/// Based on Leptonica's selaMakeThinSets() function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThinSelSet {
    /// Set 1: sel_4_1, sel_4_2, sel_4_3 (4-cc thinning, recommended)
    Set4cc1 = 1,
    /// Set 2: sel_4_1, sel_4_5, sel_4_6
    Set4cc2 = 2,
    /// Set 3: sel_4_1, sel_4_7, sel_4_7_rot
    Set4cc3 = 3,
    /// Set 4: sel_48_1, sel_48_1_rot, sel_48_2
    Set48 = 4,
    /// Set 5: sel_8_2, sel_8_3, sel_8_5, sel_8_6 (8-cc thinning, recommended)
    Set8cc1 = 5,
    /// Set 6: sel_8_2, sel_8_3, sel_48_2
    Set8cc2 = 6,
    /// Set 7: sel_8_1, sel_8_5, sel_8_6
    Set8cc3 = 7,
    /// Set 8: sel_8_2, sel_8_3, sel_8_8, sel_8_9
    Set8cc4 = 8,
    /// Set 9: sel_8_5, sel_8_6, sel_8_7, sel_8_7_rot
    Set8cc5 = 9,
    /// Set 10: sel_4_2, sel_4_3 (for thickening, use few iterations)
    Thicken4cc = 10,
    /// Set 11: sel_8_4 (for thickening, use few iterations)
    Thicken8cc = 11,
}

/// Create a specific set of thinning SELs
///
/// # Arguments
/// * `set` - The SEL set to create
///
/// # Notes
/// For a very smooth skeleton:
/// - Use Set4cc1 (set 1) for 4-connected thinning
/// - Use Set8cc1 (set 5) for 8-connected thinning
pub fn make_thin_sels(set: ThinSelSet) -> Vec<Sel> {
    let sels_4cc = sels_4cc_thin();
    let sels_8cc = sels_8cc_thin();
    let sels_48cc = sels_4and8cc_thin();

    match set {
        ThinSelSet::Set4cc1 => {
            // sel_4_1, sel_4_2, sel_4_3
            vec![
                sels_4cc[0].clone(),
                sels_4cc[1].clone(),
                sels_4cc[2].clone(),
            ]
        }
        ThinSelSet::Set4cc2 => {
            // sel_4_1, sel_4_5, sel_4_6
            vec![
                sels_4cc[0].clone(),
                sels_4cc[4].clone(),
                sels_4cc[5].clone(),
            ]
        }
        ThinSelSet::Set4cc3 => {
            // sel_4_1, sel_4_7, sel_4_7_rot
            let sel_4_7_rot = sels_4cc[6].rotate_orth(1);
            vec![sels_4cc[0].clone(), sels_4cc[6].clone(), sel_4_7_rot]
        }
        ThinSelSet::Set48 => {
            // sel_48_1, sel_48_1_rot, sel_48_2
            let sel_48_1_rot = sels_48cc[0].rotate_orth(1);
            vec![sels_48cc[0].clone(), sel_48_1_rot, sels_48cc[1].clone()]
        }
        ThinSelSet::Set8cc1 => {
            // sel_8_2, sel_8_3, sel_8_5, sel_8_6
            vec![
                sels_8cc[1].clone(),
                sels_8cc[2].clone(),
                sels_8cc[4].clone(),
                sels_8cc[5].clone(),
            ]
        }
        ThinSelSet::Set8cc2 => {
            // sel_8_2, sel_8_3, sel_48_2
            vec![
                sels_8cc[1].clone(),
                sels_8cc[2].clone(),
                sels_48cc[1].clone(),
            ]
        }
        ThinSelSet::Set8cc3 => {
            // sel_8_1, sel_8_5, sel_8_6
            vec![
                sels_8cc[0].clone(),
                sels_8cc[4].clone(),
                sels_8cc[5].clone(),
            ]
        }
        ThinSelSet::Set8cc4 => {
            // sel_8_2, sel_8_3, sel_8_8, sel_8_9
            vec![
                sels_8cc[1].clone(),
                sels_8cc[2].clone(),
                sels_8cc[7].clone(),
                sels_8cc[8].clone(),
            ]
        }
        ThinSelSet::Set8cc5 => {
            // sel_8_5, sel_8_6, sel_8_7, sel_8_7_rot
            let sel_8_7_rot = sels_8cc[6].rotate_orth(1);
            vec![
                sels_8cc[4].clone(),
                sels_8cc[5].clone(),
                sels_8cc[6].clone(),
                sel_8_7_rot,
            ]
        }
        ThinSelSet::Thicken4cc => {
            // sel_4_2, sel_4_3
            vec![sels_4cc[1].clone(), sels_4cc[2].clone()]
        }
        ThinSelSet::Thicken8cc => {
            // sel_8_4
            vec![sels_8cc[3].clone()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sels_4cc_thin() {
        let sels = sels_4cc_thin();
        assert_eq!(sels.len(), 9);

        // Check first SEL (sel_4_1)
        let sel = &sels[0];
        assert_eq!(sel.width(), 3);
        assert_eq!(sel.height(), 3);
        assert_eq!(sel.origin_x(), 1);
        assert_eq!(sel.origin_y(), 1);
        assert_eq!(sel.name(), Some("sel_4_1"));
    }

    #[test]
    fn test_sels_8cc_thin() {
        let sels = sels_8cc_thin();
        assert_eq!(sels.len(), 9);

        // Check first SEL (sel_8_1)
        let sel = &sels[0];
        assert_eq!(sel.width(), 3);
        assert_eq!(sel.height(), 3);
        assert_eq!(sel.name(), Some("sel_8_1"));
    }

    #[test]
    fn test_sels_4and8cc_thin() {
        let sels = sels_4and8cc_thin();
        assert_eq!(sels.len(), 2);
    }

    #[test]
    fn test_make_thin_sels_set1() {
        let sels = make_thin_sels(ThinSelSet::Set4cc1);
        assert_eq!(sels.len(), 3);
        assert_eq!(sels[0].name(), Some("sel_4_1"));
        assert_eq!(sels[1].name(), Some("sel_4_2"));
        assert_eq!(sels[2].name(), Some("sel_4_3"));
    }

    #[test]
    fn test_make_thin_sels_set5() {
        let sels = make_thin_sels(ThinSelSet::Set8cc1);
        assert_eq!(sels.len(), 4);
        assert_eq!(sels[0].name(), Some("sel_8_2"));
        assert_eq!(sels[1].name(), Some("sel_8_3"));
        assert_eq!(sels[2].name(), Some("sel_8_5"));
        assert_eq!(sels[3].name(), Some("sel_8_6"));
    }

    #[test]
    fn test_make_thin_sels_with_rotation() {
        // Set 3 includes a rotated SEL
        let sels = make_thin_sels(ThinSelSet::Set4cc3);
        assert_eq!(sels.len(), 3);
        assert_eq!(sels[0].name(), Some("sel_4_1"));
        assert_eq!(sels[1].name(), Some("sel_4_7"));
        // Rotated SEL should have "_rot" suffix
        assert!(sels[2].name().unwrap().contains("rot"));
    }

    #[test]
    fn test_sel_pattern_correctness() {
        // Verify sel_4_1 pattern
        //   x
        // oCx
        //   x
        let sels = sels_4cc_thin();
        let sel = &sels[0];

        // Center should be hit (C)
        assert_eq!(sel.get_element(1, 1), Some(crate::SelElement::Hit));
        // Left should be miss (o)
        assert_eq!(sel.get_element(0, 1), Some(crate::SelElement::Miss));
        // Right should be hit (x)
        assert_eq!(sel.get_element(2, 1), Some(crate::SelElement::Hit));
        // Top-right should be hit (x)
        assert_eq!(sel.get_element(2, 0), Some(crate::SelElement::Hit));
        // Bottom-right should be hit (x)
        assert_eq!(sel.get_element(2, 2), Some(crate::SelElement::Hit));
    }
}
