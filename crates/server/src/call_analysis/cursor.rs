//! Locate the active call and argument in lexer tokens, including unfinished calls.

use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

pub(crate) struct CallCursor<'a> {
    pub name: usize,
    pub open: usize,
    pub argument_start: usize,
    pub argument: u32,
    pub argument_name: Option<&'a [u8]>,
    pub attribute: bool,
}

/// `tokens` excludes trivia. Delimiter frames keep commas in nested expressions
/// out of the call count.
pub(crate) fn enclosing_call<'a>(tokens: &[Token<'a>], offset: u32) -> Option<CallCursor<'a>> {
    let mut frames: Vec<(TokenKind, usize, u32, usize)> = Vec::new();
    let end = tokens.partition_point(|token| token.start.offset < offset);
    for (index, token) in tokens[..end].iter().enumerate() {
        match token.kind {
            TokenKind::LeftParenthesis
            | TokenKind::LeftBracket
            | TokenKind::HashLeftBracket
            | TokenKind::LeftBrace
            | TokenKind::DollarLeftBrace => {
                frames.push((token.kind, index, 0, index + 1));
            }
            TokenKind::RightParenthesis | TokenKind::RightBracket | TokenKind::RightBrace => {
                frames.pop();
            }
            TokenKind::Comma => {
                if let Some((_, _, argument, start)) = frames.last_mut() {
                    *argument += 1;
                    *start = index + 1;
                }
            }
            _ => {}
        }
    }
    for (frame, &(kind, open, argument, start)) in frames.iter().enumerate().rev() {
        if kind == TokenKind::LeftBrace {
            return None;
        }
        if kind != TokenKind::LeftParenthesis || open == 0 {
            continue;
        }
        let name = open - 1;
        let previous = name.checked_sub(1).map(|index| tokens[index].kind);
        let member = matches!(
            previous,
            Some(TokenKind::ColonColon | TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan)
        );
        if !member
            && (matches!(tokens[name].kind, TokenKind::Function | TokenKind::Fn | TokenKind::Use)
                || tokens[name].kind == TokenKind::Ampersand
                    && matches!(previous, Some(TokenKind::Function | TokenKind::Fn)))
        {
            return None;
        }
        if !(is_name(tokens[name].kind)
            || matches!(
                tokens[name].kind,
                TokenKind::Variable | TokenKind::LiteralString | TokenKind::RightParenthesis | TokenKind::RightBracket
            )
            || member && tokens[name].kind.is_identifier_maybe_reserved()
            || previous == Some(TokenKind::New)
                && matches!(tokens[name].kind, TokenKind::Self_ | TokenKind::Static | TokenKind::Parent))
        {
            continue;
        }
        if previous == Some(TokenKind::Function)
            || (previous == Some(TokenKind::Ampersand) && name >= 2 && tokens[name - 2].kind == TokenKind::Function)
        {
            return None;
        }
        let argument_name = tokens
            .get(start)
            .filter(|token| token.kind.is_identifier_maybe_reserved())
            .filter(|_| start + 1 < end && tokens[start + 1].kind == TokenKind::Colon)
            .map(|token| token.value);
        let attribute = frame.checked_sub(1).is_some_and(|parent| frames[parent].0 == TokenKind::HashLeftBracket);
        return Some(CallCursor { name, open, argument_start: start, argument, argument_name, attribute });
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallEnd {
    Closed,
    Boundary,
    Eof,
}

/// Split a call's arguments without treating nested commas as separators, and
/// report how the call ends. An unclosed call ends at EOF or at a statement
/// boundary of its own level, but unmatched nested delimiters fail.
pub(crate) fn argument_ranges(tokens: &[Token<'_>], open: usize) -> Option<(Vec<std::ops::Range<usize>>, CallEnd)> {
    let mut ranges = Vec::new();
    let mut closers = Vec::new();
    let mut start = open + 1;
    let mut end = (tokens.len(), CallEnd::Eof);
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            TokenKind::LeftParenthesis => closers.push(TokenKind::RightParenthesis),
            TokenKind::LeftBracket | TokenKind::HashLeftBracket => closers.push(TokenKind::RightBracket),
            TokenKind::LeftBrace | TokenKind::DollarLeftBrace => closers.push(TokenKind::RightBrace),
            TokenKind::RightParenthesis if closers.is_empty() => {
                ranges.push(start..index);
                return Some((ranges, CallEnd::Closed));
            }
            TokenKind::Semicolon | TokenKind::RightBracket | TokenKind::RightBrace if closers.is_empty() => {
                end = (index, CallEnd::Boundary);
                break;
            }
            TokenKind::RightParenthesis | TokenKind::RightBracket | TokenKind::RightBrace => {
                if closers.pop() != Some(token.kind) {
                    return None;
                }
            }
            TokenKind::Comma if closers.is_empty() => {
                ranges.push(start..index);
                start = index + 1;
            }
            _ => {}
        }
    }
    if !closers.is_empty() {
        return None;
    }
    ranges.push(start..end.0);
    Some((ranges, end.1))
}

pub(super) fn is_name(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier)
}

#[cfg(test)]
mod tests {
    use mago_database::file::File;
    use mago_syntax::settings::LexerSettings;

    use crate::lookup::{TokenCache, is_trivia};

    use super::*;

    #[test]
    fn argument_ranges_stop_unclosed_calls_at_their_statement_boundary() {
        for (source, expected) in [
            ("f(a, b);", Some((&["a", "b"][..], CallEnd::Closed))),
            ("f();", Some((&[""][..], CallEnd::Closed))),
            ("f(a, g(b, c), [d, e]);", Some((&["a", "g ( b , c )", "[ d , e ]"][..], CallEnd::Closed))),
            ("f(a, b", Some((&["a", "b"][..], CallEnd::Eof))),
            ("f(a,\n    b", Some((&["a", "b"][..], CallEnd::Eof))),
            ("f(a,\n$x = 1;", Some((&["a", "$x = 1"][..], CallEnd::Boundary))),
            ("{ f(a\n}", Some((&["a"][..], CallEnd::Boundary))),
            ("[f(a]", Some((&["a"][..], CallEnd::Boundary))),
            ("f(function () { $x; }, b\n$y;", Some((&["function ( ) { $x ; }", "b $y"][..], CallEnd::Boundary))),
            ("f([a, b", None),
            ("f([a, b;", None),
            ("f(g(a;\n}", None),
            ("f((a]", None),
        ] {
            let file = File::ephemeral(b"a.php".as_slice().into(), format!("<?php {source}").into_bytes().into());
            let tokens: Vec<_> = TokenCache::new(LexerSettings::default())
                .lex(&file)
                .into_iter()
                .filter(|token| !is_trivia(token.kind))
                .collect();
            let Some(open) = tokens.iter().position(|token| token.kind == TokenKind::LeftParenthesis) else {
                panic!("{source}: missing call")
            };
            let actual = argument_ranges(&tokens, open).map(|(ranges, end)| {
                let arguments: Vec<_> = ranges
                    .into_iter()
                    .map(|range| {
                        tokens[range]
                            .iter()
                            .map(|token| String::from_utf8_lossy(token.value))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect();
                (arguments, end)
            });
            let expected = expected.map(|(arguments, end)| {
                (arguments.iter().map(|argument| (*argument).to_owned()).collect::<Vec<_>>(), end)
            });
            assert_eq!(actual, expected, "{source}");
        }
    }
}
