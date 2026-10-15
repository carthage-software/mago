//! Convert server values from byte offsets to LSP positions and response types.
//! Hover and completion rendering live in their own modules.

mod actions;
mod code_lens;
mod completion;
mod documentation;
mod hover;

pub use actions::{APPLY_CODE_ACTION, ActionDocument, CodeActionSource, code_actions};
pub use code_lens::{CodeLensData, code_lens, reference_command};
pub use completion::{completion, signature_help};
pub use hover::{HoverCodeLanguage, hover};

use tower_lsp_server::gen_lsp_types::BaseSymbolInformation;
use tower_lsp_server::gen_lsp_types::DocumentHighlight;
use tower_lsp_server::gen_lsp_types::DocumentHighlightKind;
use tower_lsp_server::gen_lsp_types::DocumentLink;
use tower_lsp_server::gen_lsp_types::DocumentSymbol;
use tower_lsp_server::gen_lsp_types::DocumentSymbolResponse;
use tower_lsp_server::gen_lsp_types::FoldingRange;
use tower_lsp_server::gen_lsp_types::FoldingRangeKind;
use tower_lsp_server::gen_lsp_types::InlayHint;
use tower_lsp_server::gen_lsp_types::InlayHintKind;
use tower_lsp_server::gen_lsp_types::Label;
use tower_lsp_server::gen_lsp_types::Location;
use tower_lsp_server::gen_lsp_types::Range;
use tower_lsp_server::gen_lsp_types::SelectionRange;
use tower_lsp_server::gen_lsp_types::SymbolInformation;
use tower_lsp_server::gen_lsp_types::SymbolKind as LspSymbolKind;
use tower_lsp_server::gen_lsp_types::TextEdit;
use tower_lsp_server::gen_lsp_types::Uri;

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_server::DocumentLinkItem;
use mago_server::DocumentSymbolItem;
use mago_server::FoldKind;
use mago_server::FoldRange;
use mago_server::FormattedDocument;
use mago_server::HintKind;
use mago_server::InlayHintItem;
use mago_server::SelectionRangeItem;
use mago_server::SymbolKind;
use mago_server::SymbolLocation;
use mago_server::SymbolRank;
use mago_server::TextReplacement;
use mago_server::WorkspaceSymbolItem;
use mago_server::capabilities::highlights::{Highlight, HighlightKind};

use crate::language_server::position::position_at_offset;
use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

/// Convert a [`SymbolLocation`] to an LSP [`Location`], resolving its file to a
/// `file://` URI and its byte span to a line/character range. `None` if the
/// file has no on-disk path (e.g. a builtin stub) or isn't in the database.
pub fn location(workspace: &WorkspaceState, location: &SymbolLocation) -> Option<Location> {
    let file = workspace.database().get(&location.file).ok()?;
    let uri = workspace.document_uri(&file)?;
    Some(Location { uri, range: range_at_offsets(&file, location.range.start, location.range.end) })
}

/// Convert highlights in one file to UTF-16 ranges.
pub fn document_highlights(file: &MagoFile, items: Vec<Highlight>) -> Vec<DocumentHighlight> {
    items
        .into_iter()
        .map(|item| DocumentHighlight {
            range: range_at_offsets(file, item.range.start, item.range.end),
            kind: Some(match item.kind {
                HighlightKind::Text => DocumentHighlightKind::Text,
                HighlightKind::Read => DocumentHighlightKind::Read,
                HighlightKind::Write => DocumentHighlightKind::Write,
            }),
        })
        .collect()
}

/// Convert a document link after its target URI has been resolved.
pub fn document_link(file: &MagoFile, item: DocumentLinkItem, target: Uri) -> DocumentLink {
    DocumentLink {
        range: range_at_offsets(file, item.range.start, item.range.end),
        target: Some(target),
        tooltip: Some(item.tooltip),
        data: None,
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
    TextEdit { range: range_at_offsets(file, 0, file.size), new_text: document.new_text }
}

/// Convert a byte-range replacement into an LSP edit.
pub fn text_edit(file: &MagoFile, edit: TextReplacement) -> TextEdit {
    TextEdit { range: range_at_offsets(file, edit.range.start, edit.range.end), new_text: edit.new_text }
}

/// Convert workspace-symbol matches to ranked LSP [`SymbolInformation`],
/// dropping any whose location can't be resolved to a URI.
pub fn symbols(workspace: &WorkspaceState, items: Vec<WorkspaceSymbolItem>) -> Vec<(SymbolRank, SymbolInformation)> {
    items
        .into_iter()
        .filter_map(|item| {
            let location = location(workspace, &item.location)?;
            #[allow(deprecated)]
            Some((
                item.rank,
                SymbolInformation {
                    deprecated: None,
                    location,
                    base_symbol_information: BaseSymbolInformation {
                        name: item.name,
                        kind: symbol_kind(item.kind),
                        tags: None,
                        container_name: item.container,
                    },
                },
            ))
        })
        .collect()
}

fn symbol_kind(kind: SymbolKind) -> LspSymbolKind {
    match kind {
        SymbolKind::Class | SymbolKind::Trait => LspSymbolKind::Class,
        SymbolKind::Interface => LspSymbolKind::Interface,
        SymbolKind::Enum => LspSymbolKind::Enum,
        SymbolKind::Function => LspSymbolKind::Function,
        SymbolKind::Constant => LspSymbolKind::Constant,
        SymbolKind::Namespace => LspSymbolKind::Namespace,
        SymbolKind::Method => LspSymbolKind::Method,
        SymbolKind::Constructor => LspSymbolKind::Constructor,
        SymbolKind::Property => LspSymbolKind::Property,
        SymbolKind::EnumMember => LspSymbolKind::EnumMember,
    }
}

/// Convert an outline to the tree or flat response supported by the client.
pub fn document_symbols(
    file: &MagoFile,
    uri: &Uri,
    items: Vec<DocumentSymbolItem>,
    hierarchical: bool,
) -> DocumentSymbolResponse {
    if hierarchical {
        DocumentSymbolResponse::DocumentSymbolList(items.into_iter().map(|item| document_symbol(file, item)).collect())
    } else {
        let mut out = Vec::new();
        flat_document_symbols(file, uri, items, None, &mut out);
        DocumentSymbolResponse::SymbolInformationList(out)
    }
}

#[allow(deprecated)]
fn document_symbol(file: &MagoFile, item: DocumentSymbolItem) -> DocumentSymbol {
    DocumentSymbol {
        name: item.name,
        kind: symbol_kind(item.kind),
        detail: None,
        tags: None,
        deprecated: None,
        range: range_at_offsets(file, item.range.start, item.range.end),
        selection_range: range_at_offsets(file, item.selection_range.start, item.selection_range.end),
        children: (!item.children.is_empty())
            .then(|| item.children.into_iter().map(|child| document_symbol(file, child)).collect()),
    }
}

#[allow(deprecated)]
fn flat_document_symbols(
    file: &MagoFile,
    uri: &Uri,
    items: Vec<DocumentSymbolItem>,
    container: Option<&str>,
    out: &mut Vec<SymbolInformation>,
) {
    for item in items {
        out.push(SymbolInformation {
            deprecated: None,
            location: Location {
                uri: uri.clone(),
                range: range_at_offsets(file, item.selection_range.start, item.selection_range.end),
            },
            base_symbol_information: BaseSymbolInformation {
                name: item.name.clone(),
                kind: symbol_kind(item.kind),
                tags: None,
                container_name: container.map(str::to_owned),
            },
        });
        flat_document_symbols(file, uri, item.children, Some(&item.name), out);
    }
}

/// Convert byte-offset [`InlayHintItem`]s to LSP [`InlayHint`]s.
pub fn inlay_hints(file: &MagoFile, items: Vec<InlayHintItem>) -> Vec<InlayHint> {
    items
        .into_iter()
        .map(|item| InlayHint {
            position: position_at_offset(file, item.offset),
            label: Label::String(item.label),
            kind: Some(match item.kind {
                HintKind::Parameter => InlayHintKind::Parameter,
                HintKind::Type => InlayHintKind::Type,
            }),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: Some(true),
            data: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use mago_server::Range as ByteRange;
    use mago_server::SignatureInfo;
    use tower_lsp_server::gen_lsp_types::ParameterInformationLabel;
    use tower_lsp_server::gen_lsp_types::Position;

    use super::*;

    fn file(contents: &'static str) -> MagoFile {
        MagoFile::ephemeral(b"test.php".as_slice().into(), contents.as_bytes().into())
    }

    #[test]
    fn formatting_uses_utf16_at_eof() {
        let file = file("<?php\n// café😀");
        let edit = formatting(&file, FormattedDocument { new_text: "formatted".into() });
        assert_eq!(edit.range, Range::new(Position::new(0, 0), Position::new(1, 9)));
    }

    #[test]
    fn signature_parameter_offsets_use_utf16() {
        let label = "function café(string $名, int $😀)";
        let info = signature_help(
            vec![SignatureInfo {
                label: label.into(),
                documentation: None,
                parameters: vec![
                    ByteRange::new(label.find("string").unwrap() as u32, label.find(',').unwrap() as u32),
                    ByteRange::new(label.find("int").unwrap() as u32, label.find(')').unwrap() as u32),
                ],
                active_parameter: 1,
            }],
            true,
            true,
            false,
        )
        .unwrap();
        let parameters = info.signatures[0].parameters.as_ref().unwrap();
        assert_eq!(parameters[0].label, ParameterInformationLabel::Tuple((14, 23)));
        assert_eq!(parameters[1].label, ParameterInformationLabel::Tuple((25, 32)));
    }
}
