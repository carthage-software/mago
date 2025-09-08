use serde::Serialize;
use termcolor::WriteColor;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;

use crate::internal::emitter::utils::long_message;

#[derive(Serialize)]
struct CodeQualityIssue<'a> {
    description: String,
    check_name: &'a str,
    fingerprint: String,
    severity: &'a str,
    location: Location,
}

#[derive(Serialize)]
struct Location {
    path: String,
    lines: Lines,
}

#[derive(Serialize)]
struct Lines {
    begin: u32,
}

pub fn gitlab_format(
    writer: &mut dyn WriteColor,
    database: &ReadDatabase,
    issues: IssueCollection,
) -> Result<(), ReportingError> {
    let code_quality_issues = issues
        .iter()
        .map(|issue| {
            let severity = match &issue.level {
                Level::Note | Level::Help => "info",
                Level::Warning => "minor",
                Level::Error => "major",
            };

            let (path, line) = match issue.annotations.iter().find(|annotation| annotation.is_primary()) {
                Some(annotation) => {
                    let file = database.get(&annotation.span.file_id()).unwrap();
                    let line = file.line_number(annotation.span.start.offset) + 1;

                    (file.name.to_string(), line)
                }
                None => ("<unknown>".to_string(), 0),
            };

            let description = long_message(issue);

            let check_name = issue.code.as_deref().unwrap_or("other");

            let fingerprint = {
                let mut hasher = blake3::Hasher::new();
                hasher.update(check_name.as_bytes());
                hasher.update(path.as_bytes());
                hasher.update(line.to_le_bytes().as_slice());
                hasher.update(description.as_bytes());
                hasher.finalize().to_hex()[..32].to_string()
            };

            CodeQualityIssue {
                description,
                check_name,
                fingerprint,
                severity,
                location: Location { path, lines: Lines { begin: line } },
            }
        })
        .collect::<Vec<_>>();

    serde_json::to_writer_pretty(writer, &code_quality_issues)?;

    Ok(())
}
