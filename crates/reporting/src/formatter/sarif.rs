use std::io::Write;

use serde_sarif::sarif::ArtifactLocation;
use serde_sarif::sarif::Location;
use serde_sarif::sarif::Message;
use serde_sarif::sarif::PhysicalLocation;
use serde_sarif::sarif::Region;
use serde_sarif::sarif::Result as SarifResult;
use serde_sarif::sarif::ResultLevel;
use serde_sarif::sarif::Run;
use serde_sarif::sarif::Sarif;
use serde_sarif::sarif::Tool;
use serde_sarif::sarif::ToolComponent;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::HasFileId;
use serde_sarif::sarif::Version;

use crate::Annotation;
use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::formatter::Formatter;
use crate::formatter::FormatterConfig;

/// Formatter that outputs issues in SARIF (Static Analysis Results Interchange Format) 2.1.0.
///
/// SARIF is an OASIS standard for representing static analysis tool output, enabling
/// integration with GitHub Code Scanning, GitLab Code Quality, and other CI/CD platforms.
pub(crate) struct SarifFormatter;

impl Formatter for SarifFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        issues: &IssueCollection,
        database: &ReadDatabase,
        config: &FormatterConfig,
    ) -> Result<(), ReportingError> {
        let issues = apply_filters(issues, config);
        let sarif_log = build_sarif_log(&issues, database)?;
        serde_json::to_writer_pretty(writer, &sarif_log)?;

        Ok(())
    }
}

fn apply_filters(issues: &IssueCollection, config: &FormatterConfig) -> IssueCollection {
    let mut filtered = issues.clone();

    if let Some(min_level) = config.minimum_level {
        filtered = filtered.with_minimum_level(min_level);
    }

    if config.filter_fixable {
        filtered = filtered.filter_fixable();
    }

    if config.sort {
        filtered = filtered.sorted();
    }

    filtered
}

fn build_sarif_log(issues: &IssueCollection, database: &ReadDatabase) -> Result<Sarif, ReportingError> {
    let tool = Tool::builder()
        .driver(
            ToolComponent::builder()
                .name("mago")
                .version(env!("CARGO_PKG_VERSION"))
                .information_uri("https://github.com/carthage-software/mago")
                .build(),
        )
        .build();

    let mut results = Vec::new();
    for issue in &issues.issues {
        let sarif_result = convert_issue_to_result(issue, database)?;
        results.push(sarif_result);
    }

    Ok(Sarif::builder()
        .version(Version::V2_1_0.to_string())
        .runs(vec![Run::builder().tool(tool).results(results).build()])
        .build())
}

fn convert_issue_to_result(issue: &Issue, database: &ReadDatabase) -> Result<SarifResult, ReportingError> {
    let level = level_to_sarif(issue.level);

    let message = if !issue.notes.is_empty() || issue.help.is_some() {
        let mut markdown = issue.message.clone();

        if !issue.notes.is_empty() {
            markdown.push_str("\n\n**Notes:**\n");
            for note in &issue.notes {
                markdown.push_str("- ");
                markdown.push_str(note);
                markdown.push('\n');
            }
        }

        if let Some(help) = &issue.help {
            markdown.push_str("\n**Help:** ");
            markdown.push_str(help);
        }

        Message::builder().text(issue.message.clone()).markdown(markdown).build()
    } else {
        Message::builder().text(issue.message.clone()).build()
    };

    let mut locations = Vec::new();
    for annotation in &issue.annotations {
        if annotation.is_primary() {
            let location = annotation_to_location(annotation, database)?;
            locations.insert(0, location);
        } else {
            let location = annotation_to_location(annotation, database)?;
            locations.push(location);
        }
    }

    let result = match (&issue.code, locations.is_empty()) {
        (Some(code), false) => {
            SarifResult::builder().message(message).level(level).rule_id(code.clone()).locations(locations).build()
        }
        (Some(code), true) => SarifResult::builder().message(message).level(level).rule_id(code.clone()).build(),
        (None, false) => SarifResult::builder().message(message).level(level).locations(locations).build(),
        (None, true) => SarifResult::builder().message(message).level(level).build(),
    };

    Ok(result)
}

fn annotation_to_location(annotation: &Annotation, database: &ReadDatabase) -> Result<Location, ReportingError> {
    let file = database.get(&annotation.span.file_id())?;

    let start_line = file.line_number(annotation.span.start.offset);
    let start_column = file.column_number(annotation.span.start.offset);
    let end_line = file.line_number(annotation.span.end.offset);
    let end_column = file.column_number(annotation.span.end.offset);

    let uri = if let Some(path) = &file.path { path.to_string_lossy().to_string() } else { file.name.to_string() };

    let artifact_location = ArtifactLocation::builder().uri(uri).build();

    let region = Region::builder()
        .start_line(i64::from(start_line) + 1)
        .start_column(i64::from(start_column) + 1)
        .end_line(i64::from(end_line) + 1)
        .end_column(i64::from(end_column) + 1)
        .build();

    let physical_location = PhysicalLocation::builder().artifact_location(artifact_location).region(region).build();

    let location = if let Some(msg) = &annotation.message {
        Location::builder()
            .physical_location(physical_location)
            .message(Message::builder().text(msg.clone()).build())
            .build()
    } else {
        Location::builder().physical_location(physical_location).build()
    };

    Ok(location)
}

fn level_to_sarif(level: Level) -> ResultLevel {
    match level {
        Level::Error => ResultLevel::Error,
        Level::Warning => ResultLevel::Warning,
        Level::Help | Level::Note => ResultLevel::Note,
    }
}
