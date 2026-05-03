use std::io::Write;

use serde::Serialize;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;

use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::formatter::utils::long_message;

#[derive(Serialize)]
struct CodeQualityIssue<'issue> {
    description: String,
    check_name: &'issue str,
    fingerprint: String,
    severity: &'issue str,
    location: Location,
}

#[derive(Serialize)]
struct Location {
    path: String,
    positions: Positions,
}

#[derive(Serialize)]
struct Positions {
    begin: Position,
    end: Position,
}

#[derive(Serialize)]
struct Position {
    line: u32,
    column: u32,
}

/// Formatter that outputs issues in GitLab Code Quality JSON format.
pub(crate) struct GitlabFormatter;

impl Formatter for GitlabFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        let code_quality_issues = crate::formatter::utils::filter_issues(issues, config, false)
            .map(|issue| {
                let severity = match &issue.level {
                    Level::Note | Level::Help => "info",
                    Level::Warning => "minor",
                    Level::Error => "major",
                };

                let (path, positions) =
                    match issue.annotations.iter().find(|annotation| annotation.is_primary()).and_then(|annotation| {
                        database.get(&annotation.span.file_id()).ok().map(|file| (annotation, file))
                    }) {
                        Some((annotation, file)) => {
                            let begin = Position {
                                line: file.line_number(annotation.span.start.offset) + 1,
                                column: file.column_number(annotation.span.start.offset) + 1,
                            };

                            let end = Position {
                                line: file.line_number(annotation.span.end.offset) + 1,
                                column: file.column_number(annotation.span.end.offset) + 1,
                            };

                            (file.name.to_string(), Positions { begin, end })
                        }
                        None => (
                            "<unknown>".to_string(),
                            Positions { begin: Position { line: 0, column: 0 }, end: Position { line: 0, column: 0 } },
                        ),
                    };

                let description = long_message(issue, true);

                let check_name = issue.code.as_deref().unwrap_or("other");

                let fingerprint = {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(check_name.as_bytes());
                    hasher.update(path.as_bytes());
                    // GitLab Code Quality fingerprints must be stable across machines, so we
                    // deliberately serialize line/column as little-endian regardless of host.
                    #[allow(clippy::little_endian_bytes)]
                    let line_bytes = positions.begin.line.to_le_bytes();
                    #[allow(clippy::little_endian_bytes)]
                    let col_bytes = positions.begin.column.to_le_bytes();
                    hasher.update(&line_bytes);
                    hasher.update(&col_bytes);
                    hasher.update(description.as_bytes());
                    hasher.finalize().to_hex()[..32].to_string()
                };

                CodeQualityIssue {
                    description,
                    check_name,
                    fingerprint,
                    severity,
                    location: Location { path, positions },
                }
            })
            .collect::<Vec<_>>();

        serde_json::to_writer_pretty(writer, &code_quality_issues)?;

        Ok(())
    }
}
