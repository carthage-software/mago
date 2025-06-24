use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_source::SourceIdentifier;

use crate::symbol::SymbolIdentifier;

/// Represents the differences between two states of a codebase, typically used for incremental analysis.
///
/// It tracks symbols/members to keep, those whose signatures changed but bodies might be reusable,
/// added/deleted symbols/members, and detailed text diff/deletion ranges per file.
/// Provides a comprehensive API for modification and querying following established conventions.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodebaseDiff {
    /// Set of `(Symbol, Member)` pairs whose definition and signature are unchanged and can be kept as is.
    /// Member is empty for top-level symbols.
    keep: HashSet<SymbolIdentifier>,

    /// Set of `(Symbol, Member)` pairs whose signature (e.g., parameter types, return type)
    /// is unchanged, allowing potential reuse of inferred body information, even if the body itself changed.
    /// Member is empty for top-level symbols.
    keep_signature: HashSet<SymbolIdentifier>,

    /// Set of `(Symbol, Member)` pairs that were either added or deleted entirely between states.
    /// Member is empty for top-level symbols.
    add_or_delete: HashSet<SymbolIdentifier>,

    /// Map from source file identifier to a vector of text diff hunks.
    /// Each tuple typically represents `(old_start, old_len, new_start, new_len)` line info for a change.
    /// (Exact tuple meaning depends on the diffing library used).
    diff_map: HashMap<SourceIdentifier, Vec<(usize, usize, isize, isize)>>,

    /// Map from source file identifier to a vector of deleted line ranges `(start_line, end_line)`.
    deletion_ranges_map: HashMap<SourceIdentifier, Vec<(usize, usize)>>,
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
        self.keep_signature.extend(other.keep_signature);
        self.add_or_delete.extend(other.add_or_delete);
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

    /// Returns a reference to the set of symbols/members whose signatures can be kept.
    #[inline]
    pub fn get_keep_signature(&self) -> &HashSet<SymbolIdentifier> {
        &self.keep_signature
    }

    /// Returns a reference to the set of added or deleted symbols/members.
    #[inline]
    pub fn get_add_or_delete(&self) -> &HashSet<SymbolIdentifier> {
        &self.add_or_delete
    }

    /// Returns a reference to the map of source files to text diff hunks.
    #[inline]
    pub fn get_diff_map(&self) -> &HashMap<SourceIdentifier, Vec<(usize, usize, isize, isize)>> {
        &self.diff_map
    }

    /// Returns a reference to the map of source files to deletion ranges.
    #[inline]
    pub fn get_deletion_ranges_map(&self) -> &HashMap<SourceIdentifier, Vec<(usize, usize)>> {
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

    /// Sets the 'keep_signature' set, replacing the existing one.
    #[inline]
    pub fn set_keep_signature(&mut self, keep_set: impl IntoIterator<Item = SymbolIdentifier>) {
        self.keep_signature = keep_set.into_iter().collect();
    }

    /// Returns a new instance with the 'keep_signature' set replaced.
    #[inline]
    pub fn with_keep_signature(mut self, keep_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.set_keep_signature(keep_set);
        self
    }

    /// Adds a single entry to the 'keep_signature' set. Returns `true` if the entry was not already present.
    #[inline]
    pub fn add_keep_signature_entry(&mut self, entry: SymbolIdentifier) -> bool {
        self.keep_signature.insert(entry)
    }

    /// Returns a new instance with the entry added to the 'keep_signature' set.
    #[inline]
    pub fn with_added_keep_signature_entry(mut self, entry: SymbolIdentifier) -> Self {
        self.add_keep_signature_entry(entry);
        self
    }

    /// Adds multiple entries to the 'keep_signature' set.
    #[inline]
    pub fn add_keep_signature_entries(&mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) {
        self.keep_signature.extend(entries);
    }

    /// Returns a new instance with multiple entries added to the 'keep_signature' set.
    #[inline]
    pub fn with_added_keep_signature_entries(mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.add_keep_signature_entries(entries);
        self
    }

    /// Clears the 'keep_signature' set.
    #[inline]
    pub fn unset_keep_signature(&mut self) {
        self.keep_signature.clear();
    }

    /// Returns a new instance with an empty 'keep_signature' set.
    #[inline]
    pub fn without_keep_signature(mut self) -> Self {
        self.unset_keep_signature();
        self
    }

    /// Sets the 'add_or_delete' set, replacing the existing one.
    #[inline]
    pub fn set_add_or_delete(&mut self, change_set: impl IntoIterator<Item = SymbolIdentifier>) {
        self.add_or_delete = change_set.into_iter().collect();
    }

    /// Returns a new instance with the 'add_or_delete' set replaced.
    #[inline]
    pub fn with_add_or_delete(mut self, change_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.set_add_or_delete(change_set);
        self
    }

    /// Adds a single entry to the 'add_or_delete' set. Returns `true` if the entry was not already present.
    #[inline]
    pub fn add_add_or_delete_entry(&mut self, entry: SymbolIdentifier) -> bool {
        self.add_or_delete.insert(entry)
    }

    /// Checks if the 'add_or_delete' set contains a specific entry.
    #[inline]
    pub fn contains_add_or_delete_entry(&self, entry: &SymbolIdentifier) -> bool {
        self.add_or_delete.contains(entry)
    }

    /// Returns a new instance with the entry added to the 'add_or_delete' set.
    #[inline]
    pub fn with_added_add_or_delete_entry(mut self, entry: SymbolIdentifier) -> Self {
        self.add_add_or_delete_entry(entry);
        self
    }

    /// Adds multiple entries to the 'add_or_delete' set.
    #[inline]
    pub fn add_add_or_delete_entries(&mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) {
        self.add_or_delete.extend(entries);
    }

    /// Returns a new instance with multiple entries added to the 'add_or_delete' set.
    #[inline]
    pub fn with_added_add_or_delete_entries(mut self, entries: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.add_add_or_delete_entries(entries);
        self
    }

    /// Clears the 'add_or_delete' set.
    #[inline]
    pub fn unset_add_or_delete(&mut self) {
        self.add_or_delete.clear();
    }

    /// Returns a new instance with an empty 'add_or_delete' set.
    #[inline]
    pub fn without_add_or_delete(mut self) -> Self {
        self.unset_add_or_delete();
        self
    }

    /// Sets the diff map, replacing the existing one.
    #[inline]
    pub fn set_diff_map(&mut self, map: HashMap<SourceIdentifier, Vec<(usize, usize, isize, isize)>>) {
        self.diff_map = map;
    }

    /// Returns a new instance with the diff map replaced.
    #[inline]
    pub fn with_diff_map(mut self, map: HashMap<SourceIdentifier, Vec<(usize, usize, isize, isize)>>) -> Self {
        self.set_diff_map(map);
        self
    }

    /// Adds or replaces the diff hunks for a specific source file. Returns previous hunks if any.
    #[inline]
    pub fn add_diff_map_entry(
        &mut self,
        source: SourceIdentifier,
        diffs: Vec<(usize, usize, isize, isize)>,
    ) -> Option<Vec<(usize, usize, isize, isize)>> {
        self.diff_map.insert(source, diffs)
    }

    /// Returns a new instance with the diff hunks for the source file added or updated.
    #[inline]
    pub fn with_added_diff_map_entry(
        mut self,
        source: SourceIdentifier,
        diffs: Vec<(usize, usize, isize, isize)>,
    ) -> Self {
        self.add_diff_map_entry(source, diffs);
        self
    }

    /// Extends the diff hunks for a specific source file.
    #[inline]
    pub fn add_diffs_for_source(
        &mut self,
        source: SourceIdentifier,
        diffs: impl IntoIterator<Item = (usize, usize, isize, isize)>,
    ) {
        self.diff_map.entry(source).or_default().extend(diffs);
    }

    /// Returns a new instance with the diff hunks for the source file extended.
    #[inline]
    pub fn with_added_diffs_for_source(
        mut self,
        source: SourceIdentifier,
        diffs: impl IntoIterator<Item = (usize, usize, isize, isize)>,
    ) -> Self {
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
    pub fn set_deletion_ranges_map(&mut self, map: HashMap<SourceIdentifier, Vec<(usize, usize)>>) {
        self.deletion_ranges_map = map;
    }

    /// Returns a new instance with the deletion ranges map replaced.
    #[inline]
    pub fn with_deletion_ranges_map(mut self, map: HashMap<SourceIdentifier, Vec<(usize, usize)>>) -> Self {
        self.set_deletion_ranges_map(map);
        self
    }

    /// Adds or replaces the deletion ranges for a specific source file. Returns previous ranges if any.
    #[inline]
    pub fn add_deletion_ranges_entry(
        &mut self,
        source: SourceIdentifier,
        ranges: Vec<(usize, usize)>,
    ) -> Option<Vec<(usize, usize)>> {
        self.deletion_ranges_map.insert(source, ranges)
    }

    /// Returns a new instance with the deletion ranges for the source file added or updated.
    #[inline]
    pub fn with_added_deletion_ranges_entry(mut self, source: SourceIdentifier, ranges: Vec<(usize, usize)>) -> Self {
        self.add_deletion_ranges_entry(source, ranges);
        self
    }

    /// Extends the deletion ranges for a specific source file.
    #[inline]
    pub fn add_deletion_ranges_for_source(
        &mut self,
        source: SourceIdentifier,
        ranges: impl IntoIterator<Item = (usize, usize)>,
    ) {
        self.deletion_ranges_map.entry(source).or_default().extend(ranges);
    }

    /// Returns a new instance with the deletion ranges for the source file extended.
    #[inline]
    pub fn with_added_deletion_ranges_for_source(
        mut self,
        source: SourceIdentifier,
        ranges: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        self.add_deletion_ranges_for_source(source, ranges);
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
