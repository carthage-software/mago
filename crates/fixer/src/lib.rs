use core::ops::Range;

use serde::Deserialize;
use serde::Serialize;
use strum::Display;

/// Represents a single change or difference between two versions of a string.
///
/// A `Change` indicates how a specific portion of the original text has been modified,
/// whether by being left unchanged, having new content inserted, or having some content deleted.
/// It forms the core building block of representing modifications between two versions of text.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum Change {
    /// Represents a section of text that remains unchanged between the original and the modified versions.
    Unchanged(String),

    /// Represents text that has been added in the modified version.
    Inserted(String),

    /// Represents text that has been removed in the modified version.
    Deleted(String),
}

/// Represents a collection of differences (changes) between the original and modified versions of a string.
///
/// A `ChangeSet` stores the sequence of changes that have occurred between two versions of a code snippet or text.
/// This struct provides the necessary data to reconstruct both the original and modified versions from
/// the list of changes. It serves as a foundational structure for comparing two versions of content.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ChangeSet {
    /// A list of changes, where each `Change` represents an insertion, deletion, or unchanged portion of the text.
    ///
    /// These changes, when applied in sequence, reconstruct either the original text or the modified text,
    /// depending on whether insertions or deletions are ignored.
    changes: Vec<Change>,
}

/// Represents the safety classifications of a code fix operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum SafetyClassification {
    /// Safe operations that are unlikely to cause issues.
    Safe,
    /// Operations that might cause issues under certain circumstances.
    PotentiallyUnsafe,
    /// Operations that are known to be unsafe.
    Unsafe,
}

/// Represents an individual operation in a code fix plan.
///
/// A `FixOperation` can perform various types of modifications on a piece of text,
/// such as inserting new content, replacing existing content, or deleting parts of the content.
/// Each operation is associated with a safety classification that indicates how safe it is to apply
/// the operation without causing unintended side effects.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FixOperation {
    /// Inserts new text at a specified position within the content.
    Insert {
        /// The position (in bytes) where the new text will be inserted.
        offset: u32,
        /// The text to be inserted.
        text: String,
        /// The safety classification of this operation. It indicates how safe it is to apply the insertion.
        safety_classification: SafetyClassification,
    },

    /// Replaces text in the specified range with new content.
    Replace {
        /// The range of text to be replaced, specified by start and end byte indices.
        range: Range<u32>,
        /// The new text that will replace the text within the given range.
        text: String,
        /// The safety classification of this operation.
        safety_classification: SafetyClassification,
    },

    /// Deletes text within a specified range.
    Delete {
        /// The range of text to be deleted, specified by start and end byte indices.
        range: Range<u32>,
        /// The safety classification of this operation.
        safety_classification: SafetyClassification,
    },
}

/// Represents a sequence of code fix operations to be applied to a piece of content.
///
/// A `FixPlan` contains multiple operations that describe how to modify a string of code.
/// The operations can include inserting new content, replacing old content, or deleting
/// unwanted parts. The operations are ordered and will be applied sequentially to the content.
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FixPlan {
    /// A SET of `FixOperation` instances that describe the specific changes to be made.
    operations: Vec<FixOperation>,
}

impl ChangeSet {
    /// Creates a new `ChangeSet` instance from a vector of `Change` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mago_fixer::{Change, ChangeSet};
    ///
    /// let changes = vec![
    ///     Change::Unchanged("Hello".to_string()),
    ///     Change::Deleted(" World".to_string()),
    ///     Change::Inserted(" Rustaceans".to_string()),
    /// ];
    ///
    /// let change_set = ChangeSet::new(changes);
    ///
    /// assert_eq!(change_set.get_original(), "Hello World");
    /// ```
    pub fn new(changes: Vec<Change>) -> Self {
        Self { changes }
    }

    /// Creates a new `ChangeSet` instance from an iterator of `Change` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mago_fixer::{Change, ChangeSet};
    ///
    /// let changes = vec![
    ///     Change::Unchanged("Hello".to_string()),
    ///     Change::Deleted(" World".to_string()),
    ///     Change::Inserted(" Rustaceans".to_string()),
    /// ];
    ///
    /// let change_set = ChangeSet::from(changes);
    /// ```
    ///
    /// # Parameters
    ///
    /// - `changes`: An iterator of `Change` instances.
    ///
    /// # Returns
    ///
    /// A new `ChangeSet` instance.
    pub fn from(changes: impl IntoIterator<Item = Change>) -> Self {
        Self { changes: changes.into_iter().collect() }
    }

    /// Reconstructs the original content from the list of changes.
    ///
    /// This method iterates over the `changes` vector and collects all the `Deleted` and `Unchanged`
    /// parts, effectively ignoring any `Inserted` text. The result is a string identical to the original content
    /// before any fix was applied.
    ///
    /// # Returns
    ///
    /// A `String` containing the original content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mago_fixer::{Change, ChangeSet};
    ///
    /// let changes = vec![
    ///     Change::Unchanged("Hello".to_string()),
    ///     Change::Deleted(" World".to_string()),
    ///     Change::Inserted(" Rustaceans".to_string()),
    /// ];
    ///
    /// let change_set = ChangeSet::new(changes);
    ///
    /// assert_eq!(change_set.get_original(), "Hello World");
    /// ```
    #[inline]
    pub fn get_original(&self) -> String {
        let mut result = String::new();
        for change in &self.changes {
            match change {
                Change::Deleted(text) => result.push_str(text),
                Change::Unchanged(text) => result.push_str(text),
                Change::Inserted(_) => {} // Ignore inserted text
            }
        }

        result
    }

    /// Reconstructs the fixed content from the changes.
    ///
    /// This method iterates over the `changes` vector and collects all the `Inserted` and `Unchanged`
    /// parts, effectively ignoring any `Deleted` text. The result is a string representing the content
    /// after all fix has been applied.
    ///
    /// # Returns
    ///
    /// A `String` containing the fixed content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mago_fixer::{Change, ChangeSet};
    ///
    /// let changes = vec![
    ///     Change::Unchanged("Hello".to_string()),
    ///     Change::Deleted(" World".to_string()),
    ///     Change::Inserted(" Rustaceans".to_string()),
    /// ];
    ///
    /// let change_set = ChangeSet::new(changes);
    ///
    /// assert_eq!(change_set.get_fixed(), "Hello Rustaceans");
    /// ```
    #[inline]
    pub fn get_fixed(&self) -> String {
        let mut result = String::new();
        for change in &self.changes {
            match change {
                Change::Deleted(_) => {} // Ignore deleted text
                Change::Unchanged(text) => result.push_str(text),
                Change::Inserted(text) => result.push_str(text),
            }
        }
        result
    }

    /// Returns the number of changes in the sequence.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns `true` if the sequence contains no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns a reference to the changes in the sequence.
    pub fn iter(&self) -> impl Iterator<Item = &Change> {
        self.changes.iter()
    }

    /// Returns a mutable reference to the changes in the sequence.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Change> {
        self.changes.iter_mut()
    }
}

impl FixOperation {
    pub fn get_safety_classification(&self) -> SafetyClassification {
        match self {
            FixOperation::Insert { safety_classification, .. } => *safety_classification,
            FixOperation::Replace { safety_classification, .. } => *safety_classification,
            FixOperation::Delete { safety_classification, .. } => *safety_classification,
        }
    }
}

impl FixPlan {
    /// Creates a new, empty `FixPlan` instance.
    ///
    /// This function initializes a `FixPlan` with no operations. You can use methods like
    /// `insert`, `replace`, and `delete` to add specific operations to the plan.
    ///
    /// # Returns
    /// A new `FixPlan` instance with no operations.
    pub fn new() -> Self {
        Self { operations: Vec::default() }
    }

    /// Creates a new `FixPlan` instance from a vector of `FixOperation` instances.
    pub fn from_operations(operations: Vec<FixOperation>) -> Self {
        Self { operations }
    }

    /// Adds a custom `FixOperation` to the plan.
    ///
    /// This method allows you to manually add a fix operation to the plan.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to add, which can be an insertion, replacement, or deletion.
    ///
    /// # Returns
    ///
    /// The updated `FixPlan` instance with the new operation added.
    pub fn operation(&mut self, operation: FixOperation) {
        self.operations.push(operation);
    }

    /// Adds an insertion operation to the plan.
    ///
    /// This method creates and adds an `Insert` operation to the plan, specifying
    /// where to insert the text and the text content itself.
    ///
    /// # Arguments
    ///
    /// * `offset` - The position at which the new text will be inserted.
    /// * `text` - The content to insert.
    /// * `safety` - The safety classification of this insertion.
    ///
    /// # Returns
    ///
    /// The updated `FixPlan` instance.
    pub fn insert(&mut self, offset: u32, text: impl Into<String>, safety: SafetyClassification) {
        self.operation(FixOperation::Insert { offset, text: text.into(), safety_classification: safety });
    }

    /// Adds a replacement operation to the plan.
    ///
    /// This method creates and adds a `Replace` operation to the plan, specifying
    /// the range of text to be replaced and the new text to be inserted.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of text to replace.
    /// * `text` - The new content to insert.
    /// * `safety` - The safety classification of this replacement.
    ///
    /// # Returns
    ///
    /// The updated `FixPlan` instance.
    pub fn replace(&mut self, range: Range<u32>, text: impl Into<String>, safety: SafetyClassification) {
        self.operation(FixOperation::Replace { range, text: text.into(), safety_classification: safety });
    }

    /// Adds a deletion operation to the plan.
    ///
    /// This method creates and adds a `Delete` operation to the plan, specifying
    /// the range of text to be deleted.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of text to delete.
    /// * `safety` - The safety classification of this deletion.
    ///
    /// # Returns
    ///
    /// The updated `FixPlan` instance.
    pub fn delete(&mut self, range: Range<u32>, safety: SafetyClassification) {
        self.operation(FixOperation::Delete { range, safety_classification: safety });
    }

    /// Merges another `FixPlan` into this one.
    ///
    /// This method appends all the operations from another `FixPlan` to the end
    /// of the current one, effectively combining two sequences of code fixes into one.
    ///
    /// # Arguments
    ///
    /// * `other` - The other `FixPlan` to merge.
    pub fn merge(&mut self, other: FixPlan) {
        for op in other.operations {
            self.operation(op);
        }
    }

    /// Returns a reference to the operations in the plan.
    pub fn get_operations(&self) -> &Vec<FixOperation> {
        &self.operations
    }

    /// Takes ownership of the operations in the plan.
    ///
    /// This method consumes the `FixPlan` and returns the list of operations it contains.
    pub fn take_operations(self) -> Vec<FixOperation> {
        self.operations
    }

    /// Determines the minimum safety classification across all operations in the plan.
    ///
    /// This function scans the safety classifications of all the operations in the plan and
    /// returns the lowest (most restrictive) safety classification. This can be used to determine
    /// whether the entire plan is safe to apply based on the user's preferred safety threshold.
    ///
    /// # Returns
    ///
    /// The minimum `SafetyClassification` of all operations.
    #[inline]
    pub fn get_minimum_safety_classification(&self) -> SafetyClassification {
        self.operations
            .iter()
            .map(|op| match op {
                FixOperation::Insert { safety_classification, .. } => *safety_classification,
                FixOperation::Replace { safety_classification, .. } => *safety_classification,
                FixOperation::Delete { safety_classification, .. } => *safety_classification,
            })
            .min()
            .unwrap_or(SafetyClassification::Safe)
    }

    #[inline]
    pub fn to_minimum_safety_classification(&self, safety: SafetyClassification) -> Self {
        let min_safety = self.get_minimum_safety_classification();
        if min_safety > safety {
            return Self::new();
        }

        Self {
            operations: self.operations.iter().filter(|op| op.get_safety_classification() <= safety).cloned().collect(),
        }
    }

    /// Determines whether the plan is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns the number of operations in the plan.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Executes the sequence of operations in the plan to a given text content.
    ///
    /// This function processes the original content according to the operations specified
    /// in the plan. It only executes operations that have a safety classification equal to or less than
    /// the provided maximum safety classification. The result is a `ChangeSet` object containing the
    /// modified content and the list of changes made.
    ///
    /// # Arguments
    ///
    /// * `content` - The original text to which the operations will be applied.
    /// * `max_safety_classification` - The maximum allowable safety classification for operations to be applied.
    ///
    /// # Returns
    ///
    /// A `ChangeSet` object representing the changes made to the content.
    #[inline]
    pub fn execute(&self, content: &str) -> ChangeSet {
        let mut operations = self.operations.clone();

        fix_overlapping_operations(&mut operations);

        let content_len = content.len() as u32;

        // Adjust out-of-bounds operations
        operations = operations
            .into_iter()
            .filter_map(|op| match op {
                FixOperation::Insert { offset, text, safety_classification } => {
                    let adjusted_offset = offset.min(content_len);

                    Some(FixOperation::Insert { offset: adjusted_offset, text, safety_classification })
                }
                FixOperation::Replace { range, text, safety_classification } => {
                    if range.start == range.end {
                        // Empty range, treat as insert
                        let adjusted_offset = range.start.min(content_len);

                        Some(FixOperation::Insert { offset: adjusted_offset, text, safety_classification })
                    } else if range.start >= content_len || range.start > range.end {
                        tracing::trace!("skipping invalid replace operation at range {:?} `{}`", range, text,);

                        // Ignore out-of-bounds or invalid ranges
                        None
                    } else {
                        let adjusted_end = range.end.min(content_len);

                        Some(FixOperation::Replace { range: range.start..adjusted_end, text, safety_classification })
                    }
                }
                FixOperation::Delete { range, safety_classification } => {
                    if range.start >= content_len || range.start >= range.end {
                        tracing::trace!("skipping invalid delete operation at range {:?}", range);

                        // Ignore out-of-bounds or invalid ranges
                        None
                    } else {
                        let adjusted_end = range.end.min(content_len);

                        Some(FixOperation::Delete { range: range.start..adjusted_end, safety_classification })
                    }
                }
            })
            .collect::<Vec<_>>();

        // Sort operations by start position
        operations.sort_by_key(|op| match op {
            FixOperation::Insert { offset, .. } => *offset,
            FixOperation::Replace { range, .. } => range.start,
            FixOperation::Delete { range, .. } => range.start,
        });

        let mut changes = Vec::new();
        let mut current_position = 0;
        let mut op_iter = operations.into_iter().peekable();

        while current_position < content_len || op_iter.peek().is_some() {
            if let Some(op) = op_iter.peek() {
                match op {
                    FixOperation::Insert { offset, text, .. } => {
                        if *offset <= current_position {
                            // Insert at the current position
                            changes.push(Change::Inserted(text.clone()));
                            op_iter.next();
                        } else {
                            // Consume unchanged content up to the insert position
                            let end = offset.min(&content_len);
                            if current_position < *end {
                                changes.push(Change::Unchanged(
                                    content[current_position as usize..*end as usize].to_string(),
                                ));
                                current_position = *end;
                            }
                        }
                    }
                    FixOperation::Replace { range, text, .. } => {
                        if range.start <= current_position {
                            // Replace at the current position
                            let delete_len = range.end - current_position;
                            if delete_len > 0 {
                                changes.push(Change::Deleted(
                                    content[current_position as usize..range.end as usize].to_string(),
                                ));
                            }
                            changes.push(Change::Inserted(text.clone()));
                            current_position = range.end;
                            op_iter.next();
                        } else {
                            // Consume unchanged content up to the replace position
                            let end = range.start.min(content_len);
                            if current_position < end {
                                changes.push(Change::Unchanged(
                                    content[current_position as usize..end as usize].to_string(),
                                ));
                                current_position = end;
                            }
                        }
                    }
                    FixOperation::Delete { range, .. } => {
                        if range.start <= current_position {
                            // Delete at the current position
                            let delete_len = range.end - current_position;
                            if delete_len > 0 {
                                changes.push(Change::Deleted(
                                    content[current_position as usize..range.end as usize].to_string(),
                                ));
                            }
                            current_position = range.end;
                            op_iter.next();
                        } else {
                            // Consume unchanged content up to the delete position
                            let end = range.start.min(content_len);
                            if current_position < end {
                                changes.push(Change::Unchanged(
                                    content[current_position as usize..end as usize].to_string(),
                                ));
                                current_position = end;
                            }
                        }
                    }
                }
            } else {
                // No more operations, consume remaining content
                if current_position < content_len {
                    changes.push(Change::Unchanged(content[current_position as usize..].to_string()));
                    current_position = content_len;
                }
            }
        }

        ChangeSet { changes }
    }
}

impl IntoIterator for FixPlan {
    type Item = FixOperation;
    type IntoIter = std::vec::IntoIter<FixOperation>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.into_iter()
    }
}

impl IntoIterator for ChangeSet {
    type Item = Change;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.changes.into_iter()
    }
}

impl FromIterator<Change> for ChangeSet {
    fn from_iter<T: IntoIterator<Item = Change>>(iter: T) -> Self {
        let changes = iter.into_iter().collect();
        ChangeSet { changes }
    }
}

impl FromIterator<FixOperation> for FixPlan {
    fn from_iter<T: IntoIterator<Item = FixOperation>>(iter: T) -> Self {
        let operations = iter.into_iter().collect();
        FixPlan { operations }
    }
}

impl FromIterator<FixPlan> for FixPlan {
    fn from_iter<T: IntoIterator<Item = FixPlan>>(iter: T) -> Self {
        let operations = iter.into_iter().flat_map(|plan| plan.operations).collect();

        FixPlan { operations }
    }
}
fn fix_overlapping_operations(operations: &mut Vec<FixOperation>) {
    let mut filtered_operations = Vec::new();

    for op in operations.iter() {
        match op {
            FixOperation::Delete { range, .. } => {
                let mut should_add = true;
                filtered_operations.retain(|existing_op| {
                    match existing_op {
                        FixOperation::Delete { range: existing_range, .. } => {
                            if existing_range.contains(&range.start) && existing_range.contains(&(range.end - 1)) {
                                // `op` is entirely within `existing_op`'s range
                                should_add = false;
                                return true;
                            } else if range.contains(&existing_range.start) && range.contains(&(existing_range.end - 1))
                            {
                                // `existing_op` is entirely within `op`'s range, so remove it
                                return false;
                            }
                            true
                        }
                        FixOperation::Replace { range: replace_range, .. } => {
                            if range.start <= replace_range.start && range.end >= replace_range.end {
                                // `Delete` operation completely covers `Replace` range, remove `Replace`
                                return false;
                            }
                            if range.start <= replace_range.end && range.end > replace_range.start {
                                // `Replace` falls within a `Delete`, ignore `Replace`
                                return false;
                            }
                            true
                        }
                        _ => true,
                    }
                });

                if should_add {
                    filtered_operations.push(op.clone());
                }
            }
            FixOperation::Replace { range, .. } => {
                let mut should_add = true;
                for existing_op in &filtered_operations {
                    if let FixOperation::Delete { range: delete_range, .. } = existing_op
                        && delete_range.start <= range.start
                        && delete_range.end >= range.end
                    {
                        // `Replace` falls within a `Delete`, so ignore `Replace`
                        should_add = false;
                        break;
                    }
                }
                if should_add {
                    filtered_operations.push(op.clone());
                }
            }
            _ => filtered_operations.push(op.clone()),
        }
    }

    // Replace original operations with filtered ones
    *operations = filtered_operations;
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_operations() {
        let content = "$a = ($b) + ($c);";

        let expected_safe = "$a = $b * $c;";
        let expected_potentially_unsafe = "$a = ($b * $c);";
        let expected_unsafe = "$a = ((int) $b * (int) $c);";

        let mut fix = FixPlan::new();

        fix.delete(5..6, SafetyClassification::Safe); // remove the `(` before $b
        fix.delete(8..9, SafetyClassification::Safe); // remove the `)` after $b
        fix.insert(6, "(int) ", SafetyClassification::Unsafe); // insert `(int) ` before $b
        fix.replace(10..11, "*", SafetyClassification::Safe); // replace `+` with `*`
        fix.delete(12..13, SafetyClassification::Safe); // remove the `(` before $c
        fix.insert(13, "(int) ", SafetyClassification::Unsafe); // insert `(int) ` before $c
        fix.delete(15..16, SafetyClassification::Safe); // remove the `)` after $c
        fix.insert(5, "(", SafetyClassification::PotentiallyUnsafe); // insert the outer `(` before $b
        fix.insert(16, ")", SafetyClassification::PotentiallyUnsafe); // insert the outer `)` after $c

        let safe_result = fix.to_minimum_safety_classification(SafetyClassification::Safe).execute(content);
        let potentially_unsafe_result =
            fix.to_minimum_safety_classification(SafetyClassification::PotentiallyUnsafe).execute(content);
        let unsafe_result = fix.to_minimum_safety_classification(SafetyClassification::Unsafe).execute(content);

        assert_eq!(safe_result.get_fixed(), expected_safe);
        assert_eq!(potentially_unsafe_result.get_fixed(), expected_potentially_unsafe);
        assert_eq!(unsafe_result.get_fixed(), expected_unsafe);

        assert_eq!(
            safe_result.changes,
            vec![
                Change::Unchanged("$a = ".to_string()),
                Change::Deleted("(".to_string()),
                Change::Unchanged("$b".to_string()),
                Change::Deleted(")".to_string()),
                Change::Unchanged(" ".to_string()),
                Change::Deleted("+".to_string()),
                Change::Inserted("*".to_string()),
                Change::Unchanged(" ".to_string()),
                Change::Deleted("(".to_string()),
                Change::Unchanged("$c".to_string()),
                Change::Deleted(")".to_string()),
                Change::Unchanged(";".to_string()),
            ]
        );
    }

    #[test]
    fn test_insert_within_bounds() {
        // Insert at a valid position within the content
        let content = "Hello World";
        let mut fix = FixPlan::new();
        fix.insert(6, "Beautiful ", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello Beautiful World");
    }

    #[test]
    fn test_insert_at_end() {
        // Insert at an offset equal to content length
        let content = "Hello";
        let mut fix = FixPlan::new();
        fix.insert(5, " World", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello World");
    }

    #[test]
    fn test_insert_beyond_bounds() {
        // Insert at an offset beyond content length
        let content = "Hello";
        let mut fix = FixPlan::new();
        fix.insert(100, " World", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello World"); // Inserted at the end
    }

    #[test]
    fn test_delete_within_bounds() {
        // Delete a valid range within the content
        let content = "Hello Beautiful World";
        let mut fix = FixPlan::new();
        fix.delete(6..16, SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello World");
    }

    #[test]
    fn test_delete_beyond_bounds() {
        // Delete a range that is partially out of bounds
        let content = "Hello World";
        let mut fix = FixPlan::new();
        fix.delete(6..100, SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello "); // Deleted from offset 6 to end
    }

    #[test]
    fn test_delete_out_of_bounds() {
        // Delete a range completely out of bounds
        let content = "Hello";
        let mut fix = FixPlan::new();
        fix.delete(10..20, SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello"); // No changes
    }

    #[test]
    fn test_replace_within_bounds() {
        // Replace a valid range within the content
        let content = "Hello World";
        let mut fix = FixPlan::new();
        fix.replace(6..11, "Rust", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello Rust");
    }

    #[test]
    fn test_replace_beyond_bounds() {
        // Replace a range that is partially out of bounds
        let content = "Hello World";
        let mut fix = FixPlan::new();
        fix.replace(6..100, "Rustaceans", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello Rustaceans"); // Replaced from offset 6 to end
    }

    #[test]
    fn test_overlapping_deletes() {
        let content = "Hello World";
        let mut fix = FixPlan::new();
        fix.delete(3..9, SafetyClassification::Safe);
        fix.delete(4..8, SafetyClassification::Safe);
        fix.delete(5..7, SafetyClassification::Safe);
        fix.replace(5..7, "xx", SafetyClassification::Safe);
        fix.delete(10..11, SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hell");
    }

    #[test]
    fn test_replace_out_of_bounds() {
        // Replace a range completely out of bounds
        let content = "Hello";
        let mut fix = FixPlan::new();
        fix.replace(10..20, "Hi", SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello"); // No changes
    }

    #[test]
    fn test_overlapping_operations() {
        // Overlapping delete and replace operations
        let content = "The quick brown fox jumps over the lazy dog.";
        let mut fix = FixPlan::new();
        fix.delete(10..19, SafetyClassification::Safe); // Delete "brown fox"
        fix.insert(16, "cat", SafetyClassification::Safe); // Replace "fox" (which is partially deleted)
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "The quick cat jumps over the lazy dog.");
        // "brown fox" deleted, "cat" inserted
    }

    #[test]
    fn test_insert_at_zero() {
        // Insert at the beginning of the content
        let content = "World";
        let mut fix = FixPlan::new();
        fix.insert(0, "Hello ", SafetyClassification::Safe);
        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello World");
    }

    #[test]
    fn test_empty_content_insert() {
        // Insert into empty content
        let content = "";
        let mut fix = FixPlan::new();
        fix.insert(0, "Hello World", SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello World");
    }

    #[test]
    fn test_empty_content_delete() {
        // Attempt to delete from empty content
        let content = "";
        let mut fix = FixPlan::new();
        fix.delete(0..10, SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), ""); // No changes
    }

    #[test]
    fn test_multiple_operations_ordering() {
        // Multiple operations affecting ordering
        let content = "abcdef";
        let mut fix = FixPlan::new();
        fix.delete(2..4, SafetyClassification::Safe); // Delete "cd"
        fix.insert(2, "XY", SafetyClassification::Safe); // Insert "XY" at position 2
        fix.replace(0..2, "12", SafetyClassification::Safe); // Replace "ab" with "12"
        fix.insert(6, "34", SafetyClassification::Safe); // Insert "34" at the end (after fix)

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "12XYef34");
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_operations_with_invalid_ranges() {
        // Operations with invalid ranges (start >= end)
        let content = "Hello World";
        let mut fix = FixPlan::new();

        fix.delete(5..3, SafetyClassification::Safe); // Invalid range
        fix.replace(8..8, "Test", SafetyClassification::Safe); // Empty range, treated as insert
        fix.insert(6, "Beautiful ", SafetyClassification::Safe); // Valid insert

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "Hello Beautiful WoTestrld"); // Only the insert is applied
    }

    #[test]
    fn test_happy_path() {
        let content = "<?php for (;true;): endfor;";
        let mut fix = FixPlan::new();

        fix.replace(6..12, "while(", SafetyClassification::Safe);
        fix.delete(16..17, SafetyClassification::Safe);
        fix.replace(20..26, "endwhile", SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "<?php while(true): endwhile;");
    }

    #[test]
    fn test_happy_path_2() {
        let content = "<?php for (;;): endfor;";
        let mut fix = FixPlan::new();

        fix.replace(6..10, "while", SafetyClassification::Safe);
        fix.delete(11..12, SafetyClassification::Safe);
        fix.insert(12, "true", SafetyClassification::Safe);
        fix.delete(12..13, SafetyClassification::Safe);
        fix.replace(16..22, "endwhile", SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "<?php while(true): endwhile;");
    }

    #[test]
    fn test_happy_path_3() {
        let content = "<?php for(;;): endfor;";
        let mut fix = FixPlan::new();

        fix.delete(6..9, SafetyClassification::Safe);
        fix.insert(6, "while", SafetyClassification::Safe);
        fix.delete(10..11, SafetyClassification::Safe);
        fix.insert(11, "true", SafetyClassification::Safe);
        fix.delete(11..12, SafetyClassification::Safe);
        fix.replace(15..21, "endwhile", SafetyClassification::Safe);

        let result = fix.execute(content);
        assert_eq!(result.get_fixed(), "<?php while(true): endwhile;");
    }
}
