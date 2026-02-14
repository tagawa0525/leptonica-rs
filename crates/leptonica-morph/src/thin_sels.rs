//! Structuring elements for connectivity-preserving thinning operations
//!
//! Based on Leptonica's sel2.c implementation and the paper:
//! "Connectivity-preserving morphological image transformations"
//! (http://www.leptonica.com/papers/conn.pdf)

use crate::Sel;

/// Index into specific thinning SEL sets
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

/// Create the 9 basic SELs for 4-connected thinning
pub fn sels_4cc_thin() -> Vec<Sel> {
    todo!("thin_sels::sels_4cc_thin")
}

/// Create the 9 basic SELs for 8-connected thinning
pub fn sels_8cc_thin() -> Vec<Sel> {
    todo!("thin_sels::sels_8cc_thin")
}

/// Create the 2 SELs for both 4 and 8-connected thinning
pub fn sels_4and8cc_thin() -> Vec<Sel> {
    todo!("thin_sels::sels_4and8cc_thin")
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
pub fn make_thin_sels(_set: ThinSelSet) -> Vec<Sel> {
    todo!("thin_sels::make_thin_sels")
}
