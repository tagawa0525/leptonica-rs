# Core: misc 単独関数 2 件 (plan 032 カテゴリ M の一部)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数 (2)

軽量・独立性の高い 2 関数。残り 4 関数 (`pixaSaveFont`,
`rotateorthFilesToPdf`, `pixStrokeWidthTransform`,
`pixAddSingleTextblock`) はそれぞれ依存 (BMF / PDF /
runlength morphology / textops layout) が大きいため別 plan で扱う。

### sarray2.c

- `stringCompareLexical(str1, str2)` — 単純な辞書順比較

### textops.c

- `splitStringToParagraphs(textstr, splitflag)` — テキストを段落 Sarray に分割

## API 設計

```rust
/// C: `stringCompareLexical`
/// Returns 1 if `str1` > `str2`, 0 otherwise (matches C return).
pub fn string_compare_lexical(str1: &str, str2: &str) -> i32;

/// C: `splitStringToParagraphs`
pub enum ParagraphSplit {
    /// `SPLIT_ON_LEADING_WHITE`
    OnLeadingWhite,
    /// `SPLIT_ON_BLANK_LINE`
    OnBlankLine,
    /// `SPLIT_ON_BOTH`
    OnBoth,
}

pub fn split_string_to_paragraphs(textstr: &str, split: ParagraphSplit) -> Sarray;
```

## 依存

- 既存 `Sarray::from_lines`, `Sarray::push`, `Sarray::join_with_newlines`

## テスト方針

- string_compare_lexical: 同一 / 短い-長い / 大文字-小文字 / 単一文字差
- split_string_to_paragraphs:
  - 単一段落 (改行のみ)
  - SPLIT_ON_BLANK_LINE: 空行で分割
  - SPLIT_ON_LEADING_WHITE: インデント開始で分割
  - SPLIT_ON_BOTH: 両条件で分割

## 完了条件

- [x] cargo test/clippy/fmt 通過 (10 件パス)
- [x] misc.md の対応エントリ更新
- [x] plan 032 で 118 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `string_compare_lexical`: byte 比較で C 完全互換 (返り値 0/1)。 内部は `as_bytes()` の素朴ループ
- `split_string_to_paragraphs`:
  - `Sarray::from_lines(_, true)` で空行を含めて行リスト化 (改行記号自体は除去される)
  - 各行が `ParagraphSplit` の trigger 条件を満たすたびに paragraph を flush
  - C と異なり、空行 trigger 時はその空行を separator として consume し、次の段落の先頭には含めない (連続空行や先頭空行で空段落を生まないため)
  - C `isspace()` 互換のため `is_ascii_whitespace` を使用 (U+3000 等は対象外)
  - flush 時に各 paragraph 文字列の末尾に `\n` を付与 (C `sarrayToString(..., 1)` と一致)
