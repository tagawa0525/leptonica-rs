//! ASCII85 (Base-85) encoding
//!
//! Used by the PostScript output module to embed binary image data
//! in a text-safe representation.  Each group of 4 input bytes maps
//! to 5 printable ASCII characters; the special sequence `~>` marks
//! end-of-data.
//!
//! # See also
//! C version: `encodeAscii85()` in `psio2.c`

/// Encode a byte slice as an ASCII85 string.
///
/// The returned string does **not** include the `<~` prefix but does
/// include the `~>` end-of-data marker.
///
/// # Arguments
/// * `data` - Raw binary data to encode
pub fn encode(data: &[u8]) -> String {
    todo!()
}
