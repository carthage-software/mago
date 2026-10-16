//! Find the declaration that owns a native type hint, including unfinished hints.

use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::lookup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Position {
    Parameter,
    Property,
    Constant,
    Return,
    Catch,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NativeTypeContext {
    pub position: Position,
    pub start_token: usize,
    pub callable: Option<usize>,
}

/// Follow a native hint back to its declaration. Stop at values and assignments
/// so parameter defaults and ordinary calls keep expression completion.
pub(super) fn detect(tokens: &[Token<'_>], before: usize) -> Option<NativeTypeContext> {
    let mut start = before;
    let mut depth = 0u32;
    let mut earlier_parameter = false;
    for (index, token) in tokens[..before].iter().enumerate().rev() {
        if lookup::is_trivia(token.kind) {
            continue;
        }
        match token.kind {
            TokenKind::RightParenthesis | TokenKind::RightBracket => depth += 1,
            TokenKind::LeftParenthesis | TokenKind::LeftBracket if depth > 0 => depth -= 1,
            _ if depth > 0 => {}
            TokenKind::LeftParenthesis => {
                if let Some(callable) = parameter_list_owner(tokens, index) {
                    return Some(NativeTypeContext {
                        position: Position::Parameter,
                        start_token: start,
                        callable: Some(callable),
                    });
                }
                if previous(tokens, index).is_some_and(|i| tokens[i].kind == TokenKind::Catch) {
                    return Some(NativeTypeContext { position: Position::Catch, start_token: start, callable: None });
                }
                if let Some(colon) = previous(tokens, index).filter(|i| tokens[*i].kind == TokenKind::Colon) {
                    return Some(NativeTypeContext {
                        position: Position::Return,
                        start_token: start,
                        callable: Some(return_owner(tokens, colon)?),
                    });
                }
                // Parenthesized intersection in a parameter's DNF type.
                if !previous(tokens, index).is_some_and(|i| {
                    matches!(tokens[i].kind, TokenKind::LeftParenthesis | TokenKind::Pipe | TokenKind::Const)
                }) {
                    return None;
                }
            }
            TokenKind::Comma => earlier_parameter = true,
            TokenKind::Const if previous(tokens, index).is_none_or(|i| tokens[i].kind != TokenKind::ColonColon) => {
                return Some(NativeTypeContext {
                    position: Position::Constant,
                    start_token: index + 1,
                    callable: None,
                });
            }
            TokenKind::Colon if !earlier_parameter => {
                return Some(NativeTypeContext {
                    position: Position::Return,
                    start_token: start,
                    callable: Some(return_owner(tokens, index)?),
                });
            }
            TokenKind::Public | TokenKind::Protected | TokenKind::Private | TokenKind::Var | TokenKind::Readonly
                if !earlier_parameter =>
            {
                return Some(NativeTypeContext { position: Position::Property, start_token: start, callable: None });
            }
            TokenKind::LeftBrace | TokenKind::RightBrace | TokenKind::Semicolon => return None,
            _ if earlier_parameter => {}
            TokenKind::Identifier
            | TokenKind::QualifiedIdentifier
            | TokenKind::FullyQualifiedIdentifier
            | TokenKind::NamespaceSeparator
            | TokenKind::Question
            | TokenKind::Pipe
            | TokenKind::Ampersand
            | TokenKind::Self_
            | TokenKind::Parent
            | TokenKind::Static
            | TokenKind::Array
            | TokenKind::Callable
            | TokenKind::Null
            | TokenKind::False
            | TokenKind::True
            | TokenKind::Enum
            | TokenKind::From => {}
            _ => return None,
        }
        if !earlier_parameter {
            start = index;
        }
    }
    None
}

fn previous(tokens: &[Token<'_>], before: usize) -> Option<usize> {
    (0..before).rev().find(|index| !lookup::is_trivia(tokens[*index].kind))
}

fn parameter_list_owner(tokens: &[Token<'_>], open: usize) -> Option<usize> {
    let mut index = previous(tokens, open)?;
    if matches!(tokens[index].kind, TokenKind::Function | TokenKind::Fn) {
        return Some(index);
    }
    if tokens[index].kind.is_identifier_maybe_reserved() {
        index = previous(tokens, index)?;
    }
    if tokens[index].kind == TokenKind::Ampersand {
        index = previous(tokens, index)?;
    }
    (tokens[index].kind == TokenKind::Function).then_some(index)
}

fn return_owner(tokens: &[Token<'_>], before: usize) -> Option<usize> {
    let close = previous(tokens, before).filter(|index| tokens[*index].kind == TokenKind::RightParenthesis)?;
    let mut depth = 1;
    for index in (0..close).rev() {
        match tokens[index].kind {
            TokenKind::RightParenthesis => depth += 1,
            TokenKind::LeftParenthesis => {
                depth -= 1;
                if depth == 0 {
                    return parameter_list_owner(tokens, index).or_else(|| {
                        let previous = previous(tokens, index)?;
                        (tokens[previous].kind == TokenKind::Use).then(|| return_owner(tokens, previous)).flatten()
                    });
                }
            }
            _ => {}
        }
    }
    None
}
