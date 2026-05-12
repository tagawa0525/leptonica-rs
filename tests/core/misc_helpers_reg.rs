//! Regression tests for plan 118 (misc single helpers).

use leptonica::core::sarray::{ParagraphSplit, split_string_to_paragraphs, string_compare_lexical};

// -- string_compare_lexical --------------------------------------------

#[test]
fn string_compare_lexical_equal_returns_zero() {
    assert_eq!(string_compare_lexical("abc", "abc"), 0);
}

#[test]
fn string_compare_lexical_shorter_returns_zero() {
    // "ab" < "abc" -> 0 (because str1 not greater)
    assert_eq!(string_compare_lexical("ab", "abc"), 0);
}

#[test]
fn string_compare_lexical_longer_returns_one() {
    // "abcd" > "abc" -> 1
    assert_eq!(string_compare_lexical("abcd", "abc"), 1);
}

#[test]
fn string_compare_lexical_first_char_greater() {
    // 'b' > 'a' so "banana" > "apple" -> 1
    assert_eq!(string_compare_lexical("banana", "apple"), 1);
}

#[test]
fn string_compare_lexical_first_char_smaller() {
    assert_eq!(string_compare_lexical("apple", "banana"), 0);
}

#[test]
fn string_compare_lexical_case_sensitivity() {
    // 'A' = 0x41, 'a' = 0x61, so "a" > "A" -> 1
    assert_eq!(string_compare_lexical("a", "A"), 1);
    assert_eq!(string_compare_lexical("A", "a"), 0);
}

// -- split_string_to_paragraphs ---------------------------------------

#[test]
fn split_paragraphs_single_paragraph() {
    let out = split_string_to_paragraphs("one\ntwo\nthree", ParagraphSplit::OnBlankLine);
    assert_eq!(out.len(), 1);
    assert!(out.get(0).unwrap().contains("one"));
    assert!(out.get(0).unwrap().contains("three"));
}

#[test]
fn split_paragraphs_on_blank_line() {
    let txt = "p1 line1\np1 line2\n\np2 line1\np2 line2";
    let out = split_string_to_paragraphs(txt, ParagraphSplit::OnBlankLine);
    assert_eq!(out.len(), 2);
    assert!(out.get(0).unwrap().contains("p1 line1"));
    assert!(out.get(1).unwrap().contains("p2 line1"));
}

#[test]
fn split_paragraphs_on_leading_white() {
    let txt = "p1 first\np1 second\n  indented start of p2\np2 cont";
    let out = split_string_to_paragraphs(txt, ParagraphSplit::OnLeadingWhite);
    assert_eq!(out.len(), 2);
    assert!(out.get(1).unwrap().contains("indented start"));
}

#[test]
fn split_paragraphs_on_both() {
    let txt = "p1\n  indent\n\np3 after blank";
    let out = split_string_to_paragraphs(txt, ParagraphSplit::OnBoth);
    assert_eq!(out.len(), 3);
}
