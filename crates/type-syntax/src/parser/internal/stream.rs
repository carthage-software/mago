use std::collections::VecDeque;
use std::fmt::Debug;

use mago_span::Position;

use crate::error::ParseError;
use crate::error::SyntaxError;
use crate::lexer::TypeLexer;
use crate::token::TypeToken;
use crate::token::TypeTokenKind;

/// A buffered token stream that wraps a `TypeLexer`, providing lookahead
/// capabilities and automatically skipping trivia tokens (whitespace, comments).
#[derive(Debug)]
pub struct TypeTokenStream<'input> {
    lexer: TypeLexer<'input>,
    buffer: VecDeque<TypeToken<'input>>,
    position: Position,
}

impl<'input> TypeTokenStream<'input> {
    /// Creates a new `TypeTokenStream` wrapping the given `TypeLexer`.
    pub fn new(lexer: TypeLexer<'input>) -> TypeTokenStream<'input> {
        let position = lexer.current_position();

        TypeTokenStream { lexer, buffer: VecDeque::new(), position }
    }

    /// Returns the current position of the stream within the source file.
    ///
    /// This position represents the end location of the most recently
    /// consumed significant token via `advance()` or `consume()`.
    #[inline]
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
    #[inline]
    pub fn consume(&mut self) -> Result<TypeToken<'input>, ParseError> {
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
    #[inline]
    pub fn eat(&mut self, kind: TypeTokenKind) -> Result<TypeToken<'input>, ParseError> {
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
    #[inline]
    fn advance(&mut self) -> Option<Result<TypeToken<'input>, SyntaxError>> {
        match self.fill_buffer(1) {
            Ok(true) => {
                if let Some(token) = self.buffer.pop_front() {
                    self.position = token.span.end;

                    Some(Ok(token))
                } else {
                    None
                }
            }
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        }
    }

    #[inline]
    pub fn is_at(&mut self, kind: TypeTokenKind) -> Result<bool, ParseError> {
        Ok(match self.lookahead(0)? {
            Some(token) => token.kind == kind,
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
    #[inline]
    pub fn peek(&mut self) -> Result<TypeToken<'input>, ParseError> {
        match self.lookahead(0)? {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEndOfFile(vec![], self.current_position())),
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
    #[inline]
    pub fn lookahead(&mut self, n: usize) -> Result<Option<TypeToken<'input>>, ParseError> {
        // Ensure the buffer has at least n+1 tokens (or propagate EOF/error).
        match self.fill_buffer(n + 1) {
            Ok(true) => Ok(self.buffer.get(n).copied()),
            Ok(false) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Creates a `ParseError` for an unexpected token or EOF.
    /// Internal helper for `consume` and `eat`.
    #[inline]
    fn unexpected(&self, found: Option<TypeToken<'input>>, expected_one_of: &[TypeTokenKind]) -> ParseError {
        if let Some(token) = found {
            // Found a token, but it was the wrong kind
            ParseError::UnexpectedToken(expected_one_of.to_vec(), token.kind, token.span)
        } else {
            // Reached EOF when expecting specific kinds
            ParseError::UnexpectedEndOfFile(expected_one_of.to_vec(), self.current_position())
        }
    }

    /// Internal helper to ensure the lookahead buffer contains at least `n` items.
    /// Skips trivia tokens automatically. Returns `Ok(true)` on success,
    /// `Ok(false)` on EOF, `Err` on lexer error.
    #[inline]
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
