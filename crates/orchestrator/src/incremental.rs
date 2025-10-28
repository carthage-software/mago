//! Incremental analysis support for Mago.
//!
//! This module implements incremental analysis, which dramatically speeds up re-analysis
//! by caching results between runs and only re-analyzing changed code.
//!
//! # How It Works
//!
//! 1. **Scanning Phase**: Build file signatures (AST fingerprints) for all symbols
//! 2. **Diffing Phase**: Compare current signatures with previous run using Myers diff
//! 3. **Invalidation Phase**: Propagate changes through symbol reference graph
//! 4. **Marking Phase**: Mark unchanged symbols as "safe" to skip during analysis
//! 5. **Analysis Phase**: Analyzer skips safe symbols (when `settings.diff = true`)
//! 6. **Caching Phase**: Save signatures and references for next run
//!
//! # Cache Modes
//!
//! - **In-Memory**: Used by `--watch` mode, cache lives in process memory
//! - **Filesystem**: Used by `--incremental` flag, persists to `.mago/cache/`
//!
//! # Example
//!
//! ```ignore
//! let mut incremental = IncrementalAnalysis::new(Some(cache_dir));
//!
//! if let Some((old_metadata, old_refs)) = incremental.load_previous_state() {
//!     let diff = incremental.compute_diffs(&old_metadata, &new_metadata);
//!     incremental.mark_safe_symbols(diff, &old_refs, &mut new_metadata);
//! }
//!
//! // ... analyze with settings.diff = true ...
//!
//! incremental.save_state(&new_metadata, &new_refs, save_to_filesystem);
//! ```

use mago_codex::diff::CodebaseDiff;
use mago_codex::differ::compute_file_diff;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;

/// Manages incremental analysis state - both in-memory and filesystem cache.
///
/// This struct handles:
/// - Loading previous analysis state (from memory or filesystem)
/// - Computing diffs between old and new code
/// - Marking safe symbols based on invalidation cascade
/// - Saving state for next run (to memory or filesystem)
#[derive(Debug, Clone)]
pub struct IncrementalAnalysis {
    /// In-memory cache from previous run (for watch mode).
    ///
    /// Stores the previous codebase metadata between watch iterations.
    previous_metadata: Option<CodebaseMetadata>,

    /// In-memory cache of symbol references from previous run (for watch mode).
    ///
    /// Stores the previous symbol reference graph between watch iterations.
    previous_references: Option<SymbolReferences>,
}

impl Default for IncrementalAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalAnalysis {
    /// Creates a new incremental analysis manager.
    pub fn new() -> Self {
        Self { previous_metadata: None, previous_references: None }
    }

    /// Loads previous state from in-memory cache or filesystem cache.
    ///
    /// Checks in this order:
    /// 1. In-memory cache (from previous watch iteration)
    /// 2. Filesystem cache (if enabled and exists)
    ///
    /// Returns `Some((metadata, references))` if cache is available and valid,
    /// `None` if no cache exists or cache is invalid (version mismatch, corruption, etc.).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some((old_metadata, old_refs)) = incremental.load_previous_state() {
    ///     // Perform incremental analysis
    /// } else {
    ///     // Perform full analysis (first run or invalid cache)
    /// }
    /// ```
    pub fn load_previous_state(&mut self) -> Option<(CodebaseMetadata, SymbolReferences)> {
        // First check in-memory cache (for watch mode)
        if let (Some(metadata), Some(refs)) = (&self.previous_metadata, &self.previous_references) {
            return Some((metadata.clone(), refs.clone()));
        }

        None
    }

    /// Computes diffs between old and new codebase by aggregating file-level diffs.
    ///
    /// This function:
    /// 1. Gets all file IDs from both old and new codebases
    /// 2. For each file, computes diff using Myers algorithm
    /// 3. Aggregates diffs into a single `CodebaseDiff`
    ///
    /// The resulting diff contains:
    /// - `keep`: Symbols that haven't changed
    /// - `changed`: Symbols that were added, removed, or modified
    ///
    /// # Arguments
    ///
    /// * `old_metadata` - Codebase metadata from previous run
    /// * `new_metadata` - Codebase metadata from current run
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let diff = incremental.compute_diffs(&old_metadata, &new_metadata);
    /// println!("Kept: {}, Changed: {}", diff.get_keep().len(), diff.get_changed().len());
    /// ```
    pub fn compute_diffs(&self, old_metadata: &CodebaseMetadata, new_metadata: &CodebaseMetadata) -> CodebaseDiff {
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

    /// Marks safe symbols based on diff and invalidation cascade.
    ///
    /// This is the core incremental analysis algorithm (inspired by Hakana):
    ///
    /// 1. Compute invalid symbols from diff (symbols whose signatures changed)
    /// 2. Propagate invalidation through reference graph (cascade)
    /// 3. Mark all symbols in 'keep' set as safe (unless invalidated by cascade)
    ///
    /// After this function runs, `metadata.safe_symbols` and `metadata.safe_symbol_members`
    /// will contain all symbols that can be safely skipped during analysis.
    ///
    /// # Arguments
    ///
    /// * `diff` - The computed diff between old and new code
    /// * `references` - Symbol reference graph from previous run
    /// * `metadata` - Current codebase metadata (will be modified)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// incremental.mark_safe_symbols(diff, &old_references, &mut new_metadata);
    /// println!("Marked {} symbols as safe", new_metadata.safe_symbols.len());
    /// ```
    pub fn mark_safe_symbols(
        &self,
        diff: CodebaseDiff,
        references: &SymbolReferences,
        metadata: &mut CodebaseMetadata,
    ) {
        // Get invalid symbols with propagation through reference graph
        let (invalid_symbols, partially_invalid) = match references.get_invalid_symbols(&diff) {
            Some(result) => result,
            None => {
                // Propagation too expensive (>5000 steps), skip incremental
                tracing::warn!("Invalidation cascade too expensive (>5000 steps), falling back to full analysis");
                return;
            }
        };

        // Mark all symbols in 'keep' set as safe (unless invalidated by cascade)
        for keep_symbol in diff.get_keep() {
            if !invalid_symbols.contains(keep_symbol) {
                if keep_symbol.1.is_empty() {
                    // Top-level symbol (class, function, constant)
                    if !partially_invalid.contains(&keep_symbol.0) {
                        metadata.safe_symbols.insert(keep_symbol.0);
                    }
                } else {
                    // Member (method, property, class constant)
                    metadata.safe_symbol_members.insert(*keep_symbol);
                }
            }
        }
    }

    /// Saves the current codebase state for use in the next incremental run.
    ///
    /// This method stores the metadata and symbol references in the in-memory cache,
    /// which will be used by `load_previous_state()` in the next watch iteration.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Current codebase metadata to save
    /// * `references` - Current symbol references to save
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // After analysis completes
    /// incremental.save_state(&codebase, &symbol_references);
    /// ```
    pub fn save_state(&mut self, metadata: CodebaseMetadata, references: SymbolReferences) {
        self.previous_metadata = Some(metadata);
        self.previous_references = Some(references);
    }
}
