# Pixa 実装計画

## 概要

Pixa（Pix配列）構造体と基本操作を leptonica-core に実装する。

## 参照

- C版ソース: `reference/leptonica/src/pixabasic.c`
- 実装先: `crates/leptonica-core/src/pixa/mod.rs`
- 既存パターン参照: `Boxa` (`box_/mod.rs`), `Ptaa` (`pta/mod.rs`)

## 設計方針

### Rust化の方針

1. **参照カウント**: C版の `refcount` は `Arc<T>` パターンを使用しない
   - `Pix` 自体が `Arc<PixData>` を使用しているため、`Pixa` は単純な `Vec<Pix>` で十分
   - `clone()` は shallow copy（Pix の Arc をクローン）
   - `deep_clone()` で全 Pix をディープコピー

2. **Box配列**: C版は `Pixa` 内に `Boxa` を持つ
   - Rust版でも同様に `Option<Boxa>` または `Boxa` を保持
   - Box は各 Pix の位置情報（オプション）

3. **コピーモード**: C版の `L_INSERT`, `L_COPY`, `L_CLONE`
   - `L_INSERT` → move semantics（所有権移動）
   - `L_COPY` → `deep_clone()` / `clone()` を明示的に呼ぶ
   - `L_CLONE` → `Arc::clone()` による共有（Pix の標準 clone）

### 構造体設計

```rust
/// Array of Pix images
#[derive(Debug, Clone, Default)]
pub struct Pixa {
    /// The Pix images
    pix: Vec<Pix>,
    /// Optional bounding boxes for each Pix
    boxa: Boxa,
}

/// Array of Pixa
#[derive(Debug, Clone, Default)]
pub struct Pixaa {
    pixas: Vec<Pixa>,
}
```

## 実装する機能

### 基本操作

| C関数 | Rust メソッド | 説明 |
| --- | --- | --- |
| `pixaCreate` | `Pixa::new()`, `Pixa::with_capacity()` | 作成 |
| `pixaDestroy` | Drop trait (自動) | 破棄 |
| `pixaCopy` | `clone()`, `deep_clone()` | コピー |

### 追加・取得

| C関数 | Rust メソッド | 説明 |
| --- | --- | --- |
| `pixaAddPix` | `push()` | Pix を追加 |
| `pixaAddBox` | `push_with_box()` | Pix と Box を追加 |
| `pixaGetPix` | `get()`, `get_cloned()` | Pix を取得 |
| `pixaGetCount` | `len()` | 要素数 |
| `pixaGetBox` | `get_box()` | Box を取得 |
| `pixaGetBoxa` | `boxa()` | Boxa 参照を取得 |

### 配列操作

| C関数 | Rust メソッド | 説明 |
| --- | --- | --- |
| `pixaReplacePix` | `replace()` | 置換 |
| `pixaInsertPix` | `insert()` | 挿入 |
| `pixaRemovePix` | `remove()` | 削除 |
| `pixaClear` | `clear()` | クリア |

### イテレーション

| Rust trait | 説明 |
| --- | --- |
| `Iterator` | `iter()`, `iter_mut()` |
| `IntoIterator` | `for pix in &pixa` |
| `FromIterator` | `pixa.into_iter().collect()` |

### ユーティリティ

| C関数 | Rust メソッド | 説明 |
| --- | --- | --- |
| `pixaVerifyDepth` | `verify_depth()` | 深度の一貫性チェック |
| `pixaGetPixDimensions` | `get_dimensions()` | 寸法取得 |

## 実装ファイル構成

```text
crates/leptonica-core/src/
├── pixa/
│   └── mod.rs       # Pixa, Pixaa 実装
├── lib.rs           # pub mod pixa; pub use pixa::*;
└── ...
```

## テスト計画

1. **基本作成テスト**: `new()`, `with_capacity()`
2. **追加・取得テスト**: `push()`, `get()`, `len()`
3. **イテレーションテスト**: `iter()`, `for` loop
4. **Box連携テスト**: `push_with_box()`, `get_box()`
5. **コピーテスト**: `clone()` vs `deep_clone()`
6. **境界テスト**: 空配列、インデックス範囲外

## 実装手順

1. [x] `crates/leptonica-core/src/pixa/mod.rs` 作成
2. [x] `Pixa` 構造体と基本メソッド実装
3. [x] `Pixaa` 構造体実装
4. [x] Iterator traits 実装
5. [x] `lib.rs` にモジュール追加
6. [x] 単体テスト作成
7. [x] `cargo fmt && cargo clippy` 実行
8. [x] テスト実行

## 質問

（なし）
