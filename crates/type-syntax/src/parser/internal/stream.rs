use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::parser::LookaheadBuf;

use crate::error::ParseError;
use crate::error::SyntaxError;
use crate::lexer::TypeLexer;
use crate::token::TypeToken;
use crate::token::TypeTokenKind;

/// A buffered token stream that wraps a `TypeLexer`, providing lookahead
/// capabilities and automatically skipping trivia tokens (whitespace, comments).
#[derive(Debug)]
pub struct TypeTokenStream<'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) lexer: TypeLexer<'arena>,
    /// Cached file id so hot-path span construction avoids the lexer → input
    /// method hop on every consumed token.
    file_id: FileId,
    buffer: LookaheadBuf<TypeToken<'arena>, 64>,
    position: Position,
}

impl<'arena> TypeTokenStream<'arena> {
    /// Creates a new `TypeTokenStream` wrapping the given `TypeLexer`.
    #[inline(always)]
    pub fn new(arena: &'arena Bump, lexer: TypeLexer<'arena>) -> TypeTokenStream<'arena> {
        let position = lexer.current_position();
        let file_id = lexer.file_id();
        TypeTokenStream { arena, lexer, file_id, buffer: LookaheadBuf::new(), position }
    }

    /// Consume the next token and return its [`Span`]. Equivalent to
    /// `stream.consume_span()?` but avoids the extra
    /// method dispatch through `HasFileId`.
    #[inline(always)]
    pub fn consume_span(&mut self) -> Result<Span, ParseError> {
        let token = self.consume()?;
        Ok(Span::new(self.file_id, token.start, token.end()))
    }

    /// Eat a token of `kind` and return its [`Span`].
    #[inline(always)]
    pub fn eat_span(&mut self, kind: TypeTokenKind) -> Result<Span, ParseError> {
        let token = self.eat(kind)?;
        Ok(Span::new(self.file_id, token.start, token.end()))
    }

    /// Consume the next token and wrap it as a [`Keyword`](crate::ast::Keyword).
    #[inline(always)]
    pub fn consume_keyword(&mut self) -> Result<crate::ast::Keyword<'arena>, ParseError> {
        let token = self.consume()?;
        let span = Span::new(self.file_id, token.start, token.end());
        Ok(crate::ast::Keyword { span, value: token.value })
    }

    /// Eat a token of `kind` and wrap it as a [`Keyword`](crate::ast::Keyword).
    #[inline(always)]
    pub fn eat_keyword(&mut self, kind: TypeTokenKind) -> Result<crate::ast::Keyword<'arena>, ParseError> {
        let token = self.eat(kind)?;
        let span = Span::new(self.file_id, token.start, token.end());
        Ok(crate::ast::Keyword { span, value: token.value })
    }

    /// Arena-allocate a value and return an `&'arena T` reference.
    #[inline(always)]
    #[must_use]
    pub fn alloc<T>(&self, value: T) -> &'arena T {
        self.arena.alloc(value)
    }

    /// A fresh arena-backed [`BVec`].
    #[inline(always)]
    #[must_use]
    pub fn new_bvec<T>(&self) -> BVec<'arena, T> {
        BVec::new_in(self.arena)
    }

    /// Returns the current position of the stream within the source file.
    ///
    /// This position represents the end location of the most recently
    /// consumed significant token via `advance()` or `consume()`.
    #[inline(always)]
    pub const fn current_position(&self) -> Position {
        self.position
    }

    /// Consumes and returns the next significant token.
    ///
    /// Advances the stream's position.
    ///
    /// # Returns
    ///
    /// - `Ok(TypeToken)`: The next significant token.
    /// - `Err(ParseError::UnexpectedEndOfFile)`: If EOF is reached.
    /// - `Err(ParseError::SyntaxError)`: If the underlying lexer returned an error.
    #[inline(always)]
    pub fn consume(&mut self) -> Result<TypeToken<'arena>, ParseError> {
        match self.advance() {
            Some(Ok(token)) => Ok(token),
            Some(Err(error)) => Err(error.into()),
            None => Err(self.unexpected(None, &[])),
        }
    }

    /// Consumes the next token *only if* it matches the expected `kind`.
    ///
    /// Advances the stream's position if the token matches.
    ///
    /// # Returns
    ///
    /// - `Ok(TypeToken)`: If the next token matches `kind`.
    /// - `Err(ParseError::UnexpectedToken)`: If the next token does *not* match `kind`.
    /// - `Err(ParseError::UnexpectedEndOfFile)`: If EOF is reached.
    /// - `Err(ParseError::SyntaxError)`: If the underlying lexer returned an error.
    #[inline(always)]
    pub fn eat(&mut self, kind: TypeTokenKind) -> Result<TypeToken<'arena>, ParseError> {
        let token_result = self.consume();

        match token_result {
            Ok(token) => {
                if kind == token.kind {
                    Ok(token)
                } else {
                    Err(self.unexpected(Some(token), &[kind]))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Advances the underlying lexer and returns the raw result (including trivia).
    /// Internal use or when trivia needs to be observed. `consume()` is preferred for parsers.
    /// Returns `None` on EOF, `Some(Err)` on lexer error, `Some(Ok)` on success.
    #[inline(always)]
    fn advance(&mut self) -> Option<Result<TypeToken<'arena>, SyntaxError>> {
        match self.fill_buffer(1) {
            Ok(true) => {
                if let Some(token) = self.buffer.pop_front() {
                    self.position = token.end();

                    Some(Ok(token))
                } else {
                    None
                }
            }
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        }
    }

    /// Returns the kind of the next significant token without consuming it.
    /// More efficient than `peek()` when only the kind is needed.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(TypeTokenKind))`: The kind of the next token.
    /// - `Ok(None)`: If EOF is reached.
    /// - `Err(ParseError)`: If the underlying lexer produced an error.
    #[inline(always)]
    pub fn peek_kind(&mut self) -> Result<Option<TypeTokenKind>, ParseError> {
        match self.fill_buffer(1) {
            Ok(true) => Ok(self.buffer.get(0).map(|t| t.kind)),
            Ok(false) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[inline(always)]
    pub fn is_at(&mut self, kind: TypeTokenKind) -> Result<bool, ParseError> {
        Ok(match self.peek_kind()? {
            Some(k) => k == kind,
            None => false,
        })
    }

    /// Peeks at the next significant token without consuming it.
    ///
    /// Requires `&mut self` as it might need to fill the buffer.
    ///
    /// # Returns
    ///
    /// - `Ok(TypeToken)`: A copy of the next significant token.
    /// - `Err(SyntaxError::UnexpectedEndOfFile)`: If EOF is reached.
    /// - `Err(ParseError)`: If the underlying lexer produced an error while peeking.
    #[inline(always)]
    pub fn peek(&mut self) -> Result<TypeToken<'arena>, ParseError> {
        match self.lookahead(0)? {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEndOfFile(self.file_id(), vec![], self.current_position())),
        }
    }

    /// Peeks at the nth (0-indexed) significant token ahead without consuming it.
    ///
    /// `lookahead(0)` is equivalent to `peek()`, but returns `Ok(None)` on EOF
    /// instead of an error. Requires `&mut self` as it might need to fill the buffer.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(TypeToken))`: If the nth token exists.
    /// - `Ok(None)`: If EOF is reached before the nth token.
    /// - `Err(ParseError)`: If the underlying lexer produced an error.
    #[inline(always)]
    pub fn lookahead(&mut self, n: usize) -> Result<Option<TypeToken<'arena>>, ParseError> {
        match self.fill_buffer(n + 1) {
            Ok(true) => Ok(self.buffer.get(n)),
            Ok(false) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Creates a `ParseError` for an unexpected token or EOF.
    /// Internal helper for `consume` and `eat`.
    #[inline(always)]
    fn unexpected(&self, found: Option<TypeToken<'arena>>, expected_one_of: &[TypeTokenKind]) -> ParseError {
        if let Some(token) = found {
            // Found a token, but it was the wrong kind
            ParseError::UnexpectedToken(expected_one_of.to_vec(), token.kind, token.span_for(self.file_id()))
        } else {
            // Reached EOF when expecting specific kinds
            ParseError::UnexpectedEndOfFile(self.file_id(), expected_one_of.to_vec(), self.current_position())
        }
    }

    /// Internal helper to ensure the lookahead buffer contains at least `n` items.
    /// Skips trivia tokens automatically. Returns `Ok(true)` on success,
    /// `Ok(false)` on EOF, `Err` on lexer error.
    #[inline(always)]
    fn fill_buffer(&mut self, n: usize) -> Result<bool, SyntaxError> {
        while self.buffer.len() < n {
            match self.lexer.advance() {
                Some(Ok(token)) => {
                    if token.kind.is_trivia() {
                        continue; // Skip trivia
                    }
                    self.buffer.push_back(token);
                }
                Some(Err(error)) => return Err(error),
                None => return Ok(false),
            }
        }
        Ok(true) // Buffer filled successfully
    }
}

impl HasFileId for TypeTokenStream<'_> {
    #[inline(always)]
    fn file_id(&self) -> FileId {
        self.file_id
    }
}
