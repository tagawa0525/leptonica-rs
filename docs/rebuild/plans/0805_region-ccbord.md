# Border Tracing Implementation Plan

## Status: IMPLEMENTED

## Summary

境界追跡（連結成分の輪郭抽出）をleptonica-regionクレートに実装する。
C版Leptonicaのccbord.cに相当する機能を、Rust idiomaticなAPIで提供する。

## Reference

- C version: `reference/leptonica/src/ccbord.c`
- Existing patterns: `crates/leptonica-region/src/conncomp.rs`, `crates/leptonica-region/src/seedfill.rs`

## Background

境界追跡は画像処理の基本操作であり、以下の用途がある:

- 連結成分の輪郭抽出
- 形状解析（周囲長、凸性など）
- ベクトル化（SVG/PostScript出力）
- 文字認識の前処理
- オブジェクト検出の後処理

## Implementation Scope

### Core Functions (Phase 1)

1. **外部境界追跡**
   - `get_outer_border()`: 連結成分の外周を時計回りに追跡
   - `get_outer_borders()`: 画像中の全連結成分の外周を取得

2. **内部境界（穴）追跡**
   - `get_hole_borders()`: 連結成分内の穴の境界を反時計回りに追跡
   - `get_all_borders()`: 外部境界と穴境界の両方を取得

3. **チェーンコード表現**
   - `to_chain_code()`: 境界点列をチェーンコード（方向列）に変換
   - `from_chain_code()`: チェーンコードから境界点列を復元

4. **境界点リスト生成**
   - `BorderPoints`: 境界点の座標リストを保持する構造体
   - ローカル座標とグローバル座標の変換

### Phase 2 (Future)

- `to_svg()`: SVG path形式での出力
- `to_postscript()`: PostScript形式での出力
- `simplify_border()`: 境界点の間引き（Douglas-Peucker等）
- `single_path()`: 穴を含む連結成分を単一パスに変換

## API Design

```rust
// crates/leptonica-region/src/ccbord.rs

use crate::{ConnectivityType, RegionError, RegionResult};
use leptonica_core::{Box, Pix, PixelDepth};

/// Direction for chain code representation (8-connectivity)
/// Uses the standard 8-direction encoding:
///   1  2  3
///   0  P  4
///   7  6  5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    West = 0,
    NorthWest = 1,
    North = 2,
    NorthEast = 3,
    East = 4,
    SouthEast = 5,
    South = 6,
    SouthWest = 7,
}

/// A point on a border
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderPoint {
    pub x: i32,
    pub y: i32,
}

/// Border type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    /// Outer border (clockwise traversal)
    Outer,
    /// Hole border (counter-clockwise traversal)
    Hole,
}

/// A single border (outer or hole)
#[derive(Debug, Clone)]
pub struct Border {
    /// Type of this border
    pub border_type: BorderType,
    /// Starting point of the border
    pub start: BorderPoint,
    /// All points on the border (in traversal order)
    pub points: Vec<BorderPoint>,
    /// Chain code representation (if computed)
    pub chain_code: Option<Vec<Direction>>,
}

/// Collection of borders for a single connected component
#[derive(Debug, Clone)]
pub struct ComponentBorders {
    /// Bounding box of the component (in global coordinates)
    pub bounds: Box,
    /// The outer border
    pub outer: Border,
    /// Hole borders (may be empty)
    pub holes: Vec<Border>,
}

/// Collection of all borders in an image
#[derive(Debug, Clone)]
pub struct ImageBorders {
    /// Width of the source image
    pub width: u32,
    /// Height of the source image
    pub height: u32,
    /// Borders for each connected component
    pub components: Vec<ComponentBorders>,
}

impl Border {
    /// Create a new border from points
    pub fn new(border_type: BorderType, points: Vec<BorderPoint>) -> Self;

    /// Get the number of points in this border
    pub fn len(&self) -> usize;

    /// Check if the border is empty
    pub fn is_empty(&self) -> bool;

    /// Compute and store chain code representation
    pub fn compute_chain_code(&mut self);

    /// Convert points to global coordinates by adding offset
    pub fn to_global(&self, offset_x: i32, offset_y: i32) -> Border;

    /// Get perimeter (number of boundary pixels)
    pub fn perimeter(&self) -> usize;
}

impl ComponentBorders {
    /// Get total number of borders (outer + holes)
    pub fn border_count(&self) -> usize;

    /// Check if component has holes
    pub fn has_holes(&self) -> bool;
}

// Main API functions

/// Get the outer border of a single connected component
///
/// # Arguments
/// * `pix` - Binary image containing exactly one 8-connected component
/// * `bounds` - Optional bounding box for global coordinate calculation
///
/// # Returns
/// Border with points in local coordinates (relative to pix origin)
pub fn get_outer_border(pix: &Pix, bounds: Option<&Box>) -> RegionResult<Border>;

/// Get all outer borders from a binary image
///
/// # Arguments
/// * `pix` - Binary image (1-bit depth)
///
/// # Returns
/// Vector of borders, one for each connected component
pub fn get_outer_borders(pix: &Pix) -> RegionResult<Vec<Border>>;

/// Get all borders (outer and holes) for a single connected component
///
/// # Arguments
/// * `pix` - Binary image containing exactly one 8-connected component
/// * `bounds` - Bounding box for global coordinate calculation
///
/// # Returns
/// ComponentBorders containing outer border and any hole borders
pub fn get_component_borders(pix: &Pix, bounds: Box) -> RegionResult<ComponentBorders>;

/// Get all borders from a binary image
///
/// # Arguments
/// * `pix` - Binary image (1-bit depth)
///
/// # Returns
/// ImageBorders containing borders for all connected components
pub fn get_all_borders(pix: &Pix) -> RegionResult<ImageBorders>;

/// Convert border points to chain code
///
/// # Arguments
/// * `points` - Sequence of adjacent border points
///
/// # Returns
/// Vector of directions representing the chain code
pub fn to_chain_code(points: &[BorderPoint]) -> Vec<Direction>;

/// Convert chain code back to border points
///
/// # Arguments
/// * `start` - Starting point
/// * `chain` - Chain code directions
///
/// # Returns
/// Vector of border points
pub fn from_chain_code(start: BorderPoint, chain: &[Direction]) -> Vec<BorderPoint>;
```

## Implementation Details

### Module Structure

```text
crates/leptonica-region/src/
├── mod.rs       # Add: mod ccbord; pub use ccbord::*;
├── conncomp.rs  # Existing
├── seedfill.rs  # Existing
├── error.rs     # May need new error variants
└── ccbord.rs    # NEW: Border tracing functions
```

### Algorithm: Border Tracing (pixGetOuterBorder equivalent)

C版の境界追跡アルゴリズムを参考に実装する。

```text
Input: Binary image with a single connected component
Output: List of border pixels in clockwise order

1. Add 1-pixel border around image (for edge handling)
2. Find first ON pixel by raster scan (start pixel)
3. Initialize:
   - first_pixel = start_pixel
   - second_pixel = null
   - qpos = 0 (direction of search start)

4. Loop:
   a. Find next border pixel by rotating clockwise from qpos
   b. If this is first iteration, save as second_pixel
   c. If current == first_pixel AND next == second_pixel:
      BREAK (completed full circle)
   d. Add next pixel to border list
   e. Update position and qpos

5. Convert all coordinates back (subtract 1 for border offset)
```

### Direction Tables

```rust
/// X offset for each direction
const XPOSTAB: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];

/// Y offset for each direction
const YPOSTAB: [i32; 8] = [0, -1, -1, -1, 0, 1, 1, 1];

/// New qpos when moving to direction pos
const QPOSTAB: [usize; 8] = [6, 6, 0, 0, 2, 2, 4, 4];
```

### Algorithm: Hole Detection

```text
1. Fill holes in component using fill_holes()
2. XOR original with filled -> get hole pixels
3. Find connected components in hole image
4. For each hole:
   a. Find start pixel on hole boundary (adjacent to component)
   b. Trace hole border (counter-clockwise)
```

### Algorithm: Chain Code Generation

```text
For consecutive points (p1, p2):
  dx = p2.x - p1.x
  dy = p2.y - p1.y

  direction = DIRTAB[1 + dy][1 + dx]

where DIRTAB is:
  [1, 2, 3]
  [0, -, 4]
  [7, 6, 5]
```

### Error Handling

- 非1bpp画像: `UnsupportedDepth`
- 空画像: `EmptyImage`
- 連結成分なし: 空のリストを返す（エラーではない）
- 無効なチェーンコード: `InvalidParameters`

### Performance Considerations

1. **1ピクセルボーダー追加**: コピーではなくオフセット計算で対応可能な場合は最適化
2. **小さなコンポーネント**: 1-2ピクセルのコンポーネントの特別処理
3. **メモリ**: 大きな画像でも境界点のみを保存するため効率的

## Tasks

1. [x] Create implementation plan
2. [x] Add error variants to `error.rs` if needed
3. [x] Create `src/ccbord.rs` with module structure
4. [x] Implement `Direction` enum and conversion functions
5. [x] Implement `BorderPoint`, `Border`, `ComponentBorders` structs
6. [x] Implement `find_next_border_pixel()` helper
7. [x] Implement `get_outer_border()`
8. [x] Implement `get_outer_borders()`
9. [x] Implement hole detection and `get_component_borders()`
10. [x] Implement `get_all_borders()`
11. [x] Implement `to_chain_code()` and `from_chain_code()`
12. [x] Add unit tests for all functions
13. [x] Update `lib.rs` with module and re-exports
14. [x] Run `cargo fmt && cargo clippy`
15. [x] Run full test suite

## Test Plan

1. **単一ピクセル**:
   - 1ピクセルの境界は1点のみ

2. **単純な形状**:
   - 正方形、長方形の境界追跡
   - L字型、T字型の境界追跡

3. **穴のある形状**:
   - ドーナツ型（外周+1つの穴）
   - 複数の穴を持つ形状

4. **複数コンポーネント**:
   - 離れた複数の連結成分

5. **エッジケース**:
   - 画像端に接するコンポーネント
   - 空画像
   - 全て黒/全て白の画像

6. **チェーンコード**:
   - 往復変換の一致
   - 既知の形状でのチェーンコード検証

## Questions

(None at this time)

## Estimates

- Implementation: ~500-700 lines of code
- Time: ~4-5 hours
