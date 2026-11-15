//! Resolve PHPDoc class names before and after the planned import changes.

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileType};
use mago_names::{kind::NameKind, scope::NamespaceScope};
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_phpdoc_syntax::parser::{PHPDocParser, parse_type};
use mago_span::{Position, Span};
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::capabilities::rename::STALE;
use crate::semantic_index::{ImportBinding, SymbolKind};
use crate::{DocProblem, Range, RefusalReason, RenameRefusal, SymbolLocation};

use super::bindings::{self, Bindings};
use super::{Name, names};

struct ClassRename<'name> {
    target: &'name [u8],
    renamed: &'name [u8],
    leaf: &'name [u8],
}

enum NameEdit {
    Keep,
    Replace(Range),
    Reject(DocProblem),
}

pub(in super::super) fn alias_locations(
    server: &Server,
    file: &File,
    binding: &ImportBinding,
    new_name: &[u8],
) -> Result<Vec<Range>, RenameRefusal> {
    let mut locations = Vec::new();
    if binding.kind != SymbolKind::ClassLike {
        return Ok(locations);
    }
    let tokens = server.lex(file);
    let mut comments = tokens
        .iter()
        .filter(|token| {
            token.kind == TokenKind::DocBlockComment && binding.namespace_range.contains(token.start.offset)
        })
        .peekable();
    if comments.peek().is_none() {
        return Ok(locations);
    }
    let mut bindings = bindings::by_file(server.codebase(), &HashSet::from_iter([file.id]));
    let bindings = bindings.entry(file.id).or_default();
    bindings.include_docblocks(&tokens);
    for token in comments {
        let arena = LocalArena::new();
        let span = Span::new(file.id, token.start, Position::new(token.start.offset + token.value.len() as u32));
        let document = PHPDocParser::parse_without_trivia(&arena, token.value, span);
        let Some(names) = names(&document) else {
            if token
                .value
                .split(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\\') || *byte >= 0x80))
                .any(|value| {
                    first_segment(value).eq_ignore_ascii_case(&binding.alias)
                        || first_segment(value).eq_ignore_ascii_case(new_name)
                })
            {
                return Err(doc(DocProblem::Unparsed, span));
            }
            continue;
        };
        for name in names {
            let identifier = name.identifier;
            let prefix = first_segment(identifier.value);
            let old = prefix.eq_ignore_ascii_case(&binding.alias);
            if matches!(name.kind, super::NameKind::Variable) || !(old || prefix.eq_ignore_ascii_case(new_name)) {
                continue;
            }
            if bindings::uncertain_template_bound(&name, &document)
                || bindings::is_method_template(&name, identifier.value, &document)
            {
                return Err(doc(DocProblem::Template, identifier.span));
            }
            if bindings.is_type_bound(&name, identifier.value) {
                continue;
            }
            // A newly introduced alias must not capture another PHPDoc class name.
            if !old {
                return Err(doc(DocProblem::Capture, identifier.span));
            }
            let mut replacement = new_name.to_vec();
            replacement.extend_from_slice(&identifier.value[prefix.len()..]);
            if bindings.is_type_bound(&name, &replacement)
                || bindings::is_method_template(&name, &replacement, &document)
                || (matches!(name.kind, super::NameKind::Type { .. })
                    && !matches!(parse_type(&arena, &replacement, identifier.span), Ok(Type::Reference(_))))
            {
                return Err(doc(DocProblem::Capture, identifier.span));
            }
            locations
                .push(Range::new(identifier.span.start.offset, identifier.span.start.offset + prefix.len() as u32));
        }
    }
    Ok(locations)
}

fn doc(problem: DocProblem, span: Span) -> RenameRefusal {
    RenameRefusal::at(RefusalReason::Doc(problem), span.file_id, Range::new(span.start.offset, span.end.offset))
}

fn first_segment(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == b'\\').next().unwrap_or(value)
}

pub(in super::super) fn class_locations(
    server: &mut Server,
    target: &[u8],
    renamed: &[u8],
    new_name: &str,
) -> Result<Vec<SymbolLocation>, RenameRefusal> {
    let files: Vec<_> = server
        .database()
        .files()
        .filter(|file| file.file_type == FileType::Host && file.contents.windows(3).any(|bytes| bytes == b"/**"))
        .collect();
    let mut analyses = Vec::with_capacity(files.len());
    for file in files {
        let analysis = server.file_analysis_for(file.id).ok_or(STALE)?;
        analyses.push((file, analysis));
    }
    let candidates = analyses.iter().map(|(file, _)| file.id).collect();
    let mut bindings = bindings::by_file(server.codebase(), &candidates);
    let rename = ClassRename { target, renamed, leaf: new_name.as_bytes() };
    let mut locations = Vec::new();
    for (file, analysis) in analyses {
        let tokens = server.lex(&file);
        let bindings = bindings.entry(file.id).or_default();
        bindings.include_docblocks(&tokens);
        let imports = &analysis.semantic().imports;
        for token in tokens.iter().filter(|token| token.kind == TokenKind::DocBlockComment) {
            let arena = LocalArena::new();
            let span = Span::new(file.id, token.start, Position::new(token.start.offset + token.value.len() as u32));
            let document = PHPDocParser::parse_without_trivia(&arena, token.value, span);
            let before = analysis.scope_at(token.start.offset);
            let after = projected_scope(&before, imports, token.start.offset, &rename);
            let Some(names) = names(&document) else {
                if affected_unparsed(token.value, &before, &after, &rename) {
                    return Err(doc(DocProblem::Unparsed, span));
                }
                continue;
            };
            for name in names {
                match name_edit(name, &document, bindings, &before, &after, &rename, imports) {
                    NameEdit::Keep => {}
                    NameEdit::Replace(range) => locations.push(SymbolLocation { file: file.id, range }),
                    NameEdit::Reject(problem) => return Err(doc(problem, name.identifier.span)),
                }
            }
        }
    }
    Ok(locations)
}

fn projected_scope(
    before: &NamespaceScope,
    imports: &[ImportBinding],
    offset: u32,
    rename: &ClassRename<'_>,
) -> NamespaceScope {
    let mut after = NamespaceScope::new(before.namespace_name().map(<[u8]>::to_vec));
    for binding in imports
        .iter()
        .filter(|binding| binding.kind == SymbolKind::ClassLike && binding.namespace_range.contains(offset))
    {
        let changed = binding.target.eq_ignore_ascii_case(rename.target);
        let target = if changed { rename.renamed } else { &binding.target };
        let alias = if changed && binding.alias_range.is_none() { None } else { Some(binding.alias.as_ref()) };
        after.add(NameKind::Default, target, &alias);
    }
    after
}

fn name_edit(
    name: Name<'_>,
    document: &Document<'_>,
    bindings: &Bindings<'_>,
    before: &NamespaceScope,
    after: &NamespaceScope,
    rename: &ClassRename<'_>,
    imports: &[ImportBinding],
) -> NameEdit {
    if matches!(name.kind, super::NameKind::Variable) {
        return NameEdit::Keep;
    }
    let identifier = name.identifier;
    if bindings::uncertain_template_bound(&name, document)
        && [rename.target, rename.renamed].iter().any(|target| {
            target.rsplit(|byte| *byte == b'\\').next().is_some_and(|leaf| leaf.eq_ignore_ascii_case(identifier.value))
        })
    {
        return NameEdit::Reject(DocProblem::Template);
    }
    if bindings.is_type_bound(&name, identifier.value) {
        return NameEdit::Keep;
    }
    let current = before.resolve_str(NameKind::Default, identifier.value).0;
    let unchanged = after.resolve_str(NameKind::Default, identifier.value).0;
    if !current.eq_ignore_ascii_case(rename.target) {
        if !current.eq_ignore_ascii_case(&unchanged)
            || !rename.target.eq_ignore_ascii_case(rename.renamed) && current.eq_ignore_ascii_case(rename.renamed)
        {
            return NameEdit::Reject(DocProblem::Capture);
        }
        return NameEdit::Keep;
    }
    // The scanner does not yet bind templates declared inside an @method tag.
    if bindings::is_method_template(&name, identifier.value, document) {
        return NameEdit::Reject(DocProblem::Template);
    }
    let stable_alias = imports.iter().any(|binding| {
        binding.kind == SymbolKind::ClassLike
            && binding.alias_range.is_some()
            && binding.namespace_range.contains(identifier.span.start.offset)
            && binding.target.eq_ignore_ascii_case(rename.target)
            && binding.alias.eq_ignore_ascii_case(identifier.value)
    });
    if stable_alias && unchanged.eq_ignore_ascii_case(rename.renamed) {
        return NameEdit::Keep;
    }
    let prefix = identifier.value.iter().rposition(|byte| *byte == b'\\').map_or(0, |position| position + 1);
    let mut replacement = identifier.value[..prefix].to_vec();
    replacement.extend_from_slice(rename.leaf);
    if bindings.is_type_bound(&name, &replacement)
        || bindings::is_method_template(&name, &replacement, document)
        || !after.resolve_str(NameKind::Default, &replacement).0.eq_ignore_ascii_case(rename.renamed)
    {
        return NameEdit::Reject(DocProblem::Capture);
    }
    NameEdit::Replace(Range::new(identifier.span.start.offset + prefix as u32, identifier.span.end.offset))
}

fn affected_unparsed(
    content: &[u8],
    before: &NamespaceScope,
    after: &NamespaceScope,
    rename: &ClassRename<'_>,
) -> bool {
    content
        .split(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\\') || *byte >= 0x80))
        .filter(|name| !name.is_empty())
        .any(|name| {
            let current = before.resolve_str(NameKind::Default, name).0;
            let changed = after.resolve_str(NameKind::Default, name).0;
            current.eq_ignore_ascii_case(rename.target)
                || current.eq_ignore_ascii_case(rename.renamed)
                || !current.eq_ignore_ascii_case(&changed)
        })
}
