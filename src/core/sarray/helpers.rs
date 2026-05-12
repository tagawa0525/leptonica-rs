//! Misc string/sarray helpers — RED stubs (plan 118).

use crate::core::sarray::Sarray;

/// How [`split_string_to_paragraphs`] decides where one paragraph ends
/// and the next begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphSplit {
    OnLeadingWhite,
    OnBlankLine,
    OnBoth,
}

/// C: `stringCompareLexical`.
pub fn string_compare_lexical(_str1: &str, _str2: &str) -> i32 {
    unimplemented!("plan 118 RED stub")
}

/// C: `splitStringToParagraphs`.
pub fn split_string_to_paragraphs(_textstr: &str, _split: ParagraphSplit) -> Sarray {
    unimplemented!("plan 118 RED stub")
}
