//! Native type keywords, using a parsed hint fragment and the current declaration.

use mago_allocator::LocalArena;
use mago_database::file::{File, FileId};
use mago_names::scope::NamespaceScope;
use mago_syntax::cst::Statement;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::token::{Token, TokenKind};

use crate::Server;
use crate::domain::{CompletionEntry, CompletionInsert, CompletionKind, Range};
use crate::lookup;
use crate::matcher;

use super::context::{Context, NameIntent};
use super::items;
use super::native_type_context::{NativeTypeContext, Position};
use super::native_type_rules::{KEYWORDS, RelativeScope, Rules};
use super::rank::Rank;

pub(super) fn items(
    server: &Server,
    file_id: FileId,
    file: &File,
    tokens: &[Token<'_>],
    offset: u32,
    context: Context<'_>,
    names: &NamespaceScope,
) -> Vec<CompletionEntry> {
    let Context::Bare { prefix, intent: NameIntent::NativeType(context) } = context else { return Vec::new() };
    if context.position == Position::Catch {
        return Vec::new();
    }
    let replace = tokens
        .iter()
        .find(|token| {
            token.kind.is_identifier_maybe_reserved()
                && token.start.offset <= offset
                && offset <= token.start.offset + token.value.len() as u32
        })
        .map_or(Range::new(offset, offset), |token| {
            Range::new(token.start.offset, token.start.offset + token.value.len() as u32)
        });
    let range = hint_range(tokens, context, offset);
    let Some(before) = file.contents.get(range.start as usize..replace.start as usize) else { return Vec::new() };
    let Some(after) = file.contents.get(replace.end as usize..range.end as usize) else { return Vec::new() };
    let mut source = b"<?php function __mago(".to_vec();
    source.extend_from_slice(before);
    let marker = source.len() as u32;
    source.extend_from_slice(b"MagoCompletionType");
    source.extend_from_slice(after);
    source.extend_from_slice(b"\n $value) {}");
    let fragment = File::ephemeral(b"completion.php".as_slice().into(), source.into());
    let arena = LocalArena::new();
    let mut settings = server.parser_settings();
    settings.recover_incomplete = false;
    let program = parse_file_borrowed_with_settings(&arena, &fragment, settings);
    if program.has_errors() {
        return Vec::new();
    }
    let hint = program.statements.iter().find_map(|statement| match statement {
        Statement::Function(function) => function.parameter_list.parameters.iter().next()?.hint.as_ref(),
        _ => None,
    });
    let Some(hint) = hint else { return Vec::new() };
    let rules = Rules {
        version: server.php_version,
        position: context.position,
        scope: relative_scope(server, file_id, tokens, context, offset),
        names,
    };
    items::finalize(
        KEYWORDS
            .iter()
            .filter_map(|name| {
                let matched = matcher::score(prefix, name)?;
                if !rules.accepts(hint, marker, name) {
                    return None;
                }
                let label = String::from_utf8_lossy(name).into_owned();
                Some((
                    Rank::new(matched),
                    CompletionEntry {
                        insert: Some(CompletionInsert::Text(label.clone())),
                        replace: Some(replace),
                        ..CompletionEntry::new(label, CompletionKind::Keyword)
                    },
                ))
            })
            .collect(),
    )
}

/// Bound the hint without treating a parameter's reference marker as an intersection.
fn hint_range(tokens: &[Token<'_>], context: NativeTypeContext, offset: u32) -> Range {
    let mut start = context.start_token;
    if context.position == Position::Property {
        while tokens.get(start).is_some_and(|token| lookup::is_trivia(token.kind) || token.kind == TokenKind::Static) {
            start += 1;
        }
    }
    let mut range = Range::new(tokens.get(start).map_or(offset, |token| token.start.offset).min(offset), offset);
    let mut depth = 0u32;
    let mut complete = false;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            kind if lookup::is_trivia(kind) => {}
            kind if context.position == Position::Constant
                && complete
                && (kind.is_identifier_maybe_reserved()
                    || matches!(kind, TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier)) =>
            {
                break;
            }
            TokenKind::LeftParenthesis => {
                depth += 1;
                complete = false;
            }
            TokenKind::RightParenthesis if depth > 0 => {
                depth -= 1;
                complete = true;
            }
            TokenKind::Ampersand
                if tokens[index + 1..].iter().find(|token| !lookup::is_trivia(token.kind)).is_some_and(|token| {
                    matches!(token.kind, TokenKind::Variable | TokenKind::DotDotDot | TokenKind::Ampersand)
                }) =>
            {
                break;
            }
            TokenKind::Question | TokenKind::Pipe | TokenKind::Ampersand => complete = false,
            TokenKind::Identifier
            | TokenKind::QualifiedIdentifier
            | TokenKind::FullyQualifiedIdentifier
            | TokenKind::Self_
            | TokenKind::Parent
            | TokenKind::Static
            | TokenKind::Array
            | TokenKind::Callable
            | TokenKind::Null
            | TokenKind::False
            | TokenKind::True
            | TokenKind::Enum
            | TokenKind::From => complete = true,
            _ => break,
        }
        range.end = range.end.max(token.start.offset + token.value.len() as u32);
    }
    range
}

fn relative_scope(
    server: &Server,
    file_id: FileId,
    tokens: &[Token<'_>],
    context: NativeTypeContext,
    offset: u32,
) -> RelativeScope {
    if let Some(callable) = context.callable {
        let arrow = tokens[callable].kind == TokenKind::Fn;
        let closure = tokens[callable + 1..]
            .iter()
            .find(|token| !lookup::is_trivia(token.kind) && token.kind != TokenKind::Ampersand)
            .is_some_and(|token| token.kind == TokenKind::LeftParenthesis);
        // PHP permits rebinding closure scope, even when no class encloses it.
        if arrow || closure {
            return RelativeScope { allows_self: true, allows_parent: true, arrow };
        }
    }
    let declaration = context.callable.map_or(offset.saturating_sub(1), |index| tokens[index].start.offset);
    let class = server
        .codebase()
        .file_signatures
        .get(&file_id)
        .into_iter()
        .flat_map(|signature| &signature.ast_nodes)
        .filter(|node| !node.is_function)
        .filter_map(|node| server.codebase().get_class_like_by_name(node.name))
        .filter(|class| class.span.start.offset <= declaration && declaration < class.span.end.offset)
        .min_by_key(|class| class.span.end.offset - class.span.start.offset);
    let Some(class) = class else { return RelativeScope::default() };
    if context.callable.is_some() {
        let mut braces = Vec::new();
        let mut header_depth = 0u32;
        let mut class_body = None;
        for token in tokens.iter().take_while(|token| token.start.offset < declaration) {
            match token.kind {
                TokenKind::LeftBrace | TokenKind::DollarLeftBrace => braces.push(token.start.offset),
                TokenKind::RightBrace => {
                    braces.pop();
                }
                _ => {}
            }
            if token.start.offset < class.span.start.offset || class_body.is_some() {
                continue;
            }
            match token.kind {
                TokenKind::LeftParenthesis | TokenKind::LeftBracket | TokenKind::HashLeftBracket => header_depth += 1,
                TokenKind::RightParenthesis | TokenKind::RightBracket => header_depth = header_depth.saturating_sub(1),
                TokenKind::LeftBrace if header_depth == 0 => class_body = Some(token.start.offset),
                _ => {}
            }
        }
        // A recovered nested function may have lost its CST scope. Only a direct
        // class member may use the class as a named callable's type scope.
        if class_body.is_none() || braces.last().copied() != class_body {
            return RelativeScope::default();
        }
    }
    RelativeScope {
        allows_self: true,
        allows_parent: class.kind.is_trait() || class.direct_parent_class.is_some(),
        arrow: false,
    }
}
