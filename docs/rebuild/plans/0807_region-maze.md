# Maze Generation and Solving Implementation Plan

## Status: IMPLEMENTED

## Summary

迷路生成/解法（Maze）をleptonica-regionクレートに実装する。
C版Leptonicaのmaze.cに相当する機能を、Rust idiomaticなAPIで提供する。

## Reference

- C version: `reference/leptonica/src/maze.c`
- Existing patterns: `crates/leptonica-region/src/seedfill.rs`, `crates/leptonica-region/src/conncomp.rs`

## Background

迷路機能は以下の用途がある:

- 迷路ゲームの生成
- 経路探索アルゴリズムの教育・可視化
- 2点間の最短経路探索
- グラフ探索アルゴリズムの実装例

C版Leptonicaでは迷路を二値画像として表現し:

- ON pixels (1) = 壁
- OFF pixels (0) = 通路

## Implementation Scope

### Core Functions (Phase 1)

1. **迷路生成**
   - `generate_binary_maze()`: セルラーオートマトン方式で迷路を生成
     - パラメータ: サイズ、開始位置、壁確率、異方性比率

2. **二値迷路の解探索**
   - `search_binary_maze()`: BFSによる最短経路探索
     - 入力: 二値迷路画像、開始点、終了点
     - 出力: 経路（座標リスト）、可視化画像（オプション）

3. **補助機能**
   - `local_search_for_background()`: 指定点近傍で背景ピクセルを探索
   - 経路の可視化（RGB画像への描画）

### Phase 2 (Future)

- `search_gray_maze()`: グレースケール画像での最小コスト経路探索（Dijkstra）
  - コスト関数: 1 + |deltaV| （輝度勾配）
  - 優先度付きキュー（ヒープ）による実装

## API Design

```rust
// crates/leptonica-region/src/maze.rs

use crate::{RegionError, RegionResult};
use leptonica_core::{Pix, PixelDepth};
use std::collections::VecDeque;

/// Direction from parent to child in maze traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MazeDirection {
    Start = 0,
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

/// Minimum maze dimensions
pub const MIN_MAZE_WIDTH: u32 = 50;
pub const MIN_MAZE_HEIGHT: u32 = 50;

/// Default wall probability
pub const DEFAULT_WALL_PROBABILITY: f32 = 0.65;

/// Default anisotropy ratio
pub const DEFAULT_ANISOTROPY_RATIO: f32 = 0.25;

/// Options for maze generation
#[derive(Debug, Clone)]
pub struct MazeGenerationOptions {
    /// Width of the maze
    pub width: u32,
    /// Height of the maze
    pub height: u32,
    /// Initial x position (seed point)
    pub start_x: u32,
    /// Initial y position (seed point)
    pub start_y: u32,
    /// Probability that a side pixel becomes a wall (0.05 - 0.95)
    pub wall_probability: f32,
    /// Ratio of forward vs side wall probability (0.05 - 1.0)
    pub anisotropy_ratio: f32,
}

impl Default for MazeGenerationOptions {
    fn default() -> Self {
        Self {
            width: MIN_MAZE_WIDTH,
            height: MIN_MAZE_HEIGHT,
            start_x: 0, // Will be set to width/6 if 0
            start_y: 0, // Will be set to height/5 if 0
            wall_probability: DEFAULT_WALL_PROBABILITY,
            anisotropy_ratio: DEFAULT_ANISOTROPY_RATIO,
        }
    }
}

impl MazeGenerationOptions {
    /// Create options with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Set the starting position
    pub fn with_start(mut self, x: u32, y: u32) -> Self {
        self.start_x = x;
        self.start_y = y;
        self
    }

    /// Set wall probability
    pub fn with_wall_probability(mut self, prob: f32) -> Self {
        self.wall_probability = prob;
        self
    }

    /// Set anisotropy ratio
    pub fn with_anisotropy(mut self, ratio: f32) -> Self {
        self.anisotropy_ratio = ratio;
        self
    }
}

/// Result of maze search
#[derive(Debug, Clone)]
pub struct MazePath {
    /// Sequence of points from start to end
    pub points: Vec<(u32, u32)>,
    /// Whether a path was found
    pub found: bool,
}

impl MazePath {
    /// Get the length of the path
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Generate a binary maze using cellular automaton approach
///
/// Creates a maze where ON pixels (1) are walls and OFF pixels (0) are passages.
/// The maze is generated using a queue-based BFS from the starting point,
/// with probabilistic wall placement.
///
/// # Arguments
///
/// * `options` - Maze generation options
///
/// # Returns
///
/// A 1-bit binary image representing the maze
///
/// # Notes
///
/// The `wall_probability` and `anisotropy_ratio` parameters control maze density:
/// - Higher wall_probability = denser walls
/// - Lower anisotropy_ratio = longer straight passages
///
/// Phase transition boundary (approximate):
/// ```text
/// wall_prob  anisotropy
/// 0.35       1.00
/// 0.50       0.50
/// 0.65       0.25
/// 0.80       0.11
/// ```
pub fn generate_binary_maze(options: &MazeGenerationOptions) -> RegionResult<Pix>;

/// Search for the shortest path in a binary maze
///
/// Uses breadth-first search (BFS) to find the shortest Manhattan path
/// between start and end points, avoiding walls (ON pixels).
///
/// # Arguments
///
/// * `maze` - Binary maze image (1-bit, ON = wall, OFF = passage)
/// * `start` - Starting point (xi, yi)
/// * `end` - Ending point (xf, yf)
/// * `visualize` - If true, also returns a visualization image
///
/// # Returns
///
/// A tuple of (MazePath, Option<Pix>):
/// - MazePath contains the path coordinates (or empty if no path)
/// - If visualize=true, returns 32-bit RGB image with:
///   - Red pixel at start
///   - Blue pixel at end
///   - Green pixels along the path
///   - If no path, green marks explored region
pub fn search_binary_maze(
    maze: &Pix,
    start: (u32, u32),
    end: (u32, u32),
    visualize: bool,
) -> RegionResult<(MazePath, Option<Pix>)>;

/// Render the maze path onto an image
///
/// # Arguments
///
/// * `maze` - Original binary maze image
/// * `path` - Path to render
/// * `start` - Start point (marked in red)
/// * `end` - End point (marked in blue)
///
/// # Returns
///
/// 32-bit RGB image with the path visualized
pub fn render_maze_path(
    maze: &Pix,
    path: &MazePath,
    start: (u32, u32),
    end: (u32, u32),
) -> RegionResult<Pix>;
```

## Implementation Details

### Module Structure

```text
crates/leptonica-region/src/
├── lib.rs       # Add: mod maze; pub use maze::*;
├── conncomp.rs  # Existing
├── seedfill.rs  # Existing
├── error.rs     # May need new error variants
└── maze.rs      # NEW: Maze generation and search
```

### Algorithm: Maze Generation (generateBinaryMaze equivalent)

C版の迷路生成アルゴリズム:

```text
Input: width, height, start position (xi, yi), wallps, ranis
Output: Binary maze image

1. Initialize:
   - pixd = empty binary image (all OFF)
   - pixm = visited marker image (all OFF)
   - queue = empty queue
   - wallpf = wallps * ranis (forward wall probability)

2. Prime queue with start pixel:
   - Mark (xi, yi) as visited in pixm
   - Add MazeElement(xi, yi, START) to queue

3. While queue is not empty:
   a. Pop element from queue
   b. For each direction (West, North, East, South):
      - Check if neighbor is not yet visited
      - If not visited:
        - Mark as visited
        - Determine wall probability:
          - If same direction as parent: use wallpf
          - Otherwise: use wallps
        - Generate random number
        - If random < probability: set pixel as wall
        - Else: add to queue as passage

4. Return pixd
```

### Algorithm: Maze Search (pixSearchBinaryMaze equivalent)

```text
Input: maze image, start (xi, yi), end (xf, yf)
Output: path as list of points, optional visualization

1. Validate inputs:
   - Check maze is 1-bit
   - Check start/end are within bounds
   - Check start is background pixel (passage)

2. Adjust end point if on wall:
   - local_search_for_background(end, maxrad=5)

3. Initialize:
   - pixm = visited marker (1-bit)
   - pixp = parent direction map (8-bit)
   - queue = empty queue

4. Prime queue with start:
   - Mark start as visited
   - Add to queue

5. BFS loop:
   While queue not empty:
   a. Pop (x, y) from queue
   b. If (x, y) == end: found = true, break
   c. For each direction (W, N, E, S):
      - If neighbor not visited and is background:
        - Mark visited
        - Store parent direction in pixp
        - Add to queue

6. If found:
   - Backtrack from end to start using pixp
   - Build path in reverse order

7. Visualization (if requested):
   - Convert maze to 32-bit RGB
   - Mark path in green
   - Mark start in red, end in blue
```

### Direction Constants and Tables

```rust
/// X offset for each direction
const DX: [i32; 5] = [0, 0, 0, -1, 1]; // Start, N, S, W, E

/// Y offset for each direction
const DY: [i32; 5] = [0, -1, 1, 0, 0]; // Start, N, S, W, E

/// Opposite direction (for parent tracking)
const OPPOSITE: [MazeDirection; 5] = [
    MazeDirection::Start,
    MazeDirection::South,  // opposite of North
    MazeDirection::North,  // opposite of South
    MazeDirection::East,   // opposite of West
    MazeDirection::West,   // opposite of East
];
```

### Error Handling

- `UnsupportedDepth`: 非1bpp画像に対する操作
- `InvalidParameters`: 無効な開始/終了位置
- `InvalidSeed`: 開始点が壁の場合

### Performance Considerations

1. **メモリ効率**:
   - キュー要素は座標と方向のみ（C版のMazeElementより軽量）
   - 訪問済みマーカーは1ビット画像で十分

2. **乱数生成**:
   - `rand` crateを使用（stdではなく）
   - スレッドローカルRNGで効率化

3. **大きな迷路**:
   - BFSはメモリ使用量がO(perimeter)
   - 最悪ケースでもO(width * height)

## Tasks

1. [x] Create implementation plan
2. [x] Add `rand` to dependencies if not present
3. [x] Create `src/maze.rs` with module structure
4. [x] Implement `MazeDirection` enum
5. [x] Implement `MazeGenerationOptions` struct
6. [x] Implement `MazePath` struct
7. [x] Implement `generate_binary_maze()`
8. [x] Implement `local_search_for_background()` helper
9. [x] Implement `search_binary_maze()`
10. [x] Implement `render_maze_path()`
11. [x] Add unit tests for all functions
12. [x] Update `lib.rs` with module and re-exports
13. [x] Run `cargo fmt && cargo clippy`
14. [x] Run full test suite
15. [ ] Commit

## Test Plan

1. **迷路生成**:
   - 最小サイズ (50x50) での生成
   - 大きなサイズ (200x200) での生成
   - パラメータ境界値のテスト
   - 生成された迷路が有効（開始点が通路）

2. **経路探索**:
   - 単純な迷路での探索
   - 経路が存在する場合の検証
   - 経路が存在しない場合の検証
   - 開始点=終了点のケース

3. **可視化**:
   - パスが正しくRGB画像に描画される
   - 開始点と終了点の色が正しい

4. **エッジケース**:
   - 空の迷路（全て通路）
   - 完全に壁の迷路
   - 開始点/終了点が壁の近くにある場合

## Questions

(Resolved - `rand` crate was added to workspace dependencies)

## Estimates

- Implementation: ~400-500 lines of code
- Time: ~3-4 hours
