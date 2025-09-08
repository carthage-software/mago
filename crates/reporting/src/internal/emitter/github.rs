use termcolor::WriteColor;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::internal::emitter::utils::long_message;

pub fn github_format(
    writer: &mut dyn WriteColor,
    database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
    for issue in issues.iter() {
        let level = match &issue.level {
            Level::Note => "notice",
            Level::Help => "notice",
            Level::Warning => "warning",
            Level::Error => "error",
        };

        let properties = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
            Some(annotation) => {
                let file = database.get(&annotation.span.file_id())?;
                let start_line = file.line_number(annotation.span.start.offset) + 1;
                let end_line = file.line_number(annotation.span.end.offset) + 1;

                if let Some(code) = issue.code.as_ref() {
                    format!("file={},line={start_line},endLine={end_line},title={code}", file.name)
                } else {
                    format!("file={},line={start_line},endLine={end_line}", file.name)
                }
            }
            None => {
                if let Some(code) = issue.code.as_ref() {
                    format!("title={code}")
                } else {
                    String::new()
                }
            }
        };

        // we must use `%0A` instead of `\n`.
        //
        // see: https://github.com/actions/toolkit/issues/193
        let message = long_message(issue).replace("\n", "%0A");

        writeln!(writer, "::{level} {properties}::{message}")?;
    }

    Ok(())
}
