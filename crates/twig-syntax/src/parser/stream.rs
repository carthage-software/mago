//! Token stream consumed by the Twig parser.
//!
//! A lazy pull-from-lexer buffer, no rewind, no cursor. Trivia produced by
//! the lexer are collected into a separate side channel as the parser
//! advances - the stream never stores them in its lookahead buffer.
//!
//! Lexer errors propagate through the fallible methods (`has_reached_eof`,
//! `lookahead`, `peek_kind`, `is_at`, `consume`, ...) via `Result`. There is
//! no deferred-error slot: the parser sees a lex failure the moment it
//! reaches for a token that cannot be produced.
#![allow(clippy::missing_errors_doc)]

use mago_syntax_core::parser::LookaheadBuf;
use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::Trivia;
use crate::ast::TriviaKind;
use crate::error::ParseError;
use crate::error::SyntaxError;
use crate::lexer::TwigLexer;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

#[derive(Debug)]
pub struct TokenStream<'arena> {
    arena: &'arena Bump,
    lexer: TwigLexer<'arena>,
    /// Non-trivia lookahead buffer pulled on demand from the lexer.
    buffer: LookaheadBuf<TwigToken<'arena>, 8>,
    /// Trivia collected as the lexer is drained.
    trivia: BVec<'arena, Trivia<'arena>>,
    /// End position of the most recently consumed significant token.
    position: Position,
    file_id: FileId,
}

impl<'arena> TokenStream<'arena> {
    #[inline]
    pub fn new(arena: &'arena Bump, lexer: TwigLexer<'arena>) -> Self {
        let position = lexer.current_position();
        let file_id = lexer.file_id();

        Self { arena, lexer, buffer: LookaheadBuf::new(), trivia: BVec::new_in(arena), position, file_id }
    }

    #[inline]
    #[must_use]
    pub fn arena(&self) -> &'arena Bump {
        self.arena
    }

    /// Build a span anchored to this stream's file.
    #[inline]
    #[must_use]
    pub fn span(&self, start: Position, end: Position) -> Span {
        Span::new(self.file_id(), start, end)
    }

    /// Span of a token, anchored to this stream's file.
    #[inline]
    #[must_use]
    pub fn span_of(&self, tok: &TwigToken<'_>) -> Span {
        tok.span_for(self.file_id())
    }

    /// Wrap a token as a [`Keyword`].
    #[inline]
    #[must_use]
    pub fn keyword_from(&self, tok: &TwigToken<'arena>) -> Keyword<'arena> {
        Keyword { span: tok.span_for(self.file_id()), value: tok.value }
    }

    /// Wrap a token as an [`Identifier`].
    #[inline]
    #[must_use]
    pub fn identifier_from(&self, tok: &TwigToken<'arena>) -> Identifier<'arena> {
        Identifier { span: tok.span_for(self.file_id()), value: tok.value }
    }

    /// End of the last consumed significant token.
    #[inline]
    #[must_use]
    pub const fn current_position(&self) -> Position {
        self.position
    }

    /// Alias retained for existing parser callers.
    #[inline]
    #[must_use]
    pub const fn prev_end(&self) -> Position {
        self.position
    }

    /// Drain the trivia accumulator into a [`Sequence`] for attachment to
    /// a [`Template`](crate::ast::Template).
    #[inline]
    pub fn get_trivia(&mut self) -> Sequence<'arena, Trivia<'arena>> {
        let trivia = std::mem::replace(&mut self.trivia, BVec::new_in(self.arena));
        Sequence::new(trivia)
    }

    #[inline]
    #[must_use]
    pub fn alloc<T>(&self, v: T) -> &'arena T {
        self.arena.alloc(v)
    }

    #[inline]
    #[must_use]
    pub fn bvec<T>(&self) -> BVec<'arena, T> {
        BVec::new_in(self.arena)
    }

    /// Fill the lookahead buffer until it holds at least `n` tokens, or the
    /// lexer reaches EOF. Returns `Ok(Some(n))` if the buffer is large
    /// enough, `Ok(None)` if EOF was reached first, or `Err` for a lexer
    /// failure.
    #[inline]
    fn fill_buffer(&mut self, n: usize) -> Result<Option<usize>, SyntaxError> {
        if self.buffer.len() >= n {
            return Ok(Some(n));
        }
        self.fill_buffer_slow(n)
    }

    #[inline(never)]
    fn fill_buffer_slow(&mut self, n: usize) -> Result<Option<usize>, SyntaxError> {
        while self.buffer.len() < n {
            match self.lexer.advance() {
                Some(result) => {
                    let token = result?;
                    if token.kind.is_trivia() {
                        if let Some(kind) = trivia_kind_for(token.kind) {
                            self.trivia.push(Trivia { kind, span: token.span_for(self.file_id()), value: token.value });
                        }
                        continue;
                    }
                    self.buffer.push_back(token);
                }
                None => return Ok(None),
            }
        }
        Ok(Some(n))
    }

    /// Are there no more significant tokens?
    #[inline]
    pub fn has_reached_eof(&mut self) -> Result<bool, SyntaxError> {
        Ok(self.fill_buffer(1)?.is_none())
    }

    /// Consume and return the next significant token. Errors on EOF or
    /// lexer failure.
    #[inline]
    pub fn consume(&mut self) -> Result<TwigToken<'arena>, ParseError> {
        match self.advance() {
            Some(Ok(token)) => Ok(token),
            Some(Err(error)) => Err(error.into()),
            None => Err(self.unexpected(None, &[])),
        }
    }

    /// Consume iff the next token has the given kind.
    #[inline]
    pub fn eat(&mut self, kind: TwigTokenKind) -> Result<TwigToken<'arena>, ParseError> {
        if let Some(token) = self.buffer.get(0) {
            if token.kind == kind {
                let _ = self.buffer.pop_front();
                self.position = token.end();
                return Ok(token);
            }

            return Err(self.unexpected(Some(token), &[kind]));
        }

        match self.peek_kind(0)? {
            Some(k) if k == kind => self.consume(),
            Some(_) => match self.lookahead(0)? {
                Some(token) => Err(self.unexpected(Some(token), &[kind])),
                None => Err(self.unexpected(None, &[kind])),
            },
            None => Err(self.unexpected(None, &[kind])),
        }
    }

    #[inline]
    pub fn consume_span(&mut self) -> Result<Span, ParseError> {
        let file_id = self.file_id();
        self.consume().map(|t| t.span_for(file_id))
    }

    #[inline]
    pub fn eat_span(&mut self, kind: TwigTokenKind) -> Result<Span, ParseError> {
        let file_id = self.file_id();
        self.eat(kind).map(|t| t.span_for(file_id))
    }

    /// Advance the underlying lexer stream and return the next significant
    /// token. Returns `None` at EOF and propagates lexer errors.
    #[inline]
    pub fn advance(&mut self) -> Option<Result<TwigToken<'arena>, SyntaxError>> {
        match self.fill_buffer(1) {
            Ok(Some(_)) => {
                let token = self.buffer.pop_front()?;
                self.position = token.end();
                Some(Ok(token))
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }

    /// Is the next token of the given kind?
    #[inline]
    pub fn is_at(&mut self, kind: TwigTokenKind) -> Result<bool, ParseError> {
        if let Some(t) = self.buffer.get(0) {
            return Ok(t.kind == kind);
        }

        Ok(self.peek_kind(0)? == Some(kind))
    }

    /// Look at the Nth-ahead significant token (0 = next).
    #[inline]
    pub fn lookahead(&mut self, n: usize) -> Result<Option<TwigToken<'arena>>, ParseError> {
        if n < self.buffer.len() {
            return Ok(self.buffer.get(n));
        }

        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => Ok(self.buffer.get(n)),
            Ok(None) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Kind of the Nth-ahead significant token (0 = next).
    #[inline]
    pub fn peek_kind(&mut self, n: usize) -> Result<Option<TwigTokenKind>, ParseError> {
        if n < self.buffer.len() {
            return Ok(self.buffer.get(n).map(|t| t.kind));
        }

        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => Ok(self.buffer.get(n).map(|t| t.kind)),
            Ok(None) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Build a `ParseError` describing an unexpected token / EOF.
    #[inline]
    #[must_use]
    pub fn unexpected(&self, found: Option<TwigToken<'_>>, expected: &[TwigTokenKind]) -> ParseError {
        let msg =
            if expected.is_empty() { "unexpected token".to_string() } else { format!("expected one of {expected:?}") };
        match found {
            Some(tok) => ParseError::UnexpectedToken(msg, self.span_of(&tok)),
            None => ParseError::UnexpectedEof(self.file_id(), msg, self.position),
        }
    }

    /// Alias for [`is_at`].
    #[inline]
    pub fn is_kind(&mut self, kind: TwigTokenKind) -> Result<bool, ParseError> {
        self.is_at(kind)
    }

    /// Try to consume a token of the given kind; return it if matched.
    #[inline]
    pub fn try_consume(&mut self, kind: TwigTokenKind) -> Result<Option<TwigToken<'arena>>, ParseError> {
        if self.peek_kind(0)? == Some(kind) { self.consume().map(Some) } else { Ok(None) }
    }

    /// Try to consume a `Name` token with the given literal value.
    #[inline]
    pub fn try_consume_name(&mut self, name: &str) -> Result<Option<TwigToken<'arena>>, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind == TwigTokenKind::Name && tok.value == name => self.consume().map(Some),
            _ => Ok(None),
        }
    }

    /// Eat with a custom expectation message (used for diagnostics).
    pub fn expect_kind(&mut self, kind: TwigTokenKind, what: &str) -> Result<TwigToken<'arena>, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind == kind => self.consume(),
            Some(tok) => Err(ParseError::UnexpectedToken(what.to_string(), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), what.to_string(), self.position)),
        }
    }

    pub fn expect_name(&mut self, what: &str) -> Result<TwigToken<'arena>, ParseError> {
        self.expect_kind(TwigTokenKind::Name, what)
    }

    /// Expect a `Name` token with a specific literal value (e.g. `in`, `as`).
    pub fn expect_name_value(&mut self, expected: &str) -> Result<TwigToken<'arena>, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind == TwigTokenKind::Name && tok.value == expected => self.consume(),
            Some(tok) => Err(ParseError::UnexpectedToken(format!("expected `{expected}`"), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), format!("expected `{expected}`"), self.position)),
        }
    }

    /// Accept a `Name` token, or a word-keyword token that can also appear
    /// as an identifier (`in`, `not`, `is`, `and`, `or`, `xor`, `matches`,
    /// `divisible by`, etc.).
    pub fn expect_flex_name(&mut self, what: &str) -> Result<TwigToken<'arena>, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind == TwigTokenKind::Name || is_keyword_usable_as_name(tok.kind) => self.consume(),
            Some(tok) => Err(ParseError::UnexpectedToken(what.to_string(), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), what.to_string(), self.position)),
        }
    }

    pub fn expect_name_or_null(&mut self, what: &str) -> Result<TwigToken<'arena>, ParseError> {
        self.expect_flex_name(what)
    }

    pub fn is_start_of_primary(&mut self) -> Result<bool, ParseError> {
        let Some(kind) = self.peek_kind(0)? else { return Ok(false) };
        Ok(matches!(
            kind,
            TwigTokenKind::Name
                | TwigTokenKind::Number
                | TwigTokenKind::StringSingleQuoted
                | TwigTokenKind::StringDoubleQuoted
                | TwigTokenKind::DoubleQuoteStart
                | TwigTokenKind::Plus
                | TwigTokenKind::Minus
                | TwigTokenKind::Not
                | TwigTokenKind::DotDotDot
                | TwigTokenKind::LeftBracket
                | TwigTokenKind::LeftParen
                | TwigTokenKind::LeftBrace
        ))
    }

    #[inline]
    pub fn expect_block_start(&mut self) -> Result<TwigToken<'arena>, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind.is_open_block() => self.consume(),
            Some(tok) => Err(ParseError::UnexpectedToken("expected `{%`".to_string(), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), "expected `{%`".to_string(), self.position)),
        }
    }

    #[inline]
    pub fn expect_block_end(&mut self) -> Result<Span, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind.is_close_block() => {
                let span = self.span_of(&tok);
                self.consume()?;
                Ok(span)
            }
            Some(tok) => Err(ParseError::UnexpectedToken("expected `%}`".to_string(), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), "expected `%}`".to_string(), self.position)),
        }
    }

    #[inline]
    pub fn expect_variable_end(&mut self) -> Result<Span, ParseError> {
        match self.lookahead(0)? {
            Some(tok) if tok.kind.is_close_variable() => {
                let span = self.span_of(&tok);
                self.consume()?;
                Ok(span)
            }
            Some(tok) => Err(ParseError::UnexpectedToken("expected `}}`".to_string(), self.span_of(&tok))),
            None => Err(ParseError::UnexpectedEof(self.file_id(), "expected `}}`".to_string(), self.position)),
        }
    }

    #[inline]
    pub fn is_block_end(&mut self) -> Result<bool, ParseError> {
        Ok(self.peek_kind(0)?.is_some_and(TwigTokenKind::is_close_block))
    }

    #[inline]
    pub fn expect_eof(&mut self) -> Result<(), ParseError> {
        match self.lookahead(0)? {
            None => Ok(()),
            Some(tok) => Err(ParseError::UnexpectedToken(
                format!("expected end of template, got {:?}", tok.kind),
                self.span_of(&tok),
            )),
        }
    }
}

impl HasFileId for TokenStream<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.file_id
    }
}

#[inline]
#[must_use]
pub fn looks_like_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(c) = chars.next() else { return false };
    if !(c.is_ascii_alphabetic() || c == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Keyword token kinds that can also appear in identifier positions
/// inside Twig templates (e.g. `is`, `in`, `and`, `divisible by`).
#[inline]
#[must_use]
pub fn is_keyword_usable_as_name(kind: TwigTokenKind) -> bool {
    matches!(
        kind,
        TwigTokenKind::And
            | TwigTokenKind::Or
            | TwigTokenKind::Xor
            | TwigTokenKind::In
            | TwigTokenKind::NotIn
            | TwigTokenKind::Is
            | TwigTokenKind::Not
            | TwigTokenKind::Matches
            | TwigTokenKind::StartsWith
            | TwigTokenKind::EndsWith
            | TwigTokenKind::HasSome
            | TwigTokenKind::HasEvery
            | TwigTokenKind::SameAs
            | TwigTokenKind::DivisibleBy
            | TwigTokenKind::BAnd
            | TwigTokenKind::BOr
            | TwigTokenKind::BXor
    )
}

#[inline]
fn trivia_kind_for(kind: TwigTokenKind) -> Option<TriviaKind> {
    match kind {
        TwigTokenKind::Whitespace => Some(TriviaKind::Whitespace),
        TwigTokenKind::Comment => Some(TriviaKind::Comment),
        TwigTokenKind::InlineComment => Some(TriviaKind::InlineComment),
        _ => None,
    }
}
