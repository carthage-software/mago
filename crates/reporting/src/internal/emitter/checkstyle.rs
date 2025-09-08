use std::collections::HashMap;

use termcolor::WriteColor;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::internal::emitter::utils::long_message;
use crate::internal::emitter::utils::xml_encode;

pub fn checkstyle_format(
    writer: &mut dyn WriteColor,
    database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
    // Group issues by file
    let mut issues_by_file: HashMap<String, Vec<String>> = HashMap::new();

    for issue in issues.iter() {
        let (filename, line, column) = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
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
            Level::Warning => "warning",
            Level::Help | Level::Note => "info",
        };

        let message = xml_encode(long_message(issue));
        let error_tag =
            format!("    <error line=\"{line}\" column=\"{column}\" severity=\"{severity}\" message=\"{message}\" />");

        issues_by_file.entry(filename).or_default().push(error_tag);
    }

    // Begin Checkstyle XML
    writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(writer, "<checkstyle>")?;

    // Write grouped issues
    for (filename, errors) in issues_by_file {
        writeln!(writer, "  <file name=\"{}\">", xml_encode(&filename))?;
        for error in errors {
            writeln!(writer, "{error}")?;
        }

        writeln!(writer, "  </file>")?;
    }

    // Close Checkstyle XML
    writeln!(writer, "</checkstyle>")?;

    Ok(())
}
