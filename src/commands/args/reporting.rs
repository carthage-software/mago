use std::process::ExitCode;

use clap::Parser;
use tokio::task::JoinHandle;

use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_formatter::Formatter;
use mago_interner::ThreadedInterner;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;

use crate::config::Configuration;
use crate::enum_variants;
use crate::error::Error;
use crate::utils;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

/// Defines command-line options for issue reporting and fixing.
///
/// This struct is designed to be flattened into other clap commands
/// that require functionality for reporting and/or automatically fixing issues.
#[derive(Parser, Debug, Clone, Copy)]
pub struct ReportingArgs {
    /// Filter the output to only show issues that can be automatically fixed.
    #[arg(long, short = 'f', help = "Filter output to show only fixable issues", default_value_t = false)]
    pub fixable_only: bool,

    /// Sort reported issues by level, code, and location.
    #[arg(long, help = "Sort reported issues by level, code, and location")]
    pub sort: bool,

    /// Apply fixes to the source code where possible.
    #[arg(long, help = "Apply fixes to the source code", conflicts_with = "fixable_only")]
    pub fix: bool,

    /// Apply fixes that are marked as unsafe.
    ///
    /// Unsafe fixes might have unintended consequences or alter the code's behavior
    /// in a way that requires manual verification.
    #[arg(long, help = "Apply fixes marked as unsafe (requires --fix)", requires = "fix")]
    pub r#unsafe: bool,

    /// Apply fixes that are marked as potentially unsafe.
    ///
    /// Potentially unsafe fixes are less risky than unsafe ones but may still
    /// require manual review after application.
    #[arg(long, help = "Apply fixes marked as potentially unsafe (requires --fix)", requires = "fix")]
    pub potentially_unsafe: bool,

    /// Format the fixed files after applying changes.
    #[arg(long, help = "Format fixed files after applying changes (requires --fix)", alias = "fmt", requires = "fix")]
    pub format_after_fix: bool,

    /// Preview fixes without writing any changes to disk.
    ///
    /// This option shows what changes would be made if fixes were applied.
    #[arg(long, short = 'd', help = "Preview fixes without applying them (requires --fix)", requires = "fix")]
    pub dry_run: bool,

    /// Specify where the results should be reported (e.g., stdout, stderr).
    #[arg(long, default_value_t, help = "Specify reporting target (e.g., stdout, stderr)", ignore_case = true, value_parser = enum_variants!(ReportingTarget), conflicts_with = "fix")]
    pub reporting_target: ReportingTarget,

    /// Choose the format for reporting issues (e.g., human-readable, JSON).
    #[arg(long, default_value_t, help = "Choose reporting format (e.g., rich, medium, short)", ignore_case = true, value_parser = enum_variants!(ReportingFormat), conflicts_with = "fix")]
    pub reporting_format: ReportingFormat,

    /// Set the minimum issue level that will cause the command to fail.
    ///
    /// For example, if set to `Error`, warnings or notices will not cause a failure exit code.
    #[arg(long, short = 'm', help = "Set minimum issue level for failure (e.g., error, warning)", default_value_t = Level::Error, value_parser = enum_variants!(Level), conflicts_with = "fix")]
    pub minimum_fail_level: Level,
}

impl ReportingArgs {
    /// Processes and reports issues, optionally applying fixes.
    ///
    /// This is the main entry point for handling issues collected by a command.
    /// It will either report the issues or attempt to fix them based on the
    /// provided arguments.
    ///
    /// # Arguments
    ///
    /// * `self` - The reporting arguments.
    /// * `issues` - A collection of issues to process.
    /// * `configuration` - The application's configuration.
    /// * `interner` - A threaded interner for string interning.
    /// * `source_manager` - Manages source file access.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with an `ExitCode` indicating success or failure,
    /// or an `Error` if an unrecoverable problem occurs.
    pub async fn process_issues(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        interner: ThreadedInterner,
        source_manager: SourceManager,
    ) -> Result<ExitCode, Error> {
        if self.fix {
            self.handle_fix_mode(issues, configuration, interner, source_manager).await
        } else {
            self.handle_report_mode(issues, interner, source_manager).await
        }
    }

    /// Handles the logic for when the `--fix` flag is enabled.
    async fn handle_fix_mode(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        interner: ThreadedInterner,
        source_manager: SourceManager,
    ) -> Result<ExitCode, Error> {
        let (applied_fixes, skipped_unsafe, skipped_potentially_unsafe) =
            self.apply_fixes(issues, configuration, interner, source_manager).await?;

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

        if self.dry_run {
            tracing::info!("Found {} fixes that can be applied (dry-run).", applied_fixes);

            Ok(ExitCode::FAILURE)
        } else {
            tracing::info!("Successfully applied {} fixes.", applied_fixes);

            Ok(ExitCode::SUCCESS)
        }
    }

    /// Handles the logic for reporting issues (when `--fix` is not enabled).
    async fn handle_report_mode(
        self,
        mut issues: IssueCollection,
        interner: ThreadedInterner,
        source_manager: SourceManager,
    ) -> Result<ExitCode, Error> {
        let has_issues_above_threshold = issues.has_minimum_level(self.minimum_fail_level);

        if self.sort {
            issues = issues.sorted();
        }

        let reporter = Reporter::new(interner.clone(), source_manager.clone(), self.reporting_target);

        let issues_to_report = if self.fixable_only { issues.only_fixable().collect() } else { issues };

        if issues_to_report.is_empty() {
            if self.fixable_only {
                tracing::info!("No fixable issues found.");
            } else {
                tracing::info!("No issues found.");
            }
        } else {
            reporter.report(issues_to_report, self.reporting_format)?;
        }

        Ok(if has_issues_above_threshold { ExitCode::FAILURE } else { ExitCode::SUCCESS })
    }

    /// Applies fixes to the issues provided.
    ///
    /// This function filters fix plans based on safety settings,
    /// then applies the fixes concurrently.
    ///
    /// # Returns
    ///
    /// A tuple: `(applied_fix_count, skipped_unsafe_count, skipped_potentially_unsafe_count)`.
    async fn apply_fixes(
        &self,
        issues: IssueCollection,
        configuration: Configuration,
        interner: ThreadedInterner,
        source_manager: SourceManager,
    ) -> Result<(usize, usize, usize), Error> {
        let (fix_plans, skipped_unsafe, skipped_potentially_unsafe) = self.filter_fix_plans(&interner, issues);

        if fix_plans.is_empty() {
            return Ok((0, skipped_unsafe, skipped_potentially_unsafe));
        }

        let total_plans = fix_plans.len();
        let progress_bar = create_progress_bar(total_plans, "✨ Fixing", ProgressBarTheme::Cyan);
        let mut fix_tasks: Vec<JoinHandle<Result<bool, Error>>> = Vec::with_capacity(total_plans);

        for (source_id, plan) in fix_plans {
            let source_manager_clone = source_manager.clone();
            let interner_clone = interner.clone();
            let progress_bar_clone = progress_bar.clone();
            let formatter_settings_clone = configuration.format.settings;
            let php_version = configuration.php_version;
            let should_format = self.format_after_fix;
            let dry_run = self.dry_run;

            fix_tasks.push(tokio::spawn(async move {
                // Load source and original content
                let source_file = source_manager_clone.load(&source_id)?;
                let original_content = interner_clone.lookup(&source_file.content);

                // Execute fix plan
                let mut fixed_content = plan.execute(original_content).get_fixed();

                // Optionally format
                if should_format {
                    let formatter = Formatter::new(&interner_clone, php_version, formatter_settings_clone);
                    match formatter.format_code(interner_clone.lookup(&source_file.identifier.0), &fixed_content) {
                        Ok(content) => fixed_content = content,
                        Err(e) => {
                            tracing::error!("Failed to format {}: {}", interner_clone.lookup(&source_id.0), e);
                        }
                    }
                }

                let changed =
                    utils::apply_changes(&interner_clone, &source_manager_clone, &source_file, fixed_content, dry_run)?;
                progress_bar_clone.inc(1);
                Ok(changed)
            }));
        }

        let mut applied_fix_count = 0;
        for task in fix_tasks {
            if task.await?? {
                applied_fix_count += 1;
            }
        }

        remove_progress_bar(progress_bar);

        Ok((applied_fix_count, skipped_unsafe, skipped_potentially_unsafe))
    }

    /// Filters fix operations from issues based on safety settings.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A vector of `(SourceIdentifier, FixPlan)` for applicable fixes.
    /// * The count of fixes skipped due to being `Unsafe`.
    /// * The count of fixes skipped due to being `PotentiallyUnsafe`.
    #[inline]
    fn filter_fix_plans(
        &self,
        interner: &ThreadedInterner,
        issues: IssueCollection,
    ) -> (Vec<(SourceIdentifier, FixPlan)>, usize, usize) {
        let mut skipped_unsafe_count = 0;
        let mut skipped_potentially_unsafe_count = 0;
        let mut applicable_plans = Vec::new();

        for (source_id, plan) in issues.to_fix_plans() {
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
                                interner.lookup(&source_id.0)
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
                                interner.lookup(&source_id.0)
                            );
                        }
                    }
                    SafetyClassification::Safe => {
                        filtered_operations.push(operation);
                    }
                }
            }

            if !filtered_operations.is_empty() {
                applicable_plans.push((source_id, FixPlan::from_operations(filtered_operations)));
            }
        }

        (applicable_plans, skipped_unsafe_count, skipped_potentially_unsafe_count)
    }
}
