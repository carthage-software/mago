use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::parser::LookaheadBuf;

use crate::cst::trivia::Trivia;
use crate::cst::trivia::TriviaKind;
use crate::error::ParseError;
use crate::lexer::DocblockLexer;
use crate::token::Token;
use crate::token::TokenKind;

#[derive(Debug, Clone, Copy)]
struct BufferedToken<'arena> {
    token: Token<'arena>,
    starts_line: bool,
}

#[derive(Debug)]
pub struct DocblockTokenStream<'arena> {
    arena: &'arena Bump,
    lexer: DocblockLexer<'arena>,
    buffer: LookaheadBuf<BufferedToken<'arena>, 64>,
    trivia: BVec<'arena, Trivia<'arena>>,
    source: &'arena [u8],
    base: u32,
    position: Position,
    file_id: FileId,
    pending_newline: bool,
    allow_line_prefix: bool,
    reached_end: bool,
}

#[inline]
fn contains_newline(bytes: &[u8]) -> bool {
    memchr::memchr2(b'\n', b'\r', bytes).is_some()
}

impl<'arena> DocblockTokenStream<'arena> {
    #[must_use]
    pub fn new(arena: &'arena Bump, lexer: DocblockLexer<'arena>, source: &'arena [u8], base: Position) -> Self {
        let file_id = lexer.file_id();

        DocblockTokenStream {
            arena,
            lexer,
            buffer: LookaheadBuf::new(),
            trivia: BVec::new_in(arena),
            source,
            base: base.offset,
            position: base,
            file_id,
            pending_newline: false,
            allow_line_prefix: false,
            reached_end: false,
        }
    }

    #[inline]
    pub fn take_trivia(&mut self) -> BVec<'arena, Trivia<'arena>> {
        std::mem::replace(&mut self.trivia, BVec::new_in(self.arena))
    }

    #[inline]
    #[must_use]
    pub const fn current_position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn has_reached_eof(&mut self) -> bool {
        self.fill_buffer(1).is_none()
    }

    #[inline]
    pub fn consume(&mut self) -> Result<Token<'arena>, ParseError> {
        match self.advance() {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEndOfInput(self.zero_span())),
        }
    }

    #[inline]
    pub fn eat(&mut self, kind: TokenKind) -> Result<Token<'arena>, ParseError> {
        match self.buffer.get(0) {
            Some(buffered) if buffered.token.kind == kind => {
                let _ = self.buffer.pop_front();
                self.position = Self::end_of(&buffered.token);

                Ok(buffered.token)
            }
            Some(buffered) => Err(ParseError::UnexpectedToken(self.span_of(&buffered.token))),
            None => match self.fill_buffer(1) {
                Some(()) => self.eat(kind),
                None => Err(ParseError::UnexpectedEndOfInput(self.zero_span())),
            },
        }
    }

    #[inline]
    pub fn consume_span(&mut self) -> Result<Span, ParseError> {
        let token = self.consume()?;

        Ok(self.span_of(&token))
    }

    #[inline]
    pub fn eat_span(&mut self, kind: TokenKind) -> Result<Span, ParseError> {
        let token = self.eat(kind)?;

        Ok(self.span_of(&token))
    }

    #[inline]
    pub fn is_at_value(&mut self, value: &[u8]) -> bool {
        self.lookahead(0).is_some_and(|token| token.kind == TokenKind::Identifier && token.value == value)
    }

    #[inline]
    pub fn peek(&mut self) -> Result<Token<'arena>, ParseError> {
        match self.lookahead(0) {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEndOfInput(self.zero_span())),
        }
    }

    #[inline]
    pub fn lookahead(&mut self, n: usize) -> Option<Token<'arena>> {
        if n < self.buffer.len() {
            return self.buffer.get(n).map(|buffered| buffered.token);
        }

        match self.fill_buffer(n + 1) {
            Some(()) => self.buffer.get(n).map(|buffered| buffered.token),
            None => None,
        }
    }

    #[inline]
    pub fn peek_kind(&mut self, n: usize) -> Option<TokenKind> {
        self.lookahead(n).map(|token| token.kind)
    }

    #[inline]
    pub fn is_at(&mut self, kind: TokenKind) -> bool {
        self.peek_kind(0) == Some(kind)
    }

    #[inline]
    pub fn starts_line(&mut self, n: usize) -> bool {
        if n >= self.buffer.len() {
            let _ = self.fill_buffer(n + 1);
        }

        self.buffer.get(n).is_some_and(|buffered| buffered.starts_line)
    }

    #[inline]
    #[must_use]
    pub fn raw_between(&self, from: Position, to: Position) -> &'arena [u8] {
        let start = from.offset.saturating_sub(self.base) as usize;
        let end = to.offset.saturating_sub(self.base) as usize;
        let length = self.source.len();
        let start = start.min(length);
        let end = end.min(length).max(start);

        &self.source[start..end]
    }

    #[inline]
    fn advance(&mut self) -> Option<Token<'arena>> {
        self.fill_buffer(1)?;
        let buffered = self.buffer.pop_front()?;
        self.position = Self::end_of(&buffered.token);

        Some(buffered.token)
    }

    #[inline]
    fn fill_buffer(&mut self, n: usize) -> Option<()> {
        if self.buffer.len() >= n {
            return Some(());
        }

        self.fill_buffer_slow(n)
    }

    #[inline(never)]
    fn fill_buffer_slow(&mut self, n: usize) -> Option<()> {
        while self.buffer.len() < n {
            if self.reached_end {
                return None;
            }

            let token = self.lexer.advance()?;

            match token.kind {
                TokenKind::Whitespace => {
                    self.push_trivia(TriviaKind::Whitespace, token);
                    if contains_newline(token.value) {
                        self.pending_newline = true;
                        self.allow_line_prefix = true;
                    }
                }
                TokenKind::LineComment => {
                    self.push_trivia(TriviaKind::LineComment, token);
                }
                TokenKind::Asterisk if self.allow_line_prefix => {
                    self.push_trivia(TriviaKind::Asterisk, token);
                    self.allow_line_prefix = false;
                }
                TokenKind::OpeningMarker => {
                    self.push_trivia(TriviaKind::OpeningMarker, token);
                    self.pending_newline = false;
                    self.allow_line_prefix = false;
                }
                TokenKind::ClosingMarker => {
                    self.push_trivia(TriviaKind::ClosingMarker, token);

                    let after = Position::new(token.start.offset + token.value.len() as u32);
                    let end = Position::new(self.base + self.source.len() as u32);
                    if after.offset < end.offset {
                        let value = self.raw_between(after, end);
                        self.trivia.push(Trivia {
                            kind: TriviaKind::Trailing,
                            span: Span::new(self.file_id, after, end),
                            value,
                        });
                    }

                    self.reached_end = true;
                }
                _ => {
                    let starts_line = self.pending_newline;
                    self.pending_newline = false;
                    self.allow_line_prefix = false;
                    self.buffer.push_back(BufferedToken { token, starts_line });
                }
            }
        }

        Some(())
    }

    #[inline]
    fn push_trivia(&mut self, kind: TriviaKind, token: Token<'arena>) {
        self.trivia.push(Trivia::from_token(kind, token, self.file_id));
    }

    #[inline]
    fn span_of(&self, token: &Token<'arena>) -> Span {
        token.span_for(self.file_id)
    }

    #[inline]
    fn end_of(token: &Token<'arena>) -> Position {
        Position::new(token.start.offset + token.value.len() as u32)
    }

    #[inline]
    fn zero_span(&self) -> Span {
        Span::new(self.file_id, self.position, self.position)
    }
}

impl HasFileId for DocblockTokenStream<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.file_id
    }
}
