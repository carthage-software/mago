use std::borrow::Cow;
use std::io::Write;

use foldhash::HashMap;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeSeq;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;

use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;
use crate::formatter::utils::long_message;

#[derive(Serialize)]
struct CodeQualityIssue<'issue> {
    description: &'issue str,
    check_name: &'issue str,
    fingerprint: &'issue str,
    severity: &'issue str,
    location: Location<'issue>,
}

#[derive(Serialize)]
struct Location<'file> {
    path: &'file str,
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

struct FileDetails<'file> {
    file: &'file File,
    path: Cow<'file, str>,
}

struct CodeQualityIssues<'issue, 'file> {
    issues: Vec<&'issue Issue>,
    files: HashMap<FileId, Option<FileDetails<'file>>>,
}

impl Serialize for CodeQualityIssues<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut issues = serializer.serialize_seq(Some(self.issues.len()))?;
        for issue in &self.issues {
            let severity = match issue.level {
                Level::Note | Level::Help => "info",
                Level::Warning => "minor",
                Level::Error => "major",
            };

            let (path, positions) =
                match issue.annotations.iter().find(|annotation| annotation.is_primary()).and_then(|annotation| {
                    self.files[&annotation.span.file_id].as_ref().map(|details| (annotation, details))
                }) {
                    Some((annotation, details)) => {
                        let begin_line = details.file.line_number(annotation.span.start.offset);
                        let end_line = details.file.line_number(annotation.span.end.offset);
                        (
                            details.path.as_ref(),
                            Positions {
                                begin: Position {
                                    line: begin_line + 1,
                                    column: annotation.span.start.offset - details.file.lines[begin_line as usize] + 1,
                                },
                                end: Position {
                                    line: end_line + 1,
                                    column: annotation.span.end.offset - details.file.lines[end_line as usize] + 1,
                                },
                            },
                        )
                    }
                    None => (
                        "<unknown>",
                        Positions { begin: Position { line: 0, column: 0 }, end: Position { line: 0, column: 0 } },
                    ),
                };

            let description = if issue.notes.is_empty()
                && issue.help.is_none()
                && issue.link.is_none()
                && issue.annotations.iter().all(|annotation| annotation.message.is_none())
            {
                Cow::Borrowed(issue.message.as_ref())
            } else {
                Cow::Owned(long_message(issue, true))
            };

            let check_name = issue.code.as_deref().unwrap_or("other");
            let mut hasher = blake3::Hasher::new();
            hasher.update(check_name.as_bytes());
            hasher.update(path.as_bytes());
            // Keep fingerprint coordinates little-endian on every host.
            #[allow(clippy::little_endian_bytes)]
            let line_bytes = positions.begin.line.to_le_bytes();
            #[allow(clippy::little_endian_bytes)]
            let col_bytes = positions.begin.column.to_le_bytes();
            hasher.update(&line_bytes);
            hasher.update(&col_bytes);
            hasher.update(description.as_bytes());
            let fingerprint = hasher.finalize().to_hex();
            issues.serialize_element(&CodeQualityIssue {
                description: description.as_ref(),
                check_name,
                fingerprint: &fingerprint[..32],
                severity,
                location: Location { path, positions },
            })?;
        }

        issues.end()
    }
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
        let issues: Vec<_> = crate::formatter::utils::filter_issues(issues, config, false).collect();
        let mut files = HashMap::default();
        for issue in &issues {
            if let Some(annotation) = issue.annotations.iter().find(|annotation| annotation.is_primary()) {
                files.entry(annotation.span.file_id).or_insert_with(|| {
                    database
                        .get_ref(&annotation.span.file_id)
                        .ok()
                        .map(|file| FileDetails { file, path: String::from_utf8_lossy(&file.name) })
                });
            }
        }

        crate::formatter::utils::write_pretty_json(writer, &CodeQualityIssues { issues, files })?;
        Ok(())
    }
}
