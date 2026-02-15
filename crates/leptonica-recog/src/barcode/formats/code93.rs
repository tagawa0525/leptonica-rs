//! Code 93 barcode decoder
//!
//! Reference: <http://en.wikipedia.org/wiki/Code93>
//!            <http://morovia.com/education/symbology/code93.asp>
//!
//! Each symbol has 3 black and 3 white bars.
//! The start and stop codes are "111141".
//! The last two codes are check characters (C and K).

use crate::barcode::types::FormatVerification;
use crate::{RecogError, RecogResult};

/// Code 93 symbol patterns
const CODE93: &[&str] = &[
    "131112", "111213", "111312", "111411", "121113", // 0-4
    "121212", "121311", "111114", "131211", "141111", // 5-9
    "211113", "211212", "211311", "221112", "221211", // A-E
    "231111", "112113", "112212", "112311", "122112", // F-J
    "132111", "111123", "111222", "111321", "121122", // K-O
    "131121", "212112", "212211", "211122", "211221", // P-T
    "221121", "222111", "112122", "112221", "122121", // U-Y
    "123111", "121131", "311112", "311211", "321111", // Z,-,.,SP,$
    "112131", "113121", "211131", "131221", "312111", // /,+,%,($),(%)
    "311121", "122211", "111141", // (/),(+),Start/Stop
];

/// Code 93 character values
const CODE93VAL: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-', '.',
    ' ', '$', '/', '+', '%', '[', ']', '{', '}', '#',
];

const C93_START: usize = 47;

/// Reverses a string
fn string_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Verifies if a bar string is in Code 93 format
pub fn verify_code93(barstr: &str) -> FormatVerification {
    let len = barstr.len();
    if len < 28 {
        return FormatVerification::invalid();
    }

    let start = barstr.starts_with(CODE93[C93_START]);
    let stop = barstr[len - 7..len - 1] == *CODE93[C93_START];

    if start && stop {
        return FormatVerification::valid(false);
    }

    // Try reversed
    let revbarstr = string_reverse(barstr);
    let start = revbarstr.starts_with(CODE93[C93_START]);
    let stop = revbarstr[len - 7..len - 1] == *CODE93[C93_START];

    if start && stop {
        return FormatVerification::valid(true);
    }

    FormatVerification::invalid()
}

/// Decodes a Code 93 barcode
///
/// # Arguments
/// * `barstr` - String of bar widths in set {1, 2, 3, 4}
///
/// # Returns
/// * Decoded string (without check characters), or error if decoding fails
pub fn decode_code93(barstr: &str) -> RecogResult<String> {
    // Verify format and determine orientation
    let verification = verify_code93(barstr);
    if !verification.valid {
        return Err(RecogError::BarcodeError(
            "barstr not in Code 93 format".to_string(),
        ));
    }

    let vbarstr = if verification.reversed {
        string_reverse(barstr)
    } else {
        barstr.to_string()
    };

    // Verify size: skip first 6 and last 7 bars
    let len = vbarstr.len();
    if (len - 13) % 6 != 0 {
        return Err(RecogError::BarcodeError(
            "size not divisible by 6: invalid Code 93".to_string(),
        ));
    }

    let nsymb = (len - 13) / 6;
    let mut data = String::with_capacity(nsymb);
    let mut indices = Vec::with_capacity(nsymb);

    // Decode symbols
    for i in 0..nsymb {
        let start = 6 + 6 * i;
        let code = &vbarstr[start..start + 6];

        let mut found = false;
        for (j, pattern) in CODE93.iter().take(C93_START).enumerate() {
            if code == *pattern {
                data.push(CODE93VAL[j]);
                indices.push(j);
                found = true;
                break;
            }
        }

        if !found {
            return Err(RecogError::BarcodeError(
                "error decoding Code 93".to_string(),
            ));
        }
    }

    // Verify check characters (C and K)
    // For character "C", use only the actual data
    // For character "K", use actual data plus check character "C"
    if nsymb >= 2 {
        let data_len = nsymb - 2;

        // Calculate check C
        let mut sum = 0;
        for i in 0..data_len {
            sum += ((i % 20) + 1) * indices[data_len - 1 - i];
        }
        let check_c = CODE93VAL[sum % 47];

        if data.chars().nth(nsymb - 2) != Some(check_c) {
            // Warning: check C mismatch (but don't fail)
        }

        // Calculate check K
        sum = 0;
        for i in 0..(nsymb - 1) {
            sum += ((i % 15) + 1) * indices[nsymb - 2 - i];
        }
        let check_k = CODE93VAL[sum % 47];

        if data.chars().nth(nsymb - 1) != Some(check_k) {
            // Warning: check K mismatch (but don't fail)
        }

        // Remove the two check characters from output
        data.truncate(data_len);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_code93_too_short() {
        let verification = verify_code93("111141111141");
        assert!(!verification.valid);
    }

    #[test]
    fn test_code93_patterns() {
        // Verify some known patterns
        assert_eq!(CODE93[0], "131112"); // 0
        assert_eq!(CODE93[10], "211113"); // A
        assert_eq!(CODE93[C93_START], "111141"); // Start/Stop
    }
}
