//! ASCII85 encoding for PostScript output
//!
//! ASCII85 (also called Base85) is a binary-to-text encoding that uses 5 ASCII
//! characters to represent 4 bytes of binary data. It is used in PostScript
//! for embedding binary data in a text format.
//!
//! The encoding uses characters from '!' (33) to 'u' (117), plus 'z' for
//! all-zero groups. The encoded data is terminated with '~>'.

/// Encode data using ASCII85 encoding
///
/// Returns a String containing the ASCII85-encoded data with the `~>` terminator.
/// Lines are wrapped at 80 characters for readability.
pub fn encode(data: &[u8]) -> String {
    const LINE_WIDTH: usize = 80;

    let mut result = String::with_capacity((data.len() * 5 / 4) + (data.len() / 20) + 10);

    let mut col = 0;

    // Process complete 4-byte groups
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

        if value == 0 {
            // Special case: all zeros encode to 'z'
            result.push('z');
            col += 1;
        } else {
            // Encode 4 bytes to 5 ASCII85 characters
            let encoded = encode_group(value);
            result.push_str(&encoded);
            col += 5;
        }

        // Line wrapping
        if col >= LINE_WIDTH {
            result.push('\n');
            col = 0;
        }
    }

    // Handle remainder (1-3 bytes)
    if !remainder.is_empty() {
        // Pad with zeros to make 4 bytes
        let mut padded = [0u8; 4];
        padded[..remainder.len()].copy_from_slice(remainder);
        let value = u32::from_be_bytes(padded);

        // Encode and take only the necessary characters
        let encoded = encode_group(value);
        let needed = remainder.len() + 1;
        result.push_str(&encoded[..needed]);
    }

    // Add terminator
    if col > LINE_WIDTH - 2 {
        result.push('\n');
    }
    result.push_str("~>");

    result
}

/// Encode a 4-byte group to 5 ASCII85 characters
fn encode_group(value: u32) -> String {
    let mut chars = [0u8; 5];
    let mut v = value;

    // Convert to base-85, least significant digit first
    for char in chars.iter_mut().rev() {
        *char = (v % 85) as u8 + b'!';
        v /= 85;
    }

    // Safety: all characters are in the ASCII range '!' to 'u'
    String::from_utf8(chars.to_vec()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        let result = encode(&[]);
        assert_eq!(result, "~>");
    }

    #[test]
    fn test_encode_zeros() {
        // 4 zero bytes encode to 'z'
        let result = encode(&[0, 0, 0, 0]);
        assert_eq!(result, "z~>");
    }

    #[test]
    fn test_encode_basic() {
        // "Man " (0x4D616E20) encodes to "9jqo^"
        let result = encode(b"Man ");
        assert_eq!(result, "9jqo^~>");
    }

    #[test]
    fn test_encode_hello() {
        // Test with "Hello" which doesn't align to 4 bytes
        let result = encode(b"Hello");
        // "Hell" -> "87cUR" + "o" (partial) -> "D"
        assert!(result.ends_with("~>"));
        assert!(result.len() > 2);
    }

    #[test]
    fn test_encode_8_zeros() {
        // 8 zero bytes encode to 'zz'
        let result = encode(&[0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(result, "zz~>");
    }

    #[test]
    fn test_encode_partial_groups() {
        // 1 byte
        let result = encode(&[0xFF]);
        assert!(result.ends_with("~>"));
        assert!(!result.contains('z')); // Non-zero, so no 'z' shortcut

        // 2 bytes
        let result = encode(&[0xFF, 0xFF]);
        assert!(result.ends_with("~>"));

        // 3 bytes
        let result = encode(&[0xFF, 0xFF, 0xFF]);
        assert!(result.ends_with("~>"));
    }

    #[test]
    fn test_encode_mixed() {
        // Mix of zeros and non-zeros
        let data = [0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = encode(&data);
        assert!(result.starts_with('z')); // First 4 bytes are zeros
        assert!(result.ends_with("~>"));
    }
}
