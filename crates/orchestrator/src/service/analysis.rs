use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use bumpalo::Bump;
use foldhash::HashSet;

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
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::error::OrchestratorError;
use crate::service::pipeline::ParallelPipeline;
use crate::service::pipeline::Reducer;

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
        let Ok(file) = self.database.get_ref(&file_id) else {
            tracing::error!("File with ID {:?} not found in database", file_id);

            return IssueCollection::default();
        };

        let arena = Bump::new();

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

    /// Runs the full analysis pipeline.
    ///
    /// This method scans all source files, builds the codebase, and runs the analyzer.
    pub fn run(self) -> Result<AnalysisResult, OrchestratorError> {
        #[cfg(not(target_arch = "wasm32"))]
        const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);
        const ANALYSIS_PROGRESS_PREFIX: &str = "🔬 Analyzing";

        let pipeline = ParallelPipeline::new(
            ANALYSIS_PROGRESS_PREFIX,
            self.database,
            self.codebase,
            self.symbol_references,
            (self.settings.clone(), self.parser_settings),
            self.parser_settings,
            Box::new(AnalysisResultReducer),
            self.use_progress_bars,
        );

        let plugin_registry = Arc::clone(&self.plugin_registry);

        pipeline.run(move |(settings, parser_settings), arena, source_file, codebase| {
            let mut analysis_result = AnalysisResult::new(SymbolReferences::new());

            let program = parse_file_with_settings(arena, &source_file, parser_settings);
            let resolved_names = NameResolver::new(arena).resolve(program);

            if program.has_errors() {
                analysis_result.issues.extend(program.errors.iter().map(Issue::from));
            }

            let semantics_checker = SemanticsChecker::new(settings.version);
            let analyzer = Analyzer::new(arena, &source_file, &resolved_names, &codebase, &plugin_registry, settings);

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
        })
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
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut aggregated_result = AnalysisResult::new(symbol_references);
        for result in results {
            aggregated_result.extend(result);
        }

        aggregated_result.issues.extend(codebase.take_issues(true));

        Ok(aggregated_result)
    }
}
