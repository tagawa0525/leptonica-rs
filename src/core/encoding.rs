//! Encoding utilities: Base64 and ASCII85
//!
//! Provides encoding and decoding functions for binary-to-text transformations
//! commonly used in PostScript and PDF output.
//!
//! # Reference
//!
//! Based on Leptonica's `encoding.c`.

use crate::core::{Error, Result};

// ---------------------------------------------------------------------------
// Base64
// ---------------------------------------------------------------------------

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const MAX_BASE64_LINE: usize = 72;

/// Encode binary data as a Base64 string.
///
/// Output lines are wrapped at 72 characters. The result does not include
/// padding-related newlines at the very end if the data aligns perfectly.
///
/// # Reference
///
/// C Leptonica: `encodeBase64()`
pub fn encode_base64(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let out_len = 4 * (data.len().div_ceil(3));
    let line_breaks = out_len / MAX_BASE64_LINE;
    let mut result = String::with_capacity(out_len + line_breaks);
    let mut col = 0;

    let chunks = data.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = chunk[1] as u32;
        let b2 = chunk[2] as u32;

        result.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
        result.push(BASE64_CHARS[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
        result.push(BASE64_CHARS[((b1 & 0x0F) << 2 | (b2 >> 6)) as usize] as char);
        result.push(BASE64_CHARS[(b2 & 0x3F) as usize] as char);
        col += 4;
        if col >= MAX_BASE64_LINE {
            result.push('\n');
            col = 0;
        }
    }

    match remainder.len() {
        1 => {
            let b0 = remainder[0] as u32;
            result.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
            result.push(BASE64_CHARS[((b0 & 0x03) << 4) as usize] as char);
            result.push('=');
            result.push('=');
        }
        2 => {
            let b0 = remainder[0] as u32;
            let b1 = remainder[1] as u32;
            result.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
            result.push(BASE64_CHARS[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
            result.push(BASE64_CHARS[((b1 & 0x0F) << 2) as usize] as char);
            result.push('=');
        }
        _ => {}
    }

    result
}

/// Decode a Base64-encoded string to binary data.
///
/// Whitespace and newlines in the input are ignored.
///
/// # Errors
///
/// Returns an error if the input contains invalid Base64 characters.
///
/// # Reference
///
/// C Leptonica: `decodeBase64()`
pub fn decode_base64(input: &str) -> Result<Vec<u8>> {
    let filtered: Vec<u8> = input
        .bytes()
        .filter(|&b| !b.is_ascii_whitespace())
        .collect();

    if filtered.is_empty() {
        return Ok(Vec::new());
    }

    if !filtered.len().is_multiple_of(4) {
        return Err(Error::InvalidParameter(
            "Base64 input length must be a multiple of 4".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(filtered.len() * 3 / 4);
    for chunk in filtered.chunks_exact(4) {
        let v0 = base64_char_value(chunk[0])?;
        let v1 = base64_char_value(chunk[1])?;
        let v2 = base64_char_value(chunk[2])?;
        let v3 = base64_char_value(chunk[3])?;

        result.push((v0 << 2) | (v1 >> 4));
        if chunk[2] != b'=' {
            result.push((v1 << 4) | (v2 >> 2));
        }
        if chunk[3] != b'=' {
            result.push((v2 << 6) | v3);
        }
    }

    Ok(result)
}

fn base64_char_value(c: u8) -> Result<u8> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        b'=' => Ok(0),
        _ => Err(Error::InvalidParameter(format!(
            "invalid Base64 character: {c:#04x}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// ASCII85
// ---------------------------------------------------------------------------

const POWER85: [u32; 5] = [1, 85, 85 * 85, 85 * 85 * 85, 85 * 85 * 85 * 85];

/// Decode ASCII85-encoded data to binary.
///
/// The input should contain only printable ASCII85 characters ('!' through 'u',
/// plus 'z' for all-zero groups). The `~>` end-of-data marker, if present,
/// is handled automatically. Whitespace is ignored.
///
/// # Errors
///
/// Returns an error if the input contains invalid characters.
///
/// # Reference
///
/// C Leptonica: `decodeAscii85()`
pub fn decode_ascii85(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity(input.len() * 4 / 5);
    let mut group = [0u8; 5];
    let mut count = 0;
    let mut i = 0;

    while i < input.len() {
        let c = input[i];
        i += 1;

        // Skip whitespace
        if c.is_ascii_whitespace() {
            continue;
        }

        // End-of-data marker
        if c == b'~' {
            break;
        }

        if c == b'z' {
            if count != 0 {
                return Err(Error::InvalidParameter(
                    "'z' in middle of ASCII85 group".to_string(),
                ));
            }
            result.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }

        if !(b'!'..=b'u').contains(&c) {
            return Err(Error::InvalidParameter(format!(
                "invalid ASCII85 character: {c:#04x}"
            )));
        }

        group[count] = c - b'!';
        count += 1;

        if count == 5 {
            let value = group[0] as u32 * POWER85[4]
                + group[1] as u32 * POWER85[3]
                + group[2] as u32 * POWER85[2]
                + group[3] as u32 * POWER85[1]
                + group[4] as u32;
            result.extend_from_slice(&value.to_be_bytes());
            count = 0;
        }
    }

    // Handle partial group (2-4 chars → 1-3 bytes)
    if count > 1 {
        // Pad with 'u' (84) to make 5 chars
        for item in group.iter_mut().skip(count) {
            *item = 84;
        }
        let value = group[0] as u32 * POWER85[4]
            + group[1] as u32 * POWER85[3]
            + group[2] as u32 * POWER85[2]
            + group[3] as u32 * POWER85[1]
            + group[4] as u32;
        let bytes = value.to_be_bytes();
        result.extend_from_slice(&bytes[..count - 1]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_empty() {
        assert_eq!(encode_base64(&[]), "");
        assert_eq!(decode_base64("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_base64_padding() {
        // 1 byte → 2 chars + ==
        let enc1 = encode_base64(&[0x41]);
        assert!(enc1.ends_with("=="));
        let dec1 = decode_base64(&enc1).unwrap();
        assert_eq!(dec1, vec![0x41]);

        // 2 bytes → 3 chars + =
        let enc2 = encode_base64(&[0x41, 0x42]);
        assert!(enc2.ends_with('='));
        let dec2 = decode_base64(&enc2).unwrap();
        assert_eq!(dec2, vec![0x41, 0x42]);
    }

    #[test]
    fn test_ascii85_decode_basic() {
        // "Man " (0x4D616E20) encodes to "9jqo^"
        let decoded = decode_ascii85(b"9jqo^~>").unwrap();
        assert_eq!(decoded, b"Man ");
    }

    #[test]
    fn test_ascii85_decode_zeros() {
        let decoded = decode_ascii85(b"z~>").unwrap();
        assert_eq!(decoded, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_ascii85_decode_empty() {
        assert_eq!(decode_ascii85(b"").unwrap(), Vec::<u8>::new());
    }
}
