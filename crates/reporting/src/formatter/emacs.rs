use std::borrow::Cow;
use std::io::IsTerminal;
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
use crate::formatter::utils::osc8_file_hyperlink;

/// Formatter that outputs issues in Emacs compilation mode format.
pub(crate) struct EmacsFormatter;

fn escape_record_field(input: &str) -> Cow<'_, str> {
    if memchr::memchr2(b'\r', b'\n', input.as_bytes()).is_none() {
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

fn escaped_file_name(file: &File) -> Cow<'_, str> {
    match String::from_utf8_lossy(&file.name) {
        Cow::Borrowed(name) => escape_record_field(name),
        Cow::Owned(name) => Cow::Owned(escape_owned_record_field(name)),
    }
}

fn escape_owned_record_field(input: String) -> String {
    match escape_record_field(&input) {
        Cow::Borrowed(_) => input,
        Cow::Owned(escaped) => escaped,
    }
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
        let mut cached_file: Option<(&File, Cow<'_, str>)> = None;
        let mut number = itoa::Buffer::new();

        for issue in crate::formatter::utils::filter_issues(issues, config, false) {
            let (file_display, line, column) = match issue.primary_annotation() {
                Some(annotation) => {
                    let file_id = annotation.span.file_id();
                    let cached = match &mut cached_file {
                        Some(cached) if cached.0.id == file_id => cached,
                        cache => {
                            let file = database.get_ref(&file_id)?;
                            cache.insert((file, escaped_file_name(file)))
                        }
                    };
                    let (file, name) = (cached.0, &cached.1);
                    let line = file.line_number(annotation.span.start.offset);
                    let column = annotation.span.start.offset - file.lines[line as usize] + 1;
                    let line = line + 1;

                    let display = if let (Some(template), Some(path)) = (editor_url, file.path.as_ref()) {
                        let name = String::from_utf8_lossy(&file.name);
                        let display =
                            osc8_file_hyperlink(template, &path.display().to_string(), &name, line, column, &name);
                        Cow::Owned(escape_owned_record_field(display))
                    } else {
                        Cow::Borrowed(name.as_ref())
                    };

                    (display, line, column)
                }
                None => (Cow::Borrowed("<unknown>"), 0, 0),
            };

            let severity = match issue.level {
                Level::Error => "error",
                Level::Warning | Level::Note | Level::Help => "warning",
            };

            let message = escape_record_field(&issue.message);
            let issue_type = issue.code.as_deref().unwrap_or("other");
            let issue_type = escape_record_field(issue_type);

            writer.write_all(file_display.as_bytes())?;
            writer.write_all(b":")?;
            writer.write_all(number.format(line).as_bytes())?;
            writer.write_all(b":")?;
            writer.write_all(number.format(column).as_bytes())?;
            writer.write_all(b":")?;
            writer.write_all(severity.as_bytes())?;
            writer.write_all(b" - ")?;
            writer.write_all(issue_type.as_bytes())?;
            writer.write_all(b": ")?;
            writer.write_all(message.as_bytes())?;
            if let Some(link) = issue.link.as_deref() {
                writer.write_all(b" (see ")?;
                writer.write_all(escape_record_field(link).as_bytes())?;
                writer.write_all(b")")?;
            }

            writer.write_all(b"\n")?;
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
    use crate::IssueCollection;
    use crate::color::ColorChoice;
    use crate::formatter::Formatter;
    use crate::formatter::FormatterConfig;

    use super::EmacsFormatter;
    use super::escape_record_field;

    #[test]
    fn escapes_physical_line_boundaries() {
        assert_eq!(escape_record_field("plain"), "plain");
        assert_eq!(escape_record_field("first\r\nsecond"), "first\\r\\nsecond");
    }

    #[test]
    fn records_preserve_lossy_paths_links_and_line_boundaries() {
        let file = File::ephemeral(Cow::Borrowed(b"src/\xff\r\nevil.php"), Cow::Borrowed(b"<?php\n  foo();\n"));
        let file_id = file.id;
        let configuration = DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, configuration).read_only();
        let issues = IssueCollection::from([
            Issue::error("first\r\nsecond €")
                .with_code("code\r\nnext")
                .with_link("https://example.com/a\r\nb")
                .with_annotation(Annotation::primary(Span::new(file_id, 8u32.into(), 11u32.into()))),
            Issue::note("plain").with_link(""),
            Issue::warning("no link"),
        ]);

        let config = FormatterConfig {
            color_choice: ColorChoice::Never,
            sort: false,
            minimum_level: None,
            filter_fixable: false,
            editor_url: None,
        };

        let mut output = Vec::new();
        let Ok(()) = EmacsFormatter.format(&mut output, &issues, &database, &config) else {
            panic!("Emacs formatting should succeed");
        };

        assert_eq!(
            output,
            concat!(
                "src/�\\r\\nevil.php:2:3:error - code\\r\\nnext: first\\r\\nsecond € (see https://example.com/a\\r\\nb)\n",
                "<unknown>:0:0:warning - other: plain (see )\n",
                "<unknown>:0:0:warning - other: no link\n"
            )
            .as_bytes()
        );
    }

    #[test]
    fn unescaped_fields_borrow_the_input() {
        assert!(matches!(escape_record_field("plain €"), Cow::Borrowed("plain €")));
    }

    #[test]
    fn cached_names_follow_file_changes_and_keep_record_positions() {
        let configuration = DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let mut database = Database::new(configuration);
        let first = database.add(File::ephemeral(Cow::Borrowed(b"a\xff.php"), Cow::Borrowed(b"first\nsecond\n")));
        let second = database.add(File::ephemeral(Cow::Borrowed(b"b.php"), Cow::Borrowed(b"one\ntwo\n")));
        let issues =
            IssueCollection::from([(first, 0), (first, 8), (second, 5), (first, 7)].map(|(file_id, offset)| {
                Issue::error("message").with_annotation(Annotation::primary(Span::new(
                    file_id,
                    offset.into(),
                    (offset + 1).into(),
                )))
            }));

        let config = FormatterConfig {
            color_choice: ColorChoice::Never,
            sort: false,
            minimum_level: None,
            filter_fixable: false,
            editor_url: None,
        };

        let mut output = Vec::new();
        let Ok(()) = EmacsFormatter.format(&mut output, &issues, &database.read_only(), &config) else {
            panic!("Emacs formatting should succeed");
        };

        assert_eq!(
            output,
            concat!(
                "a�.php:1:1:error - other: message\n",
                "a�.php:2:3:error - other: message\n",
                "b.php:2:2:error - other: message\n",
                "a�.php:2:2:error - other: message\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn streaming_output_handles_short_writes_interruptions_and_errors() {
        struct ShortWriter {
            bytes: Vec<u8>,
            fail_at: Option<usize>,
            interrupt: bool,
        }

        impl std::io::Write for ShortWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if std::mem::take(&mut self.interrupt) {
                    return Err(std::io::ErrorKind::Interrupted.into());
                }

                self.interrupt = true;
                let remaining = self.fail_at.map_or(usize::MAX, |limit| limit - self.bytes.len());
                if remaining == 0 {
                    return Err(std::io::ErrorKind::BrokenPipe.into());
                }

                let written = bytes.len().min(3).min(remaining);
                self.bytes.extend_from_slice(&bytes[..written]);
                Ok(written)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let message = "long €\n".repeat(2048);
        let issues = IssueCollection::from([Issue::error(message.clone()), Issue::warning("short").with_link("")]);
        let expected = format!(
            "<unknown>:0:0:error - other: {}\n<unknown>:0:0:warning - other: short (see )\n",
            message.replace('\n', "\\n")
        );

        let configuration = DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::new(configuration).read_only();
        let config = FormatterConfig {
            color_choice: ColorChoice::Never,
            sort: false,
            minimum_level: None,
            filter_fixable: false,
            editor_url: None,
        };

        let mut writer = ShortWriter { bytes: Vec::new(), fail_at: None, interrupt: true };
        let Ok(()) = EmacsFormatter.format(&mut writer, &issues, &database, &config) else {
            panic!("Short writes and interruptions should be retried");
        };

        assert_eq!(writer.bytes, expected.as_bytes());

        let mut writer = ShortWriter { bytes: Vec::new(), fail_at: Some(41), interrupt: true };
        let Err(error) = EmacsFormatter.format(&mut writer, &issues, &database, &config) else {
            panic!("The write error should propagate");
        };

        assert!(error.is_broken_pipe());
        assert_eq!(writer.bytes, &expected.as_bytes()[..41]);

        let issues = IssueCollection::from([
            Issue::warning("first"),
            Issue::error("missing file").with_annotation(Annotation::primary(Span::dummy(0, 1))),
        ]);

        let mut output = Vec::new();
        assert!(matches!(
            EmacsFormatter.format(&mut output, &issues, &database, &config),
            Err(crate::error::ReportingError::DatabaseError(_))
        ));

        assert_eq!(output, b"<unknown>:0:0:warning - other: first\n");
    }
}
