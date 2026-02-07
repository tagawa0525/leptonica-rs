# DWA (高速形態学) 実装計画

## 概要

DWA (Destination Word Accumulation) は、ワードアライメントを活用した高速な形態学演算アルゴリズムです。
従来のピクセル単位の処理ではなく、32ビットまたは64ビットのワード単位で処理することで、
大幅な高速化を実現します。

## C版の分析

### morphdwa.c の主要機能

1. **基本DWA演算** (ブリックSel使用)
   - `pixDilateBrickDwa()` - DWAによる高速膨張
   - `pixErodeBrickDwa()` - DWAによる高速侵食
   - `pixOpenBrickDwa()` - DWAによる高速オープニング
   - `pixCloseBrickDwa()` - DWAによる高速クロージング

2. **複合DWA演算** (分解可能なブリックSel)
   - `pixDilateCompBrickDwa()` - 分解を用いた高速膨張
   - `pixErodeCompBrickDwa()` - 分解を用いた高速侵食
   - `pixOpenCompBrickDwa()` - 分解を用いた高速オープニング
   - `pixCloseCompBrickDwa()` - 分解を用いた高速クロージング

3. **拡張複合DWA演算** (63ピクセル超のSel)
   - `pixDilateCompBrickExtendDwa()` など

### DWAの核心原理

1. **ワードアライメント**: ビット操作を32/64ビット単位で行う
2. **分離可能分解**: 2D演算を水平・垂直の1D演算に分解
3. **コンパイル時最適化**: Selパターンをコードとして生成

## 実装方針

### フェーズ1: 基礎インフラ (本実装)

完全なDWA実装は非常に複雑（コード生成が必要）なため、
以下の簡略化アプローチを採用します：

1. **ワードアライメント最適化**
   - 32ピクセル単位での水平処理
   - シフト操作による高速膨張/侵食

2. **分離可能演算**
   - 水平方向と垂直方向を別々に処理
   - 1D演算の組み合わせで2D演算を実現

3. **基本ブリック形状のサポート**
   - 水平線 (1 x N)
   - 垂直線 (M x 1)
   - 矩形 (M x N) = 水平 + 垂直の組み合わせ

### 実装ファイル構成

```text
crates/leptonica-morph/src/
  dwa.rs          # DWA高速演算メインモジュール
  lib.rs          # 更新（dwaモジュール追加）
```

## API設計

```rust
// crates/leptonica-morph/src/dwa.rs

/// DWAによる高速膨張（ブリック形状）
///
/// ワードアライメントを活用した高速な膨張演算。
/// 水平・垂直方向に分離して処理することで高速化を実現。
pub fn dilate_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// DWAによる高速侵食（ブリック形状）
pub fn erode_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// DWAによる高速オープニング（ブリック形状）
pub fn open_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;

/// DWAによる高速クロージング（ブリック形状）
pub fn close_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix>;
```

## 詳細設計

### 1. 水平方向膨張 (dilate_horizontal_dwa)

```rust
/// ワードアライメントを活用した水平膨張
///
/// 32ビット境界にアラインされたデータに対して、
/// シフトとOR演算で高速に膨張を行う。
fn dilate_horizontal_dwa(pix: &Pix, hsize: u32) -> MorphResult<Pix> {
    // 1. 出力画像を作成
    // 2. 各行について:
    //    - ワード単位でデータを取得
    //    - 左右にシフトしてOR（膨張）
    //    - 結果を書き込み
}
```

アルゴリズム:

- hsize=3の場合: result = current | (current << 1) | (current >> 1)
- 隣接ワードからのビット取り込みが必要

### 2. 垂直方向膨張 (dilate_vertical_dwa)

```rust
/// 垂直方向の膨張
///
/// 行間でOR演算を行う。
fn dilate_vertical_dwa(pix: &Pix, vsize: u32) -> MorphResult<Pix> {
    // 1. 各列について:
    //    - 上下の行とOR演算
}
```

### 3. 2D膨張 (dilate_brick_dwa)

```rust
pub fn dilate_brick_dwa(pix: &Pix, hsize: u32, vsize: u32) -> MorphResult<Pix> {
    // 分離可能: 水平膨張 → 垂直膨張
    let temp = dilate_horizontal_dwa(pix, hsize)?;
    dilate_vertical_dwa(&temp, vsize)
}
```

### 4. 侵食の実装

侵食は膨張の双対演算:

- 水平侵食: AND演算（シフトしてAND）
- 垂直侵食: 行間でAND演算

### 5. 境界処理

C版の境界条件設定に倣い:

- 膨張: 境界外は0（背景）として扱う
- 侵食: 境界条件に応じて0または1

## テスト計画

### 単体テスト

1. **基本機能テスト**
   - 小さな画像での正確性検証
   - 既存binary.rsの結果と比較

2. **エッジケース**
   - サイズ1x1の演算（単位元）
   - 画像境界での正しい処理
   - 非ワードアライン幅の画像

3. **パフォーマンステスト**
   - 大きな画像での速度測定
   - 通常版との速度比較

### 統合テスト

1. オープニング/クロージングの正確性
2. 繰り返し演算の安定性

## 実装ステップ

1. [x] dwa.rsファイル作成
2. [x] 水平方向膨張の実装
3. [x] 垂直方向膨張の実装
4. [x] dilate_brick_dwa完成
5. [x] 侵食演算の実装
6. [x] オープニング/クロージングの実装
7. [x] テスト作成
8. [x] lib.rsへのエクスポート追加
9. [x] cargo fmt && cargo clippy
10. [x] 全テストパス確認

## パフォーマンス目標

- 通常のピクセル単位処理と比較して3-10倍の高速化
- 特に大きなSelサイズで効果が顕著

## 制限事項

本実装は簡略化版のため、以下の制限があります：

1. ブリック形状のSelのみサポート（任意形状は通常版を使用）
2. C版のようなコード生成は行わない
3. 63ピクセル超のSelは将来の拡張として検討

## 質問

現時点で質問はありません。

## 参考

- C版ソース: `reference/leptonica/src/morphdwa.c`
- 既存実装: `crates/leptonica-morph/src/binary.rs`
