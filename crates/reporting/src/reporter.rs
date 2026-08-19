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

    /// Optional editor URL template for OSC 8 terminal hyperlinks on file paths.
    ///
    /// Supported placeholders: `%file%` (absolute path), `%line%`, `%column%`.
    /// Example: `"phpstorm://open?file=%file%&line=%line%"`
    pub editor_url: Option<String>,
}

/// Status information returned after reporting issues.
///
/// This struct provides detailed statistics about the reporting operation,
/// including baseline filtering results and severity level information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportStatus {
    /// The number of baseline entries that no longer correspond to any current
    /// issue (i.e. dead / stale entries). Zero when the baseline is up to date.
    pub baseline_dead_issues: usize,

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
    pub fn report(
        &self,
        mut issues: IssueCollection,
        baseline: Option<Baseline>,
    ) -> Result<ReportStatus, ReportingError> {
        let mut writer = self.config.target.resolve();

        // Apply baseline filtering
        let mut baseline_dead_issues = 0;
        let mut baseline_filtered_issues = 0;
        if let Some(baseline) = baseline {
            let original_count = issues.len();
            let comparison = baseline.compare_with_issues(&issues, &self.database);
            let filtered_issues = baseline.filter_issues(issues, &self.database);

            baseline_filtered_issues = original_count - filtered_issues.len();
            baseline_dead_issues = comparison.removed_issues.len();
            issues = filtered_issues;
        }

        // Track reported issue stats before formatting
        let total_reported_issues = issues.len();
        let highest_reported_level = issues.get_highest_level();
        let lowest_reported_level = issues.get_lowest_level();

        // Early return if no issues to report
        if total_reported_issues == 0 && !self.config.format.requires_output_when_empty() {
            return Ok(ReportStatus {
                baseline_dead_issues,
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
            editor_url: self.config.editor_url.clone(),
        };

        // Dispatch to the appropriate formatter
        dispatch_format(self.config.format, &mut *writer, &issues, &self.database, &formatter_config)?;
        // When writing to pipes, some formatters do not flush the last line of json
        writer.flush()?;

        Ok(ReportStatus {
            baseline_dead_issues,
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
    pub fn report_to<W>(
        &self,
        mut issues: IssueCollection,
        baseline: Option<Baseline>,
        writer: &mut W,
    ) -> Result<ReportStatus, ReportingError>
    where
        W: Write,
    {
        // Apply baseline filtering
        let mut baseline_dead_issues = 0;
        let mut baseline_filtered_issues = 0;
        if let Some(baseline) = baseline {
            let original_count = issues.len();
            let comparison = baseline.compare_with_issues(&issues, &self.database);
            let filtered_issues = baseline.filter_issues(issues, &self.database);

            baseline_filtered_issues = original_count - filtered_issues.len();
            baseline_dead_issues = comparison.removed_issues.len();
            issues = filtered_issues;
        }

        // Track reported issue stats before formatting
        let total_reported_issues = issues.len();
        let highest_reported_level = issues.get_highest_level();
        let lowest_reported_level = issues.get_lowest_level();

        // Early return if no issues to report
        if total_reported_issues == 0 && !self.config.format.requires_output_when_empty() {
            return Ok(ReportStatus {
                baseline_dead_issues,
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
            editor_url: self.config.editor_url.clone(),
        };

        // Dispatch to the appropriate formatter
        dispatch_format(self.config.format, writer, &issues, &self.database, &formatter_config)?;

        Ok(ReportStatus {
            baseline_dead_issues,
            baseline_filtered_issues,
            highest_reported_level,
            lowest_reported_level,
            total_reported_issues,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;
    use mago_span::Span;

    use crate::Annotation;
    use crate::Issue;

    use super::*;

    fn reporter_for_file(format: ReportingFormat, file: File) -> Reporter {
        let configuration =
            DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, configuration).read_only();

        Reporter::new(
            database,
            ReporterConfig {
                target: ReportingTarget::Stdout,
                format,
                color_choice: ColorChoice::Never,
                filter_fixable: false,
                sort: false,
                minimum_report_level: None,
                editor_url: None,
            },
        )
    }

    fn reporter_for(format: ReportingFormat) -> Reporter {
        reporter_for_file(format, File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Borrowed(b"<?php\n")))
    }

    fn report_hostile_values(format: ReportingFormat) -> Vec<u8> {
        let file = File::ephemeral(Cow::Borrowed(b"src/<evil>\r\nname.php"), Cow::Borrowed(b"<?php\n"));
        let file_id = file.id;
        let reporter = reporter_for_file(format, file);
        let issue = Issue::error("message\0\u{1}\r\n</error>")
            .with_code("code\r\nvalue")
            .with_annotation(Annotation::primary(Span::new(file_id, 0u32.into(), 1u32.into())));
        let mut buffer = Vec::new();

        let Ok(_) = reporter.report_to(IssueCollection::from([issue]), None, &mut buffer) else {
            panic!("reporting hostile values should succeed");
        };

        buffer
    }

    fn report_invalid_utf8_source(format: ReportingFormat) -> Vec<u8> {
        let source = b"<?php\nclass \xa9 {}\n";
        let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Borrowed(source));
        let file_id = file.id;
        let reporter = reporter_for_file(format, file);
        let Some(invalid_offset) = source.iter().position(|byte| *byte == 0xa9) else {
            panic!("test source should contain invalid UTF-8");
        };
        let invalid_offset = invalid_offset as u32;
        let issue = Issue::error("invalid class name").with_annotation(Annotation::primary(Span::new(
            file_id,
            invalid_offset.into(),
            (invalid_offset + 1).into(),
        )));
        let mut buffer = Vec::new();

        let Ok(_) = reporter.report_to(IssueCollection::from([issue]), None, &mut buffer) else {
            panic!("reporting invalid UTF-8 source should succeed");
        };

        buffer
    }

    fn report_empty(format: ReportingFormat) -> Vec<u8> {
        let reporter = reporter_for(format);
        let mut buffer = Vec::new();

        let Ok(status) = reporter.report_to(IssueCollection::new(), None, &mut buffer) else {
            panic!("reporting an empty collection should succeed");
        };

        assert_eq!(status.total_reported_issues, 0);

        buffer
    }

    #[test]
    fn human_formats_write_nothing_when_empty() {
        assert!(report_empty(ReportingFormat::Rich).is_empty());
        assert!(report_empty(ReportingFormat::Medium).is_empty());
        assert!(report_empty(ReportingFormat::Short).is_empty());
    }

    #[test]
    fn source_rendering_handles_invalid_utf8() {
        for format in [ReportingFormat::Rich, ReportingFormat::Medium, ReportingFormat::Short, ReportingFormat::Ariadne]
        {
            let Ok(output) = String::from_utf8(report_invalid_utf8_source(format)) else {
                panic!("human-readable report should be UTF-8");
            };

            assert!(output.contains("invalid class name"));
        }
    }

    #[test]
    fn checkstyle_writes_document_when_empty() {
        let output = String::from_utf8_lossy(&report_empty(ReportingFormat::Checkstyle)).into_owned();

        assert!(output.contains("<?xml"));
        assert!(output.contains("<checkstyle>"));
        assert!(output.contains("</checkstyle>"));
    }

    #[test]
    fn checkstyle_escapes_hostile_values_as_valid_xml_text() {
        let Ok(output) = String::from_utf8(report_hostile_values(ReportingFormat::Checkstyle)) else {
            panic!("Checkstyle output should be UTF-8");
        };

        assert!(!output.contains('\0'));
        assert!(!output.contains('\u{1}'));
        assert!(output.contains("name=\"src/&lt;evil&gt;&#13;&#10;name.php\""));
        assert!(output.contains("message=\"message&#13;&#10;&lt;/error&gt;\""));
    }

    #[test]
    fn emacs_escapes_hostile_values_to_one_record() {
        let Ok(output) = String::from_utf8(report_hostile_values(ReportingFormat::Emacs)) else {
            panic!("Emacs output should be UTF-8");
        };

        assert_eq!(output.bytes().filter(|byte| *byte == b'\n').count(), 1);
        assert!(!output.contains('\r'));
        assert!(output.contains(r"src/<evil>\r\nname.php"));
        assert!(output.contains(r"code\r\nvalue"));
        assert!(output.contains("message\0\u{1}\\r\\n</error>"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn sarif_writes_document_when_empty() {
        let output = report_empty(ReportingFormat::Sarif);

        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output) else {
            panic!("empty SARIF report should be valid JSON");
        };

        assert_eq!(value["version"], "2.1.0");
        assert!(value["runs"][0]["results"].as_array().is_some_and(Vec::is_empty));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn sarif_serializes_hostile_values_as_json_data() {
        let output = report_hostile_values(ReportingFormat::Sarif);
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output) else {
            panic!("SARIF output should be valid JSON");
        };

        assert_eq!(value["runs"][0]["results"][0]["message"]["text"], "message\0\u{1}\r\n</error>");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_writes_document_when_empty() {
        let output = report_empty(ReportingFormat::Json);

        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output) else {
            panic!("empty JSON report should be valid JSON");
        };

        assert!(value["issues"].as_array().is_some_and(Vec::is_empty));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_serializes_hostile_values_as_data() {
        let output = report_hostile_values(ReportingFormat::Json);
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output) else {
            panic!("JSON report should be valid JSON");
        };

        assert_eq!(value["issues"][0]["message"], "message\0\u{1}\r\n</error>");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn gitlab_writes_empty_list_when_empty() {
        assert_eq!(report_empty(ReportingFormat::Gitlab), b"[]");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn gitlab_serializes_hostile_values_as_json_data() {
        let output = report_hostile_values(ReportingFormat::Gitlab);
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output) else {
            panic!("GitLab report should be valid JSON");
        };

        assert_eq!(value[0]["description"], "message\0\u{1}\r\n</error>");
    }
}
