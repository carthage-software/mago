use std::collections::VecDeque;
use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;

use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;

use crate::ast::sequence::Sequence;
use crate::ast::trivia::Trivia;
use crate::ast::trivia::TriviaKind;
use crate::error::ParseError;
use crate::error::SyntaxError;
use crate::lexer::Lexer;
use crate::token::Token;
use crate::token::TokenKind;

#[derive(Debug)]
pub struct TokenStream<'input, 'arena> {
    arena: &'arena Bump,
    lexer: Lexer<'input>,
    buffer: VecDeque<Token<'input>>,
    trivia: Vec<'arena, Token<'input>>,
    position: Position,
}

impl<'input, 'arena> TokenStream<'input, 'arena> {
    /// Initial capacity for the token lookahead buffer.
    const BUFFER_INITIAL_CAPACITY: usize = 8;

    pub fn new(arena: &'arena Bump, lexer: Lexer<'input>) -> TokenStream<'input, 'arena> {
        let position = lexer.current_position();

        TokenStream {
            arena,
            lexer,
            buffer: VecDeque::with_capacity(Self::BUFFER_INITIAL_CAPACITY),
            trivia: Vec::new_in(arena),
            position,
        }
    }

    /// Returns the current position of the stream within the source file.
    ///
    /// This position represents the end location of the most recently
    /// consumed significant token via `advance()` or `consume()`.
    #[inline]
    pub const fn current_position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn has_reached_eof(&mut self) -> Result<bool, SyntaxError> {
        Ok(self.fill_buffer(1)?.is_none())
    }

    /// Consumes and returns the next significant token.
    ///
    /// Returns an error if EOF is reached or a lexer error occurs.
    #[inline]
    pub fn consume(&mut self) -> Result<Token<'input>, ParseError> {
        match self.advance() {
            Some(Ok(token)) => Ok(token),
            Some(Err(error)) => Err(error.into()),
            None => Err(self.unexpected(None, &[])),
        }
    }

    /// Consumes the next token only if it matches the expected kind.
    ///
    /// Returns the token if it matches, otherwise returns an error.
    #[inline]
    pub fn eat(&mut self, kind: TokenKind) -> Result<Token<'input>, ParseError> {
        // Check kind first without copying full token
        let current_kind = self.peek_kind(0)?;
        match current_kind {
            Some(k) if k == kind => self.consume(),
            Some(_) => {
                let token = self.lookahead(0)?.unwrap();

                Err(self.unexpected(Some(token), &[kind]))
            }
            None => Err(self.unexpected(None, &[kind])),
        }
    }

    /// Consumes and returns the span of the next significant token.
    ///
    /// This is a convenience method equivalent to `consume()?.span_for(file_id())`.
    #[inline]
    pub fn consume_span(&mut self) -> Result<Span, ParseError> {
        let file_id = self.file_id();
        self.consume().map(|t| t.span_for(file_id))
    }

    /// Consumes the next token only if it matches the expected kind, returning its span.
    ///
    /// This is a convenience method equivalent to `eat(kind)?.span_for(file_id())`.
    #[inline]
    pub fn eat_span(&mut self, kind: TokenKind) -> Result<Span, ParseError> {
        let file_id = self.file_id();
        self.eat(kind).map(|t| t.span_for(file_id))
    }

    /// Advances the stream to the next token in the input source code and returns it.
    ///
    /// If the stream has already read the entire input source code, this method will return `None`.
    ///
    /// # Returns
    ///
    /// The next token in the input source code, or `None` if the lexer has reached the end of the input.
    #[inline]
    pub fn advance(&mut self) -> Option<Result<Token<'input>, SyntaxError>> {
        match self.fill_buffer(1) {
            Ok(Some(_)) => {
                if let Some(token) = self.buffer.pop_front() {
                    // Compute end position from start + value length
                    self.position = Position::new(token.start.offset + token.value.len() as u32);
                    Some(Ok(token))
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }

    /// Checks if the next token matches the given kind without consuming it.
    ///
    /// Returns `false` if at EOF.
    #[inline]
    pub fn is_at(&mut self, kind: TokenKind) -> Result<bool, ParseError> {
        Ok(self.peek_kind(0)? == Some(kind))
    }

    /// Peeks at the nth (0-indexed) significant token ahead without consuming it.
    ///
    /// Returns `Ok(None)` if EOF is reached before the nth token.
    #[inline]
    pub fn lookahead(&mut self, n: usize) -> Result<Option<Token<'input>>, ParseError> {
        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => Ok(self.buffer.get(n).copied()),
            Ok(None) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Peeks at the kind of the nth (0-indexed) significant token ahead.
    ///
    /// More efficient than `lookahead(n)?.map(|t| t.kind)` as it avoids
    /// copying the full token when only the kind is needed.
    #[inline]
    pub fn peek_kind(&mut self, n: usize) -> Result<Option<TokenKind>, ParseError> {
        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => Ok(self.buffer.get(n).map(|t| t.kind)),
            Ok(None) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Creates a `ParseError` for an unexpected token or EOF.
    #[inline]
    pub fn unexpected(&self, found: Option<Token<'_>>, expected: &[TokenKind]) -> ParseError {
        let expected_kinds: Box<[TokenKind]> = expected.into();
        if let Some(token) = found {
            ParseError::UnexpectedToken(expected_kinds, token.kind, token.span_for(self.file_id()))
        } else {
            ParseError::UnexpectedEndOfFile(expected_kinds, self.file_id(), self.current_position())
        }
    }

    /// Consumes the comments collected by the lexer and returns them.
    #[inline]
    pub fn get_trivia(&mut self) -> Sequence<'arena, Trivia<'arena>> {
        let mut tokens = Vec::new_in(self.arena);
        std::mem::swap(&mut self.trivia, &mut tokens);

        let file_id = self.file_id();
        Sequence::new(
            tokens
                .into_iter()
                .map(|token| {
                    let span = token.span_for(file_id);
                    match token.kind {
                        TokenKind::Whitespace => Trivia { kind: TriviaKind::WhiteSpace, span, value: token.value },
                        TokenKind::HashComment => Trivia { kind: TriviaKind::HashComment, span, value: token.value },
                        TokenKind::SingleLineComment => {
                            Trivia { kind: TriviaKind::SingleLineComment, span, value: token.value }
                        }
                        TokenKind::MultiLineComment => {
                            Trivia { kind: TriviaKind::MultiLineComment, span, value: token.value }
                        }
                        TokenKind::DocBlockComment => {
                            Trivia { kind: TriviaKind::DocBlockComment, span, value: token.value }
                        }
                        _ => unreachable!(),
                    }
                })
                .collect_in(self.arena),
        )
    }

    /// Fills the token buffer until at least `n` tokens are available, unless the lexer returns EOF.
    ///
    /// Trivia tokens are collected separately and are not stored in the main token buffer.
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
                Some(result) => match result {
                    Ok(token) => {
                        if token.kind.is_trivia() {
                            self.trivia.push(token);
                            continue;
                        }
                        self.buffer.push_back(token);
                    }
                    Err(error) => return Err(error),
                },
                None => return Ok(None),
            }
        }

        Ok(Some(n))
    }
}

impl HasFileId for TokenStream<'_, '_> {
    fn file_id(&self) -> mago_database::file::FileId {
        self.lexer.file_id()
    }
}
