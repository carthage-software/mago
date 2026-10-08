use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::io::IsTerminal;
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
use foldhash::HashMap;
use termcolor::Ansi;
use termcolor::NoColor;

use mago_database::file::File;
use mago_database::file::FileId;
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
use crate::formatter::utils::osc8_file_hyperlink;
use crate::formatter::utils::utf8_preserving_byte_offsets;

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
    let use_colors = config.color_choice.should_use_colors(std::io::stdout().is_terminal());
    let mut ansi = Ansi::new(Vec::new());
    let mut plain = NoColor::new(Vec::new());
    let styled: &mut dyn term::WriteStyle = if use_colors { &mut ansi } else { &mut plain };

    let editor_url = if use_colors { config.editor_url.as_deref() } else { None };
    let files = DatabaseFiles::new(database, editor_url, issues);
    let mut renderer = term::Renderer::new(styled, codespan_config);

    let mut highest_level: Option<Level> = None;
    let mut errors = 0;
    let mut warnings = 0;
    let mut notes = 0;
    let mut help = 0;
    let mut suggestions = 0;
    let mut diagnostic = Diagnostic::new(Severity::Note);

    for issue in crate::formatter::utils::filter_issues(issues, config, true) {
        match issue.level {
            Level::Note => notes += 1,
            Level::Help => help += 1,
            Level::Warning => warnings += 1,
            Level::Error => errors += 1,
        }

        highest_level = Some(highest_level.map_or(issue.level, |cur| cur.max(issue.level)));

        if !issue.edits.is_empty() {
            suggestions += 1;
        }

        if editor_url.is_some() {
            if let Some(annotation) = issue.annotations.iter().find(|a| a.is_primary()) {
                if let Ok(file) = files.file(annotation.span.file_id) {
                    let line = file.line_number(annotation.span.start.offset) + 1;
                    let column = file.column_number(annotation.span.start.offset) + 1;
                    files.line_hint.set(Some(line));
                    files.column_hint.set(Some(column));
                }
            } else {
                files.line_hint.set(None);
                files.column_hint.set(None);
            }
        }

        update_diagnostic(&mut diagnostic, issue);

        emit_diagnostic(&mut renderer, codespan_config, &files, &diagnostic)?;
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

        emit_diagnostic(&mut renderer, codespan_config, &files, &diagnostic)?;
    }

    // Write buffer to writer
    writer.write_all(if use_colors { ansi.get_ref() } else { plain.get_ref() })?;

    Ok(())
}

fn emit_diagnostic(
    renderer: &mut term::Renderer<'_, '_>,
    config: &Config,
    files: &DatabaseFiles<'_>,
    diagnostic: &Diagnostic<FileId>,
) -> Result<(), Error> {
    match config.display_style {
        DisplayStyle::Rich => term::RichDiagnostic::new(diagnostic, config).render(files, renderer),
        DisplayStyle::Medium => term::ShortDiagnostic::new(diagnostic, true).render(files, renderer),
        DisplayStyle::Short => term::ShortDiagnostic::new(diagnostic, false).render(files, renderer),
    }
}

struct DatabaseFiles<'db> {
    database: &'db ReadDatabase,
    editor_url: Option<&'db str>,
    line_hint: Cell<Option<u32>>,
    column_hint: Cell<Option<u32>>,
    sources: HashMap<FileId, Cow<'db, str>>,
    last_file: Cell<Option<&'db File>>,
}

impl<'db> DatabaseFiles<'db> {
    fn new(database: &'db ReadDatabase, editor_url: Option<&'db str>, issues: &IssueCollection) -> Self {
        let mut sources: HashMap<FileId, Cow<'db, str>> = HashMap::default();
        for issue in issues.iter() {
            for annotation in &issue.annotations {
                let file_id = annotation.span.file_id;
                if let Entry::Vacant(entry) = sources.entry(file_id)
                    && let Ok(file) = database.get_ref(&file_id)
                {
                    entry.insert(utf8_preserving_byte_offsets(file.contents.as_ref()));
                }
            }
        }

        DatabaseFiles {
            database,
            editor_url,
            line_hint: Cell::new(None),
            column_hint: Cell::new(None),
            sources,
            last_file: Cell::new(None),
        }
    }

    fn file(&self, file_id: FileId) -> Result<&'db File, Error> {
        if let Some(file) = self.last_file.get()
            && file.id == file_id
        {
            return Ok(file);
        }
        let file = self.database.get_ref(&file_id).map_err(|_| Error::FileMissing)?;
        self.last_file.set(Some(file));
        Ok(file)
    }
}

impl<'files> Files<'files> for DatabaseFiles<'_> {
    type FileId = FileId;
    type Name = Cow<'files, str>;
    type Source = &'files str;

    fn name(&'files self, file_id: FileId) -> Result<Cow<'files, str>, Error> {
        let file = self.file(file_id)?;
        let name = String::from_utf8_lossy(&file.name);

        if let (Some(template), Some(path)) = (self.editor_url, file.path.as_ref()) {
            let abs_path = path.display().to_string();
            let line = self.line_hint.get().unwrap_or(1);
            let column = self.column_hint.get().unwrap_or(1);

            Ok(Cow::Owned(osc8_file_hyperlink(template, &abs_path, &name, line, column, &name)))
        } else {
            Ok(name)
        }
    }

    fn source(&'files self, file_id: FileId) -> Result<&'files str, Error> {
        self.sources.get(&file_id).map(AsRef::as_ref).ok_or(Error::FileMissing)
    }

    fn line_index(&self, file_id: FileId, byte_index: usize) -> Result<usize, Error> {
        let file = self.file(file_id)?;

        Ok(file.line_number(
            byte_index.try_into().map_err(|_| Error::IndexTooLarge { given: byte_index, max: u32::MAX as usize })?,
        ) as usize)
    }

    fn line_range(&self, file_id: FileId, line_index: usize) -> Result<Range<usize>, Error> {
        let file = self.file(file_id)?;

        codespan_line_range(&file.lines, file.size, line_index)
    }
}

fn update_diagnostic(diagnostic: &mut Diagnostic<FileId>, issue: &Issue) {
    diagnostic.severity = issue.level.into();
    diagnostic.message.clone_from(&issue.message);
    diagnostic.code.clone_from(&issue.code);

    for (index, annotation) in issue.annotations.iter().enumerate() {
        if let Some(label) = diagnostic.labels.get_mut(index) {
            label.style = annotation.kind.into();
            label.file_id = annotation.span.file_id;
            label.range = annotation.span.into();
            if let Some(message) = &annotation.message {
                label.message.clone_from(message);
            } else {
                label.message.clear();
            }
        } else {
            diagnostic.labels.push(annotation.into());
        }
    }
    diagnostic.labels.truncate(issue.annotations.len());

    let note_count = issue.notes.len() + usize::from(issue.help.is_some()) + usize::from(issue.link.is_some());
    diagnostic.notes.resize_with(note_count, String::new);
    for (target, note) in diagnostic.notes.iter_mut().zip(&issue.notes) {
        target.clone_from(note);
    }
    let mut remaining = diagnostic.notes[issue.notes.len()..].iter_mut();
    for (prefix, text) in [("Help: ", issue.help.as_ref()), ("See: ", issue.link.as_ref())] {
        if let Some(text) = text
            && let Some(target) = remaining.next()
        {
            target.clear();
            target.push_str(prefix);
            target.push_str(text);
        }
    }
}

fn codespan_line_start(lines: &[u32], size: u32, line_index: usize) -> Result<usize, Error> {
    match line_index.cmp(&lines.len()) {
        // The `Ordering::Less` arm guarantees `line_index < lines.len()`, so `get` is `Some`;
        // a missing value here would mean a `Vec::len`/indexing inconsistency, so we fall back
        // to `0` defensively rather than panicking.
        Ordering::Less => Ok(lines.get(line_index).copied().unwrap_or(0) as usize),
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

impl From<&Annotation> for Label<FileId> {
    fn from(annotation: &Annotation) -> Label<FileId> {
        let mut label = Label::new(annotation.kind.into(), annotation.span.file_id, annotation.span);

        if let Some(message) = &annotation.message {
            label.message.clone_from(message);
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

impl From<&Issue> for Diagnostic<FileId> {
    fn from(issue: &Issue) -> Diagnostic<FileId> {
        let mut diagnostic = Diagnostic::new(issue.level.into()).with_message(issue.message.clone());

        if let Some(code) = &issue.code {
            diagnostic.code = Some(code.clone());
        }

        for annotation in &issue.annotations {
            diagnostic.labels.push(annotation.into());
        }

        for note in &issue.notes {
            diagnostic.notes.push(note.clone());
        }

        if let Some(help) = &issue.help {
            diagnostic.notes.push(format!("Help: {help}"));
        }

        if let Some(link) = &issue.link {
            diagnostic.notes.push(format!("See: {link}"));
        }

        diagnostic
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::path::Path;
    use std::path::PathBuf;

    use codespan_reporting::files::Files;
    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;
    use mago_database::file::FileType;

    use crate::IssueCollection;

    use super::DatabaseFiles;

    #[test]
    fn editor_url_uses_the_logical_name_for_the_relative_file_placeholder() {
        let file = File::new(
            Cow::Borrowed(b"src/Foo.php"),
            FileType::Host,
            Some(PathBuf::from("/workspace/src/Foo.php")),
            Cow::Borrowed(b"<?php\n"),
        );
        let file_id = file.id;
        let configuration =
            DatabaseConfiguration::new(Path::new("/workspace"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, configuration).read_only();
        let issues = IssueCollection::new();
        let files = DatabaseFiles::new(&database, Some("editor://%rel_file%:%line%:%column%"), &issues);
        let Ok(name) = Files::name(&files, file_id) else {
            panic!("file should exist");
        };

        assert_eq!(name, "\x1b]8;;editor://src/Foo.php:1:1\x1b\\src/Foo.php\x1b]8;;\x1b\\",);
    }
}
