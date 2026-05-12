//! Misc string/sarray helpers (plan 118 / C sarray2.c, textops.c).
//!
//! Covered:
//!
//! - `stringCompareLexical` -> [`string_compare_lexical`]
//! - `splitStringToParagraphs` -> [`split_string_to_paragraphs`]

use crate::core::sarray::Sarray;

/// How [`split_string_to_paragraphs`] decides where one paragraph ends
/// and the next begins.
///
/// C Leptonica constants: `SPLIT_ON_LEADING_WHITE`, `SPLIT_ON_BLANK_LINE`,
/// `SPLIT_ON_BOTH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphSplit {
    /// Split when a line starts with whitespace (`SPLIT_ON_LEADING_WHITE`).
    OnLeadingWhite,
    /// Split when a line is entirely whitespace (`SPLIT_ON_BLANK_LINE`).
    OnBlankLine,
    /// Split on either condition (`SPLIT_ON_BOTH`).
    OnBoth,
}

/// Lexical comparison of two strings.
///
/// Returns `1` when `str1 > str2`, `0` otherwise. The C original returns
/// `1` for "greater" and `0` for "less or equal", which is preserved
/// here for API parity. Note this is not a three-valued
/// `Ordering`-style result.
///
/// C Leptonica equivalent: `stringCompareLexical`.
pub fn string_compare_lexical(str1: &str, str2: &str) -> i32 {
    let b1 = str1.as_bytes();
    let b2 = str2.as_bytes();
    let n = b1.len().min(b2.len());
    for i in 0..n {
        if b1[i] == b2[i] {
            continue;
        }
        return if b1[i] > b2[i] { 1 } else { 0 };
    }
    if b1.len() > b2.len() { 1 } else { 0 }
}

/// Split `textstr` into paragraph strings, using `split` to choose
/// the paragraph boundary.
///
/// Each entry in the returned Sarray is the joined text of one
/// paragraph (lines re-joined with `\n` separators, mirroring C's
/// `sarrayToString(..., 1)` add-newline behaviour).
///
/// C Leptonica equivalent: `splitStringToParagraphs`.
pub fn split_string_to_paragraphs(textstr: &str, split: ParagraphSplit) -> Sarray {
    let lines = Sarray::from_lines(textstr, true);
    let mut out = Sarray::new();
    let mut current = Sarray::new();

    let push_current_to_out = |current: &mut Sarray, out: &mut Sarray| {
        if current.is_empty() {
            return;
        }
        // Re-join with \n and a trailing \n (matching C sarrayToString
        // with addnlflag=1 which appends \n after every line including
        // the last).
        let mut joined = current.join_with_newlines();
        joined.push('\n');
        out.push(joined);
        current.clear();
    };

    for i in 0..lines.len() {
        let line = lines.get(i).unwrap_or("");
        let all_white = line.chars().all(|c| c.is_whitespace());
        let lead_white = line.chars().next().is_some_and(|c| c.is_whitespace());

        // The first line is always appended; only subsequent lines may
        // trigger a paragraph break.
        if i > 0 {
            let trigger = match split {
                ParagraphSplit::OnLeadingWhite => lead_white,
                ParagraphSplit::OnBlankLine => all_white,
                ParagraphSplit::OnBoth => all_white || lead_white,
            };
            if trigger {
                push_current_to_out(&mut current, &mut out);
            }
        }
        current.push(line.to_string());
    }
    push_current_to_out(&mut current, &mut out);
    out
}
