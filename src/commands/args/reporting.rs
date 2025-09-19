use std::borrow::Cow;
use std::process::ExitCode;
use std::sync::Arc;

use bumpalo::Bump;
use clap::Parser;
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
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_reporting::reporter::Reporter;
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

/// Defines command-line options for issue reporting and fixing.
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
    /// Orchestrates the entire issue processing pipeline.
    ///
    /// # Arguments
    ///
    /// * `self`: The configured reporting arguments from the command line.
    /// * `issues`: The collection of issues detected by the preceding command.
    /// * `configuration`: The application's global configuration.
    /// * `should_use_colors`: Whether to use colors in output.
    /// * `database`: The mutable database containing all source files.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ExitCode` to indicate success or failure to the shell,
    /// or an `Error` if an unrecoverable problem occurs.
    pub fn process_issues(
        self,
        mut issues: IssueCollection,
        configuration: Configuration,
        should_use_colors: bool,
        database: Database,
    ) -> Result<ExitCode, Error> {
        if let Some(min_level) = self.minimum_report_level {
            let unfiltered_count = issues.len();
            issues = issues.with_minimum_level(min_level);

            tracing::debug!(
                "Filtered out {} issues below the minimum report level of `{}`.",
                unfiltered_count - issues.len(),
                min_level.to_string(),
            );
        }

        if self.fix {
            self.handle_fix_mode(issues, configuration, should_use_colors, database)
        } else {
            self.handle_report_mode(issues, &configuration, should_use_colors, database, false)
        }
    }

    /// Handles the logic for when the `--fix` flag is enabled.
    fn handle_fix_mode(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        should_use_colors: bool,
        mut database: Database,
    ) -> Result<ExitCode, Error> {
        let (applied_fixes, skipped_unsafe, skipped_potentially_unsafe) =
            self.apply_fixes(issues, &configuration, should_use_colors, &mut database)?;

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
    fn handle_report_mode(
        self,
        mut issues: IssueCollection,
        configuration: &Configuration,
        should_use_colors: bool,
        database: Database,
        should_fail_from_baseline: bool,
    ) -> Result<ExitCode, Error> {
        let read_database = database.read_only();

        if self.sort {
            issues = issues.sorted();
        }

        let should_fail = should_fail_from_baseline || issues.has_minimum_level(self.minimum_fail_level);
        let issues_to_report = if self.fixable_only { issues.only_fixable().collect() } else { issues };

        if issues_to_report.is_empty() {
            if self.fixable_only {
                tracing::info!("No fixable issues found.");
            } else {
                tracing::info!("No issues found.");
            }
        } else {
            let reporter = Reporter::new(
                read_database,
                self.reporting_target,
                should_use_colors,
                self.pager_args.should_use_pager(configuration),
                configuration.pager.clone(),
            );

            reporter.report(issues_to_report, self.reporting_format)?;
        }

        Ok(if should_fail { ExitCode::FAILURE } else { ExitCode::SUCCESS })
    }

    /// Applies fixes to the issues provided using a parallel pipeline.
    ///
    /// This function filters fix plans based on safety settings, then applies the
    /// fixes concurrently using a rayon thread pool.
    ///
    /// # Returns
    ///
    /// A tuple: `(applied_fix_count, skipped_unsafe_count, skipped_potentially_unsafe_count)`.
    fn apply_fixes(
        &self,
        issues: IssueCollection,
        configuration: &Configuration,
        should_use_colors: bool,
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

                let changed = utils::apply_update(
                    &change_log,
                    file,
                    final_content.as_ref(),
                    self.dry_run,
                    false,
                    should_use_colors,
                )?;
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

    /// Filters fix operations from issues based on safety settings.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A vector of `(FileId, FixPlan)` for applicable fixes.
    /// * The count of fixes skipped due to being `Unsafe`.
    /// * The count of fixes skipped due to being `PotentiallyUnsafe`.
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

    /// Extended version of process_issues that works with baseline functionality.
    ///
    /// This method accepts pre-processed issues (already filtered through baseline)
    /// and an indicator if the baseline processing determined the command should fail.
    ///
    /// # Arguments
    ///
    /// * `self`: The configured reporting arguments from the command line.
    /// * `issues`: The collection of issues (potentially already filtered by baseline).
    /// * `configuration`: The application's global configuration.
    /// * `should_use_colors`: Whether to use colors in output.
    /// * `database`: The mutable database containing all source files.
    /// * `should_fail_from_baseline`: Whether the baseline processing indicated failure.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ExitCode` to indicate success or failure to the shell,
    /// or an `Error` if an unrecoverable problem occurs.
    pub fn process_issues_with_baseline_result(
        self,
        mut issues: IssueCollection,
        configuration: Configuration,
        should_use_colors: bool,
        database: Database,
        should_fail_from_baseline: bool,
    ) -> Result<ExitCode, Error> {
        if let Some(min_level) = self.minimum_report_level {
            let unfiltered_count = issues.len();
            issues = issues.with_minimum_level(min_level);

            tracing::debug!(
                "Filtered out {} issues below the minimum report level of {:?}.",
                unfiltered_count - issues.len(),
                min_level
            );
        }

        if self.fix {
            self.handle_fix_mode(issues, configuration, should_use_colors, database)
        } else {
            self.handle_report_mode(issues, &configuration, should_use_colors, database, should_fail_from_baseline)
        }
    }
}
