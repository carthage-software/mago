//! Item builders for each [`super::Context`].

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
use tower_lsp::lsp_types::CompletionItem;
use tower_lsp::lsp_types::CompletionItemKind;
use tower_lsp::lsp_types::Documentation;
use tower_lsp::lsp_types::InsertTextFormat;
use tower_lsp::lsp_types::MarkupContent;
use tower_lsp::lsp_types::MarkupKind;

use crate::language_server::capabilities::lookup;
use crate::language_server::state::ExpressionTypeIndex;

use super::MAX_RESULTS;

pub(super) fn variable_items(
    tokens: &[mago_syntax::token::Token<'_>],
    scope_start: u32,
    offset: u32,
    prefix: &str,
) -> Vec<CompletionItem> {
    let mut seen = foldhash::HashSet::default();
    let mut out = Vec::new();
    for token in tokens {
        if !matches!(token.kind, TokenKind::Variable) {
            continue;
        }
        if token.start.offset < scope_start || token.start.offset >= offset {
            continue;
        }
        let name = token.value.trim_start_matches('$');
        if !name.starts_with(prefix) {
            continue;
        }
        if !seen.insert(name) {
            continue;
        }
        out.push(CompletionItem {
            label: format!("${name}"),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name.to_string()),
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
    prefix: &str,
) -> Vec<CompletionItem> {
    let Some(type_index) = type_index else {
        return Vec::new();
    };
    let Some(class_atoms) = type_index.by_span.get(&receiver_span) else {
        return Vec::new();
    };

    let mut out: Vec<CompletionItem> = Vec::new();
    let mut seen = foldhash::HashSet::default();
    for class in class_atoms {
        if let Some(meta) = codebase.get_class_like(class.as_str()) {
            push_unique(&mut out, &mut seen, collect_class_members(codebase, meta, prefix, false));
        }
        if out.len() >= MAX_RESULTS {
            break;
        }
    }
    out
}

pub(super) fn static_member_items(codebase: &CodebaseMetadata, class: &str, prefix: &str) -> Vec<CompletionItem> {
    if matches!(class, "self" | "static" | "parent") {
        return Vec::new();
    }
    codebase.get_class_like(class).map(|meta| collect_class_members(codebase, meta, prefix, true)).unwrap_or_default()
}

pub(super) fn bare_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    file: &MagoFile,
    offset: u32,
    prefix: &str,
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let namespace = lookup::namespace_at_offset(file, offset).unwrap_or_default();
    let ns_lc = if namespace.is_empty() { None } else { Some(namespace.to_ascii_lowercase()) };
    let in_scope = |full: &str| -> bool {
        match &ns_lc {
            Some(ns) => full.starts_with(ns) && full.as_bytes().get(ns.len()) == Some(&b'\\') || !full.contains('\\'),
            None => true,
        }
    };
    let mut out = Vec::new();

    for (key, meta) in codebase.class_likes.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        let display = local_name(meta.original_name.as_str());
        if !display.to_ascii_lowercase().starts_with(&needle) || !in_scope(key.as_str()) {
            continue;
        }
        out.push(make_class_item(meta, display));
    }

    for ((_, name), meta) in codebase.function_likes.iter() {
        if matches!(meta.kind, FunctionLikeKind::Method) || out.len() >= MAX_RESULTS {
            continue;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        let display = meta.original_name.map(|n| n.as_str()).unwrap_or_else(|| name.as_str());
        let local = local_name(display);
        if !local.to_ascii_lowercase().starts_with(&needle) || !in_scope(name.as_str()) {
            continue;
        }
        out.push(CompletionItem {
            label: local.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(render_signature(meta, local)),
            insert_text: Some(format!("{local}($1)")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        });
    }

    for (name, meta) in codebase.constants.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }
        if !is_user_symbol(database, meta.span) || !name.as_str().to_ascii_lowercase().starts_with(&needle) {
            continue;
        }
        if !in_scope(name.as_str()) {
            continue;
        }
        out.push(CompletionItem {
            label: local_name(name.as_str()).to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            ..CompletionItem::default()
        });
    }

    out
}

pub(super) fn qualified_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    qualifier: &str,
    prefix: &str,
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let qual_lc = qualifier.to_ascii_lowercase();
    let want_prefix = format!("{qual_lc}\\");
    let mut out = Vec::new();

    for (key, meta) in codebase.class_likes.iter() {
        if out.len() >= MAX_RESULTS {
            break;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        let lc = key.as_str();
        if !lc.starts_with(&want_prefix) {
            continue;
        }
        let suffix = &lc[want_prefix.len()..];
        if suffix.contains('\\') || !suffix.starts_with(&needle) {
            continue;
        }
        let display = local_name(meta.original_name.as_str());
        out.push(make_class_item(meta, display));
    }

    out
}

fn collect_class_members(
    codebase: &CodebaseMetadata,
    meta: &ClassLikeMetadata,
    prefix: &str,
    static_access: bool,
) -> Vec<CompletionItem> {
    let needle = prefix.to_ascii_lowercase();
    let mut out = Vec::new();

    if static_access {
        for name in meta.constants.keys() {
            let s = name.as_str();
            if !s.to_ascii_lowercase().starts_with(&needle) {
                continue;
            }
            out.push(CompletionItem {
                label: s.to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                ..CompletionItem::default()
            });
        }
        for name in meta.enum_cases.keys() {
            let s = name.as_str();
            if !s.to_ascii_lowercase().starts_with(&needle) {
                continue;
            }
            out.push(CompletionItem {
                label: s.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                ..CompletionItem::default()
            });
        }
    } else {
        for (name, declaring_class) in meta.appearing_property_ids.iter() {
            let visible = name.as_str().trim_start_matches('$');
            if !visible.to_ascii_lowercase().starts_with(&needle) {
                continue;
            }
            let detail = codebase
                .get_class_like(declaring_class.as_str())
                .and_then(|c| c.properties.get(name))
                .and_then(|p| p.type_metadata.as_ref())
                .map(|t| t.type_union.get_id().as_str().to_string());
            out.push(CompletionItem {
                label: visible.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail,
                ..CompletionItem::default()
            });
        }
    }

    for (name, mid) in meta.appearing_method_ids.iter() {
        if !name.as_str().to_ascii_lowercase().starts_with(&needle) {
            continue;
        }
        let Some(method) = codebase.get_method(&mid.get_class_name(), &mid.get_method_name()) else { continue };
        let display = method.original_name.map(|n| n.as_str()).unwrap_or_else(|| name.as_str());
        out.push(CompletionItem {
            label: display.to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(render_signature(method, display)),
            insert_text: Some(format!("{display}($1)")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        });
        if out.len() >= MAX_RESULTS {
            break;
        }
    }

    out
}

fn render_signature(meta: &FunctionLikeMetadata, name: &str) -> String {
    let mut sig = String::with_capacity(64);
    sig.push_str(name);
    sig.push('(');
    let mut first = true;
    for p in &meta.parameters {
        if !first {
            sig.push_str(", ");
        }
        first = false;
        if let Some(ty) = &p.type_metadata {
            sig.push_str(ty.type_union.get_id().as_str());
            sig.push(' ');
        }
        sig.push_str(p.name.0.as_str());
    }
    sig.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        sig.push_str(": ");
        sig.push_str(rt.type_union.get_id().as_str());
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

fn make_class_item(meta: &ClassLikeMetadata, display: &str) -> CompletionItem {
    use mago_codex::symbol::SymbolKind as M;
    let kind = match meta.kind {
        M::Class => CompletionItemKind::CLASS,
        M::Interface => CompletionItemKind::INTERFACE,
        M::Trait => CompletionItemKind::CLASS,
        M::Enum => CompletionItemKind::ENUM,
    };
    CompletionItem {
        label: display.to_string(),
        kind: Some(kind),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::PlainText,
            value: meta.original_name.as_str().to_string(),
        })),
        ..CompletionItem::default()
    }
}

fn is_user_symbol(database: &Database<'_>, span: Span) -> bool {
    database.get(&span.file_id).map(|f| !matches!(f.file_type, FileType::Builtin)).unwrap_or(false)
}

fn local_name(full: &str) -> &str {
    full.rsplit('\\').next().unwrap_or(full)
}
