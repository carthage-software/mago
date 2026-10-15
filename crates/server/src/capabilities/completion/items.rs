//! Shared completion item construction, ranking, and result limits.

use mago_bytes::BytesDisplay;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::display::qualified_bytes;
use mago_database::Database;

use crate::Server;
use crate::documentation::Documentation;
use crate::domain::CompletionAlternative;
use crate::domain::CompletionDocumentation;
use crate::domain::CompletionEntry;
use crate::domain::CompletionInsert;
use crate::domain::CompletionKind;
use crate::domain::SourceDocumentation;
use crate::lookup::DocblockAnchor;

use super::MAX_RESULTS;
use super::rank::Rank;
use super::rank::compare_names;
use super::rank::preselects;

pub(super) fn finalize(mut ranked: Vec<(Rank, CompletionEntry)>) -> Vec<CompletionEntry> {
    ranked.sort_unstable_by(|(left_rank, left), (right_rank, right)| {
        left_rank
            .cmp(right_rank)
            .then_with(|| compare_names(left.label.as_bytes(), right.label.as_bytes()))
            .then_with(|| (left.kind as u8).cmp(&(right.kind as u8)))
    });
    ranked.truncate(MAX_RESULTS);
    let preselect = preselects(ranked.first().map(|(rank, _)| rank), ranked.get(1).map(|(rank, _)| rank));
    let mut entries: Vec<_> = ranked.into_iter().map(|(_, entry)| entry).collect();
    finish(&mut entries, preselect);
    entries
}

pub(super) fn finish(entries: &mut [CompletionEntry], preselect: bool) {
    for entry in entries.iter_mut() {
        entry.filter_text = Some(entry.label.clone());
    }
    if let Some(first) = entries.first_mut() {
        first.preselect = preselect;
    }
}

pub(super) fn document(server: &Server, items: &mut [CompletionEntry]) {
    let mut documentation = Documentation::new(server);
    for item in items {
        if let Some(alternatives) = item.member_alternatives.take() {
            let mut distinct = Vec::new();
            for mut alternative in alternatives.into_vec() {
                if let Some(source) = alternative.documentation_source.take() {
                    alternative.description.documentation = source_document(server, &mut documentation, source);
                }
                if !distinct.iter().any(|entry: &CompletionAlternative| entry.description == alternative.description) {
                    distinct.push(alternative);
                }
            }
            distinct.sort_by(|a, b| {
                a.description
                    .scope
                    .cmp(&b.description.scope)
                    .then_with(|| a.description.declaration.cmp(&b.description.declaration))
            });
            item.documentation_source = None;
            if distinct.len() == 1 {
                item.documentation = distinct.pop().and_then(|entry| entry.description.documentation);
            } else {
                item.detail = Some(
                    distinct
                        .iter()
                        .map(|entry| {
                            format!(
                                "{}: {}",
                                entry.description.scope.as_deref().unwrap_or_default(),
                                entry.description.declaration
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                );
                item.documentation = None;
                item.member_alternatives = Some(distinct.into_boxed_slice());
            }
            continue;
        }
        let Some(source) = item.documentation_source else { continue };
        let document = source_document(server, &mut documentation, source);
        if document.is_some() {
            item.documentation = document;
        }
    }
}

fn source_document(
    server: &Server,
    documentation: &mut Documentation<'_>,
    source: CompletionDocumentation,
) -> Option<SourceDocumentation> {
    match source {
        CompletionDocumentation::Callable(key) => documentation.for_callable(key),
        CompletionDocumentation::Declaration(span) => {
            documentation.for_declaration(span, DocblockAnchor::DeclarationStart)
        }
        CompletionDocumentation::DeclarationName(span) => {
            documentation.for_declaration(span, DocblockAnchor::DeclarationName)
        }
        CompletionDocumentation::Parameter { callable, name } => {
            documentation.for_callable_parameter(callable, name.as_bytes())
        }
        CompletionDocumentation::MagicProperty { class, name } => {
            let property = server.codebase().get_declaring_magic_property(class.as_bytes(), name.as_bytes())?;
            documentation.for_magic_property(class, property)
        }
        CompletionDocumentation::Property { class, name } => {
            let codebase = server.codebase();
            codebase
                .get_declaring_property_class(class.as_bytes(), name.as_bytes())
                .zip(codebase.get_declaring_property(class.as_bytes(), name.as_bytes()))
                .and_then(|(owner, property)| documentation.for_property(owner, property))
        }
    }
}

pub(super) fn function_item(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    meta: &FunctionLikeMetadata,
    display: &[u8],
    insert: &[u8],
) -> CompletionEntry {
    CompletionEntry {
        detail: Some(render_signature(database, codebase, meta, display)),
        documentation_source: meta
            .has_docblock
            .then_some(CompletionDocumentation::Callable(FunctionLikeIdentifier::Function(meta.name))),
        insert: Some(CompletionInsert::Call(String::from_utf8_lossy(insert).into_owned())),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(display).into_owned(), CompletionKind::Function)
    }
}

pub(super) fn render_signature(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    meta: &FunctionLikeMetadata,
    name: &[u8],
) -> String {
    use std::fmt::Write;
    let mut sig = String::with_capacity(64);
    if meta.flags.is_by_reference() {
        sig.push('&');
    }
    let _ = write!(sig, "{}", BytesDisplay(name));
    sig.push('(');
    let mut first = true;
    for p in &meta.parameters {
        if !first {
            sig.push_str(", ");
        }

        first = false;
        let displayed_type = p.type_metadata.as_ref().map(|ty| qualified_bytes(codebase, &ty.type_union));
        super::super::signature::append_parameter(&mut sig, database, p, displayed_type.as_deref());
    }

    sig.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        sig.push_str(": ");
        let _ = write!(sig, "{}", BytesDisplay(&qualified_bytes(codebase, &rt.type_union)));
    }

    sig
}

/// Build a class completion. `resolves_here` is true when typing the short name
/// at the cursor already resolves to this class (it is imported or lives in the
/// current namespace); otherwise the item inserts the leading-`\` FQCN.
pub(super) fn class_item(meta: &ClassLikeMetadata, short: &[u8], resolves_here: bool) -> CompletionEntry {
    let fqcn = meta.original_name.as_bytes();
    let insert_text = (!resolves_here).then(|| {
        let mut text = String::with_capacity(fqcn.len() + 1);
        text.push('\\');
        text.push_str(&String::from_utf8_lossy(fqcn));
        text
    });

    CompletionEntry {
        insert: insert_text.map(CompletionInsert::Text),
        namespace: Some(class_namespace(meta)),
        documentation: Some(class_documentation(meta)),
        documentation_source: Some(CompletionDocumentation::Declaration(meta.span)),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(short).into_owned(), class_item_kind(meta))
    }
}

pub(super) fn qualified_class_item(meta: &ClassLikeMetadata, display: &[u8]) -> CompletionEntry {
    CompletionEntry {
        namespace: Some(class_namespace(meta)),
        documentation: Some(class_documentation(meta)),
        documentation_source: Some(CompletionDocumentation::Declaration(meta.span)),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(display).into_owned(), class_item_kind(meta))
    }
}

fn class_namespace(meta: &ClassLikeMetadata) -> String {
    let name = meta.original_name.as_bytes();
    let namespace = memchr::memrchr(b'\\', name).map_or(b"\\".as_slice(), |index| &name[..index]);
    String::from_utf8_lossy(namespace).into_owned()
}

fn class_item_kind(meta: &ClassLikeMetadata) -> CompletionKind {
    use mago_codex::symbol::SymbolKind as M;
    match meta.kind {
        M::Class => CompletionKind::Class,
        M::Interface => CompletionKind::Interface,
        M::Trait => CompletionKind::Class,
        M::Enum => CompletionKind::Enum,
    }
}

fn class_documentation(meta: &ClassLikeMetadata) -> SourceDocumentation {
    SourceDocumentation { text: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(), tags: vec![] }
}
