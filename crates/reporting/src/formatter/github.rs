use std::io::Write;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::formatter::utils::long_message;

/// Formatter that outputs issues in GitHub Actions workflow commands format.
pub(crate) struct GithubFormatter;

impl Formatter for GithubFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Apply filters
        let issues = apply_filters(issues, config);

        for issue in issues.iter() {
            let level = match &issue.level {
                Level::Note => "notice",
                Level::Help => "notice",
                Level::Warning => "warning",
                Level::Error => "error",
            };

            let properties = match issue.primary_annotation() {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id())?;
                    let start_line = file.line_number(annotation.span.start.offset) + 1;
                    let end_line = file.line_number(annotation.span.end.offset) + 1;
                    let start_col = file.column_number(annotation.span.start.offset) + 1;
                    let end_col = file.column_number(annotation.span.end.offset) + 1;

                    if let Some(code) = issue.code.as_ref() {
                        format!(
                            "file={},line={start_line},endLine={end_line},col={start_col},endColumn={end_col},title={code}",
                            file.name
                        )
                    } else {
                        format!(
                            "file={},line={start_line},endLine={end_line},col={start_col},endColumn={end_col}",
                            file.name
                        )
                    }
                }
                None => {
                    if let Some(code) = issue.code.as_ref() {
                        format!("title={code}")
                    } else {
                        String::new()
                    }
                }
            };

            // we must use `%0A` instead of `\n`.
            //
            // see: https://github.com/actions/toolkit/issues/193
            let message = long_message(issue, true).replace('\n', "%0A");

            writeln!(writer, "::{level} {properties}::{message}")?;
        }

        Ok(())
    }
}

fn apply_filters(issues: &IssueCollection, config: &FormatterConfig) -> IssueCollection {
    let mut filtered = issues.clone();

    if let Some(min_level) = config.minimum_level {
        filtered = filtered.with_minimum_level(min_level);
    }

    if config.filter_fixable {
        filtered = filtered.with_edits();
    }

    if config.sort {
        filtered = filtered.sorted();
    }

    filtered
}
