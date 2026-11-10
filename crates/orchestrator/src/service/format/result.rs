use std::error::Error;
use std::fmt;

use foldhash::HashMap;
use foldhash::HashMapExt;
use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::error::ParseError;

use crate::error::OrchestratorError;
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

impl fmt::Display for FormatParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Php(error) => fmt::Display::fmt(error, formatter),
            Self::Twig { message, .. } => formatter.write_str(message),
        }
    }
}

impl Error for FormatParseError {}

impl HasSpan for FormatParseError {
    fn span(&self) -> Span {
        match self {
            Self::Php(error) => error.span(),
            Self::Twig { span, .. } => *span,
        }
    }
}

#[derive(Debug, Default)]
pub struct FormatResult {
    pub changed_files: HashMap<FileId, FileFormatStatus>,
}

impl FormatResult {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
        !self.is_successful()
    }

    #[must_use]
    pub fn is_changed(&self) -> bool {
        self.changed_files.values().any(|status| matches!(status, FileFormatStatus::Changed(_)))
    }

    pub fn parse_errors(&self) -> impl Iterator<Item = (&FileId, &FormatParseError)> {
        self.changed_files.iter().filter_map(|(file, status)| match status {
            FileFormatStatus::FailedToParse(error) => Some((file, error)),
            _ => None,
        })
    }

    pub fn changed_files(&self) -> impl Iterator<Item = (&FileId, &Vec<u8>)> {
        self.changed_files.iter().filter_map(|(file, status)| match status {
            FileFormatStatus::Changed(content) => Some((file, content)),
            _ => None,
        })
    }

    #[must_use]
    pub fn changed_files_count(&self) -> usize {
        self.changed_files().count()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FormatReducer;

impl StatelessReducer<(FileId, FileFormatStatus), FormatResult> for FormatReducer {
    fn reduce(&self, results: Vec<(FileId, FileFormatStatus)>) -> Result<FormatResult, OrchestratorError> {
        let mut changed_files = HashMap::with_capacity(results.len());
        changed_files.extend(results);
        Ok(FormatResult { changed_files })
    }
}
