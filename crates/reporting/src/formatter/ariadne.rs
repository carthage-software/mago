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
        // Apply filters
        let issues = apply_filters(issues, config);

        // Determine if we should use colors
        let use_colors = config.color_choice.should_use_colors(atty::is(atty::Stream::Stdout));

        for issue in issues {
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

            let (file_path, range) = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id())?;

                    (
                        file.name.clone().into_owned(),
                        annotation.span.start.offset as usize..annotation.span.end.offset as usize,
                    )
                }
                None => ("<unknown>".to_owned(), 0..0),
            };

            let mut report = Report::build(kind, (file_path, range)).with_message(issue.message);

            if let Some(code) = issue.code {
                report = report.with_code(code);
            }

            for note in issue.notes {
                report = report.with_note(note);
            }

            if let Some(link) = issue.link {
                // Since ariadne doesn't support links, we can just set it as a note
                report = report.with_note(format!("For more information, see: {link}"));
            }

            if let Some(help) = issue.help {
                report = report.with_help(help);
            }

            let mut relevant_sources = vec![];
            for annotation in issue.annotations {
                let file = database.get(&annotation.span.file_id())?;
                let range = annotation.span.start.offset as usize..annotation.span.end.offset as usize;

                let mut label = Label::new((file.name.clone().into_owned(), range));
                if annotation.is_primary() {
                    label = label.with_color(color).with_priority(1);
                }

                if let Some(message) = annotation.message {
                    report = report.with_label(label.with_message(message));
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
