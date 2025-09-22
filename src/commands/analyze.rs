use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_prelude::Prelude;

use crate::commands::args::baseline::BaselineArgs;
use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::database;
use crate::error::Error;
use crate::pipeline::analysis::run_analysis_pipeline;

/// Perform static type analysis to find type errors and code issues.
///
/// The `analyze` command performs comprehensive static type analysis on your PHP codebase.
/// It builds a complete model of your code's symbols and types, then identifies potential
/// issues including:
///
/// • Type mismatches and errors
/// • Unused variables, functions, and classes
/// • Unreachable or dead code
/// • Missing or incorrect type annotations
/// • Null safety violations
///
/// The analysis includes built-in PHP functions and popular library stubs for accurate
/// type checking. You can configure analysis depth and specific issue categories in your
/// `mago.toml` file.
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
    pub paths: Vec<PathBuf>,

    /// Disable built-in PHP and library stubs for analysis.
    ///
    /// By default, the analyzer uses stubs for built-in PHP functions and popular
    /// libraries to provide accurate type information. Disabling this may result
    /// in more reported issues when external symbols can't be resolved.
    #[arg(long, default_value_t = false)]
    pub no_stubs: bool,

    /// Arguments related to reporting and fixing issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,

    /// Arguments related to baseline functionality.
    #[clap(flatten)]
    pub baseline: BaselineArgs,
}

impl AnalyzeCommand {
    /// Executes the analyze command.
    ///
    /// This function orchestrates the process of:
    ///
    /// 1. Loading source files.
    /// 2. Compiling a codebase model from these files (with progress).
    /// 3. Analyzing the user-defined sources against the compiled codebase (with progress).
    /// 4. Reporting any found issues.
    pub fn execute(self, mut configuration: Configuration, should_use_colors: bool) -> Result<ExitCode, Error> {
        configuration.source.excludes.extend(std::mem::take(&mut configuration.analyzer.excludes));

        // 1. Establish the base prelude data. We deconstruct the prelude to get the
        //    database and the already-analyzed metadata separately.
        let (base_db, codebase_metadata, symbol_references) = if self.no_stubs {
            (Default::default(), Default::default(), Default::default())
        } else {
            const PRELUDE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/prelude.bin"));

            let prelude = Prelude::decode(PRELUDE_BYTES).expect("Failed to decode embedded prelude");

            (prelude.database, prelude.metadata, prelude.symbol_references)
        };

        // 2. Load the user's codebase, passing the `base_db` to be extended.
        let final_database = if !self.paths.is_empty() {
            database::load_from_paths(&mut configuration.source, self.paths, Some(base_db))?
        } else {
            database::load_from_configuration(
                &mut configuration.source,
                /* include externals */ true,
                Some(base_db),
            )?
        };

        // Check if any user-specified files were actually added to the database.
        if !final_database.files().any(|f| f.file_type == FileType::Host) {
            tracing::warn!("No files found to analyze.");

            return Ok(ExitCode::SUCCESS);
        }

        // 3. Run the analysis pipeline with the combined database and the prelude's metadata.
        let analysis_results = run_analysis_pipeline(
            final_database.read_only(),
            codebase_metadata,
            symbol_references,
            configuration.analyzer.to_settings(configuration.php_version, should_use_colors),
        )?;

        // 4. Filter and report any found issues.
        let mut issues = analysis_results.issues;
        issues.filter_out_ignored(&configuration.analyzer.ignore);

        let config_baseline = configuration.analyzer.baseline.clone();
        let read_database = final_database.read_only();

        // Process baseline first
        let (filtered_issues, should_fail_from_baseline, early_exit) =
            self.baseline.process_baseline(issues, config_baseline.as_deref(), &read_database)?;

        // Handle early exits (baseline generation/verification)
        if let Some(exit_code) = early_exit {
            return Ok(exit_code);
        }

        // Process issues with reporting
        self.reporting.process_issues_with_baseline_result(
            filtered_issues,
            configuration,
            should_use_colors,
            final_database,
            should_fail_from_baseline,
        )
    }
}
