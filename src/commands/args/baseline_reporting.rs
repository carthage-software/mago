//! Command-line arguments for baseline-enabled issue reporting.
//!
//! This module provides command-line arguments for managing baseline files
//! in combination with issue reporting. Baselines allow teams to establish
//! a snapshot of existing issues and focus on preventing new issues from
//! being introduced.
//!
//! The baseline functionality supports generating new baselines from current
//! issues, verifying that baselines are up-to-date, and filtering reported
//! issues against a baseline. It also includes options for backup and strict
//! synchronization checking.

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

use clap::ColorChoice;
use clap::Parser;

use mago_database::Database;
use mago_orchestrator::Orchestrator;

use crate::commands::args::reporting::ReportingArgs;
use crate::service::BaselineIssueProcessor;

/// Command-line arguments for baseline functionality combined with reporting.
///
/// This struct is designed to be flattened into other clap commands
/// that require baseline functionality for filtering issues.
#[derive(Parser, Debug, Clone)]
pub struct BaselineReportingArgs {
    /// Specify a baseline file to ignore existing issues.
    ///
    /// A baseline file contains a list of known issues that should be ignored
    /// in future runs. This is useful for gradually improving code quality by
    /// focusing on new issues while suppressing existing ones.
    /// Can be overridden by configuration in mago.toml.
    #[arg(long, value_name = "PATH")]
    pub baseline: Option<PathBuf>,

    /// Generate a new baseline file from current issues.
    ///
    /// This creates a baseline file containing all issues found in the current run.
    /// Use this to establish a starting point for future issue tracking.
    /// Requires --baseline to specify where to save the file.
    #[arg(long)]
    pub generate_baseline: bool,

    /// Create a backup of the existing baseline file before generating a new one.
    ///
    /// When generating a new baseline, the old file will be saved with a .bkp extension.
    /// This provides a safety net in case you need to revert the baseline.
    /// Requires --generate-baseline to be enabled.
    #[arg(long, requires = "generate_baseline")]
    pub backup_baseline: bool,

    /// Check if the baseline file is synchronized with current issues.
    ///
    /// This compares the baseline against current issues to detect if the baseline
    /// is outdated. Exits with failure if issues have changed since the baseline
    /// was created. Cannot be used with --generate-baseline.
    #[arg(long, conflicts_with = "generate_baseline")]
    pub verify_baseline: bool,

    /// Fail the command when baseline is out of sync, even with no new issues.
    ///
    /// Normally, if there are no current issues to report, the command succeeds
    /// even if the baseline is outdated. This flag forces failure when the baseline
    /// contains issues that no longer exist, ensuring baselines stay clean.
    #[arg(long, conflicts_with = "generate_baseline", conflicts_with = "verify_baseline")]
    pub fail_on_out_of_sync_baseline: bool,

    /// Arguments related to reporting and fixing issues.
    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

impl BaselineReportingArgs {
    /// Creates a baseline-aware issue processor from these arguments.
    ///
    /// This method converts the command-line arguments into a [`BaselineIssueProcessor`]
    /// that will filter issues against a baseline file and handle baseline generation
    /// and verification according to the configured options.
    ///
    /// # Arguments
    ///
    /// * `orchestrator` - The orchestrator for formatting fixed files
    /// * `database` - The database containing source files
    /// * `color_choice` - Whether to use colored output
    /// * `baseline` - Optional baseline path from configuration (overridden by CLI arg)
    ///
    /// # Returns
    ///
    /// A [`BaselineIssueProcessor`] configured with all the baseline and reporting
    /// options from this argument set.
    pub fn get_processor<'a>(
        self,
        orchestrator: Orchestrator<'a>,
        database: Database,
        color_choice: ColorChoice,
        baseline: Option<&'a Path>,
    ) -> BaselineIssueProcessor<'a> {
        BaselineIssueProcessor {
            read_database: database.read_only(),
            baseline_path: match self.baseline {
                Some(path) => Some(Cow::Owned(path)),
                None => baseline.map(Cow::Borrowed),
            },
            generate_baseline: self.generate_baseline,
            backup_baseline: self.backup_baseline,
            verify_baseline: self.verify_baseline,
            fail_on_out_of_sync_baseline: self.fail_on_out_of_sync_baseline,
            issue_processor: self.reporting.get_processor(orchestrator, database, color_choice),
        }
    }
}
