use std::borrow::Cow;
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

/// Escape a GitHub workflow-command message.
///
/// This mirrors `@actions/core`'s `escapeData`: `%` must be escaped before the
/// encoded forms are introduced, while carriage returns and line feeds must
/// never reach the runner as physical line boundaries.
fn escape_data(input: &str) -> Cow<'_, str> {
    if !input.bytes().any(|byte| matches!(byte, b'%' | b'\r' | b'\n')) {
        return Cow::Borrowed(input);
    }

    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '\r' => escaped.push_str("%0D"),
            '\n' => escaped.push_str("%0A"),
            character => escaped.push(character),
        }
    }

    Cow::Owned(escaped)
}

/// Escape a GitHub workflow-command property value.
///
/// Properties use comma and colon as delimiters in addition to sharing the
/// message escaping rules, so those characters must also be encoded.
fn escape_property(input: &str) -> Cow<'_, str> {
    if !input.bytes().any(|byte| matches!(byte, b'%' | b'\r' | b'\n' | b':' | b',')) {
        return Cow::Borrowed(input);
    }

    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '\r' => escaped.push_str("%0D"),
            '\n' => escaped.push_str("%0A"),
            ':' => escaped.push_str("%3A"),
            ',' => escaped.push_str("%2C"),
            character => escaped.push(character),
        }
    }

    Cow::Owned(escaped)
}

impl Formatter for GithubFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        for issue in crate::formatter::utils::filter_issues(issues, config, false) {
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

                    let name = String::from_utf8_lossy(&file.name);
                    let name = escape_property(&name);
                    if let Some(code) = issue.code.as_ref() {
                        let code = escape_property(code);
                        format!(
                            "file={name},line={start_line},endLine={end_line},col={start_col},endColumn={end_col},title={code}"
                        )
                    } else {
                        format!("file={name},line={start_line},endLine={end_line},col={start_col},endColumn={end_col}")
                    }
                }
                None => {
                    if let Some(code) = issue.code.as_ref() {
                        let code = escape_property(code);
                        format!("title={code}")
                    } else {
                        String::new()
                    }
                }
            };

            let message = long_message(issue, true);
            let message = escape_data(&message);

            if properties.is_empty() {
                writeln!(writer, "::{level}::{message}")?;
            } else {
                writeln!(writer, "::{level} {properties}::{message}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::path::Path;

    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;
    use mago_span::Span;

    use crate::Annotation;
    use crate::Issue;
    use crate::color::ColorChoice;

    use super::*;

    fn config() -> FormatterConfig {
        FormatterConfig {
            color_choice: ColorChoice::Never,
            sort: false,
            minimum_level: None,
            filter_fixable: false,
            editor_url: None,
        }
    }

    #[test]
    fn escapes_workflow_command_data() {
        assert_eq!(escape_data("plain"), "plain");
        assert_eq!(escape_data("100%\r\nnext"), "100%25%0D%0Anext");
    }

    #[test]
    fn escapes_workflow_command_properties() {
        assert_eq!(escape_property("plain"), "plain");
        assert_eq!(escape_property("a:b,c%\r\nnext"), "a%3Ab%2Cc%25%0D%0Anext");
    }

    #[test]
    fn hostile_values_cannot_create_additional_workflow_commands() {
        let file = File::ephemeral(Cow::Borrowed(b"src/%\r\n::notice::injected,evil.php"), Cow::Borrowed(b"<?php\n"));
        let file_id = file.id;
        let configuration = DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, configuration).read_only();
        let issue = Issue::error("message%0A\r\n::warning::injected").with_code("analysis:%\r\n,code").with_annotation(
            Annotation::primary(Span::new(file_id, 0u32.into(), 1u32.into()))
                .with_message("annotation\r\n::error::injected"),
        );

        let mut output = Vec::new();
        let Ok(()) = GithubFormatter.format(&mut output, &IssueCollection::from([issue]), &database, &config()) else {
            panic!("GitHub formatting should succeed");
        };

        let Ok(output) = String::from_utf8(output) else {
            panic!("GitHub output should be UTF-8");
        };
        assert_eq!(output.bytes().filter(|byte| *byte == b'\n').count(), 1);
        assert!(!output.contains('\r'));
        assert!(output.contains("file=src/%25%0D%0A%3A%3Anotice%3A%3Ainjected%2Cevil.php"));
        assert!(output.contains("title=analysis%3A%25%0D%0A%2Ccode"));
        assert!(output.contains("message%250A%0D%0A::warning::injected"));
        assert!(output.contains("%0A>annotation%0D%0A::error::injected"));
    }

    #[test]
    fn omits_property_separator_when_no_properties_exist() {
        let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Borrowed(b"<?php\n"));
        let configuration = DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, configuration).read_only();
        let mut output = Vec::new();

        let Ok(()) = GithubFormatter.format(
            &mut output,
            &IssueCollection::from([Issue::error("message")]),
            &database,
            &config(),
        ) else {
            panic!("GitHub formatting should succeed");
        };

        assert_eq!(output, b"::error::message\n");
    }
}
