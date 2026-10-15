//! Shared name fallback and member lookups for editor queries.

use mago_analyzer::artifacts::editor::EditorFileData;
use mago_analyzer::artifacts::editor::MemberReference;
use mago_analyzer::artifacts::editor::MethodReference;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_span::Position;
use mago_span::Span;

use crate::Range;
use crate::semantic_index::BindingKind;
use crate::semantic_index::Occurrence;
use crate::semantic_index::ScopeKind;
use crate::semantic_index::SemanticIndex;
use crate::semantic_index::SymbolKind;

/// Apply PHP namespace fallback to one editor occurrence, without changing the index.
#[must_use]
pub fn resolved_symbol_name<'a>(
    codebase: &CodebaseMetadata,
    source: &'a [u8],
    index: &'a SemanticIndex,
    occurrence: &Occurrence,
) -> &'a [u8] {
    let symbol = &index.symbols[occurrence.symbol as usize];
    if occurrence.declaration
        || occurrence.imported
        || !matches!(symbol.kind, SymbolKind::Function | SymbolKind::Constant)
    {
        return &symbol.name;
    }
    let Some(raw) = source.get(occurrence.range.start as usize..occurrence.range.end as usize) else {
        return &symbol.name;
    };
    if raw.contains(&b'\\') {
        return &symbol.name;
    }
    let exists = |name: &[u8]| match symbol.kind {
        SymbolKind::Function => codebase.get_function(name).is_some(),
        SymbolKind::Constant => codebase.get_constant(name).is_some(),
        _ => false,
    };
    if !exists(&symbol.name) && exists(raw) { raw } else { &symbol.name }
}

/// Synthetic or reserved class-likes users cannot name; `key` is a lowercase class key.
#[inline]
#[must_use]
pub(crate) fn is_hidden_class_like(key: &[u8]) -> bool {
    match key.first() {
        Some(b'{') => true,
        Some(b'_') => key.starts_with(b"__internal_do_not_use__"),
        _ => false,
    }
}

pub(crate) fn same_name(kind: SymbolKind, left: &[u8], right: &[u8]) -> bool {
    if kind == SymbolKind::Constant { left == right } else { left.eq_ignore_ascii_case(right) }
}

/// Whether a `candidate` symbol can refer to `name`, including global fallback for unqualified functions and constants.
pub(crate) fn may_refer_to(kind: SymbolKind, candidate: &[u8], name: &[u8]) -> bool {
    same_name(kind, candidate, name)
        || (matches!(kind, SymbolKind::Function | SymbolKind::Constant)
            && !name.contains(&b'\\')
            && same_name(kind, short_name(candidate), name))
}

pub(crate) fn short_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b'\\').next().unwrap_or_default()
}

pub(crate) fn declared_member_at(codebase: &CodebaseMetadata, file: FileId, offset: u32) -> Option<Span> {
    let contains = |span: &Span| span.file_id == file && span.start.offset <= offset && offset < span.end.offset;
    for signature in &codebase.file_signatures.get(&file)?.ast_nodes {
        if signature.is_function {
            continue;
        }
        let Some(class) = codebase.get_class_like_by_name(signature.name) else { continue };
        for method in signature.children.iter().filter(|child| child.is_function) {
            if let Some(span) = codebase
                .get_method(class.name.as_bytes(), method.name.as_bytes())
                .and_then(|method| method.name_span)
                .filter(contains)
            {
                return Some(span);
            }
        }
        if let Some(span) = class.properties.values().filter_map(|property| property.name_span).find(contains) {
            return Some(span);
        }
        for constant in class.constants.values() {
            let mut span = constant.span;
            span.end.offset = span.start.offset + constant.name.as_bytes().len() as u32;
            if contains(&span) {
                return Some(span);
            }
        }
        if let Some(span) = class.enum_cases.values().map(|case| case.name_span).find(contains) {
            return Some(span);
        }
    }
    None
}

pub(crate) fn methods_at(data: &EditorFileData, offset: u32) -> &[MethodReference] {
    let end = data.methods.partition_point(|method| method.selector_span.0 <= offset);
    let Some(last) = end.checked_sub(1).and_then(|index| data.methods.get(index)) else { return &[] };
    if offset >= last.selector_span.1 {
        return &[];
    }
    let start = data.methods[..end].partition_point(|method| method.selector_span.0 < last.selector_span.0);
    &data.methods[start..end]
}

pub(crate) fn members_at(data: &EditorFileData, offset: u32) -> &[MemberReference] {
    let end = data.members.partition_point(|member| member.selector_span.0 <= offset);
    let Some(last) = end.checked_sub(1).and_then(|index| data.members.get(index)) else { return &[] };
    if offset >= last.selector_span.1 {
        return &[];
    }
    let start = data.members[..end].partition_point(|member| member.selector_span.0 < last.selector_span.0);
    &data.members[start..end]
}

pub(crate) fn parameter_at<'a>(
    codebase: &'a CodebaseMetadata,
    file: &File,
    index: &SemanticIndex,
    offset: u32,
) -> Option<(&'a FunctionLikeParameterMetadata, Span)> {
    let occurrence = index.at_offset(offset)?;
    let symbol = &index.symbols[occurrence.symbol as usize];
    if !matches!(symbol.binding, Some(BindingKind::Parameter | BindingKind::PromotedProperty)) {
        return None;
    }
    let scope = index.scopes.get(symbol.scope? as usize)?;
    let function = match scope.kind {
        ScopeKind::Closure | ScopeKind::ArrowFunction => codebase
            .get_closure_at(file, Span::new(file.id, Position::new(scope.range.start), Position::new(scope.range.end))),
        ScopeKind::Function | ScopeKind::Method => named_function(codebase, file, scope.range).or_else(|| {
            if scope.kind != ScopeKind::Method {
                return None;
            }
            // Anonymous classes have no named file-signature node.
            codebase
                .function_likes
                .values()
                .find(|method| {
                    method.kind.is_method()
                        && method.span.file_id == file.id
                        && Range::new(method.span.start.offset, method.span.end.offset) == scope.range
                })
                .map(Box::as_ref)
        }),
        _ => None,
    }?;
    let parameter = function.parameters.iter().find(|parameter| {
        let span = parameter.name_span;
        span.file_id == file.id
            && (!occurrence.declaration || Range::new(span.start.offset, span.end.offset) == occurrence.range)
            && parameter.name.0.as_bytes().strip_prefix(b"$") == Some(symbol.name.as_ref())
    })?;
    Some((parameter, function.span))
}

fn named_function<'a>(codebase: &'a CodebaseMetadata, file: &File, scope: Range) -> Option<&'a FunctionLikeMetadata> {
    codebase
        .file_signatures
        .get(&file.id)?
        .ast_nodes
        .iter()
        .flat_map(|node| {
            node.is_function.then_some(FunctionLikeIdentifier::Function(node.name)).into_iter().chain(
                node.children
                    .iter()
                    .filter(|child| child.is_function)
                    .map(move |child| FunctionLikeIdentifier::Method(node.name, child.name)),
            )
        })
        .filter_map(|key| codebase.get_function_like(&key))
        .find(|function| {
            function.span.file_id == file.id
                && Range::new(function.span.start.offset, function.span.end.offset) == scope
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_class_likes_are_synthetic_or_reserved() {
        assert!(is_hidden_class_like(b"{anonymous-class:a.php:1:1}"));
        assert!(is_hidden_class_like(b"__internal_do_not_use__x"));
        assert!(is_hidden_class_like(b"__internal_do_not_use__stringbackedenum"));
        assert!(!is_hidden_class_like(b"__php_incomplete_class"));
        assert!(!is_hidden_class_like(b"backedenum"));
        assert!(!is_hidden_class_like(b""));
    }
}
