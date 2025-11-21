use std::time::Duration;

use ahash::HashMap;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::settings::Settings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
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
            .finish()
    }
}

impl AnalysisService {
    pub fn new(
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        settings: Settings,
        use_progress_bars: bool,
    ) -> Self {
        Self {
            database,
            codebase,
            symbol_references,
            settings,
            use_progress_bars,
            incremental: None,
            file_states: HashMap::default(),
        }
    }

    /// Sets the incremental analysis manager for this service.
    ///
    /// When set, the service will use the `run_incremental()` method which detects
    /// file changes and only re-scans modified files for improved performance.
    pub fn with_incremental(mut self, incremental: IncrementalAnalysis) -> Self {
        self.incremental = Some(incremental);
        self
    }

    /// Updates the database for a new analysis run (for watch mode).
    /// This allows reusing the service without recreating it.
    ///
    /// # Arguments
    ///
    /// * `database` - The new database snapshot to use for analysis
    /// * `changed_file_ids` - File IDs that have been modified and need their cached signatures cleared
    pub fn update_database(&mut self, database: ReadDatabase, changed_file_ids: &[FileId]) {
        tracing::debug!(
            "AnalysisService::update_database() replacing database (old file count: {}, new file count: {})",
            self.database.len(),
            database.len()
        );

        // Clear file_signatures and file_states for changed files to force re-analysis
        // This prevents the incremental pipeline from using stale AST signatures or content hashes
        for file_id in changed_file_ids {
            self.codebase.remove_file_signature(file_id);
            self.file_states.remove(file_id);
            tracing::debug!("Cleared file_signature and file_state for FileId={}", file_id);
        }

        self.database = database;
    }

    /// Gets a reference to the codebase (for incremental analysis state saving).
    pub fn codebase(&self) -> &CodebaseMetadata {
        &self.codebase
    }

    /// Gets a reference to the symbol references (for incremental analysis state saving).
    pub fn symbol_references(&self) -> &SymbolReferences {
        &self.symbol_references
    }

    pub fn run(&mut self) -> Result<AnalysisResult, OrchestratorError> {
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🕵️  Analyzing";

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
            self.settings,
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
                    inc_for_callback.mark_safe_symbols(diff, &old_refs, codebase);

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

        let (analysis_result, codebase, symbol_references, database) =
            pipeline.run(|settings, arena, source_file, codebase| {
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                let (program, parsing_error) = parse_file(arena, &source_file);
                let resolved_names = NameResolver::new(arena).resolve(program);

                if let Some(parsing_error) = parsing_error {
                    analysis_result.issues.push(Issue::from(&parsing_error));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);
                let analyzer = Analyzer::new(arena, &source_file, &resolved_names, &codebase, settings);

                analysis_result.issues.extend(semantics_checker.check(&source_file, program, &resolved_names));
                analyzer.analyze(program, &mut analysis_result)?;

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

        // Store the updated codebase, symbol_references, and database back into self for next run
        self.codebase = codebase;
        self.symbol_references = symbol_references;
        self.database = database;

        // Save state to incremental analysis manager for next run
        // This allows the first run_incremental() after run() to use the previous state
        if let Some(ref mut incremental) = self.incremental {
            incremental.save_state(self.codebase.clone(), self.symbol_references.clone());
        }

        // Populate file_states so run_incremental() knows existing files and their hashes
        // This prevents the first run_incremental() from treating all files as "new"
        self.file_states = self
            .database
            .files()
            .filter(|f| f.file_type != FileType::Builtin)
            .map(|f| {
                let hash = xxhash_rust::xxh3::xxh3_64(f.contents.as_bytes());
                (f.id, FileState { content_hash: hash })
            })
            .collect();

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
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🕵️  Analyzing";

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
            self.settings,
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
                    inc_for_callback.mark_safe_symbols(diff, &old_refs, codebase);

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

        let (analysis_result, codebase, symbol_references, new_file_states, database) =
            pipeline.run(|settings, arena, source_file, codebase| {
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                let (program, parsing_error) = parse_file(arena, &source_file);
                let resolved_names = NameResolver::new(arena).resolve(program);

                if let Some(parsing_error) = parsing_error {
                    analysis_result.issues.push(Issue::from(&parsing_error));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);
                let analyzer = Analyzer::new(arena, &source_file, &resolved_names, &codebase, settings);

                analysis_result.issues.extend(semantics_checker.check(&source_file, program, &resolved_names));
                analyzer.analyze(program, &mut analysis_result)?;

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
        self.database = database;

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
        let mut aggregated_result = AnalysisResult::new(symbol_references);
        for result in results {
            aggregated_result.extend(result);
        }

        aggregated_result.issues.extend(codebase.take_issues(true));

        // Extract the merged symbol references that include references from analyzer
        let merged_references = std::mem::take(&mut aggregated_result.symbol_references);

        Ok((aggregated_result, codebase, merged_references))
    }
}
