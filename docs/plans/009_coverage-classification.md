# 未実装関数の分類と文書更新

**Status: IMPLEMENTED**

## 概要

C版leptonicaの全public関数のうち、現在❌未実装とマークされている752関数を「🚫不要」と「❌未実装」に再分類し、`docs/porting/comparison/` の各文書を更新する。

### 背景

現在のcomparison文書では❌未実装の中に以下が混在している:
- Rust標準ライブラリで代替可能なもの（データ構造等）
- C版固有の設計パターン（手動メモリ管理、グローバル変数設定等）
- デバッグ/表示専用関数
- 低レベル内部関数（高レベルAPIでカバー済み）
- 実際に機能として価値がある未実装関数

これを区別することで、真に実装すべき関数を明確にし、カバレッジ向上の優先度付けを可能にする。

## 分類基準

### 🚫不要（Unnecessary）

以下に該当する関数は「🚫不要」とする:

1. **Rust標準ライブラリで代替可能**: heap→BinaryHeap, list→LinkedList, stack→Vec, queue→VecDeque, ptra→Vec<Option<T>>, dna→Vec<f64>
2. **C版固有の設計パターン**:
   - 手動メモリ管理（destroy, freeData, extendArray等）
   - 可変フィールド設定（setWidth/setHeight/setDepth等 - Pix不変設計）
   - グローバル変数設定（resetMorphBoundaryCondition等）
3. **デバッグ/表示専用関数**: pixDisplay, recogShowContent, pixPrintStreamInfo, dewarpaShowArrays等
4. **DWAコード生成**: fmorphautogen系（Rustでは手書き実装で不要）
5. **画像処理に直接関係しない機能**: sudoku等
6. **低レベル内部関数**: roplow全体, seedfillGrayLow等（高レベルAPIでカバー済み）

### ❌未実装（Not Yet Implemented）

上記に該当せず、実際に機能として価値がある関数。

## Phase 1: 分類と文書更新

各モジュールのcomparison文書を更新し、❌を🚫と❌に再分類する。

### 対象モジュール（9ファイル）

| # | ファイル | 現在の❌数 | 作業内容 |
|---|---------|-----------|---------|
| 1 | comparison/core.md | 337 | 最大。I/O補助、colormap、roplow等の分類 |
| 2 | comparison/io.md | 61 | PDF/PS高レベル、Display機能の分類 |
| 3 | comparison/transform.md | 61 | PTA/BOXA変換、特殊スケーリングの分類 |
| 4 | comparison/morph.md | 22 | selgen、DWAコード生成の分類 |
| 5 | comparison/filter.md | 17 | adaptmap詳細、エッジ測定の分類 |
| 6 | comparison/color.md | 58 | FPIXA依存、量子化、二値化の分類 |
| 7 | comparison/region.md | 47 | CCBORDA、seedfill低レベルの分類 |
| 8 | comparison/recog.md | 45 | デバッグ/可視化の分類 |
| 9 | comparison/misc.md | 104 | データ構造、pixcomp、sudokuの分類 |

### 文書フォーマット変更

各comparison文書のテーブルに新しい状態を追加:

**変更前:**
```
| C関数 | 状態 | Rust対応 | 備考 |
| pixFoo | ❌ 未実装 | - | 説明 |
```

**変更後:**
```
| C関数 | 状態 | Rust対応 | 備考 |
| pixFoo | ❌ 未実装 | - | 説明 |
| pixBar | 🚫 不要 | - | Rust標準ライブラリで代替可能 |
```

### サマリーテーブル更新

`feature-comparison.md` のサマリーテーブルを4列に拡張:

```
| クレート | ✅ 同等 | 🔄 異なる | ❌ 未実装 | 🚫 不要 | 合計 | 実カバレッジ |
```

**実カバレッジ** = (✅ + 🔄) / (合計 - 🚫) で、不要を除いた実質的なカバレッジを算出。

## 作業手順

1. モジュールごとにサブエージェントを起動
2. 各サブエージェントが該当comparison文書の全❌関数を分析
3. C版ソース（`reference/leptonica/src/`）を参照して分類判断
4. comparison文書を更新（🚫/❌の再分類）
5. サマリーの数値を更新
6. 最後にfeature-comparison.mdのサマリーテーブルを更新

## C版ソース参照

- `reference/leptonica/src/` - C版ソースファイル
- `reference/leptonica/src/allheaders.h` - 全public関数宣言

## 成果物

1. 更新された `docs/porting/comparison/*.md` （9ファイル）
2. 更新された `docs/porting/feature-comparison.md`
3. 未実装関数の優先度リスト（Phase 2実装計画の入力として使用）

## Phase 2（将来）: 実装

Phase 1完了後、❌未実装として残った関数の実装計画を別途作成する。
