use std::sync::Arc;
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::Ordering::Relaxed;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use foldhash::HashSet;
use rayon::prelude::*;

use mago_allocator::LocalArena;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::analysis_result::LateSymbolReferenceIssueReconciler;
use mago_analyzer::artifacts::AnalysisArtifacts;
use mago_analyzer::error::AnalysisError;
use mago_analyzer::external::AFTER_FILE_ANALYSIS_BATCH_SIZE;
use mago_analyzer::external::FileAnalysisSnapshot;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
#[cfg(not(target_arch = "wasm32"))]
use mago_analyzer::telemetry as analyzer_telemetry;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::settings::ParserSettings;
use mago_word::WordSet;

use crate::error::OrchestratorError;
use crate::service::issue_reconciliation::DeferredIssueReconciler;
use crate::service::pipeline::ParallelPipeline;
use crate::service::pipeline::Reducer;
#[cfg(not(target_arch = "wasm32"))]
use crate::service::telemetry::AnalysisPhaseTelemetry;

pub struct AnalysisService {
    database: ReadDatabase,
    codebase: CodebaseMetadata,
    symbol_references: SymbolReferences,
    settings: Settings,
    parser_settings: ParserSettings,
    use_progress_bars: bool,
    plugin_registry: Arc<PluginRegistry>,
}

impl std::fmt::Debug for AnalysisService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisService")
            .field("database", &self.database)
            .field("codebase", &self.codebase)
            .field("symbol_references", &self.symbol_references)
            .field("settings", &self.settings)
            .field("parser_settings", &self.parser_settings)
            .field("use_progress_bars", &self.use_progress_bars)
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
        parser_settings: ParserSettings,
        use_progress_bars: bool,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self { database, codebase, symbol_references, settings, parser_settings, use_progress_bars, plugin_registry }
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
    /// * `file_id` - The ID of the file to analyze.
    ///
    /// # Returns
    ///
    /// An `IssueCollection` containing all issues found in the file.
    pub fn oneshot(mut self, file_id: FileId) -> IssueCollection {
        let external_session = self.plugin_registry.create_external_analysis_session(self.database.files());
        let Ok(file) = self.database.get_ref(&file_id) else {
            tracing::error!("File with ID {:?} not found in database", file_id);

            return IssueCollection::default();
        };

        let arena = LocalArena::new();

        let program = parse_file_with_settings(&arena, file, self.parser_settings);
        let resolved_names = NameResolver::new(&arena).resolve(program);

        let mut issues = IssueCollection::new();
        if program.has_errors() {
            for error in program.errors.iter() {
                issues.push(Issue::from(error));
            }
        }

        let semantics_checker = SemanticsChecker::new(self.settings.version);
        issues.extend(semantics_checker.check(file, program, &resolved_names));

        let user_codebase = scan_program(&arena, file, program, &resolved_names, self.settings.version);
        self.codebase.extend(user_codebase);

        populate_codebase(&mut self.codebase, &mut self.symbol_references, WordSet::default(), HashSet::default());

        if let Err(err) = self.plugin_registry.prepare_external_analyzer() {
            issues.push(Issue::error(format!("Analysis error: {err}")));
            return issues;
        }

        let additional_symbol_references =
            match self.plugin_registry.run_external_before_analysis_hooks(&self.codebase, external_session.as_ref()) {
                Ok(reported) => {
                    issues.extend(reported.issues);
                    reported.references
                }
                Err(err) => {
                    issues.push(Issue::error(format!("Analysis error: {err}")));
                    return issues;
                }
            };
        if !additional_symbol_references.is_empty() {
            self.symbol_references.extend(additional_symbol_references.clone());
        }

        let after_file = self.plugin_registry.has_external_after_file_analysis_hooks().unwrap_or_default();
        let after_analysis = self.plugin_registry.has_external_after_analysis_hooks().unwrap_or_default();
        let node_analysis_requirements = self.plugin_registry.external_node_analysis_requirements().unwrap_or_default();

        // Run the analyzer
        let mut analysis_result = AnalysisResult::new(self.symbol_references);
        let mut analyzer =
            Analyzer::new(&arena, file, &resolved_names, &self.codebase, &self.plugin_registry, self.settings);
        if after_file || after_analysis {
            analyzer = analyzer.with_deferred_pragmas();
        }
        if let Some(session) = external_session.as_ref() {
            analyzer = analyzer.with_external_analysis_session(session);
        }
        if !additional_symbol_references.is_empty() {
            analyzer = analyzer.with_additional_symbol_references(&additional_symbol_references);
        }

        let artifacts = match analyzer.analyze_with_artifacts(program, &mut analysis_result) {
            Ok(artifacts) => artifacts,
            Err(err) => {
                issues.push(Issue::error(format!("Analysis error: {err}")));
                AnalysisArtifacts::new()
            }
        };

        if after_file {
            match self.plugin_registry.run_external_after_file_analysis_hooks(
                file,
                program,
                &resolved_names,
                &artifacts,
                &self.codebase,
                external_session.as_ref(),
            ) {
                Ok(reported) => {
                    analysis_result.issues.extend(reported.issues);
                    let has_late_references = !reported.references_by_file.is_empty();
                    for references in reported.references_by_file.into_values() {
                        analysis_result.symbol_references.extend(references);
                    }

                    if has_late_references {
                        analysis_result.issues =
                            LateSymbolReferenceIssueReconciler::new(&self.codebase, &analysis_result.symbol_references)
                                .reconcile(std::mem::take(&mut analysis_result.issues));
                    }
                }
                Err(err) => issues.push(Issue::error(format!("Analysis error: {err}"))),
            }
        }

        let mut pragma_reconciler =
            DeferredIssueReconciler::new(analysis_result.take_deferred_pragmas(), self.database.files());
        analysis_result.issues = match pragma_reconciler.reconcile(std::mem::take(&mut analysis_result.issues)) {
            Ok(issues) => issues,
            Err(error) => {
                issues.push(Issue::error(format!("Analysis error: {error}")));
                return issues;
            }
        };
        issues.extend(analysis_result.issues.iter().cloned());
        issues.extend(self.codebase.take_issues(true));
        if after_analysis {
            let snapshot = match FileAnalysisSnapshot::new(
                file,
                program,
                &resolved_names,
                &artifacts,
                &self.codebase,
                node_analysis_requirements.as_ref(),
            ) {
                Ok(snapshot) => Arc::new(snapshot),
                Err(err) => {
                    issues.push(Issue::error(format!("Analysis error: {err}")));
                    return issues;
                }
            };
            let mut project_result = AnalysisResult::new(analysis_result.symbol_references);
            project_result.issues = issues.clone();
            match self.plugin_registry.run_external_after_analysis_hooks(
                &project_result,
                &[snapshot],
                &self.codebase,
                external_session.as_ref(),
            ) {
                Ok(reported) => match pragma_reconciler.reconcile(reported) {
                    Ok(reported) => issues.extend(reported),
                    Err(error) => issues.push(Issue::error(format!("Analysis error: {error}"))),
                },
                Err(err) => issues.push(Issue::error(format!("Analysis error: {err}"))),
            }
        }

        match pragma_reconciler.finish() {
            Ok(reported) => issues.extend(reported),
            Err(error) => issues.push(Issue::error(format!("Analysis error: {error}"))),
        }

        issues
    }

    /// Runs the full analysis pipeline.
    ///
    /// This method scans all source files, builds the codebase, and runs the analyzer.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when scanning, codebase population, or per-file analysis fails.
    pub fn run(self) -> Result<AnalysisResult, OrchestratorError> {
        #[cfg(not(target_arch = "wasm32"))]
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_secs(5);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🔬 Analyzing";

        let external_session =
            self.plugin_registry.create_external_analysis_session(self.database.files()).map(Arc::new);
        let lifecycle_capabilities = Arc::new(OnceLock::new());
        let additional_symbol_references = Arc::new(OnceLock::new());
        let reducer = AnalysisResultReducer {
            plugin_registry: Arc::clone(&self.plugin_registry),
            external_session: external_session.clone(),
            files: if external_session.is_some() {
                self.database.files().collect::<Vec<_>>().into()
            } else {
                Arc::from([])
            },
        };

        let pipeline = ParallelPipeline::new(
            ANALYSIS_PROGRESS_PREFIX,
            self.database,
            self.codebase,
            self.symbol_references,
            (self.settings.clone(), self.parser_settings),
            self.parser_settings,
            self.settings.version,
            Box::new(reducer),
            self.use_progress_bars,
        );

        let plugin_registry = Arc::clone(&self.plugin_registry);
        let before_plugin_registry = Arc::clone(&self.plugin_registry);
        let before_external_session = external_session.clone();
        let before_capabilities = Arc::clone(&lifecycle_capabilities);
        let map_capabilities = Arc::clone(&lifecycle_capabilities);
        let before_additional_symbol_references = Arc::clone(&additional_symbol_references);
        let map_additional_symbol_references = Arc::clone(&additional_symbol_references);

        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(not(target_arch = "wasm32"))]
        let telemetry = Arc::new(AnalysisPhaseTelemetry::default());
        #[cfg(not(target_arch = "wasm32"))]
        let telemetry_for_closure = Arc::clone(&telemetry);

        let result = pipeline.run(
            move |codebase, symbol_references| {
                before_plugin_registry.prepare_external_analyzer().map_err(AnalysisError::from)?;
                let capabilities = (
                    before_plugin_registry.has_external_after_file_analysis_hooks().map_err(AnalysisError::from)?,
                    before_plugin_registry.has_external_after_analysis_hooks().map_err(AnalysisError::from)?,
                    before_plugin_registry.external_node_analysis_requirements().map_err(AnalysisError::from)?,
                );

                let _result = before_capabilities.set(capabilities);
                #[cfg(not(target_arch = "wasm32"))]
                let lifecycle_start = trace_enabled.then(Instant::now);
                let before = before_plugin_registry
                    .run_external_before_analysis_hooks(codebase, before_external_session.as_deref())
                    .map_err(AnalysisError::from)?;
                if !before.references.is_empty() {
                    symbol_references.extend(before.references.clone());
                    let _result = before_additional_symbol_references.set(Arc::new(before.references));
                }
                let issues = before.issues;

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = lifecycle_start {
                    tracing::trace!(
                        issues = issues.len(),
                        elapsed = ?start.elapsed(),
                        "External before-analysis hooks completed."
                    );
                }

                if issues.is_empty() {
                    Ok(None)
                } else {
                    let mut result = AnalysisResult::new(SymbolReferences::new());
                    result.issues = issues;
                    Ok(Some(AnalysisTaskResult { result, snapshot: None }))
                }
            },
            move |(settings, parser_settings), arena, source_file, codebase| {
                let (after_file, after_analysis, node_analysis_requirements) =
                    map_capabilities.get().cloned().unwrap_or_default();

                #[cfg(not(target_arch = "wasm32"))]
                let per_file_start = trace_enabled.then(Instant::now);
                let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

                #[cfg(not(target_arch = "wasm32"))]
                let parse_start = trace_enabled.then(Instant::now);
                let program = parse_file_with_settings(arena, &source_file, parser_settings);
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = parse_start {
                    telemetry_for_closure.parse_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                }

                #[cfg(not(target_arch = "wasm32"))]
                let resolve_start = trace_enabled.then(Instant::now);
                let resolved_names = NameResolver::new(arena).resolve(program);
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = resolve_start {
                    telemetry_for_closure.resolve_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                }

                if program.has_errors() {
                    analysis_result.issues.extend(program.errors.iter().map(Issue::from));
                }

                let semantics_checker = SemanticsChecker::new(settings.version);

                #[cfg(not(target_arch = "wasm32"))]
                let analyzer_new_start = trace_enabled.then(Instant::now);
                let mut analyzer =
                    Analyzer::new(arena, &source_file, &resolved_names, &codebase, &plugin_registry, settings);
                if after_file || after_analysis {
                    analyzer = analyzer.with_deferred_pragmas();
                }
                if let Some(session) = external_session.as_deref() {
                    analyzer = analyzer.with_external_analysis_session(session);
                }
                if let Some(references) = map_additional_symbol_references.get() {
                    analyzer = analyzer.with_additional_symbol_references(references);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = analyzer_new_start {
                    telemetry_for_closure.analyzer_new_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                }

                #[cfg(not(target_arch = "wasm32"))]
                let semantics_start = trace_enabled.then(Instant::now);
                analysis_result.issues.extend(semantics_checker.check(&source_file, program, &resolved_names));
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = semantics_start {
                    telemetry_for_closure.semantics_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                }

                #[cfg(not(target_arch = "wasm32"))]
                let analyze_start = trace_enabled.then(Instant::now);
                let artifacts = analyzer.analyze_with_artifacts(program, &mut analysis_result)?;
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = analyze_start {
                    telemetry_for_closure.analyze_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = per_file_start {
                    telemetry_for_closure.per_file_total_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                    telemetry_for_closure.files.fetch_add(1, Relaxed);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if analysis_result.time_in_analysis > ANALYSIS_DURATION_THRESHOLD {
                    tracing::warn!(
                        "Analysis of source file '{}' took longer than {}s: {}s",
                        mago_bytes::BytesDisplay(&source_file.name),
                        ANALYSIS_DURATION_THRESHOLD.as_secs_f32(),
                        analysis_result.time_in_analysis.as_secs_f32()
                    );
                }

                #[cfg(not(target_arch = "wasm32"))]
                let snapshot_start = (trace_enabled && (after_file || after_analysis)).then(Instant::now);
                let snapshot = if after_file || after_analysis {
                    Some(Arc::new(
                        FileAnalysisSnapshot::new(
                            &source_file,
                            program,
                            &resolved_names,
                            &artifacts,
                            &codebase,
                            node_analysis_requirements.as_ref(),
                        )
                        .map_err(|error| {
                            OrchestratorError::General(format!("Failed to retain external analysis data: {error}"))
                        })?,
                    ))
                } else {
                    None
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(start) = snapshot_start {
                    telemetry_for_closure.snapshot_ns.fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
                    telemetry_for_closure.snapshots.fetch_add(1, Relaxed);
                }

                Ok(AnalysisTaskResult { result: analysis_result, snapshot })
            },
        );

        #[cfg(not(target_arch = "wasm32"))]
        if trace_enabled {
            telemetry.dump();
            analyzer_telemetry::dump_and_reset();
        }

        result
    }
}

/// The "reduce" step for the analysis pipeline.
///
/// This struct aggregates the `AnalysisResult` from each parallel task into a single,
/// final `AnalysisResult` for the entire project.
#[derive(Debug)]
struct AnalysisTaskResult {
    result: AnalysisResult,
    snapshot: Option<Arc<FileAnalysisSnapshot>>,
}

#[derive(Debug, Clone)]
struct AnalysisResultReducer {
    plugin_registry: Arc<PluginRegistry>,
    external_session: Option<Arc<mago_analyzer::external::ExternalAnalysisSession>>,
    files: Arc<[Arc<File>]>,
}

impl Reducer<AnalysisTaskResult, AnalysisResult> for AnalysisResultReducer {
    fn reduce(
        &self,
        mut codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        results: Vec<AnalysisTaskResult>,
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut aggregated_result = AnalysisResult::new(symbol_references);
        let mut snapshots = Vec::new();
        for result in results {
            aggregated_result.extend(result.result);
            snapshots.extend(result.snapshot);
        }

        aggregated_result.issues.extend(codebase.take_issues(true));
        let after_file = self.plugin_registry.has_external_after_file_analysis_hooks().map_err(AnalysisError::from)?;
        if after_file {
            let started_at = tracing::enabled!(tracing::Level::TRACE).then(Instant::now);
            let batches = snapshots
                .par_chunks(AFTER_FILE_ANALYSIS_BATCH_SIZE)
                .map(|files| {
                    self.plugin_registry.run_external_after_file_analysis_batch_hooks(
                        files,
                        &codebase,
                        self.external_session.as_deref(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(AnalysisError::from)?;
            let issue_count = batches.iter().map(|batch| batch.issues.len()).sum::<usize>();
            let has_late_references = batches.iter().any(|batch| !batch.references_by_file.is_empty());
            for batch in batches {
                aggregated_result.issues.extend(batch.issues);
                for references in batch.references_by_file.into_values() {
                    aggregated_result.symbol_references.extend(references);
                }
            }

            if has_late_references {
                aggregated_result.issues =
                    LateSymbolReferenceIssueReconciler::new(&codebase, &aggregated_result.symbol_references)
                        .reconcile(std::mem::take(&mut aggregated_result.issues));
            }

            if let Some(start) = started_at {
                tracing::trace!(
                    files = snapshots.len(),
                    batches = snapshots.len().div_ceil(AFTER_FILE_ANALYSIS_BATCH_SIZE),
                    issues = issue_count,
                    elapsed = ?start.elapsed(),
                    "External after-file hook batches completed."
                );
            }
        }

        let mut pragma_reconciler =
            DeferredIssueReconciler::new(aggregated_result.take_deferred_pragmas(), self.files.iter().cloned());
        aggregated_result.issues = pragma_reconciler.reconcile(std::mem::take(&mut aggregated_result.issues))?;
        let after_issues = self
            .plugin_registry
            .run_external_after_analysis_hooks(
                &aggregated_result,
                &snapshots,
                &codebase,
                self.external_session.as_deref(),
            )
            .map_err(AnalysisError::from)?;
        aggregated_result.issues.extend(pragma_reconciler.reconcile(after_issues)?);
        aggregated_result.issues.extend(pragma_reconciler.finish()?);

        Ok(aggregated_result)
    }
}
