use std::borrow::Cow;
use std::io::IsTerminal;
use std::io::Write;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::formatter::utils::osc8_hyperlink;

/// Formatter that outputs issues in Emacs compilation mode format.
pub(crate) struct EmacsFormatter;

fn escape_record_field(input: &str) -> Cow<'_, str> {
    if !input.bytes().any(|byte| matches!(byte, b'\r' | b'\n')) {
        return Cow::Borrowed(input);
    }

    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            character => escaped.push(character),
        }
    }

    Cow::Owned(escaped)
}

impl Formatter for EmacsFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        let use_colors = config.color_choice.should_use_colors(std::io::stdout().is_terminal());
        let editor_url = if use_colors { config.editor_url.as_deref() } else { None };

        for issue in crate::formatter::utils::filter_issues(issues, config, false) {
            let (file_display, line, column) = match issue.primary_annotation() {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id())?;
                    let line = file.line_number(annotation.span.start.offset) + 1;
                    let column = file.column_number(annotation.span.start.offset) + 1;

                    let name = String::from_utf8_lossy(&file.name);
                    let display = if let (Some(template), Some(path)) = (editor_url, file.path.as_ref()) {
                        osc8_hyperlink(template, &path.display().to_string(), line, column, &name)
                    } else {
                        name.into_owned()
                    };

                    (escape_record_field(&display).into_owned(), line, column)
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
            let message = escape_record_field(&message);

            let issue_type = issue.code.as_deref().unwrap_or("other");
            let issue_type = escape_record_field(issue_type);

            writeln!(writer, "{file_display}:{line}:{column}:{severity} - {issue_type}: {message}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::escape_record_field;

    #[test]
    fn escapes_physical_line_boundaries() {
        assert_eq!(escape_record_field("plain"), "plain");
        assert_eq!(escape_record_field("first\r\nsecond"), "first\\r\\nsecond");
    }
}
