# 論理演算 (Raster Operations) 実装計画

## 概要

画像の論理演算（ラスター操作）を実装する。LeptonicaのC版 `rop.c` と `roplow.c` の機能をRustで再実装する。

## 参照ファイル

- C版ソース: `reference/leptonica/src/rop.c`, `reference/leptonica/src/roplow.c`
- 既存パターン: `crates/leptonica-core/src/pix/compare.rs`, `crates/leptonica-core/src/pix/blend.rs`

## 実装対象機能

### 基本論理演算

| 演算 | C関数 | Rust関数 | 説明 |
|-----|-------|----------|-----|
| AND | pixAnd | `and` / `and_inplace` | 論理積 (s & d) |
| OR | pixOr | `or` / `or_inplace` | 論理和 (s \| d) |
| XOR | pixXor | `xor` / `xor_inplace` | 排他的論理和 (s ^ d) |
| NOT | pixInvert | `invert` / `invert_inplace` | 論理否定 (~d) |

### 拡張論理演算

| 演算 | 説明 |
|-----|-----|
| NAND | ~(s & d) |
| NOR | ~(s \| d) |
| XNOR | ~(s ^ d) |
| AND_NOT | s & ~d |
| OR_NOT | s \| ~d |

### 定数演算

| 演算 | 関数名 | 説明 |
|-----|-------|-----|
| CLR | `clear` / `clear_region` | すべてのビットを0に |
| SET | `set_all` / `set_region` | すべてのビットを1に |

## 設計方針

### 1. モジュール構成

```
crates/leptonica-core/src/pix/
├── mod.rs       (pub mod rop; を追加)
├── rop.rs       (新規作成)
├── compare.rs
└── blend.rs
```

### 2. API設計 (compare.rs, blend.rsパターンに準拠)

```rust
//! Image raster operations (logical operations)
//!
//! This module provides functions for pixel-wise logical operations:
//!
//! - AND, OR, XOR, NOT operations
//! - NAND, NOR, XNOR operations
//! - In-place operations
//! - Region-based operations
//!
//! These correspond to Leptonica's rop.c functions including
//! pixAnd, pixOr, pixXor, and pixInvert.

/// Raster operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RopOp {
    /// Clear: d = 0
    Clear,
    /// Set: d = 1
    Set,
    /// Copy source: d = s
    Src,
    /// Invert destination: d = ~d
    NotDst,
    /// Invert source: d = ~s
    NotSrc,
    /// AND: d = s & d
    And,
    /// OR: d = s | d
    Or,
    /// XOR: d = s ^ d
    Xor,
    /// NAND: d = ~(s & d)
    Nand,
    /// NOR: d = ~(s | d)
    Nor,
    /// XNOR: d = ~(s ^ d)
    Xnor,
    /// AND with inverted source: d = ~s & d
    AndNotSrc,
    /// AND with inverted dest: d = s & ~d
    AndNotDst,
    /// OR with inverted source: d = ~s | d
    OrNotSrc,
    /// OR with inverted dest: d = s | ~d
    OrNotDst,
}
```

### 3. メソッド実装

#### Pixへの拡張メソッド

```rust
impl Pix {
    /// Perform AND operation with another image
    pub fn and(&self, other: &Pix) -> Result<Pix>;

    /// Perform OR operation with another image
    pub fn or(&self, other: &Pix) -> Result<Pix>;

    /// Perform XOR operation with another image
    pub fn xor(&self, other: &Pix) -> Result<Pix>;

    /// Invert all pixels
    pub fn invert(&self) -> Result<Pix>;

    /// Apply general raster operation
    pub fn rop(&self, other: &Pix, op: RopOp) -> Result<Pix>;

    /// Apply raster operation to a region
    pub fn rop_region(&self, other: &Pix, op: RopOp,
                      dx: i32, dy: i32, dw: u32, dh: u32,
                      sx: i32, sy: i32) -> Result<Pix>;
}
```

#### PixMutへのインプレース操作

```rust
impl PixMut {
    /// In-place AND operation
    pub fn and_inplace(&mut self, other: &Pix) -> Result<()>;

    /// In-place OR operation
    pub fn or_inplace(&mut self, other: &Pix) -> Result<()>;

    /// In-place XOR operation
    pub fn xor_inplace(&mut self, other: &Pix) -> Result<()>;

    /// In-place invert
    pub fn invert_inplace(&mut self);

    /// Clear a region
    pub fn clear_region(&mut self, x: u32, y: u32, w: u32, h: u32);

    /// Set a region
    pub fn set_region(&mut self, x: u32, y: u32, w: u32, h: u32);
}
```

### 4. 実装詳細

#### バイナリ画像の最適化

1-bit画像に対しては、word単位（32-bit）で演算を行い、高速化する：

```rust
fn and_binary(&self, other: &Pix) -> Result<Pix> {
    let width = self.width();
    let height = self.height();
    let wpl = self.wpl();

    let result = Pix::new(width, height, PixelDepth::Bit1)?;
    let mut result_mut = result.try_into_mut().unwrap();

    for y in 0..height {
        let line1 = self.row_data(y);
        let line2 = other.row_data(y);
        let line_out = result_mut.row_data_mut(y);

        for w in 0..wpl as usize {
            line_out[w] = line1[w] & line2[w];
        }
    }

    Ok(result_mut.into())
}
```

#### グレースケール/RGB画像の演算

8/32-bit画像では、ピクセル値またはチャンネル単位で演算する：

```rust
fn and_gray(&self, other: &Pix) -> Result<Pix> {
    let width = self.width();
    let height = self.height();

    let result = Pix::new(width, height, self.depth())?;
    let mut result_mut = result.try_into_mut().unwrap();

    for y in 0..height {
        for x in 0..width {
            let v1 = self.get_pixel(x, y).unwrap_or(0);
            let v2 = other.get_pixel(x, y).unwrap_or(0);
            unsafe { result_mut.set_pixel_unchecked(x, y, v1 & v2) };
        }
    }

    Ok(result_mut.into())
}
```

### 5. エラーハンドリング

以下のケースでエラーを返す：

- 画像サイズの不一致 → `Error::DimensionMismatch`
- ビット深度の不一致 → `Error::IncompatibleDepths`
- 領域が画像外 → `Error::InvalidParameter`

## 実装手順

### Phase 1: 基本構造

1. [x] `rop.rs`ファイル作成
2. [x] `RopOp` enum定義
3. [x] モジュールをlib.rsに追加

### Phase 2: コア実装

1. [x] バイナリ画像用の基本演算実装（AND, OR, XOR, NOT）
2. [x] グレースケール画像用の演算実装
3. [x] RGB画像用の演算実装

### Phase 3: インプレース操作

1. [x] PixMutへのインプレース操作実装
2. [x] 領域指定演算の実装

### Phase 4: テスト

1. [x] ユニットテスト作成
2. [x] 各深度でのテスト

## テスト計画

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_and_binary() {
        // 1-bit画像のAND演算テスト
    }

    #[test]
    fn test_or_binary() {
        // 1-bit画像のOR演算テスト
    }

    #[test]
    fn test_xor_binary() {
        // 1-bit画像のXOR演算テスト
    }

    #[test]
    fn test_invert() {
        // NOT演算テスト
    }

    #[test]
    fn test_and_gray() {
        // 8-bit画像のAND演算テスト
    }

    #[test]
    fn test_dimension_mismatch_error() {
        // サイズ不一致エラーのテスト
    }

    #[test]
    fn test_inplace_operations() {
        // インプレース操作のテスト
    }
}
```

## 成功基準

- [x] すべての基本論理演算（AND, OR, XOR, NOT）が実装される
- [x] 1-bit, 8-bit, 32-bit画像に対応
- [x] インプレース操作が実装される
- [x] 既存のcompare.rs, blend.rsと同様のパターンに従う
- [x] cargo fmt && cargo clippy が警告なしでパス
- [x] すべてのユニットテストがパス

## 質問

（現時点では質問なし）
