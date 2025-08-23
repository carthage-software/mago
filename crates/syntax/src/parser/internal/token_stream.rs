use std::collections::VecDeque;
use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_database::file::HasFileId;
use mago_span::Position;

use crate::ast::sequence::Sequence;
use crate::ast::trivia::Trivia;
use crate::ast::trivia::TriviaKind;
use crate::error::SyntaxError;
use crate::lexer::Lexer;
use crate::token::Token;
use crate::token::TokenKind;

#[derive(Debug, Default)]
pub struct State {
    pub within_indirect_variable: bool,
}

#[derive(Debug)]
pub struct TokenStream<'input, 'arena> {
    arena: &'arena Bump,
    lexer: Lexer<'input, 'arena>,
    buffer: VecDeque<Token<'arena>>,
    trivia: Vec<'arena, Token<'arena>>,
    position: Position,
    pub(crate) state: State,
}

impl<'input, 'arena> TokenStream<'input, 'arena> {
    pub fn new(arena: &'arena Bump, lexer: Lexer<'input, 'arena>) -> TokenStream<'input, 'arena> {
        let position = lexer.get_position();

        TokenStream {
            arena,
            lexer,
            buffer: VecDeque::new(),
            trivia: Vec::new_in(arena),
            position,
            state: Default::default(),
        }
    }

    pub fn str(&self, string: &str) -> &'arena str {
        self.arena.alloc_str(string)
    }

    pub fn new_vec<T>(&self) -> Vec<'arena, T> {
        Vec::new_in(self.arena)
    }

    pub fn new_vec_of<T>(&self, value: T) -> Vec<'arena, T> {
        vec![in self.arena; value]
    }

    pub fn alloc<T>(&self, value: T) -> &'arena T {
        self.arena().alloc(value)
    }

    #[inline]
    pub const fn arena(&self) -> &'arena Bump {
        self.arena
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

    /// Return the current position of the stream in the input source code.
    #[inline]
    pub const fn get_position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn has_reached_eof(&mut self) -> Result<bool, SyntaxError> {
        Ok(self.fill_buffer(1)?.is_none())
    }

    /// Peeks at the next token in the input source code without consuming it.
    ///
    /// This method returns the next token that the lexer would produce if `advance` were called.
    ///
    /// If the lexer has already read the entire input source code, this method will return `None`.
    #[inline]
    pub fn peek(&mut self) -> Option<Result<Token<'arena>, SyntaxError>> {
        self.peek_nth(0)
    }

    /// Peeks at the `n`-th token in the input source code without consuming it.
    ///
    /// This method returns the `n`-th token that the lexer would produce if `advance` were called `n` times.
    ///
    /// If the lexer has already read the entire input source code, this method will return `None`.
    #[inline]
    pub fn peek_nth(&mut self, n: usize) -> Option<Result<Token<'arena>, SyntaxError>> {
        // Ensure the buffer has at least n+1 tokens.
        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => {
                // Return the nth token (0-indexed) if available.
                self.buffer.get(n).copied().map(Ok)
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
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
