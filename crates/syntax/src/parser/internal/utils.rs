use mago_database::file::HasFileId;
use mago_span::Span;

use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::token::Token;
use crate::token::TokenKind;

#[inline]
pub fn peek<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Token<'arena>, ParseError> {
    match stream.peek() {
        Some(Ok(token)) => Ok(token),
        Some(Err(error)) => Err(error.into()),
        None => Err(unexpected(stream, None, &[])),
    }
}

#[inline]
pub fn maybe_peek<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Option<Token<'arena>>, ParseError> {
    match stream.peek() {
        Some(Ok(token)) => Ok(Some(token)),
        Some(Err(error)) => Err(error.into()),
        None => Ok(None),
    }
}

#[inline]
pub fn peek_nth<'arena>(stream: &mut TokenStream<'_, 'arena>, n: usize) -> Result<Token<'arena>, ParseError> {
    match stream.peek_nth(n) {
        Some(Ok(token)) => Ok(token),
        Some(Err(error)) => Err(error.into()),
        None => Err(unexpected(stream, None, &[])),
    }
}

#[inline]
pub fn maybe_peek_nth<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    n: usize,
) -> Result<Option<Token<'arena>>, ParseError> {
    match stream.peek_nth(n) {
        Some(Ok(token)) => Ok(Some(token)),
        Some(Err(error)) => Err(error.into()),
        None => Ok(None),
    }
}

#[inline]
pub fn expect_any<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Token<'arena>, ParseError> {
    match stream.advance() {
        Some(Ok(token)) => Ok(token),
        Some(Err(error)) => Err(error.into()),
        None => Err(unexpected(stream, None, &[])),
    }
}

#[inline]
pub fn expect<'arena>(stream: &mut TokenStream<'_, 'arena>, kind: TokenKind) -> Result<Token<'arena>, ParseError> {
    let token = expect_any(stream)?;

    if kind == token.kind { Ok(token) } else { Err(unexpected(stream, Some(token), &[kind])) }
}

#[inline]
pub fn expect_one_of<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    one_of: &[TokenKind],
) -> Result<Token<'arena>, ParseError> {
    let token = expect_any(stream)?;

    if one_of.contains(&token.kind) { Ok(token) } else { Err(unexpected(stream, Some(token), one_of)) }
}

#[inline]
pub fn maybe_expect<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    kind: TokenKind,
) -> Result<Option<Token<'arena>>, ParseError> {
    let next = match stream.peek() {
        Some(Ok(token)) => token,
        Some(Err(error)) => return Err(error.into()),
        None => return Ok(None),
    };

    if kind == next.kind {
        let token = match stream.advance() {
            Some(result) => result?,
            None => unreachable!("the token was peeked, so it should be available"),
        };

        Ok(Some(token))
    } else {
        Ok(None)
    }
}

#[inline]
pub fn expect_span<'arena>(stream: &mut TokenStream<'_, 'arena>, kind: TokenKind) -> Result<Span, ParseError> {
    expect(stream, kind).map(|token| token.span)
}

#[inline]
pub fn expect_one_of_keyword<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    one_of: &[TokenKind],
) -> Result<Keyword<'arena>, ParseError> {
    expect_one_of(stream, one_of).map(to_keyword)
}

#[inline]
pub fn maybe_expect_keyword<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    kind: TokenKind,
) -> Result<Option<Keyword<'arena>>, ParseError> {
    maybe_expect(stream, kind).map(|maybe_token| maybe_token.map(to_keyword))
}

#[inline]
pub fn expect_keyword<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    kind: TokenKind,
) -> Result<Keyword<'arena>, ParseError> {
    expect(stream, kind).map(to_keyword)
}

#[inline]
pub fn expect_any_keyword<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Keyword<'arena>, ParseError> {
    expect_any(stream).map(to_keyword)
}

#[inline]
pub fn to_keyword(token: Token) -> Keyword {
    Keyword { span: token.span, value: token.value }
}

#[inline]
pub fn unexpected<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    token: Option<Token>,
    one_of: &[TokenKind],
) -> ParseError {
    if let Some(token) = token {
        ParseError::UnexpectedToken(one_of.to_vec(), token.kind, token.span)
    } else {
        ParseError::UnexpectedEndOfFile(one_of.to_vec(), stream.file_id(), stream.get_position())
    }
}
