# Numa Implementation Plan

## Overview

Numa（数値配列）構造体と基本操作を leptonica-core クレートに実装する。

## 参照情報

- C版ソース: `reference/leptonica/src/numabasic.c`, `numafunc1.c`
- 既存パターン: `Pta`, `Pixa` の実装を参考

## C版 NUMA の特徴

- `l_float32` (f32) の配列を管理
- `startx`, `delx` パラメータ（ヒストグラム等で使用）
- 参照カウントによるクローン

## Rust実装方針

### 1. 構造体設計

```rust
#[derive(Debug, Clone, Default)]
pub struct Numa {
    data: Vec<f32>,
    startx: f32,  // default: 0.0
    delx: f32,    // default: 1.0
}
```

**設計判断:**

- `Vec<f32>` を使用（Rustの動的配列、自動メモリ管理）
- 参照カウントは不要（Rustの所有権システムで管理、`Clone`で明示的コピー）
- C版の `nalloc`, `n`, `refcount` フィールドは不要

### 2. 実装する機能

#### 基本操作（numabasic.c相当）

- `numaCreate` → `Numa::new()`, `Numa::with_capacity()`
- `numaDestroy` → (Drop trait自動)
- `numaCopy` → Clone trait
- `numaEmpty` → `clear()`
- `numaAddNumber` → `push()`
- `numaGetCount` → `len()`
- `numaGetFValue` → `get()`
- `numaGetIValue` → `get_i32()`
- `numaSetValue` → `set()`
- `numaInsertNumber` → `insert()`
- `numaRemoveNumber` → `remove()`
- `numaGetParameters` → `parameters()`
- `numaSetParameters` → `set_parameters()`

#### 統計操作（numafunc1.c相当）

- `numaGetMin` → `min()` (最小値と位置)
- `numaGetMax` → `max()` (最大値と位置)
- `numaGetSum` → `sum()` (合計)
- `numaGetMean` → `mean()` (平均)

#### Rust固有の機能

- `Iterator` trait 実装
- `IntoIterator` trait 実装
- `FromIterator` trait 実装
- `Index` trait 実装
- スライスアクセス `as_slice()`, `as_slice_mut()`

### 3. Numaa（配列の配列）

```rust
#[derive(Debug, Clone, Default)]
pub struct Numaa {
    numas: Vec<Numa>,
}
```

### 4. エラーハンドリング

- 既存の `Error` enum を使用
- `IndexOutOfBounds` - 範囲外アクセス
- `NullInput` - 空配列に対する操作

### 5. テスト計画

- 基本操作（作成、追加、取得、設定）
- 統計関数
- イテレーター
- パラメーター（startx, delx）
- Numaa

## ファイル構成

```text
crates/leptonica-core/src/
├── numa/
│   └── mod.rs    # Numa, Numaa 実装
└── lib.rs        # pub mod numa; を追加
```

## 実装順序

1. `Numa` 構造体と基本操作
2. 統計関数
3. Iterator 実装
4. `Numaa` 実装
5. テスト
6. lib.rs への export 追加

## 質問

（なし）
