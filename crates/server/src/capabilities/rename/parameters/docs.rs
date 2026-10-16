//! Rename bound PHPDoc variables without changing prose or type names.

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_phpdoc_syntax::cst::{AssertSubject, TagValue};
use mago_phpdoc_syntax::parser::PHPDocParser;
use mago_span::{Position, Span};
use mago_syntax::token::TokenKind;

use crate::lookup::DocblockAnchor;
use crate::semantic_index::{SemanticIndex, SymbolKind};
use crate::{Range, Server, SymbolLocation};

use super::super::phpdoc;
use super::Parameter;

pub(super) fn collect(
    server: &Server,
    file: &File,
    parameter: &Parameter,
    index: &SemanticIndex,
    locations: &mut Vec<SymbolLocation>,
) {
    collect_inline(server, file, parameter, index, locations);
    let Some(comment) = server.docblock_before(file, parameter.function.start.offset, DocblockAnchor::DeclarationStart)
    else {
        return;
    };
    let start = (comment.as_ptr().addr() - file.contents.as_ptr().addr()) as u32;
    let span = Span::new(file.id, Position::new(start), Position::new(start + comment.len() as u32));
    let arena = LocalArena::new();
    let document = PHPDocParser::parse_without_trivia(&arena, comment, span);
    for tag in document.tags() {
        let variable = match tag.value {
            TagValue::Param(value) => value.parameter,
            TagValue::TypelessParam(value) => Some(value.parameter),
            TagValue::ParamOut(value) => Some(value.parameter),
            TagValue::ParamClosureThis(value) => Some(value.parameter),
            TagValue::ParamImmediatelyInvokedCallable(value) => Some(value.parameter),
            TagValue::ParamLaterInvokedCallable(value) => Some(value.parameter),
            TagValue::Assert(value) | TagValue::AssertIfTrue(value) | TagValue::AssertIfFalse(value) => {
                Some(subject_variable(value.subject))
            }
            _ => None,
        };
        if let Some(variable) =
            variable.filter(|variable| variable.value.strip_prefix(b"$") == Some(parameter.name.as_slice()))
        {
            locations.push(SymbolLocation {
                file: file.id,
                range: Range::new(variable.span.start.offset + 1, variable.span.end.offset),
            });
        }
    }
    collect_type_variables(phpdoc::parameter_names(&document), parameter, locations);
}

fn collect_inline(
    server: &Server,
    file: &File,
    parameter: &Parameter,
    index: &SemanticIndex,
    locations: &mut Vec<SymbolLocation>,
) {
    let Some(declaration) = index.at_offset(parameter.declaration.start.offset) else { return };
    let function = Range::new(parameter.function.start.offset, parameter.function.end.offset);
    let classes: Vec<_> = server
        .codebase()
        .class_likes
        .values()
        .filter(|class| class.span.file_id == file.id)
        .map(|class| Range::new(class.span.start.offset, class.span.end.offset))
        .collect();
    for token in server
        .lex(file)
        .into_iter()
        .filter(|token| token.kind == TokenKind::DocBlockComment && function.contains(token.start.offset))
    {
        let Some(scope) = index.scope_at(token.start.offset) else { continue };
        // A nested class body does not bind variables from its enclosing function.
        if classes.iter().any(|class| class.start > scope.range.start && class.contains(token.start.offset))
            || binding_at(index, &parameter.name, token.start.offset, scope.range) != Some(declaration.symbol)
        {
            continue;
        }
        let arena = LocalArena::new();
        let span = Span::new(file.id, token.start, Position::new(token.start.offset + token.value.len() as u32));
        let document = PHPDocParser::parse_without_trivia(&arena, token.value, span);
        for tag in document.tags() {
            let variable = match tag.value {
                TagValue::Var(value) => {
                    collect_type_variables(phpdoc::type_names(value.r#type), parameter, locations);
                    value.variable
                }
                TagValue::Trace(value) => Some(value.variable),
                _ => None,
            };
            if let Some(variable) =
                variable.filter(|variable| variable.value.strip_prefix(b"$") == Some(parameter.name.as_slice()))
            {
                locations.push(SymbolLocation {
                    file: file.id,
                    range: Range::new(variable.span.start.offset + 1, variable.span.end.offset),
                });
            }
        }
    }
}

fn binding_at(index: &SemanticIndex, name: &[u8], offset: u32, scope: Range) -> Option<u32> {
    let next = index.occurrences.partition_point(|occurrence| occurrence.range.start < offset);
    // A preceding occurrence records rebinding by `global`; implicit captures
    // may first appear after the comment, within the same lexical scope.
    index.occurrences[..next]
        .iter()
        .rev()
        .chain(index.occurrences[next..].iter())
        .find(|occurrence| {
            let symbol = &index.symbols[occurrence.symbol as usize];
            symbol.kind == SymbolKind::Variable
                && symbol.name.as_ref() == name
                && index.scope_at(occurrence.range.start).is_some_and(|candidate| candidate.range == scope)
        })
        .map(|occurrence| occurrence.symbol)
}

fn collect_type_variables(
    names: Option<Vec<phpdoc::Name<'_>>>,
    parameter: &Parameter,
    locations: &mut Vec<SymbolLocation>,
) {
    if let Some(names) = names {
        locations.extend(names.into_iter().filter_map(|name| {
            (matches!(name.kind, phpdoc::NameKind::Variable)
                && name.identifier.value.strip_prefix(b"$") == Some(parameter.name.as_slice()))
            .then_some(SymbolLocation {
                file: parameter.declaration.file_id,
                range: Range::new(name.identifier.span.start.offset + 1, name.identifier.span.end.offset),
            })
        }));
    }
}

fn subject_variable(subject: AssertSubject<'_>) -> mago_phpdoc_syntax::cst::Variable<'_> {
    match subject {
        AssertSubject::Parameter { variable } => variable,
        AssertSubject::Method { object, .. } | AssertSubject::Property { object, .. } => subject_variable(*object),
    }
}
