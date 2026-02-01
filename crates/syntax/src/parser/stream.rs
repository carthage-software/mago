use std::collections::VecDeque;
use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;

use mago_database::file::HasFileId;
use mago_span::Position;

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
    lexer: Lexer<'input, 'arena>,
    buffer: VecDeque<Token<'arena>>,
    trivia: Vec<'arena, Token<'arena>>,
    position: Position,
}

impl<'input, 'arena> TokenStream<'input, 'arena> {
    /// Initial capacity for the token lookahead buffer.
    const BUFFER_INITIAL_CAPACITY: usize = 8;

    pub fn new(arena: &'arena Bump, lexer: Lexer<'input, 'arena>) -> TokenStream<'input, 'arena> {
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
    pub fn consume(&mut self) -> Result<Token<'arena>, ParseError> {
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
    pub fn eat(&mut self, kind: TokenKind) -> Result<Token<'arena>, ParseError> {
        let current = self.lookahead(0)?;
        if let Some(token) = current {
            if token.kind != kind {
                return Err(self.unexpected(Some(token), &[kind]));
            }

            let token = self.consume()?;

            Ok(token)
        } else {
            Err(self.unexpected(None, &[kind]))
        }
    }

    /// Advances the stream to the next token in the input source code and returns it.
    ///
    /// If the stream has already read the entire input source code, this method will return `None`.
    ///
    /// # Returns
    ///
    /// The next token in the input source code, or `None` if the lexer has reached the end of the input.
    #[inline]
    pub fn advance(&mut self) -> Option<Result<Token<'arena>, SyntaxError>> {
        match self.fill_buffer(1) {
            Ok(Some(_)) => {
                if let Some(token) = self.buffer.pop_front() {
                    self.position = token.span.end;
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
        match self.lookahead(0)? {
            Some(token) => Ok(token.kind == kind),
            None => Ok(false),
        }
    }

    /// Peeks at the nth (0-indexed) significant token ahead without consuming it.
    ///
    /// Returns `Ok(None)` if EOF is reached before the nth token.
    #[inline(always)]
    pub fn lookahead(&mut self, n: usize) -> Result<Option<Token<'arena>>, ParseError> {
        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => Ok(self.buffer.get(n).copied()),
            Ok(None) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Creates a `ParseError` for an unexpected token or EOF.
    #[inline]
    pub fn unexpected(&self, found: Option<Token<'_>>, expected: &[TokenKind]) -> ParseError {
        let expected_kinds: Box<[TokenKind]> = expected.into();
        if let Some(token) = found {
            ParseError::UnexpectedToken(expected_kinds, token.kind, token.span)
        } else {
            ParseError::UnexpectedEndOfFile(expected_kinds, self.file_id(), self.current_position())
        }
    }

    /// Consumes the comments collected by the lexer and returns them.
    #[inline]
    pub fn get_trivia(&mut self) -> Sequence<'arena, Trivia<'arena>> {
        let mut tokens = Vec::new_in(self.arena);
        std::mem::swap(&mut self.trivia, &mut tokens);

        Sequence::new(
            tokens
                .into_iter()
                .map(|token| match token.kind {
                    TokenKind::Whitespace => {
                        Trivia { kind: TriviaKind::WhiteSpace, span: token.span, value: token.value }
                    }
                    TokenKind::HashComment => {
                        Trivia { kind: TriviaKind::HashComment, span: token.span, value: token.value }
                    }
                    TokenKind::SingleLineComment => {
                        Trivia { kind: TriviaKind::SingleLineComment, span: token.span, value: token.value }
                    }
                    TokenKind::MultiLineComment => {
                        Trivia { kind: TriviaKind::MultiLineComment, span: token.span, value: token.value }
                    }
                    TokenKind::DocBlockComment => {
                        Trivia { kind: TriviaKind::DocBlockComment, span: token.span, value: token.value }
                    }
                    _ => unreachable!(),
                })
                .collect_in(self.arena),
        )
    }

    /// Fills the token buffer until at least `n` tokens are available, unless the lexer returns EOF.
    ///
    /// Trivia tokens are collected separately and are not stored in the main token buffer.
    #[inline]
    fn fill_buffer(&mut self, n: usize) -> Result<Option<usize>, SyntaxError> {
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
