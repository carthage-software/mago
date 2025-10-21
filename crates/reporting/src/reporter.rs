//! Issue reporter and output formatting.
//!
//! This module provides the core reporter functionality that formats and outputs
//! issues in various formats. It supports multiple output targets (stdout/stderr),
//! different formatting styles (rich, medium, short, JSON, etc.), and optional
//! pagination for terminal output.
//!
//! The reporter can filter issues based on baseline files and severity levels,
//! and can sort issues for better readability.

use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::VariantNames;

use mago_database::ReadDatabase;
use termcolor::ColorChoice;

use crate::IssueCollection;
use crate::Level;
use crate::baseline::Baseline;
use crate::error::ReportingError;
use crate::internal::emitter::Emitter;
use crate::internal::writer::ReportWriter;

/// Defines the output target for the reporter.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReportingTarget {
    /// Direct output to standard output (stdout).
    #[default]
    Stdout,
    /// Direct output to standard error (stderr).
    Stderr,
}

/// The format to use when writing the report.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, VariantNames)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ReportingFormat {
    #[default]
    Rich,
    Medium,
    Short,
    Ariadne,
    Github,
    Gitlab,
    Json,
    Count,
    CodeCount,
    Checkstyle,
    Emacs,
}

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
    pub fn new(database: ReadDatabase, config: ReporterConfig) -> Self {
        Self { database, config }
    }

    pub fn report(&self, issues: IssueCollection, baseline: Option<Baseline>) -> Result<ReportStatus, ReportingError> {
        let mut issues = issues;

        let mut baseline_has_dead_issues = false;
        let mut baseline_filtered_issues = 0;
        if let Some(baseline) = baseline {
            let original_count = issues.len();
            let filtered_issues = baseline.filter_issues(issues, &self.database);
            let comparison = baseline.compare_with_issues(&filtered_issues, &self.database);

            baseline_filtered_issues = original_count - filtered_issues.len();
            baseline_has_dead_issues = comparison.removed_issues_count > 0;
            issues = filtered_issues;
        }

        if let Some(min_level) = self.config.minimum_report_level {
            issues = issues.with_minimum_level(min_level);
        }

        if self.config.filter_fixable {
            issues = issues.filter_fixable();
        }

        if self.config.sort {
            issues = issues.sorted();
        }

        let total_reported_issues = issues.len();
        let highest_reported_level = issues.get_highest_level();
        let lowest_reported_level = issues.get_lowest_level();

        if total_reported_issues == 0 {
            return Ok(ReportStatus {
                baseline_dead_issues: baseline_has_dead_issues,
                baseline_filtered_issues,
                highest_reported_level: None,
                lowest_reported_level: None,
                total_reported_issues: 0,
            });
        }

        let writer = ReportWriter::new(self.config.target, self.config.color_choice);
        self.config.format.emit(&mut writer.lock(), &self.database, issues)?;

        Ok(ReportStatus {
            baseline_dead_issues: baseline_has_dead_issues,
            baseline_filtered_issues,
            highest_reported_level,
            lowest_reported_level,
            total_reported_issues,
        })
    }
}

impl FromStr for ReportingTarget {
    type Err = ReportingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdout" | "out" => Ok(Self::Stdout),
            "stderr" | "err" => Ok(Self::Stderr),
            _ => Err(ReportingError::InvalidTarget(s.to_string())),
        }
    }
}

impl FromStr for ReportingFormat {
    type Err = ReportingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rich" => Ok(Self::Rich),
            "medium" => Ok(Self::Medium),
            "short" => Ok(Self::Short),
            "ariadne" => Ok(Self::Ariadne),
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "json" => Ok(Self::Json),
            "count" => Ok(Self::Count),
            "codecode" | "code-count" => Ok(Self::CodeCount),
            "checkstyle" => Ok(Self::Checkstyle),
            "emacs" => Ok(Self::Emacs),
            _ => Err(ReportingError::InvalidFormat(s.to_string())),
        }
    }
}
