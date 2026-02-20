//! Sarray, Sarraya - String arrays
//!
//! Arrays of strings for text processing operations.
//! This module provides Leptonica-style convenience methods
//! wrapped around Rust's `Vec<String>`.

pub mod serial;

use crate::error::{Error, Result};
use std::collections::HashSet;

/// Array of strings
///
/// `Sarray` wraps `Vec<String>` and provides Leptonica-style convenience
/// methods for text processing. It supports operations like splitting text
/// into words or lines, joining strings with separators, filtering by
/// substrings, sorting, and set operations.
///
/// # Examples
///
/// ```
/// use leptonica_core::Sarray;
///
/// // Create from words
/// let sa = Sarray::from_words("hello world");
/// assert_eq!(sa.len(), 2);
///
/// // Join with separator
/// let joined = sa.join(" ");
/// assert_eq!(joined, "hello world");
///
/// // Filter by substring
/// let filtered = sa.filter_by_substring("wor");
/// assert_eq!(filtered.len(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sarray {
    data: Vec<String>,
}

impl Sarray {
    // ========================================================================
    // Creation
    // ========================================================================

    /// Create a new empty Sarray
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a Sarray with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Create a Sarray from a vector of strings
    pub fn from_vec(data: Vec<String>) -> Self {
        Self { data }
    }

    /// Create a Sarray from a slice of string slices
    pub fn from_str_slice(data: &[&str]) -> Self {
        Self {
            data: data.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create a Sarray initialized with n copies of a string
    ///
    /// # Arguments
    /// * `n` - Number of strings
    /// * `init_str` - Initial string value
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::initialized(3, "hello");
    /// assert_eq!(sa.len(), 3);
    /// assert_eq!(sa.get(0), Some("hello"));
    /// ```
    pub fn initialized(n: usize, init_str: &str) -> Self {
        Self {
            data: vec![init_str.to_string(); n],
        }
    }

    /// Create a Sarray from a string, splitting on whitespace
    ///
    /// Words are separated by spaces, tabs, or newlines.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_words("hello  world\nfoo\tbar");
    /// assert_eq!(sa.len(), 4);
    /// assert_eq!(sa.get(0), Some("hello"));
    /// assert_eq!(sa.get(1), Some("world"));
    /// ```
    pub fn from_words(text: &str) -> Self {
        Self {
            data: text.split_whitespace().map(|s| s.to_string()).collect(),
        }
    }

    /// Create a Sarray from a string, splitting on newlines
    ///
    /// # Arguments
    /// * `text` - Input text
    /// * `include_blank` - If true, include empty lines; if false, skip them
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_lines("line1\n\nline2", true);
    /// assert_eq!(sa.len(), 3);
    ///
    /// let sa = Sarray::from_lines("line1\n\nline2", false);
    /// assert_eq!(sa.len(), 2);
    /// ```
    pub fn from_lines(text: &str, include_blank: bool) -> Self {
        // Handle both Unix (\n) and Windows (\r\n) line endings
        let lines = text.lines();
        if include_blank {
            // lines() iterator doesn't preserve empty lines at the end,
            // so we need to handle this specially for full compatibility
            let mut result: Vec<String> = Vec::new();
            let mut start = 0;
            for (i, c) in text.char_indices() {
                if c == '\n' {
                    let line = &text[start..i];
                    // Remove trailing \r if present
                    let line = line.strip_suffix('\r').unwrap_or(line);
                    result.push(line.to_string());
                    start = i + 1;
                }
            }
            // Handle last line if no trailing newline
            if start < text.len() {
                let line = &text[start..];
                let line = line.strip_suffix('\r').unwrap_or(line);
                result.push(line.to_string());
            }
            Self { data: result }
        } else {
            Self {
                data: lines
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
            }
        }
    }

    /// Generate a Sarray containing string representations of integers 0..n
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::generate_integers(5);
    /// assert_eq!(sa.len(), 5);
    /// assert_eq!(sa.get(0), Some("0"));
    /// assert_eq!(sa.get(4), Some("4"));
    /// ```
    pub fn generate_integers(n: usize) -> Self {
        Self {
            data: (0..n).map(|i| i.to_string()).collect(),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the number of strings
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a string by index (as &str)
    pub fn get(&self, index: usize) -> Option<&str> {
        self.data.get(index).map(|s| s.as_str())
    }

    /// Get a mutable string reference by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut String> {
        self.data.get_mut(index)
    }

    /// Get a string by index, cloning it
    pub fn get_clone(&self, index: usize) -> Option<String> {
        self.data.get(index).cloned()
    }

    /// Add a string to the end
    pub fn push(&mut self, s: impl Into<String>) {
        self.data.push(s.into());
    }

    /// Remove and return the last string
    pub fn pop(&mut self) -> Option<String> {
        self.data.pop()
    }

    /// Set a string at index
    pub fn set(&mut self, index: usize, s: impl Into<String>) -> Result<()> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        self.data[index] = s.into();
        Ok(())
    }

    /// Insert a string at index
    pub fn insert(&mut self, index: usize, s: impl Into<String>) -> Result<()> {
        if index > self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        self.data.insert(index, s.into());
        Ok(())
    }

    /// Remove a string at index
    pub fn remove(&mut self, index: usize) -> Result<String> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        Ok(self.data.remove(index))
    }

    /// Replace a string at index, returning the old value
    pub fn replace(&mut self, index: usize, s: impl Into<String>) -> Result<String> {
        if index >= self.data.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.data.len(),
            });
        }
        Ok(std::mem::replace(&mut self.data[index], s.into()))
    }

    /// Clear all strings
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the underlying data as a slice
    pub fn as_slice(&self) -> &[String] {
        &self.data
    }

    /// Get the underlying data as a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [String] {
        &mut self.data
    }

    /// Get the underlying vector (consuming self)
    pub fn into_vec(self) -> Vec<String> {
        self.data
    }

    // ========================================================================
    // Join/Concatenate
    // ========================================================================

    /// Join all strings with a separator
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["a", "b", "c"]);
    /// assert_eq!(sa.join(", "), "a, b, c");
    /// ```
    pub fn join(&self, separator: &str) -> String {
        self.data.join(separator)
    }

    /// Join all strings with newlines
    pub fn join_with_newlines(&self) -> String {
        self.join("\n")
    }

    /// Join all strings with spaces
    pub fn join_with_spaces(&self) -> String {
        self.join(" ")
    }

    /// Join all strings with commas
    pub fn join_with_commas(&self) -> String {
        self.join(",")
    }

    /// Join a range of strings with a separator
    ///
    /// # Arguments
    /// * `first` - Starting index
    /// * `count` - Number of strings to join (0 for all remaining)
    /// * `separator` - Separator string
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["a", "b", "c", "d"]);
    /// assert_eq!(sa.join_range(1, 2, "-"), Some("b-c".to_string()));
    /// ```
    pub fn join_range(&self, first: usize, count: usize, separator: &str) -> Option<String> {
        if self.data.is_empty() {
            return if first == 0 {
                Some(String::new())
            } else {
                None
            };
        }

        if first >= self.data.len() {
            return None;
        }

        let end = if count == 0 {
            self.data.len()
        } else {
            (first + count).min(self.data.len())
        };

        Some(self.data[first..end].join(separator))
    }

    /// Concatenate strings uniformly into n groups
    ///
    /// Divides the array into n essentially equal sets of strings,
    /// concatenates each set individually.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["a", "b", "c", "d"]);
    /// let result = sa.concat_uniformly(2, " ");
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result.get(0), Some("a b"));
    /// assert_eq!(result.get(1), Some("c d"));
    /// ```
    pub fn concat_uniformly(&self, n: usize, separator: &str) -> Sarray {
        if n == 0 || n > self.data.len() {
            return self.clone();
        }

        let len = self.data.len();
        let base_size = len / n;
        let extra = len % n;

        let mut result = Sarray::with_capacity(n);
        let mut start = 0;

        for i in 0..n {
            let size = base_size + if i < extra { 1 } else { 0 };
            let end = start + size;
            result.push(self.data[start..end].join(separator));
            start = end;
        }

        result
    }

    // ========================================================================
    // Extend/Append
    // ========================================================================

    /// Extend with strings from another Sarray
    pub fn extend_from(&mut self, other: &Sarray) {
        self.data.extend(other.data.iter().cloned());
    }

    /// Extend with a range of strings from another Sarray
    ///
    /// # Arguments
    /// * `other` - Source Sarray
    /// * `start` - Starting index in other
    /// * `end` - Ending index (exclusive), or None for all remaining
    pub fn extend_from_range(&mut self, other: &Sarray, start: usize, end: Option<usize>) {
        if start >= other.data.len() {
            return;
        }
        let end = end.unwrap_or(other.data.len()).min(other.data.len());
        if start < end {
            self.data.extend(other.data[start..end].iter().cloned());
        }
    }

    /// Pad both arrays to the same size
    ///
    /// If one array is smaller, pad it with copies of pad_str.
    /// Returns the final length of both arrays.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let mut sa1 = Sarray::from_str_slice(&["a", "b"]);
    /// let mut sa2 = Sarray::from_str_slice(&["x"]);
    /// Sarray::pad_to_same_size(&mut sa1, &mut sa2, "");
    /// assert_eq!(sa1.len(), 2);
    /// assert_eq!(sa2.len(), 2);
    /// ```
    pub fn pad_to_same_size(sa1: &mut Sarray, sa2: &mut Sarray, pad_str: &str) -> usize {
        let n1 = sa1.len();
        let n2 = sa2.len();
        let target = n1.max(n2);

        while sa1.len() < target {
            sa1.push(pad_str);
        }
        while sa2.len() < target {
            sa2.push(pad_str);
        }

        target
    }

    // ========================================================================
    // Split/Parse
    // ========================================================================

    /// Split a string on separators and add to this Sarray
    ///
    /// # Arguments
    /// * `text` - String to split
    /// * `separators` - Characters to split on
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let mut sa = Sarray::new();
    /// sa.split_string("a,b;c", &[',', ';']);
    /// assert_eq!(sa.len(), 3);
    /// ```
    pub fn split_string(&mut self, text: &str, separators: &[char]) {
        for part in text.split(|c| separators.contains(&c)) {
            if !part.is_empty() {
                self.push(part);
            }
        }
    }

    /// Convert word array to line array with word wrapping
    ///
    /// Concatenates words into lines, wrapping when a line would
    /// exceed the specified size. Empty strings in the input are
    /// treated as paragraph separators.
    ///
    /// # Arguments
    /// * `line_size` - Maximum characters per line
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let words = Sarray::from_words("hello world foo bar");
    /// let lines = words.words_to_lines(12);
    /// assert!(lines.len() > 0);
    /// ```
    pub fn words_to_lines(&self, line_size: usize) -> Sarray {
        let mut result = Sarray::new();
        let mut current_line = Vec::new();
        let mut current_len = 0;

        for word in &self.data {
            if word.is_empty() {
                // Paragraph separator
                if !current_line.is_empty() {
                    result.push(current_line.join(" "));
                    current_line.clear();
                    current_len = 0;
                }
                result.push("");
            } else if current_len == 0 && word.len() + 1 > line_size {
                // Long word on its own line
                result.push(word.as_str());
            } else if current_len + word.len() + 1 > line_size {
                // Word would exceed line size, start new line
                if !current_line.is_empty() {
                    result.push(current_line.join(" "));
                    current_line.clear();
                }
                current_line.push(word.clone());
                current_len = word.len() + 1;
            } else {
                // Add word to current line
                current_line.push(word.clone());
                current_len += word.len() + 1;
            }
        }

        // Output remaining content
        if !current_line.is_empty() {
            result.push(current_line.join(" "));
        }

        result
    }

    // ========================================================================
    // Filter/Select
    // ========================================================================

    /// Filter strings containing a substring
    ///
    /// Returns a new Sarray with only the strings that contain the substring.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["apple", "banana", "apricot"]);
    /// let filtered = sa.filter_by_substring("ap");
    /// assert_eq!(filtered.len(), 2); // "apple" and "apricot"
    /// ```
    pub fn filter_by_substring(&self, substr: &str) -> Sarray {
        Sarray {
            data: self
                .data
                .iter()
                .filter(|s| s.contains(substr))
                .cloned()
                .collect(),
        }
    }

    /// Select a range of strings
    ///
    /// # Arguments
    /// * `first` - Starting index
    /// * `last` - Ending index (inclusive), or None for all remaining
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["a", "b", "c", "d"]);
    /// let selected = sa.select_range(1, Some(2));
    /// assert_eq!(selected.len(), 2);
    /// assert_eq!(selected.get(0), Some("b"));
    /// ```
    pub fn select_range(&self, first: usize, last: Option<usize>) -> Sarray {
        if self.data.is_empty() || first >= self.data.len() {
            return Sarray::new();
        }

        let last = last.unwrap_or(self.data.len() - 1).min(self.data.len() - 1);
        if first > last {
            return Sarray::new();
        }

        Sarray {
            data: self.data[first..=last].to_vec(),
        }
    }

    // ========================================================================
    // Sort
    // ========================================================================

    /// Sort strings in ascending order
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let mut sa = Sarray::from_str_slice(&["c", "a", "b"]);
    /// sa.sort();
    /// assert_eq!(sa.get(0), Some("a"));
    /// assert_eq!(sa.get(2), Some("c"));
    /// ```
    pub fn sort(&mut self) {
        self.data.sort();
    }

    /// Sort strings in descending order
    pub fn sort_descending(&mut self) {
        self.data.sort_by(|a, b| b.cmp(a));
    }

    /// Sort by a custom comparator
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&String, &String) -> std::cmp::Ordering,
    {
        self.data.sort_by(compare);
    }

    /// Create a sorted copy (ascending)
    pub fn sorted(&self) -> Sarray {
        let mut copy = self.clone();
        copy.sort();
        copy
    }

    /// Create a sorted copy (descending)
    pub fn sorted_descending(&self) -> Sarray {
        let mut copy = self.clone();
        copy.sort_descending();
        copy
    }

    /// Reorder strings according to indices
    ///
    /// Returns a new Sarray where the i-th element is self[indices[i]].
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["a", "b", "c"]);
    /// let indices = [2, 0, 1];
    /// let reordered = sa.reorder_by_indices(&indices);
    /// assert_eq!(reordered.get(0), Some("c"));
    /// assert_eq!(reordered.get(1), Some("a"));
    /// ```
    pub fn reorder_by_indices(&self, indices: &[usize]) -> Sarray {
        Sarray {
            data: indices
                .iter()
                .filter_map(|&i| self.data.get(i).cloned())
                .collect(),
        }
    }

    /// Return a new Sarray sorted according to an index Numa.
    ///
    /// Each value in `naindex` is an index into `self`; the output contains
    /// `self[naindex[0]]`, `self[naindex[1]]`, …
    ///
    /// # Notes
    /// - Negative indices in `naindex` are silently skipped (not inserted
    ///   into the output), matching the C behaviour of an out-of-range
    ///   access returning `NULL`.
    /// - Out-of-bounds (but non-negative) indices are similarly skipped.
    ///
    /// C equivalent: `sarraySortByIndex()` in `sarray2.c`
    pub fn sort_by_index(&self, naindex: &crate::numa::Numa) -> Sarray {
        let n = naindex.len();
        let mut out = Sarray::with_capacity(n);
        for i in 0..n {
            if let Some(idx) = naindex.get_i32(i)
                && idx >= 0
                && let Some(s) = self.data.get(idx as usize)
            {
                out.data.push(s.clone());
            }
        }
        out
    }

    /// Find the next contiguous range of strings in `self` that do **not**
    /// contain `substr` (at optional byte offset `loc`, or anywhere if `None`).
    ///
    /// Returns `Some((actual_start, end, new_start))` where:
    /// - `actual_start`: index of first string in the range
    /// - `end`: index of last string in the range (inclusive)
    /// - `new_start`: index to use for the next call (first past the range)
    ///
    /// Returns `None` if no valid range is found starting at or after `start`.
    ///
    /// C equivalent: `sarrayParseRange()` in `sarray1.c`
    pub fn parse_range(
        &self,
        start: usize,
        substr: &str,
        loc: Option<usize>,
    ) -> Option<(usize, usize, usize)> {
        let n = self.data.len();
        if start >= n {
            return None;
        }

        let matches = |s: &str| -> bool {
            if let Some(offset) = loc {
                // substr must appear at byte position `offset`
                s.len() >= offset + substr.len() && &s[offset..offset + substr.len()] == substr
            } else {
                s.contains(substr)
            }
        };

        // Skip leading strings that DO have the marker
        let actual_start = (start..n).find(|&i| !matches(&self.data[i]))?;

        // Find end: last consecutive string without the marker
        let end = (actual_start + 1..n)
            .take_while(|&i| !matches(&self.data[i]))
            .last()
            .unwrap_or(actual_start);

        let new_start = end + 1;
        Some((actual_start, end, new_start))
    }

    // ========================================================================
    // Set Operations
    // ========================================================================

    /// Remove duplicate strings, preserving order of first occurrence
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let mut sa = Sarray::from_str_slice(&["a", "b", "a", "c", "b"]);
    /// sa.remove_duplicates();
    /// assert_eq!(sa.len(), 3);
    /// assert_eq!(sa.get(0), Some("a"));
    /// assert_eq!(sa.get(1), Some("b"));
    /// assert_eq!(sa.get(2), Some("c"));
    /// ```
    pub fn remove_duplicates(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|s| seen.insert(s.clone()));
    }

    /// Create a copy with duplicates removed
    pub fn unique(&self) -> Sarray {
        let mut copy = self.clone();
        copy.remove_duplicates();
        copy
    }

    /// Compute the union with another Sarray
    ///
    /// Returns a new Sarray containing all unique strings from both arrays.
    pub fn union(&self, other: &Sarray) -> Sarray {
        let mut result = self.clone();
        result.extend_from(other);
        result.remove_duplicates();
        result
    }

    /// Compute the intersection with another Sarray
    ///
    /// Returns a new Sarray containing strings present in both arrays.
    pub fn intersection(&self, other: &Sarray) -> Sarray {
        let other_set: HashSet<&str> = other.data.iter().map(|s| s.as_str()).collect();
        let mut seen = HashSet::new();

        Sarray {
            data: self
                .data
                .iter()
                .filter(|s| other_set.contains(s.as_str()) && seen.insert(s.as_str()))
                .cloned()
                .collect(),
        }
    }

    /// Compute the difference (self - other)
    ///
    /// Returns strings that are in self but not in other.
    pub fn difference(&self, other: &Sarray) -> Sarray {
        let other_set: HashSet<&str> = other.data.iter().map(|s| s.as_str()).collect();
        let mut seen = HashSet::new();

        Sarray {
            data: self
                .data
                .iter()
                .filter(|s| !other_set.contains(s.as_str()) && seen.insert(s.as_str()))
                .cloned()
                .collect(),
        }
    }

    // ========================================================================
    // Utility
    // ========================================================================

    /// Check if the array contains a string
    pub fn contains(&self, s: &str) -> bool {
        self.data.iter().any(|item| item == s)
    }

    /// Find the first index of a string
    pub fn find(&self, s: &str) -> Option<usize> {
        self.data.iter().position(|item| item == s)
    }

    /// Find all indices of a string
    pub fn find_all(&self, s: &str) -> Vec<usize> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, item)| if item == s { Some(i) } else { None })
            .collect()
    }

    /// Lookup a comma-separated key-value pair
    ///
    /// Searches for strings in the format "key,value" and returns
    /// the value for the matching key.
    ///
    /// # Examples
    ///
    /// ```
    /// use leptonica_core::Sarray;
    ///
    /// let sa = Sarray::from_str_slice(&["name,John", "age,30", "city,NY"]);
    /// assert_eq!(sa.lookup_csv_kv("age"), Some("30".to_string()));
    /// assert_eq!(sa.lookup_csv_kv("unknown"), None);
    /// ```
    pub fn lookup_csv_kv(&self, key: &str) -> Option<String> {
        for s in &self.data {
            if let Some((k, v)) = s.split_once(',')
                && k == key
            {
                return Some(v.to_string());
            }
        }
        None
    }

    /// Reverse the order of strings in place
    pub fn reverse(&mut self) {
        self.data.reverse();
    }

    /// Create a reversed copy
    pub fn reversed(&self) -> Sarray {
        let mut copy = self.clone();
        copy.reverse();
        copy
    }

    // ========================================================================
    // Iterator
    // ========================================================================

    /// Create an iterator over string references
    pub fn iter(&self) -> SarrayIter<'_> {
        SarrayIter {
            inner: self.data.iter(),
        }
    }

    /// Create a mutable iterator over strings
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut String> {
        self.data.iter_mut()
    }
}

// ============================================================================
// Iterator
// ============================================================================

/// Iterator over Sarray strings (as &str)
pub struct SarrayIter<'a> {
    inner: std::slice::Iter<'a, String>,
}

impl<'a> Iterator for SarrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| s.as_str())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for SarrayIter<'_> {}

impl<'a> IntoIterator for &'a Sarray {
    type Item = &'a str;
    type IntoIter = SarrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Sarray {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl FromIterator<String> for Sarray {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            data: iter.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<&'a str> for Sarray {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self {
            data: iter.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Extend<String> for Sarray {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.data.extend(iter);
    }
}

impl<'a> Extend<&'a str> for Sarray {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        self.data.extend(iter.into_iter().map(|s| s.to_string()));
    }
}

impl std::ops::Index<usize> for Sarray {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for Sarray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

// ============================================================================
// Sarraya - Array of Sarray
// ============================================================================

/// Array of Sarray
///
/// `Sarraya` manages a collection of `Sarray` arrays, useful for storing
/// related sets of string data.
#[derive(Debug, Clone, Default)]
pub struct Sarraya {
    sarrays: Vec<Sarray>,
}

impl Sarraya {
    /// Create a new empty Sarraya
    pub fn new() -> Self {
        Self {
            sarrays: Vec::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sarrays: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of Sarray
    #[inline]
    pub fn len(&self) -> usize {
        self.sarrays.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sarrays.is_empty()
    }

    /// Get a Sarray by index
    pub fn get(&self, index: usize) -> Option<&Sarray> {
        self.sarrays.get(index)
    }

    /// Get a mutable Sarray by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Sarray> {
        self.sarrays.get_mut(index)
    }

    /// Add a Sarray
    pub fn push(&mut self, sarray: Sarray) {
        self.sarrays.push(sarray);
    }

    /// Remove and return the last Sarray
    pub fn pop(&mut self) -> Option<Sarray> {
        self.sarrays.pop()
    }

    /// Replace a Sarray at index
    pub fn replace(&mut self, index: usize, sarray: Sarray) -> Result<Sarray> {
        if index >= self.sarrays.len() {
            return Err(Error::IndexOutOfBounds {
                index,
                len: self.sarrays.len(),
            });
        }
        Ok(std::mem::replace(&mut self.sarrays[index], sarray))
    }

    /// Clear all Sarray
    pub fn clear(&mut self) {
        self.sarrays.clear();
    }

    /// Get all Sarray as a slice
    pub fn as_slice(&self) -> &[Sarray] {
        &self.sarrays
    }

    /// Get the total number of strings across all Sarray
    pub fn total_count(&self) -> usize {
        self.sarrays.iter().map(|s| s.len()).sum()
    }

    /// Get a string from a specific Sarray
    pub fn get_string(&self, sarray_index: usize, string_index: usize) -> Option<&str> {
        self.sarrays.get(sarray_index)?.get(string_index)
    }

    /// Add a string to a specific Sarray
    pub fn add_string(&mut self, sarray_index: usize, s: impl Into<String>) -> Result<()> {
        let len = self.sarrays.len();
        let sarray = self
            .sarrays
            .get_mut(sarray_index)
            .ok_or(Error::IndexOutOfBounds {
                index: sarray_index,
                len,
            })?;
        sarray.push(s);
        Ok(())
    }

    /// Flatten into a single Sarray
    pub fn flatten(&self) -> Sarray {
        let total = self.total_count();
        let mut result = Sarray::with_capacity(total);
        for sarray in &self.sarrays {
            result.extend_from(sarray);
        }
        result
    }

    /// Create an iterator over Sarray
    pub fn iter(&self) -> impl Iterator<Item = &Sarray> {
        self.sarrays.iter()
    }

    /// Create a mutable iterator over Sarray
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Sarray> {
        self.sarrays.iter_mut()
    }
}

impl std::ops::Index<usize> for Sarraya {
    type Output = Sarray;

    fn index(&self, index: usize) -> &Self::Output {
        &self.sarrays[index]
    }
}

impl std::ops::IndexMut<usize> for Sarraya {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.sarrays[index]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Sarray creation tests
    // ========================================================================

    #[test]
    fn test_sarray_new() {
        let sa = Sarray::new();
        assert!(sa.is_empty());
        assert_eq!(sa.len(), 0);
    }

    #[test]
    fn test_sarray_with_capacity() {
        let sa = Sarray::with_capacity(100);
        assert!(sa.is_empty());
    }

    #[test]
    fn test_sarray_from_vec() {
        let sa = Sarray::from_vec(vec!["a".into(), "b".into()]);
        assert_eq!(sa.len(), 2);
        assert_eq!(sa.get(0), Some("a"));
    }

    #[test]
    fn test_sarray_from_str_slice() {
        let sa = Sarray::from_str_slice(&["hello", "world"]);
        assert_eq!(sa.len(), 2);
        assert_eq!(sa.get(0), Some("hello"));
    }

    #[test]
    fn test_sarray_initialized() {
        let sa = Sarray::initialized(3, "test");
        assert_eq!(sa.len(), 3);
        for i in 0..3 {
            assert_eq!(sa.get(i), Some("test"));
        }
    }

    #[test]
    fn test_sarray_from_words() {
        let sa = Sarray::from_words("hello  world\nfoo\tbar");
        assert_eq!(sa.len(), 4);
        assert_eq!(sa.get(0), Some("hello"));
        assert_eq!(sa.get(1), Some("world"));
        assert_eq!(sa.get(2), Some("foo"));
        assert_eq!(sa.get(3), Some("bar"));

        let sa = Sarray::from_words("");
        assert!(sa.is_empty());
    }

    #[test]
    fn test_sarray_from_lines() {
        // With blank lines
        let sa = Sarray::from_lines("line1\n\nline2\nline3", true);
        assert_eq!(sa.len(), 4);
        assert_eq!(sa.get(0), Some("line1"));
        assert_eq!(sa.get(1), Some(""));
        assert_eq!(sa.get(2), Some("line2"));
        assert_eq!(sa.get(3), Some("line3"));

        // Without blank lines
        let sa = Sarray::from_lines("line1\n\nline2\nline3", false);
        assert_eq!(sa.len(), 3);

        // Windows line endings
        let sa = Sarray::from_lines("line1\r\nline2\r\n", true);
        assert_eq!(sa.len(), 2);
        assert_eq!(sa.get(0), Some("line1"));
        assert_eq!(sa.get(1), Some("line2"));
    }

    #[test]
    fn test_sarray_generate_integers() {
        let sa = Sarray::generate_integers(5);
        assert_eq!(sa.len(), 5);
        assert_eq!(sa.get(0), Some("0"));
        assert_eq!(sa.get(4), Some("4"));

        let sa = Sarray::generate_integers(0);
        assert!(sa.is_empty());
    }

    // ========================================================================
    // Accessor tests
    // ========================================================================

    #[test]
    fn test_sarray_get_set() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "c"]);

        assert_eq!(sa.get(1), Some("b"));
        assert_eq!(sa.get(10), None);

        sa.set(1, "B").unwrap();
        assert_eq!(sa.get(1), Some("B"));

        assert!(sa.set(10, "X").is_err());
    }

    #[test]
    fn test_sarray_push_pop() {
        let mut sa = Sarray::new();
        sa.push("first");
        sa.push("second".to_string());

        assert_eq!(sa.len(), 2);
        assert_eq!(sa.pop(), Some("second".to_string()));
        assert_eq!(sa.len(), 1);
        assert_eq!(sa.pop(), Some("first".to_string()));
        assert_eq!(sa.pop(), None);
    }

    #[test]
    fn test_sarray_insert_remove() {
        let mut sa = Sarray::from_str_slice(&["a", "c"]);

        sa.insert(1, "b").unwrap();
        assert_eq!(sa.as_slice(), &["a", "b", "c"]);

        let removed = sa.remove(1).unwrap();
        assert_eq!(removed, "b");
        assert_eq!(sa.as_slice(), &["a", "c"]);

        assert!(sa.insert(10, "x").is_err());
        assert!(sa.remove(10).is_err());
    }

    #[test]
    fn test_sarray_replace() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "c"]);

        let old = sa.replace(1, "B").unwrap();
        assert_eq!(old, "b");
        assert_eq!(sa.get(1), Some("B"));

        assert!(sa.replace(10, "x").is_err());
    }

    #[test]
    fn test_sarray_clear() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "c"]);
        sa.clear();
        assert!(sa.is_empty());
    }

    // ========================================================================
    // Join tests
    // ========================================================================

    #[test]
    fn test_sarray_join() {
        let sa = Sarray::from_str_slice(&["a", "b", "c"]);

        assert_eq!(sa.join(", "), "a, b, c");
        assert_eq!(sa.join_with_spaces(), "a b c");
        assert_eq!(sa.join_with_newlines(), "a\nb\nc");
        assert_eq!(sa.join_with_commas(), "a,b,c");

        let empty = Sarray::new();
        assert_eq!(empty.join(", "), "");
    }

    #[test]
    fn test_sarray_join_range() {
        let sa = Sarray::from_str_slice(&["a", "b", "c", "d"]);

        assert_eq!(sa.join_range(1, 2, "-"), Some("b-c".to_string()));
        assert_eq!(sa.join_range(0, 0, "-"), Some("a-b-c-d".to_string())); // 0 means all
        assert_eq!(sa.join_range(2, 10, "-"), Some("c-d".to_string())); // clipped
        assert_eq!(sa.join_range(10, 1, "-"), None); // out of bounds
    }

    #[test]
    fn test_sarray_concat_uniformly() {
        let sa = Sarray::from_str_slice(&["a", "b", "c", "d", "e"]);

        let result = sa.concat_uniformly(2, " ");
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0), Some("a b c"));
        assert_eq!(result.get(1), Some("d e"));

        let result = sa.concat_uniformly(3, "-");
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0), Some("a-b"));
        assert_eq!(result.get(1), Some("c-d"));
        assert_eq!(result.get(2), Some("e"));
    }

    // ========================================================================
    // Extend tests
    // ========================================================================

    #[test]
    fn test_sarray_extend_from() {
        let mut sa1 = Sarray::from_str_slice(&["a", "b"]);
        let sa2 = Sarray::from_str_slice(&["c", "d"]);

        sa1.extend_from(&sa2);
        assert_eq!(sa1.len(), 4);
        assert_eq!(sa1.as_slice(), &["a", "b", "c", "d"]);
    }

    #[test]
    fn test_sarray_extend_from_range() {
        let mut sa1 = Sarray::from_str_slice(&["a"]);
        let sa2 = Sarray::from_str_slice(&["b", "c", "d", "e"]);

        sa1.extend_from_range(&sa2, 1, Some(3));
        assert_eq!(sa1.as_slice(), &["a", "c", "d"]);
    }

    #[test]
    fn test_sarray_pad_to_same_size() {
        let mut sa1 = Sarray::from_str_slice(&["a", "b"]);
        let mut sa2 = Sarray::from_str_slice(&["x"]);

        let len = Sarray::pad_to_same_size(&mut sa1, &mut sa2, "");
        assert_eq!(len, 2);
        assert_eq!(sa1.len(), 2);
        assert_eq!(sa2.len(), 2);
        assert_eq!(sa2.get(1), Some(""));
    }

    // ========================================================================
    // Split tests
    // ========================================================================

    #[test]
    fn test_sarray_split_string() {
        let mut sa = Sarray::new();
        sa.split_string("a,b;c:d", &[',', ';', ':']);
        assert_eq!(sa.len(), 4);
        assert_eq!(sa.as_slice(), &["a", "b", "c", "d"]);

        let mut sa = Sarray::new();
        sa.split_string("hello world", &[' ']);
        assert_eq!(sa.as_slice(), &["hello", "world"]);
    }

    #[test]
    fn test_sarray_words_to_lines() {
        let words = Sarray::from_words("one two three four five six");
        let lines = words.words_to_lines(10);

        assert!(lines.len() > 1);
        for line in lines.iter() {
            // Each line should be <= 10 chars, or a single long word
            let is_single_word = !line.contains(' ');
            assert!(line.len() <= 10 || is_single_word);
        }

        // Test paragraph separator
        let words = Sarray::from_vec(vec![
            "hello".into(),
            "world".into(),
            "".into(),
            "new".into(),
            "paragraph".into(),
        ]);
        let lines = words.words_to_lines(80);
        assert!(lines.contains(""));
    }

    // ========================================================================
    // Filter tests
    // ========================================================================

    #[test]
    fn test_sarray_filter_by_substring() {
        let sa = Sarray::from_str_slice(&["apple", "banana", "apricot", "cherry"]);

        let filtered = sa.filter_by_substring("ap");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains("apple"));
        assert!(filtered.contains("apricot"));

        let filtered = sa.filter_by_substring("xyz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_sarray_select_range() {
        let sa = Sarray::from_str_slice(&["a", "b", "c", "d", "e"]);

        let selected = sa.select_range(1, Some(3));
        assert_eq!(selected.as_slice(), &["b", "c", "d"]);

        let selected = sa.select_range(2, None);
        assert_eq!(selected.as_slice(), &["c", "d", "e"]);

        let selected = sa.select_range(10, None);
        assert!(selected.is_empty());
    }

    // ========================================================================
    // Sort tests
    // ========================================================================

    #[test]
    fn test_sarray_sort() {
        let mut sa = Sarray::from_str_slice(&["cherry", "apple", "banana"]);
        sa.sort();
        assert_eq!(sa.as_slice(), &["apple", "banana", "cherry"]);

        sa.sort_descending();
        assert_eq!(sa.as_slice(), &["cherry", "banana", "apple"]);
    }

    #[test]
    fn test_sarray_sorted() {
        let sa = Sarray::from_str_slice(&["c", "a", "b"]);

        let sorted = sa.sorted();
        assert_eq!(sorted.as_slice(), &["a", "b", "c"]);
        // Original unchanged
        assert_eq!(sa.as_slice(), &["c", "a", "b"]);

        let sorted_desc = sa.sorted_descending();
        assert_eq!(sorted_desc.as_slice(), &["c", "b", "a"]);
    }

    #[test]
    fn test_sarray_reorder_by_indices() {
        let sa = Sarray::from_str_slice(&["a", "b", "c", "d"]);
        let indices = [3, 1, 2, 0];
        let reordered = sa.reorder_by_indices(&indices);
        assert_eq!(reordered.as_slice(), &["d", "b", "c", "a"]);
    }

    // ========================================================================
    // Set operation tests
    // ========================================================================

    #[test]
    fn test_sarray_remove_duplicates() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "a", "c", "b", "a"]);
        sa.remove_duplicates();
        assert_eq!(sa.as_slice(), &["a", "b", "c"]);
    }

    #[test]
    fn test_sarray_unique() {
        let sa = Sarray::from_str_slice(&["a", "b", "a", "c"]);
        let unique = sa.unique();
        assert_eq!(unique.as_slice(), &["a", "b", "c"]);
        // Original unchanged
        assert_eq!(sa.len(), 4);
    }

    #[test]
    fn test_sarray_union() {
        let sa1 = Sarray::from_str_slice(&["a", "b", "c"]);
        let sa2 = Sarray::from_str_slice(&["b", "c", "d"]);

        let union = sa1.union(&sa2);
        assert_eq!(union.len(), 4);
        assert!(union.contains("a"));
        assert!(union.contains("b"));
        assert!(union.contains("c"));
        assert!(union.contains("d"));
    }

    #[test]
    fn test_sarray_intersection() {
        let sa1 = Sarray::from_str_slice(&["a", "b", "c", "d"]);
        let sa2 = Sarray::from_str_slice(&["b", "d", "e"]);

        let intersection = sa1.intersection(&sa2);
        assert_eq!(intersection.len(), 2);
        assert!(intersection.contains("b"));
        assert!(intersection.contains("d"));
    }

    #[test]
    fn test_sarray_difference() {
        let sa1 = Sarray::from_str_slice(&["a", "b", "c", "d"]);
        let sa2 = Sarray::from_str_slice(&["b", "d"]);

        let diff = sa1.difference(&sa2);
        assert_eq!(diff.len(), 2);
        assert!(diff.contains("a"));
        assert!(diff.contains("c"));
    }

    // ========================================================================
    // Utility tests
    // ========================================================================

    #[test]
    fn test_sarray_contains_find() {
        let sa = Sarray::from_str_slice(&["apple", "banana", "apple", "cherry"]);

        assert!(sa.contains("banana"));
        assert!(!sa.contains("grape"));

        assert_eq!(sa.find("apple"), Some(0));
        assert_eq!(sa.find("cherry"), Some(3));
        assert_eq!(sa.find("grape"), None);

        assert_eq!(sa.find_all("apple"), vec![0, 2]);
        assert_eq!(sa.find_all("banana"), vec![1]);
        assert!(sa.find_all("grape").is_empty());
    }

    #[test]
    fn test_sarray_lookup_csv_kv() {
        let sa = Sarray::from_str_slice(&["name,John", "age,30", "city,New York", "invalid"]);

        assert_eq!(sa.lookup_csv_kv("name"), Some("John".to_string()));
        assert_eq!(sa.lookup_csv_kv("age"), Some("30".to_string()));
        assert_eq!(sa.lookup_csv_kv("city"), Some("New York".to_string()));
        assert_eq!(sa.lookup_csv_kv("unknown"), None);
    }

    #[test]
    fn test_sarray_reverse() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "c"]);
        sa.reverse();
        assert_eq!(sa.as_slice(), &["c", "b", "a"]);

        let sa = Sarray::from_str_slice(&["a", "b", "c"]);
        let reversed = sa.reversed();
        assert_eq!(reversed.as_slice(), &["c", "b", "a"]);
        assert_eq!(sa.as_slice(), &["a", "b", "c"]); // original unchanged
    }

    // ========================================================================
    // Iterator tests
    // ========================================================================

    #[test]
    fn test_sarray_iterator() {
        let sa = Sarray::from_str_slice(&["a", "b", "c"]);

        let collected: Vec<&str> = sa.iter().collect();
        assert_eq!(collected, vec!["a", "b", "c"]);

        // Test for loop
        let mut result = String::new();
        for s in &sa {
            result.push_str(s);
        }
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_sarray_into_iterator() {
        let sa = Sarray::from_str_slice(&["a", "b", "c"]);
        let collected: Vec<String> = sa.into_iter().collect();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_sarray_from_iterator() {
        let sa: Sarray = vec!["a", "b", "c"].into_iter().collect();
        assert_eq!(sa.len(), 3);

        let sa: Sarray = vec!["a".to_string(), "b".to_string()].into_iter().collect();
        assert_eq!(sa.len(), 2);
    }

    #[test]
    fn test_sarray_extend() {
        let mut sa = Sarray::from_str_slice(&["a"]);
        sa.extend(vec!["b".to_string(), "c".to_string()]);
        assert_eq!(sa.len(), 3);

        sa.extend(vec!["d", "e"]);
        assert_eq!(sa.len(), 5);
    }

    #[test]
    fn test_sarray_indexing() {
        let mut sa = Sarray::from_str_slice(&["a", "b", "c"]);

        assert_eq!(&sa[0], "a");
        assert_eq!(&sa[1], "b");

        sa[1] = "B".to_string();
        assert_eq!(&sa[1], "B");
    }

    // ========================================================================
    // Sarraya tests
    // ========================================================================

    #[test]
    fn test_sarraya_new() {
        let saa = Sarraya::new();
        assert!(saa.is_empty());
        assert_eq!(saa.len(), 0);
    }

    #[test]
    fn test_sarraya_push_get() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a", "b"]));
        saa.push(Sarray::from_str_slice(&["c"]));

        assert_eq!(saa.len(), 2);
        assert_eq!(saa.get(0).unwrap().len(), 2);
        assert_eq!(saa.get(1).unwrap().len(), 1);
        assert!(saa.get(2).is_none());
    }

    #[test]
    fn test_sarraya_total_count() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a", "b"]));
        saa.push(Sarray::from_str_slice(&["c", "d", "e"]));

        assert_eq!(saa.total_count(), 5);
    }

    #[test]
    fn test_sarraya_get_string() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a", "b"]));
        saa.push(Sarray::from_str_slice(&["c", "d"]));

        assert_eq!(saa.get_string(0, 1), Some("b"));
        assert_eq!(saa.get_string(1, 0), Some("c"));
        assert!(saa.get_string(0, 10).is_none());
        assert!(saa.get_string(10, 0).is_none());
    }

    #[test]
    fn test_sarraya_add_string() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::new());

        saa.add_string(0, "hello").unwrap();
        saa.add_string(0, "world").unwrap();

        assert_eq!(saa[0].len(), 2);
        assert_eq!(saa.get_string(0, 1), Some("world"));

        assert!(saa.add_string(10, "x").is_err());
    }

    #[test]
    fn test_sarraya_flatten() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a", "b"]));
        saa.push(Sarray::from_str_slice(&["c", "d"]));

        let flat = saa.flatten();
        assert_eq!(flat.as_slice(), &["a", "b", "c", "d"]);
    }

    #[test]
    fn test_sarraya_replace() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a"]));

        let old = saa.replace(0, Sarray::from_str_slice(&["b", "c"])).unwrap();
        assert_eq!(old.len(), 1);
        assert_eq!(saa[0].len(), 2);

        assert!(saa.replace(10, Sarray::new()).is_err());
    }

    #[test]
    fn test_sarraya_indexing() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a", "b"]));

        assert_eq!(saa[0].len(), 2);
        assert_eq!(&saa[0][0], "a");

        saa[0][0] = "A".to_string();
        assert_eq!(&saa[0][0], "A");
    }

    #[test]
    fn test_sarraya_pop_clear() {
        let mut saa = Sarraya::new();
        saa.push(Sarray::from_str_slice(&["a"]));
        saa.push(Sarray::from_str_slice(&["b"]));

        let popped = saa.pop().unwrap();
        assert_eq!(&popped[0], "b");
        assert_eq!(saa.len(), 1);

        saa.clear();
        assert!(saa.is_empty());
    }

    // -- Phase 16.5 new functions --

    #[test]
    fn test_sort_by_index() {
        use crate::numa::Numa;
        let sa = Sarray::from_str_slice(&["c", "a", "b"]);
        // index [1, 2, 0] maps new pos → old pos: new[0]=old[1]="a", etc.
        let na = Numa::from_slice(&[1.0, 2.0, 0.0]);
        let sorted = sa.sort_by_index(&na);
        assert_eq!(sorted.get(0), Some("a"));
        assert_eq!(sorted.get(1), Some("b"));
        assert_eq!(sorted.get(2), Some("c"));
    }

    #[test]
    fn test_parse_range() {
        // Lines: "ok1", "--skip", "ok2", "ok3"
        // Range of non-'--' lines starting at 0: should be [0,0], next=1
        let sa = Sarray::from_str_slice(&["ok1", "--skip", "ok2", "ok3"]);
        let range = sa.parse_range(0, "--", None);
        assert_eq!(range, Some((0, 0, 1)));

        // Starting from 1, the '--' line is at 1, so actual start is 2
        let range2 = sa.parse_range(1, "--", None);
        assert_eq!(range2, Some((2, 3, 4)));

        // No valid range when start is past end
        let range3 = sa.parse_range(4, "--", None);
        assert!(range3.is_none());
    }
}
