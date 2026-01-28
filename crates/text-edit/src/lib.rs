use serde::Deserialize;
use serde::Serialize;

/// Represents the safety of applying a specific edit.
///
/// Ordered from most safe to least safe.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Safety {
    /// Safe to apply automatically. The semantic meaning of the code is preserved.
    /// Example: Formatting, renaming a local variable.
    #[default]
    Safe,
    /// Likely safe, but changes semantics slightly or relies on heuristics.
    /// Example: Removing an unused variable (might have side effects in constructor).
    PotentiallyUnsafe,
    /// Requires manual user review. Valid code, but changes logic significantly.
    /// Example: Changing type casts, altering control flow logic.
    Unsafe,
}

/// Represents a range in the source text identified by byte offsets.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    #[inline(always)]
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns the length of the range in bytes.
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns true if the range has a length of zero.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Checks if this range overlaps with another.
    ///
    /// We consider touching ranges as overlapping (e.g. `0..5` and `5..10` overlap at 5).
    /// This strictness is required to prevent ambiguity in insertion order (e.g. which
    /// insertion happens first at offset 5?).
    ///
    /// Zero-length ranges (insertions) at the boundary of another range are also
    /// considered overlapping (e.g. insert at 5 overlaps with `5..10`).
    #[inline(always)]
    pub fn overlaps(&self, other: &TextRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Checks if this range contains a specific offset.
    #[inline(always)]
    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }
}

impl<T> From<T> for TextRange
where
    T: std::ops::RangeBounds<u32>,
{
    #[inline(always)]
    fn from(r: T) -> Self {
        let start = match r.start_bound() {
            std::ops::Bound::Included(&s) => s,
            std::ops::Bound::Excluded(&s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match r.end_bound() {
            std::ops::Bound::Included(&e) => e + 1,
            std::ops::Bound::Excluded(&e) => e,
            std::ops::Bound::Unbounded => u32::MAX, // Will fail bounds check later
        };

        Self::new(start, end)
    }
}

/// A unified atomic edit operation.
///
/// This struct holds the data for a modification but does not execute it.
/// It always refers to the byte offsets in the **ORIGINAL** source code.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TextEdit {
    /// The range in the original text to be replaced.
    pub range: TextRange,
    /// The new text to replace the range with.
    pub new_text: String,
    /// How safe this specific edit is.
    pub safety: Safety,
}

impl TextEdit {
    /// Creates a delete edit (defaults to Safe).
    pub fn delete(range: impl Into<TextRange>) -> Self {
        Self { range: range.into(), new_text: String::new(), safety: Safety::Safe }
    }

    /// Creates an insert edit (defaults to Safe).
    pub fn insert(offset: u32, text: impl Into<String>) -> Self {
        Self { range: TextRange::new(offset, offset), new_text: text.into(), safety: Safety::Safe }
    }

    /// Creates a replace edit (defaults to Safe).
    pub fn replace(range: impl Into<TextRange>, text: impl Into<String>) -> Self {
        Self { range: range.into(), new_text: text.into(), safety: Safety::Safe }
    }

    /// Builder method to change the safety level of this edit.
    ///
    /// # Example
    /// ```
    /// use mago_text_edit::{TextEdit, Safety};
    ///
    /// let edit = TextEdit::replace(1..2, "b").with_safety(Safety::Unsafe);
    /// assert_eq!(edit.safety, Safety::Unsafe);
    /// ```
    #[must_use]
    pub fn with_safety(mut self, safety: Safety) -> Self {
        self.safety = safety;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ApplyResult {
    /// The edits were successfully applied.
    Applied,
    /// The edits were invalid (e.g., start > end or > file length).
    OutOfBounds,
    /// The edits overlapped with previously confirmed edits or each other.
    Overlap,
    /// The provided checker function returned `false`.
    Rejected,
    /// Edit rejected because it's unsafe and we're in safe or potentially-unsafe mode.
    Unsafe,
    /// Edit rejected because it's potentially-unsafe and we're in safe mode.
    PotentiallyUnsafe,
}

/// A high-performance, transactional text editor.
///
/// It accumulates edits and applies them in a single pass when `finish()` is called.
/// It ensures all edits are valid, non-overlapping, and safe according to optional user checks.
#[derive(Debug, Clone)]
pub struct TextEditor<'a> {
    original_text: &'a str,
    original_len: u32,
    edits: Vec<TextEdit>,
    /// Maximum safety level to accept. Edits above this level are rejected.
    safety_threshold: Safety,
}

impl<'a> TextEditor<'a> {
    /// Creates a new TextEditor with the default safety threshold (Unsafe - accepts all edits).
    pub fn new(text: &'a str) -> Self {
        Self {
            original_text: text,
            original_len: text.len() as u32,
            edits: Vec::new(),
            safety_threshold: Safety::Unsafe,
        }
    }

    /// Creates a new TextEditor with a specific safety threshold.
    ///
    /// Edits with a safety level above the threshold will be rejected.
    ///
    /// # Example
    /// ```
    /// use mago_text_edit::{TextEditor, Safety};
    ///
    /// // Only accept Safe edits
    /// let editor = TextEditor::with_safety("hello", Safety::Safe);
    /// ```
    pub fn with_safety(text: &'a str, threshold: Safety) -> Self {
        Self { original_text: text, original_len: text.len() as u32, edits: Vec::new(), safety_threshold: threshold }
    }

    /// Checks if an edit's safety level exceeds the threshold.
    /// Returns the appropriate rejection result, or None if the edit is acceptable.
    #[inline]
    fn check_safety(&self, edit_safety: Safety) -> Option<ApplyResult> {
        if edit_safety > self.safety_threshold {
            Some(match edit_safety {
                Safety::Unsafe => ApplyResult::Unsafe,
                Safety::PotentiallyUnsafe => ApplyResult::PotentiallyUnsafe,
                Safety::Safe => unreachable!(),
            })
        } else {
            None
        }
    }

    /// Applies a single edit.
    ///
    /// Uses binary search to check for overlaps in O(log N).
    /// Rejects edits that exceed the safety threshold.
    pub fn apply<F>(&mut self, edit: TextEdit, checker: Option<F>) -> ApplyResult
    where
        F: FnOnce(&str) -> bool,
    {
        // Check safety first
        if let Some(rejection) = self.check_safety(edit.safety) {
            return rejection;
        }

        if edit.range.end > self.original_len || edit.range.start > edit.range.end {
            return ApplyResult::OutOfBounds;
        }

        let search_idx = self.edits.partition_point(|e| e.range.end <= edit.range.start);

        if let Some(existing) = self.edits.get(search_idx)
            && existing.range.overlaps(&edit.range)
        {
            return ApplyResult::Overlap;
        }

        if let Some(check_fn) = checker {
            let simulated_str = stitch_one(self.original_text, &self.edits, &edit);
            if !check_fn(&simulated_str) {
                return ApplyResult::Rejected;
            }
        }

        self.edits.insert(search_idx, edit);

        ApplyResult::Applied
    }

    /// Applies a batch of edits atomically.
    ///
    /// Either all edits are applied, or none are (if overlap/check/safety fails).
    /// If any edit in the batch exceeds the safety threshold, the entire batch is rejected.
    pub fn apply_batch<F>(&mut self, mut new_edits: Vec<TextEdit>, checker: Option<F>) -> ApplyResult
    where
        F: FnOnce(&str) -> bool,
    {
        if new_edits.is_empty() {
            return ApplyResult::Applied;
        }

        // Check safety of all edits first
        for edit in &new_edits {
            if let Some(rejection) = self.check_safety(edit.safety) {
                return rejection;
            }
        }

        new_edits
            .sort_unstable_by(|a, b| a.range.start.cmp(&b.range.start).then_with(|| a.range.end.cmp(&b.range.end)));

        for i in 0..new_edits.len() {
            let edit = &new_edits[i];

            if edit.range.end > self.original_len || edit.range.start > edit.range.end {
                return ApplyResult::OutOfBounds;
            }

            if i > 0 && new_edits[i - 1].range.overlaps(&edit.range) {
                return ApplyResult::Overlap;
            }
        }

        {
            let mut old_iter = self.edits.iter();
            let mut new_iter = new_edits.iter();
            let mut next_old = old_iter.next();
            let mut next_new = new_iter.next();

            while let (Some(old), Some(new)) = (next_old, next_new) {
                if old.range.overlaps(&new.range) {
                    return ApplyResult::Overlap;
                }
                if old.range.start < new.range.start {
                    next_old = old_iter.next();
                } else {
                    next_new = new_iter.next();
                }
            }
        }

        if let Some(check_fn) = checker {
            let simulated_str = stitch_merged(self.original_text, &self.edits, &new_edits);
            if !check_fn(&simulated_str) {
                return ApplyResult::Rejected;
            }
        }

        self.edits.reserve(new_edits.len());
        self.edits.extend(new_edits);
        self.edits.sort_by_key(|a| a.range.start);

        ApplyResult::Applied
    }

    /// Consumes the editor and returns the final modified string.
    pub fn finish(self) -> String {
        stitch(self.original_text, &self.edits)
    }

    /// Returns a slice of the currently applied edits.
    pub fn get_edits(&self) -> &[TextEdit] {
        &self.edits
    }

    /// Returns the current safety threshold.
    pub fn safety_threshold(&self) -> Safety {
        self.safety_threshold
    }
}

/// Standard stitching of a sorted list.
/// Calculates exact capacity first to guarantee exactly 1 allocation.
fn stitch(original: &str, edits: &[TextEdit]) -> String {
    let mut final_len = original.len();
    for edit in edits {
        final_len = final_len.saturating_sub(edit.range.len() as usize).saturating_add(edit.new_text.len());
    }

    let mut output = String::with_capacity(final_len);
    let mut last_processed = 0;

    for edit in edits {
        let start = edit.range.start as usize;
        let end = edit.range.end as usize;

        if start > last_processed {
            output.push_str(&original[last_processed..start]);
        }
        output.push_str(&edit.new_text);
        last_processed = end;
    }

    if last_processed < original.len() {
        output.push_str(&original[last_processed..]);
    }

    output
}

/// Simulation for a single new edit (avoids creating a new vector).
fn stitch_one(original: &str, existing_edits: &[TextEdit], new_edit: &TextEdit) -> String {
    let slice = std::slice::from_ref(new_edit);
    stitch_merged(original, existing_edits, slice)
}

/// Simulation for merging two sorted lists of edits without mutating the original.
/// Used by the checker to verify validity before committing.
fn stitch_merged(original: &str, old_edits: &[TextEdit], new_edits: &[TextEdit]) -> String {
    let mut final_len = original.len();
    for e in old_edits {
        final_len = final_len - e.range.len() as usize + e.new_text.len();
    }
    for e in new_edits {
        final_len = final_len - e.range.len() as usize + e.new_text.len();
    }

    let mut output = String::with_capacity(final_len);
    let mut last_processed = 0;

    let mut old_iter = old_edits.iter();
    let mut new_iter = new_edits.iter();
    let mut next_old = old_iter.next();
    let mut next_new = new_iter.next();

    loop {
        let next_edit = match (next_old, next_new) {
            (Some(o), Some(n)) => {
                if o.range.start < n.range.start {
                    next_old = old_iter.next();
                    o
                } else {
                    next_new = new_iter.next();
                    n
                }
            }
            (Some(o), None) => {
                next_old = old_iter.next();
                o
            }
            (None, Some(n)) => {
                next_new = new_iter.next();
                n
            }
            (None, None) => break,
        };

        let start = next_edit.range.start as usize;
        let end = next_edit.range.end as usize;

        if start > last_processed {
            output.push_str(&original[last_processed..start]);
        }
        output.push_str(&next_edit.new_text);
        last_processed = end;
    }

    if last_processed < original.len() {
        output.push_str(&original[last_processed..]);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_single() {
        let mut editor = TextEditor::new("hello world");
        editor.apply(TextEdit::replace(0..5, "hi"), None::<fn(&str) -> bool>);
        assert_eq!(editor.finish(), "hi world");
    }

    #[test]
    fn test_checker_fail() {
        let mut editor = TextEditor::new("abc");
        // Fail if length of result > 10 (it won't be, so this passes check logic is inverted? No.)
        // Checker logic: return TRUE if valid.
        let res = editor.apply(TextEdit::delete(0..1), Some(|s: &str| s.len() > 10));
        // "bc" len is 2. 2 > 10 is false. Checker returns false.
        assert_eq!(res, ApplyResult::Rejected);
        assert_eq!(editor.finish(), "abc"); // Unchanged
    }

    #[test]
    fn test_overlap_search() {
        let mut editor = TextEditor::new("0123456789");
        editor.apply(TextEdit::replace(2..4, "x"), None::<fn(&str) -> bool>); // 2,3

        // Try edit at 3..5 (Overlaps 2..4)
        assert_eq!(editor.apply(TextEdit::replace(3..5, "y"), None::<fn(&str) -> bool>), ApplyResult::Overlap);

        // Try edit at 1..3 (Overlaps 2..4)
        assert_eq!(editor.apply(TextEdit::replace(1..3, "y"), None::<fn(&str) -> bool>), ApplyResult::Overlap);

        // Try edit at 4..5 (Safe)
        assert_eq!(editor.apply(TextEdit::replace(4..5, "y"), None::<fn(&str) -> bool>), ApplyResult::Applied);

        assert_eq!(editor.finish(), "01xy56789");
    }

    #[test]
    fn test_batch_apply_ordering() {
        let mut editor = TextEditor::new("abcdef");

        // Batch with mixed order inputs
        let batch = vec![
            TextEdit::replace(4..5, "E"), // e -> E
            TextEdit::replace(0..1, "A"), // a -> A
        ];

        editor.apply_batch(batch, None::<fn(&str) -> bool>);
        assert_eq!(editor.finish(), "AbcdEf");
    }

    #[test]
    fn test_safety_default_is_safe() {
        let edit = TextEdit::replace(0..1, "x");
        assert_eq!(edit.safety, Safety::Safe);
    }

    #[test]
    fn test_with_safety_builder() {
        let edit = TextEdit::replace(0..1, "x").with_safety(Safety::Unsafe);
        assert_eq!(edit.safety, Safety::Unsafe);

        let edit = TextEdit::delete(0..1).with_safety(Safety::PotentiallyUnsafe);
        assert_eq!(edit.safety, Safety::PotentiallyUnsafe);
    }

    #[test]
    fn test_safety_threshold_safe_mode() {
        let mut editor = TextEditor::with_safety("hello world", Safety::Safe);

        // Safe edit should be accepted
        let res = editor.apply(TextEdit::replace(0..5, "hi"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        // PotentiallyUnsafe edit should be rejected
        let res = editor
            .apply(TextEdit::replace(6..11, "there").with_safety(Safety::PotentiallyUnsafe), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::PotentiallyUnsafe);

        // Unsafe edit should be rejected
        let res = editor.apply(TextEdit::replace(6..11, "there").with_safety(Safety::Unsafe), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Unsafe);

        assert_eq!(editor.finish(), "hi world"); // Only safe edit applied
    }

    #[test]
    fn test_safety_threshold_potentially_unsafe_mode() {
        let mut editor = TextEditor::with_safety("hello world", Safety::PotentiallyUnsafe);

        // Safe edit should be accepted
        let res = editor.apply(TextEdit::replace(0..5, "hi"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        // PotentiallyUnsafe edit should be accepted
        let res = editor
            .apply(TextEdit::replace(6..11, "there").with_safety(Safety::PotentiallyUnsafe), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        assert_eq!(editor.finish(), "hi there");
    }

    #[test]
    fn test_safety_threshold_unsafe_mode() {
        let mut editor = TextEditor::with_safety("hello world", Safety::Unsafe);

        // All safety levels should be accepted
        let res = editor.apply(TextEdit::replace(0..1, "H"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        let res =
            editor.apply(TextEdit::replace(1..2, "E").with_safety(Safety::PotentiallyUnsafe), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        let res = editor.apply(TextEdit::replace(2..3, "L").with_safety(Safety::Unsafe), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        assert_eq!(editor.finish(), "HELlo world");
    }

    #[test]
    fn test_batch_safety_rejection() {
        let mut editor = TextEditor::with_safety("hello", Safety::Safe);

        // Batch with one unsafe edit should reject entire batch
        let batch = vec![
            TextEdit::replace(0..1, "H"),                             // Safe
            TextEdit::replace(1..2, "E").with_safety(Safety::Unsafe), // Unsafe - should cause rejection
        ];

        let res = editor.apply_batch(batch, None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Unsafe);

        // Original text unchanged
        assert_eq!(editor.finish(), "hello");
    }

    #[test]
    fn test_safety_ordering() {
        // Test that Safety enum orders correctly (Safe < PotentiallyUnsafe < Unsafe)
        assert!(Safety::Safe < Safety::PotentiallyUnsafe);
        assert!(Safety::PotentiallyUnsafe < Safety::Unsafe);
        assert!(Safety::Safe < Safety::Unsafe);
    }

    #[test]
    fn test_insert_at_same_offset_overlaps_replace() {
        let mut editor = TextEditor::new("0123456789");

        let res = editor.apply(TextEdit::replace(2..8, "replaced"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        let res = editor.apply(TextEdit::insert(2, "inserted"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Overlap);

        assert_eq!(editor.finish(), "01replaced89");
    }

    #[test]
    fn test_insert_after_replace_at_different_offset() {
        let mut editor = TextEditor::new("0123456789");

        let res = editor.apply(TextEdit::replace(2..5, "ABC"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        let res = editor.apply(TextEdit::insert(6, "X"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        assert_eq!(editor.finish(), "01ABC5X6789");
    }

    #[test]
    fn test_insert_at_start_of_replace_overlaps() {
        // Regression test for issue #828:
        // An insert at the start of a replace should overlap
        let mut editor = TextEditor::new("0123456789");

        let res = editor.apply(TextEdit::replace(2..5, "ABC"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Applied);

        let res = editor.apply(TextEdit::insert(2, "X"), None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Overlap);

        assert_eq!(editor.finish(), "01ABC56789");
    }

    #[test]
    fn test_batch_insert_and_replace_at_same_offset_overlap() {
        let mut editor = TextEditor::new("0123456789");

        let batch = vec![
            TextEdit::insert(2, "inserted"), // insert at 2
            TextEdit::replace(2..5, "ABC"),  // replace starting at 2
        ];

        let res = editor.apply_batch(batch, None::<fn(&str) -> bool>);
        assert_eq!(res, ApplyResult::Overlap);

        assert_eq!(editor.finish(), "0123456789");
    }

    #[test]
    fn test_touching_ranges_overlap() {
        let range1 = TextRange::new(0, 5);
        let range2 = TextRange::new(5, 10);
        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
    }

    #[test]
    fn test_insert_range_overlaps_with_adjacent_range() {
        let insert_range = TextRange::new(5, 5);
        let replace_range = TextRange::new(5, 10);
        assert!(insert_range.overlaps(&replace_range));
        assert!(replace_range.overlaps(&insert_range));
    }
}
