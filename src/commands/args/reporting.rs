use std::borrow::Cow;
use std::path::PathBuf;
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

use crate::baseline;
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
    #[arg(
        long,
        short = 'd',
        help = "Preview fixes without applying them (requires --fix)",
        requires = "fix",
        alias = "diff"
    )]
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

    /// Generate a baseline file to ignore existing issues.
    #[arg(
        long,
        help = "Generate a baseline file to ignore existing issues",
        conflicts_with = "fix",
        requires = "baseline"
    )]
    pub generate_baseline: bool,

    /// Create a backup of the old baseline file (`.bkp`) when generating a new one.
    #[arg(long, help = "Backup the old baseline file when generating a new one", requires = "generate_baseline")]
    pub backup_baseline: bool,

    /// Specify a baseline file to ignore issues listed within it.
    #[arg(long, help = "Specify a baseline file to ignore issues", value_name = "PATH", conflicts_with = "fix")]
    pub baseline: Option<PathBuf>,

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
    /// * `database`: The mutable database containing all source files.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ExitCode` to indicate success or failure to the shell,
    /// or an `Error` if an unrecoverable problem occurs.
    pub fn process_issues(
        self,
        issues: IssueCollection,
        configuration: Configuration,
        should_use_colors: bool,
        database: Database,
    ) -> Result<ExitCode, Error> {
        if self.fix {
            self.handle_fix_mode(issues, configuration, should_use_colors, database)
        } else {
            self.handle_report_mode(issues, &configuration, should_use_colors, database)
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
    ) -> Result<ExitCode, Error> {
        let read_database = database.read_only();

        if self.sort {
            issues = issues.sorted();
        }

        if let Some(baseline_path) = &self.baseline {
            if self.generate_baseline {
                tracing::info!("Generating baseline file...");
                let baseline = baseline::generate_baseline_from_issues(issues, &read_database)?;
                baseline::serialize_baseline(baseline_path, &baseline, self.backup_baseline)?;
                tracing::info!("Baseline file successfully generated at `{}`.", baseline_path.display());

                return Ok(ExitCode::SUCCESS);
            }

            if !baseline_path.exists() {
                tracing::warn!(
                    "Baseline file `{}` does not exist. Issues will not be filtered.",
                    baseline_path.display()
                );
            } else {
                let baseline = baseline::unserialize_baseline(baseline_path)?;
                let (filtered_issues, filtered_out_count, has_dead_issues) =
                    baseline::filter_issues(&baseline, issues, &read_database)?;

                if has_dead_issues {
                    tracing::warn!(
                        "Your baseline file contains entries for issues that no longer exist. Consider regenerating it with `--generate-baseline`."
                    );
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

        let has_issues_above_threshold = issues.has_minimum_level(self.minimum_fail_level);
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

        Ok(if has_issues_above_threshold { ExitCode::FAILURE } else { ExitCode::SUCCESS })
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
}
