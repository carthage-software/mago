//! Item builders for each [`super::Context`].

use std::collections::hash_map::Entry;

use foldhash::HashMap;
use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
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

use crate::language_server::state::ExpressionTypeIndex;

use super::MAX_RESULTS;
use super::matcher;
use super::matcher::Score;

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

    let mut best: HashMap<String, (Score, CompletionItem)> = HashMap::default();
    for class in class_words {
        if let Some(meta) = codebase.get_class_like(class.as_bytes()) {
            for (score, item) in collect_class_members(codebase, meta, prefix, false) {
                match best.entry(item.label.clone()) {
                    Entry::Occupied(mut e) => {
                        if score < e.get().0 {
                            e.insert((score, item));
                        }
                    }
                    Entry::Vacant(e) => {
                        e.insert((score, item));
                    }
                }
            }
        }
    }

    finalize(best.into_values().collect())
}

pub(super) fn static_member_items(
    codebase: &CodebaseMetadata,
    scope: &NamespaceScope,
    class: &[u8],
    prefix: &[u8],
) -> Vec<CompletionItem> {
    if matches!(class, b"self" | b"static" | b"parent") {
        return Vec::new();
    }

    // The receiver is a name written in source; the scope resolves it to its
    // FQCN exactly as PHP would (aliases, namespace relativity, leading `\`).
    let (fqcn, _) = scope.resolve_str(NameKind::Default, class);
    let Some(meta) = codebase.get_class_like(&fqcn) else {
        return Vec::new();
    };

    finalize(collect_class_members(codebase, meta, prefix, true))
}

pub(super) fn bare_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    scope: &NamespaceScope,
    prefix: &[u8],
    classes_only: bool,
) -> Vec<CompletionItem> {
    let namespace = scope.namespace_name().unwrap_or_default();
    let current_ns = split_namespace(namespace);

    let in_scope = |full: &[u8]| -> bool {
        if namespace.is_empty() {
            return true;
        }
        let ns = namespace.to_ascii_lowercase();
        let lc = full.to_ascii_lowercase();
        (lc.starts_with(&ns) && lc.get(ns.len()) == Some(&b'\\')) || !lc.contains(&b'\\')
    };

    let mut scored: Vec<(Score, CompletionItem)> = Vec::new();

    for (_, meta) in codebase.class_likes.iter() {
        if !is_user_symbol(database, meta.span) || is_synthetic_name(meta.original_name.as_bytes()) {
            continue;
        }

        let fqcn = meta.original_name.as_bytes();
        let short = local_name(fqcn);
        let Some(score) = matcher::score(prefix, short) else {
            continue;
        };

        // What the short name resolves to here decides both ranking (is it
        // imported?) and insertion (can we drop the leading `\`?).
        let (resolved, imported) = scope.resolve_str(NameKind::Default, short);
        let resolves_here = resolved.eq_ignore_ascii_case(fqcn);
        let (up, down) = distance_up_down(&current_ns, fqcn);
        let score = score.with_locality(imported, is_vendor(database, meta.span), up, down);

        scored.push((score, make_class_item(meta, short, resolves_here)));
    }

    if !classes_only {
        for ((_, name), meta) in codebase.function_likes.iter() {
            if !matches!(meta.kind, FunctionLikeKind::Function) || !is_user_symbol(database, meta.span) {
                continue;
            }

            if !in_scope(name.as_bytes()) {
                continue;
            }

            let fqcn = meta.original_name.as_bytes();
            let local = local_name(fqcn);
            let Some(score) = matcher::score(prefix, local) else {
                continue;
            };

            let (_, imported) = scope.resolve_str(NameKind::Function, local);
            let (up, down) = distance_up_down(&current_ns, fqcn);
            let score = score.with_locality(imported, is_vendor(database, meta.span), up, down);

            let local_str = String::from_utf8_lossy(local).into_owned();
            scored.push((
                score,
                CompletionItem {
                    label: local_str.clone(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(render_signature(meta, local)),
                    insert_text: Some(format!("{local_str}($1)")),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                },
            ));
        }

        for (name, meta) in codebase.constants.iter() {
            if !is_user_symbol(database, meta.span) || !in_scope(name.as_bytes()) {
                continue;
            }

            let fqcn = name.as_bytes();
            let local = local_name(fqcn);
            let Some(score) = matcher::score(prefix, local) else {
                continue;
            };

            let (_, imported) = scope.resolve_str(NameKind::Constant, local);
            let (up, down) = distance_up_down(&current_ns, fqcn);
            let score = score.with_locality(imported, is_vendor(database, meta.span), up, down);

            scored.push((
                score,
                CompletionItem {
                    label: String::from_utf8_lossy(local).into_owned(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    ..CompletionItem::default()
                },
            ));
        }
    }

    finalize(scored)
}

pub(super) fn qualified_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    qualifier: &[u8],
    prefix: &[u8],
) -> Vec<CompletionItem> {
    let want_prefix = if qualifier.is_empty() {
        Vec::new()
    } else {
        let mut want = qualifier.to_ascii_lowercase();
        want.push(b'\\');
        want
    };

    let mut scored: Vec<(Score, CompletionItem)> = Vec::new();
    for (key, meta) in codebase.class_likes.iter() {
        if !is_user_symbol(database, meta.span) || is_synthetic_name(meta.original_name.as_bytes()) {
            continue;
        }

        if !key.as_bytes().starts_with(&want_prefix) {
            continue;
        }

        let fqcn = meta.original_name.as_bytes();
        let rel = &fqcn[want_prefix.len().min(fqcn.len())..];
        let Some(score) = matcher::score(prefix, rel) else {
            continue;
        };

        let score = score.with_locality(false, is_vendor(database, meta.span), 0, 0);
        scored.push((score, qualified_class_item(meta, rel)));
    }

    finalize(scored)
}

fn finalize(mut scored: Vec<(Score, CompletionItem)>) -> Vec<CompletionItem> {
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.label.cmp(&b.1.label)));
    scored.truncate(MAX_RESULTS);
    scored
        .into_iter()
        .map(|(score, mut item)| {
            item.sort_text = Some(score.sort_text());
            item.filter_text = Some(item.label.clone());
            item
        })
        .collect()
}

fn collect_class_members(
    codebase: &CodebaseMetadata,
    meta: &ClassLikeMetadata,
    prefix: &[u8],
    static_access: bool,
) -> Vec<(Score, CompletionItem)> {
    let mut out = Vec::new();

    if static_access {
        for name in meta.constants.keys() {
            let s = name.as_bytes();
            let Some(score) = matcher::score(prefix, s) else {
                continue;
            };

            out.push((
                score,
                CompletionItem {
                    label: String::from_utf8_lossy(s).into_owned(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    ..CompletionItem::default()
                },
            ));
        }

        for name in meta.enum_cases.keys() {
            let s = name.as_bytes();
            let Some(score) = matcher::score(prefix, s) else {
                continue;
            };

            out.push((
                score,
                CompletionItem {
                    label: String::from_utf8_lossy(s).into_owned(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    ..CompletionItem::default()
                },
            ));
        }
    }

    for (name, declaring_class) in meta.appearing_property_ids.iter() {
        let visible = mago_bytes::trim_start_byte(name.as_bytes(), b'$');
        let Some(score) = matcher::score(prefix, visible) else {
            continue;
        };

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
        out.push((
            score,
            CompletionItem { label, kind: Some(CompletionItemKind::FIELD), detail, ..CompletionItem::default() },
        ));
    }

    for (name, mid) in meta.appearing_method_ids.iter() {
        let Some(score) = matcher::score(prefix, name.as_bytes()) else {
            continue;
        };

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
        out.push((
            score,
            CompletionItem {
                label: display_str.clone(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(render_signature(method, display)),
                insert_text: Some(format!("{display_str}($1)")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..CompletionItem::default()
            },
        ));
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

/// Build a class completion. `resolves_here` is true when typing the short name
/// at the cursor already resolves to this class (it is imported or lives in the
/// current namespace); otherwise the item inserts the leading-`\` FQCN.
fn make_class_item(meta: &ClassLikeMetadata, short: &[u8], resolves_here: bool) -> CompletionItem {
    let fqcn = meta.original_name.as_bytes();
    let insert_text = (!resolves_here).then(|| {
        let mut text = String::with_capacity(fqcn.len() + 1);
        text.push('\\');
        text.push_str(&String::from_utf8_lossy(fqcn));
        text
    });

    CompletionItem {
        label: String::from_utf8_lossy(short).into_owned(),
        kind: Some(class_item_kind(meta)),
        insert_text,
        documentation: Some(class_documentation(meta)),
        ..CompletionItem::default()
    }
}

fn qualified_class_item(meta: &ClassLikeMetadata, display: &[u8]) -> CompletionItem {
    CompletionItem {
        label: String::from_utf8_lossy(display).into_owned(),
        kind: Some(class_item_kind(meta)),
        documentation: Some(class_documentation(meta)),
        ..CompletionItem::default()
    }
}

fn class_item_kind(meta: &ClassLikeMetadata) -> CompletionItemKind {
    use mago_codex::symbol::SymbolKind as M;
    match meta.kind {
        M::Class => CompletionItemKind::CLASS,
        M::Interface => CompletionItemKind::INTERFACE,
        M::Trait => CompletionItemKind::CLASS,
        M::Enum => CompletionItemKind::ENUM,
    }
}

fn class_documentation(meta: &ClassLikeMetadata) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::PlainText,
        value: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
    })
}

fn is_user_symbol(database: &Database<'_>, span: Span) -> bool {
    database.get(&span.file_id).map(|f| !matches!(f.file_type, FileType::Builtin)).unwrap_or(false)
}

fn is_vendor(database: &Database<'_>, span: Span) -> bool {
    database.get(&span.file_id).map(|f| matches!(f.file_type, FileType::Vendored)).unwrap_or(false)
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

fn namespace_of(fqcn: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', fqcn) {
        Some(i) => &fqcn[..i],
        None => &[],
    }
}

fn split_namespace(ns: &[u8]) -> Vec<&[u8]> {
    if ns.is_empty() { Vec::new() } else { ns.split(|&b| b == b'\\').collect() }
}

/// Namespace tree distance from the cursor namespace `current` to the namespace
/// of `fqcn`, as `(steps up to the common ancestor, steps back down)`. A name
/// in the same namespace is `(0, 0)`; in the parent `(1, 0)`; a sibling
/// namespace `(1, 1)`; the grandparent `(2, 0)`.
fn distance_up_down(current: &[&[u8]], fqcn: &[u8]) -> (u16, u16) {
    let candidate = namespace_of(fqcn);
    let cand = split_namespace(candidate);
    let mut k = 0;
    while k < current.len() && k < cand.len() && current[k].eq_ignore_ascii_case(cand[k]) {
        k += 1;
    }
    ((current.len() - k) as u16, (cand.len() - k) as u16)
}
