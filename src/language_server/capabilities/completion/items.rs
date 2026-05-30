//! Item builders for each [`super::Context`].

use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileType;
use mago_span::Span;
use mago_syntax::token::TokenKind;
use tower_lsp_server::ls_types::CompletionItem;
use tower_lsp_server::ls_types::CompletionItemKind;
use tower_lsp_server::ls_types::CompletionTextEdit;
use tower_lsp_server::ls_types::Documentation;
use tower_lsp_server::ls_types::InsertTextFormat;
use tower_lsp_server::ls_types::MarkupContent;
use tower_lsp_server::ls_types::MarkupKind;
use tower_lsp_server::ls_types::Range;
use tower_lsp_server::ls_types::TextEdit;

use crate::language_server::capabilities::lookup;
use crate::language_server::state::ExpressionTypeIndex;

use super::MAX_RESULTS;

pub(super) fn variable_items(
    tokens: &[mago_syntax::token::Token<'_>],
    scope_start: u32,
    offset: u32,
    prefix: &[u8],
    replace: Range,
) -> Vec<CompletionItem> {
    let mut seen = foldhash::HashSet::default();
    let mut out = Vec::new();
    for token in tokens {
        if !matches!(token.kind, TokenKind::Variable) {
            continue;
        }

        let token_end = token.start.offset + token.value.len() as u32;
        if token.start.offset < scope_start || token.start.offset >= offset || token_end >= offset {
            continue;
        }

        let name = mago_bytes::trim_start_byte(token.value, b'$');
        if !name.starts_with(prefix) {
            continue;
        }

        if !seen.insert(name) {
            continue;
        }

        let label = format!("${}", String::from_utf8_lossy(name));
        out.push(CompletionItem {
            kind: Some(CompletionItemKind::VARIABLE),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit { range: replace, new_text: label.clone() })),
            label,
            ..CompletionItem::default()
        });

        if out.len() >= MAX_RESULTS {
            break;
        }
    }
    out
}

pub(super) fn instance_member_items(
    codebase: &CodebaseMetadata,
    type_index: Option<&ExpressionTypeIndex>,
    receiver_span: (u32, u32),
    prefix: &[u8],
) -> Vec<CompletionItem> {
    let Some(type_index) = type_index else {
        return Vec::new();
    };

    let Some(class_words) = type_index.by_span.get(&receiver_span) else {
        return Vec::new();
    };

    let mut out: Vec<CompletionItem> = Vec::new();
    let mut seen = foldhash::HashSet::default();
    for class in class_words {
        if let Some(meta) = codebase.get_class_like(class.as_bytes()) {
            push_unique(&mut out, &mut seen, collect_class_members(codebase, meta, prefix, false));
        }

        if out.len() >= MAX_RESULTS {
            break;
        }
    }

    out
}

pub(super) fn static_member_items(codebase: &CodebaseMetadata, class: &[u8], prefix: &[u8]) -> Vec<CompletionItem> {
    if matches!(class, b"self" | b"static" | b"parent") {
        return Vec::new();
    }

    codebase.get_class_like(class).map(|meta| collect_class_members(codebase, meta, prefix, true)).unwrap_or_default()
}

pub(super) fn bare_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    file: &MagoFile,
    offset: u32,
    prefix: &[u8],
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let namespace = lookup::namespace_at_offset(file, offset).unwrap_or_default();
    let ns_lc = if namespace.is_empty() { None } else { Some(namespace.to_ascii_lowercase()) };
    let in_scope = |full: &[u8]| -> bool {
        match &ns_lc {
            Some(ns) => full.starts_with(ns) && full.get(ns.len()) == Some(&b'\\') || !full.contains(&b'\\'),
            None => true,
        }
    };

    let mut out = Vec::new();

    for (key, meta) in codebase.class_likes.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }

        if !is_user_symbol(database, meta.span) || is_synthetic_name(meta.original_name.as_bytes()) {
            continue;
        }

        let display = local_name(meta.original_name.as_bytes());
        if !display.to_ascii_lowercase().starts_with(&needle) || !in_scope(key.as_bytes()) {
            continue;
        }

        out.push(make_class_item(meta, display));
    }

    for ((_, name), meta) in codebase.function_likes.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }

        if !matches!(meta.kind, FunctionLikeKind::Function) {
            continue;
        }

        if !is_user_symbol(database, meta.span) {
            continue;
        }

        let display: &[u8] = meta.original_name.as_bytes();
        let local = local_name(display);
        if !local.to_ascii_lowercase().starts_with(&needle) || !in_scope(name.as_bytes()) {
            continue;
        }

        let local_str = String::from_utf8_lossy(local).into_owned();
        out.push(CompletionItem {
            label: local_str.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(render_signature(meta, local)),
            insert_text: Some(format!("{local_str}($1)")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        });
    }

    for (name, meta) in codebase.constants.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }

        if !is_user_symbol(database, meta.span) || !name.as_bytes().to_ascii_lowercase().starts_with(&needle) {
            continue;
        }

        if !in_scope(name.as_bytes()) {
            continue;
        }

        out.push(CompletionItem {
            label: String::from_utf8_lossy(local_name(name.as_bytes())).into_owned(),
            kind: Some(CompletionItemKind::CONSTANT),
            ..CompletionItem::default()
        });
    }

    out
}

pub(super) fn qualified_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    qualifier: &[u8],
    prefix: &[u8],
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let want_prefix = if qualifier.is_empty() {
        Vec::new()
    } else {
        let mut want = qualifier.to_ascii_lowercase();
        want.push(b'\\');
        want
    };

    let mut out = Vec::new();
    for (key, meta) in codebase.class_likes.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }

        if !is_user_symbol(database, meta.span) || is_synthetic_name(meta.original_name.as_bytes()) {
            continue;
        }

        let lc = key.as_bytes();
        if !lc.starts_with(&want_prefix) {
            continue;
        }

        if !lc[want_prefix.len()..].starts_with(&needle) {
            continue;
        }

        let display = &meta.original_name.as_bytes()[want_prefix.len().min(meta.original_name.as_bytes().len())..];
        out.push(make_class_item(meta, display));
    }

    out
}

fn collect_class_members(
    codebase: &CodebaseMetadata,
    meta: &ClassLikeMetadata,
    prefix: &[u8],
    static_access: bool,
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let mut out = Vec::new();

    if static_access {
        for name in meta.constants.keys() {
            let s = name.as_bytes();
            if !s.to_ascii_lowercase().starts_with(&needle) {
                continue;
            }

            out.push(CompletionItem {
                label: String::from_utf8_lossy(s).into_owned(),
                kind: Some(CompletionItemKind::CONSTANT),
                ..CompletionItem::default()
            });
        }

        for name in meta.enum_cases.keys() {
            let s = name.as_bytes();
            if !s.to_ascii_lowercase().starts_with(&needle) {
                continue;
            }

            out.push(CompletionItem {
                label: String::from_utf8_lossy(s).into_owned(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                ..CompletionItem::default()
            });
        }
    }

    for (name, declaring_class) in meta.appearing_property_ids.iter() {
        let visible = mago_bytes::trim_start_byte(name.as_bytes(), b'$');
        if !visible.to_ascii_lowercase().starts_with(&needle) {
            continue;
        }

        let property = codebase.get_class_like(declaring_class.as_bytes()).and_then(|c| c.properties.get(name));
        let is_static = property.is_some_and(|p| p.flags.is_static());
        if is_static != static_access {
            continue;
        }

        let detail = property
            .and_then(|p| p.type_metadata.as_ref())
            .map(|t| String::from_utf8_lossy(t.type_union.get_id().as_bytes()).into_owned());
        let label = if static_access {
            format!("${}", String::from_utf8_lossy(visible))
        } else {
            String::from_utf8_lossy(visible).into_owned()
        };
        out.push(CompletionItem { label, kind: Some(CompletionItemKind::FIELD), detail, ..CompletionItem::default() });
    }

    for (name, mid) in meta.appearing_method_ids.iter() {
        if !name.as_bytes().to_ascii_lowercase().starts_with(&needle) {
            continue;
        }

        let Some(method) = codebase.get_method(mid.get_class_name().as_bytes(), mid.get_method_name().as_bytes())
        else {
            continue;
        };

        let is_static = method.method_metadata.as_ref().is_some_and(|m| m.is_static);
        if is_static != static_access {
            continue;
        }

        let display: &[u8] = method.original_name.as_bytes();
        let display_str = String::from_utf8_lossy(display).into_owned();
        out.push(CompletionItem {
            label: display_str.clone(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(render_signature(method, display)),
            insert_text: Some(format!("{display_str}($1)")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        });

        if out.len() >= MAX_RESULTS {
            break;
        }
    }

    out
}

fn render_signature(meta: &FunctionLikeMetadata, name: &[u8]) -> String {
    use std::fmt::Write;
    let mut sig = String::with_capacity(64);
    let _ = write!(sig, "{}", BytesDisplay(name));
    sig.push('(');
    let mut first = true;
    for p in &meta.parameters {
        if !first {
            sig.push_str(", ");
        }

        first = false;
        if let Some(ty) = &p.type_metadata {
            let _ = write!(sig, "{}", BytesDisplay(ty.type_union.get_id().as_bytes()));
            sig.push(' ');
        }

        let _ = write!(sig, "{}", BytesDisplay(p.name.0.as_bytes()));
    }

    sig.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        sig.push_str(": ");
        let _ = write!(sig, "{}", BytesDisplay(rt.type_union.get_id().as_bytes()));
    }

    sig
}

fn push_unique(out: &mut Vec<CompletionItem>, seen: &mut foldhash::HashSet<String>, items: Vec<CompletionItem>) {
    for item in items {
        if seen.insert(item.label.clone()) {
            out.push(item);
        }
    }
}

fn make_class_item(meta: &ClassLikeMetadata, display: &[u8]) -> CompletionItem {
    use mago_codex::symbol::SymbolKind as M;
    let kind = match meta.kind {
        M::Class => CompletionItemKind::CLASS,
        M::Interface => CompletionItemKind::INTERFACE,
        M::Trait => CompletionItemKind::CLASS,
        M::Enum => CompletionItemKind::ENUM,
    };

    CompletionItem {
        label: String::from_utf8_lossy(display).into_owned(),
        kind: Some(kind),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::PlainText,
            value: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
        })),
        ..CompletionItem::default()
    }
}

fn is_user_symbol(database: &Database<'_>, span: Span) -> bool {
    database.get(&span.file_id).map(|f| !matches!(f.file_type, FileType::Builtin)).unwrap_or(false)
}

fn is_synthetic_name(name: &[u8]) -> bool {
    name.first() == Some(&b'{')
}

fn local_name(full: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', full) {
        Some(i) => &full[i + 1..],
        None => full,
    }
}
