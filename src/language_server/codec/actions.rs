//! Convert code actions and record document hashes for legacy commands.

use foldhash::HashMap;
use serde::{Deserialize, Serialize};
use tower_lsp_server::gen_lsp_types::{
    Code, CodeAction, CodeActionKind, CodeActionResponse, Command, Diagnostic, DiagnosticSeverity, Message, TextEdit,
    Uri, WorkspaceEdit,
};
use xxhash_rust::xxh3::xxh3_128;

use mago_database::file::File as MagoFile;
use mago_database::{Database, DatabaseReader};
use mago_server::{CodeActionItem, DiagnosticData, Severity};

use crate::language_server::diagnostics::published_message_matches;
use crate::language_server::position::range_at_offsets;

pub const APPLY_CODE_ACTION: &str = "mago.applyCodeAction";

#[derive(Clone, Deserialize, Serialize)]
pub struct ActionDocument {
    pub uri: Uri,
    pub content_hash: String,
}

impl ActionDocument {
    fn new(uri: Uri, file: &MagoFile) -> Self {
        Self { uri, content_hash: format!("{:032x}", xxh3_128(&file.contents)) }
    }

    pub fn matches(&self, file: &MagoFile) -> bool {
        self.content_hash == format!("{:032x}", xxh3_128(&file.contents))
    }
}

/// The requested document and its current client diagnostics.
pub struct CodeActionSource<'a> {
    pub uri: &'a Uri,
    pub file: &'a MagoFile,
    pub diagnostics: &'a [Diagnostic],
    pub markdown_diagnostics: bool,
}

/// Convert code actions to the negotiated format, omitting actions with no edits.
pub fn code_actions(
    database: &Database<'_>,
    document_uri: impl Fn(&MagoFile) -> Option<Uri>,
    items: Vec<CodeActionItem>,
    source: CodeActionSource<'_>,
    literals: bool,
    preferred_support: bool,
) -> Vec<CodeActionResponse> {
    let source_document = (!literals).then(|| ActionDocument::new(source.uri.clone(), source.file));
    items
        .into_iter()
        .filter_map(|item| {
            let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::default();
            let mut snapshots: Vec<_> = source_document.iter().cloned().collect();
            for (file_id, replacements) in item.edits {
                let file = database.get(&file_id).ok()?;
                let uri = document_uri(&file)?;
                let edits: Vec<TextEdit> = replacements
                    .iter()
                    .map(|replacement| TextEdit {
                        range: range_at_offsets(&file, replacement.range.start, replacement.range.end),
                        new_text: replacement.new_text.clone(),
                    })
                    .collect();
                if !edits.is_empty() {
                    if !literals {
                        if let Some(snapshot) = snapshots.iter().find(|snapshot| snapshot.uri == uri) {
                            if !snapshot.matches(&file) {
                                return None;
                            }
                        } else {
                            snapshots.push(ActionDocument::new(uri.clone(), &file));
                        }
                    }
                    changes.entry(uri).or_default().extend(edits);
                }
            }

            if changes.is_empty() {
                return None;
            }

            let edit = WorkspaceEdit {
                changes: Some(changes.into_iter().collect()),
                document_changes: None,
                change_annotations: None,
            };
            if !literals {
                return Some(CodeActionResponse::Command(Command {
                    title: item.title,
                    command: APPLY_CODE_ACTION.into(),
                    tooltip: None,
                    arguments: Some(vec![
                        serde_json::json!(source.uri),
                        serde_json::json!(edit),
                        serde_json::json!(snapshots),
                    ]),
                }));
            }
            let diagnostics =
                item.diagnostic.and_then(|data| diagnostic(database, data, &source)).map(|diagnostic| vec![diagnostic]);
            let preferred =
                preferred_support && item.kind == mago_server::CodeActionKind::QuickFix && diagnostics.is_some();

            Some(CodeActionResponse::CodeAction(CodeAction {
                title: item.title,
                kind: Some(CodeActionKind::from(item.kind.as_str())),
                diagnostics,
                edit: Some(edit),
                command: None,
                is_preferred: preferred.then_some(true),
                disabled: None,
                data: None,
                tags: None,
            }))
        })
        .collect()
}

fn diagnostic(database: &Database<'_>, data: DiagnosticData, source: &CodeActionSource<'_>) -> Option<Diagnostic> {
    let file = database.get(&data.file).ok()?;
    let range = range_at_offsets(&file, data.range.start, data.range.end);
    let severity = Some(severity(data.severity));
    let code = data.code.map(Code::String);
    // Context diagnostics belong only to the requested document.
    if file.id == source.file.id
        && let Some(published) = source.diagnostics.iter().find(|published| {
            published.source.as_deref() == Some("mago")
                && published.code == code
                && published.range == range
                && published.severity == severity
                && (source.markdown_diagnostics || matches!(&published.message, Message::String(_)))
                && published_message_matches(&data.message, &published.message)
        })
    {
        return Some(published.clone());
    }
    Some(Diagnostic {
        range,
        severity,
        code,
        source: Some("mago".into()),
        message: Message::String(data.message),
        ..Diagnostic::default()
    })
}

fn severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::Error,
        Severity::Warning => DiagnosticSeverity::Warning,
        Severity::Information => DiagnosticSeverity::Information,
        Severity::Hint => DiagnosticSeverity::Hint,
    }
}

#[cfg(test)]
mod tests;
