use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_prelude::Prelude;

use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
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

    /// Arguments related to reporting issues with baseline support.
    #[clap(flatten)]
    pub baseline_reporting: BaselineReportingArgs,
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
    pub fn execute(self, mut configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        configuration.source.excludes.extend(std::mem::take(&mut configuration.analyzer.excludes));

        // 1. Establish the base prelude data. We deconstruct the prelude to get the
        //    database and the already-analyzed metadata separately.
        let (base_db, codebase_metadata, symbol_references) = if self.no_stubs {
            (Default::default(), Default::default(), Default::default())
        } else {
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
            configuration.analyzer.to_settings(configuration.php_version, color_choice),
        )?;

        // 4. Filter and report any found issues.
        let mut issues = analysis_results.issues;
        issues.filter_out_ignored(&configuration.analyzer.ignore);

        let baseline = configuration.analyzer.baseline.clone();

        // 5. Process issues with baseline reporting if configured.
        self.baseline_reporting.process_issues_with_baseline(
            issues,
            configuration,
            color_choice,
            final_database,
            baseline,
        )
    }
}
