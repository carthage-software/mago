use std::sync::Arc;
use std::time::Duration;

use ahash::HashMap;
use ahash::HashSet;
use bumpalo::Bump;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_atom::AtomSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::FileId;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

use crate::error::OrchestratorError;
use crate::incremental::IncrementalAnalysis;
use crate::service::incremental_pipeline::FileState;
use crate::service::incremental_pipeline::IncrementalParallelPipeline;
use crate::service::pipeline::ParallelPipeline;
use crate::service::pipeline::Reducer;

pub struct AnalysisService {
    database: ReadDatabase,
    codebase: CodebaseMetadata,
    symbol_references: SymbolReferences,
    settings: Settings,
    use_progress_bars: bool,
    incremental: Option<IncrementalAnalysis>,
    file_states: HashMap<FileId, FileState>,
    plugin_registry: Arc<PluginRegistry>,
}

impl std::fmt::Debug for AnalysisService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisService")
            .field("database", &self.database)
            .field("codebase", &self.codebase)
            .field("symbol_references", &self.symbol_references)
            .field("settings", &self.settings)
            .field("use_progress_bars", &self.use_progress_bars)
            .field("incremental", &self.incremental)
            .field("file_states", &format!("{} tracked files", self.file_states.len()))
            .field("plugin_registry", &self.plugin_registry)
            .finish()
    }
}

impl AnalysisService {
    #[must_use]
    pub fn new(
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        settings: Settings,
        use_progress_bars: bool,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            database,
            codebase,
            symbol_references,
            settings,
            use_progress_bars,
            incremental: None,
            file_states: HashMap::default(),
            plugin_registry,
        }
    }

    /// Sets the incremental analysis manager for this service.
    ///
    /// When set, the service will use the `run_incremental()` method which detects
    /// file changes and only re-scans modified files for improved performance.
    #[must_use]
    pub fn with_incremental(mut self, incremental: IncrementalAnalysis) -> Self {
        self.incremental = Some(incremental);
        self
    }

    /// Analyzes a single file synchronously without using parallel processing.
    ///
    /// This method is designed for environments where threading is not available,
    /// such as WebAssembly. It performs static analysis on a single file by:
    /// 1. Parsing the file
    /// 2. Resolving names
    /// 3. Scanning symbols and extending the provided codebase
    /// 4. Populating the codebase (resolving inheritance, traits, etc.)
    /// 5. Running the analyzer
    ///
    /// # Arguments
    ///
    /// * `file` - The file to analyze.
    /// * `settings` - The analyzer settings.
    /// * `codebase` - The base codebase metadata (e.g., from prelude with PHP built-in symbols).
    /// * `symbol_references` - The base symbol references (e.g., from prelude).
    ///
    /// # Returns
    ///
    /// An `IssueCollection` containing all issues found in the file.
    pub fn oneshot(mut self, file_id: FileId) -> IssueCollection {
        let Ok(file) = self.database.get_ref(&file_id) else {
            tracing::error!("File with ID {:?} not found in database", file_id);

            return IssueCollection::default();
        };

        let arena = Bump::new();

        let (program, parsing_error) = parse_file(&arena, file);
        let resolved_names = NameResolver::new(&arena).resolve(program);

        let mut issues = IssueCollection::new();
        if let Some(error) = parsing_error {
            issues.push(Issue::from(&error));
            return issues;
        }

        let semantics_checker = SemanticsChecker::new(self.settings.version);
        issues.extend(semantics_checker.check(file, program, &resolved_names));

        let user_codebase = scan_program(&arena, file, program, &resolved_names);
        self.codebase.extend(user_codebase);

        populate_codebase(&mut self.codebase, &mut self.symbol_references, AtomSet::default(), HashSet::default());

        // Run the analyzer
        let mut analysis_result = AnalysisResult::new(self.symbol_references);
        let analyzer =
            Analyzer::new(&arena, file, &resolved_names, &self.codebase, &self.plugin_registry, self.settings);

        if let Err(err) = analyzer.analyze(program, &mut analysis_result) {
            issues.push(Issue::error(format!("Analysis error: {err}")));
        }

        issues.extend(analysis_result.issues);
        issues.extend(self.codebase.take_issues(true));
        issues
    }

    /// Updates the database for a new analysis run (for watch mode).
    /// This allows reusing the service without recreating it.
    pub fn update_database(&mut self, database: ReadDatabase) {
        self.database = database;
    }

    /// Gets a reference to the codebase (for incremental analysis state saving).
    #[must_use]
    pub fn codebase(&self) -> &CodebaseMetadata {
        &self.codebase
    }

    /// Gets a reference to the symbol references (for incremental analysis state saving).
    #[must_use]
    pub fn symbol_references(&self) -> &SymbolReferences {
        &self.symbol_references
    }

    pub fn run(&mut self) -> Result<AnalysisResult, OrchestratorError> {
        #[cfg(not(target_arch = "wasm32"))]
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🔬 Analyzing";

        // Temporarily take ownership of fields to pass to pipeline
        let database = std::mem::replace(&mut self.database, ReadDatabase::empty());
        let codebase = std::mem::take(&mut self.codebase);
        let symbol_references = std::mem::take(&mut self.symbol_references);
        let incremental = self.incremental.take();

        let mut pipeline = ParallelPipeline::new(
            ANALYSIS_PROGRESS_PREFIX,
            database,
            codebase,
            symbol_references,
            self.settings.clone(),
            Box::new(AnalysisResultReducer),
            self.use_progress_bars,
        );

        if let Some(inc) = incremental {
            let mut inc_for_callback = inc.clone();
            pipeline = pipeline.with_after_scanning(move |codebase, _symbol_refs| {
                if let Some((old_metadata, old_refs)) = inc_for_callback.load_previous_state() {
                    tracing::debug!("Applying incremental analysis...");

                    // Compute diffs
                    let diff = inc_for_callback.compute_diffs(&old_metadata, codebase);

                    // Mark safe symbols (includes invalidation cascade)
                    inc_for_callback.mark_safe_symbols(&diff, &old_refs, codebase);

                    tracing::debug!(
                        "Incremental analysis complete: {} safe symbols, {} safe members",
                        codebase.safe_symbols.len(),
                        codebase.safe_symbol_members.len()
                    );
                } else {
                    tracing::debug!("No previous cache found, performing full analysis");
                }
            });
            // Restore incremental to self for next run
            self.incremental = Some(inc);
        }

        let plugin_registry = Arc::clone(&self.plugin_registry);
        let (analysis_result, codebase, symbol_references) =
            pipeline.run(move |settings, arena, source_file, codebase| {
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                let (program, parsing_error) = parse_file(arena, &source_file);
                let resolved_names = NameResolver::new(arena).resolve(program);

                if let Some(parsing_error) = parsing_error {
                    analysis_result.issues.push(Issue::from(&parsing_error));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);
                let analyzer =
                    Analyzer::new(arena, &source_file, &resolved_names, &codebase, &plugin_registry, settings);

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

                Ok(analysis_result)
            })?;

        // Store the updated codebase and symbol_references back into self for next run
        self.codebase = codebase;
        self.symbol_references = symbol_references;

        Ok(analysis_result)
    }

    /// Runs incremental analysis optimized for watch mode.
    ///
    /// This method uses file content hashing to detect changes and only re-scans
    /// changed files, significantly improving performance for subsequent runs.
    ///
    /// # Returns
    ///
    /// Returns the analysis result for the current run.
    pub fn run_incremental(&mut self) -> Result<AnalysisResult, OrchestratorError> {
        #[cfg(not(target_arch = "wasm32"))]
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🔬 Analyzing";

        // Temporarily take ownership of fields to pass to pipeline
        let database = std::mem::replace(&mut self.database, ReadDatabase::empty());
        let codebase = std::mem::take(&mut self.codebase);
        let symbol_references = std::mem::take(&mut self.symbol_references);
        let file_states = std::mem::take(&mut self.file_states);
        let incremental = self.incremental.take();

        let mut pipeline = IncrementalParallelPipeline::new(
            ANALYSIS_PROGRESS_PREFIX,
            database,
            codebase,
            symbol_references,
            self.settings.clone(),
            Box::new(AnalysisResultReducer),
            file_states,
        );

        if let Some(inc) = incremental {
            let mut inc_for_callback = inc.clone();
            pipeline = pipeline.with_after_scanning(move |codebase, _symbol_refs| {
                if let Some((old_metadata, old_refs)) = inc_for_callback.load_previous_state() {
                    tracing::debug!("Applying incremental analysis...");

                    // Compute diffs
                    let diff = inc_for_callback.compute_diffs(&old_metadata, codebase);

                    // Mark safe symbols (includes invalidation cascade)
                    inc_for_callback.mark_safe_symbols(&diff, &old_refs, codebase);

                    tracing::debug!(
                        "Incremental analysis complete: {} safe symbols, {} safe members",
                        codebase.safe_symbols.len(),
                        codebase.safe_symbol_members.len()
                    );
                } else {
                    tracing::debug!("No previous cache found, performing full analysis");
                }
            });
            // Restore incremental to self for next run
            self.incremental = Some(inc);
        }

        let plugin_registry = Arc::clone(&self.plugin_registry);
        let (analysis_result, codebase, symbol_references, new_file_states) =
            pipeline.run(move |settings, arena, source_file, codebase| {
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                let (program, parsing_error) = parse_file(arena, &source_file);
                let resolved_names = NameResolver::new(arena).resolve(program);

                if let Some(parsing_error) = parsing_error {
                    analysis_result.issues.push(Issue::from(&parsing_error));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);
                let analyzer =
                    Analyzer::new(arena, &source_file, &resolved_names, &codebase, &plugin_registry, settings);

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

                Ok(analysis_result)
            })?;

        // Store the updated state back into self for next run
        self.codebase = codebase.clone();
        self.symbol_references = symbol_references.clone();
        self.file_states = new_file_states;

        // Save state to incremental analysis manager for next run
        if let Some(ref mut incremental) = self.incremental {
            incremental.save_state(codebase, symbol_references);
        }

        Ok(analysis_result)
    }
}

/// The "reduce" step for the analysis pipeline.
///
/// This struct aggregates the `AnalysisResult` from each parallel task into a single,
/// final `AnalysisResult` for the entire project.
#[derive(Debug, Clone)]
struct AnalysisResultReducer;

impl Reducer<AnalysisResult, AnalysisResult> for AnalysisResultReducer {
    fn reduce(
        &self,
        mut codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        results: Vec<AnalysisResult>,
    ) -> Result<(AnalysisResult, CodebaseMetadata, SymbolReferences), OrchestratorError> {
        let mut aggregated_result = AnalysisResult::new(symbol_references.clone());
        for result in results {
            aggregated_result.extend(result);
        }

        aggregated_result.issues.extend(codebase.take_issues(true));

        Ok((aggregated_result, codebase, symbol_references))
    }
}
