use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use tokio::task::JoinHandle;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_interner::ThreadedInterner;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_source::Source;
use mago_source::SourceCategory;
use mago_source::SourceManager;
use mago_syntax::parser::parse_source;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::error::Error;
use crate::metadata::compile_codebase_for_sources;
use crate::source;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

/// Command to perform static type analysis on PHP source code.
///
/// This command identifies potential type errors, unused code, and other
/// type-related issues within the specified PHP project or files.
#[derive(Parser, Debug)]
#[command(
    name = "analyze",
    about = "Find typing issues in the project source code using configurable type checker settings.",
    long_about = "The `analyze` command is a fast type checker for PHP. It scans your codebase, \
                  builds a model of its symbols and types, and then analyzes it to find \
                  potential type errors, unused code, and other configurable checks."
)]
pub struct AnalyzeCommand {
    /// Specific files or directories to analyze.
    /// If provided, this overrides the source configuration from `mago.toml`.
    #[arg(help = "Analyze specific files or directories, overriding source configuration")]
    pub path: Vec<PathBuf>,

    /// Disable the use of stubs (e.g., for built-in PHP functions or popular libraries).
    /// Disabling stubs might lead to more reported issues if type information for external symbols is missing.
    #[arg(long, help = "Disable stubs, potentially leading to more issues", default_value_t = false)]
    pub no_stubs: bool,

    /// Enable or disable finding unused expressions.
    /// Overrides the corresponding setting in `mago.toml` if provided.
    #[arg(long, help = "Find unused expressions (e.g., function calls whose results are not used)")]
    pub find_unused_expressions: Option<bool>,

    /// Enable or disable finding unused definitions (e.g., unreferenced functions, classes).
    /// Overrides the corresponding setting in `mago.toml` if provided.
    #[arg(long, help = "Find unused definitions (functions, classes, constants, etc.)")]
    pub find_unused_definitions: Option<bool>,

    /// Enable or disable analysis of code known to be unreachable (dead code).
    /// Overrides the corresponding setting in `mago.toml` if provided.
    #[arg(long, help = "Analyze dead code for errors (code known to be unreachable)")]
    pub analyze_dead_code: Option<bool>,

    /// Enable or disable the analysis of effects (e.g., side effects of function calls).
    /// Overrides the corresponding setting in `mago.toml` if provided.
    #[arg(long, help = "Analyze effects (e.g., side effects of function calls)")]
    pub analyze_effects: Option<bool>,

    /// Enable or disable memoization of properties.
    #[arg(long, help = "Enable memoization for property assignments and accesses")]
    pub memoize_properties: Option<bool>,

    /// Enable or disable the use of `include` statements in the analysis.
    #[arg(long, help = "Allow the use of `include` statements in the analysis")]
    pub allow_include: Option<bool>,

    /// Arguments related to reporting and fixing issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,

    /// Run the analysis sequentially instead of concurrently.
    #[arg(long, help = "Run the analysis sequentially instead of concurrently, which can be useful for debugging.")]
    pub sequential: bool,
}

/// Executes the analyze command.
///
/// This function orchestrates the process of:
///
/// 1. Loading source files.
/// 2. Compiling a codebase model from these files (with progress).
/// 3. Analyzing the user-defined sources against the compiled codebase (with progress).
/// 4. Reporting any found issues.
pub async fn execute(command: AnalyzeCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();

    let source_manager = if !command.path.is_empty() {
        source::from_paths(&interner, &configuration.source, command.path, !command.no_stubs).await?
    } else {
        source::load(&interner, &configuration.source, true, !command.no_stubs).await?
    };

    if source_manager.is_empty() {
        tracing::info!("No source files found to analyze.");

        return Ok(ExitCode::SUCCESS);
    }

    let mut symbol_references = SymbolReferences::new();
    let analyzer_settings = AnalyzerSettings {
        version: configuration.php_version,
        analyze_dead_code: command.analyze_dead_code.unwrap_or(configuration.analyze.analyze_dead_code),
        find_unused_definitions: command
            .find_unused_definitions
            .unwrap_or(configuration.analyze.find_unused_definitions),
        find_unused_expressions: command
            .find_unused_expressions
            .unwrap_or(configuration.analyze.find_unused_expressions),
        analyze_effects: command.analyze_effects.unwrap_or(configuration.analyze.analyze_effects),
        memoize_properties: command.memoize_properties.unwrap_or(configuration.analyze.memoize_properties),
        allow_include: command.allow_include.unwrap_or(configuration.analyze.allow_include),
        ..Default::default()
    };

    tracing::debug!("Compiling codebase...");
    let mut codebase = compile_codebase_for_sources(&source_manager, &mut symbol_references, &interner).await?;

    tracing::debug!("Analyzing sources...");
    let analysis_result = analyze_user_sources(
        &source_manager,
        analyzer_settings,
        symbol_references,
        &codebase,
        &interner,
        command.sequential,
    )
    .await?;

    let mut issues = codebase.take_issues(true);
    issues.extend(analysis_result.emitted_issues.into_values().flatten());

    command.reporting.process_issues(issues, configuration, interner, source_manager).await
}

/// Analyzes user-defined source files concurrently with a progress bar.
///
/// Iterates over sources categorized as `UserDefined`, spawns a Tokio task for each,
/// updates the progress bar, and aggregates the results.
async fn analyze_user_sources(
    manager: &SourceManager,
    settings: AnalyzerSettings,
    symbol_references: SymbolReferences,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    sequential: bool,
) -> Result<AnalysisResult, Error> {
    let codebase = Arc::new(codebase.clone());

    let mut aggregated_analysis_result = AnalysisResult::new(settings.graph_kind, symbol_references);
    let source_ids = manager.source_ids_for_category(SourceCategory::UserDefined);

    if source_ids.is_empty() {
        tracing::debug!("No user-defined sources to analyze.");

        return Ok(aggregated_analysis_result);
    }

    let total_files = source_ids.len();
    let progress_bar = create_progress_bar(total_files, " Analyzing sources", ProgressBarTheme::Blue);

    let mut analysis_tasks: Vec<JoinHandle<Result<AnalysisResult, Error>>> = Vec::with_capacity(total_files);

    for source_id in source_ids {
        if sequential {
            let source_file = manager.load(&source_id)?;
            let result = perform_single_source_analysis(source_file, settings, &codebase, interner)?;
            aggregated_analysis_result.extend(result);
            progress_bar.inc(1);
            continue;
        }

        let interner_clone = interner.clone();
        let manager_clone = manager.clone();
        let codebase_clone = codebase.clone();
        let progress_bar_clone = progress_bar.clone();

        analysis_tasks.push(tokio::spawn(async move {
            let source_file = manager_clone.load(&source_id)?;
            let result = perform_single_source_analysis(source_file, settings, &codebase_clone, &interner_clone);
            progress_bar_clone.inc(1);
            result
        }));
    }

    if !sequential {
        for task_handle in analysis_tasks {
            match task_handle.await {
                Ok(Ok(source_analysis_result)) => {
                    aggregated_analysis_result.extend(source_analysis_result);
                }
                Ok(Err(e)) => {
                    tracing::error!("Error during analysis: {}", e);

                    if sequential {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => {
                    tracing::error!("Task failed: {}", e);

                    if sequential {
                        continue;
                    } else {
                        return Err(Error::from(e));
                    }
                }
            }
        }
    }

    remove_progress_bar(progress_bar);
    Ok(aggregated_analysis_result)
}

/// Performs static analysis on a single parsed PHP source file.
fn perform_single_source_analysis(
    source: Source,
    settings: AnalyzerSettings,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> Result<AnalysisResult, Error> {
    let (program, parsing_error) = parse_source(interner, &source);
    let mut analysis_result = AnalysisResult::new(settings.graph_kind, SymbolReferences::new());
    if let Some(parsing_error) = parsing_error {
        analysis_result.emitted_issues.entry(source.identifier).or_default().push(Issue::from(&parsing_error));
    }

    let resolver = NameResolver::new(interner);
    let resolved_names = resolver.resolve(&program);

    tracing::trace!("Analyzing source: {}", interner.lookup(&source.identifier.0));
    let mut analyzer = Analyzer::new(source, &resolved_names, codebase, interner, settings);
    analyzer.analyze(&program, &mut analysis_result)?;
    Ok(analysis_result)
}
