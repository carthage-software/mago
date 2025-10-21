use ahash::HashMap;
use ahash::HashMapExt;

use bumpalo::Bump;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;
use mago_syntax::error::ParseError;

use crate::error::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormatStatus {
    Unchanged,
    Changed(String),
    FailedToParse(ParseError),
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
    use_progress_bars: bool,
}

impl FormatService {
    pub fn new(
        database: ReadDatabase,
        php_version: PHPVersion,
        settings: FormatSettings,
        use_progress_bars: bool,
    ) -> Self {
        Self { database, php_version, settings, use_progress_bars }
    }

    pub fn format_file(self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        let arena = Bump::new();

        self.format_file_in(file, &arena)
    }

    pub fn format_file_in(self, file: &File, arena: &Bump) -> Result<FileFormatStatus, OrchestratorError> {
        let formatter = Formatter::new(arena, self.php_version, self.settings);

        match formatter.format_file(file) {
            Ok(formatted_content) => {
                if file.contents == formatted_content {
                    Ok(FileFormatStatus::Unchanged)
                } else {
                    Ok(FileFormatStatus::Changed(formatted_content.to_string()))
                }
            }
            Err(parse_error) => Ok(FileFormatStatus::FailedToParse(parse_error)),
        }
    }

    pub fn run(self) -> Result<FormatResult, OrchestratorError> {
        let context = FormatContext { php_version: self.php_version, settings: self.settings };

        let pipeline = StatelessParallelPipeline::new(
            "✨ Formatting",
            self.database,
            context,
            Box::new(FormatReducer),
            self.use_progress_bars,
        );

        pipeline.run(|context, arena, file| {
            let formatter = Formatter::new(arena, context.php_version, context.settings);
            let status = match formatter.format_file(&file) {
                Ok(formatted_content) => {
                    if file.contents == formatted_content {
                        FileFormatStatus::Unchanged
                    } else {
                        FileFormatStatus::Changed(formatted_content.to_string())
                    }
                }
                Err(parse_error) => FileFormatStatus::FailedToParse(parse_error),
            };

            let mut changed_files = HashMap::with_capacity(1);
            changed_files.insert(file.id, status);

            Ok(FormatResult { changed_files })
        })
    }
}

impl Default for FormatResult {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatResult {
    pub fn new() -> Self {
        Self { changed_files: HashMap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.changed_files.is_empty()
    }

    pub fn is_successful(&self) -> bool {
        self.changed_files.values().all(|status| !matches!(status, FileFormatStatus::FailedToParse(_)))
    }

    pub fn is_failed(&self) -> bool {
        self.changed_files.values().any(|status| matches!(status, FileFormatStatus::FailedToParse(_)))
    }

    pub fn is_changed(&self) -> bool {
        self.changed_files.values().any(|status| matches!(status, FileFormatStatus::Changed(_)))
    }

    pub fn parse_errors(&self) -> impl Iterator<Item = (&FileId, &ParseError)> {
        self.changed_files.iter().filter_map(|(file_id, status)| {
            if let FileFormatStatus::FailedToParse(error) = status { Some((file_id, error)) } else { None }
        })
    }

    pub fn changed_files(&self) -> impl Iterator<Item = (&FileId, &String)> {
        self.changed_files.iter().filter_map(|(file_id, status)| {
            if let FileFormatStatus::Changed(content) = status { Some((file_id, content)) } else { None }
        })
    }

    pub fn changed_files_count(&self) -> usize {
        self.changed_files.values().filter(|status| matches!(status, FileFormatStatus::Changed(_))).count()
    }
}

/// Shared, read-only context provided to each parallel formatting task.
#[derive(Clone)]
struct FormatContext {
    /// The target PHP version for formatting rules.
    php_version: PHPVersion,
    /// The configured settings for the formatter.
    settings: FormatSettings,
}

#[derive(Debug, Clone)]
struct FormatReducer;

impl StatelessReducer<FormatResult, FormatResult> for FormatReducer {
    fn reduce(&self, results: Vec<FormatResult>) -> Result<FormatResult, OrchestratorError> {
        let mut changed_files = HashMap::with_capacity(results.len());

        for result in results {
            changed_files.extend(result.changed_files);
        }

        Ok(FormatResult { changed_files })
    }
}
