use termcolor::WriteColor;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;

pub fn emacs_format(
    writer: &mut dyn WriteColor,
    database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
    for issue in issues.iter() {
        let (file_path, line, column) = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
            Some(annotation) => {
                let file = database.get(&annotation.span.file_id())?;
                let line = file.line_number(annotation.span.start.offset) + 1;
                let column = file.column_number(annotation.span.start.offset) + 1;

                (file.name.to_string(), line, column)
            }
            None => ("<unknown>".to_string(), 0, 0),
        };

        let severity = match issue.level {
            Level::Error => "error",
            Level::Warning | Level::Note | Level::Help => "warning",
        };

        let mut message = issue.message.clone();
        if let Some(link) = issue.link.as_deref() {
            message.push_str(" (see ");
            message.push_str(link);
            message.push(')');
        }

        let issue_type = issue.code.as_deref().unwrap_or("other");

        writeln!(writer, "{file_path}:{line}:{column}:{severity} - {issue_type}: {message}")?;
    }

    Ok(())
}
