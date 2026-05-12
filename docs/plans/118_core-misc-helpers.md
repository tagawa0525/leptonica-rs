# Core: misc 単独関数 2 件 (plan 032 カテゴリ M の一部)

Status: PLANNED
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

- `splitStringToParagraphs(textstr, splitflag)` — テキストを
  段落 Sarray に分割

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

- [ ] cargo test/clippy/fmt 通過
- [ ] misc.md / core.md の対応エントリ更新
- [ ] plan 032 で 118 を IMPLEMENTED に分割反映
- [ ] PR + Copilot レビュー対応 + マージ
