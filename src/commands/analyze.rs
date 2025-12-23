//! Static analysis command implementation.
//!
//! This module implements the `mago analyze` command, which performs comprehensive
//! static type analysis on PHP codebases to identify type errors, unused code,
//! null safety violations, and other logical issues.
//!
//! # Analysis Process
//!
//! The analyzer follows a multi-phase approach:
//!
//! 1. **Prelude Loading**: Load embedded stubs for PHP built-ins and popular libraries
//! 2. **Database Loading**: Scan and load source files from the workspace
//! 3. **Codebase Model Building**: Construct a complete symbol table and type graph
//! 4. **Analysis**: Perform type checking, control flow analysis, and issue detection
//! 5. **Filtering**: Apply ignore rules and baseline comparisons
//! 6. **Reporting**: Output issues in the configured format
//!
//! # Type Analysis
//!
//! The analyzer performs deep type analysis including:
//!
//! - Type inference and propagation
//! - Type mismatch detection
//! - Null safety checking
//! - Return type validation
//! - Parameter type checking
//! - Property access validation
//!
//! # Stub Support
//!
//! The analyzer includes embedded stubs (`prelude`) containing type information
//! for PHP built-in functions and popular libraries. This enables accurate type
//! checking even for external symbols. Stubs can be disabled with `--no-stubs`
//! for debugging or testing purposes.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::ColorChoice;
use clap::Parser;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_database::watcher::DatabaseWatcher;
use mago_database::watcher::WatchOptions;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::incremental::IncrementalAnalysis;
use mago_prelude::Prelude;

use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::config::Configuration;
use crate::consts::ISSUE_URL;
use crate::consts::PRELUDE_BYTES;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Command for performing static type analysis on PHP code.
///
/// This command runs comprehensive static analysis to detect type errors,
/// unused code, unreachable code paths, and other logical issues that can
/// be found without executing the code.
///
/// # Analysis Features
///
/// The analyzer provides:
///
/// - **Type Checking**: Validates type compatibility across assignments, calls, and returns
/// - **Unused Detection**: Finds unused variables, functions, classes, and expressions
/// - **Dead Code Analysis**: Identifies unreachable code paths
/// - **Null Safety**: Detects potential null pointer dereferences
/// - **Exception Tracking**: Validates thrown exceptions are handled or declared
/// - **Type Inference**: Infers types where not explicitly annotated
///
/// # Stubs and Context
///
/// By default, the analyzer loads embedded stubs for PHP built-ins and popular
/// libraries, providing accurate type information for external symbols. This can
/// be disabled with `--no-stubs` for testing or debugging.
#[derive(Parser, Debug)]
#[command(
    name = "analyze",
    // Alias for the British
    alias = "analyse",
)]
pub struct AnalyzeCommand {
    /// Specific files or directories to analyze instead of using configuration.
    ///
    /// When provided, these paths override the source configuration in mago.toml.
    /// The analyzer will focus only on the specified files or directories.
    ///
    /// This is useful for targeted analysis, testing changes, or integrating
    /// with development workflows and CI systems.
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Disable built-in PHP and library stubs for analysis.
    ///
    /// By default, the analyzer uses stubs for built-in PHP functions and popular
    /// libraries to provide accurate type information. Disabling this may result
    /// in more reported issues when external symbols can't be resolved.
    #[arg(long, default_value_t = false)]
    pub no_stubs: bool,

    /// Enable watch mode for continuous analysis (experimental).
    ///
    /// When enabled, the analyzer watches the workspace for file changes and
    /// automatically re-runs analysis whenever PHP files are modified,
    /// created, or deleted. This provides instant feedback during development.
    ///
    /// Press Ctrl+C to stop watching.
    #[arg(long, default_value_t = false)]
    pub watch: bool,

    /// Arguments related to reporting issues with baseline support.
    #[clap(flatten)]
    pub baseline_reporting: BaselineReportingArgs,
}

impl AnalyzeCommand {
    /// Executes the static analysis process.
    ///
    /// This method orchestrates the complete analysis workflow:
    ///
    /// 1. **Load Prelude**: Decode embedded stubs for PHP built-ins (unless `--no-stubs`)
    /// 2. **Create Orchestrator**: Initialize with configuration and color settings
    /// 3. **Apply Overrides**: Use `path` argument if provided to override config paths
    /// 4. **Load Database**: Scan workspace and include external files for context
    /// 5. **Validate Files**: Ensure at least one host file exists to analyze
    /// 6. **Create Service**: Initialize analysis service with database and prelude
    /// 7. **Run Analysis**: Perform type checking and issue detection
    /// 8. **Filter Issues**: Apply ignore rules from configuration
    /// 9. **Process Results**: Report issues through baseline processor
    ///
    /// # Arguments
    ///
    /// * `configuration` - The loaded configuration containing analyzer settings
    /// * `color_choice` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` if analysis completed successfully
    /// - `Err(Error)` if database loading, analysis, or reporting failed
    ///
    /// # File Types
    ///
    /// The analyzer distinguishes between:
    /// - **Host files**: Source files to analyze (from configured paths)
    /// - **External files**: Context files (from includes) that provide type information
    ///
    /// Only host files are analyzed for issues; external files only contribute to
    /// the symbol table and type graph.
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        // 1. Establish the base prelude data.
        let Prelude { database, metadata, symbol_references } = if self.no_stubs {
            Prelude::default()
        } else {
            Prelude::decode(PRELUDE_BYTES).expect("Failed to decode embedded prelude")
        };

        let mut orchestrator = create_orchestrator(&configuration, color_choice, false, !self.watch, self.watch);
        orchestrator.add_exclude_patterns(configuration.analyzer.excludes.iter());

        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter().map(|p| p.to_string_lossy().to_string()));
        }

        // Check if watch mode is enabled early, since it needs ownership of configuration
        if self.watch {
            return self.run_watch_mode(
                orchestrator,
                &configuration,
                color_choice,
                database,
                metadata,
                symbol_references,
            );
        }

        let mut database = orchestrator.load_database(&configuration.source.workspace, true, Some(database))?;

        if !database.files().any(|f| f.file_type == FileType::Host) {
            tracing::warn!("No files found to analyze.");

            return Ok(ExitCode::SUCCESS);
        }

        let mut service = orchestrator.get_analysis_service(
            database.read_only(),
            metadata,
            symbol_references,
            self.create_incremental_analysis(),
        );

        let analysis_result = service.run()?;

        let mut issues = analysis_result.issues;
        issues.filter_out_ignored(&configuration.analyzer.ignore);

        let baseline = configuration.analyzer.baseline.as_deref();
        let baseline_variant = configuration.analyzer.baseline_variant;
        let processor = self.baseline_reporting.get_processor(color_choice, baseline, baseline_variant);

        processor.process_issues(&orchestrator, &mut database, issues)
    }

    /// Creates incremental analysis manager based on flags.
    ///
    /// Returns `Some(IncrementalAnalysis)` if incremental analysis should be used,
    /// `None` otherwise.
    fn create_incremental_analysis(&self) -> Option<IncrementalAnalysis> {
        if !self.watch {
            return None;
        }

        Some(IncrementalAnalysis::new())
    }

    /// Runs in watch mode, continuously monitoring for file changes and re-analyzing.
    ///
    /// This method sets up a file watcher on the workspace directory and runs
    /// full analysis whenever PHP files are modified, created, or deleted.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The loaded configuration
    /// * `color_choice` - Whether to use colored output
    /// * `prelude_database` - The prelude database with builtin symbols
    /// * `metadata` - The codebase metadata (from prelude)
    /// * `symbol_references` - Symbol references (from prelude)
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` when user interrupts with Ctrl+C
    /// - `Err(e)` if watcher setup or analysis fails
    fn run_watch_mode(
        &self,
        orchestrator: Orchestrator<'_>,
        configuration: &Configuration,
        color_choice: ColorChoice,
        prelude_database: Database<'static>,
        metadata: CodebaseMetadata,
        symbol_references: SymbolReferences,
    ) -> Result<ExitCode, Error> {
        tracing::warn!("Watch mode is an experimental feature and may be unstable.");
        tracing::warn!("If you encounter issues, please report them at {}", ISSUE_URL);

        tracing::info!("Starting watch mode. Press Ctrl+C to stop.");

        let database = orchestrator.load_database(&configuration.source.workspace, true, Some(prelude_database))?;

        let mut watcher = DatabaseWatcher::new(database);

        watcher.watch(WatchOptions { poll_interval: Some(Duration::from_millis(500)), ..Default::default() })?;

        tracing::info!("Watching {} for changes...", configuration.source.workspace.display());
        tracing::info!("Running initial analysis...");
        let mut service = orchestrator.get_analysis_service(
            watcher.read_only_database(),
            metadata,
            symbol_references,
            self.create_incremental_analysis(),
        );

        let analysis_result = service.run()?;

        let mut issues = analysis_result.issues;
        issues.filter_out_ignored(&configuration.analyzer.ignore);
        let baseline = configuration.analyzer.baseline.as_deref();
        let baseline_variant = configuration.analyzer.baseline_variant;

        let processor = self.baseline_reporting.get_processor(color_choice, baseline, baseline_variant);

        watcher.with_database_mut(|database| processor.process_issues(&orchestrator, database, issues))?;

        tracing::info!("Initial analysis complete. Watching for changes...");

        loop {
            let changed_file_ids = watcher.wait()?;
            if changed_file_ids.is_empty() {
                continue;
            }

            tracing::info!("Detected {} file change(s), re-analyzing...", changed_file_ids.len());

            service.update_database(watcher.database().read_only());

            let analysis_result = service.run_incremental()?;

            let mut issues = analysis_result.issues;
            issues.filter_out_ignored(&configuration.analyzer.ignore);

            watcher.with_database_mut(|database| processor.process_issues(&orchestrator, database, issues))?;

            tracing::info!("Analysis complete. Watching for changes...");
        }
    }
}
