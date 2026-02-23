//! Code 39 barcode decoder
//!
//! Reference: <http://en.wikipedia.org/wiki/Code39>
//!            <http://morovia.com/education/symbology/code39.asp>
//!
//! Each symbol has 5 black and 4 white bars.
//! The start and stop codes are "121121211" (the asterisk).

use crate::barcode::types::FormatVerification;
use crate::{RecogError, RecogResult};

/// Code 39 symbol patterns
const CODE39: &[&str] = &[
    "111221211",
    "211211112",
    "112211112",
    "212211111", // 0-3
    "111221112",
    "211221111",
    "112221111",
    "111211212", // 4-7
    "211211211",
    "112211211",
    "211112112",
    "112112112", // 8-B
    "212112111",
    "111122112",
    "211122111",
    "112122111", // C-F
    "111112212",
    "211112211",
    "112112211",
    "111122211", // G-J
    "211111122",
    "112111122",
    "212111121",
    "111121122", // K-N
    "211121121",
    "112121121",
    "111111222",
    "211111221", // O-R
    "112111221",
    "111121221",
    "221111112",
    "122111112", // S-V
    "222111111",
    "121121112",
    "221121111",
    "122121111", // W-Z
    "121111212",
    "221111211",
    "122111211",
    "121212111", // -,.,SP,$
    "121211121",
    "121112121",
    "111212121",
    "121121211", // /,+,%,*
];

/// Code 39 character values
const CODE39VAL: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-', '.',
    ' ', '$', '/', '+', '%', '*',
];

const C39_START: usize = 43; // '*' is both start and stop

/// Reverses a string
fn string_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Verifies if a bar string is in Code 39 format
pub fn verify_code39(barstr: &str) -> FormatVerification {
    let len = barstr.len();
    if len < 30 {
        return FormatVerification::invalid();
    }

    let start = barstr.starts_with(CODE39[C39_START]);
    let stop = barstr[len - 9..] == *CODE39[C39_START];

    if start && stop {
        return FormatVerification::valid(false);
    }

    // Try reversed
    let revbarstr = string_reverse(barstr);
    let start = revbarstr.starts_with(CODE39[C39_START]);
    let stop = revbarstr[len - 9..] == *CODE39[C39_START];

    if start && stop {
        return FormatVerification::valid(true);
    }

    FormatVerification::invalid()
}

/// Decodes a Code 39 barcode
///
/// # Arguments
/// * `barstr` - String of bar widths in set {1, 2}
///
/// # Returns
/// * Decoded string, or error if decoding fails
pub fn decode_code39(barstr: &str) -> RecogResult<String> {
    // Verify format and determine orientation
    let verification = verify_code39(barstr);
    if !verification.valid {
        return Err(RecogError::BarcodeError(
            "barstr not in Code 39 format".to_string(),
        ));
    }

    let vbarstr = if verification.reversed {
        string_reverse(barstr)
    } else {
        barstr.to_string()
    };

    // Verify size
    let len = vbarstr.len();
    if (len + 1) % 10 != 0 {
        return Err(RecogError::BarcodeError(
            "size+1 not divisible by 10: invalid Code 39".to_string(),
        ));
    }

    // Number of data symbols (excluding start and stop)
    let nsymb = (len - 19) / 10;
    let mut data = String::with_capacity(nsymb);

    // Decode symbols
    for i in 0..nsymb {
        let start = 10 + 10 * i;
        let code = &vbarstr[start..start + 9];

        let mut found = false;
        for (j, pattern) in CODE39.iter().take(C39_START).enumerate() {
            if code == *pattern {
                data.push(CODE39VAL[j]);
                found = true;
                break;
            }
        }

        if !found {
            return Err(RecogError::BarcodeError(
                "error decoding Code 39".to_string(),
            ));
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_code39_too_short() {
        let verification = verify_code39("121121211121121211");
        assert!(!verification.valid);
    }

    #[test]
    fn test_code39_patterns() {
        // Verify some known patterns
        assert_eq!(CODE39[0], "111221211"); // 0
        assert_eq!(CODE39[10], "211112112"); // A
        assert_eq!(CODE39[C39_START], "121121211"); // * (start/stop)
    }

    #[test]
    fn test_code39_values() {
        assert_eq!(CODE39VAL[0], '0');
        assert_eq!(CODE39VAL[10], 'A');
        assert_eq!(CODE39VAL[36], '-');
        assert_eq!(CODE39VAL[C39_START], '*');
    }
}
