//! Barcode format decoders
//!
//! This module contains decoders for various 1D barcode formats.

mod codabar;
mod code2of5;
mod code39;
mod code93;
mod codei2of5;
mod ean13;
mod upca;

pub use codabar::decode_codabar;
pub use code2of5::decode_code2of5;
pub use code39::decode_code39;
pub use code93::decode_code93;
pub use codei2of5::decode_codei2of5;
pub use ean13::decode_ean13;
pub use upca::decode_upca;

// Re-export verification functions
pub use codabar::verify_codabar;
pub use code2of5::verify_code2of5;
pub use code39::verify_code39;
pub use code93::verify_code93;
pub use codei2of5::verify_codei2of5;
pub use ean13::verify_ean13;
pub use upca::verify_upca;
