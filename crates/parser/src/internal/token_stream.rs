use std::collections::VecDeque;
use std::fmt::Debug;

use bumpalo::boxed::Box;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::Bump;

use mago_ast::sequence::Sequence;
use mago_ast::trivia::Trivia;
use mago_ast::trivia::TriviaKind;
use mago_interner::ThreadedInterner;
use mago_lexer::error::SyntaxError;
use mago_lexer::Lexer;
use mago_span::Position;
use mago_token::Token;
use mago_token::TokenKind;

#[derive(Debug)]
pub struct TokenStream<'i, 'alloc> {
    interner: &'i ThreadedInterner,
    lexer: Lexer<'i, 'i>,
    bump: &'alloc Bump,
    buffer: VecDeque<Token>,
    trivia: Vec<'alloc, Token>,
    position: Position,
}

impl<'i, 'alloc> TokenStream<'i, 'alloc> {
    pub fn new(interner: &'i ThreadedInterner, bump: &'alloc Bump, lexer: Lexer<'i, 'i>) -> TokenStream<'i, 'alloc> {
        let position = lexer.get_position();

        TokenStream { interner, bump, lexer, buffer: VecDeque::new(), trivia: Vec::new_in(bump), position }
    }

    pub fn boxed<T>(&self, value: T) -> Box<'alloc, T> {
        Box::new_in(value, self.bump)
    }

    pub fn vec<T>(&self) -> Vec<'alloc, T> {
        Vec::new_in(self.bump)
    }

    pub fn interner(&self) -> &'i ThreadedInterner {
        self.interner
    }

    /// Advances the stream to the next token in the input source code and returns it.
    ///
    /// If the stream has already read the entire input source code, this method will return `None`.
    ///
    /// # Returns
    ///
    /// The next token in the input source code, or `None` if the lexer has reached the end of the input.
    #[inline]
    pub fn advance(&mut self) -> Option<Result<Token, SyntaxError>> {
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
    pub fn peek(&mut self) -> Option<Result<Token, SyntaxError>> {
        self.peek_nth(0)
    }

    /// Peeks at the `n`-th token in the input source code without consuming it.
    ///
    /// This method returns the `n`-th token that the lexer would produce if `advance` were called `n` times.
    ///
    /// If the lexer has already read the entire input source code, this method will return `None`.
    #[inline]
    pub fn peek_nth(&mut self, n: usize) -> Option<Result<Token, SyntaxError>> {
        // Ensure the buffer has at least n+1 tokens.
        match self.fill_buffer(n + 1) {
            Ok(Some(_)) => {
                // Return the nth token (0-indexed) if available.
                self.buffer.get(n).cloned().map(Ok)
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }

    /// Consumes the comments collected by the lexer and returns them.
    #[inline]
    pub fn get_trivia(&mut self) -> Sequence<'alloc, Trivia> {
        Sequence::new(
            self.trivia
                .drain(..)
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
                .collect_in(self.bump),
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
