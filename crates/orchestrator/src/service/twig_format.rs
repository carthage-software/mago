use mago_allocator::LocalArena;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::FormatSettings;

use crate::error::OrchestratorError;
use crate::service::format::FileFormatStatus;
use crate::service::format::FormatParseError;
use crate::service::format::FormatReducer;
use crate::service::format::FormatResult;
use crate::service::pipeline::StatelessParallelPipeline;

/// Formats every selected input as Twig, regardless of its filename.
#[derive(Debug)]
pub struct TwigFormatService {
    database: ReadDatabase,
    settings: FormatSettings,
    use_progress_bars: bool,
}

impl TwigFormatService {
    #[must_use]
    pub fn new(database: ReadDatabase, settings: FormatSettings, use_progress_bars: bool) -> Self {
        Self { database, settings, use_progress_bars }
    }

    /// Formats one Twig input.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting fails outside a recoverable parse error.
    pub fn format_file(self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        Ok(format_file(file, &LocalArena::new(), self.settings))
    }

    /// Formats all selected host files as Twig.
    ///
    /// # Errors
    ///
    /// Returns an error if the parallel pipeline fails.
    pub fn run(self) -> Result<FormatResult, OrchestratorError> {
        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting Twig",
            self.database,
            self.settings,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run(|settings, arena, file| Ok((file.id, format_file(&file, arena, settings))))
    }
}

fn format_file(file: &File, arena: &LocalArena, settings: FormatSettings) -> FileFormatStatus {
    match Formatter::new(arena, settings).format_file(file) {
        Ok(content) if file.contents == content => FileFormatStatus::Unchanged,
        Ok(content) => FileFormatStatus::Changed(content.to_vec()),
        Err(error) => {
            FileFormatStatus::FailedToParse(FormatParseError::Twig { message: error.to_string(), span: error.span() })
        }
    }
}
