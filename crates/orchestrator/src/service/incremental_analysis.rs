use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use bumpalo::Bump;
use foldhash::HashMap;
use foldhash::HashSet;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_atom::AtomSet;
use mago_codex::diff::CodebaseDiff;
use mago_codex::metadata::CodebaseEntryKeys;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::populator::populate_codebase_targeted;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::settings::ParserSettings;
use rayon::prelude::*;

use crate::error::OrchestratorError;

/// Per-file cached state for incremental analysis.
#[derive(Debug, Clone)]
struct FileState {
    content_hash: u64,
    entry_keys: CodebaseEntryKeys,
    analysis_issues: IssueCollection,
    codebase_issues: IssueCollection,
}

/// A self-contained incremental analysis service.
///
/// This service manages all the state needed for incremental analysis:
///
/// - Base codebase (prelude/builtins only)
/// - Current fully-populated codebase
/// - File content hashes and cached per-file scan results
/// - Symbol reference graph
/// - Incremental diff/invalidation state
///
/// It provides two analysis modes:
///
/// - [`analyze()`](Self::analyze): Full analysis from scratch (used for initial run)
/// - [`analyze_incremental()`](Self::analyze_incremental): Incremental analysis that only re-scans changed files and skips analysis of unchanged symbols
pub struct IncrementalAnalysisService {
    database: ReadDatabase,
    codebase: CodebaseMetadata,
    symbol_references: SymbolReferences,
    base_codebase: Arc<CodebaseMetadata>,
    base_symbol_references: Arc<SymbolReferences>,
    settings: Settings,
    parser_settings: ParserSettings,
    file_states: HashMap<FileId, FileState>,
    plugin_registry: Arc<PluginRegistry>,
    initialized: bool,
    codebase_issues: IssueCollection,
}

impl std::fmt::Debug for IncrementalAnalysisService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncrementalAnalysisService")
            .field("database", &"<ReadDatabase>")
            .field("settings", &self.settings)
            .field("parser_settings", &self.parser_settings)
            .field("file_states", &format!("{} tracked files", self.file_states.len()))
            .field("initialized", &self.initialized)
            .field("codebase_issues", &self.codebase_issues.len())
            .finish()
    }
}

impl IncrementalAnalysisService {
    /// Creates a new incremental analysis service with the given database, base codebase, and settings.
    ///
    /// The provided `codebase` and `symbol_references` should contain only prelude/builtin
    /// data (no user symbols). They will be used as the base for every analysis run.
    ///
    /// # Arguments
    ///
    /// * `database` - Read-only file database
    /// * `codebase` - Base codebase metadata (prelude only)
    /// * `symbol_references` - Base symbol references (prelude only)
    /// * `settings` - Analyzer settings
    /// * `parser_settings` - Parser settings
    /// * `plugin_registry` - Analyzer plugin registry
    #[must_use]
    pub fn new(
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        settings: Settings,
        parser_settings: ParserSettings,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        let base_codebase = Arc::new(codebase.clone());
        let base_symbol_references = Arc::new(symbol_references.clone());

        Self {
            database,
            codebase,
            symbol_references,
            base_codebase,
            base_symbol_references,
            settings,
            parser_settings,
            file_states: HashMap::default(),
            plugin_registry,
            initialized: false,
            codebase_issues: IssueCollection::default(),
        }
    }

    /// Updates the database for a new analysis run.
    ///
    /// Call this before [`analyze_incremental()`](Self::analyze_incremental) with the
    /// updated database that reflects file changes.
    pub fn update_database(&mut self, database: ReadDatabase) {
        self.database = database;
    }

    /// Returns whether the service has been initialized (initial full analysis completed).
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns a reference to the current codebase metadata.
    #[must_use]
    pub fn codebase(&self) -> &CodebaseMetadata {
        &self.codebase
    }

    /// Returns a reference to the current symbol references.
    #[must_use]
    pub fn symbol_references(&self) -> &SymbolReferences {
        &self.symbol_references
    }

    /// Returns a reference to the current database.
    #[must_use]
    pub fn database(&self) -> &ReadDatabase {
        &self.database
    }

    /// Reconstructs the full issue list from cached per-file issues and codebase-level issues.
    ///
    /// Returns `None` if no analysis has been run yet.
    #[must_use]
    pub fn last_issues(&self) -> Option<IssueCollection> {
        if !self.initialized {
            return None;
        }

        Some(self.collect_all_issues())
    }

    /// Returns cached per-file diagnostics for a specific file.
    ///
    /// This returns the issues from the last completed analysis run for the given file,
    /// without re-analyzing. Returns `None` if the file is not tracked or no analysis
    /// has been run yet.
    #[must_use]
    pub fn get_file_diagnostics(&self, file_id: &FileId) -> Option<&IssueCollection> {
        self.file_states.get(file_id).map(|state| &state.analysis_issues)
    }

    /// Returns the number of tracked source files.
    #[must_use]
    pub fn tracked_file_count(&self) -> usize {
        self.file_states.len()
    }

    /// Distributes codebase-level issues into per-file caches based on their primary annotation's file ID.
    ///
    /// Issues that cannot be attributed to a tracked file (no primary annotation, zero file ID,
    /// or file not in `file_states`) are stored in `self.codebase_issues` as orphans.
    fn distribute_codebase_issues(&mut self, issues: IssueCollection) {
        for issue in issues {
            let file_id = issue
                .annotations
                .iter()
                .find(|a| a.kind.is_primary())
                .map(|a| a.span.file_id)
                .filter(|fid| !fid.is_zero());

            if let Some(fid) = file_id
                && let Some(state) = self.file_states.get_mut(&fid)
            {
                state.codebase_issues.push(issue);
                continue;
            }

            self.codebase_issues.push(issue);
        }
    }

    /// Assembles the full issue list from codebase-level issues + all per-file cached issues.
    ///
    /// This avoids storing a separate snapshot by reconstructing on demand.
    fn collect_all_issues(&self) -> IssueCollection {
        let total: usize = self.codebase_issues.len()
            + self.file_states.values().map(|s| s.analysis_issues.len() + s.codebase_issues.len()).sum::<usize>();

        let mut issues = IssueCollection::default();
        issues.reserve(total);
        issues.extend(self.codebase_issues.iter().cloned());
        for state in self.file_states.values() {
            if !state.analysis_issues.is_empty() {
                issues.extend(state.analysis_issues.iter().cloned());
            }

            if !state.codebase_issues.is_empty() {
                issues.extend(state.codebase_issues.iter().cloned());
            }
        }

        issues
    }

    /// Runs a full analysis from scratch.
    ///
    /// After this call, [`analyze_incremental()`](Self::analyze_incremental) can be used
    /// for subsequent runs.
    pub fn analyze(&mut self) -> Result<AnalysisResult, OrchestratorError> {
        let source_files: Vec<_> = self.database.files().filter(|f| f.file_type != FileType::Builtin).collect();

        if source_files.is_empty() {
            tracing::info!("No source files found for analysis.");
            self.initialized = true;
            return Ok(AnalysisResult::new(SymbolReferences::new()));
        }

        let parser_settings = self.parser_settings;
        let per_file_results: Vec<(FileId, u64, CodebaseMetadata)> = source_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| {
                let content_hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());

                let program = parse_file_with_settings(arena, &file, parser_settings);
                if program.has_errors() {
                    tracing::warn!(
                        "Encountered {} parsing error(s) in '{}'. Codebase analysis may be incomplete.",
                        program.errors.len(),
                        file.name,
                    );
                }

                let resolver = NameResolver::new(arena);
                let resolved_names = resolver.resolve(program);

                let file_signature = signature_builder::build_file_signature(&file, program, &resolved_names);
                let mut metadata = scan_program(arena, &file, program, &resolved_names);
                metadata.set_file_signature(file.id, file_signature);

                arena.reset();

                (file.id, content_hash, metadata)
            })
            .collect();

        let mut merged_codebase = (*self.base_codebase).clone();
        let mut file_states = HashMap::default();

        for (file_id, content_hash, metadata) in per_file_results {
            let entry_keys = metadata.extract_keys();
            merged_codebase.extend(metadata);
            file_states.insert(
                file_id,
                FileState {
                    content_hash,
                    entry_keys,
                    analysis_issues: IssueCollection::default(),
                    codebase_issues: IssueCollection::default(),
                },
            );
        }

        let mut symbol_references = (*self.base_symbol_references).clone();
        populate_codebase(&mut merged_codebase, &mut symbol_references, AtomSet::default(), HashSet::default());

        let (mut analysis_result, per_file_issues) =
            self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, &HashSet::default())?;

        let all_codebase_issues = merged_codebase.take_issues(true);
        analysis_result.issues.extend(all_codebase_issues.iter().cloned());

        // Distribute codebase issues to per-file caches.
        self.file_states = file_states;
        self.codebase_issues = IssueCollection::default();
        self.distribute_codebase_issues(all_codebase_issues);

        for (file_id, issues) in per_file_issues {
            analysis_result.issues.extend(issues.iter().cloned());
            if let Some(state) = self.file_states.get_mut(&file_id) {
                state.analysis_issues = issues;
            }
        }

        tracing::debug!(
            "Initial analysis: {} total issues ({} codebase-level)",
            analysis_result.issues.len(),
            self.codebase_issues.len(),
        );

        self.codebase = merged_codebase;
        self.symbol_references = std::mem::take(&mut analysis_result.symbol_references);
        self.initialized = true;

        Ok(analysis_result)
    }

    /// Runs incremental analysis optimized for subsequent runs after file changes.
    ///
    /// # Arguments
    ///
    /// * `changed_hint` - Optional set of file IDs known to have changed (e.g. from
    ///   a file watcher). When provided, only these files are hashed to detect changes.
    ///   Files not in this set are assumed unchanged, avoiding O(all files) hashing.
    ///   Pass `None` to hash all files (correct but slower).
    ///
    /// # Panics
    ///
    /// Panics if called before [`analyze()`](Self::analyze) has been run at least once.
    pub fn analyze_incremental(
        &mut self,
        changed_hint: Option<&[FileId]>,
    ) -> Result<AnalysisResult, OrchestratorError> {
        assert!(self.initialized, "analyze() must be called before analyze_incremental()");

        let source_files: Vec<_> = self.database.files().filter(|f| f.file_type != FileType::Builtin).collect();

        if source_files.is_empty() {
            tracing::info!("No source files found for analysis.");
            return Ok(AnalysisResult::new(SymbolReferences::new()));
        }

        let mut changed_files = Vec::new();
        let mut unchanged_file_ids = Vec::with_capacity(source_files.len());
        let mut file_hashes: HashMap<FileId, u64> = HashMap::default();

        // Detect deleted files (present in cache but no longer in database)
        let current_file_ids: HashSet<FileId> = source_files.iter().map(|f| f.id).collect();
        let deleted_count = self.file_states.keys().filter(|id| !current_file_ids.contains(id)).count();
        if deleted_count > 0 {
            tracing::debug!("{} file(s) deleted since last run", deleted_count);
        }

        if let Some(hint) = changed_hint {
            let hint_set: HashSet<FileId> = hint.iter().copied().collect();

            for file in &source_files {
                if hint_set.contains(&file.id) || !self.file_states.contains_key(&file.id) {
                    let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());
                    file_hashes.insert(file.id, hash);

                    if let Some(prev_state) = self.file_states.get(&file.id) {
                        if prev_state.content_hash == hash {
                            unchanged_file_ids.push(file.id);
                            tracing::trace!("File unchanged (hash match despite watcher hint): {}", file.name);
                        } else {
                            changed_files.push(file);
                            tracing::debug!("File changed: {}", file.name);
                        }
                    } else {
                        changed_files.push(file);
                        tracing::debug!("New file: {}", file.name);
                    }
                } else {
                    if let Some(prev_state) = self.file_states.get(&file.id) {
                        file_hashes.insert(file.id, prev_state.content_hash);
                    }

                    unchanged_file_ids.push(file.id);
                }
            }
        } else {
            let all_hashes: HashMap<FileId, u64> = source_files
                .par_iter()
                .map(|file| {
                    let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());
                    (file.id, hash)
                })
                .collect();

            for file in &source_files {
                let current_hash = all_hashes[&file.id];
                file_hashes.insert(file.id, current_hash);

                if let Some(prev_state) = self.file_states.get(&file.id) {
                    if prev_state.content_hash == current_hash {
                        unchanged_file_ids.push(file.id);
                        tracing::trace!("File unchanged (cached): {}", file.name);
                    } else {
                        changed_files.push(file);
                        tracing::debug!("File changed (hash mismatch): {}", file.name);
                    }
                } else {
                    changed_files.push(file);
                    tracing::debug!("New file (not in cache): {}", file.name);
                }
            }
        }

        if changed_files.is_empty() && deleted_count == 0 {
            tracing::debug!("No files changed, reconstructing cached issues");
            let mut result = AnalysisResult::new(SymbolReferences::new());
            result.issues = self.collect_all_issues();
            return Ok(result);
        }

        let parser_settings = self.parser_settings;
        let new_file_scans: Vec<(FileId, CodebaseMetadata)> = changed_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| {
                let program = parse_file_with_settings(arena, file, parser_settings);
                if program.has_errors() {
                    tracing::warn!(
                        "Encountered {} parsing error(s) in '{}'. Codebase analysis may be incomplete.",
                        program.errors.len(),
                        file.name,
                    );
                }

                let resolver = NameResolver::new(arena);
                let resolved_names = resolver.resolve(program);

                let mut metadata = scan_program(arena, file, program, &resolved_names);
                metadata.set_file_signature(
                    file.id,
                    signature_builder::build_file_signature(file, program, &resolved_names),
                );

                arena.reset();

                (file.id, metadata)
            })
            .collect();

        let mut diff = {
            let mut diff = CodebaseDiff::new();
            let mut old_sigs = CodebaseMetadata::new();
            let mut new_sigs = CodebaseMetadata::new();

            for (file_id, metadata) in &new_file_scans {
                if let Some(old_sig) = self.codebase.get_file_signature(file_id) {
                    old_sigs.set_file_signature(*file_id, old_sig.clone());
                }
                if let Some(new_sig) = metadata.file_signatures.get(file_id) {
                    new_sigs.set_file_signature(*file_id, new_sig.clone());
                }
            }

            for &file_id in self.file_states.keys() {
                if !current_file_ids.contains(&file_id)
                    && let Some(sig) = self.codebase.get_file_signature(&file_id)
                {
                    old_sigs.set_file_signature(file_id, sig.clone());
                }
            }

            diff.extend(CodebaseDiff::between(&old_sigs, &new_sigs));
            diff
        };

        let body_only = diff.get_changed().is_empty() && deleted_count == 0;

        if !body_only {
            for &file_id in &unchanged_file_ids {
                let Some(sig) = self.codebase.get_file_signature(&file_id) else {
                    continue;
                };

                for node in &sig.ast_nodes {
                    diff.add_keep_entry((node.name, mago_atom::empty_atom()));

                    for child in &node.children {
                        diff.add_keep_entry((node.name, child.name));
                    }
                }
            }
        }

        if body_only {
            let mut merged_codebase = std::mem::take(&mut self.codebase);
            // Two-pass update: remove all old entries first, then add all new entries.
            for (file_id, _new_metadata) in &new_file_scans {
                if let Some(prev_state) = self.file_states.get(file_id) {
                    merged_codebase.remove_entries_by_keys(&prev_state.entry_keys);
                }
            }
            for (_file_id, new_metadata) in &new_file_scans {
                merged_codebase.extend_ref(new_metadata);
            }

            let files_to_skip: HashSet<FileId> = unchanged_file_ids.iter().copied().collect();
            let mut symbol_references = std::mem::take(&mut self.symbol_references);

            let mut changed_symbols: HashSet<(mago_atom::Atom, mago_atom::Atom)> = HashSet::default();
            let mut changed_file_names: Vec<mago_atom::Atom> = Vec::new();

            for (file_id, metadata) in &new_file_scans {
                for &key in metadata.function_likes.keys() {
                    changed_symbols.insert(key);
                }

                for &name in metadata.class_likes.keys() {
                    changed_symbols.insert((name, mago_atom::empty_atom()));
                }

                for &name in metadata.constants.keys() {
                    changed_symbols.insert((name, mago_atom::empty_atom()));
                }

                if let Some(sig) = metadata.file_signatures.values().next() {
                    for node in &sig.ast_nodes {
                        for child in &node.children {
                            changed_symbols.insert((node.name, child.name));
                        }
                    }
                }

                // Collect file names for file-level reference cleanup
                if let Ok(file) = self.database.get(file_id) {
                    changed_file_names.push(mago_atom::atom(&file.name));
                }
            }

            symbol_references.remove_body_references_for_symbols(&changed_symbols, &changed_file_names);

            // Collect class_like names from changed files so we exclude them from safe_symbols.
            // Changed classes had their old metadata removed and fresh (unpopulated) metadata added,
            // so the populator must repopulate them to rebuild parent resolution, overridden_method_ids, etc.
            let changed_class_like_names: AtomSet =
                new_file_scans.iter().flat_map(|(_, metadata)| metadata.class_likes.keys().copied()).collect();
            let safe_symbols: AtomSet = merged_codebase
                .class_likes
                .keys()
                .copied()
                .filter(|name| !changed_class_like_names.contains(name))
                .collect();
            populate_codebase_targeted(
                &mut merged_codebase,
                &mut symbol_references,
                safe_symbols,
                HashSet::default(),
                changed_symbols,
            );

            let (mut analysis_result, mut per_file_issues) =
                self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, &files_to_skip)?;

            // Drain codebase issues from metadata and distribute to changed files only.
            // Unchanged files keep their cached codebase_issues from previous runs.
            let new_codebase_issues = merged_codebase.take_issues(true);
            let changed_file_ids: HashSet<FileId> = new_file_scans.iter().map(|(fid, _)| *fid).collect();

            // Clear codebase issues for changed files (they'll be re-distributed below).
            for (file_id, state) in self.file_states.iter_mut() {
                if changed_file_ids.contains(file_id) {
                    state.codebase_issues = IssueCollection::default();
                }
            }

            // Keep orphan codebase issues; distribute file-attributable ones.
            self.codebase_issues = IssueCollection::default();
            self.distribute_codebase_issues(new_codebase_issues);

            for (file_id, metadata) in new_file_scans {
                let content_hash = file_hashes[&file_id];
                let analysis_issues = per_file_issues.remove(&file_id).unwrap_or_default();
                let entry_keys = metadata.extract_keys();
                let codebase_issues =
                    self.file_states.get(&file_id).map(|s| s.codebase_issues.clone()).unwrap_or_default();
                self.file_states
                    .insert(file_id, FileState { content_hash, entry_keys, analysis_issues, codebase_issues });
            }

            self.codebase = merged_codebase;
            self.symbol_references = std::mem::take(&mut analysis_result.symbol_references);
            analysis_result.issues = self.collect_all_issues();

            return Ok(analysis_result);
        }

        let mut merged_codebase = std::mem::take(&mut self.codebase);

        for (file_id, prev_state) in &self.file_states {
            if !current_file_ids.contains(file_id) {
                merged_codebase.remove_entries_by_keys(&prev_state.entry_keys);
            }
        }

        // Two-pass update: remove all old entries first, then add all new entries.
        // This prevents ordering issues where adding a class from a new file and then
        // removing old entries from a different file could accidentally remove the
        // newly-added class (when a class moves between files).
        for (file_id, _new_metadata) in &new_file_scans {
            if let Some(prev_state) = self.file_states.get(file_id) {
                merged_codebase.remove_entries_by_keys(&prev_state.entry_keys);
            }
        }
        for (_file_id, new_metadata) in &new_file_scans {
            merged_codebase.extend_ref(new_metadata);
        }

        merged_codebase.safe_symbols.clear();
        merged_codebase.safe_symbol_members.clear();

        if !merged_codebase.mark_safe_symbols(&diff, &self.symbol_references) {
            tracing::warn!("Invalidation cascade too expensive (>5000 steps), falling back to full analysis");

            return self.analyze();
        }

        // Ensure classes that depend on changed class_likes are not marked safe.
        // This handles cases where reference graph edges were lost (e.g., a parent
        // class was deleted and later re-added — the child→parent reference edge
        // was removed when the parent was deleted, so the cascade can't reach the child).
        {
            let changed_class_like_names: AtomSet = diff
                .get_changed()
                .iter()
                .filter(|key| key.1.is_empty() && merged_codebase.class_likes.contains_key(&key.0))
                .map(|key| key.0)
                .collect();

            if !changed_class_like_names.is_empty() {
                let to_unsafify: Vec<mago_atom::Atom> = merged_codebase
                    .class_likes
                    .iter()
                    .filter(|(name, _)| merged_codebase.safe_symbols.contains(name))
                    .filter(|(_, metadata)| {
                        metadata.direct_parent_class.is_some_and(|p| changed_class_like_names.contains(&p))
                            || metadata.direct_parent_interfaces.iter().any(|i| changed_class_like_names.contains(i))
                            || metadata.used_traits.iter().any(|t| changed_class_like_names.contains(t))
                    })
                    .map(|(name, _)| *name)
                    .collect();

                for name in to_unsafify {
                    merged_codebase.safe_symbols.remove(&name);
                }
            }
        }

        let safe_symbols = std::mem::take(&mut merged_codebase.safe_symbols);
        let safe_symbol_members = std::mem::take(&mut merged_codebase.safe_symbol_members);

        let mut dirty_symbols: HashSet<(mago_atom::Atom, mago_atom::Atom)> = diff.get_changed().clone();
        for (_file_id, metadata) in &new_file_scans {
            for &key in metadata.function_likes.keys() {
                dirty_symbols.insert(key);
            }

            for &name in metadata.class_likes.keys() {
                dirty_symbols.insert((name, mago_atom::empty_atom()));
            }

            for &name in metadata.constants.keys() {
                dirty_symbols.insert((name, mago_atom::empty_atom()));
            }
        }

        let mut symbol_references = std::mem::take(&mut self.symbol_references);
        symbol_references.remove_dirty_symbol_references(&dirty_symbols);

        populate_codebase_targeted(
            &mut merged_codebase,
            &mut symbol_references,
            safe_symbols,
            safe_symbol_members,
            dirty_symbols,
        );

        let mut files_to_skip: HashSet<FileId> = HashSet::default();
        for &file_id in &unchanged_file_ids {
            if let Some(sig) = merged_codebase.get_file_signature(&file_id) {
                let all_safe = sig.ast_nodes.iter().all(|node| {
                    let symbol_safe = node.name.is_empty() || merged_codebase.safe_symbols.contains(&node.name);
                    let children_safe = node.children.iter().all(|child| {
                        child.name.is_empty() || merged_codebase.safe_symbol_members.contains(&(node.name, child.name))
                    });
                    symbol_safe && children_safe
                });

                if all_safe {
                    files_to_skip.insert(file_id);
                }
            }
        }

        // For files that will be re-analyzed (not skipped), remove their symbols from
        // safe_symbols/safe_symbol_members. When diff=true, the analyzer skips safe classes,
        // but since we replace per-file issues wholesale, skipped classes would lose their issues.
        // Only files in files_to_skip keep their cached issues; all others are fully re-analyzed.
        {
            let mut symbols_to_unsafify = Vec::new();
            let mut members_to_unsafify = Vec::new();

            for &file_id in &unchanged_file_ids {
                if !files_to_skip.contains(&file_id)
                    && let Some(sig) = merged_codebase.get_file_signature(&file_id)
                {
                    for node in &sig.ast_nodes {
                        symbols_to_unsafify.push(node.name);
                        for child in &node.children {
                            members_to_unsafify.push((node.name, child.name));
                        }
                    }
                }
            }

            // Also clear for changed files (they are always re-analyzed).
            for (_file_id, metadata) in &new_file_scans {
                for &name in metadata.class_likes.keys() {
                    symbols_to_unsafify.push(name);
                }

                // Use file signature nodes (not metadata keys) for consistency with
                // how unchanged files are unsafified. File signature nodes use
                // (name, empty) for top-level symbols, matching safe_symbols format.
                // This differs from function_likes keys which use (empty, name) for
                // standalone functions.
                for sig in metadata.file_signatures.values() {
                    for node in &sig.ast_nodes {
                        symbols_to_unsafify.push(node.name);
                        for child in &node.children {
                            members_to_unsafify.push((node.name, child.name));
                        }
                    }
                }
            }

            for name in symbols_to_unsafify {
                merged_codebase.safe_symbols.remove(&name);
            }
            for key in members_to_unsafify {
                merged_codebase.safe_symbol_members.remove(&key);
            }
        }

        let (mut analysis_result, mut per_file_issues) =
            self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, &files_to_skip)?;

        let new_codebase_issues = merged_codebase.take_issues(true);
        self.file_states.retain(|id, _| current_file_ids.contains(id));

        let changed_file_ids: HashSet<FileId> = new_file_scans.iter().map(|(fid, _)| *fid).collect();
        let re_analyzed_unchanged: Vec<FileId> = per_file_issues
            .keys()
            .filter(|fid| self.file_states.contains_key(fid) && !changed_file_ids.contains(fid))
            .copied()
            .collect();

        // Clear codebase issues for files that were NOT skipped (their symbols were re-populated).
        // Files in files_to_skip keep their old codebase_issues intact.
        for (file_id, state) in self.file_states.iter_mut() {
            if !files_to_skip.contains(file_id) {
                state.codebase_issues = IssueCollection::default();
            }
        }

        // Distribute new codebase issues (from re-populated symbols) to per-file caches.
        self.codebase_issues = IssueCollection::default();
        self.distribute_codebase_issues(new_codebase_issues);

        for file_id in re_analyzed_unchanged {
            if let Some(issues) = per_file_issues.remove(&file_id)
                && let Some(state) = self.file_states.get_mut(&file_id)
            {
                state.analysis_issues = issues;
            }
        }

        for (file_id, metadata) in new_file_scans {
            let content_hash = file_hashes[&file_id];
            let analysis_issues = per_file_issues.remove(&file_id).unwrap_or_default();
            let entry_keys = metadata.extract_keys();
            let codebase_issues = self.file_states.get(&file_id).map(|s| s.codebase_issues.clone()).unwrap_or_default();
            self.file_states.insert(file_id, FileState { content_hash, entry_keys, analysis_issues, codebase_issues });
        }

        self.codebase = merged_codebase;
        self.symbol_references = std::mem::take(&mut analysis_result.symbol_references);

        analysis_result.issues = self.collect_all_issues();

        Ok(analysis_result)
    }

    /// Analyzes a single file synchronously, returning its issues.
    ///
    /// This is useful for LSP single-file analysis. It uses the current codebase
    /// state for type resolution but only reports issues for the specified file.
    pub fn analyze_file(&self, file_id: FileId) -> IssueCollection {
        let Ok(file) = self.database.get(&file_id) else {
            tracing::error!("File with ID {:?} not found in database", file_id);
            return IssueCollection::default();
        };

        let arena = Bump::new();
        let program = parse_file_with_settings(&arena, &file, self.parser_settings);
        let resolved_names = NameResolver::new(&arena).resolve(program);

        let mut issues = IssueCollection::new();
        if program.has_errors() {
            for error in program.errors.iter() {
                issues.push(Issue::from(error));
            }
        }

        let semantics_checker = SemanticsChecker::new(self.settings.version);
        issues.extend(semantics_checker.check(&file, program, &resolved_names));

        let mut analysis_result = AnalysisResult::new(SymbolReferences::new());
        let analyzer =
            Analyzer::new(&arena, &file, &resolved_names, &self.codebase, &self.plugin_registry, self.settings.clone());

        if let Err(err) = analyzer.analyze(program, &mut analysis_result) {
            issues.push(Issue::error(format!("Analysis error: {err}")));
        }

        issues.extend(analysis_result.issues);
        issues
    }

    /// Runs the analyzer on host files.
    ///
    /// Returns `(aggregated_result, per_file_issues)` where `per_file_issues` contains
    /// only the files that were actually analyzed (not skipped).
    fn run_analyzer_selective(
        &self,
        codebase: &CodebaseMetadata,
        current_symbol_references: SymbolReferences,
        settings: &Settings,
        skip_files: &HashSet<FileId>,
    ) -> Result<(AnalysisResult, HashMap<FileId, IssueCollection>), OrchestratorError> {
        #[cfg(not(target_arch = "wasm32"))]
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);

        let host_files: Vec<_> = self
            .database
            .files()
            .filter(|f| f.file_type == FileType::Host && !skip_files.contains(&f.id))
            .map(|f| self.database.get(&f.id))
            .collect::<Result<Vec<_>, _>>()?;

        if host_files.is_empty() && skip_files.is_empty() {
            tracing::warn!("No host files found for analysis.");

            return Ok((AnalysisResult::new(SymbolReferences::new()), HashMap::default()));
        }

        let plugin_registry = &self.plugin_registry;
        let settings = settings.clone();
        let parser_settings = self.parser_settings;

        let results: Vec<(FileId, AnalysisResult)> = host_files
            .into_par_iter()
            .map_init(Bump::new, |arena, source_file| {
                let file_id = source_file.id;
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                let program = parse_file_with_settings(arena, &source_file, parser_settings);
                let resolved_names = NameResolver::new(arena).resolve(program);

                if program.has_errors() {
                    analysis_result.issues.extend(program.errors.iter().map(Issue::from));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);
                let analyzer =
                    Analyzer::new(arena, &source_file, &resolved_names, codebase, plugin_registry, settings.clone());

                analysis_result.issues.extend(semantics_checker.check(&source_file, program, &resolved_names));
                analyzer.analyze(program, &mut analysis_result)?;

                #[cfg(not(target_arch = "wasm32"))]
                if analysis_result.time_in_analysis > ANALYSIS_DURATION_THRESHOLD {
                    tracing::warn!(
                        "Analysis of source file '{}' took longer than {}s: {}s",
                        source_file.name,
                        ANALYSIS_DURATION_THRESHOLD.as_secs_f32(),
                        analysis_result.time_in_analysis.as_secs_f32()
                    );
                }

                arena.reset();

                Ok((file_id, analysis_result))
            })
            .collect::<Result<Vec<_>, OrchestratorError>>()?;

        let mut aggregated_result = AnalysisResult::new(current_symbol_references);
        let mut per_file_issues: HashMap<FileId, IssueCollection> = HashMap::default();

        for (file_id, result) in results {
            aggregated_result.symbol_references.extend(result.symbol_references);
            per_file_issues.insert(file_id, result.issues);
        }

        Ok((aggregated_result, per_file_issues))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::borrow::Cow;
    use std::path::Path;
    use std::sync::LazyLock;

    use mago_analyzer::plugin::PluginRegistry;
    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;

    static PLUGIN_REGISTRY: LazyLock<Arc<PluginRegistry>> =
        LazyLock::new(|| Arc::new(PluginRegistry::with_library_providers()));

    fn make_database(files: Vec<(&str, &str)>) -> Database<'static> {
        let config = DatabaseConfiguration {
            workspace: Cow::Owned(Path::new("/test").to_path_buf()),
            paths: vec![Cow::Borrowed("src")],
            includes: vec![],
            excludes: vec![],
            extensions: vec![Cow::Borrowed("php")],
        };

        let mut db = Database::new(config);
        for (name, contents) in files {
            db.add(File::new(Cow::Owned(name.to_string()), FileType::Host, None, Cow::Owned(contents.to_string())));
        }

        db
    }

    fn make_service(db: &Database<'_>) -> IncrementalAnalysisService {
        IncrementalAnalysisService::new(
            db.read_only(),
            CodebaseMetadata::new(),
            SymbolReferences::new(),
            Settings::default(),
            ParserSettings::default(),
            Arc::clone(&PLUGIN_REGISTRY),
        )
    }

    fn diff_issues(a: &IssueCollection, b: &IssueCollection) -> (Vec<String>, Vec<String>) {
        let sig = |i: &Issue| -> String {
            let span_info = i
                .annotations
                .iter()
                .find(|ann| ann.kind == mago_reporting::AnnotationKind::Primary)
                .map(|ann| format!("{:?}:{:?}", ann.span.file_id, ann.span.start))
                .unwrap_or_default();
            format!("{}:{}:{}", i.code.as_deref().unwrap_or("?"), span_info, i.message)
        };

        let sigs_a: std::collections::HashSet<String> = a.iter().map(sig).collect();
        let sigs_b: std::collections::HashSet<String> = b.iter().map(sig).collect();

        let only_a: Vec<_> = sigs_a.difference(&sigs_b).cloned().collect();
        let only_b: Vec<_> = sigs_b.difference(&sigs_a).cloned().collect();

        (only_a, only_b)
    }

    #[test]
    fn test_initial_analysis_runs_successfully() {
        let db = make_database(vec![(
            "src/main.php",
            "<?php\nfunction greet(string $name): string { return 'Hello ' . $name; }\n",
        )]);

        let mut service = make_service(&db);
        service.analyze().expect(
            "Initial analysis failed when it should have succeeded. Check that the parser and analyzer can handle basic PHP code."
        );
        assert!(service.is_initialized());
    }

    #[test]
    fn test_incremental_no_change_returns_same_result() {
        let db = make_database(vec![
            ("src/a.php", "<?php\nfunction foo(): int { return 42; }\n"),
            ("src/b.php", "<?php\nfunction bar(): string { return 'hello'; }\n"),
        ]);

        let mut service = make_service(&db);
        let initial = service.analyze().expect("Full analysis failed.");

        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        assert_eq!(initial.issues.len(), incremental.issues.len());
    }

    #[test]
    fn test_incremental_after_body_change_matches_full() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction get_value(): int { return 42; }\n"),
            ("src/b.php", "<?php\nfunction use_value(): int { return get_value(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction get_value(): int { return 99; }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Incremental and full analysis produced different issues.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_after_signature_change_matches_full() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction compute(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction caller(): int { return compute(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(): string { return 'hello'; }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After signature change, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_after_file_deletion_matches_full() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction helper(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction main_func(): int { return helper(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.delete(FileId::new("src/a.php"));

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After file deletion, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_after_file_addition_matches_full() {
        let mut db = make_database(vec![("src/a.php", "<?php\nfunction existing(): int { return 1; }\n")]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.add(File::new(
            Cow::Owned("src/b.php".to_string()),
            FileType::Host,
            None,
            Cow::Owned("<?php\nfunction new_func(): int { return existing(); }\n".to_string()),
        ));

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis after file addition failed");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis after file addition failed");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After file addition, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_revert_matches_full() {
        let original_content = "<?php\nfunction get_value(): int { return 42; }\n";
        let modified_content = "<?php\nfunction get_value(): string { return 'hello'; }\n";

        let mut db = make_database(vec![
            ("src/a.php", original_content),
            ("src/b.php", "<?php\nfunction use_it(): int { return get_value(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(FileId::new("src/a.php"), Cow::Owned(modified_content.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("incremental analysis after modification failed");

        db.update(FileId::new("src/a.php"), Cow::Owned(original_content.to_string()));
        service.update_database(db.read_only());
        let reverted = service.analyze_incremental(None).expect("incremental analysis after revert failed");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("full analysis after revert failed");

        let (only_incr, only_full) = diff_issues(&reverted.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After revert, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_multiple_cycles_consistent() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction compute(int $x): int { return $x * 2; }\n"),
            ("src/b.php", "<?php\nfunction wrapper(): int { return compute(5); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(int $x): int { return $x * 3; }\n".to_string()),
        );
        service.update_database(db.read_only());
        let cycle1 = service.analyze_incremental(None).expect("First incremental analysis failed");
        let cycle2 = service.analyze_incremental(None).expect("Second incremental analysis failed");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(int $x): string { return (string)($x * 3); }\n".to_string()),
        );
        service.update_database(db.read_only());
        let cycle3 = service.analyze_incremental(None).expect("Third incremental analysis failed");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis after multiple cycles failed");

        let (only_incr, only_full) = diff_issues(&cycle3.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After multiple cycles, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        assert_eq!(cycle1.issues.len(), cycle2.issues.len(), "No-change cycle should produce same issue count");
    }

    #[test]
    fn test_incremental_class_inheritance_change() {
        let parent_v1 = concat!(
            "<?php\n",
            "class Animal {\n",
            "    public function speak(): string { return 'generic'; }\n",
            "}\n",
        );
        let child = concat!(
            "<?php\n",
            "class Dog extends Animal {\n",
            "    public function bark(): string { return $this->speak(); }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Animal.php", parent_v1), ("src/Dog.php", child)]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        let parent_v2 =
            concat!("<?php\n", "class Animal {\n", "    public function speak(): int { return 42; }\n", "}\n",);
        db.update(FileId::new("src/Animal.php"), Cow::Owned(parent_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis after class change failed");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis after class change failed");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Class inheritance change: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_type_error_introduced_and_fixed() {
        let a_valid = "<?php\nfunction get_count(): int { return 42; }\n";
        let b_code = "<?php\nfunction print_count(): void { echo get_count() + 1; }\n";

        let mut db = make_database(vec![("src/a.php", a_valid), ("src/b.php", b_code)]);

        let mut service = make_service(&db);
        let initial = service.analyze().expect("Initial analysis failed.");
        let initial_count = initial.issues.len();

        let a_broken = "<?php\nfunction get_count(): string { return 'not a number'; }\n";
        db.update(FileId::new("src/a.php"), Cow::Owned(a_broken.to_string()));
        service.update_database(db.read_only());
        let broken = service.analyze_incremental(None).expect("Incremental analysis after breaking change failed");

        let mut fresh_service = make_service(&db);
        let full_broken = fresh_service.analyze().expect("Full analysis after breaking change failed");
        let (only_incr, only_full) = diff_issues(&broken.issues, &full_broken.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Broken state: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        db.update(FileId::new("src/a.php"), Cow::Owned(a_valid.to_string()));
        service.update_database(db.read_only());
        let fixed = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service2 = make_service(&db);
        let full_fixed = fresh_service2.analyze().expect("Full analysis failed.");
        let (only_incr, only_full) = diff_issues(&fixed.issues, &full_fixed.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Fixed state: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        assert_eq!(initial_count, fixed.issues.len(), "After fix, issue count should match initial");
    }

    #[test]
    fn test_incremental_multiple_files_changed_simultaneously() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction alpha(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction beta(): int { return 2; }\n"),
            ("src/c.php", "<?php\nfunction gamma(): int { return alpha() + beta(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        // Change both a.php and b.php at the same time
        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction alpha(): string { return 'a'; }\n".to_string()),
        );
        db.update(FileId::new("src/b.php"), Cow::Owned("<?php\nfunction beta(): string { return 'b'; }\n".to_string()));

        service.update_database(db.read_only());
        let incremental =
            service.analyze_incremental(None).expect("Incremental analysis after multiple simultaneous changes failed");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis after multiple simultaneous changes failed");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Multiple simultaneous changes: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_parse_error_recovery() {
        let mut db = make_database(vec![("src/a.php", "<?php\nfunction valid(): int { return 1; }\n")]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction broken( { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let with_error = service.analyze_incremental(None).expect("Incremental analysis failed.");

        // Should have at least one parse error
        assert!(!with_error.issues.is_empty(), "Parse error should produce at least one issue");

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction valid(): int { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let fixed = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&fixed.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After parse error recovery: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_symbol_added_to_existing_file() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction first(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction caller(): int { return first(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned(
                "<?php\nfunction first(): int { return 1; }\nfunction second(): string { return 'two'; }\n".to_string(),
            ),
        );
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After symbol addition: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_symbol_removed_from_existing_file() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction first(): int { return 1; }\nfunction second(): string { return 'two'; }\n"),
            ("src/b.php", "<?php\nfunction caller(): int { return first(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction first(): int { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After symbol removal: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_rapid_edit_cycles_stress() {
        let base_a = "<?php\nfunction compute(int $x): int { return $x * 2; }\n";

        let mut db = make_database(vec![
            ("src/a.php", base_a),
            ("src/b.php", "<?php\nfunction use_compute(): int { return compute(5); }\n"),
            ("src/c.php", "<?php\nfunction chain(): int { return compute(use_compute()); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        let edits = [
            "<?php\nfunction compute(int $x): int { return $x * 3; }\n",
            "<?php\nfunction compute(int $x): int { return $x + 1; }\n",
            "<?php\nfunction compute(int $x): string { return (string)$x; }\n",
            base_a,
            "<?php\nfunction compute(int $x): int { return $x * 2; }\nfunction helper(): int { return 0; }\n",
            "<?php\nfunction compute(int $x): int { return $x * 2; }\nfunction helper(): string { return ''; }\n",
            base_a,
            "<?php\nfunction compute(string $x): int { return (int)$x * 2; }\n",
            base_a,
            "<?php\n",
        ];

        for (i, edit) in edits.iter().enumerate() {
            db.update(FileId::new("src/a.php"), Cow::Owned(edit.to_string()));
            service.update_database(db.read_only());
            let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

            let mut fresh_service = make_service(&db);
            let full = fresh_service.analyze().expect("Full analysis failed.");

            let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
            assert!(
                only_incr.is_empty() && only_full.is_empty(),
                "Rapid edit cycle {}: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}",
                i + 1
            );
        }
    }

    #[test]
    fn test_incremental_all_files_changed() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction a(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction b(): int { return 2; }\n"),
            ("src/c.php", "<?php\nfunction c(): int { return a() + b(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction a(): string { return 'a'; }\n".to_string()));
        db.update(FileId::new("src/b.php"), Cow::Owned("<?php\nfunction b(): string { return 'b'; }\n".to_string()));
        db.update(
            FileId::new("src/c.php"),
            Cow::Owned("<?php\nfunction c(): string { return a() . b(); }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "All files changed: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_file_replaced_entirely() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction old_func(): int { return 1; }\n"),
            ("src/b.php", "<?php\nfunction caller(): int { return old_func(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned(
                "<?php\nclass NewClass {\n    public function method(): string { return 'hello'; }\n}\n".to_string(),
            ),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "File replaced entirely: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    #[test]
    fn test_incremental_empty_to_nonempty_and_back() {
        let mut db = make_database(vec![("src/a.php", "<?php\n"), ("src/b.php", "<?php\nfunction user(): void {}\n")]);

        let mut service = make_service(&db);
        let initial = service.analyze().expect("Full analysis failed.");

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction new_thing(): int { return 42; }\n".to_string()),
        );
        service.update_database(db.read_only());
        let with_content = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full_with = fresh_service.analyze().expect("Full analysis failed.");
        let (only_incr, only_full) = diff_issues(&with_content.issues, &full_with.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Empty to nonempty: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\n".to_string()));
        service.update_database(db.read_only());
        let back_to_empty = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service2 = make_service(&db);
        let full_empty = fresh_service2.analyze().expect("Full analysis failed.");
        let (only_incr, only_full) = diff_issues(&back_to_empty.issues, &full_empty.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Back to empty: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        assert_eq!(
            initial.issues.len(),
            back_to_empty.issues.len(),
            "After reverting to empty, issue count should match initial"
        );
    }

    #[test]
    fn test_memory_stability_body_only_edits() {
        let original =
            "<?php\nclass Greeter {\n    public function greet(string $name): string { return 'Hello ' . $name; }\n}\n";

        let mut db = make_database(vec![
            ("src/greeter.php", original),
            ("src/main.php", "<?php\nfunction main(): void { echo (new Greeter())->greet('world'); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        let baseline_refs = service.symbol_references().total_map_entries();
        let baseline_files = service.tracked_file_count();

        for i in 0..50 {
            let edited = format!(
                "<?php\nclass Greeter {{\n    public function greet(string $name): string {{ return 'Hi #{} ' . $name; }}\n}}\n",
                i
            );
            db.update(FileId::new("src/greeter.php"), Cow::Owned(edited));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");

            db.update(FileId::new("src/greeter.php"), Cow::Owned(original.to_string()));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");
        }

        let final_refs = service.symbol_references().total_map_entries();
        let final_files = service.tracked_file_count();

        assert_eq!(
            baseline_files, final_files,
            "File count should remain stable: baseline={}, final={}",
            baseline_files, final_files
        );

        let diff = (final_refs as i64 - baseline_refs as i64).unsigned_abs();
        assert!(
            diff <= 5,
            "Reference entries should remain stable across body-only edit cycles.\n  Baseline: {}\n  After 100 cycles: {}\n  Growth: {}",
            baseline_refs,
            final_refs,
            diff
        );
    }

    #[test]
    fn test_memory_stability_signature_edits() {
        let original = "<?php\nfunction compute(): int { return 42; }\n";

        let mut db = make_database(vec![
            ("src/compute.php", original),
            ("src/caller.php", "<?php\nfunction caller(): int { return compute(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        let baseline_refs = service.symbol_references().total_map_entries();

        for i in 0..30 {
            let edited = format!("<?php\nfunction compute(): string {{ return 'value_{}'; }}\n", i);
            db.update(FileId::new("src/compute.php"), Cow::Owned(edited));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");

            db.update(FileId::new("src/compute.php"), Cow::Owned(original.to_string()));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");
        }

        let final_refs = service.symbol_references().total_map_entries();
        let diff = (final_refs as i64 - baseline_refs as i64).unsigned_abs();

        assert!(
            diff <= 5,
            "Reference entries should remain stable across signature edit cycles.\n  Baseline: {}\n  After 60 cycles: {}\n  Growth: {}",
            baseline_refs,
            final_refs,
            diff
        );
    }

    #[test]
    fn test_memory_stability_mixed_edits() {
        let base = "<?php\nfunction alpha(): int { return 1; }\n";

        let mut db = make_database(vec![
            ("src/a.php", base),
            ("src/b.php", "<?php\nfunction beta(): int { return alpha(); }\n"),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        let baseline_refs = service.symbol_references().total_map_entries();
        let baseline_files = service.tracked_file_count();

        for i in 0..20 {
            let body_edit = format!("<?php\nfunction alpha(): int {{ return {}; }}\n", i);
            db.update(FileId::new("src/a.php"), Cow::Owned(body_edit));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");

            db.update(
                FileId::new("src/a.php"),
                Cow::Owned("<?php\nfunction alpha(): string { return 'x'; }\n".to_string()),
            );
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");

            db.update(
                FileId::new("src/a.php"),
                Cow::Owned("<?php\nfunction alpha(): string { return 'x'; }\nfunction extra(): void {}\n".to_string()),
            );
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");

            db.update(FileId::new("src/a.php"), Cow::Owned(base.to_string()));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental analysis failed.");
        }

        let final_refs = service.symbol_references().total_map_entries();
        let final_files = service.tracked_file_count();

        assert_eq!(baseline_files, final_files, "File count drift");
        let diff = (final_refs as i64 - baseline_refs as i64).unsigned_abs();
        assert!(
            diff <= 5,
            "Reference entries should remain stable across mixed edit cycles.\n  Baseline: {}\n  After 80 cycles: {}\n  Growth: {}",
            baseline_refs,
            final_refs,
            diff
        );
    }

    /// Regression test for https://github.com/carthage-software/mago/issues/1178
    ///
    /// Editing an unrelated file (body-only change) must not lose codebase-level
    /// issues from other files (e.g. incompatible return types from class hierarchy).
    #[test]
    fn test_incremental_body_only_unrelated_file_preserves_codebase_issues() {
        let base =
            concat!("<?php\n", "class Base {\n", "    public function process(): string { return 'hello'; }\n", "}\n",);
        let child = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "}\n",
        );
        let unrelated = concat!(
            "<?php\n",
            "class Unrelated {\n",
            "    public function greet(): string { return 'hello'; }\n",
            "}\n",
        );

        let mut db =
            make_database(vec![("src/Base.php", base), ("src/Child.php", child), ("src/Unrelated.php", unrelated)]);

        let mut service = make_service(&db);
        let initial = service.analyze().expect("Full analysis failed.");

        // Sanity: initial analysis should have at least one issue (incompatible return type).
        let mut fresh_service = make_service(&db);
        let full_initial = fresh_service.analyze().expect("Full analysis failed.");
        assert_eq!(initial.issues.len(), full_initial.issues.len());

        // Body-only change to an unrelated file.
        let unrelated_v2 =
            concat!("<?php\n", "class Unrelated {\n", "    public function greet(): string { return 'hi'; }\n", "}\n",);
        db.update(FileId::new("src/Unrelated.php"), Cow::Owned(unrelated_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service2 = make_service(&db);
        let full = fresh_service2.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Body-only change to unrelated file lost codebase issues.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    /// Regression test for https://github.com/carthage-software/mago/issues/1176
    ///
    /// Editing a file with codebase-level issues (body-only change) must preserve
    /// those codebase-level issues and not lose them after re-population.
    #[test]
    fn test_incremental_body_only_same_file_preserves_codebase_issues() {
        let base = concat!(
            "<?php\n",
            "class Base {\n",
            "    public function process(int $id): string { return 'hello'; }\n",
            "}\n",
        );
        let child = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(int $id): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child)]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        // Body-only change to Child.php (modify method body, keep signature).
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(int $id): float { return 2.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Body-only change to file with codebase issues lost them.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    /// Regression test: signature change in one file must preserve codebase-level
    /// issues in other files that are not affected by the change.
    #[test]
    fn test_incremental_signature_change_preserves_unaffected_codebase_issues() {
        let base =
            concat!("<?php\n", "class Base {\n", "    public function process(): string { return 'hello'; }\n", "}\n",);
        let child = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "}\n",
        );
        // Independent class in a separate file - signature change here should
        // not affect the codebase issues for Base/Child hierarchy.
        let other =
            concat!("<?php\n", "class Other {\n", "    public function compute(): int { return 42; }\n", "}\n",);
        let user = concat!(
            "<?php\n",
            "class OtherUser {\n",
            "    public function run(): int { return (new Other())->compute(); }\n",
            "}\n",
        );

        let mut db = make_database(vec![
            ("src/Base.php", base),
            ("src/Child.php", child),
            ("src/Other.php", other),
            ("src/OtherUser.php", user),
        ]);

        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        // Signature change to Other.php (unrelated to Base/Child hierarchy).
        let other_v2 = concat!(
            "<?php\n",
            "class Other {\n",
            "    public function compute(): string { return 'hello'; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Other.php"), Cow::Owned(other_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Signature change in unrelated file lost codebase issues.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    /// Regression test: body-only change in a child class must re-populate its metadata
    /// so the analyzer can still detect incompatible return types, undefined method calls, etc.
    #[test]
    fn test_incremental_body_only_change_in_child_class_preserves_analysis() {
        let base =
            concat!("<?php\n", "class Base {\n", "    public function process(): string { return 'base'; }\n", "}\n",);
        let child_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_service(&db);
        let initial = service.analyze().expect("Full analysis failed.");

        // Body-only change: add a statement inside the method body.
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { echo 'hi'; return 1.0; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Body-only change in child class lost analysis issues.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        // Ensure we didn't lose issues compared to initial analysis
        // (the incompatible-return-type should still be present).
        assert!(
            incremental.issues.len() >= initial.issues.len(),
            "Body-only change reduced issue count from {} to {}",
            initial.issues.len(),
            incremental.issues.len()
        );
    }

    /// Regression test: signature change (adding #[Override]) must not lose unrelated
    /// per-file analyzer issues like unused-method.
    #[test]
    fn test_incremental_signature_change_override_preserves_unused_method() {
        let base =
            concat!("<?php\n", "class Base {\n", "    public function process(): string { return 'base'; }\n", "}\n",);
        let child_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_service(&db);
        service.analyze().expect("Initial analysis failed.");

        // Signature change: add #[Override] attribute.
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    #[\\Override] public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().expect("Full analysis failed.");

        let (only_incr, only_full) = diff_issues(&incremental.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Signature change (add Override) lost issues.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    fn make_service_with_settings(db: &Database<'_>, settings: Settings) -> IncrementalAnalysisService {
        IncrementalAnalysisService::new(
            db.read_only(),
            CodebaseMetadata::new(),
            SymbolReferences::new(),
            settings,
            ParserSettings::default(),
            Arc::clone(&PLUGIN_REGISTRY),
        )
    }

    /// Regression test: body-only change followed by signature change must not lose issues.
    /// This reproduces the exact user scenario: first add `$this->something()` (body-only),
    /// then add `#[Override]` (signature change).
    #[test]
    fn test_incremental_body_then_signature_change_preserves_all_issues() {
        let settings =
            Settings { check_missing_override: true, find_unused_parameters: true, diff: true, ..Default::default() };

        let base = concat!(
            "<?php\n",
            "class Base {\n",
            "    public function process(int $id): string { return 'base'; }\n",
            "}\n",
        );
        let child_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(int $id): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_service_with_settings(&db, settings.clone());
        service.analyze().expect("Initial analysis failed.");

        // Cycle 1: body-only change - add $this->something() call.
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(int $id): float { $this->something(); return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        let cycle1 = service.analyze_incremental(None).expect("Incremental analysis failed.");

        let mut fresh1 = make_service_with_settings(&db, settings.clone());
        let full1 = fresh1.analyze().expect("Full analysis failed.");
        let (only_incr1, only_full1) = diff_issues(&cycle1.issues, &full1.issues);
        assert!(
            only_incr1.is_empty() && only_full1.is_empty(),
            "Cycle 1 (body-only) mismatch.\n  Only in incremental: {only_incr1:?}\n  Only in full: {only_full1:?}"
        );

        // Cycle 2: signature change - add #[Override] attribute.
        let child_v3 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    #[\\Override] public function process(int $id): float { $this->something(); return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v3.to_string()));
        service.update_database(db.read_only());
        let child_id = FileId::new("src/Child.php");
        let cycle2 = service
            .analyze_incremental(Some(&[child_id]))
            .expect("Incremental analysis failed after signature change following body-only change.");

        let mut fresh2 = make_service_with_settings(&db, settings.clone());
        let full2 = fresh2.analyze().expect("Full analysis failed.");

        let (only_incr2, only_full2) = diff_issues(&cycle2.issues, &full2.issues);
        assert!(
            only_incr2.is_empty() && only_full2.is_empty(),
            "Cycle 2 (signature change after body-only) mismatch.\n  Only in incremental: {only_incr2:?}\n  Only in full: {only_full2:?}"
        );
    }

    fn make_watch_service(db: &Database<'_>) -> IncrementalAnalysisService {
        make_service_with_settings(
            db,
            Settings {
                diff: true,
                find_unused_definitions: true,
                find_unused_parameters: true,
                check_missing_override: true,
                ..Default::default()
            },
        )
    }

    /// Compare incremental analysis against a fresh full analysis.
    /// Panics with a detailed diff if they disagree.
    fn assert_matches_full(service: &IncrementalAnalysisService, db: &Database<'_>, context: &str) {
        let incremental_issues = service.collect_all_issues();
        let mut fresh = make_watch_service(db);
        let full = fresh.analyze().expect("Full analysis failed.");
        let (only_incr, only_full) = diff_issues(&incremental_issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "[{context}] incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );
    }

    /// Parent changes return type making child's override incompatible.
    #[test]
    fn test_watch_parent_return_type_change_child_untouched() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent changes return type to int — child's override is now incompatible.
        let base_v2 = "<?php\nclass Base {\n    public function run(): int { return 42; }\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent return type changed");

        // Revert parent — child should be compatible again.
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent reverted");
    }

    /// Parent changes parameter type making child's override incompatible.
    #[test]
    fn test_watch_parent_param_type_change_child_untouched() {
        let base_v1 = "<?php\nclass Base {\n    public function process(int $x): void { echo $x; }\n}\n";
        let child =
            "<?php\nclass Child extends Base {\n    public function process(int $x): void { echo $x + 1; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent changes parameter type to string.
        let base_v2 = "<?php\nclass Base {\n    public function process(string $x): void { echo $x; }\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent param type changed");
    }

    /// Parent adds a new method — child's existing override should still work.
    #[test]
    fn test_watch_parent_adds_method_child_untouched() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent adds a new method.
        let base_v2 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n    public function extra(): void {}\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent added method");
    }

    /// Parent removes the method that child overrides.
    #[test]
    fn test_watch_parent_removes_overridden_method() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n    public function other(): void {}\n}\n";
        let child = "<?php\nclass Child extends Base {\n    #[\\Override] public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent removes the overridden method — child's #[Override] now has no parent method.
        let base_v2 = "<?php\nclass Base {\n    public function other(): void {}\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent removed overridden method");
    }

    /// Child changes return type to become incompatible with parent.
    #[test]
    fn test_watch_child_return_type_becomes_incompatible() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - compatible");

        // Child changes return type to float — now incompatible.
        let child_v2 = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child return type incompatible");

        // Child fixes return type — compatible again.
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child return type fixed");
    }

    /// Child adds #[Override] attribute — signature change.
    #[test]
    fn test_watch_child_adds_override_attribute() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - no Override");

        // Add #[Override].
        let child_v2 = "<?php\nclass Child extends Base {\n    #[\\Override] public function run(): string { return 'child'; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child added Override");
    }

    /// Child removes #[Override] attribute — signature change, should report missing-override.
    #[test]
    fn test_watch_child_removes_override_attribute() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    #[\\Override] public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - with Override");

        // Remove #[Override].
        let child_v2 = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child removed Override");
    }

    /// Body-only change in child class with diff=true.
    #[test]
    fn test_watch_body_change_in_child() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body-only change in child.
        let child_v2 = "<?php\nclass Child extends Base {\n    public function run(): float { echo 'hi'; return 1.0; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body change in child");
    }

    /// Body-only change in parent class with diff=true.
    #[test]
    fn test_watch_body_change_in_parent() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body-only change in parent.
        let base_v2 = "<?php\nclass Base {\n    public function run(): string { return 'changed'; }\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body change in parent");
    }

    /// Body-only change in unrelated file with diff=true.
    #[test]
    fn test_watch_body_change_in_unrelated_file() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";
        let unrelated_v1 = "<?php\nclass Unrelated {\n    public function greet(): string { return 'hello'; }\n}\n";

        let mut db =
            make_database(vec![("src/Base.php", base), ("src/Child.php", child), ("src/Unrelated.php", unrelated_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body-only change in unrelated file.
        let unrelated_v2 = "<?php\nclass Unrelated {\n    public function greet(): string { return 'hi'; }\n}\n";
        db.update(FileId::new("src/Unrelated.php"), Cow::Owned(unrelated_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body change in unrelated file");
    }

    /// Add a method to one class in a multi-class file — other class's issues preserved.
    #[test]
    fn test_watch_add_method_multi_class_file() {
        let base = "<?php\nclass Base {\n    public function process(): string { return 'ok'; }\n}\n";
        let child_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add a method to Helper — signature change for Helper, Child untouched.
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "    public function extra(): void { echo []; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "added method to Helper");

        // Revert — should restore original issues exactly.
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "reverted Helper method addition");
    }

    /// Remove a method from a class.
    #[test]
    fn test_watch_remove_method_from_class() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function run(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "    private function alsoUnused(): void {}\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Remove alsoUnused method.
        let child_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function run(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
        );
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "removed method");
    }

    /// Add a new private unused method.
    #[test]
    fn test_watch_add_unused_private_method() {
        let code_v1 = "<?php\nclass Foo {\n    public function bar(): void { echo 'bar'; }\n}\n";

        let mut db = make_database(vec![("src/Foo.php", code_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add an unused private method.
        let code_v2 = "<?php\nclass Foo {\n    public function bar(): void { echo 'bar'; }\n    private function secret(): void {}\n}\n";
        db.update(FileId::new("src/Foo.php"), Cow::Owned(code_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "added unused private method");
    }

    /// A private method was called; caller is removed → method becomes unused.
    #[test]
    fn test_watch_method_becomes_unused_after_caller_removed() {
        let code_v1 = concat!(
            "<?php\n",
            "class Foo {\n",
            "    public function bar(): void { $this->helper(); }\n",
            "    private function helper(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Foo.php", code_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - helper is used");

        // Remove the call to helper — it becomes unused.
        let code_v2 = concat!(
            "<?php\n",
            "class Foo {\n",
            "    public function bar(): void { echo 'no helper'; }\n",
            "    private function helper(): void { echo 'help'; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Foo.php"), Cow::Owned(code_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "helper became unused");
    }

    /// A private method was unused; caller is added → method is now used.
    #[test]
    fn test_watch_method_becomes_used_after_caller_added() {
        let code_v1 = concat!(
            "<?php\n",
            "class Foo {\n",
            "    public function bar(): void { echo 'no helper'; }\n",
            "    private function helper(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Foo.php", code_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - helper is unused");

        // Add a call to helper — it becomes used.
        let code_v2 = concat!(
            "<?php\n",
            "class Foo {\n",
            "    public function bar(): void { $this->helper(); }\n",
            "    private function helper(): void { echo 'help'; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Foo.php"), Cow::Owned(code_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "helper became used");
    }

    /// Cross-file reference: function called from another file, that file removes the call.
    #[test]
    fn test_watch_cross_file_reference_removed() {
        let lib = "<?php\nfunction helper(): int { return 42; }\n";
        let caller_v1 = "<?php\nfunction main_fn(): int { return helper(); }\n";

        let mut db = make_database(vec![("src/lib.php", lib), ("src/caller.php", caller_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Caller stops calling helper.
        let caller_v2 = "<?php\nfunction main_fn(): int { return 99; }\n";
        db.update(FileId::new("src/caller.php"), Cow::Owned(caller_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "cross-file reference removed");
    }

    /// Cross-file reference: function was not called, another file adds a call.
    #[test]
    fn test_watch_cross_file_reference_added() {
        let lib = "<?php\nfunction helper(): int { return 42; }\n";
        let caller_v1 = "<?php\nfunction main_fn(): int { return 99; }\n";

        let mut db = make_database(vec![("src/lib.php", lib), ("src/caller.php", caller_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - no call");

        // Caller adds a call to helper.
        let caller_v2 = "<?php\nfunction main_fn(): int { return helper(); }\n";
        db.update(FileId::new("src/caller.php"), Cow::Owned(caller_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "cross-file reference added");
    }

    /// Parent class deleted — child's extends is now dangling.
    #[test]
    fn test_watch_parent_class_deleted() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Delete parent class file.
        db.delete(FileId::new("src/Base.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent deleted");
    }

    /// Child class deleted — parent should be unaffected.
    #[test]
    fn test_watch_child_class_deleted() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Delete child class file.
        db.delete(FileId::new("src/Child.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child deleted");
    }

    /// Parent class re-added after deletion.
    #[test]
    fn test_watch_parent_class_deleted_then_readded() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Delete parent.
        db.delete(FileId::new("src/Base.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent deleted");

        // Re-add parent.
        db.add(File::new(Cow::Owned("src/Base.php".to_string()), FileType::Host, None, Cow::Owned(base.to_string())));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent re-added");
    }

    /// File has two classes; change signature of one, other's issues preserved.
    #[test]
    fn test_watch_multi_class_file_one_class_signature_change() {
        let base = "<?php\nclass Base {\n    public function process(): string { return 'ok'; }\n}\n";
        let file_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class AnotherClass {\n",
            "    public function doStuff(): void { echo 'stuff'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Classes.php", file_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change AnotherClass signature — Child issues must be preserved.
        let file_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class AnotherClass {\n",
            "    public function doStuff(): void { echo 'stuff'; }\n",
            "    public function newMethod(): void { echo []; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Classes.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "one class changed in multi-class file");
    }

    /// File has two classes; change signature of one, then revert.
    #[test]
    fn test_watch_multi_class_file_change_and_revert() {
        let base = "<?php\nclass Base {\n    public function process(): string { return 'ok'; }\n}\n";
        let file_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Classes.php", file_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change Helper.
        let file_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "    public function extra(): int { return 1; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Classes.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "changed Helper");

        // Revert.
        db.update(FileId::new("src/Classes.php"), Cow::Owned(file_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "reverted multi-class file");
    }

    /// File has two classes; body-only change in one, other's issues preserved.
    #[test]
    fn test_watch_multi_class_file_body_only_change() {
        let base = "<?php\nclass Base {\n    public function process(): string { return 'ok'; }\n}\n";
        let file_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Classes.php", file_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body-only change in Helper.
        let file_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'changed'; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Classes.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body-only change in multi-class file");
    }

    /// A new child class is added in a new file extending an existing parent.
    #[test]
    fn test_watch_new_child_class_added() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - no child");

        // Add a new child class file with incompatible return type.
        let child = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n}\n";
        db.add(File::new(Cow::Owned("src/Child.php".to_string()), FileType::Host, None, Cow::Owned(child.to_string())));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "new child class added");
    }

    /// An interface is implemented; then the implementation is removed.
    #[test]
    fn test_watch_interface_implementation_added_and_removed() {
        let iface = "<?php\ninterface Printable {\n    public function print(): string;\n}\n";
        let class_v1 = "<?php\nclass Printer {\n    public function print(): string { return 'hello'; }\n}\n";

        let mut db = make_database(vec![("src/Printable.php", iface), ("src/Printer.php", class_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - no implementation");

        // Add implements clause.
        let class_v2 =
            "<?php\nclass Printer implements Printable {\n    public function print(): string { return 'hello'; }\n}\n";
        db.update(FileId::new("src/Printer.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "added implements");

        // Remove implements clause.
        db.update(FileId::new("src/Printer.php"), Cow::Owned(class_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "removed implements");
    }

    /// Multiple sequential signature changes across different files.
    #[test]
    fn test_watch_sequential_changes_across_files() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n    private function unused(): void {}\n}\n";
        let user_code = "<?php\nfunction useChild(): string { return (new Child())->run(); }\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child), ("src/user.php", user_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Cycle 1: Change base return type.
        db.update(
            FileId::new("src/Base.php"),
            Cow::Owned("<?php\nclass Base {\n    public function run(): int { return 42; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "cycle 1: base changed");

        // Cycle 2: Change child return type to match new base.
        db.update(
            FileId::new("src/Child.php"),
            Cow::Owned("<?php\nclass Child extends Base {\n    public function run(): int { return 99; }\n    private function unused(): void {}\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "cycle 2: child matched base");

        // Cycle 3: Change user code.
        db.update(
            FileId::new("src/user.php"),
            Cow::Owned("<?php\nfunction useChild(): int { return (new Child())->run() + 1; }\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "cycle 3: user code changed");
    }

    /// Alternating body and signature changes in the same file.
    #[test]
    fn test_watch_alternating_body_and_signature_changes() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body change.
        let child_body = "<?php\nclass Child extends Base {\n    public function run(): float { echo 'x'; return 1.0; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_body.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body change");

        // Signature change.
        let child_sig = "<?php\nclass Child extends Base {\n    #[\\Override] public function run(): float { echo 'x'; return 1.0; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_sig.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "signature change");

        // Another body change.
        let child_body2 = "<?php\nclass Child extends Base {\n    #[\\Override] public function run(): float { echo 'y'; return 2.0; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_body2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "another body change");

        // Revert to original.
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "reverted to original");
    }

    /// Three-level inheritance chain: grandparent, parent, child.
    #[test]
    fn test_watch_three_level_inheritance() {
        let grandparent = "<?php\nclass GrandParent_ {\n    public function run(): string { return 'gp'; }\n}\n";
        let parent =
            "<?php\nclass Parent_ extends GrandParent_ {\n    public function run(): string { return 'parent'; }\n}\n";
        let child = "<?php\nclass Child extends Parent_ {\n    public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![
            ("src/GrandParent.php", grandparent),
            ("src/Parent.php", parent),
            ("src/Child.php", child),
        ]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change grandparent return type — should cascade to parent and child.
        db.update(
            FileId::new("src/GrandParent.php"),
            Cow::Owned("<?php\nclass GrandParent_ {\n    public function run(): int { return 42; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "grandparent changed");

        // Fix parent to match.
        db.update(
            FileId::new("src/Parent.php"),
            Cow::Owned(
                "<?php\nclass Parent_ extends GrandParent_ {\n    public function run(): int { return 1; }\n}\n"
                    .to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent fixed");

        // Fix child to match.
        db.update(
            FileId::new("src/Child.php"),
            Cow::Owned(
                "<?php\nclass Child extends Parent_ {\n    public function run(): int { return 2; }\n}\n".to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child fixed");
    }

    /// Delete and re-add a file with different content.
    #[test]
    fn test_watch_delete_and_readd_different_content() {
        let a = "<?php\nfunction helper(): int { return 42; }\n";
        let b = "<?php\nfunction caller(): int { return helper(); }\n";

        let mut db = make_database(vec![("src/a.php", a), ("src/b.php", b)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Delete a.php.
        db.delete(FileId::new("src/a.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "a.php deleted");

        // Re-add a.php with different content.
        db.add(File::new(
            Cow::Owned("src/a.php".to_string()),
            FileType::Host,
            None,
            Cow::Owned("<?php\nfunction helper(): string { return 'oops'; }\n".to_string()),
        ));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "a.php re-added with different content");
    }

    /// File with only a class (no standalone functions) — body change.
    #[test]
    fn test_watch_class_only_file_body_change() {
        let code_v1 = concat!(
            "<?php\n",
            "class Service {\n",
            "    public function handle(): void { echo 'v1'; }\n",
            "    private function log(): void { echo 'log'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Service.php", code_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Body change that doesn't affect signature.
        let code_v2 = concat!(
            "<?php\n",
            "class Service {\n",
            "    public function handle(): void { echo 'v2'; $this->log(); }\n",
            "    private function log(): void { echo 'log'; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Service.php"), Cow::Owned(code_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "body change — log now used");

        // Remove the call again — log becomes unused.
        db.update(FileId::new("src/Service.php"), Cow::Owned(code_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "reverted — log unused again");
    }

    /// Multiple inheritance hierarchies in different files; change in one hierarchy
    /// must not affect the other.
    #[test]
    fn test_watch_independent_hierarchies() {
        let animal_base = "<?php\nclass Animal {\n    public function speak(): string { return 'generic'; }\n}\n";
        let dog = "<?php\nclass Dog extends Animal {\n    public function speak(): string { return 'woof'; }\n}\n";
        let vehicle_base = "<?php\nclass Vehicle {\n    public function speed(): int { return 0; }\n}\n";
        let car = "<?php\nclass Car extends Vehicle {\n    public function speed(): float { return 60.0; }\n}\n";

        let mut db = make_database(vec![
            ("src/Animal.php", animal_base),
            ("src/Dog.php", dog),
            ("src/Vehicle.php", vehicle_base),
            ("src/Car.php", car),
        ]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change Animal hierarchy — Vehicle hierarchy must be unaffected.
        db.update(
            FileId::new("src/Animal.php"),
            Cow::Owned("<?php\nclass Animal {\n    public function speak(): int { return 42; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "animal hierarchy changed");

        // Change Vehicle hierarchy — Animal hierarchy must be unaffected.
        db.update(
            FileId::new("src/Vehicle.php"),
            Cow::Owned(
                "<?php\nclass Vehicle {\n    public function speed(): string { return 'fast'; }\n}\n".to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "vehicle hierarchy changed");
    }

    /// Rapid edits to a file with multiple classes — stress test.
    #[test]
    fn test_watch_rapid_edits_multi_class_stress() {
        let base = "<?php\nclass Base {\n    public function process(): string { return 'ok'; }\n}\n";
        let file_original = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function process(): float { return 1.0; }\n",
            "    private function unused(): void {}\n",
            "}\n",
            "\n",
            "class Helper {\n",
            "    public function help(): void { echo 'help'; }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/Classes.php", file_original)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        let edits = [
            // Body change in Helper.
            concat!(
                "<?php\n",
                "class Child extends Base {\n",
                "    public function process(): float { return 1.0; }\n",
                "    private function unused(): void {}\n",
                "}\n",
                "\n",
                "class Helper {\n",
                "    public function help(): void { echo 'changed'; }\n",
                "}\n",
            ),
            // Signature change in Helper (add method).
            concat!(
                "<?php\n",
                "class Child extends Base {\n",
                "    public function process(): float { return 1.0; }\n",
                "    private function unused(): void {}\n",
                "}\n",
                "\n",
                "class Helper {\n",
                "    public function help(): void { echo 'changed'; }\n",
                "    public function extra(): int { return 1; }\n",
                "}\n",
            ),
            // Signature change in Child (add Override).
            concat!(
                "<?php\n",
                "class Child extends Base {\n",
                "    #[\\Override] public function process(): float { return 1.0; }\n",
                "    private function unused(): void {}\n",
                "}\n",
                "\n",
                "class Helper {\n",
                "    public function help(): void { echo 'changed'; }\n",
                "    public function extra(): int { return 1; }\n",
                "}\n",
            ),
            // Remove unused method from Child.
            concat!(
                "<?php\n",
                "class Child extends Base {\n",
                "    #[\\Override] public function process(): float { return 1.0; }\n",
                "}\n",
                "\n",
                "class Helper {\n",
                "    public function help(): void { echo 'changed'; }\n",
                "    public function extra(): int { return 1; }\n",
                "}\n",
            ),
            // Remove extra from Helper.
            concat!(
                "<?php\n",
                "class Child extends Base {\n",
                "    #[\\Override] public function process(): float { return 1.0; }\n",
                "}\n",
                "\n",
                "class Helper {\n",
                "    public function help(): void { echo 'changed'; }\n",
                "}\n",
            ),
        ];

        for (i, edit) in edits.iter().enumerate() {
            db.update(FileId::new("src/Classes.php"), Cow::Owned(edit.to_string()));
            service.update_database(db.read_only());
            service.analyze_incremental(None).expect("Incremental failed.");
            assert_matches_full(&service, &db, &format!("rapid edit {}", i + 1));
        }

        // Revert to original.
        db.update(FileId::new("src/Classes.php"), Cow::Owned(file_original.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "reverted to original");
    }

    /// Trait usage: class uses a trait, trait method changes.
    #[test]
    fn test_watch_trait_method_change() {
        let trait_v1 = "<?php\ntrait Greeter {\n    public function greet(): string { return 'hello'; }\n}\n";
        let class_code = "<?php\nclass MyClass {\n    use Greeter;\n    public function run(): string { return $this->greet(); }\n}\n";

        let mut db = make_database(vec![("src/Greeter.php", trait_v1), ("src/MyClass.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change trait method return type.
        let trait_v2 = "<?php\ntrait Greeter {\n    public function greet(): int { return 42; }\n}\n";
        db.update(FileId::new("src/Greeter.php"), Cow::Owned(trait_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "trait method return type changed");
    }

    /// Abstract class with concrete child — abstract method signature changes.
    #[test]
    fn test_watch_abstract_method_signature_change() {
        let abstract_v1 = "<?php\nabstract class Shape {\n    abstract public function area(): float;\n}\n";
        let concrete = "<?php\nclass Circle extends Shape {\n    public function area(): float { return 3.14; }\n}\n";

        let mut db = make_database(vec![("src/Shape.php", abstract_v1), ("src/Circle.php", concrete)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change abstract method return type to int — Circle's implementation is now incompatible.
        let abstract_v2 = "<?php\nabstract class Shape {\n    abstract public function area(): int;\n}\n";
        db.update(FileId::new("src/Shape.php"), Cow::Owned(abstract_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "abstract method return type changed");

        // Revert abstract method.
        db.update(FileId::new("src/Shape.php"), Cow::Owned(abstract_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "abstract method reverted");
    }

    /// Enum with methods — method signature change in enum.
    #[test]
    fn test_watch_enum_method_change() {
        let enum_v1 = concat!(
            "<?php\n",
            "enum Color: string {\n",
            "    case Red = 'red';\n",
            "    case Blue = 'blue';\n",
            "    public function label(): string { return $this->value; }\n",
            "}\n",
        );
        let user_code = "<?php\nfunction getLabel(): string { return Color::Red->label(); }\n";

        let mut db = make_database(vec![("src/Color.php", enum_v1), ("src/user.php", user_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add a new case to enum.
        let enum_v2 = concat!(
            "<?php\n",
            "enum Color: string {\n",
            "    case Red = 'red';\n",
            "    case Blue = 'blue';\n",
            "    case Green = 'green';\n",
            "    public function label(): string { return $this->value; }\n",
            "}\n",
        );
        db.update(FileId::new("src/Color.php"), Cow::Owned(enum_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "enum case added");
    }

    /// Simultaneous changes to parent and child in the same cycle.
    #[test]
    fn test_watch_parent_and_child_change_simultaneously() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_v1 = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Both change at the same time.
        let base_v2 = "<?php\nclass Base {\n    public function run(): int { return 42; }\n}\n";
        let child_v2 = "<?php\nclass Child extends Base {\n    public function run(): int { return 99; }\n    private function unused(): void {}\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "both parent and child changed");
    }

    /// Move a class from one file to another (delete from old, add to new).
    #[test]
    fn test_watch_move_class_between_files() {
        let file_a_v1 = concat!(
            "<?php\n",
            "class Foo {\n",
            "    public function bar(): string { return 'foo'; }\n",
            "}\n",
            "class Helper {\n",
            "    public function help(): void { echo (new Foo())->bar(); }\n",
            "}\n",
        );

        let mut db = make_database(vec![("src/a.php", file_a_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - both in a.php");

        // Move Foo to b.php.
        let file_a_v2 = "<?php\nclass Helper {\n    public function help(): void { echo (new Foo())->bar(); }\n}\n";
        let file_b = "<?php\nclass Foo {\n    public function bar(): string { return 'foo'; }\n}\n";
        db.update(FileId::new("src/a.php"), Cow::Owned(file_a_v2.to_string()));
        db.add(File::new(Cow::Owned("src/b.php".to_string()), FileType::Host, None, Cow::Owned(file_b.to_string())));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "Foo moved to b.php");
    }

    /// Change a class from non-abstract to abstract.
    #[test]
    fn test_watch_class_becomes_abstract() {
        let class_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let user = "<?php\nfunction make(): Base { return new Base(); }\n";

        let mut db = make_database(vec![("src/Base.php", class_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - concrete class");

        // Make it abstract.
        let class_v2 = "<?php\nabstract class Base {\n    abstract public function run(): string;\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "class became abstract");
    }

    /// Property type changes in parent, child inherits.
    #[test]
    fn test_watch_property_type_change_in_parent() {
        let base_v1 = "<?php\nclass Base {\n    public string $name = 'default';\n    public function getName(): string { return $this->name; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function greet(): string { return 'Hello ' . $this->name; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change property type.
        let base_v2 = "<?php\nclass Base {\n    public int $name = 42;\n    public function getName(): int { return $this->name; }\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent property type changed");
    }

    /// Multiple files with issues; edit one, all others' issues must survive.
    #[test]
    fn test_watch_many_files_with_issues_edit_one() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child1 = "<?php\nclass Child1 extends Base {\n    public function run(): float { return 1.0; }\n}\n";
        let child2 = "<?php\nclass Child2 extends Base {\n    public function run(): int { return 42; }\n}\n";
        let child3 = "<?php\nclass Child3 extends Base {\n    public function run(): bool { return true; }\n}\n";

        let mut db = make_database(vec![
            ("src/Base.php", base),
            ("src/Child1.php", child1),
            ("src/Child2.php", child2),
            ("src/Child3.php", child3),
        ]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Edit only Child1 (body change) — Child2 and Child3 issues must persist.
        db.update(
            FileId::new("src/Child1.php"),
            Cow::Owned(
                "<?php\nclass Child1 extends Base {\n    public function run(): float { echo 'x'; return 1.0; }\n}\n"
                    .to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "edit Child1 body");

        // Edit only Child2 (signature change) — Child1 and Child3 issues must persist.
        db.update(
            FileId::new("src/Child2.php"),
            Cow::Owned(
                "<?php\nclass Child2 extends Base {\n    #[\\Override] public function run(): int { return 42; }\n}\n"
                    .to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "edit Child2 signature");
    }

    /// Trait used by two classes. Trait method return type changes — both users must be re-checked.
    #[test]
    fn test_watch_trait_used_by_multiple_classes() {
        let trait_v1 = "<?php\ntrait Logger {\n    public function log(): string { return 'log'; }\n}\n";
        let class_a =
            "<?php\nclass A {\n    use Logger;\n    public function run(): string { return $this->log(); }\n}\n";
        let class_b =
            "<?php\nclass B {\n    use Logger;\n    public function exec(): string { return $this->log(); }\n}\n";

        let mut db = make_database(vec![("src/Logger.php", trait_v1), ("src/A.php", class_a), ("src/B.php", class_b)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change trait method return type.
        let trait_v2 = "<?php\ntrait Logger {\n    public function log(): int { return 42; }\n}\n";
        db.update(FileId::new("src/Logger.php"), Cow::Owned(trait_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "trait return type changed");
    }

    /// Trait adds a new method — classes using the trait should see it.
    #[test]
    fn test_watch_trait_adds_method() {
        let trait_v1 = "<?php\ntrait Greet {\n    public function hello(): string { return 'hi'; }\n}\n";
        let class_code =
            "<?php\nclass MyClass {\n    use Greet;\n    public function run(): string { return $this->hello(); }\n}\n";

        let mut db = make_database(vec![("src/Greet.php", trait_v1), ("src/MyClass.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add a method to trait.
        let trait_v2 = "<?php\ntrait Greet {\n    public function hello(): string { return 'hi'; }\n    public function goodbye(): int { return 0; }\n}\n";
        db.update(FileId::new("src/Greet.php"), Cow::Owned(trait_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "trait added method");
    }

    /// Trait removes a method that a class calls.
    #[test]
    fn test_watch_trait_removes_method_used_by_class() {
        let trait_v1 = "<?php\ntrait Helper {\n    public function assist(): string { return 'help'; }\n    public function other(): void {}\n}\n";
        let class_code = "<?php\nclass Worker {\n    use Helper;\n    public function work(): string { return $this->assist(); }\n}\n";

        let mut db = make_database(vec![("src/Helper.php", trait_v1), ("src/Worker.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Remove assist method from trait.
        let trait_v2 = "<?php\ntrait Helper {\n    public function other(): void {}\n}\n";
        db.update(FileId::new("src/Helper.php"), Cow::Owned(trait_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "trait method removed");
    }

    /// Interface method signature changes — implementor becomes incompatible.
    #[test]
    fn test_watch_interface_method_signature_change() {
        let iface_v1 = "<?php\ninterface Renderable {\n    public function render(): string;\n}\n";
        let class_code =
            "<?php\nclass Page implements Renderable {\n    public function render(): string { return '<html>'; }\n}\n";

        let mut db = make_database(vec![("src/Renderable.php", iface_v1), ("src/Page.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Interface changes return type.
        let iface_v2 = "<?php\ninterface Renderable {\n    public function render(): int;\n}\n";
        db.update(FileId::new("src/Renderable.php"), Cow::Owned(iface_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "interface return type changed");

        // Revert.
        db.update(FileId::new("src/Renderable.php"), Cow::Owned(iface_v1.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "interface reverted");
    }

    /// Interface adds a new required method — implementor now missing it.
    #[test]
    fn test_watch_interface_adds_required_method() {
        let iface_v1 = "<?php\ninterface Cacheable {\n    public function getKey(): string;\n}\n";
        let class_code =
            "<?php\nclass Item implements Cacheable {\n    public function getKey(): string { return 'item'; }\n}\n";

        let mut db = make_database(vec![("src/Cacheable.php", iface_v1), ("src/Item.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add required method to interface.
        let iface_v2 = "<?php\ninterface Cacheable {\n    public function getKey(): string;\n    public function getTtl(): int;\n}\n";
        db.update(FileId::new("src/Cacheable.php"), Cow::Owned(iface_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "interface added method");
    }

    /// Multiple interfaces — class implements two, one changes.
    #[test]
    fn test_watch_class_implements_multiple_interfaces_one_changes() {
        let iface_a = "<?php\ninterface Readable {\n    public function read(): string;\n}\n";
        let iface_b = "<?php\ninterface Writable {\n    public function write(string $data): void;\n}\n";
        let class_code = concat!(
            "<?php\n",
            "class File implements Readable, Writable {\n",
            "    public function read(): string { return 'data'; }\n",
            "    public function write(string $data): void { echo $data; }\n",
            "}\n",
        );

        let mut db = make_database(vec![
            ("src/Readable.php", iface_a),
            ("src/Writable.php", iface_b),
            ("src/File.php", class_code),
        ]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change Writable interface.
        let iface_b_v2 = "<?php\ninterface Writable {\n    public function write(string $data): int;\n}\n";
        db.update(FileId::new("src/Writable.php"), Cow::Owned(iface_b_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "one of two interfaces changed");
    }

    /// Parent constructor changes — child calling parent::__construct should be re-checked.
    #[test]
    fn test_watch_parent_constructor_changes() {
        let parent_v1 = "<?php\nclass Base {\n    public function __construct(public string $name) {}\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function __construct() { parent::__construct('default'); }\n}\n";

        let mut db = make_database(vec![("src/Base.php", parent_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent adds required parameter.
        let parent_v2 =
            "<?php\nclass Base {\n    public function __construct(public string $name, public int $age) {}\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(parent_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent constructor added param");
    }

    /// Constructor promotion — property type changes via constructor.
    #[test]
    fn test_watch_constructor_promoted_property_type_change() {
        let class_v1 = "<?php\nclass Config {\n    public function __construct(public string $value) {}\n    public function get(): string { return $this->value; }\n}\n";
        let user = "<?php\nfunction useConfig(): string { return (new Config('x'))->get(); }\n";

        let mut db = make_database(vec![("src/Config.php", class_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change promoted property type.
        let class_v2 = "<?php\nclass Config {\n    public function __construct(public int $value) {}\n    public function get(): int { return $this->value; }\n}\n";
        db.update(FileId::new("src/Config.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "promoted property type changed");
    }

    /// Class constant value/type changes.
    #[test]
    fn test_watch_class_constant_type_change() {
        let class_v1 = "<?php\nclass Settings {\n    public const string VERSION = '1.0';\n    public function getVersion(): string { return self::VERSION; }\n}\n";
        let user = "<?php\nfunction printVersion(): void { echo Settings::VERSION; }\n";

        let mut db = make_database(vec![("src/Settings.php", class_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change constant type.
        let class_v2 = "<?php\nclass Settings {\n    public const int VERSION = 2;\n    public function getVersion(): int { return self::VERSION; }\n}\n";
        db.update(FileId::new("src/Settings.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "constant type changed");
    }

    /// Enum case added/removed.
    #[test]
    fn test_watch_enum_case_added_removed() {
        let enum_v1 = "<?php\nenum Status: string {\n    case Active = 'active';\n    case Inactive = 'inactive';\n}\n";
        let user = "<?php\nfunction getStatus(): Status { return Status::Active; }\n";

        let mut db = make_database(vec![("src/Status.php", enum_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add a case.
        let enum_v2 = "<?php\nenum Status: string {\n    case Active = 'active';\n    case Inactive = 'inactive';\n    case Pending = 'pending';\n}\n";
        db.update(FileId::new("src/Status.php"), Cow::Owned(enum_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "enum case added");

        // Remove a case.
        let enum_v3 = "<?php\nenum Status: string {\n    case Active = 'active';\n}\n";
        db.update(FileId::new("src/Status.php"), Cow::Owned(enum_v3.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "enum cases removed");
    }

    /// File with multiple functions — one changes signature, other's issues preserved.
    #[test]
    fn test_watch_multi_function_file_one_changes() {
        let file_v1 = concat!(
            "<?php\n",
            "function alpha(): int { return 'not_int'; }\n",
            "function beta(): string { return 42; }\n",
        );

        let mut db = make_database(vec![("src/funcs.php", file_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - both have issues");

        // Change alpha's signature — beta's issues must persist.
        let file_v2 = concat!(
            "<?php\n",
            "function alpha(): string { return 'now_correct'; }\n",
            "function beta(): string { return 42; }\n",
        );
        db.update(FileId::new("src/funcs.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "alpha changed, beta preserved");
    }

    /// Mixed functions and classes in the same file.
    #[test]
    fn test_watch_mixed_functions_and_classes() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let file_v1 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function run(): float { return 1.0; }\n",
            "}\n",
            "function standalone(): int { return 'oops'; }\n",
        );

        let mut db = make_database(vec![("src/Base.php", base), ("src/mixed.php", file_v1)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change standalone function signature.
        let file_v2 = concat!(
            "<?php\n",
            "class Child extends Base {\n",
            "    public function run(): float { return 1.0; }\n",
            "}\n",
            "function standalone(): string { return 'fixed'; }\n",
        );
        db.update(FileId::new("src/mixed.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "function changed, class issues preserved");
    }

    /// Method visibility changes from public to private.
    #[test]
    fn test_watch_method_visibility_change() {
        let class_v1 = "<?php\nclass Service {\n    public function doWork(): void { echo 'work'; }\n}\n";
        let user = "<?php\nfunction callService(): void { (new Service())->doWork(); }\n";

        let mut db = make_database(vec![("src/Service.php", class_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - public");

        // Make method private.
        let class_v2 = "<?php\nclass Service {\n    private function doWork(): void { echo 'work'; }\n}\n";
        db.update(FileId::new("src/Service.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "method became private");
    }

    /// Class becomes final — existing child should error.
    #[test]
    fn test_watch_class_becomes_final() {
        let base_v1 = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base_v1), ("src/Child.php", child)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial - non-final");

        // Make base final.
        let base_v2 = "<?php\nfinal class Base {\n    public function run(): string { return 'ok'; }\n}\n";
        db.update(FileId::new("src/Base.php"), Cow::Owned(base_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "base became final");
    }

    /// Method becomes static.
    #[test]
    fn test_watch_method_becomes_static() {
        let class_v1 = "<?php\nclass Math {\n    public function add(int $a, int $b): int { return $a + $b; }\n}\n";
        let user = "<?php\nfunction calc(): int { return (new Math())->add(1, 2); }\n";

        let mut db = make_database(vec![("src/Math.php", class_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Make method static.
        let class_v2 =
            "<?php\nclass Math {\n    public static function add(int $a, int $b): int { return $a + $b; }\n}\n";
        db.update(FileId::new("src/Math.php"), Cow::Owned(class_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "method became static");
    }

    /// Four-level inheritance: change at top cascades all the way down.
    #[test]
    fn test_watch_four_level_inheritance_cascade() {
        let l1 = "<?php\nclass L1 {\n    public function method(): string { return 'l1'; }\n}\n";
        let l2 = "<?php\nclass L2 extends L1 {\n    public function method(): string { return 'l2'; }\n}\n";
        let l3 = "<?php\nclass L3 extends L2 {\n    public function method(): string { return 'l3'; }\n}\n";
        let l4 = "<?php\nclass L4 extends L3 {\n    public function method(): string { return 'l4'; }\n}\n";

        let mut db =
            make_database(vec![("src/L1.php", l1), ("src/L2.php", l2), ("src/L3.php", l3), ("src/L4.php", l4)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change L1 return type.
        db.update(
            FileId::new("src/L1.php"),
            Cow::Owned("<?php\nclass L1 {\n    public function method(): int { return 1; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "L1 return type changed — cascade to L2/L3/L4");
    }

    /// Diamond pattern: class implements two interfaces with same method, interface changes.
    #[test]
    fn test_watch_diamond_interfaces() {
        let iface_a = "<?php\ninterface A {\n    public function process(): string;\n}\n";
        let iface_b = "<?php\ninterface B {\n    public function process(): string;\n}\n";
        let class_code =
            "<?php\nclass Impl implements A, B {\n    public function process(): string { return 'ok'; }\n}\n";

        let mut db = make_database(vec![("src/A.php", iface_a), ("src/B.php", iface_b), ("src/Impl.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change interface A return type — now class is incompatible with A.
        db.update(
            FileId::new("src/A.php"),
            Cow::Owned("<?php\ninterface A {\n    public function process(): int;\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "interface A changed in diamond");
    }

    /// Class extends parent AND implements interface — both change.
    #[test]
    fn test_watch_class_with_parent_and_interface_both_change() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'base'; }\n}\n";
        let iface = "<?php\ninterface HasId {\n    public function getId(): int;\n}\n";
        let class_code = concat!(
            "<?php\n",
            "class Entity extends Base implements HasId {\n",
            "    public function run(): string { return 'entity'; }\n",
            "    public function getId(): int { return 1; }\n",
            "}\n",
        );

        let mut db =
            make_database(vec![("src/Base.php", base), ("src/HasId.php", iface), ("src/Entity.php", class_code)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Both parent and interface change simultaneously.
        db.update(
            FileId::new("src/Base.php"),
            Cow::Owned("<?php\nclass Base {\n    public function run(): int { return 42; }\n}\n".to_string()),
        );
        db.update(
            FileId::new("src/HasId.php"),
            Cow::Owned("<?php\ninterface HasId {\n    public function getId(): string;\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent and interface both changed");
    }

    /// One parent, many children. Parent changes — all children must be re-checked.
    #[test]
    fn test_watch_one_parent_many_children() {
        let base = "<?php\nclass Base {\n    public function method(): string { return 'base'; }\n}\n";
        let c1 = "<?php\nclass C1 extends Base {\n    public function method(): string { return 'c1'; }\n}\n";
        let c2 = "<?php\nclass C2 extends Base {\n    public function method(): string { return 'c2'; }\n}\n";
        let c3 = "<?php\nclass C3 extends Base {\n    public function method(): string { return 'c3'; }\n}\n";

        let mut db =
            make_database(vec![("src/Base.php", base), ("src/C1.php", c1), ("src/C2.php", c2), ("src/C3.php", c3)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Parent changes return type — all children become incompatible.
        db.update(
            FileId::new("src/Base.php"),
            Cow::Owned("<?php\nclass Base {\n    public function method(): int { return 1; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent changed — all children affected");
    }

    /// Two files with only functions. One function's return type changes.
    #[test]
    fn test_watch_standalone_function_return_type_cascade() {
        let lib = "<?php\nfunction helper(): int { return 42; }\n";
        let caller = "<?php\nfunction caller(): int { return helper() + 1; }\n";

        let mut db = make_database(vec![("src/lib.php", lib), ("src/caller.php", caller)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Change helper return type.
        db.update(
            FileId::new("src/lib.php"),
            Cow::Owned("<?php\nfunction helper(): string { return 'nope'; }\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "function return type changed");

        // Revert.
        db.update(FileId::new("src/lib.php"), Cow::Owned(lib.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "function reverted");
    }

    /// Create a class, add a child in another cycle, then modify parent, then delete child.
    #[test]
    fn test_watch_lifecycle_create_inherit_modify_delete() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial — base only");

        // Add a child.
        let child = "<?php\nclass Child extends Base {\n    public function run(): string { return 'child'; }\n    private function secret(): void {}\n}\n";
        db.add(File::new(Cow::Owned("src/Child.php".to_string()), FileType::Host, None, Cow::Owned(child.to_string())));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child added");

        // Modify parent return type.
        db.update(
            FileId::new("src/Base.php"),
            Cow::Owned("<?php\nclass Base {\n    public function run(): int { return 42; }\n}\n".to_string()),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent modified after child added");

        // Delete child.
        db.delete(FileId::new("src/Child.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child deleted");

        // Revert parent.
        db.update(FileId::new("src/Base.php"), Cow::Owned(base.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "parent reverted after child deleted");
    }

    /// Back-to-back file additions then deletion of all at once.
    #[test]
    fn test_watch_add_multiple_files_then_delete_all() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";

        let mut db = make_database(vec![("src/Base.php", base)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Add three children at once.
        for name in ["src/A.php", "src/B.php", "src/C.php"] {
            let content = format!(
                "<?php\nclass {} extends Base {{\n    public function run(): float {{ return 1.0; }}\n}}\n",
                &name[4..5]
            );
            db.add(File::new(Cow::Owned(name.to_string()), FileType::Host, None, Cow::Owned(content)));
        }
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "three children added");

        // Delete all three at once.
        for name in ["src/A.php", "src/B.php", "src/C.php"] {
            db.delete(FileId::new(name));
        }
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "all children deleted");
    }

    /// Swap class between two files (move A→B and B→A simultaneously).
    #[test]
    fn test_watch_swap_classes_between_files() {
        let file_a_v1 = "<?php\nclass Alpha {\n    public function name(): string { return 'alpha'; }\n}\n";
        let file_b_v1 = "<?php\nclass Beta {\n    public function name(): string { return 'beta'; }\n}\n";
        let user = "<?php\nfunction test(): string { return (new Alpha())->name() . (new Beta())->name(); }\n";

        let mut db = make_database(vec![("src/a.php", file_a_v1), ("src/b.php", file_b_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Swap: Alpha goes to b.php, Beta goes to a.php.
        let file_a_v2 = "<?php\nclass Beta {\n    public function name(): string { return 'beta'; }\n}\n";
        let file_b_v2 = "<?php\nclass Alpha {\n    public function name(): string { return 'alpha'; }\n}\n";
        db.update(FileId::new("src/a.php"), Cow::Owned(file_a_v2.to_string()));
        db.update(FileId::new("src/b.php"), Cow::Owned(file_b_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "classes swapped between files");
    }

    /// Rename a class (old class disappears, new class appears in same file).
    #[test]
    fn test_watch_rename_class() {
        let file_v1 = "<?php\nclass OldName {\n    public function run(): string { return 'ok'; }\n}\n";
        let user = "<?php\nfunction test(): string { return (new OldName())->run(); }\n";

        let mut db = make_database(vec![("src/class.php", file_v1), ("src/user.php", user)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Rename class.
        let file_v2 = "<?php\nclass NewName {\n    public function run(): string { return 'ok'; }\n}\n";
        db.update(FileId::new("src/class.php"), Cow::Owned(file_v2.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "class renamed");
    }

    /// Rapid alternation: delete, re-add, modify, revert over multiple cycles.
    #[test]
    fn test_watch_chaotic_lifecycle() {
        let base = "<?php\nclass Base {\n    public function run(): string { return 'ok'; }\n}\n";
        let child_original = "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n    private function unused(): void {}\n}\n";

        let mut db = make_database(vec![("src/Base.php", base), ("src/Child.php", child_original)]);
        let mut service = make_watch_service(&db);
        service.analyze().expect("Initial analysis failed.");
        assert_matches_full(&service, &db, "initial");

        // Delete child.
        db.delete(FileId::new("src/Child.php"));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child deleted");

        // Re-add child with different content.
        db.add(File::new(
            Cow::Owned("src/Child.php".to_string()),
            FileType::Host,
            None,
            Cow::Owned(
                "<?php\nclass Child extends Base {\n    public function run(): string { return 'compatible'; }\n}\n"
                    .to_string(),
            ),
        ));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child re-added compatible");

        // Modify child to be incompatible again.
        db.update(FileId::new("src/Child.php"), Cow::Owned(child_original.to_string()));
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "child made incompatible again");

        // Modify base at the same time as child.
        db.update(
            FileId::new("src/Base.php"),
            Cow::Owned("<?php\nclass Base {\n    public function run(): float { return 0.5; }\n}\n".to_string()),
        );
        db.update(
            FileId::new("src/Child.php"),
            Cow::Owned(
                "<?php\nclass Child extends Base {\n    public function run(): float { return 1.0; }\n}\n".to_string(),
            ),
        );
        service.update_database(db.read_only());
        service.analyze_incremental(None).expect("Incremental failed.");
        assert_matches_full(&service, &db, "both changed to be compatible");
    }
}
