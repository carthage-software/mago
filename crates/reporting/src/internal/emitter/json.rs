use termcolor::WriteColor;

use mago_database::ReadDatabase;

use crate::IssueCollection;
use crate::error::ReportingError;
use crate::internal::Expandable;

pub fn json_format(
    writer: &mut dyn WriteColor,
    database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
    let issues = issues.expand(database)?;

    serde_json::to_writer_pretty(writer, &issues)?;

    Ok(())
}
