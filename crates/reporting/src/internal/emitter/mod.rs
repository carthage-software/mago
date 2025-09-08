use termcolor::WriteColor;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::reporter::ReportingFormat;

mod utils;

pub mod ariadne;
pub mod checkstyle;
pub mod code_count;
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
        database: &ReadDatabase,
        issues: IssueCollection,
    ) -> Result<(), ReportingError>;
}

impl<T> Emitter for T
where
    T: Fn(&mut dyn WriteColor, &ReadDatabase, IssueCollection) -> Result<(), ReportingError>,
{
    fn emit(
        &self,
        writer: &mut dyn WriteColor,
        database: &ReadDatabase,
        issues: IssueCollection,
    ) -> Result<(), ReportingError> {
        self(writer, database, issues)
    }
}

impl Emitter for ReportingFormat {
    fn emit(
        &self,
        writer: &mut dyn WriteColor,
        database: &ReadDatabase,
        issues: IssueCollection,
    ) -> Result<(), ReportingError> {
        match self {
            ReportingFormat::Rich => codespan::rich_format.emit(writer, database, issues),
            ReportingFormat::Medium => codespan::medium_format.emit(writer, database, issues),
            ReportingFormat::Short => codespan::short_format.emit(writer, database, issues),
            ReportingFormat::Ariadne => ariadne::ariadne_format.emit(writer, database, issues),
            ReportingFormat::Github => github::github_format.emit(writer, database, issues),
            ReportingFormat::Gitlab => gitlab::gitlab_format.emit(writer, database, issues),
            ReportingFormat::Json => json::json_format.emit(writer, database, issues),
            ReportingFormat::Count => count::count_format.emit(writer, database, issues),
            ReportingFormat::CodeCount => code_count::code_count_format.emit(writer, database, issues),
            ReportingFormat::Checkstyle => checkstyle::checkstyle_format.emit(writer, database, issues),
            ReportingFormat::Emacs => emacs::emacs_format.emit(writer, database, issues),
        }
    }
}
