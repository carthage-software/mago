use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_database::ReadDatabase;
use mago_reporting::IssueCollection;

use crate::baseline;
use crate::error::Error;

/// Defines command-line options for baseline functionality.
///
/// This struct is designed to be flattened into other clap commands
/// that require baseline functionality for filtering issues.
#[derive(Parser, Debug, Clone)]
pub struct BaselineArgs {
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
    #[arg(long, requires = "baseline")]
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
    #[arg(long, conflicts_with = "generate_baseline", requires = "baseline")]
    pub verify_baseline: bool,

    /// Fail the command when baseline is out of sync, even with no new issues.
    ///
    /// Normally, if there are no current issues to report, the command succeeds
    /// even if the baseline is outdated. This flag forces failure when the baseline
    /// contains issues that no longer exist, ensuring baselines stay clean.
    #[arg(long, conflicts_with = "generate_baseline", conflicts_with = "verify_baseline", requires = "baseline")]
    pub fail_on_out_of_sync_baseline: bool,
}

impl BaselineArgs {
    /// Resolves the baseline path by taking the command-line argument if provided,
    /// otherwise falling back to the configuration default for the specified component.
    ///
    /// # Arguments
    ///
    /// * `config_baseline`: The baseline path from the configuration (linter or analyzer).
    ///
    /// # Returns
    ///
    /// The effective baseline path, prioritizing command-line arguments over configuration defaults.
    pub fn resolve_baseline(&self, config_baseline: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
        self.baseline.clone().or_else(|| config_baseline.map(|p| p.to_path_buf()))
    }

    /// Processes issues using baseline filtering.
    ///
    /// # Arguments
    ///
    /// * `issues`: The collection of issues to process.
    /// * `config_baseline`: Optional baseline from configuration to use as default.
    /// * `read_database`: The read-only database for issue context.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The filtered issues
    /// - Whether the command should fail due to baseline operations
    /// - An optional exit code for early termination (e.g., baseline generation/verification)
    pub fn process_baseline(
        &self,
        mut issues: IssueCollection,
        config_baseline: Option<&std::path::Path>,
        read_database: &ReadDatabase,
    ) -> Result<(IssueCollection, bool, Option<ExitCode>), Error> {
        let baseline_path = self.resolve_baseline(config_baseline);
        let mut should_fail = false;

        if let Some(baseline_path) = &baseline_path {
            if self.generate_baseline {
                tracing::info!("Generating baseline file...");
                let baseline = baseline::generate_baseline_from_issues(issues, read_database)?;
                baseline::serialize_baseline(baseline_path, &baseline, self.backup_baseline)?;
                tracing::info!("Baseline file successfully generated at `{}`.", baseline_path.display());

                return Ok((IssueCollection::default(), false, Some(ExitCode::SUCCESS)));
            }

            if self.verify_baseline {
                return Ok((
                    IssueCollection::default(),
                    false,
                    Some(self.handle_baseline_verification(issues, baseline_path, read_database)?),
                ));
            }

            if !baseline_path.exists() {
                tracing::warn!(
                    "Baseline file `{}` does not exist. Issues will not be filtered.",
                    baseline_path.display()
                );
            } else {
                let baseline = baseline::unserialize_baseline(baseline_path)?;
                let (filtered_issues, filtered_out_count, has_dead_issues) =
                    baseline::filter_issues(&baseline, issues, read_database)?;

                if has_dead_issues {
                    tracing::warn!(
                        "Your baseline file contains entries for issues that no longer exist. Consider regenerating it with `--generate-baseline`."
                    );

                    if self.fail_on_out_of_sync_baseline {
                        should_fail = true;
                    }
                }

                if filtered_out_count > 0 {
                    tracing::info!(
                        "Filtered out {} issues based on the baseline file at `{}`.",
                        filtered_out_count,
                        baseline_path.display()
                    );
                }

                issues = filtered_issues;
            }
        }

        Ok((issues, should_fail, None))
    }

    /// Handles baseline verification logic.
    ///
    /// Compares the current issues against the baseline to determine if it's up to date.
    /// Returns ExitCode::SUCCESS (0) if baseline is up to date, ExitCode::FAILURE (1) if not.
    fn handle_baseline_verification(
        &self,
        issues: IssueCollection,
        baseline_path: &std::path::Path,
        read_database: &ReadDatabase,
    ) -> Result<ExitCode, Error> {
        if !baseline_path.exists() {
            tracing::info!("Baseline file `{}` does not exist.", baseline_path.display());
            return Ok(ExitCode::FAILURE);
        }

        tracing::info!("Verifying baseline file at `{}`...", baseline_path.display());

        let baseline = baseline::unserialize_baseline(baseline_path)?;
        let comparison = baseline::compare_baseline_with_issues(&baseline, issues, read_database)?;

        if comparison.is_up_to_date {
            tracing::info!("Baseline is up to date.");

            Ok(ExitCode::SUCCESS)
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

            Ok(ExitCode::FAILURE)
        }
    }
}
