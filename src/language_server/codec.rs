//! Translates `mago_server` domain values into LSP wire types.
//!
//! The server speaks byte offsets and plain structs; this is the single place
//! that turns them into `tower-lsp` types (resolving `FileId`s to `file://`
//! URIs and byte offsets to line/character positions). Keeping every
//! domain→LSP conversion here is what lets the capability handlers stay thin.

use foldhash::HashMap;
use tower_lsp_server::ls_types::CodeAction;
use tower_lsp_server::ls_types::CodeActionKind;
use tower_lsp_server::ls_types::CodeActionOrCommand;
use tower_lsp_server::ls_types::CodeLens;
use tower_lsp_server::ls_types::Command;
use tower_lsp_server::ls_types::CompletionItem;
use tower_lsp_server::ls_types::CompletionItemKind;
use tower_lsp_server::ls_types::CompletionList as LspCompletionList;
use tower_lsp_server::ls_types::CompletionResponse;
use tower_lsp_server::ls_types::CompletionTextEdit;
use tower_lsp_server::ls_types::Diagnostic;
use tower_lsp_server::ls_types::DiagnosticSeverity;
use tower_lsp_server::ls_types::DocumentLink;
use tower_lsp_server::ls_types::Documentation;
use tower_lsp_server::ls_types::FoldingRange;
use tower_lsp_server::ls_types::FoldingRangeKind;
use tower_lsp_server::ls_types::Hover;
use tower_lsp_server::ls_types::HoverContents;
use tower_lsp_server::ls_types::InlayHint;
use tower_lsp_server::ls_types::InlayHintKind;
use tower_lsp_server::ls_types::InlayHintLabel;
use tower_lsp_server::ls_types::InsertTextFormat;
use tower_lsp_server::ls_types::Location;
use tower_lsp_server::ls_types::MarkupContent;
use tower_lsp_server::ls_types::MarkupKind;
use tower_lsp_server::ls_types::NumberOrString;
use tower_lsp_server::ls_types::ParameterInformation;
use tower_lsp_server::ls_types::ParameterLabel;
use tower_lsp_server::ls_types::Position;
use tower_lsp_server::ls_types::Range;
use tower_lsp_server::ls_types::SelectionRange;
use tower_lsp_server::ls_types::SemanticToken;
use tower_lsp_server::ls_types::SignatureHelp;
use tower_lsp_server::ls_types::SignatureInformation;
use tower_lsp_server::ls_types::SymbolInformation;
use tower_lsp_server::ls_types::SymbolKind as LspSymbolKind;
use tower_lsp_server::ls_types::TextEdit;
use tower_lsp_server::ls_types::Uri;
use tower_lsp_server::ls_types::WorkspaceEdit;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_server::CodeActionItem;
use mago_server::CodeLensItem;
use mago_server::CompletionEntry;
use mago_server::CompletionKind;
use mago_server::CompletionList;
use mago_server::DiagnosticData;
use mago_server::DocumentLinkItem;
use mago_server::FoldKind;
use mago_server::FoldRange;
use mago_server::FormattedDocument;
use mago_server::HintKind;
use mago_server::HoverInfo;
use mago_server::InlayHintItem;
use mago_server::SelectionRangeItem;
use mago_server::SemanticTokenItem;
use mago_server::Severity;
use mago_server::SignatureInfo;
use mago_server::SymbolKind;
use mago_server::SymbolLocation;
use mago_server::WorkspaceSymbolItem;

use crate::language_server::position::position_at_offset;
use crate::language_server::position::range_at_offsets;

/// Convert a [`SymbolLocation`] to an LSP [`Location`], resolving its file to a
/// `file://` URI and its byte span to a line/character range. `None` if the
/// file has no on-disk path (e.g. a builtin stub) or isn't in the database.
pub fn location(database: &Database<'_>, location: &SymbolLocation) -> Option<Location> {
    let file = database.get(&location.file).ok()?;
    let path = file.path.as_ref()?;
    let uri = Uri::from_file_path(path)?;
    Some(Location { uri, range: range_at_offsets(&file, location.range.start, location.range.end) })
}

/// Convert a batch of [`SymbolLocation`]s to LSP [`Location`]s, dropping any
/// that can't be resolved to a `file://` URI.
pub fn locations(database: &Database<'_>, locations: &[SymbolLocation]) -> Vec<Location> {
    locations.iter().filter_map(|location| self::location(database, location)).collect()
}

/// Convert [`HoverInfo`] to an LSP [`Hover`]. The span is in `file`, so its
/// byte range is resolved against that file directly.
pub fn hover(file: &MagoFile, info: HoverInfo) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: info.markdown }),
        range: Some(range_at_offsets(file, info.range.start, info.range.end)),
    }
}

/// Convert [`DocumentLinkItem`]s to LSP [`DocumentLink`]s. The link's source
/// range is in `file`; its target is resolved to a `file://` URI via `database`.
pub fn document_links(file: &MagoFile, database: &Database<'_>, items: Vec<DocumentLinkItem>) -> Vec<DocumentLink> {
    items
        .into_iter()
        .filter_map(|item| {
            let target_file = database.get(&item.target).ok()?;
            let path = target_file.path.as_ref()?;
            let uri = Uri::from_file_path(path)?;
            Some(DocumentLink {
                range: range_at_offsets(file, item.range.start, item.range.end),
                target: Some(uri),
                tooltip: Some(item.tooltip),
                data: None,
            })
        })
        .collect()
}

/// Convert [`SignatureInfo`] to an LSP [`SignatureHelp`]. Parameter ranges are
/// offsets within the signature label string, not file offsets.
pub fn signature_help(info: SignatureInfo) -> SignatureHelp {
    let parameters = info
        .parameters
        .iter()
        .map(|range| ParameterInformation {
            label: ParameterLabel::LabelOffsets([range.start, range.end]),
            documentation: None,
        })
        .collect();

    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: info.label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: Some(info.active_parameter),
    }
}

/// Convert byte-offset [`FoldRange`]s to LSP line-based [`FoldingRange`]s.
pub fn folding_ranges(file: &MagoFile, ranges: Vec<FoldRange>) -> Vec<FoldingRange> {
    ranges
        .into_iter()
        .map(|range| FoldingRange {
            start_line: file.line_number(range.range.start),
            start_character: None,
            end_line: file.line_number(range.range.end),
            end_character: None,
            kind: range.kind.map(|kind| match kind {
                FoldKind::Comment => FoldingRangeKind::Comment,
                FoldKind::Region => FoldingRangeKind::Region,
            }),
            collapsed_text: None,
        })
        .collect()
}

/// Convert each [`SelectionRangeItem`] (innermost-first span chain) into a
/// nested LSP [`SelectionRange`].
pub fn selection_ranges(file: &MagoFile, items: Vec<SelectionRangeItem>) -> Vec<SelectionRange> {
    items
        .into_iter()
        .map(|item| {
            let mut current: Option<SelectionRange> = None;
            for span in item.ranges.into_iter().rev() {
                current = Some(SelectionRange {
                    range: range_at_offsets(file, span.start, span.end),
                    parent: current.map(Box::new),
                });
            }
            current.unwrap_or(SelectionRange { range: Range::default(), parent: None })
        })
        .collect()
}

/// Convert a [`FormattedDocument`] to a whole-file replacement [`TextEdit`].
pub fn formatting(file: &MagoFile, document: FormattedDocument) -> TextEdit {
    let end_line = if file.lines.is_empty() { 0 } else { (file.lines.len() - 1) as u32 };
    let end_character = file.size - file.get_line_start_offset(end_line).unwrap_or(0);
    TextEdit {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: end_line, character: end_character },
        },
        new_text: document.new_text,
    }
}

/// Convert workspace-symbol matches to LSP [`SymbolInformation`], dropping any
/// whose location can't be resolved to a URI.
pub fn symbols(database: &Database<'_>, items: Vec<WorkspaceSymbolItem>) -> Vec<SymbolInformation> {
    items
        .into_iter()
        .filter_map(|item| {
            let location = location(database, &item.location)?;
            #[allow(deprecated)]
            Some(SymbolInformation {
                name: item.name,
                kind: symbol_kind(item.kind),
                tags: None,
                deprecated: None,
                location,
                container_name: None,
            })
        })
        .collect()
}

fn symbol_kind(kind: SymbolKind) -> LspSymbolKind {
    match kind {
        SymbolKind::Class | SymbolKind::Trait => LspSymbolKind::CLASS,
        SymbolKind::Interface => LspSymbolKind::INTERFACE,
        SymbolKind::Enum => LspSymbolKind::ENUM,
        SymbolKind::Function => LspSymbolKind::FUNCTION,
        SymbolKind::Constant => LspSymbolKind::CONSTANT,
    }
}

/// Convert code-lens items to LSP [`CodeLens`]es, anchoring each at its offset
/// and rendering a "N references" title.
pub fn code_lens(file: &MagoFile, items: Vec<CodeLensItem>) -> Vec<CodeLens> {
    items
        .into_iter()
        .map(|item| {
            let position = position_at_offset(file, item.offset);
            let count = item.reference_count;
            let title = format!("{count} reference{}", if count == 1 { "" } else { "s" });
            CodeLens {
                range: Range { start: position, end: position },
                command: Some(Command { title, command: String::new(), arguments: None }),
                data: None,
            }
        })
        .collect()
}

/// Convert code-action items to LSP quickfix [`CodeActionOrCommand`]s,
/// resolving each per-file edit set into a [`WorkspaceEdit`]. Actions whose
/// edits resolve to no files are dropped.
pub fn code_actions(database: &Database<'_>, items: Vec<CodeActionItem>) -> Vec<CodeActionOrCommand> {
    items
        .into_iter()
        .filter_map(|item| {
            let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::default();
            for (file_id, replacements) in item.edits {
                let Ok(file) = database.get(&file_id) else { continue };
                let Some(path) = file.path.as_ref() else { continue };
                let Some(uri) = Uri::from_file_path(path) else { continue };
                let edits: Vec<TextEdit> = replacements
                    .iter()
                    .map(|replacement| TextEdit {
                        range: range_at_offsets(&file, replacement.range.start, replacement.range.end),
                        new_text: replacement.new_text.clone(),
                    })
                    .collect();
                if !edits.is_empty() {
                    changes.entry(uri).or_default().extend(edits);
                }
            }

            if changes.is_empty() {
                return None;
            }

            let diagnostics = item.diagnostic.and_then(|data| diagnostic(database, data)).map(|d| vec![d]);

            Some(CodeActionOrCommand::CodeAction(CodeAction {
                title: item.title,
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes.into_iter().collect()),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }))
        })
        .collect()
}

fn diagnostic(database: &Database<'_>, data: DiagnosticData) -> Option<Diagnostic> {
    let file = database.get(&data.file).ok()?;
    Some(Diagnostic {
        range: range_at_offsets(&file, data.range.start, data.range.end),
        severity: Some(severity(data.severity)),
        code: data.code.map(NumberOrString::String),
        source: Some("mago".into()),
        message: data.message,
        ..Diagnostic::default()
    })
}

fn severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Information => DiagnosticSeverity::INFORMATION,
        Severity::Hint => DiagnosticSeverity::HINT,
    }
}

/// Convert a domain [`CompletionList`] to an LSP [`CompletionResponse`].
pub fn completion(file: &MagoFile, list: CompletionList) -> CompletionResponse {
    let items = list.items.into_iter().map(|entry| completion_item(file, entry)).collect();
    CompletionResponse::List(LspCompletionList { is_incomplete: list.is_incomplete, items })
}

fn completion_item(file: &MagoFile, entry: CompletionEntry) -> CompletionItem {
    // A `replace` span rewrites a range (e.g. the variable's `$name`) with the
    // label; everything else inserts at the cursor.
    let text_edit = entry.replace.map(|span| {
        CompletionTextEdit::Edit(TextEdit {
            range: range_at_offsets(file, span.start, span.end),
            new_text: entry.label.clone(),
        })
    });

    CompletionItem {
        label: entry.label,
        kind: Some(completion_kind(entry.kind)),
        detail: entry.detail,
        documentation: entry
            .documentation
            .map(|value| Documentation::MarkupContent(MarkupContent { kind: MarkupKind::PlainText, value })),
        insert_text: entry.insert_text,
        insert_text_format: entry.snippet.then_some(InsertTextFormat::SNIPPET),
        text_edit,
        sort_text: entry.sort_text,
        filter_text: entry.filter_text,
        ..CompletionItem::default()
    }
}

fn completion_kind(kind: CompletionKind) -> CompletionItemKind {
    match kind {
        CompletionKind::Variable => CompletionItemKind::VARIABLE,
        CompletionKind::Function => CompletionItemKind::FUNCTION,
        CompletionKind::Constant => CompletionItemKind::CONSTANT,
        CompletionKind::Class => CompletionItemKind::CLASS,
        CompletionKind::Interface => CompletionItemKind::INTERFACE,
        CompletionKind::Enum => CompletionItemKind::ENUM,
        CompletionKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
        CompletionKind::Field => CompletionItemKind::FIELD,
        CompletionKind::Method => CompletionItemKind::METHOD,
    }
}

/// Delta-encode absolute [`SemanticTokenItem`]s into the LSP wire format,
/// resolving byte offsets to line/character. Items must be in source order.
pub fn semantic_tokens(file: &MagoFile, items: Vec<SemanticTokenItem>) -> Vec<SemanticToken> {
    let mut out = Vec::with_capacity(items.len());
    let mut prev_line: u32 = 0;
    let mut prev_start: u32 = 0;

    for item in items {
        let line = file.line_number(item.offset);
        let line_start = file.get_line_start_offset(line).unwrap_or(item.offset);
        let column = item.offset - line_start;
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 { column - prev_start } else { column };

        out.push(SemanticToken {
            delta_line,
            delta_start,
            length: item.length,
            token_type: item.kind.index(),
            token_modifiers_bitset: 0,
        });

        prev_line = line;
        prev_start = column;
    }

    out
}

/// Convert byte-offset [`InlayHintItem`]s to LSP [`InlayHint`]s.
pub fn inlay_hints(file: &MagoFile, items: Vec<InlayHintItem>) -> Vec<InlayHint> {
    items
        .into_iter()
        .map(|item| InlayHint {
            position: position_at_offset(file, item.offset),
            label: InlayHintLabel::String(item.label),
            kind: Some(match item.kind {
                HintKind::Parameter => InlayHintKind::PARAMETER,
                HintKind::Type => InlayHintKind::TYPE,
            }),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: Some(true),
            data: None,
        })
        .collect()
}
