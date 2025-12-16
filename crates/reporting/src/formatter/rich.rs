use std::cmp::Ordering;
use std::io::Write;
use std::ops::Range;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::diagnostic::LabelStyle;
use codespan_reporting::diagnostic::Severity;
use codespan_reporting::files::Error;
use codespan_reporting::files::Files;
use codespan_reporting::term;
use codespan_reporting::term::Config;
use codespan_reporting::term::DisplayStyle;
use mago_database::file::FileId;
use termcolor::Buffer;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;

use crate::Annotation;
use crate::AnnotationKind;
use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issues in rich diagnostic format with full context.
pub(crate) struct RichFormatter;

impl Formatter for RichFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        codespan_format_with_config(
            writer,
            issues,
            database,
            config,
            &Config { display_style: DisplayStyle::Rich, ..Default::default() },
        )
    }
}

pub(super) fn codespan_format_with_config(
    writer: &mut dyn Write,
    issues: &IssueCollection,
    database: &ReadDatabase,
    config: &FormatterConfig,
    codespan_config: &Config,
) -> Result<(), ReportingError> {
    // Apply filters
    let issues = apply_filters(issues, config);

    // Determine if we should use colors
    let use_colors = config.color_choice.should_use_colors(atty::is(atty::Stream::Stdout));

    // Create a buffer for codespan (it requires WriteColor)
    let mut buffer = if use_colors { Buffer::ansi() } else { Buffer::no_color() };

    let files = DatabaseFiles(database);

    let highest_level = issues.get_highest_level();
    let mut errors = 0;
    let mut warnings = 0;
    let mut notes = 0;
    let mut help = 0;
    let mut suggestions = 0;

    for issue in issues {
        match &issue.level {
            Level::Note => {
                notes += 1;
            }
            Level::Help => {
                help += 1;
            }
            Level::Warning => {
                warnings += 1;
            }
            Level::Error => {
                errors += 1;
            }
        }

        if !issue.suggestions.is_empty() {
            suggestions += 1;
        }

        let diagnostic: Diagnostic<FileId> = issue.into();

        term::emit_to_write_style(&mut buffer, codespan_config, &files, &diagnostic)?;
    }

    if let Some(highest_level) = highest_level {
        let total_issues = errors + warnings + notes + help;
        let mut message_notes = vec![];
        if errors > 0 {
            message_notes.push(format!("{errors} error(s)"));
        }

        if warnings > 0 {
            message_notes.push(format!("{warnings} warning(s)"));
        }

        if notes > 0 {
            message_notes.push(format!("{notes} note(s)"));
        }

        if help > 0 {
            message_notes.push(format!("{help} help message(s)"));
        }

        let mut diagnostic: Diagnostic<FileId> = Diagnostic::new(highest_level.into()).with_message(format!(
            "found {} issues: {}",
            total_issues,
            message_notes.join(", ")
        ));

        if suggestions > 0 {
            diagnostic = diagnostic.with_notes(vec![format!("{} issues contain auto-fix suggestions", suggestions)]);
        }

        term::emit_to_write_style(&mut buffer, codespan_config, &files, &diagnostic)?;
    }

    // Write buffer to writer
    writer.write_all(buffer.as_slice())?;

    Ok(())
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

struct DatabaseFiles<'a>(&'a ReadDatabase);

impl<'a> Files<'a> for DatabaseFiles<'_> {
    type FileId = FileId;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, file_id: FileId) -> Result<&'a str, Error> {
        self.0.get_ref(&file_id).map(|source| source.name.as_ref()).map_err(|_| Error::FileMissing)
    }

    fn source(&'a self, file_id: FileId) -> Result<&'a str, Error> {
        self.0.get_ref(&file_id).map(|source| source.contents.as_ref()).map_err(|_| Error::FileMissing)
    }

    fn line_index(&self, file_id: FileId, byte_index: usize) -> Result<usize, Error> {
        let file = self.0.get_ref(&file_id).map_err(|_| Error::FileMissing)?;

        Ok(file.line_number(
            byte_index.try_into().map_err(|_| Error::IndexTooLarge { given: byte_index, max: u32::MAX as usize })?,
        ) as usize)
    }

    fn line_range(&self, file_id: FileId, line_index: usize) -> Result<Range<usize>, Error> {
        let file = self.0.get(&file_id).map_err(|_| Error::FileMissing)?;

        codespan_line_range(&file.lines, file.size, line_index)
    }
}

fn codespan_line_start(lines: &[u32], size: u32, line_index: usize) -> Result<usize, Error> {
    match line_index.cmp(&lines.len()) {
        Ordering::Less => Ok(lines.get(line_index).copied().expect("failed despite previous check") as usize),
        Ordering::Equal => Ok(size as usize),
        Ordering::Greater => Err(Error::LineTooLarge { given: line_index, max: lines.len() - 1 }),
    }
}

fn codespan_line_range(lines: &[u32], size: u32, line_index: usize) -> Result<Range<usize>, Error> {
    let line_start = codespan_line_start(lines, size, line_index)?;
    let next_line_start = codespan_line_start(lines, size, line_index + 1)?;

    Ok(line_start..next_line_start)
}

impl From<AnnotationKind> for LabelStyle {
    fn from(kind: AnnotationKind) -> LabelStyle {
        match kind {
            AnnotationKind::Primary => LabelStyle::Primary,
            AnnotationKind::Secondary => LabelStyle::Secondary,
        }
    }
}

impl From<Annotation> for Label<FileId> {
    fn from(annotation: Annotation) -> Label<FileId> {
        let mut label = Label::new(annotation.kind.into(), annotation.span.file_id, annotation.span);

        if let Some(message) = annotation.message {
            label.message = message;
        }

        label
    }
}

impl From<Level> for Severity {
    fn from(level: Level) -> Severity {
        match level {
            Level::Note => Severity::Note,
            Level::Help => Severity::Help,
            Level::Warning => Severity::Warning,
            Level::Error => Severity::Error,
        }
    }
}

impl From<Issue> for Diagnostic<FileId> {
    fn from(issue: Issue) -> Diagnostic<FileId> {
        let mut diagnostic = Diagnostic::new(issue.level.into()).with_message(issue.message);

        if let Some(code) = issue.code {
            diagnostic.code = Some(code);
        }

        for annotation in issue.annotations {
            diagnostic.labels.push(annotation.into());
        }

        for note in issue.notes {
            diagnostic.notes.push(note);
        }

        if let Some(help) = issue.help {
            diagnostic.notes.push(format!("Help: {help}"));
        }

        if let Some(link) = issue.link {
            diagnostic.notes.push(format!("See: {link}"));
        }

        diagnostic
    }
}
