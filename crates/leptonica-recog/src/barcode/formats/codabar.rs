//! Codabar barcode decoder
//!
//! Reference: <http://en.wikipedia.org/wiki/Codabar>
//!            <http://morovia.com/education/symbology/codabar.asp>
//!
//! Each symbol has 4 black and 3 white bars. They represent the
//! 10 digits, and optionally 6 other characters. The start and
//! stop codes can be any of four (typically denoted A, B, C, D).

use crate::barcode::types::FormatVerification;
use crate::{RecogError, RecogResult};

/// Codabar symbol patterns
const CODABAR: &[&str] = &[
    "1111122", // 0
    "1111221", // 1
    "1112112", // 2
    "2211111", // 3
    "1121121", // 4
    "2111121", // 5
    "1211112", // 6
    "1211211", // 7
    "1221111", // 8
    "2112111", // 9
    "1112211", // - (10)
    "1122111", // $ (11)
    "2111212", // : (12)
    "2121112", // / (13)
    "2121211", // . (14)
    "1121212", // + (15)
    "1122121", // A (16) - start/stop
    "1212112", // B (17) - start/stop
    "1112122", // C (18) - start/stop
    "1112221", // D (19) - start/stop
];

/// Codabar character values
const CODABARVAL: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '$', ':', '/', '.', '+', 'A', 'B', 'C',
    'D',
];

/// Reverses a string
fn string_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Verifies if a bar string is in Codabar format
pub fn verify_codabar(barstr: &str) -> FormatVerification {
    let len = barstr.len();
    if len < 26 {
        return FormatVerification::invalid();
    }

    // Check for valid start code (any of A, B, C, D - indices 16-19)
    let start_matches: usize = (16..=19)
        .filter(|&i| barstr.starts_with(CODABAR[i]))
        .count();

    // Check for valid stop code
    let stop_matches: usize = (16..=19)
        .filter(|&i| barstr[len - 7..] == *CODABAR[i])
        .count();

    if start_matches > 0 && stop_matches > 0 {
        return FormatVerification::valid(false);
    }

    // Try reversed
    let revbarstr = string_reverse(barstr);
    let start_matches: usize = (16..=19)
        .filter(|&i| revbarstr.starts_with(CODABAR[i]))
        .count();
    let stop_matches: usize = (16..=19)
        .filter(|&i| revbarstr[len - 7..] == *CODABAR[i])
        .count();

    if start_matches > 0 && stop_matches > 0 {
        return FormatVerification::valid(true);
    }

    FormatVerification::invalid()
}

/// Decodes a Codabar barcode
///
/// # Arguments
/// * `barstr` - String of bar widths in set {1, 2}
///
/// # Returns
/// * Decoded string (digits and special characters), or error if decoding fails
pub fn decode_codabar(barstr: &str) -> RecogResult<String> {
    // Verify format and determine orientation
    let verification = verify_codabar(barstr);
    if !verification.valid {
        return Err(RecogError::BarcodeError(
            "barstr not in Codabar format".to_string(),
        ));
    }

    let vbarstr = if verification.reversed {
        string_reverse(barstr)
    } else {
        barstr.to_string()
    };

    // Verify size
    let len = vbarstr.len();
    if (len + 1) % 8 != 0 {
        return Err(RecogError::BarcodeError(
            "size+1 not divisible by 8: invalid Codabar".to_string(),
        ));
    }

    // Number of data symbols (excluding start and stop)
    let nsymb = (len - 15) / 8;
    let mut data = String::with_capacity(nsymb);

    // Decode symbols
    for i in 0..nsymb {
        let start = 8 + 8 * i;
        let code = &vbarstr[start..start + 7];

        let mut found = false;
        for (j, pattern) in CODABAR.iter().take(16).enumerate() {
            if code == *pattern {
                data.push(CODABARVAL[j]);
                found = true;
                break;
            }
        }

        if !found {
            return Err(RecogError::BarcodeError(
                "error decoding Codabar".to_string(),
            ));
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_codabar_too_short() {
        let verification = verify_codabar("11221211112221");
        assert!(!verification.valid);
    }

    #[test]
    fn test_codabar_patterns() {
        // Verify some known patterns
        assert_eq!(CODABAR[0], "1111122"); // 0
        assert_eq!(CODABAR[9], "2112111"); // 9
        assert_eq!(CODABAR[16], "1122121"); // A (start/stop)
        assert_eq!(CODABAR[17], "1212112"); // B (start/stop)
    }

    #[test]
    fn test_codabar_values() {
        assert_eq!(CODABARVAL[0], '0');
        assert_eq!(CODABARVAL[9], '9');
        assert_eq!(CODABARVAL[10], '-');
        assert_eq!(CODABARVAL[16], 'A');
    }
}
