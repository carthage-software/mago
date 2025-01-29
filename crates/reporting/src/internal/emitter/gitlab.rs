use mago_interner::ThreadedInterner;
use mago_source::HasSource;
use mago_source::SourceManager;

use serde::Serialize;
use termcolor::WriteColor;

use crate::error::ReportingError;
use crate::IssueCollection;
use crate::Level;

use super::utils::long_message;

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
    begin: usize,
}

pub fn gitlab_format(
    writer: &mut dyn WriteColor,
    sources: &SourceManager,
    interner: &ThreadedInterner,
    issues: IssueCollection,
) -> Result<Option<Level>, ReportingError> {
    let highest_level = issues.get_highest_level();

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
                    let source = sources.load(&annotation.span.source()).unwrap();

                    let file_path = interner.lookup(&source.identifier.0).to_string();
                    let line = source.line_number(annotation.span.start.offset) + 1;

                    (file_path, line)
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

    Ok(highest_level)
}
