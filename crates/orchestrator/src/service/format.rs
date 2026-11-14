use mago_allocator::LocalArena;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::error::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;

mod result;
pub use result::FileFormatStatus;
pub use result::FormatParseError;
pub(crate) use result::FormatReducer;
pub use result::FormatResult;

/// Formats PHP inputs. The calling tool selects the language.
#[derive(Debug)]
pub struct FormatService {
    database: ReadDatabase,
    context: FormatContext,
    use_progress_bars: bool,
}

impl FormatService {
    #[must_use]
    pub fn new(
        database: ReadDatabase,
        php_version: PHPVersion,
        settings: FormatSettings,
        parser_settings: ParserSettings,
        use_progress_bars: bool,
    ) -> Self {
        Self { database, context: FormatContext { php_version, settings, parser_settings }, use_progress_bars }
    }

    /// Formats one PHP file with a fresh arena.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting fails outside a recoverable parse error.
    pub fn format_file(self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        self.format_file_in(file, &LocalArena::new())
    }

    /// Formats one PHP file with the supplied arena.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting fails outside a recoverable parse error.
    pub fn format_file_in(self, file: &File, arena: &LocalArena) -> Result<FileFormatStatus, OrchestratorError> {
        Ok(self.context.format_file(file, arena))
    }

    /// Formats every selected host file as PHP.
    ///
    /// # Errors
    ///
    /// Returns an error if the parallel pipeline fails.
    pub fn run(self) -> Result<FormatResult, OrchestratorError> {
        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting",
            self.database,
            self.context,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run(|context, arena, file| Ok((file.id, context.format_file(&file, arena))))
    }

    /// Formats the selected file IDs as PHP.
    ///
    /// # Errors
    ///
    /// Returns an error if the pipeline fails or a file ID cannot be resolved.
    pub fn run_on_files<Iter>(self, file_ids: Iter) -> Result<FormatResult, OrchestratorError>
    where
        Iter: IntoIterator<Item = FileId>,
    {
        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting",
            self.database,
            self.context,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run_on_files(file_ids, |context, arena, file| Ok((file.id, context.format_file(&file, arena))))
    }
}

#[derive(Clone, Copy, Debug)]
struct FormatContext {
    php_version: PHPVersion,
    settings: FormatSettings,
    parser_settings: ParserSettings,
}

impl FormatContext {
    fn format_file(self, file: &File, arena: &LocalArena) -> FileFormatStatus {
        let program = parse_file_borrowed_with_settings(arena, file, self.parser_settings);
        if let Some(parse_error) = program.errors.first() {
            return FileFormatStatus::FailedToParse(FormatParseError::Php(parse_error.clone()));
        }

        let formatted = Formatter::new(arena, self.php_version, self.settings).format(file, program);
        if file.contents == formatted {
            FileFormatStatus::Unchanged
        } else {
            FileFormatStatus::Changed(formatted.to_vec())
        }
    }
}
