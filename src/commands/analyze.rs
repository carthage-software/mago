use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_database::DatabaseReader;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::database;
use crate::error::Error;
use crate::pipeline::analysis::run_analysis_pipeline;

/// Command to perform static type analysis on PHP source code.
///
/// This command identifies potential type errors, unused code, and other
/// type-related issues within the specified PHP project or files.
#[derive(Parser, Debug)]
#[command(
    name = "analyze",
    // Alias for the British
    alias = "analyse",
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

    /// Arguments related to reporting and fixing issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

/// Executes the analyze command.
///
/// This function orchestrates the process of:
///
/// 1. Loading source files.
/// 2. Compiling a codebase model from these files (with progress).
/// 3. Analyzing the user-defined sources against the compiled codebase (with progress).
/// 4. Reporting any found issues.
pub fn execute(command: AnalyzeCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    let database = if !command.path.is_empty() {
        database::from_paths(&configuration.source, command.path, !command.no_stubs)?
    } else {
        database::load(&configuration.source, true, !command.no_stubs)?
    };

    if database.is_empty() {
        tracing::info!("No files found to analyze.");

        return Ok(ExitCode::SUCCESS);
    }

    let analyzer_settings = configuration.analyzer.to_setttings(configuration.php_version);
    let analysis_results = run_analysis_pipeline(database.read_only(), analyzer_settings)?;

    let mut issues = analysis_results.issues;
    issues.filter_out_ignored(&configuration.analyzer.ignore);

    command.reporting.process_issues(issues, configuration, database)
}
