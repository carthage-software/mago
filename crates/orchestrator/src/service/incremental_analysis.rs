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

    /// Assembles the full issue list from codebase-level issues + all per-file cached issues.
    ///
    /// This avoids storing a separate snapshot by reconstructing on demand.
    fn collect_all_issues(&self) -> IssueCollection {
        let total: usize =
            self.codebase_issues.len() + self.file_states.values().map(|s| s.analysis_issues.len()).sum::<usize>();

        let mut issues = IssueCollection::default();
        issues.reserve(total);
        issues.extend(self.codebase_issues.iter().cloned());
        for state in self.file_states.values() {
            if !state.analysis_issues.is_empty() {
                issues.extend(state.analysis_issues.iter().cloned());
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
            file_states
                .insert(file_id, FileState { content_hash, entry_keys, analysis_issues: IssueCollection::default() });
        }

        let mut symbol_references = (*self.base_symbol_references).clone();
        populate_codebase(&mut merged_codebase, &mut symbol_references, AtomSet::default(), HashSet::default());

        let (mut analysis_result, per_file_issues) =
            self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, HashSet::default())?;

        self.codebase_issues = merged_codebase.take_issues(true);
        analysis_result.issues.extend(self.codebase_issues.iter().cloned());

        for (file_id, issues) in per_file_issues {
            analysis_result.issues.extend(issues.iter().cloned());
            if let Some(state) = file_states.get_mut(&file_id) {
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
        self.file_states = file_states;
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
            for (file_id, new_metadata) in &new_file_scans {
                if let Some(prev_state) = self.file_states.get(file_id) {
                    merged_codebase.remove_entries_by_keys(&prev_state.entry_keys);
                }

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

            let safe_symbols: AtomSet = merged_codebase.class_likes.keys().copied().collect();
            populate_codebase_targeted(
                &mut merged_codebase,
                &mut symbol_references,
                safe_symbols,
                HashSet::default(),
                changed_symbols,
            );

            let (mut analysis_result, mut per_file_issues) =
                self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, files_to_skip)?;

            self.codebase_issues = merged_codebase.take_issues(true);

            for (file_id, metadata) in new_file_scans {
                let content_hash = file_hashes[&file_id];
                let issues = per_file_issues.remove(&file_id).unwrap_or_default();
                let entry_keys = metadata.extract_keys();
                self.file_states.insert(file_id, FileState { content_hash, entry_keys, analysis_issues: issues });
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

        for (file_id, new_metadata) in &new_file_scans {
            if let Some(prev_state) = self.file_states.get(file_id) {
                merged_codebase.remove_entries_by_keys(&prev_state.entry_keys);
            }

            merged_codebase.extend_ref(new_metadata);
        }

        merged_codebase.safe_symbols.clear();
        merged_codebase.safe_symbol_members.clear();

        if !merged_codebase.mark_safe_symbols(&diff, &self.symbol_references) {
            tracing::warn!("Invalidation cascade too expensive (>5000 steps), falling back to full analysis");

            return self.analyze();
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

        let (mut analysis_result, mut per_file_issues) =
            self.run_analyzer_selective(&merged_codebase, symbol_references, &self.settings, files_to_skip)?;

        self.codebase_issues = merged_codebase.take_issues(true);
        self.file_states.retain(|id, _| current_file_ids.contains(id));

        let changed_file_ids: HashSet<FileId> = new_file_scans.iter().map(|(fid, _)| *fid).collect();
        let re_analyzed_unchanged: Vec<FileId> = per_file_issues
            .keys()
            .filter(|fid| self.file_states.contains_key(fid) && !changed_file_ids.contains(fid))
            .copied()
            .collect();

        for file_id in re_analyzed_unchanged {
            if let Some(issues) = per_file_issues.remove(&file_id)
                && let Some(state) = self.file_states.get_mut(&file_id)
            {
                state.analysis_issues = issues;
            }
        }

        for (file_id, metadata) in new_file_scans {
            let content_hash = file_hashes[&file_id];
            let issues = per_file_issues.remove(&file_id).unwrap_or_default();
            let entry_keys = metadata.extract_keys();
            self.file_states.insert(file_id, FileState { content_hash, entry_keys, analysis_issues: issues });
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
        skip_files: HashSet<FileId>,
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
        let _ = service.analyze().unwrap();
        assert!(service.is_initialized());
    }

    #[test]
    fn test_incremental_no_change_returns_same_result() {
        let db = make_database(vec![
            ("src/a.php", "<?php\nfunction foo(): int { return 42; }\n"),
            ("src/b.php", "<?php\nfunction bar(): string { return 'hello'; }\n"),
        ]);

        let mut service = make_service(&db);
        let initial = service.analyze().unwrap();

        let incremental = service.analyze_incremental(None).unwrap();

        assert_eq!(initial.issues.len(), incremental.issues.len());
    }

    #[test]
    fn test_incremental_after_body_change_matches_full() {
        let mut db = make_database(vec![
            ("src/a.php", "<?php\nfunction get_value(): int { return 42; }\n"),
            ("src/b.php", "<?php\nfunction use_value(): int { return get_value(); }\n"),
        ]);

        let mut service = make_service(&db);
        let _initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction get_value(): int { return 99; }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(): string { return 'hello'; }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.delete(FileId::new("src/a.php"));

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.add(File::new(
            Cow::Owned("src/b.php".to_string()),
            FileType::Host,
            None,
            Cow::Owned("<?php\nfunction new_func(): int { return existing(); }\n".to_string()),
        ));

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(FileId::new("src/a.php"), Cow::Owned(modified_content.to_string()));
        service.update_database(db.read_only());
        let _modified = service.analyze_incremental(None).unwrap();

        db.update(FileId::new("src/a.php"), Cow::Owned(original_content.to_string()));
        service.update_database(db.read_only());
        let reverted = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(int $x): int { return $x * 3; }\n".to_string()),
        );
        service.update_database(db.read_only());
        let _cycle1 = service.analyze_incremental(None).unwrap();

        let cycle2 = service.analyze_incremental(None).unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction compute(int $x): string { return (string)($x * 3); }\n".to_string()),
        );
        service.update_database(db.read_only());
        let cycle3 = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

        let (only_incr, only_full) = diff_issues(&cycle3.issues, &full.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "After multiple cycles, incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        assert_eq!(_cycle1.issues.len(), cycle2.issues.len(), "No-change cycle should produce same issue count");
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
        let _initial = service.analyze().unwrap();

        let parent_v2 =
            concat!("<?php\n", "class Animal {\n", "    public function speak(): int { return 42; }\n", "}\n",);
        db.update(FileId::new("src/Animal.php"), Cow::Owned(parent_v2.to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let initial = service.analyze().unwrap();
        let initial_count = initial.issues.len();

        let a_broken = "<?php\nfunction get_count(): string { return 'not a number'; }\n";
        db.update(FileId::new("src/a.php"), Cow::Owned(a_broken.to_string()));
        service.update_database(db.read_only());
        let broken = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full_broken = fresh_service.analyze().unwrap();
        let (only_incr, only_full) = diff_issues(&broken.issues, &full_broken.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Broken state: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        db.update(FileId::new("src/a.php"), Cow::Owned(a_valid.to_string()));
        service.update_database(db.read_only());
        let fixed = service.analyze_incremental(None).unwrap();

        let mut fresh_service2 = make_service(&db);
        let full_fixed = fresh_service2.analyze().unwrap();
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
        let _initial = service.analyze().unwrap();

        // Change both a.php and b.php at the same time
        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction alpha(): string { return 'a'; }\n".to_string()),
        );
        db.update(FileId::new("src/b.php"), Cow::Owned("<?php\nfunction beta(): string { return 'b'; }\n".to_string()));

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction broken( { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let with_error = service.analyze_incremental(None).unwrap();

        // Should have at least one parse error
        assert!(!with_error.issues.is_empty(), "Parse error should produce at least one issue");

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction valid(): int { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let fixed = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned(
                "<?php\nfunction first(): int { return 1; }\nfunction second(): string { return 'two'; }\n".to_string(),
            ),
        );
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction first(): int { return 1; }\n".to_string()));
        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

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
            let incremental = service.analyze_incremental(None).unwrap();

            let mut fresh_service = make_service(&db);
            let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\nfunction a(): string { return 'a'; }\n".to_string()));
        db.update(FileId::new("src/b.php"), Cow::Owned("<?php\nfunction b(): string { return 'b'; }\n".to_string()));
        db.update(
            FileId::new("src/c.php"),
            Cow::Owned("<?php\nfunction c(): string { return a() . b(); }\n".to_string()),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let _initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned(
                "<?php\nclass NewClass {\n    public function method(): string { return 'hello'; }\n}\n".to_string(),
            ),
        );

        service.update_database(db.read_only());
        let incremental = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full = fresh_service.analyze().unwrap();

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
        let initial = service.analyze().unwrap();

        db.update(
            FileId::new("src/a.php"),
            Cow::Owned("<?php\nfunction new_thing(): int { return 42; }\n".to_string()),
        );
        service.update_database(db.read_only());
        let with_content = service.analyze_incremental(None).unwrap();

        let mut fresh_service = make_service(&db);
        let full_with = fresh_service.analyze().unwrap();
        let (only_incr, only_full) = diff_issues(&with_content.issues, &full_with.issues);
        assert!(
            only_incr.is_empty() && only_full.is_empty(),
            "Empty to nonempty: incremental != full.\n  Only in incremental: {only_incr:?}\n  Only in full: {only_full:?}"
        );

        db.update(FileId::new("src/a.php"), Cow::Owned("<?php\n".to_string()));
        service.update_database(db.read_only());
        let back_to_empty = service.analyze_incremental(None).unwrap();

        let mut fresh_service2 = make_service(&db);
        let full_empty = fresh_service2.analyze().unwrap();
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
        let _initial = service.analyze().unwrap();

        let baseline_refs = service.symbol_references().total_map_entries();
        let baseline_files = service.tracked_file_count();

        for i in 0..50 {
            let edited = format!(
                "<?php\nclass Greeter {{\n    public function greet(string $name): string {{ return 'Hi #{} ' . $name; }}\n}}\n",
                i
            );
            db.update(FileId::new("src/greeter.php"), Cow::Owned(edited));
            service.update_database(db.read_only());
            let _result = service.analyze_incremental(None).unwrap();

            db.update(FileId::new("src/greeter.php"), Cow::Owned(original.to_string()));
            service.update_database(db.read_only());
            let _result = service.analyze_incremental(None).unwrap();
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
        let _initial = service.analyze().unwrap();

        let baseline_refs = service.symbol_references().total_map_entries();

        for i in 0..30 {
            let edited = format!("<?php\nfunction compute(): string {{ return 'value_{}'; }}\n", i);
            db.update(FileId::new("src/compute.php"), Cow::Owned(edited));
            service.update_database(db.read_only());
            let _result = service.analyze_incremental(None).unwrap();

            db.update(FileId::new("src/compute.php"), Cow::Owned(original.to_string()));
            service.update_database(db.read_only());
            let _result = service.analyze_incremental(None).unwrap();
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
        let _initial = service.analyze().unwrap();

        let baseline_refs = service.symbol_references().total_map_entries();
        let baseline_files = service.tracked_file_count();

        for i in 0..20 {
            let body_edit = format!("<?php\nfunction alpha(): int {{ return {}; }}\n", i);
            db.update(FileId::new("src/a.php"), Cow::Owned(body_edit));
            service.update_database(db.read_only());
            let _r = service.analyze_incremental(None).unwrap();

            db.update(
                FileId::new("src/a.php"),
                Cow::Owned("<?php\nfunction alpha(): string { return 'x'; }\n".to_string()),
            );
            service.update_database(db.read_only());
            let _r = service.analyze_incremental(None).unwrap();

            db.update(
                FileId::new("src/a.php"),
                Cow::Owned("<?php\nfunction alpha(): string { return 'x'; }\nfunction extra(): void {}\n".to_string()),
            );
            service.update_database(db.read_only());
            let _r = service.analyze_incremental(None).unwrap();

            db.update(FileId::new("src/a.php"), Cow::Owned(base.to_string()));
            service.update_database(db.read_only());
            let _r = service.analyze_incremental(None).unwrap();
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
}
