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

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::Database;
use mago_database::ReadDatabase;
use mago_reporting::IssueCollection;

use crate::baseline;
use crate::baseline::Baseline;
use crate::baseline::unserialize_baseline;
use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::error::Error;

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
    /// Resolves the effective baseline path from arguments and configuration.
    ///
    /// Command-line arguments take precedence over configuration file settings.
    /// This allows users to override the default baseline path on a per-run basis.
    fn resolve_baseline_path(&self, config_baseline: Option<PathBuf>) -> Option<PathBuf> {
        self.baseline.clone().or(config_baseline)
    }

    /// Loads a baseline file from disk if it exists.
    ///
    /// This method attempts to read and deserialize the baseline file at the
    /// specified path. If the file doesn't exist or cannot be read, appropriate
    /// warnings or errors are logged and `None` is returned.
    fn get_baseline(&self, baseline_path: Option<&Path>) -> Option<Baseline> {
        let path = baseline_path?;
        if !path.exists() {
            tracing::warn!("Baseline file `{}` does not exist.", path.display());

            return None;
        }

        match unserialize_baseline(path) {
            Ok(baseline) => Some(baseline),
            Err(err) => {
                tracing::error!("Failed to read baseline file at `{}`: {}", path.display(), err);

                None
            }
        }
    }

    /// Creates a new baseline file from current issues.
    ///
    /// This method generates a baseline containing all provided issues and
    /// writes it to the specified path. If `--backup-baseline` is enabled,
    /// any existing baseline file is backed up before being overwritten.
    ///
    /// The baseline captures the issue code and location for each issue,
    /// providing a snapshot that can be used to filter these same issues
    /// in future runs.
    fn generate_baseline(
        &self,
        baseline_path: PathBuf,
        issues: IssueCollection,
        read_database: &ReadDatabase,
    ) -> Result<(), Error> {
        tracing::info!("Generating baseline file...");
        let baseline = Baseline::generate_from_issues(&issues, read_database);
        baseline::serialize_baseline(&baseline_path, &baseline, self.backup_baseline)?;
        tracing::info!("Baseline file successfully generated at `{}`.", baseline_path.display());

        Ok(())
    }

    /// Checks whether the baseline file is synchronized with current issues.
    ///
    /// This method compares the baseline against the current set of issues to
    /// determine if they match. It reports any new issues not in the baseline
    /// and any issues in the baseline that no longer exist.
    ///
    /// Returns `true` if the baseline is up-to-date, `false` if there are
    /// any differences. This is useful in CI/CD pipelines to ensure baselines
    /// don't become stale.
    fn verify_baseline(
        &self,
        baseline_path: PathBuf,
        issues: IssueCollection,
        read_database: &ReadDatabase,
    ) -> Result<bool, Error> {
        if !baseline_path.exists() {
            tracing::info!("Baseline file `{}` does not exist.", baseline_path.display());
            return Ok(false);
        }

        tracing::info!("Verifying baseline file at `{}`...", baseline_path.display());

        let baseline = unserialize_baseline(&baseline_path)?;
        let comparison = baseline.compare_with_issues(&issues, read_database);

        if comparison.is_up_to_date {
            tracing::info!("Baseline is up to date.");

            Ok(true)
        } else {
            if comparison.new_issues_count > 0 {
                tracing::warn!("Found {} new issues not in the baseline.", comparison.new_issues_count);
            }

            if comparison.removed_issues_count > 0 {
                tracing::warn!(
                    "Found {} issues in the baseline that no longer exist.",
                    comparison.removed_issues_count
                );
            }

            tracing::error!("Baseline is outdated. {} files have changes.", comparison.files_with_changes_count);
            tracing::error!("Run with `--generate-baseline` to update the baseline file.");

            Ok(false)
        }
    }

    /// Validates that baseline flags are used with an actual baseline path.
    ///
    /// This method checks whether baseline-related flags like `--generate-baseline`
    /// or `--verify-baseline` are used without specifying a baseline path. It logs
    /// helpful warnings when such misconfigurations are detected.
    ///
    /// Returns `false` if a configuration error is detected, `true` otherwise.
    fn check_baseline_flags(&self) -> bool {
        if self.generate_baseline {
            tracing::warn!("Cannot generate baseline file because no baseline path was specified.");
            tracing::warn!("Use the `--baseline <PATH>` option to specify where to save the baseline file.");
            tracing::warn!("Or set a default baseline path in the configuration file.");

            false
        } else if self.verify_baseline {
            tracing::warn!("Cannot verify baseline file because no baseline path was specified.");
            tracing::warn!("Use the `--baseline <PATH>` option to specify the baseline file to verify.");
            tracing::warn!("Or set a default baseline path in the configuration file.");

            false
        } else if self.fail_on_out_of_sync_baseline {
            tracing::warn!("Cannot fail on out-of-sync baseline because no baseline path was specified.");
            tracing::warn!("Use the `--baseline <PATH>` option to specify the baseline file.");
            tracing::warn!("Or set a default baseline path in the configuration file.");
            true
        } else {
            true
        }
    }

    /// Processes issues with baseline support.
    ///
    /// This is the main entry point for baseline-aware issue processing. It handles
    /// three distinct modes based on command-line flags:
    ///
    /// 1. **Generate mode** (`--generate-baseline`): Creates a new baseline from
    ///    current issues and exits.
    /// 2. **Verify mode** (`--verify-baseline`): Checks if the baseline is
    ///    synchronized with current issues and exits with success or failure.
    /// 3. **Filter mode** (default): Filters current issues against the baseline
    ///    and proceeds to normal reporting.
    ///
    /// The baseline path can come from either the command-line argument or the
    /// configuration file, with command-line taking precedence.
    pub fn process_issues_with_baseline(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        color_choice: ColorChoice,
        database: Database,
        configuration_baseline: Option<PathBuf>,
    ) -> Result<ExitCode, Error> {
        let baseline = match self.resolve_baseline_path(configuration_baseline) {
            Some(baseline_path) => {
                if self.generate_baseline {
                    let read_database = database.read_only();

                    self.generate_baseline(baseline_path, issues, &read_database)?;

                    return Ok(ExitCode::SUCCESS);
                }

                if self.verify_baseline {
                    let read_database = database.read_only();
                    let success = self.verify_baseline(baseline_path, issues, &read_database)?;

                    return Ok(if success { ExitCode::SUCCESS } else { ExitCode::FAILURE });
                }

                self.get_baseline(Some(&baseline_path))
            }
            None => {
                if !self.check_baseline_flags() {
                    return Ok(ExitCode::FAILURE);
                }

                None
            }
        };

        self.reporting.process_issues(
            issues,
            configuration,
            color_choice,
            database,
            baseline,
            self.fail_on_out_of_sync_baseline,
        )
    }
}
