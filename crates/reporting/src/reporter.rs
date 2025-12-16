//! Issue reporter and output formatting.
//!
//! This module provides the core reporter functionality that formats and outputs
//! issues in various formats. It supports multiple output targets (stdout/stderr),
//! different formatting styles (rich, medium, short, JSON, etc.), and optional
//! pagination for terminal output.
//!
//! The reporter can filter issues based on baseline files and severity levels,
//! and can sort issues for better readability.

use std::io::Write;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::baseline::Baseline;
use crate::color::ColorChoice;
use crate::error::ReportingError;
use crate::formatter::FormatterConfig;
use crate::formatter::ReportingFormat;
use crate::formatter::dispatch_format;
use crate::output::ReportingTarget;

/// Configuration options for the reporter.
///
/// This struct controls how issues are formatted and displayed, including
/// the output target, format style, color usage, and filtering options.
#[derive(Debug)]
pub struct ReporterConfig {
    /// The target where the report will be sent.
    pub target: ReportingTarget,

    /// The format to use for the report output.
    pub format: ReportingFormat,

    /// Color choice for the report output.
    pub color_choice: ColorChoice,

    /// Filter the output to only show issues that can be automatically fixed.
    ///
    /// When enabled, only issues that have available automatic fixes will be displayed.
    /// This is useful when you want to focus on issues that can be resolved immediately.
    pub filter_fixable: bool,

    /// Sort reported issues by severity level, rule code, and file location.
    ///
    /// By default, issues are reported in the order they appear in files.
    /// This option provides a more organized view for reviewing large numbers of issues.
    pub sort: bool,

    /// the minimum issue severity to be shown in the report.
    ///
    /// Issues below this level will be completely ignored and not displayed.
    pub minimum_report_level: Option<Level>,
}

/// Status information returned after reporting issues.
///
/// This struct provides detailed statistics about the reporting operation,
/// including baseline filtering results and severity level information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportStatus {
    /// Indicates whether the baseline contains dead issues.
    pub baseline_dead_issues: bool,

    /// The number of issues that were filtered out by the baseline.
    pub baseline_filtered_issues: usize,

    /// The highest severity level among the reported issues.
    pub highest_reported_level: Option<Level>,

    /// The lowest severity level among the reported issues.
    pub lowest_reported_level: Option<Level>,

    /// The total number of issues reported.
    pub total_reported_issues: usize,
}

/// The main reporter that handles formatting and outputting issues.
///
/// The reporter takes a collection of issues and outputs them according to
/// the configured format and options. It can apply baseline filtering,
/// severity filtering, and sorting before output.
#[derive(Debug)]
pub struct Reporter {
    database: ReadDatabase,
    config: ReporterConfig,
}

impl Reporter {
    /// Create a new reporter with the given database and configuration.
    #[must_use]
    pub fn new(database: ReadDatabase, config: ReporterConfig) -> Self {
        Self { database, config }
    }

    /// Report issues to the configured target.
    ///
    /// This method applies baseline filtering, severity filtering, and sorting
    /// based on the reporter configuration, then formats and outputs the issues.
    ///
    /// # Errors
    ///
    /// Returns a [`ReportingError`] if formatting or writing the issues fails.
    pub fn report(&self, issues: IssueCollection, baseline: Option<Baseline>) -> Result<ReportStatus, ReportingError> {
        let mut writer = self.config.target.resolve();

        let mut issues = issues;

        // Apply baseline filtering
        let mut baseline_has_dead_issues = false;
        let mut baseline_filtered_issues = 0;
        if let Some(baseline) = baseline {
            let original_count = issues.len();
            let comparison = baseline.compare_with_issues(&issues, &self.database);
            let filtered_issues = baseline.filter_issues(issues, &self.database);

            baseline_filtered_issues = original_count - filtered_issues.len();
            baseline_has_dead_issues = comparison.removed_issues_count > 0;
            issues = filtered_issues;
        }

        // Track reported issue stats before formatting
        let total_reported_issues = issues.len();
        let highest_reported_level = issues.get_highest_level();
        let lowest_reported_level = issues.get_lowest_level();

        // Early return if no issues to report
        if total_reported_issues == 0 {
            return Ok(ReportStatus {
                baseline_dead_issues: baseline_has_dead_issues,
                baseline_filtered_issues,
                highest_reported_level: None,
                lowest_reported_level: None,
                total_reported_issues: 0,
            });
        }

        // Build formatter config
        let formatter_config = FormatterConfig {
            color_choice: self.config.color_choice,
            sort: self.config.sort,
            minimum_level: self.config.minimum_report_level,
            filter_fixable: self.config.filter_fixable,
        };

        // Dispatch to the appropriate formatter
        dispatch_format(self.config.format, &mut *writer, &issues, &self.database, &formatter_config)?;

        Ok(ReportStatus {
            baseline_dead_issues: baseline_has_dead_issues,
            baseline_filtered_issues,
            highest_reported_level,
            lowest_reported_level,
            total_reported_issues,
        })
    }

    /// Report issues to a custom writer.
    ///
    /// This method allows writing to any `Write` implementation, making it useful
    /// for testing, capturing output to strings, writing to files, or streaming
    /// over network sockets.
    ///
    /// # Errors
    ///
    /// Returns a [`ReportingError`] if formatting or writing the issues fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Write to a buffer for testing
    /// let mut buffer = Vec::new();
    /// reporter.report_to(issues, None, &mut buffer)?;
    /// let output = String::from_utf8(buffer)?;
    ///
    /// // Write to a file
    /// let mut file = File::create("report.txt")?;
    /// reporter.report_to(issues, None, &mut file)?;
    /// ```
    pub fn report_to<W: Write>(
        &self,
        issues: IssueCollection,
        baseline: Option<Baseline>,
        writer: &mut W,
    ) -> Result<ReportStatus, ReportingError> {
        let mut issues = issues;

        // Apply baseline filtering
        let mut baseline_has_dead_issues = false;
        let mut baseline_filtered_issues = 0;
        if let Some(baseline) = baseline {
            let original_count = issues.len();
            let comparison = baseline.compare_with_issues(&issues, &self.database);
            let filtered_issues = baseline.filter_issues(issues, &self.database);

            baseline_filtered_issues = original_count - filtered_issues.len();
            baseline_has_dead_issues = comparison.removed_issues_count > 0;
            issues = filtered_issues;
        }

        // Track reported issue stats before formatting
        let total_reported_issues = issues.len();
        let highest_reported_level = issues.get_highest_level();
        let lowest_reported_level = issues.get_lowest_level();

        // Early return if no issues to report
        if total_reported_issues == 0 {
            return Ok(ReportStatus {
                baseline_dead_issues: baseline_has_dead_issues,
                baseline_filtered_issues,
                highest_reported_level: None,
                lowest_reported_level: None,
                total_reported_issues: 0,
            });
        }

        // Build formatter config
        let formatter_config = FormatterConfig {
            color_choice: self.config.color_choice,
            sort: self.config.sort,
            minimum_level: self.config.minimum_report_level,
            filter_fixable: self.config.filter_fixable,
        };

        // Dispatch to the appropriate formatter
        dispatch_format(self.config.format, writer, &issues, &self.database, &formatter_config)?;

        Ok(ReportStatus {
            baseline_dead_issues: baseline_has_dead_issues,
            baseline_filtered_issues,
            highest_reported_level,
            lowest_reported_level,
            total_reported_issues,
        })
    }
}
