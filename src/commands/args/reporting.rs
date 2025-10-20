//! Command-line arguments for issue reporting and fixing.
//!
//! This module defines the command-line interface for controlling how issues are
//! reported and optionally fixed. It provides options for filtering, sorting, and
//! formatting issue output, as well as applying automatic fixes to code.
//!
//! The reporting functionality supports multiple output formats (rich terminal output,
//! JSON, checkstyle, etc.), different output targets (stdout/stderr), and various
//! filtering options. The fixing functionality can apply safe, potentially unsafe,
//! or unsafe fixes based on user preferences.

use std::borrow::Cow;
use std::process::ExitCode;
use std::sync::Arc;

use bumpalo::Bump;
use clap::ColorChoice;
use clap::Parser;
use mago_reporting::baseline::Baseline;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::change::ChangeLog;
use mago_database::file::FileId;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_formatter::Formatter;
use mago_reporting::ColorChoice as ReportingColorChoice;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReporterConfig;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;

use crate::commands::args::pager::PagerArgs;
use crate::config::Configuration;
use crate::enum_variants;
use crate::error::Error;
use crate::utils;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

/// Command-line arguments for issue reporting and fixing.
///
/// This struct is designed to be flattened into other clap commands
/// that require functionality for reporting and/or automatically fixing issues.
#[derive(Parser, Debug, Clone)]
pub struct ReportingArgs {
    /// Filter the output to only show issues that can be automatically fixed.
    ///
    /// When enabled, only issues that have available automatic fixes will be displayed.
    /// This is useful when you want to focus on issues that can be resolved immediately.
    #[arg(long, short = 'f')]
    pub fixable_only: bool,

    /// Sort reported issues by severity level, rule code, and file location.
    ///
    /// By default, issues are reported in the order they appear in files.
    /// This option provides a more organized view for reviewing large numbers of issues.
    #[arg(long)]
    pub sort: bool,

    /// Apply automatic fixes to the source code where possible.
    ///
    /// This will modify your files to fix issues that have automatic solutions.
    /// Only safe fixes are applied by default. Use --unsafe or --potentially-unsafe
    /// to enable riskier fixes. Cannot be used with --fixable-only.
    #[arg(long, conflicts_with = "fixable_only")]
    pub fix: bool,

    /// Apply fixes that are marked as unsafe.
    ///
    /// Unsafe fixes might change code behavior or have unintended consequences.
    /// Always review changes carefully after applying unsafe fixes.
    /// Requires --fix to be enabled.
    #[arg(long, requires = "fix")]
    pub r#unsafe: bool,

    /// Apply fixes that are marked as potentially unsafe.
    ///
    /// These fixes are less risky than unsafe ones but may still require
    /// manual review to ensure they don't break your code's intended behavior.
    /// Requires --fix to be enabled.
    #[arg(long, requires = "fix")]
    pub potentially_unsafe: bool,

    /// Format the fixed files after applying changes.
    ///
    /// This runs the formatter on any files that were modified by fixes
    /// to ensure consistent code style. Requires --fix to be enabled.
    #[arg(long, alias = "fmt", requires = "fix")]
    pub format_after_fix: bool,

    /// Preview fixes without writing any changes to disk.
    ///
    /// Shows exactly what changes would be made if fixes were applied,
    /// but doesn't modify any files. Useful for reviewing fixes before applying them.
    /// Requires --fix to be enabled.
    #[arg(long, short = 'd', requires = "fix", alias = "diff")]
    pub dry_run: bool,

    /// Specify where to send the output.
    ///
    /// Choose stdout for normal output or stderr for error streams.
    /// Not available when using --fix mode.
    #[arg(
        long,
        default_value_t,
        ignore_case = true,
        value_parser = enum_variants!(ReportingTarget),
        conflicts_with = "fix"
    )]
    pub reporting_target: ReportingTarget,

    /// Choose the output format for issue reports.
    ///
    /// Available formats: rich (colorful, detailed), medium (balanced),
    /// short (compact), json (machine-readable), and others.
    ///
    /// Not available when using --fix mode.
    #[arg(
        long,
        default_value_t,
        ignore_case = true,
        value_parser = enum_variants!(ReportingFormat),
        conflicts_with = "fix"
    )]
    pub reporting_format: ReportingFormat,

    /// Set the minimum issue severity that causes the command to fail.
    ///
    /// The command will exit with a non-zero status if any issues at or above
    /// this level are found. For example, setting this to 'warning' means
    /// the command fails on warnings and errors, but not on notes or help suggestions.
    #[arg(
        long,
        short = 'm',
        default_value_t = Level::Error,
        value_parser = enum_variants!(Level),
        conflicts_with = "fix"
    )]
    pub minimum_fail_level: Level,

    /// Set the minimum issue severity to be shown in the report.
    ///
    /// Issues below this level will be completely ignored and not displayed.
    /// This is different from --minimum-fail-level which only affects exit status.
    /// Useful for filtering out low-priority suggestions.
    #[arg(
        long,
        value_parser = enum_variants!(Level)
    )]
    pub minimum_report_level: Option<Level>,

    #[clap(flatten)]
    pub pager_args: PagerArgs,
}

impl ReportingArgs {
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
        configuration: Configuration,
        color_choice: ColorChoice,
        database: Database,
        baseline: Option<Baseline>,
        fail_on_out_of_sync_baseline: bool,
    ) -> Result<ExitCode, Error> {
        if self.fix {
            self.handle_fix_mode(issues, configuration, color_choice, database)
        } else {
            self.handle_report_mode(
                issues,
                configuration,
                color_choice,
                database,
                baseline,
                fail_on_out_of_sync_baseline,
            )
        }
    }

    /// Applies automatic fixes to code when the `--fix` flag is enabled.
    ///
    /// This method filters fixes based on safety classification and applies them
    /// in parallel. It respects the `--unsafe` and `--potentially-unsafe` flags
    /// to determine which fixes are safe to apply. When `--format-after-fix` is
    /// enabled, modified files are automatically formatted. When `--dry-run` is
    /// enabled, changes are previewed but not written to disk.
    fn handle_fix_mode(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        color_choice: ColorChoice,
        mut database: Database,
    ) -> Result<ExitCode, Error> {
        let (applied_fixes, skipped_unsafe, skipped_potentially_unsafe) =
            self.apply_fixes(issues, configuration, color_choice, &mut database)?;

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
        configuration: Configuration,
        color_choice: ColorChoice,
        database: Database,
        baseline: Option<Baseline>,
        fail_on_out_of_sync_baseline: bool,
    ) -> Result<ExitCode, Error> {
        let read_database = database.read_only();

        let issues_to_report = issues;

        let reporter_configuration = ReporterConfig {
            target: self.reporting_target,
            format: self.reporting_format,
            color_choice: match color_choice {
                ColorChoice::Auto => ReportingColorChoice::Auto,
                ColorChoice::Always => ReportingColorChoice::Always,
                ColorChoice::Never => ReportingColorChoice::Never,
            },
            filter_fixable: self.fixable_only,
            sort: self.sort,
            minimum_report_level: self.minimum_report_level,
            use_pager: self.pager_args.should_use_pager(&configuration),
            pager_command: configuration.pager.clone(),
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
    fn apply_fixes(
        &self,
        issues: IssueCollection,
        configuration: Configuration,
        color_choice: ColorChoice,
        database: &mut Database,
    ) -> Result<(usize, usize, usize), Error> {
        let read_database = Arc::new(database.read_only());
        let change_log = ChangeLog::new();

        let (fix_plans, skipped_unsafe, skipped_potentially_unsafe) = self.filter_fix_plans(&read_database, issues);

        if fix_plans.is_empty() {
            return Ok((0, skipped_unsafe, skipped_potentially_unsafe));
        }

        let progress_bar = create_progress_bar(fix_plans.len(), "✨ Fixing", ProgressBarTheme::Cyan);

        let changed_results: Vec<bool> = fix_plans
            .into_par_iter()
            .map_init(Bump::new, |arena, (file_id, plan)| {
                arena.reset();

                let file = read_database.get_ref(&file_id)?;
                let fixed_content = plan.execute(&file.contents).get_fixed();
                let final_content = if self.format_after_fix {
                    let formatter = Formatter::new(arena, configuration.php_version, configuration.formatter.settings);

                    if let Ok(content) = formatter.format_code(file.name.clone(), Cow::Owned(fixed_content.clone())) {
                        Cow::Borrowed(content)
                    } else {
                        Cow::Owned(fixed_content)
                    }
                } else {
                    Cow::Owned(fixed_content)
                };

                let changed =
                    utils::apply_update(&change_log, file, final_content.as_ref(), self.dry_run, false, color_choice)?;
                progress_bar.inc(1);
                Ok(changed)
            })
            .collect::<Result<Vec<bool>, Error>>()?;

        remove_progress_bar(progress_bar);

        if !self.dry_run {
            database.commit(change_log, true)?;
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
