# Sarray Implementation Plan

## Overview

Sarrayは文字列の動的配列で、Leptonicaにおいてテキスト処理に広く使用されるデータ構造です。
Rustにはすでに`Vec<String>`があるため、Leptonicaスタイルの便利メソッドを追加したラッパーとして設計します。

## 参照C版関数（sarray1.c, sarray2.c）

### sarray1.c - 基本機能

- **Create/Destroy/Copy**
  - `sarrayCreate()` - 作成
  - `sarrayCreateInitialized()` - 初期値で作成
  - `sarrayCreateWordsFromString()` - 文字列から単語配列を作成
  - `sarrayCreateLinesFromString()` - 文字列から行配列を作成
  - `sarrayDestroy()` - 破棄（Rustでは自動Drop）
  - `sarrayCopy()` - コピー（Clone trait）
  - `sarrayClone()` - 参照カウント増加（RustではArc/Rc）

- **Add/Remove**
  - `sarrayAddString()` - 追加
  - `sarrayRemoveString()` - 削除
  - `sarrayReplaceString()` - 置換
  - `sarrayClear()` - クリア

- **Accessors**
  - `sarrayGetCount()` - len()
  - `sarrayGetArray()` - as_slice()
  - `sarrayGetString()` - get()

- **Conversion to string**
  - `sarrayToString()` - 連結
  - `sarrayToStringRange()` - 範囲指定連結

- **Join/Concatenate**
  - `sarrayJoin()` - 結合
  - `sarrayAppendRange()` - 範囲追加
  - `sarrayConcatUniformly()` - 均等分割連結

- **Pad**
  - `sarrayPadToSameSize()` - 同じサイズにパディング

- **Word/Line conversion**
  - `sarrayConvertWordsToLines()` - 単語を行に変換

- **Split**
  - `sarraySplitString()` - 文字列を分割

- **Filter**
  - `sarraySelectBySubstring()` - 部分文字列でフィルタ
  - `sarraySelectRange()` - 範囲でフィルタ
  - `sarrayParseRange()` - 範囲パース

- **I/O**
  - `sarrayRead()` / `sarrayReadStream()` / `sarrayReadMem()` - 読み込み
  - `sarrayWrite()` / `sarrayWriteStream()` / `sarrayWriteMem()` - 書き込み
  - `sarrayAppend()` - ファイル追記

### sarray2.c - ソートと集合演算

- **Sort**
  - `sarraySort()` - ソート
  - `sarraySortByIndex()` - インデックスでソート
  - `stringCompareLexical()` - 辞書順比較

- **Set operations (aset/rbtree)**
  - `sarrayRemoveDupsByAset()` - 重複削除
  - `sarrayUnionByAset()` - 和集合
  - `sarrayIntersectionByAset()` - 積集合

- **Hashmap operations**
  - `sarrayRemoveDupsByHmap()` - 重複削除（高速）
  - `sarrayUnionByHmap()` - 和集合（高速）
  - `sarrayIntersectionByHmap()` - 積集合（高速）

- **Misc**
  - `sarrayGenerateIntegers()` - 整数文字列の配列生成
  - `sarrayLookupCSKV()` - CSV key-value検索

## Rust実装設計

### 構造体定義

```rust
/// Array of strings
///
/// `Sarray` provides a wrapper around `Vec<String>` with Leptonica-style
/// convenience methods for text processing operations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sarray {
    data: Vec<String>,
}
```

### 実装するメソッド

#### 作成・アクセス

- `new()` - 空の配列作成
- `with_capacity(n)` - 容量指定作成
- `from_vec(Vec<String>)` - Vecから作成
- `from_str_slice(&[&str])` - &strスライスから作成
- `from_words(text)` - テキストを単語に分割
- `from_lines(text, include_blank)` - テキストを行に分割
- `initialized(n, init_str)` - 初期値で作成
- `len()`, `is_empty()` - サイズ確認
- `get(index)` - 取得
- `push(s)`, `pop()` - 追加/削除
- `insert(index, s)` - 挿入
- `remove(index)` - 削除
- `replace(index, s)` - 置換
- `clear()` - クリア
- `as_slice()`, `as_slice_mut()` - スライス取得
- `into_vec()` - Vecに変換

#### 連結・変換

- `join(separator)` - 区切り文字で連結
- `join_with_newlines()` - 改行で連結
- `join_with_spaces()` - スペースで連結
- `join_range(first, nstrings, separator)` - 範囲指定連結
- `concat_uniformly(n, separator)` - 均等分割連結

#### 結合・フィルタリング

- `extend_from(&other)` - 他のSarrayを追加
- `extend_from_range(&other, start, end)` - 範囲指定追加
- `pad_to_same_size(other, pad_str)` - 同サイズにパディング
- `filter_by_substring(substr)` - 部分文字列でフィルタ
- `select_range(first, last)` - 範囲選択

#### 変換

- `split_string(text, separators)` - 文字列を分割して追加
- `words_to_lines(line_size)` - 単語を行に変換（折り返し）

#### ソート

- `sort()` - 昇順ソート
- `sort_descending()` - 降順ソート
- `sort_by_indices(indices)` - インデックス配列でソート

#### 集合演算

- `remove_duplicates()` - 重複削除
- `union(&other)` - 和集合
- `intersection(&other)` - 積集合

#### ユーティリティ

- `generate_integers(n)` - "0"..."n-1"の配列生成
- `lookup_csv_kv(key)` - CSV形式key-value検索
- `contains(&str)` - 含有チェック
- `find(&str)` - 検索

#### I/O（オプション - 基本実装では含めない）

- `read_from_file(path)` - ファイル読み込み
- `write_to_file(path)` - ファイル書き込み

### Sarrayaの実装

```rust
/// Array of Sarray
#[derive(Debug, Clone, Default)]
pub struct Sarraya {
    sarrays: Vec<Sarray>,
}
```

基本的なNumaaパターンに従う。

### イテレータ・トレイト実装

- `Iterator`, `IntoIterator`
- `FromIterator<String>`, `FromIterator<&str>`
- `Index<usize>`, `IndexMut<usize>`
- `Extend<String>`, `Extend<&str>`

## 実装ステップ

### Phase 1: 基本構造（Core）

1. `Sarray` 構造体定義
2. 作成・アクセスメソッド
3. Iterator/トレイト実装
4. 基本テスト

### Phase 2: テキスト処理

1. `from_words()`, `from_lines()`
2. `join()` 系メソッド
3. `split_string()`
4. `words_to_lines()`

### Phase 3: フィルタリング・集合演算

1. `filter_by_substring()`
2. `select_range()`
3. `remove_duplicates()`
4. `union()`, `intersection()`

### Phase 4: ソート

1. `sort()`, `sort_descending()`
2. `sort_by_indices()`

### Phase 5: Sarraya

1. 基本構造
2. アクセサメソッド
3. イテレータ

## ファイル構成

```text
crates/leptonica-core/src/
├── sarray/
│   └── mod.rs        # Sarray, Sarraya実装
└── lib.rs            # pub mod sarray追加
```

## 質問

なし。設計は十分に明確であり、既存のNumaパターンに従って実装を進めます。

## テスト計画

- 基本的なCRUD操作のテスト
- `from_words()`, `from_lines()` のパースロジック
- `join()` 系の連結ロジック
- ソートの昇順/降順
- 集合演算（重複削除、和集合、積集合）
- 境界条件（空配列、大きなインデックス等）
- イテレータ動作

## 進捗

- [x] Phase 1: 基本構造
- [x] Phase 2: テキスト処理
- [x] Phase 3: フィルタリング・集合演算
- [x] Phase 4: ソート
- [x] Phase 5: Sarraya
- [x] テスト完了（48テスト + 18 doctest）
- [x] fmt/clippy
- [ ] コミット
