use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_prelude::Prelude;

use crate::commands::args::baseline::BaselineArgs;
use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
use crate::database;
use crate::error::Error;
use crate::pipeline::guard::run_guard_pipeline;

/// Check architectural boundaries using guard rules.
///
/// The `guard` command performs architectural boundary checking on your PHP codebase.
/// It analyzes symbol dependencies and ensures they comply with the architectural rules
/// defined in your configuration.
///
/// Guard helps enforce:
///
/// • Layer boundaries between different parts of your application
/// • Dependency direction rules (e.g., domain should not depend on infrastructure)
/// • Allowed symbol types for specific dependencies
/// • Namespace isolation and architectural constraints
///
/// You can define rules in your `mago.toml` file to specify which namespaces can
/// depend on others and what types of symbols are allowed.
#[derive(Parser, Debug)]
#[command(name = "guard")]
pub struct GuardCommand {
    /// Specific files or directories to check instead of using configuration.
    ///
    /// When provided, these paths override the source configuration in mago.toml.
    /// The guard will focus only on the specified files or directories.
    ///
    /// This is useful for targeted checking, testing changes, or integrating
    /// with development workflows and CI systems.
    #[arg()]
    pub paths: Vec<PathBuf>,

    /// Disable built-in PHP and library stubs for checking.
    ///
    /// By default, guard uses stubs for built-in PHP functions and popular
    /// libraries to provide accurate symbol information. Disabling this may result
    /// in more warnings when external symbols can't be resolved.
    #[arg(long, default_value_t = false)]
    pub no_stubs: bool,

    /// Arguments related to reporting issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,

    /// Arguments related to baseline functionality.
    #[clap(flatten)]
    pub baseline: BaselineArgs,
}

impl GuardCommand {
    /// Executes the guard command.
    ///
    /// This function orchestrates the process of:
    ///
    /// 1. Loading source files.
    /// 2. Compiling a codebase model from these files (with progress).
    /// 3. Checking architectural boundaries against guard rules (with progress).
    /// 4. Reporting any found violations.
    pub fn execute(self, mut configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        if self.reporting.fix {
            tracing::warn!("Fix mode is not yet implemented for guard. Running in check-only mode.");
        }

        configuration.source.excludes.extend(std::mem::take(&mut configuration.guard.excludes));

        let (base_db, codebase_metadata, _) = {
            let prelude = Prelude::decode(PRELUDE_BYTES).expect("Failed to decode embedded prelude");

            (prelude.database, prelude.metadata, prelude.symbol_references)
        };

        let final_database = if !self.paths.is_empty() {
            database::load_from_paths(&mut configuration.source, self.paths, Some(base_db))?
        } else {
            database::load_from_configuration(&mut configuration.source, false, Some(base_db))?
        };

        if !final_database.files().any(|f| f.file_type == FileType::Host) {
            tracing::warn!("No files found to check with guard.");

            return Ok(ExitCode::SUCCESS);
        }

        let guard_settings = configuration.guard.settings.clone();
        let issues = run_guard_pipeline(final_database.read_only(), codebase_metadata, guard_settings)?;

        let config_baseline = configuration.guard.baseline.clone();
        let read_database = final_database.read_only();

        let (filtered_issues, should_fail_from_baseline, early_exit) =
            self.baseline.process_baseline(issues, config_baseline.as_deref(), &read_database)?;

        if let Some(exit_code) = early_exit {
            return Ok(exit_code);
        }

        self.reporting.process_issues_with_baseline_result(
            filtered_issues,
            configuration,
            color_choice,
            final_database,
            should_fail_from_baseline,
        )
    }
}
