use std::io::IsTerminal;
use std::io::Write;

use ariadne::Color;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::sources as ariadne_sources;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issues using the Ariadne diagnostic library.
pub(crate) struct AriadneFormatter;

impl Formatter for AriadneFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        let use_colors = config.color_choice.should_use_colors(std::io::stdout().is_terminal());

        for issue in crate::formatter::utils::filter_issues(issues, config, true) {
            let kind = match issue.level {
                Level::Help | Level::Note => ReportKind::Advice,
                Level::Warning => ReportKind::Warning,
                Level::Error => ReportKind::Error,
            };

            let color = match issue.level {
                Level::Help | Level::Note => Color::Blue,
                Level::Warning => Color::Yellow,
                Level::Error => Color::Red,
            };

            let (file_path, range) = match issue.primary_annotation() {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id())?;

                    (
                        file.name.clone().into_owned(),
                        annotation.span.start.offset as usize..annotation.span.end.offset as usize,
                    )
                }
                None => ("<unknown>".to_owned(), 0..0),
            };

            let mut report = Report::build(kind, (file_path, range)).with_message(issue.message.clone());

            if let Some(code) = &issue.code {
                report = report.with_code(code.clone());
            }

            for note in &issue.notes {
                report = report.with_note(note.clone());
            }

            if let Some(link) = &issue.link {
                report = report.with_note(format!("For more information, see: {link}"));
            }

            if let Some(help) = &issue.help {
                report = report.with_help(help.clone());
            }

            let mut relevant_sources = vec![];
            for annotation in &issue.annotations {
                let file = database.get(&annotation.span.file_id())?;
                let range = annotation.span.start.offset as usize..annotation.span.end.offset as usize;

                let mut label = Label::new((file.name.clone().into_owned(), range));
                if annotation.is_primary() {
                    label = label.with_color(color).with_priority(1);
                }

                if let Some(message) = &annotation.message {
                    report = report.with_label(label.with_message(message.clone()));
                } else {
                    report = report.with_label(label);
                }

                relevant_sources.push((file.name.clone().into_owned(), file.contents.to_string()));
            }

            let config =
                if use_colors { ariadne::Config::default() } else { ariadne::Config::default().with_color(false) };

            // Ariadne's write methods consume the writer, so we write to a buffer first
            let mut buffer = Vec::new();
            report
                .with_config(config)
                .finish()
                .write_for_stdout(ariadne_sources(relevant_sources), &mut buffer)
                .map_err(std::io::Error::other)?;
            writer.write_all(&buffer)?;
        }

        Ok(())
    }
}
