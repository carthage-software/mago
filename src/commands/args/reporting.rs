//! Command-line arguments for issue reporting and fixing.
//!
//! This module defines [`ReportingArgs`], a reusable set of command-line arguments
//! for controlling how issues are reported and optionally fixed. These arguments
//! can be flattened into any command that needs to report analysis results.
//!
//! # Features
//!
//! The reporting arguments control several aspects:
//!
//! - **Output Formatting**: Choose from rich, medium, short, JSON, and other formats
//! - **Output Targeting**: Send output to stdout or stderr
//! - **Issue Filtering**: Filter by fixability, severity level
//! - **Issue Sorting**: Sort issues for better organization
//! - **Automatic Fixing**: Apply fixes with various safety levels
//! - **Fix Previewing**: Dry-run mode to preview fixes without applying them
//!
//! # Fix Safety Levels
//!
//! Fixes are categorized by safety:
//!
//! - **Safe**: Applied by default with `--fix`
//! - **Potentially Unsafe**: Requires `--potentially-unsafe` flag
//! - **Unsafe**: Requires `--unsafe` flag
//!
//! # Exit Codes
//!
//! The `minimum_fail_level` determines when the command exits with failure.
//! This enables CI integration where certain issue severities should fail builds.

use clap::ColorChoice;
use clap::Parser;
use mago_orchestrator::Orchestrator;

use mago_database::Database;
use mago_reporting::Level;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;

use crate::enum_variants;
use crate::service::IssueProcessor;

/// Command-line arguments for issue reporting and fixing.
///
/// This struct defines all options for controlling issue output and automatic
/// fix application. It's designed to be flattened into command structs using
/// `#[clap(flatten)]`.
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
}

impl ReportingArgs {
    /// Creates an issue processor from these reporting arguments.
    ///
    /// This method converts the command-line arguments into an [`IssueProcessor`]
    /// that will handle issue reporting and optionally apply fixes according to
    /// the configured options.
    ///
    /// # Arguments
    ///
    /// * `orchestrator` - The orchestrator for formatting fixed files
    /// * `database` - The database containing source files
    /// * `color_choice` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// An [`IssueProcessor`] configured with all the reporting and fixing options
    /// from this argument set.
    pub fn get_processor<'a>(
        self,
        orchestrator: Orchestrator<'a>,
        database: Database,
        color_choice: ColorChoice,
    ) -> IssueProcessor<'a> {
        IssueProcessor {
            orchestrator,
            database,
            fixable_only: self.fixable_only,
            sort: self.sort,
            fix: self.fix,
            r#unsafe: self.r#unsafe,
            potentially_unsafe: self.potentially_unsafe,
            format_after_fix: self.format_after_fix,
            dry_run: self.dry_run,
            reporting_target: self.reporting_target,
            reporting_format: self.reporting_format,
            minimum_fail_level: self.minimum_fail_level,
            minimum_report_level: self.minimum_report_level,
            color_choice,
        }
    }
}
