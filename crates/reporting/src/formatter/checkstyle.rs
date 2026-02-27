use std::collections::HashMap;
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
use crate::formatter::utils::xml_encode;

/// Formatter that outputs issues in Checkstyle XML format.
pub(crate) struct CheckstyleFormatter;

impl Formatter for CheckstyleFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        // Apply filters
        let issues = apply_filters(issues, config);

        // Group issues by file
        let mut issues_by_file: HashMap<String, Vec<String>> = HashMap::new();

        for issue in issues.iter() {
            let (filename, line, column) = match issue.primary_annotation() {
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
                Level::Warning => "warning",
                Level::Help | Level::Note => "info",
            };

            let message = xml_encode(long_message(issue, true));
            let error_tag = format!(
                "    <error line=\"{line}\" column=\"{column}\" severity=\"{severity}\" message=\"{message}\" />"
            );

            issues_by_file.entry(filename).or_default().push(error_tag);
        }

        // Begin Checkstyle XML
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(writer, "<checkstyle>")?;

        // Write grouped issues
        for (filename, errors) in issues_by_file {
            writeln!(writer, "  <file name=\"{}\">", xml_encode(&filename))?;
            for error in errors {
                writeln!(writer, "{error}")?;
            }

            writeln!(writer, "  </file>")?;
        }

        // Close Checkstyle XML
        writeln!(writer, "</checkstyle>")?;

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
