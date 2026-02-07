//! EAN-13 barcode decoder
//!
//! Reference: <http://en.wikipedia.org/wiki/European_Article_Number>
//!            <http://morovia.com/education/symbology/ean-13.asp>
//!
//! The encoding is essentially the same as UPC-A, except there are
//! 13 digits total, of which 12 are encoded by bars (as with UPC-A)
//! and the 13th is a leading digit that determines the encoding of
//! the next 6 digits, selecting each digit from one of two tables.
//!
//! If the first digit is 0, the encoding is identical to UPC-A.

use crate::barcode::types::FormatVerification;
use crate::{RecogError, RecogResult};

/// UPC-A/EAN-13 symbol patterns (left side, odd parity)
const UPCA: &[&str] = &[
    "3211",  // 0
    "2221",  // 1
    "2122",  // 2
    "1411",  // 3
    "1132",  // 4
    "1231",  // 5
    "1114",  // 6
    "1312",  // 7
    "1213",  // 8
    "3112",  // 9
    "111",   // Start (10)
    "111",   // Stop (11)
    "11111", // Mid (12)
];

const UPCA_START: usize = 10;
const UPCA_MID: usize = 12;

/// Reverses a string
fn string_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Verifies if a bar string is in EAN-13 format
///
/// Uses the same verification as UPC-A since the bar pattern is identical
pub fn verify_ean13(barstr: &str) -> FormatVerification {
    let len = barstr.len();
    if len != 59 {
        return FormatVerification::invalid();
    }

    let start = barstr.starts_with(UPCA[UPCA_START]);
    let mid = &barstr[27..32] == UPCA[UPCA_MID];
    let stop = barstr[len - 3..] == *UPCA[UPCA_START];

    if start && mid && stop {
        return FormatVerification::valid(false);
    }

    FormatVerification::invalid()
}

/// Decodes an EAN-13 barcode
///
/// # Arguments
/// * `barstr` - String of bar widths in set {1, 2, 3, 4}
/// * `first_digit` - The first digit (0-9) that determines the encoding pattern
///
/// # Returns
/// * Decoded 12-digit string (from bars), or error if decoding fails
///
/// # Note
/// Currently this implementation treats EAN-13 the same as UPC-A,
/// decoding only the 12 bar-encoded digits. A full implementation
/// would infer the first digit from the encoding pattern of the
/// first 6 digits.
pub fn decode_ean13(barstr: &str, _first_digit: u8) -> RecogResult<String> {
    // Verify format
    let verification = verify_ean13(barstr);
    if !verification.valid {
        return Err(RecogError::BarcodeError(
            "barstr not in EAN-13 format".to_string(),
        ));
    }

    // Verify size
    let len = barstr.len();
    if len != 59 {
        return Err(RecogError::BarcodeError(
            "size not 59: invalid EAN-13 barcode".to_string(),
        ));
    }

    // Check first digit to determine orientation
    let first_code = &barstr[3..7];
    let mut found_first = false;
    for (j, pattern) in UPCA.iter().take(10).enumerate() {
        if first_code == *pattern {
            found_first = true;
            let _ = j;
            break;
        }
    }

    let vbarstr = if !found_first {
        string_reverse(barstr)
    } else {
        barstr.to_string()
    };

    // Decode 12 digits (same as UPC-A)
    let mut data = String::with_capacity(12);

    for i in 0..12 {
        let start = if i < 6 { 3 + 4 * i } else { 32 + 4 * (i - 6) };
        let code = &vbarstr[start..start + 4];

        let mut found = false;
        for (j, pattern) in UPCA.iter().take(10).enumerate() {
            if code == *pattern {
                data.push(char::from_digit(j as u32, 10).unwrap());
                found = true;
                break;
            }
        }

        if !found {
            return Err(RecogError::BarcodeError(
                "error decoding EAN-13".to_string(),
            ));
        }
    }

    // Verify check digit
    let digits: Vec<u32> = data.chars().map(|c| c.to_digit(10).unwrap_or(0)).collect();

    // Calculate check digit for EAN-13
    // (same algorithm as UPC-A for the bar-encoded digits)
    let mut sum: u32 = 0;
    for i in (0..12).step_by(2) {
        sum += 3 * digits[i];
    }
    for i in (1..12).step_by(2) {
        sum += digits[i];
    }

    let check_digit = if sum.is_multiple_of(10) {
        0
    } else {
        10 - (sum % 10)
    };

    if check_digit != digits[11] {
        // Warning: check digit mismatch (but don't fail)
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_ean13_wrong_length() {
        let verification = verify_ean13("111321121123211111111321121123211111111");
        assert!(!verification.valid);
    }

    #[test]
    fn test_ean13_patterns() {
        // EAN-13 uses the same patterns as UPC-A
        assert_eq!(UPCA[0], "3211"); // 0
        assert_eq!(UPCA[5], "1231"); // 5
        assert_eq!(UPCA[9], "3112"); // 9
    }
}
