//! Issue processing services for reporting and fixing.
//!
//! This module provides the core services for handling analysis issues in the Mago CLI.
//! It contains two main processors that work together to provide flexible issue management:
//!
//! - [`IssueProcessor`]: Core processor for reporting and fixing issues
//! - [`BaselineIssueProcessor`]: Wrapper that adds baseline functionality to issue processing
//!
//! # Issue Processing Workflow
//!
//! The issue processing workflow follows these general steps:
//!
//! 1. **Issue Collection**: Issues are gathered from linter, analyzer, or other sources
//! 2. **Baseline Filtering** (optional): Issues are filtered against a baseline file
//! 3. **Processing**: Issues are either fixed automatically or reported to the user
//! 4. **Exit Code**: Appropriate exit code is returned based on results
//!
//! # Processor Types
//!
//! ## IssueProcessor
//!
//! The core processor handles two distinct modes:
//!
//! - **Report Mode**: Formats and displays issues using configurable reporters
//! - **Fix Mode**: Applies automatic fixes with configurable safety levels
//!
//! Report mode supports multiple output formats (rich, JSON, SARIF, GitHub Actions),
//! filtering, sorting, and severity-based exit codes.
//!
//! Fix mode applies code transformations in parallel with safety checks, optional
//! formatting, and dry-run preview capabilities.
//!
//! ## BaselineIssueProcessor
//!
//! Adds baseline management capabilities on top of IssueProcessor:
//!
//! - **Baseline Loading**: Reads existing baseline files to filter known issues
//! - **Baseline Generation**: Creates new baseline files from current issues
//! - **Baseline Verification**: Validates that baseline files are up-to-date
//! - **Synchronization Checks**: Optionally fails when baseline is out of sync
//!
//! # Safety Classifications
//!
//! Automatic fixes are categorized by safety:
//!
//! - **Safe**: Always applied with `--fix`
//! - **Potentially Unsafe**: Requires `--potentially-unsafe` flag
//! - **Unsafe**: Requires `--unsafe` flag
//!
//! This graduated approach allows teams to adopt automatic fixing incrementally,
//! starting with safe fixes and progressively enabling more aggressive transformations.

use std::borrow::Cow;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use bumpalo::Bump;
use clap::ColorChoice;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::change::ChangeLog;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::service::format::FileFormatStatus;
use mago_reporting::ColorChoice as ReportingColorChoice;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_reporting::baseline::Baseline;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReporterConfig;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::baseline;
use crate::baseline::unserialize_baseline;
use crate::error::Error;
use crate::utils;

/// Core issue processor for reporting and fixing analysis issues.
///
/// This processor is the main service for handling issues found during linting,
/// type analysis, or other code inspections. It supports two primary modes:
///
/// - **Report Mode** (`fix = false`): Issues are formatted and displayed using
///   the configured reporter with filtering, sorting, and severity-based exit codes.
/// - **Fix Mode** (`fix = true`): Issues with available automatic fixes are applied
///   to the source code with configurable safety levels and optional formatting.
///
/// # Lifecycle
///
/// The processor is typically created from `ReportingArgs` via the `get_processor` method
/// and consumed by calling [`process_issues`](Self::process_issues).
///
/// # Configuration
///
/// The processor's behavior is controlled by its fields, which are typically
/// populated from command-line arguments:
///
/// - Reporting options control output format, target, and filtering
/// - Fix options control safety levels, formatting, and dry-run mode
/// - Exit code behavior is determined by severity thresholds
#[derive(Debug)]
pub struct IssueProcessor<'a> {
    /// Orchestrator for formatting files after fixes are applied.
    ///
    /// Contains the formatter configuration and lifetime-managed resources
    /// needed for code formatting operations.
    pub orchestrator: Orchestrator<'a>,

    /// Database containing all source files being processed.
    ///
    /// In fix mode, this database is modified to apply changes. In report mode,
    /// it provides read-only access to file contents for reporting.
    pub database: Database,

    /// Filter to show only issues that have automatic fixes available.
    ///
    /// When `true`, issues without fixes are excluded from the report.
    /// This is useful for focusing on actionable issues that can be resolved
    /// automatically. Only applies in report mode.
    pub fixable_only: bool,

    /// Sort issues by severity, rule code, and location before reporting.
    ///
    /// When `true`, issues are sorted for better readability instead of being
    /// displayed in file order. Only applies in report mode.
    pub sort: bool,

    /// Enable automatic fix application mode.
    ///
    /// When `true`, switches from report mode to fix mode. Issues with automatic
    /// fixes will be applied to source files instead of being reported.
    pub fix: bool,

    /// Allow application of fixes marked as unsafe.
    ///
    /// Unsafe fixes may change code behavior or have unintended consequences.
    /// Setting this to `true` also enables potentially unsafe fixes.
    /// Only applies in fix mode.
    pub r#unsafe: bool,

    /// Allow application of fixes marked as potentially unsafe.
    ///
    /// Potentially unsafe fixes carry some risk but are generally safer than
    /// unsafe fixes. Only applies in fix mode.
    pub potentially_unsafe: bool,

    /// Automatically format files after applying fixes.
    ///
    /// When `true`, the formatter is run on any files modified by fixes to
    /// ensure consistent code style. Only applies in fix mode.
    pub format_after_fix: bool,

    /// Preview fixes without writing changes to disk.
    ///
    /// When `true`, displays diffs of proposed changes but doesn't modify files.
    /// Useful for reviewing fixes before applying them. Only applies in fix mode.
    pub dry_run: bool,

    /// Output target for issue reports (stdout or stderr).
    ///
    /// Determines where the reporter sends its output. Only applies in report mode.
    pub reporting_target: ReportingTarget,

    /// Output format for issue reports (rich, JSON, SARIF, etc.).
    ///
    /// Determines how issues are formatted for display. Only applies in report mode.
    pub reporting_format: ReportingFormat,

    /// Minimum severity level that causes the command to fail.
    ///
    /// If any issues at or above this level are found, the command exits with
    /// a failure code. Only applies in report mode.
    pub minimum_fail_level: Level,

    /// Minimum severity level to include in the report.
    ///
    /// Issues below this level are completely filtered out. When `None`, all
    /// issues are reported. Only applies in report mode.
    pub minimum_report_level: Option<Level>,

    /// Color output choice for diffs and reports.
    ///
    /// Controls whether colored output is used in diffs (fix mode) and reports
    /// (report mode).
    pub color_choice: ColorChoice,
}

/// Baseline-aware issue processor for incremental issue adoption.
///
/// This processor wraps [`IssueProcessor`] with baseline management capabilities,
/// enabling teams to establish a snapshot of existing issues and focus on preventing
/// new issues from being introduced.
///
/// # Baseline Workflow
///
/// The baseline functionality supports three primary operations:
///
/// 1. **Filtering** (default): Filter reported issues against an existing baseline,
///    suppressing known issues and reporting only new ones.
/// 2. **Generation** (`generate_baseline = true`): Create or update a baseline file
///    from the current set of issues.
/// 3. **Verification** (`verify_baseline = true`): Check if the baseline is up-to-date
///    with current issues, failing if there are discrepancies.
///
/// # Incremental Adoption
///
/// Baselines are particularly useful when introducing linting or analysis to existing
/// codebases. Teams can:
///
/// - Generate a baseline of all existing issues
/// - Focus on preventing new issues in code reviews
/// - Gradually fix old issues and regenerate the baseline
/// - Use `--verify-baseline` in CI to ensure baselines stay current
///
/// # Lifecycle
///
/// The processor is typically created from `BaselineReportingArgs` via the `get_processor`
/// method and consumed by calling [`process_issues`](Self::process_issues).
#[derive(Debug)]
pub struct BaselineIssueProcessor<'a> {
    /// Read-only database access for baseline operations.
    ///
    /// Baseline generation and verification require read access to file metadata
    /// but don't modify the database. The wrapped `issue_processor` has its own
    /// database instance for potential modifications.
    pub read_database: ReadDatabase,

    /// Optional path to the baseline file.
    ///
    /// Can be specified via CLI argument (`--baseline`) or configuration file.
    /// Uses `Cow` to avoid cloning paths from configuration while allowing
    /// owned paths from CLI arguments.
    ///
    /// When `None`, baseline operations are disabled and issues are processed
    /// without filtering.
    pub baseline_path: Option<Cow<'a, Path>>,

    /// Generate a new baseline file from current issues.
    ///
    /// When `true`, instead of processing issues normally, generates a baseline
    /// file and exits. Requires `baseline_path` to be set.
    pub generate_baseline: bool,

    /// Create a backup of the existing baseline before regenerating.
    ///
    /// When `true` and `generate_baseline` is enabled, the existing baseline
    /// file (if any) is renamed with a `.bkp` extension before the new baseline
    /// is written. Only applies when generating baselines.
    pub backup_baseline: bool,

    /// Verify that the baseline is synchronized with current issues.
    ///
    /// When `true`, instead of processing issues normally, compares the baseline
    /// against current issues and reports discrepancies. Exits with failure if
    /// the baseline is out of sync. Requires `baseline_path` to be set.
    pub verify_baseline: bool,

    /// Fail even when only the baseline is out of sync.
    ///
    /// Normally, if there are no new issues to report, the command succeeds even
    /// if the baseline contains issues that no longer exist. When `true`, the
    /// command fails whenever the baseline is out of sync, ensuring baselines
    /// stay clean and up-to-date.
    pub fail_on_out_of_sync_baseline: bool,

    /// Wrapped issue processor for actual issue processing.
    ///
    /// After baseline operations (loading, generation, or verification), this
    /// processor handles the actual reporting or fixing of issues.
    pub issue_processor: IssueProcessor<'a>,
}

impl<'a> IssueProcessor<'a> {
    /// Processes issues by either reporting them or applying fixes.
    ///
    /// This is the main entry point for issue processing. Depending on whether the `--fix`
    /// flag is enabled, it either applies automatic fixes to the code or reports the issues
    /// using the configured format and output settings.
    ///
    /// When applying fixes, only safe fixes are applied by default unless `--unsafe` or
    /// `--potentially-unsafe` flags are provided. When reporting, issues can be filtered,
    /// sorted, and formatted according to the configured options.
    ///
    /// Returns an exit code indicating success or failure based on whether issues were
    /// found and whether they meet the configured failure threshold.
    pub fn process_issues(
        self,
        issues: IssueCollection,
        baseline: Option<Baseline>,
        fail_on_out_of_sync_baseline: bool,
    ) -> Result<ExitCode, Error> {
        if self.fix {
            self.handle_fix_mode(issues)
        } else {
            self.handle_report_mode(issues, baseline, fail_on_out_of_sync_baseline)
        }
    }

    /// Applies automatic fixes to code when the `--fix` flag is enabled.
    ///
    /// This method filters fixes based on safety classification and applies them
    /// in parallel. It respects the `--unsafe` and `--potentially-unsafe` flags
    /// to determine which fixes are safe to apply. When `--format-after-fix` is
    /// enabled, modified files are automatically formatted. When `--dry-run` is
    /// enabled, changes are previewed but not written to disk.
    fn handle_fix_mode(self, issues: IssueCollection) -> Result<ExitCode, Error> {
        let dry_run = self.dry_run;
        let (applied_fixes, skipped_unsafe, skipped_potentially_unsafe) = self.apply_fixes(issues)?;

        if skipped_unsafe > 0 {
            tracing::warn!("Skipped {} unsafe fixes. Use `--unsafe` to apply them.", skipped_unsafe);
        }

        if skipped_potentially_unsafe > 0 {
            tracing::warn!(
                "Skipped {} potentially unsafe fixes. Use `--potentially-unsafe` or `--unsafe` to apply them.",
                skipped_potentially_unsafe
            );
        }

        if applied_fixes == 0 {
            tracing::info!("No fixes were applied.");

            return Ok(ExitCode::SUCCESS);
        }

        if dry_run {
            tracing::info!("Found {} fixes that can be applied (dry-run).", applied_fixes);

            Ok(ExitCode::FAILURE)
        } else {
            tracing::info!("Successfully applied {} fixes.", applied_fixes);

            Ok(ExitCode::SUCCESS)
        }
    }

    /// Reports issues to the configured output target when `--fix` is not enabled.
    ///
    /// This method creates a reporter with the configured settings and outputs
    /// issues according to the specified format. It applies baseline filtering
    /// if a baseline is provided, filters by severity level if configured, and
    /// can optionally filter to show only fixable issues or sort issues for
    /// better readability.
    ///
    /// The exit code is determined by the highest severity level of reported
    /// issues compared to the `--minimum-fail-level` threshold.
    fn handle_report_mode(
        self,
        issues: IssueCollection,
        baseline: Option<Baseline>,
        fail_on_out_of_sync_baseline: bool,
    ) -> Result<ExitCode, Error> {
        let read_database = self.database.read_only();

        let issues_to_report = issues;

        let reporter_configuration = ReporterConfig {
            target: self.reporting_target,
            format: self.reporting_format,
            color_choice: match self.color_choice {
                ColorChoice::Auto => ReportingColorChoice::Auto,
                ColorChoice::Always => ReportingColorChoice::Always,
                ColorChoice::Never => ReportingColorChoice::Never,
            },
            filter_fixable: self.fixable_only,
            sort: self.sort,
            minimum_report_level: self.minimum_report_level,
        };

        let reporter = Reporter::new(read_database, reporter_configuration);
        let status = reporter.report(issues_to_report, baseline)?;

        if status.baseline_dead_issues {
            tracing::warn!(
                "Your baseline file contains entries for issues that no longer exist. Consider regenerating it with `--generate-baseline`."
            );

            if fail_on_out_of_sync_baseline {
                return Ok(ExitCode::FAILURE);
            }
        }

        if status.baseline_filtered_issues > 0 {
            tracing::info!("Filtered out {} issues based on the baseline file.", status.baseline_filtered_issues);
        }

        if let Some(highest_reported_level) = status.highest_reported_level
            && self.minimum_fail_level <= highest_reported_level
        {
            return Ok(ExitCode::FAILURE);
        }

        if status.total_reported_issues == 0 {
            if self.fixable_only {
                tracing::info!("No fixable issues found.");
            } else {
                tracing::info!("No issues found.");
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    /// Applies code fixes in parallel according to safety settings.
    ///
    /// This method extracts fix plans from issues, filters them based on the
    /// configured safety level (safe, potentially unsafe, or unsafe), and applies
    /// them concurrently using a parallel thread pool. Each fix can optionally be
    /// followed by code formatting if `--format-after-fix` is enabled.
    ///
    /// Returns the count of applied fixes and the counts of skipped fixes by
    /// safety classification.
    fn apply_fixes(mut self, issues: IssueCollection) -> Result<(usize, usize, usize), Error> {
        let read_database = Arc::new(self.database.read_only());
        let change_log = ChangeLog::new();

        let (fix_plans, skipped_unsafe, skipped_potentially_unsafe) = self.filter_fix_plans(&read_database, issues);

        if fix_plans.is_empty() {
            return Ok((0, skipped_unsafe, skipped_potentially_unsafe));
        }

        let changed_results: Vec<bool> = fix_plans
            .into_par_iter()
            .map_init(Bump::new, |arena, (file_id, plan)| {
                let file = read_database.get_ref(&file_id)?;
                let fixed_content = plan.execute(&file.contents).get_fixed();
                let final_content = if self.format_after_fix {
                    let file = File::ephemeral(file.name.clone(), Cow::Owned(fixed_content));
                    let format_status = self.orchestrator.format_file_in(&file, arena)?;

                    match format_status {
                        FileFormatStatus::Unchanged => file.contents.into_owned(),
                        FileFormatStatus::Changed(new_content) => new_content,
                        FileFormatStatus::FailedToParse(parse_error) => {
                            tracing::warn!(
                                "Failed to format file `{}` after applying fixes: {}",
                                file.name.as_ref(),
                                parse_error
                            );

                            file.contents.into_owned()
                        }
                    }
                } else {
                    fixed_content
                };

                arena.reset();

                utils::apply_update(&change_log, file, final_content.as_ref(), self.dry_run, self.color_choice)
            })
            .collect::<Result<Vec<bool>, Error>>()?;

        if !self.dry_run {
            self.database.commit(change_log, true)?;
        }

        let applied_fix_count = changed_results.into_iter().filter(|&c| c).count();

        Ok((applied_fix_count, skipped_unsafe, skipped_potentially_unsafe))
    }

    /// Filters fix plans based on configured safety thresholds.
    ///
    /// This method examines each fix operation's safety classification and
    /// includes or skips it based on the `--unsafe` and `--potentially-unsafe`
    /// flags. Safe fixes are always included.
    ///
    /// Returns a tuple containing the list of applicable fix plans and the
    /// counts of skipped fixes by safety classification.
    #[inline]
    fn filter_fix_plans(
        &self,
        database: &ReadDatabase,
        issues: IssueCollection,
    ) -> (Vec<(FileId, FixPlan)>, usize, usize) {
        let mut skipped_unsafe_count = 0;
        let mut skipped_potentially_unsafe_count = 0;
        let mut applicable_plans = Vec::new();

        for (file_id, plan) in issues.to_fix_plans() {
            if plan.is_empty() {
                continue;
            }

            let mut filtered_operations = Vec::new();
            for operation in plan.take_operations() {
                // Consumes operations from the plan
                match operation.get_safety_classification() {
                    SafetyClassification::Unsafe => {
                        if self.r#unsafe {
                            filtered_operations.push(operation);
                        } else {
                            skipped_unsafe_count += 1;
                            tracing::debug!(
                                "Skipping unsafe fix for `{}`. Use --unsafe to apply.",
                                database.get_ref(&file_id).map(|f| f.name.as_ref()).unwrap_or("<unknown>"),
                            );
                        }
                    }
                    SafetyClassification::PotentiallyUnsafe => {
                        if self.r#unsafe || self.potentially_unsafe {
                            filtered_operations.push(operation);
                        } else {
                            skipped_potentially_unsafe_count += 1;
                            tracing::debug!(
                                "Skipping potentially unsafe fix for `{}`. Use --potentially-unsafe or --unsafe to apply.",
                                database.get_ref(&file_id).map(|f| f.name.as_ref()).unwrap_or("<unknown>"),
                            );
                        }
                    }
                    SafetyClassification::Safe => {
                        filtered_operations.push(operation);
                    }
                }
            }

            if !filtered_operations.is_empty() {
                applicable_plans.push((file_id, FixPlan::from_operations(filtered_operations)));
            }
        }

        (applicable_plans, skipped_unsafe_count, skipped_potentially_unsafe_count)
    }
}

impl<'a> BaselineIssueProcessor<'a> {
    /// Processes issues with baseline awareness.
    ///
    /// This method orchestrates the complete baseline-aware issue processing workflow.
    /// Depending on the configured flags, it either generates a baseline, verifies a
    /// baseline, or processes issues normally with optional baseline filtering.
    ///
    /// # Workflow
    ///
    /// The method follows this decision tree:
    ///
    /// 1. **Baseline Path Provided**:
    ///    - If `generate_baseline` is `true`: Generate new baseline and exit with success
    ///    - If `verify_baseline` is `true`: Verify baseline and exit with success/failure
    ///    - Otherwise: Load baseline and pass to wrapped processor for filtering
    /// 2. **No Baseline Path**:
    ///    - Validate that baseline-related flags are consistent
    ///    - Process issues normally without baseline filtering
    ///
    /// # Arguments
    ///
    /// * `issues` - The collection of issues to process
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` - Operation completed successfully
    /// - `Ok(ExitCode::FAILURE)` - Issues found above fail threshold, baseline out of sync,
    ///   or validation failed
    /// - `Err(Error)` - Baseline I/O error or issue processing error
    pub fn process_issues(self, issues: IssueCollection) -> Result<ExitCode, Error> {
        let baseline = match self.baseline_path.as_deref() {
            Some(baseline_path) => {
                if self.generate_baseline {
                    self.generate_baseline(baseline_path, issues)?;

                    return Ok(ExitCode::SUCCESS);
                }

                if self.verify_baseline {
                    let success = self.verify_baseline(baseline_path, issues)?;

                    return Ok(if success { ExitCode::SUCCESS } else { ExitCode::FAILURE });
                }

                self.get_baseline(Some(baseline_path))
            }
            None => {
                if !self.validate_baseline_parameters() {
                    return Ok(ExitCode::FAILURE);
                }

                None
            }
        };

        self.issue_processor.process_issues(issues, baseline, self.fail_on_out_of_sync_baseline)
    }

    /// Loads an existing baseline file from disk.
    ///
    /// This method attempts to read and deserialize a baseline file from the specified
    /// path. If the file doesn't exist or cannot be read, appropriate warnings or
    /// errors are logged and `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `baseline_path` - Optional path to the baseline file
    ///
    /// # Returns
    ///
    /// - `Some(Baseline)` if the file exists and was successfully deserialized
    /// - `None` if no path was provided, the file doesn't exist, or deserialization failed
    fn get_baseline(&self, baseline_path: Option<&'a Path>) -> Option<Baseline> {
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

    /// Generates a new baseline file from the provided issues.
    ///
    /// This method creates a baseline containing all issues in the provided collection,
    /// writes it to the specified path, and optionally creates a backup of any existing
    /// baseline file.
    ///
    /// # Arguments
    ///
    /// * `baseline_path` - Path where the baseline file should be written
    /// * `issues` - Collection of issues to include in the baseline
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the baseline was successfully generated and written
    /// - `Err(Error)` if file I/O operations failed
    ///
    /// # Side Effects
    ///
    /// - Creates or overwrites the baseline file at `baseline_path`
    /// - If `backup_baseline` is `true` and a baseline already exists, creates a backup
    ///   with a `.bkp` extension
    /// - Logs informational messages about the generation process
    fn generate_baseline(&self, baseline_path: &'a Path, issues: IssueCollection) -> Result<(), Error> {
        tracing::info!("Generating baseline file...");
        let baseline = Baseline::generate_from_issues(&issues, &self.read_database);
        baseline::serialize_baseline(baseline_path, &baseline, self.backup_baseline)?;
        tracing::info!("Baseline file successfully generated at `{}`.", baseline_path.display());

        Ok(())
    }

    /// Verifies that an existing baseline is synchronized with current issues.
    ///
    /// This method loads the baseline file, compares it against the current set of issues,
    /// and reports any discrepancies. Discrepancies include new issues not in the baseline
    /// and baseline entries for issues that no longer exist.
    ///
    /// # Arguments
    ///
    /// * `baseline_path` - Path to the baseline file to verify
    /// * `issues` - Current collection of issues to compare against the baseline
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the baseline is up-to-date with no discrepancies
    /// - `Ok(false)` if the baseline is out of sync (new issues, removed issues, or file changes)
    /// - `Err(Error)` if the baseline file cannot be read or deserialized
    ///
    /// # Side Effects
    ///
    /// Logs informational messages, warnings, and errors about the verification results:
    /// - Info: Whether verification is starting or if baseline is up-to-date
    /// - Warning: Counts of new issues and removed issues
    /// - Error: Overall summary of files with changes and suggestion to regenerate
    fn verify_baseline(&self, baseline_path: &'a Path, issues: IssueCollection) -> Result<bool, Error> {
        if !baseline_path.exists() {
            tracing::info!("Baseline file `{}` does not exist.", baseline_path.display());
            return Ok(false);
        }

        tracing::info!("Verifying baseline file at `{}`...", baseline_path.display());

        let baseline = unserialize_baseline(baseline_path)?;
        let comparison = baseline.compare_with_issues(&issues, &self.read_database);

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

    /// Validates that baseline-related flags are consistent with the absence of a baseline path.
    ///
    /// This method checks whether baseline operations were requested (`generate_baseline`,
    /// `verify_baseline`, or `fail_on_out_of_sync_baseline`) without providing a baseline
    /// path. It logs appropriate warnings when invalid combinations are detected.
    ///
    /// # Returns
    ///
    /// - `true` if the configuration is valid (no baseline operations requested, or only
    ///   `fail_on_out_of_sync_baseline` is set)
    /// - `false` if `generate_baseline` or `verify_baseline` is set without a baseline path
    ///
    /// # Side Effects
    ///
    /// Logs warnings when invalid configurations are detected, guiding the user to either:
    /// - Use the `--baseline <PATH>` CLI option
    /// - Set a default baseline path in the configuration file
    fn validate_baseline_parameters(&self) -> bool {
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
}
