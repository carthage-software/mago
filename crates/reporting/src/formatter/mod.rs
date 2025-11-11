use std::io::Write;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::Level;
use crate::color::ColorChoice;
use crate::error::ReportingError;

pub mod ariadne;
pub mod checkstyle;
pub mod code_count;
pub mod count;
pub mod emacs;
pub mod github;
pub mod gitlab;
pub mod json;
pub mod medium;
pub mod rich;
pub mod short;
pub mod utils;

/// Configuration for formatters.
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Choice for colorizing output.
    pub color_choice: ColorChoice,
    /// Whether to sort issues before formatting.
    pub sort: bool,
    /// Minimum report level (filter out lower severity issues).
    pub minimum_level: Option<Level>,
    /// Whether to filter to only fixable issues.
    pub filter_fixable: bool,
}

/// Trait for formatting issues to a writer.
pub trait Formatter {
    /// Format issues and write them to the provided writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to output formatted issues to
    /// * `issues` - The collection of issues to format
    /// * `database` - The read database for accessing source files
    /// * `config` - Configuration for formatting behavior
    ///
    /// # Errors
    ///
    /// Returns an error if formatting or writing fails.
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError>;
}

/// The format to use for reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::VariantNames, Default)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportingFormat {
    /// Rich diagnostic format with full context.
    #[default]
    Rich,
    /// Medium diagnostic format with balanced context.
    Medium,
    /// Short diagnostic format with minimal context.
    Short,
    /// Ariadne diagnostic format.
    Ariadne,
    /// GitHub Actions format.
    Github,
    /// GitLab Code Quality format.
    Gitlab,
    /// JSON format.
    Json,
    /// Issue count by severity.
    Count,
    /// Issue count by code.
    CodeCount,
    /// Checkstyle XML format.
    Checkstyle,
    /// Emacs compilation mode format.
    Emacs,
}

/// Dispatch to the appropriate formatter based on the format type.
///
/// This function performs static dispatch using enum matching for optimal performance.
pub(crate) fn dispatch_format(
    format: ReportingFormat,
    writer: &mut dyn Write,
    issues: &IssueCollection,
    database: &ReadDatabase,
    config: &FormatterConfig,
) -> Result<(), ReportingError> {
    match format {
        ReportingFormat::Rich => rich::RichFormatter.format(writer, issues, database, config),
        ReportingFormat::Medium => medium::MediumFormatter.format(writer, issues, database, config),
        ReportingFormat::Short => short::ShortFormatter.format(writer, issues, database, config),
        ReportingFormat::Ariadne => ariadne::AriadneFormatter.format(writer, issues, database, config),
        ReportingFormat::Json => json::JsonFormatter.format(writer, issues, database, config),
        ReportingFormat::Github => github::GithubFormatter.format(writer, issues, database, config),
        ReportingFormat::Gitlab => gitlab::GitlabFormatter.format(writer, issues, database, config),
        ReportingFormat::Checkstyle => checkstyle::CheckstyleFormatter.format(writer, issues, database, config),
        ReportingFormat::Emacs => emacs::EmacsFormatter.format(writer, issues, database, config),
        ReportingFormat::Count => count::CountFormatter.format(writer, issues, database, config),
        ReportingFormat::CodeCount => code_count::CodeCountFormatter.format(writer, issues, database, config),
    }
}
