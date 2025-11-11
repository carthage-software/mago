use std::io::Write;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issues in Emacs compilation mode format.
pub(crate) struct EmacsFormatter;

impl Formatter for EmacsFormatter {
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
            let (file_path, line, column) = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id())?;
                    let line = file.line_number(annotation.span.start.offset) + 1;
                    let column = file.column_number(annotation.span.start.offset) + 1;

                    (file.name.to_string(), line, column)
                }
                None => ("<unknown>".to_string(), 0, 0),
            };

            let severity = match issue.level {
                Level::Error => "error",
                Level::Warning | Level::Note | Level::Help => "warning",
            };

            let mut message = issue.message.clone();
            if let Some(link) = issue.link.as_deref() {
                message.push_str(" (see ");
                message.push_str(link);
                message.push(')');
            }

            let issue_type = issue.code.as_deref().unwrap_or("other");

            writeln!(writer, "{file_path}:{line}:{column}:{severity} - {issue_type}: {message}")?;
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
        filtered = filtered.filter_fixable();
    }

    if config.sort {
        filtered = filtered.sorted();
    }

    filtered
}
