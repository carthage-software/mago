use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_database::file::FileId;

use crate::symbol::SymbolIdentifier;

/// Represents a text diff hunk with position and offset information.
///
/// Format: `(old_start, old_length, line_offset, column_offset)`
/// - `old_start`: Starting byte offset in the old version
/// - `old_length`: Length of the changed region in bytes
/// - `line_offset`: Line number change (new_line - old_line)
/// - `column_offset`: Column number change (new_column - old_column)
pub type DiffHunk = (usize, usize, isize, isize);

/// Represents a range of deleted code.
///
/// Format: `(start_offset, end_offset)`
/// - `start_offset`: Starting byte offset of deletion
/// - `end_offset`: Ending byte offset of deletion
pub type DeletionRange = (usize, usize);

/// Represents the differences between two states of a codebase, typically used for incremental analysis.
///
/// This structure uses a single fingerprint hash per symbol to determine changes. Any change to a symbol
/// (signature, body, modifiers, attributes) produces a different hash, triggering re-analysis.
///
/// Provides a comprehensive API for modification and querying following established conventions.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodebaseDiff {
    /// Set of `(Symbol, Member)` pairs whose fingerprint hash is UNCHANGED.
    /// These symbols can be safely skipped during re-analysis.
    /// Member is empty for top-level symbols.
    keep: HashSet<SymbolIdentifier>,

    /// Set of `(Symbol, Member)` pairs that are new, deleted, or have a different fingerprint hash.
    /// These symbols MUST be re-analyzed.
    /// Member is empty for top-level symbols.
    changed: HashSet<SymbolIdentifier>,

    /// Map from source file identifier to a vector of text diff hunks.
    /// Used for mapping issue positions between old and new code.
    diff_map: HashMap<FileId, Vec<DiffHunk>>,

    /// Map from source file identifier to a vector of deleted code ranges.
    /// Used for filtering out issues in deleted code regions.
    deletion_ranges_map: HashMap<FileId, Vec<DeletionRange>>,
}

impl CodebaseDiff {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges changes from another `CodebaseDiff` into this one.
    #[inline]
    pub fn extend(&mut self, other: Self) {
        self.keep.extend(other.keep);
        self.changed.extend(other.changed);
        for (source, diffs) in other.diff_map {
            self.diff_map.entry(source).or_default().extend(diffs);
        }
        for (source, ranges) in other.deletion_ranges_map {
            self.deletion_ranges_map.entry(source).or_default().extend(ranges);
        }
    }

    /// Returns a reference to the set of symbols/members to keep unchanged.
    #[inline]
    pub fn get_keep(&self) -> &HashSet<SymbolIdentifier> {
        &self.keep
    }

    /// Returns a reference to the set of changed symbols/members.
    #[inline]
    pub fn get_changed(&self) -> &HashSet<SymbolIdentifier> {
        &self.changed
    }

    /// Returns a reference to the map of source files to text diff hunks.
    #[inline]
    pub fn get_diff_map(&self) -> &HashMap<FileId, Vec<DiffHunk>> {
        &self.diff_map
    }

    /// Returns a reference to the map of source files to deletion ranges.
    #[inline]
    pub fn get_deletion_ranges_map(&self) -> &HashMap<FileId, Vec<DeletionRange>> {
        &self.deletion_ranges_map
    }

    /// Sets the 'keep' set, replacing the existing one.
    #[inline]
    pub fn set_keep(&mut self, keep_set: impl IntoIterator<Item = SymbolIdentifier>) {
        self.keep = keep_set.into_iter().collect();
    }

    /// Returns a new instance with the 'keep' set replaced.
    #[inline]
    pub fn with_keep(mut self, keep_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.set_keep(keep_set);
        self
    }

    /// Adds a single entry to the 'keep' set. Returns `true` if the entry was not already present.
    #[inline]
    pub fn add_keep_entry(&mut self, entry: SymbolIdentifier) -> bool {
        self.keep.insert(entry)
    }

    /// Returns a new instance with the entry added to the 'keep' set.
    #[inline]
    pub fn with_added_keep_entry(mut self, entry: SymbolIdentifier) -> Self {
        self.add_keep_entry(entry);
        self
    }

    /// Adds multiple entries to the 'keep' set.
    #[inline]
    pub fn add_keep_entries(&mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) {
        self.keep.extend(entries);
    }

    /// Returns a new instance with multiple entries added to the 'keep' set.
    #[inline]
    pub fn with_added_keep_entries(mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.add_keep_entries(entries);
        self
    }

    /// Clears the 'keep' set.
    #[inline]
    pub fn unset_keep(&mut self) {
        self.keep.clear();
    }

    /// Returns a new instance with an empty 'keep' set.
    #[inline]
    pub fn without_keep(mut self) -> Self {
        self.unset_keep();
        self
    }

    /// Sets the 'changed' set, replacing the existing one.
    #[inline]
    pub fn set_changed(&mut self, change_set: impl IntoIterator<Item = SymbolIdentifier>) {
        self.changed = change_set.into_iter().collect();
    }

    /// Returns a new instance with the 'changed' set replaced.
    #[inline]
    pub fn with_changed(mut self, change_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.set_changed(change_set);
        self
    }

    /// Adds a single entry to the 'changed' set. Returns `true` if the entry was not already present.
    #[inline]
    pub fn add_changed_entry(&mut self, entry: SymbolIdentifier) -> bool {
        self.changed.insert(entry)
    }

    /// Checks if the 'changed' set contains a specific entry.
    #[inline]
    pub fn contains_changed_entry(&self, entry: &SymbolIdentifier) -> bool {
        self.changed.contains(entry)
    }

    /// Returns a new instance with the entry added to the 'changed' set.
    #[inline]
    pub fn with_added_changed_entry(mut self, entry: SymbolIdentifier) -> Self {
        self.add_changed_entry(entry);
        self
    }

    /// Adds multiple entries to the 'changed' set.
    #[inline]
    pub fn add_changed_entries(&mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) {
        self.changed.extend(entries);
    }

    /// Returns a new instance with multiple entries added to the 'changed' set.
    #[inline]
    pub fn with_added_changed_entries(mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.add_changed_entries(entries);
        self
    }

    /// Clears the 'changed' set.
    #[inline]
    pub fn unset_changed(&mut self) {
        self.changed.clear();
    }

    /// Returns a new instance with an empty 'changed' set.
    #[inline]
    pub fn without_changed(mut self) -> Self {
        self.unset_changed();
        self
    }

    /// Sets the diff map, replacing the existing one.
    #[inline]
    pub fn set_diff_map(&mut self, map: HashMap<FileId, Vec<DiffHunk>>) {
        self.diff_map = map;
    }

    /// Returns a new instance with the diff map replaced.
    #[inline]
    pub fn with_diff_map(mut self, map: HashMap<FileId, Vec<DiffHunk>>) -> Self {
        self.set_diff_map(map);
        self
    }

    /// Adds or replaces the diff hunks for a specific source file. Returns previous hunks if any.
    #[inline]
    pub fn add_diff_map_entry(&mut self, source: FileId, diffs: Vec<DiffHunk>) -> Option<Vec<DiffHunk>> {
        self.diff_map.insert(source, diffs)
    }

    /// Returns a new instance with the diff hunks for the source file added or updated.
    #[inline]
    pub fn with_added_diff_map_entry(mut self, source: FileId, diffs: Vec<DiffHunk>) -> Self {
        self.add_diff_map_entry(source, diffs);
        self
    }

    /// Extends the diff hunks for a specific source file.
    #[inline]
    pub fn add_diffs_for_source(&mut self, source: FileId, diffs: impl IntoIterator<Item = DiffHunk>) {
        self.diff_map.entry(source).or_default().extend(diffs);
    }

    /// Returns a new instance with the diff hunks for the source file extended.
    #[inline]
    pub fn with_added_diffs_for_source(mut self, source: FileId, diffs: impl IntoIterator<Item = DiffHunk>) -> Self {
        self.add_diffs_for_source(source, diffs);
        self
    }

    /// Clears the diff map.
    #[inline]
    pub fn unset_diff_map(&mut self) {
        self.diff_map.clear();
    }

    /// Returns a new instance with an empty diff map.
    #[inline]
    pub fn without_diff_map(mut self) -> Self {
        self.unset_diff_map();
        self
    }

    /// Sets the deletion ranges map, replacing the existing one.
    #[inline]
    pub fn set_deletion_ranges_map(&mut self, map: HashMap<FileId, Vec<DeletionRange>>) {
        self.deletion_ranges_map = map;
    }

    /// Returns a new instance with the deletion ranges map replaced.
    #[inline]
    pub fn with_deletion_ranges_map(mut self, map: HashMap<FileId, Vec<DeletionRange>>) -> Self {
        self.set_deletion_ranges_map(map);
        self
    }

    /// Adds or replaces the deletion ranges for a specific source file. Returns previous ranges if any.
    #[inline]
    pub fn add_deletion_ranges_entry(
        &mut self,
        source: FileId,
        ranges: Vec<DeletionRange>,
    ) -> Option<Vec<DeletionRange>> {
        self.deletion_ranges_map.insert(source, ranges)
    }

    /// Returns a new instance with the deletion ranges for the source file added or updated.
    #[inline]
    pub fn with_added_deletion_ranges_entry(mut self, file: FileId, ranges: Vec<DeletionRange>) -> Self {
        self.add_deletion_ranges_entry(file, ranges);
        self
    }

    /// Extends the deletion ranges for a specific source file.
    #[inline]
    pub fn add_deletion_ranges_for_source(&mut self, file: FileId, ranges: impl IntoIterator<Item = (usize, usize)>) {
        self.deletion_ranges_map.entry(file).or_default().extend(ranges);
    }

    /// Returns a new instance with the deletion ranges for the source file extended.
    #[inline]
    pub fn with_added_deletion_ranges_for_source(
        mut self,
        file: FileId,
        ranges: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        self.add_deletion_ranges_for_source(file, ranges);
        self
    }

    /// Clears the deletion ranges map.
    #[inline]
    pub fn unset_deletion_ranges_map(&mut self) {
        self.deletion_ranges_map.clear();
    }

    /// Returns a new instance with an empty deletion ranges map.
    #[inline]
    pub fn without_deletion_ranges_map(mut self) -> Self {
        self.unset_deletion_ranges_map();
        self
    }
}
