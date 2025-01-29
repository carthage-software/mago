use termcolor::WriteColor;

use mago_interner::ThreadedInterner;
use mago_source::SourceManager;

use crate::error::ReportingError;
use crate::reporter::ReportingFormat;
use crate::IssueCollection;
use crate::Level;

mod utils;

pub mod checkstyle;
pub mod codespan;
pub mod count;
pub mod emacs;
pub mod github;
pub mod gitlab;
pub mod json;

pub trait Emitter {
    fn emit(
        &self,
        writer: &mut dyn WriteColor,
        sources: &SourceManager,
        interner: &ThreadedInterner,
        issues: IssueCollection,
    ) -> Result<Option<Level>, ReportingError>;
}

impl<T> Emitter for T
where
    T: Fn(
        &mut dyn WriteColor,
        &SourceManager,
        &ThreadedInterner,
        IssueCollection,
    ) -> Result<Option<Level>, ReportingError>,
{
    fn emit(
        &self,
        writer: &mut dyn WriteColor,
        sources: &SourceManager,
        interner: &ThreadedInterner,
        issues: IssueCollection,
    ) -> Result<Option<Level>, ReportingError> {
        self(writer, sources, interner, issues)
    }
}

impl Emitter for ReportingFormat {
    fn emit(
        &self,
        writer: &mut dyn WriteColor,
        sources: &SourceManager,
        interner: &ThreadedInterner,
        issues: IssueCollection,
    ) -> Result<Option<Level>, ReportingError> {
        match self {
            ReportingFormat::Rich => codespan::rich_format.emit(writer, sources, interner, issues),
            ReportingFormat::Medium => codespan::medium_format.emit(writer, sources, interner, issues),
            ReportingFormat::Short => codespan::short_format.emit(writer, sources, interner, issues),
            ReportingFormat::Github => github::github_format.emit(writer, sources, interner, issues),
            ReportingFormat::Gitlab => gitlab::gitlab_format.emit(writer, sources, interner, issues),
            ReportingFormat::Json => json::json_format.emit(writer, sources, interner, issues),
            ReportingFormat::Count => count::count_format.emit(writer, sources, interner, issues),
            ReportingFormat::Checkstyle => checkstyle::checkstyle_format.emit(writer, sources, interner, issues),
            ReportingFormat::Emacs => emacs::emacs_format.emit(writer, sources, interner, issues),
        }
    }
}
