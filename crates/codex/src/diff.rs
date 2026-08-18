use foldhash::HashSet;

use crate::differ::compute_file_diff;
use crate::metadata::CodebaseMetadata;
use crate::symbol::SymbolIdentifier;

/// Represents the differences between two states of a codebase, typically used for incremental analysis.
///
/// This structure uses a single fingerprint hash per symbol to determine changes. Any change to a symbol
/// (signature, body, modifiers, attributes) produces a different hash, triggering re-analysis.
///
/// Provides a comprehensive API for modification and querying following established conventions.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodebaseDiff {
    /// Set of `(Symbol, Member)` pairs whose fingerprint hash is UNCHANGED.
    /// These symbols can be safely skipped during re-analysis.
    /// Member is empty for top-level symbols.
    keep: HashSet<SymbolIdentifier>,

    /// Set of `(Symbol, Member)` pairs that are new, deleted, or have a different fingerprint hash.
    /// These symbols MUST be re-analyzed.
    /// Member is empty for top-level symbols.
    changed: HashSet<SymbolIdentifier>,
}

impl CodebaseDiff {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes the `CodebaseDiff` between two `CodebaseMetadata` instances.
    ///
    /// This method compares the metadata of the old and new codebases to determine which symbols have changed,
    /// which can be kept unchanged, and what text diffs exist for source files.
    ///
    /// It aggregates this information into a `CodebaseDiff` instance that can be used for incremental analysis.
    #[must_use]
    pub fn between(old_metadata: &CodebaseMetadata, new_metadata: &CodebaseMetadata) -> Self {
        let mut aggregate_diff = CodebaseDiff::new();

        let mut all_file_ids = old_metadata.get_all_file_ids();
        all_file_ids.extend(new_metadata.get_all_file_ids());
        all_file_ids.sort();
        all_file_ids.dedup();

        for file_id in all_file_ids {
            let old_sig = old_metadata.get_file_signature(&file_id);
            let new_sig = new_metadata.get_file_signature(&file_id);

            let file_diff = compute_file_diff(file_id, old_sig, new_sig);

            aggregate_diff.extend(file_diff);
        }

        aggregate_diff
    }

    /// Merges changes from another `CodebaseDiff` into this one.
    #[inline]
    pub fn extend(&mut self, other: Self) {
        self.keep.extend(other.keep);
        self.changed.extend(other.changed);
    }

    /// Returns a reference to the set of symbols/members to keep unchanged.
    #[inline]
    #[must_use]
    pub fn get_keep(&self) -> &HashSet<SymbolIdentifier> {
        &self.keep
    }

    /// Returns a reference to the set of changed symbols/members.
    #[inline]
    #[must_use]
    pub fn get_changed(&self) -> &HashSet<SymbolIdentifier> {
        &self.changed
    }

    /// Returns a new instance with the 'keep' set replaced.
    #[inline]
    #[must_use]
    pub fn with_keep(mut self, keep_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.keep = keep_set.into_iter().collect();
        self
    }

    /// Adds a single entry to the 'keep' set. Returns `true` if the entry was not already present.
    #[inline]
    pub fn add_keep_entry(&mut self, entry: SymbolIdentifier) -> bool {
        self.keep.insert(entry)
    }

    /// Returns a new instance with the 'changed' set replaced.
    #[inline]
    #[must_use]
    pub fn with_changed(mut self, change_set: impl IntoIterator<Item = SymbolIdentifier>) -> Self {
        self.changed = change_set.into_iter().collect();
        self
    }

    /// Checks if the 'changed' set contains a specific entry.
    #[inline]
    #[must_use]
    pub fn contains_changed_entry(&self, entry: &SymbolIdentifier) -> bool {
        self.changed.contains(entry)
    }
}
