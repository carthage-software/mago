use std::time::Duration;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::settings::Settings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::ReadDatabase;
use mago_interner::ThreadedInterner;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

use crate::error::Error;
use crate::pipeline::ParallelPipeline;
use crate::pipeline::Reducer;

const ANALYSIS_DURATION_THRESHOLD: Duration = Duration::from_millis(5000);

/// The "reduce" step for the analysis pipeline.
///
/// This struct aggregates the `AnalysisResult` from each parallel task into a single,
/// final `AnalysisResult` for the entire project. It also collects any issues
/// that were generated during the final codebase population step.
#[derive(Debug, Clone)]
struct AnalysisResultReducer;

impl Reducer<AnalysisResult, AnalysisResult> for AnalysisResultReducer {
    fn reduce(
        &self,
        mut codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        results: Vec<AnalysisResult>,
    ) -> Result<AnalysisResult, Error> {
        let mut aggregated_result = AnalysisResult::new(symbol_references);
        for result in results {
            aggregated_result.extend(result);
        }

        aggregated_result.issues.extend(codebase.take_issues(true));

        Ok(aggregated_result)
    }
}

/// The main entry point for running the parallel static analysis pipeline.
///
/// This function orchestrates the two-phase "compile-then-analyze" workflow
/// using the [`StatefulParallelPipeline`]. It compiles a global codebase and
/// then runs a full semantic and type analysis on each host file.
///
/// # Arguments
///
/// * `interner`: The shared string interner.
/// * `database`: The read-only database containing the files to analyze.
/// * `analyzer_settings`: The configured settings for the analyzer.
///
/// # Returns
///
/// A `Result` containing the final, aggregated [`AnalysisResult`] for the
/// entire project, or an [`Error`].
pub fn run_analysis_pipeline(
    interner: &ThreadedInterner,
    database: ReadDatabase,
    analyzer_settings: Settings,
) -> Result<AnalysisResult, Error> {
    ParallelPipeline::new("🕵️ Analyzing", database, interner, analyzer_settings, Box::new(AnalysisResultReducer)).run(
        |settings, interner, source_file, codebase| {
            let (program, parsing_error) = parse_file(&interner, &source_file);
            let resolved_names = NameResolver::new(&interner).resolve(&program);

            let mut analysis_result = AnalysisResult::new(SymbolReferences::new());
            if let Some(parsing_error) = parsing_error {
                analysis_result.issues.push(Issue::from(&parsing_error));
            }

            let semantics_checker = SemanticsChecker::new(&settings.version, &interner);
            let analyzer = Analyzer::new(&source_file, &resolved_names, &codebase, &interner, settings);

            analysis_result.issues.extend(semantics_checker.check(&source_file, &program, &resolved_names));
            analyzer.analyze(&program, &mut analysis_result)?;

            if analysis_result.time_in_analysis > ANALYSIS_DURATION_THRESHOLD {
                tracing::warn!(
                    "Analysis of source file '{}' took longer than {}s: {}s",
                    source_file.name,
                    ANALYSIS_DURATION_THRESHOLD.as_secs_f32(),
                    analysis_result.time_in_analysis.as_secs_f32()
                );
            }

            Ok(analysis_result)
        },
    )
}
