# Morphological Sequence Operations Implementation Plan

## Overview

シーケンス操作は形態学演算の連続適用を文字列で指定できる機能です。
C版の `morphseq.c` を参考に Rust 実装を行います。

## Target Functions

### Primary Functions

- `morph_sequence()` - 二値画像用シーケンス
- `morph_comp_sequence()` - 複合シーケンス（大きなSEL用に最適化）
- `gray_morph_sequence()` - グレースケール画像用シーケンス

### Supporting Functions

- `MorphSequence::parse()` - シーケンス文字列パーサー
- `MorphSequence::verify()` - シーケンス検証

## Sequence String Format

### Binary Operations (d, e, o, c)

- `d<w>.<h>` - Dilation with w x h brick SE
- `e<w>.<h>` - Erosion with w x h brick SE
- `o<w>.<h>` - Opening with w x h brick SE
- `c<w>.<h>` - Closing with w x h brick SE

### Special Operations

- `r<levels>` - Rank binary reduction
  (e.g., `r23` = 2x reductions with ranks 2 and 3)
- `x<factor>` - Replicative expansion (factor must be 2, 4, 8, or 16)
- `b<size>` - Add border of specified size (must be first operation)

### Grayscale Operations

- `d<w>.<h>` - Grayscale dilation
- `e<w>.<h>` - Grayscale erosion
- `o<w>.<h>` - Grayscale opening
- `c<w>.<h>` - Grayscale closing
- `t<type><w>.<h>` - Tophat (type: 'w' for white, 'b' for black)

### Syntax

- Operations separated by `+`
- Whitespace ignored
- Case insensitive

### Examples

```text
"o5.5 + e3.3"           # 5x5 opening, then 3x3 erosion
"b32 + o1.3 + r23 + X4" # Add border, opening, reduce, expand
"c9.9 + tw9.9"          # 9x9 closing, then white tophat
```

## Implementation Design

### Module Structure

```text
crates/leptonica-morph/src/
  sequence.rs          # New file for sequence operations
  lib.rs               # Add module export
```

### Type Definitions

```rust
/// Parsed morphological operation
#[derive(Debug, Clone, PartialEq)]
pub enum MorphOp {
    /// Dilation with brick SE
    Dilate { width: u32, height: u32 },
    /// Erosion with brick SE
    Erode { width: u32, height: u32 },
    /// Opening with brick SE
    Open { width: u32, height: u32 },
    /// Closing with brick SE
    Close { width: u32, height: u32 },
    /// Rank binary reduction
    RankReduce { levels: Vec<u8> },
    /// Replicative expansion
    Expand { factor: u32 },
    /// Add border
    Border { size: u32 },
    /// Tophat (grayscale only)
    Tophat { white: bool, width: u32, height: u32 },
}

/// Parsed morphological sequence
#[derive(Debug, Clone)]
pub struct MorphSequence {
    ops: Vec<MorphOp>,
    border: Option<u32>,
}
```

### Public API

```rust
// sequence.rs

impl MorphSequence {
    /// Parse a sequence string
    pub fn parse(sequence: &str) -> MorphResult<Self>;

    /// Verify the sequence is valid for binary operations
    pub fn verify_binary(&self) -> MorphResult<()>;

    /// Verify the sequence is valid for grayscale operations
    pub fn verify_grayscale(&self) -> MorphResult<()>;
}

/// Execute binary morphological sequence
pub fn morph_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix>;

/// Execute binary morphological sequence with composite operations
/// (optimized for large structuring elements)
pub fn morph_comp_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix>;

/// Execute grayscale morphological sequence
pub fn gray_morph_sequence(pix: &Pix, sequence: &str) -> MorphResult<Pix>;
```

## Implementation Steps

### Step 1: Parser Implementation

1. Create `sequence.rs` module
2. Define `MorphOp` enum
3. Define `MorphSequence` struct
4. Implement `MorphSequence::parse()` using string parsing
5. Implement `MorphSequence::verify_binary()`
6. Implement `MorphSequence::verify_grayscale()`

### Step 2: Binary Sequence Execution

1. Implement `morph_sequence()` function
2. Handle all binary operations (d, e, o, c, r, x, b)
3. Implement border handling (add at start, remove at end)

### Step 3: Composite Sequence

1. Implement `morph_comp_sequence()`
2. Note: For now, can delegate to `morph_sequence()` as we
   don't have composite operations yet

### Step 4: Grayscale Sequence

1. Implement `gray_morph_sequence()`
2. Handle grayscale operations (d, e, o, c, t)
3. Validate SE sizes are odd for grayscale

### Step 5: Module Integration

1. Update `lib.rs` to export new module
2. Add re-exports for public functions

### Step 6: Testing

1. Unit tests for parser
2. Unit tests for each operation type
3. Integration tests with actual images
4. Edge case tests (empty sequence, invalid operations, etc.)

## Dependencies

### Internal Dependencies

- `leptonica_core::Pix` - Image type
- `crate::binary::*` - Binary morphology functions
- `crate::grayscale::*` - Grayscale morphology functions
- `crate::Sel` - Structuring element

### Missing Functionality

Note: The following C functions are referenced but may not be implemented yet:

- `pixReduceRankBinaryCascade` - Rank binary reduction (skip for initial implementation)
- `pixExpandReplicate` - Replicative expansion (skip for initial implementation)
- `pixAddBorder` / `pixRemoveBorder` - Border operations (skip for initial implementation)

For the initial implementation, we will:

1. Implement basic operations (d, e, o, c) fully
2. Return errors for unsupported operations (r, x, b) with clear messages
3. Add TODO comments for future implementation

## Error Handling

```rust
// Add to error.rs
pub enum MorphError {
    // ... existing variants ...

    /// Invalid sequence format
    InvalidSequence(String),

    /// Unsupported operation in sequence
    UnsupportedOperation(String),
}
```

## Test Plan

### Parser Tests

- Valid operation parsing: "d3.3", "e5.1", "o7.7", "c11.3"
- Sequence parsing: "d3.3 + e5.5"
- Whitespace handling
- Case insensitivity
- Invalid format detection
- Grayscale tophat parsing: "tw5.5", "tb3.3"

### Execution Tests

- Single operation sequences
- Multiple operation sequences
- Binary vs grayscale validation
- Size validation for grayscale (must be odd)
- Error cases

### Integration Tests

- Compare results with individual function calls
- Verify idempotent operations work correctly

## Questions

1. Should we support DWA (destination word aligned) operations?
   - Decision: Skip for now, not critical for initial implementation

2. Should we support color morphology sequences?
   - Decision: Skip for now, can add later as `color_morph_sequence()`

3. How to handle rank reduction and expansion operations?
   - Decision: Return `UnsupportedOperation` error with message
     suggesting future support

## Timeline Estimate

- Parser implementation: ~30 min
- Binary sequence execution: ~30 min
- Grayscale sequence execution: ~20 min
- Testing: ~30 min
- Integration and cleanup: ~10 min

Total: ~2 hours

## Approval

Please review this plan and confirm before implementation begins.
