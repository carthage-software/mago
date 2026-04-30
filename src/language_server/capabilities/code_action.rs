//! `textDocument/codeAction`; surfaces the autofixes that mago's analyzer
//! and linter already attach to issues.
//!
//! For every diagnostic in the requested range we emit one `quickfix`
//! action per fix bundle, each carrying the [`WorkspaceEdit`] that resolves
//! the issue.

use foldhash::HashMap;
use tower_lsp::lsp_types::CodeAction;
use tower_lsp::lsp_types::CodeActionKind;
use tower_lsp::lsp_types::CodeActionOrCommand;
use tower_lsp::lsp_types::CodeActionParams;
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::TextEdit;
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::WorkspaceEdit;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_text_edit::TextEdit as MagoTextEdit;

use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

pub fn compute(workspace: &WorkspaceState, params: &CodeActionParams) -> Vec<CodeActionOrCommand> {
    let request_range = params.range;
    let request_uri = &params.text_document.uri;

    let mut actions = Vec::new();

    if let Some(issues) = workspace.service.last_issues() {
        for issue in issues.iter() {
            if issue.edits.is_empty() {
                continue;
            }
            if !issue_overlaps(workspace, issue, request_uri, request_range) {
                continue;
            }
            actions.extend(issue_to_actions(&workspace.database, issue));
        }
    }

    for (_, analysis) in workspace.file_analyses.values() {
        for issue in analysis.lint_issues.iter() {
            if issue.edits.is_empty() {
                continue;
            }
            if !issue_overlaps(workspace, issue, request_uri, request_range) {
                continue;
            }
            actions.extend(issue_to_actions(&workspace.database, issue));
        }
    }

    actions
}

fn issue_overlaps(workspace: &WorkspaceState, issue: &Issue, uri: &Url, requested: Range) -> bool {
    let primary = primary_annotation(issue);
    let Some(primary) = primary else {
        return false;
    };
    let Ok(file) = workspace.database.get(&primary.span.file_id) else {
        return false;
    };
    let Some(path) = &file.path else {
        return false;
    };
    let Ok(issue_uri) = Url::from_file_path(path) else {
        return false;
    };
    if issue_uri != *uri {
        return false;
    }
    let issue_range = range_at_offsets(&file, primary.span.start.offset, primary.span.end.offset);
    ranges_overlap(issue_range, requested)
}

fn issue_to_actions(database: &Database<'_>, issue: &Issue) -> Vec<CodeActionOrCommand> {
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::default();
    for (file_id, edits) in &issue.edits {
        let Ok(file) = database.get(file_id) else { continue };
        let Some(path) = &file.path else { continue };
        let Ok(target_uri) = Url::from_file_path(path) else { continue };

        let converted: Vec<TextEdit> = edits.iter().map(|e| convert_edit(&file, e)).collect();
        if converted.is_empty() {
            continue;
        }

        changes.entry(target_uri).or_default().extend(converted);
    }

    if changes.is_empty() {
        return Vec::new();
    }

    let title = issue.help.clone().unwrap_or_else(|| format!("Apply mago fix: {}", issue.message));
    let diagnostic = issue_to_diagnostic(database, issue);

    vec![CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: diagnostic.map(|d| vec![d]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes.into_iter().collect()),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })]
}

fn convert_edit(file: &MagoFile, edit: &MagoTextEdit) -> TextEdit {
    TextEdit { range: range_at_offsets(file, edit.range.start, edit.range.end), new_text: edit.new_text.clone() }
}

fn issue_to_diagnostic(database: &Database<'_>, issue: &Issue) -> Option<Diagnostic> {
    let primary = primary_annotation(issue)?;
    let file = database.get(&primary.span.file_id).ok()?;
    let range = range_at_offsets(&file, primary.span.start.offset, primary.span.end.offset);

    Some(Diagnostic {
        range,
        severity: Some(crate::language_server::diagnostics::level_to_severity(issue.level)),
        code: issue.code.clone().map(tower_lsp::lsp_types::NumberOrString::String),
        source: Some("mago".into()),
        message: issue.message.clone(),
        ..Diagnostic::default()
    })
}

fn primary_annotation(issue: &Issue) -> Option<&mago_reporting::Annotation> {
    issue.annotations.iter().find(|a| matches!(a.kind, AnnotationKind::Primary)).or_else(|| issue.annotations.first())
}

fn ranges_overlap(a: Range, b: Range) -> bool {
    !(a.end < b.start || b.end < a.start)
}
