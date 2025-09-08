use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_prelude::Prelude;

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
    pub paths: Vec<PathBuf>,

    /// Disable the use of stubs (e.g., for built-in PHP functions or popular libraries).
    /// Disabling stubs might lead to more reported issues if type information for external symbols is missing.
    #[arg(long, help = "Disable stubs, potentially leading to more issues", default_value_t = false)]
    pub no_stubs: bool,

    /// Arguments related to reporting and fixing issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,
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
            configuration.analyzer.to_settings(configuration.php_version),
        )?;

        // 4. Filter and report any found issues.
        let mut issues = analysis_results.issues;
        issues.filter_out_ignored(&configuration.analyzer.ignore);

        self.reporting.process_issues(issues, configuration, should_use_colors, final_database)
    }
}
