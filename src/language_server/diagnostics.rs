//! Issue → LSP `Diagnostic` conversion and publish helpers.

use foldhash::HashMap;
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::DiagnosticRelatedInformation;
use tower_lsp::lsp_types::DiagnosticSeverity;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::NumberOrString;
use tower_lsp::lsp_types::Url;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;

use crate::language_server::position::range_at_offsets;

/// Convert an [`AnalysisResult`] (analyzer issues) plus an iterator of
/// linter issues into a per-file diagnostic map keyed by URL.
///
/// Issues without a primary annotation, or whose file has no on-disk path,
/// are dropped; the LSP client cannot navigate to them.
pub fn build_diagnostics<'a, I>(
    database: &Database<'_>,
    result: &AnalysisResult,
    lint_issues: I,
) -> HashMap<Url, Vec<Diagnostic>>
where
    I: IntoIterator<Item = &'a IssueCollection>,
{
    let mut by_url: HashMap<Url, Vec<Diagnostic>> = HashMap::default();

    for issue in result.issues.iter() {
        push_issue(database, &mut by_url, issue);
    }

    for collection in lint_issues {
        for issue in collection.iter() {
            push_issue(database, &mut by_url, issue);
        }
    }

    by_url
}

fn push_issue(database: &Database<'_>, by_url: &mut HashMap<Url, Vec<Diagnostic>>, issue: &Issue) {
    let Some(annotation) = primary_annotation(issue) else {
        return;
    };

    let Ok(file) = database.get(&annotation.span.file_id) else {
        return;
    };

    let Some(path) = &file.path else {
        return;
    };

    let Ok(url) = Url::from_file_path(path) else {
        return;
    };

    let range = range_at_offsets(&file, annotation.span.start.offset, annotation.span.end.offset);

    let related = secondary_related_info(database, issue);
    let message = render_message(issue);

    by_url.entry(url).or_default().push(Diagnostic {
        range,
        severity: Some(level_to_severity(issue.level)),
        code: issue.code.clone().map(NumberOrString::String),
        source: Some("mago".into()),
        message,
        related_information: if related.is_empty() { None } else { Some(related) },
        ..Diagnostic::default()
    });
}

fn render_message(issue: &Issue) -> String {
    let mut out = issue.message.clone();
    for note in &issue.notes {
        out.push_str("\n\nnote: ");
        out.push_str(note);
    }
    if let Some(help) = &issue.help {
        out.push_str("\n\nhelp: ");
        out.push_str(help);
    }
    out
}

fn secondary_related_info(database: &Database<'_>, issue: &Issue) -> Vec<DiagnosticRelatedInformation> {
    let mut related = Vec::new();

    for annotation in &issue.annotations {
        if matches!(annotation.kind, AnnotationKind::Primary) {
            continue;
        }

        let Ok(file) = database.get(&annotation.span.file_id) else {
            continue;
        };
        let Some(path) = &file.path else {
            continue;
        };
        let Ok(url) = Url::from_file_path(path) else {
            continue;
        };

        let range = range_at_offsets(&file, annotation.span.start.offset, annotation.span.end.offset);
        let message = annotation.message.clone().unwrap_or_default().to_string();

        related.push(DiagnosticRelatedInformation { location: Location { uri: url, range }, message });
    }

    related
}

fn primary_annotation(issue: &Issue) -> Option<&Annotation> {
    issue
        .annotations
        .iter()
        .find(|annotation| matches!(annotation.kind, AnnotationKind::Primary))
        .or_else(|| issue.annotations.first())
}

#[must_use]
pub fn level_to_severity(level: Level) -> DiagnosticSeverity {
    match level {
        Level::Error => DiagnosticSeverity::ERROR,
        Level::Warning => DiagnosticSeverity::WARNING,
        Level::Help => DiagnosticSeverity::HINT,
        Level::Note => DiagnosticSeverity::INFORMATION,
    }
}
