use foldhash::HashMap;
use foldhash::HashMapExt;

use mago_allocator::LocalArena;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::error::ParseError;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;
use mago_twig_formatter::Formatter as TwigFormatter;
use mago_twig_formatter::settings::FormatSettings as TwigFormatSettings;

use crate::error::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormatStatus {
    Unchanged,
    Changed(Vec<u8>),
    FailedToParse(FormatParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatParseError {
    Php(ParseError),
    Twig { message: String, span: Span },
}

impl std::fmt::Display for FormatParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Php(error) => std::fmt::Display::fmt(error, formatter),
            Self::Twig { message, .. } => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FormatParseError {}

impl HasSpan for FormatParseError {
    fn span(&self) -> Span {
        match self {
            Self::Php(error) => error.span(),
            Self::Twig { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub struct FormatResult {
    pub changed_files: HashMap<FileId, FileFormatStatus>,
}

#[derive(Debug)]
pub struct FormatService {
    database: ReadDatabase,
    php_version: PHPVersion,
    settings: FormatSettings,
    twig_settings: Option<TwigFormatSettings>,
    parser_settings: ParserSettings,
    use_progress_bars: bool,
}

impl FormatService {
    #[must_use]
    pub fn new(
        database: ReadDatabase,
        php_version: PHPVersion,
        settings: FormatSettings,
        twig_settings: Option<TwigFormatSettings>,
        parser_settings: ParserSettings,
        use_progress_bars: bool,
    ) -> Self {
        Self { database, php_version, settings, twig_settings, parser_settings, use_progress_bars }
    }

    /// Formats a single file, allocating a fresh arena.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] if the formatter fails outside of recoverable parse errors.
    pub fn format_file(self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        let arena = LocalArena::new();

        self.format_file_in(file, &arena)
    }

    /// Formats a single file using a caller-provided bump arena.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] if the formatter fails outside of recoverable parse errors.
    pub fn format_file_in(self, file: &File, arena: &LocalArena) -> Result<FileFormatStatus, OrchestratorError> {
        let context = FormatContext {
            php_version: self.php_version,
            settings: self.settings,
            twig_settings: self.twig_settings,
            parser_settings: self.parser_settings,
        };

        Ok(context.format_file(file, arena))
    }

    /// Runs the formatter pipeline over every file in the database.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when a worker fails or the pipeline cannot reduce results.
    pub fn run(self) -> Result<FormatResult, OrchestratorError> {
        let context = FormatContext {
            php_version: self.php_version,
            settings: self.settings,
            twig_settings: self.twig_settings,
            parser_settings: self.parser_settings,
        };

        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting",
            self.database,
            context,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run(|context, arena, file| Ok((file.id, context.format_file(&file, arena))))
    }

    /// Runs the formatter on a specific subset of files by ID.
    ///
    /// This method formats only the files with the given IDs, rather than all files
    /// in the database. This is useful for formatting only staged files in git
    /// pre-commit hooks.
    ///
    /// # Arguments
    ///
    /// * `file_ids` - Iterator of file IDs to format
    ///
    /// # Returns
    ///
    /// A [`FormatResult`] containing the formatting status for each processed file.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when a worker fails or a requested `FileId` cannot be resolved.
    pub fn run_on_files<Iter>(self, file_ids: Iter) -> Result<FormatResult, OrchestratorError>
    where
        Iter: IntoIterator<Item = FileId>,
    {
        let context = FormatContext {
            php_version: self.php_version,
            settings: self.settings,
            twig_settings: self.twig_settings,
            parser_settings: self.parser_settings,
        };

        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting",
            self.database,
            context,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run_on_files(file_ids, |context, arena, file| Ok((file.id, context.format_file(&file, arena))))
    }
}

impl Default for FormatResult {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatResult {
    #[must_use]
    pub fn new() -> Self {
        Self { changed_files: HashMap::new() }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changed_files.is_empty()
    }

    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.changed_files.values().all(|status| !matches!(status, FileFormatStatus::FailedToParse(_)))
    }

    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.changed_files.values().any(|status| matches!(status, FileFormatStatus::FailedToParse(_)))
    }

    #[must_use]
    pub fn is_changed(&self) -> bool {
        self.changed_files.values().any(|status| matches!(status, FileFormatStatus::Changed(_)))
    }

    pub fn parse_errors(&self) -> impl Iterator<Item = (&FileId, &FormatParseError)> {
        self.changed_files.iter().filter_map(|(file_id, status)| {
            if let FileFormatStatus::FailedToParse(error) = status { Some((file_id, error)) } else { None }
        })
    }

    pub fn changed_files(&self) -> impl Iterator<Item = (&FileId, &Vec<u8>)> {
        self.changed_files.iter().filter_map(|(file_id, status)| {
            if let FileFormatStatus::Changed(content) = status { Some((file_id, content)) } else { None }
        })
    }

    #[must_use]
    pub fn changed_files_count(&self) -> usize {
        self.changed_files.values().filter(|status| matches!(status, FileFormatStatus::Changed(_))).count()
    }
}

/// Shared, read-only context provided to each parallel formatting task.
#[derive(Clone, Copy)]
struct FormatContext {
    /// The target PHP version for formatting rules.
    php_version: PHPVersion,
    /// The configured settings for the formatter.
    settings: FormatSettings,
    twig_settings: Option<TwigFormatSettings>,
    /// The parser settings.
    parser_settings: ParserSettings,
}

impl FormatContext {
    fn format_file(self, file: &File, arena: &LocalArena) -> FileFormatStatus {
        if file
            .name
            .get(file.name.len().saturating_sub(5)..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(b".twig"))
        {
            let Some(settings) = self.twig_settings else {
                return FileFormatStatus::Unchanged;
            };

            return match TwigFormatter::new(arena, settings).format_file(file) {
                Ok(content) if file.contents == content => FileFormatStatus::Unchanged,
                Ok(content) => FileFormatStatus::Changed(content.to_vec()),
                Err(error) => FileFormatStatus::FailedToParse(FormatParseError::Twig {
                    message: error.to_string(),
                    span: error.span(),
                }),
            };
        }

        let program = parse_file_borrowed_with_settings(arena, file, self.parser_settings);
        if let Some(parse_error) = program.errors.first() {
            return FileFormatStatus::FailedToParse(FormatParseError::Php(parse_error.clone()));
        }

        let formatter = Formatter::new(arena, self.php_version, self.settings);
        let formatted_content = formatter.format(file, program);
        if file.contents == formatted_content {
            FileFormatStatus::Unchanged
        } else {
            FileFormatStatus::Changed(formatted_content.to_vec())
        }
    }
}

#[derive(Debug, Clone)]
struct FormatReducer;

impl StatelessReducer<(FileId, FileFormatStatus), FormatResult> for FormatReducer {
    fn reduce(&self, results: Vec<(FileId, FileFormatStatus)>) -> Result<FormatResult, OrchestratorError> {
        let mut changed_files = HashMap::with_capacity(results.len());

        changed_files.extend(results);

        Ok(FormatResult { changed_files })
    }
}
