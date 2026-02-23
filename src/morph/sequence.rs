//! Morphological sequence operations
//!
//! This module provides functionality to execute sequences of morphological
//! operations specified as strings. This allows for flexible composition of
//! morphological transformations.
//!
//! # Sequence String Format
//!
//! Operations are separated by `+` and whitespace is ignored.
//! Each operation begins with a case-insensitive character:
//!
//! ## Binary Operations
//! - `d<w>.<h>` - Dilation with w x h brick structuring element
//! - `e<w>.<h>` - Erosion with w x h brick structuring element
//! - `o<w>.<h>` - Opening with w x h brick structuring element
//! - `c<w>.<h>` - Closing with w x h brick structuring element
//!
//! ## Grayscale Operations
//! Same as binary, plus:
//! - `tw<w>.<h>` - White tophat (original - opening)
//! - `tb<w>.<h>` - Black tophat (closing - original)
//!
//! Note: For grayscale operations, width and height must be odd numbers.
//!
//! # Examples
//!
//! ```
//! use leptonica_morph::sequence::{MorphSequence, morph_sequence, gray_morph_sequence};
//! use leptonica_core::{Pix, PixelDepth};
//!
//! // Parse and validate a sequence
//! let seq = MorphSequence::parse("o5.5 + e3.3").unwrap();
//! assert_eq!(seq.ops().len(), 2);
//!
//! // Execute on a binary image
//! let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
//! let result = morph_sequence(&pix, "d3.3 + e3.3").unwrap();
//!
//! // Execute on a grayscale image
//! let gray = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
//! let result = gray_morph_sequence(&gray, "o5.5 + c3.3").unwrap();
//! ```
//!
//! # Reference
//!
//! Based on Leptonica's `morphseq.c` implementation.

use crate::{MorphError, MorphResult};
use leptonica_core::{Pix, PixelDepth};
use leptonica_transform::{expand_replicate, reduce_rank_binary_cascade};

/// A parsed morphological operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MorphOp {
    /// Dilation with a brick structuring element
    Dilate {
        /// Width of the brick SE
        width: u32,
        /// Height of the brick SE
        height: u32,
    },
    /// Erosion with a brick structuring element
    Erode {
        /// Width of the brick SE
        width: u32,
        /// Height of the brick SE
        height: u32,
    },
    /// Opening (erosion followed by dilation)
    Open {
        /// Width of the brick SE
        width: u32,
        /// Height of the brick SE
        height: u32,
    },
    /// Closing (dilation followed by erosion)
    Close {
        /// Width of the brick SE
        width: u32,
        /// Height of the brick SE
        height: u32,
    },
    /// Tophat transform (grayscale only)
    Tophat {
        /// true for white tophat (original - opening), false for black (closing - original)
        white: bool,
        /// Width of the brick SE
        width: u32,
        /// Height of the brick SE
        height: u32,
    },
    /// Cascaded 2x rank reduction (binary only)
    ///
    /// Each element of `levels` is a rank threshold (1–4) for one halving step.
    /// Corresponds to the `r<digits>` operator in morphological sequences.
    RankReduce {
        /// Rank thresholds, one per halving step
        levels: Vec<u8>,
    },
    /// Binary image expansion by replication.
    ///
    /// Corresponds to the `x<factor>` operator in morphological sequences.
    /// The sequence parser only accepts power-of-2 factors (1, 2, 4, 8, …).
    /// Note: the underlying [`expand_replicate`] function does not enforce the
    /// power-of-2 constraint, so `BinaryExpand` structs constructed directly
    /// may carry non-power-of-2 values.
    BinaryExpand {
        /// Expansion factor; the sequence parser enforces that this is a power of 2
        factor: u32,
    },
}

impl MorphOp {
    /// Get the width and height of the operation's structuring element.
    ///
    /// For `RankReduce` and `BinaryExpand`, which do not use a brick SE,
    /// returns `(0, 0)`.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            MorphOp::Dilate { width, height }
            | MorphOp::Erode { width, height }
            | MorphOp::Open { width, height }
            | MorphOp::Close { width, height }
            | MorphOp::Tophat { width, height, .. } => (*width, *height),
            MorphOp::RankReduce { .. } | MorphOp::BinaryExpand { .. } => (0, 0),
        }
    }

    /// Check if both dimensions are odd (required for grayscale operations)
    pub fn has_odd_dimensions(&self) -> bool {
        let (w, h) = self.dimensions();
        w % 2 == 1 && h % 2 == 1
    }
}

/// A parsed morphological sequence
#[derive(Debug, Clone)]
pub struct MorphSequence {
    /// The operations in the sequence
    ops: Vec<MorphOp>,
}

impl MorphSequence {
    /// Parse a sequence string into a MorphSequence
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence string (e.g., "o5.5 + e3.3")
    ///
    /// # Returns
    ///
    /// A parsed MorphSequence, or an error if the sequence is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_morph::sequence::MorphSequence;
    ///
    /// let seq = MorphSequence::parse("d3.3 + e5.5").unwrap();
    /// assert_eq!(seq.ops().len(), 2);
    /// ```
    pub fn parse(sequence: &str) -> MorphResult<Self> {
        if sequence.trim().is_empty() {
            return Err(MorphError::InvalidSequence("empty sequence".to_string()));
        }

        let parts: Vec<&str> = sequence.split('+').collect();
        let mut ops = Vec::with_capacity(parts.len());

        for (i, part) in parts.iter().enumerate() {
            let op_str = part.trim();
            if op_str.is_empty() {
                return Err(MorphError::InvalidSequence(format!(
                    "empty operation at position {}",
                    i + 1
                )));
            }

            let op = Self::parse_operation(op_str)?;
            ops.push(op);
        }

        Ok(MorphSequence { ops })
    }

    /// Parse a single operation string
    fn parse_operation(op_str: &str) -> MorphResult<MorphOp> {
        // Remove whitespace
        let op_str: String = op_str.chars().filter(|c| !c.is_whitespace()).collect();

        if op_str.is_empty() {
            return Err(MorphError::InvalidSequence("empty operation".to_string()));
        }

        let first_char = op_str.chars().next().unwrap().to_ascii_lowercase();

        match first_char {
            'd' | 'e' | 'o' | 'c' => {
                let (width, height) = Self::parse_dimensions(&op_str[1..])?;
                let op = match first_char {
                    'd' => MorphOp::Dilate { width, height },
                    'e' => MorphOp::Erode { width, height },
                    'o' => MorphOp::Open { width, height },
                    'c' => MorphOp::Close { width, height },
                    _ => unreachable!(),
                };
                Ok(op)
            }
            't' => {
                // Tophat: tw<w>.<h> or tb<w>.<h>
                if op_str.len() < 2 {
                    return Err(MorphError::InvalidSequence(format!(
                        "invalid tophat operation: {}",
                        op_str
                    )));
                }

                let tophat_type = op_str.chars().nth(1).unwrap().to_ascii_lowercase();
                let white = match tophat_type {
                    'w' => true,
                    'b' => false,
                    _ => {
                        return Err(MorphError::InvalidSequence(format!(
                            "invalid tophat type '{}' in '{}', expected 'w' or 'b'",
                            tophat_type, op_str
                        )));
                    }
                };

                let (width, height) = Self::parse_dimensions(&op_str[2..])?;
                Ok(MorphOp::Tophat {
                    white,
                    width,
                    height,
                })
            }
            'r' => {
                // Rank reduction: r<d1><d2>...<dN>
                // 各桁 d_i は rank level (1–4)
                let digits_str = &op_str[1..];
                if digits_str.is_empty() {
                    return Err(MorphError::InvalidSequence(
                        "rank reduce 'r' requires at least one level digit".to_string(),
                    ));
                }
                let mut levels = Vec::with_capacity(digits_str.len());
                for ch in digits_str.chars() {
                    match ch {
                        '1'..='4' => levels.push(ch as u8 - b'0'),
                        '0' | '5'..='9' => {
                            return Err(MorphError::InvalidSequence(format!(
                                "rank level must be 1–4, got '{ch}' in '{op_str}'"
                            )));
                        }
                        _ => {
                            return Err(MorphError::InvalidSequence(format!(
                                "non-digit character '{ch}' in rank reduce '{op_str}'"
                            )));
                        }
                    }
                }
                Ok(MorphOp::RankReduce { levels })
            }
            'x' => {
                // Binary expansion: x<factor>
                let factor_str = &op_str[1..];
                if factor_str.is_empty() {
                    return Err(MorphError::InvalidSequence(
                        "binary expand 'x' requires a factor".to_string(),
                    ));
                }
                let factor: u32 = factor_str.parse().map_err(|_| {
                    MorphError::InvalidSequence(format!(
                        "invalid expansion factor '{}' in '{}'",
                        factor_str, op_str
                    ))
                })?;
                if factor == 0 || !factor.is_power_of_two() {
                    return Err(MorphError::InvalidSequence(format!(
                        "expansion factor must be a power of 2 > 0, got {factor}"
                    )));
                }
                Ok(MorphOp::BinaryExpand { factor })
            }
            'b' => {
                // Border operations are not supported
                Err(MorphError::UnsupportedOperation(
                    "operation 'b' (border) is not supported".to_string(),
                ))
            }
            _ => Err(MorphError::InvalidSequence(format!(
                "unknown operation '{}' in '{}'",
                first_char, op_str
            ))),
        }
    }

    /// Parse dimensions from a string like "3.5" -> (3, 5)
    fn parse_dimensions(dim_str: &str) -> MorphResult<(u32, u32)> {
        let parts: Vec<&str> = dim_str.split('.').collect();

        if parts.len() != 2 {
            return Err(MorphError::InvalidSequence(format!(
                "invalid dimensions format '{}', expected 'width.height'",
                dim_str
            )));
        }

        let width: u32 = parts[0].parse().map_err(|_| {
            MorphError::InvalidSequence(format!("invalid width '{}' in '{}'", parts[0], dim_str))
        })?;

        let height: u32 = parts[1].parse().map_err(|_| {
            MorphError::InvalidSequence(format!("invalid height '{}' in '{}'", parts[1], dim_str))
        })?;

        if width == 0 || height == 0 {
            return Err(MorphError::InvalidSequence(format!(
                "dimensions must be > 0, got {}x{}",
                width, height
            )));
        }

        Ok((width, height))
    }

    /// Get the operations in this sequence
    pub fn ops(&self) -> &[MorphOp] {
        &self.ops
    }

    /// Check if this sequence is valid for binary operations
    ///
    /// Returns an error if the sequence contains operations that are
    /// only valid for grayscale images (e.g., tophat).
    pub fn verify_binary(&self) -> MorphResult<()> {
        for (i, op) in self.ops.iter().enumerate() {
            if let MorphOp::Tophat { .. } = op {
                return Err(MorphError::InvalidSequence(format!(
                    "operation {} (tophat) is only valid for grayscale images",
                    i + 1
                )));
            }
        }
        Ok(())
    }

    /// Check if this sequence is valid for grayscale operations
    ///
    /// The underlying grayscale morphology functions accept even-sized
    /// structuring elements and internally coerce them to the next odd
    /// size (via `ensure_odd`). This verification step is kept for API
    /// symmetry with `verify_binary` and may be extended with additional
    /// grayscale-specific checks in the future.
    pub fn verify_grayscale(&self) -> MorphResult<()> {
        // Currently there are no additional structural constraints specific
        // to grayscale sequences beyond those enforced by the underlying ops.
        // Even dimensions are silently coerced to odd by the direct APIs.
        Ok(())
    }

    /// Get the number of operations in the sequence
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Check if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// Execute a binary morphological sequence on an image
///
/// # Arguments
///
/// * `pix` - A 1-bpp binary image
/// * `sequence` - A sequence string (e.g., "o5.5 + e3.3")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
///
/// # Examples
///
/// ```
/// use leptonica_morph::sequence::morph_sequence;
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit1).unwrap();
/// let result = morph_sequence(&pix, "d3.3 + e3.3").unwrap();
/// ```
pub fn morph_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    // Validate input
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }

    // Parse and verify sequence
    let seq = MorphSequence::parse(sequence)?;
    seq.verify_binary()?;

    // Execute operations
    let mut result = pix.clone();

    for op in seq.ops() {
        result = execute_binary_op(&result, op)?;
    }

    Ok(result)
}

/// Execute a binary composite morphological sequence
///
/// This is similar to `morph_sequence` but uses composite operations
/// for better performance with large structuring elements.
///
/// Note: Currently this delegates to `morph_sequence` as we don't have
/// composite operations implemented yet.
///
/// # Arguments
///
/// * `pix` - A 1-bpp binary image
/// * `sequence` - A sequence string (e.g., "o5.5 + e3.3")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
pub fn morph_comp_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    // For now, delegate to morph_sequence
    // In the future, this could use composite operations for large SEs
    morph_sequence(pix, sequence)
}

/// Execute a binary morphological sequence using DWA (word-aligned) operations
///
/// Same as [`morph_sequence`] but dispatches to DWA functions for performance.
///
/// # Arguments
///
/// * `pix` - A 1-bpp binary image
/// * `sequence` - A sequence string (e.g., "d3.3 + e3.3")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
pub fn morph_sequence_dwa(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    let seq = MorphSequence::parse(sequence)?;
    seq.verify_binary()?;
    let mut result = pix.clone();
    for op in seq.ops() {
        result = execute_dwa_op(&result, op)?;
    }
    Ok(result)
}

/// Execute a binary composite morphological sequence using DWA operations
///
/// Similar to [`morph_sequence_dwa`] but uses composite DWA operations
/// that support sizes up to 63 pixels per dimension.
///
/// # Arguments
///
/// * `pix` - A 1-bpp binary image
/// * `sequence` - A sequence string (e.g., "d3.3 + e3.3")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
pub fn morph_comp_sequence_dwa(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(MorphError::UnsupportedDepth {
            expected: "1-bpp binary",
            actual: pix.depth().bits(),
        });
    }
    let seq = MorphSequence::parse(sequence)?;
    seq.verify_binary()?;
    let mut result = pix.clone();
    for op in seq.ops() {
        result = execute_comp_dwa_op(&result, op)?;
    }
    Ok(result)
}

/// Execute a color (32 bpp) morphological sequence on an image
///
/// Processes each RGB channel independently using brick structuring elements.
/// Even-valued dimensions are silently coerced to the next odd value, matching
/// the behavior of `dilate_color`, `erode_color`, and other direct color
/// morphology functions.
///
/// # Arguments
///
/// * `pix` - A 32-bpp RGB image
/// * `sequence` - A sequence string with d/e/o/c operations (e.g., "c5.3 + o7.5")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
///
/// # Errors
///
/// Returns an error if `pix` is not 32-bpp, the sequence is invalid, or a
/// tophat operation is specified (which is not valid for color morphology).
pub fn color_morph_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    if pix.depth() != PixelDepth::Bit32 {
        return Err(MorphError::UnsupportedDepth {
            expected: "32-bpp RGB",
            actual: pix.depth().bits(),
        });
    }
    let seq = MorphSequence::parse(sequence)?;
    for (i, op) in seq.ops().iter().enumerate() {
        if let MorphOp::Tophat { .. } = op {
            return Err(MorphError::InvalidSequence(format!(
                "operation {} (tophat) is not valid for color morphology",
                i + 1
            )));
        }
    }
    let mut result = pix.clone();
    for op in seq.ops() {
        result = execute_color_op(&result, op)?;
    }
    Ok(result)
}

/// Execute a grayscale morphological sequence on an image
///
/// # Arguments
///
/// * `pix` - An 8-bpp grayscale image
/// * `sequence` - A sequence string (e.g., "o5.5 + c3.3")
///
/// # Returns
///
/// A new image with all operations applied, or an error.
///
/// # Notes
///
/// - All structuring element dimensions must be odd numbers
/// - Supports tophat operations (`tw` for white, `tb` for black)
///
/// # Examples
///
/// ```
/// use leptonica_morph::sequence::gray_morph_sequence;
/// use leptonica_core::{Pix, PixelDepth};
///
/// let pix = Pix::new(100, 100, PixelDepth::Bit8).unwrap();
/// let result = gray_morph_sequence(&pix, "o5.5 + c3.3").unwrap();
/// ```
pub fn gray_morph_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix> {
    // Validate input
    if pix.depth() != PixelDepth::Bit8 {
        return Err(MorphError::UnsupportedDepth {
            expected: "8-bpp grayscale",
            actual: pix.depth().bits(),
        });
    }

    // Parse and verify sequence
    let seq = MorphSequence::parse(sequence)?;
    seq.verify_grayscale()?;

    // Execute operations
    let mut result = pix.clone();

    for op in seq.ops() {
        result = execute_gray_op(&result, op)?;
    }

    Ok(result)
}

/// Execute a single binary morphological operation
fn execute_binary_op(pix: &Pix, op: &MorphOp) -> MorphResult<Pix> {
    match op {
        MorphOp::Dilate { width, height } => crate::dilate_brick(pix, *width, *height),
        MorphOp::Erode { width, height } => crate::erode_brick(pix, *width, *height),
        MorphOp::Open { width, height } => crate::open_brick(pix, *width, *height),
        MorphOp::Close { width, height } => crate::close_brick(pix, *width, *height),
        MorphOp::Tophat { .. } => Err(MorphError::InvalidSequence(
            "tophat is only valid for grayscale operations".to_string(),
        )),
        MorphOp::RankReduce { levels } => reduce_rank_binary_cascade(pix, levels)
            .map_err(|e| MorphError::InvalidSequence(e.to_string())),
        MorphOp::BinaryExpand { factor } => {
            expand_replicate(pix, *factor).map_err(|e| MorphError::InvalidSequence(e.to_string()))
        }
    }
}

/// Execute a single binary DWA morphological operation
fn execute_dwa_op(pix: &Pix, op: &MorphOp) -> MorphResult<Pix> {
    match op {
        MorphOp::Dilate { width, height } => crate::dwa::dilate_brick_dwa(pix, *width, *height),
        MorphOp::Erode { width, height } => crate::dwa::erode_brick_dwa(pix, *width, *height),
        MorphOp::Open { width, height } => crate::dwa::open_brick_dwa(pix, *width, *height),
        MorphOp::Close { width, height } => crate::dwa::close_brick_dwa(pix, *width, *height),
        MorphOp::Tophat { .. } => Err(MorphError::InvalidSequence(
            "tophat is only valid for grayscale operations".to_string(),
        )),
        MorphOp::RankReduce { levels } => reduce_rank_binary_cascade(pix, levels)
            .map_err(|e| MorphError::InvalidSequence(e.to_string())),
        MorphOp::BinaryExpand { factor } => {
            expand_replicate(pix, *factor).map_err(|e| MorphError::InvalidSequence(e.to_string()))
        }
    }
}

/// Execute a single binary composite DWA morphological operation
fn execute_comp_dwa_op(pix: &Pix, op: &MorphOp) -> MorphResult<Pix> {
    match op {
        MorphOp::Dilate { width, height } => {
            crate::dwa::dilate_comp_brick_dwa(pix, *width, *height)
        }
        MorphOp::Erode { width, height } => crate::dwa::erode_comp_brick_dwa(pix, *width, *height),
        MorphOp::Open { width, height } => crate::dwa::open_comp_brick_dwa(pix, *width, *height),
        MorphOp::Close { width, height } => crate::dwa::close_comp_brick_dwa(pix, *width, *height),
        MorphOp::Tophat { .. } => Err(MorphError::InvalidSequence(
            "tophat is only valid for grayscale operations".to_string(),
        )),
        MorphOp::RankReduce { levels } => reduce_rank_binary_cascade(pix, levels)
            .map_err(|e| MorphError::InvalidSequence(e.to_string())),
        MorphOp::BinaryExpand { factor } => {
            expand_replicate(pix, *factor).map_err(|e| MorphError::InvalidSequence(e.to_string()))
        }
    }
}

/// Execute a single color morphological operation
fn execute_color_op(pix: &Pix, op: &MorphOp) -> MorphResult<Pix> {
    match op {
        MorphOp::Dilate { width, height } => crate::color::dilate_color(pix, *width, *height),
        MorphOp::Erode { width, height } => crate::color::erode_color(pix, *width, *height),
        MorphOp::Open { width, height } => crate::color::open_color(pix, *width, *height),
        MorphOp::Close { width, height } => crate::color::close_color(pix, *width, *height),
        MorphOp::Tophat { .. } => Err(MorphError::InvalidSequence(
            "tophat is not valid for color morphology".to_string(),
        )),
        MorphOp::RankReduce { .. } | MorphOp::BinaryExpand { .. } => {
            Err(MorphError::UnsupportedOperation(
                "RankReduce/BinaryExpand are binary-only operations".to_string(),
            ))
        }
    }
}

/// Execute a single grayscale morphological operation
fn execute_gray_op(pix: &Pix, op: &MorphOp) -> MorphResult<Pix> {
    match op {
        MorphOp::Dilate { width, height } => crate::dilate_gray(pix, *width, *height),
        MorphOp::Erode { width, height } => crate::erode_gray(pix, *width, *height),
        MorphOp::Open { width, height } => crate::open_gray(pix, *width, *height),
        MorphOp::Close { width, height } => crate::close_gray(pix, *width, *height),
        MorphOp::Tophat {
            white,
            width,
            height,
        } => {
            if *white {
                crate::top_hat_gray(pix, *width, *height)
            } else {
                crate::bottom_hat_gray(pix, *width, *height)
            }
        }
        MorphOp::RankReduce { .. } | MorphOp::BinaryExpand { .. } => {
            Err(MorphError::UnsupportedOperation(
                "RankReduce/BinaryExpand are binary-only operations".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_operation() {
        let seq = MorphSequence::parse("d3.5").unwrap();
        assert_eq!(seq.ops().len(), 1);
        assert_eq!(
            seq.ops()[0],
            MorphOp::Dilate {
                width: 3,
                height: 5
            }
        );
    }

    #[test]
    fn test_parse_multiple_operations() {
        let seq = MorphSequence::parse("d3.3 + e5.5 + o7.7").unwrap();
        assert_eq!(seq.ops().len(), 3);
        assert_eq!(
            seq.ops()[0],
            MorphOp::Dilate {
                width: 3,
                height: 3
            }
        );
        assert_eq!(
            seq.ops()[1],
            MorphOp::Erode {
                width: 5,
                height: 5
            }
        );
        assert_eq!(
            seq.ops()[2],
            MorphOp::Open {
                width: 7,
                height: 7
            }
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        let seq1 = MorphSequence::parse("D3.3").unwrap();
        let seq2 = MorphSequence::parse("d3.3").unwrap();
        assert_eq!(seq1.ops()[0], seq2.ops()[0]);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let seq = MorphSequence::parse("  d3.3  +  e5.5  ").unwrap();
        assert_eq!(seq.ops().len(), 2);
    }

    #[test]
    fn test_parse_all_operations() {
        let seq = MorphSequence::parse("d3.3 + e5.5 + o7.7 + c9.9").unwrap();
        assert_eq!(seq.ops().len(), 4);
    }

    #[test]
    fn test_parse_tophat_white() {
        let seq = MorphSequence::parse("tw5.5").unwrap();
        assert_eq!(
            seq.ops()[0],
            MorphOp::Tophat {
                white: true,
                width: 5,
                height: 5
            }
        );
    }

    #[test]
    fn test_parse_tophat_black() {
        let seq = MorphSequence::parse("tb3.3").unwrap();
        assert_eq!(
            seq.ops()[0],
            MorphOp::Tophat {
                white: false,
                width: 3,
                height: 3
            }
        );
    }

    #[test]
    fn test_parse_tophat_case_insensitive() {
        let seq1 = MorphSequence::parse("TW5.5").unwrap();
        let seq2 = MorphSequence::parse("tw5.5").unwrap();
        assert_eq!(seq1.ops()[0], seq2.ops()[0]);
    }

    #[test]
    fn test_parse_empty_error() {
        let result = MorphSequence::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_operation_error() {
        let result = MorphSequence::parse("z3.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_dimensions_error() {
        let result = MorphSequence::parse("d3.abc");
        assert!(result.is_err());

        let result = MorphSequence::parse("d33");
        assert!(result.is_err());

        let result = MorphSequence::parse("d0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unsupported_operations() {
        // Border は未サポート
        let result = MorphSequence::parse("b32");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_r_and_x_now_supported() {
        // r と x は実装済み
        assert!(MorphSequence::parse("r23").is_ok());
        assert!(MorphSequence::parse("x4").is_ok());
    }

    #[test]
    fn test_verify_binary_with_tophat_error() {
        let seq = MorphSequence::parse("tw5.5").unwrap();
        assert!(seq.verify_binary().is_err());
    }

    #[test]
    fn test_verify_binary_success() {
        let seq = MorphSequence::parse("d3.3 + e5.5").unwrap();
        assert!(seq.verify_binary().is_ok());
    }

    #[test]
    fn test_verify_grayscale_even_dimensions_accepted() {
        // Even dimensions are accepted because the underlying grayscale ops
        // silently coerce them to the next odd size.
        let seq = MorphSequence::parse("d4.4").unwrap();
        assert!(seq.verify_grayscale().is_ok());
    }

    #[test]
    fn test_verify_grayscale_success() {
        let seq = MorphSequence::parse("d3.3 + e5.5 + tw7.7").unwrap();
        assert!(seq.verify_grayscale().is_ok());
    }

    // -----------------------------------------------------------------------
    // Phase 5: DWA and color sequence tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_morph_sequence_dwa_basic() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = morph_sequence_dwa(&pix, "d3.3 + e3.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_morph_sequence_dwa_non_binary_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(morph_sequence_dwa(&pix, "d3.3").is_err());
    }

    #[test]
    fn test_morph_comp_sequence_dwa_basic() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = morph_comp_sequence_dwa(&pix, "d3.3 + e3.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_morph_comp_sequence_dwa_non_binary_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(morph_comp_sequence_dwa(&pix, "d3.3").is_err());
    }

    #[test]
    fn test_color_morph_sequence_basic() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        let result = color_morph_sequence(&pix, "d3.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_color_morph_sequence_non_rgb_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        assert!(color_morph_sequence(&pix, "d3.3").is_err());
    }

    #[test]
    fn test_color_morph_sequence_even_dim_coerced() {
        let pix = Pix::new(20, 20, PixelDepth::Bit32).unwrap();
        // Even dimensions are silently coerced to odd, matching dilate_color behavior
        assert!(color_morph_sequence(&pix, "d4.4").is_ok());
    }

    #[test]
    fn test_morph_sequence_non_binary_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = morph_sequence(&pix, "d3.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_morph_sequence_execution() {
        let pix = Pix::new(20, 20, PixelDepth::Bit1).unwrap();
        let result = morph_sequence(&pix, "d3.3 + e3.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_gray_morph_sequence_non_grayscale_error() {
        let pix = Pix::new(10, 10, PixelDepth::Bit1).unwrap();
        let result = gray_morph_sequence(&pix, "d3.3");
        assert!(result.is_err());
    }

    #[test]
    fn test_gray_morph_sequence_even_dimensions_accepted() {
        // Even dimensions are accepted and coerced to odd by the underlying ops.
        let pix = Pix::new(10, 10, PixelDepth::Bit8).unwrap();
        let result = gray_morph_sequence(&pix, "d4.4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_gray_morph_sequence_execution() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = gray_morph_sequence(&pix, "d3.3 + e3.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().width(), 20);
    }

    #[test]
    fn test_gray_morph_sequence_with_tophat() {
        let pix = Pix::new(20, 20, PixelDepth::Bit8).unwrap();
        let result = gray_morph_sequence(&pix, "tw5.5 + tb3.3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_morph_op_dimensions() {
        let op = MorphOp::Dilate {
            width: 3,
            height: 5,
        };
        assert_eq!(op.dimensions(), (3, 5));
    }

    #[test]
    fn test_morph_op_has_odd_dimensions() {
        let odd = MorphOp::Dilate {
            width: 3,
            height: 5,
        };
        assert!(odd.has_odd_dimensions());

        let even = MorphOp::Dilate {
            width: 4,
            height: 5,
        };
        assert!(!even.has_odd_dimensions());
    }

    #[test]
    fn test_sequence_len_and_is_empty() {
        let seq = MorphSequence::parse("d3.3 + e5.5").unwrap();
        assert_eq!(seq.len(), 2);
        assert!(!seq.is_empty());
    }

    // -----------------------------------------------------------------------
    // r / x 演算子テスト（jbig2enc で使用する演算子）
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_rank_reduce_r11() {
        // "r11" → RankReduce { levels: [1, 1] }
        let seq = MorphSequence::parse("r11").unwrap();
        assert_eq!(seq.len(), 1);
        match &seq.ops()[0] {
            MorphOp::RankReduce { levels } => assert_eq!(levels, &[1u8, 1]),
            other => panic!("expected RankReduce, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_rank_reduce_r1143() {
        // "r1143" → RankReduce { levels: [1, 1, 4, 3] }
        let seq = MorphSequence::parse("r1143").unwrap();
        match &seq.ops()[0] {
            MorphOp::RankReduce { levels } => assert_eq!(levels, &[1u8, 1, 4, 3]),
            other => panic!("expected RankReduce, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_binary_expand_x4() {
        // "x4" → BinaryExpand { factor: 4 }
        let seq = MorphSequence::parse("x4").unwrap();
        assert_eq!(seq.len(), 1);
        match &seq.ops()[0] {
            MorphOp::BinaryExpand { factor } => assert_eq!(*factor, 4u32),
            other => panic!("expected BinaryExpand, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_jbig2enc_seed_sequence() {
        // jbig2.cc の "r1143 + o4.4 + x4" がパース可能かつ 3 演算子
        let seq = MorphSequence::parse("r1143 + o4.4 + x4").unwrap();
        assert_eq!(seq.len(), 3);
    }

    #[test]
    fn test_morph_sequence_r11_size() {
        // "r11": 16x16 → 4x4
        let pix = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let result = morph_sequence(&pix, "r11").unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn test_morph_sequence_x4_size() {
        // "x4": 4x4 → 16x16
        let pix = Pix::new(4, 4, PixelDepth::Bit1).unwrap();
        let result = morph_sequence(&pix, "x4").unwrap();
        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn test_morph_sequence_r11_x4_roundtrip_size() {
        // "r11 + x4": 縮小後に拡大 → 元サイズに戻る
        let pix = Pix::new(16, 16, PixelDepth::Bit1).unwrap();
        let result = morph_sequence(&pix, "r11 + x4").unwrap();
        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn test_rank_reduce_invalid_digit() {
        // level 0 または 5 は無効
        assert!(MorphSequence::parse("r05").is_err());
    }
}
