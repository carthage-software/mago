//! Encode completion insertions and call signatures for the client.

use tower_lsp_server::gen_lsp_types::ActiveParameter;
use tower_lsp_server::gen_lsp_types::CompletionItem;
use tower_lsp_server::gen_lsp_types::CompletionItemKind;
use tower_lsp_server::gen_lsp_types::CompletionItemLabelDetails;
use tower_lsp_server::gen_lsp_types::CompletionItemTag;
use tower_lsp_server::gen_lsp_types::CompletionItemTextEdit;
use tower_lsp_server::gen_lsp_types::CompletionList as LspCompletionList;
use tower_lsp_server::gen_lsp_types::CompletionResponse;
use tower_lsp_server::gen_lsp_types::Documentation;
use tower_lsp_server::gen_lsp_types::InsertTextFormat;
use tower_lsp_server::gen_lsp_types::InsertTextMode;
use tower_lsp_server::gen_lsp_types::MarkupContent;
use tower_lsp_server::gen_lsp_types::MarkupKind;
use tower_lsp_server::gen_lsp_types::ParameterInformation;
use tower_lsp_server::gen_lsp_types::ParameterInformationLabel;
use tower_lsp_server::gen_lsp_types::SignatureHelp;
use tower_lsp_server::gen_lsp_types::SignatureInformation;
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_database::file::File as MagoFile;
use mago_server::CompletionEntry;
use mago_server::CompletionInsert;
use mago_server::CompletionKind;
use mago_server::CompletionList;
use mago_server::SignatureInfo;

use crate::language_server::position::range_at_offsets;
use crate::language_server::position::utf16_offset_at_byte;

use super::hover::HoverCodeLanguage;

/// Convert [`SignatureInfo`] to an LSP [`SignatureHelp`]. Parameter ranges are
/// offsets within the signature label string, not file offsets.
pub fn signature_help(
    infos: Vec<SignatureInfo>,
    offset_labels: bool,
    active_parameters: bool,
    markdown: bool,
) -> Option<SignatureHelp> {
    let active_parameter = infos.first()?.active_parameter;
    let signatures = infos
        .into_iter()
        .map(|info| {
            let parameters = info
                .parameters
                .iter()
                .map(|range| ParameterInformation {
                    label: if offset_labels {
                        ParameterInformationLabel::Tuple((
                            utf16_offset_at_byte(info.label.as_bytes(), range.start),
                            utf16_offset_at_byte(info.label.as_bytes(), range.end),
                        ))
                    } else {
                        ParameterInformationLabel::String(
                            info.label[range.start as usize..range.end as usize].to_owned(),
                        )
                    },
                    documentation: None,
                })
                .collect();
            SignatureInformation {
                label: info.label,
                documentation: info.documentation.map(|document| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: if markdown { MarkupKind::Markdown } else { MarkupKind::PlainText },
                        value: super::documentation::render(&document, markdown),
                    })
                }),
                parameters: Some(parameters),
                active_parameter: active_parameters.then_some(ActiveParameter::Int(info.active_parameter)),
            }
        })
        .collect();
    Some(SignatureHelp {
        signatures,
        active_signature: Some(0),
        active_parameter: Some(ActiveParameter::Int(active_parameter)),
    })
}

/// Convert a domain [`CompletionList`] to an LSP [`CompletionResponse`].
/// A plain-text qualified insertion keeps the list incomplete: clients filter it by the inserted text.
#[allow(clippy::too_many_arguments)]
pub fn completion(
    file: &MagoFile,
    list: CompletionList,
    snippets: bool,
    markdown: Option<HoverCodeLanguage>,
    as_is: bool,
    insert_mode: Option<InsertTextMode>,
    supports_label_details: bool,
    supports_deprecated_tag: bool,
) -> CompletionResponse {
    let width = list.items.len().saturating_sub(1).checked_ilog10().map_or(1, |digits| digits as usize + 1);
    let is_incomplete = list.is_incomplete || list.items.iter().any(|entry| inserts_qualified_text(entry, snippets));
    let items = list
        .items
        .into_iter()
        .filter(|entry| snippets || !matches!(entry.insert, Some(CompletionInsert::Snippet(_))))
        .enumerate()
        .map(|(index, entry)| CompletionItem {
            sort_text: Some(format!("{index:0width$}")),
            ..completion_item(
                file,
                entry,
                snippets,
                markdown,
                as_is,
                insert_mode,
                supports_label_details,
                supports_deprecated_tag,
            )
        })
        .collect();
    CompletionResponse::CompletionList(LspCompletionList { is_incomplete, items, ..LspCompletionList::default() })
}

fn inserts_qualified_text(entry: &CompletionEntry, snippets: bool) -> bool {
    matches!(
        (&entry.insert, snippets),
        (Some(CompletionInsert::Text(text)), _) | (Some(CompletionInsert::Call(text)), false) if text.starts_with('\\')
    )
}

#[allow(clippy::too_many_arguments)]
fn completion_item(
    file: &MagoFile,
    entry: CompletionEntry,
    snippets: bool,
    markdown: Option<HoverCodeLanguage>,
    as_is: bool,
    insert_mode: Option<InsertTextMode>,
    supports_label_details: bool,
    supports_deprecated_tag: bool,
) -> CompletionItem {
    let (insert_text, snippet) = match entry.insert {
        Some(CompletionInsert::Snippet(mut text)) => {
            if as_is && let Some(range) = entry.replace {
                let prefix = &file.contents[..range.start as usize];
                let line_start = prefix.iter().rposition(|byte| *byte == b'\n').map_or(0, |i| i + 1);
                let indent: String = prefix[line_start..]
                    .iter()
                    .take_while(|byte| matches!(byte, b' ' | b'\t'))
                    .map(|byte| *byte as char)
                    .collect();
                text = text.replace('\n', &format!("\n{indent}"));
            }
            (Some(text), true)
        }
        Some(CompletionInsert::Call(name)) if snippets => {
            let mut text = String::with_capacity(name.len() + 4);
            for character in name.chars() {
                if matches!(character, '\\' | '$' | '}') {
                    text.push('\\');
                }
                text.push(character);
            }
            text.push_str("($1)");
            (Some(text), true)
        }
        Some(CompletionInsert::Text(text) | CompletionInsert::Call(text)) => (Some(text), false),
        None => (None, false),
    };
    // A replacement uses the insertion text, including snippets when present.
    let text_edit = entry.replace.map(|span| {
        CompletionItemTextEdit::TextEdit(TextEdit {
            range: range_at_offsets(file, span.start, span.end),
            new_text: insert_text.clone().unwrap_or_else(|| entry.label.clone()),
        })
    });
    let (detail, label_details) = if supports_label_details {
        (
            entry.detail,
            entry.namespace.map(|namespace| CompletionItemLabelDetails { detail: None, description: Some(namespace) }),
        )
    } else {
        (entry.detail.or(entry.namespace), None)
    };
    let documentation = if let Some(alternatives) = entry.member_alternatives {
        let descriptions: Vec<_> = alternatives
            .into_vec()
            .into_iter()
            .map(|alternative| {
                super::hover::render_entry(alternative.description, markdown.is_some(), markdown.unwrap_or_default())
            })
            .collect();
        Some(descriptions.join(if markdown.is_some() { "\n\n---\n\n" } else { "\n\n" }))
    } else {
        entry.documentation.map(|document| super::documentation::render(&document, markdown.is_some()))
    };

    CompletionItem {
        label: entry.label,
        kind: Some(completion_kind(entry.kind)),
        detail,
        label_details,
        documentation: documentation.map(|value| {
            Documentation::MarkupContent(MarkupContent {
                kind: if markdown.is_some() { MarkupKind::Markdown } else { MarkupKind::PlainText },
                value,
            })
        }),
        insert_text,
        insert_text_format: snippet.then_some(InsertTextFormat::Snippet),
        insert_text_mode: snippet.then_some(insert_mode).flatten(),
        text_edit,
        filter_text: entry.filter_text,
        preselect: entry.preselect.then_some(true),
        tags: (entry.deprecated && supports_deprecated_tag).then(|| vec![CompletionItemTag::Deprecated]),
        #[allow(deprecated)]
        deprecated: (entry.deprecated && !supports_deprecated_tag).then_some(true),
        ..CompletionItem::default()
    }
}

fn completion_kind(kind: CompletionKind) -> CompletionItemKind {
    match kind {
        CompletionKind::Snippet => CompletionItemKind::Snippet,
        CompletionKind::Keyword => CompletionItemKind::Keyword,
        CompletionKind::Variable => CompletionItemKind::Variable,
        CompletionKind::Function => CompletionItemKind::Function,
        CompletionKind::Constant => CompletionItemKind::Constant,
        CompletionKind::Class => CompletionItemKind::Class,
        CompletionKind::Interface => CompletionItemKind::Interface,
        CompletionKind::Enum => CompletionItemKind::Enum,
        CompletionKind::EnumMember => CompletionItemKind::EnumMember,
        CompletionKind::Field => CompletionItemKind::Field,
        CompletionKind::Method => CompletionItemKind::Method,
    }
}
