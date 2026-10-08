use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
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
        let mut issues_by_file: HashMap<String, Vec<String>> = HashMap::new();
        let mut cached_file: Option<(&File, Cow<'_, str>)> = None;

        for issue in crate::formatter::utils::filter_issues(issues, config, false) {
            let (filename, line, column) = match issue.primary_annotation() {
                Some(annotation) => {
                    let file_id = annotation.span.file_id();
                    let cached = match &mut cached_file {
                        Some(cached) if cached.0.id == file_id => cached,
                        cache => {
                            let file = database.get_ref(&file_id)?;
                            cache.insert((file, String::from_utf8_lossy(&file.name)))
                        }
                    };

                    let (file, name) = (cached.0, &cached.1);
                    let line = file.line_number(annotation.span.start.offset);
                    let column = annotation.span.start.offset - file.lines[line as usize] + 1;

                    (name.as_ref(), line + 1, column)
                }
                None => ("<unknown>", 0, 0),
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

            if let Some(errors) = issues_by_file.get_mut(filename) {
                errors.push(error_tag);
            } else {
                issues_by_file.entry(filename.to_owned()).or_default().push(error_tag);
            }
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
